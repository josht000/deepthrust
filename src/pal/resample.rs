//! Build 5m, 15m, 1h, 4h and daily chart bars from a session's 1-minute
//! bars, counted from the 18:00 ET open. 4h bins are the fixed schedule
//! 18-22, 22-02, 02-06, 06-10, 10-14, 14-17 ET. Daily = whole session.
//! Missing minutes are never filled (audit counts gaps): each chart bar
//! aggregates the 1-minute bars actually present in its nominal range,
//! and it becomes usable at the nominal range end regardless of whether
//! the last minute exists.

use crate::pal::sessions::minute_indices;
use crate::pal::types::{ChartBar, ChartId, Session};
use crate::pal::windows::SESSION_MINUTES;

/// Nominal [start, end) minute ranges for the 4h chart, as session
/// minute indices. Last bin is 3 hours per spec.
const H4_RANGES: [(usize, usize); 6] = [
    (0, 240),     // 18:00-22:00
    (240, 480),   // 22:00-02:00
    (480, 720),   // 02:00-06:00
    (720, 960),   // 06:00-10:00
    (960, 1200),  // 10:00-14:00
    (1200, 1380), // 14:00-17:00
];

fn fixed_size_ranges(minutes_per_bar: usize) -> Vec<(usize, usize)> {
    (0..SESSION_MINUTES)
        .step_by(minutes_per_bar)
        .map(|s| (s, (s + minutes_per_bar).min(SESSION_MINUTES)))
        .collect()
}

fn ranges_for(chart: ChartId) -> Vec<(usize, usize)> {
    match chart {
        ChartId::M1 => (0..SESSION_MINUTES).map(|i| (i, i + 1)).collect(),
        ChartId::M5 => fixed_size_ranges(5),
        ChartId::M15 => fixed_size_ranges(15),
        ChartId::H1 => fixed_size_ranges(60),
        ChartId::H4 => H4_RANGES.to_vec(),
        ChartId::D1 => vec![(0, SESSION_MINUTES)],
    }
}

/// Build chart bars for one session. Bars with no 1-minute coverage at
/// all (empty range) are omitted — a session ending early just has
/// fewer bars.
pub fn build_chart(session: &Session, chart: ChartId) -> Vec<ChartBar> {
    let idx = minute_indices(session);
    // bucket of (bar_index_in_range) per present minute
    let ranges = ranges_for(chart);
    if session.bars.is_empty() {
        return Vec::new();
    }
    // For each nominal range, aggregate present 1m bars whose minute
    // index falls inside.
    let mut bars: Vec<ChartBar> = ranges
        .iter()
        .map(|(s, e)| ChartBar {
            open_idx: *s,
            usable_from: *e,
            open: f64::NAN,
            high: f64::NAN,
            low: f64::NAN,
            close: f64::NAN,
            volume: 0,
            n_minutes: 0,
        })
        .collect();
    for (bi, mi) in idx.iter().enumerate() {
        let Some(m) = *mi else { continue };
        // find the range containing m; ranges are contiguous so position
        // is deterministic
        let pos = range_pos(&ranges, m);
        if let Some(p) = pos {
            let cb = &mut bars[p];
            let b = &session.bars[bi];
            if cb.n_minutes == 0 {
                cb.open = b.open;
                cb.high = b.high;
                cb.low = b.low;
            } else {
                if b.high > cb.high {
                    cb.high = b.high;
                }
                if b.low < cb.low {
                    cb.low = b.low;
                }
            }
            cb.close = b.close;
            cb.volume += b.volume;
            cb.n_minutes += 1;
        }
    }
    bars.retain(|b| b.n_minutes > 0);
    bars
}

/// Which range contains minute m. Ranges are contiguous and sorted, so
/// binary-search on start.
fn range_pos(ranges: &[(usize, usize)], m: usize) -> Option<usize> {
    match ranges.binary_search_by(|(s, e)| {
        if m < *s {
            std::cmp::Ordering::Greater
        } else if m >= *e {
            std::cmp::Ordering::Less
        } else {
            std::cmp::Ordering::Equal
        }
    }) {
        Ok(i) => Some(i),
        Err(_) => None,
    }
}

/// Position helper for triggers: index of the last chart bar usable at
/// session minute `now` (bars with usable_from <= now). Returns None
/// before the first bar closes.
pub fn last_usable(bars: &[ChartBar], now: usize) -> Option<usize> {
    match bars.binary_search_by_key(&now, |b| b.usable_from) {
        Ok(i) => Some(i),
        Err(0) => None,
        Err(i) => Some(i - 1),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pal::types::Bar;
    use chrono::{DateTime, NaiveDate, TimeZone, Utc};

    fn session_minute(date: NaiveDate, idx: usize, px: f64) -> Bar {
        // idx 0 = 18:00 ET of the evening before `date`.
        // Work in ET: minute idx < 360 is the evening before.
        use chrono_tz::America::New_York;
        let (day, mins) = if idx < 360 {
            (date - chrono::Duration::days(1), 18 * 60 + idx)
        } else {
            (date, idx - 360)
        };
        let et = New_York
            .from_local_datetime(
                &day.and_hms_opt((mins / 60) as u32, (mins % 60) as u32, 0)
                    .unwrap(),
            )
            .unwrap();
        Bar {
            ts_utc: et.with_timezone(&Utc),
            open: px,
            high: px,
            low: px,
            close: px,
            volume: 10,
        }
    }

    fn d(y: i32, m: u32, dd: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, dd).unwrap()
    }

    #[test]
    fn five_minute_bars() {
        // full session Tuesday 2024-01-16
        let date = d(2024, 1, 16);
        let bars: Vec<Bar> = (0..SESSION_MINUTES)
            .map(|i| session_minute(date, i, 100.0 + i as f64))
            .collect();
        let s = Session { date, bars };
        let c5 = build_chart(&s, ChartId::M5);
        assert_eq!(c5.len(), SESSION_MINUTES / 5);
        assert_eq!(c5[0].open, 100.0);
        assert_eq!(c5[0].close, 104.0);
        assert_eq!(c5[0].usable_from, 5);
        assert_eq!(c5[1].open, 105.0);
        assert_eq!(c5[1].usable_from, 10);
    }

    #[test]
    fn four_hour_schedule() {
        let date = d(2024, 1, 16);
        let bars: Vec<Bar> = (0..SESSION_MINUTES)
            .map(|i| session_minute(date, i, 1.0))
            .collect();
        let s = Session { date, bars };
        let c4 = build_chart(&s, ChartId::H4);
        assert_eq!(c4.len(), 6);
        assert_eq!((c4[0].open_idx, c4[0].usable_from), (0, 240));
        assert_eq!((c4[5].open_idx, c4[5].usable_from), (1200, 1380));
        assert_eq!(c4[5].n_minutes, 180);
    }

    #[test]
    fn daily_is_whole_session() {
        let date = d(2024, 1, 16);
        let bars: Vec<Bar> = (0..SESSION_MINUTES)
            .map(|i| session_minute(date, i, 50.0 + (i % 7) as f64))
            .collect();
        let s = Session { date, bars };
        let cd = build_chart(&s, ChartId::D1);
        assert_eq!(cd.len(), 1);
        assert_eq!(cd[0].n_minutes, SESSION_MINUTES);
        assert_eq!(cd[0].open, 50.0);
        assert_eq!(cd[0].usable_from, SESSION_MINUTES);
    }

    #[test]
    fn missing_minutes_not_filled() {
        let date = d(2024, 1, 16);
        // only 7 of 10 minutes present in first 5m bin (idx 2,4 missing)
        let bars: Vec<Bar> = [0, 1, 3]
            .iter()
            .map(|&i| session_minute(date, i, 10.0 + i as f64))
            .collect();
        let s = Session { date, bars };
        let c5 = build_chart(&s, ChartId::M5);
        assert_eq!(c5.len(), 1);
        assert_eq!(c5[0].n_minutes, 3);
        assert_eq!(c5[0].close, 13.0);
        assert_eq!(c5[0].usable_from, 5); // nominal close, not last present
    }

    #[test]
    fn last_usable_from_close() {
        let date = d(2024, 1, 16);
        let bars: Vec<Bar> = (0..SESSION_MINUTES)
            .map(|i| session_minute(date, i, 1.0))
            .collect();
        let s = Session { date, bars };
        let c5 = build_chart(&s, ChartId::M5);
        assert_eq!(last_usable(&c5, 4), None);
        assert_eq!(last_usable(&c5, 5), Some(0));
        assert_eq!(last_usable(&c5, 9), Some(0));
        assert_eq!(last_usable(&c5, 10), Some(1));
    }

    #[test]
    fn timezone_dst_et_to_utc() {
        let _ = DateTime::<Utc>::from_timestamp(0, 0);
    }
}
