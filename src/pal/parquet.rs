//! Canonical 1-minute bar parquet I/O for the lab.
//! Schema: ts_event (INT64 nanos UTC, bar open), session_date (INT32
//! days since epoch), open/high/low/close (DOUBLE), volume (INT64),
//! contract (BYTE_ARRAY UTF8). session_date is stored so the guard can
//! verify date ranges even when files are moved around.

use chrono::{NaiveDate, TimeZone, Utc};
use parquet::data_type::{ByteArray, ByteArrayType, DoubleType, Int32Type, Int64Type};
use parquet::file::properties::WriterProperties;
use parquet::file::reader::{FileReader, SerializedFileReader};
use parquet::file::writer::SerializedFileWriter;
use parquet::record::Field;
use parquet::schema::parser::parse_message_type;
use std::path::Path;
use std::sync::Arc;

use crate::pal::types::{Bar, Session};

pub const BAR_SCHEMA: &str = "message bars {
    REQUIRED INT64 ts_event;
    REQUIRED INT32 session_date (DATE);
    REQUIRED DOUBLE open;
    REQUIRED DOUBLE high;
    REQUIRED DOUBLE low;
    REQUIRED DOUBLE close;
    REQUIRED INT64 volume;
    REQUIRED BYTE_ARRAY contract (UTF8);
}";

const EPOCH: NaiveDate = match NaiveDate::from_ymd_opt(1970, 1, 1) {
    Some(d) => d,
    None => panic!(),
};

fn days(d: NaiveDate) -> i32 {
    d.signed_duration_since(EPOCH).num_days() as i32
}

fn undays(i: i32) -> NaiveDate {
    EPOCH + chrono::Duration::days(i as i64)
}

/// One row of the canonical store.
pub struct BarRow {
    pub bar: Bar,
    pub session: NaiveDate,
    pub contract: String,
}

/// Write rows to parquet. Deterministic for identical input rows.
pub fn write_bars(path: &Path, rows: &[BarRow]) -> Result<(), String> {
    if let Some(p) = path.parent() {
        std::fs::create_dir_all(p).map_err(|e| format!("mkdir {}: {e}", p.display()))?;
    }
    let schema = Arc::new(parse_message_type(BAR_SCHEMA).map_err(|e| e.to_string())?);
    let props = Arc::new(WriterProperties::builder().build());
    let file = std::fs::File::create(path).map_err(|e| format!("create {}: {e}", path.display()))?;
    let mut writer = SerializedFileWriter::new(file, schema, props).map_err(|e| e.to_string())?;
    let mut group = writer.next_row_group().map_err(|e| e.to_string())?;

    let ts: Vec<i64> = rows
        .iter()
        .map(|r| r.bar.ts_utc.timestamp_nanos_opt().unwrap_or(0))
        .collect();
    let sd: Vec<i32> = rows.iter().map(|r| days(r.session)).collect();
    let open: Vec<f64> = rows.iter().map(|r| r.bar.open).collect();
    let high: Vec<f64> = rows.iter().map(|r| r.bar.high).collect();
    let low: Vec<f64> = rows.iter().map(|r| r.bar.low).collect();
    let close: Vec<f64> = rows.iter().map(|r| r.bar.close).collect();
    let vol: Vec<i64> = rows.iter().map(|r| r.bar.volume as i64).collect();
    let contract: Vec<ByteArray> = rows
        .iter()
        .map(|r| ByteArray::from(r.contract.as_str()))
        .collect();

    macro_rules! col {
        ($ty:ty, $vals:expr) => {{
            let mut c = group.next_column().map_err(|e| e.to_string())?.unwrap();
            c.typed::<$ty>()
                .write_batch(&$vals, None, None)
                .map_err(|e| e.to_string())?;
            c.close().map_err(|e| e.to_string())?;
        }};
    }
    col!(Int64Type, ts);
    col!(Int32Type, sd);
    col!(DoubleType, open);
    col!(DoubleType, high);
    col!(DoubleType, low);
    col!(DoubleType, close);
    col!(Int64Type, vol);
    col!(ByteArrayType, contract);
    group.close().map_err(|e| e.to_string())?;
    writer.close().map_err(|e| e.to_string())?;
    Ok(())
}

fn get_i64(f: &Field) -> Option<i64> {
    match f {
        Field::Long(v) => Some(*v),
        Field::Int(v) => Some(*v as i64),
        _ => None,
    }
}

fn get_f64(f: &Field) -> Option<f64> {
    match f {
        Field::Double(v) => Some(*v),
        Field::Float(v) => Some(*v as f64),
        Field::Long(v) => Some(*v as f64),
        Field::Int(v) => Some(*v as f64),
        Field::ULong(v) => Some(*v as f64),
        Field::UInt(v) => Some(*v as f64),
        _ => None,
    }
}

fn get_bytes(f: &Field) -> Option<String> {
    match f {
        Field::Bytes(b) => Some(String::from_utf8_lossy(b.data()).into_owned()),
        Field::Str(s) => Some(s.clone()),
        _ => None,
    }
}

/// Read rows from a canonical parquet file.
pub fn read_bars(path: &Path) -> Result<Vec<BarRow>, String> {
    let file = std::fs::File::open(path).map_err(|e| format!("open {}: {e}", path.display()))?;
    let reader = SerializedFileReader::new(file).map_err(|e| e.to_string())?;
    let mut out = Vec::with_capacity(reader.metadata().file_metadata().num_rows() as usize);
    let mut iter = reader.get_row_iter(None).map_err(|e| e.to_string())?;
    while let Some(row) = iter.next() {
        let row = row.map_err(|e| e.to_string())?;
        let mut ts = None;
        let mut sd = None;
        let (mut o, mut h, mut l, mut c) = (None, None, None, None);
        let mut v = 0i64;
        let mut contract = String::new();
        for (name, field) in row.get_column_iter() {
            match name.as_str() {
                "ts_event" => ts = get_i64(field),
                "session_date" => sd = get_i64(field).map(|x| x as i32),
                "open" => o = get_f64(field),
                "high" => h = get_f64(field),
                "low" => l = get_f64(field),
                "close" => c = get_f64(field),
                "volume" => v = get_i64(field).unwrap_or(0),
                "contract" => contract = get_bytes(field).unwrap_or_default(),
                _ => {}
            }
        }
        let ts = ts.ok_or_else(|| format!("{}: missing ts_event", path.display()))?;
        let sd = sd.ok_or_else(|| format!("{}: missing session_date", path.display()))?;
        let ts_utc = Utc.timestamp_nanos(ts);
        out.push(BarRow {
            bar: Bar {
                ts_utc,
                open: o.unwrap_or(f64::NAN),
                high: h.unwrap_or(f64::NAN),
                low: l.unwrap_or(f64::NAN),
                close: c.unwrap_or(f64::NAN),
                volume: v.max(0) as u64,
            },
            session: undays(sd),
            contract,
        });
    }
    Ok(out)
}

/// Group stored rows into sessions (they're stored sorted already, but
/// grouping only by stored session_date column).
pub fn rows_to_sessions(rows: Vec<BarRow>, from: NaiveDate, to: NaiveDate) -> Vec<Session> {
    let mut out: Vec<Session> = Vec::new();
    for r in rows {
        if r.session < from || r.session > to {
            continue;
        }
        match out.last_mut() {
            Some(s) if s.date == r.session => s.bars.push(r.bar),
            _ => out.push(Session {
                date: r.session,
                bars: vec![r.bar],
            }),
        }
    }
    out
}
