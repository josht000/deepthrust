//! GitHub-catalog tier-3 strategies: tschm CTA 1.0-5.0 experiments (G077-G081),
//! futures->ETF DELTA factors (G085/G095/G106/G109/G063), crypto z-score blend
//! (G010), NSE->US swing scanner (G114), VIX term-structure pair study (G139).

use crate::config::Config;
use crate::data::Panel;
use crate::engine::Trade;
use crate::family4::{
    benchmark_returns, blank_rebalance_signals, momentum_matrix, rebalance_days,
};
use crate::signals::{Signals};

pub const G077_NAME: &str = "g077_cta_binary";
pub const G077_TITLE: &str = "G077 CTA 1.0 Binary EWM Crossover";
pub const G078_NAME: &str = "g078_cta_volscaled";
pub const G078_TITLE: &str = "G078 CTA 2.0 Vol-Scaled EWM Crossover";
pub const G079_NAME: &str = "g079_cta_robust";
pub const G079_TITLE: &str = "G079 CTA 3.0 Robust Vol-Adjusted Oscillator";
pub const G080_NAME: &str = "g080_cta_riskparity";
pub const G080_TITLE: &str = "G080 CTA 4.0 Oscillator + Risk-Parity Allocation";
pub const G081_NAME: &str = "g081_cta_dcc";
pub const G081_TITLE: &str = "G081 CTA 5.0 DCC Correlation Risk Parity";
pub const G085_NAME: &str = "g085_local_extreme";
pub const G085_TITLE: &str = "G085 Local-Extreme Breakout (ETF proxy)";
pub const G095_NAME: &str = "g095_ssa_trend";
pub const G095_TITLE: &str = "G095 SSA First-Component Trend Signal";
pub const G106_NAME: &str = "g106_xs_com_mom";
pub const G106_TITLE: &str = "G106 Cross-Sectional 252d Commodity Momentum";
pub const G109_NAME: &str = "g109_xs_skew";
pub const G109_TITLE: &str = "G109 Cross-Sectional Skewness Risk Premium";
pub const G063_NAME: &str = "g063_tsy_spread";
pub const G063_TITLE: &str = "G063 Treasury Z/T-Score RV Entry (TLT/IEF proxy)";
pub const G010_NAME: &str = "g010_crypto_blend";
pub const G010_TITLE: &str = "G010 Crypto Z-Score/Momentum Signal Blend";
pub const G114_NAME: &str = "g114_swing_scan";
pub const G114_TITLE: &str = "G114 Momentum/Swing Scanner (US universe)";
pub const G139_NAME: &str = "g139_vix_ts_pair";
pub const G139_TITLE: &str = "G139 VIX Term-Structure Pair-Spread Study";

/// pandas/polars ewm_mean(com, adjust=True): alpha = 1/(1+com); NaN inputs
/// skipped; emits after `min_samples` finite observations.
fn ewm_mean(x: &[f64], com: usize, min_samples: usize) -> Vec<f64> {
    let a = 1.0 / (1.0 + com as f64);
    let mut out = vec![f64::NAN; x.len()];
    let (mut m, mut k) = (0.0f64, 0usize);
    for (t, &v) in x.iter().enumerate() {
        if !v.is_finite() {
            continue;
        }
        m = if k == 0 { v } else { (1.0 - a) * m + a * v };
        k += 1;
        if k >= min_samples {
            out[t] = m;
        }
    }
    out
}

/// pandas ewm_std(com, adjust=True, ddof=1):
/// var = (Sw2 - Sw^2/S) / (S - Ssq/S), weights decayed by (1-alpha) per step.
fn ewm_std(x: &[f64], com: usize, min_samples: usize) -> Vec<f64> {
    let a = 1.0 / (1.0 + com as f64);
    let d = 1.0 - a;
    let mut out = vec![f64::NAN; x.len()];
    let (mut s, mut sq, mut sx, mut sx2, mut k) = (0.0f64, 0.0f64, 0.0f64, 0.0f64, 0usize);
    for (t, &v) in x.iter().enumerate() {
        if !v.is_finite() {
            continue;
        }
        s = d * s + 1.0;
        sq = d * d * sq + 1.0;
        sx = d * sx + v;
        sx2 = d * sx2 + v * v;
        k += 1;
        if k >= min_samples {
            let num = sx2 - sx * sx / s;
            let den = s - sq / s;
            if den > 0.0 && num >= 0.0 {
                out[t] = (num / den).sqrt();
            }
        }
    }
    out
}

fn pct_change(close: &[f64]) -> Vec<f64> {
    let n = close.len();
    let mut r = vec![f64::NAN; n];
    for t in 1..n {
        if close[t].is_finite() && close[t - 1].is_finite() && close[t - 1] != 0.0 {
            r[t] = close[t] / close[t - 1] - 1.0;
        }
    }
    r
}

fn log_returns(close: &[f64]) -> Vec<f64> {
    let n = close.len();
    let mut r = vec![f64::NAN; n];
    for t in 1..n {
        if close[t].is_finite() && close[t - 1].is_finite() && close[t - 1] > 0.0 {
            r[t] = (close[t] / close[t - 1]).ln();
        }
    }
    r
}

/// tinycta.util.vol_adj: log-return / ewm_std(com=vola-1) clipped to +-clip.
fn vol_adj(close: &[f64], vola: usize, clip: f64, min_samples: usize) -> Vec<f64> {
    let r = log_returns(close);
    let vol = ewm_std(&r, vola.saturating_sub(1), min_samples);
    let mut out = vec![f64::NAN; close.len()];
    for t in 0..close.len() {
        if r[t].is_finite() && vol[t].is_finite() && vol[t] > 0.0 {
            out[t] = (r[t] / vol[t]).clamp(-clip, clip);
        }
    }
    out
}

fn cumsum_forward(x: &[f64]) -> Vec<f64> {
    let mut out = vec![f64::NAN; x.len()];
    let mut acc = 0.0;
    for (t, &v) in x.iter().enumerate() {
        if v.is_finite() {
            acc += v;
            out[t] = acc;
        }
    }
    out
}

/// tinycta.osc.osc: analytic-scale EWM-difference oscillator.
fn osc(x: &[f64], fast: usize, slow: usize) -> Vec<f64> {
    let f = 1.0 - 1.0 / fast as f64;
    let g = 1.0 - 1.0 / slow as f64;
    let s = (1.0 / (1.0 - f * f) - 2.0 / (1.0 - f * g) + 1.0 / (1.0 - g * g)).sqrt();
    let ef = ewm_mean(x, fast - 1, 1);
    let es = ewm_mean(x, slow - 1, 1);
    let mut out = vec![f64::NAN; x.len()];
    for t in 0..x.len() {
        if ef[t].is_finite() && es[t].is_finite() {
            out[t] = (ef[t] - es[t]) / s;
        }
    }
    out
}

fn finish_weights(panel: &Panel, mut result: Signals, weights: Vec<Vec<f64>>) -> Signals {
    for (j, w) in weights.iter().enumerate() {
        for (t, &v) in w.iter().enumerate() {
            if v.is_finite() && v != 0.0 {
                result.signal[j][t] = true;
            }
        }
    }
    result.trade_weights = Some(weights);
    result.benchmark_returns = benchmark_returns(panel, "SPY");
    result
}

/// G077 CTA 1.0: sign(EWM com32 - EWM com96, min 100) x cash_scale daily.
pub fn compute_g077_signals(panel: &Panel, cfg: &Config) -> Result<Signals, String> {
    let fast = cfg
        .ewma_fast
        .trim()
        .parse::<usize>()
        .map_err(|_| "G077 needs integer ewma_fast (com)".to_string())?;
    let slow = cfg
        .ewma_slow
        .trim()
        .parse::<usize>()
        .map_err(|_| "G077 needs integer ewma_slow (com)".to_string())?;
    if fast >= slow || cfg.cash_scale <= 0.0 {
        return Err("G077 requires ewma_fast < ewma_slow and positive cash_scale".into());
    }
    let ns = panel.symbols.len();
    let mut weights = vec![vec![0.0; panel.dates.len()]; ns];
    let mut result = blank_rebalance_signals(panel, "cta1_binary");
    for j in 0..ns {
        let ef = ewm_mean(&panel.close[j], fast, cfg.min_history);
        let es = ewm_mean(&panel.close[j], slow, cfg.min_history);
        for t in 0..panel.dates.len() {
            if ef[t].is_finite() && es[t].is_finite() {
                let s = (ef[t] - es[t]).signum();
                result.rsi[j][t] = ef[t] - es[t];
                weights[j][t] = s * cfg.cash_scale;
            }
        }
    }
    Ok(finish_weights(panel, result, weights))
}

/// G078 CTA 2.0: sign(EWM diff) / ewm_std(pct_change, com=vola) x cash_scale.
pub fn compute_g078_signals(panel: &Panel, cfg: &Config) -> Result<Signals, String> {
    let fast = cfg
        .ewma_fast
        .trim()
        .parse::<usize>()
        .map_err(|_| "G078 needs integer ewma_fast".to_string())?;
    let slow = cfg
        .ewma_slow
        .trim()
        .parse::<usize>()
        .map_err(|_| "G078 needs integer ewma_slow".to_string())?;
    if fast >= slow || cfg.vol_window == 0 || cfg.cash_scale <= 0.0 {
        return Err("G078 requires ewma_fast < ewma_slow, vol_window, cash_scale".into());
    }
    let ns = panel.symbols.len();
    let mut weights = vec![vec![0.0; panel.dates.len()]; ns];
    let mut result = blank_rebalance_signals(panel, "cta2_volscaled");
    for j in 0..ns {
        let ef = ewm_mean(&panel.close[j], fast, cfg.min_history);
        let es = ewm_mean(&panel.close[j], slow, cfg.min_history);
        let vol = ewm_std(&pct_change(&panel.close[j]), cfg.vol_window, cfg.min_history);
        for t in 0..panel.dates.len() {
            if ef[t].is_finite() && es[t].is_finite() && vol[t].is_finite() && vol[t] > 0.0 {
                let s = (ef[t] - es[t]).signum();
                result.rsi[j][t] = s / vol[t];
                weights[j][t] = s / vol[t] * cfg.cash_scale;
            }
        }
    }
    Ok(finish_weights(panel, result, weights))
}

/// G079 CTA 3.0: tanh(osc(cumsum(vol_adj))) / ewm_std(com=slow) x cash_scale.
/// Catalog clip=3 (function signature default; notebook slider shows 4.2 - DELTA noted).
pub fn compute_g079_signals(panel: &Panel, cfg: &Config) -> Result<Signals, String> {
    let fast = cfg
        .ewma_fast
        .trim()
        .parse::<usize>()
        .map_err(|_| "G079 needs integer ewma_fast".to_string())?;
    let slow = cfg
        .ewma_slow
        .trim()
        .parse::<usize>()
        .map_err(|_| "G079 needs integer ewma_slow".to_string())?;
    if fast >= slow || cfg.vol_window == 0 || cfg.cash_scale <= 0.0 {
        return Err("G079 requires ewma_fast < ewma_slow, vol_window, cash_scale".into());
    }
    let ns = panel.symbols.len();
    let mut weights = vec![vec![0.0; panel.dates.len()]; ns];
    let mut result = blank_rebalance_signals(panel, "cta3_robust");
    for j in 0..ns {
        let adj = cumsum_forward(&vol_adj(
            &panel.close[j],
            cfg.vol_window,
            cfg.clip,
            cfg.min_history,
        ));
        let mu = osc(&adj, fast, slow);
        let vol = ewm_std(&pct_change(&panel.close[j]), slow, cfg.min_history);
        for t in 0..panel.dates.len() {
            if mu[t].is_finite() && vol[t].is_finite() && vol[t] > 0.0 {
                let m = mu[t].tanh();
                result.rsi[j][t] = m;
                weights[j][t] = m / vol[t] * cfg.cash_scale;
            }
        }
    }
    Ok(finish_weights(panel, result, weights))
}

/// Shared robust tanh oscillator for G079/G080/G081 (mu matrix).
fn cta_mu(panel: &Panel, cfg: &Config, fast: usize, slow: usize) -> Vec<Vec<f64>> {
    panel
        .close
        .iter()
        .map(|c| {
            let adj = cumsum_forward(&vol_adj(c, cfg.vol_window, cfg.clip, cfg.min_history));
            osc(&adj, fast, slow)
                .iter()
                .map(|v| if v.is_finite() { v.tanh() } else { f64::NAN })
                .collect()
        })
        .collect()
}

/// G080 CTA 4.0: risk_scaled = mu / ||mu||_2 per day; w = scale*risk_scaled/vo.
pub fn compute_g080_signals(panel: &Panel, cfg: &Config) -> Result<Signals, String> {
    let fast = cfg
        .ewma_fast
        .trim()
        .parse::<usize>()
        .map_err(|_| "G080 needs integer ewma_fast".to_string())?;
    let slow = cfg
        .ewma_slow
        .trim()
        .parse::<usize>()
        .map_err(|_| "G080 needs integer ewma_slow".to_string())?;
    if fast >= slow || cfg.vol_window == 0 || cfg.cash_scale <= 0.0 {
        return Err("G080 requires ewma_fast < ewma_slow, vol_window, cash_scale".into());
    }
    let n = panel.dates.len();
    let ns = panel.symbols.len();
    let mu = cta_mu(panel, cfg, fast, slow);
    let vo: Vec<Vec<f64>> = (0..ns)
        .map(|j| ewm_std(&pct_change(&panel.close[j]), cfg.vol_window, cfg.vol_window))
        .collect();
    let mut weights = vec![vec![0.0; n]; ns];
    let mut result = blank_rebalance_signals(panel, "cta4_riskparity");
    for j in 0..ns {
        result.rsi[j] = mu[j].clone();
    }
    for t in 0..n {
        let norm: f64 = (0..ns)
            .map(|j| if mu[j][t].is_finite() { mu[j][t] * mu[j][t] } else { 0.0 })
            .sum::<f64>()
            .sqrt();
        if norm == 0.0 || !norm.is_finite() {
            continue;
        }
        for j in 0..ns {
            if mu[j][t].is_finite() && vo[j][t].is_finite() && vo[j][t] > 0.0 {
                weights[j][t] = cfg.cash_scale * (mu[j][t] / norm) / vo[j][t];
            }
        }
    }
    Ok(finish_weights(panel, result, weights))
}

/// Solve A x = b by Gaussian elimination with partial pivoting; None if singular.
fn solve_linear(a: &[Vec<f64>], b: &[f64]) -> Option<Vec<f64>> {
    let n = b.len();
    let mut m: Vec<Vec<f64>> = (0..n)
        .map(|i| {
            let mut row = a[i].clone();
            row.push(b[i]);
            row
        })
        .collect();
    for col in 0..n {
        let pivot = (col..n)
            .max_by(|&i, &j| m[i][col].abs().partial_cmp(&m[j][col].abs()).unwrap())?;
        if m[pivot][col].abs() < 1e-12 {
            return None;
        }
        m.swap(col, pivot);
        for row in (col + 1)..n {
            let f = m[row][col] / m[col][col];
            for k in col..=n {
                m[row][k] -= f * m[col][k];
            }
        }
    }
    let mut x = vec![0.0; n];
    for i in (0..n).rev() {
        x[i] = (m[i][n] - (i + 1..n).map(|k| m[i][k] * x[k]).sum::<f64>()) / m[i][i];
    }
    Some(x)
}

/// G081 CTA 5.0: DCC (EWM corr of vol_adj returns, shrunk) risk-parity positions.
pub fn compute_g081_signals(panel: &Panel, cfg: &Config) -> Result<Signals, String> {
    let fast = cfg
        .ewma_fast
        .trim()
        .parse::<usize>()
        .map_err(|_| "G081 needs integer ewma_fast".to_string())?;
    let slow = cfg
        .ewma_slow
        .trim()
        .parse::<usize>()
        .map_err(|_| "G081 needs integer ewma_slow".to_string())?;
    if fast >= slow || cfg.vol_window == 0 || cfg.corr_window == 0 || cfg.cash_scale <= 0.0 {
        return Err("G081 requires ewma_fast < ewma_slow, vol_window, corr_window".into());
    }
    let n = panel.dates.len();
    let ns = panel.symbols.len();
    let mu = cta_mu(panel, cfg, fast, slow);
    let vo: Vec<Vec<f64>> = (0..ns)
        .map(|j| ewm_std(&pct_change(&panel.close[j]), cfg.vol_window, cfg.vol_window))
        .collect();
    let radj: Vec<Vec<f64>> = (0..ns)
        .map(|j| vol_adj(&panel.close[j], cfg.vol_window, cfg.clip, cfg.min_history))
        .collect();
    let em: Vec<Vec<f64>> = (0..ns)
        .map(|j| ewm_mean(&radj[j], cfg.corr_window, cfg.corr_window))
        .collect();
    let ep: Vec<Vec<Vec<f64>>> = (0..ns)
        .map(|i| {
            (0..ns)
                .map(|j| {
                    let prod: Vec<f64> = (0..n)
                        .map(|t| {
                            if radj[i][t].is_finite() && radj[j][t].is_finite() {
                                radj[i][t] * radj[j][t]
                            } else {
                                f64::NAN
                            }
                        })
                        .collect();
                    ewm_mean(&prod, cfg.corr_window, cfg.corr_window)
                })
                .collect()
        })
        .collect();
    let mut weights = vec![vec![0.0; n]; ns];
    let mut result = blank_rebalance_signals(panel, "cta5_dcc");
    for j in 0..ns {
        result.rsi[j] = mu[j].clone();
    }
    for t in 0..n {
        let mask: Vec<usize> = (0..ns).filter(|&j| panel.close[j][t].is_finite()).collect();
        if mask.is_empty() {
            continue;
        }
        let k = mask.len();
        let mut cor = vec![vec![f64::NAN; k]; k];
        for (a, &i) in mask.iter().enumerate() {
            for (b, &j) in mask.iter().enumerate() {
                let cov = ep[i][j][t] - em[i][t] * em[j][t];
                let vi = ep[i][i][t] - em[i][t] * em[i][t];
                let vj = ep[j][j][t] - em[j][t] * em[j][t];
                if a == b {
                    cor[a][b] = if vi > 0.0 { 1.0 } else { f64::NAN };
                } else if vi > 0.0 && vj > 0.0 && cov.is_finite() {
                    cor[a][b] = cov / (vi * vj).sqrt();
                }
            }
        }
        for a in 0..k {
            for b in 0..k {
                if !cor[a][b].is_finite() {
                    cor[a][b] = if a == b { 1.0 } else { 0.0 };
                }
            }
        }
        // shrink2id: C = C*lamb + (1-lamb)*I
        let lamb = cfg.shrinkage;
        for a in 0..k {
            for b in 0..k {
                cor[a][b] *= lamb;
            }
            cor[a][a] += 1.0 - lamb;
        }
        let muv: Vec<f64> = mask
            .iter()
            .map(|&j| if mu[j][t].is_finite() { mu[j][t] } else { 0.0 })
            .collect();
        let Some(x) = solve_linear(&cor, &muv) else {
            continue;
        };
        let norm: f64 = muv.iter().zip(&x).map(|(m, xv)| m * xv).sum::<f64>().sqrt();
        if norm <= 0.0 || !norm.is_finite() {
            continue;
        }
        for (a, &j) in mask.iter().enumerate() {
            let v = vo[j][t];
            if v.is_finite() && v > 0.0 {
                weights[j][t] = cfg.cash_scale * (x[a] / norm) / v;
            }
        }
    }
    Ok(finish_weights(panel, result, weights))
}

/// G085: local-extreme Donchian breakout sign x EWM-vol scaling to target_vol.
/// Private vivace signal -> DELTA: sign at lookback high/low, carried between.
pub fn compute_g085_signals(panel: &Panel, cfg: &Config) -> Result<Signals, String> {
    if cfg.lookback < 2 || cfg.vol_window == 0 {
        return Err("G085 requires lookback >= 2 and vol_window >= 1".into());
    }
    let n = panel.dates.len();
    let ns = panel.symbols.len();
    let days = rebalance_days(panel, &cfg.rebalance)?;
    let mut result = blank_rebalance_signals(panel, "local_extreme");
    let mut weights = vec![vec![0.0; n]; ns];
    for j in 0..ns {
        let vol = ewm_std(&pct_change(&panel.close[j]), cfg.vol_window, cfg.vol_window);
        let mut sign = 0.0f64;
        for t in 0..n {
            let c = panel.close[j][t];
            if !c.is_finite() {
                continue;
            }
            if t + 1 >= cfg.lookback {
                let lo = t + 1 - cfg.lookback;
                let (mut mx, mut mn) = (f64::NEG_INFINITY, f64::INFINITY);
                for u in lo..=t {
                    let v = panel.close[j][u];
                    if v.is_finite() {
                        mx = mx.max(v);
                        mn = mn.min(v);
                    }
                }
                if c >= mx && mx > mn {
                    sign = 1.0;
                } else if c <= mn && mx > mn {
                    sign = -1.0;
                }
            }
            result.rsi[j][t] = sign;
            let va = vol[t];
            if sign != 0.0 && va.is_finite() && va > 0.0 {
                let ann = va * (cfg.periods_per_year as f64).sqrt();
                if ann > 0.0 {
                    weights[j][t] = sign * cfg.pos_cap.min(cfg.target_vol / ann);
                }
            }
        }
    }
    // carry signal to month-end rebalances only: zero out non-rebalance days
    let on_day: Vec<bool> = {
        let mut marks = vec![false; n];
        for &d in &days {
            marks[d] = true;
        }
        marks
    };
    for j in 0..ns {
        for t in 0..n {
            if !on_day[t] {
                weights[j][t] = 0.0;
            }
        }
    }
    Ok(finish_weights(panel, result, weights))
}

/// Dominant eigenvector of a symmetric matrix via power iteration.
fn top_eigenvector(c: &[Vec<f64>], iters: usize) -> Vec<f64> {
    let n = c.len();
    let mut v: Vec<f64> = {
        let s = (n as f64).sqrt();
        vec![1.0 / s; n]
    };
    for _ in 0..iters {
        let mut w = vec![0.0; n];
        for i in 0..n {
            for j in 0..n {
                w[i] += c[i][j] * v[j];
            }
        }
        let norm: f64 = w.iter().map(|x| x * x).sum::<f64>().sqrt();
        if norm <= 0.0 || !norm.is_finite() {
            return v;
        }
        for x in &mut w {
            *x /= norm;
        }
        v = w;
    }
    // deterministic sign: first nonzero element positive
    if let Some(&first) = v.iter().find(|x| x.abs() > 1e-9) {
        if first < 0.0 {
            for x in &mut v {
                *x = -*x;
            }
        }
    }
    v
}

/// SSA_Signal for one trailing window of `m+1` closes, embedding dim `l`.
fn ssa_signal_one(window: &[f64], l: usize, m: usize) -> f64 {
    let nn = window.len();
    let (mut mean, mut var) = (0.0f64, 0.0f64);
    for &v in window {
        mean += v;
    }
    mean /= nn as f64;
    for &v in window {
        var += (v - mean) * (v - mean);
    }
    let sd = (var / (nn as f64 - 1.0)).sqrt();
    if sd <= 0.0 || !sd.is_finite() {
        return 0.0;
    }
    let x: Vec<f64> = window.iter().map(|v| (v - mean) / sd).collect();
    // unbiased xcorr lags 0..l-1 -> symmetric Toeplitz covariance
    let mut cov = vec![vec![0.0; l]; l];
    for lag in 0..l {
        let mut s = 0.0;
        for i in 0..nn - lag {
            s += x[i] * x[i + lag];
        }
        let c = s / (nn - lag) as f64;
        for i in 0..l {
            let j = i + lag;
            if j < l {
                cov[i][j] = c;
                cov[j][i] = c;
            }
        }
    }
    let rho = top_eigenvector(&cov, 200);
    let rows = m - l + 1;
    if rows < 2 {
        return 0.0;
    }
    let mut pc = vec![0.0; rows];
    for m_ in 0..rows {
        for l_ in 0..l {
            pc[m_] += x[m_ + l_] * rho[l_];
        }
    }
    let last = pc[rows - 1];
    if last == 0.0 || !last.is_finite() {
        return 0.0;
    }
    // (PC_end - PC_1)/PC_end, clipped to [-2,2] then rescaled to [-1,1]
    ((last - pc[0]) / last).clamp(-2.0, 2.0) / 2.0
}

/// Allocation days for the every-N-sessions schedule (G095): first day at
/// `lookback` (0-indexed) then every `rebalance_step`.
fn step_days(panel: &Panel, cfg: &Config) -> Vec<usize> {
    (cfg.lookback..panel.dates.len())
        .step_by(cfg.rebalance_step.max(1))
        .collect()
}

/// G095: SSA first-PC trend signal, vol-parity weights, vol-target leverage.
pub fn compute_g095_signals(panel: &Panel, cfg: &Config) -> Result<Signals, String> {
    let m = cfg.lookback;
    let l = cfg.latent_dim;
    if l < 2 || m < l + 5 || cfg.rebalance_step == 0 {
        return Err("G095 requires latent_dim >= 2, lookback > latent_dim+5, step".into());
    }
    let n = panel.dates.len();
    let ns = panel.symbols.len();
    let days = step_days(panel, cfg);
    let mut result = blank_rebalance_signals(panel, "ssa_trend");
    let mut weights = vec![vec![0.0; n]; ns];
    let mut prev_s = vec![f64::NAN; ns];
    let mut first_alloc = true;
    for &t in &days {
        let lo = t.saturating_sub(m);
        // trailing m+1 closes (inclusive window), plus returns for weighting
        let mut s = vec![f64::NAN; ns];
        let mut vols = vec![f64::NAN; ns];
        let mut rets: Vec<Vec<f64>> = vec![Vec::new(); ns];
        for j in 0..ns {
            let window = &panel.close[j][lo..=t];
            if window.iter().all(|v| v.is_finite()) {
                s[j] = ssa_signal_one(window, l, m);
            }
            let mut r = Vec::with_capacity(m);
            let mut ok = true;
            for u in lo + 1..=t {
                let (a, b) = (panel.close[j][u], panel.close[j][u - 1]);
                if a.is_finite() && b.is_finite() && b != 0.0 {
                    r.push(a / b - 1.0);
                } else {
                    ok = false;
                    break;
                }
            }
            if ok && r.len() >= m {
                let (mu, va) = {
                    let mean: f64 = r.iter().sum::<f64>() / r.len() as f64;
                    let var: f64 = r
                        .iter()
                        .map(|x| (x - mean) * (x - mean))
                        .sum::<f64>()
                        / (r.len() as f64 - 1.0);
                    (mean, var)
                };
                let _ = mu;
                if va > 0.0 {
                    vols[j] = va.sqrt();
                    rets[j] = r;
                }
            }
            result.rsi[j][t] = s[j];
        }
        let avail: Vec<usize> = (0..ns)
            .filter(|&j| s[j].is_finite() && vols[j].is_finite() && vols[j] > 0.0)
            .collect();
        if avail.is_empty() {
            continue;
        }
        // trading rules on signal quantity
        match cfg.strategy_mode.as_str() {
            "overQuantity" => {
                let qt: f64 = avail.iter().map(|&j| s[j].abs()).sum();
                let th = avail.len() as f64 * cfg.trend_threshold;
                if qt <= th {
                    if first_alloc {
                        for &j in &avail {
                            s[j] = 1.0;
                        }
                    } else {
                        for &j in &avail {
                            s[j] = if prev_s[j].is_finite() { prev_s[j] } else { 0.0 };
                        }
                    }
                }
            }
            "indQuantity" => {
                for &j in &avail {
                    if s[j].abs() <= cfg.trend_threshold {
                        s[j] = if first_alloc {
                            s[j].abs()
                        } else if prev_s[j].is_finite() {
                            prev_s[j]
                        } else {
                            0.0
                        };
                    }
                }
            }
            _ => {}
        }
        first_alloc = false;
        for &j in &avail {
            prev_s[j] = s[j];
        }
        // weighting scheme
        let mut w = vec![0.0; ns];
        match cfg.weighting_mode.as_str() {
            "EW" => {
                for &j in &avail {
                    w[j] = 1.0 / avail.len() as f64;
                }
            }
            _ => {
                let inv: f64 = avail.iter().map(|&j| 1.0 / vols[j]).sum();
                for &j in &avail {
                    w[j] = (1.0 / vols[j]) / inv;
                }
            }
        }
        // leverage to vol target: L = target / (sqrt(w' S w) * sqrt(252))
        let mut lev = 1.0;
        if cfg.target_vol > 0.0 {
            let mut pv = 0.0;
            for &a in &avail {
                for &b in &avail {
                    let wa = w[a] * s[a];
                    let wb = w[b] * s[b];
                    if wa == 0.0 || wb == 0.0 {
                        continue;
                    }
                    let (ra, rb) = (&rets[a], &rets[b]);
                    let (ma, mb) = (
                        ra.iter().sum::<f64>() / ra.len() as f64,
                        rb.iter().sum::<f64>() / rb.len() as f64,
                    );
                    let cov: f64 = ra
                        .iter()
                        .zip(rb.iter())
                        .map(|(x, y)| (x - ma) * (y - mb))
                        .sum::<f64>()
                        / (ra.len() as f64 - 1.0);
                    pv += wa * wb * cov;
                }
            }
            if pv > 0.0 {
                lev = cfg.target_vol / (pv.sqrt() * (cfg.periods_per_year as f64).sqrt());
            }
        }
        for &j in &avail {
            let v = w[j] * s[j] * lev;
            if v != 0.0 && v.is_finite() {
                weights[j][t] = v;
            }
        }
    }
    Ok(finish_weights(panel, result, weights))
}

/// Cross-sectional rank portfolio helper: top n_legs long, bottom n_legs short,
/// each side scaled to 0.5 gross; evaluated on `days`.
fn xs_rank_weights(
    panel: &Panel,
    scores: &[Vec<f64>],
    days: &[usize],
    n_legs: usize,
) -> Vec<Vec<f64>> {
    let ns = panel.symbols.len();
    let mut weights = vec![vec![0.0; panel.dates.len()]; ns];
    for &t in days {
        let mut ranked: Vec<(usize, f64)> = (0..ns)
            .filter(|&j| scores[j][t].is_finite())
            .map(|j| (j, scores[j][t]))
            .collect();
        if ranked.len() < 2 {
            continue;
        }
        ranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        let k = n_legs.min(ranked.len() / 2).max(1);
        let w = 0.5 / k as f64;
        for (i, &(j, _)) in ranked.iter().enumerate() {
            if i < k {
                weights[j][t] = w;
            } else if i >= ranked.len() - k {
                weights[j][t] = -w;
            }
        }
    }
    weights
}

/// G106: XS rank on 252d momentum, monthly, commodity ETF sleeve (DELTA).
pub fn compute_g106_signals(panel: &Panel, cfg: &Config) -> Result<Signals, String> {
    if cfg.mom_window == 0 || cfg.n_legs == 0 {
        return Err("G106 requires positive mom_window and n_legs".into());
    }
    let days = rebalance_days(panel, &cfg.rebalance)?;
    let scores = momentum_matrix(panel, cfg.mom_window, cfg.min_history);
    let mut result = blank_rebalance_signals(panel, "xs_mom252");
    result.rsi = scores.clone();
    let weights = xs_rank_weights(panel, &scores, &days, cfg.n_legs);
    Ok(finish_weights(panel, result, weights))
}

/// G109: XS rank on negated realized skewness of `mom_window` daily returns.
/// Commodity variant negates the score (Fernandez-Perez 2018) -> DELTA sleeve.
pub fn compute_g109_signals(panel: &Panel, cfg: &Config) -> Result<Signals, String> {
    if cfg.mom_window < 10 || cfg.n_legs == 0 {
        return Err("G109 requires mom_window >= 10 and n_legs".into());
    }
    let n = panel.dates.len();
    let ns = panel.symbols.len();
    let days = rebalance_days(panel, &cfg.rebalance)?;
    let mut result = blank_rebalance_signals(panel, "xs_skew");
    let mut scores = vec![vec![f64::NAN; n]; ns];
    for j in 0..ns {
        let r = pct_change(&panel.close[j]);
        let mut hist = 0usize;
        for t in 0..n {
            if r[t].is_finite() {
                hist += 1;
            }
            if t + 1 < cfg.mom_window {
                continue;
            }
            let lo = t + 1 - cfg.mom_window;
            let w: Vec<f64> = (lo..=t).map(|u| r[u]).filter(|v| v.is_finite()).collect();
            if w.len() < cfg.mom_window || hist < cfg.min_history {
                continue;
            }
            let mean: f64 = w.iter().sum::<f64>() / w.len() as f64;
            let (m2, m3) = w.iter().fold((0.0, 0.0), |(a2, a3), &x| {
                let d = x - mean;
                (a2 + d * d, a3 + d * d * d)
            });
            let m2 = m2 / w.len() as f64;
            let m3 = m3 / w.len() as f64;
            if m2 > 0.0 {
                scores[j][t] = -(m3 / m2.powf(1.5));
            }
        }
    }
    result.rsi = scores.clone();
    let weights = xs_rank_weights(panel, &scores, &days, cfg.n_legs);
    Ok(finish_weights(panel, result, weights))
}

/// G010: sign(mean(-zscore(close,W), rolling-sum pct_change W)) on BTC/ETH.
/// Long/flat baseline (catalog direction=long); `ls` mode allows signed.
pub fn compute_g010_signals(panel: &Panel, cfg: &Config) -> Result<Signals, String> {
    let w = cfg.z_window.max(5);
    let n = panel.dates.len();
    let ns = panel.symbols.len();
    let long_only = cfg.strategy_mode != "ls";
    let mut result = blank_rebalance_signals(panel, "crypto_blend");
    let mut weights = vec![vec![0.0; n]; ns];
    for j in 0..ns {
        let r = pct_change(&panel.close[j]);
        for t in 0..n {
            if t + 1 < w {
                continue;
            }
            let lo = t + 1 - w;
            let win: Vec<f64> = (lo..=t)
                .map(|u| panel.close[j][u])
                .filter(|v| v.is_finite())
                .collect();
            if win.len() < w {
                continue;
            }
            let mean: f64 = win.iter().sum::<f64>() / w as f64;
            let var: f64 = win.iter().map(|x| (x - mean) * (x - mean)).sum::<f64>()
                / (w as f64 - 1.0);
            if var <= 0.0 {
                continue;
            }
            let z = (panel.close[j][t] - mean) / var.sqrt();
            let mom: f64 = (lo..=t).map(|u| r[u]).filter(|v| v.is_finite()).sum();
            let combined = (-z + mom) / 2.0;
            result.rsi[j][t] = combined;
            let pos = combined.signum();
            if long_only && pos <= 0.0 {
                continue;
            }
            weights[j][t] = pos / ns as f64;
        }
    }
    Ok(finish_weights(panel, result, weights))
}

/// ---------------------------------------------------------------------
/// Custom trade builders (non-rebalance strategies).
/// ---------------------------------------------------------------------

fn symbol_index(panel: &Panel, name: &str) -> Result<usize, String> {
    panel
        .symbols
        .iter()
        .position(|s| s == name)
        .ok_or_else(|| format!("{name} not in panel"))
}

/// Days where any trade weight is nonzero (for custom schedules like G095).
fn weights_days(sig: &Signals, n: usize) -> Vec<usize> {
    let mut marks = vec![false; n];
    if let Some(w) = &sig.trade_weights {
        for series in w {
            for (t, &v) in series.iter().enumerate() {
                if v.is_finite() && v != 0.0 {
                    marks[t] = true;
                }
            }
        }
    }
    (0..n).filter(|&t| marks[t]).collect()
}

/// Same loop as family4::weighted_rebalance_trades but on an explicit day list.
fn weighted_trades_on(
    panel: &Panel,
    sig: &Signals,
    cfg: &Config,
    fill: &str,
    days: &[usize],
) -> Result<Vec<Trade>, String> {
    let is_open = fill == "next_open";
    let weights = sig.trade_weights.as_ref().unwrap();
    let mut trades = Vec::new();
    for (period, &signal_day) in days.iter().enumerate() {
        let entry = signal_day + usize::from(is_open);
        if entry >= panel.dates.len() || (!is_open && entry + 1 == panel.dates.len()) {
            continue;
        }
        let boundary = days
            .get(period + 1)
            .copied()
            .unwrap_or(panel.dates.len() - 1);
        let exit = if is_open {
            boundary.saturating_add(1).min(panel.dates.len() - 1)
        } else {
            boundary
        };
        if exit < entry {
            continue;
        }
        for symbol in 0..panel.symbols.len() {
            let weight = weights[symbol][signal_day];
            if !weight.is_finite() || weight == 0.0 {
                continue;
            }
            let entry_px = if is_open {
                panel.open[symbol][entry]
            } else {
                panel.close[symbol][signal_day]
            };
            let exit_px = if is_open && panel.open[symbol][exit].is_finite() {
                panel.open[symbol][exit]
            } else {
                panel.close[symbol][exit]
            };
            if !entry_px.is_finite() || entry_px <= 0.0 || !exit_px.is_finite() {
                continue;
            }
            let prior_weight = period
                .checked_sub(1)
                .map(|prev| weights[symbol][days[prev]])
                .unwrap_or(0.0);
            let next_weight = days
                .get(period + 1)
                .filter(|&&nd| nd + 1 < panel.dates.len())
                .map(|&nd| weights[symbol][nd])
                .unwrap_or(0.0);
            let entry_turnover = (weight - prior_weight).abs();
            let exit_turnover = if next_weight == 0.0 { weight.abs() } else { 0.0 };
            let gross = exit_px / entry_px - 1.0;
            let cost = cfg.cost_for_symbol(&panel.symbols[symbol]);
            let direction = weight.signum();
            let borrow_cost = if weight < 0.0 && exit > entry {
                cfg.borrow_bps_annual / 10_000.0 / cfg.periods_per_year as f64
                    * (exit - entry + 1) as f64
            } else {
                0.0
            };
            trades.push(Trade {
                sym_idx: symbol,
                signal_idx: signal_day,
                entry_idx: entry,
                exit_idx: exit,
                entry_px,
                exit_px,
                ret: direction * gross
                    - cost * (entry_turnover + exit_turnover) / weight.abs()
                    - borrow_cost,
                ret_gross: direction * gross,
                exit_reason: if period + 1 == days.len() {
                    "censored"
                } else {
                    "rebalance"
                },
                hold: exit.saturating_sub(entry),
                signal_value: sig.rsi[symbol][signal_day],
                weight,
                entry_turnover,
                exit_turnover,
                weighted: true,
                borrow_cost_per_bar: if weight < 0.0 && exit > entry {
                    cfg.borrow_bps_annual / 10_000.0 / cfg.periods_per_year as f64
                } else {
                    0.0
                },
                dir_sign: direction,
            });
        }
    }
    trades.sort_by(|a, b| {
        (a.entry_idx, &panel.symbols[a.sym_idx]).cmp(&(b.entry_idx, &panel.symbols[b.sym_idx]))
    });
    Ok(trades)
}

pub fn g095_trades(
    panel: &Panel,
    sig: &Signals,
    cfg: &Config,
    fill: &str,
) -> Result<Vec<Trade>, String> {
    if !matches!(fill, "close" | "next_open") {
        return Err(format!("G095 does not support fill {fill:?}"));
    }
    let days = weights_days(sig, panel.dates.len());
    weighted_trades_on(panel, sig, cfg, fill, &days)
}

/// G063: TLT/IEF log-spread z/t-score tranche entries (DELTA for the futures
/// basket; roll date proxied by max_hold, tick by rolling mean |ds|).
pub fn compute_g063_signals(panel: &Panel, cfg: &Config) -> Result<Signals, String> {
    if cfg.z_window < 10 {
        return Err("G063 requires z_window >= 10".into());
    }
    let (a, b) = pair_indices(panel, cfg)?;
    let n = panel.dates.len();
    let mut result = blank_rebalance_signals(panel, "tsy_zt_spread");
    let mut s = vec![f64::NAN; n];
    for t in 0..n {
        let (ca, cb) = (panel.close[a][t], panel.close[b][t]);
        if ca.is_finite() && cb.is_finite() && ca > 0.0 && cb > 0.0 {
            s[t] = ca.ln() - cfg.spread_weight * cb.ln();
        }
    }
    let mut z = vec![f64::NAN; n];
    let mut tick = vec![f64::NAN; n];
    let mut tscore = vec![f64::NAN; n];
    for t in 0..n {
        if t + 1 < cfg.z_window {
            continue;
        }
        let lo = t + 1 - cfg.z_window;
        let w: Vec<f64> = (lo..=t).map(|u| s[u]).filter(|v| v.is_finite()).collect();
        if w.len() < cfg.z_window {
            continue;
        }
        let mean: f64 = w.iter().sum::<f64>() / w.len() as f64;
        let var: f64 =
            w.iter().map(|x| (x - mean) * (x - mean)).sum::<f64>() / (w.len() as f64 - 1.0);
        let mut md = 0.0;
        let mut mk = 0usize;
        for u in lo + 1..=t {
            if s[u].is_finite() && s[u - 1].is_finite() {
                md += (s[u] - s[u - 1]).abs();
                mk += 1;
            }
        }
        if var > 0.0 && s[t].is_finite() {
            z[t] = (s[t] - mean) / var.sqrt();
        }
        if mk > 0 && s[t].is_finite() {
            tick[t] = md / mk as f64;
            if tick[t] > 0.0 {
                tscore[t] = (s[t] - mean) / tick[t];
            }
        }
    }
    result.rsi[a] = z.clone();
    result.rsi[b] = z.clone();
    for t in 0..n {
        if z[t].is_finite()
            && tscore[t].is_finite()
            && z[t] <= -cfg.z_entry.abs()
            && tscore[t] <= -cfg.min_t_stat.abs()
        {
            result.signal[a][t] = true;
            result.signal[b][t] = true;
        }
    }
    Ok(result)
}

fn pair_indices(panel: &Panel, cfg: &Config) -> Result<(usize, usize), String> {
    let parts = crate::config::split_csv(&cfg.pairs);
    if parts.len() < 2 {
        return Err("requires `pairs` config (e.g. pairs: TLT,IEF)".into());
    }
    Ok((symbol_index(panel, &parts[0])?, symbol_index(panel, &parts[1])?))
}

pub fn g063_trades(
    panel: &Panel,
    _sig: &Signals,
    cfg: &Config,
    fill: &str,
) -> Result<Vec<Trade>, String> {
    if !matches!(fill, "close" | "next_open") {
        return Err(format!("G063 does not support fill {fill:?}"));
    }
    let (a, b) = pair_indices(panel, cfg)?;
    let n = panel.dates.len();
    let is_open = fill == "next_open";
    let mut s = vec![f64::NAN; n];
    for t in 0..n {
        let (ca, cb) = (panel.close[a][t], panel.close[b][t]);
        if ca.is_finite() && cb.is_finite() && ca > 0.0 && cb > 0.0 {
            s[t] = ca.ln() - cfg.spread_weight * cb.ln();
        }
    }
    let px = |j: usize, t: usize| -> f64 {
        if is_open {
            panel.open[j][t]
        } else {
            panel.close[j][t]
        }
    };
    let borrow_per_bar = cfg.borrow_bps_annual / 10_000.0 / cfg.periods_per_year as f64;
    let max_tranches = cfg.max_units.max(1);
    let up_ticks = cfg.take_profit.max(0.0);
    let dn_ticks = cfg.stop_loss.max(0.0);
    let mut trades = Vec::new();
    let mut open: Vec<(usize, usize, f64, f64)> = Vec::new(); // (sig_day, entry_idx, s0, tick0)
    for t in 0..n {
        // exits
        let mut still = Vec::new();
        for &(sig_day, entry_idx, s0, tick0) in &open {
            let age = t.saturating_sub(entry_idx);
            let mut reason = "";
            if s[t].is_finite() && tick0 > 0.0 {
                if s[t] - s0 >= up_ticks * tick0 {
                    reason = "target";
                } else if s[t] - s0 <= -dn_ticks * tick0 {
                    reason = "stop";
                }
            }
            if reason.is_empty() && cfg.max_hold > 0 && age >= cfg.max_hold {
                reason = "roll";
            }
            if !reason.is_empty() && t > entry_idx {
                push_pair_legs(
                    panel, cfg, a, b, sig_day, entry_idx, t, px(a, entry_idx),
                    px(b, entry_idx), px(a, t), px(b, t), reason, borrow_per_bar,
                    cfg.spread_weight, &mut trades,
                );
            } else {
                still.push((sig_day, entry_idx, s0, tick0));
            }
        }
        open = still;
        // entry check
        if open.len() >= max_tranches || !s[t].is_finite() {
            continue;
        }
        if t + 1 < cfg.z_window {
            continue;
        }
        let lo = t + 1 - cfg.z_window;
        let w: Vec<f64> = (lo..=t).map(|u| s[u]).filter(|v| v.is_finite()).collect();
        if w.len() < cfg.z_window {
            continue;
        }
        let mean: f64 = w.iter().sum::<f64>() / w.len() as f64;
        let var: f64 =
            w.iter().map(|x| (x - mean) * (x - mean)).sum::<f64>() / (w.len() as f64 - 1.0);
        let (mut md, mut mk) = (0.0, 0usize);
        for u in lo + 1..=t {
            if s[u].is_finite() && s[u - 1].is_finite() {
                md += (s[u] - s[u - 1]).abs();
                mk += 1;
            }
        }
        if var <= 0.0 || mk == 0 {
            continue;
        }
        let z = (s[t] - mean) / var.sqrt();
        let tick = md / mk as f64;
        let tscore = if tick > 0.0 { (s[t] - mean) / tick } else { 0.0 };
        if z <= -cfg.z_entry.abs() && tscore <= -cfg.min_t_stat.abs() {
            let entry_idx = t + usize::from(is_open);
            if entry_idx < n && px(a, entry_idx).is_finite() && px(b, entry_idx).is_finite() {
                open.push((t, entry_idx, s[t], tick));
            }
        }
    }
    // close leftovers at end
    let last = n - 1;
    for (sig_day, entry_idx, s0, _tick0) in open {
        let _ = s0;
        push_pair_legs(
            panel, cfg, a, b, sig_day, entry_idx, last, px(a, entry_idx), px(b, entry_idx),
            px(a, last), px(b, last), "censored", borrow_per_bar, cfg.spread_weight, &mut trades,
        );
    }
    trades.sort_by(|x, y| {
        (x.entry_idx, &panel.symbols[x.sym_idx]).cmp(&(y.entry_idx, &panel.symbols[y.sym_idx]))
    });
    Ok(trades)
}

/// Emit the two legs of a pair trade: long leg A, short leg B (w-scaled).
fn push_pair_legs(
    panel: &Panel,
    cfg: &Config,
    a: usize,
    b: usize,
    sig_day: usize,
    entry_idx: usize,
    exit_idx: usize,
    e_a: f64,
    e_b: f64,
    x_a: f64,
    x_b: f64,
    reason: &'static str,
    borrow_per_bar: f64,
    spread_weight: f64,
    trades: &mut Vec<Trade>,
) {
    let gross_total = 1.0 + spread_weight;
    for (j, w, e_px, x_px) in [
        (a, 1.0 / gross_total, e_a, x_a),
        (b, -spread_weight / gross_total, e_b, x_b),
    ] {
        if !e_px.is_finite() || !x_px.is_finite() || e_px <= 0.0 || x_px <= 0.0 {
            continue;
        }
        let gross = w.signum() * (x_px / e_px - 1.0);
        let cost = cfg.cost_for_symbol(&panel.symbols[j]);
        let borrow = if w < 0.0 {
            borrow_per_bar * (exit_idx - entry_idx + 1) as f64
        } else {
            0.0
        };
        trades.push(Trade {
            sym_idx: j,
            signal_idx: sig_day,
            entry_idx,
            exit_idx,
            entry_px: e_px,
            exit_px: x_px,
            ret: gross - cost * 2.0 * w.abs() - borrow,
            ret_gross: gross,
            exit_reason: reason,
            hold: exit_idx - entry_idx,
            signal_value: 1.0,
            weight: w,
            entry_turnover: w.abs(),
            exit_turnover: w.abs(),
            weighted: true,
            borrow_cost_per_bar: if w < 0.0 { borrow_per_bar } else { 0.0 },
            dir_sign: w.signum(),
        });
    }
}

/// G114: momentum/swing screen -> buy above signal-day high, SL below
/// min(close, prev close), target = risk_reward x risk. US-universe DELTA.
pub fn compute_g114_signals(panel: &Panel, cfg: &Config) -> Result<Signals, String> {
    if cfg.lookback < 20 || cfg.sma_window == 0 {
        return Err("G114 requires lookback >= 20 and sma_window".into());
    }
    let n = panel.dates.len();
    let ns = panel.symbols.len();
    let mut result = blank_rebalance_signals(panel, "swing_scan");
    let rsis: Vec<Vec<f64>> = if cfg.rsi_threshold > 0.0 {
        panel
            .close
            .iter()
            .map(|c| {
                crate::signals::wilder_rsi(c, cfg.rsi_len)
                    .unwrap_or_else(|_| vec![f64::NAN; n])
            })
            .collect()
    } else {
        vec![vec![f64::NAN; n]; ns]
    };
    for j in 0..ns {
        let sma = crate::signals::rolling_mean(&panel.close[j], cfg.sma_window);
        for t in 0..n {
            let c = panel.close[j][t];
            if !c.is_finite() || t + 1 < cfg.lookback {
                continue;
            }
            let lo = t + 1 - cfg.lookback;
            let mut hi = f64::NEG_INFINITY;
            for u in lo..=t {
                let v = panel.close[j][u];
                if v.is_finite() {
                    hi = hi.max(v);
                }
            }
            if !hi.is_finite() || hi <= 0.0 {
                continue;
            }
            result.rsi[j][t] = c / hi;
            let near_high = c >= (1.0 - cfg.screen_pct) * hi;
            let above_ma = sma[t].is_finite() && c > sma[t];
            let consol_ok = if cfg.consol_pct > 0.0 && cfg.consol_window > 0 {
                if t + 1 < cfg.consol_window {
                    false
                } else {
                    let lo2 = t + 1 - cfg.consol_window;
                    let (mut hh, mut ll) = (f64::NEG_INFINITY, f64::INFINITY);
                    for u in lo2..=t {
                        let (h, l) = (panel.high[j][u], panel.low[j][u]);
                        if h.is_finite() {
                            hh = hh.max(h);
                        }
                        if l.is_finite() {
                            ll = ll.min(l);
                        }
                    }
                    hh.is_finite() && ll.is_finite() && (hh - ll) / c <= cfg.consol_pct
                }
            } else {
                true
            };
            let rsi_ok = cfg.rsi_threshold <= 0.0
                || (rsis[j][t].is_finite() && rsis[j][t] >= cfg.rsi_threshold);
            if near_high && above_ma && consol_ok && rsi_ok {
                result.signal[j][t] = true;
            }
        }
    }
    Ok(result)
}

pub fn g114_trades(
    panel: &Panel,
    sig: &Signals,
    cfg: &Config,
    fill: &str,
) -> Result<Vec<Trade>, String> {
    if !matches!(fill, "close" | "next_open") {
        return Err(format!("G114 does not support fill {fill:?}"));
    }
    let n = panel.dates.len();
    let max_hold = cfg.gap_max_hold.max(1);
    let mut trades = Vec::new();
    for j in 0..panel.symbols.len() {
        let cost = cfg.cost_for_symbol(&panel.symbols[j]);
        let mut available_after = 0usize;
        for t in 0..n {
            if t < available_after || !sig.signal[j][t] || t + 1 >= n {
                continue;
            }
            let trigger = panel.high[j][t];
            let (c_t, c_prev) = (panel.close[j][t], panel.close[j][t.saturating_sub(1)]);
            if !trigger.is_finite() || !c_t.is_finite() || !c_prev.is_finite() {
                continue;
            }
            let entry = t + 1;
            let entry_px = if fill == "next_open" {
                let o = panel.open[j][entry];
                if o.is_finite() && o > trigger {
                    o
                } else if panel.high[j][entry].is_finite() && panel.high[j][entry] > trigger {
                    trigger
                } else {
                    continue;
                }
            } else {
                // close fill: only if the day traded above the trigger
                if panel.high[j][entry].is_finite() && panel.high[j][entry] > trigger {
                    trigger
                } else {
                    continue;
                }
            };
            let stop = c_t.min(c_prev);
            let risk = entry_px - stop;
            if risk <= 0.0 {
                continue;
            }
            let target = entry_px + cfg.risk_reward * risk;
            let final_session = (entry + max_hold - 1).min(n - 1);
            let mut exit_idx = final_session;
            let mut exit_px = panel.close[j][final_session];
            let mut reason = "time";
            for day in entry..=final_session {
                let (o, h, l) = (
                    panel.open[j][day],
                    panel.high[j][day],
                    panel.low[j][day],
                );
                if !o.is_finite() || !h.is_finite() || !l.is_finite() {
                    continue;
                }
                if day > entry || fill == "next_open" {
                    if o <= stop || l <= stop {
                        exit_idx = day;
                        exit_px = o.min(stop);
                        reason = "stop";
                        break;
                    }
                    if o >= target || h >= target {
                        exit_idx = day;
                        exit_px = o.max(target);
                        reason = "target";
                        break;
                    }
                }
            }
            if exit_idx < entry || !exit_px.is_finite() {
                continue;
            }
            let net = exit_px * (1.0 - cost) / (entry_px * (1.0 + cost)) - 1.0;
            trades.push(Trade {
                sym_idx: j,
                signal_idx: t,
                entry_idx: entry,
                exit_idx,
                entry_px,
                exit_px,
                ret: net,
                ret_gross: exit_px / entry_px - 1.0,
                exit_reason: reason,
                hold: exit_idx - entry + 1,
                signal_value: sig.rsi[j][t],
                weight: 1.0,
                entry_turnover: 1.0,
                exit_turnover: 1.0,
                weighted: false,
                borrow_cost_per_bar: 0.0,
                dir_sign: 1.0,
            });
            available_after = exit_idx + 1;
        }
    }
    trades.sort_by(|a, b| {
        (a.entry_idx, &panel.symbols[a.sym_idx]).cmp(&(b.entry_idx, &panel.symbols[b.sym_idx]))
    });
    Ok(trades)
}

/// G139: VIX/VIX3M backwardation-conditioned SPY/TLT spread trade (partial
/// DELTA: spot VIX history instead of futures term structure).
pub fn compute_g139_signals(panel: &Panel, cfg: &Config) -> Result<Signals, String> {
    if cfg.z_window < 10 || cfg.pairs.trim().is_empty() {
        return Err("G139 requires z_window >= 10 and pairs".into());
    }
    let (a, b) = pair_indices(panel, cfg)?;
    let n = panel.dates.len();
    let ratio = vix_ratio_series(panel, cfg)?;
    let mut result = blank_rebalance_signals(panel, "vix_ts_pair");
    let mut s = vec![f64::NAN; n];
    for t in 0..n {
        let (ca, cb) = (panel.close[a][t], panel.close[b][t]);
        if ca.is_finite() && cb.is_finite() && ca > 0.0 && cb > 0.0 {
            s[t] = ca.ln() - cfg.spread_weight * cb.ln();
        }
    }
    for t in 0..n {
        if t + 1 < cfg.z_window || !s[t].is_finite() {
            continue;
        }
        let lo = t + 1 - cfg.z_window;
        let w: Vec<f64> = (lo..=t).map(|u| s[u]).filter(|v| v.is_finite()).collect();
        if w.len() < cfg.z_window {
            continue;
        }
        let mean: f64 = w.iter().sum::<f64>() / w.len() as f64;
        let var: f64 =
            w.iter().map(|x| (x - mean) * (x - mean)).sum::<f64>() / (w.len() as f64 - 1.0);
        if var <= 0.0 {
            continue;
        }
        let z = (s[t] - mean) / var.sqrt();
        result.rsi[a][t] = z;
        result.rsi[b][t] = z;
        if ratio[t].is_finite()
            && ratio[t] >= cfg.vix_ratio_threshold
            && z <= -cfg.z_entry.abs()
        {
            result.signal[a][t] = true;
            result.signal[b][t] = true;
        }
    }
    Ok(result)
}

fn vix_ratio_series(panel: &Panel, cfg: &Config) -> Result<Vec<f64>, String> {
    let vix = crate::family7::read_vol_series(&cfg.vix_file, "DATE", "CLOSE")?;
    let vix3m = crate::family7::read_vol_series(&cfg.vix3m_file, "DATE", "CLOSE")?;
    let mut out = vec![f64::NAN; panel.dates.len()];
    for (t, d) in panel.dates.iter().enumerate() {
        let v = vix.range(..=*d).next_back().map(|(_, x)| *x);
        let m = vix3m.range(..=*d).next_back().map(|(_, x)| *x);
        if let (Some(v), Some(m)) = (v, m) {
            if v.is_finite() && m.is_finite() && m != 0.0 {
                out[t] = v / m;
            }
        }
    }
    Ok(out)
}

pub fn g139_trades(
    panel: &Panel,
    sig: &Signals,
    cfg: &Config,
    fill: &str,
) -> Result<Vec<Trade>, String> {
    if !matches!(fill, "close" | "next_open") {
        return Err(format!("G139 does not support fill {fill:?}"));
    }
    let (a, b) = pair_indices(panel, cfg)?;
    let n = panel.dates.len();
    let is_open = fill == "next_open";
    let ratio = vix_ratio_series(panel, cfg)?;
    let px = |j: usize, t: usize| -> f64 {
        if is_open {
            panel.open[j][t]
        } else {
            panel.close[j][t]
        }
    };
    let borrow_per_bar = cfg.borrow_bps_annual / 10_000.0 / cfg.periods_per_year as f64;
    let mut trades = Vec::new();
    let mut open_pos: Option<(usize, usize)> = None; // (sig_day, entry_idx)
    for t in 0..n {
        if let Some((sig_day, entry_idx)) = open_pos {
            let z = sig.rsi[a][t];
            let age = t.saturating_sub(entry_idx);
            let done = (z.is_finite() && z >= cfg.exit_z)
                || (ratio[t].is_finite() && ratio[t] < cfg.vix_ratio_threshold)
                || (cfg.max_hold > 0 && age >= cfg.max_hold)
                || t + 1 == n;
            if done && t > entry_idx {
                push_pair_legs(
                    panel, cfg, a, b, sig_day, entry_idx, t, px(a, entry_idx),
                    px(b, entry_idx), px(a, t), px(b, t),
                    if t + 1 == n { "censored" } else { "signal" },
                    borrow_per_bar, cfg.spread_weight, &mut trades,
                );
                open_pos = None;
            }
        }
        if open_pos.is_none() && sig.signal[a][t] {
            let entry_idx = t + usize::from(is_open);
            if entry_idx < n && px(a, entry_idx).is_finite() && px(b, entry_idx).is_finite() {
                open_pos = Some((t, entry_idx));
            }
        }
    }
    trades.sort_by(|x, y| {
        (x.entry_idx, &panel.symbols[x.sym_idx]).cmp(&(y.entry_idx, &panel.symbols[y.sym_idx]))
    });
    Ok(trades)
}
