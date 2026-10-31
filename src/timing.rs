//! Lightweight stage timing with an end-of-run summary table (port of src/utils/timing.py).
//!
//! ```ignore
//! timing::install("run");                   // once per entry point
//! let _g = timing::stage("load panel");     // nested guards build a tree
//! timing::finish(profile);                  // prints summary + writes results/timing/*.json
//! ```

use std::collections::BTreeMap;
use std::sync::{Mutex, OnceLock};
use std::time::Instant;

#[derive(Clone, Copy, Default)]
struct Rec {
    wall: f64,
    calls: u64,
    min: f64,
    max: f64,
}

struct State {
    t0: Instant,
    name: String,
    stack: Vec<String>,
    tree: BTreeMap<Vec<String>, Rec>,
}

fn state() -> &'static Mutex<State> {
    static S: OnceLock<Mutex<State>> = OnceLock::new();
    S.get_or_init(|| {
        Mutex::new(State {
            t0: Instant::now(),
            name: "run".into(),
            stack: vec![],
            tree: BTreeMap::new(),
        })
    })
}

/// Reset the run clock and name the run.
pub fn install(name: &str) {
    let mut s = state().lock().unwrap();
    s.t0 = Instant::now();
    s.name = name.to_string();
    s.stack.clear();
    s.tree.clear();
}

/// RAII guard returned by [`stage`]; records wall time on drop.
pub struct Guard {
    t0: Instant,
}

pub fn stage(name: &str) -> Guard {
    state().lock().unwrap().stack.push(name.to_string());
    Guard { t0: Instant::now() }
}

impl Drop for Guard {
    fn drop(&mut self) {
        let dt = self.t0.elapsed().as_secs_f64();
        let mut s = state().lock().unwrap();
        let key = s.stack.clone();
        let rec = s.tree.entry(key).or_insert_with(|| Rec {
            min: f64::INFINITY,
            ..Default::default()
        });
        rec.wall += dt;
        rec.calls += 1;
        rec.min = rec.min.min(dt);
        rec.max = rec.max.max(dt);
        s.stack.pop();
    }
}

/// Time a closure as a named stage.
pub fn timed<T>(name: &str, f: impl FnOnce() -> T) -> T {
    let _g = stage(name);
    f()
}

fn fmt(s: f64) -> String {
    if s >= 600.0 {
        format!("{:.1}m", s / 60.0)
    } else {
        format!("{s:.2}s")
    }
}

fn children(tree: &BTreeMap<Vec<String>, Rec>, prefix: &[String]) -> Vec<Vec<String>> {
    let mut kids: Vec<Vec<String>> = tree
        .keys()
        .filter(|k| k.len() == prefix.len() + 1 && k[..prefix.len()] == *prefix)
        .cloned()
        .collect();
    kids.sort_by(|a, b| tree[b].wall.partial_cmp(&tree[a].wall).unwrap());
    kids
}

/// Build the summary table text. `profile` adds mean/min/max per-call columns.
pub fn summary(profile: bool) -> String {
    let s = state().lock().unwrap();
    let total = s.t0.elapsed().as_secs_f64();
    let mut lines = vec![
        format!("\n{}", "=".repeat(78)),
        format!(
            "TIMING SUMMARY: {}  (total wall {}, {:.1}s)",
            s.name,
            fmt(total),
            total
        ),
        "=".repeat(78),
    ];
    let mut head = format!("{:<52}{:>9}{:>7}{:>7}", "stage", "wall", "%run", "calls");
    if profile {
        head += &format!("{:>11}{:>11}{:>11}", "mean ms", "min ms", "max ms");
    }
    lines.push(head);
    fn walk(
        tree: &BTreeMap<Vec<String>, Rec>,
        prefix: &[String],
        total: f64,
        profile: bool,
        out: &mut Vec<String>,
    ) {
        for k in children(tree, prefix) {
            let r = tree[&k];
            let label = format!("{}{}", "  ".repeat(k.len() - 1), k.last().unwrap());
            let mut line = format!(
                "{:<52}{:>9}{:>6.1}%{:>7}",
                label,
                fmt(r.wall),
                100.0 * r.wall / total,
                r.calls
            );
            if profile {
                line += &format!(
                    "{:>11.3}{:>11.3}{:>11.3}",
                    1e3 * r.wall / r.calls as f64,
                    1e3 * r.min,
                    1e3 * r.max
                );
            }
            out.push(line);
            walk(tree, &k, total, profile, out);
            let kids = children(tree, &k);
            let sub: f64 = kids.iter().map(|c| tree[c].wall).sum();
            if !kids.is_empty() && r.wall - sub > (0.05 * r.wall).max(0.2) {
                out.push(format!(
                    "{:<52}{:>9}{:>6.1}%",
                    format!("{}<untracked>", "  ".repeat(k.len())),
                    fmt(r.wall - sub),
                    100.0 * (r.wall - sub) / total
                ));
            }
        }
    }
    walk(&s.tree, &[], total, profile, &mut lines);
    let top: f64 = children(&s.tree, &[]).iter().map(|c| s.tree[c].wall).sum();
    if total - top > (0.02 * total).max(0.5) {
        lines.push(format!(
            "{:<52}{:>9}{:>6.1}%",
            "<untracked>",
            fmt(total - top),
            100.0 * (total - top) / total
        ));
    }
    lines.push("=".repeat(78));
    lines.join("\n")
}

/// Print the summary and write `results/timing/<name>_<stamp>.json`.
pub fn finish(profile: bool, results_dir: &std::path::Path) {
    println!("{}", summary(profile));
    let s = state().lock().unwrap();
    let stages: serde_json::Map<String, serde_json::Value> = s
        .tree
        .iter()
        .map(|(k, r)| {
            (
                k.join(" > "),
                serde_json::json!({"wall_s": r.wall, "calls": r.calls, "min_s": r.min, "max_s": r.max}),
            )
        })
        .collect();
    let data = serde_json::json!({"name": s.name, "total_wall_s": s.t0.elapsed().as_secs_f64(), "stages": stages});
    let dir = results_dir.join("timing");
    let stamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
    let path = dir.join(format!("{}_{}.json", s.name, stamp));
    if std::fs::create_dir_all(&dir).is_ok()
        && std::fs::write(&path, serde_json::to_string_pretty(&data).unwrap()).is_ok()
    {
        println!("timing json: {}", path.display());
    }
}
