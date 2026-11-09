//! The seal guard. The final window stays sealed: only the ingest writes
//! it, and nothing reads it until the final test (spec). Every lab read
//! of bar data routes through `Store`, which requires an `Unlock` for
//! data/sealed/ paths or any session dated >= SEAL_START.
//!
//! An unlock is a named step plus an environment variable:
//!   PAL_STEP=<step name>  AND  PAL_UNLOCK_SEALED=1
//! Every granted sealed access appends one line to
//! data/sealed/access_log.jsonl.

use chrono::{NaiveDate, Utc};
use serde::Serialize;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

use crate::pal::windows::is_sealed;

#[derive(Debug)]
pub enum GuardError {
    /// A sealed read was attempted without a valid unlock.
    SealedAccessDenied(String),
    Io(std::io::Error),
}

impl std::fmt::Display for GuardError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GuardError::SealedAccessDenied(m) => {
                write!(f, "sealed access denied: {m}")
            }
            GuardError::Io(e) => write!(f, "io error: {e}"),
        }
    }
}

impl std::error::Error for GuardError {}

/// Proof that the caller unlocked the seal for this read.
/// Constructed only via `Unlock::from_env()`.
#[derive(Clone, Debug)]
pub struct Unlock {
    pub step: String,
}

impl Unlock {
    pub fn from_env() -> Option<Unlock> {
        let on = std::env::var("PAL_UNLOCK_SEALED").ok().as_deref() == Some("1");
        let step = std::env::var("PAL_STEP").ok().filter(|s| !s.is_empty());
        match (on, step) {
            (true, Some(step)) => Some(Unlock { step }),
            _ => None,
        }
    }
}

/// Check a read request against the seal. `what` describes the read for
/// the access log (e.g. "read_1m es 2024-05-01..2026-10-16").
/// `touches_seal` is computed by the caller from paths/session dates.
pub fn check(
    sealed_dir: &Path,
    touches_seal: bool,
    what: &str,
    unlock: Option<&Unlock>,
) -> Result<(), GuardError> {
    if !touches_seal {
        return Ok(());
    }
    match unlock {
        Some(u) => {
            log_access(sealed_dir, &u.step, what)?;
            Ok(())
        }
        None => Err(GuardError::SealedAccessDenied(format!(
            "{what} (set PAL_STEP and PAL_UNLOCK_SEALED=1 to unlock; every unlock is logged)"
        ))),
    }
}

/// Whether a filesystem path is inside the sealed directory.
pub fn path_is_sealed(sealed_dir: &Path, path: &Path) -> bool {
    path.starts_with(sealed_dir)
}

/// Whether a session-date range touches the sealed window.
pub fn range_touches_seal(to_inclusive: NaiveDate) -> bool {
    is_sealed(to_inclusive)
}

#[derive(Serialize)]
struct AccessLogEntry {
    ts_utc: String,
    event: &'static str,
    step: String,
    what: String,
}

fn log_access(sealed_dir: &Path, step: &str, what: &str) -> Result<(), GuardError> {
    fs::create_dir_all(sealed_dir).map_err(GuardError::Io)?;
    let path = sealed_dir.join("access_log.jsonl");
    let entry = AccessLogEntry {
        ts_utc: Utc::now().to_rfc3339(),
        event: "sealed_read",
        step: step.to_string(),
        what: what.to_string(),
    };
    let line = serde_json::to_string(&entry).map_err(|e| {
        GuardError::Io(std::io::Error::new(std::io::ErrorKind::Other, e))
    })?;
    let mut f = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .map_err(GuardError::Io)?;
    writeln!(f, "{}", line).map_err(GuardError::Io)?;
    Ok(())
}

pub fn access_log_path(sealed_dir: &Path) -> PathBuf {
    sealed_dir.join("access_log.jsonl")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn denies_without_unlock() {
        let dir = Path::new("data/sealed");
        let r = check(dir, true, "test read", None);
        assert!(matches!(r, Err(GuardError::SealedAccessDenied(_))));
    }

    #[test]
    fn passes_unsealed_without_unlock() {
        let dir = Path::new("data/sealed");
        assert!(check(dir, false, "test read", None).is_ok());
    }

    #[test]
    fn detects_sealed_paths() {
        let dir = Path::new("data/sealed");
        assert!(path_is_sealed(dir, Path::new("data/sealed/es_1m.parquet")));
        assert!(!path_is_sealed(
            dir,
            Path::new("data/futures/processed/es_1m.parquet")
        ));
    }
}
