//! Raw input loading: spec-format files dropped into data/futures/raw/
//! with columns ts_event (bar open UTC), open, high, low, close, volume,
//! contract. Accepts parquet or CSV. These reads are of raw inputs only
//! (never sealed), so they bypass the Store guard — but raw inputs are
//! never allowed to live under data/sealed/.

use chrono::{DateTime, NaiveDateTime, TimeZone, Utc};
use parquet::file::reader::{FileReader, SerializedFileReader};
use parquet::record::Field;
use std::path::{Path, PathBuf};

use crate::pal::types::Bar;

#[derive(Clone)]
pub struct RawBar {
    pub bar: Bar,
    pub contract: String,
}

/// Load every raw file in `dir` (sorted by name for determinism).
/// Returns (file, bars) per file so ingest can hash and audit per file.
pub fn load_raw_dir(dir: &Path) -> Result<Vec<(PathBuf, Vec<RawBar>)>, String> {
    if !dir.exists() {
        return Err(format!("raw dir {} does not exist", dir.display()));
    }
    let mut files: Vec<PathBuf> = std::fs::read_dir(dir)
        .map_err(|e| e.to_string())?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| {
            p.extension()
                .and_then(|e| e.to_str())
                .map(|e| e == "parquet" || e == "csv")
                .unwrap_or(false)
        })
        .collect();
    files.sort();
    let mut out = Vec::new();
    for f in files {
        let bars = load_raw_file(&f)?;
        out.push((f, bars));
    }
    Ok(out)
}

pub fn load_raw_file(path: &Path) -> Result<Vec<RawBar>, String> {
    match path.extension().and_then(|e| e.to_str()) {
        Some("parquet") => load_raw_parquet(path),
        Some("csv") => load_raw_csv(path),
        _ => Err(format!("{}: unsupported raw format", path.display())),
    }
}

fn parse_ts(field: &Field, path: &Path) -> Result<DateTime<Utc>, String> {
    match field {
        Field::Long(v) => Ok(Utc.timestamp_nanos(*v)),
        Field::Int(v) => Ok(Utc.timestamp_opt(*v as i64, 0).unwrap()),
        Field::TimestampMicros(v) => Ok(Utc.timestamp_nanos(*v * 1_000)),
        Field::TimestampMillis(v) => Ok(Utc.timestamp_nanos(*v * 1_000_000)),
        Field::Str(s) => parse_ts_str(s).map_err(|e| format!("{}: {e}", path.display())),
        other => Err(format!(
            "{}: unsupported ts_event field {other:?}",
            path.display()
        )),
    }
}

/// Parse a timestamp string: RFC3339 or "YYYY-MM-DD HH:MM:SS[.n]"
/// assumed UTC.
fn parse_ts_str(s: &str) -> Result<DateTime<Utc>, String> {
    if let Ok(t) = s.parse::<DateTime<Utc>>() {
        return Ok(t);
    }
    for fmt in ["%Y-%m-%d %H:%M:%S%.f", "%Y-%m-%dT%H:%M:%S%.f"] {
        if let Ok(n) = NaiveDateTime::parse_from_str(s, fmt) {
            return Ok(Utc.from_utc_datetime(&n));
        }
    }
    Err(format!("unparseable ts_event {s:?}"))
}

fn f64_of(f: &Field) -> Option<f64> {
    match f {
        Field::Double(v) => Some(*v),
        Field::Float(v) => Some(*v as f64),
        Field::Long(v) => Some(*v as f64),
        Field::Int(v) => Some(*v as f64),
        Field::ULong(v) => Some(*v as f64),
        Field::UInt(v) => Some(*v as f64),
        Field::Str(s) => s.parse().ok(),
        _ => None,
    }
}

fn load_raw_parquet(path: &Path) -> Result<Vec<RawBar>, String> {
    let file = std::fs::File::open(path).map_err(|e| format!("open {}: {e}", path.display()))?;
    let reader = SerializedFileReader::new(file).map_err(|e| e.to_string())?;
    let mut out = Vec::new();
    let mut iter = reader.get_row_iter(None).map_err(|e| e.to_string())?;
    while let Some(row) = iter.next() {
        let row = row.map_err(|e| e.to_string())?;
        let mut ts = None;
        let (mut o, mut h, mut l, mut c) = (None, None, None, None);
        let mut v = 0u64;
        let mut contract = String::new();
        for (name, field) in row.get_column_iter() {
            match name.as_str() {
                "ts_event" | "timestamp" | "time" | "datetime" => {
                    ts = Some(parse_ts(field, path)?)
                }
                "open" => o = f64_of(field),
                "high" => h = f64_of(field),
                "low" => l = f64_of(field),
                "close" => c = f64_of(field),
                "volume" | "vol" => v = f64_of(field).unwrap_or(0.0).max(0.0) as u64,
                "contract" | "symbol" | "ticker" => {
                    contract = match field {
                        Field::Str(s) => s.clone(),
                        Field::Bytes(b) => {
                            String::from_utf8_lossy(b.data()).into_owned()
                        }
                        _ => String::new(),
                    }
                }
                _ => {}
            }
        }
        let ts = ts.ok_or_else(|| format!("{}: missing ts_event column", path.display()))?;
        out.push(RawBar {
            bar: Bar {
                ts_utc: ts,
                open: o.unwrap_or(f64::NAN),
                high: h.unwrap_or(f64::NAN),
                low: l.unwrap_or(f64::NAN),
                close: c.unwrap_or(f64::NAN),
                volume: v,
            },
            contract,
        });
    }
    Ok(out)
}

fn load_raw_csv(path: &Path) -> Result<Vec<RawBar>, String> {
    let mut rdr = csv::Reader::from_path(path).map_err(|e| e.to_string())?;
    let headers = rdr.headers().map_err(|e| e.to_string())?.clone();
    let find = |names: &[&str]| -> Option<usize> {
        headers.iter().position(|h| names.iter().any(|n| h.eq_ignore_ascii_case(n)))
    };
    let ci = find(&["ts_event", "timestamp", "time", "datetime"])
        .ok_or_else(|| format!("{}: no ts_event column", path.display()))?;
    let oi = find(&["open"]).ok_or_else(|| format!("{}: no open column", path.display()))?;
    let hi = find(&["high"]).ok_or_else(|| format!("{}: no high column", path.display()))?;
    let li = find(&["low"]).ok_or_else(|| format!("{}: no low column", path.display()))?;
    let cci = find(&["close"]).ok_or_else(|| format!("{}: no close column", path.display()))?;
    let vi = find(&["volume", "vol"]);
    let ki = find(&["contract", "symbol", "ticker"]);

    let mut out = Vec::new();
    for (line, rec) in rdr.records().enumerate() {
        let rec = rec.map_err(|e| format!("{}:{}: {e}", path.display(), line + 2))?;
        let get = |i: Option<usize>| i.and_then(|i| rec.get(i)).unwrap_or("").trim();
        let ts_raw = get(Some(ci));
        let ts = ts_raw
            .parse::<i64>()
            .map(|n| {
                if n > 1_000_000_000_000 {
                    Utc.timestamp_nanos(n)
                } else {
                    Utc.timestamp_opt(n, 0).unwrap()
                }
            })
            .or_else(|_| Ok::<_, String>(parse_ts_str(ts_raw)?))
            .map_err(|e: String| format!("{}:{}: {e}", path.display(), line + 2))?;
        let num = |i: usize| -> f64 { get(Some(i)).parse().unwrap_or(f64::NAN) };
        out.push(RawBar {
            bar: Bar {
                ts_utc: ts,
                open: num(oi),
                high: num(hi),
                low: num(li),
                close: num(cci),
                volume: vi.map(|i| get(Some(i)).parse().unwrap_or(0)).unwrap_or(0),
            },
            contract: ki.map(|i| get(Some(i)).to_string()).unwrap_or_default(),
        });
    }
    Ok(out)
}
