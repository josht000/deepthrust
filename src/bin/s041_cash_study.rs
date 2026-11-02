use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use chrono::Datelike;
use clap::Parser;
use deepthrust::config::{self, Cli, Config};
use deepthrust::data::{self, Bars, Panel};
use deepthrust::s041_cash_study::{
    run_cash_replay_with_spinoffs, CashExecution, CashReplay, SpinOffAction, SpinOffDistribution,
};
use deepthrust::signals::{self, Signals};
use deepthrust::stats::{self, Rng};
use plotters::prelude::*;
use serde::{Deserialize, Serialize};
use serde_yaml::{Mapping, Value};
use sha2::{Digest, Sha256};

#[derive(Parser)]
#[command(about = "Run the separate S041 whole-share cash-account study")]
struct Args {
    #[arg(long, default_value = "configs/s041_cash.yaml")]
    config: String,
    #[arg(long)]
    variant: Option<String>,
    #[arg(long)]
    data_dir: Option<String>,
    #[arg(long)]
    raw_data_dir: Option<String>,
    #[arg(long)]
    symbols: Option<String>,
    #[arg(long)]
    random_draws: Option<usize>,
    #[arg(long)]
    check_data: bool,
}

fn write_price_gaps(
    path: &Path,
    gaps: &BTreeSet<(chrono::NaiveDate, String)>,
) -> Result<(), String> {
    let mut writer =
        csv::Writer::from_path(path).map_err(|error| format!("{}: {error}", path.display()))?;
    writer
        .write_record(["date", "symbol"])
        .map_err(|error| error.to_string())?;
    for (date, symbol) in gaps {
        writer
            .write_record([date.to_string(), symbol.clone()])
            .map_err(|error| error.to_string())?;
    }
    writer.flush().map_err(|error| error.to_string())
}

fn write_missing_raw_symbols(
    path: &Path,
    symbols: &[String],
    raw_dir: &str,
    suffix: &str,
) -> Result<(), String> {
    let mut writer =
        csv::Writer::from_path(path).map_err(|error| format!("{}: {error}", path.display()))?;
    writer
        .write_record(["symbol", "expected_raw_file", "status"])
        .map_err(|error| error.to_string())?;
    for symbol in symbols {
        writer
            .write_record([
                symbol.clone(),
                data::input_path(raw_dir, symbol, suffix)
                    .display()
                    .to_string(),
                "missing; adjusted prices are not substituted".into(),
            ])
            .map_err(|error| error.to_string())?;
    }
    writer.flush().map_err(|error| error.to_string())
}

#[derive(Clone, Debug, Deserialize)]
#[serde(default)]
struct CashSettings {
    raw_data_dir: String,
    raw_data_suffix: String,
    raw_price_basis: String,
    corporate_actions_file: String,
    cash_initial_cash: f64,
    cash_long_fraction: f64,
    cash_short_fraction: f64,
    cash_short_count: usize,
    cash_max_participation: f64,
    cash_max_liquidity_age_seconds: i64,
    cash_max_pending_sessions: usize,
    cash_max_stale_mark_sessions: usize,
    cash_dca_installments: usize,
    cash_stop_loss_fraction: f64,
    cash_pyramid_profit_fraction: f64,
    cash_pyramid_add_fraction: f64,
    cash_slippage_bps: f64,
    cash_commission_per_share: f64,
    cash_annual_borrow_rate: f64,
    cash_delisting_return: f64,
    cash_spinoffs: Vec<CashSpinOffSettings>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct CashSpinOffSettings {
    date: String,
    source_symbol: String,
    distributions: Vec<CashSpinOffDistributionSettings>,
    source_url: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct CashSpinOffDistributionSettings {
    symbol: String,
    shares_per_source_share: f64,
}

impl Default for CashSettings {
    fn default() -> Self {
        Self {
            raw_data_dir: String::new(),
            raw_data_suffix: ".parquet".into(),
            raw_price_basis: String::new(),
            corporate_actions_file: String::new(),
            cash_initial_cash: 10_000.0,
            cash_long_fraction: 1.0,
            cash_short_fraction: 0.0,
            cash_short_count: 0,
            cash_max_participation: 0.01,
            cash_max_liquidity_age_seconds: 691_200,
            cash_max_pending_sessions: 5,
            cash_max_stale_mark_sessions: 3,
            cash_dca_installments: 1,
            cash_stop_loss_fraction: 0.0,
            cash_pyramid_profit_fraction: 0.0,
            cash_pyramid_add_fraction: 0.25,
            cash_slippage_bps: 0.0,
            cash_commission_per_share: 0.0,
            cash_annual_borrow_rate: 0.0,
            cash_delisting_return: -0.30,
            cash_spinoffs: Vec::new(),
        }
    }
}

#[derive(Clone, Debug)]
struct Metrics {
    total_log: f64,
    trend_log: f64,
    random_pick_log: f64,
    skill: f64,
    bias: f64,
    skill_pct: f64,
    sharpe: f64,
    profit_factor: f64,
    max_drawdown_pct: f64,
    trades: usize,
    win_pct: f64,
}

struct TrialResult {
    variant: String,
    dev: Metrics,
    test: Metrics,
    benchmark_dev_drawdown_pct: f64,
}

struct BlockedVariant {
    variant: String,
    reason: String,
}

enum VariantOutcome {
    Completed(TrialResult),
    Blocked(BlockedVariant),
    CheckData,
}

struct LoadedData {
    signal_panel: Panel,
    raw_panel: Panel,
    signal_adjustments: Vec<serde_json::Value>,
    benchmark: BTreeMap<chrono::NaiveDate, f64>,
    actions: Vec<data::CorporateAction>,
    missing_raw: Vec<String>,
    input_files: Vec<PathBuf>,
    identity: InputIdentity,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct InputIdentity {
    signal_data_dir: String,
    signal_data_suffix: String,
    symbols: String,
    benchmark: String,
    benchmark_data_dir: String,
    benchmark_data_suffix: String,
    raw_data_dir: String,
    raw_data_suffix: String,
    corporate_actions_file: String,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct SignalKey {
    mom_window: usize,
    skip_window: usize,
    rank_window: usize,
    rank_min_observations: usize,
    min_price_bits: u64,
    min_history: usize,
    universe_top_n: usize,
    rank_cache_dir: String,
    industry_neutral: bool,
    simfin_dir: String,
}

fn signal_key(cfg: &Config) -> SignalKey {
    SignalKey {
        mom_window: cfg.mom_window,
        skip_window: cfg.skip_window,
        rank_window: cfg.rank_window,
        rank_min_observations: cfg.rank_min_observations,
        min_price_bits: cfg.min_price.to_bits(),
        min_history: cfg.min_history,
        universe_top_n: cfg.universe_top_n,
        rank_cache_dir: cfg.rank_cache_dir.clone(),
        industry_neutral: cfg.industry_neutral,
        simfin_dir: cfg.simfin_dir.clone(),
    }
}

fn input_identity(cfg: &Config, settings: &CashSettings) -> InputIdentity {
    InputIdentity {
        signal_data_dir: cfg.data_dir.clone(),
        signal_data_suffix: cfg.data_suffix.clone(),
        symbols: cfg.symbols.clone(),
        benchmark: cfg.benchmark.clone(),
        benchmark_data_dir: cfg.benchmark_data_dir.clone(),
        benchmark_data_suffix: cfg.benchmark_data_suffix.clone(),
        raw_data_dir: settings.raw_data_dir.clone(),
        raw_data_suffix: settings.raw_data_suffix.clone(),
        corporate_actions_file: settings.corporate_actions_file.clone(),
    }
}

fn variants(value: Option<&str>) -> Vec<Option<String>> {
    let mut names = config::split_csv(value.unwrap_or(""));
    if value.is_some() && !names.is_empty() && !names.iter().any(|name| name == "baseline") {
        names.insert(0, "baseline".into());
    }
    if names.is_empty() {
        vec![None]
    } else {
        names
            .into_iter()
            .map(|name| (name != "baseline").then_some(name))
            .collect()
    }
}

fn load_cash_settings(
    path: &str,
    variant: Option<&str>,
    raw_dir_override: Option<&str>,
) -> Result<CashSettings, String> {
    let text = fs::read_to_string(path).map_err(|error| format!("cannot read {path}: {error}"))?;
    let mut doc: Mapping =
        serde_yaml::from_str(&text).map_err(|error| format!("bad YAML in {path}: {error}"))?;
    let overlays = doc
        .remove(Value::String("variants".into()))
        .and_then(|value| value.as_mapping().cloned())
        .unwrap_or_default();
    if let Some(name) = variant {
        if let Some(overlay) = overlays
            .get(Value::String(name.into()))
            .and_then(Value::as_mapping)
        {
            for (key, value) in overlay {
                doc.insert(key.clone(), value.clone());
            }
        }
    }
    if let Some(path) = raw_dir_override {
        doc.insert(
            Value::String("raw_data_dir".into()),
            Value::String(path.into()),
        );
    }
    let settings: CashSettings = serde_yaml::from_value(Value::Mapping(doc))
        .map_err(|error| format!("bad cash settings in {path}: {error}"))?;
    validate_settings(&settings)?;
    Ok(settings)
}

fn validate_settings(settings: &CashSettings) -> Result<(), String> {
    if settings.raw_data_dir.trim().is_empty()
        || settings.raw_data_suffix.trim().is_empty()
        || settings.raw_price_basis != "raw_unadjusted"
        || settings.corporate_actions_file.trim().is_empty()
        || !settings.cash_initial_cash.is_finite()
        || settings.cash_initial_cash <= 0.0
        || !settings.cash_long_fraction.is_finite()
        || !settings.cash_short_fraction.is_finite()
        || settings.cash_long_fraction < 0.0
        || settings.cash_short_fraction < 0.0
        || settings.cash_long_fraction + settings.cash_short_fraction > 1.0
        || (settings.cash_short_fraction > 0.0) != (settings.cash_short_count > 0)
        || !settings.cash_max_participation.is_finite()
        || settings.cash_max_participation <= 0.0
        || settings.cash_max_participation > 1.0
        || settings.cash_max_liquidity_age_seconds <= 0
        || settings.cash_max_pending_sessions == 0
        || settings.cash_max_stale_mark_sessions == 0
        || settings.cash_dca_installments == 0
        || !settings.cash_stop_loss_fraction.is_finite()
        || !(0.0..1.0).contains(&settings.cash_stop_loss_fraction)
        || !settings.cash_pyramid_profit_fraction.is_finite()
        || settings.cash_pyramid_profit_fraction < 0.0
        || !settings.cash_pyramid_add_fraction.is_finite()
        || !(0.0..=1.0).contains(&settings.cash_pyramid_add_fraction)
        || (settings.cash_pyramid_profit_fraction > 0.0
            && settings.cash_pyramid_add_fraction == 0.0)
        || !settings.cash_slippage_bps.is_finite()
        || !(0.0..10_000.0).contains(&settings.cash_slippage_bps)
        || !settings.cash_commission_per_share.is_finite()
        || settings.cash_commission_per_share < 0.0
        || !settings.cash_annual_borrow_rate.is_finite()
        || settings.cash_annual_borrow_rate < 0.0
        || !settings.cash_delisting_return.is_finite()
        || settings.cash_delisting_return <= -1.0
    {
        return Err(
            "invalid cash-study inputs or execution settings; require raw unadjusted prices, valid cash sizing, and finite execution assumptions"
                .into(),
        );
    }
    let mut spin_off_dates = BTreeSet::new();
    for spinoff in &settings.cash_spinoffs {
        let date = data::parse_date(&spinoff.date)?;
        if spinoff.source_symbol.trim().is_empty()
            || !spinoff.source_url.starts_with("https://")
            || spinoff.distributions.is_empty()
            || !spin_off_dates.insert((date, spinoff.source_symbol.clone()))
        {
            return Err(format!(
                "invalid or duplicate spin-off configuration for {} on {}",
                spinoff.source_symbol, spinoff.date
            ));
        }
        let mut distribution_symbols = BTreeSet::new();
        for distribution in &spinoff.distributions {
            if distribution.symbol.trim().is_empty()
                || distribution.symbol == spinoff.source_symbol
                || !distribution_symbols.insert(distribution.symbol.clone())
                || !distribution.shares_per_source_share.is_finite()
                || distribution.shares_per_source_share <= 0.0
            {
                return Err(format!(
                    "invalid distribution in spin-off for {} on {}",
                    spinoff.source_symbol, spinoff.date
                ));
            }
        }
    }
    Ok(())
}

fn configured_spinoffs(
    settings: &CashSettings,
    symbols: &str,
) -> Result<Vec<SpinOffAction>, String> {
    let universe: BTreeSet<String> = config::split_csv(symbols).into_iter().collect();
    settings
        .cash_spinoffs
        .iter()
        .map(|spinoff| {
            if !universe.contains(&spinoff.source_symbol) {
                return Err(format!(
                    "spin-off source {} is not in the configured universe",
                    spinoff.source_symbol
                ));
            }
            let distributions = spinoff
                .distributions
                .iter()
                .map(|distribution| {
                    if !universe.contains(&distribution.symbol) {
                        return Err(format!(
                            "spin-off asset {} is not in the configured universe",
                            distribution.symbol
                        ));
                    }
                    Ok(SpinOffDistribution {
                        symbol: distribution.symbol.clone(),
                        shares_per_source_share: distribution.shares_per_source_share,
                    })
                })
                .collect::<Result<Vec<_>, String>>()?;
            Ok(SpinOffAction {
                date: data::parse_date(&spinoff.date)?,
                source_symbol: spinoff.source_symbol.clone(),
                distributions,
                source_url: spinoff.source_url.clone(),
            })
        })
        .collect()
}

fn correct_signal_series_for_spinoffs(
    signal_panel: &mut Panel,
    raw_panel: &Panel,
    spinoffs: &[SpinOffAction],
) -> Result<Vec<serde_json::Value>, String> {
    let mut adjustments = Vec::with_capacity(spinoffs.len());
    for spinoff in spinoffs {
        let signal_symbol = signal_panel
            .symbols
            .iter()
            .position(|symbol| symbol == &spinoff.source_symbol)
            .ok_or_else(|| {
                format!(
                    "spin-off source {} is absent from the signal panel",
                    spinoff.source_symbol
                )
            })?;
        let raw_symbol = raw_panel
            .symbols
            .iter()
            .position(|symbol| symbol == &spinoff.source_symbol)
            .ok_or_else(|| {
                format!(
                    "spin-off source {} is absent from the raw panel",
                    spinoff.source_symbol
                )
            })?;
        let raw_row = raw_panel
            .dates
            .binary_search(&spinoff.date)
            .map_err(|_| format!("raw panel has no row for spin-off date {}", spinoff.date))?;
        let signal_row = signal_panel
            .dates
            .binary_search(&spinoff.date)
            .map_err(|_| format!("signal panel has no row for spin-off date {}", spinoff.date))?;
        let previous_raw_row = raw_row
            .checked_sub(1)
            .ok_or_else(|| format!("spin-off {} has no prior raw session", spinoff.date))?;
        let previous_signal_row = signal_row
            .checked_sub(1)
            .ok_or_else(|| format!("spin-off {} has no prior signal session", spinoff.date))?;
        let previous_raw_close = raw_panel.close[raw_symbol][previous_raw_row];
        let raw_source_close = raw_panel.close[raw_symbol][raw_row];
        if !previous_raw_close.is_finite()
            || previous_raw_close <= 0.0
            || !raw_source_close.is_finite()
            || raw_source_close <= 0.0
        {
            return Err(format!(
                "spin-off source {} lacks a valid raw close around {}",
                spinoff.source_symbol, spinoff.date
            ));
        }
        let mut distribution_values = Vec::with_capacity(spinoff.distributions.len());
        for distribution in &spinoff.distributions {
            let symbol_index = raw_panel
                .symbols
                .iter()
                .position(|symbol| symbol == &distribution.symbol)
                .ok_or_else(|| {
                    format!(
                        "spin-off asset {} is absent from the raw panel",
                        distribution.symbol
                    )
                })?;
            let close = raw_panel.close[symbol_index][raw_row];
            if !close.is_finite() || close <= 0.0 {
                return Err(format!(
                    "spin-off asset {} lacks a valid raw close on {}",
                    distribution.symbol, spinoff.date
                ));
            }
            distribution_values.push((
                distribution.symbol.clone(),
                distribution.shares_per_source_share,
                close,
            ));
        }
        let total_value = raw_source_close
            + distribution_values
                .iter()
                .map(|(_, ratio, close)| ratio * close)
                .sum::<f64>();
        if !total_value.is_finite() || total_value <= 0.0 {
            return Err(format!(
                "invalid raw spin-off value for {} on {}",
                spinoff.source_symbol, spinoff.date
            ));
        }
        let raw_total_return = total_value / previous_raw_close;
        let signal_before = signal_panel.close[signal_symbol][previous_signal_row];
        let signal_after = signal_panel.close[signal_symbol][signal_row];
        if !signal_before.is_finite()
            || signal_before <= 0.0
            || !signal_after.is_finite()
            || signal_after <= 0.0
        {
            return Err(format!(
                "spin-off source {} lacks a valid adjusted close around {}",
                spinoff.source_symbol, spinoff.date
            ));
        }
        let original_signal_return = signal_after / signal_before;
        let adjustment_factor = raw_total_return / original_signal_return;
        if !adjustment_factor.is_finite() || adjustment_factor <= 0.0 {
            return Err(format!(
                "invalid adjusted signal correction for {} on {}",
                spinoff.source_symbol, spinoff.date
            ));
        }
        for row in signal_row..signal_panel.dates.len() {
            for value in [
                &mut signal_panel.open[signal_symbol][row],
                &mut signal_panel.high[signal_symbol][row],
                &mut signal_panel.low[signal_symbol][row],
                &mut signal_panel.close[signal_symbol][row],
            ] {
                if value.is_finite() {
                    *value *= adjustment_factor;
                }
            }
        }
        adjustments.push(serde_json::json!({
            "date": spinoff.date.to_string(),
            "source_symbol": &spinoff.source_symbol,
            "source_url": &spinoff.source_url,
            "previous_raw_close": previous_raw_close,
            "raw_source_close": raw_source_close,
            "distribution_values": distribution_values.iter().map(|(symbol, ratio, close)| serde_json::json!({
                "symbol": symbol,
                "shares_per_source_share": ratio,
                "raw_close": close
            })).collect::<Vec<_>>(),
            "original_signal_return": original_signal_return,
            "corrected_total_return": raw_total_return,
            "adjustment_factor": adjustment_factor
        }));
    }
    Ok(adjustments)
}

fn empty_bars() -> Bars {
    Bars {
        dates: Vec::new(),
        open: Vec::new(),
        high: Vec::new(),
        low: Vec::new(),
        close: Vec::new(),
        volume: Vec::new(),
    }
}

fn load_data(cfg: &Config, settings: &CashSettings) -> Result<LoadedData, String> {
    let start = data::parse_date(&cfg.start_date)?;
    let symbols = config::split_csv(&cfg.symbols);
    if symbols.is_empty() {
        return Err("cash study has no configured stock symbols".into());
    }
    let mut signal_bars = Vec::with_capacity(symbols.len());
    let mut raw_bars = Vec::with_capacity(symbols.len());
    let mut missing_raw = Vec::new();
    let mut input_files = Vec::new();
    for symbol in &symbols {
        let signal_path = data::input_path(&cfg.data_dir, symbol, &cfg.data_suffix);
        signal_bars.push(data::load_symbol(
            &cfg.data_dir,
            &cfg.data_suffix,
            symbol,
            start,
        )?);
        input_files.push(signal_path);
        let raw_path = data::input_path(&settings.raw_data_dir, symbol, &settings.raw_data_suffix);
        if raw_path.exists() {
            raw_bars.push(data::load_symbol(
                &settings.raw_data_dir,
                &settings.raw_data_suffix,
                symbol,
                start,
            )?);
            input_files.push(raw_path);
        } else {
            missing_raw.push(symbol.clone());
            raw_bars.push(empty_bars());
        }
    }
    let mut signal_panel = data::build_panel(&symbols, &signal_bars);
    let raw_panel = data::build_panel(&symbols, &raw_bars);
    let spinoffs = configured_spinoffs(settings, &cfg.symbols)?;
    let signal_adjustments =
        correct_signal_series_for_spinoffs(&mut signal_panel, &raw_panel, &spinoffs)?;
    let benchmark_path = data::input_path(
        &cfg.benchmark_data_dir,
        &cfg.benchmark,
        &cfg.benchmark_data_suffix,
    );
    let benchmark_bars = data::load_symbol(
        &cfg.benchmark_data_dir,
        &cfg.benchmark_data_suffix,
        &cfg.benchmark,
        start,
    )?;
    input_files.push(benchmark_path);
    let mut benchmark = BTreeMap::new();
    let mut previous_close: Option<f64> = None;
    for (date, close) in benchmark_bars.dates.iter().zip(&benchmark_bars.close) {
        if close.is_finite() && *close > 0.0 {
            if let Some(previous) = previous_close {
                benchmark.insert(*date, (close / previous).ln());
            }
            previous_close = Some(*close);
        }
    }
    let action_path = PathBuf::from(&settings.corporate_actions_file);
    if !action_path.exists() {
        return Err(format!(
            "missing corporate-actions input: {}",
            action_path.display()
        ));
    }
    input_files.push(action_path.clone());
    let actions = data::read_corporate_actions(&action_path)?;
    Ok(LoadedData {
        signal_panel,
        raw_panel,
        signal_adjustments,
        benchmark,
        actions,
        missing_raw,
        input_files,
        identity: input_identity(cfg, settings),
    })
}

fn sha256(path: &Path) -> Result<String, String> {
    let mut file = File::open(path).map_err(|error| format!("{}: {error}", path.display()))?;
    let mut digest = Sha256::new();
    let mut buffer = [0; 64 * 1024];
    loop {
        let read = file
            .read(&mut buffer)
            .map_err(|error| format!("{}: {error}", path.display()))?;
        if read == 0 {
            break;
        }
        digest.update(&buffer[..read]);
    }
    Ok(format!("{:x}", digest.finalize()))
}

fn fingerprint_inputs(inputs: &LoadedData) -> Result<Vec<serde_json::Value>, String> {
    let mut files = Vec::with_capacity(inputs.input_files.len());
    for file in &inputs.input_files {
        let metadata =
            fs::metadata(file).map_err(|error| format!("{}: {error}", file.display()))?;
        files.push(serde_json::json!({
            "path": file,
            "bytes": metadata.len(),
            "modified_unix_seconds": metadata.modified().ok().and_then(|time| time.duration_since(UNIX_EPOCH).ok()).map(|duration| duration.as_secs()),
            "sha256": sha256(file)?
        }));
    }
    Ok(files)
}

fn write_manifest(
    path: &Path,
    cfg: &Config,
    settings: &CashSettings,
    inputs: &LoadedData,
    files: &[serde_json::Value],
) -> Result<(), String> {
    let files = files.to_vec();
    let manifest = serde_json::json!({
        "study": "s041_cash",
        "variant": cfg.variant,
        "manifest_created_at_utc": chrono::Utc::now().to_rfc3339(),
        "signal_price_basis": "configured total-return daily bars (signal-only)",
        "execution_price_basis": settings.raw_price_basis,
        "signal_data_dir": cfg.data_dir,
        "raw_data_dir": settings.raw_data_dir,
        "execution_settings": {
            "initial_cash": settings.cash_initial_cash,
            "long_fraction": settings.cash_long_fraction,
            "short_fraction": settings.cash_short_fraction,
            "short_count": settings.cash_short_count,
            "max_participation": settings.cash_max_participation,
            "max_liquidity_age_seconds": settings.cash_max_liquidity_age_seconds,
            "max_pending_sessions": settings.cash_max_pending_sessions,
            "max_stale_mark_sessions": settings.cash_max_stale_mark_sessions,
            "delisting_return": settings.cash_delisting_return,
            "dca_installments": settings.cash_dca_installments,
            "stop_loss_fraction": settings.cash_stop_loss_fraction,
            "pyramid_profit_fraction": settings.cash_pyramid_profit_fraction,
            "pyramid_add_fraction": settings.cash_pyramid_add_fraction,
            "slippage_bps": settings.cash_slippage_bps,
            "commission_per_share": settings.cash_commission_per_share,
            "annual_borrow_rate": settings.cash_annual_borrow_rate
        },
        "configured_spin_offs": settings.cash_spinoffs,
        "signal_adjustments": inputs.signal_adjustments,
        "symbols": cfg.symbols,
        "raw_missing_symbols": inputs.missing_raw,
        "signal_coverage": {
            "first_date": inputs.signal_panel.dates.first().map(ToString::to_string),
            "last_date": inputs.signal_panel.dates.last().map(ToString::to_string),
            "rows": inputs.signal_panel.dates.len()
        },
        "raw_coverage": {
            "first_date": inputs.raw_panel.dates.first().map(ToString::to_string),
            "last_date": inputs.raw_panel.dates.last().map(ToString::to_string),
            "rows": inputs.raw_panel.dates.len()
        },
        "files": files
    });
    fs::write(
        path,
        serde_json::to_vec_pretty(&manifest).map_err(|error| error.to_string())?,
    )
    .map_err(|error| format!("{}: {error}", path.display()))
}

fn evaluation_start(
    cfg: &Config,
    signals: &Signals,
    panel: &Panel,
) -> Result<chrono::NaiveDate, String> {
    let row = (0..panel.dates.len()).find(|row| signals.signal.iter().any(|series| series[*row]));
    row.map(|row| panel.dates[row])
        .or_else(|| panel.dates.last().copied())
        .ok_or_else(|| format!("no signal dates available for {}", cfg.strategy))
}

fn metrics(
    replay: &CashReplay,
    benchmark: &BTreeMap<chrono::NaiveDate, f64>,
    start: chrono::NaiveDate,
    end: Option<chrono::NaiveDate>,
    random_pick_log: f64,
) -> Metrics {
    let rows: Vec<_> = replay
        .days
        .iter()
        .filter(|day| day.date >= start && end.is_none_or(|end| day.date < end))
        .collect();
    let prior_exposure: BTreeMap<_, _> = replay
        .days
        .windows(2)
        .map(|pair| (pair[1].date, (pair[0].net_exposure, pair[0].equity)))
        .collect();
    let total_log = rows.iter().map(|day| day.log_return).sum::<f64>();
    let trend_log = rows
        .iter()
        .map(|day| {
            let benchmark_return = benchmark.get(&day.date).copied().unwrap_or(0.0);
            prior_exposure
                .get(&day.date)
                .map_or(0.0, |(net_exposure, equity)| {
                    if *equity > 0.0 {
                        net_exposure / equity * benchmark_return
                    } else {
                        0.0
                    }
                })
        })
        .sum::<f64>();
    let bias = 0.0;
    let skill = total_log - trend_log - bias;
    let trades: Vec<_> = replay
        .trades
        .iter()
        .filter(|trade| trade.date >= start && end.is_none_or(|end| trade.date < end))
        .collect();
    let positive = trades
        .iter()
        .map(|trade| trade.realized_pnl.max(0.0))
        .sum::<f64>();
    let negative = -trades
        .iter()
        .map(|trade| trade.realized_pnl.min(0.0))
        .sum::<f64>();
    let sd = stats::std(&rows.iter().map(|day| day.log_return).collect::<Vec<_>>());
    let sharpe = if sd > 0.0 && sd.is_finite() {
        stats::mean(&rows.iter().map(|day| day.log_return).collect::<Vec<_>>()) / sd
            * 252.0_f64.sqrt()
    } else {
        0.0
    };
    let mut peak = rows
        .first()
        .map_or(10_000.0, |day| day.equity / day.log_return.exp());
    let mut max_drawdown = 0.0f64;
    for day in &rows {
        peak = peak.max(day.equity);
        max_drawdown = max_drawdown.min(day.equity / peak - 1.0);
    }
    let wins = trades
        .iter()
        .filter(|trade| trade.realized_pnl > 0.0)
        .count();
    Metrics {
        total_log,
        trend_log,
        random_pick_log,
        skill,
        bias,
        skill_pct: if total_log > 0.0 {
            skill / total_log * 100.0
        } else {
            f64::NAN
        },
        sharpe,
        profit_factor: if negative > 0.0 {
            positive / negative
        } else if positive > 0.0 {
            f64::INFINITY
        } else {
            0.0
        },
        max_drawdown_pct: max_drawdown * 100.0,
        trades: trades.len(),
        win_pct: if !trades.is_empty() {
            wins as f64 / trades.len() as f64 * 100.0
        } else {
            0.0
        },
    }
}

fn benchmark_max_drawdown_pct(
    benchmark: &BTreeMap<chrono::NaiveDate, f64>,
    start: chrono::NaiveDate,
    end: chrono::NaiveDate,
) -> f64 {
    let mut cumulative_log = 0.0f64;
    let mut peak_log = 0.0f64;
    let mut max_drawdown = 0.0f64;
    for (_, daily_log_return) in benchmark.range(start..end) {
        cumulative_log += daily_log_return;
        peak_log = peak_log.max(cumulative_log);
        max_drawdown = max_drawdown.min((cumulative_log - peak_log).exp() - 1.0);
    }
    max_drawdown * 100.0
}

fn percentile_ranks(
    trials: &[TrialResult],
    indices: &[usize],
    metric: fn(&Metrics) -> f64,
) -> Vec<f64> {
    if indices.is_empty() {
        return vec![0.0; trials.len()];
    }
    let mut ordered: Vec<(usize, f64)> = indices
        .iter()
        .map(|&index| (index, metric(&trials[index].dev)))
        .collect();
    ordered.sort_by(|left, right| {
        right
            .1
            .total_cmp(&left.1)
            .then_with(|| left.0.cmp(&right.0))
    });
    let mut ranks = vec![0.0; trials.len()];
    let mut start = 0;
    while start < ordered.len() {
        let mut end = start + 1;
        while end < ordered.len() && ordered[end].1.total_cmp(&ordered[start].1).is_eq() {
            end += 1;
        }
        let rank_score = 1.0 - ((start + end - 1) as f64 / 2.0) / ordered.len() as f64;
        for &(index, _) in &ordered[start..end] {
            ranks[index] = rank_score;
        }
        start = end;
    }
    ranks
}

fn write_selection_summary(
    results_dir: &Path,
    trials: &[TrialResult],
    blocked: &[BlockedVariant],
    random_draws: usize,
) -> Result<(), String> {
    if trials.is_empty() && blocked.is_empty() {
        return Ok(());
    }
    let benchmark_drawdown = trials
        .first()
        .map_or(0.0, |trial| trial.benchmark_dev_drawdown_pct);
    let drawdown_limit = 25.0f64.min(benchmark_drawdown.abs());
    let eligible: Vec<usize> = trials
        .iter()
        .enumerate()
        .filter(|(_, trial)| trial.dev.skill > 0.0 && trial.dev.max_drawdown_pct >= -drawdown_limit)
        .map(|(index, _)| index)
        .collect();
    let skill_ranks = percentile_ranks(trials, &eligible, |metrics| metrics.skill);
    let sharpe_ranks = percentile_ranks(trials, &eligible, |metrics| metrics.sharpe);
    let pf_ranks = percentile_ranks(trials, &eligible, |metrics| metrics.profit_factor);
    let drawdown_ranks = percentile_ranks(trials, &eligible, |metrics| metrics.max_drawdown_pct);
    let scores: Vec<f64> = (0..trials.len())
        .map(|index| {
            (skill_ranks[index] + sharpe_ranks[index] + pf_ranks[index] + drawdown_ranks[index])
                / 4.0
        })
        .collect();
    let winner = eligible.iter().copied().max_by(|&left, &right| {
        scores[left]
            .total_cmp(&scores[right])
            .then_with(|| trials[right].variant.cmp(&trials[left].variant))
    });
    let mut report = format!(
        "# S041 cash DEV-only selection\n\n\
         Latest results for {} named configuration(s): {} completed and {} blocked. Completed trials use the mean of {random_draws} deterministic random-pick draw(s). Eligibility requires positive DEV skill and DEV drawdown no worse than {:.2}% (the stricter of 25% or SPY DEV drawdown of {:.2}%). Selection score is the equal-weight average of DEV percentile ranks for skill, Sharpe, profit factor, and drawdown. TEST metrics do not participate in eligibility or ranking. Existing TEST data has previously been inspected and is not a fresh holdout. Blocked configurations are recorded as trials but excluded from ranking.\n\n\
         | Variant | DEV skill | DEV Sharpe | DEV PF | DEV max DD | DEV trades | Eligible | DEV score |\n\
         |---|---:|---:|---:|---:|---:|:---:|---:|\n",
        trials.len() + blocked.len(),
        trials.len(),
        blocked.len(),
        drawdown_limit,
        benchmark_drawdown
    );
    for (index, trial) in trials.iter().enumerate() {
        let is_eligible = eligible.contains(&index);
        let score = if is_eligible {
            format!("{:.4}", scores[index])
        } else {
            "—".into()
        };
        report.push_str(&format!(
            "| {} | {:.4} | {:.3} | {:.3} | {:.2}% | {} | {} | {} |\n",
            trial.variant,
            trial.dev.skill,
            trial.dev.sharpe,
            trial.dev.profit_factor,
            trial.dev.max_drawdown_pct,
            trial.dev.trades,
            if is_eligible { "yes" } else { "no" },
            score
        ));
    }
    if !blocked.is_empty() {
        report.push_str("\nBlocked configurations:\n\n| Variant | Reason |\n|---|---|\n");
        for variant in blocked {
            report.push_str(&format!(
                "| {} | {} |\n",
                variant.variant,
                variant.reason.replace('|', "\\|")
            ));
        }
    }
    let winner_json = if let Some(index) = winner {
        let trial = &trials[index];
        report.push_str(&format!(
            "\nDEV-selected variant: **{}** (score {:.4}). Evidence gates: DEV trades >= 100: {}; TEST trades >= 30: {}. TEST is reported for this selected variant only below and must not be interpreted as a fresh holdout.\n\n\
             | Partition | Total log | Trend | Random-pick | Skill | Bias | Skill % | Sharpe | Profit factor | Max DD | Trades | Win rate |\n\
             |---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|\n\
             | DEV | {:.4} | {:.4} | {:.4} | {:.4} | {:.4} | {:.2} | {:.3} | {:.3} | {:.2}% | {} | {:.2}% |\n\
             | TEST | {:.4} | {:.4} | {:.4} | {:.4} | {:.4} | {:.2} | {:.3} | {:.3} | {:.2}% | {} | {:.2}% |\n",
            trial.variant,
            scores[index],
            trial.dev.trades >= 100,
            trial.test.trades >= 30,
            trial.dev.total_log,
            trial.dev.trend_log,
            trial.dev.random_pick_log,
            trial.dev.skill,
            trial.dev.bias,
            trial.dev.skill_pct,
            trial.dev.sharpe,
            trial.dev.profit_factor,
            trial.dev.max_drawdown_pct,
            trial.dev.trades,
            trial.dev.win_pct,
            trial.test.total_log,
            trial.test.trend_log,
            trial.test.random_pick_log,
            trial.test.skill,
            trial.test.bias,
            trial.test.skill_pct,
            trial.test.sharpe,
            trial.test.profit_factor,
            trial.test.max_drawdown_pct,
            trial.test.trades,
            trial.test.win_pct,
        ));
        serde_json::json!({
            "variant": trial.variant,
            "dev_score": scores[index],
            "dev": {
                "skill": trial.dev.skill,
                "sharpe": trial.dev.sharpe,
                "profit_factor": trial.dev.profit_factor,
                "max_drawdown_pct": trial.dev.max_drawdown_pct,
                "trades": trial.dev.trades
            },
            "test": {
                "skill": trial.test.skill,
                "sharpe": trial.test.sharpe,
                "profit_factor": trial.test.profit_factor,
                "max_drawdown_pct": trial.test.max_drawdown_pct,
                "trades": trial.test.trades
            }
        })
    } else {
        report.push_str(
            "\n**No eligible DEV winner.** No variant met both positive DEV skill and the DEV drawdown limit; TEST is not used to select a candidate.\n",
        );
        serde_json::Value::Null
    };
    fs::write(results_dir.join("selection.md"), &report)
        .map_err(|error| format!("cannot write selection report: {error}"))?;
    let selection = serde_json::json!({
        "selection_basis": "DEV only",
        "trial_count": trials.len() + blocked.len(),
        "completed_trial_count": trials.len(),
        "blocked_variants": blocked.iter().map(|variant| serde_json::json!({
            "variant": variant.variant,
            "reason": variant.reason
        })).collect::<Vec<_>>(),
        "random_draws": random_draws,
        "benchmark_dev_drawdown_pct": benchmark_drawdown,
        "max_allowed_drawdown_pct": drawdown_limit,
        "eligible_variants": eligible.iter().map(|&index| trials[index].variant.clone()).collect::<Vec<_>>(),
        "selected": winner_json
    });
    let selection_json =
        serde_json::to_vec_pretty(&selection).map_err(|error| error.to_string())?;
    fs::write(results_dir.join("selection.json"), &selection_json)
        .map_err(|error| format!("cannot write selection data: {error}"))?;
    let blocked_path = results_dir.join("blocked_variants.csv");
    let mut blocked_writer = csv::Writer::from_path(&blocked_path)
        .map_err(|error| format!("{}: {error}", blocked_path.display()))?;
    blocked_writer
        .write_record(["variant", "reason"])
        .map_err(|error| error.to_string())?;
    for variant in blocked {
        blocked_writer
            .write_record([variant.variant.clone(), variant.reason.clone()])
            .map_err(|error| error.to_string())?;
    }
    blocked_writer.flush().map_err(|error| error.to_string())?;
    if trials.len() + blocked.len() >= 30 {
        fs::write(results_dir.join("selection_grid.md"), report)
            .map_err(|error| format!("cannot write grid selection report: {error}"))?;
        fs::write(results_dir.join("selection_grid.json"), selection_json)
            .map_err(|error| format!("cannot write grid selection data: {error}"))?;
    }
    Ok(())
}

fn random_baseline(
    cfg: &Config,
    inputs: &LoadedData,
    base_signals: &Signals,
    settings: CashExecution,
    spinoffs: &[SpinOffAction],
    draws: usize,
    start: chrono::NaiveDate,
    split: chrono::NaiveDate,
) -> Result<(f64, f64), String> {
    if draws == 0 {
        return Err("random baseline requires at least one draw".into());
    }
    let mut rng = Rng::new(cfg.seed);
    let mut dev_sum = 0.0;
    let mut test_sum = 0.0;
    let rebalance_rows: BTreeSet<usize> = (0..inputs.signal_panel.dates.len())
        .filter(|&row| {
            row + 1 == inputs.signal_panel.dates.len()
                || inputs.signal_panel.dates[row].month()
                    != inputs.signal_panel.dates[row + 1].month()
                || inputs.signal_panel.dates[row].year()
                    != inputs.signal_panel.dates[row + 1].year()
        })
        .collect();
    for _ in 0..draws {
        let mut placebo = base_signals.clone();
        for row in &rebalance_rows {
            for symbol in 0..placebo.signal.len() {
                if placebo.signal[symbol][*row] {
                    placebo.rsi[symbol][*row] = rng.next_f64();
                }
            }
        }
        let replay = run_cash_replay_with_spinoffs(
            &inputs.signal_panel,
            &inputs.raw_panel,
            &placebo,
            &inputs.actions,
            spinoffs,
            settings,
            &cfg.breadth_mode,
            cfg.breadth,
            cfg.min_breadth,
            cfg.decile,
        )?;
        for day in replay.days.iter().filter(|day| day.date >= start) {
            if day.date < split {
                dev_sum += day.log_return;
            } else {
                test_sum += day.log_return;
            }
        }
    }
    Ok((dev_sum / draws as f64, test_sum / draws as f64))
}

fn write_daily(
    path: &Path,
    replay: &CashReplay,
    benchmark: &BTreeMap<chrono::NaiveDate, f64>,
    start: chrono::NaiveDate,
) -> Result<(), String> {
    let mut writer =
        csv::Writer::from_path(path).map_err(|error| format!("{}: {error}", path.display()))?;
    writer
        .write_record([
            "date",
            "cash",
            "equity",
            "gross_exposure",
            "net_exposure",
            "positions",
            "log_return",
            "strategy_cumulative_log",
            "benchmark_cumulative_log",
            "stock_entries",
            "completed_round_trips",
            "winning_round_trips",
        ])
        .map_err(|error| error.to_string())?;
    let (mut strategy_cumulative, mut benchmark_cumulative) = (0.0, 0.0);
    for day in &replay.days {
        if day.date >= start {
            strategy_cumulative += day.log_return;
            benchmark_cumulative += benchmark.get(&day.date).copied().unwrap_or(0.0);
        }
        writer
            .write_record([
                day.date.to_string(),
                day.cash.to_string(),
                day.equity.to_string(),
                day.gross_exposure.to_string(),
                day.net_exposure.to_string(),
                day.positions.to_string(),
                day.log_return.to_string(),
                strategy_cumulative.to_string(),
                benchmark_cumulative.to_string(),
                day.stock_entries.to_string(),
                day.completed_round_trips.to_string(),
                day.winning_round_trips.to_string(),
            ])
            .map_err(|error| error.to_string())?;
    }
    writer.flush().map_err(|error| error.to_string())
}

fn write_events(path: &Path, replay: &CashReplay) -> Result<(), String> {
    let mut writer =
        csv::Writer::from_path(path).map_err(|error| format!("{}: {error}", path.display()))?;
    writer
        .write_record([
            "event",
            "symbol",
            "shares",
            "previous_shares",
            "event_time",
            "price",
            "commission",
            "realized_pnl",
        ])
        .map_err(|error| error.to_string())?;
    for event in replay.account.events() {
        let row = match event {
            deepthrust::s041_replay::ReplayEvent::OrderSubmitted { order } => vec![
                "submitted".into(),
                order.symbol.clone(),
                order.shares.to_string(),
                String::new(),
                order.decided_at.to_rfc3339(),
                String::new(),
                String::new(),
                String::new(),
            ],
            deepthrust::s041_replay::ReplayEvent::OrderCancelled {
                order,
                cancelled_at,
            } => vec![
                "cancelled".into(),
                order.symbol.clone(),
                order.shares.to_string(),
                String::new(),
                cancelled_at.to_rfc3339(),
                String::new(),
                String::new(),
                String::new(),
            ],
            deepthrust::s041_replay::ReplayEvent::Filled {
                order,
                fill,
                filled_at,
            } => vec![
                "filled".into(),
                order.symbol.clone(),
                fill.shares.to_string(),
                String::new(),
                filled_at.to_rfc3339(),
                fill.price.to_string(),
                fill.commission.to_string(),
                fill.realized_pnl.to_string(),
            ],
            deepthrust::s041_replay::ReplayEvent::OrderAdjusted {
                previous,
                adjusted,
                adjusted_at,
            } => vec![
                "adjusted".into(),
                adjusted.symbol.clone(),
                adjusted.shares.to_string(),
                previous.shares.to_string(),
                adjusted_at.to_rfc3339(),
                String::new(),
                String::new(),
                String::new(),
            ],
        };
        writer
            .write_record(row)
            .map_err(|error| error.to_string())?;
    }
    writer.flush().map_err(|error| error.to_string())
}

fn write_actions(path: &Path, replay: &CashReplay) -> Result<(), String> {
    let mut writer =
        csv::Writer::from_path(path).map_err(|error| format!("{}: {error}", path.display()))?;
    writer
        .write_record(["date", "action_id", "symbol", "action", "amount"])
        .map_err(|error| error.to_string())?;
    for action in &replay.actions {
        writer
            .write_record([
                action.date.to_string(),
                action.action_id.clone(),
                action.symbol.clone(),
                action.action.clone(),
                action.amount.to_string(),
            ])
            .map_err(|error| error.to_string())?;
    }
    writer.flush().map_err(|error| error.to_string())
}

fn write_lifecycle_events(path: &Path, replay: &CashReplay) -> Result<(), String> {
    let mut writer =
        csv::Writer::from_path(path).map_err(|error| format!("{}: {error}", path.display()))?;
    writer
        .write_record(["date", "symbol", "action", "shares"])
        .map_err(|error| error.to_string())?;
    for event in &replay.lifecycle_events {
        writer
            .write_record([
                event.date.to_string(),
                event.symbol.clone(),
                event.action.clone(),
                event.shares.to_string(),
            ])
            .map_err(|error| error.to_string())?;
    }
    writer.flush().map_err(|error| error.to_string())
}

fn write_trades(path: &Path, replay: &CashReplay) -> Result<(), String> {
    let mut writer =
        csv::Writer::from_path(path).map_err(|error| format!("{}: {error}", path.display()))?;
    writer
        .write_record(["date", "symbol", "realized_pnl", "win"])
        .map_err(|error| error.to_string())?;
    for trade in &replay.trades {
        writer
            .write_record([
                trade.date.to_string(),
                trade.symbol.clone(),
                trade.realized_pnl.to_string(),
                (trade.realized_pnl > 0.0).to_string(),
            ])
            .map_err(|error| error.to_string())?;
    }
    writer.flush().map_err(|error| error.to_string())
}

fn write_rejections(path: &Path, replay: &CashReplay) -> Result<(), String> {
    let mut writer =
        csv::Writer::from_path(path).map_err(|error| format!("{}: {error}", path.display()))?;
    writer
        .write_record(["date", "symbol", "reason"])
        .map_err(|error| error.to_string())?;
    for (date, symbol, reason) in &replay.rejections {
        writer
            .write_record([date.to_string(), symbol.clone(), reason.clone()])
            .map_err(|error| error.to_string())?;
    }
    writer.flush().map_err(|error| error.to_string())
}

fn write_plot(
    path: &Path,
    replay: &CashReplay,
    benchmark: &BTreeMap<chrono::NaiveDate, f64>,
    start: chrono::NaiveDate,
) -> Result<(), String> {
    let rows: Vec<_> = replay.days.iter().filter(|day| day.date >= start).collect();
    if rows.is_empty() {
        return Err("cannot plot an empty evaluation period".into());
    }
    let (mut strategy, mut market) = (
        Vec::with_capacity(rows.len()),
        Vec::with_capacity(rows.len()),
    );
    let (mut strategy_sum, mut market_sum) = (0.0, 0.0);
    for day in &rows {
        strategy_sum += day.log_return;
        market_sum += benchmark.get(&day.date).copied().unwrap_or(0.0);
        strategy.push(strategy_sum);
        market.push(market_sum);
    }
    let low = strategy
        .iter()
        .chain(&market)
        .copied()
        .fold(f64::INFINITY, f64::min);
    let high = strategy
        .iter()
        .chain(&market)
        .copied()
        .fold(f64::NEG_INFINITY, f64::max);
    let padding = (high - low).abs().max(0.1) * 0.05;
    let (low, high) = (low - padding, high + padding);
    let root = SVGBackend::new(path, (1000, 600)).into_drawing_area();
    root.fill(&WHITE).map_err(|error| error.to_string())?;
    let mut chart = ChartBuilder::on(&root)
        .caption(
            "S041 cash study vs SPY (cumulative log return)",
            ("sans-serif", 24),
        )
        .margin(20)
        .x_label_area_size(35)
        .y_label_area_size(55)
        .build_cartesian_2d(0usize..rows.len(), low..high)
        .map_err(|error| error.to_string())?;
    chart
        .configure_mesh()
        .x_desc("session")
        .y_desc("cumulative log return")
        .draw()
        .map_err(|error| error.to_string())?;
    chart
        .draw_series(LineSeries::new(
            strategy
                .iter()
                .enumerate()
                .map(|(index, value)| (index, *value)),
            &BLUE,
        ))
        .map_err(|error| error.to_string())?
        .label("S041 cash")
        .legend(|(x, y)| PathElement::new([(x, y), (x + 20, y)], BLUE));
    chart
        .draw_series(LineSeries::new(
            market
                .iter()
                .enumerate()
                .map(|(index, value)| (index, *value)),
            &RED,
        ))
        .map_err(|error| error.to_string())?
        .label("SPY")
        .legend(|(x, y)| PathElement::new([(x, y), (x + 20, y)], RED));
    chart
        .configure_series_labels()
        .border_style(BLACK)
        .draw()
        .map_err(|error| error.to_string())?;
    root.present().map_err(|error| error.to_string())
}

fn write_report(
    path: &Path,
    variant: &str,
    replay: &CashReplay,
    dev: &Metrics,
    test: &Metrics,
    settings: &CashSettings,
    random_draws: usize,
    start: chrono::NaiveDate,
    split: chrono::NaiveDate,
    inputs: &LoadedData,
) -> Result<(), String> {
    let stop_description = if settings.cash_stop_loss_fraction > 0.0 {
        format!("{:.1}%", settings.cash_stop_loss_fraction * 100.0)
    } else {
        "disabled".into()
    };
    let pyramid_description = if settings.cash_pyramid_profit_fraction > 0.0 {
        format!(
            "{:.1}% trigger, {:.1}% of current shares",
            settings.cash_pyramid_profit_fraction * 100.0,
            settings.cash_pyramid_add_fraction * 100.0
        )
    } else {
        "disabled".into()
    };
    let spinoff_assumption = if settings.cash_spinoffs.is_empty() {
        "No spin-off actions are configured.".to_string()
    } else {
        "Configured spin-offs use raw ex-date opening prices; fractional shares are credited immediately as cash-in-lieu at those prices.".to_string()
    };
    let report = format!(
        "# S041 cash study — {variant}\n\n\
         This is a daily-open execution scenario, not verified intraday execution. Signals use adjusted total-return bars; fills and marks use the separately configured `{}` raw-price files. Only preceding completed-session volume constrains fills. After {} consecutive missing raw closes, a held position is settled using the configured {:.1}% delisting-return assumption; this is a proxy, not a verified delisting payment. DCA uses {} installment(s); stop loss is {}; winner adds are {}. Trend is a close-to-close proxy using prior-close net exposure and does not model intraday exposure changes. The UTC timestamps in the event file are synthetic ordering keys, not observed exchange times. There is no leverage, commissions are ${:.4}/share, slippage is {:.2} bps, and annual short borrow is {:.2}%. Short availability is assumed when the short leg is enabled. {}\n\n\
         Initial cash: ${:.2}. Random-pick baseline: mean of {random_draws} deterministic draw(s). Evaluation begins {start}; DEV is before {split}, TEST is from {split}. The existing TEST period has been inspected previously and is not a fresh holdout. DEV variant-selection bias is not corrected for the full search.\n\n\
         | Partition | Total log | Trend | Random-pick | Skill | Bias | Skill % | Sharpe | Profit factor | Max DD | Trades | Win rate |\n\
         |---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|\n\
         | DEV | {:.4} | {:.4} | {:.4} | {:.4} | {:.4} | {:.2} | {:.3} | {:.3} | {:.2}% | {} | {:.2}% |\n\
         | TEST | {:.4} | {:.4} | {:.4} | {:.4} | {:.4} | {:.2} | {:.3} | {:.3} | {:.2}% | {} | {:.2}% |\n\n\
         Rebalance dates: {}. Stock entries: {}. Completed round trips: {}. Rejected orders: {}. DCA installments submitted: {}. Stop exits submitted: {}. Pyramiding adds submitted: {}. Missing held-position open quotes: {}. Stale held-position close marks: {}. Raw symbols without files: {}. Dividend entitlements missing payable dates: {}. Pending orders at end: {}.\n\n\
         Missing raw symbols are not substituted with adjusted bars. A selected symbol without an executable raw close or an unsupported corporate action on a held position blocks the run. Entitlements with missing payable dates remain receivables in equity, not spendable cash, and are listed in the corporate-action audit. The TEST period is already inspected; this is not a fresh holdout. The event file records daily-open scenario ordering, not verified execution timestamps.\n",
        settings.raw_data_dir,
        settings.cash_max_stale_mark_sessions,
        settings.cash_delisting_return * 100.0,
        settings.cash_dca_installments,
        stop_description,
        pyramid_description,
        settings.cash_commission_per_share,
        settings.cash_slippage_bps,
        settings.cash_annual_borrow_rate * 100.0,
        spinoff_assumption,
        settings.cash_initial_cash,
        dev.total_log,
        dev.trend_log,
        dev.random_pick_log,
        dev.skill,
        dev.bias,
        dev.skill_pct,
        dev.sharpe,
        dev.profit_factor,
        dev.max_drawdown_pct,
        dev.trades,
        dev.win_pct,
        test.total_log,
        test.trend_log,
        test.random_pick_log,
        test.skill,
        test.bias,
        test.skill_pct,
        test.sharpe,
        test.profit_factor,
        test.max_drawdown_pct,
        test.trades,
        test.win_pct,
        replay.rebalance_dates.len(),
        replay.stock_entries,
        replay.completed_round_trips,
        replay.rejections.len(),
        replay
            .lifecycle_events
            .iter()
            .filter(|event| event.action == "dca_installment_submitted")
            .count(),
        replay
            .lifecycle_events
            .iter()
            .filter(|event| event.action == "stop_exit_submitted")
            .count(),
        replay
            .lifecycle_events
            .iter()
            .filter(|event| event.action == "pyramid_add_submitted")
            .count(),
        replay.missing_open_quotes.len(),
        replay.stale_marks.len(),
        inputs.missing_raw.len(),
        replay.unpaid_dividends.len(),
        replay.account.pending().len()
    );
    fs::write(path, report).map_err(|error| format!("{}: {error}", path.display()))
}

fn update_comparison(
    path: &Path,
    variant: &str,
    dev: &Metrics,
    test: &Metrics,
) -> Result<(), String> {
    let header = "variant,dev_total_log,dev_trend,dev_random_pick,dev_skill,dev_bias,dev_skill_pct,dev_sharpe,dev_pf,dev_max_dd_pct,dev_trades,dev_win_pct,test_total_log,test_trend,test_random_pick,test_skill,test_bias,test_skill_pct,test_sharpe,test_pf,test_max_dd_pct,test_trades,test_win_pct\n";
    let mut lines = if path.exists() {
        fs::read_to_string(path)
            .map_err(|error| format!("{}: {error}", path.display()))?
            .lines()
            .map(str::to_owned)
            .collect::<Vec<_>>()
    } else {
        vec![header.trim_end().into()]
    };
    let fields = [
        variant.to_owned(),
        dev.total_log.to_string(),
        dev.trend_log.to_string(),
        dev.random_pick_log.to_string(),
        dev.skill.to_string(),
        dev.bias.to_string(),
        dev.skill_pct.to_string(),
        dev.sharpe.to_string(),
        dev.profit_factor.to_string(),
        dev.max_drawdown_pct.to_string(),
        dev.trades.to_string(),
        dev.win_pct.to_string(),
        test.total_log.to_string(),
        test.trend_log.to_string(),
        test.random_pick_log.to_string(),
        test.skill.to_string(),
        test.bias.to_string(),
        test.skill_pct.to_string(),
        test.sharpe.to_string(),
        test.profit_factor.to_string(),
        test.max_drawdown_pct.to_string(),
        test.trades.to_string(),
        test.win_pct.to_string(),
    ];
    lines.push(fields.join(","));
    fs::write(path, format!("{}\n", lines.join("\n")))
        .map_err(|error| format!("{}: {error}", path.display()))
}

fn comparison_f64(record: &csv::StringRecord, index: usize, variant: &str) -> Result<f64, String> {
    record
        .get(index)
        .ok_or_else(|| format!("comparison row for {variant} is missing column {index}"))?
        .parse::<f64>()
        .map_err(|error| {
            format!("invalid comparison value for {variant} at column {index}: {error}")
        })
}

fn comparison_usize(
    record: &csv::StringRecord,
    index: usize,
    variant: &str,
) -> Result<usize, String> {
    record
        .get(index)
        .ok_or_else(|| format!("comparison row for {variant} is missing column {index}"))?
        .parse::<usize>()
        .map_err(|error| {
            format!("invalid comparison value for {variant} at column {index}: {error}")
        })
}

fn read_latest_trials(
    path: &Path,
    benchmark_dev_drawdown_pct: f64,
    blocked: &[BlockedVariant],
) -> Result<Vec<TrialResult>, String> {
    let mut reader =
        csv::Reader::from_path(path).map_err(|error| format!("{}: {error}", path.display()))?;
    let mut latest = BTreeMap::new();
    for row in reader.records() {
        let record = row.map_err(|error| format!("{}: {error}", path.display()))?;
        let variant = record
            .get(0)
            .ok_or_else(|| format!("{} contains a row without a variant", path.display()))?
            .to_string();
        let metrics = |start: usize| -> Result<Metrics, String> {
            Ok(Metrics {
                total_log: comparison_f64(&record, start, &variant)?,
                trend_log: comparison_f64(&record, start + 1, &variant)?,
                random_pick_log: comparison_f64(&record, start + 2, &variant)?,
                skill: comparison_f64(&record, start + 3, &variant)?,
                bias: comparison_f64(&record, start + 4, &variant)?,
                skill_pct: comparison_f64(&record, start + 5, &variant)?,
                sharpe: comparison_f64(&record, start + 6, &variant)?,
                profit_factor: comparison_f64(&record, start + 7, &variant)?,
                max_drawdown_pct: comparison_f64(&record, start + 8, &variant)?,
                trades: comparison_usize(&record, start + 9, &variant)?,
                win_pct: comparison_f64(&record, start + 10, &variant)?,
            })
        };
        let dev = metrics(1)?;
        let test = metrics(12)?;
        latest.insert(
            variant.clone(),
            TrialResult {
                variant,
                dev,
                test,
                benchmark_dev_drawdown_pct,
            },
        );
    }
    for variant in blocked {
        latest.remove(&variant.variant);
    }
    Ok(latest.into_values().collect())
}

fn append_cost_ladder(
    path: &Path,
    variant: &str,
    settings: &CashSettings,
    dev: &Metrics,
    test: &Metrics,
) -> Result<(), String> {
    let ladder_variants = [
        "baseline",
        "participation005",
        "participation025",
        "participation100",
        "borrow30",
        "borrow300",
        "borrow1000",
        "slip5",
        "slip10",
        "slip25",
        "commission005",
        "commission01",
    ];
    if !ladder_variants.contains(&variant) {
        return Ok(());
    }
    let is_new = !path.exists();
    let mut writer = csv::WriterBuilder::new().has_headers(false).from_writer(
        fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .map_err(|error| format!("{}: {error}", path.display()))?,
    );
    if is_new {
        writer
            .write_record([
                "variant",
                "commission_per_share",
                "slippage_bps",
                "annual_borrow_rate",
                "max_participation",
                "dev_skill",
                "dev_sharpe",
                "dev_profit_factor",
                "dev_max_drawdown_pct",
                "dev_trades",
                "test_skill",
                "test_sharpe",
                "test_profit_factor",
                "test_max_drawdown_pct",
                "test_trades",
            ])
            .map_err(|error| error.to_string())?;
    }
    writer
        .write_record([
            variant.to_owned(),
            settings.cash_commission_per_share.to_string(),
            settings.cash_slippage_bps.to_string(),
            settings.cash_annual_borrow_rate.to_string(),
            settings.cash_max_participation.to_string(),
            dev.skill.to_string(),
            dev.sharpe.to_string(),
            dev.profit_factor.to_string(),
            dev.max_drawdown_pct.to_string(),
            dev.trades.to_string(),
            test.skill.to_string(),
            test.sharpe.to_string(),
            test.profit_factor.to_string(),
            test.max_drawdown_pct.to_string(),
            test.trades.to_string(),
        ])
        .map_err(|error| error.to_string())?;
    writer.flush().map_err(|error| error.to_string())
}

fn trial_parameters(cfg: &Config, settings: &CashSettings) -> serde_json::Value {
    serde_json::json!({
            "mom_window": cfg.mom_window,
            "skip_window": cfg.skip_window,
            "breadth_mode": cfg.breadth_mode,
            "breadth": cfg.breadth,
            "decile": cfg.decile,
            "initial_cash": settings.cash_initial_cash,
            "long_fraction": settings.cash_long_fraction,
            "short_fraction": settings.cash_short_fraction,
            "short_count": settings.cash_short_count,
            "max_participation": settings.cash_max_participation,
            "max_stale_mark_sessions": settings.cash_max_stale_mark_sessions,
            "dca_installments": settings.cash_dca_installments,
            "stop_loss_fraction": settings.cash_stop_loss_fraction,
            "pyramid_profit_fraction": settings.cash_pyramid_profit_fraction,
            "pyramid_add_fraction": settings.cash_pyramid_add_fraction,
            "slippage_bps": settings.cash_slippage_bps,
            "commission_per_share": settings.cash_commission_per_share,
            "annual_borrow_rate": settings.cash_annual_borrow_rate,
            "delisting_return": settings.cash_delisting_return
    })
}

fn append_trial(
    path: &Path,
    cfg: &Config,
    settings: &CashSettings,
    dev: &Metrics,
    test: &Metrics,
) -> Result<(), String> {
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|error| format!("{}: {error}", path.display()))?;
    let record = serde_json::json!({
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "study": "s041_cash",
        "status": "completed",
        "variant": cfg.variant,
        "parameters": trial_parameters(cfg, settings),
        "dev": {"total_log": dev.total_log, "skill": dev.skill, "sharpe": dev.sharpe, "pf": dev.profit_factor, "trades": dev.trades},
        "test": {"total_log": test.total_log, "skill": test.skill, "sharpe": test.sharpe, "pf": test.profit_factor, "trades": test.trades}
    });
    writeln!(file, "{record}").map_err(|error| error.to_string())
}

fn append_blocked_trial(
    path: &Path,
    cfg: &Config,
    settings: &CashSettings,
    reason: &str,
) -> Result<(), String> {
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|error| format!("{}: {error}", path.display()))?;
    let record = serde_json::json!({
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "study": "s041_cash",
        "status": "blocked",
        "variant": cfg.variant,
        "parameters": trial_parameters(cfg, settings),
        "reason": reason
    });
    writeln!(file, "{record}").map_err(|error| error.to_string())
}

fn is_input_data_blocker(error: &str) -> bool {
    error.starts_with("missing raw open for ")
        || error.starts_with("unsupported corporate action ")
        || error.contains("has no raw execution series")
        || (error.starts_with("selected symbol ") && error.contains(" has no raw close"))
}

fn run_variant(
    args: &Args,
    variant: Option<&str>,
    random_draws: usize,
    inputs: &LoadedData,
    file_fingerprints: &[serde_json::Value],
    signals_cache: &mut BTreeMap<SignalKey, Signals>,
) -> Result<VariantOutcome, String> {
    let cli = Cli {
        data_dir: args.data_dir.clone(),
        symbols: args.symbols.clone(),
    };
    let cfg = config::load_config("s041_cash", Some(&args.config), &cli, variant)?;
    let settings = load_cash_settings(&args.config, variant, args.raw_data_dir.as_deref())?;
    let spinoffs = configured_spinoffs(&settings, &cfg.symbols)?;
    if input_identity(&cfg, &settings) != inputs.identity {
        return Err(
            "variant changes data inputs within a shared batch; run that variant separately".into(),
        );
    }
    let results = PathBuf::from("results").join("s041_cash");
    fs::create_dir_all(&results).map_err(|error| format!("{}: {error}", results.display()))?;
    let prefix = format!("{}__", cfg.variant);
    write_manifest(
        &results.join(format!("{prefix}manifest.json")),
        &cfg,
        &settings,
        inputs,
        file_fingerprints,
    )?;
    if args.check_data {
        println!(
            "Input check {}: {} signal symbols, {} raw files, {} missing raw files",
            cfg.variant,
            cfg.symbols.split(',').count(),
            cfg.symbols.split(',').count() - inputs.missing_raw.len(),
            inputs.missing_raw.len()
        );
        return Ok(VariantOutcome::CheckData);
    }
    let key = signal_key(&cfg);
    if !signals_cache.contains_key(&key) {
        let computed = signals::compute_s041_signals(&inputs.signal_panel, &cfg)?;
        signals_cache.insert(key.clone(), computed);
    }
    let signals = signals_cache
        .get(&key)
        .ok_or("cached S041 signals are missing")?;
    let start = evaluation_start(&cfg, signals, &inputs.signal_panel)?;
    let split = data::parse_date(&cfg.partition_split_date)?;
    let execution = CashExecution {
        initial_cash: settings.cash_initial_cash,
        long_fraction: settings.cash_long_fraction,
        short_fraction: settings.cash_short_fraction,
        short_count: settings.cash_short_count,
        max_participation: settings.cash_max_participation,
        max_liquidity_age_seconds: settings.cash_max_liquidity_age_seconds,
        max_pending_sessions: settings.cash_max_pending_sessions,
        max_stale_mark_sessions: settings.cash_max_stale_mark_sessions,
        dca_installments: settings.cash_dca_installments,
        stop_loss_fraction: settings.cash_stop_loss_fraction,
        pyramid_profit_fraction: settings.cash_pyramid_profit_fraction,
        pyramid_add_fraction: settings.cash_pyramid_add_fraction,
        slippage_bps: settings.cash_slippage_bps,
        commission_per_share: settings.cash_commission_per_share,
        annual_borrow_rate: settings.cash_annual_borrow_rate,
        delisting_return: settings.cash_delisting_return,
    };
    let replay = match run_cash_replay_with_spinoffs(
        &inputs.signal_panel,
        &inputs.raw_panel,
        &signals,
        &inputs.actions,
        &spinoffs,
        execution,
        &cfg.breadth_mode,
        cfg.breadth,
        cfg.min_breadth,
        cfg.decile,
    ) {
        Ok(replay) => replay,
        Err(error) if is_input_data_blocker(&error) => {
            append_blocked_trial(
                &PathBuf::from("results").join("trial_registry.jsonl"),
                &cfg,
                &settings,
                &error,
            )?;
            println!("S041 cash {}: BLOCKED: {error}", cfg.variant);
            return Ok(VariantOutcome::Blocked(BlockedVariant {
                variant: cfg.variant,
                reason: error,
            }));
        }
        Err(error) => return Err(error),
    };
    let (dev_random, test_random) = match random_baseline(
        &cfg,
        &inputs,
        &signals,
        execution,
        &spinoffs,
        random_draws,
        start,
        split,
    ) {
        Ok(random) => random,
        Err(error) if is_input_data_blocker(&error) => {
            append_blocked_trial(
                &PathBuf::from("results").join("trial_registry.jsonl"),
                &cfg,
                &settings,
                &error,
            )?;
            println!("S041 cash {}: BLOCKED: {error}", cfg.variant);
            return Ok(VariantOutcome::Blocked(BlockedVariant {
                variant: cfg.variant,
                reason: error,
            }));
        }
        Err(error) => return Err(error),
    };
    let dev = metrics(&replay, &inputs.benchmark, start, Some(split), dev_random);
    let test = metrics(&replay, &inputs.benchmark, split, None, test_random);
    let benchmark_dev_drawdown_pct = benchmark_max_drawdown_pct(&inputs.benchmark, start, split);
    write_daily(
        &results.join(format!("{prefix}daily_equity.csv")),
        &replay,
        &inputs.benchmark,
        start,
    )?;
    write_events(&results.join(format!("{prefix}orders_fills.csv")), &replay)?;
    write_actions(
        &results.join(format!("{prefix}corporate_actions.csv")),
        &replay,
    )?;
    write_lifecycle_events(
        &results.join(format!("{prefix}lifecycle_events.csv")),
        &replay,
    )?;
    write_trades(
        &results.join(format!("{prefix}completed_trades.csv")),
        &replay,
    )?;
    write_rejections(&results.join(format!("{prefix}rejections.csv")), &replay)?;
    write_missing_raw_symbols(
        &results.join(format!("{prefix}missing_raw_symbols.csv")),
        &inputs.missing_raw,
        &settings.raw_data_dir,
        &settings.raw_data_suffix,
    )?;
    write_price_gaps(
        &results.join(format!("{prefix}missing_open_quotes.csv")),
        &replay.missing_open_quotes,
    )?;
    write_price_gaps(
        &results.join(format!("{prefix}stale_close_marks.csv")),
        &replay.stale_marks,
    )?;
    write_plot(
        &results.join(format!("{prefix}cumulative_log_returns.svg")),
        &replay,
        &inputs.benchmark,
        start,
    )?;
    write_report(
        &results.join(format!("{prefix}report.md")),
        &cfg.variant,
        &replay,
        &dev,
        &test,
        &settings,
        random_draws,
        start,
        split,
        &inputs,
    )?;
    update_comparison(
        &results.join("variants_comparison.csv"),
        &cfg.variant,
        &dev,
        &test,
    )?;
    append_cost_ladder(
        &results.join("cost_ladder.csv"),
        &cfg.variant,
        &settings,
        &dev,
        &test,
    )?;
    append_trial(
        &PathBuf::from("results").join("trial_registry.jsonl"),
        &cfg,
        &settings,
        &dev,
        &test,
    )?;
    println!(
        "S041 cash {}: DEV skill {:.4}, TEST skill {:.4}; {} fills, {} rebalances, {} rejections",
        cfg.variant,
        dev.skill,
        test.skill,
        replay.account.events().len(),
        replay.rebalance_dates.len(),
        replay.rejections.len()
    );
    Ok(VariantOutcome::Completed(TrialResult {
        variant: cfg.variant,
        dev,
        test,
        benchmark_dev_drawdown_pct,
    }))
}

fn run() -> Result<(), String> {
    let args = Args::parse();
    let random_draws = args.random_draws.unwrap_or(100);
    if random_draws == 0 {
        return Err("--random-draws must be positive".into());
    }
    let variants = variants(args.variant.as_deref());
    let first_variant = variants.first().and_then(|variant| variant.as_deref());
    let cli = Cli {
        data_dir: args.data_dir.clone(),
        symbols: args.symbols.clone(),
    };
    let base_cfg = config::load_config("s041_cash", Some(&args.config), &cli, first_variant)?;
    let base_settings =
        load_cash_settings(&args.config, first_variant, args.raw_data_dir.as_deref())?;
    let inputs = load_data(&base_cfg, &base_settings)?;
    let file_fingerprints = fingerprint_inputs(&inputs)?;
    let mut trials = Vec::new();
    let mut blocked = Vec::new();
    let mut signals_cache = BTreeMap::new();
    for variant in variants {
        match run_variant(
            &args,
            variant.as_deref(),
            random_draws,
            &inputs,
            &file_fingerprints,
            &mut signals_cache,
        )? {
            VariantOutcome::Completed(trial) => trials.push(trial),
            VariantOutcome::Blocked(variant) => blocked.push(variant),
            VariantOutcome::CheckData => {}
        }
    }
    if !args.check_data {
        let benchmark_drawdown = trials
            .first()
            .map_or(0.0, |trial| trial.benchmark_dev_drawdown_pct);
        let latest_trials = read_latest_trials(
            &PathBuf::from("results")
                .join("s041_cash")
                .join("variants_comparison.csv"),
            benchmark_drawdown,
            &blocked,
        )?;
        write_selection_summary(
            &PathBuf::from("results").join("s041_cash"),
            &latest_trials,
            &blocked,
            random_draws,
        )?;
    }
    Ok(())
}

fn main() -> std::process::ExitCode {
    match run() {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error}");
            std::process::ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn panel_for_spin_off_test(dates: Vec<chrono::NaiveDate>, closes: Vec<Vec<f64>>) -> Panel {
        let rows = dates.len();
        let symbols = vec!["RTX".into(), "CARR".into(), "OTIS".into()];
        let opens = closes
            .iter()
            .map(|series| series.iter().map(|close| close * 0.99).collect())
            .collect();
        let highs = closes
            .iter()
            .map(|series| series.iter().map(|close| close * 1.01).collect())
            .collect();
        let lows = closes
            .iter()
            .map(|series| series.iter().map(|close| close * 0.98).collect())
            .collect();
        Panel {
            dates,
            symbols,
            open: opens,
            high: highs,
            low: lows,
            close: closes,
            volume: vec![vec![1_000.0; rows]; 3],
        }
    }

    #[test]
    fn spinoff_signal_correction_preserves_the_distributed_total_value_return() {
        let previous = data::parse_date("2020-04-02").unwrap();
        let ex_date = data::parse_date("2020-04-03").unwrap();
        let mut signal_panel = panel_for_spin_off_test(
            vec![previous, ex_date],
            vec![vec![100.0, 30.0], vec![10.0, 10.0], vec![20.0, 20.0]],
        );
        let raw_panel = panel_for_spin_off_test(
            vec![previous, ex_date],
            vec![vec![100.0, 50.0], vec![10.0, 10.0], vec![20.0, 20.0]],
        );
        let corrections = correct_signal_series_for_spinoffs(
            &mut signal_panel,
            &raw_panel,
            &[SpinOffAction {
                date: ex_date,
                source_symbol: "RTX".into(),
                distributions: vec![
                    SpinOffDistribution {
                        symbol: "CARR".into(),
                        shares_per_source_share: 1.0,
                    },
                    SpinOffDistribution {
                        symbol: "OTIS".into(),
                        shares_per_source_share: 0.5,
                    },
                ],
                source_url: "https://example.com/spin-off".into(),
            }],
        )
        .unwrap();

        assert!((signal_panel.close[0][1] - 70.0).abs() < 1e-10);
        assert!((signal_panel.open[0][1] - 30.0 * 0.99 * 7.0 / 3.0).abs() < 1e-10);
        assert_eq!(corrections.len(), 1);
        assert_eq!(corrections[0]["source_url"], "https://example.com/spin-off");
        assert!((corrections[0]["corrected_total_return"].as_f64().unwrap() - 0.7).abs() < 1e-10);
    }

    fn sample_metrics(skill: f64, test_like: f64) -> Metrics {
        Metrics {
            total_log: skill + 0.1,
            trend_log: 0.1,
            random_pick_log: test_like,
            skill,
            bias: 0.0,
            skill_pct: skill * 100.0,
            sharpe: skill,
            profit_factor: 1.0 + skill.max(0.0),
            max_drawdown_pct: -10.0,
            trades: 120,
            win_pct: 50.0,
        }
    }

    #[test]
    fn selection_is_dev_only_and_handles_no_eligible_variants() {
        let trials = vec![
            TrialResult {
                variant: "dev_eligible".into(),
                dev: sample_metrics(0.1, 0.0),
                test: sample_metrics(-0.2, 0.0),
                benchmark_dev_drawdown_pct: -30.0,
            },
            TrialResult {
                variant: "test_only_winner".into(),
                dev: sample_metrics(-0.1, 0.0),
                test: sample_metrics(1.0, 0.0),
                benchmark_dev_drawdown_pct: -30.0,
            },
        ];
        let output_dir =
            std::env::temp_dir().join(format!("s041-cash-selection-{}", std::process::id()));
        fs::create_dir_all(&output_dir).unwrap();
        write_selection_summary(&output_dir, &trials, &[], 1).unwrap();
        let selected: serde_json::Value =
            serde_json::from_slice(&fs::read(output_dir.join("selection.json")).unwrap()).unwrap();
        assert_eq!(selected["selected"]["variant"], "dev_eligible");
        assert_eq!(selected["trial_count"], 2);
        fs::remove_file(output_dir.join("selection.md")).unwrap();
        fs::remove_file(output_dir.join("selection.json")).unwrap();
        fs::remove_file(output_dir.join("blocked_variants.csv")).unwrap();
        fs::remove_dir(&output_dir).unwrap();

        let no_eligible = vec![TrialResult {
            variant: "none".into(),
            dev: sample_metrics(-0.1, 0.0),
            test: sample_metrics(2.0, 0.0),
            benchmark_dev_drawdown_pct: -30.0,
        }];
        assert_eq!(
            percentile_ranks(&no_eligible, &[], |metrics| metrics.skill),
            vec![0.0]
        );
    }

    #[test]
    fn only_known_execution_data_gaps_are_treated_as_blocked_trials() {
        assert!(is_input_data_blocker(
            "missing raw open for UTX on 2020-04-03"
        ));
        assert!(is_input_data_blocker(
            "unsupported corporate action \"merger\" for ABC on 2020-04-03"
        ));
        assert!(!is_input_data_blocker("cash account arithmetic overflow"));
    }
}
