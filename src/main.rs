//! Study runner: `deepthrust --strategy s001_rsi2_connors [--variant NAME[,NAME2]] ...`
//! (CLI mirrors trading_framework's run.py).

use clap::Parser;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use deepthrust::config::{self, Cli, Config};
use deepthrust::study::{self, Flags, Study, NAME};
use deepthrust::{data, timing};

#[derive(Parser, Debug)]
#[command(
    name = "deepthrust",
    about = "Run a strategy study (Rust port of trading_framework run.py)"
)]
struct Args {
    #[arg(long, default_value = NAME)]
    strategy: String,
    #[arg(long)]
    config: Option<String>,
    #[arg(long, default_value = "trades,table,rank,partition,charts")]
    stages: String,
    #[arg(long)]
    test: bool,
    #[arg(long)]
    rebuild: bool,
    #[arg(long)]
    rebuild_cache: bool,
    #[arg(long)]
    profile: bool,
    /// validate inputs and write a manifest without running the study
    #[arg(long)]
    check_data: bool,
    /// comma-separated names from the strategy config's variants: block (default: baseline)
    #[arg(long)]
    variant: Option<String>,
    /// folder of daily-bar CSV files (overrides data_dir in the strategy config)
    #[arg(long)]
    data_dir: Option<String>,
    /// comma-separated symbols (overrides symbols in the strategy config; default AAPL)
    #[arg(long)]
    symbols: Option<String>,
}

fn check_inputs(
    cfg: &Config,
    check_only: bool,
    rebuild_cache: bool,
    results: &Path,
) -> Result<String, String> {
    let _g = timing::stage("input check");
    let symbols = cfg.symbol_list();
    let mut warnings = vec![];
    let mut files = vec![];
    for s in &symbols {
        let p = data::cache_path(&cfg.data_dir, s, &cfg.data_suffix);
        if !p.exists() {
            if check_only {
                return Err(format!("missing input data: {}", p.display()));
            }
            warnings.push(format!(
                "{} not cached; it will be downloaded from Yahoo Finance",
                p.display()
            ));
        } else if rebuild_cache {
            warnings.push(format!(
                "{} will be rebuilt from Yahoo Finance (--rebuild-cache)",
                p.display()
            ));
        }
        files.push(p.display().to_string());
    }
    let dir = results.join("manifests");
    let manifest = serde_json::json!({"strategy": NAME, "variant": cfg.variant, "inputs": {"data_dir": {"symbols": symbols.len(), "symbol_names": symbols, "files": files}}, "warnings": warnings});
    if std::fs::create_dir_all(&dir).is_ok() {
        let _ = std::fs::write(
            dir.join(format!("{}__{}.json", NAME, cfg.variant)),
            serde_json::to_string_pretty(&manifest).unwrap(),
        );
    }
    for w in &warnings {
        println!("Input warning: {w}");
    }
    Ok(format!("data_dir={}", symbols.len()))
}

fn run(args: Args) -> Result<(), String> {
    if args.strategy != NAME {
        return Err(format!(
            "unknown strategy {:?}; implemented: {NAME}",
            args.strategy
        ));
    }
    let results: PathBuf = PathBuf::from("results");
    if !args.check_data {
        timing::install("run");
    }
    let stages_req: Vec<String> = config::split_csv(&args.stages);
    let stages = study::resolve(&stages_req)?;
    let flags = Flags {
        test: args.test,
        rebuild: args.rebuild,
        rebuild_cache: args.rebuild_cache,
        profile: args.profile,
    };
    let cli = Cli {
        data_dir: args.data_dir.clone(),
        symbols: args.symbols.clone(),
    };
    let variants: Vec<Option<String>> = {
        let v = config::split_csv(args.variant.as_deref().unwrap_or(""));
        if v.is_empty() {
            vec![None]
        } else {
            v.into_iter().map(Some).collect()
        }
    };
    for variant in variants {
        let cfg = config::load_config(
            &args.strategy,
            args.config.as_deref(),
            &cli,
            variant.as_deref(),
        )?;
        let checked = check_inputs(&cfg, args.check_data, args.rebuild_cache, &results)?;
        println!("Input check {}:{}: {}", args.strategy, cfg.variant, checked);
        if args.check_data {
            continue;
        }
        let _v = timing::stage(&format!("variant {}", cfg.variant));
        let mut study = Study::new(&cfg, &results, &flags);
        for name in &stages {
            let _g = timing::stage(name);
            study
                .run_stage(name)
                .map_err(|e| format!("study stage {name:?} failed: {e}"))?;
        }
    }
    if !args.check_data {
        timing::finish(args.profile, &results);
    }
    Ok(())
}

fn main() -> ExitCode {
    match run(Args::parse()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
}
