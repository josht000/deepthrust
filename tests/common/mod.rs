//! Shared helpers for writing synthetic daily-bar Parquet files.
#![allow(dead_code)]

use deepthrust::data::Bars;
use parquet::data_type::{DoubleType, Int32Type, Int64Type};
use parquet::file::properties::WriterProperties;
use parquet::file::writer::SerializedFileWriter;
use parquet::schema::parser::parse_message_type;
use std::path::Path;
use std::sync::Arc;

pub fn epoch_days(date: chrono::NaiveDate) -> i32 {
    let epoch = chrono::NaiveDate::from_ymd_opt(1970, 1, 1).unwrap();
    date.signed_duration_since(epoch).num_days() as i32
}

/// Write bars with a DATE `date` column, or a TIMESTAMP_MILLIS one when `timestamps` is set.
/// Columns named in `omit` are left out of the file.
pub fn write_parquet_with(path: &Path, bars: &Bars, timestamps: bool, omit: &[&str]) {
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    let date_col = if timestamps {
        "REQUIRED INT64 date (TIMESTAMP(MILLIS,true));"
    } else {
        "REQUIRED INT32 date (DATE);"
    };
    let numeric: Vec<(&str, &Vec<f64>)> = vec![
        ("open", &bars.open),
        ("high", &bars.high),
        ("low", &bars.low),
        ("close", &bars.close),
        ("volume", &bars.volume),
    ]
    .into_iter()
    .filter(|(name, _)| !omit.contains(name))
    .collect();
    let mut msg = format!("message bars {{ {date_col}");
    for (name, _) in &numeric {
        msg += &format!(" REQUIRED DOUBLE {name};");
    }
    msg += " }";
    let schema = Arc::new(parse_message_type(&msg).unwrap());
    let mut writer = SerializedFileWriter::new(
        std::fs::File::create(path).unwrap(),
        schema,
        Arc::new(WriterProperties::builder().build()),
    )
    .unwrap();
    let mut group = writer.next_row_group().unwrap();
    {
        let mut column = group.next_column().unwrap().unwrap();
        if timestamps {
            let ms: Vec<i64> = bars
                .dates
                .iter()
                .map(|d| epoch_days(*d) as i64 * 86_400_000)
                .collect();
            column
                .typed::<Int64Type>()
                .write_batch(&ms, None, None)
                .unwrap();
        } else {
            let days: Vec<i32> = bars.dates.iter().map(|d| epoch_days(*d)).collect();
            column
                .typed::<Int32Type>()
                .write_batch(&days, None, None)
                .unwrap();
        }
        column.close().unwrap();
    }
    for (_, values) in numeric {
        let mut column = group.next_column().unwrap().unwrap();
        column
            .typed::<DoubleType>()
            .write_batch(values, None, None)
            .unwrap();
        column.close().unwrap();
    }
    group.close().unwrap();
    writer.close().unwrap();
}

pub fn write_parquet(path: &Path, bars: &Bars) {
    write_parquet_with(path, bars, false, &[]);
}

pub fn bars(dates: &[&str], base: f64) -> Bars {
    let dates: Vec<chrono::NaiveDate> = dates
        .iter()
        .map(|d| deepthrust::data::parse_date(d).unwrap())
        .collect();
    let n = dates.len();
    let ramp = |off: f64| (0..n).map(|i| base + i as f64 + off).collect::<Vec<f64>>();
    Bars {
        dates,
        open: ramp(0.0),
        high: ramp(1.0),
        low: ramp(-1.0),
        close: ramp(0.5),
        volume: ramp(1000.0),
    }
}
