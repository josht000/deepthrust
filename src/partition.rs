//! Daily-bar partition test (trend / random-pick / bias / skill) for equal-weight books
//! (port of src/framework/daily_partition.py and the helpers in daily_study.py).
//!
//!   account  every open position gets weight 1 / max(open positions, slots) each session
//!            (daily mark-to-market); the rest is cash. Costs are charged on entry/exit sessions.
//!   total    sum over the period's sessions of log(1 + account return).
//!   trend    sum over sessions of fraction_invested * log(1 + benchmark return); benchmark = equal-weight
//!            return of the tradable universe.
//!   random   mean total of `n_draws` seeded books where every trade is replaced by a random candidate.
//!   bias     0 on the locked test period; on dev the expected-maximum haircut for N_eff variants.
//!   skill    total - trend - bias (and vs. random); skill_pct = skill / total when both are >= 0.

use crate::engine::Trade;
use crate::signals::{Mask, Mat};
use crate::stats::{self, Rng};
use crate::timing;

pub type Masks = Vec<(String, Vec<bool>)>;

pub struct Book {
    pub close_ff: Mat,
    pub entry_mat: Mat,
    pub exit_mat: Mat,
    pub cost: f64,
    pub symbol_costs: Vec<f64>,
    pub n_rows: usize,
    pub b_day: Vec<f64>,
    pub slots: usize,
    pub periods_per_year: usize,
}

#[derive(Clone, Copy)]
pub struct Leg {
    pub col: usize,
    pub entry: usize,
    pub exit: usize,
    pub entry_px: f64,
    pub exit_px: f64,
    pub weight: f64,
    pub entry_turnover: f64,
    pub exit_turnover: f64,
    pub weighted: bool,
    pub borrow_cost_per_bar: f64,
}

pub fn legs_of(trades: &[Trade]) -> Vec<Leg> {
    trades
        .iter()
        .map(|t| Leg {
            col: t.sym_idx,
            entry: t.entry_idx,
            exit: t.exit_idx,
            entry_px: t.entry_px,
            exit_px: t.exit_px,
            weight: t.weight,
            entry_turnover: t.entry_turnover,
            exit_turnover: t.exit_turnover,
            weighted: t.weighted,
            borrow_cost_per_bar: t.borrow_cost_per_bar,
        })
        .collect()
}

/// Daily account return and session position count.
pub fn account_series(
    legs: &[Leg],
    close_ff: &Mat,
    cost: f64,
    n_rows: usize,
    slots: usize,
) -> (Vec<f64>, Vec<f64>) {
    let costs = vec![cost; close_ff.len()];
    account_series_with_costs(legs, close_ff, &costs, n_rows, slots)
}

pub fn account_series_with_costs(
    legs: &[Leg],
    close_ff: &Mat,
    costs: &[f64],
    n_rows: usize,
    slots: usize,
) -> (Vec<f64>, Vec<f64>) {
    let weighted_mode = legs.iter().any(|leg| leg.weighted);
    let (mut sums, mut count) = (vec![0.0f64; n_rows], vec![0.0f64; n_rows]);
    for l in legs {
        let cost = costs.get(l.col).copied().unwrap_or(0.0);
        for day in l.entry..=l.exit {
            if weighted_mode {
                let start = if day == l.entry {
                    l.entry_px
                } else {
                    close_ff[l.col][day.saturating_sub(1)]
                };
                let end = if day == l.exit {
                    l.exit_px
                } else {
                    close_ff[l.col][day]
                };
                let entry_cost = if day == l.entry {
                    cost * l.entry_turnover
                } else {
                    0.0
                };
                let exit_cost = if day == l.exit {
                    cost * l.exit_turnover
                } else {
                    0.0
                };
                let borrow_cost = l.weight.abs() * l.borrow_cost_per_bar;
                sums[day] += l.weight * (end / start - 1.0) - entry_cost - exit_cost - borrow_cost;
                count[day] += 1.0;
                continue;
            }
            let start = if day == l.entry {
                l.entry_px * (1.0 + cost)
            } else {
                close_ff[l.col][day.saturating_sub(1)]
            };
            let end = if day == l.exit {
                l.exit_px * (1.0 - cost)
            } else {
                close_ff[l.col][day]
            };
            sums[day] += end / start - 1.0;
            count[day] += 1.0;
        }
    }
    let r = sums
        .iter()
        .zip(&count)
        .map(|(s, c)| {
            if weighted_mode {
                *s
            } else {
                s / c.max(slots as f64)
            }
        })
        .collect();
    (r, count)
}

fn candidate_lists(mask: &Mask) -> Vec<Vec<usize>> {
    let n_rows = mask.first().map_or(0, |m| m.len());
    (0..n_rows)
        .map(|t| (0..mask.len()).filter(|&j| mask[j][t]).collect())
        .collect()
}

fn period_total(log_r: &[f64], mask: &[bool]) -> f64 {
    log_r
        .iter()
        .zip(mask)
        .filter(|(_, &m)| m)
        .map(|(v, _)| v)
        .sum()
}

/// Totals per draw and period plus the mean per-session log return across draws.
#[allow(clippy::too_many_arguments)]
fn random_books(
    trades: &[Trade],
    cand: &[Vec<usize>],
    book: &Book,
    masks: &Masks,
    n_draws: usize,
    seed: u64,
) -> (Vec<Vec<f64>>, Vec<f64>) {
    let mut rng = Rng::new(seed);
    let mut totals = vec![vec![0.0; masks.len()]; n_draws];
    let mut mean_log = vec![0.0; book.n_rows];
    for total in totals.iter_mut() {
        let legs: Vec<Leg> = trades
            .iter()
            .map(|t| {
                let c = &cand[t.signal_idx];
                let ok = |col: usize| {
                    book.entry_mat[col][t.entry_idx].is_finite()
                        && book.exit_mat[col][t.exit_idx].is_finite()
                };
                let mut col = t.sym_idx;
                if !c.is_empty() {
                    for _ in 0..9 {
                        let pick = c[rng.below(c.len())];
                        if ok(pick) {
                            col = pick;
                            break;
                        }
                    }
                }
                Leg {
                    col,
                    entry: t.entry_idx,
                    exit: t.exit_idx,
                    entry_px: book.entry_mat[col][t.entry_idx],
                    exit_px: book.exit_mat[col][t.exit_idx],
                    weight: t.weight,
                    entry_turnover: t.entry_turnover,
                    exit_turnover: t.exit_turnover,
                    weighted: t.weighted,
                    borrow_cost_per_bar: t.borrow_cost_per_bar,
                }
            })
            .collect();
        let (r, _) = account_series_with_costs(
            &legs,
            &book.close_ff,
            &book.symbol_costs,
            book.n_rows,
            book.slots,
        );
        let log_r: Vec<f64> = r.iter().map(|v| v.ln_1p()).collect();
        for (m, (_, mask)) in masks.iter().enumerate() {
            total[m] = period_total(&log_r, mask);
        }
        for (a, b) in mean_log.iter_mut().zip(&log_r) {
            *a += b;
        }
    }
    for v in mean_log.iter_mut() {
        *v /= n_draws.max(1) as f64;
    }
    (totals, mean_log)
}

/// Ordered name -> value row of one period's statistics.
#[derive(Clone, Debug, Default)]
pub struct Row(pub Vec<(&'static str, f64)>);

impl Row {
    pub fn get(&self, k: &str) -> f64 {
        self.0
            .iter()
            .find(|(n, _)| *n == k)
            .map_or(f64::NAN, |(_, v)| *v)
    }
    pub fn set(&mut self, k: &'static str, v: f64) {
        match self.0.iter_mut().find(|(n, _)| *n == k) {
            Some(e) => e.1 = v,
            None => self.0.push((k, v)),
        }
    }
}

fn perf(
    r: &[f64],
    trade_ret: &[f64],
    hold: &[f64],
    periods_per_year: usize,
) -> Vec<(&'static str, f64)> {
    let mut eq = 1.0;
    let (mut peak, mut dd) = (f64::MIN, 0.0f64);
    for v in r {
        eq *= 1.0 + v;
        peak = peak.max(eq);
        dd = dd.min(eq / peak - 1.0);
    }
    let wins: f64 = trade_ret.iter().filter(|v| **v > 0.0).sum();
    let losses: f64 = -trade_ret.iter().filter(|v| **v <= 0.0).sum::<f64>();
    let sd = stats::std(r);
    let n = trade_ret.len() as f64;
    let nan_if_empty = |v: f64| if trade_ret.is_empty() { f64::NAN } else { v };
    vec![
        (
            "win_pct",
            nan_if_empty(100.0 * trade_ret.iter().filter(|v| **v > 0.0).count() as f64 / n),
        ),
        ("mean_ret_pct", 100.0 * stats::mean(trade_ret)),
        ("median_ret_pct", 100.0 * stats::median(trade_ret)),
        (
            "pf",
            if losses > 0.0 {
                wins / losses
            } else {
                f64::NAN
            },
        ),
        (
            "cagr_pct",
            100.0
                * (r.iter().map(|v| v.ln_1p()).sum::<f64>() * periods_per_year as f64
                    / r.len().max(1) as f64)
                    .exp_m1(),
        ),
        (
            "sharpe",
            if sd > 0.0 {
                stats::mean(r) / sd * (periods_per_year as f64).sqrt()
            } else {
                f64::NAN
            },
        ),
        (
            "max_dd_pct",
            if r.is_empty() { f64::NAN } else { 100.0 * dd },
        ),
        ("avg_hold", stats::mean(hold)),
    ]
}

fn profit_factor(returns: &[f64]) -> f64 {
    let wins: f64 = returns.iter().filter(|r| **r > 0.0).sum();
    let losses: f64 = -returns.iter().filter(|r| **r <= 0.0).sum::<f64>();
    if losses > 0.0 {
        wins / losses
    } else if wins > 0.0 {
        f64::INFINITY
    } else {
        f64::NAN
    }
}

fn bootstrap_pf_ci(returns: &[f64], n_boot: usize, seed: u64) -> (f64, f64) {
    if returns.is_empty() || n_boot == 0 {
        return (f64::NAN, f64::NAN);
    }
    let mut rng = Rng::new(seed);
    let mut values = Vec::with_capacity(n_boot);
    for _ in 0..n_boot {
        let sample: Vec<f64> = (0..returns.len())
            .map(|_| returns[rng.below(returns.len())])
            .collect();
        let pf = profit_factor(&sample);
        if !pf.is_nan() {
            values.push(pf);
        }
    }
    if values.is_empty() {
        return (f64::NAN, f64::NAN);
    }
    values.sort_by(f64::total_cmp);
    let quantile = |q: f64| {
        let index = q * (values.len() - 1) as f64;
        if values[index.floor() as usize].is_infinite()
            || values[index.ceil() as usize].is_infinite()
        {
            f64::INFINITY
        } else {
            stats::quantile_sorted(&values, q)
        }
    };
    (quantile(0.025), quantile(0.975))
}

pub struct Bias {
    pub value: f64,
    pub src: String,
}

fn bias_of(r_period: &[f64], n_eff: f64, oos: bool, note: &str) -> Bias {
    if oos {
        return Bias {
            value: 0.0,
            src: "0: no test bias haircut; an inspected test period is not a fresh holdout".into(),
        };
    }
    let logs: Vec<f64> = r_period.iter().map(|v| v.ln_1p()).collect();
    let sd = stats::std(&logs) * (r_period.len() as f64).sqrt();
    let value = stats::expected_max_noise(sd, n_eff);
    let note = if note.is_empty() {
        String::new()
    } else {
        format!("; {note}")
    };
    Bias {
        value,
        src: format!(
            "B haircut: expected max of N_eff={n_eff:.2} comparable variants, sd={sd:.4}{note}"
        ),
    }
}

pub struct FillResult {
    pub by_period: Vec<(String, Row, String)>, // (period, stats, bias source)
    pub r_day: Vec<f64>,
    pub f_day: Vec<f64>,
    pub trend_exposure: Vec<f64>,
    pub count: Vec<f64>,
    pub random_mean_log: Vec<f64>,
}

#[allow(clippy::too_many_arguments)]
pub fn evaluate_fill(
    trades: &[Trade],
    book: &Book,
    masks: &Masks,
    universe: &Mask,
    pool: &Mask,
    n_eff: f64,
    n_draws: usize,
    n_boot: usize,
    seed: u64,
    bias_note: &str,
) -> FillResult {
    let _g = timing::stage("daily partition");
    let n_rows = book.n_rows;
    let legs = legs_of(trades);
    let (r_day, count, f_day, trend_exposure, entry_w, exit_w) = {
        let _g = timing::stage("account");
        let (r_day, mut count) = account_series_with_costs(
            &legs,
            &book.close_ff,
            &book.symbol_costs,
            n_rows,
            book.slots,
        );
        let (mut ew, mut xw) = (vec![0.0; n_rows], vec![0.0; n_rows]);
        let weighted_mode = trades.iter().any(|trade| trade.weighted);
        let mut net_exposure = vec![0.0; n_rows];
        let f_day = if weighted_mode {
            let mut exposure = vec![0.0; n_rows];
            count.fill(0.0);
            for trade in trades {
                let first = if trade.entry_idx == trade.exit_idx {
                    trade.entry_idx
                } else {
                    trade.entry_idx + 1
                };
                for day in first..=trade.exit_idx {
                    exposure[day] += trade.weight.abs();
                    net_exposure[day] += trade.weight;
                    count[day] += 1.0;
                }
                ew[trade.entry_idx] += trade.entry_turnover;
                xw[trade.exit_idx] += trade.exit_turnover;
            }
            exposure
        } else {
            for trade in trades {
                ew[trade.entry_idx] += 1.0 / count[trade.entry_idx].max(book.slots as f64);
                xw[trade.exit_idx] += 1.0 / count[trade.exit_idx].max(book.slots as f64);
            }
            count
                .iter()
                .map(|c: &f64| c / c.max(book.slots as f64))
                .collect()
        };
        if !weighted_mode {
            net_exposure.clone_from(&f_day);
        }
        (r_day, count, f_day, net_exposure, ew, xw)
    };
    let mut random = vec![];
    for (label, cands, offset) in [("universe", universe, 17u64), ("pool", pool, 0u64)] {
        let _g = timing::stage(&format!("random picks {label}"));
        let cand = candidate_lists(cands);
        random.push((
            label,
            random_books(trades, &cand, book, masks, n_draws, seed + offset),
        ));
    }
    let mut period_of = vec![usize::MAX; n_rows];
    for (m, (_, mask)) in masks.iter().enumerate() {
        for (t, &on) in mask.iter().enumerate() {
            if on {
                period_of[t] = m;
            }
        }
    }
    let mut by_period = vec![];
    for (m, (name, mask)) in masks.iter().enumerate() {
        let sel: Vec<&Trade> = trades
            .iter()
            .filter(|t| period_of[t.entry_idx] == m)
            .collect();
        let r_period: Vec<f64> = (0..n_rows).filter(|&t| mask[t]).map(|t| r_day[t]).collect();
        let bias = bias_of(&r_period, n_eff, name == "test", bias_note);
        // exclude the five best trades of this period
        let mut order: Vec<usize> = (0..trades.len())
            .filter(|&i| period_of[trades[i].entry_idx] == m)
            .collect();
        order.sort_by(|&a, &b| trades[b].ret.partial_cmp(&trades[a].ret).unwrap());
        let drop: std::collections::HashSet<usize> = order.into_iter().take(5).collect();
        let r_ex5 = {
            let _g = timing::stage("ex5 account");
            let sub: Vec<Leg> = legs
                .iter()
                .enumerate()
                .filter(|(i, _)| !drop.contains(i))
                .map(|(_, l)| *l)
                .collect();
            account_series_with_costs(&sub, &book.close_ff, &book.symbol_costs, n_rows, book.slots)
                .0
        };
        let _g = timing::stage("bootstrap");
        let rnd: Vec<(&str, Vec<f64>)> = random
            .iter()
            .map(|(k, (tot, _))| (*k, tot.iter().map(|row| row[m]).collect()))
            .collect();
        let row = partition_period(
            &r_day,
            &f_day,
            &trend_exposure,
            &count,
            &book.b_day,
            mask,
            &sel,
            &rnd,
            &bias,
            &r_ex5,
            n_boot,
            seed,
            &entry_w,
            &exit_w,
            book.periods_per_year,
        );
        by_period.push((name.clone(), row, bias.src));
    }
    let random_mean_log = random
        .last()
        .map(|(_, (_, m))| m.clone())
        .unwrap_or_default();
    FillResult {
        by_period,
        r_day,
        f_day,
        trend_exposure,
        count,
        random_mean_log,
    }
}

#[allow(clippy::too_many_arguments)]
fn partition_period(
    r_day: &[f64],
    f_day: &[f64],
    trend_exposure: &[f64],
    count: &[f64],
    b_day: &[f64],
    mask: &[bool],
    trades: &[&Trade],
    random: &[(&str, Vec<f64>)],
    bias: &Bias,
    r_ex5: &[f64],
    n_boot: usize,
    seed: u64,
    entry_w: &[f64],
    exit_w: &[f64],
    periods_per_year: usize,
) -> Row {
    let idx: Vec<usize> = (0..mask.len()).filter(|&t| mask[t]).collect();
    let n_days = idx.len();
    let log_r: Vec<f64> = idx.iter().map(|&t| r_day[t].ln_1p()).collect();
    let log_b: Vec<f64> = idx.iter().map(|&t| b_day[t].ln_1p()).collect();
    let f: Vec<f64> = idx.iter().map(|&t| f_day[t]).collect();
    let net: Vec<f64> = idx.iter().map(|&t| trend_exposure[t]).collect();
    let total: f64 = log_r.iter().sum();
    let trend: f64 = net.iter().zip(&log_b).map(|(a, b)| a * b).sum();
    let inv: Vec<usize> = (0..n_days).filter(|&i| f[i] > 0.0).collect();
    let (ly, lx): (Vec<f64>, Vec<f64>) = inv.iter().map(|&i| (log_r[i], log_b[i])).unzip();
    let (beta, beta_t) = stats::ols(&ly, &lx);
    let mut out = Row::default();
    out.set("days", n_days as f64);
    out.set("trades", trades.len() as f64);
    out.set("total_log", total);
    out.set("trend", trend);
    out.set(
        "trend_beta",
        if beta.is_finite() {
            beta * lx.iter().sum::<f64>()
        } else {
            f64::NAN
        },
    );
    out.set("beta", beta);
    out.set("beta_t", beta_t);
    out.set(
        "avg_fraction_invested",
        if inv.is_empty() {
            0.0
        } else {
            stats::mean(&inv.iter().map(|&i| f[i]).collect::<Vec<_>>())
        },
    );
    out.set(
        "avg_positions",
        if inv.is_empty() {
            0.0
        } else {
            stats::mean(&inv.iter().map(|&i| count[idx[i]]).collect::<Vec<_>>())
        },
    );
    let w: f64 = idx.iter().map(|&t| entry_w[t] + exit_w[t]).sum();
    out.set(
        "turnover_annual",
        w * periods_per_year as f64 / n_days.max(1) as f64,
    );
    out.set("bias", bias.value);
    let tr_ret: Vec<f64> = trades.iter().map(|t| t.ret).collect();
    let tr_hold: Vec<f64> = trades.iter().map(|t| t.hold as f64).collect();
    let r_period: Vec<f64> = idx.iter().map(|&t| r_day[t]).collect();
    for (k, v) in perf(&r_period, &tr_ret, &tr_hold, periods_per_year) {
        out.set(k, v);
    }
    let (pf_lo, pf_hi) = bootstrap_pf_ci(&tr_ret, n_boot, seed + 2);
    out.set("pf_ci_low", pf_lo);
    out.set("pf_ci_high", pf_hi);
    for (name, totals) in random {
        let key = |p: &str| -> &'static str {
            match (p, *name) {
                ("pick", "universe") => "random_pick_universe",
                ("pick", _) => "random_pick_pool",
                ("p", "universe") => "random_p_universe",
                ("p", _) => "random_p_pool",
                (_, "universe") => "skill_vs_universe",
                _ => "skill_vs_pool",
            }
        };
        let mean = stats::mean(totals);
        out.set(key("pick"), mean);
        out.set(
            key("p"),
            (totals.iter().filter(|&&v| v >= total).count() as f64 + 1.0)
                / (totals.len() as f64 + 1.0),
        );
        out.set(key("skill"), total - mean - bias.value);
    }
    let skill = total - trend - bias.value;
    out.set("skill", skill);
    out.set(
        "skill_pct",
        if total > 0.0 && skill >= 0.0 {
            skill / total
        } else {
            f64::NAN
        },
    );
    let svp = out.get("skill_vs_pool");
    out.set(
        "skill_pct_random",
        if total > 0.0 && svp >= 0.0 {
            svp / total
        } else {
            f64::NAN
        },
    );
    let (mut lo, mut hi, mut p) = (f64::NAN, f64::NAN, f64::NAN);
    if n_days > 0 && n_boot > 0 {
        let mut rng = Rng::new(seed + 1);
        let resid: Vec<f64> = (0..n_days).map(|i| log_r[i] - net[i] * log_b[i]).collect();
        let mut sk: Vec<f64> = (0..n_boot)
            .map(|_| (0..n_days).map(|_| resid[rng.below(n_days)]).sum::<f64>() - bias.value)
            .collect();
        p = (sk.iter().filter(|&&v| v <= 0.0).count() as f64 + 1.0) / (n_boot as f64 + 1.0);
        sk.sort_by(|a, b| a.partial_cmp(b).unwrap());
        lo = stats::quantile_sorted(&sk, 0.025);
        hi = stats::quantile_sorted(&sk, 0.975);
    }
    out.set("skill_lo", lo);
    out.set("skill_hi", hi);
    out.set("skill_p", p);
    out.set(
        "skill_ex5",
        idx.iter().map(|&t| r_ex5[t].ln_1p()).sum::<f64>() - trend - bias.value,
    );
    out
}

/// Exact account totals / Sharpe per period at cost x multiplier: (mult, bps, period, total_log, sharpe).
pub fn cost_ladder(
    trades: &[Trade],
    book: &Book,
    masks: &Masks,
    multipliers: &[f64],
) -> Vec<(f64, f64, String, f64, f64)> {
    let legs = legs_of(trades);
    let mut rows = vec![];
    for &mult in multipliers {
        let costs: Vec<f64> = book.symbol_costs.iter().map(|cost| cost * mult).collect();
        let (r_day, _) =
            account_series_with_costs(&legs, &book.close_ff, &costs, book.n_rows, book.slots);
        for (name, mask) in masks {
            let r: Vec<f64> = (0..book.n_rows)
                .filter(|&t| mask[t])
                .map(|t| r_day[t])
                .collect();
            let sd = stats::std(&r);
            rows.push((
                mult,
                book.cost * mult * 1e4,
                name.clone(),
                r.iter().map(|v| v.ln_1p()).sum(),
                if sd > 0.0 {
                    stats::mean(&r) / sd * (book.periods_per_year as f64).sqrt()
                } else {
                    f64::NAN
                },
            ));
        }
    }
    rows
}

/// Exposure-matched random-timing null: same instruments and hold lengths, uniform random entry rows.
pub fn random_timing(
    trades: &[Trade],
    book: &Book,
    masks: &Masks,
    first_row: usize,
    last_by_col: &[Option<usize>],
    n_draws: usize,
    seed: u64,
) -> Vec<Vec<f64>> {
    let mut rng = Rng::new(seed + 31);
    let mut totals = vec![vec![0.0; masks.len()]; n_draws];
    for total in totals.iter_mut() {
        let mut legs = vec![];
        let mut placed: Vec<Vec<(usize, usize)>> = vec![vec![]; book.close_ff.len()];
        for t in trades {
            let (s, h) = (t.sym_idx, t.exit_idx - t.entry_idx);
            let Some(last) = last_by_col[s] else { continue };
            for _ in 0..10 {
                let e = rng.range(first_row, book.n_rows.saturating_sub(h).max(first_row + 1));
                if e > last {
                    continue;
                }
                let x = (e + h).min(last);
                if !book.entry_mat[s][e].is_finite()
                    || !book.exit_mat[s][x].is_finite()
                    || placed[s].iter().any(|&(pe, px)| e < px && x > pe)
                {
                    continue;
                }
                placed[s].push((e, x));
                legs.push(Leg {
                    col: s,
                    entry: e,
                    exit: x,
                    entry_px: book.entry_mat[s][e],
                    exit_px: book.exit_mat[s][x],
                    weight: t.weight,
                    entry_turnover: t.entry_turnover,
                    exit_turnover: t.exit_turnover,
                    weighted: t.weighted,
                    borrow_cost_per_bar: t.borrow_cost_per_bar,
                });
                break;
            }
        }
        if legs.is_empty() {
            continue;
        }
        let (r, _) = account_series_with_costs(
            &legs,
            &book.close_ff,
            &book.symbol_costs,
            book.n_rows,
            book.slots,
        );
        let log_r: Vec<f64> = r.iter().map(|v| v.ln_1p()).collect();
        for (m, (_, mask)) in masks.iter().enumerate() {
            total[m] = period_total(&log_r, mask);
        }
    }
    totals
}

/// Baseline stats of a daily-return series: (period, total_log, sharpe, cagr_pct, max_dd_pct).
pub fn baseline_stats(
    r_day: &[f64],
    masks: &Masks,
    periods_per_year: usize,
) -> Vec<(String, f64, f64, f64, f64)> {
    masks
        .iter()
        .map(|(period, mask)| {
            let r: Vec<f64> = r_day
                .iter()
                .zip(mask)
                .filter(|(_, &m)| m)
                .map(|(v, _)| *v)
                .collect();
            let total: f64 = r.iter().map(|v| v.ln_1p()).sum();
            let sd = stats::std(&r);
            let (mut eq, mut peak, mut dd) = (1.0, f64::MIN, 0.0f64);
            for v in &r {
                eq *= 1.0 + v;
                peak = peak.max(eq);
                dd = dd.min(eq / peak - 1.0);
            }
            (
                period.clone(),
                total,
                if sd > 0.0 {
                    stats::mean(&r) / sd * (periods_per_year as f64).sqrt()
                } else {
                    f64::NAN
                },
                100.0 * (total * periods_per_year as f64 / r.len().max(1) as f64).exp_m1(),
                if r.is_empty() { f64::NAN } else { 100.0 * dd },
            )
        })
        .collect()
}

/// Scale benchmark daily returns so its fit-period volatility matches the strategy's.
pub fn vol_matched(bench: &[f64], strat: &[f64], fit: &[bool]) -> Vec<f64> {
    let pick = |x: &[f64]| -> Vec<f64> {
        x.iter()
            .zip(fit)
            .filter(|(_, &m)| m)
            .map(|(v, _)| *v)
            .collect()
    };
    let (sb, ss) = (stats::std(&pick(bench)), stats::std(&pick(strat)));
    let w = if sb > 0.0 && ss.is_finite() {
        ss / sb
    } else {
        1.0
    };
    bench.iter().map(|v| w * v).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn trade(entry: usize, exit: usize, ep: f64, xp: f64) -> Trade {
        Trade {
            sym_idx: 0,
            signal_idx: entry.saturating_sub(1),
            entry_idx: entry,
            exit_idx: exit,
            entry_px: ep,
            exit_px: xp,
            ret: xp / ep - 1.0,
            ret_gross: xp / ep - 1.0,
            exit_reason: "signal",
            hold: exit - entry,
            signal_value: 5.0,
            weight: 1.0,
            entry_turnover: 1.0,
            exit_turnover: 1.0,
            weighted: false,
            borrow_cost_per_bar: 0.0,
        }
    }

    #[test]
    fn account_marks_to_market_and_charges_costs() {
        let close = vec![vec![100.0, 101.0, 103.0, 102.0]];
        let legs = vec![Leg {
            col: 0,
            entry: 1,
            exit: 3,
            entry_px: 100.0,
            exit_px: 102.0,
            weight: 1.0,
            entry_turnover: 1.0,
            exit_turnover: 1.0,
            weighted: false,
            borrow_cost_per_bar: 0.0,
        }];
        let (r, c) = account_series(&legs, &close, 0.001, 4, 1);
        assert_eq!(c, vec![0.0, 1.0, 1.0, 1.0]);
        assert!((r[1] - (101.0 / 100.1 - 1.0)).abs() < 1e-12);
        assert!((r[2] - (103.0 / 101.0 - 1.0)).abs() < 1e-12);
        assert!((r[3] - (102.0 * 0.999 / 103.0 - 1.0)).abs() < 1e-12);
        // with 2 slots the lone position is half weight
        assert!(
            (account_series(&legs, &close, 0.0, 4, 2).0[2] - 0.5 * (103.0 / 101.0 - 1.0)).abs()
                < 1e-12
        );
    }

    #[test]
    fn account_applies_source_specific_cost_per_symbol() {
        let close = vec![vec![100.0], vec![100.0]];
        let legs = vec![
            Leg {
                col: 0,
                entry: 0,
                exit: 0,
                entry_px: 100.0,
                exit_px: 100.0,
                weight: 1.0,
                entry_turnover: 1.0,
                exit_turnover: 1.0,
                weighted: false,
                borrow_cost_per_bar: 0.0,
            },
            Leg {
                col: 1,
                entry: 0,
                exit: 0,
                entry_px: 100.0,
                exit_px: 100.0,
                weight: 1.0,
                entry_turnover: 1.0,
                exit_turnover: 1.0,
                weighted: false,
                borrow_cost_per_bar: 0.0,
            },
        ];
        let costs = [0.0005, 0.0015];
        let expected = costs
            .iter()
            .map(|cost| (1.0 - cost) / (1.0 + cost) - 1.0)
            .sum::<f64>()
            / 2.0;
        let (returns, _) = account_series_with_costs(&legs, &close, &costs, 1, 1);
        assert!((returns[0] - expected).abs() < 1e-12);
    }

    #[test]
    fn weighted_account_scales_exposure_and_turnover_costs() {
        let close = vec![vec![100.0; 2]];
        let legs = vec![Leg {
            col: 0,
            entry: 1,
            exit: 1,
            entry_px: 100.0,
            exit_px: 101.0,
            weight: 0.5,
            entry_turnover: 0.5,
            exit_turnover: 0.5,
            weighted: true,
            borrow_cost_per_bar: 0.0,
        }];
        let (returns, _) = account_series(&legs, &close, 0.001, 2, 1);
        assert!((returns[1] - (0.5 * 0.01 - 0.001)).abs() < 1e-12);
    }

    #[test]
    fn weighted_partition_reports_exposure_and_weighted_turnover() {
        let close = vec![vec![100.0; 2]];
        let book = Book {
            close_ff: close.clone(),
            entry_mat: close.clone(),
            exit_mat: close,
            cost: 0.001,
            symbol_costs: vec![0.001],
            n_rows: 2,
            b_day: vec![0.0; 2],
            slots: 1,
            periods_per_year: 252,
        };
        let mut position = trade(1, 1, 100.0, 101.0);
        position.weight = 0.5;
        position.entry_turnover = 0.5;
        position.exit_turnover = 0.5;
        position.weighted = true;
        position.ret = 0.004;
        let all: Mask = vec![vec![true; 2]];
        let masks: Masks = vec![
            ("dev".into(), vec![false, true]),
            ("test".into(), vec![true, false]),
        ];
        let result = evaluate_fill(&[position], &book, &masks, &all, &all, 1.0, 2, 10, 1, "");
        assert_eq!(result.count[1], 1.0);
        assert_eq!(result.f_day[1], 0.5);
        let row = &result.by_period[0].1;
        assert_eq!(row.get("avg_fraction_invested"), 0.5);
        assert_eq!(row.get("avg_positions"), 1.0);
        assert_eq!(row.get("turnover_annual"), 252.0);
    }

    #[test]
    fn weighted_rebalances_do_not_double_count_exposure() {
        let close = vec![vec![100.0; 5], vec![100.0; 5]];
        let book = Book {
            close_ff: close.clone(),
            entry_mat: close.clone(),
            exit_mat: close,
            cost: 0.0,
            symbol_costs: vec![0.0; 2],
            n_rows: 5,
            b_day: vec![0.0, 0.01, 0.01, 0.01, 0.01],
            slots: 1,
            periods_per_year: 12,
        };
        let mut first = trade(0, 2, 100.0, 100.0);
        first.weighted = true;
        let mut second = trade(2, 4, 100.0, 100.0);
        second.sym_idx = 1;
        second.weighted = true;
        let masks: Masks = vec![("dev".into(), vec![true; 5])];
        let universe: Mask = vec![vec![true; 5]; 2];
        let result = evaluate_fill(
            &[first, second],
            &book,
            &masks,
            &universe,
            &universe,
            1.0,
            1,
            0,
            1,
            "",
        );
        assert_eq!(result.f_day, vec![0.0, 1.0, 1.0, 1.0, 1.0]);
        assert_eq!(result.count, vec![0.0, 1.0, 1.0, 1.0, 1.0]);
        assert!((result.by_period[0].1.get("trend") - 4.0 * 0.01_f64.ln_1p()).abs() < 1e-12);
    }

    #[test]
    fn weighted_open_rebalance_preserves_both_return_segments() {
        let close = vec![vec![100.0, 100.0, 105.0], vec![100.0, 100.0, 110.0]];
        let legs = vec![
            Leg {
                col: 0,
                entry: 0,
                exit: 2,
                entry_px: 100.0,
                exit_px: 105.0,
                weight: 1.0,
                entry_turnover: 1.0,
                exit_turnover: 1.0,
                weighted: true,
                borrow_cost_per_bar: 0.0,
            },
            Leg {
                col: 1,
                entry: 2,
                exit: 2,
                entry_px: 100.0,
                exit_px: 110.0,
                weight: 1.0,
                entry_turnover: 1.0,
                exit_turnover: 1.0,
                weighted: true,
                borrow_cost_per_bar: 0.0,
            },
        ];
        let (returns, _) = account_series(&legs, &close, 0.0, 3, 1);
        assert!((returns[2] - 0.15).abs() < 1e-12);
    }

    #[test]
    fn monthly_performance_and_baselines_use_twelve_periods() {
        let returns = [0.02, -0.01, 0.03, 0.01];
        let monthly = perf(&returns, &[0.02, -0.01], &[1.0, 1.0], 12);
        let daily = perf(&returns, &[0.02, -0.01], &[1.0, 1.0], 252);
        let find =
            |values: &[(&str, f64)], key: &str| values.iter().find(|x| x.0 == key).unwrap().1;
        assert!(
            (find(&monthly, "sharpe") / find(&daily, "sharpe") - (12.0_f64 / 252.0).sqrt()).abs()
                < 1e-12
        );
        let expected = 100.0 * (returns.iter().map(|r| f64::ln_1p(*r)).sum::<f64>() * 3.0).exp_m1();
        assert!((find(&monthly, "cagr_pct") - expected).abs() < 1e-12);
        let masks = vec![("dev".to_string(), vec![true; 4])];
        let baseline = baseline_stats(&returns, &masks, 12);
        assert!((baseline[0].2 - find(&monthly, "sharpe")).abs() < 1e-12);
        assert!((baseline[0].3 - expected).abs() < 1e-12);
    }

    #[test]
    fn long_short_trend_uses_net_not_gross_exposure() {
        let close = vec![vec![100.0; 3]; 2];
        let book = Book {
            close_ff: close.clone(),
            entry_mat: close.clone(),
            exit_mat: close,
            cost: 0.0,
            symbol_costs: vec![0.0; 2],
            n_rows: 3,
            b_day: vec![0.0, 0.1, 0.1],
            slots: 1,
            periods_per_year: 12,
        };
        let mut long = trade(0, 2, 100.0, 100.0);
        long.weighted = true;
        let mut short = long.clone();
        short.sym_idx = 1;
        short.weight = -1.0;
        let masks = vec![("dev".to_string(), vec![true; 3])];
        let universe = vec![vec![true; 3]; 2];
        let result = evaluate_fill(
            &[long, short],
            &book,
            &masks,
            &universe,
            &universe,
            1.0,
            1,
            0,
            1,
            "",
        );
        assert_eq!(result.f_day, vec![0.0, 2.0, 2.0]);
        assert_eq!(result.trend_exposure, vec![0.0, 0.0, 0.0]);
        assert_eq!(result.by_period[0].1.get("trend"), 0.0);
        assert_eq!(result.by_period[0].1.get("avg_fraction_invested"), 2.0);
    }

    #[test]
    fn trade_profit_factor_interval_is_deterministic() {
        let returns = [0.1, 0.04, -0.05, -0.02, 0.03];
        let first = bootstrap_pf_ci(&returns, 500, 7);
        let second = bootstrap_pf_ci(&returns, 500, 7);
        assert_eq!(first, second);
        assert!(first.0 <= first.1);
        assert!(first.0.is_finite());
    }

    #[test]
    fn partition_identities_hold() {
        let n = 400;
        let close: Vec<f64> = (0..n)
            .map(|i| 100.0 * (1.0 + 0.0005 * i as f64 + 0.01 * ((i as f64) * 0.7).sin()))
            .collect();
        let b_day: Vec<f64> = (0..n)
            .map(|i| {
                if i == 0 {
                    0.0
                } else {
                    close[i] / close[i - 1] - 1.0
                }
            })
            .collect();
        let mat = vec![close.clone()];
        let book = Book {
            close_ff: mat.clone(),
            entry_mat: mat.clone(),
            exit_mat: mat,
            cost: 0.0005,
            symbol_costs: vec![0.0005],
            n_rows: n,
            b_day,
            slots: 1,
            periods_per_year: 252,
        };
        let trades = vec![
            trade(10, 15, close[10], close[15]),
            trade(100, 104, close[100], close[104]),
            trade(300, 310, close[300], close[310]),
        ];
        let all = vec![true; n];
        let masks: Masks = vec![
            (
                "dev".into(),
                (0..n).map(|i| (5..200).contains(&i)).collect(),
            ),
            ("test".into(), (0..n).map(|i| i >= 200).collect()),
        ];
        let cand: Mask = vec![vec![true; n]];
        let res = evaluate_fill(&trades, &book, &masks, &cand, &cand, 2.0, 50, 200, 1, "");
        assert_eq!(all.len(), n);
        for (name, row, _) in &res.by_period {
            let (total, trend, bias, skill) = (
                row.get("total_log"),
                row.get("trend"),
                row.get("bias"),
                row.get("skill"),
            );
            assert!((skill - (total - trend - bias)).abs() < 1e-12, "{name}");
            // one symbol: random pick is the real book, so skill_vs_pool == -bias
            assert!((row.get("skill_vs_pool") + bias).abs() < 1e-12);
        }
        assert_eq!(res.by_period[1].1.get("bias"), 0.0);
        assert!(res.by_period[0].1.get("bias") > 0.0);
        assert_eq!(res.by_period[0].1.get("trades"), 2.0);
    }
}
