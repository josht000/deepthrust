//! Detection context: one market's bar universe as a continuous global
//! timeline. The 1-minute bars of all sessions sit in one flat array;
//! each higher chart's bars form a parallel continuous series indexed
//! by global 1-minute position, so swings/levels/ATR span sessions and
//! detectors only ever see structures confirmed before the trigger.

use std::collections::HashMap;

use crate::pal::atr::atr_series;
use crate::pal::detect::swings::{self, Swing, SwingIndex};
use crate::pal::engine::BarCtx;
use crate::pal::resample::build_chart;
use crate::pal::sessions::minute_index;
use crate::pal::types::{Bar, ChartBar, ChartId, Session};

pub struct DetCtx {
    /// all 1-minute bars, sessions concatenated in order
    pub bars: Vec<Bar>,
    /// engine context per 1-minute bar
    pub ctx: Vec<BarCtx>,
    /// session index per 1-minute bar (== ctx.session but kept handy)
    pub session_of: Vec<i32>,
    /// minute index within session per 1-minute bar
    pub minute_of: Vec<usize>,
    /// session dates in order
    pub session_dates: Vec<chrono::NaiveDate>,
    /// first global bar position of each session
    pub session_start: Vec<usize>,
    /// per-chart continuous series
    pub charts: HashMap<ChartId, ChartSeries>,
    /// per-(chart, strength) swing list, lazily filled
    pub swings: HashMap<(ChartId, usize), Vec<Swing>>,
    /// per-(chart, strength) binary-searchable swing index
    pub swing_idx: HashMap<(ChartId, usize), SwingIndex>,
    /// per-chart ATR series (usable at bar i from bars[0..i])
    pub atr: HashMap<ChartId, Vec<Option<f64>>>,
}

/// One chart's continuous bar series across all sessions.
pub struct ChartSeries {
    pub bars: Vec<ChartBar>,
    /// global 1-minute position at which each chart bar becomes usable:
    /// the first 1m bar index >= the bar's nominal close minute within
    /// its session (or the next session's first bar when it ran long).
    pub usable_pos: Vec<usize>,
    /// session index each chart bar belongs to
    pub session_of: Vec<i32>,
    /// minute-index-of-close-within-session per chart bar
    pub close_minute: Vec<usize>,
}

impl DetCtx {
    /// Build from sessions. `roll_minutes` marks session-minute indices
    /// that are within 60 minutes before a roll boundary (no entries);
    /// `last_before_roll` marks the global 1m position of the last bar
    /// before a roll for each affected session (usize::MAX default).
    pub fn build(sessions: &[Session], roll_boundaries: &HashMap<i32, usize>) -> DetCtx {
        let mut bars = Vec::new();
        let mut ctx = Vec::new();
        let mut session_of = Vec::new();
        let mut minute_of = Vec::new();
        let mut session_dates = Vec::new();
        let mut session_start = Vec::new();
        // global minute -> position maps per session for usable_pos
        let mut session_minute_pos: Vec<HashMap<usize, usize>> = Vec::new();

        for (si, s) in sessions.iter().enumerate() {
            session_start.push(bars.len());
            session_dates.push(s.date);
            let mut minpos = HashMap::new();
            for b in &s.bars {
                let mi = minute_index(b.ts_utc);
                let Some(mi) = mi else { continue };
                let pos = bars.len();
                minpos.entry(mi).or_insert(pos);
                let last_roll = roll_boundaries
                    .get(&(si as i32))
                    .map(|&p| p == pos)
                    .unwrap_or(false);
                session_of.push(si as i32);
                minute_of.push(mi);
                ctx.push(BarCtx {
                    session: si as i32,
                    minute_global: si as u64 * 1380 + mi as u64,
                    pre_roll: false, // filled below
                    last_before_roll: last_roll,
                });
                bars.push(b.clone());
            }
            session_minute_pos.push(minpos);
        }

        // pre_roll flags: bars in the 60 minutes before each session's
        // roll boundary
        for (si, s) in sessions.iter().enumerate() {
            if let Some(&roll_pos) = roll_boundaries.get(&(si as i32)) {
                // mark 60 minutes of session time before the roll bar
                let roll_minute = minute_of
                    .get(roll_pos)
                    .copied()
                    .unwrap_or(usize::MAX);
                if roll_minute != usize::MAX {
                    let lo = roll_minute.saturating_sub(60);
                    for pos in session_start[si]..bars.len().min(session_start.get(si + 1).copied().unwrap_or(bars.len())) {
                        if minute_of[pos] >= lo && minute_of[pos] < roll_minute {
                            ctx[pos].pre_roll = true;
                        }
                    }
                }
            }
            let _ = s;
        }

        // chart series
        let mut charts = HashMap::new();
        for chart in ChartId::ALL {
            if chart == ChartId::M1 {
                // 1m chart = the raw stream
                let mut series = ChartSeries {
                    bars: Vec::new(),
                    usable_pos: Vec::new(),
                    session_of: Vec::new(),
                    close_minute: Vec::new(),
                };
                for (pos, b) in bars.iter().enumerate() {
                    series.bars.push(ChartBar {
                        open_idx: minute_of[pos],
                        usable_from: minute_of[pos] + 1,
                        open: b.open,
                        high: b.high,
                        low: b.low,
                        close: b.close,
                        volume: b.volume,
                        n_minutes: 1,
                    });
                    series.usable_pos.push(pos + 1);
                    series.session_of.push(session_of[pos]);
                    series.close_minute.push(minute_of[pos]);
                }
                charts.insert(chart, series);
                continue;
            }
            let mut series = ChartSeries {
                bars: Vec::new(),
                usable_pos: Vec::new(),
                session_of: Vec::new(),
                close_minute: Vec::new(),
            };
            for (si, s) in sessions.iter().enumerate() {
                let cbs = build_chart(s, chart);
                for cb in cbs {
                    // first 1m pos at/after usable_from minute in this session
                    let pos = session_minute_pos[si]
                        .iter()
                        .filter(|(m, _)| **m >= cb.usable_from)
                        .map(|(_, p)| *p)
                        .min()
                        .unwrap_or_else(|| {
                            session_start.get(si + 1).copied().unwrap_or(bars.len())
                        });
                    series.usable_pos.push(pos);
                    series.session_of.push(si as i32);
                    series.close_minute.push(cb.usable_from.saturating_sub(1));
                    series.bars.push(cb);
                }
            }
            charts.insert(chart, series);
        }

        DetCtx {
            bars,
            ctx,
            session_of,
            minute_of,
            session_dates,
            session_start,
            charts,
            swings: HashMap::new(),
            swing_idx: HashMap::new(),
            atr: HashMap::new(),
        }
    }

    /// Swings for (chart, strength), computed once.
    pub fn swings(&mut self, chart: ChartId, strength: usize) -> &Vec<Swing> {
        self.swings
            .entry((chart, strength))
            .or_insert_with(|| swings::swings(&self.charts[&chart].bars, strength))
    }

    /// Binary-searchable swing index for (chart, strength), built once.
    pub fn swing_index(&mut self, chart: ChartId, strength: usize) -> &SwingIndex {
        if !self.swing_idx.contains_key(&(chart, strength)) {
            let si = SwingIndex::new(self.swings(chart, strength));
            self.swing_idx.insert((chart, strength), si);
        }
        &self.swing_idx[&(chart, strength)]
    }

    /// ATR series for a chart (usable at bar i from bars[..i]).
    pub fn atr(&mut self, chart: ChartId) -> &Vec<Option<f64>> {
        self.atr
            .entry(chart)
            .or_insert_with(|| atr_series(&self.charts[&chart].bars, crate::pal::atr::ATR_PERIOD))
    }

    /// Number of chart bars usable at global 1-minute position `pos`
    /// (all chart bars with usable_pos <= pos).
    pub fn usable_count(&self, chart: ChartId, pos: usize) -> usize {
        let s = &self.charts[&chart];
        s.usable_pos.partition_point(|&p| p <= pos)
    }

    /// Global 1-minute position of the first bar at/after a session's
    /// minute index (None when beyond the session's bars — caller uses
    /// next session start).
    pub fn pos_at_minute(&self, session: i32, minute: usize) -> Option<usize> {
        let start = self.session_start[session as usize];
        let end = self
            .session_start
            .get(session as usize + 1)
            .copied()
            .unwrap_or(self.bars.len());
        (start..end).find(|&p| self.minute_of[p] >= minute)
    }
}
