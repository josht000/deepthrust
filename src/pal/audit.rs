//! File audit per spec: duplicate timestamps, ordering, intra-session
//! gaps (>1m, >5m, >60m — counted, never filled), OHLC consistency,
//! zero-volume bars, adjusted-minus-unadjusted stepping only at rolls,
//! and halt-hour drops. Results saved to results/pal/audit_<market>.csv
//! and a readable .md summary.

use chrono::NaiveDate;
use std::collections::BTreeMap;
use std::path::Path;

use crate::pal::raw::RawBar;
use crate::pal::sessions::bar_session_date;
use crate::pal::types::Market;

#[derive(Default)]
pub struct Audit {
    pub rows_in: usize,
    pub duplicate_ts: usize,
    pub out_of_order_ts: usize,
    pub halt_dropped: usize,
    pub zero_volume: usize,
    pub ohlc_violations: usize,
    pub gaps_gt_1m: usize,
    pub gaps_gt_5m: usize,
    pub gaps_gt_60m: usize,
    pub sessions: usize,
    /// adjustment-step check results (only when unadjusted twin exists)
    pub adj_steps_off_roll: usize,
    pub nan_prices: usize,
    /// largest intra-session gap minutes, (session, minutes)
    pub largest_gap: Option<(NaiveDate, i64)>,
}

/// A roll record: roll_time_utc, old_contract, new_contract, spread.
#[derive(Clone, Debug)]
pub struct Roll {
    pub roll_time_utc: chrono::DateTime<chrono::Utc>,
    pub old_contract: String,
    pub new_contract: String,
    pub spread: f64,
}

pub fn load_rolls(path: &Path) -> Result<Vec<Roll>, String> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    let mut rdr = csv::Reader::from_path(path).map_err(|e| e.to_string())?;
    let mut out = Vec::new();
    for rec in rdr.records() {
        let rec = rec.map_err(|e| e.to_string())?;
        let ts = rec.get(0).unwrap_or("").trim().replace(' ', "T");
        let ts = format!("{}Z", ts.trim_end_matches('Z'));
        let roll_time_utc = ts
            .parse::<chrono::DateTime<chrono::Utc>>()
            .map_err(|e| format!("{}: bad roll_time_utc {:?}: {e}", path.display(), rec.get(0)))?;
        out.push(Roll {
            roll_time_utc,
            old_contract: rec.get(1).unwrap_or("").to_string(),
            new_contract: rec.get(2).unwrap_or("").to_string(),
            spread: rec.get(3).unwrap_or("0").parse().unwrap_or(0.0),
        });
    }
    Ok(out)
}

/// Audit one market's raw adjusted bars. `unadjusted` is the optional
/// row-for-row unadjusted twin used only for the adjustment-step check.
pub fn audit_bars(
    market: Market,
    bars: &[RawBar],
    unadjusted: Option<&[RawBar]>,
    rolls: &[Roll],
) -> Audit {
    let mut a = Audit {
        rows_in: bars.len(),
        ..Audit::default()
    };
    let mut seen = std::collections::HashSet::new();
    let mut prev_ts = None;
    // per-session last timestamp for gap counting
    let mut last_in_session: BTreeMap<NaiveDate, chrono::DateTime<chrono::Utc>> =
        BTreeMap::new();
    let mut sessions_seen = std::collections::HashSet::new();

    for rb in bars {
        let b = &rb.bar;
        if !seen.insert(b.ts_utc) {
            a.duplicate_ts += 1;
        }
        if let Some(p) = prev_ts {
            if b.ts_utc < p {
                a.out_of_order_ts += 1;
            }
        }
        prev_ts = Some(b.ts_utc);

        if b.open.is_nan() || b.high.is_nan() || b.low.is_nan() || b.close.is_nan() {
            a.nan_prices += 1;
            continue;
        }
        if b.high < b.open.max(b.close) || b.low > b.open.min(b.close) || b.high < b.low {
            a.ohlc_violations += 1;
        }
        if b.volume == 0 {
            a.zero_volume += 1;
        }

        let Some(sess) = bar_session_date(b.ts_utc) else {
            a.halt_dropped += 1;
            continue;
        };
        sessions_seen.insert(sess);
        if let Some(prev) = last_in_session.insert(sess, b.ts_utc) {
            let gap = (b.ts_utc - prev).num_minutes() - 1; // missing minutes between bars
            if gap > 0 {
                let span = (b.ts_utc - prev).num_minutes();
                if span > 1 {
                    a.gaps_gt_1m += 1;
                }
                if span > 5 {
                    a.gaps_gt_5m += 1;
                }
                if span > 60 {
                    a.gaps_gt_60m += 1;
                }
                let better = match a.largest_gap {
                    Some((_, g)) => span > g,
                    None => true,
                };
                if better {
                    a.largest_gap = Some((sess, span));
                }
            }
        }
    }
    a.sessions = sessions_seen.len();

    // adjustment-step check: diff = adjusted.close - unadjusted.close
    // must step only at roll times (within +-1 bar of a roll).
    if let Some(unadj) = unadjusted {
        if unadj.len() == bars.len() {
            let mut prev_diff: Option<f64> = None;
            for (ra, ru) in bars.iter().zip(unadj.iter()) {
                let diff = ra.bar.close - ru.bar.close;
                if let Some(pd) = prev_diff {
                    if (diff - pd).abs() > 1e-6 {
                        // step must coincide with a roll near this timestamp
                        let near_roll = rolls.iter().any(|r| {
                            (ra.bar.ts_utc - r.roll_time_utc).num_minutes().abs() <= 2
                        });
                        if !near_roll {
                            a.adj_steps_off_roll += 1;
                        }
                    }
                }
                prev_diff = Some(diff);
            }
        }
    }
    let _ = market;
    a
}

/// Write the audit outputs: CSV of counts + md summary.
pub fn write_audit(out_dir: &Path, market: Market, a: &Audit) -> Result<(), String> {
    std::fs::create_dir_all(out_dir).map_err(|e| e.to_string())?;
    let csv_path = out_dir.join(format!("audit_{}.csv", market.as_str()));
    let md_path = out_dir.join(format!("audit_{}.md", market.as_str()));
    let rows: [(&str, String); 12] = [
        ("rows_in", a.rows_in.to_string()),
        ("sessions", a.sessions.to_string()),
        ("duplicate_ts", a.duplicate_ts.to_string()),
        ("out_of_order_ts", a.out_of_order_ts.to_string()),
        ("halt_dropped", a.halt_dropped.to_string()),
        ("zero_volume", a.zero_volume.to_string()),
        ("ohlc_violations", a.ohlc_violations.to_string()),
        ("nan_prices", a.nan_prices.to_string()),
        ("gaps_gt_1m", a.gaps_gt_1m.to_string()),
        ("gaps_gt_5m", a.gaps_gt_5m.to_string()),
        ("gaps_gt_60m", a.gaps_gt_60m.to_string()),
        ("adj_steps_off_roll", a.adj_steps_off_roll.to_string()),
    ];
    let mut csv = String::from("check,count\n");
    let mut md = format!("# Audit — {}\n\n| check | count |\n|---|---|\n", market.symbol());
    for (k, v) in &rows {
        csv.push_str(&format!("{k},{v}\n"));
        md.push_str(&format!("| {k} | {v} |\n"));
    }
    if let Some((s, m)) = a.largest_gap {
        md.push_str(&format!(
            "\nLargest intra-session gap: {m} min (session {s}).\n"
        ));
        csv.push_str(&format!("largest_gap_minutes,{m}\n"));
    }
    std::fs::write(&csv_path, csv).map_err(|e| e.to_string())?;
    std::fs::write(&md_path, md).map_err(|e| e.to_string())?;
    Ok(())
}
