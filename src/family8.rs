//! Family 8 calendar studies: weekday reversal, turn-of-the-month, pre-holiday.

use chrono::{Datelike, Duration, NaiveDate, Weekday};

use crate::config::Config;
use crate::data::Panel;
use crate::engine::Trade;
use crate::family2::blank_signals;
use crate::signals::Signals;

pub const S010_NAME: &str = "s010_turnaround_tuesday";
pub const S010_TITLE: &str = "S010 Turnaround Tuesday";
pub const S079_NAME: &str = "s079_turn_of_month";
pub const S079_TITLE: &str = "S079 Turn-of-the-Month Effect";
pub const S080_NAME: &str = "s080_pre_holiday";
pub const S080_TITLE: &str = "S080 Pre-Holiday Effect";

fn weekday_of(date: NaiveDate) -> i64 {
    date.weekday().num_days_from_monday() as i64 + 1
}

/// Sessions remaining in the same calendar month after each row (row is nth-from-last).
fn sessions_left_in_month(dates: &[NaiveDate]) -> Vec<usize> {
    let n = dates.len();
    let mut left = vec![1usize; n];
    for t in (0..n.saturating_sub(1)).rev() {
        let same = dates[t].year() == dates[t + 1].year() && dates[t].month() == dates[t + 1].month();
        left[t] = if same { left[t + 1] + 1 } else { 1 };
    }
    left
}

/// Row index of the k-th session of the month containing the row (k = 1-based ordinal).
fn month_ordinal_rows(dates: &[NaiveDate]) -> Vec<usize> {
    let n = dates.len();
    let mut ordinal = vec![0usize; n];
    let mut seen = 0usize;
    for t in 0..n {
        if t == 0 || dates[t].month() != dates[t - 1].month() || dates[t].year() != dates[t - 1].year()
        {
            seen = 0;
        }
        seen += 1;
        ordinal[t] = seen;
    }
    ordinal
}

/// True when weekday dates strictly between d0 and d1 are absent from the calendar
/// (an exchange holiday fell in the gap; weekends and early closes are sessions or not gaps).
fn gap_has_holiday(d0: NaiveDate, d1: NaiveDate, calendar: &std::collections::BTreeSet<NaiveDate>) -> bool {
    let mut day = d0 + Duration::days(1);
    while day < d1 {
        if day.weekday() != Weekday::Sat && day.weekday() != Weekday::Sun && !calendar.contains(&day) {
            return true;
        }
        day += Duration::days(1);
    }
    false
}

fn is_early_close_candidate(holiday: NaiveDate) -> bool {
    holiday.month() == 7 && holiday.day() <= 5 || holiday.month() == 12 && holiday.day() >= 24 && holiday.day() <= 26
}

// S010: weakness on `weekday` -> overnight hold to next session open (or close variant).
pub fn compute_s010_signals(panel: &Panel, cfg: &Config) -> Result<Signals, String> {
    if cfg.weekday < -1 || cfg.weekday == 0 || cfg.weekday > 5 {
        return Err(format!("S010 weekday must be -1..5, got {}", cfg.weekday));
    }
    if !matches!(cfg.exit_offset, 0 | 1) {
        return Err(format!("S010 exit_offset must be 0 (open) or 1 (close), got {}", cfg.exit_offset));
    }
    let n = panel.dates.len();
    let mut result = blank_signals(panel, "weekday_return");
    let n_sym = panel.symbols.len();
    let mut entries = vec![vec![f64::NAN; n]; n_sym];
    let mut exits = vec![vec![f64::NAN; n]; n_sym];
    let mut weights = vec![vec![0.0; n]; n_sym];
    for j in 0..n_sym {
        let (close, open) = (&panel.close[j], &panel.open[j]);
        for day in 1..n {
            if cfg.weekday > 0 && weekday_of(panel.dates[day]) != cfg.weekday {
                continue;
            }
            let (c, p, o) = (close[day], close[day - 1], open[day]);
            if !c.is_finite() || !p.is_finite() || p <= 0.0 || !o.is_finite() {
                continue;
            }
            result.tradable[j][day - 1] = true;
            result.pool[j][day - 1] = true;
            let ret = c / p - 1.0;
            let weak = match cfg.signal_mode.as_str() {
                "ret_le" => ret <= cfg.weakness_threshold,
                "gap_down" => o < p,
                _ => c < p,
            };
            result.rsi[j][day - 1] = ret;
            if !weak {
                continue;
            }
            result.signal[j][day - 1] = true;
            let trade_row = day;
            entries[j][trade_row] = c;
            exits[j][trade_row] = if cfg.exit_offset == 0 { open[trade_row] } else { close[trade_row] };
            if exits[j][trade_row].is_finite() {
                weights[j][trade_row] = 1.0;
            }
        }
    }
    result.trade_weights = Some(weights);
    result.entry_mat = Some(entries);
    result.exit_mat = Some(exits);
    Ok(result)
}

pub fn s010_trades(panel: &Panel, sig: &Signals, cfg: &Config, fill: &str) -> Result<Vec<Trade>, String> {
    crate::signals::overnight_trades(panel, sig, cfg, fill)
}

// S079: signal on the second-to-last session of the month, exit on session `exit_offset` of next month.
pub fn compute_s079_signals(panel: &Panel, cfg: &Config) -> Result<Signals, String> {
    if !(1..=5).contains(&cfg.exit_offset) {
        return Err(format!("S079 exit_offset must be 1..5, got {}", cfg.exit_offset));
    }
    if !matches!(cfg.entry_offset, -2 | -1) {
        return Err(format!("S079 entry_offset must be -2 or -1, got {}", cfg.entry_offset));
    }
    let n = panel.dates.len();
    let left = sessions_left_in_month(&panel.dates);
    let ordinal = month_ordinal_rows(&panel.dates);
    let target = cfg.entry_offset.unsigned_abs() as usize;
    let mut result = blank_signals(panel, "month_ret");
    let mut prior_month_ret = vec![f64::NAN; n];
    let mut month_start_close = vec![f64::NAN; n];
    for j in 0..panel.symbols.len() {
        for t in 0..n {
            if ordinal[t] == 1 {
                month_start_close[t] = panel.close[j][t];
            } else if t > 0 {
                month_start_close[t] = month_start_close[t - 1];
            }
            if !panel.close[j][t].is_finite() {
                continue;
            }
            result.tradable[j][t] = true;
            result.pool[j][t] = true;
        }
        for t in 0..n {
            if left[t] != target {
                continue;
            }
            // prior full-month return for the regime split variants
            let mut prior_start = t;
            while prior_start > 0 && ordinal[prior_start] != 1 {
                prior_start -= 1;
            }
            if prior_start > 0 && month_start_close[prior_start].is_finite() && panel.close[j][prior_start - 1].is_finite() {
                prior_month_ret[t] = month_start_close[prior_start] / panel.close[j][prior_start - 1] - 1.0;
            }
            let gate = match cfg.regime_mode.as_str() {
                "prior_month_up" => prior_month_ret[t] > 0.0,
                "prior_month_down" => prior_month_ret[t] <= 0.0,
                _ => true,
            };
            if gate {
                result.signal[j][t] = true;
                result.rsi[j][t] = prior_month_ret[t];
            }
        }
    }
    // exit_now rows: the exit_offset-th session of the month following each signal's month
    let exit_target = cfg.exit_offset as usize;
    for t in 0..n {
        if ordinal[t] == exit_target {
            for j in 0..panel.symbols.len() {
                result.exit_now[j][t] = true;
            }
        }
    }
    Ok(result)
}

/// Close-to-close (or open entry) holds from each signal day to the next `exit_now` day.
fn calendar_hold_trades(
    panel: &Panel,
    sig: &Signals,
    cfg: &Config,
    fill: &str,
) -> Result<Vec<Trade>, String> {
    if !matches!(fill, "close" | "open" | "next_open") {
        return Err(format!("{} does not support fill {fill:?}", cfg.strategy));
    }
    let n = panel.dates.len();
    let last = crate::engine::last_valid_rows(&panel.close);
    let mut trades = Vec::new();
    for j in 0..panel.symbols.len() {
        let cost = cfg.cost_for_symbol(&panel.symbols[j]);
        let Some(last_row) = last[j] else { continue };
        for t in 0..n {
            if !sig.signal[j][t] {
                continue;
            }
            let Some(u) = (t + 1..n).find(|&u| sig.exit_now[j][u] && panel.close[j][u].is_finite())
            else {
                continue;
            };
            if u > last_row {
                continue;
            }
            let (entry_idx, entry_px) = match fill {
                "open" => (t, panel.open[j][t]),
                "next_open" => (t + 1, panel.open[j][t + 1]),
                _ => (t, panel.close[j][t]),
            };
            if entry_idx >= u || !entry_px.is_finite() || entry_px <= 0.0 {
                continue;
            }
            let exit_px = panel.close[j][u];
            let gross = exit_px / entry_px - 1.0;
            trades.push(Trade {
                sym_idx: j,
                signal_idx: t,
                entry_idx,
                exit_idx: u,
                entry_px,
                exit_px,
                ret: exit_px * (1.0 - cost) / (entry_px * (1.0 + cost)) - 1.0,
                ret_gross: gross,
                exit_reason: "window_end",
                hold: u - entry_idx,
                signal_value: sig.rsi[j][t],
                weight: 1.0,
                entry_turnover: 1.0,
                exit_turnover: 1.0,
                weighted: false,
                borrow_cost_per_bar: 0.0,
            });
        }
    }
    trades.sort_by(|a, b| (a.entry_idx, &panel.symbols[a.sym_idx]).cmp(&(b.entry_idx, &panel.symbols[b.sym_idx])));
    Ok(trades)
}

pub fn s079_trades(panel: &Panel, sig: &Signals, cfg: &Config, fill: &str) -> Result<Vec<Trade>, String> {
    calendar_hold_trades(panel, sig, cfg, fill)
}

// S080: signal two sessions before a holiday gap; exit on the last session before it (or first after).
pub fn compute_s080_signals(panel: &Panel, cfg: &Config) -> Result<Signals, String> {
    if !matches!(cfg.exit_offset, 0 | 1) {
        return Err(format!("S080 exit_offset must be 0 or 1, got {}", cfg.exit_offset));
    }
    let n = panel.dates.len();
    let calendar: std::collections::BTreeSet<NaiveDate> = panel.dates.iter().copied().collect();
    // gaps: session t whose next session d' leaves missing weekday(s) => holiday gap; t is last session before
    let mut last_before = vec![false; n];
    let mut holiday_first = vec![f64::NAN; n];
    for t in 0..n - 1 {
        if gap_has_holiday(panel.dates[t], panel.dates[t + 1], &calendar) {
            last_before[t] = true;
            holiday_first[t] = (panel.dates[t] + Duration::days(1)).num_days_from_ce() as f64;
        }
    }
    let mut result = blank_signals(panel, "preholiday");
    for t in 0..n - 1 {
        if !last_before[t] {
            continue;
        }
        if cfg.exclude_early_close {
            // the session before the holiday is an early-close candidate (Jul-3 / Dec-24 style)
            let mut probe = panel.dates[t] + Duration::days(1);
            while probe < panel.dates[t + 1] {
                if probe.weekday() != Weekday::Sat && probe.weekday() != Weekday::Sun && !calendar.contains(&probe) {
                    if is_early_close_candidate(probe) {
                        break;
                    }
                }
                probe += Duration::days(1);
            }
            if probe < panel.dates[t + 1] && is_early_close_candidate(probe) {
                continue;
            }
        }
        if t == 0 {
            continue;
        }
        let signal_day = t - 1; // second-to-last session before the holiday
        let exit_day = if cfg.exit_offset == 1 { t + 1 } else { t };
        for j in 0..panel.symbols.len() {
            result.tradable[j][signal_day] = panel.close[j][signal_day].is_finite();
            result.pool[j][signal_day] = result.tradable[j][signal_day];
            if !result.tradable[j][signal_day] || !panel.close[j][exit_day].is_finite() {
                continue;
            }
            result.signal[j][signal_day] = true;
            result.exit_now[j][exit_day] = true;
        }
    }
    let _ = holiday_first;
    Ok(result)
}

pub fn s080_trades(panel: &Panel, sig: &Signals, cfg: &Config, fill: &str) -> Result<Vec<Trade>, String> {
    calendar_hold_trades(panel, sig, cfg, fill)
}
