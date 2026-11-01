//! Family 4 strategy signals and execution models.

use crate::config::Config;
use crate::data::{self, Panel};
use crate::engine::{self, SwingParams, Trade};
use crate::signals::{self, Signals};
use chrono::{Datelike, NaiveDate};
use std::collections::{BTreeMap, HashMap};
use std::path::Path;

fn blank_signals(panel: &Panel, label: &str) -> Signals {
    let rows = panel.dates.len();
    let symbols = panel.symbols.len();
    Signals {
        rsi: vec![vec![f64::NAN; rows]; symbols],
        indicator_name: label.into(),
        signal: vec![vec![false; rows]; symbols],
        exit_now: vec![vec![false; rows]; symbols],
        add_signal: None,
        atr: None,
        stop_pct: None,
        tradable: vec![vec![false; rows]; symbols],
        pool: vec![vec![false; rows]; symbols],
        trade_weights: None,
        entry_mat: None,
        exit_mat: None,
        benchmark_returns: None,
    }
}

#[derive(Clone, Debug)]
struct Membership {
    start: NaiveDate,
    end: Option<NaiveDate>,
}

fn canonical_ticker(ticker: &str) -> String {
    ticker.trim().to_ascii_uppercase().replace('.', "-")
}

fn read_sp500_membership(path: &str) -> Result<BTreeMap<String, Vec<Membership>>, String> {
    if path.trim().is_empty() {
        return Err("S040/S050 require sp500_membership_file".into());
    }
    let path = Path::new(path);
    let mut reader =
        csv::Reader::from_path(path).map_err(|error| format!("{}: {error}", path.display()))?;
    let headers = reader
        .headers()
        .map_err(|error| format!("{}: {error}", path.display()))?
        .clone();
    let column = |name: &str| {
        headers
            .iter()
            .position(|header| header.eq_ignore_ascii_case(name))
            .ok_or_else(|| format!("{}: missing {name:?} column", path.display()))
    };
    let ticker_col = column("ticker")?;
    let start_col = column("start_date")?;
    let end_col = column("end_date")?;
    let mut members = BTreeMap::<String, Vec<Membership>>::new();
    for (row, result) in reader.records().enumerate() {
        let record =
            result.map_err(|error| format!("{} row {}: {error}", path.display(), row + 2))?;
        let ticker = record
            .get(ticker_col)
            .ok_or_else(|| format!("{} row {}: missing ticker", path.display(), row + 2))?;
        let start = record
            .get(start_col)
            .ok_or_else(|| format!("{} row {}: missing start_date", path.display(), row + 2))?;
        let end = record.get(end_col).unwrap_or_default();
        if ticker.trim().is_empty() {
            return Err(format!("{} row {}: empty ticker", path.display(), row + 2));
        }
        let start = data::parse_date(start.trim())
            .map_err(|error| format!("{} row {}: {error}", path.display(), row + 2))?;
        let end = if end.trim().is_empty() {
            None
        } else {
            Some(
                data::parse_date(end.trim())
                    .map_err(|error| format!("{} row {}: {error}", path.display(), row + 2))?,
            )
        };
        if matches!(end, Some(date) if date < start) {
            return Err(format!(
                "{} row {}: membership ends before it starts",
                path.display(),
                row + 2
            ));
        }
        members
            .entry(canonical_ticker(ticker))
            .or_default()
            .push(Membership { start, end });
    }
    if members.is_empty() {
        return Err(format!("{}: no membership rows", path.display()));
    }
    Ok(members)
}

/// Return listed S&P 500 symbols present in a configured data source.
pub fn sp500_symbols(path: &str, available: &[String]) -> Result<Vec<String>, String> {
    let members = read_sp500_membership(path)?;
    let listed: Vec<_> = available
        .iter()
        .filter(|symbol| members.contains_key(&canonical_ticker(symbol)))
        .cloned()
        .collect();
    if listed.is_empty() {
        return Err(format!(
            "S&P 500 membership file {path:?} has no symbols in the configured data source"
        ));
    }
    Ok(listed)
}

fn s_and_p_membership(panel: &Panel, path: &str) -> Result<Vec<Vec<bool>>, String> {
    let members = read_sp500_membership(path)?;
    Ok(panel
        .symbols
        .iter()
        .map(|symbol| {
            let intervals = members.get(&canonical_ticker(symbol));
            panel
                .dates
                .iter()
                .map(|date| {
                    intervals.is_some_and(|periods| {
                        periods.iter().any(|period| {
                            period.start <= *date && period.end.is_none_or(|end| *date <= end)
                        })
                    })
                })
                .collect()
        })
        .collect())
}

fn benchmark_returns(panel: &Panel, symbol: &str) -> Option<Vec<f64>> {
    let column = panel.symbols.iter().position(|name| name == symbol)?;
    let close = &panel.close[column];
    Some(
        (0..close.len())
            .map(|row| {
                if row == 0 || !close[row].is_finite() || close[row - 1] <= 0.0 {
                    0.0
                } else {
                    close[row] / close[row - 1] - 1.0
                }
            })
            .collect(),
    )
}

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

fn rolling_median_before(values: &[f64], day: usize, window: usize) -> f64 {
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

fn read_earnings_dates(path: &str) -> Result<HashMap<String, Vec<NaiveDate>>, String> {
    let path = Path::new(path);
    let mut reader = csv::ReaderBuilder::new()
        .delimiter(b';')
        .from_path(path)
        .map_err(|error| format!("{}: {error}", path.display()))?;
    let headers = reader
        .headers()
        .map_err(|error| format!("{}: {error}", path.display()))?
        .clone();
    let find_column = |names: &[&str]| {
        headers
            .iter()
            .position(|header| names.iter().any(|name| header.eq_ignore_ascii_case(name)))
    };
    let ticker_col = find_column(&["ticker", "symbol"])
        .ok_or_else(|| format!("{}: missing ticker/symbol column", path.display()))?;
    let date_col = find_column(&[
        "date",
        "earnings_date",
        "announcement_date",
        "publish date",
        "report date",
    ])
    .ok_or_else(|| format!("{}: missing earnings date column", path.display()))?;
    let mut dates = HashMap::<String, Vec<NaiveDate>>::new();
    for (row, result) in reader.records().enumerate() {
        let record =
            result.map_err(|error| format!("{} row {}: {error}", path.display(), row + 2))?;
        let ticker = record.get(ticker_col).unwrap_or_default();
        let date = record.get(date_col).unwrap_or_default();
        let parsed = data::parse_date(date.trim())
            .map_err(|error| format!("{} row {}: {error}", path.display(), row + 2))?;
        dates
            .entry(canonical_ticker(ticker))
            .or_default()
            .push(parsed);
    }
    for values in dates.values_mut() {
        values.sort_unstable();
        values.dedup();
    }
    if dates.is_empty() {
        return Err(format!("{}: no earnings dates", path.display()));
    }
    Ok(dates)
}

pub fn compute_s040_signals(panel: &Panel, cfg: &Config) -> Result<Signals, String> {
    if cfg.entry_window == 0 || cfg.atr_len == 0 || cfg.gap_volume_window == 0 {
        return Err("S040 lookback windows must be greater than zero".into());
    }
    if !matches!(cfg.gap_volume_mode.as_str(), "high" | "quiet") {
        return Err(format!(
            "S040 volume mode must be high or quiet, got {:?}",
            cfg.gap_volume_mode
        ));
    }
    if cfg.breakout_distance_atr < 0.0 || cfg.gap_guard_atr < 0.0 {
        return Err("S040 breakout distances must not be negative".into());
    }
    if cfg.gap_volume_multiple <= 0.0 || cfg.gap_reward_risk <= 0.0 || cfg.gap_max_hold == 0 {
        return Err("S040 volume threshold, reward/risk, and max hold must be positive".into());
    }
    let membership = s_and_p_membership(panel, &cfg.sp500_membership_file)?;
    let mut result = blank_signals(panel, "volume_breakout_distance_atr");
    for (symbol, close) in panel.close.iter().enumerate() {
        let atr = signals::wilder_atr(&panel.high[symbol], &panel.low[symbol], close, cfg.atr_len);
        for day in 0..panel.dates.len() {
            if !membership[symbol][day] || !close[day].is_finite() {
                continue;
            }
            result.tradable[symbol][day] = true;
            result.pool[symbol][day] = true;
            let prior_high = rolling_high_before(&panel.high[symbol], day, cfg.entry_window);
            let median_volume =
                rolling_median_before(&panel.volume[symbol], day, cfg.gap_volume_window);
            if !prior_high.is_finite()
                || !median_volume.is_finite()
                || !atr[day].is_finite()
                || median_volume <= 0.0
            {
                continue;
            }
            let relative_volume = panel.volume[symbol][day] / median_volume;
            let volume_ok = if cfg.gap_volume_mode == "high" {
                relative_volume > cfg.gap_volume_multiple
            } else {
                relative_volume < cfg.gap_volume_multiple
            };
            let distance = close[day] - prior_high;
            if distance > cfg.breakout_distance_atr * atr[day] && volume_ok {
                result.signal[symbol][day] = true;
                result.rsi[symbol][day] = distance / atr[day];
            }
        }
    }
    result.atr = Some(
        panel
            .close
            .iter()
            .enumerate()
            .map(|(symbol, close)| {
                signals::wilder_atr(&panel.high[symbol], &panel.low[symbol], close, cfg.atr_len)
            })
            .collect(),
    );
    result.benchmark_returns = benchmark_returns(panel, "SPY");
    Ok(result)
}

pub fn s040_trades(
    panel: &Panel,
    sig: &Signals,
    cfg: &Config,
    fill: &str,
) -> Result<Vec<Trade>, String> {
    if !matches!(fill, "next_open" | "close") {
        return Err(format!("S040 does not support fill {fill:?}"));
    }
    let mut trades = Vec::new();
    for symbol in 0..panel.symbols.len() {
        let cost = cfg.cost_for_symbol(&panel.symbols[symbol]);
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
            if entry >= panel.dates.len() {
                continue;
            }
            let entry_px = if fill == "next_open" {
                panel.open[symbol][entry]
            } else {
                panel.close[symbol][entry]
            };
            let signal_close = panel.close[symbol][signal_day];
            let signal_atr = sig.atr.as_ref().map(|atr| atr[symbol][signal_day]);
            if !entry_px.is_finite()
                || entry_px <= 0.0
                || !signal_close.is_finite()
                || !signal_atr.is_some_and(f64::is_finite)
            {
                continue;
            }
            let atr = signal_atr.unwrap();
            if fill == "next_open"
                && panel.open[symbol][entry] > signal_close + cfg.gap_guard_atr * atr
            {
                continue;
            }
            let stop = panel.low[symbol][signal_day];
            let risk = entry_px - stop;
            if !stop.is_finite() || risk <= 0.0 {
                continue;
            }
            let target = entry_px + cfg.gap_reward_risk * risk;
            let final_session = (entry + cfg.gap_max_hold - 1).min(panel.dates.len() - 1);
            let mut exit_idx = final_session;
            let mut exit_px = panel.close[symbol][final_session];
            let mut reason = "time";
            let first_check = if fill == "close" { entry + 1 } else { entry };
            for day in first_check..=final_session {
                let open = panel.open[symbol][day];
                let high = panel.high[symbol][day];
                let low = panel.low[symbol][day];
                if !open.is_finite() || !high.is_finite() || !low.is_finite() {
                    continue;
                }
                if open <= stop || low <= stop {
                    exit_idx = day;
                    exit_px = open.min(stop);
                    reason = "stop";
                    break;
                }
                if open >= target || high >= target {
                    exit_idx = day;
                    exit_px = open.max(target);
                    reason = "target";
                    break;
                }
            }
            if exit_idx < entry || !exit_px.is_finite() {
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

fn pivot_high(high: &[f64], day: usize) -> bool {
    if day < 3 || day + 3 >= high.len() || !high[day].is_finite() {
        return false;
    }
    (day - 3..day).all(|index| high[index].is_finite() && high[day] > high[index])
        && (day + 1..=day + 3).all(|index| high[index].is_finite() && high[day] >= high[index])
}

fn pivot_low(low: &[f64], day: usize) -> bool {
    if day < 3 || day + 3 >= low.len() || !low[day].is_finite() {
        return false;
    }
    (day - 3..day).all(|index| low[index].is_finite() && low[day] < low[index])
        && (day + 1..=day + 3).all(|index| low[index].is_finite() && low[day] <= low[index])
}

pub fn compute_s051_signals(panel: &Panel, cfg: &Config) -> Result<Signals, String> {
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
    let mut result = blank_signals(panel, "darvas_box");
    for symbol in 0..panel.symbols.len() {
        let (high, low, close) = (
            &panel.high[symbol],
            &panel.low[symbol],
            &panel.close[symbol],
        );
        let (mut current_top, mut current_bottom) = (None::<(usize, f64)>, None::<(usize, f64)>);
        for day in 0..panel.dates.len() {
            if day >= 6 {
                let pivot = day - 3;
                if pivot_high(high, pivot) {
                    current_top = Some((pivot, high[pivot]));
                }
                if pivot_low(low, pivot) && current_top.is_some_and(|(top_day, _)| top_day < pivot)
                {
                    let value = low[pivot];
                    current_bottom = Some(match current_bottom {
                        Some((_, old_value)) if value <= old_value => (pivot, old_value),
                        _ => (pivot, value),
                    });
                }
            }
            let history = close[..=day]
                .iter()
                .filter(|value| value.is_finite())
                .count();
            let liquid =
                ranks[symbol][day].is_finite() && ranks[symbol][day] <= cfg.universe_top_n as f64;
            if !close[day].is_finite() || !liquid || history < cfg.min_history {
                continue;
            }
            result.tradable[symbol][day] = true;
            result.pool[symbol][day] = true;
            if let (Some((top_day, top)), Some((bottom_day, bottom))) =
                (current_top, current_bottom)
            {
                if top_day < bottom_day {
                    result.signal[symbol][day] = close[day] > top
                        && (day == 0 || !close[day - 1].is_finite() || close[day - 1] <= top);
                    result.exit_now[symbol][day] = close[day] < bottom;
                    result.rsi[symbol][day] = close[day] / top - 1.0;
                }
            }
        }
    }
    result.benchmark_returns = benchmark_returns(panel, "SPY");
    Ok(result)
}

pub fn s051_trades(
    panel: &Panel,
    sig: &Signals,
    cfg: &Config,
    fill: &str,
) -> Result<Vec<Trade>, String> {
    if !matches!(fill, "close" | "next_open") {
        return Err(format!("S051 does not support fill {fill:?}"));
    }
    let params = SwingParams {
        signal: &sig.signal,
        exit_now: &sig.exit_now,
        add_signal: None,
        max_units: 1,
        stop_atr: None,
        stop_pct: None,
        stop_mult: 0.0,
        signal_value: &sig.rsi,
    };
    Ok(engine::swing_trades(panel, &params, cfg, fill))
}

pub fn compute_s050_signals(panel: &Panel, cfg: &Config) -> Result<Signals, String> {
    if cfg.entry_window == 0
        || cfg.formation_window == 0
        || cfg.pullback_days == 0
        || cfg.exit_window == 0
        || cfg.sma_window == 0
        || cfg.max_hold == 0
    {
        return Err("S050 lookback windows and max hold must be greater than zero".into());
    }
    if !cfg.pullback_distance.is_finite() || !(0.0..1.0).contains(&cfg.pullback_distance) {
        return Err("S050 pullback distance must be in [0, 1)".into());
    }
    if !matches!(cfg.strategy_mode.as_str(), "baseline" | "avoid_earnings") {
        return Err(format!(
            "unknown S050 strategy_mode {:?}",
            cfg.strategy_mode
        ));
    }
    let membership = s_and_p_membership(panel, &cfg.sp500_membership_file)?;
    let earnings = if cfg.strategy_mode == "avoid_earnings" {
        if cfg.earnings_dates_file.trim().is_empty() {
            return Err("S050 avoid_earnings mode requires earnings_dates_file".into());
        }
        Some(read_earnings_dates(&cfg.earnings_dates_file)?)
    } else {
        None
    };
    let mut result = blank_signals(panel, "first_pullback_after_breakout");
    for symbol in 0..panel.symbols.len() {
        let close = &panel.close[symbol];
        let sma_fast = signals::rolling_mean(close, cfg.sma_window);
        let mut breakout = None::<(usize, f64)>;
        for day in 0..panel.dates.len() {
            if !membership[symbol][day] || !close[day].is_finite() {
                continue;
            }
            result.tradable[symbol][day] = true;
            result.pool[symbol][day] = true;
            let prior_high = rolling_high_before(&panel.high[symbol], day, cfg.entry_window);
            if prior_high.is_finite() && close[day] > prior_high {
                breakout = Some((day, panel.high[symbol][day]));
                continue;
            }
            let Some((breakout_day, breakout_high)) = breakout else {
                continue;
            };
            if day.saturating_sub(breakout_day) > cfg.formation_window {
                breakout = None;
                continue;
            }
            if day.saturating_sub(breakout_day) < cfg.pullback_days
                || day < cfg.pullback_days
                || (day - cfg.pullback_days + 1..=day)
                    .any(|pullback_day| pullback_day <= breakout_day)
            {
                continue;
            }
            let first_pullback = day - cfg.pullback_days + 1;
            let consecutive_down = (first_pullback..=day).all(|pullback_day| {
                close[pullback_day].is_finite()
                    && close[pullback_day - 1].is_finite()
                    && close[pullback_day] < close[pullback_day - 1]
            });
            let near_sma = (first_pullback..=day).all(|pullback_day| {
                sma_fast[pullback_day].is_finite()
                    && (close[pullback_day] / sma_fast[pullback_day] - 1.0).abs()
                        <= cfg.pullback_distance
            });
            let earnings_near = earnings.as_ref().is_some_and(|events| {
                let ticker_events = events.get(&canonical_ticker(&panel.symbols[symbol]));
                ticker_events.is_some_and(|dates| {
                    dates.iter().any(|event| {
                        let event_day = panel.dates.partition_point(|date| date < event);
                        event_day < panel.dates.len()
                            && (day..=day.saturating_add(cfg.earnings_window_sessions))
                                .contains(&event_day)
                    })
                })
            });
            if consecutive_down && near_sma && !earnings_near {
                result.signal[symbol][day] = true;
                result.rsi[symbol][day] = breakout_high;
                breakout = None;
            }
        }
    }
    result.benchmark_returns = benchmark_returns(panel, "SPY");
    Ok(result)
}

pub fn s050_trades(
    panel: &Panel,
    sig: &Signals,
    cfg: &Config,
    fill: &str,
) -> Result<Vec<Trade>, String> {
    if !matches!(fill, "close" | "next_open") {
        return Err(format!("S050 does not support fill {fill:?}"));
    }
    let mut trades = Vec::new();
    for symbol in 0..panel.symbols.len() {
        let cost = cfg.cost_for_symbol(&panel.symbols[symbol]);
        let sma_exit = signals::rolling_mean(&panel.close[symbol], cfg.exit_window);
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
            if entry >= panel.dates.len() {
                continue;
            }
            let entry_px = if fill == "next_open" {
                panel.open[symbol][entry]
            } else {
                panel.close[symbol][entry]
            };
            let breakout_high = sig.rsi[symbol][signal_day];
            if !entry_px.is_finite() || entry_px <= 0.0 || !breakout_high.is_finite() {
                continue;
            }
            let final_session = (entry + cfg.max_hold - 1).min(panel.dates.len() - 1);
            let mut exit_idx = final_session;
            let mut exit_px = panel.close[symbol][final_session];
            let mut reason = "time";
            for day in entry + 1..=final_session {
                let close = panel.close[symbol][day];
                if !close.is_finite() {
                    continue;
                }
                let exit_day = if fill == "next_open" { day + 1 } else { day };
                if close > breakout_high {
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
                    exit_idx = exit_day;
                    exit_px = candidate_px;
                    reason = "breakout_confirmed";
                    break;
                }
                if sma_exit[day].is_finite() && close < sma_exit[day] {
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
                    exit_idx = exit_day;
                    exit_px = candidate_px;
                    reason = "sma_exit";
                    break;
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
                signal_value: breakout_high,
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

fn rebalance_days(panel: &Panel, frequency: &str) -> Result<Vec<usize>, String> {
    if !matches!(frequency, "monthly" | "weekly") {
        return Err(format!("unsupported rebalance frequency {frequency:?}"));
    }
    let rows = panel.dates.len();
    Ok((0..rows)
        .filter(|&day| {
            day + 1 == rows
                || if frequency == "monthly" {
                    let current = panel.dates[day];
                    let next = panel.dates[day + 1];
                    current.year() != next.year() || current.month() != next.month()
                } else {
                    panel.dates[day].iso_week() != panel.dates[day + 1].iso_week()
                }
        })
        .collect())
}

fn momentum_matrix(panel: &Panel, window: usize, min_history: usize) -> Vec<Vec<f64>> {
    panel
        .close
        .iter()
        .map(|close| {
            let mut values = vec![f64::NAN; panel.dates.len()];
            let mut history = 0usize;
            for day in 0..panel.dates.len() {
                if close[day].is_finite() {
                    history += 1;
                }
                if day >= window
                    && history >= min_history
                    && close[day].is_finite()
                    && close[day - window].is_finite()
                    && close[day - window] > 0.0
                {
                    values[day] = close[day] / close[day - window] - 1.0;
                }
            }
            values
        })
        .collect()
}

fn blank_rebalance_signals(panel: &Panel, label: &str) -> Signals {
    let mut result = blank_signals(panel, label);
    result.tradable = panel
        .close
        .iter()
        .map(|close| close.iter().map(|value| value.is_finite()).collect())
        .collect();
    result.pool.clone_from(&result.tradable);
    result
}

const S044_ASSETS: [&str; 13] = [
    "SPY", "IWM", "EFA", "TLT", "LQD", "HYG", "EEM", "EMB", "IYR", "GSG", "GLD", "TIP", "BIL",
];
const S044_ASSET_CAPS: [u8; 13] = [4, 2, 4, 4, 4, 4, 4, 4, 4, 2, 2, 10, 10];
const S044_CLASS_GROUPS: [(&[usize], u8); 5] = [
    (&[0, 1, 2], 10),
    (&[3, 4, 5], 10),
    (&[6, 7], 5),
    (&[8, 9, 10], 5),
    (&[11, 12], 20),
];

fn read_weight_grid(path: &str) -> Result<(usize, Vec<u8>), String> {
    if path.trim().is_empty() {
        return Err("S044 requires weight_grid_path".into());
    }
    let path = Path::new(path);
    let bytes = std::fs::read(path).map_err(|error| format!("{}: {error}", path.display()))?;
    if bytes.len() < 10 || &bytes[..6] != b"\x93NUMPY" {
        return Err(format!("{}: invalid NPY magic/header", path.display()));
    }
    let (header_start, header_len): (usize, usize) = match (bytes[6], bytes[7]) {
        (1, 0) => (10, u16::from_le_bytes([bytes[8], bytes[9]]) as usize),
        (2 | 3, 0) if bytes.len() >= 12 => (
            12,
            u32::from_le_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]) as usize,
        ),
        _ => return Err(format!("{}: unsupported NPY version", path.display())),
    };
    let data_start = header_start
        .checked_add(header_len)
        .filter(|&end| end <= bytes.len())
        .ok_or_else(|| format!("{}: truncated NPY header", path.display()))?;
    let header = std::str::from_utf8(&bytes[header_start..data_start])
        .map_err(|error| format!("{}: invalid NPY header encoding: {error}", path.display()))?;
    if !(header.contains("'descr': '|u1'") || header.contains("\"descr\": \"|u1\""))
        || header.contains("'fortran_order': True")
        || header.contains("\"fortran_order\": true")
    {
        return Err(format!(
            "{}: expected a C-order uint8 weight grid",
            path.display()
        ));
    }
    let shape_start = header
        .find("'shape'")
        .or_else(|| header.find("\"shape\""))
        .ok_or_else(|| format!("{}: NPY shape is missing", path.display()))?;
    let shape = &header[shape_start..];
    let open = shape
        .find('(')
        .ok_or_else(|| format!("{}: invalid NPY shape", path.display()))?;
    let close = shape[open + 1..]
        .find(')')
        .map(|index| index + open + 1)
        .ok_or_else(|| format!("{}: invalid NPY shape", path.display()))?;
    let dimensions: Vec<usize> = shape[open + 1..close]
        .split(',')
        .filter_map(|value| {
            let value = value.trim();
            (!value.is_empty())
                .then(|| value.parse::<usize>().ok())
                .flatten()
        })
        .collect();
    if dimensions.len() != 2 || dimensions[1] != S044_ASSETS.len() {
        return Err(format!(
            "{}: expected N x {} weight grid, got {dimensions:?}",
            path.display(),
            S044_ASSETS.len()
        ));
    }
    let expected = dimensions[0]
        .checked_mul(dimensions[1])
        .ok_or_else(|| format!("{}: NPY shape overflows", path.display()))?;
    if bytes.len() - data_start != expected {
        return Err(format!(
            "{}: NPY payload has {} bytes, expected {expected}",
            path.display(),
            bytes.len() - data_start
        ));
    }
    let grid = bytes[data_start..].to_vec();
    for (row, allocation) in grid.chunks_exact(13).enumerate() {
        let total: usize = allocation.iter().map(|&units| units as usize).sum();
        let valid_assets = allocation
            .iter()
            .zip(S044_ASSET_CAPS)
            .all(|(&units, cap)| units <= cap);
        let valid_classes = S044_CLASS_GROUPS.iter().all(|(columns, cap)| {
            columns
                .iter()
                .map(|&column| allocation[column] as usize)
                .sum::<usize>()
                <= *cap as usize
        });
        if total != 20 || !valid_assets || !valid_classes {
            return Err(format!(
                "{} row {row}: allocation violates Efficiente 5 constraints",
                path.display()
            ));
        }
    }
    Ok((dimensions[0], grid))
}

fn s044_weights(
    grid: &[u8],
    rows: usize,
    mean: &[f64; 13],
    covariance: &[[f64; 13]; 13],
    target_vol: f64,
    max_target_vol: f64,
) -> Result<(Vec<f64>, f64, bool), String> {
    let start_step = (target_vol * 100.0).round() as usize;
    let max_step = (max_target_vol * 100.0).floor() as usize;
    if (target_vol * 100.0 - start_step as f64).abs() > 1e-8
        || target_vol <= 0.0
        || max_target_vol < target_vol
        || max_target_vol > 0.5
    {
        return Err("S044 target vol must be a positive whole-percent step up to 50%".into());
    }
    if grid.len() != rows * S044_ASSETS.len() || rows == 0 {
        return Err("S044 weight grid dimensions are invalid".into());
    }
    let workers = std::thread::available_parallelism()
        .map_or(1, std::num::NonZeroUsize::get)
        .min(rows);
    let chunk_size = rows.div_ceil(workers);
    let partials = std::thread::scope(|scope| {
        let mut handles = Vec::new();
        for start in (0..rows).step_by(chunk_size) {
            let end = (start + chunk_size).min(rows);
            handles.push(scope.spawn(move || {
                let mut best_score = vec![f64::NEG_INFINITY; max_step + 1];
                let mut best_row = vec![None; max_step + 1];
                let (mut min_variance, mut min_variance_row) = (f64::INFINITY, start);
                for row in start..end {
                    let q = &grid[row * 13..(row + 1) * 13];
                    let mut variance = 0.0;
                    let mut score = 0.0;
                    for i in 0..13 {
                        let wi = q[i] as f64 * 0.05;
                        score += wi * mean[i];
                        let mut covariance_weight = 0.0;
                        for j in 0..13 {
                            covariance_weight += covariance[i][j] * q[j] as f64 * 0.05;
                        }
                        variance += wi * covariance_weight;
                    }
                    if variance < 0.0 && variance > -1e-12 {
                        variance = 0.0;
                    }
                    if !variance.is_finite() || variance < 0.0 || !score.is_finite() {
                        continue;
                    }
                    if variance < min_variance {
                        min_variance = variance;
                        min_variance_row = row;
                    }
                    let realized_vol = (variance * 252.0).sqrt();
                    let required_step = (realized_vol * 100.0 - 1e-9).ceil() as usize;
                    if required_step <= max_step {
                        let step = required_step.max(start_step);
                        if score > best_score[step]
                            || (score == best_score[step]
                                && best_row[step].is_some_and(|previous| row < previous))
                        {
                            best_score[step] = score;
                            best_row[step] = Some(row);
                        }
                    }
                }
                (best_score, best_row, min_variance, min_variance_row)
            }));
        }
        handles
            .into_iter()
            .map(|handle| handle.join().expect("S044 grid worker panicked"))
            .collect::<Vec<_>>()
    });
    let mut best_score = vec![f64::NEG_INFINITY; max_step + 1];
    let mut best_row = vec![None; max_step + 1];
    let (mut min_variance, mut min_variance_row) = (f64::INFINITY, 0usize);
    for (thread_score, thread_row, thread_variance, thread_min_row) in partials {
        for step in start_step..=max_step {
            if thread_score[step] > best_score[step]
                || (thread_score[step] == best_score[step]
                    && thread_row[step]
                        .is_some_and(|row| best_row[step].is_some_and(|old_row| row < old_row)))
            {
                best_score[step] = thread_score[step];
                best_row[step] = thread_row[step];
            }
        }
        if thread_variance < min_variance
            || (thread_variance == min_variance && thread_min_row < min_variance_row)
        {
            min_variance = thread_variance;
            min_variance_row = thread_min_row;
        }
    }
    let mut running_score = f64::NEG_INFINITY;
    let mut running_row = None;
    for step in start_step..=max_step {
        if best_score[step] > running_score {
            running_score = best_score[step];
            running_row = best_row[step];
        }
        if let Some(row) = running_row {
            let units = &grid[row * 13..(row + 1) * 13];
            return Ok((
                units.iter().map(|&value| value as f64 * 0.05).collect(),
                step as f64 / 100.0,
                false,
            ));
        }
    }
    if !min_variance.is_finite() {
        return Err("S044 weight grid contains no feasible allocation".into());
    }
    let units = &grid[min_variance_row * 13..(min_variance_row + 1) * 13];
    Ok((
        units.iter().map(|&value| value as f64 * 0.05).collect(),
        (max_step + 1) as f64 / 100.0,
        true,
    ))
}

pub fn compute_s044_signals(panel: &Panel, cfg: &Config) -> Result<Signals, String> {
    if cfg.formation_window < 2 {
        return Err("S044 requires at least two return observations".into());
    }
    let asset_columns: Vec<_> = S044_ASSETS
        .iter()
        .map(|asset| {
            panel
                .symbols
                .iter()
                .position(|symbol| symbol == asset)
                .ok_or_else(|| format!("S044 requires ETF {asset} in the panel"))
        })
        .collect::<Result<_, _>>()?;
    let (grid_rows, grid) = read_weight_grid(&cfg.weight_grid_path)?;
    let n_rows = panel.dates.len();
    let returns: Vec<Vec<f64>> = asset_columns
        .iter()
        .map(|&symbol| {
            let close = &panel.close[symbol];
            (0..n_rows)
                .map(|day| {
                    if day == 0 || !close[day].is_finite() || !close[day - 1].is_finite() {
                        f64::NAN
                    } else {
                        close[day] / close[day - 1] - 1.0
                    }
                })
                .collect()
        })
        .collect();
    let mut result = blank_rebalance_signals(panel, "constrained_mean_variance");
    let mut target_weights = vec![vec![0.0; n_rows]; panel.symbols.len()];
    let days = rebalance_days(panel, &cfg.rebalance)?;
    for day in days {
        if day + 1 < cfg.formation_window {
            continue;
        }
        let start = day + 1 - cfg.formation_window;
        let mut sample = vec![[0.0; 13]; cfg.formation_window];
        let mut valid = true;
        for row in 0..cfg.formation_window {
            for asset in 0..13 {
                let value = returns[asset][start + row];
                if !value.is_finite() {
                    valid = false;
                    break;
                }
                sample[row][asset] = value;
            }
            if !valid {
                break;
            }
        }
        if !valid {
            continue;
        }
        let mut mean = [0.0; 13];
        for row in &sample {
            for asset in 0..13 {
                mean[asset] += row[asset] / cfg.formation_window as f64;
            }
        }
        let mut covariance = [[0.0; 13]; 13];
        for i in 0..13 {
            for j in 0..13 {
                covariance[i][j] = sample
                    .iter()
                    .map(|row| (row[i] - mean[i]) * (row[j] - mean[j]))
                    .sum::<f64>()
                    / (cfg.formation_window - 1) as f64;
            }
        }
        let (weights, _, _) = s044_weights(
            &grid,
            grid_rows,
            &mean,
            &covariance,
            cfg.target_vol,
            cfg.max_target_vol,
        )?;
        for (asset, &symbol) in asset_columns.iter().enumerate() {
            target_weights[symbol][day] = weights[asset];
            result.rsi[symbol][day] = mean[asset];
            result.signal[symbol][day] = weights[asset] > 0.0;
        }
    }
    result.trade_weights = Some(target_weights);
    result.benchmark_returns = benchmark_returns(panel, "SPY");
    Ok(result)
}

fn rolling_std(values: &[f64], day: usize, window: usize) -> f64 {
    if window < 2 || day + 1 < window {
        return f64::NAN;
    }
    let slice = &values[day + 1 - window..=day];
    if slice.iter().any(|value| !value.is_finite()) {
        return f64::NAN;
    }
    let mean = slice.iter().sum::<f64>() / window as f64;
    (slice
        .iter()
        .map(|value| (value - mean).powi(2))
        .sum::<f64>()
        / (window - 1) as f64)
        .sqrt()
}

fn cross_section_zscore(values: &[f64], eligible: &[usize]) -> Vec<f64> {
    let valid: Vec<_> = eligible
        .iter()
        .map(|&symbol| values[symbol])
        .filter(|value| value.is_finite())
        .collect();
    if valid.len() < 2 {
        return vec![f64::NAN; values.len()];
    }
    let mean = valid.iter().sum::<f64>() / valid.len() as f64;
    let std = (valid
        .iter()
        .map(|value| (value - mean).powi(2))
        .sum::<f64>()
        / (valid.len() - 1) as f64)
        .sqrt();
    if std <= 0.0 || !std.is_finite() {
        return vec![0.0; values.len()];
    }
    values
        .iter()
        .map(|value| {
            if value.is_finite() {
                (value - mean) / std
            } else {
                f64::NAN
            }
        })
        .collect()
}

pub fn compute_s045_signals(panel: &Panel, cfg: &Config) -> Result<Signals, String> {
    if !matches!(cfg.factor_design.as_str(), "S045a" | "S045b") {
        return Err(format!(
            "unknown S045 factor design {:?}",
            cfg.factor_design
        ));
    }
    if cfg.rebalance != "monthly" {
        return Err("S045 requires monthly rebalancing".into());
    }
    let n_rows = panel.dates.len();
    let etf_symbols = crate::config::split_csv(&cfg.etf_symbols);
    let stock_columns: Vec<_> = panel
        .symbols
        .iter()
        .enumerate()
        .filter_map(|(column, symbol)| (!etf_symbols.contains(symbol)).then_some(column))
        .collect();
    if stock_columns.is_empty() {
        return Err("S045 requires a stock universe".into());
    }
    let ranks = if cfg.rank_cache_dir.trim().is_empty() {
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
    let mut return21 = vec![vec![f64::NAN; n_rows]; panel.symbols.len()];
    let mut return1260 = vec![vec![f64::NAN; n_rows]; panel.symbols.len()];
    let mut high52_distance = vec![vec![f64::NAN; n_rows]; panel.symbols.len()];
    let mut vol252 = vec![vec![f64::NAN; n_rows]; panel.symbols.len()];
    for symbol in 0..panel.symbols.len() {
        let close = &panel.close[symbol];
        let returns: Vec<_> = (0..n_rows)
            .map(|day| {
                if day == 0 || !close[day].is_finite() || !close[day - 1].is_finite() {
                    f64::NAN
                } else {
                    close[day] / close[day - 1] - 1.0
                }
            })
            .collect();
        for day in 0..n_rows {
            if day >= 21 && close[day].is_finite() && close[day - 21].is_finite() {
                return21[symbol][day] = close[day] / close[day - 21] - 1.0;
            }
            if day >= 1260 && close[day].is_finite() && close[day - 1260].is_finite() {
                return1260[symbol][day] = close[day] / close[day - 1260] - 1.0;
            }
            if day >= 251 {
                let high = panel.high[symbol][day + 1 - 252..=day]
                    .iter()
                    .copied()
                    .filter(|value| value.is_finite())
                    .fold(f64::NEG_INFINITY, f64::max);
                if high.is_finite() && high > 0.0 && close[day].is_finite() {
                    high52_distance[symbol][day] = close[day] / high - 1.0;
                }
            }
            vol252[symbol][day] = rolling_std(&returns, day, 252);
        }
    }
    let spy = panel
        .symbols
        .iter()
        .position(|symbol| symbol == "SPY")
        .ok_or_else(|| "S045 requires SPY in the panel for its market gate".to_string())?;
    let spy_sma = signals::rolling_mean(&panel.close[spy], 200);
    let spy_returns: Vec<_> = (0..n_rows)
        .map(|day| {
            if day == 0
                || !panel.close[spy][day].is_finite()
                || !panel.close[spy][day - 1].is_finite()
            {
                f64::NAN
            } else {
                panel.close[spy][day] / panel.close[spy][day - 1] - 1.0
            }
        })
        .collect();
    let spy_vol = (0..n_rows)
        .map(|day| rolling_std(&spy_returns, day, 60))
        .collect::<Vec<_>>();
    let days = rebalance_days(panel, "monthly")?;
    let mut result = blank_rebalance_signals(panel, "price_characteristic_blend");
    let mut weights = vec![vec![0.0; n_rows]; panel.symbols.len()];
    let mut prior_holdings = Vec::<usize>::new();
    for day in days {
        let eligible: Vec<_> = stock_columns
            .iter()
            .copied()
            .filter(|&symbol| {
                ranks[symbol][day].is_finite()
                    && ranks[symbol][day] <= cfg.universe_top_n as f64
                    && panel.close[symbol][day].is_finite()
            })
            .collect();
        if eligible.len() < 10 {
            continue;
        }
        let vol_score: Vec<_> = vol252.iter().map(|series| -series[day]).collect();
        let high_score: Vec<_> = high52_distance.iter().map(|series| series[day]).collect();
        let reversal_score: Vec<_> = return21.iter().map(|series| -series[day]).collect();
        let (vol_z, high_z, reversal_z) = (
            cross_section_zscore(&vol_score, &eligible),
            cross_section_zscore(&high_score, &eligible),
            cross_section_zscore(&reversal_score, &eligible),
        );
        let composite: Vec<f64> = (0..panel.symbols.len())
            .map(|symbol| {
                let value_z = -return1260[symbol][day];
                let factors = if cfg.factor_design == "S045a" {
                    [vol_z[symbol], value_z, high_z[symbol], reversal_z[symbol]]
                } else {
                    [vol_z[symbol], high_z[symbol], reversal_z[symbol], 0.0]
                };
                if factors[..if cfg.factor_design == "S045a" { 4 } else { 3 }]
                    .iter()
                    .all(|value| value.is_finite())
                {
                    if cfg.factor_design == "S045a" {
                        factors.iter().sum::<f64>() / 4.0
                    } else {
                        factors[..3].iter().sum::<f64>() / 3.0
                    }
                } else {
                    f64::NAN
                }
            })
            .collect();
        let mut ranked = eligible;
        ranked.retain(|&symbol| composite[symbol].is_finite());
        ranked.sort_by(|&left, &right| {
            composite[right]
                .total_cmp(&composite[left])
                .then_with(|| panel.symbols[left].cmp(&panel.symbols[right]))
        });
        if ranked.len() < 10 {
            continue;
        }
        let top30: std::collections::BTreeSet<_> = ranked.iter().take(30).copied().collect();
        let mut selected: Vec<_> = prior_holdings
            .iter()
            .copied()
            .filter(|symbol| top30.contains(symbol))
            .collect();
        let mut lower_vol_candidates: Vec<_> = ranked.iter().take(20).copied().collect();
        lower_vol_candidates.sort_by(|&left, &right| {
            vol252[left][day]
                .total_cmp(&vol252[right][day])
                .then_with(|| composite[right].total_cmp(&composite[left]))
        });
        for symbol in lower_vol_candidates {
            if selected.len() >= 10 {
                break;
            }
            if !selected.contains(&symbol) {
                selected.push(symbol);
            }
        }
        if selected.len() < 10 {
            continue;
        }
        let portfolio_returns: Vec<_> = (day.saturating_sub(59)..=day)
            .filter_map(|row| {
                let values: Option<Vec<_>> = selected
                    .iter()
                    .map(|&symbol| {
                        (row > 0
                            && panel.close[symbol][row].is_finite()
                            && panel.close[symbol][row - 1].is_finite())
                        .then_some(panel.close[symbol][row] / panel.close[symbol][row - 1] - 1.0)
                    })
                    .collect();
                values.map(|values| values.iter().sum::<f64>() / values.len() as f64)
            })
            .collect();
        let realized = if portfolio_returns.len() >= 40 {
            let mean = portfolio_returns.iter().sum::<f64>() / portfolio_returns.len() as f64;
            (portfolio_returns
                .iter()
                .map(|value| (value - mean).powi(2))
                .sum::<f64>()
                / (portfolio_returns.len() - 1) as f64
                * 252.0)
                .sqrt()
        } else {
            f64::NAN
        };
        let scale = if realized > 0.0 && realized.is_finite() {
            (cfg.target_vol / realized).min(cfg.max_leverage).min(1.0)
        } else {
            0.0
        };
        let regime = if cfg.factor_design == "S045a" {
            let vol_threshold = {
                let mut history: Vec<_> = (0..=day)
                    .filter_map(|row| spy_vol[row].is_finite().then_some(spy_vol[row]))
                    .collect();
                if history.is_empty() {
                    f64::NAN
                } else {
                    history.sort_by(f64::total_cmp);
                    history[(history.len() * 4 / 5).min(history.len() - 1)]
                }
            };
            if panel.close[spy][day] > spy_sma[day] && spy_vol[day] <= vol_threshold {
                1.0
            } else if panel.close[spy][day] > spy_sma[day] {
                0.7
            } else {
                0.4
            }
        } else if panel.close[spy][day] > spy_sma[day] {
            1.0
        } else {
            0.0
        };
        let exposure = scale * regime;
        let weight = exposure / selected.len() as f64;
        for &symbol in &selected {
            weights[symbol][day] = weight;
            result.signal[symbol][day] = weight > 0.0;
            result.rsi[symbol][day] = composite[symbol];
        }
        prior_holdings.clear();
        prior_holdings.append(&mut selected);
    }
    result.trade_weights = Some(weights);
    result.benchmark_returns = benchmark_returns(panel, "SPY");
    Ok(result)
}

fn parse_french_month(value: &str) -> Result<NaiveDate, String> {
    if value.len() != 6 || !value.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(format!("invalid French monthly date {value:?}"));
    }
    let year = value[..4]
        .parse::<i32>()
        .map_err(|error| format!("invalid French monthly year {value:?}: {error}"))?;
    let month = value[4..]
        .parse::<u32>()
        .map_err(|error| format!("invalid French monthly month {value:?}: {error}"))?;
    NaiveDate::from_ymd_opt(year, month, 1)
        .and_then(|date| date.checked_add_months(chrono::Months::new(1)))
        .and_then(|date| date.pred_opt())
        .ok_or_else(|| format!("invalid French monthly date {value:?}"))
}

fn monthly_rows(
    path: &str,
    header_prefix: &str,
) -> Result<(Vec<String>, Vec<(NaiveDate, Vec<f64>)>), String> {
    let path_obj = Path::new(path);
    let text = std::fs::read_to_string(path_obj)
        .map_err(|error| format!("{}: {error}", path_obj.display()))?;
    let lines: Vec<_> = text.lines().collect();
    let header_index = lines
        .iter()
        .position(|line| line.trim_start().starts_with(header_prefix))
        .ok_or_else(|| format!("{}: monthly data header not found", path_obj.display()))?;
    let headers: Vec<_> = lines[header_index]
        .split(',')
        .skip(1)
        .map(|column| column.trim().to_string())
        .collect();
    let mut rows = Vec::new();
    for (line_index, line) in lines.iter().enumerate().skip(header_index + 1) {
        let fields: Vec<_> = line.split(',').collect();
        if fields.is_empty() || fields[0].trim().len() != 6 {
            break;
        }
        let date = parse_french_month(fields[0].trim())
            .map_err(|error| format!("{} line {}: {error}", path_obj.display(), line_index + 1))?;
        if fields.len() != headers.len() + 1 {
            return Err(format!(
                "{} line {}: expected {} values, got {}",
                path_obj.display(),
                line_index + 1,
                headers.len(),
                fields.len() - 1
            ));
        }
        let mut values = Vec::with_capacity(headers.len());
        for field in &fields[1..] {
            let value = field.trim().parse::<f64>().map_err(|error| {
                format!(
                    "{} line {}: invalid numeric value {:?}: {error}",
                    path_obj.display(),
                    line_index + 1,
                    field.trim()
                )
            })?;
            values.push(if value <= -99.0 {
                f64::NAN
            } else {
                value / 100.0
            });
        }
        rows.push((date, values));
    }
    if rows.is_empty() || headers.is_empty() {
        return Err(format!("{}: no monthly observations", path_obj.display()));
    }
    Ok((headers, rows))
}

pub fn load_french_industry_panel(path: &str, start_date: &str) -> Result<Panel, String> {
    let start = data::parse_date(start_date)?;
    let (symbols, rows) = monthly_rows(path, ",Agric")?;
    if symbols.len() != 49 {
        return Err(format!(
            "{} must contain the 49-industry monthly series; found {}",
            path,
            symbols.len()
        ));
    }
    let mut dates = Vec::new();
    let mut returns = vec![Vec::new(); symbols.len()];
    for (date, values) in rows.into_iter().filter(|(date, _)| *date >= start) {
        dates.push(date);
        for (symbol, value) in values.into_iter().enumerate() {
            returns[symbol].push(value);
        }
    }
    if dates.len() < 3 {
        return Err(format!(
            "{path}: fewer than three months after {start_date}"
        ));
    }
    let close: Vec<Vec<f64>> = returns
        .into_iter()
        .map(|series| {
            let mut last_price = 100.0;
            series
                .into_iter()
                .map(|value| {
                    if value.is_finite() {
                        last_price *= 1.0 + value;
                        last_price
                    } else {
                        f64::NAN
                    }
                })
                .collect()
        })
        .collect();
    let volume = close
        .iter()
        .map(|series| {
            series
                .iter()
                .map(|value| if value.is_finite() { 1.0 } else { f64::NAN })
                .collect()
        })
        .collect();
    Ok(Panel {
        dates,
        symbols,
        open: close.clone(),
        high: close.clone(),
        low: close.clone(),
        close,
        volume,
    })
}

fn read_market_factor(path: &str) -> Result<HashMap<NaiveDate, f64>, String> {
    let (_, rows) = monthly_rows(path, ",Mkt-RF")?;
    Ok(rows
        .into_iter()
        .filter_map(|(date, values)| values.first().copied().map(|value| (date, value)))
        .collect())
}

pub fn compute_s046_signals(panel: &Panel, cfg: &Config) -> Result<Signals, String> {
    if cfg.analog_count == 0 || cfg.formation_window < 2 {
        return Err("S046 analog count and standardization window must be positive".into());
    }
    if !matches!(cfg.strategy_mode.as_str(), "long_short" | "long_only") {
        return Err(format!(
            "unknown S046 strategy_mode {:?}",
            cfg.strategy_mode
        ));
    }
    let factors = read_market_factor(&cfg.market_factor_file)?;
    let market: Vec<_> = panel
        .dates
        .iter()
        .map(|date| factors.get(date).copied().unwrap_or(f64::NAN))
        .collect();
    let mut state = vec![f64::NAN; panel.dates.len()];
    for day in cfg.formation_window..panel.dates.len() {
        let sample = &market[day - cfg.formation_window..day];
        if sample.iter().any(|value| !value.is_finite()) || !market[day].is_finite() {
            continue;
        }
        let mean = sample.iter().sum::<f64>() / sample.len() as f64;
        let std = (sample
            .iter()
            .map(|value| (value - mean).powi(2))
            .sum::<f64>()
            / (sample.len() - 1) as f64)
            .sqrt();
        if std > 0.0 && std.is_finite() {
            state[day] = (market[day] - mean) / std;
        }
    }
    let industry_returns: Vec<Vec<f64>> = panel
        .close
        .iter()
        .map(|close| {
            (0..panel.dates.len())
                .map(|day| {
                    if day == 0
                        || !close[day].is_finite()
                        || !close[day - 1].is_finite()
                        || close[day - 1] <= 0.0
                    {
                        f64::NAN
                    } else {
                        close[day] / close[day - 1] - 1.0
                    }
                })
                .collect()
        })
        .collect();
    let mut result = blank_rebalance_signals(panel, "market_state_analog_industries");
    let mut weights = vec![vec![0.0; panel.dates.len()]; panel.symbols.len()];
    let days = rebalance_days(panel, "monthly")?;
    for day in days {
        if !state[day].is_finite() || day < cfg.formation_window + 1 {
            continue;
        }
        let mut analogs: Vec<_> = (cfg.formation_window..day.saturating_sub(1))
            .filter(|&analog| state[analog].is_finite())
            .map(|analog| (analog, (state[analog] - state[day]).abs()))
            .collect();
        analogs.sort_by(|(day_a, distance_a), (day_b, distance_b)| {
            distance_a
                .total_cmp(distance_b)
                .then_with(|| day_a.cmp(day_b))
        });
        analogs.truncate(cfg.analog_count.min(analogs.len()));
        if analogs.len() < cfg.analog_count {
            continue;
        }
        let mut expected = vec![f64::NAN; panel.symbols.len()];
        for symbol in 0..panel.symbols.len() {
            let samples: Vec<_> = analogs
                .iter()
                .filter_map(|(analog, _)| {
                    industry_returns[symbol][analog + 1]
                        .is_finite()
                        .then_some(industry_returns[symbol][analog + 1])
                })
                .collect();
            if samples.len() * 2 >= cfg.analog_count {
                expected[symbol] = samples.iter().sum::<f64>() / samples.len() as f64;
            }
        }
        let mut ranked: Vec<_> = (0..panel.symbols.len())
            .filter(|&symbol| expected[symbol].is_finite() && panel.close[symbol][day].is_finite())
            .collect();
        ranked.sort_by(|&left, &right| {
            expected[right]
                .total_cmp(&expected[left])
                .then_with(|| panel.symbols[left].cmp(&panel.symbols[right]))
        });
        if ranked.len() < 3 {
            continue;
        }
        let breadth = (ranked.len() / 3).max(1);
        let long_weight = 1.0 / breadth as f64;
        let short_weight = if cfg.strategy_mode == "long_short" {
            -1.0 / breadth as f64
        } else {
            0.0
        };
        for &symbol in ranked.iter().take(breadth) {
            weights[symbol][day] = long_weight;
            result.signal[symbol][day] = true;
            result.rsi[symbol][day] = expected[symbol];
        }
        if short_weight < 0.0 {
            for &symbol in ranked.iter().rev().take(breadth) {
                weights[symbol][day] = short_weight;
                result.signal[symbol][day] = true;
                result.rsi[symbol][day] = expected[symbol];
            }
        }
    }
    result.trade_weights = Some(weights);
    let benchmark = (0..panel.dates.len())
        .map(|day| {
            let values: Vec<_> = industry_returns
                .iter()
                .filter_map(|series| series[day].is_finite().then_some(series[day]))
                .collect();
            if values.is_empty() {
                0.0
            } else {
                values.iter().sum::<f64>() / values.len() as f64
            }
        })
        .collect();
    result.benchmark_returns = Some(benchmark);
    Ok(result)
}

pub fn compute_s043_signals(panel: &Panel, cfg: &Config) -> Result<Signals, String> {
    if cfg.mom_window == 0 || cfg.breadth == 0 {
        return Err("S043 momentum lookback and breadth must be positive".into());
    }
    let defensive = panel
        .symbols
        .iter()
        .position(|symbol| symbol == &cfg.defensive_asset)
        .ok_or_else(|| {
            format!(
                "S043 defensive ETF {:?} is not in the panel",
                cfg.defensive_asset
            )
        })?;
    let returns = momentum_matrix(panel, cfg.mom_window, cfg.min_history);
    let days = rebalance_days(panel, &cfg.rebalance)?;
    let mut result = blank_rebalance_signals(panel, "multi_asset_momentum");
    result.rsi = returns.clone();
    let mut weights = vec![vec![0.0; panel.dates.len()]; panel.symbols.len()];
    for day in days {
        let mut candidates: Vec<_> = (0..panel.symbols.len())
            .filter(|&symbol| symbol != defensive && returns[symbol][day].is_finite())
            .collect();
        candidates.sort_by(|&left, &right| {
            returns[right][day]
                .total_cmp(&returns[left][day])
                .then_with(|| panel.symbols[left].cmp(&panel.symbols[right]))
        });
        candidates.truncate(cfg.breadth);
        if candidates.is_empty() {
            continue;
        }
        let slot_weight = 1.0 / candidates.len() as f64;
        for symbol in candidates {
            let target = if returns[symbol][day] < 0.0 {
                defensive
            } else {
                symbol
            };
            weights[target][day] += slot_weight;
        }
    }
    for symbol in 0..panel.symbols.len() {
        for day in 0..panel.dates.len() {
            result.signal[symbol][day] = weights[symbol][day] > 0.0;
        }
    }
    result.trade_weights = Some(weights);
    result.benchmark_returns = benchmark_returns(panel, "SPY");
    Ok(result)
}

pub fn compute_s047_signals(panel: &Panel, cfg: &Config) -> Result<Signals, String> {
    if cfg.mom_window == 0 {
        return Err("S047 momentum lookback must be positive".into());
    }
    if !matches!(cfg.strategy_mode.as_str(), "absolute" | "gem" | "sma10") {
        return Err(format!(
            "unknown S047 strategy_mode {:?}",
            cfg.strategy_mode
        ));
    }
    let cash = panel
        .symbols
        .iter()
        .position(|symbol| symbol == &cfg.defensive_asset)
        .ok_or_else(|| {
            format!(
                "S047 defensive ETF {:?} is not in the panel",
                cfg.defensive_asset
            )
        })?;
    let spy = panel
        .symbols
        .iter()
        .position(|symbol| symbol == "SPY")
        .ok_or_else(|| "S047 requires SPY in the panel".to_string())?;
    let efa = panel.symbols.iter().position(|symbol| symbol == "EFA");
    let agg = panel.symbols.iter().position(|symbol| symbol == "AGG");
    if cfg.strategy_mode == "gem" && (efa.is_none() || agg.is_none()) {
        return Err("S047 GEM mode requires EFA and AGG in the panel".into());
    }
    let returns = momentum_matrix(panel, cfg.mom_window, cfg.min_history);
    let spy_sma = signals::rolling_mean(&panel.close[spy], cfg.sma_window);
    let days = rebalance_days(panel, &cfg.rebalance)?;
    let mut result = blank_rebalance_signals(panel, "absolute_dual_momentum");
    result.rsi = returns.clone();
    let mut weights = vec![vec![0.0; panel.dates.len()]; panel.symbols.len()];
    for day in days {
        let selected = if cfg.strategy_mode == "sma10" {
            if !panel.close[spy][day].is_finite() || !spy_sma[day].is_finite() {
                continue;
            }
            if panel.close[spy][day] > spy_sma[day] {
                spy
            } else {
                cash
            }
        } else if !returns[cash][day].is_finite() || !returns[spy][day].is_finite() {
            continue;
        } else if cfg.strategy_mode == "absolute" {
            if returns[spy][day] > returns[cash][day] {
                spy
            } else {
                cash
            }
        } else {
            let international = efa.unwrap();
            if !returns[international][day].is_finite() || !returns[agg.unwrap()][day].is_finite() {
                continue;
            }
            let relative = if returns[spy][day] >= returns[international][day] {
                spy
            } else {
                international
            };
            if returns[relative][day] > returns[cash][day] {
                relative
            } else {
                agg.unwrap()
            }
        };
        weights[selected][day] = 1.0;
        result.signal[selected][day] = true;
    }
    result.trade_weights = Some(weights);
    result.benchmark_returns = benchmark_returns(panel, "SPY");
    Ok(result)
}

pub fn weighted_rebalance_trades(
    panel: &Panel,
    sig: &Signals,
    cfg: &Config,
    fill: &str,
) -> Result<Vec<Trade>, String> {
    if !matches!(fill, "close" | "next_open") {
        return Err(format!("{} does not support fill {fill:?}", cfg.strategy));
    }
    let weights = sig
        .trade_weights
        .as_ref()
        .ok_or_else(|| format!("{} requires target weights", cfg.strategy))?;
    if weights.len() != panel.symbols.len()
        || weights
            .iter()
            .any(|series| series.len() != panel.dates.len())
    {
        return Err("rebalance weight dimensions do not match the panel".into());
    }
    let days = rebalance_days(panel, &cfg.rebalance)?;
    let is_open = fill == "next_open";
    let mut trades = Vec::new();
    for (period, &signal_day) in days.iter().enumerate() {
        let entry = signal_day + usize::from(is_open);
        if entry >= panel.dates.len() || (!is_open && entry + 1 == panel.dates.len()) {
            continue;
        }
        let boundary = days
            .get(period + 1)
            .copied()
            .unwrap_or(panel.dates.len() - 1);
        let exit = if is_open {
            boundary.saturating_add(1).min(panel.dates.len() - 1)
        } else {
            boundary
        };
        if exit < entry {
            continue;
        }
        for symbol in 0..panel.symbols.len() {
            let weight = weights[symbol][signal_day];
            if !weight.is_finite() || weight == 0.0 {
                continue;
            }
            let entry_px = if is_open {
                panel.open[symbol][entry]
            } else {
                panel.close[symbol][signal_day]
            };
            let exit_px = if is_open && panel.open[symbol][exit].is_finite() {
                panel.open[symbol][exit]
            } else {
                panel.close[symbol][exit]
            };
            if !entry_px.is_finite() || entry_px <= 0.0 || !exit_px.is_finite() {
                continue;
            }
            let prior_weight = period
                .checked_sub(1)
                .map(|previous| weights[symbol][days[previous]])
                .unwrap_or(0.0);
            let next_weight = days
                .get(period + 1)
                .filter(|&&next_day| next_day + 1 < panel.dates.len())
                .map(|&next_day| weights[symbol][next_day])
                .unwrap_or(0.0);
            let entry_turnover = (weight - prior_weight).abs();
            let exit_turnover = if next_weight == 0.0 {
                weight.abs()
            } else {
                0.0
            };
            let gross = exit_px / entry_px - 1.0;
            let cost = cfg.cost_for_symbol(&panel.symbols[symbol]);
            let direction = weight.signum();
            let borrow_cost = if weight < 0.0 && exit > entry {
                cfg.borrow_bps_annual / 10_000.0 / cfg.periods_per_year as f64
            } else {
                0.0
            };
            trades.push(Trade {
                sym_idx: symbol,
                signal_idx: signal_day,
                entry_idx: entry,
                exit_idx: exit,
                entry_px,
                exit_px,
                ret: direction * gross
                    - cost * (entry_turnover + exit_turnover) / weight.abs()
                    - borrow_cost,
                ret_gross: direction * gross,
                exit_reason: if period + 1 == days.len() {
                    "censored"
                } else {
                    "rebalance"
                },
                hold: exit.saturating_sub(entry),
                signal_value: sig.rsi[symbol][signal_day],
                weight,
                entry_turnover,
                exit_turnover,
                weighted: true,
                borrow_cost_per_bar: borrow_cost / (exit - entry + 1) as f64,
            });
        }
    }
    trades.sort_by(|a, b| {
        (a.entry_idx, &panel.symbols[a.sym_idx]).cmp(&(b.entry_idx, &panel.symbols[b.sym_idx]))
    });
    Ok(trades)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config() -> Config {
        serde_yaml::from_str("partition_split_date: '2021-01-01'\n")
            .expect("default Family 4 config")
    }

    fn panel() -> Panel {
        let dates = (0..9)
            .map(|day| NaiveDate::from_ymd_opt(2020, 1, 1).unwrap() + chrono::Days::new(day))
            .collect();
        let high = vec![8.0, 9.0, 10.0, 9.0, 11.6, 12.0, 14.0, 14.0, 14.0];
        let low = vec![7.0, 8.0, 9.0, 8.0, 10.5, 11.0, 12.0, 13.0, 13.0];
        let close = vec![7.5, 8.5, 9.5, 8.5, 11.5, 11.8, 13.8, 13.5, 13.5];
        Panel {
            dates,
            symbols: vec!["AAA".into()],
            open: vec![vec![7.5, 8.5, 9.5, 8.5, 11.0, 11.5, 12.0, 13.0, 13.0]],
            high: vec![high],
            low: vec![low],
            close: vec![close],
            volume: vec![vec![
                100.0, 100.0, 100.0, 100.0, 200.0, 100.0, 100.0, 100.0, 100.0,
            ]],
        }
    }

    fn membership_file() -> String {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("deepthrust-sp500-{unique}.csv"));
        std::fs::write(&path, "ticker,start_date,end_date\nAAA,2020-01-01,\n").unwrap();
        path.to_string_lossy().into_owned()
    }

    fn etf_panel(symbols: &[&str], closes: &[Vec<f64>]) -> Panel {
        let dates = [
            NaiveDate::from_ymd_opt(2020, 1, 31).unwrap(),
            NaiveDate::from_ymd_opt(2020, 2, 28).unwrap(),
            NaiveDate::from_ymd_opt(2020, 3, 31).unwrap(),
            NaiveDate::from_ymd_opt(2020, 4, 30).unwrap(),
        ];
        let symbols = symbols.iter().map(|symbol| (*symbol).to_string()).collect();
        let prices = closes.to_vec();
        let mut high = prices.clone();
        let mut low = prices.clone();
        let open = prices.clone();
        for series in &mut high {
            for value in series {
                *value += 0.1;
            }
        }
        for series in &mut low {
            for value in series {
                *value -= 0.1;
            }
        }
        Panel {
            dates: dates.to_vec(),
            symbols,
            open,
            high,
            low,
            close: prices,
            volume: vec![vec![100.0; dates.len()]; closes.len()],
        }
    }

    #[test]
    fn s040_uses_point_in_time_membership_and_next_open_bracket() {
        let path = membership_file();
        let mut cfg = config();
        cfg.sp500_membership_file.clone_from(&path);
        cfg.entry_window = 3;
        cfg.atr_len = 2;
        cfg.gap_volume_window = 3;
        cfg.gap_volume_multiple = 1.5;
        cfg.gap_volume_mode = "high".into();
        cfg.gap_max_hold = 8;
        cfg.cost_bps_per_side = 0.0;
        let panel = panel();
        let sig = compute_s040_signals(&panel, &cfg).unwrap();
        assert!(sig.signal[0][4]);
        let trades = s040_trades(&panel, &sig, &cfg, "next_open").unwrap();
        assert_eq!(trades.len(), 1);
        assert_eq!(trades[0].entry_idx, 5);
        assert_eq!(trades[0].exit_idx, 6);
        assert_eq!(trades[0].exit_reason, "target");
        assert_eq!(trades[0].exit_px, 13.5);
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn darvas_pivots_need_three_following_sessions() {
        let highs = [7.0, 8.0, 9.0, 10.0, 9.0, 8.0, 9.0];
        let lows = [10.0, 9.0, 8.0, 7.0, 8.0, 9.0, 8.0];
        assert!(pivot_high(&highs, 3));
        assert!(pivot_low(&lows, 3));
        assert!(!pivot_high(&highs[..6], 3));
        assert!(!pivot_low(&lows[..6], 3));
    }

    #[test]
    fn s043_replaces_negative_momentum_slots_with_cash() {
        let panel = etf_panel(
            &["AAA", "BBB", "SHY", "SPY"],
            &[
                vec![10.0, 10.0, 11.0, 12.0],
                vec![10.0, 10.0, 9.0, 8.0],
                vec![10.0, 10.0, 10.1, 10.2],
                vec![10.0, 10.0, 9.0, 7.0],
            ],
        );
        let mut cfg = config();
        cfg.mom_window = 2;
        cfg.min_history = 2;
        cfg.breadth = 2;
        cfg.defensive_asset = "SHY".into();
        let sig = compute_s043_signals(&panel, &cfg).unwrap();
        let weights = sig.trade_weights.unwrap();
        assert_eq!(weights[0][3], 0.5);
        assert_eq!(weights[1][3], 0.0);
        assert_eq!(weights[2][3], 0.5);
    }

    #[test]
    fn weighted_short_trade_returns_include_direction_and_borrow() {
        let panel = etf_panel(&["AAA"], &[vec![100.0, 90.0, 90.0, 90.0]]);
        let mut cfg = config();
        cfg.rebalance = "monthly".into();
        cfg.cost_bps_per_side = 0.0;
        cfg.borrow_bps_annual = 1200.0;
        cfg.periods_per_year = 12;
        let mut sig = blank_rebalance_signals(&panel, "test");
        sig.trade_weights = Some(vec![vec![-1.0, 0.0, 0.0, 0.0]]);
        let trades = weighted_rebalance_trades(&panel, &sig, &cfg, "close").unwrap();
        assert_eq!(trades.len(), 1);
        assert!((trades[0].ret_gross - 0.1).abs() < 1e-12);
        assert!((trades[0].ret - 0.09).abs() < 1e-12);
        assert!((trades[0].borrow_cost_per_bar * 2.0 - 0.01).abs() < 1e-12);
    }

    #[test]
    fn weighted_rebalance_closes_at_end_without_a_phantom_new_trade() {
        let panel = etf_panel(&["AAA"], &[vec![100.0; 4]]);
        let mut cfg = config();
        cfg.rebalance = "monthly".into();
        let mut sig = blank_rebalance_signals(&panel, "test");
        sig.trade_weights = Some(vec![vec![1.0; 4]]);
        for fill in ["close", "next_open"] {
            let trades = weighted_rebalance_trades(&panel, &sig, &cfg, fill).unwrap();
            assert!(trades.iter().all(|trade| trade.signal_idx < 3));
            let last = trades.last().unwrap();
            assert_eq!(last.exit_idx, 3);
            assert_eq!(last.exit_turnover, 1.0);
        }
    }

    #[test]
    fn s047_absolute_momentum_compares_spy_with_cash() {
        let panel = etf_panel(
            &["SPY", "EFA", "AGG", "BIL"],
            &[
                vec![10.0, 10.0, 10.5, 11.0],
                vec![10.0, 10.0, 10.2, 10.3],
                vec![10.0, 10.0, 10.1, 10.1],
                vec![10.0, 10.0, 10.1, 10.2],
            ],
        );
        let mut cfg = config();
        cfg.mom_window = 2;
        cfg.min_history = 2;
        cfg.strategy_mode = "absolute".into();
        cfg.defensive_asset = "BIL".into();
        let sig = compute_s047_signals(&panel, &cfg).unwrap();
        assert!(sig.signal[0][3]);
        assert_eq!(sig.trade_weights.unwrap()[0][3], 1.0);
    }

    #[test]
    fn s044_selects_the_highest_return_feasible_grid_allocation() {
        let rows: [[u8; 13]; 2] = [
            [4, 2, 4, 4, 4, 2, 0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 4, 4, 2, 2, 0, 4, 0, 0, 0, 4],
        ];
        let grid: Vec<_> = rows.into_iter().flatten().collect();
        let mut mean = [0.0; 13];
        mean[0] = 0.01;
        let mut covariance = [[0.0; 13]; 13];
        for (index, row) in covariance.iter_mut().enumerate() {
            row[index] = 0.0001;
        }
        let (weights, target, fallback) =
            s044_weights(&grid, 2, &mean, &covariance, 0.05, 0.50).unwrap();
        assert_eq!(weights[0], 0.2);
        assert!((target - 0.07).abs() < 1e-12);
        assert!(!fallback);
    }

    #[test]
    fn reads_and_validates_the_efficiente_weight_grid_format() {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("deepthrust-s044-grid-{unique}.npy"));
        let mut header = "{'descr': '|u1', 'fortran_order': False, 'shape': (1, 13), }".to_string();
        while (10 + header.len() + 1) % 16 != 0 {
            header.push(' ');
        }
        header.push('\n');
        let allocation = [4, 2, 4, 4, 4, 2, 0, 0, 0, 0, 0, 0, 0];
        let mut npy = b"\x93NUMPY\x01\x00".to_vec();
        npy.extend_from_slice(&(header.len() as u16).to_le_bytes());
        npy.extend_from_slice(header.as_bytes());
        npy.extend_from_slice(&allocation);
        std::fs::write(&path, npy).unwrap();
        let (rows, grid) = read_weight_grid(path.to_str().unwrap()).unwrap();
        assert_eq!(rows, 1);
        assert_eq!(grid, allocation);
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn parses_french_monthly_dates_as_calendar_month_end() {
        assert_eq!(
            parse_french_month("202402").unwrap(),
            NaiveDate::from_ymd_opt(2024, 2, 29).unwrap()
        );
    }
}
