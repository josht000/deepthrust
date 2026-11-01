//! Daily-bar loading from split-adjusted Parquet inputs, aligned onto a (dates x symbols) panel.

use chrono::{DateTime, NaiveDate};
use parquet::file::reader::{FileReader, SerializedFileReader};
use parquet::record::Field;
use serde::Deserialize;
use std::io::{Read, Seek, SeekFrom};
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
    pub volume: Vec<Vec<f64>>,
}

#[derive(Debug, Clone)]
pub struct CorporateAction {
    pub symbol: String,
    pub date: NaiveDate,
    pub action_type: String,
    pub rate: Option<f64>,
    pub old_rate: Option<f64>,
    pub new_rate: Option<f64>,
}

pub fn input_path(dir: &str, symbol: &str, suffix: &str) -> PathBuf {
    Path::new(dir).join(format!("{symbol}{suffix}"))
}

pub fn parse_date(s: &str) -> Result<NaiveDate, String> {
    NaiveDate::parse_from_str(s, "%Y-%m-%d").map_err(|e| format!("bad date {s:?}: {e}"))
}

fn parquet_string(field: &Field, name: &str, path: &Path, row: usize) -> Result<String, String> {
    match field {
        Field::Str(value) => Ok(value.clone()),
        _ => Err(format!(
            "{} row {row}: Parquet column {name:?} is not a string ({field})",
            path.display()
        )),
    }
}

fn optional_parquet_number(
    field: Option<&Field>,
    name: &str,
    path: &Path,
    row: usize,
) -> Result<Option<f64>, String> {
    match field {
        None | Some(Field::Null) => Ok(None),
        Some(value) => parquet_number(value, name, path, row).map(Some),
    }
}

/// Read cash dividends and split ratios from the daily-bar corporate-action Parquet file.
pub fn read_corporate_actions(path: &Path) -> Result<Vec<CorporateAction>, String> {
    let file = std::fs::File::open(path).map_err(|error| format!("{}: {error}", path.display()))?;
    let reader = SerializedFileReader::new(file)
        .map_err(|error| format!("{}: invalid Parquet file: {error}", path.display()))?;
    let rows = reader
        .get_row_iter(None)
        .map_err(|error| format!("{}: cannot read Parquet rows: {error}", path.display()))?;
    let mut actions = Vec::new();
    for (row_index, row) in rows.enumerate() {
        let row_number = row_index + 1;
        let row = row.map_err(|error| format!("{} row {row_number}: {error}", path.display()))?;
        let columns: std::collections::HashMap<String, &Field> = row
            .get_column_iter()
            .map(|(name, value)| (name.to_ascii_lowercase(), value))
            .collect();
        let get = |name: &str| columns.get(name).copied();
        let symbol_field = get("symbol").ok_or_else(|| {
            format!(
                "{} row {row_number}: missing Parquet column \"symbol\"",
                path.display()
            )
        })?;
        if matches!(symbol_field, Field::Null) {
            continue;
        }
        let symbol = parquet_string(symbol_field, "symbol", path, row_number)?;
        let date = parquet_date(
            get("ex_date").ok_or_else(|| {
                format!(
                    "{} row {row_number}: missing Parquet column \"ex_date\"",
                    path.display()
                )
            })?,
            path,
            row_number,
        )?;
        let action_type = get("action_type")
            .map(|field| parquet_string(field, "action_type", path, row_number))
            .transpose()?
            .unwrap_or_else(|| "cash_dividend".into());
        actions.push(CorporateAction {
            symbol,
            date,
            action_type,
            rate: optional_parquet_number(get("rate"), "rate", path, row_number)?,
            old_rate: optional_parquet_number(get("old_rate"), "old_rate", path, row_number)?,
            new_rate: optional_parquet_number(get("new_rate"), "new_rate", path, row_number)?,
        });
    }
    Ok(actions)
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
        volume: vec![vec![f64::NAN; n]; symbols.len()],
    };
    for (j, b) in bars.iter().enumerate() {
        for (i, d) in b.dates.iter().enumerate() {
            if let Ok(r) = p.dates.binary_search(d) {
                p.open[j][r] = b.open[i];
                p.high[j][r] = b.high[i];
                p.low[j][r] = b.low[i];
                p.close[j][r] = b.close[i];
                p.volume[j][r] = b.volume[i];
            }
        }
    }
    p
}

/// Point-in-time rank by trailing median dollar volume, using only observations
/// through the prior session. `columns` defines the eligible stock universe.
pub fn liquidity_ranks(
    panel: &Panel,
    columns: &[usize],
    window: usize,
    min_observations: usize,
    min_price: f64,
    min_history: usize,
) -> Result<Vec<Vec<f64>>, String> {
    if window == 0 || min_observations == 0 || min_observations > window {
        return Err("liquidity rank window and minimum observations are invalid".into());
    }
    if !min_price.is_finite() || min_price < 0.0 {
        return Err("liquidity rank minimum price must be finite and nonnegative".into());
    }
    let rows = panel.dates.len();
    if panel.close.len() != panel.symbols.len()
        || panel.volume.len() != panel.symbols.len()
        || panel
            .close
            .iter()
            .chain(&panel.volume)
            .any(|series| series.len() != rows)
        || columns.iter().any(|&col| col >= panel.symbols.len())
    {
        return Err("liquidity rank panel dimensions or column indices are invalid".into());
    }
    let mut ranks = vec![vec![f64::NAN; rows]; panel.symbols.len()];
    let mut history = vec![0usize; panel.symbols.len()];
    for day in 0..rows {
        let start = day.saturating_sub(window);
        let mut eligible = Vec::new();
        if day > 0 {
            for &col in columns {
                let close = panel.close[col][day - 1];
                if !close.is_finite() || close < min_price || history[col] < min_history {
                    continue;
                }
                let mut dollar_volume: Vec<f64> = (start..day)
                    .filter_map(|row| {
                        let value = panel.close[col][row] * panel.volume[col][row];
                        value.is_finite().then_some(value)
                    })
                    .collect();
                if dollar_volume.len() < min_observations {
                    continue;
                }
                let mid = dollar_volume.len() / 2;
                dollar_volume.select_nth_unstable_by(mid, f64::total_cmp);
                let median = if dollar_volume.len() % 2 == 0 {
                    let lower = dollar_volume[..mid]
                        .iter()
                        .copied()
                        .fold(f64::NEG_INFINITY, f64::max);
                    (lower + dollar_volume[mid]) / 2.0
                } else {
                    dollar_volume[mid]
                };
                eligible.push((col, median));
            }
        }
        eligible.sort_by(|(col_a, value_a), (col_b, value_b)| {
            value_b.total_cmp(value_a).then_with(|| col_a.cmp(col_b))
        });
        for (rank, (col, _)) in eligible.into_iter().enumerate() {
            ranks[col][day] = rank as f64 + 1.0;
        }
        for (col, series) in panel.close.iter().enumerate() {
            if series[day].is_finite() {
                history[col] += 1;
            }
        }
    }
    Ok(ranks)
}

#[derive(Deserialize)]
struct LiquidRankIndex {
    dates: Vec<String>,
    symbols: Vec<String>,
}

/// Align the framework's precomputed PIT liquidity ranks to the loaded stock panel.
pub fn read_liquid_rank_cache(
    root: &str,
    dates: &[NaiveDate],
    symbols: &[String],
) -> Result<Vec<Vec<f64>>, String> {
    let root = Path::new(root);
    let index_path = root.join("index.json");
    let index_text = std::fs::read_to_string(&index_path)
        .map_err(|error| format!("{}: {error}", index_path.display()))?;
    let index: LiquidRankIndex = serde_json::from_str(&index_text)
        .map_err(|error| format!("{}: invalid rank index: {error}", index_path.display()))?;
    if index.dates.is_empty() || index.symbols.is_empty() {
        return Err(format!("{}: rank index is empty", index_path.display()));
    }

    let cache_dates = index
        .dates
        .iter()
        .map(|date| parse_date(date))
        .collect::<Result<Vec<_>, _>>()?;
    if cache_dates.windows(2).any(|window| window[0] >= window[1]) {
        return Err(format!(
            "{}: rank dates must be strictly increasing",
            index_path.display()
        ));
    }
    let mut source_col = std::collections::HashMap::with_capacity(index.symbols.len());
    for (column, symbol) in index.symbols.iter().enumerate() {
        if source_col.insert(symbol.as_str(), column).is_some() {
            return Err(format!(
                "{}: duplicate rank symbol {symbol:?}",
                index_path.display()
            ));
        }
    }

    let mut destination_rows = vec![None; cache_dates.len()];
    let mut cache_row = std::collections::HashMap::with_capacity(cache_dates.len());
    for (row, date) in cache_dates.iter().enumerate() {
        cache_row.insert(*date, row);
    }
    for (row, date) in dates.iter().enumerate() {
        if let Some(&cached_row) = cache_row.get(date) {
            destination_rows[cached_row] = Some(row);
        }
    }
    let source_cols: Vec<_> = symbols
        .iter()
        .map(|symbol| source_col.get(symbol.as_str()).copied())
        .collect();
    let mut ranks = vec![vec![f64::NAN; dates.len()]; symbols.len()];
    let rank_path = root.join("ranks.npy");
    let mut file = std::fs::File::open(&rank_path)
        .map_err(|error| format!("{}: {error}", rank_path.display()))?;
    let mut prefix = [0u8; 8];
    file.read_exact(&mut prefix)
        .map_err(|error| format!("{}: invalid NPY header: {error}", rank_path.display()))?;
    if &prefix[..6] != b"\x93NUMPY" {
        return Err(format!("{}: invalid NPY magic", rank_path.display()));
    }
    let header_len = match (prefix[6], prefix[7]) {
        (1, 0) => {
            let mut len = [0u8; 2];
            file.read_exact(&mut len)
                .map_err(|error| format!("{}: invalid NPY header: {error}", rank_path.display()))?;
            u16::from_le_bytes(len) as usize
        }
        (2 | 3, 0) => {
            let mut len = [0u8; 4];
            file.read_exact(&mut len)
                .map_err(|error| format!("{}: invalid NPY header: {error}", rank_path.display()))?;
            u32::from_le_bytes(len) as usize
        }
        version => {
            return Err(format!(
                "{}: unsupported NPY version {:?}",
                rank_path.display(),
                version
            ))
        }
    };
    if header_len > 1_000_000 {
        return Err(format!(
            "{}: NPY header is unreasonably large",
            rank_path.display()
        ));
    }
    let mut header = vec![0u8; header_len];
    file.read_exact(&mut header)
        .map_err(|error| format!("{}: truncated NPY header: {error}", rank_path.display()))?;
    let header = String::from_utf8_lossy(&header);
    let fortran_order = if header.contains("'fortran_order': False")
        || header.contains("\"fortran_order\": false")
    {
        false
    } else if header.contains("'fortran_order': True") || header.contains("\"fortran_order\": true")
    {
        true
    } else {
        return Err(format!(
            "{}: NPY fortran_order is missing or invalid",
            rank_path.display()
        ));
    };
    if header.contains(">f") {
        return Err(format!(
            "{}: rank NPY must use little-endian floats",
            rank_path.display()
        ));
    }
    let bytes_per_value = if header.contains("<f4") {
        4
    } else if header.contains("<f8") {
        8
    } else {
        return Err(format!(
            "{}: expected a little-endian float32 or float64 NPY",
            rank_path.display()
        ));
    };
    let shape_start = header
        .find("'shape'")
        .or_else(|| header.find("\"shape\""))
        .ok_or_else(|| format!("{}: NPY shape is missing", rank_path.display()))?;
    let shape_tail = &header[shape_start..];
    let shape_open = shape_tail
        .find('(')
        .ok_or_else(|| format!("{}: invalid NPY shape", rank_path.display()))?;
    let shape_close = shape_tail[shape_open + 1..]
        .find(')')
        .map(|offset| shape_open + 1 + offset)
        .ok_or_else(|| format!("{}: invalid NPY shape", rank_path.display()))?;
    let dimensions = shape_tail[shape_open + 1..shape_close]
        .split(',')
        .filter_map(|value| value.trim().parse::<usize>().ok())
        .collect::<Vec<_>>();
    if dimensions.len() != 2
        || dimensions[0] != cache_dates.len()
        || dimensions[1] != index.symbols.len()
    {
        return Err(format!(
            "{}: NPY shape does not match index.json (expected {}x{}, got {:?})",
            rank_path.display(),
            cache_dates.len(),
            index.symbols.len(),
            dimensions
        ));
    }
    let data_start = file
        .stream_position()
        .map_err(|error| format!("{}: {error}", rank_path.display()))?;
    if fortran_order {
        let column_bytes_len = dimensions[0]
            .checked_mul(bytes_per_value)
            .ok_or_else(|| format!("{}: NPY column size overflow", rank_path.display()))?;
        let mut column_bytes = vec![0u8; column_bytes_len];
        for (symbol, column) in source_cols.iter().enumerate() {
            let Some(column) = column else {
                continue;
            };
            let offset = (*column as u64)
                .checked_mul(column_bytes_len as u64)
                .and_then(|column_offset| data_start.checked_add(column_offset))
                .ok_or_else(|| format!("{}: NPY column offset overflow", rank_path.display()))?;
            file.seek(SeekFrom::Start(offset))
                .map_err(|error| format!("{}: {error}", rank_path.display()))?;
            file.read_exact(&mut column_bytes)
                .map_err(|error| format!("{} column {column}: {error}", rank_path.display()))?;
            for (row, destination) in destination_rows.iter().enumerate() {
                let Some(destination) = destination else {
                    continue;
                };
                let start = row * bytes_per_value;
                let value = if bytes_per_value == 4 {
                    f32::from_le_bytes(column_bytes[start..start + 4].try_into().unwrap()) as f64
                } else {
                    f64::from_le_bytes(column_bytes[start..start + 8].try_into().unwrap())
                };
                ranks[symbol][*destination] = value;
            }
        }
    } else {
        let mut row_bytes = vec![0u8; dimensions[1] * bytes_per_value];
        for (row, destination) in destination_rows.into_iter().enumerate() {
            file.read_exact(&mut row_bytes)
                .map_err(|error| format!("{} row {row}: {error}", rank_path.display()))?;
            let Some(destination) = destination else {
                continue;
            };
            for (symbol, column) in source_cols.iter().enumerate() {
                let Some(column) = column else {
                    continue;
                };
                let start = column * bytes_per_value;
                let value = if bytes_per_value == 4 {
                    f32::from_le_bytes(row_bytes[start..start + 4].try_into().unwrap()) as f64
                } else {
                    f64::from_le_bytes(row_bytes[start..start + 8].try_into().unwrap())
                };
                ranks[symbol][destination] = value;
            }
        }
    }
    Ok(ranks)
}

#[cfg(test)]
mod liquidity_rank_tests {
    use super::*;
    use std::io::Write;

    fn panel() -> Panel {
        let dates = (0..5)
            .map(|day| parse_date(&format!("2024-01-0{}", day + 1)).unwrap())
            .collect::<Vec<_>>();
        Panel {
            dates,
            symbols: vec!["A".into(), "B".into()],
            open: vec![vec![10.0; 5], vec![10.0; 5]],
            high: vec![vec![10.0; 5], vec![10.0; 5]],
            low: vec![vec![10.0; 5], vec![10.0; 5]],
            close: vec![vec![10.0; 5], vec![20.0; 5]],
            volume: vec![vec![10.0; 5], vec![5.0; 5]],
        }
    }

    #[test]
    fn liquidity_rank_uses_prior_session_only_and_stable_ties() {
        let mut panel = panel();
        panel.volume[1][2] = 100.0;
        let ranks = liquidity_ranks(&panel, &[0, 1], 2, 2, 5.0, 1).unwrap();
        assert!(ranks[0][0].is_nan());
        assert_eq!(ranks[0][2], 1.0);
        assert_eq!(ranks[1][2], 2.0);
        assert_eq!(ranks[0][3], 2.0);
        assert_eq!(ranks[1][3], 1.0);
    }

    #[test]
    fn liquidity_rank_enforces_price_history_and_history_length() {
        let mut panel = panel();
        panel.close[0][0] = 4.0;
        panel.close[0][2] = 4.0;
        panel.close[1][0] = f64::NAN;
        panel.close[1][3] = f64::NAN;
        let ranks = liquidity_ranks(&panel, &[0, 1], 3, 2, 5.0, 3).unwrap();
        assert!(ranks[0][3].is_nan());
        assert!(ranks[1][3].is_nan());
        assert!(ranks[0][4].is_finite());
        assert!(ranks[1][4].is_nan());
    }

    #[test]
    fn liquid_rank_cache_aligns_dates_and_symbols() {
        let root =
            std::env::temp_dir().join(format!("deepthrust_liquid_ranks_{}", std::process::id()));
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(
            root.join("index.json"),
            r#"{"dates":["2024-01-02","2024-01-03"],"symbols":["A","B"]}"#,
        )
        .unwrap();
        let mut header = "{'descr': '<f4', 'fortran_order': False, 'shape': (2, 2), }".to_string();
        while (10 + header.len() + 1) % 16 != 0 {
            header.push(' ');
        }
        header.push('\n');
        let mut npy = b"\x93NUMPY\x01\x00".to_vec();
        npy.extend_from_slice(&(header.len() as u16).to_le_bytes());
        npy.extend_from_slice(header.as_bytes());
        for value in [1.0f32, 2.0, 2.0, 1.0] {
            npy.write_all(&value.to_le_bytes()).unwrap();
        }
        std::fs::write(root.join("ranks.npy"), npy).unwrap();
        let ranks = read_liquid_rank_cache(
            root.to_str().unwrap(),
            &[
                parse_date("2024-01-03").unwrap(),
                parse_date("2024-01-02").unwrap(),
                parse_date("2024-01-04").unwrap(),
            ],
            &["B".into(), "A".into(), "C".into()],
        )
        .unwrap();
        assert_eq!(ranks[0][0], 1.0);
        assert_eq!(ranks[0][1], 2.0);
        assert!(ranks[0][2].is_nan());
        assert_eq!(ranks[1][0], 2.0);
        assert_eq!(ranks[1][1], 1.0);
        assert!(ranks[2].iter().all(|rank| rank.is_nan()));
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn liquid_rank_cache_reads_fortran_order_arrays() {
        let root = std::env::temp_dir().join(format!(
            "deepthrust_liquid_ranks_fortran_{}",
            std::process::id()
        ));
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(
            root.join("index.json"),
            r#"{"dates":["2024-01-02","2024-01-03"],"symbols":["A","B"]}"#,
        )
        .unwrap();
        let mut header = "{'descr': '<f4', 'fortran_order': True, 'shape': (2, 2), }".to_string();
        while (10 + header.len() + 1) % 16 != 0 {
            header.push(' ');
        }
        header.push('\n');
        let mut npy = b"\x93NUMPY\x01\x00".to_vec();
        npy.extend_from_slice(&(header.len() as u16).to_le_bytes());
        npy.extend_from_slice(header.as_bytes());
        for value in [1.0f32, 2.0, 10.0, 20.0] {
            npy.write_all(&value.to_le_bytes()).unwrap();
        }
        std::fs::write(root.join("ranks.npy"), npy).unwrap();

        let ranks = read_liquid_rank_cache(
            root.to_str().unwrap(),
            &[
                parse_date("2024-01-03").unwrap(),
                parse_date("2024-01-02").unwrap(),
            ],
            &["B".into(), "A".into()],
        )
        .unwrap();
        assert_eq!(ranks[0], vec![20.0, 10.0]);
        assert_eq!(ranks[1], vec![2.0, 1.0]);
        std::fs::remove_dir_all(root).unwrap();
    }
}

#[cfg(test)]
mod corporate_action_tests {
    use super::*;
    use parquet::data_type::{ByteArray, ByteArrayType, DoubleType, Int32Type};
    use parquet::file::properties::WriterProperties;
    use parquet::file::writer::SerializedFileWriter;
    use parquet::schema::parser::parse_message_type;
    use std::sync::Arc;

    #[test]
    fn reads_dividends_and_split_ratios_from_parquet() {
        let path = std::env::temp_dir().join(format!(
            "deepthrust_corporate_actions_{}.parquet",
            std::process::id()
        ));
        let schema = Arc::new(
            parse_message_type(
                "message actions {
                    OPTIONAL BYTE_ARRAY symbol (UTF8);
                    REQUIRED INT32 ex_date (DATE);
                    REQUIRED BYTE_ARRAY action_type (UTF8);
                    REQUIRED DOUBLE rate;
                    REQUIRED DOUBLE old_rate;
                    REQUIRED DOUBLE new_rate;
                }",
            )
            .unwrap(),
        );
        let mut writer = SerializedFileWriter::new(
            std::fs::File::create(&path).unwrap(),
            schema,
            Arc::new(WriterProperties::builder().build()),
        )
        .unwrap();
        let mut group = writer.next_row_group().unwrap();
        let symbols = [ByteArray::from("ABC"), ByteArray::from("XYZ")];
        let dates = [
            NaiveDate::from_ymd_opt(2024, 1, 3).unwrap(),
            NaiveDate::from_ymd_opt(2024, 1, 4).unwrap(),
            NaiveDate::from_ymd_opt(2024, 1, 5).unwrap(),
        ];
        let epoch = NaiveDate::from_ymd_opt(1970, 1, 1).unwrap();
        let symbol_def_levels = [1, 0, 1];
        let mut column = group.next_column().unwrap().unwrap();
        column
            .typed::<ByteArrayType>()
            .write_batch(&symbols, Some(&symbol_def_levels), None)
            .unwrap();
        column.close().unwrap();
        let mut column = group.next_column().unwrap().unwrap();
        let days = dates
            .iter()
            .map(|date| date.signed_duration_since(epoch).num_days() as i32)
            .collect::<Vec<_>>();
        column
            .typed::<Int32Type>()
            .write_batch(&days, None, None)
            .unwrap();
        column.close().unwrap();
        let mut column = group.next_column().unwrap().unwrap();
        let actions = [
            ByteArray::from("cash_dividend"),
            ByteArray::from("cash_dividend"),
            ByteArray::from("forward_split"),
        ];
        column
            .typed::<ByteArrayType>()
            .write_batch(&actions, None, None)
            .unwrap();
        column.close().unwrap();
        for numbers in [
            &[0.25, 0.0, 0.0][..],
            &[1.0, 1.0, 2.0][..],
            &[1.0, 1.0, 1.0][..],
        ] {
            let mut column = group.next_column().unwrap().unwrap();
            column
                .typed::<DoubleType>()
                .write_batch(numbers, None, None)
                .unwrap();
            column.close().unwrap();
        }
        group.close().unwrap();
        writer.close().unwrap();

        let actions = read_corporate_actions(&path).unwrap();
        assert_eq!(actions.len(), 2);
        assert_eq!(actions[0].symbol, "ABC");
        assert_eq!(actions[0].date, dates[0]);
        assert_eq!(actions[0].action_type, "cash_dividend");
        assert_eq!(actions[0].rate, Some(0.25));
        assert_eq!(actions[1].symbol, "XYZ");
        assert_eq!(actions[1].date, dates[2]);
        assert_eq!(actions[1].action_type, "forward_split");
        assert_eq!(actions[1].old_rate, Some(2.0));
        assert_eq!(actions[1].new_rate, Some(1.0));
        std::fs::remove_file(path).unwrap();
    }
}
