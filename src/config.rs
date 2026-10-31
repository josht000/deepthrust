//! Strategy configuration: yaml < named `variants:` overlay < CLI.

use serde::Deserialize;
use serde_yaml::{Mapping, Value};
use std::path::Path;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub data_dir: String,
    #[serde(default = "d_suffix")]
    pub data_suffix: String,
    pub symbols: String,
    #[serde(default)]
    pub benchmark: String,
    #[serde(default = "d_start")]
    pub start_date: String,
    #[serde(default = "d_min_history")]
    pub min_history: usize,
    #[serde(default = "d_rsi_len")]
    pub rsi_len: usize,
    #[serde(default = "d_rsi_thr")]
    pub rsi_threshold: f64,
    #[serde(default = "d_gate")]
    pub gate: String,
    #[serde(default = "d_sma_window")]
    pub sma_window: usize,
    #[serde(default = "d_gate_ret")]
    pub gate_ret_window: usize,
    #[serde(default = "d_exit_mode")]
    pub exit_mode: String,
    #[serde(default = "d_exit_sma")]
    pub exit_sma: usize,
    #[serde(default = "d_exit_rsi")]
    pub exit_rsi: f64,
    #[serde(default = "d_sizing")]
    pub sizing: String,
    #[serde(default = "d_add_rsi")]
    pub add_rsi: f64,
    #[serde(default = "d_max_units")]
    pub max_units: usize,
    #[serde(default)]
    pub disaster_stop_atr: f64,
    #[serde(default = "d_atr_len")]
    pub atr_len: usize,
    #[serde(default = "d_cost")]
    pub cost_bps_per_side: f64,
    #[serde(default = "d_fills")]
    pub fills: String,
    pub partition_split_date: String,
    #[serde(default = "d_draws")]
    pub n_draws: usize,
    #[serde(default = "d_boot")]
    pub n_boot: usize,
    #[serde(default = "d_timing_draws")]
    pub n_timing_draws: usize,
    #[serde(default = "d_seed")]
    pub seed: u64,
    #[serde(default = "d_slots")]
    pub book_slots: usize,
    #[serde(default)]
    pub results_tag: String,
    /// Resolved variant name ("baseline" when no overlay is applied).
    #[serde(skip, default = "d_baseline")]
    pub variant: String,
    #[serde(skip)]
    pub variant_overlay: String,
}

fn d_suffix() -> String {
    "_1Day_split.parquet".into()
}
fn d_start() -> String {
    "2005-01-01".into()
}
fn d_min_history() -> usize {
    250
}
fn d_rsi_len() -> usize {
    2
}
fn d_rsi_thr() -> f64 {
    10.0
}
fn d_gate() -> String {
    "sma200".into()
}
fn d_sma_window() -> usize {
    200
}
fn d_gate_ret() -> usize {
    21
}
fn d_exit_mode() -> String {
    "sma5".into()
}
fn d_exit_sma() -> usize {
    5
}
fn d_exit_rsi() -> f64 {
    70.0
}
fn d_sizing() -> String {
    "single".into()
}
fn d_add_rsi() -> f64 {
    5.0
}
fn d_max_units() -> usize {
    3
}
fn d_atr_len() -> usize {
    14
}
fn d_cost() -> f64 {
    5.0
}
fn d_fills() -> String {
    "next_open,close".into()
}
fn d_draws() -> usize {
    1000
}
fn d_boot() -> usize {
    5000
}
fn d_timing_draws() -> usize {
    500
}
fn d_seed() -> u64 {
    20261101
}
fn d_slots() -> usize {
    1
}
fn d_baseline() -> String {
    "baseline".into()
}

impl Config {
    pub fn symbol_list(&self) -> Vec<String> {
        split_csv(&self.symbols)
    }
    pub fn fills_list(&self) -> Vec<String> {
        split_csv(&self.fills)
    }
    /// Filename prefix for outputs of non-baseline variants (`<variant>__`).
    pub fn prefix(&self) -> String {
        if self.results_tag.is_empty() {
            String::new()
        } else {
            format!("{}__", self.results_tag)
        }
    }
    pub fn cost(&self) -> f64 {
        self.cost_bps_per_side / 1e4
    }
}

pub fn split_csv(s: &str) -> Vec<String> {
    s.split(',')
        .map(|x| x.trim().to_string())
        .filter(|x| !x.is_empty())
        .collect()
}

pub struct Cli {
    pub data_dir: Option<String>,
    pub symbols: Option<String>,
}

/// Load one strategy config.
pub fn load_config(
    strategy: &str,
    path: Option<&str>,
    cli: &Cli,
    variant: Option<&str>,
) -> Result<Config, String> {
    let default = format!("configs/{strategy}.yaml");
    let path = path.unwrap_or(&default);
    let text = std::fs::read_to_string(Path::new(path))
        .map_err(|e| format!("cannot read config {path}: {e}"))?;
    let mut doc: Mapping =
        serde_yaml::from_str(&text).map_err(|e| format!("bad yaml in {path}: {e}"))?;
    let variants = doc
        .remove("variants")
        .and_then(|v| v.as_mapping().cloned())
        .unwrap_or_default();
    let mut overlay_text = String::new();
    if let Some(name) = variant {
        let overlay = variants
            .get(name)
            .and_then(|v| v.as_mapping())
            .ok_or_else(|| {
                format!(
                    "unknown variant {name:?}; choose from {}",
                    variants
                        .keys()
                        .filter_map(|k| k.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            })?;
        for (k, v) in overlay {
            doc.insert(k.clone(), v.clone());
        }
        doc.insert("results_tag".into(), Value::String(name.to_string()));
        overlay_text = serde_yaml::to_string(overlay)
            .unwrap_or_default()
            .trim()
            .replace('\n', ", ");
    }
    if let Some(d) = &cli.data_dir {
        doc.insert("data_dir".into(), Value::String(d.clone()));
    }
    if let Some(s) = &cli.symbols {
        doc.insert("symbols".into(), Value::String(s.clone()));
    }
    let mut cfg: Config = serde_yaml::from_value(Value::Mapping(doc))
        .map_err(|e| format!("bad config {path}: {e}"))?;
    cfg.variant = variant.unwrap_or("baseline").to_string();
    cfg.variant_overlay = overlay_text;
    Ok(cfg)
}
