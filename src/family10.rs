//! Family 10 candlestick and bar pattern studies.

use crate::config::Config;
use crate::data::{self, Panel};
use crate::engine::{self, Trade};
use crate::signals::{self, Signals};

pub const S099_NAME: &str = "s099_outside_day_reversal";
pub const S099_TITLE: &str = "S099 Outside-Day Key Reversal at 200-SMA";

/// S099: outside day (higher high, lower low) closing green against a prior
/// down-close, with close within `sma_distance` of SMA(`sma_window`).
/// `rsi` carries the signal day's low (the reversal low) for the exit rule.
pub fn compute_s099_signals(panel: &Panel, cfg: &Config) -> Result<Signals, String> {
    if cfg.sma_window == 0
        || cfg.exit_sma == 0
        || cfg.max_hold == 0
        || cfg.universe_top_n == 0
        || cfg.min_history == 0
    {
        return Err(
            "S099 requires positive sma_window, exit_sma, max_hold, universe_top_n, min_history"
                .into(),
        );
    }
    if !cfg.sma_distance.is_finite() || cfg.sma_distance < 0.0 {
        return Err("S099 sma_distance must be finite and >= 0".into());
    }
    if !matches!(cfg.sma_side.as_str(), "any" | "below" | "above") {
        return Err(format!("unknown S099 sma_side {:?}", cfg.sma_side));
    }
    if !cfg.rel_vol_min.is_finite() || cfg.rel_vol_min < 0.0 {
        return Err("S099 rel_vol_min must be finite and >= 0".into());
    }
    if !cfg.min_range_pct.is_finite() || cfg.min_range_pct < 0.0 {
        return Err("S099 min_range_pct must be finite and >= 0".into());
    }
    let ranks = if cfg.rank_cache_dir.trim().is_empty() {
        let columns: Vec<_> = (0..panel.symbols.len()).collect();
        data::liquidity_ranks(
            panel,
            &columns,
            cfg.rank_window,
            cfg.rank_min_observations,
            cfg.min_price,
            cfg.min_history,
        )?
    } else {
        data::read_liquid_rank_cache(&cfg.rank_cache_dir, &panel.dates, &panel.symbols)?
    };
    let mut result = crate::family4::blank_signals(panel, "outside_day_reversal");
    for symbol in 0..panel.symbols.len() {
        let (open, high, low, close, volume) = (
            &panel.open[symbol],
            &panel.high[symbol],
            &panel.low[symbol],
            &panel.close[symbol],
            &panel.volume[symbol],
        );
        let sma = signals::rolling_mean(close, cfg.sma_window);
        let vol_sma = (cfg.rel_vol_min > 0.0)
            .then(|| signals::rolling_mean(volume, cfg.vol_window));
        let mut history = 0usize;
        for day in 0..panel.dates.len() {
            if close[day].is_finite() {
                history += 1;
            }
            let liquid =
                ranks[symbol][day].is_finite() && ranks[symbol][day] <= cfg.universe_top_n as f64;
            if !close[day].is_finite() || !liquid || history < cfg.min_history {
                continue;
            }
            result.tradable[symbol][day] = true;
            result.pool[symbol][day] = true;
            if day == 0
                || !open[day].is_finite()
                || !high[day].is_finite()
                || !low[day].is_finite()
                || !open[day - 1].is_finite()
                || !high[day - 1].is_finite()
                || !low[day - 1].is_finite()
                || !close[day - 1].is_finite()
                || !sma[day].is_finite()
                || close[day] <= 0.0
            {
                continue;
            }
            if cfg.require_outside && !(high[day] > high[day - 1] && low[day] < low[day - 1]) {
                continue;
            }
            if cfg.require_green_close && close[day] <= open[day] {
                continue;
            }
            if cfg.require_prior_down && close[day - 1] >= open[day - 1] {
                continue;
            }
            if (close[day] - sma[day]).abs() / close[day] >= cfg.sma_distance {
                continue;
            }
            match cfg.sma_side.as_str() {
                "below" if close[day] > sma[day] => continue,
                "above" if close[day] < sma[day] => continue,
                _ => {}
            }
            if let Some(vol_mean) = &vol_sma {
                if !vol_mean[day].is_finite()
                    || !volume[day].is_finite()
                    || volume[day] < cfg.rel_vol_min * vol_mean[day]
                {
                    continue;
                }
            }
            if cfg.min_range_pct > 0.0
                && (high[day] - low[day]) / close[day] < cfg.min_range_pct
            {
                continue;
            }
            result.signal[symbol][day] = true;
            result.rsi[symbol][day] = low[day];
        }
    }
    result.benchmark_returns = crate::family4::benchmark_returns(panel, "SPY");
    Ok(result)
}

/// fill="close": enter at the signal close (upper bound); fill="next_open":
/// enter at the next session's open. Exit on the first completed session whose
/// close breaks the signal-day low or rises above SMA(`exit_sma`), else after
/// `max_hold` sessions; close-fill exits at that close, next_open at the next open.
pub fn s099_trades(
    panel: &Panel,
    sig: &Signals,
    cfg: &Config,
    fill: &str,
) -> Result<Vec<Trade>, String> {
    if !matches!(fill, "close" | "next_open") {
        return Err(format!("S099 does not support fill {fill:?}"));
    }
    let last = engine::last_valid_rows(&panel.close);
    let mut trades = Vec::new();
    for symbol in 0..panel.symbols.len() {
        let cost = cfg.cost_for_symbol(&panel.symbols[symbol]);
        let sma_exit = signals::rolling_mean(&panel.close[symbol], cfg.exit_sma);
        let Some(last_valid) = last[symbol] else { continue };
        let mut available_after = 0usize;
        for signal_day in 0..panel.dates.len() {
            if signal_day < available_after || !sig.signal[symbol][signal_day] {
                continue;
            }
            let entry = if fill == "next_open" {
                signal_day.saturating_add(1)
            } else {
                signal_day
            };
            if entry > last_valid {
                continue;
            }
            let entry_px = if fill == "next_open" {
                panel.open[symbol][entry]
            } else {
                panel.close[symbol][entry]
            };
            let reversal_low = sig.rsi[symbol][signal_day];
            if !entry_px.is_finite() || entry_px <= 0.0 || !reversal_low.is_finite() {
                continue;
            }
            let final_session = (entry + cfg.max_hold - 1).min(last_valid);
            let delisted = last_valid < panel.dates.len() - 1;
            let mut exit_idx = final_session;
            let mut exit_px = panel.close[symbol][final_session];
            let mut reason = if delisted { "delisted" } else { "time" };
            for day in entry + 1..=final_session {
                let close = panel.close[symbol][day];
                if !close.is_finite() {
                    continue;
                }
                let exit_day = if fill == "next_open" { day + 1 } else { day };
                if exit_day > final_session {
                    continue;
                }
                let candidate_px = if fill == "next_open" {
                    panel.open[symbol][exit_day]
                } else {
                    close
                };
                if !candidate_px.is_finite() {
                    continue;
                }
                if close < reversal_low {
                    exit_idx = exit_day;
                    exit_px = candidate_px;
                    reason = "reversal_low";
                    break;
                }
                if sma_exit[day].is_finite() && close > sma_exit[day] {
                    exit_idx = exit_day;
                    exit_px = candidate_px;
                    reason = "sma_exit";
                    break;
                }
            }
            if reason == "delisted" && cfg.delisting_return != 0.0 {
                exit_px *= 1.0 + cfg.delisting_return;
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
                signal_value: reversal_low,
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
