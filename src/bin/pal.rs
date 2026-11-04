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
    Funnel {
        /// cap the number of versions (smoke runs)
        #[arg(long)]
        limit: Option<usize>,
    },
    /// The sealed final test (requires PAL_STEP=final-test +
    /// PAL_UNLOCK_SEALED=1). --rehearse runs the same code on the last
    /// discovery month, price-shifted, seal closed.
    FinalTest {
        /// rehearsal on shifted discovery data (no unlock needed)
        #[arg(long)]
        rehearse: bool,
        /// cap the number of versions (smoke runs)
        #[arg(long)]
        limit: Option<usize>,
    },
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
        Cmd::Behavior => run_behavior(&store),
        Cmd::Funnel { limit } => run_funnel(&store, limit),
        Cmd::FinalTest { rehearse, limit } => run_final_test(&store, rehearse, limit),
        Cmd::Dash => todo("dash"),
    }
}

/// Build a DetCtx over discovery sessions; falls back to seeded
/// synthetic sessions when a market has no processed data yet.
fn disc_ctx(store: &Store, m: Market, label: &str) -> deepthrust::pal::detctx::DetCtx {
    use deepthrust::pal::audit::load_rolls;
    use deepthrust::pal::detctx::DetCtx;
    use deepthrust::pal::enginecheck::roll_boundaries;
    use deepthrust::pal::synth;
    use deepthrust::pal::windows::{DISCOVERY_END, DISCOVERY_START};
    let sessions = match store.read_1m(m, DISCOVERY_START, DISCOVERY_END, None) {
        Ok(s) if !s.is_empty() => s,
        _ => {
            println!("{label}: no {} data — using synthetic", m.symbol());
            synth::synth_sessions(
                chrono::NaiveDate::from_ymd_opt(2022, 1, 3).unwrap(),
                120,
                &format!("{label}-synth-{}", m.as_str()),
                4.0,
            )
        }
    };
    let rolls = load_rolls(&store.rolls_file(m)).unwrap_or_default();
    let rb = roll_boundaries(&sessions, &rolls);
    DetCtx::build(&sessions, &rb)
}

fn run_funnel(store: &Store, limit: Option<usize>) -> ExitCode {
    use deepthrust::pal::enginecheck::news_set;
    use deepthrust::pal::funnel;
    let news = news_set(&store.read_news().unwrap_or_default());
    let mut nq = disc_ctx(store, Market::Nq, "funnel");
    let mut es = disc_ctx(store, Market::Es, "funnel");
    println!("funnel: {} news days", news.len());
    let out = funnel::run_funnel(&mut nq, &mut es, &news, limit);
    let dir = store.results_dir().join("funnel");
    match funnel::write_outputs(&out, &dir) {
        Ok(()) => {
            println!("funnel: wrote {} ({} versions, {} candidates)",
                     dir.display(), out.rows.len(), out.candidates.len());
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("funnel write: {e}");
            ExitCode::FAILURE
        }
    }
}

fn run_final_test(store: &Store, rehearse: bool, limit: Option<usize>) -> ExitCode {
    use deepthrust::pal::enginecheck::news_set;
    use deepthrust::pal::finaltest;
    let funnel_dir = store.results_dir().join("funnel");
    let out_dir = finaltest::output_dir(store, rehearse);

    // 1. hash preflight on the saved funnel selections
    let versions = match finaltest::verify_and_load(&funnel_dir) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("final-test preflight: {e}");
            return ExitCode::FAILURE;
        }
    };
    let funnel_rows = match finaltest::load_funnel_rows(&funnel_dir) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("final-test preflight: {e}");
            return ExitCode::FAILURE;
        }
    };
    println!(
        "final-test: {} kept versions; {} candidates",
        versions.len(),
        funnel_rows.iter().filter(|r| r.candidate).count()
    );

    // 2. run-once rule
    if !rehearse && finaltest::dir_has_results(&out_dir) {
        eprintln!(
            "final-test: {} already has results — refusing to rerun",
            out_dir.display()
        );
        return ExitCode::FAILURE;
    }

    // 3. unlock (rehearse skips it — the seal stays closed)
    let unlock = if rehearse {
        None
    } else {
        match finaltest::require_final_unlock() {
            Ok(u) => Some(u),
            Err(e) => {
                eprintln!("final-test: {e}");
                return ExitCode::FAILURE;
            }
        }
    };

    // 4. windows per market
    let news = news_set(&store.read_news().unwrap_or_default());
    let mut ctxs = Vec::new();
    let mut dates = Vec::new();
    for m in Market::ALL {
        let sessions = if rehearse {
            finaltest::rehearsal_sessions(store, m)
        } else {
            match finaltest::read_window(store, m, unlock.as_ref().unwrap()) {
                Ok(s) if !s.is_empty() => s,
                Ok(_) => {
                    eprintln!("final-test: no {} sessions — run ingest first", m.symbol());
                    return ExitCode::FAILURE;
                }
                Err(e) => {
                    eprintln!("final-test read: {e}");
                    return ExitCode::FAILURE;
                }
            }
        };
        let rolls = deepthrust::pal::audit::load_rolls(&store.rolls_file(m))
            .unwrap_or_default();
        let (ctx, ds) = finaltest::final_ctx(&sessions, &rolls);
        println!(
            "final-test: {} {} sessions, {} in window",
            m.symbol(),
            ctx.session_dates.len(),
            ds.len()
        );
        ctxs.push(ctx);
        dates.push(ds);
    }
    // Market::ALL = [Es, Nq]
    let mut es = ctxs.remove(0);
    let mut nq = ctxs.remove(0);
    let es_dates = dates.remove(0);
    let nq_dates = dates.remove(0);

    let out = finaltest::run_final(
        &mut nq,
        &mut es,
        &nq_dates,
        &es_dates,
        &versions,
        &funnel_rows,
        &news,
        limit,
    );
    match finaltest::write_final_outputs(&out, &funnel_rows, &versions, &out_dir) {
        Ok(()) => {
            println!(
                "final-test{}: wrote {} ({} versions, nq held {} es held {})",
                if rehearse { " rehearsal" } else { "" },
                out_dir.display(),
                out.rows.len(),
                out.rows.iter().filter(|r| r.held_nq).count(),
                out.rows.iter().filter(|r| r.held_es).count(),
            );
            // provenance: snapshot the sealed manifest + access log
            if !rehearse {
                for src in [
                    store.sealed_dir().join("manifest.sha256"),
                    store.sealed_dir().join("access_log.jsonl"),
                ] {
                    if src.exists() {
                        let dst = out_dir.join(src.file_name().unwrap());
                        let _ = std::fs::copy(&src, &dst);
                    }
                }
            }
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("final-test write: {e}");
            ExitCode::FAILURE
        }
    }
}

fn run_behavior(store: &Store) -> ExitCode {
    use deepthrust::pal::behavior::{self, Behavior, TestResult};
    use deepthrust::pal::detctx::DetCtx;
    use deepthrust::pal::enginecheck::roll_boundaries;
    use deepthrust::pal::audit::load_rolls;
    use deepthrust::pal::synth;
    use deepthrust::pal::types::ChartId;
    use deepthrust::pal::windows::{DISCOVERY_END, DISCOVERY_START};

    type Collector = fn(&mut Behavior, ChartId, bool) -> Vec<behavior::Measure>;
    let tests: Vec<(&str, Collector, bool)> = vec![
        ("levels_hold_vs_random", |b, c, d| b.level_touch_measures(c, d), true),
        ("flip_hold", |b, c, d| b.flip_hold(c, d), true),
        ("sweep_vs_close", |b, c, d| b.sweep_vs_close(c, d), true),
        ("fvg_hold", |b, c, d| b.fvg_hold(c, d), true),
        ("zone_first_vs_later", |b, c, d| b.zone_first_vs_later(c, d), true),
        ("pullback_depth", |b, c, d| b.pullback_depth(c, d), true),
        ("candles_at_levels", |b, c, d| b.candles_at_levels(c, d), true),
        ("trendline_touch", |b, c, d| b.trendline_touch(c, d), true),
    ];
    let charts = [ChartId::M5, ChartId::M15, ChartId::H1, ChartId::H4, ChartId::D1];

    // build contexts per market (NQ primary, ES replication)
    let mut diffs: std::collections::HashMap<(String, String), f64> = Default::default();
    let mut all_rows: Vec<(String, Vec<TestResult>)> = Vec::new();
    for m in Market::ALL {
        let sessions = match store.read_1m(m, DISCOVERY_START, DISCOVERY_END, None) {
            Ok(s) if !s.is_empty() => s,
            _ => {
                println!("behavior: no {} data — using synthetic", m.symbol());
                synth::synth_sessions(
                    chrono::NaiveDate::from_ymd_opt(2022, 1, 3).unwrap(),
                    120,
                    &format!("behavior-synth-{}", m.as_str()),
                    4.0,
                )
            }
        };
        let rolls = load_rolls(&store.rolls_file(m)).unwrap_or_default();
        let rb = roll_boundaries(&sessions, &rolls);
        let mut ctx = DetCtx::build(&sessions, &rb);
        let session_dates = ctx.session_dates.clone();
        for (name, col, claimed_pos) in &tests {
            let mut measures = Vec::new();
            for &c in &charts {
                for dir in [true, false] {
                    let mut b = Behavior { ctx: &mut ctx };
                    measures.extend(col(&mut b, c, dir));
                }
            }
            let results = behavior::summarize(name, &measures, &session_dates, *claimed_pos, 0.0);
            if let Some(pooled) = results.first() {
                diffs.insert((m.symbol().to_string(), name.to_string()), pooled.diff);
            }
            all_rows.push((format!("{}:{}", m.symbol(), name), results));
        }
        println!("behavior: {} done", m.symbol());
    }

    // write tables + verdicts
    let out = store.results_dir();
    let _ = std::fs::create_dir_all(&out);
    let mut csv = String::from("market_test,chart,n_real,n_ctrl,real_mean,ctrl_mean,diff,ci_lo,ci_hi,era1_diff,era2_diff\n");
    for (label, rows) in &all_rows {
        for r in rows {
            csv.push_str(&format!(
                "{},{},{},{},{:.4},{:.4},{:.4},{:.4},{:.4},{:.4},{:.4}\n",
                label, r.chart, r.n_real, r.n_ctrl, r.real_mean, r.ctrl_mean,
                r.diff, r.ci_lo, r.ci_hi, r.era1_diff, r.era2_diff
            ));
        }
    }
    let table_path = out.join("behavior_tables.csv");
    let _ = std::fs::write(&table_path, csv);

    let mut vc = String::from("test,nq_diff,nq_ci_lo,nq_ci_hi,es_diff,era1_diff,era2_diff,verdict\n");
    for (name, _, claimed) in &tests {
        let nq = all_rows.iter().find(|(l, _)| l == &format!("NQ:{name}"));
        let pooled = nq.and_then(|(_, r)| r.first());
        let (d, lo, hi, e1, e2) = pooled
            .map(|p| (p.diff, p.ci_lo, p.ci_hi, p.era1_diff, p.era2_diff))
            .unwrap_or((f64::NAN, f64::NAN, f64::NAN, f64::NAN, f64::NAN));
        let es = diffs.get(&("ES".to_string(), name.to_string())).copied().unwrap_or(f64::NAN);
        let v = behavior::verdict(d, lo, hi, *claimed, es, e1, e2);
        vc.push_str(&format!(
            "{},{:.4},{:.4},{:.4},{:.4},{:.4},{:.4},{}\n",
            name, d, lo, hi, es, e1, e2, v.as_str()
        ));
    }
    let verdicts_path = out.join("behavior_verdicts.csv");
    let _ = std::fs::write(&verdicts_path, &vc);
    println!("wrote {} and {}", table_path.display(), verdicts_path.display());
    println!("\nverdicts:\n{vc}");
    ExitCode::SUCCESS
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
