//! The funnel (spec prompt 3): generate the version sets (common grid,
//! spread, human-sample, six named), run every version on NQ and ES
//! discovery over cached detection, apply the five stages, judge each
//! setup as a whole, and save + hash everything.
//!
//! Detection is shared per detector-dial tuple (DetKey): thousands of
//! versions collapse to hundreds of detection passes.
//!
//! Stages (thresholds fixed before any run):
//!   1. >= 30 NQ trades
//!   2. NQ profitable after costs
//!   3. NQ daily Sharpe >= 0.5
//!   4. beats matched-random twins: < 5% of 10,000 twin sets >= it
//!   5. ES: profitable and Sharpe >= 0.3
//! Candidate = stages 1-4 + positive pnl in both eras + stage 5.

use std::collections::{BTreeMap, HashMap, HashSet};
use std::fmt::Write as _;
use std::io::Write as IoWrite;
use std::path::Path;

use chrono::{Datelike, NaiveDate};
use serde::Serialize;

use crate::pal::detctx::DetCtx;
use crate::pal::engine::{run_order, CostModel, Entry, OrderSpec, Target};
use crate::pal::rng::Rng;
use crate::pal::runner::{parse_entry, run_version, RunResult, TwinInfo};
use crate::pal::rulebook::{
    DirSel, EntryKind, Filters, LevelShape, Setup, StopKind, TargetKind, Version, LEVEL_TOLS,
    SWING_STRENGTHS, TIME_EXIT, TOUCHES, TRENDLINE_ANCHORS, ZONE_ATR, ZONE_RUNS,
};
use crate::pal::setups::{detect_events, DetectCfg, Event};
use crate::pal::types::{ChartId, Market};
use crate::pal::windows::{DISCOVERY_END, DISCOVERY_START};

pub const SET_COMMON: u8 = 1;
pub const SET_SPREAD: u8 = 2;
pub const SET_HUMAN: u8 = 4;
pub const SET_NAMED: u8 = 8;

const MIN_NQ_TRADES: usize = 30;
const MIN_NQ_SIGNALS_SPREAD: usize = 40; // "at least 40 signals" keep-rule
const SHARPE_NQ: f64 = 0.5;
const SHARPE_ES: f64 = 0.3;
const SHARPE_SETUP: f64 = 0.5;
const TWIN_POOL: usize = 1000; // twin draws per trade (outcome pool)
const TWIN_SETS: usize = 10_000;
const TWIN_P: f64 = 0.05;
const SPREAD_TARGET_PER_SETUP: usize = 6_000; // oversample; keep-qualified cap 4,000
const SPREAD_KEEP_PER_SETUP: usize = 4_000;
const HUMAN_PER_SETUP: usize = 2_000;

#[derive(Clone, Debug)]
pub struct FunnelVersion {
    pub v: Version,
    pub sets: u8,
    pub sha: String,
}

/// Detector-dial tuple: versions sharing it share one detection pass.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct DetKey {
    setup: u8,
    chart: u8,
    trigger: u8,
    swing: u16,
    tol_x100: i64,
    touches: u16,
    zone_run: u16,
    zone_atr_x100: i64,
    tl_anchors: u16,
}

fn setup_idx(s: Setup) -> u8 {
    Setup::ALL.iter().position(|&x| x == s).unwrap_or(0) as u8
}

fn detkey(v: &Version) -> DetKey {
    let setup = crate::pal::runner::parse_setup(&v.setup);
    DetKey {
        setup: setup_idx(setup),
        chart: ChartId::parse(&v.chart).unwrap_or(ChartId::M1) as u8,
        trigger: ChartId::parse(&v.trigger).unwrap_or(ChartId::M1) as u8,
        swing: v.swing_strength as u16,
        tol_x100: (v.level_tol_atr * 100.0).round() as i64,
        touches: v.touches as u16,
        zone_run: ((v.zone_run_min as u16) << 8) | v.zone_run_max as u16,
        zone_atr_x100: (v.zone_atr_min * 100.0).round() as i64 * 1000
            + (v.zone_atr_max * 100.0).round() as i64,
        tl_anchors: v.trendline_anchors as u16,
    }
}

fn cfg_of(k: &DetKey) -> DetectCfg {
    DetectCfg {
        chart: ChartId::ALL[k.chart as usize],
        trigger: ChartId::ALL[k.trigger as usize],
        swing_strength: k.swing as usize,
        level_tol_atr: k.tol_x100 as f64 / 100.0,
        touches: k.touches as usize,
        zone_run: ((k.zone_run >> 8) as usize, (k.zone_run & 0xff) as usize),
        zone_atr: (
            (k.zone_atr_x100 / 1000) as f64 / 100.0,
            (k.zone_atr_x100 % 1000) as f64 / 100.0,
        ),
        trendline_anchors: k.tl_anchors as usize,
    }
}

fn mk_version(
    name: String,
    setup: Setup,
    chart: ChartId,
    trigger: ChartId,
    direction: DirSel,
    entry: EntryKind,
    stop_kind: StopKind,
    target_kind: TargetKind,
    filters: Filters,
    swing_strength: usize,
    level_tol_atr: f64,
    touches: usize,
    level_shape: LevelShape,
    zone_run: (usize, usize),
    zone_atr: (f64, f64),
    trendline_anchors: usize,
    size: f64,
) -> Version {
    Version {
        name,
        setup: setup.as_str().to_string(),
        chart: chart.as_str().to_string(),
        trigger: trigger.as_str().to_string(),
        direction,
        swing_strength,
        level_tol_atr,
        touches,
        level_shape,
        zone_run_min: zone_run.0,
        zone_run_max: zone_run.1,
        zone_atr_min: zone_atr.0,
        zone_atr_max: zone_atr.1,
        trendline_anchors,
        entry: entry.as_str().to_string(),
        stop_kind,
        target_kind,
        time_exit_bars: TIME_EXIT,
        filters,
        size,
    }
}

/// A random valid version for `setup` (all dials uniform over valid values).
fn draw_random(rng: &mut Rng, setup: Setup, name: String) -> Version {
    let chart = ChartId::ALL[rng.gen_range(0, 6)];
    let trigs = crate::pal::rulebook::valid_triggers(chart);
    let trigger = trigs[rng.gen_range(0, trigs.len())];
    let direction = match rng.gen_range(0, 3) {
        0 => DirSel::Both,
        1 => DirSel::Long,
        _ => DirSel::Short,
    };
    let entries = setup.entries();
    let entry = entries[rng.gen_range(0, entries.len())];
    let stop_kind = StopKind::ALL[rng.gen_range(0, 2)];
    let target_kind = TargetKind::ALL[rng.gen_range(0, 4)];
    let filters = Filters::COMMON[rng.gen_range(0, 4)];
    mk_version(
        name,
        setup,
        chart,
        trigger,
        direction,
        entry,
        stop_kind,
        target_kind,
        filters,
        SWING_STRENGTHS[rng.gen_range(0, 4)],
        LEVEL_TOLS[rng.gen_range(0, 3)],
        TOUCHES[rng.gen_range(0, 4)],
        if rng.next_f64() < 0.5 {
            LevelShape::Line
        } else {
            LevelShape::Box
        },
        ZONE_RUNS[rng.gen_range(0, 4)],
        ZONE_ATR[rng.gen_range(0, 3)],
        TRENDLINE_ANCHORS[rng.gen_range(0, 2)],
        if rng.next_f64() < 0.75 { 1.0 } else { 0.5 },
    )
}

/// Every (dial_id, value) of a version for share counting.
fn dial_vals(v: &Version) -> Vec<(u8, u64)> {
    let mut out = vec![
        (0, ChartId::parse(&v.chart).unwrap_or(ChartId::M1) as u64),
        (1, ChartId::parse(&v.trigger).unwrap_or(ChartId::M1) as u64),
        (
            2,
            match v.direction {
                DirSel::Both => 0,
                DirSel::Long => 1,
                DirSel::Short => 2,
            },
        ),
        (3, v.swing_strength as u64),
        (4, (v.level_tol_atr * 100.0).round() as u64),
        (5, v.touches as u64),
        (
            6,
            match v.level_shape {
                LevelShape::Line => 0,
                LevelShape::Box => 1,
            },
        ),
        (
            7,
            match parse_entry(&v.entry) {
                EntryKind::LimitAtLevel => 0,
                EntryKind::CloseBack => 1,
                EntryKind::Rejection => 2,
                EntryKind::Engulfing => 3,
                EntryKind::AtClose => 4,
                EntryKind::LimitBack => 5,
            },
        ),
        (
            8,
            match v.stop_kind {
                StopKind::BeyondLevel => 0,
                StopKind::BeyondSwing => 1,
            },
        ),
        (
            9,
            match v.target_kind {
                TargetKind::R1 => 0,
                TargetKind::R2 => 1,
                TargetKind::R3 => 2,
                TargetKind::NextSwing => 3,
            },
        ),
        (
            10,
            (v.filters.trend_agree as u64) << 2
                | (v.filters.ny_only as u64) << 1
                | v.filters.skip_news as u64,
        ),
        (11, (v.size * 2.0).round() as u64),
        (12, ((v.zone_run_min as u64) << 8) | v.zone_run_max as u64),
        (
            13,
            (v.zone_atr_min * 100.0).round() as u64 * 1000
                + (v.zone_atr_max * 100.0).round() as u64,
        ),
        (14, v.trendline_anchors as u64),
    ];
    out.shrink_to_fit();
    out
}

/// The common set: usual settings crossed with the execution grid.
/// (swing 3, both directions, trigger = setup chart [daily -> 1h],
/// every usual entry x 2 stops x 4 targets x {no filter or exactly 1})
pub fn common_set() -> Vec<FunnelVersion> {
    let mut out = Vec::new();
    for setup in Setup::ALL {
        for chart in ChartId::ALL {
            let trigger = if chart == ChartId::D1 {
                ChartId::H1
            } else {
                chart
            };
            for &entry in setup.entries() {
                for stop_kind in StopKind::ALL {
                    for target_kind in TargetKind::ALL {
                        for filters in Filters::COMMON {
                            let name = format!(
                                "c:{}:{}:{}:{}:{}:{}:{}",
                                setup.as_str(),
                                chart.as_str(),
                                entry.as_str(),
                                stop_kind.as_str(),
                                target_kind.as_str(),
                                filters.count(),
                                out.len()
                            );
                            out.push(mk_version(
                                name,
                                setup,
                                chart,
                                trigger,
                                DirSel::Both,
                                entry,
                                stop_kind,
                                target_kind,
                                filters,
                                3,
                                0.25,
                                2,
                                LevelShape::Line,
                                (2, 5),
                                (1.5, 3.0),
                                2,
                                1.0,
                            ));
                        }
                    }
                }
            }
        }
    }
    out.into_iter()
        .map(|v| FunnelVersion {
            sha: v.sha(),
            v,
            sets: SET_COMMON,
        })
        .collect()
}

/// Six named systems built as usually taught; lab defaults elsewhere.
pub fn named_versions() -> Vec<FunnelVersion> {
    use EntryKind::*;
    use StopKind::*;
    use TargetKind::*;
    let f = |t: bool, n: bool, s: bool| Filters {
        trend_agree: t,
        ny_only: n,
        skip_news: s,
    };
    let defs = vec![
        // The Bare Minimum: one level, one limit, one fixed target.
        ("bare_minimum", Setup::LevelBounce, ChartId::H1, ChartId::M15,
         LimitAtLevel, BeyondLevel, R2, f(false, false, false), 3, 0.25, 2, (2, 5), (1.5, 3.0), 2),
        // The Daily Version: trade the daily chart only.
        ("daily_version", Setup::LevelBounce, ChartId::D1, ChartId::H1,
         LimitAtLevel, BeyondLevel, NextSwing, f(false, false, false), 3, 0.25, 2, (2, 5), (1.5, 3.0), 2),
        // The Checklist: trend filter + confirmed pullback.
        ("checklist", Setup::TrendPullback, ChartId::H4, ChartId::H1,
         CloseBack, BeyondSwing, R2, f(true, false, false), 5, 0.25, 2, (2, 5), (1.5, 3.0), 2),
        // The Sweep System: liquidity grab then reclaim, skip news.
        ("sweep_system", Setup::SweepReclaim, ChartId::M15, ChartId::M5,
         LimitAtLevel, BeyondLevel, R2, f(false, false, true), 3, 0.25, 2, (2, 5), (1.5, 3.0), 2),
        // The Zone System: fresh supply/demand zones, NY hours.
        ("zone_system", Setup::SupplyDemandZone, ChartId::H1, ChartId::M15,
         LimitAtLevel, BeyondLevel, R3, f(false, true, false), 3, 0.25, 2, (2, 5), (1.5, 3.0), 2),
        // The Entry Model: FVG retracement entries, NY hours.
        ("entry_model", Setup::FairValueGap, ChartId::M15, ChartId::M5,
         LimitAtLevel, BeyondLevel, R2, f(false, true, false), 3, 0.25, 2, (2, 5), (1.5, 3.0), 2),
    ];
    defs.into_iter()
        .map(|(name, setup, chart, trig, e, st, tg, fl, sw, tol, tc, zr, za, tl)| {
            let v = mk_version(
                name.to_string(),
                setup,
                chart,
                trig,
                DirSel::Both,
                e,
                st,
                tg,
                fl,
                sw,
                tol,
                tc,
                LevelShape::Line,
                zr,
                za,
                tl,
                1.0,
            );
            FunnelVersion {
                sha: v.sha(),
                v,
                sets: SET_NAMED,
            }
        })
        .collect()
}

/// The spread set: greedy dial-share fill, `target` versions per setup.
pub fn spread_set(target: usize) -> Vec<FunnelVersion> {
    let mut out = Vec::new();
    for setup in Setup::ALL {
        let mut rng = Rng::fork(&format!("funnel/spread/{}", setup.as_str()));
        let mut counts: HashMap<(u8, u64), u64> = HashMap::new();
        let mut seen: HashSet<String> = HashSet::new();
        let mut i = 0usize;
        while i < target {
            let mut best: Option<(Version, u64)> = None;
            for _ in 0..400 {
                let cand = draw_random(&mut rng, setup, String::new());
                let mut score = 0u64;
                for key in dial_vals(&cand) {
                    let c = counts.get(&key).copied().unwrap_or(0) + 1;
                    score += c * c;
                }
                if best.as_ref().map(|(_, s)| score < *s).unwrap_or(true) {
                    best = Some((cand, score));
                }
            }
            let mut v = best.unwrap().0;
            if !seen.insert(v.to_json()) {
                continue; // duplicate: rescore next batch
            }
            v.name = format!("s:{}:{i}", setup.as_str());
            for key in dial_vals(&v) {
                *counts.entry(key).or_insert(0) += 1;
            }
            out.push(FunnelVersion {
                sha: v.sha(),
                v,
                sets: SET_SPREAD,
            });
            i += 1;
        }
    }
    out
}

/// The human set: 40% no filter / 40% one / 20% two, other dials random.
pub fn human_set(per_setup: usize) -> Vec<FunnelVersion> {
    let mut out = Vec::new();
    for setup in Setup::ALL {
        let mut rng = Rng::fork(&format!("funnel/human/{}", setup.as_str()));
        let mut seen: HashSet<String> = HashSet::new();
        let mut i = 0usize;
        while i < per_setup {
            let mut v = draw_random(&mut rng, setup, String::new());
            // filter mix override
            let r = rng.next_f64();
            let n_f = if r < 0.4 {
                0
            } else if r < 0.8 {
                1
            } else {
                2
            };
            let mut pick = [false; 3];
            let mut picked = 0usize;
            while picked < n_f {
                let k = rng.gen_range(0, 3);
                if !pick[k] {
                    pick[k] = true;
                    picked += 1;
                }
            }
            v.filters = Filters {
                trend_agree: pick[0],
                ny_only: pick[1],
                skip_news: pick[2],
            };
            if !seen.insert(v.to_json()) {
                continue;
            }
            v.name = format!("h:{}:{i}", setup.as_str());
            out.push(FunnelVersion {
                sha: v.sha(),
                v,
                sets: SET_HUMAN,
            });
            i += 1;
        }
    }
    out
}

/// All sets deduped by version JSON (sets bitmask merged).
pub fn all_versions() -> Vec<FunnelVersion> {
    let mut map: HashMap<String, FunnelVersion> = HashMap::new();
    let mut order = Vec::new();
    for fv in common_set()
        .into_iter()
        .chain(spread_set(SPREAD_TARGET_PER_SETUP))
        .chain(human_set(HUMAN_PER_SETUP))
        .chain(named_versions())
    {
        match map.get_mut(&fv.sha) {
            Some(e) => e.sets |= fv.sets,
            None => {
                order.push(fv.sha.clone());
                map.insert(fv.sha.clone(), fv);
            }
        }
    }
    order.into_iter().map(|s| map.remove(&s).unwrap()).collect()
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct Metrics {
    pub n_trades: usize,
    pub n_signals: usize,
    pub pnl: f64,
    pub sharpe: f64,
    pub win_rate: f64,
    pub profit_factor: f64,
    pub era1_pnl: f64,
    pub era2_pnl: f64,
    pub pnl_per_year: f64,
}

/// Metrics from a run: daily-P&L Sharpe over the full session grid.
pub fn metrics(rr: &RunResult, session_dates: &[NaiveDate]) -> Metrics {
    let mut sum = 0.0;
    let mut sumsq = 0.0;
    let mut era1 = 0.0;
    let mut era2 = 0.0;
    let era2_lo = NaiveDate::from_ymd_opt(2023, 1, 1).unwrap();
    for d in session_dates {
        let p = rr.daily_pnl.get(d).copied().unwrap_or(0.0);
        sum += p;
        sumsq += p * p;
        if d.year() == 2022 {
            era1 += p;
        } else if *d >= era2_lo {
            era2 += p;
        }
    }
    let n_days = session_dates.len().max(1) as f64;
    let mean = sum / n_days;
    let var = (sumsq / n_days) - mean * mean;
    let sharpe = if var > 0.0 {
        mean / var.sqrt() * (252.0f64).sqrt()
    } else {
        0.0
    };
    let wins = rr.trades.iter().filter(|t| t.pnl_usd > 0.0).count();
    let gross_win: f64 = rr.trades.iter().filter(|t| t.pnl_usd > 0.0).map(|t| t.pnl_usd).sum();
    let gross_loss: f64 = rr
        .trades
        .iter()
        .filter(|t| t.pnl_usd < 0.0)
        .map(|t| -t.pnl_usd)
        .sum();
    let years = n_days / 252.0;
    Metrics {
        n_trades: rr.trades.len(),
        n_signals: rr.n_signals,
        pnl: sum,
        sharpe,
        win_rate: if rr.trades.is_empty() {
            0.0
        } else {
            wins as f64 / rr.trades.len() as f64
        },
        profit_factor: if gross_loss > 0.0 {
            gross_win / gross_loss
        } else {
            f64::INFINITY
        },
        era1_pnl: era1,
        era2_pnl: era2,
        pnl_per_year: if years > 0.0 { sum / years } else { 0.0 },
    }
}

/// Stage 4: matched-random twins. Each real trade gets a pool of twins
/// with the same direction, a random entry minute inside the same
/// 30-minute block of day, the same stop/target distances expressed in
/// ATR (1m ATR at the twin's entry), the same holding minutes and the
/// same costs. Returns the share of the 10,000 twin sets that made at
/// least as much as the version.
pub fn matched_random_share(
    ctx: &mut DetCtx,
    market: Market,
    trades: &[crate::pal::engine::Trade],
    twins: &[TwinInfo],
    sha: &str,
) -> f64 {
    ctx.atr(ChartId::M1);
    let atr = ctx.atr[&ChartId::M1].clone();
    let costs = CostModel::for_market(market);

    // eligible entry positions per 30-minute bucket
    let mut buckets: HashMap<usize, Vec<usize>> = HashMap::new();
    for pos in 1..ctx.bars.len().saturating_sub(1) {
        if ctx.ctx[pos].pre_roll || ctx.ctx[pos].last_before_roll {
            continue;
        }
        buckets.entry(ctx.minute_of[pos] / 30).or_default().push(pos);
    }

    // outcome pool per trade
    let mut pools: Vec<Vec<f64>> = Vec::with_capacity(trades.len());
    for tw in twins {
        let mut rng = Rng::fork(&format!("funnel/twins/{sha}/{}", pools.len()));
        let bucket = ctx.minute_of.get(tw.entry_bar).copied().unwrap_or(0) / 30;
        let hold = ctx
            .ctx
            .get(tw.exit_bar)
            .map(|e| e.minute_global)
            .unwrap_or(0)
            .saturating_sub(ctx.ctx.get(tw.entry_bar).map(|e| e.minute_global).unwrap_or(0))
            as u32;
        let Some(eligible) = buckets.get(&bucket) else {
            pools.push(vec![0.0]);
            continue;
        };
        let mut pool = Vec::with_capacity(TWIN_POOL);
        let mut tries = 0usize;
        while pool.len() < TWIN_POOL && tries < TWIN_POOL * 4 {
            tries += 1;
            let pos = eligible[rng.gen_range(0, eligible.len())];
            let Some(a) = atr.get(pos.saturating_sub(1)).copied().flatten() else {
                continue;
            };
            if !a.is_finite() || a <= 0.0 || !tw.stop_atr.is_finite() {
                continue;
            }
            let px = ctx.bars[pos].close;
            let spec = OrderSpec {
                dir: tw.dir,
                entry: Entry::Market,
                stop: px - tw.dir.sign() * tw.stop_atr * a,
                target: if tw.target_r.is_finite() {
                    Target::RMult(tw.target_r)
                } else {
                    Target::None
                },
                time_exit_minutes: hold,
                cancel_after_minutes: 0,
                size: tw.size,
            };
            let t = run_order(&ctx.bars, &ctx.ctx, pos, &spec, &costs);
            pool.push(t.pnl_usd);
        }
        if pool.is_empty() {
            pool.push(0.0);
        }
        pools.push(pool);
    }

    let real_total: f64 = trades.iter().map(|t| t.pnl_usd).sum();
    let mut rng = Rng::fork(&format!("funnel/twin-sets/{sha}"));
    let mut ge = 0usize;
    for _ in 0..TWIN_SETS {
        let mut tot = 0.0;
        for pool in &pools {
            tot += pool[rng.gen_range(0, pool.len())];
        }
        if tot >= real_total {
            ge += 1;
        }
    }
    ge as f64 / TWIN_SETS as f64
}

#[derive(Clone, Debug, Serialize)]
pub struct VersionRow {
    pub name: String,
    pub sha: String,
    pub sets: u8,
    pub version: String,
    pub detkey: String,
    pub nq: Option<Metrics>,
    pub es: Option<Metrics>,
    pub s1_trades: bool,
    pub s2_profit: bool,
    pub s3_sharpe: bool,
    pub s4_twin_share: Option<f64>,
    pub s4_pass: bool,
    pub era1_pos: bool,
    pub era2_pos: bool,
    pub s5_es_hold: bool,
    /// spec keep-rules: common/named always kept; spread+human need
    /// >=40 NQ discovery signals (spread capped at 4,000 per setup).
    pub kept: bool,
    pub candidate: bool,
}

#[derive(Clone, Debug, Serialize)]
pub struct SetupResult {
    pub setup: String,
    pub chart: String,
    pub market: String,
    pub n_versions: usize,
    pub pnl: f64,
    pub pnl_per_year: f64,
    pub sharpe: f64,
    pub era1_pnl: f64,
    pub era2_pnl: f64,
    pub works: bool,
}

pub struct FunnelOut {
    pub rows: Vec<VersionRow>,
    /// (version index, market, daily pnl) — version rows carry the name
    pub daily: Vec<(usize, Market, BTreeMap<NaiveDate, f64>)>,
    pub setups: Vec<SetupResult>,
    pub candidates: Vec<VersionRow>,
    pub n_detect_keys: usize,
}

/// Run the whole funnel over both markets' discovery contexts.
/// `limit` caps the version count (smoke runs).
pub fn run_funnel(
    nq: &mut DetCtx,
    es: &mut DetCtx,
    news: &HashSet<NaiveDate>,
    limit: Option<usize>,
) -> FunnelOut {
    let versions = all_versions();
    let versions: Vec<FunnelVersion> = match limit {
        Some(n) => versions.into_iter().take(n).collect(),
        None => versions,
    };
    println!("funnel: {} versions after dedupe", versions.len());

    // group by DetKey
    let mut groups: BTreeMap<DetKey, Vec<usize>> = BTreeMap::new();
    for (i, fv) in versions.iter().enumerate() {
        groups.entry(detkey(&fv.v)).or_default().push(i);
    }
    let n_keys = groups.len();
    println!("funnel: {n_keys} distinct detector-dial tuples");

    let mut rows: Vec<VersionRow> = versions
        .iter()
        .map(|fv| VersionRow {
            name: fv.v.name.clone(),
            sha: fv.sha.clone(),
            sets: fv.sets,
            version: fv.v.to_json(),
            detkey: format!("{:?}", detkey(&fv.v)),
            nq: None,
            es: None,
            s1_trades: false,
            s2_profit: false,
            s3_sharpe: false,
            s4_twin_share: None,
            s4_pass: false,
            era1_pos: false,
            era2_pos: false,
            s5_es_hold: false,
            kept: false,
            candidate: false,
        })
        .collect();
    let mut daily: Vec<(usize, Market, BTreeMap<NaiveDate, f64>)> = Vec::new();

    for (gi, (key, idxs)) in groups.iter().enumerate() {
        if gi % 50 == 0 {
            println!("funnel: key {gi}/{n_keys}");
        }
        let setup = Setup::ALL[key.setup as usize];
        let cfg = cfg_of(key);
        for market in [Market::Nq, Market::Es] {
            let ctx = if market == Market::Nq { &mut *nq } else { &mut *es };
            let events: Vec<Event> = detect_events(ctx, setup, &cfg);
            for &i in idxs {
                let fv = &versions[i];
                let rr = run_version(ctx, market, &events, &fv.v, news);
                let m = metrics(&rr, &ctx.session_dates);
                if market == Market::Nq {
                    rows[i].s1_trades = m.n_trades >= MIN_NQ_TRADES;
                    rows[i].s2_profit = m.pnl > 0.0;
                    rows[i].s3_sharpe = m.sharpe >= SHARPE_NQ;
                    rows[i].era1_pos = m.era1_pnl > 0.0;
                    rows[i].era2_pos = m.era2_pnl > 0.0;
                    if rows[i].s1_trades && rows[i].s2_profit && rows[i].s3_sharpe {
                        let share = matched_random_share(ctx, market, &rr.trades, &rr.twins, &fv.sha);
                        rows[i].s4_twin_share = Some(share);
                        rows[i].s4_pass = share < TWIN_P;
                    }
                    rows[i].nq = Some(m);
                } else {
                    rows[i].s5_es_hold = m.pnl > 0.0 && m.sharpe >= SHARPE_ES;
                    rows[i].es = Some(m);
                }
                daily.push((i, market, rr.daily_pnl));
            }
        }
    }

    // keep-rules: common/named always kept; spread+human need >=40 NQ
    // signals; spread kept in generation order capped at 4,000/setup.
    let mut spread_n: HashMap<String, usize> = HashMap::new();
    for r in &mut rows {
        let v: serde_json::Value =
            serde_json::from_str(&r.version).unwrap_or_default();
        let setup = v["setup"].as_str().unwrap_or("").to_string();
        let sampled = r.sets & (SET_SPREAD | SET_HUMAN) != 0;
        let sigs = r.nq.as_ref().map(|m| m.n_signals).unwrap_or(0);
        let e = spread_n.entry(setup).or_insert(0);
        r.kept = if sampled {
            let ok = sigs >= MIN_NQ_SIGNALS_SPREAD
                && (r.sets & SET_SPREAD == 0 || *e < SPREAD_KEEP_PER_SETUP);
            if ok {
                *e += 1;
            }
            // also kept when it belongs to a non-sampled set too
            ok || r.sets & (SET_COMMON | SET_NAMED) != 0
        } else {
            true
        };
    }

    // setup-level equal-weight judgment over kept versions
    // (normalize each version to size 1)
    let mut setup_days: HashMap<(String, String, Market), Vec<(f64, &BTreeMap<NaiveDate, f64>)>> =
        HashMap::new();
    for (i, market, dm) in &daily {
        if !rows[*i].kept {
            continue;
        }
        let fv = &versions[*i];
        setup_days
            .entry((fv.v.setup.clone(), fv.v.chart.clone(), *market))
            .or_default()
            .push((fv.v.size, dm));
    }
    let mut setups = Vec::new();
    for ((setup, chart, market), dailies) in &setup_days {
        let dates = if *market == Market::Nq {
            &nq.session_dates
        } else {
            &es.session_dates
        };
        let mut avg: Vec<f64> = Vec::with_capacity(dates.len());
        for d in dates {
            let mut s = 0.0;
            for (size, dm) in dailies {
                let p = dm.get(d).copied().unwrap_or(0.0);
                s += if *size > 0.0 { p / *size } else { p };
            }
            avg.push(s / dailies.len() as f64);
        }
        let n_days = avg.len().max(1) as f64;
        let sum: f64 = avg.iter().sum();
        let mean = sum / n_days;
        let var: f64 = avg.iter().map(|x| x * x).sum::<f64>() / n_days - mean * mean;
        let sharpe = if var > 0.0 {
            mean / var.sqrt() * (252.0f64).sqrt()
        } else {
            0.0
        };
        let era2_lo = NaiveDate::from_ymd_opt(2023, 1, 1).unwrap();
        let mut era1 = 0.0;
        let mut era2 = 0.0;
        for (d, p) in dates.iter().zip(avg.iter()) {
            if d.year() == 2022 {
                era1 += p;
            } else if *d >= era2_lo {
                era2 += p;
            }
        }
        let years = n_days / 252.0;
        let ppy = if years > 0.0 { sum / years } else { 0.0 };
        let works = ppy > 0.0 && sharpe >= SHARPE_SETUP && era1 > 0.0 && era2 > 0.0;
        setups.push(SetupResult {
            setup: setup.clone(),
            chart: chart.clone(),
            market: market.as_str().to_string(),
            n_versions: dailies.len(),
            pnl: sum,
            pnl_per_year: ppy,
            sharpe,
            era1_pnl: era1,
            era2_pnl: era2,
            works,
        });
    }

    for r in &mut rows {
        r.candidate = r.kept
            && r.s1_trades
            && r.s2_profit
            && r.s3_sharpe
            && r.s4_pass
            && r.era1_pos
            && r.era2_pos
            && r.s5_es_hold;
    }
    let candidates: Vec<VersionRow> = rows.iter().filter(|r| r.candidate).cloned().collect();
    FunnelOut {
        rows,
        daily,
        setups,
        candidates,
        n_detect_keys: n_keys,
    }
}

fn sha256_file(p: &Path) -> String {
    use sha2::{Digest, Sha256};
    let b = std::fs::read(p).unwrap_or_default();
    format!("{:x}", Sha256::digest(&b))
}

/// Write every funnel artifact under `results/pal/funnel/`.
pub fn write_outputs(out: &FunnelOut, dir: &Path) -> Result<(), String> {
    std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;

    // versions.jsonl — full config + metrics + stages per version
    let mut f = std::io::BufWriter::new(
        std::fs::File::create(dir.join("versions.jsonl")).map_err(|e| e.to_string())?,
    );
    for r in &out.rows {
        let line = serde_json::to_string(r).map_err(|e| e.to_string())?;
        writeln!(f, "{line}").map_err(|e| e.to_string())?;
    }
    drop(f);

    // daily_pnl.jsonl — one line per version per market
    let mut f = std::io::BufWriter::new(
        std::fs::File::create(dir.join("daily_pnl.jsonl")).map_err(|e| e.to_string())?,
    );
    for (i, mkt, dm) in &out.daily {
        let obj = serde_json::json!({
            "name": out.rows[*i].name,
            "market": mkt.as_str(),
            "pnl": dm.iter().map(|(d, p)| (d.to_string(), p)).collect::<BTreeMap<_, _>>(),
        });
        writeln!(f, "{}", serde_json::to_string(&obj).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
    }
    drop(f);

    // stage shares by setup, setup+chart, dial value, filter
    let mut csv = String::from("group,kind,n,s1,s2,s3,s4,s5,candidates\n");
    let mut groups: HashMap<(String, String), Vec<&VersionRow>> = HashMap::new();
    for r in &out.rows {
        if !r.kept {
            continue;
        }
        let v: serde_json::Value =
            serde_json::from_str(&r.version).map_err(|e| e.to_string())?;
        let setup = v["setup"].as_str().unwrap_or("").to_string();
        let chart = v["chart"].as_str().unwrap_or("").to_string();
        groups
            .entry(("setup".into(), setup.clone()))
            .or_default()
            .push(r);
        groups
            .entry(("setup_chart".into(), format!("{setup}/{chart}")))
            .or_default()
            .push(r);
        for (dial, key) in [
            ("swing", v["swing_strength"].to_string()),
            ("tol_atr", v["level_tol_atr"].to_string()),
            ("touches", v["touches"].to_string()),
            ("entry", v["entry"].as_str().unwrap_or("").to_string()),
            ("stop", v["stop_kind"].as_str().unwrap_or("").to_string()),
            ("target", v["target_kind"].as_str().unwrap_or("").to_string()),
            ("direction", v["direction"].as_str().unwrap_or("").to_string()),
            ("trigger", v["trigger"].as_str().unwrap_or("").to_string()),
        ] {
            groups
                .entry(("dial".into(), format!("{dial}={key}")))
                .or_default()
                .push(r);
        }
        let fl = v["filters"].to_string();
        groups
            .entry(("filter".into(), fl))
            .or_default()
            .push(r);
    }
    let mut keys: Vec<_> = groups.keys().collect();
    keys.sort();
    for k in keys {
        let rows = &groups[k];
        let n = rows.len() as f64;
        let sh = |f: &dyn Fn(&VersionRow) -> bool| {
            rows.iter().filter(|r| f(r)).count() as f64 / n
        };
        writeln!(
            csv,
            "{},{},{},{:.4},{:.4},{:.4},{:.4},{:.4},{}",
            k.0,
            k.1,
            rows.len(),
            sh(&|r| r.s1_trades),
            sh(&|r| r.s1_trades && r.s2_profit),
            sh(&|r| r.s1_trades && r.s2_profit && r.s3_sharpe),
            sh(&|r| r.s4_pass),
            sh(&|r| r.s5_es_hold),
            rows.iter().filter(|r| r.candidate).count()
        )
        .map_err(|e| e.to_string())?;
    }
    std::fs::write(dir.join("stage_shares.csv"), csv).map_err(|e| e.to_string())?;

    // per-setup results
    let mut csv = String::from(
        "setup,chart,market,n_versions,pnl,pnl_per_year,sharpe,era1_pnl,era2_pnl,works\n",
    );
    for s in &out.setups {
        writeln!(
            csv,
            "{},{},{},{},{:.2},{:.2},{:.3},{:.2},{:.2},{}",
            s.setup, s.chart, s.market, s.n_versions, s.pnl, s.pnl_per_year, s.sharpe,
            s.era1_pnl, s.era2_pnl, s.works
        )
        .map_err(|e| e.to_string())?;
    }
    std::fs::write(dir.join("setup_results.csv"), csv).map_err(|e| e.to_string())?;

    // win rate vs profit
    let mut csv = String::from("name,setup,chart,market,win_rate,pnl\n");
    for r in &out.rows {
        let v: serde_json::Value =
            serde_json::from_str(&r.version).map_err(|e| e.to_string())?;
        for (mkt, m) in [("nq", &r.nq), ("es", &r.es)] {
            if let Some(m) = m {
                writeln!(
                    csv,
                    "{},{},{},{},{:.4},{:.2}",
                    r.name,
                    v["setup"].as_str().unwrap_or(""),
                    v["chart"].as_str().unwrap_or(""),
                    mkt,
                    m.win_rate,
                    m.pnl
                )
                .map_err(|e| e.to_string())?;
            }
        }
    }
    std::fs::write(dir.join("winrate_profit.csv"), csv).map_err(|e| e.to_string())?;

    // rule effects: group versions identical except one dial; compare
    // mean NQ pnl across that dial's values ("everything else equal")
    let mut csv = String::from("dial,n_groups,value,mean_pnl,n\n");
    let dial_keys = [
        "swing_strength",
        "level_tol_atr",
        "touches",
        "entry",
        "stop_kind",
        "target_kind",
        "direction",
        "trigger",
        "chart",
        "level_shape",
        "size",
    ];
    for dial in dial_keys {
        let mut by_rest: HashMap<String, Vec<(&VersionRow, String)>> = HashMap::new();
        for r in &out.rows {
            if r.nq.is_none() {
                continue;
            }
            let v: serde_json::Value =
                serde_json::from_str(&r.version).map_err(|e| e.to_string())?;
            let mut rest = v.clone();
            rest.as_object_mut().unwrap().remove(dial);
            let key = serde_json::to_string(&rest).map_err(|e| e.to_string())?;
            by_rest
                .entry(key)
                .or_default()
                .push((r, v[dial].to_string()));
        }
        // aggregate mean per value over groups with >1 distinct value
        let mut agg: HashMap<String, (f64, usize)> = HashMap::new();
        let mut n_groups = 0usize;
        for (_k, members) in &by_rest {
            let distinct: HashSet<&String> = members.iter().map(|(_, v)| v).collect();
            if distinct.len() < 2 {
                continue;
            }
            n_groups += 1;
            for (r, val) in members {
                let p = r.nq.as_ref().map(|m| m.pnl).unwrap_or(0.0);
                let e = agg.entry(val.clone()).or_default();
                e.0 += p;
                e.1 += 1;
            }
        }
        let mut vals: Vec<_> = agg.iter().collect();
        vals.sort_by(|a, b| a.0.cmp(b.0));
        for (val, (sum, n)) in vals {
            writeln!(csv, "{},{},{},{:.2},{}", dial, n_groups, val, sum / *n as f64, n)
                .map_err(|e| e.to_string())?;
        }
    }
    std::fs::write(dir.join("rule_effects.csv"), csv).map_err(|e| e.to_string())?;

    // candidates + working setups + named, each saved and hashed
    let cand: Vec<serde_json::Value> = out
        .candidates
        .iter()
        .map(|r| {
            serde_json::json!({
                "name": r.name, "sha": r.sha,
                "version": serde_json::from_str::<serde_json::Value>(&r.version).unwrap_or_default(),
                "nq": r.nq, "es": r.es, "sets": r.sets,
            })
        })
        .collect();
    std::fs::write(
        dir.join("candidates.json"),
        serde_json::to_string_pretty(&cand).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    std::fs::write(
        dir.join("candidates.sha256"),
        sha256_file(&dir.join("candidates.json")),
    )
    .map_err(|e| e.to_string())?;

    let working: Vec<serde_json::Value> = out
        .setups
        .iter()
        .filter(|s| s.works)
        .map(|s| serde_json::to_value(s).unwrap_or_default())
        .collect();
    std::fs::write(
        dir.join("working_setups.json"),
        serde_json::to_string_pretty(&working).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    std::fs::write(
        dir.join("working_setups.sha256"),
        sha256_file(&dir.join("working_setups.json")),
    )
    .map_err(|e| e.to_string())?;

    let named: Vec<serde_json::Value> = out
        .rows
        .iter()
        .filter(|r| r.sets & SET_NAMED != 0)
        .map(|r| {
            serde_json::json!({
                "name": r.name, "sha": r.sha,
                "version": serde_json::from_str::<serde_json::Value>(&r.version).unwrap_or_default(),
                "nq": r.nq, "es": r.es, "candidate": r.candidate,
            })
        })
        .collect();
    std::fs::write(
        dir.join("named_versions.json"),
        serde_json::to_string_pretty(&named).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    std::fs::write(
        dir.join("named_versions.sha256"),
        sha256_file(&dir.join("named_versions.json")),
    )
    .map_err(|e| e.to_string())?;

    // manifest over every artifact
    let mut man = String::new();
    for p in [
        "versions.jsonl",
        "daily_pnl.jsonl",
        "stage_shares.csv",
        "setup_results.csv",
        "winrate_profit.csv",
        "rule_effects.csv",
        "candidates.json",
        "working_setups.json",
        "named_versions.json",
    ] {
        writeln!(man, "{}  {}", sha256_file(&dir.join(p)), p).map_err(|e| e.to_string())?;
    }
    std::fs::write(dir.join("manifest.sha256"), man).map_err(|e| e.to_string())?;
    Ok(())
}

/// The version->DetCtx wiring shared by funnel runs and the final test:
/// load sessions, build roll boundaries, build the context.
pub fn build_ctx(
    sessions: &[crate::pal::types::Session],
    rolls: &[crate::pal::audit::Roll],
) -> DetCtx {
    let rb = crate::pal::enginecheck::roll_boundaries(sessions, rolls);
    DetCtx::build(sessions, &rb)
}

pub fn discovery_range() -> (NaiveDate, NaiveDate) {
    (DISCOVERY_START, DISCOVERY_END)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn common_grid_is_valid() {
        let set = common_set();
        assert!(!set.is_empty());
        for fv in &set {
            let chart = ChartId::parse(&fv.v.chart).unwrap();
            let trigger = ChartId::parse(&fv.v.trigger).unwrap();
            assert!(crate::pal::rulebook::valid_triggers(chart).contains(&trigger));
            assert_eq!(fv.v.swing_strength, 3);
            let setup = crate::pal::runner::parse_setup(&fv.v.setup);
            assert!(setup.entries().contains(&parse_entry(&fv.v.entry)));
        }
    }

    #[test]
    fn named_are_six_and_unique() {
        let n = named_versions();
        assert_eq!(n.len(), 6);
        let shas: HashSet<_> = n.iter().map(|f| f.sha.clone()).collect();
        assert_eq!(shas.len(), 6);
    }

    #[test]
    fn spread_fills_and_dedupes() {
        let s = spread_set(50);
        assert_eq!(s.len(), 500); // 10 setups x 50
        // determinism: same call -> same shas
        let s2 = spread_set(50);
        assert_eq!(
            s.iter().map(|f| &f.sha).collect::<Vec<_>>(),
            s2.iter().map(|f| &f.sha).collect::<Vec<_>>()
        );
    }

    #[test]
    fn human_filter_mix() {
        let h = human_set(200);
        assert_eq!(h.len(), 2000);
        let none = h.iter().filter(|f| f.v.filters.count() == 0).count();
        let two = h.iter().filter(|f| f.v.filters.count() == 2).count();
        // rough 40/40/20 tolerance on 200 draws per setup
        assert!(none > 500 && none < 1100, "none={none}");
        assert!(two > 200 && two < 700, "two={two}");
    }

    #[test]
    fn detkey_ignores_execution_dials() {
        let mut a = common_set();
        let mut b = a[0].v.clone();
        b.target_kind = TargetKind::R3;
        b.stop_kind = StopKind::BeyondSwing;
        assert_eq!(detkey(&a[0].v), detkey(&b));
        a.clear();
    }
}
