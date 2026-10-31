//! End-to-end smoke test on synthetic AAPL/SPY Parquet bars.

mod common;

use common::write_parquet;
use deepthrust::config::{load_config, Cli};
use deepthrust::data::Bars;
use deepthrust::study::{resolve, Flags, Study, ORDER};

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
    for (s, seed) in [("AAPL", 0.0), ("SPY", 1.0)] {
        write_parquet(
            &data.join(format!("{s}_1Day_split.parquet")),
            &synthetic(seed, 3200),
        );
    }
    let cli = Cli {
        data_dir: Some(data.display().to_string()),
        symbols: None,
    };
    let cfg_path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/configs/s001_rsi2_connors.yaml"
    );
    for variant in [None, Some("rsi5_pyramid3"), Some("stop3atr")] {
        let mut cfg = load_config("s001_rsi2_connors", Some(cfg_path), &cli, variant).unwrap();
        cfg.n_draws = 20;
        cfg.n_boot = 100;
        cfg.n_timing_draws = 20;
        let flags = Flags {
            test: false,
            rebuild: false,
            profile: true,
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
fn stage_resolution_adds_dependencies_and_rejects_unknown() {
    assert_eq!(
        resolve(&["rank".into()]).unwrap(),
        vec!["trades", "table", "rank"]
    );
    assert!(resolve(&["nope".into()]).is_err());
}
