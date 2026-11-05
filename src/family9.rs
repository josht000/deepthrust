//! Family 9 trend and pullback systems (stocks).

use crate::config::Config;
use crate::data::Panel;
use crate::engine::Trade;
use crate::signals::{self, Signals};

pub const S091_NAME: &str = "s091_donchian55_chandelier";
pub const S091_TITLE: &str = "S091 Donchian 55 with Chandelier Trail";

fn rolling_high_before(values: &[f64], day: usize, window: usize) -> f64 {
    if window == 0 || day < window {
        return f64::NAN;
    }
    let sample = &values[day - window..day];
    if sample.iter().any(|value| !value.is_finite()) {
        f64::NAN
    } else {
        sample.iter().copied().fold(f64::NEG_INFINITY, f64::max)
    }
}

fn rolling_low_before(values: &[f64], day: usize, window: usize) -> f64 {
    if window == 0 || day < window {
        return f64::NAN;
    }
    let sample = &values[day - window..day];
    if sample.iter().any(|value| !value.is_finite()) {
        f64::NAN
    } else {
        sample.iter().copied().fold(f64::INFINITY, f64::min)
    }
}

/// S091: close above/below the `entry_window`-day Donchian channel; exit on a
/// close past `extreme_since_entry -/+ turtle_chandelier_mult x ATR(22)`
/// (mode `chandelier`) or the opposite `exit_window`-day channel (mode
/// `channel`), plus an optional N-stop (`disaster_stop_atr`, 0 disables).
pub fn compute_s091_signals(panel: &Panel, cfg: &Config) -> Result<Signals, String> {
    if cfg.entry_window == 0 || cfg.atr_len == 0 || cfg.turtle_chandelier_len == 0 {
        return Err("S091 entry/ATR/chandelier windows must be greater than zero".into());
    }
    if !matches!(
        cfg.strategy_mode.as_str(),
        "long" | "long_only" | "long_short" | "short"
    ) {
        return Err(format!(
            "unknown S091 strategy_mode {:?}",
            cfg.strategy_mode
        ));
    }
    if !matches!(cfg.turtle_exit_mode.as_str(), "chandelier" | "channel") {
        return Err(format!(
            "unknown S091 turtle_exit_mode {:?}",
            cfg.turtle_exit_mode
        ));
    }
    if !matches!(cfg.turtle_filter_outcome.as_str(), "stop" | "pnl") {
        return Err(format!(
            "unknown S091 turtle_filter_outcome {:?}",
            cfg.turtle_filter_outcome
        ));
    }
    if !cfg.disaster_stop_atr.is_finite() || cfg.disaster_stop_atr < 0.0 {
        return Err("S091 disaster_stop_atr must be nonnegative (0 disables the N-stop)".into());
    }
    if !(cfg.turtle_chandelier_mult.is_finite() && cfg.turtle_chandelier_mult > 0.0) {
        return Err("S091 turtle_chandelier_mult must be positive".into());
    }
    if !(cfg.turtle_risk_pct.is_finite() && cfg.turtle_risk_pct > 0.0)
        || !(cfg.turtle_max_position_weight.is_finite()
            && cfg.turtle_max_position_weight > 0.0)
    {
        return Err("S091 turtle_risk_pct and turtle_max_position_weight must be positive".into());
    }
    let allow_long = cfg.strategy_mode != "short";
    let allow_short = matches!(cfg.strategy_mode.as_str(), "long_short" | "short");
    let n_rows = panel.dates.len();
    let mut result = crate::family4::blank_signals(panel, "donchian_breakout_atr_distance");
    let mut atr_all = Vec::with_capacity(panel.symbols.len());
    for symbol in 0..panel.symbols.len() {
        let (high, low, close) = (
            &panel.high[symbol],
            &panel.low[symbol],
            &panel.close[symbol],
        );
        let atr = signals::wilder_atr(high, low, close, cfg.atr_len);
        atr_all.push(atr.clone());
        let mut history = 0usize;
        for day in 0..n_rows {
            if close[day].is_finite() {
                history += 1;
            }
            if !close[day].is_finite() || history < cfg.min_history || !atr[day].is_finite() {
                continue;
            }
            result.tradable[symbol][day] = true;
            result.pool[symbol][day] = true;
            let prior_high = rolling_high_before(high, day, cfg.entry_window);
            let prior_low = rolling_low_before(low, day, cfg.entry_window);
            if allow_long && prior_high.is_finite() && close[day] > prior_high {
                result.signal[symbol][day] = true;
                result.rsi[symbol][day] = (close[day] - prior_high) / atr[day];
            } else if allow_short && prior_low.is_finite() && close[day] < prior_low {
                result.signal[symbol][day] = true;
                result.rsi[symbol][day] = -(prior_low - close[day]) / atr[day];
            }
        }
    }
    result.atr = Some(atr_all);
    result.benchmark_returns = crate::family4::benchmark_returns(panel, "SPY");
    Ok(result)
}

#[derive(Clone)]
struct S091Leg {
    dir: f64,
    signal_idx: usize,
    entry_idx: usize,
    entry_px: f64,
    stop: f64,
    weight: f64,
    queued_exit: Option<usize>,
    extreme_since: f64,
    real: bool,
}

/// true = losing breakout (stop hit, or nonpositive gross in `pnl` mode).
fn s091_outcome_loss(cfg: &Config, leg: &S091Leg, reason: &str, exit_px: f64) -> bool {
    if cfg.turtle_filter_outcome == "pnl" {
        leg.dir * (exit_px / leg.entry_px - 1.0) <= 0.0
    } else {
        reason == "stop"
    }
}

#[allow(clippy::too_many_arguments)]
fn s091_resolve(
    slot: &mut Option<S091Leg>,
    exit_idx: usize,
    exit_px: f64,
    reason: &'static str,
    symbol: usize,
    sig: &Signals,
    cfg: &Config,
    cost: f64,
    last_failed: &mut Option<bool>,
    trades: &mut Vec<Trade>,
) {
    let Some(leg) = slot.take() else { return };
    if !exit_px.is_finite() || exit_px <= 0.0 || exit_idx < leg.entry_idx {
        return;
    }
    *last_failed = Some(s091_outcome_loss(cfg, &leg, reason, exit_px));
    if !leg.real {
        return;
    }
    let gross = leg.dir * (exit_px / leg.entry_px - 1.0);
    let hold = exit_idx - leg.entry_idx + 1;
    let borrow_per_bar = if leg.dir < 0.0 {
        cfg.borrow_bps_annual / 10_000.0 / cfg.periods_per_year as f64
    } else {
        0.0
    };
    trades.push(Trade {
        sym_idx: symbol,
        signal_idx: leg.signal_idx,
        entry_idx: leg.entry_idx,
        exit_idx,
        entry_px: leg.entry_px,
        exit_px,
        ret: gross - 2.0 * cost - borrow_per_bar * hold as f64,
        ret_gross: gross,
        exit_reason: reason,
        hold,
        signal_value: sig.rsi[symbol][leg.signal_idx],
        weight: leg.weight,
        entry_turnover: leg.weight.abs(),
        exit_turnover: leg.weight.abs(),
        weighted: true,
        borrow_cost_per_bar: borrow_per_bar,
    });
}

pub fn s091_trades(
    panel: &Panel,
    sig: &Signals,
    cfg: &Config,
    fill: &str,
) -> Result<Vec<Trade>, String> {
    if !matches!(fill, "close" | "next_open") {
        return Err(format!("S091 does not support fill {fill:?}"));
    }
    let atr = sig
        .atr
        .as_ref()
        .ok_or_else(|| "S091 requires the ATR(N) matrix".to_string())?;
    let is_open = fill == "next_open";
    let mut trades = Vec::new();
    for symbol in 0..panel.symbols.len() {
        if !sig.signal[symbol].iter().any(|&s| s) {
            continue;
        }
        let (open, high, low, close) = (
            &panel.open[symbol],
            &panel.high[symbol],
            &panel.low[symbol],
            &panel.close[symbol],
        );
        let cost = cfg.cost_for_symbol(&panel.symbols[symbol]);
        let trail_atr = if cfg.turtle_exit_mode == "chandelier" {
            signals::wilder_atr(high, low, close, cfg.turtle_chandelier_len)
        } else {
            Vec::new()
        };
        let Some(last_idx) = close.iter().rposition(|v| v.is_finite()) else {
            continue;
        };
        let mut active: Option<S091Leg> = None;
        let mut last_failed: Option<bool> = None;
        for day in 0..=last_idx {
            if let Some(leg) = active.as_mut() {
                // a close-triggered exit fills at today's open
                if leg.queued_exit == Some(day) {
                    let px = if open[day].is_finite() {
                        open[day]
                    } else {
                        close[day]
                    };
                    let reason = if cfg.turtle_exit_mode == "chandelier" {
                        "chandelier"
                    } else {
                        "signal"
                    };
                    s091_resolve(
                        &mut active, day, px, reason, symbol, sig, cfg, cost, &mut last_failed,
                        &mut trades,
                    );
                }
                if let Some(leg) = active.as_mut() {
                    if day >= leg.entry_idx && close[day].is_finite() {
                        let stopped = cfg.disaster_stop_atr > 0.0
                            && if leg.dir > 0.0 {
                                low[day].is_finite() && low[day] <= leg.stop
                            } else {
                                high[day].is_finite() && high[day] >= leg.stop
                            };
                        if stopped {
                            let px = if open[day].is_finite() {
                                if leg.dir > 0.0 {
                                    open[day].min(leg.stop)
                                } else {
                                    open[day].max(leg.stop)
                                }
                            } else {
                                leg.stop
                            };
                            s091_resolve(
                                &mut active, day, px, "stop", symbol, sig, cfg, cost,
                                &mut last_failed, &mut trades,
                            );
                        } else {
                            if leg.dir > 0.0 {
                                if high[day].is_finite() {
                                    leg.extreme_since = leg.extreme_since.max(high[day]);
                                }
                            } else if low[day].is_finite() {
                                leg.extreme_since = leg.extreme_since.min(low[day]);
                            }
                            let trigger = if cfg.turtle_exit_mode == "chandelier" {
                                let a = trail_atr[day];
                                a.is_finite()
                                    && if leg.dir > 0.0 {
                                        close[day]
                                            < leg.extreme_since - cfg.turtle_chandelier_mult * a
                                    } else {
                                        close[day]
                                            > leg.extreme_since + cfg.turtle_chandelier_mult * a
                                    }
                            } else if leg.dir > 0.0 {
                                let level = rolling_low_before(low, day, cfg.exit_window);
                                level.is_finite() && close[day] < level
                            } else {
                                let level = rolling_high_before(high, day, cfg.exit_window);
                                level.is_finite() && close[day] > level
                            };
                            if trigger {
                                if is_open && day + 1 <= last_idx {
                                    leg.queued_exit = Some(day + 1);
                                } else {
                                    let reason = if cfg.turtle_exit_mode == "chandelier" {
                                        "chandelier"
                                    } else {
                                        "signal"
                                    };
                                    s091_resolve(
                                        &mut active, day, close[day], reason, symbol, sig, cfg,
                                        cost, &mut last_failed, &mut trades,
                                    );
                                }
                            } else if day == last_idx {
                                s091_resolve(
                                    &mut active, day, close[day], "censored", symbol, sig, cfg,
                                    cost, &mut last_failed, &mut trades,
                                );
                            }
                        }
                    }
                }
            }
            if active.is_some() || !sig.signal[symbol][day] {
                continue;
            }
            let dir = sig.rsi[symbol][day].signum();
            if dir == 0.0 || !close[day].is_finite() {
                continue;
            }
            let mut take = !cfg.turtle_last_trade_filter || last_failed != Some(false);
            if !take && cfg.turtle_failsafe_window > 0 {
                take = if dir > 0.0 {
                    let level = rolling_high_before(high, day, cfg.turtle_failsafe_window);
                    level.is_finite() && close[day] > level
                } else {
                    let level = rolling_low_before(low, day, cfg.turtle_failsafe_window);
                    level.is_finite() && close[day] < level
                };
            }
            let entry_idx = if is_open { day + 1 } else { day };
            if entry_idx > last_idx {
                continue;
            }
            let entry_px = if is_open { open[entry_idx] } else { close[entry_idx] };
            let n = atr[symbol][day];
            if !entry_px.is_finite()
                || entry_px <= 0.0
                || !n.is_finite()
                || n <= 0.0
            {
                continue;
            }
            let weight = (cfg.turtle_risk_pct * entry_px / n)
                .min(cfg.turtle_max_position_weight)
                * dir;
            let stop = if cfg.disaster_stop_atr > 0.0 {
                entry_px - dir * cfg.disaster_stop_atr * n
            } else if dir > 0.0 {
                f64::NEG_INFINITY
            } else {
                f64::INFINITY
            };
            active = Some(S091Leg {
                dir,
                signal_idx: day,
                entry_idx,
                entry_px,
                stop,
                weight,
                queued_exit: None,
                extreme_since: if dir > 0.0 {
                    high[entry_idx].max(entry_px)
                } else {
                    low[entry_idx].min(entry_px)
                },
                real: take,
            });
        }
    }
    trades.sort_by(|a, b| {
        (a.entry_idx, &panel.symbols[a.sym_idx]).cmp(&(b.entry_idx, &panel.symbols[b.sym_idx]))
    });
    Ok(trades)
}
