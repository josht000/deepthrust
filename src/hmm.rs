//! Gaussian hidden Markov regime model — port of `GaussianHiddenMarkovModel`
//! and `build_market_features` from
//! `docs/strategies/markov/markov_regime_models.py`. Baum-Welch EM is refit on
//! a walk-forward schedule; between refits the frozen model filters causally.
//!
//! Emits `markov::Chain` so the shared positions/overlay/study machinery is
//! unchanged: labels are the argmax regime-probability state mapped to
//! Bear/Sideways/Bull by post-fit return ordering, and the signal is the
//! bull-vs-bear probability spread (or stationary-relative bear excess under
//! `markov_signal_mode: bear_rel`). Filtered probabilities (P(S_t | X<=t)) are
//! the baseline; `hmm_probs: predicted` uses P(S_t | X<t).

use crate::config::Config;
use crate::markov::{Chain, MarkovParams, SignalMode, BEAR, BULL, SIDEWAYS, UNLABELED};
use crate::signals::rolling_mean;
use crate::stats::Rng;

pub const HMM_NAME: &str = "hmm_regime";
pub const HMM_TITLE: &str = "Gaussian HMM Regime";

const EPS: f64 = 1e-12;
const LOG_2PI: f64 = 1.8378770664093453;

const F_RET: usize = 0;
const FEATURE_NAMES: [&str; 6] = [
    "daily_log_return",
    "rolling_realized_volatility",
    "normalized_atr",
    "log_volume_change",
    "rolling_return_skewness",
    "moving_average_spread",
];

#[derive(Debug, Clone)]
pub struct HmmParams {
    pub states: usize,
    pub max_iter: usize,
    pub tol: f64,
    pub min_iter: usize,
    pub restarts: usize,
    pub cov_floor: f64,
    pub trans_prior: f64,
    pub refit_every: usize,
    pub train_window: usize,
    pub scale: bool,
    pub filtered: bool,
    pub rv_window: usize,
    pub atr_window: usize,
    pub volchg_window: usize,
    pub skew_window: usize,
    pub ma_fast: usize,
    pub ma_slow: usize,
}

pub fn params(cfg: &Config) -> Result<HmmParams, String> {
    if cfg.hmm_states < 2 {
        return Err(format!("hmm_states must be >= 2, got {}", cfg.hmm_states));
    }
    if cfg.hmm_max_iter == 0 {
        return Err("hmm_max_iter must be positive".to_string());
    }
    if !(cfg.hmm_tol > 0.0 && cfg.hmm_tol.is_finite()) {
        return Err(format!("hmm_tol must be positive, got {}", cfg.hmm_tol));
    }
    if cfg.hmm_min_iter == 0 || cfg.hmm_min_iter > cfg.hmm_max_iter {
        return Err(format!(
            "hmm_min_iter must be in 1..=hmm_max_iter, got {}",
            cfg.hmm_min_iter
        ));
    }
    if cfg.hmm_restarts == 0 {
        return Err("hmm_restarts must be >= 1".to_string());
    }
    if !(cfg.hmm_cov_floor > 0.0 && cfg.hmm_cov_floor.is_finite()) {
        return Err(format!(
            "hmm_cov_floor must be positive, got {}",
            cfg.hmm_cov_floor
        ));
    }
    if !(cfg.hmm_trans_prior >= 0.0 && cfg.hmm_trans_prior.is_finite()) {
        return Err(format!(
            "hmm_trans_prior must be >= 0, got {}",
            cfg.hmm_trans_prior
        ));
    }
    if cfg.hmm_refit_every == 0 {
        return Err("hmm_refit_every must be >= 1".to_string());
    }
    if cfg.hmm_train_window != 0 && cfg.hmm_train_window < cfg.hmm_states.max(20) {
        return Err(format!(
            "hmm_train_window must be 0 (expanding) or >= {}, got {}",
            cfg.hmm_states.max(20),
            cfg.hmm_train_window
        ));
    }
    if !matches!(cfg.hmm_probs.as_str(), "filtered" | "predicted") {
        return Err(format!(
            "hmm_probs must be filtered|predicted, got {:?}",
            cfg.hmm_probs
        ));
    }
    for (name, w) in [
        ("hmm_rv_window", cfg.hmm_rv_window),
        ("hmm_atr_window", cfg.hmm_atr_window),
        ("hmm_volchg_window", cfg.hmm_volchg_window),
        ("hmm_skew_window", cfg.hmm_skew_window),
        ("hmm_ma_fast", cfg.hmm_ma_fast),
        ("hmm_ma_slow", cfg.hmm_ma_slow),
    ] {
        if w == 0 {
            return Err(format!("{name} must be >= 1"));
        }
    }
    if cfg.hmm_ma_fast >= cfg.hmm_ma_slow {
        return Err(format!(
            "hmm_ma_fast ({}) must be < hmm_ma_slow ({})",
            cfg.hmm_ma_fast, cfg.hmm_ma_slow
        ));
    }
    Ok(HmmParams {
        states: cfg.hmm_states,
        max_iter: cfg.hmm_max_iter,
        tol: cfg.hmm_tol,
        min_iter: cfg.hmm_min_iter,
        restarts: cfg.hmm_restarts,
        cov_floor: cfg.hmm_cov_floor,
        trans_prior: cfg.hmm_trans_prior,
        refit_every: cfg.hmm_refit_every,
        train_window: cfg.hmm_train_window,
        scale: cfg.hmm_scale,
        filtered: cfg.hmm_probs == "filtered",
        rv_window: cfg.hmm_rv_window,
        atr_window: cfg.hmm_atr_window,
        volchg_window: cfg.hmm_volchg_window,
        skew_window: cfg.hmm_skew_window,
        ma_fast: cfg.hmm_ma_fast,
        ma_slow: cfg.hmm_ma_slow,
    })
}

// ---------------- features ----------------

/// Compact feature matrix: `rows[i]` is the panel row of `values[i]`; warm-up
/// and non-finite rows are dropped (mid-series gaps bridge, matching the
/// observable-chain convention).
pub struct FeatureSet {
    pub rows: Vec<usize>,
    pub values: Vec<Vec<f64>>,
    /// Column names kept, in FEATURE_NAMES order (unusable columns dropped).
    pub names: Vec<&'static str>,
}

/// Rolling sample stdev (ddof=1), NaN until a full finite window — same
/// convention as `markov::rolling_std`, kept local so hmm.rs is self-contained.
fn rolling_std(x: &[f64], w: usize) -> Vec<f64> {
    let mut out = vec![f64::NAN; x.len()];
    if w == 0 {
        return out;
    }
    for t in (w - 1)..x.len() {
        let win = &x[t + 1 - w..=t];
        if win.iter().all(|v| v.is_finite()) {
            let mean = win.iter().sum::<f64>() / w as f64;
            let var = win.iter().map(|v| (v - mean).powi(2)).sum::<f64>()
                / (w - 1).max(1) as f64;
            out[t] = var.max(0.0).sqrt();
        }
    }
    out
}

/// `build_market_features` port. ATR is the simple rolling mean of true range
/// (not Wilder). A feature column that is entirely non-finite (e.g. a symbol
/// with no usable volume) is dropped rather than poisoning every row.
fn market_features(
    close: &[f64],
    high: &[f64],
    low: &[f64],
    volume: &[f64],
    hp: &HmmParams,
) -> Result<FeatureSet, String> {
    let n = close.len();
    if high.len() != n || low.len() != n || volume.len() != n {
        return Err("OHLCV series must have identical lengths".into());
    }
    let mut ret = vec![f64::NAN; n];
    for t in 1..n {
        let (a, b) = (close[t - 1], close[t]);
        if a.is_finite() && b.is_finite() && a > 0.0 && b > 0.0 {
            ret[t] = (b / a).ln();
        }
    }
    let rv = rolling_std(&ret, hp.rv_window);

    let mut tr = vec![f64::NAN; n];
    for t in 0..n {
        let (h, l) = (high[t], low[t]);
        if h.is_finite() && l.is_finite() && h >= l {
            let mut m = h - l;
            if t > 0 && close[t - 1].is_finite() {
                m = m.max((h - close[t - 1]).abs()).max((l - close[t - 1]).abs());
            }
            tr[t] = m;
        }
    }
    let atr_mean = rolling_mean(&tr, hp.atr_window);
    let mut atr = vec![f64::NAN; n];
    for t in 0..n {
        if atr_mean[t].is_finite() && close[t].is_finite() && close[t] > 0.0 {
            atr[t] = atr_mean[t] / close[t];
        }
    }

    let mut volchg = vec![f64::NAN; n];
    for t in hp.volchg_window..n {
        let (a, b) = (volume[t - hp.volchg_window], volume[t]);
        if a.is_finite() && b.is_finite() && a > 0.0 && b > 0.0 {
            volchg[t] = (b / a).ln();
        }
    }

    let mut skew = vec![f64::NAN; n];
    for t in (hp.skew_window - 1)..n {
        let win = &ret[t + 1 - hp.skew_window..=t];
        if win.iter().all(|v| v.is_finite()) {
            let mean = win.iter().sum::<f64>() / hp.skew_window as f64;
            let m2 =
                win.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / hp.skew_window as f64;
            let m3 =
                win.iter().map(|v| (v - mean).powi(3)).sum::<f64>() / hp.skew_window as f64;
            skew[t] = m3 / (m2.sqrt().powi(3) + EPS);
        }
    }

    let fast = rolling_mean(close, hp.ma_fast);
    let slow = rolling_mean(close, hp.ma_slow);
    let mut ma = vec![f64::NAN; n];
    for t in 0..n {
        if fast[t].is_finite() && slow[t].is_finite() && slow[t] != 0.0 {
            ma[t] = fast[t] / slow[t] - 1.0;
        }
    }

    let cols = [ret, rv, atr, volchg, skew, ma];
    let mut keep = [true; FEATURE_NAMES.len()];
    for (c, col) in cols.iter().enumerate() {
        if !col.iter().any(|v| v.is_finite()) {
            keep[c] = false;
        }
    }
    let mut rows = Vec::new();
    let mut values = Vec::new();
    for t in 0..n {
        if keep
            .iter()
            .enumerate()
            .all(|(c, &k)| !k || cols[c][t].is_finite())
        {
            rows.push(t);
            values.push(
                keep.iter()
                    .enumerate()
                    .filter(|(_, &k)| k)
                    .map(|(c, _)| cols[c][t])
                    .collect(),
            );
        }
    }
    if rows.is_empty() {
        return Err("no valid feature rows after warm-up".into());
    }
    let names = keep
        .iter()
        .enumerate()
        .filter(|(_, &k)| k)
        .map(|(c, _)| FEATURE_NAMES[c])
        .collect();
    Ok(FeatureSet { rows, values, names })
}

// ---------------- model ----------------

#[derive(Debug, Clone)]
pub struct GaussianHmm {
    pub n_states: usize,
    pub means: Vec<Vec<f64>>,
    pub variances: Vec<Vec<f64>>,
    pub transition: Vec<Vec<f64>>,
    pub initial: Vec<f64>,
    pub loglik: f64,
    pub n_iter: usize,
    pub converged: bool,
    /// Filtered posterior over the final training observation.
    pub last_filtered: Vec<f64>,
}

fn logsumexp(v: &[f64]) -> f64 {
    let m = v.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    if !m.is_finite() {
        return m;
    }
    m + v.iter().map(|x| (x - m).exp()).sum::<f64>().ln()
}

/// Diagonal-covariance Gaussian log densities: returns [T][K].
fn log_gaussian_diag(x: &[Vec<f64>], means: &[Vec<f64>], vars: &[Vec<f64>]) -> Vec<Vec<f64>> {
    let d = x[0].len() as f64;
    let log_det: Vec<f64> = vars
        .iter()
        .map(|v| v.iter().map(|s| s.ln()).sum::<f64>())
        .collect();
    x.iter()
        .map(|row| {
            (0..means.len())
                .map(|k| {
                    let maha: f64 = row
                        .iter()
                        .enumerate()
                        .map(|(c, &v)| (v - means[k][c]).powi(2) / vars[k][c])
                        .sum();
                    -0.5 * (d * LOG_2PI + log_det[k] + maha)
                })
                .collect()
        })
        .collect()
}

fn emission_row(x: &[f64], m: &GaussianHmm) -> Vec<f64> {
    let d = x.len() as f64;
    (0..m.n_states)
        .map(|k| {
            let log_det: f64 = m.variances[k].iter().map(|v| v.ln()).sum();
            let maha: f64 = x
                .iter()
                .enumerate()
                .map(|(c, &v)| (v - m.means[k][c]).powi(2) / m.variances[k][c])
                .sum();
            -0.5 * (d * LOG_2PI + log_det + maha)
        })
        .collect()
}

/// Log-domain scaled forward-backward (port of `_forward_backward`):
/// (filtered, smoothed, xi_sum, loglik).
fn forward_backward(
    log_b: &[Vec<f64>],
    transition: &[Vec<f64>],
    initial: &[f64],
) -> (Vec<Vec<f64>>, Vec<Vec<f64>>, Vec<Vec<f64>>, f64) {
    let t_count = log_b.len();
    let k = initial.len();
    let log_p: Vec<Vec<f64>> = transition
        .iter()
        .map(|r| r.iter().map(|&v| v.max(EPS).ln()).collect())
        .collect();
    let log_pi: Vec<f64> = initial.iter().map(|&v| v.max(EPS).ln()).collect();

    let mut log_alpha = vec![vec![0.0; k]; t_count];
    let mut log_scales = vec![0.0; t_count];
    let raw0: Vec<f64> = (0..k).map(|i| log_pi[i] + log_b[0][i]).collect();
    log_scales[0] = logsumexp(&raw0);
    for i in 0..k {
        log_alpha[0][i] = raw0[i] - log_scales[0];
    }
    for t in 1..t_count {
        let mut raw = vec![0.0; k];
        for j in 0..k {
            let terms: Vec<f64> = (0..k)
                .map(|i| log_alpha[t - 1][i] + log_p[i][j])
                .collect();
            raw[j] = log_b[t][j] + logsumexp(&terms);
        }
        log_scales[t] = logsumexp(&raw);
        for j in 0..k {
            log_alpha[t][j] = raw[j] - log_scales[t];
        }
    }
    let loglik: f64 = log_scales.iter().sum();

    let mut log_beta = vec![vec![0.0; k]; t_count];
    for t in (0..t_count - 1).rev() {
        for i in 0..k {
            let terms: Vec<f64> = (0..k)
                .map(|j| log_p[i][j] + log_b[t + 1][j] + log_beta[t + 1][j])
                .collect();
            log_beta[t][i] = logsumexp(&terms) - log_scales[t + 1];
        }
    }

    let filtered: Vec<Vec<f64>> = log_alpha
        .iter()
        .map(|r| r.iter().map(|v| v.exp()).collect())
        .collect();
    let mut smoothed = vec![vec![0.0; k]; t_count];
    for t in 0..t_count {
        let lg: Vec<f64> = (0..k).map(|i| log_alpha[t][i] + log_beta[t][i]).collect();
        let lse = logsumexp(&lg);
        for i in 0..k {
            smoothed[t][i] = (lg[i] - lse).exp();
        }
    }

    let mut xi_sum = vec![vec![0.0; k]; k];
    for t in 0..t_count - 1 {
        let mut log_xi = vec![vec![0.0; k]; k];
        let mut flat = Vec::with_capacity(k * k);
        for i in 0..k {
            for j in 0..k {
                let v = log_alpha[t][i] + log_p[i][j] + log_b[t + 1][j] + log_beta[t + 1][j]
                    - log_scales[t + 1];
                log_xi[i][j] = v;
                flat.push(v);
            }
        }
        let lse = logsumexp(&flat);
        for i in 0..k {
            for j in 0..k {
                xi_sum[i][j] += (log_xi[i][j] - lse).exp();
            }
        }
    }
    (filtered, smoothed, xi_sum, loglik)
}

fn normalize(v: &mut [f64]) {
    let s: f64 = v.iter().sum();
    if s > 0.0 && s.is_finite() {
        for x in v.iter_mut() {
            *x /= s;
        }
    } else {
        let u = 1.0 / v.len() as f64;
        for x in v.iter_mut() {
            *x = u;
        }
    }
}

/// One EM restart: init means at `states` distinct random observations,
/// variances at the global column variance, uniform transition/initial.
fn em_once(x: &[Vec<f64>], hp: &HmmParams, rng: &mut Rng) -> (GaussianHmm, Vec<Vec<f64>>) {
    let t = x.len();
    let k = hp.states;
    let d = x[0].len();
    let mut idx: Vec<usize> = (0..t).collect();
    for i in 0..k {
        let j = i + rng.below(t - i);
        idx.swap(i, j);
    }
    let mut means: Vec<Vec<f64>> = (0..k).map(|s| x[idx[s]].clone()).collect();
    let gvar: Vec<f64> = (0..d)
        .map(|c| {
            let mean = x.iter().map(|r| r[c]).sum::<f64>() / t as f64;
            let var = x.iter().map(|r| (r[c] - mean).powi(2)).sum::<f64>() / t as f64;
            var.max(hp.cov_floor)
        })
        .collect();
    let mut variances = vec![gvar; k];
    let mut transition = vec![vec![1.0 / k as f64; k]; k];
    let mut initial = vec![1.0 / k as f64; k];

    let mut prev = f64::NEG_INFINITY;
    let mut loglik = f64::NEG_INFINITY;
    let mut n_iter = 0;
    let mut converged = false;
    let mut filtered = vec![vec![1.0 / k as f64; k]];
    for it in 1..=hp.max_iter {
        let log_b = log_gaussian_diag(x, &means, &variances);
        let (f, smoothed, xi_sum, ll) = forward_backward(&log_b, &transition, &initial);
        filtered = f;

        initial = smoothed[0].iter().map(|v| v + EPS).collect();
        normalize(&mut initial);
        for i in 0..k {
            for j in 0..k {
                transition[i][j] = xi_sum[i][j] + hp.trans_prior;
            }
            normalize(&mut transition[i]);
        }
        let occupancy: Vec<f64> = (0..k)
            .map(|s| smoothed.iter().map(|r| r[s]).sum::<f64>())
            .collect();
        for s in 0..k {
            let denom = occupancy[s].max(EPS);
            for c in 0..d {
                means[s][c] =
                    smoothed.iter().zip(x).map(|(w, r)| w[s] * r[c]).sum::<f64>() / denom;
            }
            for c in 0..d {
                let var = smoothed
                    .iter()
                    .zip(x)
                    .map(|(w, r)| w[s] * (r[c] - means[s][c]).powi(2))
                    .sum::<f64>()
                    / denom;
                variances[s][c] = var.max(hp.cov_floor);
            }
        }

        loglik = ll;
        n_iter = it;
        if it >= hp.min_iter && prev.is_finite() && (ll - prev).abs() <= hp.tol * (1.0 + prev.abs())
        {
            converged = true;
            break;
        }
        prev = ll;
    }
    (
        GaussianHmm {
            n_states: k,
            means,
            variances,
            transition,
            initial,
            loglik,
            n_iter,
            converged,
            last_filtered: filtered.last().cloned().unwrap_or_else(|| vec![1.0 / k as f64; k]),
        },
        filtered,
    )
}

/// Best-of-`restarts` Baum-Welch fit; returns the winner plus its final
/// E-step filtered matrix (used for state ordering).
fn fit_hmm(
    x: &[Vec<f64>],
    hp: &HmmParams,
    rng: &mut Rng,
) -> Result<(GaussianHmm, Vec<Vec<f64>>), String> {
    if x.len() < hp.states {
        return Err(format!(
            "hmm needs >= {} observations, got {}",
            hp.states,
            x.len()
        ));
    }
    if x.iter().flatten().any(|v| !v.is_finite()) {
        return Err("hmm fit data contains NaN or infinity".into());
    }
    let mut best: Option<(GaussianHmm, Vec<Vec<f64>>)> = None;
    for _ in 0..hp.restarts {
        let (m, f) = em_once(x, hp, rng);
        if best.as_ref().map_or(true, |(b, _)| m.loglik > b.loglik) {
            best = Some((m, f));
        }
    }
    best.ok_or_else(|| "hmm fit produced no restarts".into())
}

// ---------------- inference helpers ----------------

/// One causal filter step: predicted = prev @ P, filtered ∝ predicted * emission.
fn filter_step(prev: &[f64], m: &GaussianHmm, x: &[f64]) -> (Vec<f64>, Vec<f64>) {
    let k = m.n_states;
    let log_b = emission_row(x, m);
    let mut pred = vec![0.0; k];
    for j in 0..k {
        for i in 0..k {
            pred[j] += prev[i] * m.transition[i][j];
        }
    }
    let lj: Vec<f64> = (0..k).map(|j| pred[j].max(EPS).ln() + log_b[j]).collect();
    let lse = logsumexp(&lj);
    (pred, lj.iter().map(|v| (v - lse).exp()).collect())
}

fn vec_mat(v: &[f64], m: &[Vec<f64>]) -> Vec<f64> {
    let k = m.len();
    let mut out = vec![0.0; k];
    for i in 0..k {
        for j in 0..k {
            out[j] += v[i] * m[i][j];
        }
    }
    out
}

fn mat_mul(a: &[Vec<f64>], b: &[Vec<f64>]) -> Vec<Vec<f64>> {
    let k = a.len();
    let mut out = vec![vec![0.0; k]; k];
    for i in 0..k {
        for j in 0..k {
            out[i][j] = (0..k).map(|c| a[i][c] * b[c][j]).sum();
        }
    }
    out
}

/// q @ P^n (n = 0 returns q unchanged).
fn advance(q: &[f64], p: &[Vec<f64>], n: usize) -> Vec<f64> {
    if n == 0 {
        return q.to_vec();
    }
    let mut m = p.to_vec();
    let mut out = vec_mat(q, &m);
    for _ in 1..n {
        m = mat_mul(&m, p);
        out = vec_mat(q, &m);
    }
    out
}

/// Stationary distribution via power iteration from the uniform distribution.
fn stationary(p: &[Vec<f64>]) -> Vec<f64> {
    let k = p.len();
    let mut pi = vec![1.0 / k as f64; k];
    for _ in 0..128 {
        let next = vec_mat(&pi, p);
        let s: f64 = next.iter().sum();
        if !(s > 0.0) {
            return vec![1.0 / k as f64; k];
        }
        pi = next.iter().map(|v| v / s).collect();
    }
    pi
}

// ---------------- state ordering ----------------

/// Map hidden states to Bear/Sideways/Bull buckets by weighted mean return
/// over the training window (port of `summarize_states` +
/// `order_states_by_return`): lowest mean return -> BEAR, highest -> BULL,
/// everything between -> SIDEWAYS.
fn order_states(filtered: &[Vec<f64>], rets: &[f64], k: usize) -> Vec<i8> {
    let mut mean = vec![0.0; k];
    for s in 0..k {
        let mass: f64 = filtered.iter().map(|r| r[s]).sum();
        mean[s] = filtered
            .iter()
            .zip(rets)
            .map(|(w, &r)| w[s] * r)
            .sum::<f64>()
            / mass.max(EPS);
    }
    let mut ord: Vec<usize> = (0..k).collect();
    ord.sort_by(|&a, &b| mean[a].partial_cmp(&mean[b]).unwrap_or(std::cmp::Ordering::Equal));
    let mut labels = vec![SIDEWAYS as i8; k];
    labels[ord[0]] = BEAR as i8;
    labels[ord[k - 1]] = BULL as i8;
    labels
}

/// Train-window standardization; frozen between refits like the model itself.
struct Scaler {
    mean: Vec<f64>,
    std: Vec<f64>,
}

impl Scaler {
    fn fit(x: &[Vec<f64>]) -> Scaler {
        let (t, d) = (x.len(), x[0].len());
        let mut mean = vec![0.0; d];
        let mut std = vec![0.0; d];
        for c in 0..d {
            mean[c] = x.iter().map(|r| r[c]).sum::<f64>() / t as f64;
            let var = x.iter().map(|r| (r[c] - mean[c]).powi(2)).sum::<f64>()
                / (t - 1).max(1) as f64;
            std[c] = var.sqrt().max(EPS);
        }
        Scaler { mean, std }
    }

    fn apply(&self, row: &mut [f64]) {
        for (c, v) in row.iter_mut().enumerate() {
            *v = (*v - self.mean[c]) / self.std[c];
        }
    }
}

/// Per-refit diagnostics written to `hmm_fits.csv`.
#[derive(Debug, Clone)]
pub struct HmmFit {
    /// Panel row at which this fit went live.
    pub row: usize,
    pub loglik: f64,
    pub n_iter: usize,
    pub converged: bool,
    /// Effective observations per state (filtered-probability mass).
    pub occupancy: Vec<f64>,
}

struct Fitted {
    model: GaussianHmm,
    scaler: Option<Scaler>,
    /// HMM state index -> BEAR/SIDEWAYS/BULL.
    label: Vec<i8>,
    /// Stationary bucket masses (computed lazily for bear_rel).
    stat_bear: f64,
    last: Vec<f64>,
}

/// Walk-forward HMM chain on one OHLCV series. Refits at `mp.min_train` then
/// every `hp.refit_every` feature rows on an expanding or `hp.train_window`
/// rolling window; between refits the frozen model filters each new bar.
pub fn hmm_chain(
    close: &[f64],
    high: &[f64],
    low: &[f64],
    volume: &[f64],
    hp: &HmmParams,
    mp: &MarkovParams,
    seed: u64,
) -> Result<Chain, String> {
    let rows = close.len();
    let feats = market_features(close, high, low, volume, hp)?;
    let m = feats.values.len();
    let mut labels = vec![UNLABELED; rows];
    let mut signal = vec![f64::NAN; rows];
    let mut fits = Vec::new();
    if m <= mp.min_train {
        return Ok(Chain {
            labels,
            signal,
            fits,
        });
    }
    // Daily log return column (unscaled) drives the post-fit state ordering;
    // fall back to zeros if it was ever dropped.
    let ret_col = feats.names.iter().position(|&n| n == FEATURE_NAMES[F_RET]);
    let rets: Vec<f64> = match ret_col {
        Some(c) => feats.values.iter().map(|r| r[c]).collect(),
        None => vec![0.0; m],
    };
    let mut rng = Rng::new(seed);
    let mut fitted: Option<Fitted> = None;
    let mut next_refit = mp.min_train;
    for j in mp.min_train..m {
        if fitted.is_none() || j >= next_refit {
            let t0 = if hp.train_window > 0 {
                j.saturating_sub(hp.train_window)
            } else {
                0
            };
            let mut x: Vec<Vec<f64>> = feats.values[t0..j].to_vec();
            let scaler = hp.scale.then(|| {
                let s = Scaler::fit(&x);
                for r in &mut x {
                    s.apply(r);
                }
                s
            });
            let (model, f_train) = fit_hmm(&x, hp, &mut rng)?;
            let label = order_states(&f_train, &rets[t0..j], model.n_states);
            let occupancy: Vec<f64> = (0..model.n_states)
                .map(|s| f_train.iter().map(|r| r[s]).sum::<f64>())
                .collect();
            let stat_bear = if mp.signal_mode == SignalMode::BearRel {
                stationary(&model.transition)
                    .iter()
                    .zip(&label)
                    .filter(|(_, &l)| l == BEAR as i8)
                    .map(|(&p, _)| p)
                    .sum()
            } else {
                0.0
            };
            fits.push(HmmFit {
                row: feats.rows[j],
                loglik: model.loglik,
                n_iter: model.n_iter,
                converged: model.converged,
                occupancy,
            });
            fitted = Some(Fitted {
                last: model.last_filtered.clone(),
                model,
                scaler,
                label,
                stat_bear,
            });
            next_refit = j + hp.refit_every;
        }
        let f = fitted.as_mut().unwrap();
        let mut x = feats.values[j].clone();
        if let Some(s) = &f.scaler {
            s.apply(&mut x);
        }
        let (pred, filt) = filter_step(&f.last, &f.model, &x);
        let (base, extra) = if hp.filtered {
            (&filt, mp.horizon)
        } else {
            (&pred, mp.horizon - 1)
        };
        let probs = advance(base, &f.model.transition, extra);
        let (mut pb, mut pr) = (0.0, 0.0);
        let mut argmax = 0;
        for s in 0..f.model.n_states {
            match f.label[s] as usize {
                BULL => pb += probs[s],
                BEAR => pr += probs[s],
                _ => {}
            }
            if probs[s] > probs[argmax] {
                argmax = s;
            }
        }
        let row = feats.rows[j];
        signal[row] = match mp.signal_mode {
            SignalMode::Spread => pb - pr,
            SignalMode::BearRel => f.stat_bear - pr,
        };
        labels[row] = f.label[argmax];
        f.last = filt;
    }
    Ok(Chain {
        labels,
        signal,
        fits,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hp() -> HmmParams {
        HmmParams {
            states: 3,
            max_iter: 200,
            tol: 1e-8,
            min_iter: 5,
            restarts: 3,
            cov_floor: 1e-6,
            trans_prior: 1.0,
            refit_every: 10,
            train_window: 0,
            scale: true,
            filtered: true,
            rv_window: 5,
            atr_window: 5,
            volchg_window: 3,
            skew_window: 5,
            ma_fast: 3,
            ma_slow: 8,
        }
    }

    fn mp() -> MarkovParams {
        MarkovParams {
            window: 2,
            threshold: 0.05,
            min_train: 30,
            enter: 0.0,
            exit: 0.0,
            hysteresis: false,
            log_ret: false,
            warmup_pass: true,
            vol_window: 0,
            vol_mult: 0.0,
            commit: 0,
            decay: 1.0,
            shrink: 0.0,
            horizon: 1,
            signal_mode: SignalMode::Spread,
            breadth_threshold: 0.05,
        }
    }

    /// Two-regime synthetic: state A ~ N(-2, 0.5), state B ~ N(+2, 0.5) on a
    /// single feature, persistent transitions.
    fn two_regime_1d(n: usize, seed: u64) -> Vec<Vec<f64>> {
        let mut rng = Rng::new(seed);
        let p = [[0.97, 0.03], [0.03, 0.97]];
        let mut s = 0usize;
        let mut x = Vec::with_capacity(n);
        for _ in 0..n {
            let u = rng.next_f64();
            s = if u < p[s][0] { s } else { 1 - s };
            // Box-Muller around the state mean.
            let (u1, u2) = (rng.next_f64().max(1e-9), rng.next_f64());
            let z = (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos();
            x.push(vec![if s == 0 { -2.0 } else { 2.0 } + 0.5 * z]);
        }
        x
    }

    #[test]
    fn forward_backward_loglik_matches_brute_force() {
        // T=4, K=2: brute-force the 2^3 transition paths.
        let log_b = vec![
            vec![-0.3, -1.1],
            vec![-0.8, -0.4],
            vec![-1.5, -0.1],
            vec![-0.2, -0.9],
        ];
        let p = vec![vec![0.9, 0.1], vec![0.2, 0.8]];
        let pi = vec![0.6, 0.4];
        let (_, _, _, ll) = forward_backward(&log_b, &p, &pi);
        let b: Vec<Vec<f64>> = log_b.iter().map(|r| r.iter().map(|v| v.exp()).collect()).collect();
        let mut total = 0.0;
        for s1 in 0..2 {
            for s2 in 0..2 {
                for s3 in 0..2 {
                    let mut joint = pi[s1] * b[0][s1];
                    joint *= p[s1][s2] * b[1][s2];
                    joint *= p[s2][s3] * b[2][s3];
                    // t=3 sums over final state via total below.
                    total += joint * (0..2).map(|s4| p[s3][s4] * b[3][s4]).sum::<f64>();
                }
            }
        }
        assert!((ll - total.ln()).abs() < 1e-10, "{ll} vs {}", total.ln());
    }

    #[test]
    fn em_recovers_separated_states() {
        let x = two_regime_1d(300, 7);
        let p = HmmParams {
            states: 2,
            scale: false,
            ..hp()
        };
        let mut rng = Rng::new(1);
        let (m, _) = fit_hmm(&x, &p, &mut rng).unwrap();
        let mut means: Vec<f64> = m.means.iter().map(|r| r[0]).collect();
        means.sort_by(|a, b| a.partial_cmp(b).unwrap());
        assert!(means[0] < -1.0 && means[1] > 1.0, "means: {means:?}");
        // High persistence in both directions.
        let d0 = m.transition[0][0].max(m.transition[0][1]);
        let d1 = m.transition[1][0].max(m.transition[1][1]);
        assert!(d0 > 0.8 && d1 > 0.8, "diag: {d0} {d1}");
        assert!(m.loglik.is_finite());
    }

    #[test]
    fn em_is_deterministic_for_seed() {
        let x = two_regime_1d(120, 11);
        let p = HmmParams {
            states: 2,
            scale: false,
            ..hp()
        };
        let (a, _) = fit_hmm(&x, &p, &mut Rng::new(3)).unwrap();
        let (b, _) = fit_hmm(&x, &p, &mut Rng::new(3)).unwrap();
        assert_eq!(a.loglik.to_bits(), b.loglik.to_bits());
        assert_eq!(a.means, b.means);
    }

    #[test]
    fn signal_never_uses_future_observations() {
        // Alternating-volatility synthetic close; signals at earlier rows must
        // not move when a violent tail regime is appended.
        let n = 220;
        let mut rng = Rng::new(5);
        let mut close = vec![100.0; n];
        for i in 1..n {
            let vol = if (i / 30) % 2 == 0 { 0.002 } else { 0.015 };
            let drift = if (i / 60) % 3 == 2 { -0.004 } else { 0.003 };
            let z = 2.0 * rng.next_f64() - 1.0;
            close[i] = close[i - 1] * (1.0 + drift + vol * z * 3.0);
        }
        let high: Vec<f64> = close.iter().map(|c| c * 1.005).collect();
        let low: Vec<f64> = close.iter().map(|c| c * 0.995).collect();
        let vol = vec![1e6; n];
        let p = hp();
        let m = mp();
        let a = hmm_chain(&close, &high, &low, &vol, &p, &m, 9).unwrap();
        let mut tail = close.clone();
        let mut th = high.clone();
        let mut tl = low.clone();
        let mut tv = vol.clone();
        let mut rng = Rng::new(6);
        for _ in 0..40 {
            let last = *tail.last().unwrap();
            let z = 2.0 * rng.next_f64() - 1.0;
            let c = last * (1.0 - 0.01 + 0.01 * z);
            tail.push(c);
            th.push(c * 1.01);
            tl.push(c * 0.99);
            tv.push(1e6);
        }
        let b = hmm_chain(&tail, &th, &tl, &tv, &p, &m, 9).unwrap();
        for t in 0..n {
            let (x, y) = (a.signal[t], b.signal[t]);
            assert!(
                (x.is_nan() && y.is_nan()) || x == y,
                "row {t} changed: {x} -> {y}"
            );
        }
    }

    #[test]
    fn order_states_maps_extremes() {
        let filtered = vec![
            vec![0.8, 0.1, 0.1],
            vec![0.7, 0.2, 0.1],
            vec![0.1, 0.7, 0.2],
            vec![0.1, 0.2, 0.7],
            vec![0.1, 0.1, 0.8],
        ];
        let rets = vec![-0.02, -0.01, 0.001, 0.01, 0.02];
        let l = order_states(&filtered, &rets, 3);
        assert_eq!(l[0], BEAR as i8);
        assert_eq!(l[1], SIDEWAYS as i8);
        assert_eq!(l[2], BULL as i8);
    }

    #[test]
    fn features_drop_warmup_and_unusable_volume() {
        let n = 80;
        let close: Vec<f64> = (0..n).map(|i| 100.0 + i as f64 * 0.1).collect();
        let high: Vec<f64> = close.iter().map(|c| c * 1.01).collect();
        let low: Vec<f64> = close.iter().map(|c| c * 0.99).collect();
        // All-zero volume -> the volume feature column is dropped entirely.
        let vol = vec![0.0; n];
        let p = hp();
        let f = market_features(&close, &high, &low, &vol, &p).unwrap();
        assert!(!f.values.is_empty());
        assert_eq!(f.values[0].len(), 5);
        assert!(f.rows[0] >= p.ma_slow - 1);
        // Positive volume restores the sixth column.
        let vol = vec![1e6; n];
        let f = market_features(&close, &high, &low, &vol, &p).unwrap();
        assert_eq!(f.values[0].len(), 6);
    }
}
