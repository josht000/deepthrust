//! The dial space: every price-action rule turned into dials with fixed
//! values (spec prompt 1). `rulebook_spec.json` is the machine-readable
//! space; `rulebook.md` is the human render. Version = one fully
//! specified strategy configuration; saved and hashed before any run.

use serde::Serialize;
use std::fmt::Write as _;
use std::path::Path;

use crate::pal::rng::MASTER_SEED;
use crate::pal::types::ChartId;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, serde::Deserialize)]
pub enum Setup {
    LevelBounce,
    Breakout,
    FlipZone,
    TrendPullback,
    TrendLine,
    BosContinuation,
    ChochReversal,
    SweepReclaim,
    SupplyDemandZone,
    FairValueGap,
}

impl Setup {
    pub const ALL: [Setup; 10] = [
        Setup::LevelBounce,
        Setup::Breakout,
        Setup::FlipZone,
        Setup::TrendPullback,
        Setup::TrendLine,
        Setup::BosContinuation,
        Setup::ChochReversal,
        Setup::SweepReclaim,
        Setup::SupplyDemandZone,
        Setup::FairValueGap,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Setup::LevelBounce => "level_bounce",
            Setup::Breakout => "breakout",
            Setup::FlipZone => "flip_zone",
            Setup::TrendPullback => "trend_pullback",
            Setup::TrendLine => "trend_line",
            Setup::BosContinuation => "bos_continuation",
            Setup::ChochReversal => "choch_reversal",
            Setup::SweepReclaim => "sweep_reclaim",
            Setup::SupplyDemandZone => "supply_demand_zone",
            Setup::FairValueGap => "fair_value_gap",
        }
    }

    /// Entries valid for this setup (spec prompt 3): level-type setups
    /// take a limit at the level, a close back, or a rejection/engulfing
    /// candle; break setups take the close or a limit back.
    pub fn entries(self) -> &'static [EntryKind] {
        use EntryKind::*;
        match self {
            Setup::LevelBounce
            | Setup::FlipZone
            | Setup::TrendPullback
            | Setup::TrendLine
            | Setup::SweepReclaim
            | Setup::SupplyDemandZone
            | Setup::FairValueGap => {
                &[LimitAtLevel, CloseBack, Rejection, Engulfing]
            }
            Setup::Breakout | Setup::BosContinuation | Setup::ChochReversal => {
                &[AtClose, LimitBack]
            }
        }
    }

    /// Whether this setup needs a level structure (levels dials apply).
    pub fn uses_levels(self) -> bool {
        matches!(
            self,
            Setup::LevelBounce | Setup::FlipZone | Setup::SweepReclaim
        )
    }
    /// Whether the trend filter/structure dials are intrinsic.
    pub fn uses_trend(self) -> bool {
        matches!(
            self,
            Setup::TrendPullback | Setup::TrendLine | Setup::BosContinuation
        )
    }
    /// Whether zone dials apply.
    pub fn uses_zones(self) -> bool {
        matches!(self, Setup::SupplyDemandZone)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, serde::Deserialize)]
pub enum EntryKind {
    /// limit at the level (0-slippage touch fill)
    LimitAtLevel,
    /// enter when the trigger bar closes back through the level
    CloseBack,
    /// enter on a rejection candle at the level
    Rejection,
    /// enter on an engulfing candle at the level
    Engulfing,
    /// enter at the close of the break bar (market at next open)
    AtClose,
    /// limit back at the broken level after the break
    LimitBack,
}

impl EntryKind {
    pub fn as_str(self) -> &'static str {
        match self {
            EntryKind::LimitAtLevel => "limit_at_level",
            EntryKind::CloseBack => "close_back",
            EntryKind::Rejection => "rejection",
            EntryKind::Engulfing => "engulfing",
            EntryKind::AtClose => "at_close",
            EntryKind::LimitBack => "limit_back",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, serde::Deserialize)]
pub enum StopKind {
    /// hard stop 0.1 ATR beyond the setup's reference level
    BeyondLevel,
    /// hard stop 0.1 ATR beyond the last confirmed swing
    BeyondSwing,
}

impl StopKind {
    pub fn as_str(self) -> &'static str {
        match self {
            StopKind::BeyondLevel => "beyond_level",
            StopKind::BeyondSwing => "beyond_swing",
        }
    }
    pub const ALL: [StopKind; 2] = [StopKind::BeyondLevel, StopKind::BeyondSwing];
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, serde::Deserialize)]
pub enum TargetKind {
    R1,
    R2,
    R3,
    NextSwing,
}

impl TargetKind {
    pub fn as_str(self) -> &'static str {
        match self {
            TargetKind::R1 => "1R",
            TargetKind::R2 => "2R",
            TargetKind::R3 => "3R",
            TargetKind::NextSwing => "next_swing",
        }
    }
    pub const ALL: [TargetKind; 4] = [
        TargetKind::R1,
        TargetKind::R2,
        TargetKind::R3,
        TargetKind::NextSwing,
    ];
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, serde::Deserialize)]
pub enum DirSel {
    Both,
    Long,
    Short,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, serde::Deserialize)]
pub enum LevelShape {
    Line,
    Box,
}

/// Optional filters (spec: no filter or exactly one for the common set;
/// the human set allows up to two).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, serde::Deserialize)]
pub struct Filters {
    pub trend_agree: bool,
    pub ny_only: bool,
    pub skip_news: bool,
}

impl Filters {
    pub fn count(self) -> usize {
        self.trend_agree as usize + self.ny_only as usize + self.skip_news as usize
    }
    /// the common set's filter space: none, or exactly one
    pub const COMMON: [Filters; 4] = [
        Filters { trend_agree: false, ny_only: false, skip_news: false },
        Filters { trend_agree: true, ny_only: false, skip_news: false },
        Filters { trend_agree: false, ny_only: true, skip_news: false },
        Filters { trend_agree: false, ny_only: false, skip_news: true },
    ];
}

/// One fully specified strategy version (a point in the dial space).
#[derive(Clone, Debug, Serialize, serde::Deserialize, PartialEq)]
pub struct Version {
    pub name: String,
    pub setup: String,
    pub chart: String,
    pub trigger: String,
    pub direction: DirSel,
    pub swing_strength: usize,
    pub level_tol_atr: f64,
    pub touches: usize,
    pub level_shape: LevelShape,
    pub zone_run_min: usize,
    pub zone_run_max: usize,
    pub zone_atr_min: f64,
    pub zone_atr_max: f64,
    pub trendline_anchors: usize,
    pub entry: String,
    pub stop_kind: StopKind,
    pub target_kind: TargetKind,
    pub time_exit_bars: u32,
    pub filters: Filters,
    pub size: f64,
}

impl Version {
    /// Canonical JSON used for hashing (spec: save every version's
    /// configuration before running, and hash the file).
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
    pub fn sha(&self) -> String {
        use sha2::{Digest, Sha256};
        format!("{:x}", Sha256::digest(self.to_json().as_bytes()))
    }
}

/// Dial value lists.
pub const SWING_STRENGTHS: [usize; 4] = [2, 3, 5, 8];
pub const LEVEL_TOLS: [f64; 3] = [0.1, 0.25, 0.5];
pub const TOUCHES: [usize; 4] = [1, 2, 3, 4];
pub const ZONE_RUNS: [(usize, usize); 4] = [(2, 2), (3, 3), (4, 5), (2, 5)];
pub const ZONE_ATR: [(f64, f64); 3] = [(1.5, 3.0), (1.5, 2.0), (2.0, 3.0)];
pub const TRENDLINE_ANCHORS: [usize; 2] = [2, 3];
pub const TIME_EXIT: u32 = 96;
pub const STOP_BUFFER_ATR: f64 = 0.1;

/// Valid trigger charts for a setup chart (spec: same or one/two steps
/// lower; daily setups trigger on 1h).
pub fn valid_triggers(setup_chart: ChartId) -> Vec<ChartId> {
    if setup_chart == ChartId::D1 {
        return vec![ChartId::H1];
    }
    [0usize, 1, 2]
        .iter()
        .filter_map(|s| setup_chart.lower(*s))
        .collect()
}

/// Emit rulebook_spec.json + rulebook.md.
pub fn write_rulebook(dir: &Path) -> Result<(), String> {
    std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    let spec = serde_json::json!({
        "lab": "price_action_lab",
        "master_seed": MASTER_SEED,
        "setups": Setup::ALL.iter().map(|s| s.as_str()).collect::<Vec<_>>(),
        "charts": ChartId::ALL.iter().map(|c| c.as_str()).collect::<Vec<_>>(),
        "trigger_rule": "same chart or 1-2 steps lower; daily setups trigger on 1h",
        "dials": {
            "swing_strength": SWING_STRENGTHS,
            "level_tol_atr": LEVEL_TOLS,
            "touches": TOUCHES,
            "level_shape": ["line", "box"],
            "zone_run_candles": ZONE_RUNS,
            "zone_atr": ZONE_ATR,
            "trendline_anchors": TRENDLINE_ANCHORS,
            "entry": {
                "level_setups": ["limit_at_level", "close_back", "rejection", "engulfing"],
                "break_setups": ["at_close", "limit_back"]
            },
            "stop": ["beyond_level", "beyond_swing"],
            "stop_buffer_atr": STOP_BUFFER_ATR,
            "target": ["1R", "2R", "3R", "next_swing"],
            "time_exit_bars": TIME_EXIT,
            "filters": ["none", "trend_agree", "ny_only", "skip_news"],
            "filters_max": 2,
            "direction": ["both", "long", "short"],
            "size": [1.0, 0.5]
        },
        "costs": {
            "commission_per_side_usd": 2.00,
            "limit_slippage_ticks": 0,
            "market_stop_slippage_ticks": 1,
            "tick_size": {"ES": 0.25, "NQ": 0.25},
            "tick_value_usd": {"ES": 12.50, "NQ": 5.00}
        },
        "windows": {
            "warmup": "2021-10-01..2021-12-31",
            "discovery": "2022-01-03..2024-07-31",
            "era1": "2022",
            "era2": "2023-01-01..2024-07-31",
            "sealed": "2024-08-01..end"
        }
    });
    std::fs::write(
        dir.join("rulebook_spec.json"),
        serde_json::to_string_pretty(&spec).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;

    let mut md = String::from("# rulebook.md — Price Action Lab dials\n\n");
    writeln!(md, "Master seed: `{MASTER_SEED}`. Every version is a point in this space.\n").unwrap();
    writeln!(md, "## Setups\n").unwrap();
    for s in Setup::ALL {
        writeln!(md, "- `{}` — entries: {}", s.as_str(), s.entries().iter().map(|e| e.as_str()).collect::<Vec<_>>().join(", ")).unwrap();
    }
    writeln!(md, "\n## Charts\n").unwrap();
    writeln!(md, "Setup charts: 1m, 5m, 15m, 1h, 4h, daily. Trigger chart: same or 1-2 steps lower (daily -> 1h).\n").unwrap();
    writeln!(md, "## Detector dials\n").unwrap();
    writeln!(md, "- swing strength: {SWING_STRENGTHS:?}").unwrap();
    writeln!(md, "- level tolerance (ATR): {LEVEL_TOLS:?}; touches: {TOUCHES:?}; shape: line | box").unwrap();
    writeln!(md, "- trend: last two confirmed swing highs + lows").unwrap();
    writeln!(md, "- BOS/CHoCH: first close past last confirmed swing, with/against trend").unwrap();
    writeln!(md, "- zones: runs of 2-5 candles covering 1.5-3 ATR; FVG: 3-candle gap").unwrap();
    writeln!(md, "- trend lines: last {TRENDLINE_ANCHORS:?} anchors").unwrap();
    writeln!(md, "\n## Execution dials\n").unwrap();
    writeln!(md, "- stop: beyond level | beyond last swing, +{STOP_BUFFER_ATR} ATR buffer").unwrap();
    writeln!(md, "- target: 1R | 2R | 3R | next swing").unwrap();
    writeln!(md, "- time exit: {TIME_EXIT} trigger bars").unwrap();
    writeln!(md, "- filters: none | trend_agree | ny_only | skip_news (<=2)").unwrap();
    writeln!(md, "- direction: both | long | short; size 1.0 | 0.5").unwrap();
    writeln!(md, "\n## Costs\n\nCommission $2.00/side; 0 slippage on touched limit fills; 1 tick on market/stop fills; gap-through fills at the gap price.\n").unwrap();
    std::fs::write(dir.join("rulebook.md"), md).map_err(|e| e.to_string())?;
    Ok(())
}
