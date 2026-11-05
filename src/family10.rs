//! Family 10 candlestick and bar pattern studies.

use std::collections::BTreeMap;
use std::path::Path;

use chrono::NaiveDate;

use crate::config::Config;
use crate::data::{self, Panel};
use crate::engine::{self, Trade};
use crate::family2::blank_signals;
use crate::family4::{benchmark_returns, rebalance_days};
use crate::signals::{self, Signals};

pub const S103_NAME: &str = "s103_yield_curve_rotation";
pub const S103_TITLE: &str = "S103 Yield-Curve Financials/Utilities Rotation";

fn s103_symbol_index(panel: &Panel, name: &str) -> Result<usize, String> {
    panel
        .symbols
        .iter()
        .position(|symbol| symbol == name)
        .ok_or_else(|| format!("{name} is required in the S103 universe"))
}

/// FRED fredgraph.csv: `observation_date,<SERIES>` with "." for missing days.
fn read_fred_series(path: &Path) -> Result<BTreeMap<NaiveDate, f64>, String> {
    let mut reader = csv::Reader::from_path(path)
        .map_err(|error| format!("{}: {error}", path.display()))?;
    let mut series = BTreeMap::new();
    for (row, result) in reader.records().enumerate() {
        let record =
            result.map_err(|error| format!("{} row {}: {error}", path.display(), row + 2))?;
        let date = data::parse_date(record.get(0).unwrap_or_default().trim())
            .map_err(|error| format!("{} row {}: {error}", path.display(), row + 2))?;
        let value = record.get(1).unwrap_or_default().trim();
        if value.is_empty() || value == "." {
            continue;
        }
        let value = value.parse::<f64>().map_err(|error| {
            format!("{} row {}: invalid value {value:?}: {error}", path.display(), row + 2)
        })?;
        series.insert(date, value);
    }
    if series.is_empty() {
        return Err(format!("{}: no observations", path.display()));
    }
    Ok(series)
}

/// Last-observation-carried-forward alignment of a daily FRED series to the panel
/// calendar (covers bond-market holidays on stock trading days).
fn align_fred(series: &BTreeMap<NaiveDate, f64>, dates: &[NaiveDate]) -> Vec<f64> {
    let mut aligned = vec![f64::NAN; dates.len()];
    let mut last = f64::NAN;
    let mut iter = series.iter().peekable();
    for (day, date) in dates.iter().enumerate() {
        while let Some(&(&obs_date, &value)) = iter.peek() {
            if obs_date <= *date {
                last = value;
                iter.next();
            } else {
                break;
            }
        }
        aligned[day] = last;
    }
    aligned
}

fn banded(change: f64, band: f64) -> f64 {
    if !change.is_finite() || change.abs() <= band {
        0.0
    } else {
        change.signum()
    }
}

// S103: rotate financial_symbol (XLF) vs utility_symbol (XLU) on yield-curve moves.
// `separate`: banded 10Y and 2Y changes vote separately (slope vote + 10Y-level
// vote); `slope`: banded change of the 10Y-2Y spread; `level`: T10Y2Y level vs
// threshold. XLF when the curve signal favours steepening beyond the dead band,
// else XLU (`band_mode: binary`), or hold the previous leg inside the band
// (`band_mode: hold`).
pub fn compute_s103_signals(panel: &Panel, cfg: &Config) -> Result<Signals, String> {
    if !matches!(cfg.signal_mode.as_str(), "separate" | "slope" | "level") {
        return Err(format!(
            "S103 signal_mode must be separate|slope|level, got {}",
            cfg.signal_mode
        ));
    }
    if !matches!(cfg.band_mode.as_str(), "binary" | "hold") {
        return Err(format!(
            "S103 band_mode must be binary|hold, got {}",
            cfg.band_mode
        ));
    }
    if cfg.fred_dir.trim().is_empty() {
        return Err("S103 requires fred_dir with DGS10.csv, DGS2.csv, T10Y2Y.csv".into());
    }
    if cfg.rate_change_window == 0 {
        return Err("S103 rate_change_window must be > 0".into());
    }
    let fin = s103_symbol_index(panel, &cfg.financial_symbol)?;
    let util = s103_symbol_index(panel, &cfg.utility_symbol)?;
    let root = Path::new(&cfg.fred_dir);
    let dgs10 = align_fred(&read_fred_series(&root.join("DGS10.csv"))?, &panel.dates);
    let dgs2 = align_fred(&read_fred_series(&root.join("DGS2.csv"))?, &panel.dates);
    let t10y2y = if cfg.signal_mode == "level" {
        align_fred(&read_fred_series(&root.join("T10Y2Y.csv"))?, &panel.dates)
    } else {
        Vec::new()
    };
    let n = panel.dates.len();
    let w = cfg.rate_change_window;
    let band = cfg.rate_dead_band;
    let binary = cfg.band_mode == "binary";
    let mut result = blank_signals(panel, "yield_curve_vote");
    let mut weights = vec![vec![0.0; n]; panel.symbols.len()];
    let mut hold_fin = false;
    for &day in &rebalance_days(panel, &cfg.rebalance)? {
        result.tradable[fin][day] = panel.close[fin][day].is_finite();
        result.tradable[util][day] = panel.close[util][day].is_finite();
        let (d10, d2) = if day >= w {
            (dgs10[day] - dgs10[day - w], dgs2[day] - dgs2[day - w])
        } else {
            (f64::NAN, f64::NAN)
        };
        let vote = match cfg.signal_mode.as_str() {
            "slope" => banded(d10 - d2, band),
            "level" => {
                if t10y2y[day].is_finite() {
                    banded(t10y2y[day] - cfg.level_threshold, band)
                } else {
                    0.0
                }
            }
            _ => banded(d10 - d2, band) + banded(d10, band),
        };
        result.rsi[fin][day] = if cfg.signal_mode == "level" {
            t10y2y[day]
        } else {
            d10 - d2
        };
        let target = if vote > 0.0 {
            fin
        } else if vote < 0.0 || binary || !hold_fin {
            util
        } else {
            fin
        };
        hold_fin = target == fin;
        weights[target][day] = 1.0;
        result.signal[target][day] = true;
    }
    result.trade_weights = Some(weights);
    result.benchmark_returns = benchmark_returns(panel, "SPY");
    Ok(result)
}
pub const S100_NAME: &str = "s100_three_white_soldiers";
pub const S100_TITLE: &str = "S100 Three White Soldiers Continuation";

/// S100: N consecutive green candles, each closing above its prior close and
/// opening inside the prior body, confirmed by close > SMA and rising volume.
pub fn compute_s100_signals(panel: &Panel, cfg: &Config) -> Result<Signals, String> {
    if cfg.soldier_len < 2 {
        return Err("S100 soldier_len must be at least 2".into());
    }
    if !matches!(cfg.volume_mode.as_str(), "rising" | "last" | "off") {
        return Err(format!("unknown S100 volume_mode {:?}", cfg.volume_mode));
    }
    if !matches!(
        cfg.stop_mode.as_str(),
        "first_low" | "pattern_low" | "atr" | "off"
    ) {
        return Err(format!("unknown S100 stop_mode {:?}", cfg.stop_mode));
    }
    if cfg.max_hold == 0 {
        return Err("S100 max_hold must be greater than zero".into());
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
    let mut result = blank_signals(panel, "first_low");
    for symbol in 0..panel.symbols.len() {
        let (open, high, low, close, volume) = (
            &panel.open[symbol],
            &panel.high[symbol],
            &panel.low[symbol],
            &panel.close[symbol],
            &panel.volume[symbol],
        );
        let sma = if cfg.sma_window > 0 {
            signals::rolling_mean(close, cfg.sma_window)
        } else {
            Vec::new()
        };
        let atr = if cfg.min_body_atr > 0.0 {
            Some(signals::wilder_atr(high, low, close, cfg.atr_len))
        } else {
            None
        };
        let rsi = if cfg.max_rsi <= 100.0 {
            Some(signals::wilder_rsi(close, cfg.rsi_len)?)
        } else {
            None
        };
        let mut history = 0usize;
        for day in 0..panel.dates.len() {
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
            if day < cfg.soldier_len {
                continue;
            }
            let first = day + 1 - cfg.soldier_len;
            let mut pattern = true;
            for i in first..=day {
                if !open[i].is_finite()
                    || !low[i].is_finite()
                    || !close[i].is_finite()
                    || !close[i - 1].is_finite()
                    || close[i] <= open[i]
                    || close[i] <= close[i - 1]
                {
                    pattern = false;
                    break;
                }
                if cfg.min_body_pct > 0.0 && close[i] - open[i] < cfg.min_body_pct * open[i] {
                    pattern = false;
                    break;
                }
                if let Some(a) = &atr {
                    if !a[i].is_finite() || close[i] - open[i] < cfg.min_body_atr * a[i] {
                        pattern = false;
                        break;
                    }
                }
                if cfg.open_in_body && i > first && !(open[i] >= open[i - 1] && open[i] <= close[i - 1]) {
                    pattern = false;
                    break;
                }
            }
            if !pattern {
                continue;
            }
            if cfg.sma_window > 0 && !(sma[day].is_finite() && close[day] > sma[day]) {
                continue;
            }
            match cfg.volume_mode.as_str() {
                "rising" => {
                    if !(first..day)
                        .all(|i| volume[i].is_finite() && volume[i + 1] > volume[i])
                    {
                        continue;
                    }
                }
                "last" => {
                    if !(volume[day].is_finite()
                        && volume[day - 1].is_finite()
                        && volume[day] > volume[day - 1])
                    {
                        continue;
                    }
                }
                _ => {}
            }
            if let Some(r) = &rsi {
                if !(r[day].is_finite() && r[day] <= cfg.max_rsi) {
                    continue;
                }
            }
            result.signal[symbol][day] = true;
            result.rsi[symbol][day] = low[first];
        }
    }
    if cfg.skip_days > 0 {
        let original = result.signal.clone();
        let anchor = result.rsi.clone();
        let n_rows = panel.dates.len();
        result.signal.fill(vec![false; n_rows]);
        for symbol in 0..panel.symbols.len() {
            for day in cfg.skip_days..n_rows {
                result.signal[symbol][day] =
                    original[symbol][day - cfg.skip_days] && result.tradable[symbol][day];
                if result.signal[symbol][day] {
                    result.rsi[symbol][day] = anchor[symbol][day - cfg.skip_days];
                }
            }
        }
    }
    Ok(result)
}

/// Exit on the first close below the pattern stop level, or after `max_hold`
/// sessions in the trade; close-evaluated exits fill at the next open under
/// the `next_open` model and at the same close under `close`.
pub fn s100_trades(
    panel: &Panel,
    sig: &Signals,
    cfg: &Config,
    fill: &str,
) -> Result<Vec<Trade>, String> {
    if !matches!(fill, "close" | "next_open") {
        return Err(format!("S100 does not support fill {fill:?}"));
    }
    let n_rows = panel.dates.len();
    let is_open = fill == "next_open";
    let mut trades = Vec::new();
    for symbol in 0..panel.symbols.len() {
        let (open, low, close) = (&panel.open[symbol], &panel.low[symbol], &panel.close[symbol]);
        let atr = if cfg.stop_mode == "atr" {
            Some(signals::wilder_atr(
                &panel.high[symbol],
                low,
                close,
                cfg.atr_len,
            ))
        } else {
            None
        };
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
            let first = day + 1 - cfg.soldier_len;
            let stop = match cfg.stop_mode.as_str() {
                "first_low" => Some(sig.rsi[symbol][day]),
                "pattern_low" => Some(
                    (first..=day)
                        .map(|i| low[i])
                        .filter(|v| v.is_finite())
                        .fold(f64::INFINITY, f64::min),
                ),
                "atr" => atr.as_ref().and_then(|a| {
                    (a[day].is_finite()).then_some(entry_px - cfg.stop_atr_mult * a[day])
                }),
                _ => None,
            };
            let cap = entry.saturating_add(cfg.max_hold - 1).min(last);
            let mut decision = cap;
            let mut reason = if entry + cfg.max_hold - 1 > last {
                if last + 1 < n_rows {
                    "delisted"
                } else {
                    "censored"
                }
            } else {
                "cap"
            };
            for holding_day in (day + 1)..=cap {
                if !close[holding_day].is_finite() {
                    continue;
                }
                if let Some(level) = stop {
                    if level.is_finite() && close[holding_day] < level {
                        decision = holding_day;
                        reason = "stop";
                        break;
                    }
                }
                if cfg.take_profit > 0.0
                    && close[holding_day] >= entry_px * (1.0 + cfg.take_profit)
                {
                    decision = holding_day;
                    reason = "target";
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
