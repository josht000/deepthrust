//! Price Action Lab CLI.
//!   cargo run --release --bin pal -- <subcommand>
//! Spec: docs/price_action_lab_five_prompts.md

use clap::{Parser, Subcommand};
use std::process::ExitCode;

use deepthrust::pal::guard::{self, Unlock};
use deepthrust::pal::ingest;
use deepthrust::pal::store::Store;
use deepthrust::pal::types::Market;
use deepthrust::pal::windows::{DISCOVERY_START, SEAL_END_EXPECTED, SEAL_START};

#[derive(Parser)]
#[command(name = "pal", about = "Price Action Lab (ES/NQ futures research)")]
struct Args {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Exercise the seal guard: attempt a sealed read with and without
    /// the unlock env vars, and print the result.
    GuardTest,
    /// Ingest raw files from data/futures/raw/ -> processed + sealed
    /// split, sha256 manifest, audit report.
    Ingest {
        #[arg(long)]
        market: Option<String>,
    },
    /// Build all chart timeframes and ATR caches (checks only).
    Charts,
    /// Emit rulebook.md and rulebook_spec.json.
    Rulebook,
    /// Engine unit + sanity checks (random entries, cheat test).
    EngineCheck,
    /// Run the behavior tests (NQ discovery, ES replication).
    Behavior,
    /// Run the funnel over the version registry.
    Funnel,
    /// The sealed final test (requires PAL_STEP + PAL_UNLOCK_SEALED=1).
    FinalTest,
    /// Build the presenter dashboard with provenance checks.
    Dash,
}

fn main() -> ExitCode {
    let args = Args::parse();
    let store = Store::cwd();
    match args.cmd {
        Cmd::GuardTest => guard_test(&store),
        Cmd::Ingest { market } => run_ingest(&store, market),
        Cmd::Charts => todo("charts"),
        Cmd::Rulebook => match deepthrust::pal::rulebook::write_rulebook(std::path::Path::new(".")) {
            Ok(()) => {
                println!("wrote rulebook.md + rulebook_spec.json");
                ExitCode::SUCCESS
            }
            Err(e) => {
                eprintln!("rulebook: {e}");
                ExitCode::FAILURE
            }
        },
        Cmd::EngineCheck => run_engine_check(&store),
        Cmd::Behavior => todo("behavior"),
        Cmd::Funnel => todo("funnel"),
        Cmd::FinalTest => todo("final-test"),
        Cmd::Dash => todo("dash"),
    }
}

fn run_engine_check(store: &Store) -> ExitCode {
    use deepthrust::pal::detctx::DetCtx;
    use deepthrust::pal::enginecheck;
    use deepthrust::pal::synth;
    use deepthrust::pal::windows::{DISCOVERY_END, DISCOVERY_START};

    // real discovery sessions if present, else synthetic plumbing data
    let sessions = match store.read_1m(Market::Nq, DISCOVERY_START, DISCOVERY_END, None) {
        Ok(s) if !s.is_empty() => {
            println!("engine-check: using NQ discovery ({} sessions)", s.len());
            s
        }
        _ => {
            println!("engine-check: no data — using 120 synthetic sessions");
            synth::synth_sessions(
                chrono::NaiveDate::from_ymd_opt(2022, 1, 3).unwrap(),
                120,
                "enginecheck-synth",
                4.0,
            )
        }
    };
    let mut ctx = DetCtx::build(&sessions, &Default::default());

    let r = enginecheck::random_entries(&mut ctx, 20_000);
    println!(
        "random entries: n={} mean=${:.2}/trade total=${:.0} win%={:.1} (expect: lose a little; lab ref -$9.80)",
        r.n, r.mean_pnl, r.total_pnl, 100.0 * r.wins as f64 / r.n.max(1) as f64
    );
    let sane = r.mean_pnl < 0.0 && r.mean_pnl > -50.0;

    let c = enginecheck::cheat_probe(&mut ctx, 2_000);
    println!(
        "cheat probe (peeks 60m): n={} mean=${:.2}/trade total=${:.0} win%={:.1} (expect: a lot of money)",
        c.n, c.mean_pnl, c.total_pnl, 100.0 * c.wins as f64 / c.n.max(1) as f64
    );
    let cheat_ok = c.mean_pnl > 20.0;

    if sane && cheat_ok {
        println!("engine-check: PASS");
        ExitCode::SUCCESS
    } else {
        println!("engine-check: FAIL");
        ExitCode::FAILURE
    }
}

fn todo(what: &str) -> ExitCode {
    eprintln!("pal {what}: not implemented yet");
    ExitCode::FAILURE
}

fn run_ingest(store: &Store, market: Option<String>) -> ExitCode {
    let markets: Vec<Market> = match market.as_deref() {
        Some(s) => match Market::parse(s) {
            Some(m) => vec![m],
            None => {
                eprintln!("unknown market {s:?} (want es|nq)");
                return ExitCode::FAILURE;
            }
        },
        None => Market::ALL.to_vec(),
    };
    for m in markets {
        match ingest::ingest_market(store, m, true) {
            Ok(r) => {
                println!(
                    "ingest {}: {} rows open, {} rows sealed, sessions {}..{} | {} raw files",
                    r.market.symbol(),
                    r.rows_written_open,
                    r.rows_written_sealed,
                    r.first_session.map(|d| d.to_string()).unwrap_or_default(),
                    r.last_session.map(|d| d.to_string()).unwrap_or_default(),
                    r.files.len(),
                );
                println!(
                    "  audit: {} dup ts, {} gaps>1m, {} gaps>60m, {} ohlc viol, {} zero-vol, {} halt dropped",
                    r.audit.duplicate_ts,
                    r.audit.gaps_gt_1m,
                    r.audit.gaps_gt_60m,
                    r.audit.ohlc_violations,
                    r.audit.zero_volume,
                    r.audit.halt_dropped,
                );
            }
            Err(e) => {
                eprintln!("ingest {}: {e}", m.symbol());
                return ExitCode::FAILURE;
            }
        }
    }
    ExitCode::SUCCESS
}

fn guard_test(store: &Store) -> ExitCode {
    let mut ok = true;
    // 1) sealed window read without unlock must fail
    let denied = store
        .read_1m(Market::Es, SEAL_START, SEAL_END_EXPECTED, None)
        .is_err();
    println!("sealed read without unlock denied: {denied}");
    ok &= denied;

    // 2) unsealed read without unlock must pass
    let open_ok = store
        .read_1m(Market::Es, DISCOVERY_START, SEAL_START - chrono::Duration::days(1), None)
        .is_ok();
    println!("discovery read without unlock allowed: {open_ok}");
    ok &= open_ok;

    // 3) env-unlocked read must pass AND log
    let unlock = Unlock::from_env();
    match &unlock {
        Some(u) => println!("unlock present for step: {}", u.step),
        None => println!("no unlock env (PAL_STEP + PAL_UNLOCK_SEALED=1) — skipping unlock path"),
    }
    if let Some(u) = &unlock {
        let before = std::fs::read_to_string(guard::access_log_path(&store.sealed_dir()))
            .unwrap_or_default()
            .lines()
            .count();
        let r = store.read_1m(Market::Es, SEAL_START, SEAL_END_EXPECTED, Some(u));
        ok &= r.is_ok();
        let after = std::fs::read_to_string(guard::access_log_path(&store.sealed_dir()))
            .unwrap_or_default()
            .lines()
            .count();
        let logged = after > before;
        println!("unlocked read ok: {} | access log appended: {logged}", r.is_ok());
        ok &= logged;
    }
    if ok {
        println!("guard-test: PASS");
        ExitCode::SUCCESS
    } else {
        println!("guard-test: FAIL");
        ExitCode::FAILURE
    }
}
