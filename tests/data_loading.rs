//! Direct tests for Parquet daily-bar loading and panel alignment.

mod common;

use common::{bars, write_parquet, write_parquet_with};
use deepthrust::data::{build_panel, load_symbol, parse_date, read_parquet};

fn tmp(name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("deepthrust_{name}_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn parquet_round_trip_preserves_every_field() {
    let dir = tmp("roundtrip");
    let src = bars(&["2024-01-02", "2024-01-03", "2024-01-04"], 100.0);
    let path = dir.join("AAA_1Day_split.parquet");
    write_parquet(&path, &src);
    let got = read_parquet(&path).unwrap();
    assert_eq!(got.dates, src.dates);
    assert_eq!(got.open, src.open);
    assert_eq!(got.high, src.high);
    assert_eq!(got.low, src.low);
    assert_eq!(got.close, src.close);
    assert_eq!(got.volume, src.volume);
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn timestamp_dates_are_read_as_calendar_dates() {
    let dir = tmp("timestamps");
    let src = bars(&["2024-01-02", "2024-01-03"], 10.0);
    let path = dir.join("AAA_1Day_split.parquet");
    write_parquet_with(&path, &src, true, &[]);
    assert_eq!(read_parquet(&path).unwrap().dates, src.dates);
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn missing_column_is_reported_by_name() {
    let dir = tmp("missingcol");
    let path = dir.join("AAA_1Day_split.parquet");
    write_parquet_with(&path, &bars(&["2024-01-02"], 10.0), false, &["close"]);
    let err = read_parquet(&path).unwrap_err();
    assert!(err.contains("close"), "{err}");
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn corrupt_file_is_an_error_not_a_panic() {
    let dir = tmp("corrupt");
    let path = dir.join("AAA_1Day_split.parquet");
    std::fs::write(&path, b"not a parquet file").unwrap();
    assert!(read_parquet(&path).is_err());
    assert!(read_parquet(&dir.join("absent.parquet")).is_err());
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn load_symbol_applies_start_date_and_keeps_rows_aligned() {
    let dir = tmp("start");
    let src = bars(&["2023-12-29", "2024-01-02", "2024-01-03"], 50.0);
    write_parquet(&dir.join("AAA_1Day_split.parquet"), &src);
    let start = parse_date("2024-01-02").unwrap();
    let got = load_symbol(dir.to_str().unwrap(), "_1Day_split.parquet", "AAA", start).unwrap();
    assert_eq!(got.dates, src.dates[1..]);
    assert_eq!(got.open, src.open[1..]);
    assert_eq!(got.close, src.close[1..]);
    assert_eq!(got.volume, src.volume[1..]);
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn start_gap_warning_flags_late_or_empty_data_only() {
    use deepthrust::data::start_gap_warning;
    let start = parse_date("2005-01-01").unwrap();
    let late = bars(&["2015-02-02", "2015-02-03"], 10.0);
    let w = start_gap_warning("AAA", &late, start).unwrap();
    assert!(w.contains("AAA") && w.contains("2015-02-02"), "{w}");
    // first bar a few days after start (weekend/holiday) is fine
    assert!(start_gap_warning("AAA", &bars(&["2005-01-03"], 10.0), start).is_none());
    assert!(start_gap_warning("AAA", &bars(&["1999-01-04"], 10.0), start).is_none());
    let empty = bars(&[], 10.0);
    assert!(start_gap_warning("AAA", &empty, start).is_some());
}

#[test]
fn load_symbol_rejects_unsorted_or_duplicate_dates() {
    let dir = tmp("order");
    let start = parse_date("2000-01-01").unwrap();
    for (name, dates) in [
        ("DUP", ["2024-01-02", "2024-01-02"]),
        ("REV", ["2024-01-03", "2024-01-02"]),
    ] {
        write_parquet(
            &dir.join(format!("{name}_1Day_split.parquet")),
            &bars(&dates, 10.0),
        );
        let err =
            load_symbol(dir.to_str().unwrap(), "_1Day_split.parquet", name, start).unwrap_err();
        assert!(err.contains("strictly increasing"), "{name}: {err}");
    }
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn load_symbol_rejects_non_parquet_suffix_and_missing_file() {
    let dir = tmp("suffix");
    let start = parse_date("2000-01-01").unwrap();
    let d = dir.to_str().unwrap();
    assert!(load_symbol(d, "_1Day_tr.csv", "AAA", start)
        .unwrap_err()
        .contains("Parquet"));
    assert!(load_symbol(d, "_1Day_split.parquet", "AAA", start).is_err());
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn panel_aligns_on_union_of_dates_with_nan_gaps() {
    let a = bars(&["2024-01-02", "2024-01-03"], 1.0);
    let b = bars(&["2024-01-03", "2024-01-04"], 100.0);
    let p = build_panel(&["A".into(), "B".into()], &[a.clone(), b.clone()]);
    assert_eq!(p.dates.len(), 3);
    assert_eq!(p.close[0][0], a.close[0]);
    assert!(p.close[1][0].is_nan());
    assert_eq!(p.close[0][1], a.close[1]);
    assert_eq!(p.close[1][1], b.close[0]);
    assert!(p.close[0][2].is_nan());
    assert_eq!(p.close[1][2], b.close[1]);
}

#[test]
fn study_loads_mixed_sources_and_aligns_their_symbols() {
    use deepthrust::config::{load_config, Cli};
    use deepthrust::study::{Flags, Study};

    let dir = tmp("mixed_sources");
    let etfs = dir.join("etfs");
    let stocks = dir.join("stocks");
    write_parquet(
        &etfs.join("SPY_1Day_tr.parquet"),
        &bars(&["2024-01-02", "2024-01-03"], 100.0),
    );
    write_parquet(
        &stocks.join("AAA_1Day_tr.parquet"),
        &bars(&["2024-01-03", "2024-01-04"], 20.0),
    );
    let config = dir.join("mixed.yaml");
    std::fs::write(
        &config,
        format!(
            "start_date: '2024-01-01'\npartition_split_date: '2024-01-04'\n\
             sources:\n  - name: etfs\n    data_dir: '{}'\n    data_suffix: _1Day_tr.parquet\n    symbols: SPY\n    cost_bps_per_side: 5\n\
             \n  - name: stocks\n    data_dir: '{}'\n    data_suffix: _1Day_tr.parquet\n    symbols: AAA\n    cost_bps_per_side: 15\n",
            etfs.display(),
            stocks.display()
        ),
    )
    .unwrap();
    let cfg = load_config(
        "s001_rsi2_connors",
        Some(config.to_str().unwrap()),
        &Cli {
            data_dir: None,
            symbols: None,
        },
        None,
    )
    .unwrap();
    let flags = Flags {
        test: false,
        rebuild: false,
        profile: false,
    };
    let mut study = Study::new(&cfg, &dir.join("results"), &flags);
    let (panel, _) = study.load_all().unwrap();
    assert_eq!(panel.symbols, vec!["AAA", "SPY"]);
    assert_eq!(
        panel.dates,
        vec![
            parse_date("2024-01-02").unwrap(),
            parse_date("2024-01-03").unwrap(),
            parse_date("2024-01-04").unwrap()
        ]
    );
    assert!(panel.close[0][0].is_nan());
    assert_eq!(cfg.cost_for_symbol("SPY"), 0.0005);
    assert_eq!(cfg.cost_for_symbol("AAA"), 0.0015);
    study.run_stage("trades").unwrap();
    let trade_csv =
        std::fs::read_to_string(dir.join("results/s001_rsi2_connors/trades_next_open.csv"))
            .unwrap();
    assert!(trade_csv.starts_with("symbol,source,signal_date"));
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn study_loads_benchmark_from_separate_directory_without_trading_it() {
    use deepthrust::config::{load_config, Cli};
    use deepthrust::study::{Flags, Study};

    let dir = tmp("separate_benchmark");
    let etfs = dir.join("etfs");
    let stocks = dir.join("stocks");
    write_parquet(
        &etfs.join("SPY_1Day_tr.parquet"),
        &bars(&["2024-01-02", "2024-01-03"], 100.0),
    );
    write_parquet(
        &stocks.join("AAA_1Day_tr.parquet"),
        &bars(&["2024-01-02", "2024-01-03"], 20.0),
    );
    let config = dir.join("stocks.yaml");
    std::fs::write(
        &config,
        format!(
            "start_date: '2024-01-01'\npartition_split_date: '2024-01-03'\n\
             data_dir: '{}'\ndata_suffix: _1Day_tr.parquet\nsymbols: AAA\n\
             benchmark: SPY\nbenchmark_data_dir: '{}'\nbenchmark_data_suffix: _1Day_tr.parquet\n",
            stocks.display(),
            etfs.display()
        ),
    )
    .unwrap();
    let cfg = load_config(
        "s001_rsi2_connors",
        Some(config.to_str().unwrap()),
        &Cli {
            data_dir: None,
            symbols: None,
        },
        None,
    )
    .unwrap();
    let flags = Flags {
        test: false,
        rebuild: false,
        profile: false,
    };
    let study = Study::new(&cfg, &dir.join("results"), &flags);
    let (panel, benchmark) = study.load_all().unwrap();

    assert_eq!(panel.symbols, vec!["AAA"]);
    assert_eq!(benchmark.unwrap().0, "SPY");
    let _ = std::fs::remove_dir_all(dir);
}
