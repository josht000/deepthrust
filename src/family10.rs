//! Family 10 candlestick and bar pattern studies.

use crate::config::Config;
use crate::data::{self, Panel};
use crate::engine::Trade;
use crate::family4::blank_signals;
use crate::signals::{self, Signals};

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
