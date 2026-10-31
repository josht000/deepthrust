//! Daily-bar loading from split-adjusted Parquet inputs, aligned onto a (dates x symbols) panel.

use chrono::{DateTime, NaiveDate};
use parquet::file::reader::{FileReader, SerializedFileReader};
use parquet::record::Field;
use std::path::{Path, PathBuf};

use crate::timing;

#[derive(Debug, Clone)]
pub struct Bars {
    pub dates: Vec<NaiveDate>,
    pub open: Vec<f64>,
    pub high: Vec<f64>,
    pub low: Vec<f64>,
    pub close: Vec<f64>,
    pub volume: Vec<f64>,
}

/// Date-by-symbol panel; every matrix is indexed `[symbol][row]` with NaN for missing sessions.
#[derive(Debug, Clone)]
pub struct Panel {
    pub dates: Vec<NaiveDate>,
    pub symbols: Vec<String>,
    pub open: Vec<Vec<f64>>,
    pub high: Vec<Vec<f64>>,
    pub low: Vec<Vec<f64>>,
    pub close: Vec<Vec<f64>>,
}

pub fn input_path(dir: &str, symbol: &str, suffix: &str) -> PathBuf {
    Path::new(dir).join(format!("{symbol}{suffix}"))
}

pub fn parse_date(s: &str) -> Result<NaiveDate, String> {
    NaiveDate::parse_from_str(s, "%Y-%m-%d").map_err(|e| format!("bad date {s:?}: {e}"))
}

fn parquet_date(field: &Field, path: &Path, row: usize) -> Result<NaiveDate, String> {
    let date = match field {
        Field::Date(days) => NaiveDate::from_ymd_opt(1970, 1, 1)
            .and_then(|epoch| epoch.checked_add_signed(chrono::Duration::days(*days as i64))),
        Field::TimestampMillis(ts) => DateTime::from_timestamp_millis(*ts).map(|d| d.date_naive()),
        Field::TimestampMicros(ts) => DateTime::from_timestamp_micros(*ts).map(|d| d.date_naive()),
        Field::Str(s) => {
            let date_part = s.get(..10).unwrap_or(s);
            return parse_date(date_part).map_err(|e| format!("{} row {row}: {e}", path.display()));
        }
        _ => None,
    };
    date.ok_or_else(|| {
        format!(
            "{} row {row}: unsupported or invalid Parquet date value {field}",
            path.display()
        )
    })
}

fn parquet_number(field: &Field, name: &str, path: &Path, row: usize) -> Result<f64, String> {
    let value = match field {
        Field::Byte(v) => *v as f64,
        Field::Short(v) => *v as f64,
        Field::Int(v) => *v as f64,
        Field::Long(v) => *v as f64,
        Field::UByte(v) => *v as f64,
        Field::UShort(v) => *v as f64,
        Field::UInt(v) => *v as f64,
        Field::ULong(v) => *v as f64,
        Field::Float(v) => *v as f64,
        Field::Double(v) => *v,
        _ => {
            return Err(format!(
                "{} row {row}: Parquet column {name:?} is not numeric ({field})",
                path.display()
            ))
        }
    };
    if value.is_finite() {
        Ok(value)
    } else {
        Err(format!(
            "{} row {row}: Parquet column {name:?} is not finite",
            path.display()
        ))
    }
}

/// Read a split-adjusted daily-bar Parquet file.
pub fn read_parquet(path: &Path) -> Result<Bars, String> {
    let file = std::fs::File::open(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let reader = SerializedFileReader::new(file)
        .map_err(|e| format!("{}: invalid Parquet file: {e}", path.display()))?;
    let rows = reader
        .get_row_iter(None)
        .map_err(|e| format!("{}: cannot read Parquet rows: {e}", path.display()))?;
    let mut bars = Bars {
        dates: vec![],
        open: vec![],
        high: vec![],
        low: vec![],
        close: vec![],
        volume: vec![],
    };

    for (row_index, row) in rows.enumerate() {
        let row_number = row_index + 1;
        let row = row.map_err(|e| format!("{} row {row_number}: {e}", path.display()))?;
        let columns: std::collections::HashMap<String, &Field> = row
            .get_column_iter()
            .map(|(name, value)| (name.to_ascii_lowercase(), value))
            .collect();
        let get = |names: &[&str]| {
            names
                .iter()
                .find_map(|name| columns.get(*name).copied())
                .ok_or_else(|| {
                    format!(
                        "{} row {row_number}: missing Parquet column (expected {})",
                        path.display(),
                        names.join(" or ")
                    )
                })
        };
        let date = parquet_date(
            get(&["date", "datetime", "timestamp", "index"])?,
            path,
            row_number,
        )?;
        bars.dates.push(date);
        bars.open
            .push(parquet_number(get(&["open"])?, "open", path, row_number)?);
        bars.high
            .push(parquet_number(get(&["high"])?, "high", path, row_number)?);
        bars.low
            .push(parquet_number(get(&["low"])?, "low", path, row_number)?);
        bars.close
            .push(parquet_number(get(&["close"])?, "close", path, row_number)?);
        bars.volume.push(parquet_number(
            get(&["volume"])?,
            "volume",
            path,
            row_number,
        )?);
    }
    Ok(bars)
}

/// Load one symbol from the configured Parquet file, retaining bars from `start` onward.
pub fn load_symbol(
    dir: &str,
    suffix: &str,
    symbol: &str,
    start: NaiveDate,
) -> Result<Bars, String> {
    let path = input_path(dir, symbol, suffix);
    if path.extension().and_then(|ext| ext.to_str()) != Some("parquet") {
        return Err(format!(
            "{} is not a Parquet input; data_suffix must end in .parquet",
            path.display()
        ));
    }
    let mut bars = {
        let _g = timing::stage(&format!("read parquet {symbol}"));
        read_parquet(&path)?
    };
    if bars.dates.windows(2).any(|dates| dates[0] >= dates[1]) {
        return Err(format!(
            "{}: daily-bar dates must be strictly increasing",
            path.display()
        ));
    }
    let first = bars.dates.partition_point(|date| *date < start);
    bars.dates.drain(..first);
    bars.open.drain(..first);
    bars.high.drain(..first);
    bars.low.drain(..first);
    bars.close.drain(..first);
    bars.volume.drain(..first);
    Ok(bars)
}

/// Days after `start_date` the first bar may fall before we warn (covers weekends/holidays).
const START_GAP_TOLERANCE_DAYS: i64 = 7;

/// Warning text when a symbol has no data at or near `start`.
pub fn start_gap_warning(symbol: &str, bars: &Bars, start: NaiveDate) -> Option<String> {
    match bars.dates.first() {
        None => Some(format!("{symbol}: no bars on or after start_date {start}")),
        Some(first) if (*first - start).num_days() > START_GAP_TOLERANCE_DAYS => Some(format!(
            "{symbol}: data not present at or near start_date {start}; first bar is {first}"
        )),
        _ => None,
    }
}

/// Align per-symbol bars on the union of their dates.
pub fn build_panel(symbols: &[String], bars: &[Bars]) -> Panel {
    let mut dates: Vec<NaiveDate> = bars.iter().flat_map(|b| b.dates.iter().copied()).collect();
    dates.sort();
    dates.dedup();
    let n = dates.len();
    let mut p = Panel {
        dates,
        symbols: symbols.to_vec(),
        open: vec![vec![f64::NAN; n]; symbols.len()],
        high: vec![vec![f64::NAN; n]; symbols.len()],
        low: vec![vec![f64::NAN; n]; symbols.len()],
        close: vec![vec![f64::NAN; n]; symbols.len()],
    };
    for (j, b) in bars.iter().enumerate() {
        for (i, d) in b.dates.iter().enumerate() {
            if let Ok(r) = p.dates.binary_search(d) {
                p.open[j][r] = b.open[i];
                p.high[j][r] = b.high[i];
                p.low[j][r] = b.low[i];
                p.close[j][r] = b.close[i];
            }
        }
    }
    p
}
