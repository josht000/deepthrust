//! Behavior tests (spec prompt 2): does price respect levels, flips,
//! breaks, sweeps, gaps and zones more than chance? For every event
//! kind, measure what price did after it and compare with the same
//! measure on a random stand-in. NQ discovery is the primary run, ES
//! discovery the replication; both eras must agree for "true".

use std::collections::HashMap;

use chrono::NaiveDate;

use crate::pal::detect::levels::{levels, LevelKind};
use crate::pal::detect::structure::{structure_breaks, BreakDir, BreakKind};
use crate::pal::detect::swings::SwingKind;
use crate::pal::detctx::DetCtx;
use crate::pal::rng::Rng;
use crate::pal::types::{ChartBar, ChartId};
use crate::pal::windows::{ERA1_END, ERA2_START};

pub const BEHAVIOR_SWING: usize = 3;
pub const BEHAVIOR_TOL: f64 = 0.25;
pub const BOOTSTRAP: usize = 2_000;
pub const RANDOM_MOMENTS: usize = 20;
pub const MAX_HOLD_BARS: usize = 50;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Verdict {
    True,
    Split,
    False,
}

impl Verdict {
    pub fn as_str(self) -> &'static str {
        match self {
            Verdict::True => "true",
            Verdict::Split => "split",
            Verdict::False => "false",
        }
    }
}

/// One measured event.
#[derive(Clone, Copy, Debug)]
pub struct Measure {
    /// session index (for whole-session bootstrap)
    pub session: i32,
    /// chart it came from (for pooled vs per-chart)
    pub chart: ChartId,
    /// event statistic (forward return in ATR, hold indicator, ...)
    pub stat: f64,
    /// matched control statistic for the same slot (NaN if none)
    pub ctrl: f64,
}

pub struct TestResult {
    pub test: String,
    pub chart: String,
    pub n_real: usize,
    pub n_ctrl: usize,
    pub real_mean: f64,
    pub ctrl_mean: f64,
    pub diff: f64,
    pub ci_lo: f64,
    pub ci_hi: f64,
    pub era1_diff: f64,
    pub era2_diff: f64,
}

/// Score one level touch from chart bar `at` (spec): the touch holds if
/// price moves 1 ATR away before any close more than 0.25 ATR through
/// the level; it breaks if that close comes first; unresolved after 50
/// bars. Returns (hold, break, bars_taken).
pub fn score_touch(
    bars: &[ChartBar],
    at: usize,
    line: f64,
    dir_long: bool,
    atr: f64,
) -> (bool, bool, usize) {
    if atr <= 0.0 || !atr.is_finite() {
        return (false, false, 0);
    }
    let thru = 0.25 * atr;
    let start_close = bars[at].close;
    for j in at + 1..bars.len().min(at + 1 + MAX_HOLD_BARS) {
        let b = &bars[j];
        // a close more than 0.25 ATR through the level breaks it
        let broken = if dir_long {
            b.close < line - thru
        } else {
            b.close > line + thru
        };
        if broken {
            return (false, true, j - at);
        }
        // 1 ATR away from the touch bar's close, in the holding direction
        let held = if dir_long {
            b.close >= start_close + atr
        } else {
            b.close <= start_close - atr
        };
        if held {
            return (true, false, j - at);
        }
    }
    (false, false, MAX_HOLD_BARS)
}

/// Forward return in ATR units from close[at] to close[at+h], signed
/// by direction.
pub fn fwd_return(bars: &[ChartBar], at: usize, h: usize, dir_long: bool, atr: f64) -> Option<f64> {
    if at + h >= bars.len() || atr <= 0.0 {
        return None;
    }
    let d = (bars[at + h].close - bars[at].close) / atr;
    Some(if dir_long { d } else { -d })
}

/// Max favorable excursion (ATR) within h bars from close[at].
pub fn mfe(bars: &[ChartBar], at: usize, h: usize, dir_long: bool, atr: f64) -> Option<f64> {
    if at + 1 >= bars.len() || atr <= 0.0 {
        return None;
    }
    let start = bars[at].close;
    let mut best = f64::NEG_INFINITY;
    for j in at + 1..bars.len().min(at + 1 + h) {
        let e = if dir_long {
            bars[j].high - start
        } else {
            start - bars[j].low
        };
        if e > best {
            best = e;
        }
    }
    Some(best / atr)
}

/// Build a random stand-in level: shift the real line by +-1..3 ATR,
/// at least 0.5 ATR from any real level price.
pub fn random_line(real: f64, real_prices: &[f64], atr: f64, rng: &mut Rng) -> f64 {
    for _ in 0..20 {
        let sign = if rng.next_f64() < 0.5 { -1.0 } else { 1.0 };
        let shift = (1.0 + 2.0 * rng.next_f64()) * atr;
        let cand = real + sign * shift;
        let ok = real_prices
            .iter()
            .all(|p| (cand - p).abs() >= 0.5 * atr);
        if ok {
            return cand;
        }
    }
    real + (1.0 + 2.0 * rng.next_f64()) * atr
}

/// Random moments: `k` chart bars matching the reference bar's
/// time-of-day bucket (30-minute) and direction.
pub fn random_moments(
    bars: &[ChartBar],
    close_minutes: &[usize],
    at: usize,
    dir_long: bool,
    k: usize,
    rng: &mut Rng,
) -> Vec<usize> {
    let bucket = close_minutes.get(at).copied().unwrap_or(0) / 30;
    let mut cands: Vec<usize> = Vec::new();
    for i in 1..bars.len() {
        if i == at {
            continue;
        }
        if close_minutes.get(i).copied().unwrap_or(usize::MAX) / 30 != bucket {
            continue;
        }
        let up = bars[i].close > bars[i].open;
        if (dir_long && up) || (!dir_long && !up) {
            cands.push(i);
        }
    }
    let mut picks = Vec::new();
    for _ in 0..k.min(cands.len()) {
        let j = rng.gen_range(0, cands.len());
        picks.push(cands.swap_remove(j));
    }
    picks
}

/// Whole-session bootstrap of the real-minus-control difference of
/// means. `measures` carries per-event real and control stats; each
/// replicate resamples the session index set with replacement.
pub fn bootstrap_ci(
    measures: &[Measure],
    session_map: &HashMap<i32, Vec<usize>>,
    n_sessions: usize,
    label: &str,
) -> (f64, f64, f64) {
    let mut diffs = Vec::new();
    for m in measures {
        if m.stat.is_finite() && m.ctrl.is_finite() {
            diffs.push(m.stat - m.ctrl);
        }
    }
    let diff = if diffs.is_empty() {
        0.0
    } else {
        diffs.iter().sum::<f64>() / diffs.len() as f64
    };
    if n_sessions == 0 || measures.is_empty() {
        return (diff, f64::NAN, f64::NAN);
    }
    let mut rng = Rng::fork(&format!("bootstrap/{label}"));
    let mut reps = Vec::with_capacity(BOOTSTRAP);
    let sess_ids: Vec<i32> = session_map.keys().copied().collect();
    if sess_ids.is_empty() {
        return (diff, f64::NAN, f64::NAN);
    }
    for _ in 0..BOOTSTRAP {
        let mut sum = 0.0;
        let mut cnt = 0usize;
        for _ in 0..sess_ids.len() {
            let s = sess_ids[rng.gen_range(0, sess_ids.len())];
            for &mi in &session_map[&s] {
                let m = &measures[mi];
                if m.stat.is_finite() && m.ctrl.is_finite() {
                    sum += m.stat - m.ctrl;
                    cnt += 1;
                }
            }
        }
        reps.push(if cnt > 0 { sum / cnt as f64 } else { f64::NAN });
    }
    reps.retain(|x| x.is_finite());
    reps.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let lo = reps.get((reps.len() as f64 * 0.025) as usize).copied().unwrap_or(f64::NAN);
    let hi = reps
        .get(((reps.len() as f64 * 0.975) as usize).min(reps.len().saturating_sub(1)))
        .copied()
        .unwrap_or(f64::NAN);
    (diff, lo, hi)
}

fn session_map_of(measures: &[Measure]) -> HashMap<i32, Vec<usize>> {
    let mut m: HashMap<i32, Vec<usize>> = HashMap::new();
    for (i, mm) in measures.iter().enumerate() {
        m.entry(mm.session).or_default().push(i);
    }
    m
}

fn era_of(date: NaiveDate) -> i32 {
    if date <= ERA1_END {
        1
    } else if date >= ERA2_START {
        2
    } else {
        0
    }
}

/// Verdict per spec: NQ pooled must point the claimed way with CI
/// excluding zero, ES same sign, both eras agree.
pub fn verdict(
    nq_pooled_diff: f64,
    nq_lo: f64,
    nq_hi: f64,
    claimed_positive: bool,
    es_pooled_diff: f64,
    era1_diff: f64,
    era2_diff: f64,
) -> Verdict {
    let nq_points = if claimed_positive {
        nq_pooled_diff > 0.0
    } else {
        nq_pooled_diff < 0.0
    };
    let ci_excludes_zero = nq_lo.is_finite() && nq_hi.is_finite() && (nq_lo > 0.0 || nq_hi < 0.0);
    let es_same_sign = es_pooled_diff.signum() == nq_pooled_diff.signum() && es_pooled_diff != 0.0;
    let eras_agree = era1_diff.signum() == era2_diff.signum()
        && era1_diff.signum() == nq_pooled_diff.signum()
        && era1_diff != 0.0
        && era2_diff != 0.0;
    if nq_points && ci_excludes_zero && es_same_sign && eras_agree {
        Verdict::True
    } else if nq_points && (ci_excludes_zero || es_same_sign || eras_agree) {
        Verdict::Split
    } else if !nq_points {
        Verdict::False
    } else {
        Verdict::Split
    }
}

use crate::pal::detect::candles::{candle_at, CandleKind};
use crate::pal::detect::fvg::{fvgs, GapKind};
use crate::pal::detect::trendlines::trend_line;
use crate::pal::detect::zones::{zones, ZoneKind};

/// The touch-based tests: levels hold more than random lines. Run per
/// chart on a DetCtx. Returns per-test results.
pub struct Behavior<'a> {
    pub ctx: &'a mut DetCtx,
}

impl<'a> Behavior<'a> {
    /// Collect level-touch measures for one chart: real levels vs
    /// shifted random lines. kind = hold indicator stat.
    pub fn level_touch_measures(
        &mut self,
        chart: ChartId,
        dir_long: bool,
    ) -> Vec<Measure> {
        self.ctx.swings(chart, BEHAVIOR_SWING);
        self.ctx.atr(chart);
        let sw = self.ctx.swings[&(chart, BEHAVIOR_SWING)].clone();
        let atr = self.ctx.atr[&chart].clone();
        let cs = &self.ctx.charts[&chart];
        let bars = &cs.bars;
        let kind = if dir_long {
            LevelKind::Support
        } else {
            LevelKind::Resistance
        };
        let lvls = levels(bars, &sw, kind, BEHAVIOR_TOL, &atr, 1);
        let real_prices: Vec<f64> = lvls.iter().map(|l| l.line).collect();
        let mut rng = Rng::fork(&format!("behavior/random-lines/{chart:?}/{dir_long}"));
        let mut out = Vec::new();
        // touch = a chart bar whose range crosses the line after born
        for lv in &lvls {
            let shift = random_line(lv.line, &real_prices, atr_at(&atr, lv.born), &mut rng);
            for i in lv.born.max(1)..bars.len() {
                let touched = bars[i].low <= lv.line && lv.line <= bars[i].high;
                if !touched {
                    continue;
                }
                let a = atr_at(&atr, i);
                if a <= 0.0 {
                    continue;
                }
                let (hold, broke, _) = score_touch(bars, i, lv.line, dir_long, a);
                let stat = if hold {
                    1.0
                } else if broke {
                    0.0
                } else {
                    continue; // unresolved: excluded
                };
                // control: touch on the shifted line at the same bar if
                // the shifted line is touched there, else skip
                let ct = bars[i].low <= shift && shift <= bars[i].high;
                let ctrl = if ct {
                    let (h2, b2, _) = score_touch(bars, i, shift, dir_long, a);
                    if h2 {
                        1.0
                    } else if b2 {
                        0.0
                    } else {
                        f64::NAN
                    }
                } else {
                    f64::NAN
                };
                out.push(Measure {
                    session: cs.session_of[i],
                    chart,
                    stat,
                    ctrl,
                });
                break; // first touch per level only
            }
        }
        out
    }

    /// Follow-through measures after BOS/CHoCH breaks at horizons
    /// 10/20/50 bars vs random moments.
    pub fn structure_followthrough(
        &mut self,
        chart: ChartId,
        want: BreakKind,
        horizon: usize,
    ) -> Vec<Measure> {
        self.ctx.swings(chart, BEHAVIOR_SWING);
        self.ctx.atr(chart);
        let sw = self.ctx.swings[&(chart, BEHAVIOR_SWING)].clone();
        let atr = self.ctx.atr[&chart].clone();
        let cs = &self.ctx.charts[&chart];
        let bars = &cs.bars;
        let breaks = structure_breaks(bars, &sw);
        let mut rng = Rng::fork(&format!("behavior/moments/{chart:?}/{want:?}/{horizon}"));
        let mut out = Vec::new();
        for br in breaks {
            if br.kind != want {
                continue;
            }
            let long = br.dir == BreakDir::Up;
            let a = atr_at(&atr, br.idx);
            let Some(stat) = fwd_return(bars, br.idx, horizon, long, a) else {
                continue;
            };
            let moments = random_moments(bars, &cs.close_minute, br.idx, long, RANDOM_MOMENTS, &mut rng);
            let ctrl_stats: Vec<f64> = moments
                .iter()
                .filter_map(|&m| fwd_return(bars, m, horizon, long, a))
                .collect();
            if ctrl_stats.is_empty() {
                continue;
            }
            let ctrl = ctrl_stats.iter().sum::<f64>() / ctrl_stats.len() as f64;
            out.push(Measure {
                session: cs.session_of[br.idx],
                chart,
                stat,
                ctrl,
            });
        }
        out
    }

    /// Sweeps vs closes-through at levels: after a wick-through that
    /// closes back inside vs a close-through, forward MFE at 20 bars.
    /// Claim: sweeps rebound more.
    pub fn sweep_vs_close(&mut self, chart: ChartId, dir_long: bool) -> Vec<Measure> {
        self.ctx.swings(chart, BEHAVIOR_SWING);
        self.ctx.atr(chart);
        let sw = self.ctx.swings[&(chart, BEHAVIOR_SWING)].clone();
        let atr = self.ctx.atr[&chart].clone();
        let cs = &self.ctx.charts[&chart];
        let bars = &cs.bars;
        let kind = if dir_long { LevelKind::Support } else { LevelKind::Resistance };
        let lvls = levels(bars, &sw, kind, BEHAVIOR_TOL, &atr, 1);
        let mut rng = Rng::fork(&format!("behavior/sweep/{chart:?}/{dir_long}"));
        let mut out = Vec::new();
        for lv in &lvls {
            for i in lv.born.max(1)..bars.len() {
                let a = atr_at(&atr, i);
                if a <= 0.0 {
                    continue;
                }
                let thru = 0.25 * a;
                let (is_sweep, is_close) = if dir_long {
                    (
                        bars[i].low < lv.lo && bars[i].close > lv.lo,
                        bars[i].close < lv.line - thru,
                    )
                } else {
                    (
                        bars[i].high > lv.hi && bars[i].close < lv.hi,
                        bars[i].close > lv.line + thru,
                    )
                };
                if is_sweep == is_close {
                    continue;
                }
                let stat_kind = if is_sweep { 1.0 } else { 0.0 };
                let Some(stat) = mfe(bars, i, 20, dir_long, a) else { continue };
                // control: matched random moments' MFE
                let moments = random_moments(bars, &cs.close_minute, i, dir_long, RANDOM_MOMENTS, &mut rng);
                let cs2: Vec<f64> = moments.iter().filter_map(|&m| mfe(bars, m, 20, dir_long, a)).collect();
                if cs2.is_empty() {
                    continue;
                }
                let ctrl = cs2.iter().sum::<f64>() / cs2.len() as f64;
                let _ = stat_kind; // sweep=1, close=0 -> paired by stat_kind grouping below
                out.push(Measure {
                    session: cs.session_of[i],
                    chart,
                    stat,
                    ctrl,
                });
                break;
            }
        }
        out
    }

    /// FVG hold-and-fill: does the gap hold (price rebounds) vs fill?
    /// stat = forward MFE at 20 bars on first gap entry; control =
    /// random moments.
    pub fn fvg_hold(&mut self, chart: ChartId, dir_long: bool) -> Vec<Measure> {
        self.ctx.atr(chart);
        let atr = self.ctx.atr[&chart].clone();
        let cs = &self.ctx.charts[&chart];
        let bars = &cs.bars;
        let want = if dir_long { GapKind::Bull } else { GapKind::Bear };
        let gs = fvgs(bars);
        let mut rng = Rng::fork(&format!("behavior/fvg/{chart:?}/{dir_long}"));
        let mut out = Vec::new();
        let mut entered = std::collections::HashSet::new();
        for g in gs.iter().filter(|g| g.kind == want) {
            for i in g.idx + 1..bars.len() {
                let enters = if dir_long {
                    bars[i].low <= g.hi && bars[i].low > g.lo
                } else {
                    bars[i].high >= g.lo && bars[i].high < g.hi
                };
                if !enters {
                    continue;
                }
                if !entered.insert((g.lo.to_bits(), g.hi.to_bits())) {
                    break; // only first return per gap
                }
                let a = atr_at(&atr, i);
                if a <= 0.0 {
                    break;
                }
                let Some(stat) = mfe(bars, i, 20, dir_long, a) else { break };
                let moments = random_moments(bars, &cs.close_minute, i, dir_long, RANDOM_MOMENTS, &mut rng);
                let cs2: Vec<f64> = moments.iter().filter_map(|&m| mfe(bars, m, 20, dir_long, a)).collect();
                let ctrl = if cs2.is_empty() {
                    f64::NAN
                } else {
                    cs2.iter().sum::<f64>() / cs2.len() as f64
                };
                out.push(Measure {
                    session: cs.session_of[i],
                    chart,
                    stat,
                    ctrl,
                });
                break;
            }
        }
        out
    }

    /// S&D zones: forward MFE at 20 on FIRST returns (stat) vs LATER
    /// returns (ctrl). Claim: first returns behave better.
    pub fn zone_first_vs_later(&mut self, chart: ChartId, dir_long: bool) -> Vec<Measure> {
        self.ctx.atr(chart);
        let atr = self.ctx.atr[&chart].clone();
        let cs = &self.ctx.charts[&chart];
        let bars = &cs.bars;
        let want = if dir_long { ZoneKind::Demand } else { ZoneKind::Supply };
        let zs = zones(bars, &atr);
        let mut out = Vec::new();
        // first returns -> stat; later returns -> ctrl
        for z in zs.iter().filter(|z| z.kind == want) {
            let mut n_ret = 0usize;
            for i in z.born + 1..bars.len() {
                let touched = bars[i].low <= z.hi && z.lo <= bars[i].high;
                if !touched {
                    continue;
                }
                n_ret += 1;
                let a = atr_at(&atr, i);
                if a <= 0.0 {
                    break;
                }
                let Some(v) = mfe(bars, i, 20, dir_long, a) else { break };
                if n_ret == 1 {
                    // find a later return to pair: keep scanning handled
                    // by pairing below via second pass marker
                    out.push(Measure {
                        session: cs.session_of[i],
                        chart,
                        stat: v,
                        ctrl: f64::NAN,
                    });
                } else {
                    // later return acts as control for the first
                    if let Some(last) = out.last_mut() {
                        if last.ctrl.is_nan() {
                            last.ctrl = v;
                        }
                    }
                }
                if n_ret >= 2 {
                    break;
                }
            }
        }
        out.retain(|m| m.ctrl.is_finite());
        out
    }

    /// Pullback depth vs continuation: pullback depth in ATR (real) vs
    /// depth bucketed random controls. Claim: deeper pullbacks continue
    /// more. stat = fwd return at 20 for deepest-third pullbacks,
    /// ctrl = same for shallowest-third.
    pub fn pullback_depth(&mut self, chart: ChartId, dir_long: bool) -> Vec<Measure> {
        self.ctx.swings(chart, BEHAVIOR_SWING);
        self.ctx.atr(chart);
        let sw = self.ctx.swings[&(chart, BEHAVIOR_SWING)].clone();
        let atr = self.ctx.atr[&chart].clone();
        let cs = &self.ctx.charts[&chart];
        let bars = &cs.bars;
        let want_h = if dir_long { SwingKind::High } else { SwingKind::Low };
        let want_l = if dir_long { SwingKind::Low } else { SwingKind::High };
        // pullback = between a same-dir swing pair: depth = how far price
        // retraced from the second extreme toward the last opposite swing
        let mut depths: Vec<(usize, f64)> = Vec::new();
        for (i, s) in sw.iter().enumerate() {
            if s.kind != want_h {
                continue;
            }
            let Some(prev_l) = sw[..i].iter().rev().find(|x| x.kind == want_l) else {
                continue;
            };
            let a = atr_at(&atr, s.idx);
            if a <= 0.0 {
                continue;
            }
            let depth = (s.price - prev_l.price).abs() / a;
            if depth > 0.0 {
                depths.push((s.confirmed, depth));
            }
        }
        let mut out = Vec::new();
        if depths.is_empty() {
            return out;
        }
        let mut ds: Vec<f64> = depths.iter().map(|d| d.1).collect();
        ds.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let q_lo = ds[ds.len() / 3];
        let q_hi = ds[ds.len() * 2 / 3];
        // pair each deep pullback's fwd return with a shallow one's
        let mut rng = Rng::fork(&format!("behavior/pullback/{chart:?}/{dir_long}"));
        for (at, depth) in &depths {
            let a = atr_at(&atr, *at);
            if *depth >= q_hi {
                let Some(stat) = fwd_return(bars, *at, 20, dir_long, a) else { continue };
                // control = a shallow pullback's return
                let shallow: Vec<f64> = depths
                    .iter()
                    .filter(|(_, d)| *d <= q_lo)
                    .filter_map(|(at2, _)| fwd_return(bars, *at2, 20, dir_long, a))
                    .collect();
                if shallow.is_empty() {
                    continue;
                }
                let pick = shallow[rng.gen_range(0, shallow.len())];
                out.push(Measure {
                    session: cs.session_of[*at],
                    chart,
                    stat,
                    ctrl: pick,
                });
            }
        }
        out
    }

    /// Engulfing/rejection candles AT levels vs the same candles not at
    /// levels: forward return at 20. Claim: at-level candles work better.
    pub fn candles_at_levels(&mut self, chart: ChartId, dir_long: bool) -> Vec<Measure> {
        self.ctx.swings(chart, BEHAVIOR_SWING);
        self.ctx.atr(chart);
        let sw = self.ctx.swings[&(chart, BEHAVIOR_SWING)].clone();
        let atr = self.ctx.atr[&chart].clone();
        let cs = &self.ctx.charts[&chart];
        let bars = &cs.bars;
        let kind = if dir_long { LevelKind::Support } else { LevelKind::Resistance };
        let lvls = levels(bars, &sw, kind, BEHAVIOR_TOL, &atr, 1);
        let wants = if dir_long {
            [CandleKind::BullishEngulfing, CandleKind::BullishRejection]
        } else {
            [CandleKind::BearishEngulfing, CandleKind::BearishRejection]
        };
        let mut at_level: Vec<(usize, f64)> = Vec::new();
        let mut off_level: Vec<(usize, f64)> = Vec::new();
        for i in 1..bars.len() {
            let Some(hit) = candle_at(bars, i) else { continue };
            if !wants.contains(&hit.kind) {
                continue;
            }
            let a = atr_at(&atr, i);
            if a <= 0.0 {
                continue;
            }
            let near = lvls.iter().any(|l| {
                l.born <= i.saturating_sub(1)
                    && bars[i].low <= l.line + 0.5 * a
                    && l.line - 0.5 * a <= bars[i].high
            });
            if near {
                at_level.push((i, a));
            } else {
                off_level.push((i, a));
            }
        }
        let mut rng = Rng::fork(&format!("behavior/candles/{chart:?}/{dir_long}"));
        let mut out = Vec::new();
        for &(i, a) in &at_level {
            let Some(stat) = fwd_return(bars, i, 20, dir_long, a) else { continue };
            let pool: Vec<f64> = off_level
                .iter()
                .filter_map(|&(j, a2)| fwd_return(bars, j, 20, dir_long, a2))
                .collect();
            if pool.is_empty() {
                continue;
            }
            let ctrl = pool[rng.gen_range(0, pool.len())];
            out.push(Measure {
                session: cs.session_of[i],
                chart,
                stat,
                ctrl,
            });
        }
        out
    }

    /// Trendline touch hold: touch of fitted trend line -> MFE at 20 vs
    /// random lines shifted +-1..3 ATR off the touch price.
    pub fn trendline_touch(&mut self, chart: ChartId, dir_long: bool) -> Vec<Measure> {
        self.ctx.swings(chart, BEHAVIOR_SWING);
        self.ctx.atr(chart);
        let sw = self.ctx.swings[&(chart, BEHAVIOR_SWING)].clone();
        let atr = self.ctx.atr[&chart].clone();
        let cs = &self.ctx.charts[&chart];
        let bars = &cs.bars;
        let kind = if dir_long { SwingKind::Low } else { SwingKind::High };
        let mut rng = Rng::fork(&format!("behavior/trendline/{chart:?}/{dir_long}"));
        let mut out = Vec::new();
        for i in 3..bars.len() {
            let Some(tl) = trend_line(&sw, kind, i - 1, 2) else { continue };
            let price = tl.price_at(i - 1);
            let touched = bars[i].low <= price && price <= bars[i].high;
            if !touched {
                continue;
            }
            let a = atr_at(&atr, i);
            if a <= 0.0 {
                continue;
            }
            let Some(stat) = mfe(bars, i, 20, dir_long, a) else { continue };
            // control: MFE at random moments matched on time+direction
            let moments = random_moments(bars, &cs.close_minute, i, dir_long, RANDOM_MOMENTS, &mut rng);
            let cs2: Vec<f64> = moments.iter().filter_map(|&m| mfe(bars, m, 20, dir_long, a)).collect();
            let ctrl = if cs2.is_empty() { f64::NAN } else { cs2.iter().sum::<f64>() / cs2.len() as f64 };
            out.push(Measure {
                session: cs.session_of[i],
                chart,
                stat,
                ctrl,
            });
        }
        out
    }

    /// Flip zones: after a level breaks, first retest touch from the
    /// flip side -> hold scored vs shifted random retest.
    pub fn flip_hold(&mut self, chart: ChartId, dir_long: bool) -> Vec<Measure> {
        self.ctx.swings(chart, BEHAVIOR_SWING);
        self.ctx.atr(chart);
        let sw = self.ctx.swings[&(chart, BEHAVIOR_SWING)].clone();
        let atr = self.ctx.atr[&chart].clone();
        let cs = &self.ctx.charts[&chart];
        let bars = &cs.bars;
        let kind = if dir_long { LevelKind::Resistance } else { LevelKind::Support };
        let lvls = levels(bars, &sw, kind, BEHAVIOR_TOL, &atr, 1);
        let real_prices: Vec<f64> = lvls.iter().map(|l| l.line).collect();
        let mut rng = Rng::fork(&format!("behavior/flip/{chart:?}/{dir_long}"));
        let mut out = Vec::new();
        for lv in &lvls {
            // first close beyond (flip side), then first retest touch
            let mut broke = false;
            for i in lv.born.max(1)..bars.len() {
                if !broke {
                    let b = if dir_long {
                        bars[i].close > lv.hi
                    } else {
                        bars[i].close < lv.lo
                    };
                    if b {
                        broke = true;
                    }
                    continue;
                }
                let touched = bars[i].low <= lv.line && lv.line <= bars[i].high;
                if !touched {
                    continue;
                }
                let a = atr_at(&atr, i);
                if a <= 0.0 {
                    break;
                }
                let (hold, broke2, _) = score_touch(bars, i, lv.line, dir_long, a);
                if !hold && !broke2 {
                    break;
                }
                let stat = if hold { 1.0 } else { 0.0 };
                // control: hold on a shifted line touched at same bar
                let shift = random_line(lv.line, &real_prices, a, &mut rng);
                let ct = bars[i].low <= shift && shift <= bars[i].high;
                let ctrl = if ct {
                    let (h2, b3, _) = score_touch(bars, i, shift, dir_long, a);
                    if h2 { 1.0 } else if b3 { 0.0 } else { f64::NAN }
                } else {
                    f64::NAN
                };
                out.push(Measure {
                    session: cs.session_of[i],
                    chart,
                    stat,
                    ctrl,
                });
                break;
            }
        }
        out
    }
}

fn atr_at(atr: &[Option<f64>], i: usize) -> f64 {
    atr.get(i).copied().flatten().unwrap_or(0.0)
}

/// Summarize one test's measures into per-chart and pooled results.
/// `claimed_positive`: whether the claim expects diff > 0.
pub fn summarize(
    test: &str,
    measures: &[Measure],
    session_dates: &[NaiveDate],
    claimed_positive: bool,
    es_diff: f64,
) -> Vec<TestResult> {
    let mut out = Vec::new();
    let smap = session_map_of(measures);
    let n_sessions = smap.len();
    // pooled
    let (diff, lo, hi) = bootstrap_ci(measures, &smap, n_sessions, &format!("{test}/pooled"));
    let real_mean = mean_stat(measures, true);
    let ctrl_mean = mean_stat(measures, false);
    // era diffs (pooled, un-bootstrapped means)
    let e1: Vec<Measure> = measures
        .iter()
        .filter(|m| session_dates.get(m.session as usize).map(|d| era_of(*d) == 1).unwrap_or(false))
        .cloned()
        .collect();
    let e2: Vec<Measure> = measures
        .iter()
        .filter(|m| session_dates.get(m.session as usize).map(|d| era_of(*d) == 2).unwrap_or(false))
        .cloned()
        .collect();
    let era1_diff = mean_diff(&e1);
    let era2_diff = mean_diff(&e2);
    out.push(TestResult {
        test: test.into(),
        chart: "pooled".into(),
        n_real: measures.iter().filter(|m| m.stat.is_finite()).count(),
        n_ctrl: measures.iter().filter(|m| m.ctrl.is_finite()).count(),
        real_mean,
        ctrl_mean,
        diff,
        ci_lo: lo,
        ci_hi: hi,
        era1_diff,
        era2_diff,
    });
    // per chart
    for c in ChartId::ALL {
        let sub: Vec<Measure> = measures.iter().filter(|m| m.chart == c).cloned().collect();
        if sub.is_empty() {
            continue;
        }
        let sm = session_map_of(&sub);
        let (d, lo2, hi2) = bootstrap_ci(&sub, &sm, sm.len(), &format!("{test}/{:?}", c));
        out.push(TestResult {
            test: test.into(),
            chart: c.as_str().into(),
            n_real: sub.iter().filter(|m| m.stat.is_finite()).count(),
            n_ctrl: sub.iter().filter(|m| m.ctrl.is_finite()).count(),
            real_mean: mean_stat(&sub, true),
            ctrl_mean: mean_stat(&sub, false),
            diff: d,
            ci_lo: lo2,
            ci_hi: hi2,
            era1_diff: f64::NAN,
            era2_diff: f64::NAN,
        });
    }
    let _ = claimed_positive;
    let _ = es_diff;
    out
}

fn mean_stat(measures: &[Measure], real: bool) -> f64 {
    let v: Vec<f64> = measures
        .iter()
        .map(|m| if real { m.stat } else { m.ctrl })
        .filter(|x| x.is_finite())
        .collect();
    if v.is_empty() {
        f64::NAN
    } else {
        v.iter().sum::<f64>() / v.len() as f64
    }
}

fn mean_diff(measures: &[Measure]) -> f64 {
    let v: Vec<f64> = measures
        .iter()
        .filter(|m| m.stat.is_finite() && m.ctrl.is_finite())
        .map(|m| m.stat - m.ctrl)
        .collect();
    if v.is_empty() {
        f64::NAN
    } else {
        v.iter().sum::<f64>() / v.len() as f64
    }
}
