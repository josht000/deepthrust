//! Daily-bar strategy studies and shared staged evaluation pipeline.

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
use crate::symbols_overview::{Overview, SymbolSeries, SymbolStats, VariantSeries};
use crate::timing;

pub const NAME: &str = "s001_rsi2_connors";
pub const TITLE: &str = "S001 RSI(2) Classic (Connors)";
pub const S003_NAME: &str = "s003_rsi4_2575";
pub const S003_TITLE: &str = "S003 RSI(4) 25/75 (Connors)";
pub const S006_NAME: &str = "s006_double_7s";
pub const S006_TITLE: &str = "S006 Double 7s";
pub const ORDER: [&str; 5] = ["trades", "table", "rank", "partition", "charts"];
const MIN_DEV_TRADES: f64 = 100.0;
const MIN_TEST_TRADES: f64 = 30.0;

pub trait StrategyDefinition: Sync {
    fn name(&self) -> &'static str;
    fn title(&self) -> &'static str;
    fn compute_signals(&self, panel: &Panel, cfg: &Config) -> Signals;
}

struct RsiStrategyDefinition {
    name: &'static str,
    title: &'static str,
}

impl StrategyDefinition for RsiStrategyDefinition {
    fn name(&self) -> &'static str {
        self.name
    }

    fn title(&self) -> &'static str {
        self.title
    }

    fn compute_signals(&self, panel: &Panel, cfg: &Config) -> Signals {
        signals::compute_signals(panel, cfg)
    }
}

struct DoubleSevenStrategyDefinition;

impl StrategyDefinition for DoubleSevenStrategyDefinition {
    fn name(&self) -> &'static str {
        S006_NAME
    }

    fn title(&self) -> &'static str {
        S006_TITLE
    }

    fn compute_signals(&self, panel: &Panel, cfg: &Config) -> Signals {
        signals::compute_double7_signals(panel, cfg)
    }
}

static S001_STRATEGY: RsiStrategyDefinition = RsiStrategyDefinition {
    name: NAME,
    title: TITLE,
};
static S003_STRATEGY: RsiStrategyDefinition = RsiStrategyDefinition {
    name: S003_NAME,
    title: S003_TITLE,
};
static S006_STRATEGY: DoubleSevenStrategyDefinition = DoubleSevenStrategyDefinition;
static STRATEGY_REGISTRY: [&dyn StrategyDefinition; 3] =
    [&S001_STRATEGY, &S003_STRATEGY, &S006_STRATEGY];

pub fn strategy_definition(name: &str) -> Option<&'static dyn StrategyDefinition> {
    STRATEGY_REGISTRY
        .iter()
        .copied()
        .find(|strategy| strategy.name() == name)
}

pub fn supports_strategy(name: &str) -> bool {
    strategy_definition(name).is_some()
}

fn title_for(name: &str) -> &'static str {
    strategy_definition(name).map_or(TITLE, StrategyDefinition::title)
}

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
            results: results_root.join(&cfg.strategy),
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
        let sources = cfg.source_list();
        let mut symbol_bars = Vec::new();
        for source in &sources {
            for symbol in crate::config::split_csv(&source.symbols) {
                let bars =
                    data::load_symbol(&source.data_dir, &source.data_suffix, &symbol, start)?;
                if let Some(w) = data::start_gap_warning(&symbol, &bars, start) {
                    eprintln!("Warning: {w}");
                }
                symbol_bars.push((symbol, bars));
            }
        }
        symbol_bars.sort_by(|a, b| a.0.cmp(&b.0));
        let symbols: Vec<String> = symbol_bars
            .iter()
            .map(|(symbol, _)| symbol.clone())
            .collect();
        let bars: Vec<data::Bars> = symbol_bars.into_iter().map(|(_, bars)| bars).collect();
        let panel = {
            let _g = timing::stage("build panel");
            data::build_panel(&symbols, &bars)
        };
        let bench = if cfg.benchmark.is_empty() {
            None
        } else if !cfg.benchmark_data_dir.trim().is_empty() {
            let suffix = if cfg.benchmark_data_suffix.is_empty() {
                sources
                    .first()
                    .map(|source| source.data_suffix.as_str())
                    .unwrap_or(cfg.data_suffix.as_str())
            } else {
                cfg.benchmark_data_suffix.as_str()
            };
            let b = data::load_symbol(&cfg.benchmark_data_dir, suffix, &cfg.benchmark, start)
                .map_err(|e| format!("benchmark {}: {e}", cfg.benchmark))?;
            if !symbols.contains(&cfg.benchmark) {
                if let Some(w) = data::start_gap_warning(&cfg.benchmark, &b, start) {
                    eprintln!("Warning: {w}");
                }
            }
            Some((cfg.benchmark.clone(), b))
        } else {
            let mut found = None;
            let mut last_error = None;
            for source in &sources {
                match data::load_symbol(
                    &source.data_dir,
                    &source.data_suffix,
                    &cfg.benchmark,
                    start,
                ) {
                    Ok(b) => {
                        found = Some(b);
                        break;
                    }
                    Err(e) => last_error = Some(e),
                }
            }
            match found {
                Some(b) => {
                    if !symbols.contains(&cfg.benchmark) {
                        if let Some(w) = data::start_gap_warning(&cfg.benchmark, &b, start) {
                            eprintln!("Warning: {w}");
                        }
                    }
                    Some((cfg.benchmark.clone(), b))
                }
                None => {
                    eprintln!(
                        "Benchmark warning: {}; falling back to the equal-weight universe",
                        last_error
                            .unwrap_or_else(|| format!("no data source for {}", cfg.benchmark))
                    );
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
            strategy_definition(&cfg.strategy)
                .ok_or_else(|| format!("unregistered strategy {:?}", cfg.strategy))?
                .compute_signals(&panel, cfg)
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
                stop_pct: sig.stop_pct.as_ref(),
                stop_mult: cfg.disaster_stop_atr,
                signal_value: &sig.rsi,
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
        let avg_cost = s
            .trades
            .iter()
            .find(|(name, _)| name == fill)
            .map(|(_, trades)| {
                if trades.is_empty() {
                    self.cfg
                        .symbol_cost_bps
                        .values()
                        .map(|cost| cost / 1e4)
                        .sum::<f64>()
                        / self.cfg.symbol_cost_bps.len().max(1) as f64
                } else {
                    trades
                        .iter()
                        .map(|trade| self.cfg.cost_for_symbol(&s.panel.symbols[trade.sym_idx]))
                        .sum::<f64>()
                        / trades.len() as f64
                }
            })
            .unwrap_or_else(|| self.cfg.cost());
        let mat = if fill == "next_open" {
            s.panel.open.clone()
        } else {
            s.panel.close.clone()
        };
        Book {
            close_ff: s.close_ff.clone(),
            entry_mat: mat.clone(),
            exit_mat: mat,
            cost: avg_cost,
            symbol_costs: s
                .panel
                .symbols
                .iter()
                .map(|symbol| self.cfg.cost_for_symbol(symbol))
                .collect(),
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
            let source_column = if self.cfg.has_multiple_sources() {
                ",source"
            } else {
                ""
            };
            let mut out = format!("symbol{source_column},signal_date,entry_date,exit_date,entry_px,exit_px,ret,ret_gross,exit_reason,hold,{},fill\n", s.sig.indicator_name);
            for t in trades {
                let symbol = &s.panel.symbols[t.sym_idx];
                let source = self
                    .cfg
                    .symbol_sources
                    .get(symbol)
                    .map(String::as_str)
                    .unwrap_or("default");
                if self.cfg.has_multiple_sources() {
                    out += &format!(
                        "{symbol},{source},{},{},{},{},{},{},{},{},{},{},{}\n",
                        s.panel.dates[t.signal_idx],
                        s.panel.dates[t.entry_idx],
                        s.panel.dates[t.exit_idx],
                        t.entry_px,
                        t.exit_px,
                        t.ret,
                        t.ret_gross,
                        t.exit_reason,
                        t.hold,
                        csv_cell(t.signal_value),
                        fill
                    );
                } else {
                    out += &format!(
                        "{symbol},{},{},{},{},{},{},{},{},{},{},{}\n",
                        s.panel.dates[t.signal_idx],
                        s.panel.dates[t.entry_idx],
                        s.panel.dates[t.exit_idx],
                        t.entry_px,
                        t.exit_px,
                        t.ret,
                        t.ret_gross,
                        t.exit_reason,
                        t.hold,
                        csv_cell(t.signal_value),
                        fill
                    );
                }
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
            "{} trade summary (net of costs, research scale)",
            self.cfg.strategy
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
        let mut per = if self.cfg.has_multiple_sources() {
            String::from("fill,period,symbol,source,trades,win_pct,mean_ret_pct,pf,avg_hold\n")
        } else {
            String::from("fill,period,symbol,trades,win_pct,mean_ret_pct,pf,avg_hold\n")
        };
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
                    if self.cfg.has_multiple_sources() {
                        let source = self
                            .cfg
                            .symbol_sources
                            .get(sym)
                            .map(String::as_str)
                            .unwrap_or("default");
                        per += &format!(
                            "{fill},{period},{sym},{source},{},{},{},{},{}\n",
                            st[0],
                            csv_cell(st[1]),
                            csv_cell(st[2]),
                            csv_cell(st[3]),
                            csv_cell(st[4])
                        );
                    } else {
                        per += &format!(
                            "{fill},{period},{sym},{},{},{},{},{}\n",
                            st[0],
                            csv_cell(st[1]),
                            csv_cell(st[2]),
                            csv_cell(st[3]),
                            csv_cell(st[4])
                        );
                    }
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
        for (period, row) in [("dev", dev.as_ref()), ("test", test.as_ref())] {
            if let Some(row) = row {
                if row.get("sharpe") > 2.0 {
                    out.push(format!(
                        "review required: {period} Sharpe > 2; sanity-check for leakage"
                    ));
                }
                if row.get("pf") > 2.0 {
                    out.push(format!(
                        "review required: {period} profit factor > 2; sanity-check for leakage"
                    ));
                }
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
                let (r, _) = partition::account_series_with_costs(
                    &partition::legs_of(trades_of(f)),
                    &b.close_ff,
                    &b.symbol_costs,
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

        let mut flags = self.flags_of(&results, &primary);
        if let (Some((benchmark_name, benchmark_r)), Some((_, strategy))) =
            (&s.benchmark, results.first())
        {
            for (period, mask) in &s.masks {
                let benchmark_total: f64 = benchmark_r
                    .iter()
                    .zip(mask)
                    .filter(|(_, included)| **included)
                    .map(|(r, _)| r.max(-0.999_999).ln_1p())
                    .sum();
                let strategy_total = strategy
                    .by_period
                    .iter()
                    .find(|(name, _, _)| name == period)
                    .map(|(_, row, _)| row.get("total_log"))
                    .unwrap_or(f64::NAN);
                if strategy_total - benchmark_total >= 1.1f64.ln() {
                    flags.push(format!(
                        "{period} strategy outperforms {benchmark_name} by at least 10% cumulative growth"
                    ));
                }
            }
        }
        for f in &flags {
            section(&format!("{}: {f}", self.cfg.strategy));
        }
        section(&format!(
            "{} partition test (log returns, research scale, net of costs)",
            self.cfg.strategy
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
        let mut md = vec![
            format!("# {}: partition report", title_for(&cfg.strategy)),
            String::new(),
        ];
        if cfg.variant != "baseline" {
            md.push(format!(
                "- Variant: `{}` overlay {{{}}}",
                cfg.variant, cfg.variant_overlay
            ));
            md.push(String::new());
        }
        md.push(format!(
            "- Data: `{}`; sessions {} .. {}; evaluation starts {}.",
            cfg.source_list()
                .iter()
                .map(|source| source.data_dir.as_str())
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect::<Vec<_>>()
                .join(", "),
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
            "pf_ci_low",
            "pf_ci_high",
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
        let (c, s) = (self.cfg, self.state());
        let rule = if c.strategy == S006_NAME {
            let stop = if c.disaster_stop_atr == 0.0 {
                "No stop is applied in this run.".to_string()
            } else {
                format!(
                    "Stop below entry x (1 - {} x ATR({}) / signal close).",
                    c.disaster_stop_atr, c.atr_len
                )
            };
            format!(
                "- Rule: close > SMA({}) AND close <= the inclusive {}-session low; exit when close >= the inclusive {}-session high. {}",
                c.sma_window, c.lookback, c.lookback, stop
            )
        } else {
            format!("- Rule: close > gate ({}) AND RSI({}) < {}; exit {}; sizing {}; disaster stop x{} ATR({}).",
                c.gate, c.rsi_len, c.rsi_threshold, c.exit_mode, c.sizing, c.disaster_stop_atr, c.atr_len)
        };
        vec![
            format!("- Universe: {} (daily bars from {} sources); each symbol at most one position, book slots {}.",
                s.panel.symbols.join(", "),
                c.source_list().len(),
                c.book_slots),
            rule,
            "- `next_open` is the primary fill (signal on session t, fill at open t+1); `close` is the same-close UPPER BOUND. `moc_proxy` needs 15:50 minute bars, which are not fetched.".into(),
            "- Baselines: EW buy-and-hold of the tradable universe, SMA200 timing on that EW index (signal lagged one session), dev-vol-matched EW, and exposure-matched random timing.".into(),
            "- Limitations: moc_proxy fill unavailable; a single-symbol universe cannot reach the dev/test trade-count gates.".into(),
        ]
    }

    pub fn caption(&self) -> String {
        let (c, s) = (self.cfg, self.state());
        let strategy = if c.strategy == S006_NAME {
            format!(
                "Double7s lookback={} SMA{} stop={}x ATR%",
                c.lookback, c.sma_window, c.disaster_stop_atr
            )
        } else {
            format!(
                "RSI({})<{} gate={} exit={} sizing={} stop={}xATR",
                c.rsi_len, c.rsi_threshold, c.gate, c.exit_mode, c.sizing, c.disaster_stop_atr
            )
        };
        format!(
            "variant={} | {} | avg {}bps/side slots={} | {}..{}",
            c.variant,
            strategy,
            s.panel
                .symbols
                .iter()
                .map(|symbol| c.cost_for_symbol(symbol) * 1e4)
                .sum::<f64>()
                / s.panel.symbols.len().max(1) as f64,
            c.book_slots,
            s.panel.dates[0],
            s.panel.dates[s.panel.dates.len() - 1]
        )
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
            title: &format!("{}: {}", title_for(&self.cfg.strategy), self.cfg.variant),
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

    pub fn symbol_overview(&self) -> Result<Overview, String> {
        let state = self.state();
        let evaluation = self
            .evaluation
            .as_ref()
            .ok_or("symbol overview needs the partition stage")?;
        let fill = self
            .cfg
            .fills_list()
            .into_iter()
            .next()
            .ok_or("no fills configured")?;
        let fill_result = evaluation
            .fills
            .iter()
            .find(|(name, _)| name == &fill)
            .ok_or_else(|| format!("no partition result for primary fill {fill}"))?;
        let trades = state
            .trades
            .iter()
            .find(|(name, _)| name == &fill)
            .map(|(_, trades)| trades.as_slice())
            .unwrap_or(&[]);
        let split = state.split_row;
        let start = state.first_row;
        let end = state.panel.dates.len();
        let benchmark = state.benchmark.as_ref().map(|(_, returns)| returns.clone());
        let mut symbols = Vec::with_capacity(state.panel.symbols.len());
        for (symbol_idx, symbol) in state.panel.symbols.iter().enumerate() {
            let symbol_trades: Vec<_> = trades
                .iter()
                .filter(|trade| trade.sym_idx == symbol_idx)
                .collect();
            let legs: Vec<_> = symbol_trades
                .iter()
                .map(|trade| partition::Leg {
                    col: symbol_idx,
                    entry: trade.entry_idx,
                    exit: trade.exit_idx,
                    entry_px: trade.entry_px,
                    exit_px: trade.exit_px,
                })
                .collect();
            let (strategy_r, _) = partition::account_series_with_costs(
                &legs,
                &state.close_ff,
                &state
                    .panel
                    .symbols
                    .iter()
                    .map(|name| self.cfg.cost_for_symbol(name))
                    .collect::<Vec<_>>(),
                end,
                1,
            );
            let mut buy_hold_r = vec![0.0; end];
            for row in 1..end {
                let (previous, current) = (
                    state.close_ff[symbol_idx][row - 1],
                    state.close_ff[symbol_idx][row],
                );
                if previous.is_finite() && current.is_finite() && previous != 0.0 {
                    buy_hold_r[row] = current / previous - 1.0;
                }
            }
            let dev_trade_returns: Vec<f64> = symbol_trades
                .iter()
                .filter(|trade| trade.entry_idx < split)
                .map(|trade| trade.ret)
                .collect();
            let test_trade_returns: Vec<f64> = symbol_trades
                .iter()
                .filter(|trade| trade.entry_idx >= split)
                .map(|trade| trade.ret)
                .collect();
            let (dev_win_pct, dev_pf) = overview_trade_stats(&dev_trade_returns);
            let (test_win_pct, test_pf) = overview_trade_stats(&test_trade_returns);
            let dev_total = overview_total(&strategy_r, start, split);
            let test_total = overview_total(&strategy_r, split, end);
            let dev_buy_hold = overview_total(&buy_hold_r, start, split);
            let test_buy_hold = overview_total(&buy_hold_r, split, end);
            let dev_excess = dev_total - dev_buy_hold;
            let test_excess = test_total - test_buy_hold;
            let dev_sharpe = overview_sharpe(&strategy_r, start, split);
            let test_sharpe = overview_sharpe(&strategy_r, split, end);
            let dev_max_dd = overview_max_drawdown(&strategy_r, start, split);
            let test_max_dd = overview_max_drawdown(&strategy_r, split, end);
            let mut flags = Vec::new();
            if dev_trade_returns.len() < MIN_DEV_TRADES as usize
                || test_trade_returns.len() < MIN_TEST_TRADES as usize
            {
                flags.push("insufficient evidence".to_string());
            }
            if dev_excess.is_finite()
                && test_excess.is_finite()
                && dev_excess != 0.0
                && dev_excess.signum() != test_excess.signum()
            {
                flags.push("buy-and-hold excess sign flips".to_string());
            }
            if dev_excess >= 1.1f64.ln() || test_excess >= 1.1f64.ln() {
                flags.push("10%+ outperformance vs symbol buy-and-hold".to_string());
            }
            if let Some((benchmark_name, benchmark_returns)) = &state.benchmark {
                let benchmark_dev = overview_total(benchmark_returns, start, split);
                let benchmark_test = overview_total(benchmark_returns, split, end);
                if dev_total - benchmark_dev >= 1.1f64.ln()
                    || test_total - benchmark_test >= 1.1f64.ln()
                {
                    flags.push(format!("10%+ outperformance vs {benchmark_name}"));
                }
            }
            if dev_sharpe > 2.0 || test_sharpe > 2.0 || dev_pf > 2.0 || test_pf > 2.0 {
                flags.push("high metric: leakage sanity-check".to_string());
            };
            let stats = SymbolStats {
                dev_total,
                test_total,
                dev_excess,
                test_excess,
                dev_sharpe,
                test_sharpe,
                dev_pf,
                test_pf,
                dev_trades: dev_trade_returns.len(),
                test_trades: test_trade_returns.len(),
                dev_win_pct,
                test_win_pct,
                dev_max_dd,
                test_max_dd,
                flags: flags.join("; "),
            };
            symbols.push(SymbolSeries {
                symbol: symbol.clone(),
                source: self
                    .cfg
                    .symbol_sources
                    .get(symbol)
                    .cloned()
                    .unwrap_or_else(|| "default".into()),
                strategy_r,
                buy_hold_r,
                benchmark_r: benchmark.clone(),
                stats,
            });
        }
        let dev_skill = fill_result
            .1
            .by_period
            .iter()
            .find(|(period, _, _)| period == "dev")
            .map(|(_, row, _)| row.get("skill"))
            .unwrap_or(f64::NAN);
        Ok(Overview {
            dates: state.panel.dates.clone(),
            split_row: state.split_row,
            variants: vec![VariantSeries {
                name: self.cfg.variant.clone(),
                dev_skill,
                symbols,
            }],
        })
    }
}

fn overview_total(returns: &[f64], start: usize, end: usize) -> f64 {
    returns
        .iter()
        .skip(start)
        .take(end.saturating_sub(start))
        .map(|r| r.max(-0.999_999).ln_1p())
        .sum()
}

fn overview_sharpe(returns: &[f64], start: usize, end: usize) -> f64 {
    let values: Vec<f64> = returns
        .iter()
        .skip(start)
        .take(end.saturating_sub(start))
        .copied()
        .collect();
    let mean = stats::mean(&values);
    let sd = stats::std(&values);
    if sd > 0.0 {
        mean / sd * 252.0f64.sqrt()
    } else {
        f64::NAN
    }
}

fn overview_max_drawdown(returns: &[f64], start: usize, end: usize) -> f64 {
    let (mut equity, mut peak, mut drawdown) = (1.0f64, 1.0f64, 0.0f64);
    for r in returns.iter().skip(start).take(end.saturating_sub(start)) {
        equity *= 1.0 + r;
        peak = peak.max(equity);
        drawdown = drawdown.min(equity / peak - 1.0);
    }
    drawdown
}

fn overview_trade_stats(returns: &[f64]) -> (f64, f64) {
    if returns.is_empty() {
        return (f64::NAN, f64::NAN);
    }
    let wins: f64 = returns.iter().filter(|r| **r > 0.0).sum();
    let losses: f64 = -returns.iter().filter(|r| **r <= 0.0).sum::<f64>();
    let win_pct =
        returns.iter().filter(|r| **r > 0.0).count() as f64 * 100.0 / returns.len() as f64;
    (
        win_pct,
        if losses > 0.0 {
            wins / losses
        } else {
            f64::NAN
        },
    )
}
