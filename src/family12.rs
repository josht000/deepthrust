//! Family 12: GitHub-catalog strategies (Tier 1 + Tier 2 batch).

use chrono::Datelike;

use crate::config::{split_csv, Config};
use crate::data::Panel;
use crate::engine::Trade;
use crate::family4::blank_signals;
use crate::signals::Signals;

fn month_edge(panel: &Panel, t: usize) -> bool {
    let n = panel.dates.len();
    t + 1 == n
        || panel.dates[t].month() != panel.dates[t + 1].month()
        || panel.dates[t].year() != panel.dates[t + 1].year()
}

fn log_series(close: &[f64]) -> Vec<f64> {
    close
        .iter()
        .map(|v| {
            if v.is_finite() && *v > 0.0 {
                v.ln()
            } else {
                f64::NAN
            }
        })
        .collect()
}

fn window(log: &[f64], t: usize, len: usize) -> Option<Vec<f64>> {
    if t + 1 < len {
        return None;
    }
    let w: Vec<f64> = log[t + 1 - len..=t].to_vec();
    if w.iter().any(|v| !v.is_finite()) {
        None
    } else {
        Some(w)
    }
}

fn pearson(x: &[f64], y: &[f64]) -> f64 {
    let n = x.len() as f64;
    let mx = x.iter().sum::<f64>() / n;
    let my = y.iter().sum::<f64>() / n;
    let (mut sxy, mut sxx, mut syy) = (0.0, 0.0, 0.0);
    for i in 0..x.len() {
        sxy += (x[i] - mx) * (y[i] - my);
        sxx += (x[i] - mx).powi(2);
        syy += (y[i] - my).powi(2);
    }
    if sxx <= 0.0 || syy <= 0.0 {
        f64::NAN
    } else {
        sxy / (sxx * syy).sqrt()
    }
}

/// OU half-life from regressing spread diffs on lagged spread (slope, not the
/// intercept bug in the source repo). None when the slope is not negative.
fn ou_half_life(spread: &[f64]) -> Option<f64> {
    let m = spread.len() - 1;
    if m < 20 {
        return None;
    }
    let x: Vec<Vec<f64>> = vec![
        vec![1.0; m],
        (0..m).map(|i| spread[i]).collect(),
    ];
    let y: Vec<f64> = (0..m).map(|i| spread[i + 1] - spread[i]).collect();
    let beta = crate::family7::ols(&x, &y)?;
    let lambda = beta[1];
    if lambda >= 0.0 {
        return None;
    }
    Some(-std::f64::consts::LN_2 / lambda)
}

// ---------------- G023 screened pair reversion, Bollinger-distance entry ----

struct PairScreen {
    adf_t: f64,
}

/// corr + ADF(1%) + Hurst<0.5 + half-life<30d screen over the formation window.
fn g023_screen(
    log_a: &[f64],
    log_b: &[f64],
    t: usize,
    cfg: &Config,
    corr_thr: f64,
) -> Option<PairScreen> {
    let wa = window(log_a, t, cfg.formation_window)?;
    let wb = window(log_b, t, cfg.formation_window)?;
    if pearson(&wa, &wb) <= corr_thr {
        return None;
    }
    let (adf_t, hedge) = crate::family7::pair_adf(&wa, &wb, cfg.adf_lags)?;
    if adf_t >= crate::family7::adf_critical(cfg.adf_max_p) {
        return None;
    }
    let spread: Vec<f64> = wa
        .iter()
        .zip(&wb)
        .map(|(a, b)| a - hedge * b)
        .collect();
    let diffs: Vec<f64> = (1..spread.len()).map(|i| spread[i] - spread[i - 1]).collect();
    let h = crate::family7::hurst_exponent(&diffs, cfg.hurst_max_horizon);
    if !(h.is_finite() && h < cfg.hurst_lt) {
        return None;
    }
    let hl = ou_half_life(&spread)?;
    if hl >= cfg.max_half_life {
        return None;
    }
    Some(PairScreen { adf_t })
}

/// Rebased-normalized leg value and combined-mid stats over `bb_len`.
fn g023_bands(
    close_a: &[f64],
    close_b: &[f64],
    t: usize,
    cfg: &Config,
) -> Option<(f64, f64, f64, f64)> {
    if t + 1 < cfg.bb_len {
        return None;
    }
    let base_a = close_a[t + 1 - cfg.bb_len];
    let base_b = close_b[t + 1 - cfg.bb_len];
    if !base_a.is_finite() || !base_b.is_finite() || base_a <= 0.0 || base_b <= 0.0 {
        return None;
    }
    // legs rebased to 100 at window start so `pair_dist` reads as "points"
    let n_a = 100.0 * close_a[t] / base_a;
    let n_b = 100.0 * close_b[t] / base_b;
    if !n_a.is_finite() || !n_b.is_finite() {
        return None;
    }
    let (mut sum, mut sum2) = (0.0, 0.0);
    for u in t + 1 - cfg.bb_len..=t {
        let (a, b) = (close_a[u], close_b[u]);
        if !a.is_finite() || !b.is_finite() {
            return None;
        }
        let c = (a / base_a + b / base_b) * 50.0;
        sum += c;
        sum2 += c * c;
    }
    let n = cfg.bb_len as f64;
    let mean = sum / n;
    let sd = (sum2 / n - mean * mean).max(0.0).sqrt();
    Some((n_a, n_b, (n_a + n_b) / 2.0, sd))
}

fn g023_pair_list(panel: &Panel, cfg: &Config) -> Vec<(usize, usize)> {
    let listed: Vec<(usize, usize)> = split_csv(&cfg.pairs)
        .into_iter()
        .filter_map(|entry| {
            let mut parts = entry.split(':');
            let a = parts.next()?.to_string();
            let b = parts.next()?.to_string();
            Some((
                panel.symbols.iter().position(|s| *s == a)?,
                panel.symbols.iter().position(|s| *s == b)?,
            ))
        })
        .collect();
    if !listed.is_empty() {
        return listed;
    }
    let mut all = Vec::new();
    for a in 0..panel.symbols.len() {
        for b in a + 1..panel.symbols.len() {
            all.push((a, b));
        }
    }
    all
}

pub fn compute_g023_signals(panel: &Panel, cfg: &Config) -> Result<Signals, String> {
    let n = panel.dates.len();
    let mut result = blank_signals(panel, "pair_adf_t");
    for j in 0..panel.symbols.len() {
        for t in 0..n {
            result.tradable[j][t] = panel.close[j][t].is_finite();
            result.pool[j][t] = result.tradable[j][t];
        }
    }
    result.benchmark_returns = crate::family4::benchmark_returns(panel, &cfg.benchmark);
    Ok(result)
}

/// One pair at a time: monthly screen, Bollinger-distance leg entry,
/// mid-band take-profit, outer-band stop, screen-fail exit.
pub fn g023_trades(
    panel: &Panel,
    _sig: &Signals,
    cfg: &Config,
    fill: &str,
) -> Result<Vec<Trade>, String> {
    if !matches!(fill, "close" | "next_open") {
        return Err(format!("G023 does not support fill {fill:?}"));
    }
    let pairs = g023_pair_list(panel, cfg);
    if pairs.is_empty() {
        return Err("G023 requires `pairs` or a multi-symbol universe".into());
    }
    let midband = cfg.entry_mode == "midband";
    let n = panel.dates.len();
    let is_open = fill == "next_open";
    let px_at = |j: usize, t: usize| -> f64 {
        if is_open {
            panel.open[j][t]
        } else {
            panel.close[j][t]
        }
    };
    let logs: Vec<Vec<f64>> = panel.close.iter().map(|c| log_series(c)).collect();
    let mut screens: std::collections::HashMap<(usize, usize), PairScreen> =
        std::collections::HashMap::new();
    let borrow_per_bar = cfg.borrow_bps_annual / 10_000.0 / cfg.periods_per_year as f64;
    let mut trades = Vec::new();
    // open position state
    let (mut hi_leg, mut lo_leg) = (0usize, 0usize);
    let (mut entry_px_hi, mut entry_px_lo) = (0.0, 0.0);
    let (mut sig_day, mut entry_idx) = (0usize, 0usize);
    let mut open = false;
    let mut open_pair = (0usize, 0usize);

    let push_legs = |hi: usize,
                         lo: usize,
                         sig_t: usize,
                         e_idx: usize,
                         x_idx: usize,
                         e_hi: f64,
                         e_lo: f64,
                         x_hi: f64,
                         x_lo: f64,
                         reason: &'static str,
                         trades: &mut Vec<Trade>| {
        for (j, w, e_px, x_px) in [
            (hi, -0.5f64, e_hi, x_hi),
            (lo, 0.5f64, e_lo, x_lo),
        ] {
            if !e_px.is_finite() || !x_px.is_finite() || e_px <= 0.0 || x_px <= 0.0 {
                continue;
            }
            let gross = w.signum() * (x_px / e_px - 1.0);
            let cost = cfg.cost_for_symbol(&panel.symbols[j]);
            let borrow = if w < 0.0 {
                borrow_per_bar * (x_idx - e_idx + 1) as f64
            } else {
                0.0
            };
            trades.push(Trade {
                sym_idx: j,
                signal_idx: sig_t,
                entry_idx: e_idx,
                exit_idx: x_idx,
                entry_px: e_px,
                exit_px: x_px,
                ret: gross - cost * 2.0 * w.abs() - borrow,
                ret_gross: gross,
                exit_reason: reason,
                hold: x_idx - e_idx,
                signal_value: 1.0,
                weight: w,
                entry_turnover: w.abs(),
                exit_turnover: w.abs(),
                weighted: true,
                borrow_cost_per_bar: if w < 0.0 { borrow_per_bar } else { 0.0 },
            });
        }
    };

    for t in 0..n {
        if month_edge(panel, t) && t >= cfg.formation_window {
            screens.clear();
            for &(a, b) in &pairs {
                if let Some(s) =
                    g023_screen(&logs[a], &logs[b], t, cfg, cfg.corr_min)
                {
                    screens.insert((a, b), s);
                }
            }
            if open {
                let keep = g023_screen(
                    &logs[open_pair.0],
                    &logs[open_pair.1],
                    t,
                    cfg,
                    cfg.corr_keep,
                )
                .is_some();
                if !keep {
                    let x = if is_open { t + 1 } else { t };
                    if x < n {
                        push_legs(
                            hi_leg, lo_leg, sig_day, entry_idx, x,
                            entry_px_hi, entry_px_lo,
                            px_at(hi_leg, x), px_at(lo_leg, x),
                            "inactive", &mut trades,
                        );
                        open = false;
                    }
                }
            }
        }
        if open {
            let Some((n_hi, n_lo, mid, sd)) =
                g023_bands(&panel.close[hi_leg], &panel.close[lo_leg], t, cfg)
            else {
                continue;
            };
            let band = cfg.bb_mult * sd;
            let tp = n_hi <= mid || n_lo >= mid;
            let stop = n_hi > mid + band || n_lo < mid - band;
            if tp || stop {
                let x = if is_open { t + 1 } else { t };
                if x >= n {
                    continue;
                }
                push_legs(
                    hi_leg, lo_leg, sig_day, entry_idx, x,
                    entry_px_hi, entry_px_lo,
                    px_at(hi_leg, x), px_at(lo_leg, x),
                    if tp { "mid" } else { "band" },
                    &mut trades,
                );
                open = false;
            }
            continue;
        }
        // flat: pick lowest-ADF screened pair passing the band entry check
        let mut best: Option<(usize, usize, usize, usize, f64)> = None;
        for &(a, b) in &pairs {
            let Some(s) = screens.get(&(a, b)).or_else(|| screens.get(&(b, a))) else {
                continue;
            };
            let Some((n_a, n_b, mid, sd)) =
                g023_bands(&panel.close[a], &panel.close[b], t, cfg)
            else {
                continue;
            };
            let band = cfg.bb_mult * sd;
            let (hi, lo) = if n_a >= n_b { (a, b) } else { (b, a) };
            let (n_hi, n_lo) = if n_a >= n_b { (n_a, n_b) } else { (n_b, n_a) };
            let entered = if midband {
                (n_hi - mid).abs() < cfg.pair_dist
                    && (n_lo - mid).abs() < cfg.pair_dist
                    && n_hi > mid
                    && n_lo < mid
            } else {
                (n_hi - (mid + band)).abs() < cfg.pair_dist && n_lo < mid
            };
            if !entered {
                continue;
            }
            if best.is_none_or(|(_, _, _, _, adf)| s.adf_t < adf) {
                best = Some((a, b, hi, lo, s.adf_t));
            }
        }
        if let Some((_, _, hi, lo, _)) = best {
            let e = if is_open { t + 1 } else { t };
            if e >= n {
                continue;
            }
            let (e_hi, e_lo) = (px_at(hi, e), px_at(lo, e));
            if !e_hi.is_finite() || !e_lo.is_finite() || e_hi <= 0.0 || e_lo <= 0.0 {
                continue;
            }
            hi_leg = hi;
            lo_leg = lo;
            entry_px_hi = e_hi;
            entry_px_lo = e_lo;
            sig_day = t;
            entry_idx = e;
            open_pair = (hi, lo);
            open = true;
        }
    }
    if open {
        push_legs(
            hi_leg, lo_leg, sig_day, entry_idx, n - 1,
            entry_px_hi, entry_px_lo,
            panel.close[hi_leg][n - 1], panel.close[lo_leg][n - 1],
            "censored", &mut trades,
        );
    }
    trades.sort_by(|x, y| {
        (x.entry_idx, &panel.symbols[x.sym_idx]).cmp(&(y.entry_idx, &panel.symbols[y.sym_idx]))
    });
    Ok(trades)
}

// ---------------- G037 partial-cointegration Kalman residual reversion ------
//
// Clegg-Krauss PCI: e_t = logA - beta*logB decomposes as M_t + R_t with
// M_t = rho*M_{t-1} + w (AR1) and R_t = R_{t-1} + v (random walk).
// Model fits by Kalman Gaussian log-likelihood over a (rho, sigma_R/sigma_M)
// grid; PAR preferred by AIC vs AR(1) and RW nests.

/// Kalman innovations log-likelihood + filtered M for the PCI state space.
/// Returns (loglik, m_filtered). Observation noise floored for stability.
fn g037_pci_kalman(e: &[f64], rho: f64, s_m: f64, s_r: f64) -> (f64, Vec<f64>) {
    let n = e.len();
    let mut m = vec![0.0; n];
    // state [M, R], F = [[rho,0],[0,1]], Q = diag(s_m^2, s_r^2), H = [1,1]
    let (mut pm, mut pr) = (s_m * s_m / (1.0 - rho * rho).max(1e-6), 10.0 * s_r * s_r);
    let (mut xm, mut xr) = (0.0, e[0]);
    let obs_noise = 1e-8;
    let mut loglik = 0.0;
    for (t, &y) in e.iter().enumerate() {
        // predict
        let pm_p = rho * rho * pm + s_m * s_m;
        let pr_p = pr + s_r * s_r;
        let xm_p = rho * xm;
        let xr_p = xr;
        // update: y = M + R + tiny noise
        let s = pm_p + pr_p + obs_noise;
        let innov = y - xm_p - xr_p;
        if t > 0 && s > 0.0 {
            loglik += -0.5 * ((2.0 * std::f64::consts::PI * s).ln() + innov * innov / s);
        }
        let km = pm_p / s;
        let kr = pr_p / s;
        xm = xm_p + km * innov;
        xr = xr_p + kr * innov;
        pm = (1.0 - km) * pm_p;
        pr = (1.0 - kr) * pr_p;
        m[t] = xm;
    }
    (loglik, m)
}

struct G037Fit {
    alpha: f64,
    beta: f64,
    rho: f64,
    s_m: f64,
    s_r: f64,
    r2: f64,
    aic_par: f64,
    aic_ar: f64,
    aic_rw: f64,
}

/// Fit PAR/AR/RW on the residual window; pick best (rho, ratio) by PAR loglik.
fn g037_fit(log_a: &[f64], log_b: &[f64]) -> Option<G037Fit> {
    let n = log_a.len();
    if n < 120 {
        return None;
    }
    // beta = OLS slope logA ~ logB (with intercept)
    let x: Vec<Vec<f64>> = vec![vec![1.0; n], log_b.to_vec()];
    let coef = crate::family7::ols(&x, log_a)?;
    let beta = coef[1];
    if !(0.75..=1.25).contains(&beta) {
        return None;
    }
    let e: Vec<f64> = (0..n).map(|t| log_a[t] - coef[0] - beta * log_b[t]).collect();
    let var_e = {
        let mu = e.iter().sum::<f64>() / n as f64;
        e.iter().map(|v| (v - mu).powi(2)).sum::<f64>() / n as f64
    };
    if var_e <= 0.0 {
        return None;
    }
    // AR(1) fit for rho seed + AR/RW likelihoods for the AIC comparison
    let rho_ar = {
        let m = n - 1;
        let xa: Vec<Vec<f64>> = vec![vec![1.0; m], (0..m).map(|i| e[i]).collect()];
        let ya: Vec<f64> = (1..n).map(|i| e[i]).collect();
        crate::family7::ols(&xa, &ya).map(|c| c[1]).unwrap_or(0.0)
    };
    // PAR grid: rho in (0.05..0.95), sigma_R/sigma_M ratio in {0..3}
    let mut best: Option<(f64, f64, f64, f64)> = None; // ll, rho, s_m, s_r
    let mut rho = 0.05f64;
    while rho <= 0.95 + 1e-9 {
        let mut ratio = 0.0f64;
        while ratio <= 3.0 + 1e-9 {
            let s_m = var_e.sqrt();
            let s_r = ratio * s_m;
            let (ll, _) = g037_pci_kalman(&e, rho, s_m.max(1e-6), s_r.max(1e-6));
            if best.is_none_or(|(b, _, _, _)| ll > b) {
                best = Some((ll, rho, s_m.max(1e-6), s_r.max(1e-6)));
            }
            ratio += 0.25;
        }
        rho += 0.05;
    }
    let (ll_par, rho_f, s_m, s_r) = best?;
    // AR(1) nest: same Kalman with s_r ~ 0 at fitted rho_ar
    let (ll_ar, _) = g037_pci_kalman(&e, rho_ar.clamp(0.01, 0.99), var_e.sqrt(), 1e-6);
    // RW nest: rho->0 M, random walk only
    let (ll_rw, _) = g037_pci_kalman(&e, 0.0, 1e-6, var_e.sqrt());
    // AIC = 2k - 2ll ; k = 3,2,1 (variance params; beta shared)
    let aic_par = 2.0 * 3.0 - 2.0 * ll_par;
    let aic_ar = 2.0 * 2.0 - 2.0 * ll_ar;
    let aic_rw = 2.0 * 1.0 - 2.0 * ll_rw;
    // R2 = stationary share of variance: var(M) / (var(M)+sigma_r^2)
    let var_m = s_m * s_m / (1.0 - rho_f * rho_f).max(1e-6);
    let r2 = var_m / (var_m + s_r * s_r);
    Some(G037Fit {
        alpha: coef[0],
        beta,
        rho: rho_f,
        s_m,
        s_r,
        r2,
        aic_par,
        aic_ar,
        aic_rw,
    })
}

pub fn compute_g037_signals(panel: &Panel, _cfg: &Config) -> Result<Signals, String> {
    let n = panel.dates.len();
    let mut result = blank_signals(panel, "pci_m");
    for j in 0..panel.symbols.len() {
        for t in 0..n {
            result.tradable[j][t] = panel.close[j][t].is_finite();
            result.pool[j][t] = result.tradable[j][t];
        }
    }
    result.benchmark_returns = crate::family4::benchmark_returns(panel, "SPY");
    Ok(result)
}

/// Per-pair independent books; concurrent pairs allowed (unlike G023).
/// M > +sigma -> short residual (short A, long beta*B); exit at -0.5 sigma.
pub fn g037_trades(
    panel: &Panel,
    _sig: &Signals,
    cfg: &Config,
    fill: &str,
) -> Result<Vec<Trade>, String> {
    if !matches!(fill, "close" | "next_open") {
        return Err(format!("G037 does not support fill {fill:?}"));
    }
    let pairs = g023_pair_list(panel, cfg);
    if pairs.is_empty() {
        return Err("G037 requires `pairs` or a multi-symbol universe".into());
    }
    let n = panel.dates.len();
    let is_open = fill == "next_open";
    let px_at = |j: usize, t: usize| -> f64 {
        if is_open {
            panel.open[j][t]
        } else {
            panel.close[j][t]
        }
    };
    let logs: Vec<Vec<f64>> = panel.close.iter().map(|c| log_series(c)).collect();
    let borrow_per_bar = cfg.borrow_bps_annual / 10_000.0 / cfg.periods_per_year as f64;
    let mut trades = Vec::new();

    for &(a, b) in &pairs {
        let mut fit: Option<G037Fit> = None;
        let mut m_series: Vec<f64> = Vec::new();
        let mut sigma_m = f64::NAN;
        // position: 0 flat, +1 long residual (long A short beta B), -1 short residual
        let mut dir = 0.0f64;
        let (mut e_a, mut e_b) = (0.0, 0.0);
        let (mut sig_t, mut e_idx) = (0usize, 0usize);
        let mut hedge = 1.0f64;

        for t in 0..n {
            if month_edge(panel, t) && t >= cfg.formation_window {
                // re-verify: PAR must stay AIC-best with rho and R2 gates
                let ok = window(&logs[a], t, cfg.formation_window)
                    .zip(window(&logs[b], t, cfg.formation_window))
                    .and_then(|(wa, wb)| g037_fit(&wa, &wb))
                    .is_some_and(|f| {
                        f.aic_par < f.aic_ar
                            && f.aic_par < f.aic_rw
                            && f.rho > 0.5
                            && f.rho < 0.8
                            && f.r2 > 0.5
                    });
                if !ok && dir != 0.0 {
                    let x = if is_open { t + 1 } else { t };
                    if x < n {
                        push_g037_legs(
                            &mut trades, panel, cfg, a, b, dir, hedge, sig_t, e_idx, x,
                            e_a, e_b, px_at(a, x), px_at(b, x), "inactive", borrow_per_bar,
                        );
                        dir = 0.0;
                    }
                }
                if ok || dir == 0.0 {
                    if let (Some(wa), Some(wb)) = (
                        window(&logs[a], t, cfg.formation_window),
                        window(&logs[b], t, cfg.formation_window),
                    ) {
                        if let Some(f) = g037_fit(&wa, &wb) {
                            if f.aic_par < f.aic_ar && f.aic_par < f.aic_rw && f.rho > 0.5 && f.rho < 0.8 && f.r2 > 0.5 {
                                let e: Vec<f64> =
                                    wa.iter().zip(&wb).map(|(x, y)| x - f.alpha - f.beta * y).collect();
                                let (_, m) = g037_pci_kalman(&e, f.rho, f.s_m, f.s_r);
                                sigma_m = {
                                    let mu = m.iter().sum::<f64>() / m.len() as f64;
                                    (m.iter().map(|v| (v - mu).powi(2)).sum::<f64>()
                                        / m.len() as f64)
                                        .sqrt()
                                };
                                m_series = m;
                                fit = Some(f);
                            } else {
                                fit = None;
                                m_series.clear();
                            }
                        }
                    }
                }
            }
            let Some(f) = &fit else { continue };
            if m_series.is_empty() || !sigma_m.is_finite() || sigma_m <= 0.0 {
                continue;
            }
            // current M estimate: refilter over the live window is too heavy;
            // advance the filter one step on today's residual observation
            if t + 1 < cfg.formation_window {
                continue;
            }
            // live M_t: refilter the last 60-observation tail (filter mixing is
            // fast for rho<0.8, so the tail estimate approximates the full filter)
            let m_now = {
                let tail = 60usize.min(t + 1);
                let e_tail: Vec<f64> = (t + 1 - tail..=t)
                    .map(|u| logs[a][u] - f.alpha - f.beta * logs[b][u])
                    .collect();
                if e_tail.iter().any(|v| !v.is_finite()) {
                    continue;
                }
                let (_, mt) = g037_pci_kalman(&e_tail, f.rho, f.s_m, f.s_r);
                *mt.last().unwrap()
            };
            hedge = f.beta;
            if dir == 0.0 {
                if m_now > cfg.z_entry * sigma_m {
                    dir = -1.0;
                } else if m_now < -cfg.z_entry * sigma_m {
                    dir = 1.0;
                } else {
                    continue;
                }
                let e = if is_open { t + 1 } else { t };
                if e >= n {
                    dir = 0.0;
                    continue;
                }
                let (pa, pb) = (px_at(a, e), px_at(b, e));
                if !pa.is_finite() || !pb.is_finite() || pa <= 0.0 || pb <= 0.0 {
                    dir = 0.0;
                    continue;
                }
                e_a = pa;
                e_b = pb;
                sig_t = t;
                e_idx = e;
            } else {
                let exit = (dir < 0.0 && m_now < -cfg.exit_z * sigma_m)
                    || (dir > 0.0 && m_now > cfg.exit_z * sigma_m);
                if exit {
                    let x = if is_open { t + 1 } else { t };
                    if x >= n {
                        continue;
                    }
                    push_g037_legs(
                        &mut trades, panel, cfg, a, b, dir, hedge, sig_t, e_idx, x,
                        e_a, e_b, px_at(a, x), px_at(b, x), "m_exit", borrow_per_bar,
                    );
                    dir = 0.0;
                }
            }
        }
        if dir != 0.0 {
            push_g037_legs(
                &mut trades, panel, cfg, a, b, dir, hedge, sig_t, e_idx, n - 1,
                e_a, e_b, panel.close[a][n - 1], panel.close[b][n - 1], "censored",
                borrow_per_bar,
            );
        }
    }
    trades.sort_by(|x, y| {
        (x.entry_idx, &panel.symbols[x.sym_idx]).cmp(&(y.entry_idx, &panel.symbols[y.sym_idx]))
    });
    Ok(trades)
}

#[allow(clippy::too_many_arguments)]
fn push_g037_legs(
    trades: &mut Vec<Trade>,
    panel: &Panel,
    cfg: &Config,
    a: usize,
    b: usize,
    dir: f64,
    hedge: f64,
    sig_t: usize,
    e_idx: usize,
    x_idx: usize,
    e_a: f64,
    e_b: f64,
    x_a: f64,
    x_b: f64,
    reason: &'static str,
    borrow_per_bar: f64,
) {
    // dir=+1 long residual: long A, short hedge*B; weights |wa|+|wb|=1
    let wa = dir / (1.0 + hedge);
    let wb = -dir * hedge / (1.0 + hedge);
    for (j, w, e_px, x_px) in [(a, wa, e_a, x_a), (b, wb, e_b, x_b)] {
        if w.abs() < 1e-9 || !e_px.is_finite() || !x_px.is_finite() || e_px <= 0.0 {
            continue;
        }
        let gross = w.signum() * (x_px / e_px - 1.0);
        let cost = cfg.cost_for_symbol(&panel.symbols[j]);
        let borrow = if w < 0.0 {
            borrow_per_bar * (x_idx - e_idx + 1) as f64
        } else {
            0.0
        };
        trades.push(Trade {
            sym_idx: j,
            signal_idx: sig_t,
            entry_idx: e_idx,
            exit_idx: x_idx,
            entry_px: e_px,
            exit_px: x_px,
            ret: gross - cost * 2.0 * w.abs() - borrow,
            ret_gross: gross,
            exit_reason: reason,
            hold: x_idx - e_idx,
            signal_value: dir,
            weight: w,
            entry_turnover: w.abs(),
            exit_turnover: w.abs(),
            weighted: true,
            borrow_cost_per_bar: if w < 0.0 { borrow_per_bar } else { 0.0 },
        });
    }
}

// ---------------- shared: cross-sectional factor decile wrapper -------------
//
// s078-style daily L/S: on each rebalance day, rank eligible names by score
// (higher = long), hold top/bottom `top_fraction` at +-1/k weights.

fn rolling_beta(
    panel: &Panel,
    mkt: &[f64],
    j: usize,
    day: usize,
    window: usize,
    min_obs: usize,
) -> f64 {
    if day + 1 < window + 1 {
        return f64::NAN;
    }
    let start = day + 1 - window;
    let mut xs = Vec::new();
    let mut ys = Vec::new();
    for u in start..=day {
        if panel.close[j][u].is_finite()
            && panel.close[j][u - 1].is_finite()
            && panel.close[j][u - 1] > 0.0
            && mkt[u].is_finite()
        {
            xs.push(mkt[u]);
            ys.push(panel.close[j][u] / panel.close[j][u - 1] - 1.0);
        }
    }
    if ys.len() < min_obs {
        return f64::NAN;
    }
    let mx = xs.iter().sum::<f64>() / xs.len() as f64;
    let my = ys.iter().sum::<f64>() / ys.len() as f64;
    let cov: f64 = xs.iter().zip(&ys).map(|(x, y)| (x - mx) * (y - my)).sum();
    let var: f64 = xs.iter().map(|x| (x - mx).powi(2)).sum();
    if var <= 0.0 {
        f64::NAN
    } else {
        cov / var
    }
}

/// EW market return across finite names (same convention as s078).
pub(crate) fn ew_market(panel: &Panel) -> Vec<f64> {
    let n = panel.dates.len();
    let mut mkt = vec![f64::NAN; n];
    for t in 1..n {
        let (mut sum, mut count) = (0.0, 0usize);
        for j in 0..panel.symbols.len() {
            let (a, b) = (panel.close[j][t - 1], panel.close[j][t]);
            if a.is_finite() && a > 0.0 && b.is_finite() {
                sum += b / a - 1.0;
                count += 1;
            }
        }
        if count > 0 {
            mkt[t] = sum / count as f64;
        }
    }
    mkt
}

/// score[j][t] higher => long at day-t rebalance. `groups` (optional) demeans
/// score within each industry/sector group per date among eligible names.
pub(crate) fn factor_decile_signals(
    panel: &Panel,
    cfg: &Config,
    label: &str,
    mut score: Vec<Vec<f64>>,
    groups: Option<&[Option<String>]>,
) -> Result<Signals, String> {
    let n = panel.dates.len();
    let cols: Vec<usize> = (0..panel.symbols.len()).collect();
    let ranks = crate::family5::load_ranks(panel, cfg, &cols)?;
    let mut result = blank_signals(panel, label);
    for j in 0..panel.symbols.len() {
        for t in 0..n {
            result.tradable[j][t] = panel.close[j][t].is_finite()
                && panel.close[j][t] >= cfg.min_price
                && ranks[j][t].is_finite()
                && ranks[j][t] <= cfg.universe_top_n as f64;
            result.pool[j][t] = result.tradable[j][t];
        }
    }
    if let Some(groups) = groups {
        // industry demean: subtract each date-group's eligible-name mean
        let mut by_group: std::collections::BTreeMap<&str, Vec<usize>> =
            std::collections::BTreeMap::new();
        for (j, g) in groups.iter().enumerate() {
            if let Some(g) = g.as_deref() {
                by_group.entry(g).or_default().push(j);
            }
        }
        for t in 0..n {
            for members in by_group.values() {
                let vals: Vec<f64> = members
                    .iter()
                    .filter(|&&j| result.tradable[j][t] && score[j][t].is_finite())
                    .map(|&j| score[j][t])
                    .collect();
                if vals.len() < 5 {
                    continue;
                }
                let mu = vals.iter().sum::<f64>() / vals.len() as f64;
                for &j in members {
                    if score[j][t].is_finite() {
                        score[j][t] -= mu;
                    }
                }
            }
        }
    }
    if cfg.factor_invert {
        for row in &mut score {
            for v in row.iter_mut() {
                if v.is_finite() {
                    *v = -*v;
                }
            }
        }
    }
    let mkt = ew_market(panel);
    let mut weights = vec![vec![0.0; n]; panel.symbols.len()];
    for day in crate::family4::rebalance_days(panel, &cfg.rebalance)? {
        let mut scored: Vec<(usize, f64)> = (0..panel.symbols.len())
            .filter(|&j| result.tradable[j][day] && score[j][day].is_finite())
            .map(|j| (j, score[j][day]))
            .collect();
        if scored.len() < 10 {
            continue;
        }
        scored.sort_by(|a, b| {
            b.1.total_cmp(&a.1).then(panel.symbols[a.0].cmp(&panel.symbols[b.0]))
        });
        let k = ((scored.len() as f64) * cfg.top_fraction).max(1.0) as usize;
        let longs = &scored[..k.min(scored.len())];
        let shorts = &scored[scored.len().saturating_sub(k)..];
        if longs.is_empty() || shorts.is_empty() {
            continue;
        }
        let (mut beta_l, mut beta_s) = (1.0, 1.0);
        if cfg.beta_neutral {
            let l: Vec<f64> = longs
                .iter()
                .map(|&(j, _)| rolling_beta(panel, &mkt, j, day, cfg.beta_window, cfg.beta_min_obs))
                .collect();
            let s: Vec<f64> = shorts
                .iter()
                .map(|&(j, _)| rolling_beta(panel, &mkt, j, day, cfg.beta_window, cfg.beta_min_obs))
                .collect();
            if l.iter().all(|b| b.is_finite()) && s.iter().all(|b| b.is_finite()) {
                beta_l = l.iter().sum::<f64>() / l.len() as f64;
                beta_s = s.iter().sum::<f64>() / s.len() as f64;
            }
        }
        let scale = if beta_l.abs() > 1e-9 { beta_s / beta_l } else { 1.0 };
        for &(j, v) in longs {
            weights[j][day] = scale / longs.len() as f64;
            result.rsi[j][day] = v;
            result.signal[j][day] = true;
        }
        for &(j, v) in shorts {
            weights[j][day] = -1.0 / shorts.len() as f64;
            result.rsi[j][day] = v;
            result.signal[j][day] = true;
        }
    }
    result.trade_weights = Some(weights);
    result.benchmark_returns =
        crate::family5::ew_stock_benchmark(panel, &result.tradable, &std::collections::BTreeSet::new());
    Ok(result)
}

/// Cross-sectional winsorize at q / 1-q per date among finite values.
pub(crate) fn winsorize_dates(score: &mut [Vec<f64>], q: f64) {
    let n = score.first().map_or(0, Vec::len);
    for t in 0..n {
        let mut vals: Vec<f64> = (0..score.len())
            .filter_map(|j| {
                let v = score[j][t];
                if v.is_finite() { Some(v) } else { None }
            })
            .collect();
        if vals.len() < 20 {
            continue;
        }
        vals.sort_by(|a, b| a.total_cmp(b));
        let lo = vals[(q * (vals.len() - 1) as f64).round() as usize];
        let hi = vals[((1.0 - q) * (vals.len() - 1) as f64).round() as usize];
        for j in 0..score.len() {
            if score[j][t].is_finite() {
                score[j][t] = score[j][t].clamp(lo, hi);
            }
        }
    }
}

/// Simfin industry groups for the panel symbols (empty when unmapped).
pub(crate) fn industry_groups(
    panel: &Panel,
    cfg: &Config,
) -> Result<Option<Vec<Option<String>>>, String> {
    if !cfg.industry_neutral && cfg.simfin_dir.trim().is_empty() {
        return Ok(None);
    }
    if cfg.simfin_dir.trim().is_empty() {
        return Ok(None);
    }
    let (industries, _sectors) = crate::signals::simfin_groups(&panel.symbols, &cfg.simfin_dir)?;
    Ok(Some(industries))
}

// ---------------- G003 daily high-low geometric-midpoint reversal ------------
// score = -(industry-demeaned winsorized close/sqrt(high*low)), lag-averaged.
pub fn compute_g003_signals(panel: &Panel, cfg: &Config) -> Result<Signals, String> {
    let n = panel.dates.len();
    let mut score = vec![vec![f64::NAN; n]; panel.symbols.len()];
    for j in 0..panel.symbols.len() {
        for t in 0..n {
            let (c, h, l) = (panel.close[j][t], panel.high[j][t], panel.low[j][t]);
            if c.is_finite() && h.is_finite() && l.is_finite() && h * l > 0.0 {
                score[j][t] = c / (h * l).sqrt();
            }
        }
    }
    winsorize_dates(&mut score, 0.025);
    for v in &mut score {
        for x in v.iter_mut() {
            if x.is_finite() {
                *x = -*x;
            }
        }
    }
    if cfg.n_lags > 1 {
        for j in 0..score.len() {
            let orig = score[j].clone();
            for t in 0..n {
                let mut s = 0.0;
                let mut cnt = 0;
                for lag in 0..cfg.n_lags {
                    if t >= lag && orig[t - lag].is_finite() {
                        s += orig[t - lag];
                        cnt += 1;
                    }
                }
                score[j][t] = if cnt == cfg.n_lags {
                    s / cnt as f64
                } else {
                    f64::NAN
                };
            }
        }
    }
    let groups = industry_groups(panel, cfg)?;
    factor_decile_signals(panel, cfg, "hl0_demean", score, groups.as_deref())
}

// ---------------- Tier-2 cross-sectional factors (decile wrapper) -----------

fn day_ret(panel: &Panel, j: usize, t: usize) -> f64 {
    let (a, b) = (panel.close[j][t - 1], panel.close[j][t]);
    if a.is_finite() && a > 0.0 && b.is_finite() {
        b / a - 1.0
    } else {
        f64::NAN
    }
}

/// G021: -(relative volume x beta-adjusted return), averaged over n_lags.
pub fn compute_g021_signals(panel: &Panel, cfg: &Config) -> Result<Signals, String> {
    let n = panel.dates.len();
    let mkt = ew_market(panel);
    let mut score = vec![vec![f64::NAN; n]; panel.symbols.len()];
    for j in 0..panel.symbols.len() {
        for t in 1..n {
            let beta =
                rolling_beta(panel, &mkt, j, t, cfg.beta_window, cfg.beta_min_obs);
            if !beta.is_finite() {
                continue;
            }
            let (mut s, mut cnt) = (0.0, 0usize);
            for lag in 0..cfg.n_lags {
                if t < lag + 1 {
                    break;
                }
                let u = t - lag;
                let r = day_ret(panel, j, u);
                if !r.is_finite() || u < cfg.lookback + 1 {
                    continue;
                }
                let mut vs: Vec<f64> = (u + 1 - cfg.lookback..u)
                    .map(|k| panel.volume[j][k])
                    .filter(|v| v.is_finite())
                    .collect();
                if vs.is_empty() || !panel.volume[j][u].is_finite() {
                    continue;
                }
                vs.sort_by(|a, b| a.total_cmp(b));
                let med = vs[vs.len() / 2];
                if med <= 0.0 {
                    continue;
                }
                let relvol = panel.volume[j][u] / med;
                s += relvol * (r - beta * mkt[u]);
                cnt += 1;
            }
            if cnt == cfg.n_lags {
                score[j][t] = -s / cnt as f64;
            }
        }
    }
    winsorize_dates(&mut score, 0.025);
    let groups = industry_groups(panel, cfg)?;
    factor_decile_signals(panel, cfg, "vadj_rev", score, groups.as_deref())
}

/// G104: 20-session log return lagged 232 sessions, industry-demeaned.
pub fn compute_g104_signals(panel: &Panel, cfg: &Config) -> Result<Signals, String> {
    let n = panel.dates.len();
    let lag = cfg.skip_days.max(1);
    let win = cfg.lookback.max(1);
    let mut score = vec![vec![f64::NAN; n]; panel.symbols.len()];
    for j in 0..panel.symbols.len() {
        for t in lag + win..n {
            let (a, b) = (panel.close[j][t - lag - win + 1], panel.close[j][t - lag]);
            if a.is_finite() && a > 0.0 && b.is_finite() {
                score[j][t] = (b / a).ln();
            }
        }
    }
    winsorize_dates(&mut score, 0.025);
    let groups = industry_groups(panel, cfg)?;
    factor_decile_signals(panel, cfg, "lag_indmom", score, groups.as_deref())
}

/// G105: -(multi-period return / beta), lag-averaged.
pub fn compute_g105_signals(panel: &Panel, cfg: &Config) -> Result<Signals, String> {
    let n = panel.dates.len();
    let mkt = ew_market(panel);
    let mut score = vec![vec![f64::NAN; n]; panel.symbols.len()];
    for j in 0..panel.symbols.len() {
        for t in cfg.lookback..n {
            let beta =
                rolling_beta(panel, &mkt, j, t, cfg.beta_window, cfg.beta_min_obs);
            if !beta.is_finite() || beta.abs() < 1e-6 {
                continue;
            }
            let (mut s, mut cnt) = (0.0, 0usize);
            for lag in 0..cfg.n_lags {
                if t < lag + cfg.lookback {
                    break;
                }
                let u = t - lag;
                let (a, b) = (panel.close[j][u - cfg.lookback], panel.close[j][u]);
                if a.is_finite() && a > 0.0 && b.is_finite() {
                    s += (b / a - 1.0) / beta;
                    cnt += 1;
                }
            }
            if cnt == cfg.n_lags {
                score[j][t] = -s / cnt as f64;
            }
        }
    }
    winsorize_dates(&mut score, 0.025);
    let groups = industry_groups(panel, cfg)?;
    factor_decile_signals(panel, cfg, "beta_div_rev", score, groups.as_deref())
}

/// G146: forecast = trailing mean of beta-adjusted returns conditioned on the
/// next session's weekday.
pub fn compute_g146_signals(panel: &Panel, cfg: &Config) -> Result<Signals, String> {
    use chrono::Weekday::*;
    let _ = (Mon, Tue, Wed, Thu, Fri);
    let n = panel.dates.len();
    let mkt = ew_market(panel);
    let mut score = vec![vec![f64::NAN; n]; panel.symbols.len()];
    for j in 0..panel.symbols.len() {
        for t in cfg.lookback..n.saturating_sub(1) {
            let beta =
                rolling_beta(panel, &mkt, j, t, cfg.beta_window, cfg.beta_min_obs);
            if !beta.is_finite() {
                continue;
            }
            let wd_next = panel.dates[t + 1].weekday().num_days_from_monday();
            let (mut s, mut cnt) = (0.0, 0usize);
            for u in t + 1 - cfg.lookback..=t {
                if u == 0 {
                    continue;
                }
                if panel.dates[u].weekday().num_days_from_monday() != wd_next {
                    continue;
                }
                let r = day_ret(panel, j, u);
                if r.is_finite() && mkt[u].is_finite() {
                    s += r - beta * mkt[u];
                    cnt += 1;
                }
            }
            if cnt >= 4 {
                score[j][t] = s / cnt as f64;
            }
        }
    }
    winsorize_dates(&mut score, 0.025);
    let groups = industry_groups(panel, cfg)?;
    factor_decile_signals(panel, cfg, "weekday_fc", score, groups.as_deref())
}

/// G119 Alpha101-009: -d1 normally, +d1 when the last 5 deltas are all down.
pub fn compute_g119_signals(panel: &Panel, cfg: &Config) -> Result<Signals, String> {
    let n = panel.dates.len();
    let mut score = vec![vec![f64::NAN; n]; panel.symbols.len()];
    for j in 0..panel.symbols.len() {
        for t in 6..n {
            let d1 = day_ret(panel, j, t);
            if !d1.is_finite() {
                continue;
            }
            let mut all_down = true;
            for u in t - 4..=t {
                let d = day_ret(panel, j, u);
                if !d.is_finite() || d >= 0.0 {
                    all_down = false;
                    break;
                }
            }
            score[j][t] = if all_down { d1 } else { -d1 };
        }
    }
    winsorize_dates(&mut score, 0.025);
    let groups = industry_groups(panel, cfg)?;
    factor_decile_signals(panel, cfg, "alpha009", score, groups.as_deref())
}

/// G120 Alpha101-012: sign(delta volume) * -delta close.
pub fn compute_g120_signals(panel: &Panel, cfg: &Config) -> Result<Signals, String> {
    let n = panel.dates.len();
    let mut score = vec![vec![f64::NAN; n]; panel.symbols.len()];
    for j in 0..panel.symbols.len() {
        for t in 1..n {
            let d = day_ret(panel, j, t);
            let (va, vb) = (panel.volume[j][t - 1], panel.volume[j][t]);
            if d.is_finite() && va.is_finite() && vb.is_finite() {
                score[j][t] = (vb - va).signum() * -d;
            }
        }
    }
    winsorize_dates(&mut score, 0.025);
    let groups = industry_groups(panel, cfg)?;
    factor_decile_signals(panel, cfg, "alpha012", score, groups.as_deref())
}

/// G121 Alpha101-101: (close-open)/(high-low+eps) candle-location factor.
pub fn compute_g121_signals(panel: &Panel, cfg: &Config) -> Result<Signals, String> {
    let n = panel.dates.len();
    let mut score = vec![vec![f64::NAN; n]; panel.symbols.len()];
    for j in 0..panel.symbols.len() {
        for t in 0..n {
            let (o, h, l, c) = (
                panel.open[j][t],
                panel.high[j][t],
                panel.low[j][t],
                panel.close[j][t],
            );
            if o.is_finite() && h.is_finite() && l.is_finite() && c.is_finite() {
                score[j][t] = (c - o) / (h - l + 0.001);
            }
        }
    }
    winsorize_dates(&mut score, 0.025);
    let groups = industry_groups(panel, cfg)?;
    factor_decile_signals(panel, cfg, "alpha101", score, groups.as_deref())
}

/// G123 Alpha191-020: six-session percent change x 100.
pub fn compute_g123_signals(panel: &Panel, cfg: &Config) -> Result<Signals, String> {
    let n = panel.dates.len();
    let lag = cfg.skip_days.max(1);
    let mut score = vec![vec![f64::NAN; n]; panel.symbols.len()];
    for j in 0..panel.symbols.len() {
        for t in lag..n {
            let (a, b) = (panel.close[j][t - lag], panel.close[j][t]);
            if a.is_finite() && a > 0.0 && b.is_finite() {
                score[j][t] = (b / a - 1.0) * 100.0;
            }
        }
    }
    winsorize_dates(&mut score, 0.025);
    let groups = industry_groups(panel, cfg)?;
    factor_decile_signals(panel, cfg, "alpha191_020", score, groups.as_deref())
}

// ---------------- Tier-2 swing strategies -----------------------------------

fn tradable_pool(panel: &Panel, cfg: &Config, result: &mut Signals) {
    let n = panel.dates.len();
    for j in 0..panel.symbols.len() {
        for t in 0..n {
            result.tradable[j][t] =
                panel.close[j][t].is_finite() && panel.close[j][t] >= cfg.min_price;
            result.pool[j][t] = result.tradable[j][t];
        }
    }
}

/// Mirror of the long swing engine for short entries: enter on `signal`,
/// exit on `exit_now`, `stop_pct` adverse move, or `max_hold` sessions.
pub(crate) fn short_swing_trades(
    panel: &Panel,
    sig: &Signals,
    cfg: &Config,
    fill: &str,
) -> Result<Vec<Trade>, String> {
    if !matches!(fill, "close" | "next_open") {
        return Err(format!("{} does not support fill {fill:?}", cfg.strategy));
    }
    let n = panel.dates.len();
    let is_open = fill == "next_open";
    let borrow_per_bar = cfg.borrow_bps_annual / 10_000.0 / cfg.periods_per_year as f64;
    let mut trades = Vec::new();
    for j in 0..panel.symbols.len() {
        let cost = cfg.cost_for_symbol(&panel.symbols[j]);
        let mut open: Option<(usize, usize, f64)> = None; // (sig_t, entry_t, entry_px)
        for t in 0..n {
            if let Some((sig_t, e_t, e_px)) = open {
                let mut reason: Option<&'static str> = None;
                let c = panel.close[j][t];
                if t == n - 1 {
                    reason = Some("censored");
                } else if sig.exit_now[j][t] {
                    reason = Some("signal_exit");
                } else if c.is_finite() && e_px > 0.0 {
                    if let Some(stops) = &sig.stop_pct {
                        let sp = stops[j][e_t];
                        if sp.is_finite() && c / e_px - 1.0 > sp {
                            reason = Some("stop");
                        }
                    }
                    if reason.is_none()
                        && cfg.max_hold > 0
                        && t.saturating_sub(e_t) >= cfg.max_hold
                    {
                        reason = Some("time");
                    }
                }
                if let Some(r) = reason {
                    let x = if is_open { t + 1 } else { t };
                    if x >= n || r == "censored" {
                        let x = (n - 1).min(x);
                        let x_px = panel.close[j][x];
                        if x_px.is_finite() && e_px > 0.0 {
                            let gross = -(x_px / e_px - 1.0);
                            trades.push(Trade {
                                sym_idx: j,
                                signal_idx: sig_t,
                                entry_idx: e_t,
                                exit_idx: x,
                                entry_px: e_px,
                                exit_px: x_px,
                                ret: gross - cost * 2.0
                                    - borrow_per_bar * (x - e_t + 1) as f64,
                                ret_gross: gross,
                                exit_reason: r,
                                hold: x - e_t,
                                signal_value: sig.rsi[j][sig_t],
                                weight: -1.0,
                                entry_turnover: 1.0,
                                exit_turnover: 1.0,
                                weighted: false,
                                borrow_cost_per_bar: borrow_per_bar,
                            });
                        }
                        open = None;
                        continue;
                    }
                    let x_px = if is_open { panel.open[j][x] } else { panel.close[j][x] };
                    if x_px.is_finite() && e_px > 0.0 {
                        let gross = -(x_px / e_px - 1.0);
                        trades.push(Trade {
                            sym_idx: j,
                            signal_idx: sig_t,
                            entry_idx: e_t,
                            exit_idx: x,
                            entry_px: e_px,
                            exit_px: x_px,
                            ret: gross - cost * 2.0
                                - borrow_per_bar * (x - e_t + 1) as f64,
                            ret_gross: gross,
                            exit_reason: r,
                            hold: x - e_t,
                            signal_value: sig.rsi[j][sig_t],
                            weight: -1.0,
                            entry_turnover: 1.0,
                            exit_turnover: 1.0,
                            weighted: false,
                            borrow_cost_per_bar: borrow_per_bar,
                        });
                    }
                    open = None;
                    continue;
                }
            } else if sig.signal[j][t] && result_tradable(sig, j, t) {
                let e = if is_open { t + 1 } else { t };
                if e >= n {
                    break;
                }
                let e_px = if is_open { panel.open[j][e] } else { panel.close[j][e] };
                if e_px.is_finite() && e_px > 0.0 {
                    open = Some((t, e, e_px));
                }
            }
        }
        if let Some((sig_t, e_t, e_px)) = open {
            let x = n - 1;
            let x_px = panel.close[j][x];
            if x_px.is_finite() && e_px > 0.0 {
                let gross = -(x_px / e_px - 1.0);
                trades.push(Trade {
                    sym_idx: j,
                    signal_idx: sig_t,
                    entry_idx: e_t,
                    exit_idx: x,
                    entry_px: e_px,
                    exit_px: x_px,
                    ret: gross - cost * 2.0 - borrow_per_bar * (x - e_t + 1) as f64,
                    ret_gross: gross,
                    exit_reason: "censored",
                    hold: x - e_t,
                    signal_value: sig.rsi[j][sig_t],
                    weight: -1.0,
                    entry_turnover: 1.0,
                    exit_turnover: 1.0,
                    weighted: false,
                    borrow_cost_per_bar: borrow_per_bar,
                });
            }
        }
    }
    trades.sort_by(|x, y| {
        (x.entry_idx, &panel.symbols[x.sym_idx]).cmp(&(y.entry_idx, &panel.symbols[y.sym_idx]))
    });
    Ok(trades)
}

fn result_tradable(sig: &Signals, j: usize, t: usize) -> bool {
    sig.tradable.get(j).and_then(|r| r.get(t)).copied().unwrap_or(true)
}

/// G127: Bollinger(20,2) bottom-W -> long on upper-band cross; exit on
/// bandwidth collapse (band std < beta) unless a fresh pattern fires.
pub fn compute_g127_signals(panel: &Panel, cfg: &Config) -> Result<Signals, String> {
    let n = panel.dates.len();
    let mut result = blank_signals(panel, "bb_w");
    tradable_pool(panel, cfg, &mut result);
    let pat_win = cfg.lookback.max(20);
    let tol = cfg.breakout_distance_atr.max(0.0001);
    for j in 0..panel.symbols.len() {
        let sma = crate::signals::rolling_mean(&panel.close[j], cfg.bb_len);
        let mut sd = vec![f64::NAN; n];
        for t in cfg.bb_len..n {
            let w = &panel.close[j][t + 1 - cfg.bb_len..=t];
            if w.iter().any(|v| !v.is_finite()) {
                continue;
            }
            let m = w.iter().sum::<f64>() / cfg.bb_len as f64;
            sd[t] = (w.iter().map(|v| (v - m).powi(2)).sum::<f64>() / cfg.bb_len as f64)
                .sqrt();
        }
        for t in cfg.bb_len.max(2)..n {
            if !sma[t].is_finite() || !sd[t].is_finite() || !result.tradable[j][t] {
                continue;
            }
            let (lo, mid, up) = (sma[t] - cfg.bb_mult * sd[t], sma[t], sma[t] + cfg.bb_mult * sd[t]);
            let c = panel.close[j][t];
            if !c.is_finite() || lo <= 0.0 {
                continue;
            }
            // pattern scan: two lower-band tests within pat_win sessions with a
            // midpoint recovery between and the second low higher than the first
            let start = t.saturating_sub(pat_win);
            let (mut t1, mut t2) = (None, None);
            for u in start..t {
                if !sma[u].is_finite() || !sd[u].is_finite() {
                    continue;
                }
                let (lo_u, mid_u) = (sma[u] - cfg.bb_mult * sd[u], sma[u]);
                let (cu, hu) = (panel.close[j][u], panel.high[j][u]);
                if !cu.is_finite() || lo_u <= 0.0 {
                    continue;
                }
                let near_lo = (cu - lo_u).abs() / lo_u <= tol || cu <= lo_u * (1.0 + tol);
                if t1.is_none() && near_lo {
                    t1 = Some(u);
                    continue;
                }
                if let (Some(a), None) = (t1, t2) {
                    let recovered = (a..u).any(|k| {
                        panel.high[j][k].is_finite() && sma[k].is_finite() && panel.high[j][k] > mid_u
                    });
                    if near_lo && recovered && cu > panel.low[j][a] * (1.0 - tol) {
                        t2 = Some(u);
                    }
                }
            }
            let patterned = t2.is_some();
            if patterned && c > up && panel.close[j][t - 1] <= sma[t - 1] + cfg.bb_mult * sd[t - 1] {
                result.signal[j][t] = true;
                result.rsi[j][t] = c;
            }
            // exit on bandwidth collapse: rel sd below weakness_threshold x
            // the pattern window's max rel sd
            let rel_sd = sd[t] / mid;
            let mut max_rel = 0.0f64;
            for u in start..=t {
                if sd[u].is_finite() && sma[u].is_finite() && sma[u] > 0.0 {
                    max_rel = max_rel.max(sd[u] / sma[u]);
                }
            }
            if rel_sd.is_finite() && rel_sd < cfg.weakness_threshold * max_rel && !patterned {
                result.exit_now[j][t] = true;
            }
        }
    }
    Ok(result)
}

/// G128: Heikin-Ashi consecutive-candle reversal. Long on bearish HA candle
/// with open==high and body larger than prior bearish body; exit on bullish
/// HA with open==low preceded by a bullish HA.
pub fn compute_g128_signals(panel: &Panel, cfg: &Config) -> Result<Signals, String> {
    let n = panel.dates.len();
    let mut result = blank_signals(panel, "ha_rev");
    tradable_pool(panel, cfg, &mut result);
    for j in 0..panel.symbols.len() {
        let mut ho = vec![f64::NAN; n];
        let mut hc = vec![f64::NAN; n];
        for t in 0..n {
            let (o, h, l, c) = (
                panel.open[j][t],
                panel.high[j][t],
                panel.low[j][t],
                panel.close[j][t],
            );
            if !(o.is_finite() && h.is_finite() && l.is_finite() && c.is_finite()) {
                continue;
            }
            hc[t] = (o + h + l + c) / 4.0;
            ho[t] = if t == 0 || !ho[t - 1].is_finite() || !hc[t - 1].is_finite() {
                (o + c) / 2.0
            } else {
                (ho[t - 1] + hc[t - 1]) / 2.0
            };
        }
        for t in 1..n {
            if !result.tradable[j][t] || !ho[t].is_finite() || !hc[t].is_finite()
                || !ho[t - 1].is_finite() || !hc[t - 1].is_finite()
            {
                continue;
            }
            let bear = hc[t] < ho[t];
            let bull = hc[t] > ho[t];
            let body = (hc[t] - ho[t]).abs();
            let pbody = (hc[t - 1] - ho[t - 1]).abs();
            let rng_t = (panel.high[j][t] - panel.low[j][t]).max(1e-9);
            let no_up_wick = (panel.high[j][t] - ho[t]) <= 0.01 * rng_t;
            let no_dn_wick = (ho[t] - panel.low[j][t]) <= 0.01 * rng_t;
            if bear && no_up_wick && body > pbody && hc[t - 1] < ho[t - 1] {
                result.signal[j][t] = true;
                result.rsi[j][t] = body;
            }
            if bull && no_dn_wick && hc[t - 1] > ho[t - 1] {
                result.exit_now[j][t] = true;
            }
        }
    }
    Ok(result)
}

/// G129: RSI(14) head-and-shoulders -> short on completed right shoulder;
/// exit after RSI rises `add_rsi` points from entry or `max_hold` sessions.
pub fn compute_g129_signals(panel: &Panel, cfg: &Config) -> Result<Signals, String> {
    let n = panel.dates.len();
    let mut result = blank_signals(panel, "rsi_hs");
    tradable_pool(panel, cfg, &mut result);
    let tol = cfg.rsi_threshold; // shoulder-equality tolerance (RSI points)
    let mut stops = vec![vec![f64::NAN; n]; panel.symbols.len()];
    for j in 0..panel.symbols.len() {
        let rsi = crate::signals::wilder_rsi(&panel.close[j], cfg.rsi_len)?;
        let mut e_rsi = f64::NAN;
        for t in 10..n {
            if !result.tradable[j][t] {
                continue;
            }
            // local maxima among last ~15 sessions
            if rsi[t].is_finite() {
                let look = 15usize.min(t);
                let mut peaks: Vec<(usize, f64)> = Vec::new();
                for u in t + 1 - look..t {
                    if u == 0 || !rsi[u].is_finite() || !rsi[u - 1].is_finite() || !rsi[u + 1].is_finite() {
                        continue;
                    }
                    if rsi[u] >= rsi[u - 1] && rsi[u] >= rsi[u + 1] {
                        peaks.push((u, rsi[u]));
                    }
                }
                if peaks.len() >= 3 {
                    let (l, h, r) = (peaks[peaks.len() - 3], peaks[peaks.len() - 2], peaks[peaks.len() - 1]);
                    let hs = h.1 > l.1 + 1.0 && h.1 > r.1 + 1.0
                        && (l.1 - r.1).abs() <= tol
                        && rsi[t] < r.1;
                    if hs {
                        result.signal[j][t] = true;
                        result.rsi[j][t] = r.1;
                        e_rsi = rsi[t];
                    }
                }
                if e_rsi.is_finite() && rsi[t] > e_rsi + cfg.add_rsi {
                    result.exit_now[j][t] = true;
                    e_rsi = f64::NAN;
                }
            }
        }
        let _ = &mut stops;
    }
    result.stop_pct = Some(stops);
    Ok(result)
}

/// G130: shooting-star candle -> short; exit |move| > 5% or 7 sessions.
/// Canonical criteria: upper wick >= 2x body, lower wick <= 20% of range,
/// body <= 50% of range, close near session low.
pub fn compute_g130_signals(panel: &Panel, cfg: &Config) -> Result<Signals, String> {
    let n = panel.dates.len();
    let mut result = blank_signals(panel, "shooting_star");
    tradable_pool(panel, cfg, &mut result);
    let mut stops = vec![vec![f64::NAN; n]; panel.symbols.len()];
    for j in 0..panel.symbols.len() {
        for t in 0..n {
            if !result.tradable[j][t] {
                continue;
            }
            let (o, h, l, c) = (
                panel.open[j][t],
                panel.high[j][t],
                panel.low[j][t],
                panel.close[j][t],
            );
            if !(o.is_finite() && h.is_finite() && l.is_finite() && c.is_finite()) {
                continue;
            }
            let range = h - l;
            let body = (c - o).abs();
            let up_wick = h - o.max(c);
            let dn_wick = o.min(c) - l;
            if range <= 0.0 {
                continue;
            }
            // lower_bound=0.2 wick cap, body_size=0.5 body cap (source consts)
            let star = body <= cfg.rsi_threshold / 100.0 * range
                && up_wick >= 2.0 * body.max(1e-9 * range)
                && dn_wick <= cfg.weakness_threshold * range;
            if star {
                result.signal[j][t] = true;
                result.rsi[j][t] = up_wick / range;
                stops[j][t] = cfg.stop_loss;
            }
        }
    }
    result.stop_pct = Some(stops);
    Ok(result)
}

/// G202: ternary event engine. Composite of MACD(5,34) zero-cross, BB(20)
/// band breach (reversal), CMF sign change; enter on first event in a
/// direction, exit on opposite event.
pub fn compute_g202_signals(panel: &Panel, cfg: &Config) -> Result<Signals, String> {
    let n = panel.dates.len();
    let mut result = blank_signals(panel, "ternary_evt");
    tradable_pool(panel, cfg, &mut result);
    let mode = cfg.entry_mode.as_str();
    for j in 0..panel.symbols.len() {
        let c = &panel.close[j];
        let ema = |len: usize| -> Vec<f64> {
            let a = 2.0 / (len as f64 + 1.0);
            let mut out = vec![f64::NAN; n];
            for t in 0..n {
                out[t] = if !c[t].is_finite() {
                    out[t.saturating_sub(1)]
                } else if t == 0 || !out[t - 1].is_finite() {
                    c[t]
                } else {
                    a * c[t] + (1.0 - a) * out[t - 1]
                };
            }
            out
        };
        let (ef, es) = (ema(cfg.macd_fast), ema(cfg.macd_slow));
        let macd: Vec<f64> = (0..n).map(|t| ef[t] - es[t]).collect();
        let sma = crate::signals::rolling_mean(c, cfg.bb_len);
        let mut sd = vec![f64::NAN; n];
        let mut cmf = vec![f64::NAN; n];
        for t in cfg.bb_len.max(cfg.volume_window).max(1)..n {
            if t >= cfg.bb_len && sma[t].is_finite() {
                let w = &c[t + 1 - cfg.bb_len..=t];
                if w.iter().all(|v| v.is_finite()) {
                    let m = w.iter().sum::<f64>() / cfg.bb_len as f64;
                    sd[t] = (w.iter().map(|v| (v - m).powi(2)).sum::<f64>() / cfg.bb_len as f64).sqrt();
                }
            }
            if t >= cfg.volume_window {
                let mut num = 0.0;
                let mut den = 0.0;
                for u in t + 1 - cfg.volume_window..=t {
                    let (hh, ll, cc, vv) = (
                        panel.high[j][u],
                        panel.low[j][u],
                        panel.close[j][u],
                        panel.volume[j][u],
                    );
                    if hh.is_finite() && ll.is_finite() && cc.is_finite() && vv.is_finite() && hh > ll {
                        num += ((cc - ll) - (hh - cc)) / (hh - ll) * vv;
                        den += vv;
                    }
                }
                if den > 0.0 {
                    cmf[t] = num / den;
                }
            }
        }
        let mut pos = 0i32;
        for t in 1..n {
            if !result.tradable[j][t] {
                continue;
            }
            let mut evt = 0i32;
            // MACD zero-cross event
            if (mode == "macd" || mode == "vote" || mode == "all")
                && macd[t].is_finite() && macd[t - 1].is_finite()
                && (macd[t] > 0.0) != (macd[t - 1] > 0.0)
            {
                evt += if macd[t] > 0.0 { 1 } else { -1 };
            }
            // BB breach reversal event (down breach -> long event)
            if (mode == "bb" || mode == "vote" || mode == "all")
                && sma[t].is_finite() && sd[t].is_finite() && c[t].is_finite() && c[t - 1].is_finite()
            {
                let up = sma[t] + cfg.bb_mult * sd[t];
                let lo = sma[t] - cfg.bb_mult * sd[t];
                if c[t] < lo && c[t - 1] >= lo {
                    evt += 1;
                } else if c[t] > up && c[t - 1] <= up {
                    evt -= 1;
                }
            }
            // CMF sign-change event
            if (mode == "cmf" || mode == "vote" || mode == "all")
                && cmf[t].is_finite() && cmf[t - 1].is_finite()
                && (cmf[t] > 0.0) != (cmf[t - 1] > 0.0)
            {
                evt += if cmf[t] > 0.0 { 1 } else { -1 };
            }
            if evt == 0 {
                continue;
            }
            let dir = evt.signum();
            if pos == 0 {
                result.signal[j][t] = true;
                result.rsi[j][t] = evt as f64;
                pos = dir;
            } else if dir != pos {
                result.exit_now[j][t] = true;
                pos = 0;
            }
        }
    }
    Ok(result)
}

// ---------------- G099 online portfolio selection zoo ------------------------
// strategy_mode: olmar | pamr | corn | eg | up | crp. Weights evolve pathwise;
// lookback = predictor window; resp_scale = step/eta; corr_min = CORN rho.

fn simplex_project(b: &mut [f64]) {
    for v in b.iter_mut() {
        if *v < 0.0 || !v.is_finite() {
            *v = 0.0;
        }
    }
    let s: f64 = b.iter().sum();
    let b_len = b.len();
    if s <= 0.0 {
        for v in b.iter_mut() {
            *v = 1.0 / b_len as f64;
        }
    } else {
        for v in b.iter_mut() {
            *v /= s;
        }
    }
}

pub fn compute_g099_signals(panel: &Panel, cfg: &Config) -> Result<Signals, String> {
    let algo = cfg.strategy_mode.as_str();
    let n = panel.dates.len();
    let ns = panel.symbols.len();
    let w = cfg.lookback.max(2);
    let lam = cfg.resp_scale;
    let mut result = blank_signals(panel, algo);
    for j in 0..ns {
        for t in 0..n {
            result.tradable[j][t] = panel.close[j][t].is_finite();
            result.pool[j][t] = result.tradable[j][t];
        }
    }
    // daily relatives x_t = close[t]/close[t-1]
    let rel = |j: usize, t: usize| -> f64 {
        let (a, b) = (panel.close[j][t - 1], panel.close[j][t]);
        if a.is_finite() && a > 0.0 && b.is_finite() {
            b / a
        } else {
            1.0
        }
    };
    let mut b = vec![1.0 / ns as f64; ns];
    let mut weights = vec![vec![0.0; n]; ns];
    let days: std::collections::BTreeSet<usize> =
        crate::family4::rebalance_days(panel, &cfg.rebalance)?.into_iter().collect();
    for t in 1..n {
        if days.contains(&t) {
            for j in 0..ns {
                weights[j][t] = b[j];
                result.signal[j][t] = b[j] > 1e-6;
                result.rsi[j][t] = b[j];
            }
        }
        let x: Vec<f64> = (0..ns).map(|j| rel(j, t)).collect();
        match algo {
            "crp" => {}
            "up" | "eg" => {
                let dot: f64 = b.iter().zip(&x).map(|(a, v)| a * v).sum();
                if dot > 0.0 {
                    for j in 0..ns {
                        if algo == "up" {
                            b[j] *= x[j];
                        } else {
                            b[j] *= (lam * x[j] / dot).exp();
                        }
                    }
                    simplex_project(&mut b);
                }
            }
            "olmar" | "pamr" | "corn" => {
                // predictor xhat per algo
                let mut xhat = vec![1.0; ns];
                if algo == "olmar" {
                    for j in 0..ns {
                        if t >= w {
                            let m = (t + 1 - w..=t)
                                .map(|u| panel.close[j][u])
                                .filter(|v| v.is_finite())
                                .sum::<f64>()
                                / w as f64;
                            if m > 0.0 && panel.close[j][t].is_finite() {
                                xhat[j] = m / panel.close[j][t];
                            }
                        }
                    }
                } else if algo == "pamr" {
                    let eps = 0.5f64;
                    let dot: f64 = b.iter().zip(&x).map(|(a, v)| a * v).sum();
                    let xbar = x.iter().sum::<f64>() / ns as f64;
                    let dist: f64 = x.iter().map(|v| (v - xbar).powi(2)).sum();
                    if dist > 1e-12 {
                        let loss = (dot - eps).max(0.0);
                        let tau = loss / dist;
                        for j in 0..ns {
                            b[j] += tau * (x[j] - xbar);
                        }
                        simplex_project(&mut b);
                    }
                    continue;
                } else {
                    // corn: xhat = mean next-relatives of similar past windows
                    for j in 0..ns {
                        if t < 2 * w {
                            continue;
                        }
                        let cur: Vec<f64> = (t + 1 - w..=t).map(|u| rel(j, u)).collect();
                        let (mut s, mut cnt) = (0.0, 0usize);
                        for u0 in w..t + 1 - w {
                            let hist: Vec<f64> = (u0 - w + 1..=u0).map(|u| rel(j, u)).collect();
                            if pearson(&hist, &cur) > cfg.corr_min {
                                let nx = rel(j, u0 + 1);
                                s += nx;
                                cnt += 1;
                            }
                        }
                        if cnt > 0 {
                            xhat[j] = s / cnt as f64;
                        }
                    }
                }
                if algo != "pamr" {
                    let xbar = xhat.iter().sum::<f64>() / ns as f64;
                    for j in 0..ns {
                        b[j] += lam * (xhat[j] - xbar);
                    }
                    simplex_project(&mut b);
                }
            }
            _ => return Err(format!("G099 unknown algo {algo:?}")),
        }
    }
    result.trade_weights = Some(weights);
    result.benchmark_returns =
        crate::family5::ew_stock_benchmark(panel, &result.tradable, &std::collections::BTreeSet::new());
    Ok(result)
}
