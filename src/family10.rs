//! Family 10 candlestick and bar pattern studies.

use crate::config::Config;
use crate::data::{self, Panel};
use crate::engine::Trade;
use crate::signals::{self, Signals};

pub const S097_NAME: &str = "s097_bullish_engulfing";
pub const S097_TITLE: &str = "S097 Bullish Engulfing After Down-Streak";

/// S097: bullish engulfing after `down_streak`+ consecutive down closes, gated by SMA(200).
/// Entry at the close of the engulfing session (close fill) or next open.
/// Exit when close > SMA(`exit_sma`), or at the first up close (`exit_mode=up_close`),
/// or at the hold cap (`exit_mode=time`); hard cap `max_hold` sessions.
pub fn compute_s097_signals(panel: &Panel, cfg: &Config) -> Result<Signals, String> {
    if !matches!(cfg.engulf_mode.as_str(), "strict" | "loose")
        || !matches!(cfg.exit_mode.as_str(), "sma" | "up_close" | "time")
    {
        return Err("S097 has an unsupported engulf or exit mode".into());
    }
    if cfg.down_streak == 0
        || cfg.universe_top_n == 0
        || cfg.max_hold == 0
        || (cfg.exit_mode == "sma" && cfg.exit_sma == 0)
    {
        return Err("S097 has invalid streak, universe, exit, or hold settings".into());
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
    let n_rows = panel.dates.len();
    let n_symbols = panel.symbols.len();
    let mut result = crate::family4::blank_signals(panel, "bullish_engulfing");
    let mut returns = vec![vec![f64::NAN; n_rows]; n_symbols];
    for symbol in 0..n_symbols {
        let (open, close) = (&panel.open[symbol], &panel.close[symbol]);
        let sma_gate = if cfg.sma_window > 0 {
            signals::rolling_mean(close, cfg.sma_window)
        } else {
            vec![f64::NAN; n_rows]
        };
        let sma_exit = signals::rolling_mean(close, cfg.exit_sma.max(1));
        let mut streak = 0usize;
        for day in 0..n_rows {
            let prev_streak = streak;
            if day > 0
                && close[day].is_finite()
                && close[day - 1].is_finite()
                && close[day] < close[day - 1]
            {
                streak += 1;
            } else {
                streak = 0;
            }
            if day > 0 && close[day].is_finite() && close[day - 1].is_finite() {
                returns[symbol][day] = close[day] / close[day - 1] - 1.0;
            }
            let liquid = ranks[symbol][day].is_finite()
                && ranks[symbol][day] <= cfg.universe_top_n as f64;
            if !close[day].is_finite() || !liquid {
                continue;
            }
            result.tradable[symbol][day] = true;
            result.pool[symbol][day] = true;
            result.exit_now[symbol][day] = match cfg.exit_mode.as_str() {
                "sma" => sma_exit[day].is_finite() && close[day] > sma_exit[day],
                "up_close" => {
                    day > 0 && close[day - 1].is_finite() && close[day] > close[day - 1]
                }
                _ => false,
            };
            if day == 0
                || prev_streak < cfg.down_streak
                || !open[day].is_finite()
                || !open[day - 1].is_finite()
                || !close[day - 1].is_finite()
                || close[day - 1] >= open[day - 1]
                || close[day] <= open[day]
            {
                continue;
            }
            let engulfed = match cfg.engulf_mode.as_str() {
                "strict" => open[day] < close[day - 1] && close[day] > open[day - 1],
                _ => open[day] < open[day - 1] && close[day] > close[day - 1],
            };
            let gate_ok =
                cfg.sma_window == 0 || (sma_gate[day].is_finite() && close[day] > sma_gate[day]);
            if engulfed && gate_ok {
                result.signal[symbol][day] = true;
                let anchor = day.saturating_sub(prev_streak + 1);
                if close[anchor].is_finite() && close[anchor] > 0.0 {
                    result.rsi[symbol][day] = close[day - 1] / close[anchor] - 1.0;
                }
            }
        }
    }
    let mut b_day = vec![0.0; n_rows];
    for day in 0..n_rows {
        let (mut sum, mut count) = (0.0, 0usize);
        for symbol in 0..n_symbols {
            if result.tradable[symbol][day] && returns[symbol][day].is_finite() {
                sum += returns[symbol][day];
                count += 1;
            }
        }
        if count > 0 {
            b_day[day] = sum / count as f64;
        }
    }
    result.benchmark_returns = Some(b_day);
    Ok(result)
}

pub fn s097_trades(
    panel: &Panel,
    sig: &Signals,
    cfg: &Config,
    fill: &str,
) -> Result<Vec<Trade>, String> {
    if !matches!(fill, "close" | "next_open") {
        return Err(format!("S097 does not support fill {fill:?}"));
    }
    let n_rows = panel.dates.len();
    let is_open = fill == "next_open";
    let mut trades = Vec::new();
    for symbol in 0..panel.symbols.len() {
        let close = &panel.close[symbol];
        let open = &panel.open[symbol];
        let Some(last) = close.iter().rposition(|value| value.is_finite()) else {
            continue;
        };
        let cost = cfg.cost_for_symbol(&panel.symbols[symbol]);
        let mut previous_decision = None;
        for day in 0..n_rows {
            if !sig.signal[symbol][day]
                || matches!(previous_decision, Some(previous) if day <= previous)
            {
                continue;
            }
            let entry = if is_open { day + 1 } else { day };
            if entry > last || (is_open && !open[entry].is_finite()) {
                continue;
            }
            let entry_px = if is_open { open[entry] } else { close[day] };
            if !entry_px.is_finite() || entry_px <= 0.0 {
                continue;
            }
            let mut decision = day.saturating_add(cfg.max_hold).min(last);
            let mut reason = if day + cfg.max_hold > last {
                if last + 1 < n_rows {
                    "delisted"
                } else {
                    "censored"
                }
            } else {
                "cap"
            };
            for holding_day in (day + 1)..=day.saturating_add(cfg.max_hold).min(last) {
                if !close[holding_day].is_finite() {
                    continue;
                }
                if cfg.stop_loss > 0.0 && close[holding_day] <= entry_px * (1.0 - cfg.stop_loss)
                {
                    decision = holding_day;
                    reason = "stop";
                    break;
                }
                if cfg.take_profit > 0.0
                    && close[holding_day] >= entry_px * (1.0 + cfg.take_profit)
                {
                    decision = holding_day;
                    reason = "target";
                    break;
                }
                if sig.exit_now[symbol][holding_day] {
                    decision = holding_day;
                    reason = match cfg.exit_mode.as_str() {
                        "up_close" => "up_close",
                        _ => "sma_exit",
                    };
                    break;
                }
            }
            previous_decision = Some(decision);
            let mut exit = decision;
            while exit > day && !close[exit].is_finite() {
                exit -= 1;
            }
            let mut exit_px = close[exit];
            if is_open
                && reason != "delisted"
                && reason != "censored"
                && exit + 1 <= last
                && open[exit + 1].is_finite()
            {
                exit += 1;
                exit_px = open[exit];
            }
            if reason == "delisted" {
                exit_px *= 1.0 + cfg.delisting_return;
            }
            if !exit_px.is_finite() || exit < entry {
                continue;
            }
            trades.push(Trade {
                sym_idx: symbol,
                signal_idx: day,
                entry_idx: entry,
                exit_idx: exit,
                entry_px,
                exit_px,
                ret: exit_px * (1.0 - cost) / (entry_px * (1.0 + cost)) - 1.0,
                ret_gross: exit_px / entry_px - 1.0,
                exit_reason: reason,
                hold: exit - entry,
                signal_value: sig.rsi[symbol][day],
                weight: 1.0,
                entry_turnover: 1.0,
                exit_turnover: 1.0,
                weighted: false,
                borrow_cost_per_bar: 0.0,
            });
        }
    }
    trades.sort_by(|a, b| {
        (a.entry_idx, &panel.symbols[a.sym_idx]).cmp(&(b.entry_idx, &panel.symbols[b.sym_idx]))
    });
    Ok(trades)
}
