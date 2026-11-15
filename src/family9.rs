//! Family 9 trend and pullback systems (stocks).

use crate::config::{split_csv, Config};
use crate::data::{self, Panel};
use crate::engine::{self, last_valid_rows, SwingParams, Trade};
use crate::family4::blank_signals;
use crate::signals::{rolling_mean, rolling_median, self, Mask, Signals};
use crate::ta_indicators::{ema, mean_at, mean_before, median_at, median_before};
use std::collections::BTreeSet;
use ta_lib::Core;

pub const S088_NAME: &str = "s088_macd_volume_cross";
pub const S088_TITLE: &str = "S088 MACD Bullish Cross with Volume";

/// TA-Lib MACD line/signal per run of finite closes; warm-up stays NaN.
fn macd_lines(
    close: &[f64],
    fast: usize,
    slow: usize,
    signal: usize,
) -> Result<(Vec<f64>, Vec<f64>), String> {
    let core = Core::new();
    let fast = i32::try_from(fast).map_err(|_| "macd_fast is out of range")?;
    let slow = i32::try_from(slow).map_err(|_| "macd_slow is out of range")?;
    let signal = i32::try_from(signal).map_err(|_| "macd_signal is out of range")?;
    let mut macd = vec![f64::NAN; close.len()];
    let mut macd_signal = vec![f64::NAN; close.len()];
    let mut start = 0;
    while start < close.len() {
        while start < close.len() && !close[start].is_finite() {
            start += 1;
        }
        let segment_start = start;
        while start < close.len() && close[start].is_finite() {
            start += 1;
        }
        let segment = &close[segment_start..start];
        if segment.len() < 2 {
            continue;
        }
        let (mut line, mut sig, mut hist) = (
            vec![0.0; segment.len()],
            vec![0.0; segment.len()],
            vec![0.0; segment.len()],
        );
        let range = core
            .macd(
                0,
                segment.len() - 1,
                segment,
                fast,
                slow,
                signal,
                &mut line,
                &mut sig,
                &mut hist,
            )
            .map_err(|error| format!("TA-Lib MACD failed: {error:?}"))?;
        let offset = segment_start + range.beg_idx;
        macd[offset..offset + range.count].copy_from_slice(&line[..range.count]);
        macd_signal[offset..offset + range.count].copy_from_slice(&sig[..range.count]);
    }
    Ok((macd, macd_signal))
}

/// S088: MACD line crosses above its signal while MACD > 0, close > SMA(200),
/// and volume > `volume_multiple` x the `volume_window`-day volume baseline;
/// enter at the next open. Exit is the bearish cross at that session's close,
/// or `max_hold` sessions.
pub fn compute_s088_signals(panel: &Panel, cfg: &Config) -> Result<Signals, String> {
    if cfg.macd_fast < 2 || cfg.macd_slow < 2 || cfg.macd_fast >= cfg.macd_slow {
        return Err("S088 requires 2 <= macd_fast < macd_slow".into());
    }
    if cfg.macd_signal == 0 || cfg.volume_window == 0 || cfg.max_hold == 0 {
        return Err("S088 macd_signal, volume_window, and max_hold must be greater than zero".into());
    }
    if !matches!(
        cfg.volume_mode.as_str(),
        "median" | "mean" | "median_incl" | "mean_incl"
    ) {
        return Err(format!("unknown S088 volume_mode {:?}", cfg.volume_mode));
    }
    if !cfg.volume_multiple.is_finite() || cfg.volume_multiple < 0.0 {
        return Err("S088 volume_multiple must be finite and nonnegative".into());
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
    let mut result = crate::family4::blank_signals(panel, "macd_hist");
    let inclusive = cfg.volume_mode.ends_with("_incl");
    let use_mean = cfg.volume_mode.starts_with("mean");
    for symbol in 0..panel.symbols.len() {
        let close = &panel.close[symbol];
        let volume = &panel.volume[symbol];
        let sma = signals::rolling_mean(close, cfg.sma_window.max(1));
        let (macd, macd_signal) = macd_lines(
            close,
            cfg.macd_fast,
            cfg.macd_slow,
            cfg.macd_signal,
        )?;
        let mut history = 0usize;
        for day in 0..panel.dates.len() {
            if day > 0
                && macd[day].is_finite()
                && macd_signal[day].is_finite()
                && macd[day - 1].is_finite()
                && macd_signal[day - 1].is_finite()
            {
                result.rsi[symbol][day] = macd[day] - macd_signal[day];
                result.exit_now[symbol][day] =
                    macd[day] < macd_signal[day] && macd[day - 1] >= macd_signal[day - 1];
            }
            if close[day].is_finite() {
                history += 1;
            }
            let liquid = ranks[symbol][day].is_finite()
                && ranks[symbol][day] <= cfg.universe_top_n as f64;
            if !close[day].is_finite() || !liquid || history < cfg.min_history {
                continue;
            }
            result.tradable[symbol][day] = true;
            result.pool[symbol][day] = true;
            if day == 0
                || !macd[day].is_finite()
                || !macd_signal[day].is_finite()
                || !macd[day - 1].is_finite()
                || !macd_signal[day - 1].is_finite()
                || macd[day] <= macd_signal[day]
                || macd[day - 1] > macd_signal[day - 1]
            {
                continue;
            }
            if cfg.require_macd_positive && macd[day] <= 0.0 {
                continue;
            }
            if cfg.sma_window > 0 && !(sma[day].is_finite() && close[day] > sma[day]) {
                continue;
            }
            if cfg.volume_multiple > 0.0 {
                if !volume[day].is_finite() {
                    continue;
                }
                let base = if inclusive {
                    if use_mean {
                        mean_at(volume, day, cfg.volume_window)
                    } else {
                        median_at(volume, day, cfg.volume_window)
                    }
                } else if use_mean {
                    mean_before(volume, day, cfg.volume_window)
                } else {
                    median_before(volume, day, cfg.volume_window)
                };
                if !(base.is_finite()
                    && base > 0.0
                    && volume[day] > cfg.volume_multiple * base)
                {
                    continue;
                }
            }
            result.signal[symbol][day] = true;
        }
    }
    result.benchmark_returns = crate::family4::benchmark_returns(panel, "SPY");
    Ok(result)
}

/// Entry: signal day t -> open t+1 (`next_open`) or same close (`close`).
/// Exit: first bearish MACD cross at that session's close, else the
/// `max_hold`-th session close; delisted/censored at the last valid close.
pub fn s088_trades(
    panel: &Panel,
    sig: &Signals,
    cfg: &Config,
    fill: &str,
) -> Result<Vec<Trade>, String> {
    if !matches!(fill, "next_open" | "close") {
        return Err(format!("S088 does not support fill {fill:?}"));
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
                signal_day + 1
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
            let mut reason = "time";
            for day in entry..=final_session {
                if panel.close[symbol][day].is_finite() && sig.exit_now[symbol][day] {
                    exit_idx = day;
                    reason = "signal";
                    break;
                }
            }
            if reason == "time" && exit_idx == last_row {
                reason = if last_row + 1 < n_rows {
                    "delisted"
                } else {
                    "censored"
                };
            }
            let mut exit_px = panel.close[symbol][exit_idx];
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
                signal_value: sig.rsi[symbol][signal_day],
                weight: 1.0,
                entry_turnover: 1.0,
                exit_turnover: 1.0,
                weighted: false,
                borrow_cost_per_bar: 0.0,
                dir_sign: 1.0,
            });
            available_after = exit_idx + 1;
        }
    }
    trades.sort_by(|a, b| {
        (a.entry_idx, &panel.symbols[a.sym_idx]).cmp(&(b.entry_idx, &panel.symbols[b.sym_idx]))
    });
    Ok(trades)
}

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
        dir_sign: 1.0,
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
pub const S089_NAME: &str = "s089_golden_cross_slope";
pub const S089_TITLE: &str = "S089 Golden Cross with Slope Filter";

/// S089: SMA(fast) crosses above SMA(slow) while SMA(slow) is rising over the
/// slope lookback; exit on the opposite cross (or a configured alternate).
pub fn compute_s089_signals(panel: &Panel, cfg: &Config) -> Result<Signals, String> {
    if cfg.s089_fast_window == 0
        || cfg.s089_slow_window == 0
        || cfg.s089_fast_window >= cfg.s089_slow_window
    {
        return Err("S089 requires 0 < s089_fast_window < s089_slow_window".into());
    }
    if cfg.s089_require_slope && cfg.s089_slope_lookback == 0 {
        return Err("S089 slope filter requires s089_slope_lookback > 0".into());
    }
    if cfg.s089_exit_mode == "slope" && cfg.s089_slope_lookback == 0 {
        return Err("S089 slope exit requires s089_slope_lookback > 0".into());
    }
    if matches!(cfg.s089_exit_mode.as_str(), "time" | "cross_or_time")
        && cfg.s089_max_hold == 0
    {
        return Err("S089 timed exits require s089_max_hold > 0".into());
    }
    if !matches!(
        cfg.s089_exit_mode.as_str(),
        "cross" | "sma_slow" | "slope" | "time" | "cross_or_time"
    ) {
        return Err(format!(
            "unknown S089 s089_exit_mode {:?}",
            cfg.s089_exit_mode
        ));
    }
    let slope = cfg.s089_slope_lookback;
    let ranks = if cfg.rank_cache_dir.trim().is_empty() {
        let stock_columns: Vec<usize> = (0..panel.symbols.len())
            .filter(|&j| {
                cfg.symbol_sources
                    .get(&panel.symbols[j])
                    .map(String::as_str)
                    != Some("etfs")
            })
            .collect();
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
    let mut result = crate::family4::blank_signals(panel, "golden_cross_slope");
    for symbol in 0..panel.symbols.len() {
        let etf = cfg
            .symbol_sources
            .get(&panel.symbols[symbol])
            .map(String::as_str)
            == Some("etfs");
        let (close, volume) = (&panel.close[symbol], &panel.volume[symbol]);
        let sma_fast = signals::rolling_mean(close, cfg.s089_fast_window);
        let sma_slow = signals::rolling_mean(close, cfg.s089_slow_window);
        let vol_median = (cfg.s089_vol_mult > 0.0)
            .then(|| signals::rolling_median(volume, cfg.s089_vol_window));
        let (mut history, mut in_pos, mut sig_day) = (0usize, false, 0usize);
        for day in 0..panel.dates.len() {
            if close[day].is_finite() {
                history += 1;
            }
            let liquid = etf
                || (ranks[symbol][day].is_finite()
                    && ranks[symbol][day] <= cfg.universe_top_n as f64);
            if !close[day].is_finite() || !liquid || history < cfg.min_history {
                continue;
            }
            result.tradable[symbol][day] = true;
            result.pool[symbol][day] = true;
            if sma_fast[day].is_finite() && sma_slow[day].is_finite() && sma_slow[day] > 0.0 {
                result.rsi[symbol][day] = sma_fast[day] / sma_slow[day] - 1.0;
            }
            let (f_now, s_now) = (sma_fast[day], sma_slow[day]);
            let (f_prev, s_prev) = if day > 0 {
                (sma_fast[day - 1], sma_slow[day - 1])
            } else {
                (f64::NAN, f64::NAN)
            };
            let cross_up = f_now.is_finite()
                && s_now.is_finite()
                && f_prev.is_finite()
                && s_prev.is_finite()
                && f_now > s_now
                && f_prev <= s_prev;
            let cross_dn = f_now.is_finite()
                && s_now.is_finite()
                && f_prev.is_finite()
                && s_prev.is_finite()
                && f_now < s_now
                && f_prev >= s_prev;
            // Exit decided on completed-session data; engine fills at close[day]
            // ("close" fill) or open[day+1] ("next_open" fill).
            let mut exited_today = false;
            if in_pos && day > sig_day {
                let hit = match cfg.s089_exit_mode.as_str() {
                    "cross" => cross_dn,
                    "sma_slow" => s_now.is_finite() && close[day] < s_now,
                    "slope" => {
                        day >= slope
                            && s_now.is_finite()
                            && sma_slow[day - slope].is_finite()
                            && s_now < sma_slow[day - slope]
                    }
                    "time" => day - sig_day >= cfg.s089_max_hold,
                    "cross_or_time" => cross_dn || day - sig_day >= cfg.s089_max_hold,
                    _ => false,
                };
                if hit {
                    result.exit_now[symbol][day] = true;
                    in_pos = false;
                    exited_today = true;
                }
            }
            let slope_ok = !cfg.s089_require_slope
                || (day >= slope
                    && s_now.is_finite()
                    && sma_slow[day - slope].is_finite()
                    && s_now > sma_slow[day - slope] * (1.0 + cfg.s089_slope_min));
            let fast_ok = cfg.s089_fast_slope == 0
                || (day >= cfg.s089_fast_slope
                    && f_now.is_finite()
                    && sma_fast[day - cfg.s089_fast_slope].is_finite()
                    && f_now > sma_fast[day - cfg.s089_fast_slope]);
            let vol_ok = cfg.s089_vol_mult <= 0.0
                || vol_median.as_ref().is_some_and(|median| {
                    median[day].is_finite() && volume[day] > cfg.s089_vol_mult * median[day]
                });
            if cross_up && slope_ok && fast_ok && vol_ok {
                result.signal[symbol][day] = true;
                if !in_pos && !exited_today {
                    in_pos = true;
                    sig_day = day;
                }
            }
        }
    }
    if cfg.disaster_stop_atr > 0.0 {
        result.atr = Some(
            (0..panel.symbols.len())
                .map(|j| {
                    signals::wilder_atr(
                        &panel.high[j],
                        &panel.low[j],
                        &panel.close[j],
                        cfg.atr_len,
                    )
                })
                .collect(),
        );
    }
    result.benchmark_returns = crate::family4::benchmark_returns(panel, "SPY");
    Ok(result)
}

pub fn s089_trades(
    panel: &Panel,
    sig: &Signals,
    cfg: &Config,
    fill: &str,
) -> Result<Vec<Trade>, String> {
    if !matches!(fill, "close" | "next_open") {
        return Err(format!("S089 does not support fill {fill:?}"));
    }
    let params = SwingParams {
        signal: &sig.signal,
        exit_now: &sig.exit_now,
        add_signal: None,
        max_units: 1,
        stop_atr: sig.atr.as_ref(),
        stop_pct: None,
        stop_mult: cfg.disaster_stop_atr,
        signal_value: &sig.rsi,
    };
    Ok(engine::swing_trades(panel, &params, cfg, fill))
}
pub const S095_NAME: &str = "s095_keltner_breakout";
pub const S095_TITLE: &str = "S095 Keltner Channel Breakout";

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
                dir_sign: 1.0,
            });
            available_after = exit_idx + 1;
        }
    }
    trades.sort_by(|a, b| {
        (a.entry_idx, &panel.symbols[a.sym_idx]).cmp(&(b.entry_idx, &panel.symbols[b.sym_idx]))
    });
    Ok(trades)
}

pub const S087_NAME: &str = "s087_ema20_pullback";
pub const S087_TITLE: &str = "S087 20-Day-EMA Pullback in Uptrend";
pub const S093_NAME: &str = "s093_higher_low_reversal";
pub const S093_TITLE: &str = "S093 Higher-Low Reversal Entry";

fn earnings_exclusion(
    panel: &Panel,
    earnings_by_symbol: &std::collections::HashMap<String, Vec<chrono::NaiveDate>>,
    symbol: usize,
    day: usize,
    window: usize,
) -> bool {
    let ticker = crate::family4::canonical_ticker(&panel.symbols[symbol]);
    let Some(events) = earnings_by_symbol.get(&ticker) else {
        return false;
    };
    events.iter().any(|event| {
        let event_day = panel.dates.partition_point(|date| date < event);
        event_day < panel.dates.len() && event_day.abs_diff(day) <= window
    })
}

fn s087_validate(cfg: &Config) -> Result<(), String> {
    if cfg.sma_window < 2
        || cfg.exit_sma < 2
        || cfg.ema_len < 2
        || cfg.atr_len < 1
        || cfg.max_hold == 0
    {
        return Err("S087 requires SMA windows >= 2, ATR length >= 1 and max_hold > 0".into());
    }
    if cfg.pullback_days == 0 {
        return Err("S087 pullback_days must be greater than zero".into());
    }
    if !cfg.s087_pullback_depth.is_finite() || !(0.0..1.0).contains(&cfg.s087_pullback_depth) {
        return Err("S087 pullback depth must be in [0, 1)".into());
    }
    if !matches!(cfg.s087_earnings_mode.as_str(), "include" | "exclude") {
        return Err(format!(
            "unknown S087 earnings mode {:?}",
            cfg.s087_earnings_mode
        ));
    }
    Ok(())
}

fn s093_validate(cfg: &Config) -> Result<(), String> {
    if cfg.sma_window < 2 || cfg.exit_sma < 2 || cfg.max_hold == 0 || cfg.volume_window == 0 {
        return Err("S093 requires SMA windows >= 2, volume_window > 0 and max_hold > 0".into());
    }
    if !matches!(
        cfg.s093_decline_mode.as_str(),
        "drawdown" | "rsi5" | "none"
    ) {
        return Err(format!(
            "unknown S093 decline mode {:?}",
            cfg.s093_decline_mode
        ));
    }
    if !cfg.s093_drawdown_pct.is_finite() || !(0.0..1.0).contains(&cfg.s093_drawdown_pct) {
        return Err("S093 drawdown threshold must be in [0, 1)".into());
    }
    if cfg.s093_drawdown_window == 0 || cfg.pullback_days == 0 {
        return Err("S093 decline and RSI lookback windows must be greater than zero".into());
    }
    if !cfg.s093_rsi_threshold.is_finite()
        || !(0.0..=100.0).contains(&cfg.s093_rsi_threshold)
    {
        return Err("S093 RSI threshold must be in [0, 100]".into());
    }
    if !cfg.volume_multiple.is_finite() || cfg.volume_multiple < 0.0 {
        return Err("S093 volume_multiple must be finite and nonnegative".into());
    }
    if !matches!(cfg.s093_earnings_mode.as_str(), "include" | "exclude") {
        return Err(format!(
            "unknown S093 earnings mode {:?}",
            cfg.s093_earnings_mode
        ));
    }
    Ok(())
}

fn family9_stock_ranks(panel: &Panel, cfg: &Config) -> Result<Vec<Vec<f64>>, String> {
    if cfg.rank_cache_dir.trim().is_empty() {
        let columns: Vec<_> = (0..panel.symbols.len()).collect();
        data::liquidity_ranks(
            panel,
            &columns,
            cfg.rank_window,
            cfg.rank_min_observations,
            cfg.min_price,
            cfg.min_history,
        )
    } else {
        data::read_liquid_rank_cache(&cfg.rank_cache_dir, &panel.dates, &panel.symbols)
    }
}

fn family9_earnings(
    cfg: &Config,
    earnings_mode: &str,
) -> Result<Option<std::collections::HashMap<String, Vec<chrono::NaiveDate>>>, String> {
    if earnings_mode == "include" {
        return Ok(None);
    }
    if cfg.earnings_dates_file.trim().is_empty() {
        return Err("earnings exclusion requires earnings_dates_file".into());
    }
    Ok(Some(crate::family4::read_earnings_dates(
        &cfg.earnings_dates_file,
    )?))
}

/// S087: SMA(200) uptrend, EMA(20) intraday touch, and optional trailing
/// pullback-depth threshold; exit on SMA(10) reclaim, EMA-2*ATR stop, or time.
pub fn compute_s087_signals(panel: &Panel, cfg: &Config) -> Result<Signals, String> {
    s087_validate(cfg)?;
    let ranks = family9_stock_ranks(panel, cfg)?;
    let earnings = family9_earnings(cfg, &cfg.s087_earnings_mode)?;
    let mut result = blank_signals(panel, "ema20_pullback");
    let n = panel.dates.len();
    for symbol in 0..panel.symbols.len() {
        let close = &panel.close[symbol];
        let low = &panel.low[symbol];
        let sma200 = rolling_mean(close, cfg.sma_window);
        let sma10 = rolling_mean(close, cfg.exit_sma);
        let ema20 = ema(close, cfg.ema_len);
        let atr = signals::wilder_atr(
            &panel.high[symbol],
            low,
            close,
            cfg.atr_len,
        );
        let mut history = 0usize;
        for day in 0..n {
            if close[day].is_finite() {
                history += 1;
            }
            let tradable = close[day].is_finite()
                && close[day] >= cfg.min_price
                && history >= cfg.min_history
                && ranks[symbol][day].is_finite()
                && ranks[symbol][day] <= cfg.universe_top_n as f64;
            result.tradable[symbol][day] = tradable;
            if !tradable
                || !sma200[day].is_finite()
                || !ema20[day].is_finite()
                || close[day] <= sma200[day]
                || !low[day].is_finite()
                || low[day] > ema20[day]
            {
                continue;
            }
            if cfg.s087_pullback_depth > 0.0 {
                if day < cfg.pullback_days {
                    continue;
                }
                let prior_close = close[day - cfg.pullback_days];
                if !prior_close.is_finite()
                    || close[day] / prior_close - 1.0 > -cfg.s087_pullback_depth
                {
                    continue;
                }
            }
            if earnings.as_ref().is_some_and(|events| {
                earnings_exclusion(
                    panel,
                    events,
                    symbol,
                    day,
                    cfg.earnings_window_sessions,
                )
            }) {
                continue;
            }
            result.signal[symbol][day] = true;
            result.rsi[symbol][day] = (close[day] / ema20[day]) - 1.0;
        }
        for day in 0..n {
            let stop = ema20[day].is_finite()
                && atr[day].is_finite()
                && close[day].is_finite()
                && close[day] < ema20[day] - 2.0 * atr[day];
            let reclaim = sma10[day].is_finite()
                && close[day].is_finite()
                && close[day] > sma10[day];
            result.exit_now[symbol][day] = stop || reclaim;
        }
    }
    result.benchmark_returns = crate::family4::benchmark_returns(panel, "SPY");
    Ok(result)
}

/// S093: rising three-bar low structure, bullish close and prior-volume
/// confirmation, gated by an explicit prior-decline definition.
pub fn compute_s093_signals(panel: &Panel, cfg: &Config) -> Result<Signals, String> {
    s093_validate(cfg)?;
    let ranks = family9_stock_ranks(panel, cfg)?;
    let earnings = family9_earnings(cfg, &cfg.s093_earnings_mode)?;
    let mut result = blank_signals(panel, "higher_low_reversal");
    let n = panel.dates.len();
    for symbol in 0..panel.symbols.len() {
        let (open, high, low, close, volume) = (
            &panel.open[symbol],
            &panel.high[symbol],
            &panel.low[symbol],
            &panel.close[symbol],
            &panel.volume[symbol],
        );
        let sma200 = rolling_mean(close, cfg.sma_window);
        let sma10 = rolling_mean(close, cfg.exit_sma);
        let median_volume: Vec<f64> = (0..n)
            .map(|day| median_before(volume, day, cfg.volume_window))
            .collect();
        let rsi5 = if cfg.s093_decline_mode == "rsi5" {
            Some(crate::ta_indicators::rsi(close, 5).map_err(|error| {
                format!("S093 TA-Lib RSI(5) failed: {error:?}")
            })?)
        } else {
            None
        };
        let mut history = 0usize;
        for day in 0..n {
            if close[day].is_finite() {
                history += 1;
            }
            let tradable = close[day].is_finite()
                && close[day] >= cfg.min_price
                && history >= cfg.min_history
                && ranks[symbol][day].is_finite()
                && ranks[symbol][day] <= cfg.universe_top_n as f64;
            result.tradable[symbol][day] = tradable;
            if !tradable
                || day < 2
                || !sma200[day].is_finite()
                || close[day] <= sma200[day]
                || !low[day].is_finite()
                || !low[day - 1].is_finite()
                || !low[day - 2].is_finite()
                || !(low[day] > low[day - 1] && low[day - 1] > low[day - 2])
                || !open[day].is_finite()
                || close[day] <= open[day]
                || !median_volume[day].is_finite()
                || !volume[day].is_finite()
                || volume[day] <= cfg.volume_multiple * median_volume[day]
            {
                continue;
            }
            let decline = match cfg.s093_decline_mode.as_str() {
                "drawdown" => {
                    let high_before =
                        rolling_high_before(high, day, cfg.s093_drawdown_window);
                    high_before.is_finite()
                        && close[day] <= high_before * (1.0 - cfg.s093_drawdown_pct)
                }
                "rsi5" => rsi5.as_ref().is_some_and(|values| {
                    let start = day.saturating_sub(cfg.pullback_days - 1);
                    values[start..=day]
                        .iter()
                        .any(|value| value.is_finite() && *value < cfg.s093_rsi_threshold)
                }),
                "none" => true,
                _ => unreachable!("S093 config validated"),
            };
            if !decline
                || earnings.as_ref().is_some_and(|events| {
                    earnings_exclusion(
                        panel,
                        events,
                        symbol,
                        day,
                        cfg.earnings_window_sessions,
                    )
                })
            {
                continue;
            }
            result.signal[symbol][day] = true;
            result.rsi[symbol][day] = close[day] / sma200[day] - 1.0;
        }
        for day in 1..n {
            let structure_broken = close[day].is_finite()
                && low[day - 1].is_finite()
                && close[day] < low[day - 1];
            let reclaimed = sma10[day].is_finite()
                && close[day].is_finite()
                && close[day] > sma10[day];
            result.exit_now[symbol][day] = structure_broken || reclaimed;
        }
    }
    result.benchmark_returns = crate::family4::benchmark_returns(panel, "SPY");
    Ok(result)
}

fn family9_reversal_trades(
    panel: &Panel,
    sig: &Signals,
    cfg: &Config,
    fill: &str,
) -> Result<Vec<Trade>, String> {
    if !matches!(fill, "close" | "next_open") {
        return Err(format!("Family 9 reversal does not support fill {fill:?}"));
    }
    let is_open = fill == "next_open";
    let n = panel.dates.len();
    let last = last_valid_rows(&panel.close);
    let mut trades = Vec::new();
    for symbol in 0..panel.symbols.len() {
        let Some(last_row) = last[symbol] else { continue };
        let mut next_signal = 0usize;
        for signal_day in 0..n {
            if signal_day < next_signal || !sig.signal[symbol][signal_day] {
                continue;
            }
            let entry_day = if is_open { signal_day + 1 } else { signal_day };
            if entry_day > last_row {
                continue;
            }
            let entry_price = if is_open {
                panel.open[symbol][entry_day]
            } else {
                panel.close[symbol][entry_day]
            };
            if !entry_price.is_finite() || entry_price <= 0.0 {
                continue;
            }
            let final_day = (entry_day + cfg.max_hold - 1).min(last_row);
            let (mut exit_day, mut exit_reason) = (final_day, "time");
            let start_exit = if is_open { entry_day } else { entry_day + 1 };
            for day in start_exit..=final_day {
                if sig.exit_now[symbol][day] {
                    exit_day = day;
                    exit_reason = if day == final_day { "time" } else { "signal" };
                    break;
                }
            }
            let mut exit_price = panel.close[symbol][exit_day];
            if is_open
                && exit_reason != "time"
                && exit_day + 1 <= last_row
                && panel.open[symbol][exit_day + 1].is_finite()
            {
                exit_day += 1;
                exit_price = panel.open[symbol][exit_day];
            }
            if exit_day == last_row && exit_reason == "time" && last_row + 1 < n {
                exit_reason = "delisted";
                if cfg.delisting_return != 0.0 {
                    exit_price *= 1.0 + cfg.delisting_return;
                }
            } else if exit_day == last_row && exit_reason == "time" {
                exit_reason = "censored";
            }
            if !exit_price.is_finite() || exit_price <= 0.0 {
                continue;
            }
            let cost = cfg.cost_for_symbol(&panel.symbols[symbol]);
            let ret = exit_price * (1.0 - cost) / (entry_price * (1.0 + cost)) - 1.0;
            trades.push(Trade {
                sym_idx: symbol,
                signal_idx: signal_day,
                entry_idx: entry_day,
                exit_idx: exit_day,
                entry_px: entry_price,
                exit_px: exit_price,
                ret,
                ret_gross: exit_price / entry_price - 1.0,
                exit_reason,
                hold: exit_day.saturating_sub(entry_day) + 1,
                signal_value: sig.rsi[symbol][signal_day],
                weight: 1.0,
                entry_turnover: 1.0,
                exit_turnover: 1.0,
                weighted: false,
                borrow_cost_per_bar: 0.0,
                dir_sign: 1.0,
            });
            next_signal = exit_day + 1;
        }
    }
    trades.sort_by(|a, b| {
        (a.entry_idx, &panel.symbols[a.sym_idx]).cmp(&(b.entry_idx, &panel.symbols[b.sym_idx]))
    });
    Ok(trades)
}

pub fn s087_trades(
    panel: &Panel,
    sig: &Signals,
    cfg: &Config,
    fill: &str,
) -> Result<Vec<Trade>, String> {
    family9_reversal_trades(panel, sig, cfg, fill)
}

pub fn s093_trades(
    panel: &Panel,
    sig: &Signals,
    cfg: &Config,
    fill: &str,
) -> Result<Vec<Trade>, String> {
    family9_reversal_trades(panel, sig, cfg, fill)
}

#[cfg(test)]
mod family9_pullback_tests {
    use super::*;
    use chrono::{Duration, NaiveDate};
    use std::fs;

    fn config() -> Config {
        serde_yaml::from_str(
            "partition_split_date: '2024-01-01'\n\
             min_history: 1\n\
             rank_window: 1\n\
             rank_min_observations: 1\n\
             min_price: 1.0\n\
             universe_top_n: 500\n\
             sma_window: 200\n\
             exit_sma: 10\n\
             ema_len: 20\n\
             atr_len: 14\n\
             pullback_days: 3\n\
             volume_window: 20\n\
             max_hold: 8\n\
             earnings_window_sessions: 3\n",
        )
        .unwrap()
    }

    fn trending_panel(length: usize) -> Panel {
        let start = NaiveDate::from_ymd_opt(2020, 1, 1).unwrap();
        let close: Vec<f64> = (0..length).map(|day| 100.0 + day as f64 * 0.3).collect();
        let open: Vec<f64> = close.iter().map(|value| value - 0.1).collect();
        let high: Vec<f64> = close.iter().map(|value| value + 0.3).collect();
        let low: Vec<f64> = close.iter().map(|value| value - 0.3).collect();
        let volume = vec![100.0; length];
        Panel {
            dates: (0..length)
                .map(|day| start + Duration::days(day as i64))
                .collect(),
            symbols: vec!["ABC".into()],
            open: vec![open],
            high: vec![high],
            low: vec![low],
            close: vec![close],
            volume: vec![volume],
        }
    }

    #[test]
    fn s087_requires_ema_touch_and_trend_and_honors_depth() {
        let mut panel = trending_panel(230);
        panel.close[0][220] = 163.0;
        panel.close[0][221] = 162.5;
        panel.open[0][220] = 163.1;
        panel.open[0][221] = 162.6;
        panel.low[0][221] = 158.0;
        panel.high[0][220] = 163.3;
        panel.high[0][221] = 162.8;
        let mut cfg = config();
        let baseline = compute_s087_signals(&panel, &cfg).unwrap();
        assert!(baseline.signal[0][221]);

        cfg.s087_pullback_depth = 0.03;
        let depth = compute_s087_signals(&panel, &cfg).unwrap();
        assert!(!depth.signal[0][221]);
    }

    #[test]
    fn s093_requires_prior_decline_higher_lows_and_volume_confirmation() {
        let mut panel = trending_panel(230);
        panel.close[0][220] = 155.0;
        panel.open[0][220] = 154.5;
        panel.low[0][218] = 150.0;
        panel.low[0][219] = 151.0;
        panel.low[0][220] = 152.0;
        panel.high[0][220] = 155.2;
        panel.volume[0][220] = 1_000.0;
        let cfg = config();
        let signals = compute_s093_signals(&panel, &cfg).unwrap();
        assert!(signals.signal[0][220]);

        panel.volume[0][220] = 100.0;
        let no_volume = compute_s093_signals(&panel, &cfg).unwrap();
        assert!(!no_volume.signal[0][220]);
    }

    #[test]
    fn s093_rsi_decline_alternative_detects_recent_oversold_condition() {
        let mut panel = trending_panel(230);
        for (day, close) in [
            (215, 164.0),
            (216, 161.0),
            (217, 158.0),
            (218, 155.0),
            (219, 152.0),
            (220, 153.0),
        ] {
            panel.close[0][day] = close;
            panel.open[0][day] = close - 0.1;
            panel.high[0][day] = close + 0.2;
        }
        panel.low[0][218] = 148.0;
        panel.low[0][219] = 149.0;
        panel.low[0][220] = 150.0;
        panel.volume[0][220] = 1_000.0;
        let mut cfg = config();
        cfg.s093_decline_mode = "rsi5".into();
        let signals = compute_s093_signals(&panel, &cfg).unwrap();
        assert!(signals.signal[0][220]);
    }

    #[test]
    fn s093_earnings_exclusion_blocks_signals_in_the_event_window() {
        let mut panel = trending_panel(230);
        panel.close[0][220] = 155.0;
        panel.open[0][220] = 154.5;
        panel.low[0][218] = 150.0;
        panel.low[0][219] = 151.0;
        panel.low[0][220] = 152.0;
        panel.high[0][220] = 155.2;
        panel.volume[0][220] = 1_000.0;
        let path = std::env::temp_dir().join(format!(
            "deepthrust_s093_earnings_{}.csv",
            std::process::id()
        ));
        fs::write(
            &path,
            format!("ticker;date\nABC;{}\n", panel.dates[220]),
        )
        .unwrap();
        let mut cfg = config();
        cfg.s093_earnings_mode = "exclude".into();
        cfg.earnings_dates_file = path.to_string_lossy().into_owned();
        let signals = compute_s093_signals(&panel, &cfg).unwrap();
        let _ = fs::remove_file(path);
        assert!(!signals.signal[0][220]);
    }

    #[test]
    fn family9_next_open_trades_respect_the_max_hold_cap() {
        let panel = trending_panel(230);
        let mut cfg = config();
        cfg.max_hold = 3;
        let mut signals = blank_signals(&panel, "test");
        signals.signal[0][220] = true;
        signals.exit_now[0][222] = true;
        let trades = s087_trades(&panel, &signals, &cfg, "next_open").unwrap();
        assert_eq!(trades.len(), 1);
        assert_eq!(trades[0].entry_idx, 221);
        assert_eq!(trades[0].exit_idx, 223);
        assert_eq!(trades[0].hold, 3);

        signals.exit_now[0][222] = false;
        signals.exit_now[0][223] = true;
        let trades = s087_trades(&panel, &signals, &cfg, "next_open").unwrap();
        assert_eq!(trades[0].exit_idx, 223);
        assert_eq!(trades[0].hold, 3);
        assert_eq!(trades[0].exit_reason, "time");
    }
}
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
                    dir_sign: 1.0,
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
pub const S094_NAME: &str = "s094_bollinger_squeeze";
pub const S094_TITLE: &str = "S094 Bollinger Squeeze Breakout";

/// Bollinger mid/upper/lower bands (population sd over `len` closes, NaN warm-up).
fn bb_bands(close: &[f64], len: usize, mult: f64) -> (Vec<f64>, Vec<f64>, Vec<f64>) {
    let n = close.len();
    let (mut mid, mut upper, mut lower) = (
        vec![f64::NAN; n],
        vec![f64::NAN; n],
        vec![f64::NAN; n],
    );
    let (mut sum, mut sum2, mut valid) = (0.0f64, 0.0f64, 0usize);
    for t in 0..n {
        let v = close[t];
        if v.is_finite() {
            sum += v;
            sum2 += v * v;
            valid += 1;
        }
        if t >= len {
            let old = close[t - len];
            if old.is_finite() {
                sum -= old;
                sum2 -= old * old;
                valid -= 1;
            }
        }
        if t + 1 >= len && valid == len {
            let m = sum / len as f64;
            let sd = ((sum2 - sum * m) / len as f64).max(0.0).sqrt();
            mid[t] = m;
            upper[t] = m + mult * sd;
            lower[t] = m - mult * sd;
        }
    }
    (mid, upper, lower)
}

// S094: BandWidth(20, 2sd) in the bottom decile of the last 120 sessions resolves
// directionally; trade the close outside the bands, exit on a middle-band cross.
pub fn compute_s094_signals(panel: &Panel, cfg: &Config) -> Result<Signals, String> {
    if cfg.bb_len < 2 || cfg.width_lookback < 2 || cfg.max_hold == 0 {
        return Err("S094 needs bb_len>=2, width_lookback>=2 and max_hold>0".into());
    }
    if !cfg.bb_mult.is_finite() || cfg.bb_mult <= 0.0 {
        return Err("S094 bb_mult must be positive and finite".into());
    }
    if !cfg.decile.is_finite() || cfg.decile <= 0.0 || cfg.decile > 1.0 {
        return Err("S094 decile must be in (0, 1]".into());
    }
    if !matches!(cfg.leg_mode.as_str(), "long" | "short" | "long_short") {
        return Err(format!("unknown S094 leg_mode {:?}", cfg.leg_mode));
    }
    if !matches!(cfg.exit_mode.as_str(), "mid_band" | "reentry" | "fixed") {
        return Err(format!("unknown S094 exit_mode {:?}", cfg.exit_mode));
    }
    if !matches!(cfg.gate.as_str(), "none" | "sma200") {
        return Err(format!("unknown S094 gate {:?}", cfg.gate));
    }
    let etf_names: BTreeSet<String> = split_csv(&cfg.etf_symbols).into_iter().collect();
    let stock_cols: Vec<usize> = (0..panel.symbols.len())
        .filter(|&j| !etf_names.contains(panel.symbols[j].as_str()))
        .collect();
    let ranks = if cfg.rank_cache_dir.trim().is_empty() {
        crate::data::liquidity_ranks(
            panel,
            &stock_cols,
            cfg.rank_window,
            cfg.rank_min_observations,
            cfg.min_price,
            cfg.min_history,
        )?
    } else {
        crate::data::read_liquid_rank_cache(&cfg.rank_cache_dir, &panel.dates, &panel.symbols)?
    };
    let n = panel.dates.len();
    let w = cfg.width_lookback;
    let mut result = blank_signals(panel, "bb_width_pctile");
    result.entry_mat = None;
    result.exit_mat = None;
    let mut weights = vec![vec![f64::NAN; n]; panel.symbols.len()];
    for j in 0..panel.symbols.len() {
        let close = &panel.close[j];
        let is_etf = etf_names.contains(panel.symbols[j].as_str());
        let (mid, upper, lower) = bb_bands(close, cfg.bb_len, cfg.bb_mult);
        let bandwidth: Vec<f64> = (0..n)
            .map(|t| {
                if mid[t].is_finite() && mid[t] != 0.0 {
                    (upper[t] - lower[t]) / mid[t]
                } else {
                    f64::NAN
                }
            })
            .collect();
        let medvol = if cfg.vol_confirm_mult > 0.0 {
            rolling_median(&panel.volume[j], cfg.gap_volume_window)
        } else {
            vec![f64::NAN; n]
        };
        let sma_gate = if cfg.gate == "sma200" {
            rolling_mean(close, cfg.sma_window)
        } else {
            vec![f64::NAN; n]
        };
        let mut pctile = vec![f64::NAN; n];
        let mut hist = 0usize;
        for t in 0..n {
            if close[t].is_finite() {
                hist += 1;
            }
            result.tradable[j][t] = if is_etf {
                close[t].is_finite() && hist >= cfg.min_history
            } else {
                cfg.include_stocks
                    && close[t].is_finite()
                    && close[t] >= cfg.min_price
                    && hist >= cfg.min_history
                    && ranks[j][t].is_finite()
                    && ranks[j][t] <= cfg.universe_top_n as f64
            };
            if t + 1 >= w {
                let window = &bandwidth[t + 1 - w..=t];
                if window.iter().all(|v| v.is_finite()) {
                    let count = window.iter().filter(|&&v| v <= bandwidth[t]).count();
                    pctile[t] = count as f64 / w as f64;
                }
            }
            result.rsi[j][t] = pctile[t];
            if !result.tradable[j][t] || !close[t].is_finite() || !upper[t].is_finite() {
                continue;
            }
            let squeezed = t >= cfg.squeeze_lag
                && pctile[t - cfg.squeeze_lag].is_finite()
                && pctile[t - cfg.squeeze_lag] <= cfg.decile;
            if !squeezed {
                continue;
            }
            let dir = if close[t] > upper[t] {
                if cfg.require_first_break
                    && !(t > 0
                        && close[t - 1].is_finite()
                        && upper[t - 1].is_finite()
                        && close[t - 1] <= upper[t - 1])
                {
                    continue;
                }
                1.0
            } else if close[t] < lower[t] {
                if cfg.require_first_break
                    && !(t > 0
                        && close[t - 1].is_finite()
                        && lower[t - 1].is_finite()
                        && close[t - 1] >= lower[t - 1])
                {
                    continue;
                }
                -1.0
            } else {
                continue;
            };
            if (dir > 0.0 && cfg.leg_mode == "short")
                || (dir < 0.0 && cfg.leg_mode == "long")
            {
                continue;
            }
            if cfg.gate == "sma200"
                && (!sma_gate[t].is_finite() || dir * (close[t] - sma_gate[t]) <= 0.0)
            {
                continue;
            }
            if cfg.vol_confirm_mult > 0.0
                && (!medvol[t].is_finite()
                    || !panel.volume[j][t].is_finite()
                    || panel.volume[j][t] <= cfg.vol_confirm_mult * medvol[t])
            {
                continue;
            }
            result.signal[j][t] = true;
            weights[j][t] = dir;
        }
    }
    result.trade_weights = Some(weights);
    Ok(result)
}

/// Equal-slot signed book: each entry is one 1/`book_slots` slot, shorts carry
/// negative weight and are charged `borrow_bps_annual`. `close` fills and exits
/// on the signal session; `next_open` enters open[t+1] and exits open[u+1].
pub fn s094_trades(
    panel: &Panel,
    sig: &Signals,
    cfg: &Config,
    fill: &str,
) -> Result<Vec<Trade>, String> {
    if !matches!(fill, "close" | "next_open") {
        return Err(format!("S094 does not support fill {fill:?}"));
    }
    let dir_sig = sig
        .trade_weights
        .as_ref()
        .ok_or_else(|| "S094 requires trade weights".to_string())?;
    let n = panel.dates.len();
    let last = last_valid_rows(&panel.close);
    let slot_w = 1.0 / cfg.book_slots.max(1) as f64;
    let borrow_per_bar = cfg.borrow_bps_annual / 10_000.0 / cfg.periods_per_year as f64;
    let mut trades = Vec::new();
    for j in 0..panel.symbols.len() {
        let Some(lj) = last[j] else { continue };
        if !dir_sig[j].iter().any(|v| v.is_finite()) {
            continue;
        }
        let (close, open) = (&panel.close[j], &panel.open[j]);
        let (mid, upper, lower) = bb_bands(close, cfg.bb_len, cfg.bb_mult);
        let cost = cfg.cost_for_symbol(&panel.symbols[j]);
        let mut pos: Option<(usize, usize, f64, f64)> = None; // (sig_day, entry, px, dir)
        for t in 0..=lj {
            if let Some((sd, e, ep, dir)) = pos {
                if !close[t].is_finite() {
                    continue;
                }
                let held = if fill == "next_open" { t - e + 1 } else { t - e };
                if held == 0 {
                    continue;
                }
                let mut reason: Option<&'static str> = None;
                if held >= cfg.min_hold {
                    let cross = match cfg.exit_mode.as_str() {
                        "reentry" => {
                            if dir > 0.0 {
                                close[t] < upper[t]
                            } else {
                                close[t] > lower[t]
                            }
                        }
                        "fixed" => false,
                        _ => mid[t].is_finite() && dir * (mid[t] - close[t]) > 0.0,
                    };
                    if cross {
                        reason = Some(if cfg.exit_mode == "reentry" {
                            "reentry"
                        } else {
                            "mid_band"
                        });
                    }
                }
                if reason.is_none() && held >= cfg.max_hold {
                    reason = Some("time");
                }
                if reason.is_none() && cfg.exit_mode == "fixed" && held >= cfg.fixed_hold.max(1) {
                    reason = Some("fixed");
                }
                if let Some(r) = reason {
                    let (exit_idx, px) = if fill == "close" {
                        (t, close[t])
                    } else if t + 1 <= lj && open[t + 1].is_finite() {
                        (t + 1, open[t + 1])
                    } else {
                        (t, close[t])
                    };
                    if !px.is_finite() || px <= 0.0 {
                        continue;
                    }
                    let gross = dir * (px / ep - 1.0);
                    let borrow = if dir < 0.0 {
                        borrow_per_bar * (exit_idx - e + 1) as f64
                    } else {
                        0.0
                    };
                    trades.push(Trade {
                        sym_idx: j,
                        signal_idx: sd,
                        entry_idx: e,
                        exit_idx,
                        entry_px: ep,
                        exit_px: px,
                        ret: gross - cost * 2.0 * slot_w - borrow,
                        ret_gross: gross,
                        exit_reason: r,
                        hold: exit_idx - e,
                        signal_value: sig.rsi[j][sd],
                        weight: dir * slot_w,
                        entry_turnover: slot_w,
                        exit_turnover: slot_w,
                        weighted: true,
                        borrow_cost_per_bar: if dir < 0.0 { borrow_per_bar } else { 0.0 },
                        dir_sign: 1.0,
                    });
                    pos = None;
                }
                continue;
            }
            let dir = dir_sig[j][t];
            if !dir.is_finite() || dir == 0.0 {
                continue;
            }
            let (e, px) = if fill == "close" {
                (t, close[t])
            } else {
                (
                    t + 1,
                    if t + 1 <= lj {
                        open[t + 1]
                    } else {
                        f64::NAN
                    },
                )
            };
            if px.is_finite() && px > 0.0 {
                pos = Some((t, e, px, dir));
            }
        }
        if let Some((sd, e, ep, dir)) = pos {
            let delisted = lj + 1 < n;
            let mut px = close[lj];
            if delisted {
                px *= 1.0 + cfg.delisting_return;
            }
            let gross = dir * (px / ep - 1.0);
            let borrow = if dir < 0.0 {
                borrow_per_bar * (lj - e + 1) as f64
            } else {
                0.0
            };
            trades.push(Trade {
                sym_idx: j,
                signal_idx: sd,
                entry_idx: e,
                exit_idx: lj,
                entry_px: ep,
                exit_px: px,
                ret: gross - cost * 2.0 * slot_w - borrow,
                ret_gross: gross,
                exit_reason: if delisted { "delisted" } else { "censored" },
                hold: lj - e,
                signal_value: sig.rsi[j][sd],
                weight: dir * slot_w,
                entry_turnover: slot_w,
                exit_turnover: slot_w,
                weighted: true,
                borrow_cost_per_bar: if dir < 0.0 { borrow_per_bar } else { 0.0 },
                dir_sign: 1.0,
            });
        }
    }
    trades.sort_by(|a, b| {
        (a.entry_idx, &panel.symbols[a.sym_idx]).cmp(&(b.entry_idx, &panel.symbols[b.sym_idx]))
    });
    Ok(trades)
}
