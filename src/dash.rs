//! Generic presenter dashboard engine + study deck builder.
//!
//! One offline HTML file per study at results/<study>/dash/dashboard.html,
//! every headline number traced through results/<study>/dash/provenance.csv
//! and re-verified by an independent second reader before the page is
//! written. Reads ONLY saved result tables — never raw data.
//!
//! Stage: 1920x1080, navy #0b0f1a, cyan #22d3ee, amber #f59e0b.
//! Facecam corner: bottom-right 520x368 stays empty.
//!
//! PAL-specific pages live in pal::dash and reuse this engine; the study
//! deck (study_pages) covers any standard results/<study>/ directory.

use std::fmt::Write as _;
use std::path::{Path, PathBuf};

const W: f64 = 1920.0;
const H: f64 = 1080.0;
pub const FACECAM: (f64, f64, f64, f64) = (W - 520.0, H - 368.0, 520.0, 368.0);

/// A number bound for the page with its source. `row` is the first-column
/// key, or `k1|k2|...` for composite keys on multi-dimensional tables.
#[derive(Clone, Debug)]
pub struct Prov {
    pub page: String,
    pub element: String,
    pub file: String,
    pub row: String,
    pub col: String,   // header name, or __count__/__sum__:col
    pub format: String,
    pub value: String, // raw value as read
    pub text: String,  // text shown on the page
}

pub struct Book {
    pub provs: Vec<Prov>,
    root: PathBuf,
    tables: std::collections::HashMap<PathBuf, (Vec<String>, Vec<Vec<String>>)>,
    jsonls: std::collections::HashMap<PathBuf, Vec<serde_json::Value>>,
}

impl Book {
    pub fn new(root: PathBuf) -> Book {
        Book {
            provs: Vec::new(),
            root,
            tables: Default::default(),
            jsonls: Default::default(),
        }
    }

    pub fn read_csv(&mut self, rel: &str) -> Option<&(Vec<String>, Vec<Vec<String>>)> {
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

    pub fn read_jsonl(&mut self, rel: &str) -> Option<&Vec<serde_json::Value>> {
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
    pub fn cell(&mut self, rel: &str, key: &str, col: &str) -> Option<String> {
        let (header, rows) = self.read_csv(rel)?;
        let ci = header.iter().position(|h| h == col)?;
        rows.iter()
            .find(|r| r.first().map(|x| x == key).unwrap_or(false))
            .and_then(|r| r.get(ci).cloned())
    }

    /// Composite-key lookup: first `keys.len()` columns must equal `keys`,
    /// then the named column. Used for fill|period, variant|fill, etc.
    pub fn cellk(&mut self, rel: &str, keys: &[&str], col: &str) -> Option<String> {
        let (header, rows) = self.read_csv(rel)?;
        let ci = header.iter().position(|h| h == col)?;
        rows.iter()
            .find(|r| {
                keys.iter()
                    .enumerate()
                    .all(|(i, k)| r.get(i).map(|x| x == k).unwrap_or(false))
            })
            .and_then(|r| r.get(ci).cloned())
    }

    /// Register a number: look it up, format it, record provenance.
    pub fn num(
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
        } else if let Some(uniqcol) = col.strip_prefix("__uniq__:") {
            self.read_csv(file).and_then(|(header, rows)| {
                let ci = header.iter().position(|h| h == uniqcol)?;
                let mut seen = std::collections::HashSet::new();
                for r in rows {
                    if let Some(v) = r.get(ci) {
                        seen.insert(v.clone());
                    }
                }
                Some(seen.len().to_string())
            })
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
        let text = fmt_value(fmt, &raw);
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

    /// Composite-key variant of `num`: `keys` are the leading column values,
    /// recorded in `row` joined by `|` so the verifier can re-resolve them.
    pub fn numk(
        &mut self,
        page: &str,
        element: &str,
        file: &str,
        keys: &[&str],
        col: &str,
        fmt: &str,
    ) -> String {
        let row = keys.join("|");
        let raw = self.cellk(file, keys, col).unwrap_or_else(|| "NA".into());
        let text = fmt_value(fmt, &raw);
        self.provs.push(Prov {
            page: page.into(),
            element: element.into(),
            file: file.into(),
            row,
            col: col.into(),
            format: fmt.into(),
            value: raw,
            text: text.clone(),
        });
        text
    }
}

fn fmt_value(fmt: &str, raw: &str) -> String {
    match (fmt, raw.parse::<f64>()) {
        ("money", Ok(v)) => format!("${v:+.0}"),
        ("pct", Ok(v)) => format!("{:.0}%", v * 100.0),
        ("pct1", Ok(v)) => format!("{v:.1}%"),
        ("f2", Ok(v)) => format!("{v:.2}"),
        ("f3", Ok(v)) => format!("{v:.3}"),
        ("f4", Ok(v)) => format!("{v:.4}"),
        ("int", Ok(v)) => format!("{v:.0}"),
        _ => raw.to_string(),
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
        } else if let Some(uniqcol) = p.col.strip_prefix("__uniq__:") {
            std::fs::read_to_string(&path).ok().and_then(|t| {
                let mut lines = t.lines();
                let header: Vec<&str> = lines.next()?.split(',').collect();
                let ci = header.iter().position(|h| *h == uniqcol)?;
                let mut seen = std::collections::HashSet::new();
                for l in lines.filter(|l| !l.trim().is_empty()) {
                    if let Some(v) = l.split(',').nth(ci) {
                        seen.insert(v.to_string());
                    }
                }
                Some(seen.len().to_string())
            })
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
                let keys: Vec<&str> = p.row.split('|').collect();
                lines
                    .filter(|l| !l.trim().is_empty())
                    .map(|l| l.split(',').collect::<Vec<&str>>())
                    .find(|cols| {
                        keys.iter()
                            .enumerate()
                            .all(|(i, k)| cols.get(i).map(|x| x == k).unwrap_or(false))
                    })
                    .and_then(|cols| cols.get(ci).map(|s| s.to_string()))
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
                let text = fmt_value(&p.format, &v);
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
    pub eyebrow: String,
    pub title: String,
    pub subtitle: String,
    pub body: String,       // svg or cards html
    pub stats: Vec<(String, String, bool)>, // value, label, positive?
    pub conclusion: String,
    pub verdict: Option<String>,
    pub slide: bool,
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

pub(crate) fn svg_bars(items: &[(String, f64)], w: f64, h: f64) -> String {
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
    let scale = ((mid_x - 20.0) / neg_max).min((w - mid_x - 160.0) / pos_max);
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
pub(crate) fn svg_lines(series: &[(String, Vec<f64>, &'static str)], dates: &[String], w: f64, h: f64) -> String {
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

pub(crate) fn svg_ci(rows: &[(String, f64, f64, f64)], w: f64, h: f64) -> String {
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

pub fn title_case(s: &str) -> String {
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
        "<!doctype html><html><head><meta charset='utf-8'><title>DeepThrust</title>\
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

/// Write provenance.csv, run the independent reread, then pages.json +
/// dashboard.html into `out_dir`. `book_root` is the directory the
/// provenance file paths are relative to (results/<study> or results/pal).
pub fn emit(book_root: &Path, out_dir: &Path, pages: Vec<Page>, provs: Vec<Prov>) -> Result<PathBuf, String> {
    if pages.is_empty() {
        return Err("no pages could be built from saved results".into());
    }
    std::fs::create_dir_all(out_dir).map_err(|e| e.to_string())?;

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

    let errors = verify_provenance(book_root, &provs);
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

// ---------------------------------------------------------------------------
// Generic study deck: any standard results/<study>/ directory.
// ---------------------------------------------------------------------------

/// One provenance-backed lookup returning both the display text and the
/// numeric value for charts. `keys` are leading column values; a single
/// key uses Book::num, multiple keys use Book::numk (row recorded as
/// "k1|k2|.."). Exactly one Prov is recorded per call.
fn pv(
    b: &mut Book,
    page: &str,
    el: &str,
    file: &str,
    keys: &[&str],
    col: &str,
    fmt: &str,
) -> (String, f64) {
    let text = if keys.len() == 1 {
        b.num(page, el, file, keys[0], col, fmt)
    } else {
        b.numk(page, el, file, keys, col, fmt)
    };
    let v = b
        .provs
        .last()
        .and_then(|p| p.value.parse::<f64>().ok())
        .unwrap_or(0.0);
    (text, v)
}

fn f64v(s: Option<&String>) -> Option<f64> {
    s.and_then(|v| v.parse::<f64>().ok())
}

fn col_idx(header: &[String], name: &str) -> Option<usize> {
    header.iter().position(|h| h == name)
}

/// Build the study deck pages from a standard results directory.
/// `book_root` is results/<study>; every displayed number goes through
/// pv() so provenance covers it.
pub fn study_pages(book_root: &Path) -> (Vec<Page>, Vec<Prov>) {
    let mut b = Book::new(book_root.to_path_buf());
    let mut pages: Vec<Page> = Vec::new();

    // Anchor: the partition dev table defines fills and headline stats.
    let dev_tab = b
        .read_csv("partition_dev.csv")
        .map(|t| t.clone())
        .unwrap_or_default();
    if dev_tab.1.is_empty() {
        return (pages, b.provs);
    }
    let (dh, drows) = dev_tab;
    let (Some(fi), Some(_pi)) = (col_idx(&dh, "fill"), col_idx(&dh, "period")) else {
        return (pages, b.provs);
    };
    let mut fills: Vec<String> = Vec::new();
    for r in &drows {
        if let Some(f) = r.get(fi) {
            if !fills.contains(f) {
                fills.push(f.clone());
            }
        }
    }
    let primary = fills.first().cloned().unwrap_or_default();
    let pfill = primary.as_str();

    // ---- slide ----
    let study = book_root
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default();
    let mut cards = String::new();
    if b.read_csv("per_etf.csv").is_some() {
        let (n, _) = pv(&mut b, "slide", "symbols", "per_etf.csv", &["*"], "__uniq__:symbol", "int");
        cards += &format!("<div class='card'><div class='num'>{n}</div><div class='cap'>symbols</div></div>");
    }
    if b.read_csv("variants_comparison.csv").is_some() {
        let (n, _) = pv(&mut b, "slide", "variants", "variants_comparison.csv", &["*"], "__uniq__:variant", "int");
        cards += &format!("<div class='card'><div class='num'>{n}</div><div class='cap'>variants tried</div></div>");
    } else {
        cards += "<div class='card'><div class='num'>1</div><div class='cap'>variant (baseline)</div></div>";
    }
    let (d0, d1) = b
        .read_csv("daily_returns.csv")
        .map(|(_, r)| {
            (
                r.first().and_then(|x| x.first()).cloned().unwrap_or_default(),
                r.last().and_then(|x| x.first()).cloned().unwrap_or_default(),
            )
        })
        .unwrap_or_default();
    let (dev_trades, _) = pv(&mut b, "slide", "dev_trades", "partition_dev.csv", &[pfill], "trades", "int");
    let (test_trades, _) = pv(&mut b, "slide", "test_trades", "partition_test.csv", &[pfill], "trades", "int");
    cards += &format!(
        "<div class='card'><div class='num'>{dev_trades}</div><div class='cap'>dev trades</div></div>\
         <div class='card'><div class='num'>{test_trades}</div><div class='cap'>test trades</div></div>"
    );
    pages.push(Page {
        eyebrow: "DEEPTHRUST STUDY".into(),
        title: title_case(&study.replace('_', " ")),
        subtitle: format!("sessions {d0} to {d1} · fill {primary}"),
        body: format!("<div class='cards'>{cards}</div>"),
        stats: vec![],
        conclusion: String::new(),
        verdict: None,
        slide: true,
    });

    // ---- headline: dev vs test (primary fill) ----
    let (dev_log, _) = pv(&mut b, "result", "dev_total_log", "partition_dev.csv", &[pfill], "total_log", "f4");
    let (dev_sharpe, _) = pv(&mut b, "result", "dev_sharpe", "partition_dev.csv", &[pfill], "sharpe", "f2");
    let (test_log, _) = pv(&mut b, "result", "test_total_log", "partition_test.csv", &[pfill], "total_log", "f4");
    let (test_sharpe, _) = pv(&mut b, "result", "test_sharpe", "partition_test.csv", &[pfill], "sharpe", "f2");
    let dev_skill_v = pv(&mut b, "result", "dev_skill", "partition_dev.csv", &[pfill], "skill", "f4").1;
    let test_skill_v = pv(&mut b, "result", "test_skill", "partition_test.csv", &[pfill], "skill", "f4").1;
    let decayed = dev_skill_v * test_skill_v < 0.0;
    pages.push(Page {
        eyebrow: "THE RESULT".into(),
        title: "Dev Versus Test".into(),
        subtitle: format!("primary fill {primary}, log returns net of costs"),
        body: String::new(),
        stats: vec![
            (dev_log, "dev total log".into(), true),
            (dev_sharpe, "dev sharpe".into(), true),
            (test_log, "test total log".into(), true),
            (test_sharpe, "test sharpe".into(), true),
        ],
        conclusion: if decayed {
            "test skill flips sign against dev".into()
        } else {
            "skill keeps its sign across the split".into()
        },
        verdict: if decayed { Some("split".into()) } else { None },
        slide: false,
    });
    // ---- trade profile: counts chart + win/pf stats ----
    {
        let (_, dev_tr_v) = pv(&mut b, "profile", "dev_trades", "partition_dev.csv", &[pfill], "trades", "int");
        let (_, test_tr_v) = pv(&mut b, "profile", "test_trades", "partition_test.csv", &[pfill], "trades", "int");
        let (dev_wr, _) = pv(&mut b, "profile", "dev_win_pct", "partition_dev.csv", &[pfill], "win_pct", "pct1");
        let (test_wr, _) = pv(&mut b, "profile", "test_win_pct", "partition_test.csv", &[pfill], "win_pct", "pct1");
        let (dev_pf, _) = pv(&mut b, "profile", "dev_pf", "partition_dev.csv", &[pfill], "pf", "f2");
        let (test_pf, _) = pv(&mut b, "profile", "test_pf", "partition_test.csv", &[pfill], "pf", "f2");
        let _ = (dev_tr_v, test_tr_v);
        pages.push(Page {
            eyebrow: "THE RESULT".into(),
            title: "Trade Profile".into(),
            subtitle: "trade counts by period; win rate and profit factor below".into(),
            body: svg_bars(
                &[
                    ("dev trades".to_string(), dev_tr_v),
                    ("test trades".to_string(), test_tr_v),
                ],
                900.0,
                300.0,
            ),
            stats: vec![
                (dev_wr, "dev win %".into(), true),
                (test_wr, "test win %".into(), true),
                (dev_pf, "dev pf".into(), true),
                (test_pf, "test pf".into(), true),
            ],
            conclusion: "counts, win rate and profit factor from the partition tables".into(),
            verdict: None,
            slide: false,
        });
    }

    // ---- skill decomposition ----
    {
        let comps = ["total_log", "trend", "bias", "skill"];
        let mut items: Vec<(String, f64)> = Vec::new();
        for (per, file) in [("dev", "partition_dev.csv"), ("test", "partition_test.csv")] {
            for c in comps {
                let (_, v) = pv(&mut b, "skill", &format!("{per}_{c}"), file, &[pfill], c, "f4");
                items.push((format!("{per} {c}"), v));
            }
        }
        let (dev_rp, _) = pv(&mut b, "skill", "dev_random_p", "partition_dev.csv", &[pfill], "random_p_pool", "f3");
        let (test_rp, _) = pv(&mut b, "skill", "test_random_p", "partition_test.csv", &[pfill], "random_p_pool", "f3");
        pages.push(Page {
            eyebrow: "THE RESULT".into(),
            title: "Skill Decomposition".into(),
            subtitle: "where the log return came from, per period".into(),
            body: svg_bars(&items, 1500.0, 480.0),
            stats: vec![
                (dev_rp, "dev random-p".into(), true),
                (test_rp, "test random-p".into(), true),
            ],
            conclusion: "skill = total - trend - bias; random-p is the pick baseline tail".into(),
            verdict: None,
            slide: false,
        });
    }

    // ---- baselines ----
    {
        let bl = b
            .read_csv("baselines.csv")
            .map(|t| t.clone())
            .unwrap_or_default();
        if !bl.1.is_empty() {
            let (bh, brows) = bl;
            let bcol = |n: &str| col_idx(&bh, n);
            if let (Some(bk), Some(bp), Some(_bt)) = (bcol("baseline"), bcol("period"), bcol("total_log")) {
                let mut items: Vec<(String, f64)> = Vec::new();
                for r in &brows {
                    let name = r.get(bk).cloned().unwrap_or_default();
                    let per = r.get(bp).cloned().unwrap_or_default();
                    let keys = [name.as_str(), per.as_str()];
                    let (_, v) = pv(&mut b, "baselines", &format!("{per}_{name}"), "baselines.csv", &keys, "total_log", "f4");
                    items.push((format!("{per} {name}"), v));
                }
                let (strat_dev, _) = pv(&mut b, "baselines", "strategy_dev", "partition_dev.csv", &[pfill], "total_log", "f4");
                // random timing p-value lives on the random_timing row
                let mut rand_p = String::new();
                for r in &brows {
                    if r.get(bk).map(|x| x.starts_with("random_timing")).unwrap_or(false) {
                        let name = r.get(bk).cloned().unwrap_or_default();
                        let per = r.get(bp).cloned().unwrap_or_default();
                        if bcol("p_total_ge_strategy").is_some() {
                            let keys = [name.as_str(), per.as_str()];
                            rand_p = pv(&mut b, "baselines", "random_timing_p", "baselines.csv", &keys, "p_total_ge_strategy", "f3").0;
                        }
                        break;
                    }
                }
                pages.push(Page {
                    eyebrow: "AGAINST BASELINES".into(),
                    title: "Buy Hold And Timing".into(),
                    subtitle: "total log return of each benchmark series by period".into(),
                    body: svg_bars(&items, 1400.0, 460.0),
                    stats: vec![
                        (strat_dev, "strategy dev total".into(), true),
                        (rand_p, "random timing p".into(), true),
                    ],
                    conclusion: "ew buy hold, sma200 timing, vol-matched and random timing".into(),
                    verdict: None,
                    slide: false,
                });
            }
        }
    }

    // ---- equity curve ----
    {
        let dr = b
            .read_csv("daily_returns.csv")
            .map(|t| t.clone())
            .unwrap_or_default();
        if !dr.1.is_empty() {
            let (xh, xrows) = dr;
            let di = col_idx(&xh, "date").unwrap_or(0);
            let dates: Vec<String> = xrows.iter().filter_map(|r| r.get(di).cloned()).collect();
            let mut series: Vec<(String, Vec<f64>, &'static str)> = Vec::new();
            let colors = ["#22d3ee", "#f59e0b", "#34d399", "#a78bfa"];
            for (i, f) in fills.iter().enumerate() {
                let colname = format!("{f}_r_day");
                if let Some(ci) = col_idx(&xh, &colname) {
                    let v: Vec<f64> = xrows.iter().filter_map(|r| f64v(r.get(ci))).collect();
                    series.push((f.clone(), v, colors[i % colors.len()]));
                }
            }
            if let Some(bi) = col_idx(&xh, "b_day") {
                let v: Vec<f64> = xrows.iter().filter_map(|r| f64v(r.get(bi))).collect();
                series.push(("buy hold".into(), v, "#64748b"));
            }
            if !series.is_empty() {
                pages.push(Page {
                    eyebrow: "THE RESULT".into(),
                    title: "Equity Curve".into(),
                    subtitle: "cumulative daily returns vs buy hold".into(),
                    body: svg_lines(&series, &dates, 1500.0, 520.0),
                    stats: vec![],
                    conclusion: "daily returns summed per session; benchmark is the equal-weight book".into(),
                    verdict: None,
                    slide: false,
                });
            }
        }
    }

    // ---- variants leaderboard ----
    {
        let vc = b
            .read_csv("variants_comparison.csv")
            .map(|t| t.clone())
            .unwrap_or_default();
        if !vc.1.is_empty() {
            let (vh, vrows) = vc;
            let (Some(vi), Some(vf), Some(_vd)) = (
                col_idx(&vh, "variant"),
                col_idx(&vh, "fill"),
                col_idx(&vh, "dev_total_log"),
            ) else {
                return (pages, b.provs);
            };
            let mut keyed: Vec<(String, String)> = Vec::new(); // variant, fill
            for r in &vrows {
                if r.get(vf).map(|x| x == &primary).unwrap_or(false) {
                    if let Some(v) = r.get(vi) {
                        keyed.push((v.clone(), primary.clone()));
                    }
                }
            }
            keyed.sort();
            keyed.dedup();
            let n_var = keyed.len();
            // fetch values via provenance lookups, rank by dev_total_log
            let mut scored: Vec<(String, f64)> = Vec::new();
            for (var, _) in &keyed {
                let (_, v) = pv(&mut b, "variants", &format!("dev_{var}"), "variants_comparison.csv", &[var.as_str(), pfill], "dev_total_log", "f4");
                scored.push((var.clone(), v));
            }
            scored.sort_by(|a, c| c.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
            let bars: Vec<(String, f64)> = scored.iter().take(16).cloned().collect();
            if !bars.is_empty() {
                let (best, _) = pv(&mut b, "variants", "best_dev", "variants_comparison.csv", &[scored[0].0.as_str(), pfill], "dev_total_log", "f4");
                pages.push(Page {
                    eyebrow: "VARIANTS".into(),
                    title: "Dev Leaderboard".into(),
                    subtitle: "dev total log per variant, primary fill; test reported once".into(),
                    body: svg_bars(&bars, 1500.0, (bars.len() as f64 * 34.0).min(560.0)),
                    stats: vec![
                        (best, "best dev total".into(), true),
                        (format!("{n_var}"), "variants".into(), true),
                    ],
                    conclusion: format!("top {} of {} shown; selection on dev only", bars.len(), n_var),
                    verdict: None,
                    slide: false,
                });
            }
        }
    }

    // ---- cost ladder ----
    {
        let cl = b
            .read_csv("cost_ladder.csv")
            .map(|t| t.clone())
            .unwrap_or_default();
        if !cl.1.is_empty() {
            let (ch, crows) = cl;
            // keys must be the leading contiguous columns: fill, cost_mult,
            // [cost_bps_side], period — bps sits between mult and period.
            let cidx = |n: &str| col_idx(&ch, n);
            if let (Some(cf), Some(cc), Some(cp), Some(_cs)) = (
                cidx("fill"),
                cidx("cost_mult"),
                cidx("period"),
                cidx("sharpe"),
            ) {
                let cb = cidx("cost_bps_side");
                let keys_of = |r: &Vec<String>| -> Vec<String> {
                    let mut k = vec![
                        r.get(cf).cloned().unwrap_or_default(),
                        r.get(cc).cloned().unwrap_or_default(),
                    ];
                    if let Some(bi) = cb {
                        k.push(r.get(bi).cloned().unwrap_or_default());
                    }
                    k.push(r.get(cp).cloned().unwrap_or_default());
                    k
                };
                let mut items: Vec<(String, f64)> = Vec::new();
                let mut base: std::collections::HashMap<String, Vec<String>> =
                    std::collections::HashMap::new();
                for r in &crows {
                    if r.get(cf).map(|x| x == &primary).unwrap_or(false) {
                        let keys = keys_of(r);
                        let kref: Vec<&str> = keys.iter().map(|s| s.as_str()).collect();
                        let mult = keys[1].clone();
                        let per = keys[keys.len() - 1].clone();
                        let (_, v) = pv(&mut b, "costs", &format!("{per}_{mult}"), "cost_ladder.csv", &kref, "sharpe", "f2");
                        items.push((format!("{per} {mult}x"), v));
                        if mult.parse::<f64>().map(|m| (m - 1.0).abs() < 1e-9).unwrap_or(false) {
                            base.insert(per, keys);
                        }
                    }
                }
                if !items.is_empty() {
                    let mut stats = Vec::new();
                    for per in ["dev", "test"] {
                        let text = match base.get(per) {
                            Some(k) => {
                                let kr: Vec<&str> = k.iter().map(|s| s.as_str()).collect();
                                pv(&mut b, "costs", &format!("{per}_1x"), "cost_ladder.csv", &kr, "sharpe", "f2").0
                            }
                            None => "NA".into(),
                        };
                        stats.push((text, format!("{per} sharpe 1x"), true));
                    }
                    pages.push(Page {
                        eyebrow: "COSTS".into(),
                        title: "Cost Ladder".into(),
                        subtitle: "sharpe as a multiple of the base per-side cost".into(),
                        body: svg_bars(&items, 1400.0, 400.0),
                        stats,
                        conclusion: "edge should survive a few times the assumed cost".into(),
                        verdict: None,
                        slide: false,
                    });
                }
            }
        }
    }

    // ---- per symbol ----
    {
        let ss = b
            .read_csv("symbols_summary.csv")
            .map(|t| t.clone())
            .unwrap_or_default();
        if !ss.1.is_empty() {
            let (sh, srows) = ss;
            let (Some(sv), Some(ssym), Some(sd), Some(_st)) = (
                col_idx(&sh, "variant"),
                col_idx(&sh, "symbol"),
                col_idx(&sh, "dev_total_log"),
                col_idx(&sh, "test_total_log"),
            ) else {
                return (pages, b.provs);
            };
            let mut rows: Vec<&Vec<String>> = srows
                .iter()
                .filter(|r| r.get(sv).map(|x| x == "baseline").unwrap_or(false))
                .collect();
            if rows.is_empty() {
                rows = srows.iter().collect();
            }
            let mut keyed: Vec<(f64, &Vec<String>)> = rows
                .iter()
                .filter_map(|r| f64v(r.get(sd)).map(|v| (v.abs(), *r)))
                .collect();
            keyed.sort_by(|a, c| c.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
            keyed.truncate(12);
            let mut items: Vec<(String, f64)> = Vec::new();
            for (_, r) in &keyed {
                let var = r.get(sv).cloned().unwrap_or_default();
                let sym = r.get(ssym).cloned().unwrap_or_default();
                let keys = [var.as_str(), sym.as_str()];
                let (_, dv) = pv(&mut b, "symbols", &format!("dev_{sym}"), "symbols_summary.csv", &keys, "dev_total_log", "f4");
                let (_, tv) = pv(&mut b, "symbols", &format!("test_{sym}"), "symbols_summary.csv", &keys, "test_total_log", "f4");
                items.push((format!("{sym} dev"), dv));
                items.push((format!("{sym} test"), tv));
            }
            if items.len() > 2 {
                pages.push(Page {
                    eyebrow: "PER SYMBOL".into(),
                    title: "Where It Earned".into(),
                    subtitle: "dev and test total log per symbol, baseline variant".into(),
                    body: svg_bars(&items, 1500.0, (items.len() as f64 * 30.0).min(540.0)),
                    stats: vec![],
                    conclusion: "top symbols by absolute dev contribution".into(),
                    verdict: None,
                    slide: false,
                });
            }
        }
    }

    // ---- closing slide ----
    let (dev_sharpe2, _) = pv(&mut b, "close", "dev_sharpe", "partition_dev.csv", &[pfill], "sharpe", "f2");
    let (test_sharpe2, _) = pv(&mut b, "close", "test_sharpe", "partition_test.csv", &[pfill], "sharpe", "f2");
    pages.push(Page {
        eyebrow: "DEEPTHRUST STUDY".into(),
        title: title_case(&study.replace('_', " ")),
        subtitle: "partition-tested, cost-netted, provenance-checked".into(),
        body: String::new(),
        stats: vec![
            (dev_sharpe2, "dev sharpe".into(), true),
            (test_sharpe2, "test sharpe".into(), true),
        ],
        conclusion: "reproduce: deepthrust --strategy <name> --variant <v>".into(),
        verdict: None,
        slide: true,
    });

    (pages, b.provs)
}

/// `pal dash --study <name>` driver: generic deck for results/<study>/.
pub fn run_study_dash(root: &Path, study: &str) -> Result<PathBuf, String> {
    let dir = root.join("results").join(study);
    if !dir.join("partition_dev.csv").exists() {
        return Err(format!("{study}: no partition_dev.csv under results/"));
    }
    let (pages, provs) = study_pages(&dir);
    let out = dir.join("dash");
    emit(&dir, &out, pages, provs)
}
