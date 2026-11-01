//! Study runner: `deepthrust --strategy <name> [--variant NAME[,NAME2]] ...`
//! (CLI mirrors trading_framework's run.py).

use clap::Parser;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use deepthrust::config::{self, Cli, Config};
use deepthrust::study::{self, Flags, Study, NAME};
use deepthrust::symbols_overview::Overview;
use deepthrust::{data, timing};
use std::fs::OpenOptions;
use std::io::Write;

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
    profile: bool,
    /// show warnings and the per-symbol trade-count table
    #[arg(long)]
    debug: bool,
    /// validate inputs and write a manifest without running the study
    #[arg(long)]
    check_data: bool,
    /// comma-separated names from the strategy config's variants: block (default: baseline)
    #[arg(long)]
    variant: Option<String>,
    /// folder containing daily-bar files (overrides every configured source directory)
    #[arg(long)]
    data_dir: Option<String>,
    /// comma-separated symbols (overrides the configured symbols; only for single-source configs)
    #[arg(long)]
    symbols: Option<String>,
}

fn check_inputs(cfg: &Config, results: &Path) -> Result<String, String> {
    let _g = timing::stage("input check");
    let sources = cfg.source_list();
    let mut symbol_names = vec![];
    let mut files = vec![];
    let mut source_manifest = vec![];
    for source in &sources {
        let symbols = config::split_csv(&source.symbols);
        let mut source_files = vec![];
        for symbol in &symbols {
            let p = data::input_path(&source.data_dir, symbol, &source.data_suffix);
            if p.extension().and_then(|ext| ext.to_str()) != Some("parquet") {
                return Err(format!(
                    "{} is not a Parquet input; data_suffix must end in .parquet",
                    p.display()
                ));
            }
            if !p.exists() {
                return Err(format!("missing input data: {}", p.display()));
            }
            source_files.push(p.display().to_string());
            files.push(p.display().to_string());
            symbol_names.push(symbol.clone());
        }
        source_manifest.push(serde_json::json!({
            "name": source.name,
            "data_dir": source.data_dir,
            "symbols": symbols,
            "files": source_files,
            "cost_bps_per_side": source.cost_bps_per_side.unwrap_or(cfg.cost_bps_per_side)
        }));
    }
    let dir = results.join("manifests");
    let manifest = serde_json::json!({
        "strategy": cfg.strategy,
        "variant": cfg.variant,
        "inputs": {"sources": source_manifest, "files": files},
        "warnings": []
    });
    std::fs::create_dir_all(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    let path = dir.join(format!("{}__{}.json", cfg.strategy, cfg.variant));
    std::fs::write(&path, serde_json::to_string_pretty(&manifest).unwrap())
        .map_err(|e| format!("{}: {e}", path.display()))?;
    Ok(format!("{} symbols", symbol_names.len()))
}

fn csv_field(value: &str) -> String {
    if value.contains([',', '"', '\n', '\r']) {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_string()
    }
}

fn update_comparison_row(existing: &str, variant: &str, fill: &str, row: &str) -> String {
    let mut output = String::new();
    let mut found = false;
    for (index, line) in existing.lines().enumerate() {
        if index == 0 {
            output.push_str(line);
            output.push('\n');
            continue;
        }
        let mut fields = line.splitn(3, ',');
        let matches = fields.next() == Some(variant) && fields.next() == Some(fill);
        if matches {
            if !found {
                output.push_str(row);
                output.push('\n');
                found = true;
            }
        } else {
            output.push_str(line);
            output.push('\n');
        }
    }
    if !found {
        output.push_str(row);
        output.push('\n');
    }
    output
}

fn append_trial_record(cfg: &Config, study: &Study<'_>, results: &Path) -> Result<(), String> {
    let strategy_dir = results.join(&cfg.strategy);
    std::fs::create_dir_all(&strategy_dir)
        .map_err(|e| format!("{}: {e}", strategy_dir.display()))?;
    let registry_path = results.join("trial_registry.jsonl");
    let existing_registry = if registry_path.exists() {
        std::fs::read_to_string(&registry_path)
            .map_err(|e| format!("{}: {e}", registry_path.display()))?
    } else {
        String::new()
    };
    let mut registry = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&registry_path)
        .map_err(|e| format!("{}: {e}", registry_path.display()))?;
    let comparison_path = strategy_dir.join("variants_comparison.csv");
    let source_params: Vec<_> = cfg
        .source_list()
        .iter()
        .map(|source| {
            serde_json::json!({
                "name": source.name,
                "data_dir": source.data_dir,
                "data_suffix": source.data_suffix,
                "symbols": source.symbols,
                "cost_bps_per_side": source.cost_bps_per_side.unwrap_or(cfg.cost_bps_per_side)
            })
        })
        .collect();
    let params = serde_json::json!({
        "variant_overlay": cfg.variant_overlay,
        "sources": source_params,
        "rsi_len": cfg.rsi_len,
        "rsi_threshold": cfg.rsi_threshold,
        "gate": cfg.gate,
        "sma_window": cfg.sma_window,
        "exit_mode": cfg.exit_mode,
        "exit_rsi": cfg.exit_rsi,
        "cost_bps_per_side": cfg.cost_bps_per_side,
        "fills": cfg.fills,
        "gap_atr_multiple": cfg.gap_atr_multiple,
        "gap_volume_multiple": cfg.gap_volume_multiple,
        "gap_volume_window": cfg.gap_volume_window,
        "gap_volume_mode": cfg.gap_volume_mode,
        "gap_stop_mode": cfg.gap_stop_mode,
        "gap_reward_risk": cfg.gap_reward_risk,
        "gap_max_hold": cfg.gap_max_hold
    });
    for fill in cfg.fills_list() {
        let dev = study
            .row(&fill, "dev")
            .ok_or_else(|| format!("missing dev partition row for {fill}"))?;
        let test = study
            .row(&fill, "test")
            .ok_or_else(|| format!("missing test partition row for {fill}"))?;
        let record = serde_json::json!({
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "strategy": cfg.strategy,
            "variant": cfg.variant,
            "fill": fill,
            "parameters": params,
            "dev": {"trades": dev.get("trades"), "total_log": dev.get("total_log"), "skill": dev.get("skill"), "sharpe": dev.get("sharpe"), "pf": dev.get("pf"), "win_pct": dev.get("win_pct")},
            "test": {"trades": test.get("trades"), "total_log": test.get("total_log"), "skill": test.get("skill"), "sharpe": test.get("sharpe"), "pf": test.get("pf"), "win_pct": test.get("win_pct")}
        });
        let previous = existing_registry.lines().try_fold(
            None,
            |found: Option<serde_json::Value>, line| -> Result<_, String> {
                let previous: serde_json::Value = serde_json::from_str(line).map_err(|e| {
                    format!("{} contains invalid JSON: {e}", registry_path.display())
                })?;
                if previous["strategy"] == record["strategy"]
                    && previous["variant"] == record["variant"]
                    && previous["fill"] == record["fill"]
                    && previous["parameters"] == record["parameters"]
                {
                    Ok(Some(previous))
                } else {
                    Ok(found)
                }
            },
        )?;
        if previous.as_ref().map_or(true, |previous| {
            previous["dev"] != record["dev"] || previous["test"] != record["test"]
        }) {
            writeln!(registry, "{record}")
                .map_err(|e| format!("{}: {e}", registry_path.display()))?;
        }
        let row = format!(
            "{},{},{},{},{},{},{},{},{},{},{},{},{},{}",
            csv_field(&cfg.variant),
            csv_field(&fill),
            dev.get("trades"),
            dev.get("total_log"),
            dev.get("skill"),
            dev.get("sharpe"),
            dev.get("pf"),
            dev.get("win_pct"),
            test.get("trades"),
            test.get("total_log"),
            test.get("skill"),
            test.get("sharpe"),
            test.get("pf"),
            test.get("win_pct")
        );
        let existing_comparison = if comparison_path.exists() {
            std::fs::read_to_string(&comparison_path)
                .map_err(|e| format!("{}: {e}", comparison_path.display()))?
        } else {
            String::new()
        };
        let header = "variant,fill,dev_trades,dev_total_log,dev_skill,dev_sharpe,dev_pf,dev_win_pct,test_trades,test_total_log,test_skill,test_sharpe,test_pf,test_win_pct";
        let existing_comparison = if existing_comparison.is_empty() {
            format!("{header}\n")
        } else {
            existing_comparison
        };
        let updated = update_comparison_row(&existing_comparison, &cfg.variant, &fill, &row);
        std::fs::write(&comparison_path, updated)
            .map_err(|e| format!("{}: {e}", comparison_path.display()))?;
    }
    Ok(())
}

fn run(args: Args) -> Result<(), String> {
    if !study::supports_strategy(&args.strategy) {
        return Err(format!(
            "unknown strategy {:?}; implemented: {}, {}",
            args.strategy,
            study::NAME,
            study::S003_NAME
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
        profile: args.profile,
        debug: args.debug,
    };
    let cli = Cli {
        data_dir: args.data_dir.clone(),
        symbols: args.symbols.clone(),
    };
    let variants: Vec<Option<String>> = {
        let mut v = config::split_csv(args.variant.as_deref().unwrap_or(""));
        if args.variant.is_some() && !v.is_empty() && !v.iter().any(|name| name == "baseline") {
            v.insert(0, "baseline".into());
        }
        if v.is_empty() {
            vec![None]
        } else {
            v.into_iter().map(Some).collect()
        }
    };
    let mut symbol_overview: Option<Overview> = None;
    for variant in variants {
        let cfg = config::load_config(
            &args.strategy,
            args.config.as_deref(),
            &cli,
            variant.as_deref(),
        )?;
        let checked = check_inputs(&cfg, &results)?;
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
        if stages.contains(&"partition") {
            append_trial_record(&cfg, &study, &results)?;
        }
        if stages.contains(&"charts") {
            let next = study.symbol_overview()?;
            if let Some(all) = &mut symbol_overview {
                if all.dates != next.dates || all.split_row != next.split_row {
                    return Err("variant symbol-overview calendars do not match".into());
                }
                all.variants.extend(next.variants);
            } else {
                symbol_overview = Some(next);
            }
        }
    }
    if let Some(overview) = symbol_overview {
        let out = results.join(&args.strategy);
        std::fs::create_dir_all(&out).map_err(|e| format!("{}: {e}", out.display()))?;
        let svg = out.join("symbols_overview.svg");
        let csv = out.join("symbols_summary.csv");
        overview.write(&svg, &csv)?;
        println!("symbol overview: {}", svg.display());
    }
    if !args.check_data {
        timing::finish(args.profile, &results.join(&args.strategy))?;
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

#[cfg(test)]
mod tests {
    use super::update_comparison_row;

    #[test]
    fn comparison_refreshes_the_matching_variant_and_removes_duplicates() {
        let existing = concat!(
            "variant,fill,metrics\n",
            "baseline,close_to_open,old\n",
            "cost05,close_to_open,stale\n",
            "cost05,close_to_open,duplicate\n",
        );
        let updated = update_comparison_row(
            existing,
            "cost05",
            "close_to_open",
            "cost05,close_to_open,new",
        );
        assert_eq!(
            updated,
            concat!(
                "variant,fill,metrics\n",
                "baseline,close_to_open,old\n",
                "cost05,close_to_open,new\n",
            )
        );
    }

    #[test]
    fn comparison_appends_missing_rows() {
        let updated = update_comparison_row(
            "variant,fill,metrics\nbaseline,close_to_open,old\n",
            "cost05",
            "close_to_open",
            "cost05,close_to_open,new",
        );
        assert!(updated.ends_with("cost05,close_to_open,new\n"));
    }
}
