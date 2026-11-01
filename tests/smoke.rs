//! End-to-end smoke test on synthetic AAPL/SPY Parquet bars.

mod common;

use common::write_parquet;
use deepthrust::config::{load_config, Cli};
use deepthrust::data::Bars;
use deepthrust::study::{resolve, supports_strategy, Flags, Study, ORDER};
use deepthrust::symbols_overview::Overview;

fn synthetic(seed: f64, n: usize) -> Bars {
    let mut b = Bars {
        dates: vec![],
        open: vec![],
        high: vec![],
        low: vec![],
        close: vec![],
        volume: vec![],
    };
    let mut d = chrono::NaiveDate::from_ymd_opt(2015, 1, 1).unwrap();
    let mut px: f64 = 50.0;
    while b.dates.len() < n {
        d += chrono::Duration::days(1);
        if chrono::Datelike::weekday(&d).number_from_monday() > 5 {
            continue;
        }
        let i = b.dates.len() as f64;
        let ret = 0.0006 + 0.02 * (i * 0.9 + seed).sin() * (i * 0.13).cos();
        let open = px * (1.0 + 0.002 * (i * 1.7 + seed).sin());
        px *= 1.0 + ret;
        b.dates.push(d);
        b.open.push(open);
        b.high.push(open.max(px) * 1.005);
        b.low.push(open.min(px) * 0.995);
        b.close.push(px);
        b.volume.push(1e6);
    }
    b
}

#[test]
fn s001_pipeline_runs_end_to_end() {
    let dir = std::env::temp_dir().join(format!("deepthrust_smoke_{}", std::process::id()));
    let data = dir.join("cache");
    let cfg_path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/configs/s001_rsi2_connors.yaml"
    );
    let cli = Cli {
        data_dir: Some(data.display().to_string()),
        symbols: Some("AAPL".into()),
    };
    let template = load_config("s001_rsi2_connors", Some(cfg_path), &cli, None).unwrap();
    for (s, seed) in [("AAPL", 0.0), ("SPY", 1.0)] {
        write_parquet(
            &data.join(format!("{s}{}", template.data_suffix)),
            &synthetic(seed, 3200),
        );
    }
    for variant in [None, Some("rsi5_pyramid3"), Some("stop3atr")] {
        let mut cfg = load_config("s001_rsi2_connors", Some(cfg_path), &cli, variant).unwrap();
        cfg.n_draws = 20;
        cfg.n_boot = 100;
        cfg.n_timing_draws = 20;
        let flags = Flags {
            test: false,
            rebuild: false,
            profile: true,
            debug: false,
        };
        let mut study = Study::new(&cfg, &dir.join("results"), &flags);
        let stages = resolve(&["charts".to_string()]).unwrap();
        assert_eq!(stages, ORDER.to_vec());
        for s in stages {
            study.run_stage(s).unwrap();
        }

        let dev = study.row("next_open", "dev").unwrap();
        let test = study.row("next_open", "test").unwrap();
        assert!(
            dev.get("trades") > 0.0,
            "variant {variant:?} produced no dev trades"
        );
        assert!(dev.get("bias") >= 0.0 && test.get("bias") == 0.0 && test.get("trades") > 0.0);
        assert!(
            (dev.get("skill") - (dev.get("total_log") - dev.get("trend") - dev.get("bias"))).abs()
                < 1e-9
        );
        let prefix = variant.map(|v| format!("{v}__")).unwrap_or_default();
        for f in [
            "partition_dev.csv",
            "partition_test.csv",
            "cost_ladder.csv",
            "baselines.csv",
            "report.md",
            "summary.svg",
            "trades_next_open.csv",
        ] {
            assert!(
                dir.join("results/s001_rsi2_connors")
                    .join(format!("{prefix}{f}"))
                    .exists(),
                "{prefix}{f}"
            );
        }
    }
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn s003_baseline_and_variant_generate_one_symbol_overview() {
    let dir = std::env::temp_dir().join(format!("deepthrust_s003_{}", std::process::id()));
    let data = dir.join("cache");
    let cfg_path = concat!(env!("CARGO_MANIFEST_DIR"), "/configs/s003_rsi4_2575.yaml");
    let cli = Cli {
        data_dir: Some(data.display().to_string()),
        symbols: None,
    };
    let template = load_config("s003_rsi4_2575", Some(cfg_path), &cli, None).unwrap();
    for (symbol, seed) in [("QQQ", 0.0), ("SPY", 1.0)] {
        write_parquet(
            &data.join(format!("{symbol}{}", template.data_suffix)),
            &synthetic(seed, 3200),
        );
    }
    let flags = Flags {
        test: false,
        rebuild: false,
        profile: false,
        debug: false,
    };
    let mut batch: Option<Overview> = None;
    for variant in [None, Some("exit75")] {
        let mut cfg = load_config("s003_rsi4_2575", Some(cfg_path), &cli, variant).unwrap();
        cfg.n_draws = 20;
        cfg.n_boot = 100;
        cfg.n_timing_draws = 20;
        let mut study = Study::new(&cfg, &dir.join("results"), &flags);
        for stage in resolve(&["charts".into()]).unwrap() {
            study.run_stage(stage).unwrap();
        }
        assert!(study.row("next_open", "dev").unwrap().get("trades") > 0.0);
        let next = study.symbol_overview().unwrap();
        if let Some(all) = &mut batch {
            all.variants.extend(next.variants);
        } else {
            batch = Some(next);
        }
        let prefix = variant.map(|v| format!("{v}__")).unwrap_or_default();
        let trade_csv = std::fs::read_to_string(
            dir.join("results/s003_rsi4_2575")
                .join(format!("{prefix}trades_next_open.csv")),
        )
        .unwrap();
        assert!(trade_csv.lines().next().unwrap().contains("rsi4"));
    }
    let out = dir.join("results/s003_rsi4_2575");
    let batch = batch.unwrap();
    assert_eq!(batch.variants.len(), 2);
    batch
        .write(
            &out.join("symbols_overview.svg"),
            &out.join("symbols_summary.csv"),
        )
        .unwrap();
    let svg = std::fs::read_to_string(out.join("symbols_overview.svg")).unwrap();
    assert!(svg.contains("QQQ") && svg.contains("SPY"));
    assert!(svg.contains("maxDD"));
    let csv = std::fs::read_to_string(out.join("symbols_summary.csv")).unwrap();
    assert!(csv.contains("baseline,QQQ") && csv.contains("exit75,SPY"));
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn s006_double7_runs_through_shared_study_pipeline() {
    let dir = std::env::temp_dir().join(format!("deepthrust_s006_{}", std::process::id()));
    let data = dir.join("cache");
    let cfg_path = concat!(env!("CARGO_MANIFEST_DIR"), "/configs/s006_double_7s.yaml");
    let cli = Cli {
        data_dir: Some(data.display().to_string()),
        symbols: Some("AAPL".into()),
    };
    let template = load_config("s006_double_7s", Some(cfg_path), &cli, None).unwrap();
    for (symbol, seed) in [("AAPL", 0.0), ("SPY", 1.0)] {
        write_parquet(
            &data.join(format!("{symbol}{}", template.data_suffix)),
            &synthetic(seed, 3200),
        );
    }
    let mut cfg = load_config("s006_double_7s", Some(cfg_path), &cli, None).unwrap();
    cfg.n_draws = 20;
    cfg.n_boot = 100;
    cfg.n_timing_draws = 20;
    let flags = Flags {
        test: false,
        rebuild: false,
        profile: false,
        debug: false,
    };
    let mut study = Study::new(&cfg, &dir.join("results"), &flags);
    for stage in resolve(&["charts".into()]).unwrap() {
        study.run_stage(stage).unwrap();
    }
    assert!(supports_strategy("s006_double_7s"));
    assert!(study.row("next_open", "dev").unwrap().get("trades") > 0.0);
    let report = std::fs::read_to_string(dir.join("results/s006_double_7s/report.md")).unwrap();
    assert!(report.contains("inclusive 7-session low"));
    assert!(!report.contains("RSI(2)"));
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn overnight_strategies_run_close_to_open_and_write_weighted_trades() {
    let dir = std::env::temp_dir().join(format!("deepthrust_overnight_{}", std::process::id()));
    let data = dir.join("cache");
    let flags = Flags {
        test: false,
        rebuild: false,
        profile: false,
        debug: false,
    };

    for (strategy, variant) in [
        ("s021_index_overnight", None),
        ("s021_index_overnight", Some("sma50")),
        ("s105_overnight_carry", None),
        ("s105_overnight_carry", Some("cap2x")),
    ] {
        let config_path = format!(
            "{}/configs/{strategy}.yaml",
            env!("CARGO_MANIFEST_DIR").replace('\\', "/")
        );
        let cli = Cli {
            data_dir: Some(data.display().to_string()),
            symbols: Some("SPY".into()),
        };
        let template = load_config(strategy, Some(&config_path), &cli, variant).unwrap();
        if !data.join(format!("SPY{}", template.data_suffix)).exists() {
            write_parquet(
                &data.join(format!("SPY{}", template.data_suffix)),
                &synthetic(1.0, 3200),
            );
        }
        let mut cfg = load_config(strategy, Some(&config_path), &cli, variant).unwrap();
        cfg.n_draws = 20;
        cfg.n_boot = 100;
        cfg.n_timing_draws = 20;
        let mut study = Study::new(&cfg, &dir.join("results"), &flags);
        for stage in resolve(&["partition".into()]).unwrap() {
            study.run_stage(stage).unwrap();
        }
        assert!(study.row("close_to_open", "dev").unwrap().get("trades") > 0.0);
        let prefix = variant.map(|name| format!("{name}__")).unwrap_or_default();
        let trades = std::fs::read_to_string(
            dir.join("results")
                .join(strategy)
                .join(format!("{prefix}trades_close_to_open.csv")),
        )
        .unwrap();
        let mut rows = trades.lines();
        let header = rows.next().unwrap();
        let fields: Vec<_> = header.split(',').collect();
        let entry = fields
            .iter()
            .position(|field| *field == "entry_date")
            .unwrap();
        let exit = fields
            .iter()
            .position(|field| *field == "exit_date")
            .unwrap();
        let hold = fields.iter().position(|field| *field == "hold").unwrap();
        let first = rows.next().unwrap().split(',').collect::<Vec<_>>();
        assert_eq!(first[entry], first[exit]);
        assert_eq!(first[hold], "1");
        let report = std::fs::read_to_string(
            dir.join("results")
                .join(strategy)
                .join(format!("{prefix}report.md")),
        )
        .unwrap();
        assert!(report.contains("close_to_open"));
        assert!(!report.contains("RSI(2)"));
    }
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn s042_sector_rotation_runs_through_shared_study_pipeline() {
    let dir = std::env::temp_dir().join(format!("deepthrust_s042_{}", std::process::id()));
    let data = dir.join("cache");
    let cfg_path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/configs/s042_sector_momentum.yaml"
    );
    let cli = Cli {
        data_dir: Some(data.display().to_string()),
        symbols: Some("A,B".into()),
    };
    for (symbol, seed) in [("A", 0.0), ("B", 2.0)] {
        write_parquet(
            &data.join(format!("{symbol}.parquet")),
            &synthetic(seed, 1000),
        );
    }
    let mut cfg = load_config("s042_sector_momentum", Some(cfg_path), &cli, None).unwrap();
    cfg.start_date = "2015-01-01".into();
    cfg.partition_split_date = "2017-01-01".into();
    cfg.n_draws = 20;
    cfg.n_boot = 100;
    cfg.n_timing_draws = 20;
    let flags = Flags {
        test: false,
        rebuild: false,
        profile: false,
        debug: false,
    };
    let mut study = Study::new(&cfg, &dir.join("results"), &flags);
    for stage in resolve(&["partition".into()]).unwrap() {
        study.run_stage(stage).unwrap();
    }
    assert!(study.row("close", "dev").unwrap().get("trades") > 0.0);
    assert!(study.row("close", "test").unwrap().get("trades") > 0.0);
    let trades =
        std::fs::read_to_string(dir.join("results/s042_sector_momentum/trades_close.csv")).unwrap();
    assert!(trades.starts_with("symbol,signal_date,entry_date,exit_date"));
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn s041_cross_sectional_momentum_runs_with_daily_stock_bars() {
    let dir = std::env::temp_dir().join(format!("deepthrust_s041_{}", std::process::id()));
    let data = dir.join("cache");
    let cfg_path = concat!(env!("CARGO_MANIFEST_DIR"), "/configs/s041_mom_12_1.yaml");
    let cli = Cli {
        data_dir: Some(data.display().to_string()),
        symbols: Some("A,B".into()),
    };
    for (symbol, seed) in [("A", 0.0), ("B", 2.0)] {
        write_parquet(
            &data.join(format!("{symbol}.parquet")),
            &synthetic(seed, 1000),
        );
    }
    let mut cfg = load_config("s041_mom_12_1", Some(cfg_path), &cli, None).unwrap();
    cfg.rank_cache_dir.clear();
    cfg.start_date = "2015-01-01".into();
    cfg.partition_split_date = "2017-01-01".into();
    cfg.min_history = 200;
    cfg.universe_top_n = 2;
    cfg.breadth_mode = "top".into();
    cfg.breadth = 1;
    cfg.book_slots = 1;
    cfg.n_draws = 20;
    cfg.n_boot = 100;
    cfg.n_timing_draws = 20;
    let flags = Flags {
        test: false,
        rebuild: false,
        profile: false,
        debug: false,
    };
    let mut study = Study::new(&cfg, &dir.join("results"), &flags);
    for stage in resolve(&["partition".into()]).unwrap() {
        study.run_stage(stage).unwrap();
    }
    assert!(study.row("close", "dev").unwrap().get("trades") > 0.0);
    assert!(study.row("close", "test").unwrap().get("trades") > 0.0);
    let trades =
        std::fs::read_to_string(dir.join("results/s041_mom_12_1/trades_close.csv")).unwrap();
    assert!(trades.contains("mom252_skip21"));
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn s033_mixed_etf_and_stock_breakout_runs_end_to_end() {
    let dir = std::env::temp_dir().join(format!("deepthrust_s033_{}", std::process::id()));
    let data = dir.join("cache");
    for (symbol, seed) in [("ETF", 0.0), ("STOCK", 2.0)] {
        write_parquet(
            &data.join(format!("{symbol}.parquet")),
            &synthetic(seed, 1000),
        );
    }
    let config = dir.join("s033.yaml");
    std::fs::write(
        &config,
        format!(
            "start_date: '2015-01-01'\npartition_split_date: '2017-01-01'\n\
             etf_symbols: ETF\ninclude_stocks: true\nmin_history: 1\n\
             entry_window: 20\nexit_window: 5\nuniverse_top_n: 1\n\
             rank_window: 5\nrank_min_observations: 1\nmin_price: 5\n\
             delisting_return: -0.30\nfills: next_open,close\nbook_slots: 1\n\
             n_draws: 20\nn_boot: 100\n\
             sources:\n  - name: etfs\n    data_dir: '{}'\n    data_suffix: .parquet\n    symbols: ETF\n    cost_bps_per_side: 5\n\
             \n  - name: stocks\n    data_dir: '{}'\n    data_suffix: .parquet\n    symbols: STOCK\n    cost_bps_per_side: 15\n",
            data.display(),
            data.display()
        ),
    )
    .unwrap();
    let cfg = load_config(
        "s033_52w_breakout",
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
        debug: false,
    };
    let mut study = Study::new(&cfg, &dir.join("results"), &flags);
    for stage in resolve(&["partition".into()]).unwrap() {
        study.run_stage(stage).unwrap();
    }
    assert!(study.row("next_open", "dev").unwrap().get("trades") > 0.0);
    assert!(study.row("next_open", "test").unwrap().get("trades") > 0.0);
    assert!(supports_strategy("s032_52w_high_proximity"));
    assert!(supports_strategy("s033_52w_breakout"));
    assert!(supports_strategy("s035_low_volatility"));
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn stage_resolution_adds_dependencies_and_rejects_unknown() {
    assert_eq!(
        resolve(&["rank".into()]).unwrap(),
        vec!["trades", "table", "rank"]
    );
    assert!(resolve(&["nope".into()]).is_err());
}
