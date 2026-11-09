//! The runner: a Version + detected events -> trades + daily P&L.
//! Filters events by direction, entry kind, and the version's filters;
//! then runs each surviving event through the engine, one position at
//! a time. Cheap per version: detection is already done.

use std::collections::{BTreeMap, HashSet};

use chrono::NaiveDate;

use crate::pal::detctx::DetCtx;
use crate::pal::detect::trend::Trend;
use crate::pal::engine::{run_order, CostModel, Direction, Entry, ExitReason, OrderSpec, Target, Trade};
use crate::pal::rulebook::{
    DirSel, EntryKind, Setup, StopKind, TargetKind, Version, STOP_BUFFER_ATR, TIME_EXIT,
};
use crate::pal::setups::{accepts, Event};
use crate::pal::types::{ChartId, Market};

/// New York RTH window in session-minute indices: 09:30-16:00 ET.
const NY_OPEN_MIN: usize = 9 * 60 + 30 + 360; // 930
const NY_CLOSE_MIN: usize = 16 * 60 + 360; // 1320

pub struct RunResult {
    pub trades: Vec<Trade>,
    /// session_date -> pnl_usd for that session
    pub daily_pnl: BTreeMap<NaiveDate, f64>,
    /// events that passed all filters (signals attempted)
    pub n_signals: usize,
    /// events filtered out (for diagnostics)
    pub n_filtered: usize,
    /// per-filled-trade info for matched-random twins (funnel stage 4)
    pub twins: Vec<TwinInfo>,
}

/// What a matched-random twin needs to reproduce a trade's risk shape
/// at a random same-30-min entry (spec prompt 3, stage 4).
#[derive(Clone, Copy, Debug)]
pub struct TwinInfo {
    pub dir: Direction,
    /// real entry bar (for the minute-of-day bucket and holding time)
    pub entry_bar: usize,
    pub exit_bar: usize,
    /// |entry_px - stop| / setup-chart ATR at signal
    pub stop_atr: f64,
    /// target distance / stop distance (NaN = no target)
    pub target_r: f64,
    pub size: f64,
}

fn trigger_minutes(chart: ChartId) -> u32 {
    match chart {
        ChartId::M1 => 1,
        ChartId::M5 => 5,
        ChartId::M15 => 15,
        ChartId::H1 => 60,
        ChartId::H4 => 240,
        ChartId::D1 => 1380,
    }
}

fn dir_allowed(sel: DirSel, dir: Direction) -> bool {
    match sel {
        DirSel::Both => true,
        DirSel::Long => dir == Direction::Long,
        DirSel::Short => dir == Direction::Short,
    }
}

fn entry_of(kind: EntryKind, ev: &Event) -> Option<Entry> {
    Some(match kind {
        EntryKind::LimitAtLevel | EntryKind::LimitBack => Entry::Limit(ev.ref_price),
        EntryKind::CloseBack | EntryKind::Rejection | EntryKind::Engulfing | EntryKind::AtClose => {
            Entry::Market
        }
    })
}

/// Run one version over pre-detected events.
pub fn run_version(
    ctx: &DetCtx,
    market: Market,
    events: &[Event],
    v: &Version,
    news: &HashSet<NaiveDate>,
) -> RunResult {
    let entry_kind = parse_entry(&v.entry);
    let setup = parse_setup(&v.setup);
    let trigger = ChartId::parse(&v.trigger).unwrap_or(ChartId::M1);
    let trig_min = trigger_minutes(trigger);
    let time_exit_min = TIME_EXIT * trig_min;
    let cancel_min = TIME_EXIT * trig_min; // limit window = time exit
    let costs = CostModel::for_market(market);

    let mut trades = Vec::new();
    let mut twins = Vec::new();
    let mut daily = BTreeMap::new();
    let mut n_signals = 0usize;
    let mut n_filtered = 0usize;
    let mut busy_until = 0usize;

    for ev in events {
        if ev.pos <= busy_until {
            continue;
        }
        if !dir_allowed(v.direction, ev.dir) {
            continue;
        }
        if !accepts(ev.kinds, entry_kind) {
            continue;
        }
        if !setup_owns_event(setup, ev) {
            continue;
        }
        // filters
        if v.filters.trend_agree {
            let ok = match ev.dir {
                Direction::Long => ev.trend == Trend::Up,
                Direction::Short => ev.trend == Trend::Down,
            };
            if !ok {
                n_filtered += 1;
                continue;
            }
        }
        if v.filters.ny_only && !(NY_OPEN_MIN..NY_CLOSE_MIN).contains(&ev.minute) {
            n_filtered += 1;
            continue;
        }
        if v.filters.skip_news
            && ctx
                .session_dates
                .get(ev.session as usize)
                .map(|d| news.contains(d))
                .unwrap_or(false)
        {
            n_filtered += 1;
            continue;
        }
        let Some(entry) = entry_of(entry_kind, ev) else {
            continue;
        };
        let stop = match v.stop_kind {
            StopKind::BeyondLevel => ev.stop_level,
            StopKind::BeyondSwing => {
                if ev.swing_ref.is_finite() {
                    ev.swing_ref
                } else {
                    ev.stop_level
                }
            }
        };
        if !stop.is_finite() || !ev.ref_price.is_finite() {
            n_filtered += 1;
            continue;
        }
        let stop = stop - ev.dir.sign() * STOP_BUFFER_ATR * ev.atr.max(0.0);
        // degenerate: stop on the wrong side of the entry reference
        let bad = match ev.dir {
            Direction::Long => stop >= ev.ref_price,
            Direction::Short => stop <= ev.ref_price,
        };
        if bad {
            n_filtered += 1;
            continue;
        }
        let target = match v.target_kind {
            TargetKind::R1 => Target::RMult(1.0),
            TargetKind::R2 => Target::RMult(2.0),
            TargetKind::R3 => Target::RMult(3.0),
            TargetKind::NextSwing => {
                if ev.next_swing.is_finite() {
                    Target::Fixed(ev.next_swing)
                } else {
                    Target::None
                }
            }
        };
        n_signals += 1;
        let spec = OrderSpec {
            dir: ev.dir,
            entry,
            stop,
            target,
            time_exit_minutes: time_exit_min,
            cancel_after_minutes: cancel_min,
            size: v.size,
        };
        if ev.pos >= ctx.bars.len() {
            continue;
        }
        let t = run_order(&ctx.bars, &ctx.ctx, ev.pos, &spec, &costs);
        if t.reason == ExitReason::Cancelled {
            continue;
        }
        busy_until = t.exit_bar;
        let day = ctx
            .session_dates
            .get(t.exit_session as usize)
            .copied()
            .or_else(|| ctx.session_dates.last().copied());
        if let Some(d) = day {
            *daily.entry(d).or_insert(0.0) += t.pnl_usd;
        }
        // matched-random twin descriptor: risk shape in ATR/R terms
        let stop_dist = (t.entry_px - spec.stop).abs();
        let stop_atr = if ev.atr.is_finite() && ev.atr > 0.0 {
            stop_dist / ev.atr
        } else {
            f64::NAN
        };
        let target_r = match spec.target {
            Target::RMult(r) => r,
            Target::Fixed(p) => {
                if stop_dist > 0.0 {
                    (p - t.entry_px).abs() / stop_dist
                } else {
                    f64::NAN
                }
            }
            Target::None => f64::NAN,
        };
        twins.push(TwinInfo {
            dir: t.dir,
            entry_bar: t.entry_bar,
            exit_bar: t.exit_bar,
            stop_atr,
            target_r,
            size: v.size,
        });
        trades.push(t);
    }
    RunResult {
        trades,
        daily_pnl: daily,
        n_signals,
        n_filtered,
        twins,
    }
}

/// Events are shared per (setup, detector-dials) — not per version — so
/// the runner doesn't need setup gating. Kept as a hook for setups
/// whose events need setup-specific validity.
fn setup_owns_event(_setup: Setup, _ev: &Event) -> bool {
    true
}

pub fn parse_setup(s: &str) -> Setup {
    match s {
        "level_bounce" => Setup::LevelBounce,
        "breakout" => Setup::Breakout,
        "flip_zone" => Setup::FlipZone,
        "trend_pullback" => Setup::TrendPullback,
        "trend_line" => Setup::TrendLine,
        "bos_continuation" => Setup::BosContinuation,
        "choch_reversal" => Setup::ChochReversal,
        "sweep_reclaim" => Setup::SweepReclaim,
        "supply_demand_zone" => Setup::SupplyDemandZone,
        "fair_value_gap" => Setup::FairValueGap,
        _ => Setup::LevelBounce,
    }
}

pub fn parse_entry(s: &str) -> EntryKind {
    match s {
        "limit_at_level" => EntryKind::LimitAtLevel,
        "close_back" => EntryKind::CloseBack,
        "rejection" => EntryKind::Rejection,
        "engulfing" => EntryKind::Engulfing,
        "at_close" => EntryKind::AtClose,
        "limit_back" => EntryKind::LimitBack,
        _ => EntryKind::LimitAtLevel,
    }
}
