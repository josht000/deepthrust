//! The guarded data store. ALL lab reads of bar data go through here so
//! the seal guard sees every access. Layout:
//!   data/futures/raw/         untouched inputs + manifest.sha256
//!   data/futures/processed/   cleaned 1m bars, sessions < SEAL_START
//!   data/sealed/              final window bars + manifest + access log
//!   data/futures/rolls_{m}.csv, news_calendar.csv

use chrono::NaiveDate;
use std::path::{Path, PathBuf};

use crate::pal::guard::{self, GuardError, Unlock};
use crate::pal::parquet::{self, BarRow};
use crate::pal::types::{Market, Session};
use crate::pal::windows::{is_sealed, SEAL_START};

#[derive(Debug)]
pub enum StoreError {
    Guard(GuardError),
    Msg(String),
}

impl std::fmt::Display for StoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StoreError::Guard(e) => write!(f, "{e}"),
            StoreError::Msg(m) => write!(f, "{m}"),
        }
    }
}

impl std::error::Error for StoreError {}

impl From<GuardError> for StoreError {
    fn from(e: GuardError) -> Self {
        StoreError::Guard(e)
    }
}

impl From<String> for StoreError {
    fn from(e: String) -> Self {
        StoreError::Msg(e)
    }
}

pub struct Store {
    pub root: PathBuf,
}

impl Store {
    /// Store rooted at the worktree (cwd) by default.
    pub fn cwd() -> Store {
        Store {
            root: PathBuf::from("."),
        }
    }

    pub fn at(root: impl Into<PathBuf>) -> Store {
        Store { root: root.into() }
    }

    fn p(&self, rel: &str) -> PathBuf {
        self.root.join(rel)
    }

    pub fn raw_dir(&self) -> PathBuf {
        self.p("data/futures/raw")
    }
    pub fn processed_dir(&self) -> PathBuf {
        self.p("data/futures/processed")
    }
    pub fn sealed_dir(&self) -> PathBuf {
        self.p("data/sealed")
    }
    pub fn results_dir(&self) -> PathBuf {
        self.p("results/pal")
    }

    pub fn processed_1m(&self, m: Market) -> PathBuf {
        self.processed_dir().join(format!("{}_1m.parquet", m.as_str()))
    }
    pub fn sealed_1m(&self, m: Market) -> PathBuf {
        self.sealed_dir().join(format!("{}_1m.parquet", m.as_str()))
    }
    pub fn rolls_file(&self, m: Market) -> PathBuf {
        self.p(&format!("data/futures/rolls_{}.csv", m.as_str()))
    }
    pub fn news_file(&self) -> PathBuf {
        self.p("data/futures/news_calendar.csv")
    }

    /// Read 1-minute sessions in [from, to] inclusive. Any request whose
    /// range touches the sealed window requires an `Unlock`; the access
    /// is logged. This is the ONLY way lab code gets bars.
    pub fn read_1m(
        &self,
        m: Market,
        from: NaiveDate,
        to: NaiveDate,
        unlock: Option<&Unlock>,
    ) -> Result<Vec<Session>, StoreError> {
        let mut out = Vec::new();
        if from < SEAL_START {
            let end = to.min(SEAL_START - chrono::Duration::days(1));
            let path = self.processed_1m(m);
            if path.exists() {
                let rows = parquet::read_bars(&path)?;
                out.extend(parquet::rows_to_sessions(rows, from, end));
            }
        }
        if to >= SEAL_START {
            let what = format!(
                "read_1m {} {}..{}",
                m.as_str(),
                from.max(SEAL_START),
                to
            );
            guard::check(&self.sealed_dir(), true, &what, unlock)?;
            let path = self.sealed_1m(m);
            if path.exists() {
                let rows = parquet::read_bars(&path)?;
                out.extend(parquet::rows_to_sessions(rows, from.max(SEAL_START), to));
            }
        }
        Ok(out)
    }

    /// Guarded read of an arbitrary file under data/sealed/, for lab
    /// readers that bypass `read_1m` (e.g. manifest checks at final-test).
    pub fn check_sealed_path(
        &self,
        path: &Path,
        what: &str,
        unlock: Option<&Unlock>,
    ) -> Result<(), StoreError> {
        let touches = guard::path_is_sealed(&self.sealed_dir(), path);
        guard::check(&self.sealed_dir(), touches, what, unlock)?;
        Ok(())
    }

    /// Write cleaned bars, splitting at SEAL_START: earlier sessions to
    /// data/futures/processed/, sealed sessions to data/sealed/ (the only
    /// writer of the sealed window, per spec).
    pub fn write_1m(&self, m: Market, rows: &[BarRow]) -> Result<(usize, usize), StoreError> {
        let open: Vec<&BarRow> = rows.iter().filter(|r| !is_sealed(r.session)).collect();
        let sealed: Vec<&BarRow> = rows.iter().filter(|r| is_sealed(r.session)).collect();
        let n_open = open.len();
        let n_sealed = sealed.len();
        if n_open > 0 {
            let owned: Vec<BarRow> = open
                .into_iter()
                .map(|r| BarRow {
                    bar: r.bar.clone(),
                    session: r.session,
                    contract: r.contract.clone(),
                })
                .collect();
            parquet::write_bars(&self.processed_1m(m), &owned)?;
        }
        if n_sealed > 0 {
            let owned: Vec<BarRow> = sealed
                .into_iter()
                .map(|r| BarRow {
                    bar: r.bar.clone(),
                    session: r.session,
                    contract: r.contract.clone(),
                })
                .collect();
            parquet::write_bars(&self.sealed_1m(m), &owned)?;
        }
        Ok((n_open, n_sealed))
    }

    /// Append a manifest line (sha256 sidecar) — used by ingest and seal.
    pub fn write_manifest(dir: &Path, entries: &[(String, String)]) -> Result<(), StoreError> {
        std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
        let mut s = String::new();
        for (hash, name) in entries {
            s.push_str(&format!("{hash}  {name}\n"));
        }
        std::fs::write(dir.join("manifest.sha256"), s).map_err(|e| e.to_string())?;
        Ok(())
    }

    /// Read the news calendar CSV: `date,event` rows (CPI, NFP, FOMC).
    pub fn read_news(&self) -> Result<Vec<(NaiveDate, String)>, StoreError> {
        let path = self.news_file();
        if !path.exists() {
            return Ok(Vec::new());
        }
        let text = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
        let mut out = Vec::new();
        for (i, line) in text.lines().enumerate() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') || (i == 0 && line.starts_with("date")) {
                continue;
            }
            let mut parts = line.splitn(2, ',');
            let d = NaiveDate::parse_from_str(parts.next().unwrap_or("").trim(), "%Y-%m-%d")
                .map_err(|e| format!("{}:{i}: {e}", path.display()))?;
            out.push((d, parts.next().unwrap_or("").trim().to_string()));
        }
        Ok(out)
    }
}
