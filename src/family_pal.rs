//! PAL bridge: runs the Price Action Lab 5-minute sweep-reclaim engine
//! over Databento ES/NQ futures sessions and maps the resulting trades
//! onto the daily study framework's session-date grid.
//!
//! Trade-level fields (ret, ret_gross, hold, exit_reason) carry the
//! exact PAL economics — $2/side commission, stop/gap slippage, limit
//! fills, roll handling — while the framework supplies the panel,
//! partition tests, variants, and reports. The daily account series is
//! reconstructed by the framework from session closes (its standard
//! approximation); PAL's own minute-level daily P&L remains in
//! results/pal/funnel/daily_pnl.jsonl.

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex, OnceLock};

use chrono::NaiveDate;

use crate::config::Config;
use crate::data::{self, Bars, Panel};
use crate::engine::Trade;
use crate::pal::audit::load_rolls;
use crate::pal::detctx::DetCtx;
use crate::pal::engine::ExitReason;
use crate::pal::enginecheck::{news_set, roll_boundaries};
use crate::pal::rulebook::{
    DirSel, EntryKind, Filters, LevelShape, StopKind, TargetKind, Version,
};
use crate::pal::runner::run_version;
use crate::pal::setups::{detect_events, DetectCfg};
use crate::pal::store::Store;
use crate::pal::types::{ChartId, Market};
use crate::pal::windows::{DISCOVERY_END, DISCOVERY_START};
use crate::signals::Signals;

pub const PAL_NAME: &str = "pal_sweep_reclaim";
pub const PAL_TITLE: &str = "PAL Sweep & Reclaim (5m ES/NQ futures)";

fn market(cfg: &Config) -> Result<Market, String> {
    Market::parse(cfg.pal_market.trim())
        .ok_or_else(|| format!("pal_market {:?} must be \"es\" or \"nq\"", cfg.pal_market))
}

/// Daily OHLCV aggregated from PAL 1-minute sessions (18:00->17:00 ET,
/// dated by session end) plus the same bars as the buy-hold benchmark.
pub fn load_panel(cfg: &Config) -> Result<(Panel, Option<(String, Bars)>), String> {
    let m = market(cfg)?;
    let store = Store::cwd();
    let sessions = store
        .read_1m(m, DISCOVERY_START, DISCOVERY_END, None)
        .map_err(|e| format!("pal {} read_1m: {e}", m.as_str()))?;
    if sessions.is_empty() {
        return Err(format!(
            "pal {}: no sessions in {}..{} — run the PAL ingest first",
            m.as_str(),
            DISCOVERY_START,
            DISCOVERY_END
        ));
    }
    let mut bars = Bars {
        dates: Vec::with_capacity(sessions.len()),
        open: Vec::with_capacity(sessions.len()),
        high: Vec::with_capacity(sessions.len()),
        low: Vec::with_capacity(sessions.len()),
        close: Vec::with_capacity(sessions.len()),
        volume: Vec::with_capacity(sessions.len()),
    };
    for s in &sessions {
        let (Some(first), Some(last)) = (s.bars.first(), s.bars.last()) else {
            continue;
        };
        let mut high = f64::NEG_INFINITY;
        let mut low = f64::INFINITY;
        let mut volume = 0.0;
        for b in &s.bars {
            high = high.max(b.high);
            low = low.min(b.low);
            volume += b.volume as f64;
        }
        bars.dates.push(s.date);
        bars.open.push(first.open);
        bars.high.push(high);
        bars.low.push(low);
        bars.close.push(last.close);
        bars.volume.push(volume);
    }
    if bars.dates.is_empty() {
        return Err(format!("pal {}: sessions had no bars", m.as_str()));
    }
    let symbol = m.symbol().to_string();
    let panel = data::build_panel(&[symbol.clone()], &[bars.clone()]);
    Ok((panel, Some((symbol, bars))))
}

/// Detection context + news set, cached per market for the whole study
/// run — variants share the same discovery window so the multi-GB
/// DetCtx is built once, not once per variant.
struct PalCtx {
    ctx: DetCtx,
    news: HashSet<NaiveDate>,
}

static PAL_CTX: OnceLock<Mutex<HashMap<String, Arc<PalCtx>>>> = OnceLock::new();

fn pal_ctx(cfg: &Config) -> Result<Arc<PalCtx>, String> {
    let m = market(cfg)?;
    let key = m.as_str().to_string();
    let map = PAL_CTX.get_or_init(|| Mutex::new(HashMap::new()));
    if let Some(hit) = map.lock().unwrap().get(&key) {
        return Ok(hit.clone());
    }
    let store = Store::cwd();
    let sessions = store
        .read_1m(m, DISCOVERY_START, DISCOVERY_END, None)
        .map_err(|e| format!("pal {} read_1m: {e}", m.as_str()))?;
    if sessions.is_empty() {
        return Err(format!("pal {}: no discovery sessions", m.as_str()));
    }
    let rolls = load_rolls(&store.rolls_file(m)).unwrap_or_default();
    let rb = roll_boundaries(&sessions, &rolls);
    let ctx = DetCtx::build(&sessions, &rb);
    let news = news_set(&store.read_news().unwrap_or_default());
    let built = Arc::new(PalCtx { ctx, news });
    map.lock().unwrap().insert(key, built.clone());
    Ok(built)
}

fn dir_sel(s: &str) -> DirSel {
    match s.trim().to_ascii_lowercase().as_str() {
        "long" => DirSel::Long,
        "short" => DirSel::Short,
        _ => DirSel::Both,
    }
}

fn stop_kind(s: &str) -> StopKind {
    match s.trim().to_ascii_lowercase().as_str() {
        "beyond_swing" | "swing" => StopKind::BeyondSwing,
        _ => StopKind::BeyondLevel,
    }
}

fn target_kind(s: &str) -> TargetKind {
    match s.trim().to_ascii_lowercase().as_str() {
        "1r" | "r1" => TargetKind::R1,
        "2r" | "r2" => TargetKind::R2,
        "next_swing" | "swing" => TargetKind::NextSwing,
        _ => TargetKind::R3,
    }
}

fn entry_kind(s: &str) -> EntryKind {
    match s.trim().to_ascii_lowercase().as_str() {
        "close_back" => EntryKind::CloseBack,
        "rejection" => EntryKind::Rejection,
        "engulfing" => EntryKind::Engulfing,
        "at_close" => EntryKind::AtClose,
        "limit_back" => EntryKind::LimitBack,
        _ => EntryKind::LimitAtLevel,
    }
}

/// The funnel's h:sweep_reclaim:1169 spec is the baseline; every dial
/// below is overridable per variant. Zone/trendline dials are carried
/// for Version completeness — sweep detection ignores them.
fn version_from_cfg(cfg: &Config) -> Version {
    Version {
        name: format!("pal:{}", if cfg.variant.is_empty() {
            "baseline".to_string()
        } else {
            cfg.variant.clone()
        }),
        setup: "sweep_reclaim".into(),
        chart: cfg.pal_chart.clone(),
        trigger: cfg.pal_trigger.clone(),
        direction: dir_sel(&cfg.pal_dir),
        swing_strength: cfg.pal_swing,
        level_tol_atr: cfg.pal_tol_atr,
        touches: cfg.pal_touches,
        level_shape: LevelShape::Line,
        zone_run_min: 2,
        zone_run_max: 5,
        zone_atr_min: 1.5,
        zone_atr_max: 3.0,
        trendline_anchors: 3,
        entry: entry_kind_name(cfg),
        stop_kind: stop_kind(&cfg.pal_stop),
        target_kind: target_kind(&cfg.pal_target),
        time_exit_bars: 96,
        filters: Filters {
            trend_agree: cfg.pal_trend_agree,
            ny_only: cfg.pal_ny_only,
            skip_news: cfg.pal_skip_news,
        },
        size: cfg.pal_size,
    }
}

fn entry_kind_name(cfg: &Config) -> String {
    entry_kind(&cfg.pal_entry).as_str().to_string()
}

/// Empty daily masks: PAL trades arrive fully formed from the 5-minute
/// engine, so there are no daily-bar signals. `tradable`/`pool` stay
/// all-true so the partition test's random-pick baseline can draw the
/// market on any session; `benchmark_returns` stays None so b_day falls
/// back to the single-symbol buy-hold series.
pub fn pal_signals(panel: &Panel) -> Signals {
    let n_sym = panel.symbols.len();
    let n_rows = panel.dates.len();
    let mask = |v: bool| vec![vec![v; n_rows]; n_sym];
    Signals {
        rsi: vec![vec![f64::NAN; n_rows]; n_sym],
        indicator_name: "r_multiple".into(),
        signal: mask(false),
        exit_now: mask(false),
        add_signal: None,
        atr: None,
        stop_pct: None,
        tradable: mask(true),
        pool: mask(true),
        trade_weights: None,
        entry_mat: None,
        exit_mat: None,
        benchmark_returns: None,
    }
}

/// Detect events + replay the version through the PAL engine, then map
/// each PAL trade onto the panel's session-date grid.
pub fn pal_trades(panel: &Panel, cfg: &Config) -> Result<Vec<Trade>, String> {
    let run = pal_ctx(cfg)?;
    let m = market(cfg)?;
    let v = version_from_cfg(cfg);
    let chart = ChartId::parse(&v.chart)
        .ok_or_else(|| format!("pal_chart {:?} not a PAL chart id", v.chart))?;
    let trigger = ChartId::parse(&v.trigger)
        .ok_or_else(|| format!("pal_trigger {:?} not a PAL chart id", v.trigger))?;
    if !crate::pal::rulebook::valid_triggers(chart).contains(&trigger) {
        return Err(format!(
            "pal: trigger {} is not valid for chart {}",
            v.trigger, v.chart
        ));
    }
    let dcfg = DetectCfg {
        chart,
        trigger,
        swing_strength: v.swing_strength,
        level_tol_atr: v.level_tol_atr,
        touches: v.touches,
        zone_run: (v.zone_run_min, v.zone_run_max),
        zone_atr: (v.zone_atr_min, v.zone_atr_max),
        trendline_anchors: v.trendline_anchors,
    };
    let setup = crate::pal::runner::parse_setup(&v.setup);
    let events = detect_events(&run.ctx, setup, &dcfg);
    let rr = run_version(&run.ctx, m, &events, &v, &run.news);

    let point_value = m.tick_value() / m.tick_size();
    let date_row: HashMap<NaiveDate, usize> = panel
        .dates
        .iter()
        .enumerate()
        .map(|(i, d)| (*d, i))
        .collect();
    let mut out = Vec::with_capacity(rr.trades.len());
    for t in &rr.trades {
        let entry_d = t
            .entry_session
            .try_into()
            .ok()
            .and_then(|i: usize| run.ctx.session_dates.get(i))
            .copied();
        let exit_d = t
            .exit_session
            .try_into()
            .ok()
            .and_then(|i: usize| run.ctx.session_dates.get(i))
            .copied();
        let (Some(ed), Some(xd)) = (entry_d, exit_d) else {
            continue;
        };
        let (Some(&ei), Some(&xi)) = (date_row.get(&ed), date_row.get(&xd)) else {
            continue;
        };
        if xi < ei {
            continue;
        }
        let size = if v.size.is_finite() && v.size > 0.0 {
            v.size
        } else {
            1.0
        };
        let notional = t.entry_px * point_value * size;
        let ret = if notional.is_finite() && notional > 0.0 {
            t.pnl_usd / notional
        } else {
            0.0
        };
        let gross_pts = (t.exit_px - t.entry_px) * t.dir.sign();
        let ret_gross = if t.entry_px != 0.0 {
            gross_pts / t.entry_px
        } else {
            0.0
        };
        out.push(Trade {
            sym_idx: 0,
            signal_idx: ei,
            entry_idx: ei,
            exit_idx: xi,
            entry_px: t.entry_px,
            exit_px: t.exit_px,
            ret,
            ret_gross,
            exit_reason: match t.reason {
                ExitReason::Target => "target",
                ExitReason::Stop => "stop",
                ExitReason::Time => "time",
                ExitReason::Roll => "roll",
                ExitReason::EndOfData => "end",
                ExitReason::Cancelled => "cancelled",
            },
            hold: xi - ei,
            signal_value: t.r_multiple,
            // Unweighted on purpose: PAL trades are sequential
            // round-trips (~28/day), so the framework's daily account
            // series reports the *mean* trade return per session —
            // magnitudes (total_log/CAGR/max_dd) are diluted ~28x vs
            // true sequential compounding, while Sharpe/PF/win-rate and
            // the random-pick p-value stay valid. Weighted mode was
            // rejected: it sums leg weights into exposure (~28x) and
            // breaks the trend/skill decomposition. Authoritative
            // minute-level daily P&L: results/pal/funnel/daily_pnl.jsonl.
            weight: 1.0,
            entry_turnover: 1.0,
            exit_turnover: 1.0,
            weighted: false,
            borrow_cost_per_bar: 0.0,
            dir_sign: t.dir.sign(),
        });
    }
    out.sort_by(|a, b| {
        (a.entry_idx, a.exit_idx).cmp(&(b.entry_idx, b.exit_idx))
    });
    Ok(out)
}
