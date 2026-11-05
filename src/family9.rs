//! Family 9 trend and pullback systems (stocks).

use crate::config::Config;
use crate::data::{self, Panel};
use crate::engine::{self, Trade};
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
