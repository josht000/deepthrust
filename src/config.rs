//! Strategy configuration: yaml < named `variants:` overlay < CLI.

use serde::Deserialize;
use serde_yaml::{Mapping, Value};
use std::collections::BTreeMap;
use std::path::Path;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub data_dir: String,
    #[serde(default = "d_suffix")]
    pub data_suffix: String,
    #[serde(default)]
    pub symbols: String,
    #[serde(default)]
    pub sources: Vec<DataSourceConfig>,
    #[serde(default)]
    pub benchmark: String,
    #[serde(default)]
    pub benchmark_data_dir: String,
    #[serde(default)]
    pub benchmark_data_suffix: String,
    #[serde(default = "d_start")]
    pub start_date: String,
    #[serde(default = "d_min_history")]
    pub min_history: usize,
    #[serde(default = "d_rsi_len")]
    pub rsi_len: usize,
    #[serde(default = "d_lookback")]
    pub lookback: usize,
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
    #[serde(skip)]
    pub strategy: String,
    #[serde(skip)]
    pub symbol_cost_bps: BTreeMap<String, f64>,
    #[serde(skip)]
    pub symbol_sources: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DataSourceConfig {
    pub name: String,
    pub data_dir: String,
    #[serde(default = "d_suffix")]
    pub data_suffix: String,
    pub symbols: String,
    #[serde(default)]
    pub cost_bps_per_side: Option<f64>,
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
fn d_lookback() -> usize {
    7
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

    pub fn cost_for_symbol(&self, symbol: &str) -> f64 {
        self.symbol_cost_bps
            .get(symbol)
            .copied()
            .unwrap_or(self.cost_bps_per_side)
            / 1e4
    }

    pub fn source_list(&self) -> Vec<DataSourceConfig> {
        if self.sources.is_empty() {
            vec![DataSourceConfig {
                name: "default".into(),
                data_dir: self.data_dir.clone(),
                data_suffix: self.data_suffix.clone(),
                symbols: self.symbols.clone(),
                cost_bps_per_side: Some(self.cost_bps_per_side),
            }]
        } else {
            self.sources.clone()
        }
    }

    pub fn has_multiple_sources(&self) -> bool {
        self.sources.len() > 1
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
    if let Some(name) = variant.filter(|name| *name != "baseline") {
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
        if let Some(sources) = doc.get_mut("sources").and_then(Value::as_sequence_mut) {
            for source in sources {
                if let Some(source) = source.as_mapping_mut() {
                    source.insert("data_dir".into(), Value::String(d.clone()));
                }
            }
        } else {
            doc.insert("data_dir".into(), Value::String(d.clone()));
        }
    }
    if let Some(s) = &cli.symbols {
        if let Some(sources) = doc.get_mut("sources").and_then(Value::as_sequence_mut) {
            if sources.len() != 1 {
                return Err("--symbols override requires a single configured data source".into());
            }
            if let Some(source) = sources[0].as_mapping_mut() {
                source.insert("symbols".into(), Value::String(s.clone()));
            }
        } else {
            doc.insert("symbols".into(), Value::String(s.clone()));
        }
    }
    let mut cfg: Config = serde_yaml::from_value(Value::Mapping(doc))
        .map_err(|e| format!("bad config {path}: {e}"))?;
    cfg.strategy = strategy.to_string();
    let sources = cfg.source_list();
    if sources.iter().any(|source| source.name.trim().is_empty()) {
        return Err("data source names must not be empty".into());
    }
    let mut source_names = std::collections::BTreeSet::new();
    for source in &sources {
        if !source_names.insert(source.name.as_str()) {
            return Err(format!("duplicate data source name {:?}", source.name));
        }
        if source.data_dir.trim().is_empty() {
            return Err(format!(
                "data source {:?} has an empty data_dir",
                source.name
            ));
        }
        let symbols = split_csv(&source.symbols);
        if symbols.is_empty() {
            return Err(format!("data source {:?} has no symbols", source.name));
        }
        let cost = source.cost_bps_per_side.unwrap_or(cfg.cost_bps_per_side);
        if !cost.is_finite() || cost < 0.0 {
            return Err(format!(
                "data source {:?} has invalid cost_bps_per_side {cost}",
                source.name
            ));
        }
        for symbol in symbols {
            if let Some(previous) = cfg
                .symbol_sources
                .insert(symbol.clone(), source.name.clone())
            {
                if previous == source.name {
                    return Err(format!(
                        "symbol {symbol:?} is listed more than once in data source {:?}",
                        source.name
                    ));
                }
                return Err(format!(
                    "symbol {symbol:?} appears in multiple data sources"
                ));
            }
            cfg.symbol_cost_bps.insert(symbol, cost);
        }
    }
    if cfg.symbol_sources.is_empty() {
        return Err("config must provide at least one symbol".into());
    }
    if cfg.lookback == 0 {
        return Err("lookback must be greater than zero".into());
    }
    cfg.variant = variant.unwrap_or("baseline").to_string();
    cfg.variant_overlay = overlay_text;
    Ok(cfg)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn write_config(name: &str, contents: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("deepthrust_config_{}_{}", std::process::id(), name));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("strategy.yaml");
        std::fs::write(&path, contents).unwrap();
        path
    }

    fn cli() -> Cli {
        Cli {
            data_dir: None,
            symbols: None,
        }
    }

    #[test]
    fn mixed_sources_preserve_symbol_costs_and_source_names() {
        let path = write_config(
            "mixed",
            r#"
benchmark: SPY
partition_split_date: "2024-01-01"
cost_bps_per_side: 10
sources:
  - name: etfs
    data_dir: data/etfs
    data_suffix: _tr.parquet
    symbols: SPY,QQQ
    cost_bps_per_side: 5
  - name: stocks
    data_dir: data/stocks
    data_suffix: _tr.parquet
    symbols: AAPL
    cost_bps_per_side: 15
"#,
        );
        let cfg = load_config("s-test", Some(path.to_str().unwrap()), &cli(), None).unwrap();
        assert_eq!(cfg.strategy, "s-test");
        assert_eq!(cfg.symbol_sources["SPY"], "etfs");
        assert_eq!(cfg.symbol_sources["AAPL"], "stocks");
        assert_eq!(cfg.cost_for_symbol("SPY"), 0.0005);
        assert_eq!(cfg.cost_for_symbol("AAPL"), 0.0015);
    }

    #[test]
    fn mixed_sources_reject_duplicate_symbols() {
        let path = write_config(
            "duplicates",
            r#"
partition_split_date: "2024-01-01"
sources:
  - name: etfs
    data_dir: data/etfs
    symbols: SPY
  - name: stocks
    data_dir: data/stocks
    symbols: SPY
"#,
        );
        let err = load_config("s-test", Some(path.to_str().unwrap()), &cli(), None).unwrap_err();
        assert!(err.contains("appears in multiple data sources"), "{err}");
    }

    #[test]
    fn source_rejects_repeated_symbols() {
        let path = write_config(
            "repeated",
            r#"
partition_split_date: "2024-01-01"
sources:
  - name: etfs
    data_dir: data/etfs
    symbols: SPY,SPY
"#,
        );
        let err = load_config("s-test", Some(path.to_str().unwrap()), &cli(), None).unwrap_err();
        assert!(err.contains("listed more than once"), "{err}");
    }

    #[test]
    fn baseline_is_the_default_and_unknown_variant_is_rejected() {
        let path = write_config(
            "variants",
            r#"
data_dir: data
symbols: SPY
partition_split_date: "2024-01-01"
variants:
  alternate: {rsi_threshold: 20}
"#,
        );
        let baseline = load_config("s-test", Some(path.to_str().unwrap()), &cli(), None).unwrap();
        assert_eq!(baseline.variant, "baseline");
        assert!(load_config(
            "s-test",
            Some(path.to_str().unwrap()),
            &cli(),
            Some("missing")
        )
        .is_err());
    }

    #[test]
    fn implemented_strategies_have_at_least_30_loadable_variants() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("configs");
        for strategy in ["s001_rsi2_connors", "s003_rsi4_2575", "s006_double_7s"] {
            let path = root.join(format!("{strategy}.yaml"));
            let text = std::fs::read_to_string(&path).unwrap();
            let doc: Value = serde_yaml::from_str(&text).unwrap();
            let variants = doc
                .get("variants")
                .and_then(Value::as_mapping)
                .unwrap_or_else(|| panic!("{strategy} has no variants"));
            assert!(
                variants.len() >= 30,
                "{strategy} has only {} named variants",
                variants.len()
            );
            for name in variants.keys().map(|key| key.as_str().unwrap()) {
                load_config(strategy, Some(path.to_str().unwrap()), &cli(), Some(name))
                    .unwrap_or_else(|e| panic!("{strategy}/{name}: {e}"));
            }
        }
    }
}
