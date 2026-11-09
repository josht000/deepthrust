//! Synthetic session generator for plumbing checks and tests —
//! geometric random-walk 1-minute bars inside structurally valid
//! 18:00-17:00 ET sessions. NOT real data.

use chrono::{DateTime, Datelike, Duration, NaiveDate, TimeZone, Utc};
use chrono_tz::America::New_York;

use crate::pal::rng::Rng;
use crate::pal::types::{Bar, Session};
use crate::pal::windows::SESSION_MINUTES;

/// n sessions starting Monday-ish at `start` (skips Sat/Sun session
/// dates, matching the real calendar).
pub fn synth_sessions(start: NaiveDate, n: usize, seed_label: &str, vol: f64) -> Vec<Session> {
    let mut rng = Rng::fork(seed_label);
    let mut out = Vec::new();
    let mut date = start;
    let mut px = 20000.0;
    while out.len() < n {
        let wd = date.weekday();
        if matches!(wd, chrono::Weekday::Sat | chrono::Weekday::Sun) {
            date += Duration::days(1);
            continue;
        }
        let mut bars = Vec::with_capacity(SESSION_MINUTES);
        for mi in 0..SESSION_MINUTES {
            let et = if mi < 360 {
                let d = date - Duration::days(1);
                New_York
                    .from_local_datetime(&d.and_hms_opt(18 + mi as u32 / 60, mi as u32 % 60, 0).unwrap())
                    .unwrap()
            } else {
                New_York
                    .from_local_datetime(
                        &date.and_hms_opt((mi - 360) as u32 / 60, (mi - 360) as u32 % 60, 0)
                            .unwrap(),
                    )
                    .unwrap()
            };
            let ts: DateTime<Utc> = et.with_timezone(&Utc);
            let r = (rng.next_f64() - 0.5) * 2.0 * vol;
            let o = px;
            let c = px + r;
            let h = o.max(c) + rng.next_f64() * vol * 0.5;
            let l = o.min(c) - rng.next_f64() * vol * 0.5;
            bars.push(Bar {
                ts_utc: ts,
                open: o,
                high: h,
                low: l,
                close: c,
                volume: 100 + (rng.next_f64() * 500.0) as u64,
            });
            px = c;
        }
        out.push(Session { date, bars });
        date += Duration::days(1);
    }
    out
}
