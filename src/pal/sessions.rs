//! Session logic: convert UTC bar times to America/New_York, assign the
//! bar to a session (18:00 -> 17:00 ET, dated by the day it ends; Sunday
//! evening belongs to Monday), and drop the 17:00-17:59 halt hour.

use chrono::{DateTime, Datelike, NaiveDate, Timelike, Utc, Weekday};
use chrono_tz::America::New_York;

use crate::pal::types::{Bar, Session};

/// Session date for a 1-minute bar open timestamp, or `None` when the bar
/// falls in the 17:00-17:59 ET halt (dropped per spec).
///
/// Rules:
/// - 17:00 <= t < 18:00  -> halt, dropped
/// - t >= 18:00          -> session ends next calendar day
///   (Sunday 18:00 belongs to Monday; Mon-Thu 18:00 -> next day)
/// - t < 17:00           -> session ends today
pub fn bar_session_date(ts_utc: DateTime<Utc>) -> Option<NaiveDate> {
    let et = ts_utc.with_timezone(&New_York);
    let mins = et.hour() * 60 + et.minute();
    if (17 * 60..18 * 60).contains(&mins) {
        return None;
    }
    let today = et.date_naive();
    if mins >= 18 * 60 {
        Some(today + chrono::Duration::days(1))
    } else {
        Some(today)
    }
}

/// Minutes since the session's 18:00 ET open for a bar inside its session.
/// Returns `None` for halt-hour bars.
pub fn minute_index(ts_utc: DateTime<Utc>) -> Option<usize> {
    let et = ts_utc.with_timezone(&New_York);
    let mins = et.hour() * 60 + et.minute();
    if (17 * 60..18 * 60).contains(&mins) {
        return None;
    }
    // 18:00 -> 0 .. 23:59 -> 359 ; 00:00 -> 360 .. 16:59 -> 1379
    Some(if mins >= 18 * 60 {
        (mins - 18 * 60) as usize
    } else {
        (mins + 6 * 60) as usize
    })
}

/// Group sorted 1-minute bars into sessions. Bars in the halt hour are
/// dropped. Input need not be sorted; output sessions are sorted by date
/// with bars sorted by ts_utc. Duplicate timestamps are collapsed
/// (first kept; audit counts them separately).
pub fn into_sessions(mut bars: Vec<Bar>) -> Vec<Session> {
    bars.sort_by_key(|b| b.ts_utc);
    bars.dedup_by_key(|b| b.ts_utc);
    let mut out: Vec<Session> = Vec::new();
    for b in bars {
        let Some(date) = bar_session_date(b.ts_utc) else {
            continue;
        };
        match out.last_mut() {
            Some(s) if s.date == date => s.bars.push(b),
            _ => out.push(Session {
                date,
                bars: vec![b],
            }),
        }
    }
    out
}

/// Per-session minute indices, matching `bars` order; halt bars -> None.
pub fn minute_indices(session: &Session) -> Vec<Option<usize>> {
    session.bars.iter().map(|b| minute_index(b.ts_utc)).collect()
}

/// Count of distinct trading days a session spans (2 normally; Sunday
/// sessions start on Sunday). Kept simple: always 2 dates except when the
/// first bar's ET date equals the session date (early-open edge cases).
pub fn session_start_date(session: &Session) -> Option<NaiveDate> {
    session
        .bars
        .first()
        .map(|b| b.ts_utc.with_timezone(&New_York).date_naive())
}

/// Is the session date a weekday Mon-Fri (weekend sessions do not occur
/// in CME data, but Saturday session dates would mean Fri-evening bars
/// which CME does not trade).
#[allow(dead_code)]
pub fn is_weekday_session(date: NaiveDate) -> bool {
    !matches!(date.weekday(), Weekday::Sat | Weekday::Sun)
}

/// Complete session per spec's final-test rule: has a bar opening at
/// 15:59 ET or later in the session (minute index >= 1319).
/// A session missing its tail is "dropped" like a roll.
pub fn is_complete_session(session: &Session) -> bool {
    const LAST_REQUIRED: usize = 15 * 60 + 59 + 6 * 60; // 15:59 ET -> idx 1319
    session.bars.iter().any(|b| {
        minute_index(b.ts_utc).map(|i| i >= LAST_REQUIRED).unwrap_or(false)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn utc(y: i32, mo: i32, d: i32, h: i32, mi: i32) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(y as i32, mo as u32, d as u32, h as u32, mi as u32, 0)
            .unwrap()
    }

    #[test]
    fn assigns_evening_bars_to_next_day() {
        // 2024-01-15 was a Monday. Monday 20:00 ET = Tuesday session.
        let t = utc(2024, 1, 16, 1, 0); // 20:00 ET (EST, UTC-5)
        assert_eq!(
            bar_session_date(t),
            Some(NaiveDate::from_ymd_opt(2024, 1, 16).unwrap())
        );
    }

    #[test]
    fn morning_bars_belong_to_today() {
        // Tuesday 09:30 ET -> Tuesday session.
        let t = utc(2024, 1, 16, 14, 30); // 09:30 EST
        assert_eq!(
            bar_session_date(t),
            Some(NaiveDate::from_ymd_opt(2024, 1, 16).unwrap())
        );
    }

    #[test]
    fn sunday_evening_is_monday() {
        // Sunday 2024-01-14 18:30 ET = Monday session (UTC 23:30).
        let t = utc(2024, 1, 14, 23, 30);
        assert_eq!(
            bar_session_date(t),
            Some(NaiveDate::from_ymd_opt(2024, 1, 15).unwrap())
        );
        assert_eq!(minute_index(t), Some(30));
    }

    #[test]
    fn halt_hour_dropped() {
        // Tuesday 17:30 ET -> halt.
        let t = utc(2024, 1, 16, 22, 30);
        assert_eq!(bar_session_date(t), None);
        assert_eq!(minute_index(t), None);
    }

    #[test]
    fn dst_boundary_still_dates_correctly() {
        // 2024-03-11 (Mon, first EDT week): Monday 18:30 EDT = UTC 22:30.
        let t = utc(2024, 3, 11, 22, 30);
        assert_eq!(
            bar_session_date(t),
            Some(NaiveDate::from_ymd_opt(2024, 3, 12).unwrap())
        );
    }

    #[test]
    fn session_grouping() {
        // Build a mini Monday session: Sun 18:00-18:01 ET + Mon 16:58 ET.
        let bars = vec![
            Bar {
                ts_utc: utc(2024, 1, 14, 23, 0),
                open: 1.0,
                high: 1.0,
                low: 1.0,
                close: 1.0,
                volume: 1,
            },
            Bar {
                ts_utc: utc(2024, 1, 15, 21, 58),
                open: 2.0,
                high: 2.0,
                low: 2.0,
                close: 2.0,
                volume: 1,
            },
            Bar {
                ts_utc: utc(2024, 1, 15, 22, 30), // halt -> dropped
                open: 9.0,
                high: 9.0,
                low: 9.0,
                close: 9.0,
                volume: 1,
            },
        ];
        let sessions = into_sessions(bars);
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].date, NaiveDate::from_ymd_opt(2024, 1, 15).unwrap());
        assert_eq!(sessions[0].bars.len(), 2);
    }

    #[test]
    fn complete_session_needs_late_bar() {
        let late = Bar {
            ts_utc: utc(2024, 1, 16, 20, 59), // 15:59 ET, idx 1319
            open: 1.0,
            high: 1.0,
            low: 1.0,
            close: 1.0,
            volume: 1,
        };
        let early = Bar {
            ts_utc: utc(2024, 1, 15, 23, 0),
            open: 1.0,
            high: 1.0,
            low: 1.0,
            close: 1.0,
            volume: 1,
        };
        let s_full = Session {
            date: NaiveDate::from_ymd_opt(2024, 1, 16).unwrap(),
            bars: vec![early.clone(), late],
        };
        assert!(is_complete_session(&s_full));
        let s_short = Session {
            date: s_full.date,
            bars: vec![early],
        };
        assert!(!is_complete_session(&s_short));
    }
}
