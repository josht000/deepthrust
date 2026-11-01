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
    pub volume: Vec<Vec<f64>>,
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

#[cfg(test)]
mod liquidity_rank_tests {
    use super::*;

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
}
