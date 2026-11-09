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
pub const S082_NAME: &str = "s082_opex_week_drift";
pub const S082_TITLE: &str = "S082 Options-Expiration-Week Drift";

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
            let t = day + 1;
            if t >= n {
                continue;
            }
            let (c, p, o) = (close[day], close[day - 1], open[day]);
            if !c.is_finite() || !p.is_finite() || p <= 0.0 || !o.is_finite() {
                continue;
            }
            result.tradable[j][day] = true;
            result.pool[j][day] = true;
            let ret = c / p - 1.0;
            let weak = match cfg.signal_mode.as_str() {
                "ret_le" => ret <= cfg.weakness_threshold,
                "gap_down" => o < p,
                _ => c < p,
            };
            result.rsi[j][day] = ret;
            if !weak {
                continue;
            }
            result.signal[j][day] = true;
            entries[j][t] = c;
            exits[j][t] = if cfg.exit_offset == 0 { open[t] } else { close[t] };
            if c.is_finite() && exits[j][t].is_finite() {
                weights[j][t] = 1.0;
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
                dir_sign: 1.0,
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

/// Calendar 3rd Friday of a month (the monthly options-expiry anchor).
fn third_friday(year: i32, month: u32) -> Option<NaiveDate> {
    let first = NaiveDate::from_ymd_opt(year, month, 1)?;
    let offset = (Weekday::Fri.num_days_from_monday() as i64
        - first.weekday().num_days_from_monday() as i64)
        .rem_euclid(7);
    Some(first + Duration::days(offset + 14))
}

// S082: signal `entry_offset` sessions before the first session of options-expiration
// week; exit on the close `exit_offset` sessions after the expiry session.
pub fn compute_s082_signals(panel: &Panel, cfg: &Config) -> Result<Signals, String> {
    if !(-15..=-1).contains(&cfg.entry_offset) {
        return Err(format!(
            "S082 entry_offset must be -15..-1, got {}",
            cfg.entry_offset
        ));
    }
    if !(0..=10).contains(&cfg.exit_offset) {
        return Err(format!(
            "S082 exit_offset must be 0..10, got {}",
            cfg.exit_offset
        ));
    }
    if !matches!(cfg.opex_mode.as_str(), "all" | "quad" | "non_quad") {
        return Err(format!(
            "S082 opex_mode must be all|quad|non_quad, got {}",
            cfg.opex_mode
        ));
    }
    if !matches!(
        cfg.regime_mode.as_str(),
        "none" | "above_sma" | "below_sma" | "prior_month_up" | "prior_month_down"
    ) {
        return Err(format!(
            "S082 regime_mode must be none|above_sma|below_sma|prior_month_up|prior_month_down, got {}",
            cfg.regime_mode
        ));
    }
    let n = panel.dates.len();
    let ordinal = month_ordinal_rows(&panel.dates);
    // first row of each row's calendar month
    let mut month_first = vec![0usize; n];
    for t in 0..n {
        let mut m0 = t;
        while m0 > 0 && ordinal[m0] != 1 {
            m0 -= 1;
        }
        month_first[t] = m0;
    }
    // per-month (signal_row, exit_row) pairs
    let mut windows: Vec<(usize, usize)> = Vec::new();
    let mut last_exit: i64 = -1;
    let mut t = 0usize;
    while t < n {
        let (year, month) = (panel.dates[t].year(), panel.dates[t].month());
        let mut end = t;
        while end + 1 < n && panel.dates[end + 1].year() == year && panel.dates[end + 1].month() == month
        {
            end += 1;
        }
        if let Some(tf) = third_friday(year, month) {
            // expiry session: last session on or before the 3rd Friday, within the same week
            let expiry_row = (t..=end)
                .rev()
                .find(|&r| panel.dates[r] <= tf && panel.dates[r] >= tf - Duration::days(7));
            if let Some(expiry_row) = expiry_row {
                let week_monday = panel.dates[expiry_row]
                    - Duration::days(panel.dates[expiry_row].weekday().num_days_from_monday() as i64);
                let mut first_opex = expiry_row;
                while first_opex > 0 && panel.dates[first_opex - 1] >= week_monday {
                    first_opex -= 1;
                }
                let signal_row = first_opex as i64 + cfg.entry_offset;
                let exit_row = expiry_row + cfg.exit_offset as usize;
                let quad = matches!(month, 3 | 6 | 9 | 12);
                let keep = match cfg.opex_mode.as_str() {
                    "quad" => quad,
                    "non_quad" => !quad,
                    _ => true,
                };
                if keep && signal_row > last_exit && signal_row >= 0 && exit_row < n {
                    windows.push((signal_row as usize, exit_row));
                    last_exit = exit_row as i64;
                }
            }
        }
        t = end + 1;
    }
    let mut result = blank_signals(panel, "opex_week");
    for j in 0..panel.symbols.len() {
        let close = &panel.close[j];
        let sma = crate::signals::rolling_mean(close, cfg.sma_window);
        for &(signal_row, exit_row) in &windows {
            if !close[signal_row].is_finite() || !close[exit_row].is_finite() {
                continue;
            }
            // prior full calendar month close-to-close return at the signal row
            let m0 = month_first[signal_row];
            let prior_ret = if m0 >= 2 {
                let p_end = m0 - 1;
                let p0 = month_first[p_end];
                if p0 >= 1 && close[p_end].is_finite() && close[p0 - 1].is_finite() {
                    close[p_end] / close[p0 - 1] - 1.0
                } else {
                    f64::NAN
                }
            } else {
                f64::NAN
            };
            let gate = match cfg.regime_mode.as_str() {
                "above_sma" => sma[signal_row].is_finite() && close[signal_row] > sma[signal_row],
                "below_sma" => sma[signal_row].is_finite() && close[signal_row] <= sma[signal_row],
                "prior_month_up" => prior_ret > 0.0,
                "prior_month_down" => prior_ret <= 0.0,
                _ => true,
            };
            result.rsi[j][signal_row] = prior_ret;
            if gate {
                result.signal[j][signal_row] = true;
            }
            result.exit_now[j][exit_row] = true;
        }
    }
    Ok(result)
}

pub fn s082_trades(panel: &Panel, sig: &Signals, cfg: &Config, fill: &str) -> Result<Vec<Trade>, String> {
    calendar_hold_trades(panel, sig, cfg, fill)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config() -> Config {
        let mut cfg: Config = serde_yaml::from_str("partition_split_date: '2020-01-01'\n").unwrap();
        cfg.cost_bps_per_side = 0.0;
        cfg.weekday = 1;
        cfg.signal_mode = "close_below_prev".into();
        cfg.exit_offset = 0;
        cfg
    }

    // Fri, Mon (weak close), Tue, Wed
    fn panel() -> Panel {
        Panel {
            dates: vec![
                NaiveDate::from_ymd_opt(2024, 1, 5).unwrap(),
                NaiveDate::from_ymd_opt(2024, 1, 8).unwrap(),
                NaiveDate::from_ymd_opt(2024, 1, 9).unwrap(),
                NaiveDate::from_ymd_opt(2024, 1, 10).unwrap(),
            ],
            symbols: vec!["XYZ".into()],
            open: vec![vec![100.0, 99.0, 97.0, 99.0]],
            high: vec![vec![101.0; 4]],
            low: vec![vec![98.0; 4]],
            close: vec![vec![100.0, 98.0, 99.0, 100.0]],
            volume: vec![vec![1_000.0; 4]],
        }
    }

    #[test]
    fn s010_exits_next_session_open_after_weak_close() {
        let panel = panel();
        let cfg = config();
        let sig = compute_s010_signals(&panel, &cfg).unwrap();
        let trades = s010_trades(&panel, &sig, &cfg, "close_to_open").unwrap();
        assert_eq!(trades.len(), 1);
        let trade = &trades[0];
        assert_eq!(trade.signal_idx, 1);
        assert_eq!(trade.entry_idx, 2);
        assert_eq!(trade.exit_idx, 2);
        assert_eq!(trade.entry_px, 98.0);
        assert_eq!(trade.exit_px, 97.0);

        let mut cfg = config();
        cfg.exit_offset = 1;
        let sig = compute_s010_signals(&panel, &cfg).unwrap();
        let trades = s010_trades(&panel, &sig, &cfg, "close_to_open").unwrap();
        assert_eq!(trades.len(), 1);
        assert_eq!(trades[0].entry_px, 98.0);
        assert_eq!(trades[0].exit_px, 99.0);
        // entry is the prior session's close; no exit uses the same session's open
        for trade in &trades {
            assert_eq!(trade.entry_px, panel.close[trade.sym_idx][trade.entry_idx - 1]);
        }
    }
}
