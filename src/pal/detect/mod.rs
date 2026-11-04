//! Detectors: swings, trend, structure (BOS/CHoCH), levels, trend lines,
//! supply/demand zones, fair value gaps, candles. All detectors run on a
//! chart's closed-bar series and never look ahead: a structure is usable
//! only from its confirmation bar.

pub mod candles;
pub mod fvg;
pub mod levels;
pub mod structure;
pub mod swings;
pub mod trend;
pub mod trendlines;
pub mod zones;
