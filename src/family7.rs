//! Family 7 ETF timing studies: regime switches, vol-compression breakout, pairs, copper/gold tilt.

use chrono::Datelike;

use crate::config::{split_csv, Config};
use crate::data::Panel;
use crate::engine::{last_valid_rows, Trade};
use crate::family2::blank_signals;
use crate::family4::{benchmark_returns, rebalance_days};
use crate::signals::{rolling_extreme, rolling_mean, wilder_atr, wilder_rsi, Signals};

pub const S069_NAME: &str = "s069_tlt_crash_hedge";
pub const S069_TITLE: &str = "S069 TLT Crash-Hedge Rotation";
pub const S070_NAME: &str = "s070_qqq_regime_tqqq";
pub const S070_TITLE: &str = "S070 QQQ Regime Switches with Real TQQQ";
pub const S102_NAME: &str = "s102_copper_gold_tilt";
pub const S102_TITLE: &str = "S102 Copper/Gold Ratio Equity Tilt";
pub const S065_NAME: &str = "s065_vol_compression";
pub const S065_TITLE: &str = "S065 Realized-Vol Compression Breakout";
pub const S071_NAME: &str = "s071_cointegrated_pairs";
pub const S071_TITLE: &str = "S071 Cointegrated ETF Pair Z-Score";
pub const S064_NAME: &str = "s064_hurst_regime";
pub const S064_TITLE: &str = "S064 Hurst Exponent Regime Switch";

fn symbol_index(panel: &Panel, name: &str) -> Result<usize, String> {
    panel
        .symbols
        .iter()
        .position(|symbol| symbol == name)
        .ok_or_else(|| format!("{name} is required in the universe"))
}

fn ret_series(close: &[f64]) -> Vec<f64> {
    let mut out = vec![f64::NAN; close.len()];
    for t in 1..close.len() {
        if close[t].is_finite() && close[t - 1].is_finite() && close[t - 1] > 0.0 {
            out[t] = close[t] / close[t - 1] - 1.0;
        }
    }
    out
}

fn realized_vol(returns: &[f64], day: usize, window: usize) -> f64 {
    if day + 1 < window {
        return f64::NAN;
    }
    let slice = &returns[day + 1 - window..=day];
    if slice.iter().any(|value| !value.is_finite()) {
        return f64::NAN;
    }
    let mean = slice.iter().sum::<f64>() / window as f64;
    (slice
        .iter()
        .map(|value| (value - mean).powi(2))
        .sum::<f64>()
        / (window - 1) as f64)
        .sqrt()
}

/// Per-day target-weight switches: a change in w[j][t] closes the old leg and opens
/// the new one at the configured fill (signal day t -> close[t] or open[t+1]).
fn switch_trades(
    panel: &Panel,
    weights: &[Vec<f64>],
    sig: &Signals,
    cfg: &Config,
    fill: &str,
) -> Result<Vec<Trade>, String> {
    if !matches!(fill, "close" | "next_open") {
        return Err(format!("{} does not support fill {fill:?}", cfg.strategy));
    }
    let n = panel.dates.len();
    let last = last_valid_rows(&panel.close);
    let borrow_per_bar = cfg.borrow_bps_annual / 10_000.0 / cfg.periods_per_year as f64;
    let mut trades = Vec::new();
    for j in 0..panel.symbols.len() {
        let Some(last_row) = last[j] else { continue };
        let cost = cfg.cost_for_symbol(&panel.symbols[j]);
        let mut open_pos: Option<(usize, usize, f64, f64)> = None; // (signal_day, entry_idx, entry_px, weight)
        let mut cur_w = 0.0f64;
        for t in 0..n {
            let w_raw = weights[j][t];
            let w = if w_raw.is_finite() { w_raw } else { cur_w };
            if (w - cur_w).abs() < 1e-9 {
                continue;
            }
            let fill_row = if fill == "next_open" { t + 1 } else { t };
            if fill_row > last_row || fill_row >= n {
                cur_w = w;
                continue;
            }
            let px = if fill == "next_open" {
                panel.open[j][fill_row]
            } else {
                panel.close[j][fill_row]
            };
            if !px.is_finite() || px <= 0.0 {
                cur_w = w;
                continue;
            }
            if let Some((sig_day, entry_idx, entry_px, old_w)) = open_pos.take() {
                let direction = old_w.signum();
                let gross = direction * (px / entry_px - 1.0);
                let borrow = if old_w < 0.0 {
                    borrow_per_bar * (fill_row - entry_idx + 1) as f64
                } else {
                    0.0
                };
                trades.push(Trade {
                    sym_idx: j,
                    signal_idx: sig_day,
                    entry_idx,
                    exit_idx: fill_row,
                    entry_px,
                    exit_px: px,
                    ret: gross - cost * 2.0 * old_w.abs() - borrow,
                    ret_gross: gross,
                    exit_reason: "rebalance",
                    hold: fill_row - entry_idx,
                    signal_value: sig.rsi[j][t],
                    weight: old_w,
                    entry_turnover: old_w.abs(),
                    exit_turnover: old_w.abs(),
                    weighted: true,
                    borrow_cost_per_bar: if old_w < 0.0 { borrow_per_bar } else { 0.0 },
                });
            }
            if w.abs() > 1e-9 {
                open_pos = Some((t, fill_row, px, w));
            }
            cur_w = w;
        }
        if let Some((sig_day, entry_idx, entry_px, w)) = open_pos {
            let px = panel.close[j][last_row];
            let direction = w.signum();
            let gross = direction * (px / entry_px - 1.0);
            let borrow = if w < 0.0 {
                borrow_per_bar * (last_row - entry_idx + 1) as f64
            } else {
                0.0
            };
            trades.push(Trade {
                sym_idx: j,
                signal_idx: sig_day,
                entry_idx,
                exit_idx: last_row,
                entry_px,
                exit_px: px,
                ret: gross - cost * 2.0 * w.abs() - borrow,
                ret_gross: gross,
                exit_reason: "censored",
                hold: last_row - entry_idx,
                signal_value: sig.rsi[j][last_row],
                weight: w,
                entry_turnover: w.abs(),
                exit_turnover: w.abs(),
                weighted: true,
                borrow_cost_per_bar: if w < 0.0 { borrow_per_bar } else { 0.0 },
            });
        }
    }
    trades.sort_by(|a, b| {
        (a.entry_idx, &panel.symbols[a.sym_idx]).cmp(&(b.entry_idx, &panel.symbols[b.sym_idx]))
    });
    Ok(trades)
}

// S069: SPY above SMA -> SPY, else defensive (TLT primary, BIL variant).
pub fn compute_s069_signals(panel: &Panel, cfg: &Config) -> Result<Signals, String> {
    if !matches!(cfg.strategy_mode.as_str(), "daily" | "monthly") {
        return Err(format!("S069 strategy_mode must be daily|monthly, got {}", cfg.strategy_mode));
    }
    let spy = symbol_index(panel, "SPY")?;
    let defensive = symbol_index(panel, &cfg.defensive_asset)?;
    let n = panel.dates.len();
    let sma = rolling_mean(&panel.close[spy], cfg.sma_window);
    let mut result = blank_signals(panel, "spy_sma_dist");
    let mut weights = vec![vec![f64::NAN; n]; panel.symbols.len()];
    let monthly = cfg.strategy_mode == "monthly";
    let mut regime_on = false;
    for t in 0..n {
        result.tradable[spy][t] = panel.close[spy][t].is_finite();
        result.tradable[defensive][t] = panel.close[defensive][t].is_finite();
        result.pool[spy][t] = result.tradable[spy][t];
        result.pool[defensive][t] = result.tradable[defensive][t];
        if !panel.close[spy][t].is_finite() || !sma[t].is_finite() {
            continue;
        }
        result.rsi[spy][t] = panel.close[spy][t] / sma[t] - 1.0;
        let month_edge = t + 1 == n
            || panel.dates[t].month() != panel.dates[t + 1].month()
            || panel.dates[t].year() != panel.dates[t + 1].year();
        if !monthly || month_edge {
            regime_on = panel.close[spy][t] > sma[t];
        }
        let (w_spy, w_def) = if regime_on { (1.0, 0.0) } else { (0.0, 1.0) };
        weights[spy][t] = w_spy;
        weights[defensive][t] = w_def;
        result.signal[spy][t] = regime_on;
        result.signal[defensive][t] = !regime_on;
    }
    result.trade_weights = Some(weights);
    result.benchmark_returns = benchmark_returns(panel, "SPY");
    Ok(result)
}

pub fn s069_trades(panel: &Panel, sig: &Signals, cfg: &Config, fill: &str) -> Result<Vec<Trade>, String> {
    let weights = sig
        .trade_weights
        .as_ref()
        .ok_or_else(|| "S069 requires target weights".to_string())?;
    switch_trades(panel, weights, sig, cfg, fill)
}

// S070: RSI(14) hysteresis -> TQQQ capped; else QQQ above SMA, else defensive.
pub fn compute_s070_signals(panel: &Panel, cfg: &Config) -> Result<Signals, String> {
    let qqq = symbol_index(panel, "QQQ")?;
    let tqqq = symbol_index(panel, "TQQQ")?;
    let defensive = symbol_index(panel, &cfg.defensive_asset)?;
    let n = panel.dates.len();
    let rsi = wilder_rsi(&panel.close[qqq], cfg.rsi_len)?;
    let sma = rolling_mean(&panel.close[qqq], cfg.sma_window);
    let mut result = blank_signals(panel, "qqq_rsi14");
    let mut weights = vec![vec![f64::NAN; n]; panel.symbols.len()];
    let mut oversold = false;
    for t in 0..n {
        for &j in &[qqq, tqqq, defensive] {
            result.tradable[j][t] = panel.close[j][t].is_finite();
            result.pool[j][t] = result.tradable[j][t];
        }
        result.rsi[qqq][t] = rsi[t];
        if !rsi[t].is_finite() || !sma[t].is_finite() || !panel.close[tqqq][t].is_finite() {
            continue;
        }
        if !oversold && rsi[t] < cfg.oversold_in {
            oversold = true;
        } else if oversold && rsi[t] > cfg.oversold_out {
            oversold = false;
        }
        if oversold {
            weights[tqqq][t] = cfg.levered_cap;
            result.signal[tqqq][t] = true;
        } else if panel.close[qqq][t] > sma[t] {
            weights[qqq][t] = 1.0;
            result.signal[qqq][t] = true;
        } else {
            weights[defensive][t] = 1.0;
            result.signal[defensive][t] = true;
        }
    }
    result.trade_weights = Some(weights);
    result.benchmark_returns = benchmark_returns(panel, "QQQ");
    Ok(result)
}

pub fn s070_trades(panel: &Panel, sig: &Signals, cfg: &Config, fill: &str) -> Result<Vec<Trade>, String> {
    let weights = sig
        .trade_weights
        .as_ref()
        .ok_or_else(|| "S070 requires target weights".to_string())?;
    switch_trades(panel, weights, sig, cfg, fill)
}

// S102: weekly copper/gold tilt between cyclical {XLB,XLI} and defensive {XLP,XLU} baskets.
pub fn compute_s102_signals(panel: &Panel, cfg: &Config) -> Result<Signals, String> {
    if !matches!(cfg.signal_mode.as_str(), "copper_gold" | "momentum") {
        return Err(format!(
            "S102 signal_mode must be copper_gold|momentum, got {}",
            cfg.signal_mode
        ));
    }
    let n = panel.dates.len();
    let mut result = blank_signals(panel, "copper_gold");
    let mut weights = vec![vec![0.0; n]; panel.symbols.len()];
    let cyclical: Vec<usize> = split_csv(&cfg.cyclical_symbols)
        .iter()
        .map(|name| symbol_index(panel, name))
        .collect::<Result<_, _>>()?;
    let defensive: Vec<usize> = split_csv(&cfg.defensive_symbols)
        .iter()
        .map(|name| symbol_index(panel, name))
        .collect::<Result<_, _>>()?;
    let (cper, gld) = (symbol_index(panel, "CPER")?, symbol_index(panel, "GLD")?);
    let mom_days = (cfg.mom_window / 5).max(1);
    let mut ratio = vec![f64::NAN; n];
    let mut ratio_z = vec![f64::NAN; n];
    for t in 0..n {
        for &j in cyclical.iter().chain(&defensive) {
            result.tradable[j][t] = panel.close[j][t].is_finite();
            result.pool[j][t] = result.tradable[j][t];
        }
        if !panel.close[cper][t].is_finite() || !panel.close[gld][t].is_finite() {
            continue;
        }
        ratio[t] = panel.close[cper][t] / panel.close[gld][t];
        if t + 1 > cfg.z_window {
            let slice = &ratio[t + 1 - cfg.z_window..=t];
            if !slice.iter().any(|v| !v.is_finite()) {
                let mean = slice.iter().sum::<f64>() / cfg.z_window as f64;
                let sd = (slice.iter().map(|v| (v - mean).powi(2)).sum::<f64>()
                    / (cfg.z_window - 1) as f64)
                    .sqrt();
                if sd > 0.0 {
                    ratio_z[t] = (ratio[t] - mean) / sd;
                }
            }
        }
        result.rsi[cper][t] = ratio_z[t];
    }
    for &day in &rebalance_days(panel, &cfg.rebalance)? {
        let pick_cyclical = match cfg.signal_mode.as_str() {
            "momentum" => {
                let cyc: f64 = cyclical
                    .iter()
                    .filter_map(|&j| {
                        (day >= mom_days && panel.close[j][day].is_finite()
                            && panel.close[j][day - mom_days].is_finite())
                        .then(|| panel.close[j][day] / panel.close[j][day - mom_days] - 1.0)
                    })
                    .sum::<f64>()
                    / cyclical.len().max(1) as f64;
                let def: f64 = defensive
                    .iter()
                    .filter_map(|&j| {
                        (day >= mom_days && panel.close[j][day].is_finite()
                            && panel.close[j][day - mom_days].is_finite())
                        .then(|| panel.close[j][day] / panel.close[j][day - mom_days] - 1.0)
                    })
                    .sum::<f64>()
                    / defensive.len().max(1) as f64;
                cyc > def
            }
            _ => {
                day >= mom_days
                    && ratio[day].is_finite()
                    && ratio[day - mom_days].is_finite()
                    && ratio_z[day].is_finite()
                    && ratio[day] > ratio[day - mom_days]
                    && ratio_z[day] > 0.0
            }
        };
        let basket = if pick_cyclical { &cyclical } else { &defensive };
        for &j in basket {
            weights[j][day] = 1.0 / basket.len().max(1) as f64;
            result.signal[j][day] = true;
        }
        result.rsi[cper][day] = ratio_z[day];
    }
    result.trade_weights = Some(weights);
    result.benchmark_returns = benchmark_returns(panel, "SPY");
    Ok(result)
}

// S065: RV short/long compression arms stop orders at the N-day extremes.
pub fn compute_s065_signals(panel: &Panel, cfg: &Config) -> Result<Signals, String> {
    if cfg.rv_short == 0 || cfg.rv_long <= cfg.rv_short {
        return Err(format!(
            "S065 needs 0 < rv_short < rv_long, got {}/{}",
            cfg.rv_short, cfg.rv_long
        ));
    }
    let mut result = blank_signals(panel, "rv_ratio");
    for j in 0..panel.symbols.len() {
        let returns = ret_series(&panel.close[j]);
        for t in 0..panel.dates.len() {
            if !panel.close[j][t].is_finite() {
                continue;
            }
            result.tradable[j][t] = true;
            result.pool[j][t] = true;
            let (rv_s, rv_l) = (realized_vol(&returns, t, cfg.rv_short), realized_vol(&returns, t, cfg.rv_long));
            if rv_s.is_finite() && rv_l.is_finite() && rv_l > 0.0 {
                result.rsi[j][t] = rv_s / rv_l;
                if rv_s < cfg.rv_ratio * rv_l {
                    result.signal[j][t] = true;
                }
            }
        }
    }
    result.benchmark_returns = benchmark_returns(panel, "SPY");
    Ok(result)
}

/// Daily-bar approximation of the intraday stop-entry/stop-exit rules: entry fills at
/// max(open, buy_stop)/min(open, sell_stop), exits fill through the opposite N-day
/// extreme, and both-side triggers resolve to the side nearer the open.
pub fn s065_trades(panel: &Panel, sig: &Signals, cfg: &Config, fill: &str) -> Result<Vec<Trade>, String> {
    if fill != "stop" {
        return Err(format!("S065 supports fill \"stop\" only, got {fill:?}"));
    }
    let last = last_valid_rows(&panel.close);
    let mut trades = Vec::new();
    for j in 0..panel.symbols.len() {
        let Some(last_row) = last[j] else { continue };
        let cost = cfg.cost_for_symbol(&panel.symbols[j]);
        let hi = rolling_extreme(&panel.high[j], cfg.entry_window, false);
        let lo = rolling_extreme(&panel.low[j], cfg.entry_window, true);
        let exit_hi = rolling_extreme(&panel.high[j], cfg.exit_extreme_window, false);
        let exit_lo = rolling_extreme(&panel.low[j], cfg.exit_extreme_window, true);
        let mut armed: Option<(usize, usize, f64, f64)> = None; // (arm_day, expires, buy_level, sell_level)
        let mut pos: Option<(usize, usize, f64, f64)> = None; // (sig_day, entry_idx, entry_px, dir)
        for t in 0..=last_row {
            // manage open position first (stops evaluated on the day bar)
            if let Some((sig_day, entry_idx, entry_px, dir)) = pos {
                if t > entry_idx {
                    let mut exit: Option<(f64, &'static str)> = None;
                    if dir > 0.0 && exit_lo[t - 1].is_finite() && panel.low[j][t] <= exit_lo[t - 1] {
                        exit = Some((panel.open[j][t].min(exit_lo[t - 1]), "stop"));
                    } else if dir < 0.0
                        && exit_hi[t - 1].is_finite()
                        && panel.high[j][t] >= exit_hi[t - 1]
                    {
                        exit = Some((panel.open[j][t].max(exit_hi[t - 1]), "stop"));
                    }
                    if exit.is_none() && t - entry_idx >= cfg.max_hold {
                        exit = Some((panel.close[j][t], "time"));
                    }
                    if let Some((px, reason)) = exit {
                        if px.is_finite() && px > 0.0 {
                            let gross = dir * (px / entry_px - 1.0);
                            let borrow = if dir < 0.0 {
                                cfg.borrow_bps_annual / 10_000.0 / cfg.periods_per_year as f64
                                    * (t - entry_idx + 1) as f64
                            } else {
                                0.0
                            };
                            trades.push(Trade {
                                sym_idx: j,
                                signal_idx: sig_day,
                                entry_idx,
                                exit_idx: t,
                                entry_px,
                                exit_px: px,
                                ret: gross - cost * 2.0 - borrow,
                                ret_gross: gross,
                                exit_reason: reason,
                                hold: t - entry_idx,
                                signal_value: sig.rsi[j][sig_day],
                                weight: 1.0,
                                entry_turnover: 1.0,
                                exit_turnover: 1.0,
                                weighted: false,
                                borrow_cost_per_bar: 0.0,
                            });
                            pos = None;
                        }
                    }
                }
                continue;
            }
            if pos.is_none() {
                // arm orders on the compression signal (orders valid from the next session)
                if sig.signal[j][t]
                    && hi[t].is_finite()
                    && lo[t].is_finite()
                    && t + 1 <= last_row
                {
                    armed = Some((t, t + cfg.order_validity, hi[t], lo[t]));
                }
                if let Some((arm_day, expires, buy_level, sell_level)) = armed {
                    if t > expires {
                        armed = None;
                        continue;
                    }
                    if t <= arm_day {
                        continue;
                    }
                    let buy_hit = panel.high[j][t] >= buy_level;
                    let sell_hit = panel.low[j][t] <= sell_level;
                    if !buy_hit && !sell_hit {
                        continue;
                    }
                    let dir = if buy_hit && sell_hit {
                        if (buy_level - panel.open[j][t]).abs() <= (panel.open[j][t] - sell_level).abs()
                        {
                            1.0
                        } else {
                            -1.0
                        }
                    } else if buy_hit {
                        1.0
                    } else {
                        -1.0
                    };
                    let px = if dir > 0.0 {
                        panel.open[j][t].max(buy_level)
                    } else {
                        panel.open[j][t].min(sell_level)
                    };
                    if !px.is_finite() || px <= 0.0 {
                        continue;
                    }
                    pos = Some((t - 1, t, px, dir));
                    armed = None;
                }
            }
        }
        if let Some((sig_day, entry_idx, entry_px, dir)) = pos {
            let px = panel.close[j][last_row];
            let gross = dir * (px / entry_px - 1.0);
            trades.push(Trade {
                sym_idx: j,
                signal_idx: sig_day,
                entry_idx,
                exit_idx: last_row,
                entry_px,
                exit_px: px,
                ret: gross - cost * 2.0,
                ret_gross: gross,
                exit_reason: "censored",
                hold: last_row - entry_idx,
                signal_value: sig.rsi[j][sig_day],
                weight: 1.0,
                entry_turnover: 1.0,
                exit_turnover: 1.0,
                weighted: false,
                borrow_cost_per_bar: 0.0,
            });
        }
    }
    trades.sort_by(|a, b| {
        (a.entry_idx, &panel.symbols[a.sym_idx]).cmp(&(b.entry_idx, &panel.symbols[b.sym_idx]))
    });
    Ok(trades)
}

// ---------------- S071 cointegrated ETF pairs ----------------

/// Small dense OLS: solve beta = argmin ||y - X beta||^2 via normal equations.
fn ols(x: &[Vec<f64>], y: &[f64]) -> Option<Vec<f64>> {
    let n = y.len();
    let p = x.len();
    let mut xtx = vec![vec![0.0; p]; p];
    let mut xty = vec![0.0; p];
    for row in 0..n {
        for a in 0..p {
            for b in 0..p {
                xtx[a][b] += x[a][row] * x[b][row];
            }
            xty[a] += x[a][row] * y[row];
        }
    }
    for i in 0..p {
        let mut pivot = i;
        for r in i + 1..p {
            if xtx[r][i].abs() > xtx[pivot][i].abs() {
                pivot = r;
            }
        }
        if xtx[pivot][i].abs() < 1e-12 {
            return None;
        }
        xtx.swap(i, pivot);
        xty.swap(i, pivot);
        for r in i + 1..p {
            let f = xtx[r][i] / xtx[i][i];
            for c in i..p {
                xtx[r][c] -= f * xtx[i][c];
            }
            xty[r] -= f * xty[i];
        }
    }
    let mut beta = vec![0.0; p];
    for i in (0..p).rev() {
        beta[i] = (xty[i] - (i + 1..p).map(|c| xtx[i][c] * beta[c]).sum::<f64>()) / xtx[i][i];
    }
    Some(beta)
}

fn ols_se(x: &[Vec<f64>], y: &[f64], beta: &[f64]) -> Option<Vec<f64>> {
    let n = y.len();
    let p = x.len();
    let mut resid = vec![0.0; n];
    for r in 0..n {
        resid[r] = y[r] - (0..p).map(|k| x[k][r] * beta[k]).sum::<f64>();
    }
    if n <= p {
        return None;
    }
    let sigma2 = resid.iter().map(|e| e * e).sum::<f64>() / (n - p) as f64;
    // invert X'X
    let mut xtx = vec![vec![0.0; p]; p];
    for r in 0..n {
        for a in 0..p {
            for b in 0..p {
                xtx[a][b] += x[a][r] * x[b][r];
            }
        }
    }
    // Gauss-Jordan on [X'X | I]
    let mut inv = vec![vec![0.0; p]; p];
    for i in 0..p {
        inv[i][i] = 1.0;
    }
    for i in 0..p {
        let mut pivot = i;
        for r in i + 1..p {
            if xtx[r][i].abs() > xtx[pivot][i].abs() {
                pivot = r;
            }
        }
        if xtx[pivot][i].abs() < 1e-12 {
            return None;
        }
        xtx.swap(i, pivot);
        inv.swap(i, pivot);
        let d = xtx[i][i];
        for c in 0..p {
            xtx[i][c] /= d;
            inv[i][c] /= d;
        }
        for r in 0..p {
            if r == i {
                continue;
            }
            let f = xtx[r][i];
            for c in 0..p {
                xtx[r][c] -= f * xtx[i][c];
                inv[r][c] -= f * inv[i][c];
            }
        }
    }
    Some((0..p).map(|k| (sigma2 * inv[k][k]).sqrt()).collect())
}

/// Engle-Granger residual ADF t-statistic on log-price residuals over a window.
fn pair_adf(log_a: &[f64], log_b: &[f64], lags: usize) -> Option<(f64, f64)> {
    let n = log_a.len();
    if n < 60 || lags + 2 >= n {
        return None;
    }
    // stage 1: logA = a + b*logB + e
    let xa = vec![vec![1.0; n], log_b.to_vec()];
    let beta = ols(&xa, log_a)?;
    let beta_b = beta[1];
    let resid: Vec<f64> = (0..n)
        .map(|t| log_a[t] - beta[0] - beta_b * log_b[t])
        .collect();
    // stage 2: d e_t = c + phi e_{t-1} + sum gamma_k d e_{t-k}
    let rows = n - 1 - lags;
    let mut x = vec![vec![0.0; rows]; 2 + lags];
    let mut y = vec![0.0; rows];
    for r in 0..rows {
        let t = lags + 1 + r;
        x[0][r] = 1.0;
        x[1][r] = resid[t - 1];
        for k in 0..lags {
            x[2 + k][r] = resid[t - 1 - k] - resid[t - 2 - k];
        }
        y[r] = resid[t] - resid[t - 1];
    }
    let beta2 = ols(&x, &y)?;
    let se = ols_se(&x, &y, &beta2)?;
    Some((beta2[1] / se[1], beta_b))
}

fn adf_critical(max_p: f64) -> f64 {
    // MacKinnon asymptotic values, constant included, no trend
    if max_p <= 0.01 {
        -3.43
    } else if max_p <= 0.05 {
        -2.86
    } else {
        -2.57
    }
}

struct PairState {
    a: usize,
    b: usize,
    active: bool,
    dir: f64,      // +1 long spread (long B, short A); -1 short spread
    hedge: f64,    // beta from Engle-Granger stage 1
    entry_px_a: f64,
    entry_px_b: f64,
    sig_day: usize,
    entry_idx: usize,
}

pub fn compute_s071_signals(panel: &Panel, cfg: &Config) -> Result<Signals, String> {
    let pairs: Vec<(String, String)> = split_csv(&cfg.pairs)
        .into_iter()
        .filter_map(|entry| {
            let mut parts = entry.split(':');
            Some((parts.next()?.to_string(), parts.next()?.to_string()))
        })
        .collect();
    if pairs.is_empty() {
        return Err("S071 requires a `pairs` list like 'SPY:IVV,TLT:IEF'".into());
    }
    let n = panel.dates.len();
    let mut result = blank_signals(panel, "pair_z");
    for j in 0..panel.symbols.len() {
        for t in 0..n {
            result.tradable[j][t] = panel.close[j][t].is_finite();
            result.pool[j][t] = result.tradable[j][t];
        }
    }
    // monthly verification rows
    let formation = cfg.formation_window;
    for pair in &pairs {
        let a = symbol_index(panel, &pair.0)?;
        let b = symbol_index(panel, &pair.1)?;
        for t in formation..n {
            let month_edge = t + 1 == n
                || panel.dates[t].month() != panel.dates[t + 1].month()
                || panel.dates[t].year() != panel.dates[t + 1].year();
            if !month_edge {
                continue;
            }
            let win: Vec<f64> = (t + 1 - formation..=t).map(|u| panel.close[a][u].ln()).collect();
            let winb: Vec<f64> = (t + 1 - formation..=t).map(|u| panel.close[b][u].ln()).collect();
            if win.iter().chain(&winb).any(|v| !v.is_finite()) {
                continue;
            }
            if let Some((tstat, _hedge)) = pair_adf(&win, &winb, cfg.adf_lags) {
                if tstat < adf_critical(cfg.adf_max_p) {
                    result.rsi[a][t] = tstat;
                }
            }
        }
    }
    result.benchmark_returns = benchmark_returns(panel, "SPY");
    Ok(result)
}

/// Monthly ADF-verified z-band entries and exits; two weighted legs per pair.
pub fn s071_trades(panel: &Panel, _sig: &Signals, cfg: &Config, fill: &str) -> Result<Vec<Trade>, String> {
    if !matches!(fill, "close" | "next_open") {
        return Err(format!("S071 does not support fill {fill:?}"));
    }
    let pairs: Vec<(String, String)> = split_csv(&cfg.pairs)
        .into_iter()
        .filter_map(|entry| {
            let mut parts = entry.split(':');
            Some((parts.next()?.to_string(), parts.next()?.to_string()))
        })
        .collect();
    let n = panel.dates.len();
    let borrow_per_bar = cfg.borrow_bps_annual / 10_000.0 / cfg.periods_per_year as f64;
    let mut trades = Vec::new();
    for pair in &pairs {
        let a = symbol_index(panel, &pair.0)?;
        let b = symbol_index(panel, &pair.1)?;
        let mut state = PairState {
            a,
            b,
            active: false,
            dir: 0.0,
            hedge: 1.0,
            entry_px_a: 0.0,
            entry_px_b: 0.0,
            sig_day: 0,
            entry_idx: 0,
        };
        let log_a: Vec<f64> = panel.close[a].iter().map(|v| if v.is_finite() && *v > 0.0 { v.ln() } else { f64::NAN }).collect();
        let log_b: Vec<f64> = panel.close[b].iter().map(|v| if v.is_finite() && *v > 0.0 { v.ln() } else { f64::NAN }).collect();
        let resid_at = |t: usize, hedge: f64| -> f64 {
            if log_a[t].is_finite() && log_b[t].is_finite() {
                log_a[t] - hedge * log_b[t]
            } else {
                f64::NAN
            }
        };
        for t in 0..n {
            let month_edge = t + 1 == n
                || panel.dates[t].month() != panel.dates[t + 1].month()
                || panel.dates[t].year() != panel.dates[t + 1].year();
            if month_edge && t >= cfg.formation_window {
                let win: Vec<f64> = (t + 1 - cfg.formation_window..=t).map(|u| log_a[u]).collect();
                let winb: Vec<f64> = (t + 1 - cfg.formation_window..=t).map(|u| log_b[u]).collect();
                let adf = if win.iter().chain(&winb).any(|v| !v.is_finite()) {
                    None
                } else {
                    pair_adf(&win, &winb, cfg.adf_lags)
                };
                state.active = adf
                    .as_ref()
                    .is_some_and(|(tstat, _)| *tstat < adf_critical(cfg.adf_max_p));
                if let Some((_, hedge)) = &adf {
                    state.hedge = hedge.max(0.05);
                }
                if !state.active && state.dir != 0.0 {
                    // monthly re-verification failed: close at fill
                    let fill_row = if fill == "next_open" { t + 1 } else { t };
                    if fill_row < n {
                        let px_a = if fill == "next_open" { panel.open[a][fill_row] } else { panel.close[a][fill_row] };
                        let px_b = if fill == "next_open" { panel.open[b][fill_row] } else { panel.close[b][fill_row] };
                        push_pair_legs(&mut trades, panel, cfg, &state, px_a, px_b, fill_row, "inactive", borrow_per_bar);
                        state.dir = 0.0;
                    }
                }
            }
            if state.dir == 0.0 {
                if !state.active || t + 1 < cfg.z_window {
                    continue;
                }
                let slice: Vec<f64> = (t + 1 - cfg.z_window..=t)
                    .map(|u| resid_at(u, state.hedge))
                    .collect();
                if slice.iter().any(|v| !v.is_finite()) {
                    continue;
                }
                let mean = slice.iter().sum::<f64>() / cfg.z_window as f64;
                let sd = (slice.iter().map(|v| (v - mean).powi(2)).sum::<f64>()
                    / (cfg.z_window - 1) as f64)
                    .sqrt();
                if sd <= 0.0 {
                    continue;
                }
                let z = (resid_at(t, state.hedge) - mean) / sd;
                // spread = log(Pa/Pb): high z -> short A, long hedge*B
                let dir = if z > cfg.z_entry {
                    -1.0
                } else if z < -cfg.z_entry {
                    1.0
                } else {
                    continue;
                };
                let fill_row = if fill == "next_open" { t + 1 } else { t };
                if fill_row >= n {
                    continue;
                }
                let px_a = if fill == "next_open" { panel.open[a][fill_row] } else { panel.close[a][fill_row] };
                let px_b = if fill == "next_open" { panel.open[b][fill_row] } else { panel.close[b][fill_row] };
                if !px_a.is_finite() || !px_b.is_finite() || px_a <= 0.0 || px_b <= 0.0 {
                    continue;
                }
                state.dir = dir;
                state.entry_px_a = px_a;
                state.entry_px_b = px_b;
                state.sig_day = t;
                state.entry_idx = fill_row;
            } else if state.dir != 0.0 {
                let slice: Vec<f64> = (t + 1 - cfg.z_window..=t)
                    .map(|u| resid_at(u, state.hedge))
                    .collect();
                if slice.iter().any(|v| !v.is_finite()) {
                    continue;
                }
                let mean = slice.iter().sum::<f64>() / cfg.z_window as f64;
                let sd = (slice.iter().map(|v| (v - mean).powi(2)).sum::<f64>()
                    / (cfg.z_window - 1) as f64)
                    .sqrt();
                if sd <= 0.0 {
                    continue;
                }
                let z = (resid_at(t, state.hedge) - mean) / sd;
                let exit = (state.dir < 0.0 && z < cfg.z_exit_short)
                    || (state.dir > 0.0 && z > cfg.z_exit_long);
                if exit {
                    let fill_row = if fill == "next_open" { t + 1 } else { t };
                    if fill_row >= n {
                        continue;
                    }
                    let px_a = if fill == "next_open" { panel.open[a][fill_row] } else { panel.close[a][fill_row] };
                    let px_b = if fill == "next_open" { panel.open[b][fill_row] } else { panel.close[b][fill_row] };
                    push_pair_legs(&mut trades, panel, cfg, &state, px_a, px_b, fill_row, "z_exit", borrow_per_bar);
                    state.dir = 0.0;
                }
            }
        }
        if state.dir != 0.0 {
            push_pair_legs(
                &mut trades,
                panel,
                cfg,
                &state,
                panel.close[a][n - 1],
                panel.close[b][n - 1],
                n - 1,
                "censored",
                borrow_per_bar,
            );
        }
    }
    trades.sort_by(|a, b| {
        (a.entry_idx, &panel.symbols[a.sym_idx]).cmp(&(b.entry_idx, &panel.symbols[b.sym_idx]))
    });
    Ok(trades)
}

#[allow(clippy::too_many_arguments)]
fn push_pair_legs(
    trades: &mut Vec<Trade>,
    panel: &Panel,
    cfg: &Config,
    state: &PairState,
    px_a: f64,
    px_b: f64,
    exit_idx: usize,
    reason: &'static str,
    borrow_per_bar: f64,
) {
    if !px_a.is_finite() || !px_b.is_finite() {
        return;
    }
    let hedge = state.hedge;
    // dollar weights normalized so |wa| + |wb| = 1; dir=-1 => short A, long B
    let wa = state.dir * 1.0 / (1.0 + hedge);
    let wb = -state.dir * hedge / (1.0 + hedge);
    for (j, w, entry_px, exit_px) in [
        (state.a, wa, state.entry_px_a, px_a),
        (state.b, wb, state.entry_px_b, px_b),
    ] {
        if w.abs() < 1e-9 {
            continue;
        }
        let direction = w.signum();
        let gross = direction * (exit_px / entry_px - 1.0);
        let cost = cfg.cost_for_symbol(&panel.symbols[j]);
        let borrow = if w < 0.0 {
            borrow_per_bar * (exit_idx - state.entry_idx + 1) as f64
        } else {
            0.0
        };
        trades.push(Trade {
            sym_idx: j,
            signal_idx: state.sig_day,
            entry_idx: state.entry_idx,
            exit_idx,
            entry_px,
            exit_px,
            ret: gross - cost * 2.0 * w.abs() - borrow,
            ret_gross: gross,
            exit_reason: reason,
            hold: exit_idx - state.entry_idx,
            signal_value: state.dir,
            weight: w,
            entry_turnover: w.abs(),
            exit_turnover: w.abs(),
            weighted: true,
            borrow_cost_per_bar: if w < 0.0 { borrow_per_bar } else { 0.0 },
        });
    }
}

// ---------------- S064 Hurst exponent regime switch ----------------

const S064_CASH: i32 = 0;
const S064_MR: i32 = 1;
const S064_TREND: i32 = 2;

fn log_ret_series(close: &[f64]) -> Vec<f64> {
    let mut out = vec![f64::NAN; close.len()];
    for t in 1..close.len() {
        if close[t].is_finite() && close[t - 1].is_finite() && close[t] > 0.0 && close[t - 1] > 0.0
        {
            out[t] = (close[t] / close[t - 1]).ln();
        }
    }
    out
}

/// Chan's variance term structure: Var(tau-period log returns) ~ tau^(2H);
/// H = slope of log-variance vs log-horizon over powers of two, divided by 2.
fn hurst_exponent(returns: &[f64], max_horizon: usize) -> f64 {
    let w = returns.len();
    if w < 9 {
        return f64::NAN;
    }
    let mut prefix = vec![0.0; w + 1];
    for (i, &r) in returns.iter().enumerate() {
        prefix[i + 1] = prefix[i] + r;
    }
    let mut xs = Vec::new();
    let mut ys = Vec::new();
    let mut tau = 1usize;
    while tau <= max_horizon && tau < w {
        let n_tau = w - tau + 1;
        let (mut sum, mut sum2) = (0.0, 0.0);
        for i in 0..n_tau {
            let r = prefix[i + tau] - prefix[i];
            sum += r;
            sum2 += r * r;
        }
        let mean = sum / n_tau as f64;
        let var = sum2 / n_tau as f64 - mean * mean;
        if var.is_finite() && var > 0.0 {
            xs.push((tau as f64).ln());
            ys.push(var.ln());
        }
        tau *= 2;
    }
    if xs.len() < 3 {
        return f64::NAN;
    }
    crate::stats::ols(&ys, &xs).0 / 2.0
}

/// Point estimate plus a moving-block bootstrap CI of H (seeded per eval day).
fn hurst_with_ci(returns: &[f64], cfg: &Config, seed: u64) -> (f64, f64, f64) {
    let h = hurst_exponent(returns, cfg.hurst_max_horizon);
    let w = returns.len();
    let block = cfg.hurst_block.max(1);
    if !h.is_finite() || cfg.hurst_boot == 0 || w <= block + 8 {
        return (h, f64::NAN, f64::NAN);
    }
    let n_start = w - block + 1;
    let mut rng = crate::stats::Rng::new(seed);
    let mut boots = Vec::with_capacity(cfg.hurst_boot);
    let mut sample = vec![0.0; w];
    for _ in 0..cfg.hurst_boot {
        let mut pos = 0usize;
        while pos < w {
            let start = rng.below(n_start);
            let take = block.min(w - pos);
            sample[pos..pos + take].copy_from_slice(&returns[start..start + take]);
            pos += take;
        }
        let hb = hurst_exponent(&sample, cfg.hurst_max_horizon);
        if hb.is_finite() {
            boots.push(hb);
        }
    }
    if boots.len() < 10 {
        return (h, f64::NAN, f64::NAN);
    }
    boots.sort_by(f64::total_cmp);
    let tail = (1.0 - cfg.hurst_ci) / 2.0;
    (
        h,
        crate::stats::quantile_sorted(&boots, tail),
        crate::stats::quantile_sorted(&boots, 1.0 - tail),
    )
}

fn s064_classify(h: f64, lo: f64, hi: f64, cfg: &Config) -> i32 {
    if cfg.hurst_gate == "bands" {
        if h < cfg.hurst_band_lo {
            S064_MR
        } else if h > cfg.hurst_band_hi {
            S064_TREND
        } else {
            S064_CASH
        }
    } else if hi.is_finite() && hi < 0.5 {
        S064_MR
    } else if lo.is_finite() && lo > 0.5 {
        S064_TREND
    } else {
        S064_CASH
    }
}

// S064: weekly Hurst gate; mean-reverting -> S001 RSI(2) leg, trending -> S036 Donchian leg.
pub fn compute_s064_signals(panel: &Panel, cfg: &Config) -> Result<Signals, String> {
    if !matches!(cfg.hurst_gate.as_str(), "ci" | "bands" | "static") {
        return Err(format!(
            "S064 hurst_gate must be ci|bands|static, got {}",
            cfg.hurst_gate
        ));
    }
    if !matches!(cfg.hurst_neutral.as_str(), "cash" | "hold") {
        return Err(format!(
            "S064 hurst_neutral must be cash|hold, got {}",
            cfg.hurst_neutral
        ));
    }
    if cfg.hurst_window <= cfg.hurst_max_horizon {
        return Err(format!(
            "S064 needs hurst_window > hurst_max_horizon, got {}/{}",
            cfg.hurst_window, cfg.hurst_max_horizon
        ));
    }
    if cfg.mr_stop_atr > 0.0 && cfg.disaster_stop_atr <= 0.0 {
        return Err("S064 mr_stop_atr requires disaster_stop_atr > 0".into());
    }
    if !matches!(cfg.gate.as_str(), "sma200" | "ret21" | "none") {
        return Err(format!("S064 gate must be sma200|ret21|none, got {}", cfg.gate));
    }
    if !matches!(cfg.exit_mode.as_str(), "sma5" | "rsi") {
        return Err(format!(
            "S064 exit_mode must be sma5|rsi, got {}",
            cfg.exit_mode
        ));
    }
    let n = panel.dates.len();
    let mut is_eval = vec![false; n];
    for &day in &rebalance_days(panel, &cfg.rebalance)? {
        is_eval[day] = true;
    }
    let mut result = blank_signals(panel, "hurst");
    let mut add = vec![vec![false; n]; panel.symbols.len()];
    let mut atr_stop = vec![vec![f64::NAN; n]; panel.symbols.len()];
    let mut regime_csv = String::from("symbol,date,h,ci_lo,ci_hi,label\n");
    let mut summary_csv =
        String::from("symbol,evals,mr_weeks,trend_weeks,cash_weeks,persist_frac,label_autocorr,h_mean\n");
    let label_name = |l: i32| match l {
        S064_MR => "mr",
        S064_TREND => "trend",
        _ => "cash",
    };
    for j in 0..panel.symbols.len() {
        let close = &panel.close[j];
        let rsi = wilder_rsi(close, cfg.rsi_len)?;
        let sma_gate = rolling_mean(close, cfg.sma_window);
        let sma_exit = rolling_mean(close, cfg.exit_sma);
        let hi_d = rolling_extreme(&panel.high[j], cfg.entry_window, false);
        let lo_d = rolling_extreme(&panel.low[j], cfg.exit_extreme_window, true);
        let atr_d = wilder_atr(&panel.high[j], &panel.low[j], close, cfg.atr_len);
        let atr_mr = wilder_atr(&panel.high[j], &panel.low[j], close, 14);
        let lr = log_ret_series(close);
        let mut labels = vec![S064_CASH; n];
        let mut eval_labels: Vec<i32> = Vec::new();
        let (mut h_sum, mut h_cnt) = (0.0, 0usize);
        if cfg.hurst_gate == "static" {
            // instrument-level classification on the full sample, once per symbol
            let all: Vec<f64> = lr.iter().copied().filter(|v| v.is_finite()).collect();
            let (h, lo, hi) = hurst_with_ci(&all, cfg, cfg.seed.wrapping_add(j as u64 * 1_000_003));
            let label = s064_classify(h, lo, hi, cfg);
            for l in labels.iter_mut() {
                *l = label;
            }
            for t in 0..n {
                result.rsi[j][t] = h;
            }
            eval_labels.push(label);
            if h.is_finite() {
                h_sum += h;
                h_cnt += 1;
            }
            if let Some(&last) = panel.dates.last() {
                regime_csv += &format!(
                    "{},{last},{h},{lo},{hi},{}\n",
                    panel.symbols[j],
                    label_name(label)
                );
            }
        } else {
            let mut label = S064_CASH;
            let w = cfg.hurst_window;
            for t in 0..n {
                if is_eval[t] && t >= w {
                    let win = &lr[t + 1 - w..=t];
                    if win.iter().all(|v| v.is_finite()) {
                        let seed = cfg
                            .seed
                            .wrapping_add((j as u64) << 32)
                            .wrapping_add(t as u64);
                        let (h, lo, hi) = if cfg.hurst_gate == "bands" {
                            (hurst_exponent(win, cfg.hurst_max_horizon), f64::NAN, f64::NAN)
                        } else {
                            hurst_with_ci(win, cfg, seed)
                        };
                        let next = s064_classify(h, lo, hi, cfg);
                        label = if next == S064_CASH && cfg.hurst_neutral == "hold" {
                            label
                        } else {
                            next
                        };
                        result.rsi[j][t] = h;
                        if h.is_finite() {
                            h_sum += h;
                            h_cnt += 1;
                        }
                        eval_labels.push(label);
                        regime_csv += &format!(
                            "{},{},{h},{lo},{hi},{}\n",
                            panel.symbols[j],
                            panel.dates[t],
                            label_name(label)
                        );
                    }
                }
                labels[t] = label;
            }
            let mut last_h = f64::NAN;
            for t in 0..n {
                if result.rsi[j][t].is_finite() {
                    last_h = result.rsi[j][t];
                } else {
                    result.rsi[j][t] = last_h;
                }
            }
        }
        let evals = eval_labels.len();
        if evals > 0 {
            let mr_n = eval_labels.iter().filter(|&&l| l == S064_MR).count();
            let tr_n = eval_labels.iter().filter(|&&l| l == S064_TREND).count();
            let persist = if evals > 1 {
                eval_labels.windows(2).filter(|w| w[0] == w[1]).count() as f64
                    / (evals - 1) as f64
            } else {
                f64::NAN
            };
            let nums: Vec<f64> = eval_labels.iter().map(|&l| l as f64).collect();
            let autocorr = if evals > 2 {
                crate::stats::corr(&nums[..evals - 1], &nums[1..])
            } else {
                f64::NAN
            };
            summary_csv += &format!(
                "{},{evals},{mr_n},{tr_n},{},{persist},{autocorr},{}\n",
                panel.symbols[j],
                evals - mr_n - tr_n,
                if h_cnt > 0 { h_sum / h_cnt as f64 } else { f64::NAN }
            );
        }
        let mut hist = 0usize;
        let mut shadow: Option<(usize, f64, f64)> = None; // (entry_idx, entry_px, stop)
        let mut last_shadow_win = false;
        for t in 0..n {
            if close[t].is_finite() {
                hist += 1;
            }
            let trad = close[t].is_finite() && hist >= cfg.min_history;
            result.tradable[j][t] = trad;
            result.pool[j][t] = trad;
            if !close[t].is_finite() {
                continue;
            }
            let label = labels[t];
            if label != S064_TREND {
                shadow = None;
                last_shadow_win = false;
            }
            if label == S064_TREND {
                if let Some((_e, s_px, s_stop)) = shadow {
                    let stop_hit = s_stop.is_finite() && panel.low[j][t] <= s_stop;
                    let chan_exit = t >= 1 && lo_d[t - 1].is_finite() && close[t] < lo_d[t - 1];
                    if stop_hit || chan_exit {
                        let exit_px = if stop_hit {
                            panel.open[j][t].min(s_stop)
                        } else {
                            close[t]
                        };
                        last_shadow_win = exit_px > s_px;
                        shadow = None;
                    }
                }
            }
            let mr_exit = if cfg.exit_mode == "rsi" {
                rsi[t] > cfg.exit_rsi
            } else {
                close[t] > sma_exit[t]
            };
            let d_exit = t >= 1 && lo_d[t - 1].is_finite() && close[t] < lo_d[t - 1];
            let leg_exit = if cfg.regime_exit {
                match label {
                    S064_MR => mr_exit,
                    S064_TREND => d_exit,
                    _ => true,
                }
            } else {
                mr_exit || d_exit
            };
            let switched = cfg.regime_exit && t >= 1 && labels[t] != labels[t - 1];
            result.exit_now[j][t] = leg_exit || switched;
            if !trad {
                continue;
            }
            add[j][t] = label == S064_MR && rsi[t] < cfg.add_rsi;
            if label == S064_MR {
                let gate_ok = match cfg.gate.as_str() {
                    "ret21" => {
                        t >= cfg.gate_ret_window
                            && close[t - cfg.gate_ret_window].is_finite()
                            && close[t] / close[t - cfg.gate_ret_window] - 1.0 > 0.0
                    }
                    "none" => true,
                    _ => close[t] > sma_gate[t],
                };
                if gate_ok && rsi[t] < cfg.rsi_threshold {
                    result.signal[j][t] = true;
                    if cfg.mr_stop_atr > 0.0 && atr_mr[t].is_finite() {
                        // engine multiplies by disaster_stop_atr; pre-scale to mr_stop_atr
                        atr_stop[j][t] = atr_mr[t] * cfg.mr_stop_atr / cfg.disaster_stop_atr;
                    }
                }
            } else if label == S064_TREND {
                let breakout = t >= 1 && hi_d[t - 1].is_finite() && close[t] > hi_d[t - 1];
                if breakout && shadow.is_none() {
                    if !cfg.donchian_skip_winner || !last_shadow_win {
                        result.signal[j][t] = true;
                        if cfg.disaster_stop_atr > 0.0 && atr_d[t].is_finite() {
                            atr_stop[j][t] = atr_d[t];
                        }
                    }
                    if t + 1 < n && panel.open[j][t + 1].is_finite() {
                        let px = panel.open[j][t + 1];
                        let stop = if cfg.disaster_stop_atr > 0.0 && atr_d[t].is_finite() {
                            px - cfg.disaster_stop_atr * atr_d[t]
                        } else {
                            f64::NAN
                        };
                        shadow = Some((t + 1, px, stop));
                    }
                }
            }
        }
    }
    result.add_signal = if cfg.sizing == "pyramid3" {
        Some(add)
    } else {
        None
    };
    result.atr = if cfg.disaster_stop_atr > 0.0 {
        Some(atr_stop)
    } else {
        None
    };
    result.benchmark_returns =
        benchmark_returns(panel, &cfg.benchmark).or_else(|| benchmark_returns(panel, "SPY"));
    let dir = std::path::Path::new("results").join(&cfg.strategy);
    let _ = std::fs::create_dir_all(&dir);
    let _ = std::fs::write(
        dir.join(format!("{}hurst_regimes.csv", cfg.prefix())),
        regime_csv,
    );
    let _ = std::fs::write(
        dir.join(format!("{}hurst_summary.csv", cfg.prefix())),
        summary_csv,
    );
    Ok(result)
}

pub fn s064_trades(panel: &Panel, sig: &Signals, cfg: &Config, fill: &str) -> Result<Vec<Trade>, String> {
    let params = crate::engine::SwingParams {
        signal: &sig.signal,
        exit_now: &sig.exit_now,
        add_signal: sig.add_signal.as_ref(),
        max_units: cfg.max_units,
        stop_atr: sig.atr.as_ref(),
        stop_pct: sig.stop_pct.as_ref(),
        stop_mult: cfg.disaster_stop_atr,
        signal_value: &sig.rsi,
    };
    Ok(crate::engine::swing_trades(panel, &params, cfg, fill))
}
