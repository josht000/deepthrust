//! Core types for the Price Action Lab.

use chrono::{DateTime, NaiveDate, Utc};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Market {
    Es,
    Nq,
}

impl Market {
    pub const ALL: [Market; 2] = [Market::Es, Market::Nq];

    pub fn as_str(self) -> &'static str {
        match self {
            Market::Es => "es",
            Market::Nq => "nq",
        }
    }

    pub fn symbol(self) -> &'static str {
        match self {
            Market::Es => "ES",
            Market::Nq => "NQ",
        }
    }

    pub fn tick_size(self) -> f64 {
        0.25
    }

    /// USD value of one tick for one full-size contract.
    pub fn tick_value(self) -> f64 {
        match self {
            Market::Es => 12.50,
            Market::Nq => 5.00,
        }
    }

    /// Commission per side in USD (decision #8: $2.00/side).
    pub fn commission_per_side(self) -> f64 {
        2.00
    }

    pub fn parse(s: &str) -> Option<Market> {
        match s.to_ascii_lowercase().as_str() {
            "es" => Some(Market::Es),
            "nq" => Some(Market::Nq),
            _ => None,
        }
    }
}

/// One OHLCV bar. `ts_utc` is the bar's open time in UTC.
#[derive(Clone, Debug)]
pub struct Bar {
    pub ts_utc: DateTime<Utc>,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: u64,
}

/// A trading session: 18:00 -> 17:00 ET, dated by the day it ends.
/// `bars` holds the session's 1-minute bars sorted by ts_utc
/// (17:00-17:59 halt hour already dropped).
#[derive(Clone, Debug)]
pub struct Session {
    pub date: NaiveDate,
    pub bars: Vec<Bar>,
}

/// Higher-timeframe chart identifier, ordered low to high.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ChartId {
    M1 = 0,
    M5 = 1,
    M15 = 2,
    H1 = 3,
    H4 = 4,
    D1 = 5,
}

impl ChartId {
    pub const ALL: [ChartId; 6] = [
        ChartId::M1,
        ChartId::M5,
        ChartId::M15,
        ChartId::H1,
        ChartId::H4,
        ChartId::D1,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            ChartId::M1 => "1m",
            ChartId::M5 => "5m",
            ChartId::M15 => "15m",
            ChartId::H1 => "1h",
            ChartId::H4 => "4h",
            ChartId::D1 => "1d",
        }
    }

    pub fn parse(s: &str) -> Option<ChartId> {
        Some(match s {
            "1m" => ChartId::M1,
            "5m" => ChartId::M5,
            "15m" => ChartId::M15,
            "1h" => ChartId::H1,
            "4h" => ChartId::H4,
            "1d" | "d" | "daily" => ChartId::D1,
            _ => return None,
        })
    }

    /// The chart `steps` levels lower, if one exists.
    /// Daily setups are special-cased to trigger on 1h elsewhere.
    pub fn lower(self, steps: usize) -> Option<ChartId> {
        let idx = self as usize;
        idx.checked_sub(steps).map(|i| ChartId::ALL[i])
    }
}

/// A bar on a higher chart, expressed in 1-minute session indices so
/// usability ("only from its close") is exact.
#[derive(Clone, Debug)]
pub struct ChartBar {
    /// first 1-minute index covered by this bar
    pub open_idx: usize,
    /// the bar is usable by triggers at minute index >= usable_from
    /// (nominally the first index after the bar's range)
    pub usable_from: usize,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: u64,
    /// number of 1-minute bars actually present in the range
    pub n_minutes: usize,
}
