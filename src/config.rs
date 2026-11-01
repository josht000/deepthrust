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
    pub exclude_symbols: String,
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
    #[serde(default = "d_cost_ladder")]
    pub cost_ladder_multipliers: Vec<f64>,
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
    #[serde(default = "d_participation_filter")]
    pub participation_filter: String,
    #[serde(default = "d_target_vol")]
    pub target_vol: f64,
    #[serde(default = "d_vol_window")]
    pub vol_window: usize,
    #[serde(default = "d_max_leverage")]
    pub max_leverage: f64,
    #[serde(default = "d_resize_band")]
    pub resize_band: f64,
    #[serde(default = "d_gap_atr_multiple")]
    pub gap_atr_multiple: f64,
    #[serde(default = "d_gap_volume_multiple")]
    pub gap_volume_multiple: f64,
    #[serde(default = "d_gap_volume_window")]
    pub gap_volume_window: usize,
    #[serde(default = "d_gap_volume_mode")]
    pub gap_volume_mode: String,
    #[serde(default = "d_gap_stop_mode")]
    pub gap_stop_mode: String,
    #[serde(default = "d_gap_reward_risk")]
    pub gap_reward_risk: f64,
    #[serde(default = "d_gap_max_hold")]
    pub gap_max_hold: usize,
    #[serde(default = "d_universe_top_n")]
    pub universe_top_n: usize,
    #[serde(default = "d_selection_mode")]
    pub selection_mode: String,
    #[serde(default = "d_ratio_floor")]
    pub ratio_floor: f64,
    #[serde(default = "d_holding_months")]
    pub holding_months: usize,
    #[serde(default)]
    pub formation_skip_months: usize,
    #[serde(default)]
    pub sector_neutral: bool,
    #[serde(default = "d_peer_top_n")]
    pub peer_top_n: usize,
    #[serde(default = "d_peer_level")]
    pub peer_level: String,
    #[serde(default = "d_min_peers")]
    pub min_peers: usize,
    #[serde(default = "d_beta_window")]
    pub beta_window: usize,
    #[serde(default = "d_beta_min_obs")]
    pub beta_min_obs: usize,
    #[serde(default = "d_resid_window")]
    pub resid_window: usize,
    #[serde(default = "d_max_hold")]
    pub max_hold: usize,
    #[serde(default = "d_entry_window")]
    pub entry_window: usize,
    #[serde(default = "d_exit_window")]
    pub exit_window: usize,
    #[serde(default = "d_rank_window")]
    pub rank_window: usize,
    #[serde(default = "d_rank_min_observations")]
    pub rank_min_observations: usize,
    #[serde(default = "d_min_price")]
    pub min_price: f64,
    #[serde(default = "d_mom_window")]
    pub mom_window: usize,
    #[serde(default = "d_skip_window")]
    pub skip_window: usize,
    #[serde(default = "d_breadth_mode")]
    pub breadth_mode: String,
    #[serde(default = "d_breadth")]
    pub breadth: usize,
    #[serde(default = "d_min_breadth")]
    pub min_breadth: usize,
    #[serde(default = "d_decile")]
    pub decile: f64,
    #[serde(default)]
    pub industry_neutral: bool,
    #[serde(default = "d_rebalance")]
    pub rebalance: String,
    #[serde(default = "d_signal")]
    pub signal: String,
    #[serde(default = "d_signal_mode")]
    pub signal_mode: String,
    #[serde(default = "d_z_window")]
    pub z_window: usize,
    #[serde(default)]
    pub min_abs_resid: f64,
    #[serde(default)]
    pub skip_days: usize,
    #[serde(default = "d_fixed_hold")]
    pub fixed_hold: usize,
    #[serde(default)]
    pub exit_z: f64,
    #[serde(default)]
    pub stop_loss: f64,
    #[serde(default)]
    pub take_profit: f64,
    #[serde(default = "d_regime_mode")]
    pub regime_mode: String,
    #[serde(default = "d_regime_window")]
    pub regime_window: usize,
    #[serde(default = "d_regime_threshold")]
    pub regime_threshold: f64,
    #[serde(default)]
    pub simfin_dir: String,
    #[serde(default = "d_formation_cadence")]
    pub formation_cadence: String,
    #[serde(default = "d_formation_window")]
    pub formation_window: usize,
    #[serde(default = "d_min_t_stat")]
    pub min_t_stat: f64,
    #[serde(default = "d_fdr_q")]
    pub fdr_q: f64,
    #[serde(default = "d_top_fraction")]
    pub top_fraction: f64,
    #[serde(default = "d_extra_filter")]
    pub extra_filter: String,
    #[serde(default = "d_holding_mode")]
    pub holding_mode: String,
    #[serde(default)]
    pub corporate_actions_file: String,
    #[serde(default)]
    pub etf_data_dir: String,
    #[serde(default)]
    pub etf_symbols: String,
    #[serde(default)]
    pub include_stocks: bool,
    #[serde(default)]
    pub rank_source: String,
    #[serde(default)]
    pub rank_cache_dir: String,
    #[serde(default)]
    pub sp500_membership_file: String,
    #[serde(default)]
    pub earnings_dates_file: String,
    #[serde(default)]
    pub s023_eps_file: String,
    #[serde(default)]
    pub s024_min_yield: f64,
    #[serde(default)]
    pub industry_data_file: String,
    #[serde(default)]
    pub market_factor_file: String,
    #[serde(default)]
    pub weight_grid_path: String,
    #[serde(default = "d_breakout_distance_atr")]
    pub breakout_distance_atr: f64,
    #[serde(default = "d_gap_guard_atr")]
    pub gap_guard_atr: f64,
    #[serde(default = "d_max_target_vol")]
    pub max_target_vol: f64,
    #[serde(default = "d_analog_count")]
    pub analog_count: usize,
    #[serde(default = "d_borrow_bps_annual")]
    pub borrow_bps_annual: f64,
    #[serde(default = "d_factor_design")]
    pub factor_design: String,
    #[serde(default = "d_earnings_window_sessions")]
    pub earnings_window_sessions: usize,
    #[serde(default = "d_pullback_days")]
    pub pullback_days: usize,
    #[serde(default = "d_pullback_distance")]
    pub pullback_distance: f64,
    #[serde(default = "d_periods_per_year")]
    pub periods_per_year: usize,
    #[serde(default = "d_defensive_asset")]
    pub defensive_asset: String,
    #[serde(default = "d_strategy_mode")]
    pub strategy_mode: String,
    #[serde(default)]
    pub delisting_return: f64,
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
fn d_cost_ladder() -> Vec<f64> {
    vec![0.5, 1.0, 2.0, 3.0]
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
fn d_participation_filter() -> String {
    "none".into()
}
fn d_target_vol() -> f64 {
    0.10
}
fn d_vol_window() -> usize {
    20
}
fn d_max_leverage() -> f64 {
    1.0
}
fn d_resize_band() -> f64 {
    0.20
}
fn d_gap_atr_multiple() -> f64 {
    1.0
}
fn d_gap_volume_multiple() -> f64 {
    2.0
}
fn d_gap_volume_window() -> usize {
    20
}
fn d_gap_volume_mode() -> String {
    "high".into()
}
fn d_gap_stop_mode() -> String {
    "prior_close".into()
}
fn d_gap_reward_risk() -> f64 {
    2.0
}
fn d_gap_max_hold() -> usize {
    5
}
fn d_universe_top_n() -> usize {
    500
}
fn d_selection_mode() -> String {
    "rank".into()
}
fn d_ratio_floor() -> f64 {
    0.8
}
fn d_holding_months() -> usize {
    1
}
fn d_peer_top_n() -> usize {
    1500
}
fn d_peer_level() -> String {
    "sector".into()
}
fn d_min_peers() -> usize {
    5
}
fn d_beta_window() -> usize {
    60
}
fn d_beta_min_obs() -> usize {
    40
}
fn d_resid_window() -> usize {
    5
}
fn d_max_hold() -> usize {
    10
}
fn d_entry_window() -> usize {
    252
}
fn d_exit_window() -> usize {
    126
}
fn d_rank_window() -> usize {
    60
}
fn d_rank_min_observations() -> usize {
    40
}
fn d_min_price() -> f64 {
    5.0
}
fn d_mom_window() -> usize {
    252
}
fn d_skip_window() -> usize {
    21
}
fn d_breadth_mode() -> String {
    "decile".into()
}
fn d_breadth() -> usize {
    50
}
fn d_min_breadth() -> usize {
    1
}
fn d_decile() -> f64 {
    0.10
}
fn d_rebalance() -> String {
    "monthly".into()
}
fn d_signal() -> String {
    "mom12".into()
}
fn d_signal_mode() -> String {
    "raw".into()
}
fn d_formation_cadence() -> String {
    "annual".into()
}
fn d_formation_window() -> usize {
    252
}
fn d_min_t_stat() -> f64 {
    2.0
}
fn d_fdr_q() -> f64 {
    0.05
}
fn d_top_fraction() -> f64 {
    0.20
}
fn d_extra_filter() -> String {
    "none".into()
}
fn d_holding_mode() -> String {
    "overnight_only".into()
}
fn d_z_window() -> usize {
    60
}
fn d_fixed_hold() -> usize {
    5
}
fn d_regime_mode() -> String {
    "none".into()
}
fn d_regime_window() -> usize {
    20
}
fn d_regime_threshold() -> f64 {
    0.10
}
fn d_baseline() -> String {
    "baseline".into()
}
fn d_breakout_distance_atr() -> f64 {
    0.1
}
fn d_gap_guard_atr() -> f64 {
    1.0
}
fn d_max_target_vol() -> f64 {
    0.50
}
fn d_analog_count() -> usize {
    50
}
fn d_borrow_bps_annual() -> f64 {
    300.0
}
fn d_factor_design() -> String {
    "S045a".into()
}
fn d_earnings_window_sessions() -> usize {
    5
}
fn d_pullback_days() -> usize {
    3
}
fn d_pullback_distance() -> f64 {
    0.02
}
fn d_periods_per_year() -> usize {
    252
}
fn d_defensive_asset() -> String {
    "BIL".into()
}
fn d_strategy_mode() -> String {
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

fn expand_wildcard_symbols(doc: &mut Mapping) -> Result<(), String> {
    fn expand(
        dir: &str,
        suffix: &str,
        excluded: &[String],
        value: &mut Value,
    ) -> Result<(), String> {
        if value.as_str() != Some("*") {
            return Ok(());
        }
        let root = Path::new(dir);
        let entries = std::fs::read_dir(root)
            .map_err(|error| format!("cannot discover symbols in {}: {error}", root.display()))?;
        let mut symbols = Vec::new();
        for entry in entries {
            let entry = entry.map_err(|error| format!("{}: {error}", root.display()))?;
            if !entry
                .file_type()
                .map_err(|error| format!("{}: {error}", entry.path().display()))?
                .is_file()
            {
                continue;
            }
            let name = entry.file_name().to_string_lossy().into_owned();
            if let Some(symbol) = name.strip_suffix(suffix) {
                if !symbol.is_empty() && !excluded.iter().any(|excluded| excluded == symbol) {
                    symbols.push(symbol.to_string());
                }
            }
        }
        symbols.sort();
        symbols.dedup();
        if symbols.is_empty() {
            return Err(format!(
                "symbol wildcard matched no *{suffix} files in {}",
                root.display()
            ));
        }
        *value = Value::String(symbols.join(","));
        Ok(())
    }
    if let Some(sources) = doc.get_mut("sources").and_then(Value::as_sequence_mut) {
        for source in sources {
            let Some(map) = source.as_mapping_mut() else {
                continue;
            };
            let dir = map
                .get("data_dir")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string();
            let suffix = map
                .get("data_suffix")
                .and_then(Value::as_str)
                .map(str::to_string)
                .unwrap_or_else(d_suffix);
            let excluded = map
                .get("exclude_symbols")
                .and_then(Value::as_str)
                .map(split_csv)
                .unwrap_or_default();
            if let Some(symbols) = map.get_mut("symbols") {
                expand(&dir, &suffix, &excluded, symbols)?;
            }
        }
    } else if doc.get("symbols").and_then(Value::as_str) == Some("*") {
        let dir = doc
            .get("data_dir")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
        let suffix = doc
            .get("data_suffix")
            .and_then(Value::as_str)
            .map(str::to_string)
            .unwrap_or_else(d_suffix);
        let excluded = doc
            .get("exclude_symbols")
            .and_then(Value::as_str)
            .map(split_csv)
            .unwrap_or_default();
        if let Some(symbols) = doc.get_mut("symbols") {
            expand(&dir, &suffix, &excluded, symbols)?;
        }
    }
    Ok(())
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
    expand_wildcard_symbols(&mut doc)?;
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
    if cfg.periods_per_year == 0 {
        return Err("periods_per_year must be greater than zero".into());
    }
    if cfg.cost_ladder_multipliers.is_empty()
        || cfg
            .cost_ladder_multipliers
            .iter()
            .any(|multiplier| !multiplier.is_finite() || *multiplier <= 0.0)
    {
        return Err("cost_ladder_multipliers must contain positive finite values".into());
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
    fn source_wildcard_discovers_only_files_with_configured_suffix() {
        let path = write_config(
            "wildcard",
            r#"
partition_split_date: "2024-01-01"
sources:
  - name: etfs
    data_dir: PLACEHOLDER
    data_suffix: _1Day_tr.parquet
    exclude_symbols: CASH
    symbols: "*"
"#,
        );
        let dir = path.parent().unwrap();
        std::fs::write(dir.join("SPY_1Day_tr.parquet"), []).unwrap();
        std::fs::write(dir.join("QQQ_1Day_tr.parquet"), []).unwrap();
        std::fs::write(dir.join("CASH_1Day_tr.parquet"), []).unwrap();
        std::fs::write(dir.join("notes.csv"), []).unwrap();
        std::fs::create_dir(dir.join("nested_1Day_tr.parquet")).unwrap();
        let text = std::fs::read_to_string(&path)
            .unwrap()
            .replace("PLACEHOLDER", dir.to_str().unwrap());
        std::fs::write(&path, text).unwrap();

        let cfg = load_config("s-test", Some(path.to_str().unwrap()), &cli(), None).unwrap();
        assert_eq!(cfg.sources[0].symbols, "QQQ,SPY");
    }

    #[test]
    fn wildcard_rejects_empty_match() {
        let path = write_config(
            "wildcard_empty",
            r#"
partition_split_date: "2024-01-01"
data_dir: PLACEHOLDER
data_suffix: _1Day_tr.parquet
symbols: "*"
"#,
        );
        let dir = path.parent().unwrap();
        let text = std::fs::read_to_string(&path)
            .unwrap()
            .replace("PLACEHOLDER", dir.to_str().unwrap());
        std::fs::write(&path, text).unwrap();
        let err = load_config("s-test", Some(path.to_str().unwrap()), &cli(), None).unwrap_err();
        assert!(err.contains("matched no *_1Day_tr.parquet files"), "{err}");
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
        let wildcard_dir =
            std::env::temp_dir().join(format!("deepthrust_variant_symbols_{}", std::process::id()));
        std::fs::create_dir_all(&wildcard_dir).unwrap();
        std::fs::write(wildcard_dir.join("AAA.parquet"), []).unwrap();
        for strategy in [
            "s001_rsi2_connors",
            "s003_rsi4_2575",
            "s006_double_7s",
            "s017_night_trading",
            "s033_52w_breakout",
            "s041_mom_12_1",
            "s021_index_overnight",
            "s042_sector_momentum",
            "s075_residual_reversal",
            "s105_overnight_carry",
            "s024_ex_dividend_capture",
        ] {
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
            if strategy == "s017_night_trading" {
                std::fs::write(wildcard_dir.join("QQQ.parquet"), []).unwrap();
                std::fs::write(wildcard_dir.join("cash_dividends.parquet"), []).unwrap();
            }
            for name in variants.keys().map(|key| key.as_str().unwrap()) {
                let test_cli = if matches!(
                    strategy,
                    "s017_night_trading"
                        | "s033_52w_breakout"
                        | "s041_mom_12_1"
                        | "s075_residual_reversal"
                ) {
                    Cli {
                        data_dir: Some(wildcard_dir.display().to_string()),
                        symbols: None,
                    }
                } else {
                    cli()
                };
                let loaded = load_config(
                    strategy,
                    Some(path.to_str().unwrap()),
                    &test_cli,
                    Some(name),
                )
                .unwrap_or_else(|e| panic!("{strategy}/{name}: {e}"));
                if strategy == "s017_night_trading" {
                    let symbols = split_csv(&loaded.symbols);
                    assert!(symbols.iter().any(|symbol| symbol == "AAA"));
                    assert!(!symbols.iter().any(|symbol| symbol == "QQQ"));
                    assert!(!symbols.iter().any(|symbol| symbol == "cash_dividends"));
                }
            }
            if strategy == "s017_night_trading" {
                std::fs::remove_file(wildcard_dir.join("QQQ.parquet")).unwrap();
                std::fs::remove_file(wildcard_dir.join("cash_dividends.parquet")).unwrap();
            }
        }
        let _ = std::fs::remove_dir_all(wildcard_dir);
    }

    #[test]
    fn overnight_strategies_and_variants_load() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("configs");
        for strategy in ["s021_index_overnight", "s105_overnight_carry"] {
            let path = root.join(format!("{strategy}.yaml"));
            let text = std::fs::read_to_string(&path).unwrap();
            let doc: Value = serde_yaml::from_str(&text).unwrap();
            let variants = doc
                .get("variants")
                .and_then(Value::as_mapping)
                .unwrap_or_else(|| panic!("{strategy} has no variants"));
            let baseline = load_config(strategy, Some(path.to_str().unwrap()), &cli(), None)
                .unwrap_or_else(|error| panic!("{strategy}: {error}"));
            assert_eq!(baseline.cost_ladder_multipliers, [0.5, 1.0, 2.0, 5.0]);
            assert!(
                variants.len() >= 30,
                "{strategy} has only {} named variants",
                variants.len()
            );
            for name in variants.keys().map(|key| key.as_str().unwrap()) {
                load_config(strategy, Some(path.to_str().unwrap()), &cli(), Some(name))
                    .unwrap_or_else(|error| panic!("{strategy}/{name}: {error}"));
            }
        }
    }
}
