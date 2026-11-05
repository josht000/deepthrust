//! Family 9 trend and pullback systems (stocks).

use crate::config::Config;
use crate::data::{self, Panel};
use crate::engine::{self, SwingParams, Trade};
use crate::signals::{self, Signals};

pub const S089_NAME: &str = "s089_golden_cross_slope";
pub const S089_TITLE: &str = "S089 Golden Cross with Slope Filter";

/// S089: SMA(fast) crosses above SMA(slow) while SMA(slow) is rising over the
/// slope lookback; exit on the opposite cross (or a configured alternate).
pub fn compute_s089_signals(panel: &Panel, cfg: &Config) -> Result<Signals, String> {
    if cfg.s089_fast_window == 0
        || cfg.s089_slow_window == 0
        || cfg.s089_fast_window >= cfg.s089_slow_window
    {
        return Err("S089 requires 0 < s089_fast_window < s089_slow_window".into());
    }
    if cfg.s089_require_slope && cfg.s089_slope_lookback == 0 {
        return Err("S089 slope filter requires s089_slope_lookback > 0".into());
    }
    if cfg.s089_exit_mode == "slope" && cfg.s089_slope_lookback == 0 {
        return Err("S089 slope exit requires s089_slope_lookback > 0".into());
    }
    if matches!(cfg.s089_exit_mode.as_str(), "time" | "cross_or_time")
        && cfg.s089_max_hold == 0
    {
        return Err("S089 timed exits require s089_max_hold > 0".into());
    }
    if !matches!(
        cfg.s089_exit_mode.as_str(),
        "cross" | "sma_slow" | "slope" | "time" | "cross_or_time"
    ) {
        return Err(format!(
            "unknown S089 s089_exit_mode {:?}",
            cfg.s089_exit_mode
        ));
    }
    let slope = cfg.s089_slope_lookback;
    let ranks = if cfg.rank_cache_dir.trim().is_empty() {
        let stock_columns: Vec<usize> = (0..panel.symbols.len())
            .filter(|&j| {
                cfg.symbol_sources
                    .get(&panel.symbols[j])
                    .map(String::as_str)
                    != Some("etfs")
            })
            .collect();
        data::liquidity_ranks(
            panel,
            &stock_columns,
            cfg.rank_window,
            cfg.rank_min_observations,
            cfg.min_price,
            cfg.min_history,
        )?
    } else {
        data::read_liquid_rank_cache(&cfg.rank_cache_dir, &panel.dates, &panel.symbols)?
    };
    let mut result = crate::family4::blank_signals(panel, "golden_cross_slope");
    for symbol in 0..panel.symbols.len() {
        let etf = cfg
            .symbol_sources
            .get(&panel.symbols[symbol])
            .map(String::as_str)
            == Some("etfs");
        let (close, volume) = (&panel.close[symbol], &panel.volume[symbol]);
        let sma_fast = signals::rolling_mean(close, cfg.s089_fast_window);
        let sma_slow = signals::rolling_mean(close, cfg.s089_slow_window);
        let vol_median = (cfg.s089_vol_mult > 0.0)
            .then(|| signals::rolling_median(volume, cfg.s089_vol_window));
        let (mut history, mut in_pos, mut sig_day) = (0usize, false, 0usize);
        for day in 0..panel.dates.len() {
            if close[day].is_finite() {
                history += 1;
            }
            let liquid = etf
                || (ranks[symbol][day].is_finite()
                    && ranks[symbol][day] <= cfg.universe_top_n as f64);
            if !close[day].is_finite() || !liquid || history < cfg.min_history {
                continue;
            }
            result.tradable[symbol][day] = true;
            result.pool[symbol][day] = true;
            if sma_fast[day].is_finite() && sma_slow[day].is_finite() && sma_slow[day] > 0.0 {
                result.rsi[symbol][day] = sma_fast[day] / sma_slow[day] - 1.0;
            }
            let (f_now, s_now) = (sma_fast[day], sma_slow[day]);
            let (f_prev, s_prev) = if day > 0 {
                (sma_fast[day - 1], sma_slow[day - 1])
            } else {
                (f64::NAN, f64::NAN)
            };
            let cross_up = f_now.is_finite()
                && s_now.is_finite()
                && f_prev.is_finite()
                && s_prev.is_finite()
                && f_now > s_now
                && f_prev <= s_prev;
            let cross_dn = f_now.is_finite()
                && s_now.is_finite()
                && f_prev.is_finite()
                && s_prev.is_finite()
                && f_now < s_now
                && f_prev >= s_prev;
            // Exit decided on completed-session data; engine fills at close[day]
            // ("close" fill) or open[day+1] ("next_open" fill).
            let mut exited_today = false;
            if in_pos && day > sig_day {
                let hit = match cfg.s089_exit_mode.as_str() {
                    "cross" => cross_dn,
                    "sma_slow" => s_now.is_finite() && close[day] < s_now,
                    "slope" => {
                        day >= slope
                            && s_now.is_finite()
                            && sma_slow[day - slope].is_finite()
                            && s_now < sma_slow[day - slope]
                    }
                    "time" => day - sig_day >= cfg.s089_max_hold,
                    "cross_or_time" => cross_dn || day - sig_day >= cfg.s089_max_hold,
                    _ => false,
                };
                if hit {
                    result.exit_now[symbol][day] = true;
                    in_pos = false;
                    exited_today = true;
                }
            }
            let slope_ok = !cfg.s089_require_slope
                || (day >= slope
                    && s_now.is_finite()
                    && sma_slow[day - slope].is_finite()
                    && s_now > sma_slow[day - slope] * (1.0 + cfg.s089_slope_min));
            let fast_ok = cfg.s089_fast_slope == 0
                || (day >= cfg.s089_fast_slope
                    && f_now.is_finite()
                    && sma_fast[day - cfg.s089_fast_slope].is_finite()
                    && f_now > sma_fast[day - cfg.s089_fast_slope]);
            let vol_ok = cfg.s089_vol_mult <= 0.0
                || vol_median.as_ref().is_some_and(|median| {
                    median[day].is_finite() && volume[day] > cfg.s089_vol_mult * median[day]
                });
            if cross_up && slope_ok && fast_ok && vol_ok {
                result.signal[symbol][day] = true;
                if !in_pos && !exited_today {
                    in_pos = true;
                    sig_day = day;
                }
            }
        }
    }
    if cfg.disaster_stop_atr > 0.0 {
        result.atr = Some(
            (0..panel.symbols.len())
                .map(|j| {
                    signals::wilder_atr(
                        &panel.high[j],
                        &panel.low[j],
                        &panel.close[j],
                        cfg.atr_len,
                    )
                })
                .collect(),
        );
    }
    result.benchmark_returns = crate::family4::benchmark_returns(panel, "SPY");
    Ok(result)
}

pub fn s089_trades(
    panel: &Panel,
    sig: &Signals,
    cfg: &Config,
    fill: &str,
) -> Result<Vec<Trade>, String> {
    if !matches!(fill, "close" | "next_open") {
        return Err(format!("S089 does not support fill {fill:?}"));
    }
    let params = SwingParams {
        signal: &sig.signal,
        exit_now: &sig.exit_now,
        add_signal: None,
        max_units: 1,
        stop_atr: sig.atr.as_ref(),
        stop_pct: None,
        stop_mult: cfg.disaster_stop_atr,
        signal_value: &sig.rsi,
    };
    Ok(engine::swing_trades(panel, &params, cfg, fill))
}
