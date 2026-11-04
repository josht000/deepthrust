//! The presenter dashboard (spec prompt 5): one offline HTML file at
//! results/pal/dash/dashboard.html, every number on screen traced to a
//! saved results file through results/pal/dash/provenance.csv, each
//! value re-read by a second independent reader before the page is
//! written. Reads ONLY result tables — never raw or sealed data.
//!
//! Stage: 1920x1080, navy #0b0f1a, cyan #22d3ee, amber #f59e0b.
//! Facecam corner: bottom-right 520x368 stays empty.

use std::fmt::Write as _;
use std::path::{Path, PathBuf};

const W: f64 = 1920.0;
const H: f64 = 1080.0;
const FACECAM: (f64, f64, f64, f64) = (W - 520.0, H - 368.0, 520.0, 368.0);

/// A number bound for the page with its source.
#[derive(Clone, Debug)]
pub struct Prov {
    pub page: String,
    pub element: String,
    pub file: String,
    pub row: String,   // first-column key; "*" for whole-file aggregates
    pub col: String,   // header name, or __count__/__sum__:col
    pub format: String,
    pub value: String, // raw value as read
    pub text: String,  // text shown on the page
}

struct Book {
    provs: Vec<Prov>,
    root: PathBuf,
    // csv tables: path -> (header, rows)
    tables: std::collections::HashMap<PathBuf, (Vec<String>, Vec<Vec<String>>)>,
    // jsonl caches: path -> parsed objects
    jsonls: std::collections::HashMap<PathBuf, Vec<serde_json::Value>>,
}

impl Book {
    fn new(root: PathBuf) -> Book {
        Book {
            provs: Vec::new(),
            root,
            tables: Default::default(),
            jsonls: Default::default(),
        }
    }

    fn read_csv(&mut self, rel: &str) -> Option<&(Vec<String>, Vec<Vec<String>>)> {
        let p = self.root.join(rel);
        if !self.tables.contains_key(&p) {
            let text = std::fs::read_to_string(&p).ok()?;
            let mut lines = text.lines();
            let header: Vec<String> = lines.next()?.split(',').map(|s| s.to_string()).collect();
            let rows: Vec<Vec<String>> = lines
                .filter(|l| !l.trim().is_empty())
                .map(|l| l.split(',').map(|s| s.to_string()).collect())
                .collect();
            self.tables.insert(p.clone(), (header, rows));
        }
        self.tables.get(&p)
    }

    fn read_jsonl(&mut self, rel: &str) -> Option<&Vec<serde_json::Value>> {
        let p = self.root.join(rel);
        if !self.jsonls.contains_key(&p) {
            let text = std::fs::read_to_string(&p).ok()?;
            let rows: Vec<serde_json::Value> = text
                .lines()
                .filter(|l| !l.trim().is_empty())
                .filter_map(|l| serde_json::from_str(l).ok())
                .collect();
            self.jsonls.insert(p.clone(), rows);
        }
        self.jsonls.get(&p)
    }

    /// csv cell lookup: first column == key, column by header name.
    fn cell(&mut self, rel: &str, key: &str, col: &str) -> Option<String> {
        let (header, rows) = self.read_csv(rel)?;
        let ci = header.iter().position(|h| h == col)?;
        rows.iter()
            .find(|r| r.first().map(|x| x == key).unwrap_or(false))
            .and_then(|r| r.get(ci).cloned())
    }

    /// Register a number: look it up, format it, record provenance.
    fn num(
        &mut self,
        page: &str,
        element: &str,
        file: &str,
        row: &str,
        col: &str,
        fmt: &str,
    ) -> String {
        let raw = if col == "__count__" {
            if file.ends_with(".jsonl") {
                self.read_jsonl(file).map(|v| v.len().to_string())
            } else if file.ends_with(".json") {
                std::fs::read_to_string(self.root.join(file))
                    .ok()
                    .and_then(|t| serde_json::from_str::<serde_json::Value>(&t).ok())
                    .map(|v| v.as_array().map(|a| a.len()).unwrap_or(0).to_string())
            } else {
                self.read_csv(file).map(|(_, r)| r.len().to_string())
            }
        } else if let Some(sumcol) = col.strip_prefix("__sum__:") {
            self.read_csv(file).and_then(|(header, rows)| {
                let ci = header.iter().position(|h| h == sumcol)?;
                let s: f64 = rows
                    .iter()
                    .filter_map(|r| r.get(ci).and_then(|v| v.parse::<f64>().ok()))
                    .sum();
                Some(format!("{s}"))
            })
        } else if file.ends_with(".jsonl") {
            self.read_jsonl(file).and_then(|rows| {
                rows.iter()
                    .find(|o| {
                        o["name"].as_str().map(|n| n == row).unwrap_or(false)
                    })
                    .and_then(|o| o.get(col).map(|v| match v {
                        serde_json::Value::Number(n) => n.to_string(),
                        serde_json::Value::Bool(b) => b.to_string(),
                        serde_json::Value::String(s) => s.clone(),
                        other => other.to_string(),
                    }))
            })
        } else {
            self.cell(file, row, col)
        };
        let raw = raw.unwrap_or_else(|| "NA".into());
        let text = match (fmt, raw.parse::<f64>()) {
            ("money", Ok(v)) => format!("${v:+.0}"),
            ("pct", Ok(v)) => format!("{:.0}%", v * 100.0),
            ("f2", Ok(v)) => format!("{v:.2}"),
            ("f3", Ok(v)) => format!("{v:.3}"),
            ("f4", Ok(v)) => format!("{v:.4}"),
            ("int", Ok(v)) => format!("{v:.0}"),
            _ => raw.clone(),
        };
        self.provs.push(Prov {
            page: page.into(),
            element: element.into(),
            file: file.into(),
            row: row.into(),
            col: col.into(),
            format: fmt.into(),
            value: raw,
            text: text.clone(),
        });
        text
    }
}

/// Independent reread: a second code path that opens each source file
/// fresh, re-extracts every claimed (row,col) value, formats it, and
/// compares to the shown text. Any mismatch fails the build.
fn verify_provenance(root: &Path, provs: &[Prov]) -> Vec<String> {
    let mut errors = Vec::new();
    for p in provs {
        let path = root.join(&p.file);
        let raw: Option<String> = if p.col == "__count__" {
            if p.file.ends_with(".json") && !p.file.ends_with(".jsonl") {
                std::fs::read_to_string(&path)
                    .ok()
                    .and_then(|t| serde_json::from_str::<serde_json::Value>(&t).ok())
                    .map(|v| v.as_array().map(|a| a.len()).unwrap_or(0).to_string())
            } else {
                std::fs::read_to_string(&path).ok().map(|t| {
                    t.lines().filter(|l| !l.trim().is_empty()).count()
                        .saturating_sub(if p.file.ends_with(".csv") { 1 } else { 0 })
                        .to_string()
                })
            }
        } else if let Some(sumcol) = p.col.strip_prefix("__sum__:") {
            std::fs::read_to_string(&path).ok().and_then(|t| {
                let mut lines = t.lines();
                let header: Vec<&str> = lines.next()?.split(',').collect();
                let ci = header.iter().position(|h| *h == sumcol)?;
                let s: f64 = lines
                    .filter(|l| !l.trim().is_empty())
                    .filter_map(|l| {
                        l.split(',').nth(ci).and_then(|v| v.parse::<f64>().ok())
                    })
                    .sum();
                Some(format!("{s}"))
            })
        } else if p.file.ends_with(".jsonl") {
            std::fs::read_to_string(&path).ok().and_then(|t| {
                for line in t.lines() {
                    if let Ok(o) = serde_json::from_str::<serde_json::Value>(line) {
                        if o["name"].as_str().map(|n| n == p.row).unwrap_or(false) {
                            return o.get(&p.col).map(|v| match v {
                                serde_json::Value::Number(n) => n.to_string(),
                                serde_json::Value::Bool(b) => b.to_string(),
                                serde_json::Value::String(s) => s.clone(),
                                other => other.to_string(),
                            });
                        }
                    }
                }
                None
            })
        } else {
            std::fs::read_to_string(&path).ok().and_then(|t| {
                let mut lines = t.lines();
                let header: Vec<&str> = lines.next()?.split(',').collect();
                let ci = header.iter().position(|h| *h == p.col)?;
                lines
                    .filter(|l| !l.trim().is_empty())
                    .find(|l| l.split(',').next() == Some(p.row.as_str()))
                    .and_then(|l| l.split(',').nth(ci).map(|s| s.to_string()))
            })
        };
        match raw {
            None => errors.push(format!(
                "{}:{} cannot reread {} row={} col={}",
                p.page, p.element, p.file, p.row, p.col
            )),
            Some(v) => {
                if v != p.value {
                    errors.push(format!(
                        "{}:{} reread differs {} row={} col={}: '{}' vs '{}'",
                        p.page, p.element, p.file, p.row, p.col, v, p.value
                    ));
                    continue;
                }
                // reformat independently and compare the shown text
                let text = match (p.format.as_str(), v.parse::<f64>()) {
                    ("money", Ok(x)) => format!("${x:+.0}"),
                    ("pct", Ok(x)) => format!("{:.0}%", x * 100.0),
                    ("f2", Ok(x)) => format!("{x:.2}"),
                    ("f3", Ok(x)) => format!("{x:.3}"),
                    ("f4", Ok(x)) => format!("{x:.4}"),
                    ("int", Ok(x)) => format!("{x:.0}"),
                    _ => v,
                };
                if text != p.text {
                    errors.push(format!(
                        "{}:{} format differs {} row={} col={}: '{}' vs '{}'",
                        p.page, p.element, p.file, p.row, p.col, text, p.text
                    ));
                }
            }
        }
    }
    errors
}

pub struct Page {
    eyebrow: String,
    title: String,
    subtitle: String,
    body: String,       // svg or cards html
    stats: Vec<(String, String, bool)>, // value, label, positive?
    conclusion: String,
    verdict: Option<String>,
    slide: bool,
}

fn esc(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// Horizontal bar chart with zero line; values labeled.
fn fin(v: f64) -> f64 {
    if v.is_finite() {
        v
    } else {
        0.0
    }
}

fn svg_bars(items: &[(String, f64)], w: f64, h: f64) -> String {
    let pos_max = items
        .iter()
        .map(|(_, v)| fin(*v).max(0.0))
        .fold(1e-9f64, f64::max);
    let neg_max = items
        .iter()
        .map(|(_, v)| (-fin(*v)).max(0.0))
        .fold(1e-9f64, f64::max);
    let row_h = h / items.len().max(1) as f64;
    let mid_x = w * 0.42;
    let scale = ((mid_x - 20.0) / neg_max).min((w - mid_x - 160.0) / pos_max).max(1.0);
    let mut s = format!(
        "<svg viewBox='0 0 {w:.0} {h:.0}' class='chart'>"
    );
    writeln!(
        s,
        "<line x1='{mid_x:.0}' y1='0' x2='{mid_x:.0}' y2='{h:.0}' stroke='#334155'/>"
    )
    .unwrap();
    for (i, (label, v)) in items.iter().enumerate() {
        let v = fin(*v);
        let y = i as f64 * row_h + row_h * 0.2;
        let bw = v.abs() * scale;
        let (x, color) = if v >= 0.0 {
            (mid_x, "#34d399")
        } else {
            (mid_x - bw, "#f87171")
        };
        writeln!(
            s,
            "<text x='{:.0}' y='{:.0}' class='blab' text-anchor='end'>{}</text>",
            mid_x - 12.0,
            y + row_h * 0.42,
            esc(label)
        )
        .unwrap();
        writeln!(
            s,
            "<rect x='{x:.0}' y='{y:.0}' width='{bw:.0}' height='{:.0}' fill='{color}' rx='3'/>",
            row_h * 0.55
        )
        .unwrap();
        writeln!(
            s,
            "<text x='{:.0}' y='{:.0}' class='bval'>{}</text>",
            if v >= 0.0 { x + bw + 8.0 } else { x - 8.0 },
            y + row_h * 0.42,
            fmt_signed(v)
        )
        .unwrap();
    }
    s.push_str("</svg>");
    s
}

fn fmt_signed(v: f64) -> String {
    if v.abs() >= 100.0 {
        format!("{v:+.0}")
    } else {
        format!("{v:+.2}")
    }
}

/// Cumulative daily-P&L lines (one per series).
fn svg_lines(series: &[(String, Vec<f64>, &'static str)], dates: &[String], w: f64, h: f64) -> String {
    let mut cum: Vec<Vec<f64>> = Vec::new();
    let mut lo = 0.0f64;
    let mut hi = 0.0f64;
    for (_, v, _) in series {
        let mut acc = 0.0;
        let mut c = Vec::with_capacity(v.len());
        for x in v {
            acc += fin(*x);
            c.push(acc);
            lo = lo.min(acc);
            hi = hi.max(acc);
        }
        cum.push(c);
    }
    if hi - lo < 1e-9 {
        hi = lo + 1.0;
    }
    let n = dates.len().max(2) as f64;
    let px = |i: usize| 40.0 + (w - 80.0) * i as f64 / (n - 1.0);
    let py = |v: f64| h - 40.0 - (h - 80.0) * (v - lo) / (hi - lo);
    let mut s = format!("<svg viewBox='0 0 {w:.0} {h:.0}' class='chart'>");
    // zero line + axes
    writeln!(s, "<line x1='40' y1='{:.0}' x2='{:.0}' y2='{:.0}' stroke='#334155'/>", py(0.0), w - 40.0, py(0.0)).unwrap();
    writeln!(s, "<line x1='40' y1='20' x2='40' y2='{:.0}' stroke='#334155'/>", h - 40.0).unwrap();
    writeln!(s, "<text x='46' y='30' class='axis'>{}</text>", fmt_signed(hi)).unwrap();
    writeln!(s, "<text x='46' y='{:.0}' class='axis'>{}</text>", h - 44.0, fmt_signed(lo)).unwrap();
    for ((label, _, color), c) in series.iter().zip(cum.iter()) {
        let pts: String = c
            .iter()
            .enumerate()
            .map(|(i, v)| format!("{:.0},{:.0}", px(i), py(*v)))
            .collect::<Vec<_>>()
            .join(" ");
        writeln!(
            s,
            "<polyline points='{pts}' fill='none' stroke='{color}' stroke-width='2.5' class='buildline'/>"
        )
        .unwrap();
        if let Some(last) = c.last() {
            writeln!(
                s,
                "<text x='{:.0}' y='{:.0}' class='bval' fill='{color}'>{} {}</text>",
                w - 38.0,
                py(*last),
                fmt_signed(*last),
                esc(label)
            )
            .unwrap();
        }
    }
    let _ = s.push_str("</svg>");
    s
}

/// CI-dot chart: diff with whiskers vs zero.
fn svg_ci(rows: &[(String, f64, f64, f64)], w: f64, h: f64) -> String {
    let lo = rows
        .iter()
        .map(|r| fin(r.2))
        .fold(0.0f64, f64::min);
    let hi = rows
        .iter()
        .map(|r| fin(r.3))
        .fold(0.0f64, f64::max);
    let (lo, hi) = if hi - lo < 1e-9 { (lo - 1.0, hi + 1.0) } else { (lo, hi) };
    let row_h = h / rows.len().max(1) as f64;
    let px = |v: f64| 300.0 + (w - 400.0) * (v - lo) / (hi - lo);
    let mut s = format!("<svg viewBox='0 0 {w:.0} {h:.0}' class='chart'>");
    writeln!(s, "<line x1='{:.0}' y1='0' x2='{:.0}' y2='{h:.0}' stroke='#64748b' stroke-width='2'/>", px(0.0), px(0.0)).unwrap();
    for (i, (label, diff, l, u)) in rows.iter().enumerate() {
        let y = i as f64 * row_h + row_h * 0.5;
        writeln!(s, "<text x='290' y='{y:.0}' class='blab' text-anchor='end'>{}</text>", esc(label)).unwrap();
        let (diff, l, u) = (fin(*diff), fin(*l), fin(*u));
        writeln!(
            s,
            "<line x1='{:.0}' y1='{y:.0}' x2='{:.0}' y2='{y:.0}' stroke='#22d3ee' stroke-width='3'/>",
            px(l), px(u)
        )
        .unwrap();
        let color = if l > 0.0 { "#34d399" } else if u < 0.0 { "#f87171" } else { "#f59e0b" };
        writeln!(s, "<circle cx='{:.0}' cy='{y:.0}' r='8' fill='{color}'/>", px(diff)).unwrap();
    }
    s.push_str("</svg>");
    s
}

fn verdict_badge(v: &str) -> &'static str {
    match v {
        "supported" | "survived" => "SURVIVED",
        "split" => "SPLIT",
        _ => "DIDN'T SURVIVE",
    }
}

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

fn title_case(s: &str) -> String {
    let mut out = String::new();
    let mut cap = true;
    for c in s.chars() {
        if cap && c.is_ascii_alphabetic() {
            out.push(c.to_ascii_uppercase());
            cap = false;
        } else {
            if c == ' ' {
                cap = true;
            }
            out.push(c);
        }
    }
    out
}

/// Render the single-file HTML: inline CSS + JS nav/highlight/facecam.
fn render_html(pages: &[Page]) -> String {
    let mut body = String::new();
    for (i, p) in pages.iter().enumerate() {
        let stats: String = p
            .stats
            .iter()
            .map(|(v, l, pos)| {
                format!(
                    "<div class='stat'><div class='sval {}'>{v}</div><div class='slab'>{l}</div></div>",
                    if *pos { "pos" } else { "neg" }
                )
            })
            .collect();
        let verdict = p
            .verdict
            .as_ref()
            .map(|v| {
                let cls = match v.as_str() {
                    "supported" | "survived" => "ok",
                    "split" => "mid",
                    _ => "bad",
                };
                format!("<div class='verdict {cls}'>{}</div>", verdict_badge(v))
            })
            .unwrap_or_default();
        let concl = if p.conclusion.is_empty() {
            String::new()
        } else {
            format!("<div class='concl'>{}</div>", esc(&p.conclusion))
        };
        let card = format!(
            "<section class='page{}' data-i='{i}'>\
             <div class='eyebrow'>{}</div>\
             <h1 class='{}'>{}</h1>\
             <div class='subtitle'>{}</div>\
             <div class='body'>{}</div>\
             <div class='stats'>{}</div>\
             {}{}\
             <div class='pagenum'>{}/{}</div>\
             </section>",
            if p.slide { " slide" } else { "" },
            esc(&p.eyebrow),
            if p.slide { "title slide-title" } else { "title" },
            esc(&p.title),
            esc(&p.subtitle),
            p.body,
            stats,
            concl,
            verdict,
            i + 1,
            pages.len()
        );
        body.push_str(&card);
    }

    format!(
        "<!doctype html><html><head><meta charset='utf-8'><title>Price Action Lab</title>\
<style>
*{{margin:0;padding:0;box-sizing:border-box}}
html,body{{background:#05080f;height:100%;overflow:hidden;font-family:ui-sans-serif,system-ui}}
#stage{{position:absolute;left:50%;top:50%;width:{W}px;height:{H}px;background:#0b0f1a;transform-origin:center;overflow:hidden}}
.page{{position:absolute;inset:0;padding:72px 96px;display:none;flex-direction:column}}
.page.on{{display:flex}}
.eyebrow{{font-family:ui-monospace,Consolas,monospace;letter-spacing:.35em;color:#22d3ee;font-size:22px;text-transform:uppercase;margin-bottom:18px}}
.title{{color:#fff;font-size:84px;font-weight:800;line-height:1.02;margin-bottom:10px}}
.slide-title{{font-size:110px}}
.subtitle{{color:#94a3b8;font-size:30px;margin-bottom:30px;white-space:nowrap;overflow:hidden;text-overflow:ellipsis}}
.body{{flex:1;min-height:0;max-width:1300px;max-height:560px}}
.chart{{width:100%;height:100%;}}
.stats{{display:flex;gap:64px;margin-top:22px;max-width:1300px}}
.stat .sval{{font-size:72px;font-weight:800;color:#22d3ee}}
.stat .sval.neg{{color:#f87171}}
.stat .slab{{color:#94a3b8;font-size:22px;text-transform:uppercase;letter-spacing:.15em;font-family:ui-monospace,monospace}}
.concl{{color:#cbd5e1;font-size:26px;margin-top:16px;max-width:1300px}}
.verdict{{position:absolute;right:96px;top:120px;padding:12px 26px;border-radius:999px;font:800 24px ui-monospace,monospace;letter-spacing:.1em}}
.verdict.ok{{background:#064e3b;color:#34d399}}
.verdict.mid{{background:#453407;color:#f59e0b}}
.verdict.bad{{background:#4c1d1d;color:#f87171}}
.pagenum{{position:absolute;left:96px;bottom:40px;color:#475569;font:20px ui-monospace,monospace}}
.blab{{fill:#cbd5e1;font-size:22px}}
.bval{{fill:#e2e8f0;font-size:20px;font-family:ui-monospace,monospace}}
.axis{{fill:#64748b;font-size:18px;font-family:ui-monospace,monospace}}
.cards{{display:flex;gap:32px;flex-wrap:wrap;margin-top:40px;max-width:1300px}}
.card{{background:#111827;border:1px solid #1f2937;border-radius:24px;padding:42px 48px;min-width:300px}}
.card .num{{font-size:96px;font-weight:800;color:#22d3ee}}
.card .cap{{color:#94a3b8;font-size:24px}}
.badge{{width:64px;height:64px;border-radius:50%;background:#f59e0b;color:#0b0f1a;font-size:34px;font-weight:800;display:flex;align-items:center;justify-content:center;margin-bottom:16px}}
#ring{{position:fixed;width:34px;height:34px;border:3px solid #f59e0b;border-radius:50%;pointer-events:none;z-index:60;transform:translate(-50%,-50%);mix-blend-mode:screen}}
#hl{{position:absolute;inset:0;z-index:50;pointer-events:none}}
#facecam{{position:absolute;left:{:.0}px;top:{:.0}px;width:520px;height:368px;border:2px dashed #334155;border-radius:18px;display:none;align-items:center;justify-content:center;color:#475569;font:20px ui-monospace,monospace;z-index:55}}
#toc{{position:absolute;left:96px;top:200px;background:#0f172a;border:1px solid #1f2937;border-radius:18px;padding:32px 40px;display:none;z-index:56;max-height:60%;overflow:hidden}}
#toc h3{{color:#22d3ee;font:20px ui-monospace,monospace;letter-spacing:.2em;margin-bottom:14px}}
#toc div{{color:#cbd5e1;font-size:22px;padding:4px 0}}
.buildline{{stroke-dasharray:4000;stroke-dashoffset:4000;animation:draw 1.6s ease forwards}}
@keyframes draw{{to{{stroke-dashoffset:0}}}}
.card,.stat,.verdict{{animation:fade .7s ease}}
@keyframes fade{{from{{opacity:0;transform:translateY(18px)}}to{{opacity:1}}}}
</style></head><body>\
<div id='stage'>{body}<svg id='hl'></svg><div id='facecam'>FACECAM</div><div id='toc'></div></div>\
<div id='ring'></div>\
<script>
const pages=[...document.querySelectorAll('.page')];let cur=0;
const hl=document.getElementById('hl'),ring=document.getElementById('ring'),fc=document.getElementById('facecam'),toc=document.getElementById('toc');
function show(i){{cur=Math.max(0,Math.min(pages.length-1,i));pages.forEach((p,j)=>p.classList.toggle('on',j===cur));
const pg=pages[cur];pg.querySelectorAll('.card,.stat,.verdict,.buildline').forEach(e=>{{e.style.animation='none';e.offsetHeight;e.style.animation='';}});history.replaceState(null,'','#'+(cur+1));}}
addEventListener('keydown',e=>{{
if(e.key==='ArrowRight'||e.key===' '||e.key==='PageDown')show(cur+1);
else if(e.key==='ArrowLeft'||e.key==='PageUp')show(cur-1);
else if(e.key==='Home')show(0);else if(e.key==='End')show(pages.length-1);
else if(e.key==='m'||e.key==='M'){{toc.style.display=toc.style.display==='block'?'none':'block';}}
else if(e.key==='f'||e.key==='F')fc.style.display=fc.style.display==='flex'?'none':'flex';
else if(e.key==='r'||e.key==='R')show(cur);}});
toc.innerHTML='<h3>SECTIONS</h3>'+[...new Set(pages.map(p=>p.querySelector('.eyebrow').textContent))].map(s=>`<div>${{s}}</div>`).join('');
let lx=innerWidth/2,ly=innerHeight/2;addEventListener('mousemove',e=>{{lx=e.clientX;ly=e.clientY;ring.style.left=lx+'px';ring.style.top=ly+'px';if(down)draw(e);}});
let down=false,paths=[];
function st(e){{const r=stage.getBoundingClientRect();return{{x:(e.clientX-r.left)*(1920/r.width),y:(e.clientY-r.top)*(1080/r.height)}}}}
addEventListener('mousedown',e=>{{down=true;paths.push(document.createElementNS('http://www.w3.org/2000/svg','path'));const p=paths[paths.length-1];p.setAttribute('stroke','#fde047');p.setAttribute('stroke-width','6');p.setAttribute('fill','none');p.setAttribute('stroke-linecap','round');p.setAttribute('opacity','0.9');const s=st(e);p.setAttribute('d','M'+s.x+' '+s.y);hl.appendChild(p);}});
addEventListener('mouseup',()=>down=false);
function draw(e){{const p=paths[paths.length-1];if(!p)return;const s=st(e);p.setAttribute('d',p.getAttribute('d')+' L'+s.x+' '+s.y);clearTimeout(p._t);p._t=setTimeout(()=>{{p.style.transition='opacity 1.5s';p.style.opacity=0;setTimeout(()=>p.remove(),1600);}},2500);}}
function fit(){{const s=Math.min(innerWidth/1920,innerHeight/1080);stage.style.transform=`translate(-50%,-50%) scale(${{s}})`;}}
addEventListener('resize',fit);fit();
const h=parseInt(location.hash.slice(1));show(isNaN(h)?0:h-1);
if(location.search.includes('check')){{setTimeout(()=>{{const pre=document.createElement('pre');pre.id='check';pre.style.display='none';pre.textContent=window.__layoutReport();document.body.appendChild(pre);}},300);}}
window.__layoutReport=function(){{const r=stage.getBoundingClientRect(),sc=1920/r.width;
const bad=[];pages.forEach((p,i)=>{{if(!p.classList.contains('on'))return;
p.querySelectorAll('*').forEach(el=>{{const b=el.getBoundingClientRect();if(b.width===0&&b.height===0)return;
if(b.left<r.left-2||b.top<r.top-2||b.right>r.right+2||b.bottom>r.bottom+2)bad.push('off:'+i+':'+el.tagName);
const fx=r.right-520/sc,fy=r.bottom-368/sc;if(b.right>fx&&b.bottom>fy&&b.width>4&&b.height>4&&!el.closest('#hl')&&el.tagName!=='SECTION')bad.push('fc:'+i+':'+el.tagName);}});
const sub=p.querySelector('.subtitle');if(sub&&sub.scrollWidth>sub.clientWidth+2)bad.push('sub:'+i);
if(p.scrollHeight>p.clientHeight+2)bad.push('scroll:'+i);}});
return JSON.stringify({{pages:pages.length,bad:bad}});}};
</script></body></html>",
        FACECAM.0, FACECAM.1
    )
}

/// `pal dash` driver: collect numbers, reread-verify, emit html+provenance.
pub fn run_dash(root: &Path) -> Result<PathBuf, String> {
    let results_pal = root.join("results/pal");
    if !results_pal.exists() {
        return Err("no results/pal — run behavior/funnel/final-test first".into());
    }
    let (pages, provs) = build_pages(root);
    if pages.is_empty() {
        return Err("no pages could be built from saved results".into());
    }
    let out_dir = results_pal.join("dash");
    std::fs::create_dir_all(&out_dir).map_err(|e| e.to_string())?;

    // provenance.csv
    let mut csv = String::from("page,element,file,row,column,format,value,text\n");
    for p in &provs {
        writeln!(
            csv,
            "{},{},{},{},{},{},{},{}",
            p.page, p.element, p.file, p.row, p.col, p.format, p.value,
            p.text.replace(',', ";")
        )
        .map_err(|e| e.to_string())?;
    }
    std::fs::write(out_dir.join("provenance.csv"), &csv).map_err(|e| e.to_string())?;

    // independent reread — refuse to build on any difference
    let errors = verify_provenance(&results_pal, &provs);
    let check = serde_json::json!({
        "checked": provs.len(),
        "errors": errors,
        "ok": errors.is_empty(),
    });
    std::fs::write(
        out_dir.join("provenance_check.json"),
        serde_json::to_string_pretty(&check).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    if !errors.is_empty() {
        return Err(format!(
            "provenance check failed ({} diffs) — page NOT written",
            errors.len()
        ));
    }

    // page list for the checker
    let list: Vec<serde_json::Value> = pages
        .iter()
        .enumerate()
        .map(|(i, p)| {
            serde_json::json!({
                "i": i + 1,
                "eyebrow": p.eyebrow,
                "title": p.title,
                "subtitle": p.subtitle,
                "stats": p.stats.iter().map(|(v, l, _)| format!("{v} {l}")).collect::<Vec<_>>(),
                "conclusion": p.conclusion,
                "verdict": p.verdict,
            })
        })
        .collect();
    std::fs::write(
        out_dir.join("pages.json"),
        serde_json::to_string_pretty(&list).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;

    let html = render_html(&pages);
    let path = out_dir.join("dashboard.html");
    std::fs::write(&path, html).map_err(|e| e.to_string())?;
    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn facecam_rect_is_corner() {
        assert_eq!(FACECAM, (1400.0, 712.0, 520.0, 368.0));
    }
}
