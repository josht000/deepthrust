//! Supply and demand zones (spec): runs of 2-5 candles covering 1.5-3
//! ATR. The run's origin — the last opposite candle (the base) — forms
//! the zone [base.low, base.body_top] for demand, [base.body_bottom,
//! base.high] for supply.

use crate::pal::types::ChartBar;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ZoneKind {
    /// run of down candles: expect resistance on return
    Supply,
    /// run of up candles: expect support on return
    Demand,
}

#[derive(Clone, Copy, Debug)]
pub struct Zone {
    pub kind: ZoneKind,
    pub lo: f64,
    pub hi: f64,
    /// first bar after the run (zone usable from run's last bar close)
    pub born: usize,
    /// bars spanned by the impulsive run
    pub run_start: usize,
    pub run_end: usize,
    /// size of the run in ATR units
    pub size_atr: f64,
}

/// Find zones. `run` = 2-5 consecutive same-direction candles whose
/// total range covers [1.5, 3.0] * ATR at run end. The base is the
/// candle immediately before the run.
pub fn zones(bars: &[ChartBar], atr: &[Option<f64>]) -> Vec<Zone> {
    let n = bars.len();
    let mut out = Vec::new();
    for end in 2..n {
        let Some(a) = atr.get(end).and_then(|x| *x) else {
            continue;
        };
        if a <= 0.0 {
            continue;
        }
        for run in 2..=5usize {
            if end + 1 < run + 1 {
                continue;
            }
            let start = end + 1 - run;
            let base = start.checked_sub(1);
            let Some(b) = base else { continue };
            let dir = (bars[end].close - bars[start].open).signum();
            if dir == 0.0 {
                continue;
            }
            // all candles in run move with dir
            let coherent = (start..=end).all(|i| {
                (bars[i].close - bars[i].open).signum() == dir
                    || (bars[i].close - bars[i].open).abs() < 1e-9
            });
            if !coherent {
                continue;
            }
            let covered = (bars[start..=end]
                .iter()
                .map(|x| x.high)
                .fold(f64::NEG_INFINITY, f64::max)
                - bars[start..=end]
                    .iter()
                    .map(|x| x.low)
                    .fold(f64::INFINITY, f64::min))
                / a;
            if !(1.5..=3.0).contains(&covered) {
                continue;
            }
            let bb = &bars[b];
            let (kind, lo, hi) = if dir > 0.0 {
                (
                    ZoneKind::Demand,
                    bb.low,
                    bb.open.max(bb.close), // body top
                )
            } else {
                (
                    ZoneKind::Supply,
                    bb.open.min(bb.close), // body bottom
                    bb.high,
                )
            };
            if lo < hi {
                out.push(Zone {
                    kind,
                    lo,
                    hi,
                    born: end,
                    run_start: start,
                    run_end: end,
                    size_atr: covered,
                });
            }
            break; // longest matching run at this end only
        }
    }
    out
}
