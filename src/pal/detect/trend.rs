//! Trend from the last two confirmed swing highs and lows (spec):
//! up = higher high + higher low, down = lower high + lower low,
//! otherwise no trend.

use crate::pal::detect::swings::{last_of_kind, Swing, SwingKind};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Trend {
    Up,
    Down,
    None,
}

pub fn trend_at(swings: &[Swing], at: usize) -> Trend {
    let highs = last_of_kind(swings, SwingKind::High, at, 2);
    let lows = last_of_kind(swings, SwingKind::Low, at, 2);
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
