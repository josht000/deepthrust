//! Family 9 trend and pullback systems (stocks).

use crate::config::Config;
use crate::data::{self, Panel};
use crate::engine::{self, SwingParams, Trade};
use crate::family4::blank_signals;
use crate::signals::{self, Signals};
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

fn median_before(values: &[f64], day: usize, window: usize) -> f64 {
    if window == 0 || day < window {
        return f64::NAN;
    }
    let mut sample = values[day - window..day].to_vec();
    if sample.iter().any(|value| !value.is_finite()) {
        return f64::NAN;
    }
    sample.sort_by(f64::total_cmp);
    let middle = sample.len() / 2;
    if sample.len() % 2 == 0 {
        (sample[middle - 1] + sample[middle]) / 2.0
    } else {
        sample[middle]
    }
}

fn mean_before(values: &[f64], day: usize, window: usize) -> f64 {
    if window == 0 || day < window {
        return f64::NAN;
    }
    let sample = &values[day - window..day];
    if sample.iter().any(|value| !value.is_finite()) {
        return f64::NAN;
    }
    sample.iter().sum::<f64>() / window as f64
}

fn median_at(values: &[f64], day: usize, window: usize) -> f64 {
    if window == 0 || day + 1 < window {
        return f64::NAN;
    }
    let mut sample = values[day + 1 - window..=day].to_vec();
    if sample.iter().any(|value| !value.is_finite()) {
        return f64::NAN;
    }
    sample.sort_by(f64::total_cmp);
    let middle = sample.len() / 2;
    if sample.len() % 2 == 0 {
        (sample[middle - 1] + sample[middle]) / 2.0
    } else {
        sample[middle]
    }
}

fn mean_at(values: &[f64], day: usize, window: usize) -> f64 {
    if window == 0 || day + 1 < window {
        return f64::NAN;
    }
    let sample = &values[day + 1 - window..=day];
    if sample.iter().any(|value| !value.is_finite()) {
        return f64::NAN;
    }
    sample.iter().sum::<f64>() / window as f64
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
