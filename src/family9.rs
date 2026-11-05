//! Family 9 trend and pullback systems (stocks).

use crate::config::Config;
use crate::data::{self, Panel};
use crate::engine::{self, Trade};
use crate::signals::{self, Mask, Signals};
use std::collections::BTreeSet;

pub const S090_NAME: &str = "s090_kama_breakout";
pub const S090_TITLE: &str = "S090 KAMA Breakout";

/// Kaufman's Adaptive Moving Average (er_len, fast, slow); NaN during warm-up
/// and reset across non-finite close gaps.
fn kama(close: &[f64], er_len: usize, fast: usize, slow: usize) -> Vec<f64> {
    let n = close.len();
    let mut out = vec![f64::NAN; n];
    if er_len == 0 || fast == 0 || slow == 0 || n == 0 {
        return out;
    }
    let fast_sc = 2.0 / (fast as f64 + 1.0);
    let slow_sc = 2.0 / (slow as f64 + 1.0);
    let mut diff = vec![f64::NAN; n];
    for t in 1..n {
        if close[t].is_finite() && close[t - 1].is_finite() {
            diff[t] = (close[t] - close[t - 1]).abs();
        }
    }
    let (mut state, mut seeded) = (f64::NAN, false);
    for t in er_len..n {
        if !close[t].is_finite() {
            seeded = false;
            continue;
        }
        let window = &diff[t + 1 - er_len..=t];
        if !close[t - er_len].is_finite() || window.iter().any(|d| !d.is_finite()) {
            seeded = false;
            continue;
        }
        let volatility: f64 = window.iter().sum();
        let er = if volatility > 0.0 {
            (close[t] - close[t - er_len]).abs() / volatility
        } else {
            0.0
        };
        let sc = (er * (fast_sc - slow_sc) + slow_sc).powi(2);
        state = if seeded {
            state + sc * (close[t] - state)
        } else {
            seeded = true;
            close[t]
        };
        out[t] = state;
    }
    out
}

/// Annualized standard deviation of daily log returns over the trailing window.
fn realized_vol(close: &[f64], window: usize) -> Vec<f64> {
    let n = close.len();
    let mut out = vec![f64::NAN; n];
    if window < 2 || n == 0 {
        return out;
    }
    let mut lr = vec![f64::NAN; n];
    for t in 1..n {
        if close[t].is_finite() && close[t - 1].is_finite() && close[t - 1] > 0.0 {
            lr[t] = (close[t] / close[t - 1]).ln();
        }
    }
    for t in window..n {
        let w = &lr[t + 1 - window..=t];
        if w.iter().all(|v| v.is_finite()) {
            let mean = w.iter().sum::<f64>() / window as f64;
            let var = w.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / (window - 1) as f64;
            out[t] = var.sqrt() * 252f64.sqrt();
        }
    }
    out
}

/// Optional entry gates shared by the long and short sides (dir = +1 long / -1 short).
struct S090Gates {
    sma: Option<Vec<f64>>,
    rv: Option<Vec<f64>>,
    rv_med: Option<Vec<f64>>,
}

fn s090_gates(close: &[f64], cfg: &Config) -> S090Gates {
    let sma = (cfg.kama_gate == "sma").then(|| signals::rolling_mean(close, cfg.sma_window));
    let rv = (cfg.rv_gate != "none").then(|| realized_vol(close, cfg.vol_window));
    let rv_med = (cfg.rv_gate == "quiet")
        .then(|| signals::rolling_median(rv.as_ref().unwrap(), cfg.rv_quiet_lookback));
    S090Gates { sma, rv, rv_med }
}

fn s090_gate_ok(g: &S090Gates, cfg: &Config, close_px: f64, day: usize, dir: f64) -> bool {
    if let Some(s) = &g.sma {
        if !(s[day].is_finite() && dir * (close_px - s[day]) > 0.0) {
            return false;
        }
    }
    if let Some(r) = &g.rv {
        match cfg.rv_gate.as_str() {
            "cap" => {
                if !(r[day].is_finite() && r[day] <= cfg.rv_max) {
                    return false;
                }
            }
            "quiet" => {
                let m = g.rv_med.as_ref().unwrap();
                if !(r[day].is_finite() && m[day].is_finite() && r[day] <= m[day]) {
                    return false;
                }
            }
            _ => {}
        }
    }
    true
}

/// Mirrored short-side masks (entry: close < KAMA and KAMA flat-to-falling;
/// exit: close > KAMA). Recomputed from the panel so `Signals` stays unchanged.
fn s090_short_masks(panel: &Panel, cfg: &Config, tradable: &Mask) -> (Mask, Mask) {
    let n_rows = panel.dates.len();
    let mut entry = vec![vec![false; n_rows]; panel.symbols.len()];
    let mut exit = vec![vec![false; n_rows]; panel.symbols.len()];
    let mirror_floor = if cfg.kama_slope_floor > 0.0 {
        1.0 / cfg.kama_slope_floor
    } else {
        0.0
    };
    for j in 0..panel.symbols.len() {
        let close = &panel.close[j];
        let k = kama(close, cfg.kama_er_len, cfg.kama_fast, cfg.kama_slow);
        let gates = s090_gates(close, cfg);
        for day in 0..n_rows {
            if close[day].is_finite() && k[day].is_finite() && close[day] > k[day] {
                exit[j][day] = true;
            }
            if !tradable[j][day] || !k[day].is_finite() || !close[day].is_finite() {
                continue;
            }
            let slope_ok = mirror_floor == 0.0
                || (day >= cfg.kama_slope_lookback
                    && k[day - cfg.kama_slope_lookback].is_finite()
                    && k[day] < k[day - cfg.kama_slope_lookback] * mirror_floor);
            entry[j][day] = close[day] < k[day]
                && slope_ok
                && s090_gate_ok(&gates, cfg, close[day], day, -1.0);
        }
    }
    (entry, exit)
}

pub fn compute_s090_signals(panel: &Panel, cfg: &Config) -> Result<Signals, String> {
    if cfg.kama_er_len == 0 || cfg.kama_fast == 0 || cfg.kama_slow == 0 {
        return Err("S090 KAMA lengths must be greater than zero".into());
    }
    if cfg.kama_slope_lookback == 0 {
        return Err("S090 kama_slope_lookback must be greater than zero".into());
    }
    if !cfg.kama_slope_floor.is_finite() || cfg.kama_slope_floor < 0.0 {
        return Err("S090 kama_slope_floor must be finite and non-negative".into());
    }
    if !matches!(cfg.kama_side.as_str(), "long" | "short" | "long_short") {
        return Err(format!("unknown S090 kama_side {:?}", cfg.kama_side));
    }
    if !matches!(cfg.kama_gate.as_str(), "none" | "sma") {
        return Err(format!("unknown S090 kama_gate {:?}", cfg.kama_gate));
    }
    if !matches!(cfg.rv_gate.as_str(), "none" | "cap" | "quiet") {
        return Err(format!("unknown S090 rv_gate {:?}", cfg.rv_gate));
    }
    if cfg.rv_gate == "cap" && (!cfg.rv_max.is_finite() || cfg.rv_max <= 0.0) {
        return Err("S090 rv_gate=cap requires a positive rv_max".into());
    }
    if cfg.rv_gate == "quiet" && cfg.rv_quiet_lookback == 0 {
        return Err("S090 rv_gate=quiet requires rv_quiet_lookback > 0".into());
    }
    if cfg.rv_gate != "none" && cfg.vol_window < 2 {
        return Err("S090 rv gate requires vol_window >= 2".into());
    }
    let etf_symbols: BTreeSet<String> =
        crate::config::split_csv(&cfg.etf_symbols).into_iter().collect();
    let stock_columns: Vec<_> = (0..panel.symbols.len())
        .filter(|&column| !etf_symbols.contains(&panel.symbols[column]))
        .collect();
    if !cfg.include_stocks && stock_columns.len() == panel.symbols.len() {
        return Err("S090 etf_only mode requires at least one etf_symbols entry".into());
    }
    let n_rows = panel.dates.len();
    let ranks = if cfg.include_stocks && !stock_columns.is_empty() {
        if cfg.rank_cache_dir.trim().is_empty() {
            data::liquidity_ranks(
                panel,
                &stock_columns,
                cfg.rank_window,
                cfg.rank_min_observations,
                cfg.min_price,
                cfg.min_history,
            )?
        } else {
            data::read_liquid_rank_cache(&cfg.rank_cache_dir, &panel.dates, &panel.symbols)?
        }
    } else {
        vec![vec![f64::NAN; n_rows]; panel.symbols.len()]
    };
    let mut result = crate::family4::blank_signals(panel, "kama_breakout_er");
    for j in 0..panel.symbols.len() {
        let close = &panel.close[j];
        let k = kama(close, cfg.kama_er_len, cfg.kama_fast, cfg.kama_slow);
        let gates = s090_gates(close, cfg);
        let is_etf = etf_symbols.contains(&panel.symbols[j]);
        let mut history = 0usize;
        for day in 0..n_rows {
            if close[day].is_finite() {
                history += 1;
                if k[day].is_finite() {
                    result.exit_now[j][day] = close[day] < k[day];
                }
            }
            let in_universe = is_etf
                || (cfg.include_stocks
                    && ranks[j][day].is_finite()
                    && ranks[j][day] <= cfg.universe_top_n as f64);
            if !close[day].is_finite() || history < cfg.min_history || !in_universe {
                continue;
            }
            result.tradable[j][day] = true;
            result.pool[j][day] = true;
            if !k[day].is_finite() || !s090_gate_ok(&gates, cfg, close[day], day, 1.0) {
                continue;
            }
            result.rsi[j][day] = close[day] / k[day] - 1.0;
            let slope_ok = cfg.kama_slope_floor <= 0.0
                || (day >= cfg.kama_slope_lookback
                    && k[day - cfg.kama_slope_lookback].is_finite()
                    && k[day] > k[day - cfg.kama_slope_lookback] * cfg.kama_slope_floor);
            result.signal[j][day] = close[day] > k[day] && slope_ok;
        }
    }
    if cfg.disaster_stop_atr > 0.0 {
        result.atr = Some(
            panel
                .close
                .iter()
                .enumerate()
                .map(|(j, close)| {
                    signals::wilder_atr(&panel.high[j], &panel.low[j], close, cfg.atr_len)
                })
                .collect(),
        );
    }
    result.benchmark_returns = crate::family4::benchmark_returns(panel, "SPY");
    Ok(result)
}

/// Greedy concurrency cap for the weighted long/short book: keep the first
/// `slots` overlapping legs by (entry, symbol) order and skip the rest.
fn s090_cap_concurrent(mut trades: Vec<Trade>, slots: usize) -> Vec<Trade> {
    trades.sort_by(|a, b| {
        (a.entry_idx, a.sym_idx, a.exit_idx).cmp(&(b.entry_idx, b.sym_idx, b.exit_idx))
    });
    let mut open: Vec<usize> = Vec::new();
    let mut kept = Vec::with_capacity(trades.len());
    for trade in trades {
        open.retain(|&exit| exit >= trade.entry_idx);
        if open.len() >= slots {
            continue;
        }
        open.push(trade.exit_idx);
        kept.push(trade);
    }
    kept
}

pub fn s090_trades(
    panel: &Panel,
    sig: &Signals,
    cfg: &Config,
    fill: &str,
) -> Result<Vec<Trade>, String> {
    if !matches!(fill, "next_open" | "close") {
        return Err(format!("S090 does not support fill {fill:?}"));
    }
    let is_open = fill == "next_open";
    let n_rows = panel.dates.len();
    let last = engine::last_valid_rows(&panel.close);
    let weighted = cfg.kama_side != "long";
    let slot_weight = 1.0 / cfg.book_slots.max(1) as f64;
    let borrow_bar = cfg.borrow_bps_annual / 10_000.0 / cfg.periods_per_year as f64;
    let dirs: &[f64] = match cfg.kama_side.as_str() {
        "long" => &[1.0],
        "short" => &[-1.0],
        _ => &[1.0, -1.0],
    };
    let short = weighted.then(|| s090_short_masks(panel, cfg, &sig.tradable));
    let mut trades = Vec::new();
    for j in 0..panel.symbols.len() {
        let Some(lj) = last[j] else { continue };
        let cost = cfg.cost_for_symbol(&panel.symbols[j]);
        for &dir in dirs {
            let (entries, exits): (&Mask, &Mask) = if dir > 0.0 {
                (&sig.signal, &sig.exit_now)
            } else {
                let short = short.as_ref().unwrap();
                (&short.0, &short.1)
            };
            let mut t_prev: Option<usize> = None;
            for t in 0..n_rows {
                if !entries[j][t] || matches!(t_prev, Some(tp) if t <= tp) {
                    continue;
                }
                let entry = if is_open { t + 1 } else { t };
                if entry > lj || (is_open && !panel.open[j][entry].is_finite()) {
                    continue;
                }
                let entry_px = if is_open {
                    panel.open[j][entry]
                } else {
                    panel.close[j][t]
                };
                if !entry_px.is_finite() || entry_px <= 0.0 {
                    continue;
                }
                let stop_level = sig.atr.as_ref().and_then(|atr| {
                    let v = atr[j][t];
                    v.is_finite()
                        .then_some(entry_px - dir * cfg.disaster_stop_atr * v)
                });
                let (mut reason, mut u_exit) = ("signal", None);
                let mut held = 0usize;
                for u in (t + 1)..=lj {
                    if !panel.close[j][u].is_finite() {
                        continue;
                    }
                    held += 1;
                    if let Some(s) = stop_level {
                        let hit = if dir > 0.0 {
                            panel.close[j][u] < s
                        } else {
                            panel.close[j][u] > s
                        };
                        if hit {
                            reason = "stop";
                            u_exit = Some(u);
                            break;
                        }
                    }
                    if exits[j][u] {
                        u_exit = Some(u);
                        break;
                    }
                    if cfg.max_hold > 0 && held >= cfg.max_hold {
                        reason = "time";
                        u_exit = Some(u);
                        break;
                    }
                }
                let u_exit = match u_exit {
                    Some(u) => u,
                    None => {
                        reason = if lj < n_rows - 1 {
                            "delisted"
                        } else {
                            "censored"
                        };
                        lj
                    }
                };
                let (mut exit_idx, mut exit_px) = (u_exit, panel.close[j][u_exit]);
                if is_open
                    && reason != "delisted"
                    && reason != "censored"
                    && u_exit + 1 <= lj
                    && panel.open[j][u_exit + 1].is_finite()
                {
                    exit_idx = u_exit + 1;
                    exit_px = panel.open[j][exit_idx];
                }
                if reason == "delisted" && dir > 0.0 && cfg.delisting_return != 0.0 {
                    exit_px *= 1.0 + cfg.delisting_return;
                }
                if exit_idx < entry || !exit_px.is_finite() {
                    continue;
                }
                let gross = exit_px / entry_px - 1.0;
                let weight = if weighted { dir * slot_weight } else { 1.0 };
                let borrow_total = if dir < 0.0 {
                    borrow_bar * (exit_idx - entry + 1) as f64
                } else {
                    0.0
                };
                let net = if weighted {
                    dir * gross - 2.0 * cost - borrow_total
                } else {
                    exit_px * (1.0 - cost) / (entry_px * (1.0 + cost)) - 1.0
                };
                trades.push(Trade {
                    sym_idx: j,
                    signal_idx: t,
                    entry_idx: entry,
                    exit_idx,
                    entry_px,
                    exit_px,
                    ret: net,
                    ret_gross: dir * gross,
                    exit_reason: reason,
                    hold: exit_idx - entry,
                    signal_value: sig.rsi[j][t],
                    weight,
                    entry_turnover: if weighted { weight.abs() } else { 1.0 },
                    exit_turnover: if weighted { weight.abs() } else { 1.0 },
                    weighted,
                    borrow_cost_per_bar: if dir < 0.0 { borrow_bar } else { 0.0 },
                });
                t_prev = Some(u_exit);
            }
        }
    }
    if weighted {
        trades = s090_cap_concurrent(trades, cfg.book_slots.max(1));
    }
    trades.retain(|trade| trade.hold > 0);
    trades.sort_by(|a, b| {
        (a.entry_idx, &panel.symbols[a.sym_idx]).cmp(&(b.entry_idx, &panel.symbols[b.sym_idx]))
    });
    Ok(trades)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kama_tracks_flat_and_trending_prices() {
        let mut close = vec![100.0; 40];
        close.extend((41..=80).map(|i| 100.0 + (i - 40) as f64));
        let k = kama(&close, 10, 2, 30);
        assert!(k[..10].iter().all(|v| v.is_nan()));
        assert!((k[39] - 100.0).abs() < 1e-9);
        assert!(k[79] > k[60] && k[79] > 100.0);
    }

    #[test]
    fn kama_resets_after_close_gap() {
        let mut close = vec![10.0; 30];
        close[15] = f64::NAN;
        let k = kama(&close, 10, 2, 30);
        assert!(k[14].is_finite());
        for value in &k[15..26] {
            assert!(value.is_nan());
        }
        assert!(k[29].is_finite());
    }
}
