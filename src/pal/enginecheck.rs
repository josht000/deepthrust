//! Engine checks (spec prompt 2): unit tests live in engine.rs; here
//! are the data-driven checks — 20k random market entries (1 ATR stop,
//! 2R target, should lose a little after costs) and the cheat probe
//! (peeks 60 minutes ahead, must make a lot — deleted after it runs).

use std::collections::HashSet;

use chrono::NaiveDate;

use crate::pal::detctx::DetCtx;
use crate::pal::engine::{run_order, CostModel, Direction, Entry, OrderSpec, Target};
use crate::pal::rng::Rng;
use crate::pal::types::ChartId;

/// Result of a batch of simulated entries.
pub struct BatchResult {
    pub n: usize,
    pub mean_pnl: f64,
    pub total_pnl: f64,
    pub wins: usize,
}

/// `n` random market entries in the context with 1-ATR (1m chart) stop
/// and 2R target. Deterministic via the master-seeded fork.
pub fn random_entries(ctx: &mut DetCtx, n: usize) -> BatchResult {
    let mut rng = Rng::fork("enginecheck/random_entries");
    let costs = CostModel::for_market(crate::pal::types::Market::Nq);
    ctx.atr(ChartId::M1);
    let atr = ctx.atr[&ChartId::M1].clone();
    let mut total = 0.0;
    let mut wins = 0usize;
    let mut done = 0usize;
    let mut attempts = 0usize;
    while done < n && attempts < n * 20 {
        attempts += 1;
        let pos = rng.gen_range(1, ctx.bars.len().saturating_sub(2));
        if ctx.ctx[pos].pre_roll || ctx.ctx[pos].last_before_roll {
            continue;
        }
        let dir = if rng.next_f64() < 0.5 {
            Direction::Long
        } else {
            Direction::Short
        };
        let bar_idx_in_chart = pos; // 1m chart index == global pos
        let Some(a) = atr.get(bar_idx_in_chart.saturating_sub(1)).copied().flatten() else {
            continue;
        };
        if a <= 0.0 || !a.is_finite() {
            continue;
        }
        // stop beyond ref: entry-at-market, stop 1 ATR away
        let px = ctx.bars[pos].close;
        let stop = px - dir.sign() * a;
        let spec = OrderSpec {
            dir,
            entry: Entry::Market,
            stop,
            target: Target::RMult(2.0),
            time_exit_minutes: 0,
            cancel_after_minutes: 0,
            size: 1.0,
        };
        let t = run_order(&ctx.bars, &ctx.ctx, pos, &spec, &costs);
        done += 1;
        total += t.pnl_usd;
        if t.pnl_usd > 0.0 {
            wins += 1;
        }
    }
    BatchResult {
        n: done,
        mean_pnl: if done > 0 { total / done as f64 } else { 0.0 },
        total_pnl: total,
        wins,
    }
}

/// CHEAT PROBE — peeks 60 minutes ahead to pick direction. Must make a
/// lot of money; spec says delete after it runs. Kept minimal and
/// called only from `pal engine-check --cheat`.
pub fn cheat_probe(ctx: &DetCtx, n: usize) -> BatchResult {
    let mut rng = Rng::fork("enginecheck/cheat_probe");
    let costs = CostModel::for_market(crate::pal::types::Market::Nq);
    let mut total = 0.0;
    let mut wins = 0usize;
    let mut done = 0usize;
    while done < n {
        let pos = rng.gen_range(1, ctx.bars.len().saturating_sub(120));
        // PEEK: choose the direction that wins over the next 60 bars
        let now = ctx.bars[pos].close;
        let later = ctx.bars[(pos + 60).min(ctx.bars.len() - 1)].close;
        let dir = if later > now {
            Direction::Long
        } else {
            Direction::Short
        };
        let spec = OrderSpec {
            dir,
            entry: Entry::Market,
            stop: now - dir.sign() * 1000.0, // absurd stop: pure direction bet
            target: Target::None,
            time_exit_minutes: 60,
            cancel_after_minutes: 0,
            size: 1.0,
        };
        let t = run_order(&ctx.bars, &ctx.ctx, pos, &spec, &costs);
        done += 1;
        total += t.pnl_usd;
        if t.pnl_usd > 0.0 {
            wins += 1;
        }
    }
    BatchResult {
        n: done,
        mean_pnl: if done > 0 { total / done as f64 } else { 0.0 },
        total_pnl: total,
        wins,
    }
}

/// Roll boundaries for DetCtx::build: for each roll_time_utc find the
/// session index and the global 1m position of the last bar before it.
pub fn roll_boundaries(
    sessions: &[crate::pal::types::Session],
    rolls: &[crate::pal::audit::Roll],
) -> std::collections::HashMap<i32, usize> {
    let mut out = std::collections::HashMap::new();
    let mut pos = 0usize;
    for (si, s) in sessions.iter().enumerate() {
        for r in rolls {
            if let Some(last) = s.bars.iter().position(|b| b.ts_utc >= r.roll_time_utc) {
                // the bar before roll_time in this session
                if last > 0 && s.bars[last - 1].ts_utc < r.roll_time_utc {
                    out.entry(si as i32).or_insert(pos + last - 1);
                }
            }
        }
        pos += s.bars.len();
    }
    out
}

/// News-day set for the skip_news filter.
pub fn news_set(days: &[(NaiveDate, String)]) -> HashSet<NaiveDate> {
    days.iter().map(|(d, _)| *d).collect()
}
