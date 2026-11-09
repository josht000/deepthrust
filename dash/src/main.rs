//! `dash` — presenter dashboard builder for DeepThrust results.
//!
//! Bare `dash` builds the PAL deck (results/pal/dash/).
//! `dash --study <name>` builds the generic deck for results/<name>/.
//! `dash --all` builds every study directory that has partition_dev.csv.
//!
//! `pal dash` forwards to this binary; run it directly or via pal.

use std::path::Path;
use std::process::ExitCode;

use deepthrust_dash::{self, run_study_dash};

fn usage() {
    eprintln!("usage: dash [--study <name>] [--all]");
    eprintln!("  (no args)      PAL deck -> results/pal/dash/");
    eprintln!("  --study <name> generic deck -> results/<name>/dash/");
    eprintln!("  --all          every results/<dir>/ with partition_dev.csv");
}

fn report(r: Result<std::path::PathBuf, String>, tag: &str) -> ExitCode {
    match r {
        Ok(p) => {
            println!("dash: {tag} -> {}", p.display());
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("dash: {tag}: {e}");
            ExitCode::FAILURE
        }
    }
}

fn main() -> ExitCode {
    let mut study: Option<String> = None;
    let mut all = false;
    let mut it = std::env::args().skip(1);
    while let Some(a) = it.next() {
        match a.as_str() {
            "--study" => match it.next() {
                Some(s) => study = Some(s),
                None => {
                    eprintln!("dash: --study needs a name");
                    return ExitCode::FAILURE;
                }
            },
            "--all" => all = true,
            "-h" | "--help" => {
                usage();
                return ExitCode::SUCCESS;
            }
            other => {
                eprintln!("dash: unknown arg {other}");
                usage();
                return ExitCode::FAILURE;
            }
        }
    }
    let root = Path::new(".");

    if let Some(name) = study {
        return report(run_study_dash(root, &name), &name);
    }
    if all {
        let results = root.join("results");
        let mut names: Vec<String> = std::fs::read_dir(&results)
            .map(|rd| {
                rd.filter_map(|e| e.ok())
                    .filter(|e| e.path().join("partition_dev.csv").exists())
                    .filter_map(|e| e.file_name().to_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();
        names.sort();
        if names.is_empty() {
            eprintln!("dash --all: no study directories with partition_dev.csv");
            return ExitCode::FAILURE;
        }
        let mut failed = 0;
        for name in &names {
            if report(run_study_dash(root, name), name) == ExitCode::FAILURE {
                failed += 1;
            }
        }
        return if failed == 0 {
            ExitCode::SUCCESS
        } else {
            eprintln!("dash --all: {failed} study(s) failed");
            ExitCode::FAILURE
        };
    }
    // bare: PAL deck
    report(deepthrust_dash::pal::run_dash(root), "pal")
}
