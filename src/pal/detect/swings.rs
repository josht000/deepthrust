//! Swing highs and lows of strength s: a swing high at bar i needs
//! `s` strictly lower highs on each side; usable from bar i+s
//! (confirmation), never earlier.

use crate::pal::types::ChartBar;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SwingKind {
    High,
    Low,
}

#[derive(Clone, Copy, Debug)]
pub struct Swing {
    /// index of the pivot bar
    pub idx: usize,
    pub price: f64,
    pub kind: SwingKind,
    /// first bar index at which the swing is usable (idx + strength)
    pub confirmed: usize,
}

/// All swings in the series, sorted by pivot index.
pub fn swings(bars: &[ChartBar], strength: usize) -> Vec<Swing> {
    let n = bars.len();
    let mut out = Vec::new();
    if strength == 0 || n < 2 * strength + 1 {
        return out;
    }
    for i in strength..n - strength {
        let h = bars[i].high;
        let l = bars[i].low;
        let mut is_high = true;
        let mut is_low = true;
        for j in 1..=strength {
            if bars[i - j].high >= h || bars[i + j].high >= h {
                is_high = false;
            }
            if bars[i - j].low <= l || bars[i + j].low <= l {
                is_low = false;
            }
        }
        if is_high {
            out.push(Swing {
                idx: i,
                price: h,
                kind: SwingKind::High,
                confirmed: i + strength,
            });
        }
        if is_low {
            out.push(Swing {
                idx: i,
                price: l,
                kind: SwingKind::Low,
                confirmed: i + strength,
            });
        }
    }
    out
}

/// Swings usable at (i.e. confirmed before) bar `at`.
pub fn usable_at<'a>(swings: &'a [Swing], at: usize) -> impl Iterator<Item = &'a Swing> {
    swings.iter().filter(move |s| s.confirmed <= at)
}

/// Last `k` confirmed swings of one kind usable at bar `at`.
pub fn last_of_kind(swings: &[Swing], kind: SwingKind, at: usize, k: usize) -> Vec<Swing> {
    let mut v: Vec<Swing> = usable_at(swings, at)
        .filter(|s| s.kind == kind)
        .cloned()
        .collect();
    if v.len() > k {
        v = v.split_off(v.len() - k);
    }
    v
}

/// Swings split by kind and sorted by confirmation index, for
/// O(log n) "last k confirmed" queries on the per-event hot path.
/// (`confirmed = idx + strength` is monotone, so the combined list
/// sorted by idx splits into same-order per-kind lists.)
#[derive(Clone, Debug)]
pub struct SwingIndex {
    pub highs: Vec<Swing>,
    pub lows: Vec<Swing>,
}

impl SwingIndex {
    pub fn new(swings: &[Swing]) -> SwingIndex {
        let mut highs: Vec<Swing> = swings
            .iter()
            .filter(|s| s.kind == SwingKind::High)
            .cloned()
            .collect();
        let mut lows: Vec<Swing> = swings
            .iter()
            .filter(|s| s.kind == SwingKind::Low)
            .cloned()
            .collect();
        highs.sort_by_key(|s| s.confirmed);
        lows.sort_by_key(|s| s.confirmed);
        SwingIndex { highs, lows }
    }

    /// last `k` confirmed swings of `kind` usable at `at`, oldest first.
    pub fn last_of(&self, kind: SwingKind, at: usize, k: usize) -> &[Swing] {
        let v = match kind {
            SwingKind::High => &self.highs,
            SwingKind::Low => &self.lows,
        };
        let end = v.partition_point(|s| s.confirmed <= at);
        &v[end.saturating_sub(k)..end]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bar(h: f64, l: f64) -> ChartBar {
        ChartBar {
            open_idx: 0,
            usable_from: 0,
            open: (h + l) / 2.0,
            high: h,
            low: l,
            close: (h + l) / 2.0,
            volume: 0,
            n_minutes: 1,
        }
    }

    #[test]
    fn finds_pivots() {
        // peak at idx 3: highs 1,2,3,5,3,2,1 ; lows mirrored
        let bars = vec![
            bar(1.0, -1.0),
            bar(2.0, -2.0),
            bar(3.0, -3.0),
            bar(5.0, -5.0),
            bar(3.0, -3.0),
            bar(2.0, -2.0),
            bar(1.0, -1.0),
        ];
        let sw = swings(&bars, 2);
        let high = sw.iter().find(|s| s.kind == SwingKind::High).unwrap();
        assert_eq!(high.idx, 3);
        assert_eq!(high.price, 5.0);
        assert_eq!(high.confirmed, 5);
        let low = sw.iter().find(|s| s.kind == SwingKind::Low).unwrap();
        assert_eq!(low.price, -5.0);
    }

    #[test]
    fn confirmation_semantics() {
        let bars = vec![
            bar(1.0, -1.0),
            bar(2.0, -2.0),
            bar(3.0, -3.0),
            bar(5.0, -5.0),
            bar(3.0, -3.0),
            bar(2.0, -2.0),
            bar(1.0, -1.0),
        ];
        let sw = swings(&bars, 2);
        assert_eq!(usable_at(&sw, 4).count(), 0); // not yet confirmed
        assert_eq!(usable_at(&sw, 5).count(), 2);
    }

    #[test]
    fn truncated_series_matches() {
        // no-lookahead: swings usable at cut must equal swings from the
        // truncated series
        let mut bars = Vec::new();
        for i in 0..30 {
            let x = (i as f64 * 0.7).sin() * 3.0;
            bars.push(bar(x + 1.0, x - 1.0));
        }
        let full = swings(&bars, 2);
        let cut = 20;
        let truncated = swings(&bars[..cut], 2);
        let usable_full: Vec<Swing> = usable_at(&full, cut).cloned().collect();
        assert_eq!(usable_full.len(), truncated.len());
        for (a, b) in usable_full.iter().zip(truncated.iter()) {
            assert_eq!(a.idx, b.idx);
            assert_eq!(a.price, b.price);
        }
    }
}
