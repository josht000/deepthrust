//! Candle patterns: engulfing and rejection candles (spec's
//! "engulfing and rejection candles at levels").

use crate::pal::types::ChartBar;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CandleKind {
    /// body engulfs the previous body, opposite direction
    BullishEngulfing,
    BearishEngulfing,
    /// long lower wick (>= wick_mult * body), small upper wick
    BullishRejection,
    /// long upper wick
    BearishRejection,
}

pub struct CandleHit {
    pub idx: usize,
    pub kind: CandleKind,
}

pub const WICK_MULT: f64 = 2.0;
pub const WICK_EPS: f64 = 1e-9;

/// Detect pattern at bar i.
pub fn candle_at(bars: &[ChartBar], i: usize) -> Option<CandleHit> {
    if i == 0 || i >= bars.len() {
        return None;
    }
    let b = &bars[i];
    let p = &bars[i - 1];
    let body = (b.close - b.open).abs();
    let up_wick = b.high - b.open.max(b.close);
    let dn_wick = b.open.min(b.close) - b.low;

    // engulfing: current body strictly contains previous body, opposite
    // direction
    let p_lo = p.open.min(p.close);
    let p_hi = p.open.max(p.close);
    let b_lo = b.open.min(b.close);
    let b_hi = b.open.max(b.close);
    if b.close > b.open && b_lo < p_lo && b_hi > p_hi && p.close < p.open {
        return Some(CandleHit {
            idx: i,
            kind: CandleKind::BullishEngulfing,
        });
    }
    if b.close < b.open && b_lo < p_lo && b_hi > p_hi && p.close > p.open {
        return Some(CandleHit {
            idx: i,
            kind: CandleKind::BearishEngulfing,
        });
    }
    // rejection: dominant wick >= WICK_MULT * body and >= 2x other wick
    if dn_wick >= WICK_MULT * body.max(WICK_EPS) && dn_wick >= 2.0 * up_wick {
        return Some(CandleHit {
            idx: i,
            kind: CandleKind::BullishRejection,
        });
    }
    if up_wick >= WICK_MULT * body.max(WICK_EPS) && up_wick >= 2.0 * dn_wick {
        return Some(CandleHit {
            idx: i,
            kind: CandleKind::BearishRejection,
        });
    }
    None
}
