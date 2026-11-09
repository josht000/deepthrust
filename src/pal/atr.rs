//! Wilder's 14-bar ATR over chart bars, using only bars before the
//! current one (spec). `atr_before[i]` is the ATR available *at* bar i —
//! i.e. computed from bars[0..i] — so detectors never peek.

use crate::pal::types::ChartBar;

pub const ATR_PERIOD: usize = 14;

fn true_range(prev_close: f64, b: &ChartBar) -> f64 {
    (b.high - b.low)
        .max((b.high - prev_close).abs())
        .max((b.low - prev_close).abs())
}

/// Wilder ATR series. Returns a vector where `out[i]` is the ATR value
/// usable at bar i (built from bars strictly before i). `None` until at
/// least `period` true-range values exist before the bar.
pub fn atr_series(bars: &[ChartBar], period: usize) -> Vec<Option<f64>> {
    let n = bars.len();
    let mut out = vec![None; n];
    if n < 2 {
        return out;
    }
    // TR for bar j uses prev close of bar j-1.
    let mut tr = Vec::with_capacity(n - 1);
    for j in 1..n {
        tr.push(true_range(bars[j - 1].close, &bars[j]));
    }
    // atr ending at bar j (Wilder): seed with mean of first `period` TRs.
    // atr_end[j] defined for j >= period (index into bars).
    let mut atr_end: Vec<Option<f64>> = vec![None; n];
    if n - 1 >= period {
        let seed: f64 = tr[..period].iter().sum::<f64>() / period as f64;
        atr_end[period] = Some(seed);
        for j in period + 1..n {
            let prev = atr_end[j - 1].unwrap();
            atr_end[j] = Some((prev * (period as f64 - 1.0) + tr[j - 1]) / period as f64);
        }
    }
    // usable at bar i = atr_end[i-1]
    for i in 1..n {
        out[i] = atr_end[i - 1];
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bar(h: f64, l: f64, c: f64) -> ChartBar {
        ChartBar {
            open_idx: 0,
            usable_from: 0,
            open: c,
            high: h,
            low: l,
            close: c,
            volume: 0,
            n_minutes: 1,
        }
    }

    #[test]
    fn constant_range_converges() {
        // TR=2 constant -> ATR -> 2
        let mut bars = vec![bar(2.0, 0.0, 1.0)];
        for i in 1..40 {
            bars.push(bar(1.0 + i as f64, -1.0 + i as f64, i as f64));
        }
        let a = atr_series(&bars, 14);
        assert!(a[0].is_none());
        assert!(a[15].is_some());
        let v = a[39].unwrap();
        assert!((v - 2.0).abs() < 1e-9);
    }

    #[test]
    fn uses_only_prior_bars() {
        // ATR at bar i must not change if bars[i..] change.
        let mut bars = vec![bar(2.0, 0.0, 1.0)];
        for i in 1..40 {
            bars.push(bar(1.0 + i as f64, -1.0 + i as f64, i as f64));
        }
        let a1 = atr_series(&bars, 14);
        // modify the last bar violently
        bars[39].high = 10_000.0;
        let a2 = atr_series(&bars, 14);
        assert_eq!(a1[39], a2[39]); // atr usable AT 39 unchanged
        assert_ne!(a1.len(), 0);
    }
}
