//! The sealed final test (spec prompt 4).
//!
//! Order of operations:
//!   1. verify the saved-and-hashed funnel artifacts (candidates,
//!      working setups, named versions, versions.jsonl) — stop on any
//!      hash mismatch before anything else happens
//!   2. refuse if the output dir already has results (run-once rule)
//!   3. rehearse mode plays the last discovery month, price-shifted,
//!      through the exact same code — seal stays closed
//!   4. real run requires the named step PAL_STEP=final-test plus
//!      PAL_UNLOCK_SEALED=1; the guard logs every sealed read
//!   5. join discovery bars from 2024-05-01 (detector warm-up) to the
//!      sealed bars; keep only complete sessions; a dropped session is
//!      treated like a roll boundary
//!   6. run every kept funnel version on both markets with the same
//!      costs/fills; save metrics + daily P&L + reports

use std::collections::{BTreeMap, HashMap};
use std::fmt::Write as _;
use std::io::Write as IoWrite;
use std::path::{Path, PathBuf};

use chrono::{Datelike, NaiveDate};
use serde::Serialize;

use crate::pal::detctx::DetCtx;
use crate::pal::funnel::{matched_random_share, metrics, Metrics, SET_COMMON};
use crate::pal::guard::Unlock;
use crate::pal::rulebook::Version;
use crate::pal::runner::run_version;
use crate::pal::sessions::is_complete_session;
use crate::pal::setups::{detect_events, DetectCfg, Event};
use crate::pal::types::{ChartId, Market, Session};
use crate::pal::windows::{SEAL_END_EXPECTED, SEAL_START};

const FINAL_WARMUP: NaiveDate = NaiveDate::from_ymd_opt(2024, 5, 1).unwrap();
const HELD_SHARPE: f64 = 0.5;
const SETUP_HELD_SHARPE: f64 = 0.5;

fn sha256_file(p: &Path) -> String {
    use sha2::{Digest, Sha256};
    let b = std::fs::read(p).unwrap_or_default();
    format!("{:x}", Sha256::digest(&b))
}

/// Verify the saved+hashed prompt-3 artifacts before any unlock.
/// Returns the funnel versions (all rows).
pub fn verify_and_load(funnel_dir: &Path) -> Result<Vec<Version>, String> {
    for name in ["candidates.json", "working_setups.json", "named_versions.json"] {
        let p = funnel_dir.join(name);
        if !p.exists() {
            return Err(format!("missing {name} — run `pal funnel` first"));
        }
        let stem = name.trim_end_matches(".json");
        let sidecar = [format!("{name}.sha256"), format!("{stem}.sha256")]
            .iter()
            .map(|f| funnel_dir.join(f))
            .find(|p| p.exists());
        let Some(sidecar) = sidecar else {
            return Err(format!("missing hash sidecar for {name}"));
        };
        let want = std::fs::read_to_string(&sidecar)
            .map_err(|e| format!("read {}: {e}", sidecar.display()))?;
        let got = sha256_file(&p);
        if got.trim() != want.trim() {
            return Err(format!("{name} hash mismatch — selections changed; stop"));
        }
    }
    // manifest covers versions.jsonl
    let man = std::fs::read_to_string(funnel_dir.join("manifest.sha256"))
        .map_err(|e| format!("missing funnel manifest: {e}"))?;
    let got = sha256_file(&funnel_dir.join("versions.jsonl"));
    let ok = man
        .lines()
        .any(|l| l.ends_with("versions.jsonl") && l.starts_with(&got));
    if !ok {
        return Err("versions.jsonl hash mismatch against manifest — stop".into());
    }

    // parse kept versions
    let text = std::fs::read_to_string(funnel_dir.join("versions.jsonl"))
        .map_err(|e| e.to_string())?;
    let mut out = Vec::new();
    for line in text.lines() {
        let row: serde_json::Value =
            serde_json::from_str(line).map_err(|e| e.to_string())?;
        if !row["kept"].as_bool().unwrap_or(false) {
            continue;
        }
        let v: Version =
            serde_json::from_str(row["version"].as_str().unwrap_or("{}"))
                .map_err(|e| format!("version parse {}: {e}", row["name"]))?;
        out.push(v);
    }
    Ok(out)
}

/// Load candidate flags + discovery metrics for reporting.
pub fn load_funnel_rows(funnel_dir: &Path) -> Result<Vec<FunnelRowLite>, String> {
    let text = std::fs::read_to_string(funnel_dir.join("versions.jsonl"))
        .map_err(|e| e.to_string())?;
    let mut out = Vec::new();
    for line in text.lines() {
        let row: serde_json::Value =
            serde_json::from_str(line).map_err(|e| e.to_string())?;
        out.push(FunnelRowLite {
            name: row["name"].as_str().unwrap_or("").to_string(),
            sha: row["sha"].as_str().unwrap_or("").to_string(),
            sets: row["sets"].as_u64().unwrap_or(0) as u8,
            kept: row["kept"].as_bool().unwrap_or(false),
            candidate: row["candidate"].as_bool().unwrap_or(false),
            nq_sharpe: row["nq"]["sharpe"].as_f64().unwrap_or(0.0),
            setup: serde_json::from_str::<serde_json::Value>(
                row["version"].as_str().unwrap_or("{}"),
            )
            .ok()
            .and_then(|v| v["setup"].as_str().map(|s| s.to_string()))
            .unwrap_or_default(),
        });
    }
    Ok(out)
}

pub struct FunnelRowLite {
    pub name: String,
    pub sha: String,
    pub sets: u8,
    pub kept: bool,
    pub candidate: bool,
    pub nq_sharpe: f64,
    pub setup: String,
}

/// Keep only complete sessions; a dropped (incomplete) session makes
/// its predecessor's last bar a roll boundary ("treat a dropped
/// session like a roll"). Returns (sessions, extra roll map by kept
/// session idx -> last-bar offset).
pub fn complete_only(sessions: &[Session]) -> (Vec<Session>, HashMap<i32, usize>) {
    let mut kept = Vec::new();
    let mut rolls: HashMap<i32, usize> = HashMap::new();
    for s in sessions.iter() {
        if !is_complete_session(s) {
            // predecessor's last bar becomes a roll boundary
            if let Some(prev) = kept.last() {
                let p: &Session = prev;
                rolls.insert(kept.len() as i32 - 1, p.bars.len() - 1);
            }
            continue;
        }
        kept.push(s.clone());
    }
    (kept, rolls)
}

/// DetCtx over kept sessions with roll boundaries from both the rolls
/// file and dropped-session predecessors.
pub fn final_ctx(
    sessions: &[Session],
    rolls: &[crate::pal::audit::Roll],
) -> (DetCtx, Vec<NaiveDate>) {
    let (kept, dropped_rolls) = complete_only(sessions);
    let mut rb = crate::pal::enginecheck::roll_boundaries(&kept, rolls);
    // merge dropped-session boundaries (session-relative offsets ->
    // global positions)
    let mut pos = 0usize;
    for (si, s) in kept.iter().enumerate() {
        if let Some(&off) = dropped_rolls.get(&(si as i32)) {
            rb.entry(si as i32).or_insert(pos + off);
        }
        pos += s.bars.len();
    }
    let ctx = DetCtx::build(&kept, &rb);
    let sealed: Vec<NaiveDate> = ctx
        .session_dates
        .iter()
        .copied()
        .filter(|d| *d >= SEAL_START)
        .collect();
    // rehearse mode: no sealed dates -> all dates count as the window
    let dates = if sealed.is_empty() {
        ctx.session_dates.clone()
    } else {
        sealed
    };
    (ctx, dates)
}

#[derive(Clone, Debug, Serialize)]
pub struct FinalRow {
    pub name: String,
    pub sha: String,
    pub sets: u8,
    pub setup: String,
    pub chart: String,
    pub candidate: bool,
    pub nq: Option<Metrics>,
    pub es: Option<Metrics>,
    pub nq_by_year: BTreeMap<i32, f64>,
    pub es_by_year: BTreeMap<i32, f64>,
    pub held_nq: bool,
    pub held_es: bool,
}

pub struct FinalOut {
    pub rows: Vec<FinalRow>,
    pub daily: Vec<(usize, Market, BTreeMap<NaiveDate, f64>)>,
    /// working-setup matched-random shares over the final window:
    /// (setup, market, share, n_trades)
    pub setup_random: Vec<(String, String, f64, usize)>,
    pub n_versions: usize,
}

fn year_split(
    daily: &BTreeMap<NaiveDate, f64>,
    dates: &[NaiveDate],
) -> BTreeMap<i32, f64> {
    let mut out = BTreeMap::new();
    for d in dates {
        *out.entry(d.year()).or_insert(0.0) += daily.get(d).copied().unwrap_or(0.0);
    }
    out
}

/// Run all kept versions on both markets over the final-window ctxs.
/// `window_dates` = sealed session dates (or all dates in rehearsal).
pub fn run_final(
    nq: &mut DetCtx,
    es: &mut DetCtx,
    nq_dates: &[NaiveDate],
    es_dates: &[NaiveDate],
    versions: &[Version],
    funnel_rows: &[FunnelRowLite],
    news: &std::collections::HashSet<NaiveDate>,
    limit: Option<usize>,
) -> FinalOut {
    // index funnel rows by name for candidate/set info
    let by_name: HashMap<&str, &FunnelRowLite> =
        funnel_rows.iter().map(|r| (r.name.as_str(), r)).collect();
    let versions: Vec<&Version> = versions
        .iter()
        .take(limit.unwrap_or(usize::MAX))
        .collect();
    println!("final: {} kept versions", versions.len());

    // group by detkey (reuse funnel's key fn via re-derivation)
    let mut groups: BTreeMap<String, Vec<usize>> = BTreeMap::new();
    for (i, v) in versions.iter().enumerate() {
        let key = format!(
            "{}|{}|{}|{}|{}|{}|{}|{}|{}|{}",
            v.setup,
            v.chart,
            v.trigger,
            v.swing_strength,
            (v.level_tol_atr * 100.0).round() as i64,
            v.touches,
            (v.zone_run_min << 8) | v.zone_run_max,
            (v.zone_atr_min * 100.0).round() as i64 * 1000
                + (v.zone_atr_max * 100.0).round() as i64,
            v.trendline_anchors,
            ""
        );
        groups.entry(key).or_default().push(i);
    }
    println!("final: {} detection keys", groups.len());

    let mut rows: Vec<FinalRow> = versions
        .iter()
        .map(|v| FinalRow {
            name: v.name.clone(),
            sha: v.sha(),
            sets: by_name.get(v.name.as_str()).map(|r| r.sets).unwrap_or(0),
            setup: v.setup.clone(),
            chart: v.chart.clone(),
            candidate: by_name
                .get(v.name.as_str())
                .map(|r| r.candidate)
                .unwrap_or(false),
            nq: None,
            es: None,
            nq_by_year: BTreeMap::new(),
            es_by_year: BTreeMap::new(),
            held_nq: false,
            held_es: false,
        })
        .collect();
    let mut daily: Vec<(usize, Market, BTreeMap<NaiveDate, f64>)> = Vec::new();
    // trades+twins pooled per (setup, market) for working-setup randoms
    let mut pooled: HashMap<(String, Market), (Vec<crate::pal::engine::Trade>, Vec<crate::pal::runner::TwinInfo>)> =
        HashMap::new();

    for (key, idxs) in &groups {
        // rebuild setup + cfg from the key
        let parts: Vec<&str> = key.split('|').collect();
        let setup = crate::pal::runner::parse_setup(parts[0]);
        let cfg = DetectCfg {
            chart: ChartId::parse(parts[1]).unwrap_or(ChartId::M1),
            trigger: ChartId::parse(parts[2]).unwrap_or(ChartId::M1),
            swing_strength: parts[3].parse().unwrap_or(3),
            level_tol_atr: parts[4].parse::<i64>().unwrap_or(25) as f64 / 100.0,
            touches: parts[5].parse().unwrap_or(2),
            zone_run: (
                parts[6].parse::<usize>().unwrap_or(0) >> 8,
                parts[6].parse::<usize>().unwrap_or(0) & 0xff,
            ),
            zone_atr: {
                let packed = parts[7].parse::<i64>().unwrap_or(0);
                (
                    (packed / 1000) as f64 / 100.0,
                    (packed % 1000) as f64 / 100.0,
                )
            },
            trendline_anchors: parts[8].parse().unwrap_or(2),
        };
        for market in [Market::Nq, Market::Es] {
            let (ctx, dates) = if market == Market::Nq {
                (&mut *nq, nq_dates)
            } else {
                (&mut *es, es_dates)
            };
            let events: Vec<Event> = detect_events(ctx, setup, &cfg);
            for &i in idxs {
                let v = versions[i];
                let rr = run_version(ctx, market, &events, v, news);
                let m = metrics(&rr, dates);
                pooled
                    .entry((v.setup.clone(), market))
                    .or_default()
                    .0
                    .extend(rr.trades.iter().cloned());
                pooled
                    .entry((v.setup.clone(), market))
                    .or_default()
                    .1
                    .extend(rr.twins.iter().cloned());
                let by_year = year_split(&rr.daily_pnl, dates);
                let held = m.pnl > 0.0 && m.sharpe >= HELD_SHARPE;
                match market {
                    Market::Nq => {
                        rows[i].nq = Some(m);
                        rows[i].nq_by_year = by_year;
                        rows[i].held_nq = held;
                    }
                    Market::Es => {
                        rows[i].es = Some(m);
                        rows[i].es_by_year = by_year;
                        rows[i].held_es = held;
                    }
                }
                daily.push((i, market, rr.daily_pnl));
            }
        }
    }

    // matched-random per (setup, market) pooled over kept versions
    let mut setup_random = Vec::new();
    for ((setup, market), (trades, twins)) in &pooled {
        if trades.len() < 10 {
            continue;
        }
        let ctx = if *market == Market::Nq { &mut *nq } else { &mut *es };
        let share = matched_random_share(
            ctx,
            *market,
            trades,
            twins,
            &format!("final-random-{setup}-{}", market.as_str()),
        );
        setup_random.push((setup.clone(), market.as_str().to_string(), share, trades.len()));
    }

    FinalOut {
        rows,
        daily,
        setup_random,
        n_versions: versions.len(),
    }
}

/// Per-setup equal-weight judgment on final data + held-up shares.
pub fn write_final_outputs(
    out: &FinalOut,
    funnel_rows: &[FunnelRowLite],
    versions: &[Version],
    dir: &Path,
) -> Result<(), String> {
    std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;

    // versions.jsonl
    let mut f = std::io::BufWriter::new(
        std::fs::File::create(dir.join("versions.jsonl")).map_err(|e| e.to_string())?,
    );
    for r in &out.rows {
        writeln!(f, "{}", serde_json::to_string(r).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
    }
    drop(f);

    // daily_pnl.jsonl (version index -> name resolved through rows)
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

    // held-up shares: all versions + candidates, per market and chart
    let mut csv = String::from("group,n,held,share\n");
    for mkt in ["nq", "es"] {
        let held_of = |r: &FinalRow| if mkt == "nq" { r.held_nq } else { r.held_es };
        let pnl_of = |r: &FinalRow| {
            if mkt == "nq" {
                r.nq.as_ref().map(|m| m.pnl)
            } else {
                r.es.as_ref().map(|m| m.pnl)
            }
        };
        let all = out.rows.len();
        let held = out.rows.iter().filter(|r| held_of(r)).count();
        writeln!(csv, "all_{mkt},{all},{held},{:.4}", held as f64 / all.max(1) as f64)
            .map_err(|e| e.to_string())?;
        let cands: Vec<&FinalRow> = out.rows.iter().filter(|r| r.candidate).collect();
        let ch = cands.iter().filter(|r| held_of(r)).count();
        writeln!(
            csv,
            "candidates_{mkt},{},{ch},{:.4}",
            cands.len(),
            ch as f64 / cands.len().max(1) as f64
        )
        .map_err(|e| e.to_string())?;
        for chart in ChartId::ALL {
            let rows: Vec<&FinalRow> =
                out.rows.iter().filter(|r| r.chart == chart.as_str()).collect();
            let won = rows
                .iter()
                .filter(|r| pnl_of(r).map(|p| p > 0.0).unwrap_or(false))
                .count();
            writeln!(
                csv,
                "money_chart_{}_{mkt},{},{won},{:.4}",
                chart.as_str(),
                rows.len(),
                won as f64 / rows.len().max(1) as f64
            )
            .map_err(|e| e.to_string())?;
        }
    }
    std::fs::write(dir.join("held_up.csv"), csv).map_err(|e| e.to_string())?;

    // per setup x chart x market: equal-weight daily pnl, per-year + total
    let mut setup_days: HashMap<(String, String, Market), Vec<(f64, &BTreeMap<NaiveDate, f64>)>> =
        HashMap::new();
    let mut dates: HashMap<Market, Vec<NaiveDate>> = HashMap::new();
    for (i, mkt, dm) in &out.daily {
        let v = &versions[*i];
        setup_days
            .entry((v.setup.clone(), v.chart.clone(), *mkt))
            .or_default()
            .push((v.size, dm));
        for d in dm.keys() {
            let e = dates.entry(*mkt).or_default();
            if !e.contains(d) {
                e.push(*d);
            }
        }
    }
    let mut csv = String::from(
        "setup,chart,market,n_versions,pnl,pnl_per_year,sharpe,by_year,held\n",
    );
    for ((setup, chart, mkt), dailies) in &setup_days {
        let ds = dates.get(mkt).cloned().unwrap_or_default();
        let mut ds = ds;
        ds.sort();
        let avg: Vec<f64> = ds
            .iter()
            .map(|d| {
                dailies
                    .iter()
                    .map(|(size, dm)| {
                        let p = dm.get(d).copied().unwrap_or(0.0);
                        if *size > 0.0 { p / *size } else { p }
                    })
                    .sum::<f64>()
                    / dailies.len() as f64
            })
            .collect();
        let n = avg.len().max(1) as f64;
        let sum: f64 = avg.iter().sum();
        let mean = sum / n;
        let var = avg.iter().map(|x| x * x).sum::<f64>() / n - mean * mean;
        let sharpe = if var > 0.0 {
            mean / var.sqrt() * (252.0f64).sqrt()
        } else {
            0.0
        };
        let years = n / 252.0;
        let mut by_year: BTreeMap<i32, f64> = BTreeMap::new();
        for (d, p) in ds.iter().zip(avg.iter()) {
            *by_year.entry(d.year()).or_insert(0.0) += p;
        }
        let ys = by_year
            .iter()
            .map(|(y, p)| format!("{y}:{p:.0}"))
            .collect::<Vec<_>>()
            .join(";");
        let held = sum > 0.0 && sharpe >= SETUP_HELD_SHARPE;
        writeln!(
            csv,
            "{setup},{chart},{},{},{sum:.2},{:.2},{sharpe:.3},{ys},{held}",
            mkt.as_str(),
            dailies.len(),
            if years > 0.0 { sum / years } else { 0.0 }
        )
        .map_err(|e| e.to_string())?;
    }
    std::fs::write(dir.join("setup_final.csv"), csv).map_err(|e| e.to_string())?;

    // working setups vs matched random
    let mut csv = String::from("setup,market,twin_share,n_trades\n");
    for (setup, mkt, share, n) in &out.setup_random {
        writeln!(csv, "{setup},{mkt},{share:.4},{n}").map_err(|e| e.to_string())?;
    }
    std::fs::write(dir.join("working_vs_random.csv"), csv).map_err(|e| e.to_string())?;

    // best usual-settings version per setup by discovery Sharpe
    let mut csv = String::from("setup,name,disc_sharpe,nq_pnl,nq_sharpe,es_pnl,es_sharpe\n");
    for setup in crate::pal::rulebook::Setup::ALL {
        let s = setup.as_str();
        let best = funnel_rows
            .iter()
            .filter(|r| r.kept && r.sets & SET_COMMON != 0 && r.setup == s)
            .max_by(|a, b| a.nq_sharpe.partial_cmp(&b.nq_sharpe).unwrap_or(std::cmp::Ordering::Equal));
        if let Some(b) = best {
            let fr = out.rows.iter().find(|r| r.name == b.name);
            let (np, ns, ep, es) = fr
                .map(|r| {
                    (
                        r.nq.as_ref().map(|m| m.pnl).unwrap_or(0.0),
                        r.nq.as_ref().map(|m| m.sharpe).unwrap_or(0.0),
                        r.es.as_ref().map(|m| m.pnl).unwrap_or(0.0),
                        r.es.as_ref().map(|m| m.sharpe).unwrap_or(0.0),
                    )
                })
                .unwrap_or((0.0, 0.0, 0.0, 0.0));
            writeln!(csv, "{s},{},{:.3},{np:.2},{ns:.3},{ep:.2},{es:.3}", b.name, b.nq_sharpe)
                .map_err(|e| e.to_string())?;
        }
    }
    std::fs::write(dir.join("best_usual.csv"), csv).map_err(|e| e.to_string())?;

    // named versions on final
    let mut csv = String::from("name,nq_pnl,nq_sharpe,es_pnl,es_sharpe,held_nq,held_es\n");
    for r in out.rows.iter().filter(|r| r.sets & crate::pal::funnel::SET_NAMED != 0) {
        writeln!(
            csv,
            "{},{:.2},{:.3},{:.2},{:.3},{},{}",
            r.name,
            r.nq.as_ref().map(|m| m.pnl).unwrap_or(0.0),
            r.nq.as_ref().map(|m| m.sharpe).unwrap_or(0.0),
            r.es.as_ref().map(|m| m.pnl).unwrap_or(0.0),
            r.es.as_ref().map(|m| m.sharpe).unwrap_or(0.0),
            r.held_nq,
            r.held_es
        )
        .map_err(|e| e.to_string())?;
    }
    std::fs::write(dir.join("named_final.csv"), csv).map_err(|e| e.to_string())?;

    // dials the held-up versions have in common (value distribution)
    let mut csv = String::from("market,dial,value,share,n\n");
    for mkt in ["nq", "es"] {
        let held: Vec<&FinalRow> = out
            .rows
            .iter()
            .filter(|r| if mkt == "nq" { r.held_nq } else { r.held_es })
            .collect();
        if held.is_empty() {
            continue;
        }
        let mut counts: HashMap<(String, String), usize> = HashMap::new();
        for r in &held {
            let v = versions.iter().find(|v| v.name == r.name);
            if let Some(v) = v {
                for (dial, val) in [
                    ("setup", v.setup.clone()),
                    ("chart", v.chart.clone()),
                    ("trigger", v.trigger.clone()),
                    ("entry", v.entry.clone()),
                    ("stop", v.stop_kind.as_str().to_string()),
                    ("target", v.target_kind.as_str().to_string()),
                    ("swing", v.swing_strength.to_string()),
                    ("tol", v.level_tol_atr.to_string()),
                    ("touches", v.touches.to_string()),
                ] {
                    *counts.entry((dial.to_string(), val)).or_insert(0) += 1;
                }
            }
        }
        let mut rows: Vec<_> = counts.iter().collect();
        rows.sort();
        for ((dial, val), n) in rows {
            writeln!(
                csv,
                "{mkt},{dial},{val},{:.3},{n}",
                *n as f64 / held.len() as f64
            )
            .map_err(|e| e.to_string())?;
        }
    }
    std::fs::write(dir.join("common_dials.csv"), csv).map_err(|e| e.to_string())?;

    // manifest
    let mut man = String::new();
    for p in [
        "versions.jsonl",
        "daily_pnl.jsonl",
        "held_up.csv",
        "setup_final.csv",
        "working_vs_random.csv",
        "best_usual.csv",
        "named_final.csv",
        "common_dials.csv",
    ] {
        writeln!(man, "{}  {}", sha256_file(&dir.join(p)), p).map_err(|e| e.to_string())?;
    }
    std::fs::write(dir.join("manifest.sha256"), man).map_err(|e| e.to_string())?;
    Ok(())
}

/// Read discovery-warmup + sealed sessions for one market. Caller must
/// hold the unlock; every sealed read is logged by the store guard.
pub fn read_window(
    store: &crate::pal::store::Store,
    m: Market,
    unlock: &Unlock,
) -> Result<Vec<Session>, String> {
    store
        .read_1m(m, FINAL_WARMUP, SEAL_END_EXPECTED, Some(unlock))
        .map_err(|e| format!("{e}"))
}

/// Rehearsal sessions: the last discovery month, price-shifted so the
/// exact final-test code runs on data it has never touched while the
/// seal stays closed.
pub fn rehearsal_sessions(store: &crate::pal::store::Store, m: Market) -> Vec<Session> {
    use crate::pal::windows::DISCOVERY_END;
    let mut sessions = store
        .read_1m(
            m,
            NaiveDate::from_ymd_opt(2024, 6, 1).unwrap(),
            DISCOVERY_END,
            None,
        )
        .unwrap_or_default();
    if sessions.is_empty() {
        // synthetic fallback for machinery validation
        sessions = crate::pal::synth::synth_sessions(
            NaiveDate::from_ymd_opt(2024, 6, 3).unwrap(),
            40,
            &format!("final-rehearse-{}", m.as_str()),
            4.0,
        );
    }
    // shift prices so the path differs from discovery
    let shift = match m {
        Market::Nq => 500.0,
        Market::Es => 200.0,
    };
    for s in &mut sessions {
        for b in &mut s.bars {
            b.open += shift;
            b.high += shift;
            b.low += shift;
            b.close += shift;
        }
    }
    sessions
}

pub fn output_dir(store: &crate::pal::store::Store, rehearse: bool) -> PathBuf {
    store
        .results_dir()
        .join(if rehearse { "final_rehearse" } else { "final" })
}

pub fn dir_has_results(dir: &Path) -> bool {
    dir.exists()
        && std::fs::read_dir(dir)
            .map(|mut it| it.next().is_some())
            .unwrap_or(false)
}

/// The unlock must be the named step per spec.
pub fn require_final_unlock() -> Result<Unlock, String> {
    match Unlock::from_env() {
        Some(u) if u.step == "final-test" => Ok(u),
        Some(u) => Err(format!(
            "unlock step is '{}' — the spec names exactly one unlock step: PAL_STEP=final-test",
            u.step
        )),
        None => Err(
            "final test requires PAL_STEP=final-test and PAL_UNLOCK_SEALED=1".to_string(),
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dropped_session_becomes_roll() {
        // one incomplete session between two complete ones marks the
        // previous kept session's last bar as a boundary
        let mk = |date: NaiveDate, complete: bool| {
            let mut s = crate::pal::synth::synth_sessions(date, 1, "x", 4.0)
                .into_iter()
                .next()
                .unwrap();
            if !complete {
                s.bars.truncate(100); // early close -> incomplete
            }
            s.date = date;
            s
        };
        let a = mk(NaiveDate::from_ymd_opt(2024, 8, 1).unwrap(), true);
        let b = mk(NaiveDate::from_ymd_opt(2024, 8, 2).unwrap(), false);
        let c = mk(NaiveDate::from_ymd_opt(2024, 8, 5).unwrap(), true);
        let (kept, rolls) = complete_only(&[a, b, c]);
        assert_eq!(kept.len(), 2);
        assert!(rolls.contains_key(&0), "predecessor marked: {rolls:?}");
    }
}
