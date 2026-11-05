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
    #[serde(default = "d_accel_measure")]
    pub accel_measure: String,
    #[serde(default = "d_accel_gate")]
    pub accel_gate: bool,
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
    #[serde(default = "d_true")]
    pub include_etfs: bool,
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
    #[serde(default)]
    pub turtle_last_trade_filter: bool,
    #[serde(default = "d_turtle_filter_outcome")]
    pub turtle_filter_outcome: String,
    #[serde(default = "d_turtle_exit_mode")]
    pub turtle_exit_mode: String,
    #[serde(default = "d_turtle_chandelier_len")]
    pub turtle_chandelier_len: usize,
    #[serde(default = "d_turtle_chandelier_mult")]
    pub turtle_chandelier_mult: f64,
    #[serde(default = "d_turtle_risk_pct")]
    pub turtle_risk_pct: f64,
    #[serde(default = "d_turtle_max_position_weight")]
    pub turtle_max_position_weight: f64,
    #[serde(default)]
    pub turtle_failsafe_window: usize,
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
    pub news_file: String,
    #[serde(default)]
    pub sentiment_file: String,
    #[serde(default)]
    pub edgar_filings_file: String,
    #[serde(default)]
    pub pairs: String,
    #[serde(default)]
    pub sector_etf_map: String,
    #[serde(default = "d_entry_sessions_before")]
    pub entry_sessions_before: usize,
    #[serde(default = "d_expected_shift_days")]
    pub expected_shift_days: i64,
    #[serde(default = "d_surprise_mode")]
    pub surprise_mode: String,
    #[serde(default = "d_score_mode")]
    pub score_mode: String,
    #[serde(default = "d_leg_mode")]
    pub leg_mode: String,
    #[serde(default = "d_rv_short")]
    pub rv_short: usize,
    #[serde(default = "d_rv_long")]
    pub rv_long: usize,
    #[serde(default = "d_rv_ratio")]
    pub rv_ratio: f64,
    #[serde(default = "d_order_validity")]
    pub order_validity: usize,
    #[serde(default = "d_exit_extreme_window")]
    pub exit_extreme_window: usize,
    #[serde(default = "d_oversold_in")]
    pub oversold_in: f64,
    #[serde(default = "d_oversold_out")]
    pub oversold_out: f64,
    #[serde(default = "d_levered_cap")]
    pub levered_cap: f64,
    #[serde(default = "d_z_entry")]
    pub z_entry: f64,
    #[serde(default = "d_z_exit_short")]
    pub z_exit_short: f64,
    #[serde(default = "d_z_exit_long")]
    pub z_exit_long: f64,
    #[serde(default = "d_adf_max_p")]
    pub adf_max_p: f64,
    #[serde(default = "d_z_stop")]
    pub z_stop: f64,
    #[serde(default = "d_hedge_mode")]
    pub hedge_mode: String,
    #[serde(default = "d_overlay_mode")]
    pub overlay_mode: String,
    #[serde(default = "d_dd_threshold")]
    pub dd_threshold: f64,
    #[serde(default = "d_dd_high_window")]
    pub dd_high_window: usize,
    #[serde(default = "d_reentry_window")]
    pub reentry_window: usize,
    #[serde(default = "d_vol_scale_target")]
    pub vol_scale_target: f64,
    #[serde(default = "d_vol_scale_window")]
    pub vol_scale_window: usize,
    #[serde(default = "d_vol_cap")]
    pub vol_cap: f64,
    #[serde(default = "d_weighting_mode")]
    pub weighting_mode: String,
    #[serde(default = "d_volume_lookback_months")]
    pub volume_lookback_months: i64,
    #[serde(default = "d_volume_lag_months")]
    pub volume_lag_months: i64,
    #[serde(default = "d_volume_quintile")]
    pub volume_quintile: f64,
    #[serde(default = "d_required_reports")]
    pub required_reports: i64,
    #[serde(default)]
    pub beta_neutral: bool,
    #[serde(default = "d_exclude_early_close")]
    pub exclude_early_close: bool,
    #[serde(default = "d_entry_offset")]
    pub entry_offset: i64,
    #[serde(default = "d_exit_offset")]
    pub exit_offset: i64,
    #[serde(default = "d_weakness_threshold")]
    pub weakness_threshold: f64,
    #[serde(default = "d_weekday")]
    pub weekday: i64,
    #[serde(default = "d_opex_mode")]
    pub opex_mode: String,
    #[serde(default = "d_entry_mode")]
    pub entry_mode: String,
    #[serde(default = "d_topic_mode")]
    pub topic_mode: String,
    #[serde(default = "d_adf_lags")]
    pub adf_lags: usize,
    #[serde(default = "d_cyclical_symbols")]
    pub cyclical_symbols: String,
    #[serde(default = "d_defensive_symbols")]
    pub defensive_symbols: String,
    #[serde(default = "d_streak_days")]
    pub streak_days: usize,
    #[serde(default = "d_direction")]
    pub direction: String,
    #[serde(default)]
    pub vix_file: String,
    #[serde(default = "d_vix_ma_short")]
    pub vix_ma_short: usize,
    #[serde(default = "d_vix_ma_long")]
    pub vix_ma_long: usize,
    #[serde(default = "d_vix_fall_window")]
    pub vix_fall_window: usize,
    #[serde(default = "d_risk_symbol")]
    pub risk_symbol: String,
    #[serde(default)]
    pub price_sma_gate: usize,
    #[serde(default)]
    pub vix_level_exit: f64,
    #[serde(default = "d_setup_mode")]
    pub setup_mode: String,
    #[serde(default)]
    pub setup_atr_ratio: f64,
    #[serde(default = "d_hurst_window")]
    pub hurst_window: usize,
    #[serde(default = "d_hurst_max_horizon")]
    pub hurst_max_horizon: usize,
    #[serde(default = "d_hurst_boot")]
    pub hurst_boot: usize,
    #[serde(default = "d_hurst_block")]
    pub hurst_block: usize,
    #[serde(default = "d_hurst_ci")]
    pub hurst_ci: f64,
    #[serde(default = "d_hurst_gate")]
    pub hurst_gate: String,
    #[serde(default = "d_hurst_band_lo")]
    pub hurst_band_lo: f64,
    #[serde(default = "d_hurst_band_hi")]
    pub hurst_band_hi: f64,
    #[serde(default = "d_hurst_neutral")]
    pub hurst_neutral: String,
    #[serde(default = "d_regime_exit")]
    pub regime_exit: bool,
    #[serde(default = "d_donchian_skip_winner")]
    pub donchian_skip_winner: bool,
    #[serde(default)]
    pub mr_stop_atr: f64,
    #[serde(default = "d_traded_symbol")]
    pub traded_symbol: String,
    #[serde(default = "d_breadth_source")]
    pub breadth_source: String,
    #[serde(default = "d_breadth_universe")]
    pub breadth_universe: String,
    #[serde(default = "d_breadth_metric")]
    pub breadth_metric: String,
    #[serde(default = "d_breadth_ema_len")]
    pub breadth_ema_len: usize,
    #[serde(default = "d_breadth_sma_len")]
    pub breadth_sma_len: usize,
    #[serde(default = "d_thrust_low")]
    pub thrust_low: f64,
    #[serde(default = "d_thrust_high")]
    pub thrust_high: f64,
    #[serde(default = "d_thrust_window")]
    pub thrust_window: usize,
    #[serde(default = "d_s089_fast_window")]
    pub s089_fast_window: usize,
    #[serde(default = "d_s089_slow_window")]
    pub s089_slow_window: usize,
    #[serde(default = "d_s089_slope_lookback")]
    pub s089_slope_lookback: usize,
    #[serde(default)]
    pub s089_slope_min: f64,
    #[serde(default = "d_s089_require_slope")]
    pub s089_require_slope: bool,
    #[serde(default)]
    pub s089_fast_slope: usize,
    #[serde(default)]
    pub s089_vol_mult: f64,
    #[serde(default = "d_s089_vol_window")]
    pub s089_vol_window: usize,
    #[serde(default = "d_s089_exit_mode")]
    pub s089_exit_mode: String,
    #[serde(default)]
    pub s089_max_hold: usize,
    #[serde(default = "d_ema_len")]
    pub ema_len: usize,
    #[serde(default = "d_keltner_mult")]
    pub keltner_mult: f64,
    #[serde(default = "d_volume_mult")]
    pub volume_mult: f64,
    #[serde(default = "d_volume_median_window")]
    pub volume_median_window: usize,
    #[serde(default = "d_kama_er_len")]
    pub kama_er_len: usize,
    #[serde(default = "d_kama_fast")]
    pub kama_fast: usize,
    #[serde(default = "d_kama_slow")]
    pub kama_slow: usize,
    #[serde(default = "d_kama_slope_lookback")]
    pub kama_slope_lookback: usize,
    #[serde(default = "d_kama_slope_floor")]
    pub kama_slope_floor: f64,
    #[serde(default = "d_kama_side")]
    pub kama_side: String,
    #[serde(default = "d_kama_gate")]
    pub kama_gate: String,
    #[serde(default = "d_rv_gate")]
    pub rv_gate: String,
    #[serde(default)]
    pub rv_max: f64,
    #[serde(default = "d_rv_quiet_lookback")]
    pub rv_quiet_lookback: usize,
    #[serde(default = "d_bb_len")]
    pub bb_len: usize,
    #[serde(default = "d_bb_mult")]
    pub bb_mult: f64,
    #[serde(default = "d_width_lookback")]
    pub width_lookback: usize,
    #[serde(default)]
    pub squeeze_lag: usize,
    #[serde(default)]
    pub require_first_break: bool,
    #[serde(default)]
    pub vol_confirm_mult: f64,
    #[serde(default)]
    pub min_hold: usize,
    #[serde(default = "d_soldier_len")]
    pub soldier_len: usize,
    #[serde(default)]
    pub min_body_pct: f64,
    #[serde(default)]
    pub min_body_atr: f64,
    #[serde(default = "d_open_in_body")]
    pub open_in_body: bool,
    #[serde(default = "d_stop_mode")]
    pub stop_mode: String,
    #[serde(default = "d_stop_atr_mult")]
    pub stop_atr_mult: f64,
    #[serde(default = "d_max_rsi")]
    pub max_rsi: f64,
    #[serde(default)]
    pub results_tag: String,
    #[serde(default = "d_macd_fast")]
    pub macd_fast: usize,
    #[serde(default = "d_macd_slow")]
    pub macd_slow: usize,
    #[serde(default = "d_macd_signal")]
    pub macd_signal: usize,
    #[serde(default = "d_volume_multiple")]
    pub volume_multiple: f64,
    #[serde(default = "d_volume_window")]
    pub volume_window: usize,
    #[serde(default = "d_volume_mode")]
    pub volume_mode: String,
    #[serde(default = "d_require_macd_positive")]
    pub require_macd_positive: bool,
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
    #[serde(default)]
    pub fred_dir: String,
    #[serde(default = "d_rate_change_window")]
    pub rate_change_window: usize,
    #[serde(default)]
    pub rate_dead_band: f64,
    #[serde(default)]
    pub level_threshold: f64,
    #[serde(default = "d_band_mode")]
    pub band_mode: String,
    #[serde(default = "d_financial_symbol")]
    pub financial_symbol: String,
    #[serde(default = "d_utility_symbol")]
    pub utility_symbol: String,
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
fn d_accel_measure() -> String {
    "diff".into()
}
fn d_accel_gate() -> bool {
    true
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
fn d_turtle_filter_outcome() -> String {
    "stop".into()
}
fn d_turtle_exit_mode() -> String {
    "channel".into()
}
fn d_turtle_chandelier_len() -> usize {
    22
}
fn d_turtle_chandelier_mult() -> f64 {
    3.0
}
fn d_turtle_risk_pct() -> f64 {
    0.01
}
fn d_turtle_max_position_weight() -> f64 {
    0.5
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
fn d_entry_sessions_before() -> usize {
    6
}
fn d_expected_shift_days() -> i64 {
    364
}
fn d_surprise_mode() -> String {
    "sue".into()
}
fn d_score_mode() -> String {
    "finbert".into()
}
fn d_leg_mode() -> String {
    "long".into()
}
fn d_rv_short() -> usize {
    5
}
fn d_rv_long() -> usize {
    20
}
fn d_rv_ratio() -> f64 {
    0.5
}
fn d_order_validity() -> usize {
    5
}
fn d_exit_extreme_window() -> usize {
    10
}
fn d_oversold_in() -> f64 {
    29.7
}
fn d_oversold_out() -> f64 {
    30.3
}
fn d_levered_cap() -> f64 {
    1.0
}
fn d_z_entry() -> f64 {
    2.0
}
fn d_z_exit_short() -> f64 {
    0.75
}
fn d_z_exit_long() -> f64 {
    -0.5
}
fn d_adf_max_p() -> f64 {
    0.05
}
fn d_z_stop() -> f64 {
    3.0
}
fn d_hedge_mode() -> String {
    "ols".into()
}
fn d_overlay_mode() -> String {
    "none".into()
}
fn d_dd_threshold() -> f64 {
    0.10
}
fn d_dd_high_window() -> usize {
    252
}
fn d_reentry_window() -> usize {
    63
}
fn d_vol_scale_target() -> f64 {
    0.12
}
fn d_vol_scale_window() -> usize {
    126
}
fn d_vol_cap() -> f64 {
    1.0
}
fn d_weighting_mode() -> String {
    "equal".into()
}
fn d_volume_lookback_months() -> i64 {
    12
}
fn d_volume_lag_months() -> i64 {
    1
}
fn d_volume_quintile() -> f64 {
    0.20
}
fn d_required_reports() -> i64 {
    4
}
fn d_exclude_early_close() -> bool {
    true
}
fn d_entry_offset() -> i64 {
    -2
}
fn d_exit_offset() -> i64 {
    3
}
fn d_weakness_threshold() -> f64 {
    0.0
}
fn d_weekday() -> i64 {
    1
}
fn d_opex_mode() -> String {
    "all".into()
}
fn d_entry_mode() -> String {
    "close".into()
}
fn d_topic_mode() -> String {
    "all".into()
}
fn d_adf_lags() -> usize {
    1
}
fn d_streak_days() -> usize {
    5
}
fn d_direction() -> String {
    "short".into()
}
fn d_cyclical_symbols() -> String {
    "XLB,XLI".into()
}
fn d_hurst_window() -> usize {
    504
}
fn d_hurst_max_horizon() -> usize {
    64
}
fn d_hurst_boot() -> usize {
    200
}
fn d_hurst_block() -> usize {
    21
}
fn d_hurst_ci() -> f64 {
    0.95
}
fn d_hurst_gate() -> String {
    "ci".into()
}
fn d_hurst_band_lo() -> f64 {
    0.45
}
fn d_hurst_band_hi() -> f64 {
    0.55
}
fn d_hurst_neutral() -> String {
    "cash".into()
}
fn d_regime_exit() -> bool {
    true
}
fn d_donchian_skip_winner() -> bool {
    true
}
fn d_defensive_symbols() -> String {
    "XLP,XLU".into()
}
fn d_vix_ma_short() -> usize {
    20
}
fn d_vix_ma_long() -> usize {
    50
}
fn d_vix_fall_window() -> usize {
    5
}
fn d_risk_symbol() -> String {
    "SPY".into()
}
fn d_setup_mode() -> String {
    "nr7_or_inside".into()
}
fn d_macd_fast() -> usize {
    12
}
fn d_macd_slow() -> usize {
    26
}
fn d_macd_signal() -> usize {
    9
}
fn d_volume_multiple() -> f64 {
    1.3
}
fn d_volume_window() -> usize {
    20
}
fn d_volume_mode() -> String {
    "median".into()
}
fn d_require_macd_positive() -> bool {
    true
}
fn d_traded_symbol() -> String {
    "SPY".into()
}
fn d_breadth_source() -> String {
    "stocks".into()
}
fn d_breadth_universe() -> String {
    "liquid".into()
}
fn d_breadth_metric() -> String {
    "adv_dec".into()
}
fn d_breadth_ema_len() -> usize {
    10
}
fn d_breadth_sma_len() -> usize {
    50
}
fn d_thrust_low() -> f64 {
    0.40
}
fn d_thrust_high() -> f64 {
    0.615
}
fn d_thrust_window() -> usize {
    10
}
fn d_s089_fast_window() -> usize {
    50
}
fn d_s089_slow_window() -> usize {
    200
}
fn d_s089_slope_lookback() -> usize {
    20
}
fn d_s089_require_slope() -> bool {
    true
}
fn d_s089_vol_window() -> usize {
    20
}
fn d_s089_exit_mode() -> String {
    "cross".into()
}
fn d_true() -> bool {
    true
}
fn d_ema_len() -> usize {
    20
}
fn d_keltner_mult() -> f64 {
    2.0
}
fn d_volume_mult() -> f64 {
    1.3
}
fn d_volume_median_window() -> usize {
    20
}
fn d_kama_er_len() -> usize {
    10
}
fn d_kama_fast() -> usize {
    2
}
fn d_kama_slow() -> usize {
    30
}
fn d_kama_slope_lookback() -> usize {
    5
}
fn d_kama_slope_floor() -> f64 {
    0.999
}
fn d_kama_side() -> String {
    "long".into()
}
fn d_kama_gate() -> String {
    "none".into()
}
fn d_rv_gate() -> String {
    "none".into()
}
fn d_rv_quiet_lookback() -> usize {
    126
}
fn d_bb_len() -> usize {
    20
}
fn d_bb_mult() -> f64 {
    2.0
}
fn d_width_lookback() -> usize {
    120
}
fn d_rate_change_window() -> usize {
    21
}
fn d_financial_symbol() -> String {
    "XLF".into()
}
fn d_utility_symbol() -> String {
    "XLU".into()
}
fn d_band_mode() -> String {
    "binary".into()
}
fn d_soldier_len() -> usize {
    3
}
fn d_open_in_body() -> bool {
    true
}
fn d_stop_mode() -> String {
    "first_low".into()
}
fn d_stop_atr_mult() -> f64 {
    2.0
}
fn d_max_rsi() -> f64 {
    101.0
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
