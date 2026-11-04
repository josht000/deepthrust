//! Break of structure (BOS) and change of character (CHoCH): the first
//! close past the last confirmed swing — with the trend for BOS,
//! against it for CHoCH (spec). Each confirmed swing arms exactly once:
//! after its break it stays consumed until a newer swing confirms.

use crate::pal::detect::swings::{last_of_kind, Swing, SwingKind};
use crate::pal::detect::trend::{trend_at, Trend};
use crate::pal::types::ChartBar;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BreakDir {
    Up,
    Down,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BreakKind {
    /// with the prevailing trend
    Bos,
    /// against the prevailing trend (change of character)
    Choch,
}

#[derive(Clone, Copy, Debug)]
pub struct StructureBreak {
    /// bar index whose close crossed the swing
    pub idx: usize,
    pub dir: BreakDir,
    pub kind: BreakKind,
    /// the swing level that was crossed
    pub level: f64,
    /// pivot index of the broken swing
    pub swing_idx: usize,
}

pub fn structure_breaks(bars: &[ChartBar], swings: &[Swing]) -> Vec<StructureBreak> {
    let n = bars.len();
    let mut out = Vec::new();
    // swing idx already broken; never re-arms
    let mut consumed_high: Option<usize> = None;
    let mut consumed_low: Option<usize> = None;
    for i in 0..n {
        let trend = trend_at(swings, i);
        let highs = last_of_kind(swings, SwingKind::High, i, 1);
        let lows = last_of_kind(swings, SwingKind::Low, i, 1);
        if let Some(h) = highs.last() {
            if consumed_high != Some(h.idx) && bars[i].close > h.price {
                out.push(StructureBreak {
                    idx: i,
                    dir: BreakDir::Up,
                    kind: match trend {
                        Trend::Down => BreakKind::Choch,
                        _ => BreakKind::Bos,
                    },
                    level: h.price,
                    swing_idx: h.idx,
                });
                consumed_high = Some(h.idx);
            }
        }
        if let Some(l) = lows.last() {
            if consumed_low != Some(l.idx) && bars[i].close < l.price {
                out.push(StructureBreak {
                    idx: i,
                    dir: BreakDir::Down,
                    kind: match trend {
                        Trend::Up => BreakKind::Choch,
                        _ => BreakKind::Bos,
                    },
                    level: l.price,
                    swing_idx: l.idx,
                });
                consumed_low = Some(l.idx);
            }
        }
    }
    out
}
