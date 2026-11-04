//! Ingest: read every raw file in data/futures/raw/, record sha256,
//! audit it, session-date the bars (dropping the halt hour), and write
//! the canonical store split at SEAL_START. The ingest is the only
//! writer of data/sealed/.

use chrono::NaiveDate;
use sha2::{Digest, Sha256};
use std::path::Path;

use crate::pal::audit::{self, load_rolls, Audit};
use crate::pal::parquet::BarRow;
use crate::pal::raw::{self, RawBar};
use crate::pal::sessions::bar_session_date;
use crate::pal::store::{Store, StoreError};
use crate::pal::types::Market;

pub fn sha256_file(path: &Path) -> Result<String, StoreError> {
    let bytes = std::fs::read(path).map_err(|e| e.to_string())?;
    Ok(hex_sha(&bytes))
}

pub fn hex_sha(bytes: &[u8]) -> String {
    let mut h = Sha256::new();
    h.update(bytes);
    format!("{:x}", h.finalize())
}

pub struct IngestReport {
    pub market: Market,
    pub files: Vec<(String, String)>, // (sha256, filename)
    pub audit: Audit,
    pub rows_written_open: usize,
    pub rows_written_sealed: usize,
    pub first_session: Option<NaiveDate>,
    pub last_session: Option<NaiveDate>,
    pub sealed_manifest_sha: Option<String>,
}

/// Ingest one market. `kind` selects which raw files belong to it:
/// filenames must contain the market key ("es"/"nq", case-insensitive)
/// and, when `want_adjusted`, must not contain "unadjusted".
pub fn ingest_market(
    store: &Store,
    market: Market,
    want_adjusted: bool,
) -> Result<IngestReport, StoreError> {
    let key = market.as_str();
    let files = raw::load_raw_dir(&store.raw_dir())?;
    let mut picked: Vec<(std::path::PathBuf, Vec<RawBar>)> = Vec::new();
    let mut unadj: Vec<RawBar> = Vec::new();
    for (path, bars) in files {
        let name = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_lowercase();
        if !name.contains(key) {
            continue;
        }
        if name.contains("unadjusted") {
            unadj.extend(bars);
        } else {
            picked.push((path, bars));
        }
    }
    if !want_adjusted {
        picked.clear();
    }
    if picked.is_empty() {
        return Err(format!(
            "no raw {} files found in {}",
            market.as_str(),
            store.raw_dir().display()
        )
        .into());
    }

    // hash + audit each raw file
    let mut manifest = Vec::new();
    let mut audit_total = Audit::default();
    let rolls = load_rolls(&store.rolls_file(market))?;
    for (path, bars) in &picked {
        manifest.push((
            sha256_file(path)?,
            path.file_name().unwrap_or_default().to_string_lossy().into_owned(),
        ));
        // unadjusted twin is merged for the adj-step check on the
        // combined run below; per-file we run without it.
        let a = audit::audit_bars(market, bars, None, &rolls);
        merge(&mut audit_total, a);
    }
    // combined bars for the adjusted-vs-unadjusted step check
    let mut all: Vec<RawBar> = picked.iter().flat_map(|(_, b)| b.iter().cloned()).collect();
    all.sort_by_key(|r| r.bar.ts_utc);
    unadj.sort_by_key(|r| r.bar.ts_utc);
    if !unadj.is_empty() && unadj.len() == all.len() {
        let step_audit = audit::audit_bars(market, &all, Some(&unadj), &rolls);
        audit_total.adj_steps_off_roll = step_audit.adj_steps_off_roll;
    }

    // session-date, drop halt bars, dedup (store order preserved)
    let mut rows: Vec<BarRow> = Vec::with_capacity(all.len());
    for rb in all {
        match bar_session_date(rb.bar.ts_utc) {
            Some(sess) => rows.push(BarRow {
                bar: rb.bar,
                session: sess,
                contract: rb.contract,
            }),
            None => {}
        }
    }
    rows.sort_by(|a, b| {
        a.session
            .cmp(&b.session)
            .then(a.bar.ts_utc.cmp(&b.bar.ts_utc))
    });
    rows.dedup_by(|a, b| a.bar.ts_utc == b.bar.ts_utc);

    let first_session = rows.first().map(|r| r.session);
    let last_session = rows.last().map(|r| r.session);

    // write split + manifests
    let (n_open, n_sealed) = store.write_1m(market, &rows)?;
    Store::write_manifest(&store.raw_dir(), &manifest)?;
    let mut sealed_manifest_sha = None;
    if n_sealed > 0 {
        let sealed_file = store.sealed_1m(market);
        let sha = sha256_file(&sealed_file)?;
        Store::write_manifest(
            &store.sealed_dir(),
            &[(sha.clone(), sealed_file.file_name().unwrap().to_string_lossy().into_owned())],
        )?;
        sealed_manifest_sha = Some(sha);
    }

    audit::write_audit(&store.results_dir(), market, &audit_total)?;

    Ok(IngestReport {
        market,
        files: manifest,
        audit: audit_total,
        rows_written_open: n_open,
        rows_written_sealed: n_sealed,
        first_session,
        last_session,
        sealed_manifest_sha,
    })
}

fn merge(t: &mut Audit, a: Audit) {
    t.rows_in += a.rows_in;
    t.duplicate_ts += a.duplicate_ts;
    t.out_of_order_ts += a.out_of_order_ts;
    t.halt_dropped += a.halt_dropped;
    t.zero_volume += a.zero_volume;
    t.ohlc_violations += a.ohlc_violations;
    t.gaps_gt_1m += a.gaps_gt_1m;
    t.gaps_gt_5m += a.gaps_gt_5m;
    t.gaps_gt_60m += a.gaps_gt_60m;
    t.adj_steps_off_roll += a.adj_steps_off_roll;
    t.nan_prices += a.nan_prices;
    t.sessions = t.sessions.max(a.sessions);
    match (t.largest_gap, a.largest_gap) {
        (Some((s1, g1)), Some((s2, g2))) => t.largest_gap = Some(if g2 > g1 { (s2, g2) } else { (s1, g1) }),
        (None, x) => t.largest_gap = x,
        (x, _) => t.largest_gap = x,
    }
}
