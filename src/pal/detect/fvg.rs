//! Three-candle fair value gaps (spec): bullish when low[i] >
//! high[i-2]; the gap is [high[i-2], low[i]]. Bearish mirrored.

use crate::pal::types::ChartBar;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GapKind {
    /// low[i] > high[i-2]: gap up
    Bull,
    /// high[i] < low[i-2]: gap down
    Bear,
}

#[derive(Clone, Copy, Debug)]
pub struct Fvg {
    pub kind: GapKind,
    /// bottom of the gap (Bull: high[i-2]; Bear: high[i])
    pub lo: f64,
    /// top of the gap (Bull: low[i]; Bear: low[i-2])
    pub hi: f64,
    /// the third candle's index; gap usable from its close
    pub idx: usize,
}

pub fn fvgs(bars: &[ChartBar]) -> Vec<Fvg> {
    let n = bars.len();
    let mut out = Vec::new();
    for i in 2..n {
        if bars[i].low > bars[i - 2].high {
            out.push(Fvg {
                kind: GapKind::Bull,
                lo: bars[i - 2].high,
                hi: bars[i].low,
                idx: i,
            });
        } else if bars[i].high < bars[i - 2].low {
            out.push(Fvg {
                kind: GapKind::Bear,
                lo: bars[i].high,
                hi: bars[i - 2].low,
                idx: i,
            });
        }
    }
    out
}
