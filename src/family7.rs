//! Family 7 ETF timing studies: regime switches, vol-compression breakout, pairs, copper/gold tilt.

use chrono::Datelike;

use crate::config::{split_csv, Config};
use crate::data::Panel;
use crate::engine::{last_valid_rows, Trade};
use crate::family2::blank_signals;
use crate::family4::{benchmark_returns, rebalance_days};
use crate::signals::{rolling_extreme, rolling_mean, wilder_rsi, Signals};

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
pub const S066_NAME: &str = "s066_zweig_breadth_thrust";
pub const S066_TITLE: &str = "S066 Zweig Breadth Thrust";

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

// ---------------- S066 Zweig breadth thrust ----------------

/// TA-Lib-style EMA: seeds on the SMA of the first `len` finite values, NaN-safe.
fn ema(series: &[f64], len: usize) -> Vec<f64> {
    let alpha = 2.0 / (len as f64 + 1.0);
    let mut out = vec![f64::NAN; series.len()];
    let mut state = f64::NAN;
    let mut warm: Vec<f64> = Vec::with_capacity(len);
    for (t, &x) in series.iter().enumerate() {
        if !x.is_finite() {
            continue;
        }
        if state.is_nan() {
            warm.push(x);
            if warm.len() == len {
                state = warm.iter().sum::<f64>() / len as f64;
            }
        } else {
            state = alpha * x + (1.0 - alpha) * state;
        }
        out[t] = state;
    }
    out
}

// S066: EMA(10) of the own-universe advance ratio thrusts from <thrust_low to
// >thrust_high within thrust_window sessions -> long the traded symbol; exit on
// close < SMA(sma_window) (or optional defensive asset / max_hold).
pub fn compute_s066_signals(panel: &Panel, cfg: &Config) -> Result<Signals, String> {
    if !matches!(cfg.breadth_universe.as_str(), "liquid" | "all") {
        return Err(format!(
            "S066 breadth_universe must be liquid|all, got {}",
            cfg.breadth_universe
        ));
    }
    if !matches!(cfg.breadth_metric.as_str(), "adv_dec" | "pct_above_sma") {
        return Err(format!(
            "S066 breadth_metric must be adv_dec|pct_above_sma, got {}",
            cfg.breadth_metric
        ));
    }
    let spy = symbol_index(panel, &cfg.traded_symbol)?;
    let defensive: Option<usize> = if matches!(cfg.defensive_asset.as_str(), "" | "cash" | "flat") {
        None
    } else {
        Some(symbol_index(panel, &cfg.defensive_asset)?)
    };
    let n = panel.dates.len();
    let member_cols: Vec<usize> = (0..panel.symbols.len())
        .filter(|&j| {
            j != spy
                && Some(j) != defensive
                && cfg
                    .symbol_sources
                    .get(&panel.symbols[j])
                    .map_or(true, |source| source == &cfg.breadth_source)
        })
        .collect();
    if member_cols.is_empty() {
        return Err(format!(
            "S066 found no breadth members in source {:?}",
            cfg.breadth_source
        ));
    }
    let ranks = if cfg.breadth_universe == "liquid" {
        Some(crate::family5::load_ranks(panel, cfg, &member_cols)?)
    } else {
        None
    };
    let member = |j: usize, t: usize| -> bool {
        match &ranks {
            Some(ranks) => ranks[j][t].is_finite() && ranks[j][t] <= cfg.universe_top_n as f64,
            None => true,
        }
    };
    // daily breadth ratio series
    let mut ratio = vec![f64::NAN; n];
    if cfg.breadth_metric == "pct_above_sma" {
        let sma: Vec<Vec<f64>> = member_cols
            .iter()
            .map(|&j| rolling_mean(&panel.close[j], cfg.breadth_sma_len))
            .collect();
        for t in 0..n {
            let (mut above, mut total) = (0usize, 0usize);
            for (k, &j) in member_cols.iter().enumerate() {
                if member(j, t) && panel.close[j][t].is_finite() && sma[k][t].is_finite() {
                    total += 1;
                    if panel.close[j][t] > sma[k][t] {
                        above += 1;
                    }
                }
            }
            if total >= cfg.min_peers {
                ratio[t] = above as f64 / total as f64;
            }
        }
    } else {
        for t in 1..n {
            let (mut adv, mut dec) = (0usize, 0usize);
            for &j in &member_cols {
                let (a, b) = (panel.close[j][t - 1], panel.close[j][t]);
                if !member(j, t) || !a.is_finite() || a <= 0.0 || !b.is_finite() {
                    continue;
                }
                if b > a {
                    adv += 1;
                } else if b < a {
                    dec += 1;
                }
            }
            if adv + dec >= cfg.min_peers {
                ratio[t] = adv as f64 / (adv + dec) as f64;
            }
        }
    }
    let osc = if cfg.breadth_metric == "adv_dec" {
        ema(&ratio, cfg.breadth_ema_len.max(1))
    } else {
        ratio.clone()
    };
    let exit_sma = rolling_mean(&panel.close[spy], cfg.sma_window);
    let mut result = blank_signals(panel, "breadth_thrust_ema");
    let mut weights = vec![vec![f64::NAN; n]; panel.symbols.len()];
    let mut holding = false;
    let mut entry_day = 0usize;
    for t in 0..n {
        for j in 0..panel.symbols.len() {
            let tradable = (j == spy || Some(j) == defensive) && panel.close[j][t].is_finite();
            result.tradable[j][t] = tradable;
            result.pool[j][t] = tradable;
        }
        result.rsi[spy][t] = osc[t];
        if !osc[t].is_finite() || !exit_sma[t].is_finite() || !panel.close[spy][t].is_finite() {
            continue;
        }
        let crossed = osc[t] > cfg.thrust_high && t >= 1 && osc[t - 1] <= cfg.thrust_high;
        let dipped = (t.saturating_sub(cfg.thrust_window.saturating_sub(1))..=t)
            .any(|u| osc[u].is_finite() && osc[u] < cfg.thrust_low);
        let thrust = crossed && dipped;
        if thrust {
            holding = true;
            entry_day = t;
        } else if holding
            && (panel.close[spy][t] < exit_sma[t]
                || (cfg.max_hold > 0 && t.saturating_sub(entry_day) >= cfg.max_hold))
        {
            holding = false;
        }
        weights[spy][t] = if holding { 1.0 } else { 0.0 };
        if let Some(def) = defensive {
            weights[def][t] = if holding { 0.0 } else { 1.0 };
        }
        result.signal[spy][t] = holding;
    }
    result.trade_weights = Some(weights);
    result.benchmark_returns = benchmark_returns(panel, &cfg.traded_symbol);
    Ok(result)
}

pub fn s066_trades(panel: &Panel, sig: &Signals, cfg: &Config, fill: &str) -> Result<Vec<Trade>, String> {
    let weights = sig
        .trade_weights
        .as_ref()
        .ok_or_else(|| "S066 requires target weights".to_string())?;
    switch_trades(panel, weights, sig, cfg, fill)
}
