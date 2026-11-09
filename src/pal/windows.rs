//! Study windows per spec (decision #9).

use chrono::NaiveDate;

const fn d(y: i32, m: u32, day: u32) -> NaiveDate {
    match NaiveDate::from_ymd_opt(y, m, day) {
        Some(d) => d,
        None => panic!("bad date"),
    }
}

/// Warm-up: builds detector state only, never traded.
pub const WARMUP_START: NaiveDate = d(2021, 10, 1);
pub const WARMUP_END: NaiveDate = d(2021, 12, 31);

/// Discovery: every rule is found and judged here.
pub const DISCOVERY_START: NaiveDate = d(2022, 1, 3);
pub const DISCOVERY_END: NaiveDate = d(2024, 7, 31);

/// Era 1 = 2022 sessions; era 2 = 2023 -> July 2024.
pub const ERA1_START: NaiveDate = d(2022, 1, 3);
pub const ERA1_END: NaiveDate = d(2022, 12, 31);
pub const ERA2_START: NaiveDate = d(2023, 1, 1);
pub const ERA2_END: NaiveDate = DISCOVERY_END;

/// Sealed final window. Nothing reads this without a logged unlock.
pub const SEAL_START: NaiveDate = d(2024, 8, 1);
/// Expected end of data from the spec; the real end is whatever the
/// ingest wrote (recorded in data/sealed/manifest.sha256).
pub const SEAL_END_EXPECTED: NaiveDate = d(2026, 10, 16);

/// Final-test detector warm-up tail: join discovery bars from this date.
pub const FINAL_TEST_WARMUP_START: NaiveDate = d(2024, 5, 1);

/// Session length in minutes: 18:00 -> 17:00 ET = 23h
/// (the 17:00-17:59 halt hour is dropped).
pub const SESSION_MINUTES: usize = 1380;

/// True when the session date falls inside the sealed window.
pub fn is_sealed(date: NaiveDate) -> bool {
    date >= SEAL_START
}

/// True when the requested [from, to] range needs a seal unlock.
pub fn needs_unlock(_from: NaiveDate, to: NaiveDate) -> bool {
    to >= SEAL_START
}
