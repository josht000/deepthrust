//! S001 RSI(2) Classic (Connors) study: daily-book adapter + staged pipeline
//! (port of s001_rsi2_connors/strategy.py, framework/pipeline.py and framework/daily_study.py).

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use crate::config::Config;
use crate::dashboard::{self, ChartInput, FillIn, TradeIn};
use crate::data::{self, Panel};
use crate::engine::{self, SwingParams, Trade};
use crate::partition::{self, Book, FillResult, Masks, Row};
use crate::report::{fnum, section, table};
use crate::signals::{self, Signals};
use crate::stats;
use crate::timing;

pub const NAME: &str = "s001_rsi2_connors";
pub const TITLE: &str = "S001 RSI(2) Classic (Connors)";
pub const ORDER: [&str; 5] = ["trades", "table", "rank", "partition", "charts"];
const MIN_DEV_TRADES: f64 = 100.0;
const MIN_TEST_TRADES: f64 = 30.0;

fn dependencies(stage: &str) -> &'static [&'static str] {
    match stage {
        "trades" => &[],
        "table" => &["trades"],
        "rank" => &["table"],
        "charts" => &["table", "rank", "partition"],
        _ => &["table", "rank"],
    }
}

/// Requested stages plus their dependencies, in pipeline order.
pub fn resolve(requested: &[String]) -> Result<Vec<&'static str>, String> {
    let mut selected = BTreeSet::new();
    fn add(name: &'static str, sel: &mut BTreeSet<&'static str>) {
        for d in dependencies(name) {
            add(d, sel);
        }
        sel.insert(name);
    }
    for r in requested {
        let name = ORDER
            .iter()
            .find(|s| **s == r)
            .ok_or_else(|| format!("unknown stage {r:?}; choose from {}", ORDER.join(", ")))?;
        add(name, &mut selected);
    }
    Ok(ORDER
        .iter()
        .copied()
        .filter(|s| selected.contains(s))
        .collect())
}

pub struct Flags {
    pub test: bool,
    pub rebuild: bool,
    pub profile: bool,
}

pub struct State {
    pub panel: Panel,
    pub sig: Signals,
    pub b_day: Vec<f64>,
    pub close_ff: Vec<Vec<f64>>,
    pub masks: Masks,
    pub trades: Vec<(String, Vec<Trade>)>,
    pub first_row: usize,
    pub split_row: usize,
    pub benchmark: Option<(String, Vec<f64>)>,
}

pub struct Study<'a> {
    pub cfg: &'a Config,
    pub results: PathBuf,
    pub flags: &'a Flags,
    state: Option<State>,
    evaluation: Option<Evaluation>,
}

struct Evaluation {
    fills: Vec<(String, FillResult)>,
    flags: Vec<String>,
    cost_ladder: Vec<(f64, f64, String, f64, f64)>,
    baselines: Vec<(String, String, f64, f64, f64, f64)>,
}

fn csv_cell(v: f64) -> String {
    if v.is_nan() {
        String::new()
    } else {
        format!("{v}")
    }
}

impl<'a> Study<'a> {
    pub fn new(cfg: &'a Config, results_root: &Path, flags: &'a Flags) -> Self {
        Study {
            cfg,
            results: results_root.join(NAME),
            flags,
            state: None,
            evaluation: None,
        }
    }

    fn out(&self) -> PathBuf {
        let _ = fs::create_dir_all(&self.results);
        self.results.clone()
    }

    fn write(&self, name: &str, text: String) -> Result<(), String> {
        let path = self.out().join(format!("{}{}", self.cfg.prefix(), name));
        fs::write(&path, text).map_err(|e| format!("{}: {e}", path.display()))
    }

    /// Load bars for the universe and benchmark from configured Parquet inputs.
    pub fn load_all(&self) -> Result<(Panel, Option<(String, data::Bars)>), String> {
        let cfg = self.cfg;
        let start = data::parse_date(&cfg.start_date)?;
        // the Python panel loader orders symbols alphabetically
        let mut symbols = cfg.symbol_list();
        symbols.sort();
        symbols.dedup();
        let mut bars = vec![];
        for s in &symbols {
            let b = data::load_symbol(&cfg.data_dir, &cfg.data_suffix, s, start)?;
            if let Some(w) = data::start_gap_warning(s, &b, start) {
                eprintln!("Warning: {w}");
            }
            bars.push(b);
        }
        let panel = {
            let _g = timing::stage("build panel");
            data::build_panel(&symbols, &bars)
        };
        let bench = if cfg.benchmark.is_empty() || symbols.contains(&cfg.benchmark) {
            None
        } else {
            match data::load_symbol(&cfg.data_dir, &cfg.data_suffix, &cfg.benchmark, start) {
                Ok(b) => {
                    if let Some(w) = data::start_gap_warning(&cfg.benchmark, &b, start) {
                        eprintln!("Warning: {w}");
                    }
                    Some((cfg.benchmark.clone(), b))
                }
                Err(e) => {
                    eprintln!("Benchmark warning: {e}; falling back to the equal-weight universe");
                    None
                }
            }
        };
        Ok((panel, bench))
    }

    fn build_state(&mut self) -> Result<(), String> {
        if self.state.is_some() {
            return Ok(());
        }
        let cfg = self.cfg;
        let (panel, bench) = {
            let _g = timing::stage("load panel");
            self.load_all()?
        };
        if panel.dates.is_empty() {
            return Err("empty panel: no daily bars loaded".into());
        }
        let sig = {
            let _g = timing::stage("signals");
            signals::compute_signals(&panel, cfg)
        };
        let n_rows = panel.dates.len();
        let split = data::parse_date(&cfg.partition_split_date)?;
        let split_row = panel.dates.partition_point(|d| *d < split);
        let first = (0..n_rows)
            .find(|&t| sig.tradable.iter().any(|c| c[t]))
            .unwrap_or(0);
        let masks: Masks = vec![
            (
                "dev".into(),
                (0..n_rows).map(|t| t >= first && t < split_row).collect(),
            ),
            ("test".into(), (0..n_rows).map(|t| t >= split_row).collect()),
        ];
        let mut b_day = vec![0.0; n_rows];
        for t in 1..n_rows {
            let (mut s, mut n) = (0.0, 0usize);
            for j in 0..panel.symbols.len() {
                let r = panel.close[j][t] / panel.close[j][t - 1] - 1.0;
                if sig.tradable[j][t] && r.is_finite() {
                    s += r;
                    n += 1;
                }
            }
            b_day[t] = s / n.max(1) as f64;
        }
        let trades = {
            let _g = timing::stage("trades");
            let params = SwingParams {
                signal: &sig.signal,
                exit_now: &sig.exit_now,
                add_signal: sig.add_signal.as_ref(),
                max_units: cfg.max_units,
                stop_atr: sig.atr.as_ref(),
                stop_mult: cfg.disaster_stop_atr,
                rsi: &sig.rsi,
            };
            cfg.fills_list()
                .into_iter()
                .map(|f| {
                    let _g = timing::stage(&format!("swing trades {f}"));
                    let t = engine::swing_trades(&panel, &params, cfg, &f);
                    (f, t)
                })
                .collect::<Vec<_>>()
        };
        let close_ff = panel
            .close
            .iter()
            .map(|c| {
                let mut last = f64::NAN;
                c.iter()
                    .map(|v| {
                        if v.is_finite() {
                            last = *v;
                        }
                        last
                    })
                    .collect()
            })
            .collect();
        // benchmark daily returns aligned to the panel grid
        let benchmark = bench.map(|(name, b)| {
            let mut ret = vec![0.0; n_rows];
            for i in 1..b.dates.len() {
                if let Ok(r) = panel.dates.binary_search(&b.dates[i]) {
                    ret[r] = b.close[i] / b.close[i - 1] - 1.0;
                }
            }
            (name, ret)
        });
        self.state = Some(State {
            panel,
            sig,
            b_day,
            close_ff,
            masks,
            trades,
            first_row: first,
            split_row,
            benchmark,
        });
        Ok(())
    }

    fn state(&self) -> &State {
        self.state
            .as_ref()
            .expect("state built by the trades stage")
    }

    fn book(&self, fill: &str) -> Book {
        let s = self.state();
        let mat = if fill == "next_open" {
            s.panel.open.clone()
        } else {
            s.panel.close.clone()
        };
        Book {
            close_ff: s.close_ff.clone(),
            entry_mat: mat.clone(),
            exit_mat: mat,
            cost: self.cfg.cost(),
            n_rows: s.panel.dates.len(),
            b_day: s.b_day.clone(),
            slots: self.cfg.book_slots,
        }
    }

    pub fn run_stage(&mut self, name: &str) -> Result<(), String> {
        match name {
            "trades" => self.stage_trades(),
            "table" => self.build_state(),
            "rank" => self.stage_rank(),
            "partition" => self.stage_partition(),
            "charts" => self.stage_charts(),
            other => Err(format!("unknown stage {other}")),
        }
    }

    fn stage_trades(&mut self) -> Result<(), String> {
        self.build_state()?;
        let s = self.state();
        let n_rows = s.panel.dates.len();
        println!(
            "Panel: {} symbols x {} sessions ({} .. {}); evaluation starts {}; split {}",
            s.panel.symbols.len(),
            n_rows,
            s.panel.dates[0],
            s.panel.dates[n_rows - 1],
            s.panel.dates[s.first_row],
            self.cfg.partition_split_date
        );
        let _g = timing::stage("write trades csv");
        for (fill, trades) in &s.trades {
            let mut out = String::from("symbol,signal_date,entry_date,exit_date,entry_px,exit_px,ret,ret_gross,exit_reason,hold,rsi2,fill\n");
            for t in trades {
                out += &format!(
                    "{},{},{},{},{},{},{},{},{},{},{},{}\n",
                    s.panel.symbols[t.sym_idx],
                    s.panel.dates[t.signal_idx],
                    s.panel.dates[t.entry_idx],
                    s.panel.dates[t.exit_idx],
                    t.entry_px,
                    t.exit_px,
                    t.ret,
                    t.ret_gross,
                    t.exit_reason,
                    t.hold,
                    csv_cell(t.rsi2),
                    fill
                );
            }
            self.write(&format!("trades_{fill}.csv"), out)?;
        }
        Ok(())
    }

    fn period_of(&self, entry_idx: usize) -> &'static str {
        if entry_idx >= self.state().split_row {
            "test"
        } else {
            "dev"
        }
    }

    fn period_stats(trades: &[&Trade]) -> [f64; 5] {
        let r: Vec<f64> = trades.iter().map(|t| t.ret).collect();
        let wins: f64 = r.iter().filter(|v| **v > 0.0).sum();
        let losses: f64 = -r.iter().filter(|v| **v <= 0.0).sum::<f64>();
        let n = r.len() as f64;
        [
            n,
            if r.is_empty() {
                f64::NAN
            } else {
                100.0 * r.iter().filter(|v| **v > 0.0).count() as f64 / n
            },
            100.0 * stats::mean(&r),
            if losses > 0.0 {
                wins / losses
            } else {
                f64::NAN
            },
            stats::mean(&trades.iter().map(|t| t.hold as f64).collect::<Vec<_>>()),
        ]
    }

    fn stage_rank(&mut self) -> Result<(), String> {
        let s = self.state();
        let mut csv = String::from("fill,period,trades,win_pct,mean_ret_pct,pf,avg_hold\n");
        let mut rows = vec![];
        for (fill, trades) in &s.trades {
            for period in ["dev", "test"] {
                let sel: Vec<&Trade> = trades
                    .iter()
                    .filter(|t| self.period_of(t.entry_idx) == period)
                    .collect();
                let st = Self::period_stats(&sel);
                csv += &format!(
                    "{fill},{period},{},{},{},{},{}\n",
                    st[0],
                    csv_cell(st[1]),
                    csv_cell(st[2]),
                    csv_cell(st[3]),
                    csv_cell(st[4])
                );
                rows.push(vec![
                    fill.clone(),
                    period.to_string(),
                    format!("{}", st[0] as i64),
                    fnum(st[1], 3),
                    fnum(st[2], 3),
                    fnum(st[3], 3),
                    fnum(st[4], 3),
                ]);
            }
        }
        self.write("trade_summary.csv", csv)?;
        section(&format!(
            "{NAME} trade summary (net of costs, research scale)"
        ));
        table(
            &[
                "fill",
                "period",
                "trades",
                "win_pct",
                "mean_ret_pct",
                "pf",
                "avg_hold",
            ],
            &rows,
        );
        // per-symbol trade counts (per_etf.csv in the Python version)
        let mut per = String::from("fill,period,symbol,trades,win_pct,mean_ret_pct,pf,avg_hold\n");
        let mut prow = vec![];
        // pandas groupby order: fill, period, symbol; empty groups are absent
        let mut sorted_trades: Vec<_> = s.trades.iter().collect();
        sorted_trades.sort_by(|a, b| a.0.cmp(&b.0));
        for (fill, trades) in sorted_trades {
            for period in ["dev", "test"] {
                let mut syms: Vec<(usize, &String)> = s.panel.symbols.iter().enumerate().collect();
                syms.sort_by(|a, b| a.1.cmp(b.1));
                for (j, sym) in syms {
                    let sel: Vec<&Trade> = trades
                        .iter()
                        .filter(|t| t.sym_idx == j && self.period_of(t.entry_idx) == period)
                        .collect();
                    if sel.is_empty() {
                        continue;
                    }
                    let st = Self::period_stats(&sel);
                    per += &format!(
                        "{fill},{period},{sym},{},{},{},{},{}\n",
                        st[0],
                        csv_cell(st[1]),
                        csv_cell(st[2]),
                        csv_cell(st[3]),
                        csv_cell(st[4])
                    );
                    prow.push(vec![
                        sym.clone(),
                        fill.clone(),
                        period.to_string(),
                        format!("{}", st[0] as i64),
                    ]);
                }
            }
        }
        self.write("per_etf.csv", per)?;
        table(&["symbol", "fill", "period", "trades"], &prow);
        Ok(())
    }

    fn flags_of(&self, fills: &[(String, FillResult)], primary: &str) -> Vec<String> {
        let mut out = vec![];
        let Some((_, p)) = fills.iter().find(|(f, _)| f == primary) else {
            return out;
        };
        let get = |period: &str| {
            p.by_period
                .iter()
                .find(|(n, _, _)| n == period)
                .map(|(_, r, _)| r.clone())
        };
        let (dev, test) = (get("dev"), get("test"));
        if let Some(d) = &dev {
            if d.get("trades") < MIN_DEV_TRADES {
                out.push(format!(
                    "insufficient evidence: {} dev trades < {}",
                    d.get("trades") as i64,
                    MIN_DEV_TRADES as i64
                ));
            }
        }
        if let Some(t) = &test {
            if t.get("trades") < MIN_TEST_TRADES {
                out.push(format!(
                    "insufficient evidence: {} test trades < {}",
                    t.get("trades") as i64,
                    MIN_TEST_TRADES as i64
                ));
            }
        }
        if let (Some(d), Some(t)) = (&dev, &test) {
            let (ds, ts) = (d.get("skill"), t.get("skill"));
            if ds.is_finite() && ts.is_finite() && ds != 0.0 && ds.signum() != ts.signum() {
                out.push("DECAYED: test skill sign differs from dev".into());
            }
        }
        out
    }

    fn stage_partition(&mut self) -> Result<(), String> {
        let cfg = self.cfg;
        let fills = cfg.fills_list();
        let primary = fills.first().cloned().ok_or("no fills configured")?;
        let s = self.state();
        let n_rows = s.panel.dates.len();
        let books: Vec<(String, Book)> = fills.iter().map(|f| (f.clone(), self.book(f))).collect();
        let trades_of = |f: &str| &s.trades.iter().find(|(n, _)| n == f).unwrap().1;
        let dev_mask = &s.masks[0].1;

        // fill-model correlation -> effective number of comparable trials
        let logs: Vec<Vec<f64>> = books
            .iter()
            .map(|(f, b)| {
                let (r, _) = partition::account_series(
                    &partition::legs_of(trades_of(f)),
                    &b.close_ff,
                    b.cost,
                    b.n_rows,
                    b.slots,
                );
                (0..n_rows)
                    .filter(|&t| dev_mask[t])
                    .map(|t| r[t].ln_1p())
                    .collect()
            })
            .collect();
        let rho = if logs.len() > 1 {
            stats::corr(&logs[0], &logs[1])
        } else {
            0.0
        };
        let rho = if rho.is_finite() { rho } else { 0.0 };
        let n_eff = (1.0 + (fills.len() as f64 - 1.0) * (1.0 - rho)).max(1.0);
        let bias_note = format!("{} fill models, daily corr {rho:.2}", fills.len());

        let mut results = vec![];
        for (f, b) in &books {
            let r = partition::evaluate_fill(
                trades_of(f),
                b,
                &s.masks,
                &s.sig.tradable,
                &s.sig.pool,
                n_eff,
                cfg.n_draws,
                cfg.n_boot,
                cfg.seed,
                &bias_note,
            );
            results.push((f.clone(), r));
        }

        // partition CSVs
        let all_keys: Vec<&'static str> = results[0].1.by_period[0]
            .1
             .0
            .iter()
            .map(|(k, _)| *k)
            .collect();
        for period in ["dev", "test"] {
            let mut csv = format!("fill,period,{}\n", all_keys.join(","));
            for (f, r) in &results {
                if let Some((_, row, _)) = r.by_period.iter().find(|(n, _, _)| n == period) {
                    csv += &format!(
                        "{f},{period},{}\n",
                        all_keys
                            .iter()
                            .map(|k| csv_cell(row.get(k)))
                            .collect::<Vec<_>>()
                            .join(",")
                    );
                }
            }
            self.write(&format!("partition_{period}.csv"), csv)?;
        }

        let ladder = {
            let _g = timing::stage("cost ladder");
            let pb = &books[0].1;
            partition::cost_ladder(trades_of(&primary), pb, &s.masks, &[0.5, 1.0, 2.0, 3.0])
        };
        let mut csv = String::from("fill,cost_mult,cost_bps_side,period,total_log,sharpe\n");
        for (m, bps, p, tot, sh) in &ladder {
            csv += &format!("{primary},{m},{bps},{p},{tot},{}\n", csv_cell(*sh));
        }
        self.write("cost_ladder.csv", csv)?;

        // baselines
        let strat_r = &results[0].1.r_day;
        let n = s.b_day.len();
        let mut idx = vec![0.0; n];
        let mut acc = 1.0;
        for t in 0..n {
            acc *= 1.0 + s.b_day[t];
            idx[t] = acc;
        }
        let sma = signals::rolling_mean(&idx, cfg.sma_window);
        let timing_r: Vec<f64> = (0..n)
            .map(|t| {
                if t > 0 && idx[t - 1] > sma[t - 1] {
                    s.b_day[t]
                } else {
                    0.0
                }
            })
            .collect();
        let mut base = vec![];
        for (label, arr) in [
            ("ew_buy_hold", s.b_day.clone()),
            ("sma200_timing", timing_r),
            (
                "vol_matched_ew",
                partition::vol_matched(&s.b_day, strat_r, dev_mask),
            ),
        ] {
            for (period, total, sharpe, cagr, dd) in partition::baseline_stats(&arr, &s.masks) {
                base.push((label.to_string(), period, total, sharpe, cagr, dd));
            }
        }
        {
            let _g = timing::stage("random timing");
            let pb = &books[0].1;
            let last = engine::last_valid_rows(&s.panel.close);
            let totals = partition::random_timing(
                trades_of(&primary),
                pb,
                &s.masks,
                s.first_row,
                &last,
                cfg.n_timing_draws,
                cfg.seed,
            );
            for (m, (period, _)) in s.masks.iter().enumerate() {
                let tot_m: Vec<f64> = totals.iter().map(|r| r[m]).collect();
                let strat_total = results[0]
                    .1
                    .by_period
                    .iter()
                    .find(|(n, _, _)| n == period)
                    .map(|(_, r, _)| r.get("total_log"))
                    .unwrap_or(f64::NAN);
                let p = (tot_m.iter().filter(|&&v| v >= strat_total).count() as f64 + 1.0)
                    / (tot_m.len() as f64 + 1.0);
                base.push((
                    format!("random_timing (p_total_ge_strategy={})", fnum(p, 3)),
                    period.clone(),
                    stats::mean(&tot_m),
                    f64::NAN,
                    f64::NAN,
                    f64::NAN,
                ));
            }
        }
        let mut csv = String::from(
            "baseline,period,total_log,sharpe,cagr_pct,max_dd_pct,p_total_ge_strategy\n",
        );
        for (b, p, t, sh, c, dd) in &base {
            let pval = b
                .split_once("p_total_ge_strategy=")
                .map(|(_, v)| v.trim_end_matches(')').to_string())
                .unwrap_or_default();
            csv += &format!(
                "{},{p},{},{},{},{},{pval}\n",
                b.split(' ').next().unwrap(),
                csv_cell(*t),
                csv_cell(*sh),
                csv_cell(*c),
                csv_cell(*dd)
            );
        }
        self.write("baselines.csv", csv)?;

        // daily returns
        let mut csv = String::from("date,b_day");
        for (f, _) in &results {
            csv += &format!(",{f}_r_day,{f}_positions");
        }
        csv.push('\n');
        for t in s.first_row..n_rows {
            csv += &format!("{},{}", s.panel.dates[t], s.b_day[t]);
            for (_, r) in &results {
                csv += &format!(",{},{}", r.r_day[t], r.count[t]);
            }
            csv.push('\n');
        }
        self.write("daily_returns.csv", csv)?;

        let flags = self.flags_of(&results, &primary);
        for f in &flags {
            section(&format!("{NAME}: {f}"));
        }
        section(&format!(
            "{NAME} partition test (log returns, research scale, net of costs)"
        ));
        let mut rows = vec![];
        for (f, r) in &results {
            for (period, row, _) in &r.by_period {
                let mut cells = vec![
                    f.clone(),
                    period.clone(),
                    format!("{}", row.get("trades") as i64),
                ];
                for k in [
                    "total_log",
                    "trend",
                    "random_pick_universe",
                    "random_pick_pool",
                    "bias",
                    "skill",
                    "skill_pct",
                    "skill_p",
                    "sharpe",
                    "pf",
                    "win_pct",
                ] {
                    cells.push(fnum(row.get(k), 3));
                }
                rows.push(cells);
            }
        }
        table(
            &[
                "fill", "per", "n", "total", "trend", "rnd_univ", "rnd_pool", "bias", "skill",
                "skill%", "p", "sharpe", "pf", "win_pct",
            ],
            &rows,
        );

        self.write_report(&results, &flags, &ladder, &base)?;
        self.evaluation = Some(Evaluation {
            fills: results,
            flags,
            cost_ladder: ladder,
            baselines: base,
        });
        Ok(())
    }

    fn write_report(
        &self,
        results: &[(String, FillResult)],
        flags: &[String],
        ladder: &[(f64, f64, String, f64, f64)],
        base: &[(String, String, f64, f64, f64, f64)],
    ) -> Result<(), String> {
        let (cfg, s) = (self.cfg, self.state());
        let dates = &s.panel.dates;
        let mut md = vec![format!("# {TITLE}: partition report"), String::new()];
        if cfg.variant != "baseline" {
            md.push(format!(
                "- Variant: `{}` overlay {{{}}}",
                cfg.variant, cfg.variant_overlay
            ));
            md.push(String::new());
        }
        md.push(format!(
            "- Data: `{}`; sessions {} .. {}; evaluation starts {}.",
            cfg.data_dir,
            dates[0],
            dates[dates.len() - 1],
            dates[s.first_row]
        ));
        md.push(format!("- Split: dev < {} <= test (locked; parameters are the catalog baseline or registered variants).", cfg.partition_split_date));
        md.push("- Trend = exposure-weighted equal-weight tradable-universe return; skill = total - trend - bias; units are log returns.".into());
        md.extend(self.intro());
        md.push(String::new());
        for f in flags {
            md.push(format!("**{f}**"));
            md.push(String::new());
        }
        let cols = [
            "trades",
            "days",
            "total_log",
            "trend",
            "trend_beta",
            "beta",
            "random_pick_universe",
            "random_pick_pool",
            "random_p_universe",
            "random_p_pool",
            "bias",
            "skill",
            "skill_vs_universe",
            "skill_vs_pool",
            "skill_pct",
            "skill_lo",
            "skill_hi",
            "skill_p",
            "skill_ex5",
            "win_pct",
            "mean_ret_pct",
            "pf",
            "cagr_pct",
            "sharpe",
            "max_dd_pct",
            "avg_hold",
            "avg_positions",
            "avg_fraction_invested",
            "turnover_annual",
        ];
        let md_num = |v: f64| {
            if v.is_finite() {
                format!("{v:.4}")
            } else {
                "n/a".to_string()
            }
        };
        for period in ["dev", "test"] {
            md.push(format!("## {period}"));
            md.push(String::new());
            md.push(format!("| fill | {} |", cols.join(" | ")));
            md.push(format!("|{}", "---|".repeat(cols.len() + 1)));
            for (f, r) in results {
                if let Some((_, row, _)) = r.by_period.iter().find(|(n, _, _)| n == period) {
                    md.push(format!(
                        "| {f} | {} |",
                        cols.iter()
                            .map(|c| md_num(row.get(c)))
                            .collect::<Vec<_>>()
                            .join(" | ")
                    ));
                }
            }
            md.push(String::new());
            for (f, r) in results {
                if let Some((_, _, src)) = r.by_period.iter().find(|(n, _, _)| n == period) {
                    md.push(format!("Bias source ({f}): {src}"));
                }
            }
            md.push(String::new());
        }
        md.push("## Cost ladder (primary fill)".into());
        md.push(String::new());
        md.push("| fill | cost_mult | cost_bps_side | period | total_log | sharpe |".into());
        md.push("|---|---|---|---|---|---|".into());
        for (m, bps, p, t, sh) in ladder {
            md.push(format!(
                "| {} | {m} | {bps} | {p} | {} | {} |",
                cfg.fills_list()[0],
                md_num(*t),
                md_num(*sh)
            ));
        }
        md.push(String::new());
        md.push("## Baselines".into());
        md.push(String::new());
        md.push("| baseline | period | total_log | sharpe | cagr_pct | max_dd_pct |".into());
        md.push("|---|---|---|---|---|---|".into());
        for (b, p, t, sh, c, dd) in base {
            md.push(format!(
                "| {b} | {p} | {} | {} | {} | {} |",
                md_num(*t),
                md_num(*sh),
                md_num(*c),
                md_num(*dd)
            ));
        }
        md.push(String::new());
        self.write("report.md", md.join("\n"))
    }

    fn intro(&self) -> Vec<String> {
        let c = self.cfg;
        vec![
            format!("- Universe: {} (split-adjusted daily bars `{}`); each symbol at most one position, book slots {}.", c.symbols, c.data_suffix, c.book_slots),
            format!("- Rule: close > gate ({}) AND RSI({}) < {}; exit {}; sizing {}; disaster stop x{} ATR({}).", c.gate, c.rsi_len, c.rsi_threshold, c.exit_mode, c.sizing, c.disaster_stop_atr, c.atr_len),
            "- `next_open` is the primary fill (signal on session t, fill at open t+1); `close` is the same-close UPPER BOUND. `moc_proxy` needs 15:50 minute bars, which are not fetched.".into(),
            "- Baselines: EW buy-and-hold of the tradable universe, SMA200 timing on that EW index (signal lagged one session), dev-vol-matched EW, and exposure-matched random timing.".into(),
            "- Limitations: moc_proxy fill unavailable; a single-symbol universe cannot reach the dev/test trade-count gates.".into(),
        ]
    }

    pub fn caption(&self) -> String {
        let (c, s) = (self.cfg, self.state());
        format!("variant={} | RSI({})<{} gate={} exit={} sizing={} stop={}xATR | {}bps/side slots={} | {} {}..{}",
            c.variant, c.rsi_len, c.rsi_threshold, c.gate, c.exit_mode, c.sizing, c.disaster_stop_atr, c.cost_bps_per_side, c.book_slots, c.data_dir,
            s.panel.dates[0], s.panel.dates[s.panel.dates.len() - 1])
    }

    fn stage_charts(&mut self) -> Result<(), String> {
        let s = self.state();
        let ev = self
            .evaluation
            .as_ref()
            .ok_or("charts need the partition stage")?;
        let mut fills = vec![];
        for (name, r) in &ev.fills {
            let trades = s
                .trades
                .iter()
                .find(|(f, _)| f == name)
                .map(|(_, t)| t.as_slice())
                .unwrap_or(&[]);
            fills.push(FillIn {
                name: name.clone(),
                r_day: r.r_day.clone(),
                f_day: r.f_day.clone(),
                count: r.count.clone(),
                random_pool: r.random_mean_log.clone(),
                trades: trades
                    .iter()
                    .map(|t| TradeIn {
                        ret: t.ret,
                        hold: t.hold,
                        reason: t.exit_reason.to_string(),
                    })
                    .collect(),
            });
        }
        let frame: Vec<(String, String, Row)> = ev
            .fills
            .iter()
            .flat_map(|(f, r)| {
                r.by_period
                    .iter()
                    .map(move |(p, row, _)| (f.clone(), p.clone(), row.clone()))
            })
            .collect();
        let benchmarks: Vec<(String, Vec<f64>)> = s.benchmark.iter().cloned().collect();
        let path = self.out().join(format!("{}summary.svg", self.cfg.prefix()));
        let input = ChartInput {
            title: &format!("{TITLE}: {}", self.cfg.variant),
            caption: &self.caption(),
            note: "Dev bias = selection haircut (fills). Test bias = 0 by construction.",
            dates: &s.panel.dates,
            first_row: s.first_row,
            split_row: s.split_row,
            fills: &fills,
            b_day: &s.b_day,
            benchmarks: &benchmarks,
            frame: &frame,
        };
        dashboard::draw(&path, &input)?;
        println!("chart: {}", path.display());
        Ok(())
    }

    pub fn flags(&self) -> &[String] {
        self.evaluation
            .as_ref()
            .map(|e| e.flags.as_slice())
            .unwrap_or(&[])
    }

    pub fn baselines_len(&self) -> usize {
        self.evaluation
            .as_ref()
            .map_or(0, |e| e.baselines.len() + e.cost_ladder.len())
    }

    pub fn row(&self, fill: &str, period: &str) -> Option<Row> {
        let e = self.evaluation.as_ref()?;
        e.fills
            .iter()
            .find(|(f, _)| f == fill)?
            .1
            .by_period
            .iter()
            .find(|(p, _, _)| p == period)
            .map(|(_, r, _)| r.clone())
    }
}
