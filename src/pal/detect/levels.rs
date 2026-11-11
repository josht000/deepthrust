//! Levels: clusters of confirmed same-kind swings within tol*ATR of the
//! cluster mean, needing `min_touches` touches (spec: 0.1/0.25/0.5 ATR,
//! 1-4 touches, drawn as a line or a box). A level is born when its
//! `min_touches`-th swing confirms; later same-direction swings inside
//! the tolerance add touches.

use crate::pal::detect::swings::{Swing, SwingKind};
use crate::pal::types::ChartBar;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LevelKind {
    /// cluster of swing highs
    Resistance,
    /// cluster of swing lows
    Support,
}

#[derive(Clone, Debug)]
pub struct Level {
    pub kind: LevelKind,
    /// FINAL representative price: mean of ALL touch prices, including
    /// touches confirmed after `born`. Do not use at decision time —
    /// use `as_of` (look-ahead).
    pub line: f64,
    /// FINAL box [lo, hi] of all touch prices — same caveat as `line`.
    pub lo: f64,
    pub hi: f64,
    /// pivot indices of the touching swings, confirmation order
    pub touches: Vec<usize>,
    /// bar where the min_touches-th touch confirmed
    pub born: usize,
    /// confirmation bar per touch, same order as `touches`
    confirmed: Vec<usize>,
    /// prefix stats: pref_mean[k] = mean price of the first k+1 touches
    pref_mean: Vec<f64>,
    pref_min: Vec<f64>,
    pref_max: Vec<f64>,
}

/// Level geometry known at a chart bar — only touches confirmed by
/// then. This is what a real-time observer would see.
#[derive(Clone, Copy, Debug)]
pub struct LevelAt {
    pub line: f64,
    pub lo: f64,
    pub hi: f64,
    pub n_touches: usize,
}

impl Level {
    /// Geometry usable at chart bar `at` (touches confirmed <= at).
    /// `None` before `born` (fewer than min_touches touches confirmed).
    pub fn as_of(&self, at: usize) -> Option<LevelAt> {
        if at < self.born {
            return None;
        }
        let k = self.confirmed.partition_point(|&c| c <= at);
        debug_assert!(k >= 1);
        Some(LevelAt {
            line: self.pref_mean[k - 1],
            lo: self.pref_min[k - 1],
            hi: self.pref_max[k - 1],
            n_touches: k,
        })
    }
}

/// Cluster same-kind confirmed swings into levels. `atr` is the series
/// usable-at-bar ATR (atr_series output); tolerance is measured against
/// the ATR at the newest swing's confirmation.
pub fn levels(
    bars: &[ChartBar],
    swings: &[Swing],
    kind: LevelKind,
    tol_atr: f64,
    atr: &[Option<f64>],
    min_touches: usize,
) -> Vec<Level> {
    let want = match kind {
        LevelKind::Resistance => SwingKind::High,
        LevelKind::Support => SwingKind::Low,
    };
    let mut pts: Vec<&Swing> = swings.iter().filter(|s| s.kind == want).collect();
    pts.sort_by_key(|s| s.confirmed);

    let mut out = Vec::new();
    let mut cluster: Vec<&Swing> = Vec::new();
    for s in pts {
        let tol = atr
            .get(s.confirmed)
            .and_then(|a| *a)
            .unwrap_or(0.0)
            * tol_atr;
        let mean = if cluster.is_empty() {
            0.0
        } else {
            cluster.iter().map(|c| c.price).sum::<f64>() / cluster.len() as f64
        };
        if cluster.is_empty() || (s.price - mean).abs() <= tol {
            cluster.push(s);
        } else {
            emit(&mut out, &cluster, min_touches);
            cluster = vec![s];
        }
    }
    emit(&mut out, &cluster, min_touches);
    let _ = bars;
    out
}

fn emit(out: &mut Vec<Level>, cluster: &[&Swing], min_touches: usize) {
    if cluster.len() < min_touches || min_touches == 0 {
        return;
    }
    let kind = match cluster[0].kind {
        SwingKind::High => LevelKind::Resistance,
        SwingKind::Low => LevelKind::Support,
    };
    let prices: Vec<f64> = cluster.iter().map(|s| s.price).collect();
    let line = prices.iter().sum::<f64>() / prices.len() as f64;
    let lo = prices.iter().cloned().fold(f64::INFINITY, f64::min);
    let hi = prices.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let mut confirmed = Vec::with_capacity(cluster.len());
    let mut pref_mean = Vec::with_capacity(cluster.len());
    let mut pref_min = Vec::with_capacity(cluster.len());
    let mut pref_max = Vec::with_capacity(cluster.len());
    let (mut sum, mut mn, mut mx) = (0.0, f64::INFINITY, f64::NEG_INFINITY);
    for (i, s) in cluster.iter().enumerate() {
        confirmed.push(s.confirmed);
        sum += s.price;
        mn = mn.min(s.price);
        mx = mx.max(s.price);
        pref_mean.push(sum / (i + 1) as f64);
        pref_min.push(mn);
        pref_max.push(mx);
    }
    out.push(Level {
        kind,
        line,
        lo,
        hi,
        touches: cluster.iter().map(|s| s.idx).collect(),
        born: cluster[min_touches - 1].confirmed,
        confirmed,
        pref_mean,
        pref_min,
        pref_max,
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pal::detect::swings::SwingKind;

    fn swing(idx: usize, price: f64, confirmed: usize) -> Swing {
        Swing {
            idx,
            price,
            kind: SwingKind::Low,
            confirmed,
        }
    }

    fn bar() -> ChartBar {
        ChartBar {
            open_idx: 0,
            usable_from: 0,
            open: 0.0,
            high: 0.0,
            low: 0.0,
            close: 0.0,
            volume: 0,
            n_minutes: 1,
        }
    }

    #[test]
    fn as_of_uses_only_confirmed_touches() {
        // cluster: 100 (conf 10), 101 (conf 50); outlier 200 (conf 60)
        // starts a second cluster. tol big enough that 101 joins but
        // 200 doesn't.
        let bars = vec![bar(); 4];
        let atr: Vec<Option<f64>> = vec![Some(100.0); 64];
        let sw = vec![
            swing(1, 100.0, 10),
            swing(2, 101.0, 50),
            swing(3, 200.0, 60),
        ];
        let lv = levels(&bars, &sw, LevelKind::Support, 0.25, &atr, 1);
        assert_eq!(lv.len(), 2);
        let a = &lv[0];
        assert_eq!(a.born, 10);
        // final-state fields DO see the later touch (callers must not
        // use them at decision time)
        assert_eq!(a.line, 100.5);
        assert_eq!(a.lo, 100.0);
        assert_eq!(a.hi, 101.0);
        // as-of before the second touch confirms: single-touch geometry
        let g = a.as_of(10).unwrap();
        assert_eq!((g.line, g.lo, g.hi, g.n_touches), (100.0, 100.0, 100.0, 1));
        let g = a.as_of(49).unwrap();
        assert_eq!(g.n_touches, 1);
        // after the second touch: two-touch geometry
        let g = a.as_of(50).unwrap();
        assert_eq!((g.line, g.lo, g.hi, g.n_touches), (100.5, 100.0, 101.0, 2));
        // before born: nothing
        assert!(a.as_of(9).is_none());
    }

    #[test]
    fn as_of_respects_min_touches_birth() {
        let bars = vec![bar(); 4];
        let atr: Vec<Option<f64>> = vec![Some(100.0); 64];
        let sw = vec![swing(1, 100.0, 10), swing(2, 101.0, 50)];
        let lv = levels(&bars, &sw, LevelKind::Support, 0.25, &atr, 2);
        assert_eq!(lv.len(), 1);
        let a = &lv[0];
        assert_eq!(a.born, 50); // born when the 2nd touch confirms
        assert!(a.as_of(49).is_none());
        assert_eq!(a.as_of(50).unwrap().n_touches, 2);
    }
}
