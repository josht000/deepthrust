//! PAL presenter pages for the Price Action Lab (ES/NQ futures, spec
//! prompt 5). The shared dashboard machinery - Book/Prov provenance,
//! verify, page model, SVG charts, presenter HTML - lives in
//! crate:: this module only knows which result tables exist
//! under results/pal/ and how to chart them. Generic study decks for
//! any results/<study>/ directory are built by
//! crate::study_pages / run_study_dash.

use std::path::{Path, PathBuf};

use crate::{emit, Book, Page, Prov, svg_bars, svg_ci, svg_lines, title_case};

/// Build all pages from result tables. Missing tables skip their pages.
pub fn build_pages(root: &Path) -> (Vec<Page>, Vec<Prov>) {
    let results = root.join("results/pal");
    let mut b = Book::new(results.clone());
    let mut pages = Vec::new();

    // ---- slide 1: title ----
    pages.push(Page {
        eyebrow: "PRICE ACTION LAB".into(),
        title: "The Five Prompts".into(),
        subtitle: "ES and NQ 1-minute price action, tested end to end".into(),
        body: format!(
            "<div class='cards'><div class='card'><div class='num'>10</div><div class='cap'>setup families</div></div>\
             <div class='card'><div class='num'>6</div><div class='cap'>setup charts</div></div>\
             <div class='card'><div class='num'>{}</div><div class='cap'>versions tested</div></div></div>",
            b.num("title", "versions", "funnel/versions.jsonl", "*", "__count__", "int")
        ),
        stats: vec![],
        conclusion: String::new(),
        verdict: None,
        slide: true,
    });

    // ---- slide 2: the funnel ----
    if results.join("funnel/stage_shares.csv").exists() {
        let nv = b.num("funnel", "versions", "funnel/versions.jsonl", "*", "__count__", "int");
        let nc = b.num("funnel", "candidates", "funnel/candidates.json", "*", "__count__", "int");
        let nw = b.num("funnel", "working", "funnel/working_setups.json", "*", "__count__", "int");
        pages.push(Page {
            eyebrow: "THE FUNNEL".into(),
            title: "Five Gates".into(),
            subtitle: "every version ran the same five stages on discovery data".into(),
            body: "<div class='cards'>\
                <div class='card'><div class='badge'>1</div><div class='cap'>at least 30 NQ trades</div></div>\
                <div class='card'><div class='badge'>2</div><div class='cap'>profitable after costs</div></div>\
                <div class='card'><div class='badge'>3</div><div class='cap'>NQ Sharpe 0.5+</div></div>\
                <div class='card'><div class='badge'>4</div><div class='cap'>beats 10,000 matched-random sets</div></div>\
                <div class='card'><div class='badge'>5</div><div class='cap'>holds on ES, Sharpe 0.3+</div></div></div>"
                .into(),
            stats: vec![
                (nv, "versions".into(), true),
                (nc, "final-test candidates".into(), true),
                (nw, "working setups".into(), true),
            ],
            conclusion: "candidates also made money in both eras".into(),
            verdict: None,
            slide: true,
        });
    }

    // ---- behavior findings ----
    if results.join("behavior_verdicts.csv").exists() {
        let rows = {
            let (_, r) = b.read_csv("behavior_verdicts.csv").map(|t| t.clone()).unwrap_or_default();
            r
        };
        // page 1: all tests CI chart
        let mut ci: Vec<(String, f64, f64, f64)> = Vec::new();
        for r in &rows {
            let name = r.first().cloned().unwrap_or_default();
            let d = r.get(1).and_then(|x| x.parse::<f64>().ok());
            let lo = r.get(2).and_then(|x| x.parse::<f64>().ok());
            let hi = r.get(3).and_then(|x| x.parse::<f64>().ok());
            if let (Some(d), Some(lo), Some(hi)) = (d, lo, hi) {
                ci.push((name.replace('_', " "), d, lo, hi));
            }
        }
        let n_sup = rows.iter().filter(|r| r.last().map(|v| v == "supported").unwrap_or(false)).count();
        let n_false = rows.iter().filter(|r| r.last().map(|v| v == "false").unwrap_or(false)).count();
        let n_split = rows.iter().filter(|r| r.last().map(|v| v == "split").unwrap_or(false)).count();
        pages.push(Page {
            eyebrow: "BEHAVIOR TESTS".into(),
            title: "Levels and Rules".into(),
            subtitle: "real touches against seeded random controls, NQ discovery".into(),
            body: svg_ci(&ci, 1500.0, 520.0),
            stats: vec![
                (format!("{n_sup}"), "supported".into(), true),
                (format!("{n_false}"), "false".into(), false),
            ],
            conclusion: format!("{n_split} tests split or inconclusive"),
            verdict: Some(if n_sup > n_false { "supported" } else { "split" }.into()),
            slide: false,
        });
        // page per test with verdicts (compact: chart of era1/era2/es diffs)
        for r in &rows {
            let name = r.first().cloned().unwrap_or_default();
            let verdict = r.last().cloned().unwrap_or_default();
            let items = vec![
                (
                    "NQ".to_string(),
                    b.num(&name, "nq_diff", "behavior_verdicts.csv", &name, "nq_diff", "f4")
                        .parse::<f64>()
                        .unwrap_or(0.0),
                ),
                (
                    "ES".to_string(),
                    b.num(&name, "es_diff", "behavior_verdicts.csv", &name, "es_diff", "f4")
                        .parse::<f64>()
                        .unwrap_or(0.0),
                ),
                (
                    "era 1".to_string(),
                    b.num(&name, "e1", "behavior_verdicts.csv", &name, "era1_diff", "f4")
                        .parse::<f64>()
                        .unwrap_or(0.0),
                ),
                (
                    "era 2".to_string(),
                    b.num(&name, "e2", "behavior_verdicts.csv", &name, "era2_diff", "f4")
                        .parse::<f64>()
                        .unwrap_or(0.0),
                ),
            ];
            pages.push(Page {
                eyebrow: "BEHAVIOR TESTS".into(),
                title: title_case(&name.replace('_', " ")),
                subtitle: "edge over matched controls in ATR units".into(),
                body: svg_bars(&items, 1400.0, 420.0),
                stats: vec![(
                    b.num(&name, "diff", "behavior_verdicts.csv", &name, "nq_diff", "f4"),
                    "NQ diff".into(),
                    true,
                )],
                conclusion: format!("verdict: {verdict}"),
                verdict: Some(verdict.clone()),
                slide: false,
            });
        }
    }

    // ---- funnel findings ----
    if results.join("funnel/setup_results.csv").exists() {
        let rows = {
            let (_, r) = b.read_csv("funnel/setup_results.csv").map(|t| t.clone()).unwrap_or_default();
            r
        };
        let nq_rows: Vec<&Vec<String>> = rows
            .iter()
            .filter(|r| r.get(2).map(|m| m == "nq").unwrap_or(false))
            .collect();
        let mut items: Vec<(String, f64)> = nq_rows
            .iter()
            .filter_map(|r| {
                let p = r.get(6).and_then(|x| x.parse::<f64>().ok())?; // sharpe
                Some((format!("{} {}", r[0], r[1]), p))
            })
            .collect();
        items.sort_by(|a, bb| bb.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        items.truncate(12);
        let works = nq_rows
            .iter()
            .filter(|r| r.last().map(|v| v == "true").unwrap_or(false))
            .count();
        pages.push(Page {
            eyebrow: "SETUPS AS A WHOLE".into(),
            title: "Working Setups".into(),
            subtitle: "equal-weight daily P&L Sharpe per setup chart, NQ discovery".into(),
            body: svg_bars(&items, 1500.0, 560.0),
            stats: vec![(format!("{works}"), "setups work on NQ".into(), true)],
            conclusion: "Sharpe 0.5 and profit in both eras counts as working".into(),
            verdict: Some(if works > 0 { "supported" } else { "split" }.into()),
            slide: false,
        });
    }

    // ---- stage funnel chart ----
    if results.join("funnel/stage_shares.csv").exists() {
        let all = b.read_csv("funnel/stage_shares.csv").and_then(|(_, r)| {
            r.iter().find(|r| r.first().map(|x| x == "all").unwrap_or(false)).cloned()
        });
        let items: Vec<(String, f64)> = match all {
            Some(r) => vec![
                ("30+ trades".into(), r.get(3).and_then(|x| x.parse().ok()).unwrap_or(0.0)),
                ("profitable".into(), r.get(4).and_then(|x| x.parse().ok()).unwrap_or(0.0)),
                ("sharpe 0.5".into(), r.get(5).and_then(|x| x.parse().ok()).unwrap_or(0.0)),
                ("beat random".into(), r.get(6).and_then(|x| x.parse().ok()).unwrap_or(0.0)),
                ("held on ES".into(), r.get(7).and_then(|x| x.parse().ok()).unwrap_or(0.0)),
            ],
            None => Vec::new(),
        };
        if !items.is_empty() {
            pages.push(Page {
                eyebrow: "THE FUNNEL".into(),
                title: "Share Surviving".into(),
                subtitle: "fraction of all kept versions clearing each stage".into(),
                body: svg_bars(&items, 1400.0, 420.0),
                stats: vec![(
                    b.num("surviving", "cands", "funnel/candidates.json", "*", "__count__", "int"),
                    "final-test candidates".into(),
                    true,
                )],
                conclusion: "stages compound: each keeps only the previous survivors".into(),
                verdict: None,
                slide: false,
            });
        }
    }

    // ---- final test findings (real or rehearsal dir) ----
    let final_dir = if results.join("final/versions.jsonl").exists() {
        Some("final")
    } else if results.join("final_rehearse/versions.jsonl").exists() {
        Some("final_rehearse")
    } else {
        None
    };
    if let Some(fd) = final_dir {
        let tag = if fd == "final" { "FINAL TEST" } else { "FINAL REHEARSAL" };
        // held-up shares
        for mkt in ["nq", "es"] {
            let share_all = b.num(
                "final", "all", &format!("{fd}/held_up.csv"), &format!("all_{mkt}"), "share", "f4");
            let share_cand = b.num(
                "final", "cand", &format!("{fd}/held_up.csv"), &format!("candidates_{mkt}"), "share", "f4");
            let items = vec![
                ("all versions".into(), share_all.parse::<f64>().unwrap_or(0.0)),
                ("candidates".into(), share_cand.parse::<f64>().unwrap_or(0.0)),
            ];
            pages.push(Page {
                eyebrow: tag.into(),
                title: format!("Held Up on {}", mkt.to_uppercase()),
                subtitle: "share of versions profitable with Sharpe 0.5+ on unseen data".into(),
                body: svg_bars(&items, 1400.0, 300.0),
                stats: vec![(share_cand, "candidate hold rate".into(), true)],
                conclusion: "selection was frozen before the seal opened".into(),
                verdict: Some("split".into()),
                slide: false,
            });
        }
        // named versions
        if results.join(format!("{fd}/named_final.csv")).exists() {
            let rows = {
                let (_, r) = b
                    .read_csv(&format!("{fd}/named_final.csv"))
                    .map(|t| t.clone())
                    .unwrap_or_default();
                r
            };
            let items: Vec<(String, f64)> = rows
                .iter()
                .filter_map(|r| {
                    let p = r.get(1).and_then(|x| x.parse::<f64>().ok())?;
                    Some((r[0].replace('_', " "), p))
                })
                .collect();
            pages.push(Page {
                eyebrow: tag.into(),
                title: "The Named Systems".into(),
                subtitle: "six textbook builds on the final window, NQ P&L".into(),
                body: svg_bars(&items, 1500.0, 420.0),
                stats: vec![],
                conclusion: "built as usually taught, lab defaults elsewhere".into(),
                verdict: None,
                slide: false,
            });
        }
    }

    // ---- equity curve of working setups (from daily pnl) ----
    if let Some(fd) = final_dir {
        if let Some(rows) = b.read_jsonl(&format!("{fd}/daily_pnl.jsonl")) {
            // top-5 kept versions by nq sharpe for an equity panel
            let mut named: Vec<(String, Vec<f64>, &'static str)> = Vec::new();
            let colors = ["#22d3ee", "#f59e0b", "#34d399", "#f87171", "#a78bfa"];
            let mut cand: Vec<(f64, String, Vec<f64>)> = Vec::new();
            for o in rows.iter().filter(|o| o["market"].as_str() == Some("nq")) {
                let pnl = o["pnl"].as_object();
                if let Some(p) = pnl {
                    let mut ds: Vec<(String, f64)> = p
                        .iter()
                        .filter_map(|(d, v)| v.as_f64().map(|x| (d.clone(), x)))
                        .collect();
                    ds.sort_by(|a, c| a.0.cmp(&c.0));
                    let total: f64 = ds.iter().map(|(_, x)| x).sum();
                    cand.push((total, o["name"].as_str().unwrap_or("?").into(), ds.iter().map(|(_, x)| *x).collect()));
                }
            }
            cand.sort_by(|a, c| c.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
            for (i, (_, name, v)) in cand.into_iter().take(5).enumerate() {
                named.push((name, v, colors[i % colors.len()]));
            }
            if !named.is_empty() {
                let dates: Vec<String> = (0..named[0].1.len()).map(|i| i.to_string()).collect();
                pages.push(Page {
                    eyebrow: "DAILY P&L".into(),
                    title: "Top Five Cumulative".into(),
                    subtitle: "best NQ totals over the window, one contract each".into(),
                    body: svg_lines(&named, &dates, 1500.0, 520.0),
                    stats: vec![],
                    conclusion: "daily marks from saved results".into(),
                    verdict: None,
                    slide: false,
                });
            }
        }
    }

    (pages, b.provs)
}

/// `pal dash` driver: PAL deck only. For a standard study directory use
/// `pal dash --study <name>` (`crate::run_study_dash`).
pub fn run_dash(root: &Path) -> Result<PathBuf, String> {
    let results_pal = root.join("results/pal");
    if !results_pal.exists() {
        return Err("no results/pal - run behavior/funnel/final-test first".into());
    }
    let (pages, provs) = build_pages(root);
    emit(&results_pal, &results_pal.join("dash"), pages, provs)
}

#[cfg(test)]
mod tests {
    #[test]
    fn facecam_rect_is_corner() {
        assert_eq!(crate::FACECAM, (1400.0, 712.0, 520.0, 368.0));
    }
}
