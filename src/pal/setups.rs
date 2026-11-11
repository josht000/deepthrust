//! The ten setups as event detectors. Each maps dial values to `Event`s
//! at trigger-chart closes on the global timeline; the runner filters
//! by direction/entry-kind/filters and hands each surviving event to
//! the engine. Events carry every reference the engine might need so
//! one detection pass serves all entry/stop/target dial combinations.

use std::collections::HashMap;

use crate::pal::detect::candles::{candle_at, CandleKind};
use crate::pal::detect::fvg::{fvgs, GapKind};
use crate::pal::detect::levels::{levels, LevelKind};
use crate::pal::detect::structure::{structure_breaks, BreakDir, BreakKind};
use crate::pal::detect::swings::{SwingIndex, SwingKind};
use crate::pal::detect::trend::{trend_idx, Trend};
use crate::pal::detect::trendlines::trend_line;
use crate::pal::detect::zones::{zones, ZoneKind};
use crate::pal::detctx::DetCtx;
use crate::pal::engine::Direction;
use crate::pal::rulebook::Setup;
use crate::pal::types::{ChartBar, ChartId};

pub mod kind {
    pub const TOUCH: u16 = 1;
    pub const CLOSEBACK: u16 = 2;
    pub const REJ: u16 = 4;
    pub const ENG: u16 = 8;
    pub const BREAK: u16 = 16;
    pub const FLIP: u16 = 32;
    pub const SWEEP: u16 = 64;
}

/// Which entry kinds consume which event flags.
pub fn accepts(kinds: u16, entry: crate::pal::rulebook::EntryKind) -> bool {
    use crate::pal::rulebook::EntryKind::*;
    let need = match entry {
        LimitAtLevel => kind::TOUCH | kind::FLIP | kind::SWEEP,
        CloseBack => kind::CLOSEBACK | kind::SWEEP,
        Rejection => kind::REJ,
        Engulfing => kind::ENG,
        AtClose | LimitBack => kind::BREAK,
    };
    kinds & need != 0
}

#[derive(Clone, Copy, Debug)]
pub struct Event {
    /// global 1m position where the order goes live (the bar after the
    /// trigger bar's close)
    pub pos: usize,
    pub dir: Direction,
    /// level/zone/gap reference for limit entries
    pub ref_price: f64,
    /// "beyond level" stop reference (before the ATR buffer)
    pub stop_level: f64,
    /// "beyond swing" stop reference (NaN if unavailable)
    pub swing_ref: f64,
    /// opposing swing for next-swing targets (NaN if unavailable)
    pub next_swing: f64,
    /// ATR at the trigger (setup chart)
    pub atr: f64,
    pub kinds: u16,
    pub session: i32,
    pub minute: usize,
    pub trend: Trend,
    /// free ordinal slot: zone return count, gap visit count, ...
    pub meta: u16,
}

/// Detector dial bundle for one detection pass.
#[derive(Clone, Copy, Debug)]
pub struct DetectCfg {
    pub chart: ChartId,
    pub trigger: ChartId,
    pub swing_strength: usize,
    pub level_tol_atr: f64,
    pub touches: usize,
    pub zone_run: (usize, usize),
    pub zone_atr: (f64, f64),
    pub trendline_anchors: usize,
}

fn dir_of_trend(t: Trend) -> Option<Direction> {
    match t {
        Trend::Up => Some(Direction::Long),
        Trend::Down => Some(Direction::Short),
        Trend::None => None,
    }
}

/// Candle flag for a direction at a trigger bar.
fn candle_flags(tbars: &[ChartBar], ti: usize, dir: Direction) -> u16 {
    let Some(hit) = candle_at(tbars, ti) else { return 0 };
    match (dir, hit.kind) {
        (Direction::Long, CandleKind::BullishRejection)
        | (Direction::Short, CandleKind::BearishRejection) => kind::REJ,
        (Direction::Long, CandleKind::BullishEngulfing)
        | (Direction::Short, CandleKind::BearishEngulfing) => kind::ENG,
        _ => 0,
    }
}

fn last_swing(ctx: &DetCtx, chart: ChartId, strength: usize, kind: SwingKind, u: usize) -> f64 {
    ctx.swing_index(chart, strength)
        .last_of(kind, u.saturating_sub(1), 1)
        .last()
        .map(|s| s.price)
        .unwrap_or(f64::NAN)
}

fn base_event(
    ctx: &DetCtx,
    cfg: &DetectCfg,
    pos: usize,
    session: i32,
    minute: usize,
    u: usize,
) -> Event {
    let atr = ctx
        .atr(cfg.chart)
        .get(u.saturating_sub(1))
        .copied()
        .flatten()
        .unwrap_or(f64::NAN);
    let trend = trend_idx(
        &ctx.swing_index(cfg.chart, cfg.swing_strength),
        u.saturating_sub(1),
    );
    Event {
        pos,
        dir: Direction::Long,
        ref_price: f64::NAN,
        stop_level: f64::NAN,
        swing_ref: f64::NAN,
        next_swing: f64::NAN,
        atr,
        kinds: 0,
        session,
        minute,
        trend,
        meta: 0,
    }
}

/// Sorted price index for "which structures overlap this bar's range"
/// queries — turns the trigger x structure loops into range lookups.
struct MidIndex {
    keys: Vec<(f64, usize)>,
}

impl MidIndex {
    fn new(n: usize, key: impl Fn(usize) -> f64) -> MidIndex {
        let mut keys: Vec<(f64, usize)> = (0..n).map(|i| (key(i), i)).collect();
        keys.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
        MidIndex { keys }
    }
    /// indices with key in [lo, hi]
    fn range(&self, lo: f64, hi: f64) -> Vec<usize> {
        let a = self.keys.partition_point(|&(p, _)| p < lo);
        let b = self.keys.partition_point(|&(p, _)| p <= hi);
        self.keys[a..b].iter().map(|&(_, i)| i).collect()
    }
    /// indices with key in (lo, hi]
    fn range_gt(&self, lo: f64, hi: f64) -> Vec<usize> {
        let a = self.keys.partition_point(|&(p, _)| p <= lo);
        let b = self.keys.partition_point(|&(p, _)| p <= hi);
        self.keys[a..b].iter().map(|&(_, i)| i).collect()
    }
}

/// Run detection for one setup at one dial combination over the whole
/// context. This is the expensive, cacheable unit — every version with
/// these detector dials shares it.
pub fn detect_events(ctx: &DetCtx, setup: Setup, cfg: &DetectCfg) -> Vec<Event> {
    match setup {
        Setup::LevelBounce => detect_level_bounce(ctx, cfg),
        Setup::Breakout => detect_breakout(ctx, cfg),
        Setup::FlipZone => detect_flip(ctx, cfg),
        Setup::TrendPullback => detect_pullback(ctx, cfg),
        Setup::TrendLine => detect_trendline(ctx, cfg),
        Setup::BosContinuation => detect_structure(ctx, cfg, BreakKind::Bos),
        Setup::ChochReversal => detect_structure(ctx, cfg, BreakKind::Choch),
        Setup::SweepReclaim => detect_sweep(ctx, cfg),
        Setup::SupplyDemandZone => detect_zone(ctx, cfg),
        Setup::FairValueGap => detect_fvg(ctx, cfg),
    }
}

/// Trigger-bar iteration helper: for each trigger-chart bar, yields
/// (trigger index, global live pos, session, minute, setup-chart usable
/// count u).
fn each_trigger(ctx: &DetCtx, cfg: &DetectCfg) -> Vec<(usize, usize, i32, usize, usize)> {
    let ts = &ctx.charts[&cfg.trigger];
    let mut out = Vec::with_capacity(ts.bars.len());
    for (ti, cb) in ts.bars.iter().enumerate() {
        let pos = ts.usable_pos[ti];
        let u = ctx.usable_count(cfg.chart, pos);
        out.push((ti, pos, ts.session_of[ti], cb.usable_from.saturating_sub(1), u));
    }
    out
}

fn detect_level_bounce(ctx: &DetCtx, cfg: &DetectCfg) -> Vec<Event> {
    ctx.swings(cfg.chart, cfg.swing_strength);
    ctx.atr(cfg.chart);
    let sw = ctx.swings(cfg.chart, cfg.swing_strength);
    let atr = ctx.atr(cfg.chart);
    let sbars = &ctx.charts[&cfg.chart].bars;
    let mut lvls = Vec::new();
    for k in [LevelKind::Support, LevelKind::Resistance] {
        lvls.extend(levels(sbars, &sw, k, cfg.level_tol_atr, &atr, cfg.touches));
    }
    let tbars = ctx.charts[&cfg.trigger].bars.clone();
    let pad = lvls
        .iter()
        .map(|lv| lv.line - lv.lo)
        .chain(lvls.iter().map(|lv| lv.hi - lv.line))
        .fold(0.0f64, f64::max);
    let index = MidIndex::new(lvls.len(), |i| lvls[i].line);
    let mut out = Vec::new();
    // usable chart count at the previous trigger bar — the previous
    // contact check must see the level as it stood then, not now
    let mut pu = 0usize;
    for (ti, pos, session, minute, u) in each_trigger(ctx, cfg) {
        let tb = &tbars[ti];
        let mut cands = index.range(tb.low - pad, tb.high + pad);
        cands.sort_unstable();
        for li in cands {
            let lv = &lvls[li];
            let Some(la) = lv.as_of(u.saturating_sub(1)) else {
                continue;
            };
            let (dir, inside, stop_ref) = match lv.kind {
                LevelKind::Support => (Direction::Long, tb.close > la.line, la.lo),
                LevelKind::Resistance => (Direction::Short, tb.close < la.line, la.hi),
            };
            let touched = tb.low <= la.line && la.line <= tb.high;
            let poked = match dir {
                Direction::Long => tb.low < la.lo,
                Direction::Short => tb.high > la.hi,
            };
            if !(touched || poked && inside) {
                continue;
            }
            // one event per contact episode: skip bars continuing a
            // touch that already started on the previous trigger bar
            // (level geometry as of that bar's own decision time)
            let prev_contact = ti > 0
                && pu > 0
                && lv.as_of(pu.saturating_sub(1))
                    .map(|pa| {
                        let pb = &tbars[ti - 1];
                        (pb.low <= pa.line && pa.line <= pb.high)
                            || match dir {
                                Direction::Long => pb.low < pa.lo,
                                Direction::Short => pb.high > pa.hi,
                            }
                    })
                    .unwrap_or(false);
            if prev_contact {
                continue;
            }
            let mut ev = base_event(ctx, cfg, pos, session, minute, u);
            ev.dir = dir;
            ev.ref_price = la.line;
            ev.stop_level = stop_ref;
            ev.kinds = kind::TOUCH;
            if inside && poked {
                ev.kinds |= kind::CLOSEBACK;
            } else if inside && touched {
                // touched and closed inside without poking through
                ev.kinds |= kind::CLOSEBACK;
            }
            ev.kinds |= candle_flags(&tbars, ti, dir);
            ev.swing_ref = match dir {
                Direction::Long => last_swing(ctx, cfg.chart, cfg.swing_strength, SwingKind::Low, u),
                Direction::Short => last_swing(ctx, cfg.chart, cfg.swing_strength, SwingKind::High, u),
            };
            ev.next_swing = match dir {
                Direction::Long => last_swing(ctx, cfg.chart, cfg.swing_strength, SwingKind::High, u),
                Direction::Short => last_swing(ctx, cfg.chart, cfg.swing_strength, SwingKind::Low, u),
            };
            out.push(ev);
        }
        pu = u;
    }
    out
}

fn detect_breakout(ctx: &DetCtx, cfg: &DetectCfg) -> Vec<Event> {
    ctx.swings(cfg.chart, cfg.swing_strength);
    ctx.atr(cfg.chart);
    let sw = ctx.swings(cfg.chart, cfg.swing_strength);
    let atr = ctx.atr(cfg.chart);
    let sbars = &ctx.charts[&cfg.chart].bars;
    let mut lvls = Vec::new();
    for k in [LevelKind::Support, LevelKind::Resistance] {
        lvls.extend(levels(sbars, &sw, k, cfg.level_tol_atr, &atr, cfg.touches));
    }
    // for each level, scan trigger bars after born for the first close
    // beyond the box
    let tbars = ctx.charts[&cfg.trigger].bars.clone();
    let trig = each_trigger(ctx, cfg);
    let us: Vec<usize> = trig.iter().map(|t| t.4).collect();
    let mut out = Vec::new();
    for lv in &lvls {
        // first trigger index whose usable chart count exceeds born
        let start = us.partition_point(|&uu| uu <= lv.born);
        let mut broken = false;
        for &(ti, pos, session, minute, u) in &trig[start..] {
            if broken {
                continue;
            }
            let Some(la) = lv.as_of(u.saturating_sub(1)) else {
                continue;
            };
            let tb = &tbars[ti];
            let (dir, broke) = match lv.kind {
                LevelKind::Resistance => (Direction::Long, tb.close > la.hi),
                LevelKind::Support => (Direction::Short, tb.close < la.lo),
            };
            if !broke {
                continue;
            }
            broken = true;
            let mut ev = base_event(ctx, cfg, pos, session, minute, u);
            ev.dir = dir;
            ev.ref_price = la.line;
            ev.stop_level = match dir {
                Direction::Long => la.lo,
                Direction::Short => la.hi,
            };
            ev.kinds = kind::BREAK;
            ev.swing_ref = match dir {
                Direction::Long => last_swing(ctx, cfg.chart, cfg.swing_strength, SwingKind::Low, u),
                Direction::Short => last_swing(ctx, cfg.chart, cfg.swing_strength, SwingKind::High, u),
            };
            ev.next_swing = match dir {
                Direction::Long => last_swing(ctx, cfg.chart, cfg.swing_strength, SwingKind::High, u),
                Direction::Short => last_swing(ctx, cfg.chart, cfg.swing_strength, SwingKind::Low, u),
            };
            out.push(ev);
        }
    }
    out.sort_by_key(|e| e.pos);
    out
}

fn detect_flip(ctx: &DetCtx, cfg: &DetectCfg) -> Vec<Event> {
    ctx.swings(cfg.chart, cfg.swing_strength);
    ctx.atr(cfg.chart);
    let sw = ctx.swings(cfg.chart, cfg.swing_strength);
    let atr = ctx.atr(cfg.chart);
    let sbars = &ctx.charts[&cfg.chart].bars;
    let mut lvls = Vec::new();
    for k in [LevelKind::Support, LevelKind::Resistance] {
        lvls.extend(levels(sbars, &sw, k, cfg.level_tol_atr, &atr, cfg.touches));
    }
    let tbars = ctx.charts[&cfg.trigger].bars.clone();
    let trig = each_trigger(ctx, cfg);
    let us: Vec<usize> = trig.iter().map(|t| t.4).collect();
    let mut out = Vec::new();
    for lv in &lvls {
        // first break beyond, then first retest touch from the flip side
        let start = us.partition_point(|&uu| uu <= lv.born);
        let mut broke_at: Option<usize> = None;
        for &(ti, pos, session, minute, u) in &trig[start..] {
            let Some(la) = lv.as_of(u.saturating_sub(1)) else {
                continue;
            };
            let tb = &tbars[ti];
            if broke_at.is_none() {
                let broke = match lv.kind {
                    LevelKind::Resistance => tb.close > la.hi,
                    LevelKind::Support => tb.close < la.lo,
                };
                if broke {
                    broke_at = Some(ti);
                }
                continue;
            }
            // retest: bar's range contains the line
            let touched = tb.low <= la.line && la.line <= tb.high;
            if !touched {
                continue;
            }
            let dir = match lv.kind {
                LevelKind::Resistance => Direction::Long,  // was resistance, now support
                LevelKind::Support => Direction::Short,
            };
            let inside = match dir {
                Direction::Long => tb.close >= la.line,
                Direction::Short => tb.close <= la.line,
            };
            let mut ev = base_event(ctx, cfg, pos, session, minute, u);
            ev.dir = dir;
            ev.ref_price = la.line;
            ev.stop_level = match dir {
                Direction::Long => la.lo,
                Direction::Short => la.hi,
            };
            ev.kinds = kind::FLIP | if inside { kind::CLOSEBACK } else { 0 };
            ev.kinds |= candle_flags(&tbars, ti, dir);
            ev.swing_ref = match dir {
                Direction::Long => last_swing(ctx, cfg.chart, cfg.swing_strength, SwingKind::Low, u),
                Direction::Short => last_swing(ctx, cfg.chart, cfg.swing_strength, SwingKind::High, u),
            };
            ev.next_swing = match dir {
                Direction::Long => last_swing(ctx, cfg.chart, cfg.swing_strength, SwingKind::High, u),
                Direction::Short => last_swing(ctx, cfg.chart, cfg.swing_strength, SwingKind::Low, u),
            };
            out.push(ev);
            break; // one flip retest per level
        }
    }
    out.sort_by_key(|e| e.pos);
    out
}

fn detect_pullback(ctx: &DetCtx, cfg: &DetectCfg) -> Vec<Event> {
    ctx.swings(cfg.chart, cfg.swing_strength);
    ctx.atr(cfg.chart);
    let sw = ctx.swings(cfg.chart, cfg.swing_strength);
    let sidx = SwingIndex::new(&sw);
    let tbars = ctx.charts[&cfg.trigger].bars.clone();
    let mut out = Vec::new();
    for (ti, pos, session, minute, u) in each_trigger(ctx, cfg) {
        if u < 3 {
            continue;
        }
        let trend = trend_idx(&sidx, u - 1);
        let Some(dir) = dir_of_trend(trend) else { continue };
        // pullback touches the last confirmed opposite-side swing:
        // uptrend -> prior swing high as new support
        let (kind2, ref_swing) = match dir {
            Direction::Long => (SwingKind::High, sidx.last_of(SwingKind::High, u - 1, 1)),
            Direction::Short => (SwingKind::Low, sidx.last_of(SwingKind::Low, u - 1, 1)),
        };
        let Some(anchor) = ref_swing.first() else { continue };
        let _ = kind2;
        let tb = &tbars[ti];
        let touched = tb.low <= anchor.price && anchor.price <= tb.high;
        if !touched {
            continue;
        }
        // one event per contact episode
        if ti > 0 {
            let pb = &tbars[ti - 1];
            if pb.low <= anchor.price && anchor.price <= pb.high {
                continue;
            }
        }
        let inside = match dir {
            Direction::Long => tb.close >= anchor.price,
            Direction::Short => tb.close <= anchor.price,
        };
        let mut ev = base_event(ctx, cfg, pos, session, minute, u);
        ev.dir = dir;
        ev.ref_price = anchor.price;
        ev.stop_level = anchor.price;
        ev.kinds = kind::TOUCH | if inside { kind::CLOSEBACK } else { 0 };
        ev.kinds |= candle_flags(&tbars, ti, dir);
        ev.swing_ref = match dir {
            Direction::Long => sidx.last_of(SwingKind::Low, u - 1, 1).first().map(|s| s.price),
            Direction::Short => sidx.last_of(SwingKind::High, u - 1, 1).first().map(|s| s.price),
        }
        .unwrap_or(f64::NAN);
        ev.next_swing = match dir {
            Direction::Long => sidx.last_of(SwingKind::High, u - 1, 2)
                .first()
                .map(|s| s.price),
            Direction::Short => sidx.last_of(SwingKind::Low, u - 1, 2)
                .first()
                .map(|s| s.price),
        }
        .unwrap_or(f64::NAN);
        out.push(ev);
    }
    out
}

fn detect_trendline(ctx: &DetCtx, cfg: &DetectCfg) -> Vec<Event> {
    ctx.swings(cfg.chart, cfg.swing_strength);
    ctx.atr(cfg.chart);
    let sw = ctx.swings(cfg.chart, cfg.swing_strength);
    let sidx = SwingIndex::new(&sw);
    let tbars = ctx.charts[&cfg.trigger].bars.clone();
    let mut out = Vec::new();
    for (ti, pos, session, minute, u) in each_trigger(ctx, cfg) {
        let tb = &tbars[ti];
        for (k, dir) in [
            (SwingKind::Low, Direction::Long),
            (SwingKind::High, Direction::Short),
        ] {
            let Some(tl) = trend_line(&sidx, k, u.saturating_sub(1), cfg.trendline_anchors) else {
                continue;
            };
            let price = tl.price_at(u.saturating_sub(1));
            let touched = tb.low <= price && price <= tb.high;
            if !touched {
                continue;
            }
            // one event per contact episode
            if ti > 0 {
                let pb = &tbars[ti - 1];
                if pb.low <= price && price <= pb.high {
                    continue;
                }
            }
            let inside = match dir {
                Direction::Long => tb.close >= price,
                Direction::Short => tb.close <= price,
            };
            let mut ev = base_event(ctx, cfg, pos, session, minute, u);
            ev.dir = dir;
            ev.ref_price = price;
            ev.stop_level = price;
            ev.kinds = kind::TOUCH | if inside { kind::CLOSEBACK } else { 0 };
            ev.kinds |= candle_flags(&tbars, ti, dir);
            ev.swing_ref = match dir {
                Direction::Long => sidx.last_of(SwingKind::Low, u - 1, 1).first().map(|s| s.price),
                Direction::Short => sidx.last_of(SwingKind::High, u - 1, 1).first().map(|s| s.price),
            }
            .unwrap_or(f64::NAN);
            ev.next_swing = match dir {
                Direction::Long => sidx.last_of(SwingKind::High, u - 1, 1).first().map(|s| s.price),
                Direction::Short => sidx.last_of(SwingKind::Low, u - 1, 1).first().map(|s| s.price),
            }
            .unwrap_or(f64::NAN);
            out.push(ev);
        }
    }
    out
}

fn detect_structure(ctx: &DetCtx, cfg: &DetectCfg, want: BreakKind) -> Vec<Event> {
    ctx.swings(cfg.chart, cfg.swing_strength);
    ctx.atr(cfg.chart);
    let sw = ctx.swings(cfg.chart, cfg.swing_strength);
    let sidx = SwingIndex::new(&sw);
    let sbars = ctx.charts[&cfg.chart].bars.clone();
    let sup = ctx.charts[&cfg.chart].usable_pos.clone();
    let sses = ctx.charts[&cfg.chart].session_of.clone();
    let breaks = structure_breaks(&sbars, &sw);
    let mut out = Vec::new();
    for br in breaks {
        if br.kind != want {
            continue;
        }
        // event goes live when the setup bar closes
        let pos = sup[br.idx];
        let session = sses[br.idx];
        let minute = sbars[br.idx].usable_from.saturating_sub(1);
        let u = br.idx + 1;
        let dir = match br.dir {
            BreakDir::Up => Direction::Long,
            BreakDir::Down => Direction::Short,
        };
        let mut ev = base_event(ctx, cfg, pos, session, minute, u);
        ev.dir = dir;
        ev.ref_price = br.level;
        ev.stop_level = br.level;
        ev.kinds = kind::BREAK;
        ev.swing_ref = match dir {
            Direction::Long => sidx.last_of(SwingKind::Low, br.idx, 1).first().map(|s| s.price),
            Direction::Short => sidx.last_of(SwingKind::High, br.idx, 1).first().map(|s| s.price),
        }
        .unwrap_or(f64::NAN);
        ev.next_swing = match dir {
            Direction::Long => sidx.last_of(SwingKind::High, br.idx, 2).first().map(|s| s.price),
            Direction::Short => sidx.last_of(SwingKind::Low, br.idx, 2).first().map(|s| s.price),
        }
        .unwrap_or(f64::NAN);
        out.push(ev);
    }
    out
}

fn detect_sweep(ctx: &DetCtx, cfg: &DetectCfg) -> Vec<Event> {
    ctx.swings(cfg.chart, cfg.swing_strength);
    ctx.atr(cfg.chart);
    let sw = ctx.swings(cfg.chart, cfg.swing_strength);
    let atr = ctx.atr(cfg.chart);
    let sbars = &ctx.charts[&cfg.chart].bars;
    let mut lvls = Vec::new();
    for k in [LevelKind::Support, LevelKind::Resistance] {
        lvls.extend(levels(sbars, &sw, k, cfg.level_tol_atr, &atr, cfg.touches));
    }
    let tbars = ctx.charts[&cfg.trigger].bars.clone();
    // index the edge being swept: support -> lo, resistance -> hi.
    // Prefilter only: the as-of edge lives inside the final box, so
    // widen the query by the max box width to stay a superset.
    let index = MidIndex::new(lvls.len(), |i| match lvls[i].kind {
        LevelKind::Support => lvls[i].lo,
        LevelKind::Resistance => lvls[i].hi,
    });
    let pad = lvls
        .iter()
        .map(|lv| lv.hi - lv.lo)
        .fold(0.0f64, f64::max);
    let mut out = Vec::new();
    for (ti, pos, session, minute, u) in each_trigger(ctx, cfg) {
        let tb = &tbars[ti];
        let mut cands = index.range_gt(tb.low - pad, tb.high + pad);
        cands.sort_unstable();
        for li in cands {
            let lv = &lvls[li];
            let Some(la) = lv.as_of(u.saturating_sub(1)) else {
                continue;
            };
            // sweep: wick through the box then close back inside
            let (dir, swept) = match lv.kind {
                LevelKind::Support => {
                    (Direction::Long, tb.low < la.lo && tb.close > la.lo)
                }
                LevelKind::Resistance => {
                    (Direction::Short, tb.high > la.hi && tb.close < la.hi)
                }
            };
            if !swept {
                continue;
            }
            let mut ev = base_event(ctx, cfg, pos, session, minute, u);
            ev.dir = dir;
            ev.ref_price = la.line;
            ev.stop_level = match dir {
                Direction::Long => tb.low,   // stop under the sweep wick
                Direction::Short => tb.high,
            };
            ev.kinds = kind::SWEEP | kind::CLOSEBACK;
            ev.kinds |= candle_flags(&tbars, ti, dir);
            ev.swing_ref = match dir {
                Direction::Long => last_swing(ctx, cfg.chart, cfg.swing_strength, SwingKind::Low, u),
                Direction::Short => last_swing(ctx, cfg.chart, cfg.swing_strength, SwingKind::High, u),
            };
            ev.next_swing = match dir {
                Direction::Long => last_swing(ctx, cfg.chart, cfg.swing_strength, SwingKind::High, u),
                Direction::Short => last_swing(ctx, cfg.chart, cfg.swing_strength, SwingKind::Low, u),
            };
            out.push(ev);
        }
    }
    out
}

fn detect_zone(ctx: &DetCtx, cfg: &DetectCfg) -> Vec<Event> {
    ctx.atr(cfg.chart);
    let atr = ctx.atr(cfg.chart);
    let sbars = &ctx.charts[&cfg.chart];
    let zs = zones(&sbars.bars, &atr);
    ctx.swings(cfg.chart, cfg.swing_strength);
    let tbars = ctx.charts[&cfg.trigger].bars.clone();
    let pad = zs
        .iter()
        .map(|z| (z.hi - z.lo) / 2.0)
        .fold(0.0f64, f64::max);
    let index = MidIndex::new(zs.len(), |i| (zs[i].hi + zs[i].lo) / 2.0);
    let mut out = Vec::new();
    let mut entered: HashMap<usize, u16> = HashMap::new();
    for (ti, pos, session, minute, u) in each_trigger(ctx, cfg) {
        let tb = &tbars[ti];
        let mut cands = index.range(tb.low - pad, tb.high + pad);
        cands.sort_unstable();
        for zi in cands {
            let z = &zs[zi];
            if z.born > u.saturating_sub(1) {
                continue;
            }
            // zone run/size dials filter
            let run_len = z.run_end - z.run_start + 1;
            if run_len < cfg.zone_run.0 || run_len > cfg.zone_run.1 {
                continue;
            }
            if z.size_atr < cfg.zone_atr.0 || z.size_atr > cfg.zone_atr.1 {
                continue;
            }
            let touched = tb.low <= z.hi && z.lo <= tb.high;
            if !touched {
                continue;
            }
            // one event per contact episode
            if ti > 0 && z.born <= u.saturating_sub(2) {
                let pb = &tbars[ti - 1];
                if pb.low <= z.hi && z.lo <= pb.high {
                    continue;
                }
            }
            let count = entered.entry(zi).or_insert(0);
            *count += 1;
            let ordinal = *count;
            let dir = match z.kind {
                ZoneKind::Demand => Direction::Long,
                ZoneKind::Supply => Direction::Short,
            };
            let inside = match dir {
                Direction::Long => tb.close >= z.lo,
                Direction::Short => tb.close <= z.hi,
            };
            let mut ev = base_event(ctx, cfg, pos, session, minute, u);
            ev.dir = dir;
            ev.ref_price = match dir {
                Direction::Long => z.hi,
                Direction::Short => z.lo,
            };
            ev.stop_level = match dir {
                Direction::Long => z.lo,
                Direction::Short => z.hi,
            };
            ev.kinds = kind::TOUCH | if inside { kind::CLOSEBACK } else { 0 };
            ev.kinds |= candle_flags(&tbars, ti, dir);
            ev.meta = ordinal;
            ev.swing_ref = match dir {
                Direction::Long => last_swing(ctx, cfg.chart, cfg.swing_strength, SwingKind::Low, u),
                Direction::Short => last_swing(ctx, cfg.chart, cfg.swing_strength, SwingKind::High, u),
            };
            ev.next_swing = match dir {
                Direction::Long => last_swing(ctx, cfg.chart, cfg.swing_strength, SwingKind::High, u),
                Direction::Short => last_swing(ctx, cfg.chart, cfg.swing_strength, SwingKind::Low, u),
            };
            out.push(ev);
        }
    }
    out
}

fn detect_fvg(ctx: &DetCtx, cfg: &DetectCfg) -> Vec<Event> {
    ctx.atr(cfg.chart);
    let sbars = &ctx.charts[&cfg.chart];
    let gs = fvgs(&sbars.bars);
    ctx.swings(cfg.chart, cfg.swing_strength);
    let tbars = ctx.charts[&cfg.trigger].bars.clone();
    let pad = gs
        .iter()
        .map(|g| (g.hi - g.lo) / 2.0)
        .fold(0.0f64, f64::max);
    let index = MidIndex::new(gs.len(), |i| (gs[i].hi + gs[i].lo) / 2.0);
    let mut out = Vec::new();
    let mut entered: HashMap<usize, u16> = HashMap::new();
    for (ti, pos, session, minute, u) in each_trigger(ctx, cfg) {
        let tb = &tbars[ti];
        let mut cands = index.range(tb.low - pad, tb.high + pad);
        cands.sort_unstable();
        for gi in cands {
            let g = &gs[gi];
            if g.idx > u.saturating_sub(1) {
                continue;
            }
            let touched = tb.low <= g.hi && g.lo <= tb.high;
            if !touched {
                continue;
            }
            // one event per contact episode
            if ti > 0 && g.idx <= u.saturating_sub(2) {
                let pb = &tbars[ti - 1];
                if pb.low <= g.hi && g.lo <= pb.high {
                    continue;
                }
            }
            let count = entered.entry(gi).or_insert(0);
            *count += 1;
            let ordinal = *count;
            let dir = match g.kind {
                GapKind::Bull => Direction::Long,
                GapKind::Bear => Direction::Short,
            };
            let inside = match dir {
                Direction::Long => tb.close >= g.lo,
                Direction::Short => tb.close <= g.hi,
            };
            let mut ev = base_event(ctx, cfg, pos, session, minute, u);
            ev.dir = dir;
            ev.ref_price = match dir {
                Direction::Long => g.hi,
                Direction::Short => g.lo,
            };
            ev.stop_level = match dir {
                Direction::Long => g.lo,
                Direction::Short => g.hi,
            };
            ev.kinds = kind::TOUCH | if inside { kind::CLOSEBACK } else { 0 };
            ev.kinds |= candle_flags(&tbars, ti, dir);
            ev.meta = ordinal;
            ev.swing_ref = match dir {
                Direction::Long => last_swing(ctx, cfg.chart, cfg.swing_strength, SwingKind::Low, u),
                Direction::Short => last_swing(ctx, cfg.chart, cfg.swing_strength, SwingKind::High, u),
            };
            ev.next_swing = match dir {
                Direction::Long => last_swing(ctx, cfg.chart, cfg.swing_strength, SwingKind::High, u),
                Direction::Short => last_swing(ctx, cfg.chart, cfg.swing_strength, SwingKind::Low, u),
            };
            out.push(ev);
        }
    }
    out
}

/// Stand-in events for the "random" comparisons: `count` events sampled
/// from trigger bars matching direction and time-of-day, with swing
/// refs filled. Used by the matched-random controls.
pub fn random_events(
    _ctx: &DetCtx,
    _cfg: &DetectCfg,
    _count: usize,
) -> Vec<Event> {
    Vec::new() // filled in the behavior-test module with seeded draws
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pal::detctx::DetCtx;
    use crate::pal::synth::synth_sessions;
    use crate::pal::types::ChartId;

    fn same_event(a: &Event, b: &Event) -> bool {
        a.pos == b.pos
            && a.dir == b.dir
            && a.ref_price.to_bits() == b.ref_price.to_bits()
            && a.stop_level.to_bits() == b.stop_level.to_bits()
            && a.swing_ref.to_bits() == b.swing_ref.to_bits()
            && a.next_swing.to_bits() == b.next_swing.to_bits()
            && a.atr.to_bits() == b.atr.to_bits()
            && a.kinds == b.kinds
            && a.session == b.session
            && a.minute == b.minute
            && a.trend == b.trend
            && a.meta == b.meta
    }

    /// No-look-ahead proof: events decided inside the truncated window
    /// must be identical to the full-history run. Any leak of swings or
    /// level touches confirmed after the cut changes the event stream.
    #[test]
    fn detectors_are_causal_under_truncation() {
        let sessions = synth_sessions(
            chrono::NaiveDate::from_ymd_opt(2022, 1, 3).unwrap(),
            60,
            "setups-causality",
            4.0,
        );
        let cut = 40usize; // drop the last 20 sessions
        let full = DetCtx::build(&sessions, &Default::default());
        let trunc = DetCtx::build(&sessions[..cut], &Default::default());
        let cut_bars = trunc.bars.len();

        for tol in [0.25, 3.0] {
            let cfg = DetectCfg {
                chart: ChartId::M5,
                trigger: ChartId::M5,
                swing_strength: 2,
                level_tol_atr: tol,
                touches: 1,
                zone_run: (2, 5),
                zone_atr: (1.5, 3.0),
                trendline_anchors: 3,
            };

            // sanity: this dial must create multi-touch clusters —
            // otherwise the as-of/final distinction is never exercised
            let sw = trunc.swings(cfg.chart, cfg.swing_strength);
            let atr = trunc.atr(cfg.chart);
            let sbars = &trunc.charts[&cfg.chart].bars;
            let multi = [LevelKind::Support, LevelKind::Resistance]
                .iter()
                .flat_map(|k| levels(sbars, &sw, *k, tol, &atr, 1))
                .filter(|l| l.touches.len() > 1)
                .count();
            if tol > 1.0 {
                assert!(multi > 0, "tol {tol}: no multi-touch levels");
                // the leak channel must actually be open: some level's
                // final geometry differs from what was known at born
                let sw_full = full.swings(cfg.chart, cfg.swing_strength);
                let atr_full = full.atr(cfg.chart);
                let sb_full = &full.charts[&cfg.chart].bars;
                let revised = [LevelKind::Support, LevelKind::Resistance]
                    .iter()
                    .flat_map(|k| levels(sb_full, &sw_full, *k, tol, &atr_full, 1))
                    .any(|l| {
                        let g = l.as_of(l.born).unwrap();
                        g.line != l.line || g.lo != l.lo || g.hi != l.hi
                    });
                assert!(revised, "tol {tol}: no level revises after born");
            }

            for setup in [
                Setup::SweepReclaim,
                Setup::LevelBounce,
                Setup::Breakout,
                Setup::FlipZone,
            ] {
                let fev = detect_events(&full, setup, &cfg);
                let tev = detect_events(&trunc, setup, &cfg);
                let fp: Vec<&Event> = fev.iter().filter(|e| e.pos < cut_bars).collect();
                let tp: Vec<&Event> = tev.iter().filter(|e| e.pos < cut_bars).collect();
                assert!(!tp.is_empty(), "{setup:?}: no events — vacuous test");
                assert_eq!(
                    fp.len(),
                    tp.len(),
                    "{setup:?} tol {tol}: event count changed under truncation"
                );
                for (a, b) in fp.iter().zip(tp.iter()) {
                    assert!(
                        same_event(a, b),
                        "{setup:?} tol {tol}: event at pos {} changed under truncation",
                        a.pos
                    );
                }
            }
        }
    }

    #[test]
    fn accepts_maps_entries() {
        use crate::pal::rulebook::EntryKind;
        assert!(accepts(kind::TOUCH, EntryKind::LimitAtLevel));
        assert!(!accepts(kind::TOUCH, EntryKind::AtClose));
        assert!(accepts(kind::BREAK, EntryKind::AtClose));
        assert!(accepts(kind::BREAK, EntryKind::LimitBack));
        assert!(accepts(kind::REJ, EntryKind::Rejection));
        assert!(accepts(kind::SWEEP, EntryKind::CloseBack));
    }
}
