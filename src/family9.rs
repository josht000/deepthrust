//! Family 9 trend and pullback systems (stocks).

use crate::config::Config;
use crate::data::{self, Panel};
use crate::engine::{self, Trade};
use crate::family4::blank_signals;
use crate::signals::{self, Signals};

pub const S095_NAME: &str = "s095_keltner_breakout";
pub const S095_TITLE: &str = "S095 Keltner Channel Breakout";

/// TA-Lib-style EMA(n): seeded by the SMA of the first full finite window of n.
/// Once seeded, non-finite inputs emit NaN but keep the state.
fn ema(x: &[f64], n: usize) -> Vec<f64> {
    let mut out = vec![f64::NAN; x.len()];
    if n == 0 {
        return out;
    }
    let k = 2.0 / (n as f64 + 1.0);
    let (mut run, mut sum, mut state) = (0usize, 0.0, f64::NAN);
    for (i, &v) in x.iter().enumerate() {
        if !v.is_finite() {
            if state.is_nan() {
                run = 0;
                sum = 0.0;
            }
            continue;
        }
        if state.is_nan() {
            run += 1;
            sum += v;
            if run == n {
                state = sum / n as f64;
                out[i] = state;
            }
        } else {
            state = k * v + (1.0 - k) * state;
            out[i] = state;
        }
    }
    out
}

// S095: close > EMA(ema_len) + keltner_mult x ATR(atr_len) on volume >
// volume_mult x the prior volume_median_window median; exit close < EMA or
// after max_hold sessions. Long-only (catalog shared spec: long first).
pub fn compute_s095_signals(panel: &Panel, cfg: &Config) -> Result<Signals, String> {
    if cfg.ema_len == 0 || cfg.atr_len == 0 || cfg.volume_median_window == 0 || cfg.max_hold == 0 {
        return Err("S095 lookback windows and max hold must be greater than zero".into());
    }
    if !cfg.keltner_mult.is_finite() || cfg.keltner_mult <= 0.0 {
        return Err("S095 keltner_mult must be positive".into());
    }
    if !cfg.volume_mult.is_finite() || cfg.volume_mult < 0.0 {
        return Err("S095 volume_mult must not be negative".into());
    }
    let etf_set: std::collections::BTreeSet<String> =
        crate::config::split_csv(&cfg.etf_symbols).into_iter().collect();
    let stock_columns: Vec<_> = (0..panel.symbols.len())
        .filter(|column| !etf_set.contains(&panel.symbols[*column]))
        .collect();
    let ranks = if !cfg.include_stocks || stock_columns.is_empty() {
        vec![vec![f64::NAN; panel.dates.len()]; panel.symbols.len()]
    } else if cfg.rank_cache_dir.trim().is_empty() {
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
    };
    let n_rows = panel.dates.len();
    let mut result = blank_signals(panel, "keltner_band_distance_atr");
    for symbol in 0..panel.symbols.len() {
        let close = &panel.close[symbol];
        let mid = ema(close, cfg.ema_len);
        let atr = signals::wilder_atr(&panel.high[symbol], &panel.low[symbol], close, cfg.atr_len);
        let median_volume = signals::rolling_median(&panel.volume[symbol], cfg.volume_median_window);
        let is_etf = etf_set.contains(&panel.symbols[symbol]);
        let mut history = 0usize;
        for day in 0..n_rows {
            if close[day].is_finite() {
                history += 1;
            }
            if !close[day].is_finite() {
                continue;
            }
            if mid[day].is_finite() {
                result.exit_now[symbol][day] = close[day] < mid[day];
            }
            if history < cfg.min_history {
                continue;
            }
            let in_universe = if is_etf {
                cfg.include_etfs
            } else {
                cfg.include_stocks
                    && ranks[symbol][day].is_finite()
                    && ranks[symbol][day] <= cfg.universe_top_n as f64
            };
            if !in_universe {
                continue;
            }
            result.tradable[symbol][day] = true;
            result.pool[symbol][day] = true;
            let prior_median = if day > 0 {
                median_volume[day - 1]
            } else {
                f64::NAN
            };
            if !mid[day].is_finite()
                || !atr[day].is_finite()
                || !prior_median.is_finite()
                || prior_median <= 0.0
            {
                continue;
            }
            let upper = mid[day] + cfg.keltner_mult * atr[day];
            if close[day] > upper && panel.volume[symbol][day] > cfg.volume_mult * prior_median {
                result.signal[symbol][day] = true;
                result.rsi[symbol][day] = (close[day] - upper) / atr[day];
            }
        }
    }
    Ok(result)
}

pub fn s095_trades(
    panel: &Panel,
    sig: &Signals,
    cfg: &Config,
    fill: &str,
) -> Result<Vec<Trade>, String> {
    if !matches!(fill, "close" | "next_open") {
        return Err(format!("S095 does not support fill {fill:?}"));
    }
    let n_rows = panel.dates.len();
    let last = engine::last_valid_rows(&panel.close);
    let mut trades = Vec::new();
    for symbol in 0..panel.symbols.len() {
        let cost = cfg.cost_for_symbol(&panel.symbols[symbol]);
        let Some(last_row) = last[symbol] else { continue };
        let mut available_after = 0usize;
        for signal_day in 0..n_rows {
            if signal_day < available_after || !sig.signal[symbol][signal_day] {
                continue;
            }
            let entry = if fill == "next_open" {
                signal_day.saturating_add(1)
            } else {
                signal_day
            };
            if entry > last_row {
                continue;
            }
            let entry_px = if fill == "next_open" {
                panel.open[symbol][entry]
            } else {
                panel.close[symbol][entry]
            };
            if !entry_px.is_finite() || entry_px <= 0.0 {
                continue;
            }
            let final_session = (entry + cfg.max_hold - 1).min(last_row);
            let mut exit_idx = final_session;
            let mut exit_px = panel.close[symbol][final_session];
            let mut reason = "time";
            let first_check = if fill == "next_open" {
                entry
            } else {
                entry + 1
            };
            for day in first_check..=final_session {
                if !panel.close[symbol][day].is_finite() || !sig.exit_now[symbol][day] {
                    continue;
                }
                let exit_day = if fill == "next_open" { day + 1 } else { day };
                if exit_day > final_session {
                    break;
                }
                let candidate = if fill == "next_open" {
                    panel.open[symbol][exit_day]
                } else {
                    panel.close[symbol][exit_day]
                };
                if !candidate.is_finite() {
                    continue;
                }
                exit_idx = exit_day;
                exit_px = candidate;
                reason = "signal";
                break;
            }
            if reason == "time" && final_session == last_row && last_row < n_rows - 1 {
                reason = "delisted";
                if cfg.delisting_return != 0.0 {
                    exit_px *= 1.0 + cfg.delisting_return;
                }
            }
            if !exit_px.is_finite() {
                continue;
            }
            let net = exit_px * (1.0 - cost) / (entry_px * (1.0 + cost)) - 1.0;
            trades.push(Trade {
                sym_idx: symbol,
                signal_idx: signal_day,
                entry_idx: entry,
                exit_idx,
                entry_px,
                exit_px,
                ret: net,
                ret_gross: exit_px / entry_px - 1.0,
                exit_reason: reason,
                hold: exit_idx - entry + 1,
                signal_value: sig.rsi[symbol][signal_day],
                weight: 1.0,
                entry_turnover: 1.0,
                exit_turnover: 1.0,
                weighted: false,
                borrow_cost_per_bar: 0.0,
            });
            available_after = exit_idx + 1;
        }
    }
    trades.sort_by(|a, b| {
        (a.entry_idx, &panel.symbols[a.sym_idx]).cmp(&(b.entry_idx, &panel.symbols[b.sym_idx]))
    });
    Ok(trades)
}
