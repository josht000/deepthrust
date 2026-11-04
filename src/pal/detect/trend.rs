//! Trend from the last two confirmed swing highs and lows (spec):
//! up = higher high + higher low, down = lower high + lower low,
//! otherwise no trend.

use crate::pal::detect::swings::{Swing, SwingIndex, SwingKind};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Trend {
    Up,
    Down,
    None,
}

/// Trend at bar `at` from an index (hot path).
pub fn trend_idx(idx: &SwingIndex, at: usize) -> Trend {
    let highs = idx.last_of(SwingKind::High, at, 2);
    let lows = idx.last_of(SwingKind::Low, at, 2);
    if highs.len() < 2 || lows.len() < 2 {
        return Trend::None;
    }
    let hh = highs[1].price > highs[0].price;
    let hl = lows[1].price > lows[0].price;
    let lh = highs[1].price < highs[0].price;
    let ll = lows[1].price < lows[0].price;
    match (hh, hl, lh, ll) {
        (true, true, _, _) => Trend::Up,
        (_, _, true, true) => Trend::Down,
        _ => Trend::None,
    }
}

pub fn trend_at(swings: &[Swing], at: usize) -> Trend {
    trend_idx(&SwingIndex::new(swings), at)
}
