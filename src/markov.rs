//! Observable Markov regime chain (Roan / @RohOnChain): regime labels from a
//! rolling-return rule, a 3x3 MLE transition matrix re-estimated walk-forward
//! with no lookahead, the stationary regime mix, and the resulting risk layer.
//!
//! Port of `markov-hedge-fund-method/scripts/markov_regime.py`, structured per
//! `docs/strategies/markov/Overview.md` so the hidden-state models (Gaussian
//! HMM, MS-AR, MS-GARCH, TVTP) can plug into the same filtered/predicted
//! regime-probability interface when ported.

use crate::config::Config;
use crate::data::{Bars, Panel};
use crate::engine::Trade;
use crate::signals::Signals;
use chrono::NaiveDate;

pub const MARKOV_NAME: &str = "markov_regime";
pub const MARKOV_TITLE: &str = "Markov Regime Chain";

/// State indices are fixed: 0 = Bear, 1 = Sideways, 2 = Bull, matching the
/// reference implementation. `UNLABELED` marks rows without a full window or
/// with missing prices.
pub const BEAR: usize = 0;
pub const SIDEWAYS: usize = 1;
pub const BULL: usize = 2;
pub const UNLABELED: i8 = -1;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SignalMode {
    /// s(t) = (P^h)[s,Bull] - (P^h)[s,Bear]; positive is bullish.
    Spread,
    /// s(t) = pi_bear - P(next=Bear|s); negative when tomorrow is more bearish
    /// than the asset's own stationary baseline.
    BearRel,
}

#[derive(Debug, Clone)]
pub struct MarkovParams {
    pub window: usize,
    pub threshold: f64,
    pub min_train: usize,
    pub enter: f64,
    pub exit: f64,
    pub hysteresis: bool,
    pub log_ret: bool,
    pub warmup_pass: bool,
    pub vol_window: usize,
    pub vol_mult: f64,
    pub commit: usize,
    pub decay: f64,
    pub shrink: f64,
    pub horizon: usize,
    pub signal_mode: SignalMode,
    pub breadth_threshold: f64,
}

pub fn params(cfg: &Config) -> Result<MarkovParams, String> {
    if cfg.markov_window < 2 {
        return Err(format!("markov_window must be >= 2, got {}", cfg.markov_window));
    }
    if !(cfg.markov_threshold > 0.0 && cfg.markov_threshold.is_finite()) {
        return Err(format!(
            "markov_threshold must be positive, got {}",
            cfg.markov_threshold
        ));
    }
    if cfg.markov_min_train < 2 {
        return Err(format!(
            "markov_min_train must be >= 2, got {}",
            cfg.markov_min_train
        ));
    }
    if cfg.markov_enter < cfg.markov_exit {
        return Err(format!(
            "markov_enter ({}) must be >= markov_exit ({})",
            cfg.markov_enter, cfg.markov_exit
        ));
    }
    if !matches!(cfg.markov_regime.as_str(), "strict" | "hysteresis") {
        return Err(format!(
            "markov_regime must be strict|hysteresis, got {:?}",
            cfg.markov_regime
        ));
    }
    if !matches!(cfg.markov_warmup.as_str(), "pass" | "block") {
        return Err(format!(
            "markov_warmup must be pass|block, got {:?}",
            cfg.markov_warmup
        ));
    }
    if !matches!(cfg.markov_blend_form.as_str(), "tilt" | "mix" | "conf") {
        return Err(format!(
            "markov_blend_form must be tilt|mix|conf, got {:?}",
            cfg.markov_blend_form
        ));
    }
    if !(0.0..=1.0).contains(&cfg.markov_blend_lambda) {
        return Err(format!(
            "markov_blend_lambda must be in [0,1], got {}",
            cfg.markov_blend_lambda
        ));
    }
    if !(cfg.markov_size_min <= cfg.markov_size_max) {
        return Err(format!(
            "markov_size_min ({}) must be <= markov_size_max ({})",
            cfg.markov_size_min, cfg.markov_size_max
        ));
    }
    if !matches!(
        cfg.markov_position.as_str(),
        "long_flat" | "long_short" | "conviction"
    ) {
        return Err(format!(
            "markov_position must be long_flat|long_short|conviction, got {:?}",
            cfg.markov_position
        ));
    }
    for token in cfg.markov_mode.split(',') {
        let token = token.trim();
        if token.is_empty() || matches!(token, "off" | "none") {
            continue;
        }
        if !matches!(token, "gate" | "exit" | "size") {
            return Err(format!(
                "markov_mode tokens must be gate|exit|size, got {token:?}"
            ));
        }
    }
    if cfg.markov_vol_window == 1 {
        return Err("markov_vol_window must be 0 (off) or >= 2".into());
    }
    if cfg.markov_vol_window > 0 && !(cfg.markov_vol_mult > 0.0 && cfg.markov_vol_mult.is_finite()) {
        return Err(format!(
            "markov_vol_window={} requires markov_vol_mult > 0, got {}",
            cfg.markov_vol_window, cfg.markov_vol_mult
        ));
    }
    if !(cfg.markov_decay > 0.0 && cfg.markov_decay <= 1.0) {
        return Err(format!(
            "markov_decay must be in (0,1], got {}",
            cfg.markov_decay
        ));
    }
    if !(cfg.markov_shrink >= 0.0 && cfg.markov_shrink.is_finite()) {
        return Err(format!(
            "markov_shrink must be >= 0, got {}",
            cfg.markov_shrink
        ));
    }
    if cfg.markov_horizon == 0 || cfg.markov_horizon > 64 {
        return Err(format!(
            "markov_horizon must be in 1..=64, got {}",
            cfg.markov_horizon
        ));
    }
    let signal_mode = match cfg.markov_signal_mode.as_str() {
        "spread" => SignalMode::Spread,
        "bear_rel" => SignalMode::BearRel,
        other => {
            return Err(format!(
                "markov_signal_mode must be spread|bear_rel, got {other:?}"
            ))
        }
    };
    if !(cfg.markov_breadth_threshold > 0.0 && cfg.markov_breadth_threshold.is_finite()) {
        return Err(format!(
            "markov_breadth_threshold must be positive, got {}",
            cfg.markov_breadth_threshold
        ));
    }
    if !matches!(cfg.markov_model.as_str(), "chain" | "hmm") {
        return Err(format!(
            "markov_model must be chain|hmm, got {:?}",
            cfg.markov_model
        ));
    }
    Ok(MarkovParams {
        window: cfg.markov_window,
        threshold: cfg.markov_threshold,
        min_train: cfg.markov_min_train,
        enter: cfg.markov_enter,
        exit: cfg.markov_exit,
        hysteresis: cfg.markov_regime == "hysteresis",
        log_ret: cfg.markov_logret,
        warmup_pass: cfg.markov_warmup == "pass",
        vol_window: cfg.markov_vol_window,
        vol_mult: cfg.markov_vol_mult,
        commit: cfg.markov_commit,
        decay: cfg.markov_decay,
        shrink: cfg.markov_shrink,
        horizon: cfg.markov_horizon,
        signal_mode,
        breadth_threshold: cfg.markov_breadth_threshold,
    })
}

/// Rolling sample stdev (pandas `std(ddof=1)`, `min_periods=window`).
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

/// Per-bar labeling thresholds: fixed `p.threshold`, or vol-scaled
/// `vol_mult * stdev(daily returns, vol_window) * sqrt(label window)` — days
/// without a full vol window stay unlabeled (no fallback to the fixed cutoff).
fn daily_thresholds(close: &[f64], p: &MarkovParams) -> Vec<f64> {
    if p.vol_window == 0 || p.vol_mult <= 0.0 {
        return vec![p.threshold; close.len()];
    }
    let n = close.len();
    let mut rets = vec![f64::NAN; n];
    for i in 1..n {
        let (a, b) = (close[i - 1], close[i]);
        if a.is_finite() && b.is_finite() && a > 0.0 && b > 0.0 {
            rets[i] = if p.log_ret { (b / a).ln() } else { b / a - 1.0 };
        }
    }
    let sd = rolling_std(&rets, p.vol_window);
    let scale = (p.window as f64).sqrt() * p.vol_mult;
    sd.iter().map(|s| s * scale).collect()
}

/// Label each bar from the trailing `window`-bar return, returning the
/// (row, label) pairs in order. Rows without a full window or with missing /
/// non-positive prices produce no label — matching `pct_change().dropna()` in
/// the reference, so a mid-series gap bridges rather than poisoning the chain.
fn label_regimes(close: &[f64], p: &MarkovParams) -> Vec<(usize, i8)> {
    let thr = daily_thresholds(close, p);
    let mut out = Vec::with_capacity(close.len().saturating_sub(p.window));
    for t in p.window..close.len() {
        let (c0, c1) = (close[t - p.window], close[t]);
        if !c0.is_finite() || !c1.is_finite() || c0 <= 0.0 || c1 <= 0.0 {
            continue;
        }
        let th = thr[t];
        if !th.is_finite() || th <= 0.0 {
            continue;
        }
        let r = if p.log_ret { (c1 / c0).ln() } else { c1 / c0 - 1.0 };
        let label = if r > th {
            BULL as i8
        } else if r < -th {
            BEAR as i8
        } else {
            SIDEWAYS as i8
        };
        out.push((t, label));
    }
    out
}

/// Debounce a label stream: a new state is adopted only after `commit`
/// consecutive raw labels (commit-at-confirmation — nothing looks ahead).
/// `commit <= 1` passes through unchanged.
fn commit_stream(labeled: &[(usize, i8)], commit: usize) -> Vec<(usize, i8)> {
    if commit <= 1 || labeled.is_empty() {
        return labeled.to_vec();
    }
    let mut out = Vec::with_capacity(labeled.len());
    let mut cur = labeled[0].1;
    let (mut cand, mut run) = (cur, 0usize);
    for &(t, l) in labeled {
        if l == cur {
            cand = l;
            run = 0;
        } else {
            if l == cand {
                run += 1;
            } else {
                cand = l;
                run = 1;
            }
            if run >= commit {
                cur = l;
                cand = l;
                run = 0;
            }
        }
        out.push((t, cur));
    }
    out
}

/// Walk-forward signal on the compacted label stream. At step t the weighted
/// MLE counts cover transitions strictly inside labels[..t]: `decay` down-
/// weights old transitions each step, and `shrink` pseudo-counts pull each row
/// toward `prior` (Dirichlet shrinkage — the pooled-universe matrix for
/// per-symbol chains). Signal mode: `spread` = (P^h)[s,Bull] - (P^h)[s,Bear];
/// `bear_rel` = pi_bear - P(next=Bear|s) vs the chain's stationary baseline.
/// Identical to the reference implementation when decay=1, shrink=0, h=1.
fn compact_signal(labels: &[i8], p: &MarkovParams, prior: Option<&[[f64; 3]; 3]>) -> Vec<f64> {
    let n = labels.len();
    let mut out = vec![f64::NAN; n];
    if n < p.min_train + 1 {
        return out;
    }
    let mut counts = [[0.0f64; 3]; 3];
    for i in 0..p.min_train - 1 {
        counts[labels[i] as usize][labels[i + 1] as usize] += 1.0;
    }
    for t in p.min_train..n {
        let mut m = [[0.0f64; 3]; 3];
        for i in 0..3 {
            let mut rs = 0.0;
            for c in 0..3 {
                m[i][c] = counts[i][c] + p.shrink * prior.map_or(0.0, |pr| pr[i][c]);
                rs += m[i][c];
            }
            if rs > 0.0 {
                for c in 0..3 {
                    m[i][c] /= rs;
                }
            }
        }
        let row = labels[t] as usize;
        out[t] = match p.signal_mode {
            SignalMode::Spread => {
                let mrow = if p.horizon > 1 {
                    matrix_power(&m, p.horizon)[row]
                } else {
                    m[row]
                };
                mrow[BULL] - mrow[BEAR]
            }
            SignalMode::BearRel => {
                let pi = stationary_distribution(&m);
                pi[BEAR] - m[row][BEAR]
            }
        };
        for row in counts.iter_mut() {
            for cell in row.iter_mut() {
                *cell *= p.decay;
            }
        }
        counts[labels[t - 1] as usize][labels[t] as usize] += 1.0;
    }
    out
}

/// Full-history MLE transition matrix; zero rows for unobserved states.
fn transition_matrix(labels: &[i8]) -> [[f64; 3]; 3] {
    let mut counts = [[0.0f64; 3]; 3];
    for w in labels.windows(2) {
        counts[w[0] as usize][w[1] as usize] += 1.0;
    }
    let mut p = [[0.0f64; 3]; 3];
    for i in 0..3 {
        let sum: f64 = counts[i].iter().sum();
        if sum > 0.0 {
            for j in 0..3 {
                p[i][j] = counts[i][j] / sum;
            }
        }
    }
    p
}

/// Left eigenvector of P at eigenvalue 1 via power iteration from the uniform
/// distribution — the same limit the Pine implementation reaches by squaring.
pub fn stationary_distribution(p: &[[f64; 3]; 3]) -> [f64; 3] {
    let mut pi = [1.0 / 3.0; 3];
    for _ in 0..64 {
        let mut next = [0.0f64; 3];
        for i in 0..3 {
            for j in 0..3 {
                next[j] += pi[i] * p[i][j];
            }
        }
        let sum: f64 = next.iter().sum();
        if !(sum > 0.0) {
            return [1.0 / 3.0; 3];
        }
        pi = [next[0] / sum, next[1] / sum, next[2] / sum];
    }
    pi
}

fn mat_mul(a: &[[f64; 3]; 3], b: &[[f64; 3]; 3]) -> [[f64; 3]; 3] {
    let mut out = [[0.0f64; 3]; 3];
    for i in 0..3 {
        for j in 0..3 {
            out[i][j] = a[i][0] * b[0][j] + a[i][1] * b[1][j] + a[i][2] * b[2][j];
        }
    }
    out
}

/// Chapman-Kolmogorov: the n-step-ahead transition matrix is P^n.
pub fn matrix_power(p: &[[f64; 3]; 3], n: usize) -> [[f64; 3]; 3] {
    let mut m = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
    for _ in 0..n {
        m = mat_mul(&m, p);
    }
    m
}

/// Per-bar regime chain on one close series, panel-row aligned.
pub struct Chain {
    /// Regime label per row (`UNLABELED` during warm-up / across gaps).
    pub labels: Vec<i8>,
    /// Walk-forward s(t) per row; NaN before `min_train` valid observations.
    pub signal: Vec<f64>,
    /// Per-refit EM diagnostics; populated only by the `hmm` model.
    pub fits: Vec<crate::hmm::HmmFit>,
}

fn finish_chain(
    rows: usize,
    labeled: &[(usize, i8)],
    p: &MarkovParams,
    prior: Option<&[[f64; 3]; 3]>,
) -> Chain {
    let compact: Vec<i8> = labeled.iter().map(|(_, l)| *l).collect();
    let sig = compact_signal(&compact, p, prior);
    let mut labels = vec![UNLABELED; rows];
    let mut signal = vec![f64::NAN; rows];
    for (k, (t, l)) in labeled.iter().enumerate() {
        labels[*t] = *l;
        signal[*t] = sig[k];
    }
    Chain {
        labels,
        signal,
        fits: Vec::new(),
    }
}

fn build_chain(close: &[f64], p: &MarkovParams, prior: Option<&[[f64; 3]; 3]>) -> Chain {
    finish_chain(
        close.len(),
        &commit_stream(&label_regimes(close, p), p.commit),
        p,
        prior,
    )
}

pub fn chain(close: &[f64], p: &MarkovParams) -> Chain {
    build_chain(close, p, None)
}

/// Model dispatch for `markov_model`: the observable chain consumes close
/// only; `hmm` fits Gaussian emissions on the OHLCV feature set.
fn model_chain(
    close: &[f64],
    high: &[f64],
    low: &[f64],
    volume: &[f64],
    cfg: &Config,
    p: &MarkovParams,
    prior: Option<&[[f64; 3]; 3]>,
    seed: u64,
) -> Result<Chain, String> {
    if cfg.markov_model == "hmm" {
        crate::hmm::hmm_chain(close, high, low, volume, &crate::hmm::params(cfg)?, p, seed)
    } else {
        Ok(build_chain(close, p, prior))
    }
}

/// Pooled-universe transition probabilities — the Dirichlet prior for
/// `markov_shrink`, estimated from every symbol's committed label stream.
fn pooled_prior(panel: &Panel, p: &MarkovParams) -> [[f64; 3]; 3] {
    let mut counts = [[0.0f64; 3]; 3];
    for j in 0..panel.symbols.len() {
        let labeled = commit_stream(&label_regimes(&panel.close[j], p), p.commit);
        for w in labeled.windows(2) {
            counts[w[0].1 as usize][w[1].1 as usize] += 1.0;
        }
    }
    let mut prior = [[0.0f64; 3]; 3];
    for i in 0..3 {
        let s: f64 = counts[i].iter().sum();
        if s > 0.0 {
            for c in 0..3 {
                prior[i][c] = counts[i][c] / s;
            }
        } else {
            prior[i] = [1.0 / 3.0; 3];
        }
    }
    prior
}

/// `markov_source: breadth` — per-symbol committed labels define the daily
/// Bear fraction of the universe; the breadth *change* over `window` days is
/// labelled (rising bear share = Bear market) and chained like any series.
fn breadth_chain(panel: &Panel, p: &MarkovParams) -> Chain {
    let n = panel.dates.len();
    let (mut bears, mut tot) = (vec![0usize; n], vec![0usize; n]);
    for j in 0..panel.symbols.len() {
        for &(t, l) in &commit_stream(&label_regimes(&panel.close[j], p), p.commit) {
            tot[t] += 1;
            if l as usize == BEAR {
                bears[t] += 1;
            }
        }
    }
    let mut labeled = Vec::new();
    for t in p.window..n {
        if tot[t] == 0 || tot[t - p.window] == 0 {
            continue;
        }
        let d = bears[t] as f64 / tot[t] as f64
            - bears[t - p.window] as f64 / tot[t - p.window] as f64;
        let label = if d > p.breadth_threshold {
            BEAR as i8
        } else if d < -p.breadth_threshold {
            BULL as i8
        } else {
            SIDEWAYS as i8
        };
        labeled.push((t, label));
    }
    finish_chain(n, &commit_stream(&labeled, p.commit), p, None)
}

/// Regime position state per row: +1 risk-on, -1 risk-off, 0 uncommitted
/// (strict mode only, when s sits inside the enter/exit band or is neutral).
/// `warmup_pass` seeds the state before the signal is estimable.
fn positions(signal: &[f64], p: &MarkovParams) -> Vec<i8> {
    let init: i8 = if p.warmup_pass { 1 } else { -1 };
    let mut cur = init;
    signal
        .iter()
        .map(|&s| {
            if p.hysteresis {
                if s.is_finite() {
                    if s > p.enter {
                        cur = 1;
                    } else if s < p.exit {
                        cur = -1;
                    }
                }
                cur
            } else if !s.is_finite() {
                init
            } else if s > p.enter {
                1
            } else if s < p.exit {
                -1
            } else {
                0
            }
        })
        .collect()
}

/// Benchmark series re-indexed onto the panel grid: for each panel date, the
/// last value on or before it (NaN before the first benchmark bar).
fn aligned_field(dates: &[NaiveDate], bars: &Bars, field: &[f64]) -> Vec<f64> {
    let mut out = vec![f64::NAN; dates.len()];
    let mut bi = 0usize;
    for (t, d) in dates.iter().enumerate() {
        while bi + 1 < bars.dates.len() && bars.dates[bi + 1] <= *d {
            bi += 1;
        }
        if bars.dates.first().is_some_and(|first| first <= d) {
            out[t] = field[bi];
        }
    }
    out
}

/// Benchmark closes re-indexed onto the panel grid: for each panel date, the
/// last benchmark close on or before it (NaN before the first benchmark bar).
pub fn aligned_close(dates: &[NaiveDate], bars: &Bars) -> Vec<f64> {
    aligned_field(dates, bars, &bars.close)
}

/// `aligned_close` for every OHLCV field — the benchmark input for
/// `markov_source: benchmark` under `markov_model: hmm`.
fn aligned_bars(dates: &[NaiveDate], bars: &Bars) -> (Vec<f64>, Vec<f64>, Vec<f64>, Vec<f64>, Vec<f64>) {
    (
        aligned_field(dates, bars, &bars.open),
        aligned_field(dates, bars, &bars.high),
        aligned_field(dates, bars, &bars.low),
        aligned_field(dates, bars, &bars.close),
        aligned_field(dates, bars, &bars.volume),
    )
}

/// Synthetic OHLCV path for `markov_source: universe`: equal-weight daily
/// ratios per bar field, compounded (the close leg is identical to the former
/// `universe_close`). Volume compounds the mean volume ratio and holds flat
/// on days no symbol reports it.
fn universe_bars(panel: &Panel) -> (Vec<f64>, Vec<f64>, Vec<f64>, Vec<f64>, Vec<f64>) {
    let n = panel.dates.len();
    let (mut o, mut h, mut l, mut c, mut v) = (
        vec![1.0f64; n],
        vec![1.0; n],
        vec![1.0; n],
        vec![1.0; n],
        vec![1.0; n],
    );
    for t in 1..n {
        let (mut ro, mut rh, mut rl, mut rc, mut rv) = (0.0, 0.0, 0.0, 0.0, 0.0);
        let (mut no, mut nh, mut nl, mut nc, mut nv) = (0usize, 0, 0, 0, 0);
        for j in 0..panel.symbols.len() {
            let (pc, cl) = (panel.close[j][t - 1], panel.close[j][t]);
            if pc.is_finite() && cl.is_finite() && pc > 0.0 && cl > 0.0 {
                rc += cl / pc;
                nc += 1;
                let op = panel.open[j][t];
                if op.is_finite() && op > 0.0 {
                    ro += op / pc;
                    no += 1;
                }
                let (hi, lo) = (panel.high[j][t], panel.low[j][t]);
                if hi.is_finite() && hi > 0.0 {
                    rh += hi / cl;
                    nh += 1;
                }
                if lo.is_finite() && lo > 0.0 {
                    rl += lo / cl;
                    nl += 1;
                }
            }
            let (pv, vo) = (panel.volume[j][t - 1], panel.volume[j][t]);
            if pv.is_finite() && vo.is_finite() && pv > 0.0 && vo > 0.0 {
                rv += vo / pv;
                nv += 1;
            }
        }
        c[t] = c[t - 1] * if nc > 0 { rc / nc as f64 } else { 1.0 };
        o[t] = c[t - 1] * if no > 0 { ro / no as f64 } else { 1.0 };
        h[t] = c[t] * if nh > 0 { rh / nh as f64 } else { 1.0 };
        l[t] = c[t] * if nl > 0 { rl / nl as f64 } else { 1.0 };
        v[t] = v[t - 1] * if nv > 0 { rv / nv as f64 } else { 1.0 };
    }
    (o, h, l, c, v)
}

/// `markov_mode=size` blend of the strategy weight w with the chain signal s.
fn blend(w: f64, s: f64, cfg: &Config) -> f64 {
    if !w.is_finite() || !s.is_finite() {
        return w;
    }
    let (k, lo, hi) = (
        cfg.markov_size_k,
        cfg.markov_size_min,
        cfg.markov_size_max,
    );
    match cfg.markov_blend_form.as_str() {
        "mix" => ((1.0 - cfg.markov_blend_lambda) * w + cfg.markov_blend_lambda * s)
            .clamp(-hi, hi),
        "conf" => (w * (1.0 + k * (s - cfg.markov_blend_theta).max(0.0))).clamp(lo.min(0.0), hi),
        _ => (w * (1.0 + k * s)).clamp(lo, hi),
    }
}

pub struct OverlayReport {
    pub warnings: Vec<String>,
    pub summary_csv: Option<String>,
    pub hmm_csv: Option<String>,
}

/// Apply the regime layer to a computed `Signals`: `gate` vetoes entries and
/// adds while the chain is risk-off, `exit` forces `exit_now`, `size` rescales
/// `trade_weights` via `markov_blend_form`. Modes compose (`gate,exit`).
/// `tradable`/`pool` are never touched — they drive the universe baselines.
pub fn apply_overlay(
    sig: &mut Signals,
    panel: &Panel,
    benchmark: Option<&Bars>,
    cfg: &Config,
) -> Result<OverlayReport, String> {
    let p = params(cfg)?;
    let n = panel.dates.len();
    let mut warnings = Vec::new();
    let hmm = cfg.markov_model == "hmm";

    if hmm {
        for (name, set) in [
            ("markov_window", cfg.markov_window != 20),
            ("markov_threshold", cfg.markov_threshold != 0.05),
            ("markov_logret", cfg.markov_logret),
            ("markov_vol_window", cfg.markov_vol_window != 0),
            ("markov_vol_mult", cfg.markov_vol_mult != 0.0),
            ("markov_commit", cfg.markov_commit != 0),
            ("markov_decay", cfg.markov_decay != 1.0),
            ("markov_shrink", cfg.markov_shrink != 0.0),
        ] {
            if set {
                warnings.push(format!("{name} ignored under markov_model=hmm"));
            }
        }
    }

    let prior = (!hmm && p.shrink > 0.0).then(|| pooled_prior(panel, &p));
    let shared: Option<Chain> = if !cfg.markov_symbol.is_empty() {
        let j = crate::family7::symbol_index(panel, &cfg.markov_symbol)?;
        Some(model_chain(
            &panel.close[j],
            &panel.high[j],
            &panel.low[j],
            &panel.volume[j],
            cfg,
            &p,
            prior.as_ref(),
            cfg.seed + j as u64,
        )?)
    } else {
        match cfg.markov_source.as_str() {
            "symbol" | "" => None,
            "benchmark" => {
                if prior.is_some() {
                    warnings.push("markov_shrink ignored for benchmark source".into());
                }
                let (_o, h, l, c, v) = aligned_bars(
                    &panel.dates,
                    benchmark.ok_or_else(|| {
                        "markov_source=benchmark needs a loaded benchmark".to_string()
                    })?,
                );
                Some(model_chain(&c, &h, &l, &v, cfg, &p, None, cfg.seed ^ 0xBEA4)?)
            }
            "universe" => {
                if prior.is_some() {
                    warnings.push("markov_shrink ignored for universe source".into());
                }
                let (_o, h, l, c, v) = universe_bars(panel);
                Some(model_chain(&c, &h, &l, &v, cfg, &p, None, cfg.seed ^ 0xA115)?)
            }
            "breadth" => {
                if hmm {
                    return Err(
                        "markov_model=hmm does not support markov_source=breadth".into()
                    );
                }
                if panel.symbols.len() < 2 {
                    warnings.push(
                        "markov_source=breadth degenerates on a single-symbol universe".into(),
                    );
                }
                if prior.is_some() {
                    warnings.push("markov_shrink ignored for breadth source".into());
                }
                Some(breadth_chain(panel, &p))
            }
            other => {
                return Err(format!(
                    "markov_source must be symbol|benchmark|universe|breadth, got {other:?}"
                ))
            }
        }
    };
    let mut per_symbol: Vec<Option<Chain>> = Vec::new();
    if shared.is_none() {
        for j in 0..panel.symbols.len() {
            if panel.close[j].iter().any(|v| v.is_finite()) {
                per_symbol.push(Some(model_chain(
                    &panel.close[j],
                    &panel.high[j],
                    &panel.low[j],
                    &panel.volume[j],
                    cfg,
                    &p,
                    prior.as_ref(),
                    cfg.seed + j as u64,
                )?));
            } else {
                per_symbol.push(None);
            }
        }
    }

    let (gate, exit, size) = cfg
        .markov_mode
        .split(',')
        .map(str::trim)
        .fold((false, false, false), |(g, e, s), token| match token {
            "gate" => (true, e, s),
            "exit" => (g, true, s),
            "size" => (g, e, true),
            _ => (g, e, s),
        });

    if size && sig.trade_weights.is_none() {
        warnings.push(format!(
            "{} markov_mode=size has no trade_weights; nothing scaled",
            cfg.strategy
        ));
    }

    let mut summary = String::from(
        "symbol,obs,bear_pct,side_pct,bull_pct,persist_bear,persist_side,persist_bull,stat_bear,stat_side,stat_bull,riskoff_pct,flips\n",
    );
    let mut write_summary = |name: &str, c: &Chain| {
        let compact: Vec<i8> = c.labels.iter().copied().filter(|&l| l >= 0).collect();
        if compact.is_empty() {
            return;
        }
        let m = compact.len() as f64;
        let (mut bear, mut bull, mut flips) = (0usize, 0usize, 0usize);
        for (i, &l) in compact.iter().enumerate() {
            match l as usize {
                BEAR => bear += 1,
                BULL => bull += 1,
                _ => {}
            }
            if i > 0 && l != compact[i - 1] {
                flips += 1;
            }
        }
        let tm = transition_matrix(&compact);
        let stat = stationary_distribution(&tm);
        let pos = positions(&c.signal, &p);
        let riskoff = pos.iter().filter(|&&s| s < 0).count() as f64 / n as f64;
        summary.push_str(&format!(
            "{name},{},{:.4},{:.4},{:.4},{:.4},{:.4},{:.4},{:.4},{:.4},{:.4},{:.4},{}\n",
            compact.len(),
            bear as f64 / m,
            1.0 - bear as f64 / m - bull as f64 / m,
            bull as f64 / m,
            tm[BEAR][BEAR],
            tm[SIDEWAYS][SIDEWAYS],
            tm[BULL][BULL],
            stat[BEAR],
            stat[SIDEWAYS],
            stat[BULL],
            riskoff,
            flips,
        ));
    };

    if let Some(c) = &shared {
        write_summary("MARKET", c);
    }

    for j in 0..panel.symbols.len() {
        let c = match shared.as_ref().or(per_symbol.get(j).and_then(|c| c.as_ref())) {
            Some(c) => c,
            None => continue,
        };
        if shared.is_none() {
            write_summary(&panel.symbols[j], c);
        }
        if !gate && !exit && !(size && sig.trade_weights.is_some()) {
            continue;
        }
        let pos = positions(&c.signal, &p);
        for t in 0..n {
            let on = pos[t] >= 0;
            if gate {
                sig.signal[j][t] &= on;
                if let Some(add) = sig.add_signal.as_mut() {
                    add[j][t] &= on;
                }
            }
            if exit && !on {
                sig.exit_now[j][t] = true;
            }
            if size {
                if let Some(w) = sig.trade_weights.as_mut() {
                    w[j][t] = blend(w[j][t], c.signal[t], cfg);
                }
            }
        }
    }

    let hmm_csv = hmm.then(|| {
        let mut csv = String::from("symbol,row,loglik,n_iter,converged,occupancy\n");
        let emit = |name: &str, c: &Chain, csv: &mut String| {
            for f in &c.fits {
                let occ = f
                    .occupancy
                    .iter()
                    .map(|v| format!("{v:.2}"))
                    .collect::<Vec<_>>()
                    .join(";");
                csv.push_str(&format!(
                    "{name},{},{:.4},{},{},{}\n",
                    f.row, f.loglik, f.n_iter, f.converged, occ
                ));
            }
        };
        if let Some(c) = &shared {
            emit("MARKET", c, &mut csv);
        }
        for (j, c) in per_symbol.iter().enumerate() {
            if let Some(c) = c {
                emit(&panel.symbols[j], c, &mut csv);
            }
        }
        csv
    });

    Ok(OverlayReport {
        warnings,
        summary_csv: Some(summary),
        hmm_csv,
    })
}

// ---------------- markov_regime standalone strategy ----------------

/// Standalone study: each symbol trades its own walk-forward chain. Weights are
/// NaN until the signal is estimable, so no position is held during warm-up.
pub fn compute_markov_signals(panel: &Panel, cfg: &Config) -> Result<Signals, String> {
    let p = params(cfg)?;
    let n = panel.dates.len();
    let mut result = crate::family4::blank_signals(panel, "markov_signal");
    let mut weights = vec![vec![f64::NAN; n]; panel.symbols.len()];
    let hmm = cfg.markov_model == "hmm";
    let prior = (!hmm && p.shrink > 0.0).then(|| pooled_prior(panel, &p));
    for j in 0..panel.symbols.len() {
        let c = model_chain(
            &panel.close[j],
            &panel.high[j],
            &panel.low[j],
            &panel.volume[j],
            cfg,
            &p,
            prior.as_ref(),
            cfg.seed + j as u64,
        )?;
        let pos = positions(&c.signal, &p);
        for t in 0..n {
            result.tradable[j][t] = panel.close[j][t].is_finite();
            result.pool[j][t] = result.tradable[j][t];
            result.rsi[j][t] = c.signal[t];
            let w = if !c.signal[t].is_finite() {
                f64::NAN
            } else {
                match cfg.markov_position.as_str() {
                    "long_short" => pos[t] as f64,
                    "conviction" => c.signal[t].clamp(-cfg.markov_cap, cfg.markov_cap),
                    _ => pos[t].max(0) as f64,
                }
            };
            weights[j][t] = w;
            result.signal[j][t] = w.is_finite() && w.abs() > 1e-9;
        }
    }
    result.trade_weights = Some(weights);
    result.benchmark_returns = crate::family4::benchmark_returns(panel, &cfg.risk_symbol);
    Ok(result)
}

pub fn markov_trades(
    panel: &Panel,
    sig: &Signals,
    cfg: &Config,
    fill: &str,
) -> Result<Vec<Trade>, String> {
    let weights = sig
        .trade_weights
        .as_ref()
        .ok_or_else(|| "markov_regime requires target weights".to_string())?;
    crate::family7::switch_trades(panel, weights, sig, cfg, fill)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn params() -> MarkovParams {
        MarkovParams {
            window: 2,
            threshold: 0.05,
            min_train: 4,
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

    #[test]
    fn labels_match_threshold_and_skip_gaps() {
        let close = [100.0, 100.0, 110.0, f64::NAN, 100.0, 90.0, 100.0];
        let labeled = label_regimes(&close, &params());
        // t=2: 110/100=+10% bull; t=3: NaN close skipped; t=4: 100/110=-9% bear;
        // t=5: close[3] NaN skipped; t=6: 100/100=0% sideways (gap bridges).
        assert_eq!(
            labeled,
            vec![
                (2, BULL as i8),
                (4, BEAR as i8),
                (6, SIDEWAYS as i8)
            ]
        );
    }

    #[test]
    fn incremental_signal_matches_batch_mle() {
        let p = params();
        let close: Vec<f64> = (0..60)
            .map(|i| 100.0 * (1.0 + 0.001 * ((i % 7) as f64 - 3.0)).powi(i as i32 % 3))
            .collect();
        let c = chain(&close, &p);
        let compact: Vec<i8> = c.labels.iter().copied().filter(|&l| l >= 0).collect();
        // At the last labeled row, recompute the matrix from scratch over the
        // same prefix the walk-forward used and compare the signal row.
        let t = compact.len() - 1;
        let mut counts = [[0.0f64; 3]; 3];
        for i in 0..t {
            counts[compact[i] as usize][compact[i + 1] as usize] += 1.0;
        }
        let row = compact[t] as usize;
        let sum: f64 = counts[row].iter().sum();
        let expect = if sum > 0.0 {
            (counts[row][BULL] - counts[row][BEAR]) / sum
        } else {
            0.0
        };
        let last_row = c
            .labels
            .iter()
            .rposition(|&l| l >= 0)
            .expect("labeled row");
        assert_eq!(c.signal[last_row], expect);
    }

    #[test]
    fn signal_never_uses_future_labels() {
        let p = params();
        let mut close: Vec<f64> = (0..40)
            .map(|i| 100.0 + (i as f64 * 0.7).sin() * 8.0 + i as f64 * 0.1)
            .collect();
        let a = chain(&close, &p);
        // Append a violent bear regime; signals at earlier rows must not move.
        for i in 0..20 {
            close.push(close.last().unwrap() * if i % 3 == 0 { 0.97 } else { 1.001 });
        }
        let b = chain(&close, &p);
        for t in 0..40 {
            let (x, y) = (a.signal[t], b.signal[t]);
            assert!(
                (x.is_nan() && y.is_nan()) || x == y,
                "row {t} changed: {x} -> {y}"
            );
        }
    }

    #[test]
    fn transition_matrix_rows_are_stochastic_or_zero() {
        let labels = [0, 0, 1, 1, 2, 2, 1, 0];
        let p = transition_matrix(&labels);
        for row in p {
            let sum: f64 = row.iter().sum();
            assert!(sum == 0.0 || (sum - 1.0).abs() < 1e-9);
        }
        assert_eq!(p[2][2], 0.5); // bull splits 2->2 / 2->1
    }

    #[test]
    fn stationary_sums_to_one() {
        let labels = [0, 0, 1, 2, 2, 1, 0, 1, 2, 1];
        let pi = stationary_distribution(&transition_matrix(&labels));
        assert!((pi.iter().sum::<f64>() - 1.0).abs() < 1e-9);
        let p5 = matrix_power(&transition_matrix(&labels), 5);
        for row in p5 {
            assert!((row.iter().sum::<f64>() - 1.0).abs() < 1e-9);
        }
    }

    #[test]
    fn hysteresis_holds_state_between_crossings() {
        let p = MarkovParams {
            hysteresis: true,
            enter: 0.2,
            exit: -0.2,
            ..params()
        };
        let s = [f64::NAN, 0.05, -0.05, 0.3, 0.1, -0.05, -0.3, 0.0, 0.4];
        let pos = positions(&s, &p);
        assert_eq!(pos, vec![1, 1, 1, 1, 1, 1, -1, -1, 1]);
    }

    #[test]
    fn strict_positions_split_on_zero() {
        let p = params();
        let s = [f64::NAN, 0.1, -0.1, 0.0];
        let pos = positions(&s, &p);
        assert_eq!(pos, vec![1, 1, -1, 0]);
    }

    #[test]
    fn commit_debounces_state_changes() {
        let labeled: Vec<(usize, i8)> = [
            (0, 1),
            (1, 1),
            (2, 0),
            (3, 0),
            (4, 0),
            (5, 1),
            (6, 0),
            (7, 0),
        ]
        .into_iter()
        .collect();
        let committed = commit_stream(&labeled, 2);
        // Bear commits at t=3 (2nd consecutive); single bull at t=5 ignored;
        // bears at t=6..7 recommit.
        assert_eq!(
            committed.iter().map(|&(_, l)| l).collect::<Vec<_>>(),
            vec![1, 1, 1, 0, 0, 0, 0, 0]
        );
        assert_eq!(commit_stream(&labeled, 1), labeled);
    }

    #[test]
    fn shrink_pulls_rows_toward_prior() {
        let p = MarkovParams {
            shrink: 1e9,
            ..params()
        };
        let prior = [[0.9, 0.05, 0.05], [0.4, 0.2, 0.4], [0.1, 0.1, 0.8]];
        let labels = [0, 1, 2, 0, 1, 2, 0, 1, 2, 0, 1, 2];
        let s = compact_signal(&labels, &p, Some(&prior));
        // With shrink >> counts every row is ~prior: signal at a bear-labeled
        // step ~= prior[BEAR][BULL] - prior[BEAR][BEAR].
        let last = *s.last().unwrap();
        let expect = prior[labels[labels.len() - 1] as usize][BULL]
            - prior[labels[labels.len() - 1] as usize][BEAR];
        assert!((last - expect).abs() < 1e-6);
    }

    #[test]
    fn vol_scaled_threshold_responds_to_local_vol() {
        // Alternating +/-8% closes: fixed 5% window-2 threshold labels every
        // bar; a high vol-scaled threshold (mult*sigma*sqrt(2) > 8%) labels none.
        let close: Vec<f64> = (0..40)
            .map(|i| 100.0 * if i % 2 == 0 { 1.08f64 } else { 1.0 }.powi(i as i32 / 2 + 1))
            .collect();
        let fixed = label_regimes(&close, &params());
        let vol = label_regimes(
            &close,
            &MarkovParams {
                vol_window: 10,
                vol_mult: 3.0,
                ..params()
            },
        );
        assert!(fixed.len() > vol.len());
        assert!(vol.iter().all(|&(_, l)| l == SIDEWAYS as i8));
    }

    #[test]
    fn decay_downweights_old_transitions() {
        // Early alternating bull<->bear (mixed bull row), then a bull run.
        // Decay forgets the old bull->bear mass: s(last bull) rises vs equal-weight.
        let mut labels: Vec<i8> = (0..30).map(|i| (i % 2 == 0) as i8 * 2).collect();
        labels.extend([2i8; 30]);
        let fresh = compact_signal(
            &labels,
            &MarkovParams {
                min_train: 5,
                decay: 0.9,
                ..params()
            },
            None,
        );
        let flat = compact_signal(
            &labels,
            &MarkovParams {
                min_train: 5,
                ..params()
            },
            None,
        );
        assert!(fresh.last().unwrap() > flat.last().unwrap());
    }

    #[test]
    fn horizon_uses_matrix_power() {
        let p = MarkovParams {
            horizon: 2,
            min_train: 3,
            ..params()
        };
        let labels = [2, 2, 1, 0, 2, 2, 2, 1, 0, 2, 2, 2];
        let s = compact_signal(&labels, &p, None);
        let t = labels.len() - 1;
        let mut counts = [[0.0f64; 3]; 3];
        for i in 0..t - 1 {
            counts[labels[i] as usize][labels[i + 1] as usize] += 1.0;
        }
        let mut m = [[0.0f64; 3]; 3];
        for i in 0..3 {
            let rs: f64 = counts[i].iter().sum();
            if rs > 0.0 {
                for c in 0..3 {
                    m[i][c] = counts[i][c] / rs;
                }
            }
        }
        let row = matrix_power(&m, 2)[labels[t] as usize];
        assert_eq!(s[t], row[BULL] - row[BEAR]);
    }

    #[test]
    fn breadth_chain_tracks_bear_fraction_change() {
        let n = 40;
        let dates: Vec<NaiveDate> = (0..n)
            .map(|i| NaiveDate::from_ymd_opt(2020, 1, 1).unwrap() + chrono::Duration::days(i as i64))
            .collect();
        // Two symbols trending up (bull), one collapsing (bear) after day 20.
        let up: Vec<f64> = (0..n).map(|i| 100.0 + i as f64).collect();
        let mut dn = up.clone();
        for t in 20..n {
            dn[t] = dn[t - 1] * 0.9;
        }
        let panel = Panel {
            dates,
            symbols: vec!["A".into(), "B".into(), "C".into()],
            open: vec![up.clone(), up.clone(), dn.clone()],
            high: vec![up.clone(), up.clone(), dn.clone()],
            low: vec![up.clone(), up.clone(), dn.clone()],
            close: vec![up.clone(), up.clone(), dn.clone()],
            volume: vec![vec![1.0; n]; 3],
        };
        let p = MarkovParams {
            window: 5,
            breadth_threshold: 0.02,
            min_train: 3,
            ..params()
        };
        let c = breadth_chain(&panel, &p);
        // C is bear-labelled from ~t=24; the bear-fraction jump (0 -> 1/3) is
        // detected by the window-5 breadth delta at t=24..28.
        assert!((24..29).any(|t| c.labels[t] == BEAR as i8));
        assert!(c.labels[5] != BEAR as i8);
    }
}
