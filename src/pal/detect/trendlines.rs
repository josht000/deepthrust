//! Trend lines: through the last `n_anchor` confirmed same-kind swings
//! (least-squares through 2-3 anchors), projected forward. Usable from
//! the last anchor's confirmation. `line_at` gives the projected price.

use crate::pal::detect::swings::{SwingIndex, SwingKind};

#[derive(Clone, Debug)]
pub struct TrendLine {
    /// slope in price per bar
    pub slope: f64,
    /// price at bar index (line = slope*idx + intercept)
    pub intercept: f64,
    /// bar where the line becomes usable (last anchor confirmed)
    pub born: usize,
    /// anchor pivot indices
    pub anchors: Vec<usize>,
    pub kind: SwingKind,
}

/// Fit a line through the last `n_anchor` confirmed swings of `kind`
/// usable at bar `at`. Returns None with fewer than 2 anchors.
pub fn trend_line(idx: &SwingIndex, kind: SwingKind, at: usize, n_anchor: usize) -> Option<TrendLine> {
    let anchors = idx.last_of(kind, at, n_anchor);
    if anchors.len() < 2 {
        return None;
    }
    // least squares through (idx, price)
    let n = anchors.len() as f64;
    let sx: f64 = anchors.iter().map(|a| a.idx as f64).sum();
    let sy: f64 = anchors.iter().map(|a| a.price).sum();
    let sxx: f64 = anchors.iter().map(|a| (a.idx as f64).powi(2)).sum();
    let sxy: f64 = anchors.iter().map(|a| a.idx as f64 * a.price).sum();
    let denom = n * sxx - sx * sx;
    if denom.abs() < 1e-12 {
        return None;
    }
    let slope = (n * sxy - sx * sy) / denom;
    let intercept = (sy - slope * sx) / n;
    Some(TrendLine {
        slope,
        intercept,
        born: anchors.last().unwrap().confirmed,
        anchors: anchors.iter().map(|a| a.idx).collect(),
        kind,
    })
}

impl TrendLine {
    pub fn price_at(&self, idx: usize) -> f64 {
        self.slope * idx as f64 + self.intercept
    }
}
