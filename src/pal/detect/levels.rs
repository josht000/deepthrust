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
    /// representative price: mean of touch prices
    pub line: f64,
    /// box: [lo, hi] of touch prices
    pub lo: f64,
    pub hi: f64,
    /// pivot indices of the touching swings
    pub touches: Vec<usize>,
    /// bar where the min_touches-th touch confirmed
    pub born: usize,
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
    out.push(Level {
        kind,
        line,
        lo,
        hi,
        touches: cluster.iter().map(|s| s.idx).collect(),
        born: cluster[min_touches - 1].confirmed,
    });
}
