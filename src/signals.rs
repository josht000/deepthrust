//! Daily-panel signal calculations for the registered strategy definitions.

use crate::config::Config;
use crate::data::Panel;

pub type Mat = Vec<Vec<f64>>;
pub type Mask = Vec<Vec<bool>>;

/// pandas `ewm(alpha, adjust=False, min_periods)` mean of a series (NaN inputs are skipped and output NaN).
fn ewm(x: &[f64], alpha: f64, min_periods: usize) -> Vec<f64> {
    let mut out = vec![f64::NAN; x.len()];
    let (mut state, mut seen) = (f64::NAN, 0usize);
    for (i, &v) in x.iter().enumerate() {
        if v.is_nan() {
            continue;
        }
        state = if seen == 0 {
            v
        } else {
            alpha * v + (1.0 - alpha) * state
        };
        seen += 1;
        if seen >= min_periods {
            out[i] = state;
        }
    }
    out
}

/// Wilder RSI(n).
pub fn wilder_rsi(close: &[f64], n: usize) -> Vec<f64> {
    let mut up = vec![f64::NAN; close.len()];
    let mut dn = vec![f64::NAN; close.len()];
    for i in 1..close.len() {
        let d = close[i] - close[i - 1];
        if d.is_finite() {
            up[i] = d.max(0.0);
            dn[i] = (-d).max(0.0);
        }
    }
    let alpha = 1.0 / n as f64;
    let (u, d) = (ewm(&up, alpha, n), ewm(&dn, alpha, n));
    u.iter()
        .zip(&d)
        .map(|(u, d)| 100.0 - 100.0 / (1.0 + u / d))
        .collect()
}

/// Wilder ATR(n): ewm(alpha=1/n) of the true range.
pub fn wilder_atr(high: &[f64], low: &[f64], close: &[f64], n: usize) -> Vec<f64> {
    let tr: Vec<f64> = (0..close.len())
        .map(|i| {
            let mut t = high[i] - low[i];
            if i > 0 {
                for c in [
                    (high[i] - close[i - 1]).abs(),
                    (low[i] - close[i - 1]).abs(),
                ] {
                    if t.is_nan() || c > t {
                        t = c;
                    }
                }
            }
            t
        })
        .collect();
    ewm(&tr, 1.0 / n as f64, n)
}

/// Rolling mean that is NaN until the window is full of finite values (pandas `min_periods=window`).
pub fn rolling_mean(x: &[f64], w: usize) -> Vec<f64> {
    let mut out = vec![f64::NAN; x.len()];
    for i in (w.max(1) - 1)..x.len() {
        let win = &x[i + 1 - w..=i];
        if win.iter().all(|v| v.is_finite()) {
            out[i] = win.iter().sum::<f64>() / w as f64;
        }
    }
    out
}

fn rolling_extreme(x: &[f64], w: usize, minimum: bool) -> Vec<f64> {
    let width = w.max(1);
    let mut out = vec![f64::NAN; x.len()];
    for t in (width - 1)..x.len() {
        let window = &x[t + 1 - width..=t];
        if window.iter().all(|value| value.is_finite()) {
            out[t] = if minimum {
                window.iter().copied().fold(f64::INFINITY, f64::min)
            } else {
                window.iter().copied().fold(f64::NEG_INFINITY, f64::max)
            };
        }
    }
    out
}

pub struct Signals {
    pub rsi: Mat,
    pub indicator_name: String,
    pub signal: Mask,
    pub exit_now: Mask,
    pub add_signal: Option<Mask>,
    pub atr: Option<Mat>,
    pub stop_pct: Option<Mat>,
    pub tradable: Mask,
    pub pool: Mask,
}

pub fn compute_signals(panel: &Panel, cfg: &Config) -> Signals {
    let n_sym = panel.symbols.len();
    let (mut rsi, mut signal, mut exit_now, mut tradable, mut add) =
        (vec![], vec![], vec![], vec![], vec![]);
    let mut atr_all = vec![];
    for j in 0..n_sym {
        let close = &panel.close[j];
        let n = close.len();
        let r = wilder_rsi(close, cfg.rsi_len);
        let sma_gate = rolling_mean(close, cfg.sma_window);
        let sma_exit = rolling_mean(close, cfg.exit_sma);
        let gate: Vec<bool> = if cfg.gate == "ret21" {
            let w = cfg.gate_ret_window;
            (0..n)
                .map(|t| t >= w && close[t].is_finite() && close[t] / close[t - w] - 1.0 > 0.0)
                .collect()
        } else {
            (0..n).map(|t| close[t] > sma_gate[t]).collect()
        };
        let mut hist = 0usize;
        let trad: Vec<bool> = close
            .iter()
            .map(|c| {
                if c.is_finite() {
                    hist += 1;
                }
                c.is_finite() && hist >= cfg.min_history
            })
            .collect();
        let ex: Vec<bool> = (0..n)
            .map(|t| {
                if cfg.exit_mode == "rsi" {
                    r[t] > cfg.exit_rsi
                } else {
                    close[t] > sma_exit[t]
                }
            })
            .collect();
        signal.push(
            (0..n)
                .map(|t| trad[t] && gate[t] && r[t] < cfg.rsi_threshold)
                .collect::<Vec<_>>(),
        );
        add.push((0..n).map(|t| r[t] < cfg.add_rsi).collect::<Vec<_>>());
        if cfg.disaster_stop_atr > 0.0 {
            atr_all.push(wilder_atr(
                &panel.high[j],
                &panel.low[j],
                close,
                cfg.atr_len,
            ));
        }
        exit_now.push(ex);
        tradable.push(trad);
        rsi.push(r);
    }
    Signals {
        rsi,
        indicator_name: format!("rsi{}", cfg.rsi_len),
        signal,
        exit_now,
        add_signal: if cfg.sizing == "pyramid3" {
            Some(add)
        } else {
            None
        },
        atr: if cfg.disaster_stop_atr > 0.0 {
            Some(atr_all)
        } else {
            None
        },
        stop_pct: None,
        pool: tradable.clone(),
        tradable,
    }
}

pub fn compute_double7_signals(panel: &Panel, cfg: &Config) -> Signals {
    let mut rsi = Vec::with_capacity(panel.symbols.len());
    let mut signal = Vec::with_capacity(panel.symbols.len());
    let mut exit_now = Vec::with_capacity(panel.symbols.len());
    let mut tradable = Vec::with_capacity(panel.symbols.len());
    let mut stop_pct = Vec::with_capacity(panel.symbols.len());
    for j in 0..panel.symbols.len() {
        let close = &panel.close[j];
        let n = close.len();
        let low = rolling_extreme(close, cfg.lookback, true);
        let high = rolling_extreme(close, cfg.lookback, false);
        let sma = rolling_mean(close, cfg.sma_window);
        let mut history = 0usize;
        let trad: Vec<bool> = close
            .iter()
            .map(|value| {
                if value.is_finite() {
                    history += 1;
                }
                value.is_finite() && history >= cfg.min_history
            })
            .collect();
        signal.push(
            (0..n)
                .map(|t| trad[t] && close[t] > sma[t] && close[t] <= low[t])
                .collect(),
        );
        exit_now.push((0..n).map(|t| close[t] >= high[t]).collect());
        if cfg.disaster_stop_atr > 0.0 {
            let atr = wilder_atr(&panel.high[j], &panel.low[j], close, cfg.atr_len);
            stop_pct.push(
                (0..n)
                    .map(|t| {
                        if atr[t].is_finite() && close[t].is_finite() && close[t] != 0.0 {
                            atr[t] / close[t]
                        } else {
                            f64::NAN
                        }
                    })
                    .collect(),
            );
        }
        tradable.push(trad);
        rsi.push(close.clone());
    }
    Signals {
        rsi,
        indicator_name: "close".into(),
        signal,
        exit_now,
        add_signal: None,
        atr: None,
        stop_pct: if cfg.disaster_stop_atr > 0.0 {
            Some(stop_pct)
        } else {
            None
        },
        pool: tradable.clone(),
        tradable,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rsi_matches_wilder_reference() {
        // alpha=1/2: up=[_,1,1,0], dn=[_,0,0,2]
        let rsi = wilder_rsi(&[10.0, 11.0, 12.0, 10.0], 2);
        assert!(rsi[0].is_nan() && rsi[1].is_nan());
        assert!((rsi[2] - 100.0).abs() < 1e-12); // dn == 0 -> RSI 100
                                                 // up = .5*0 + .5*1 = 0.5 ; dn = .5*2 + .5*0 = 1.0 -> RSI = 100 - 100/(1+0.5)
        assert!((rsi[3] - (100.0 - 100.0 / 1.5)).abs() < 1e-9);
    }

    #[test]
    fn rolling_mean_needs_full_window() {
        let m = rolling_mean(&[1.0, 2.0, 3.0, f64::NAN, 5.0, 6.0, 7.0], 3);
        assert!(m[1].is_nan() && (m[2] - 2.0).abs() < 1e-12 && m[3].is_nan() && m[5].is_nan());
        assert!((m[6] - 6.0).abs() < 1e-12);
    }

    #[test]
    fn atr_first_bar_is_range() {
        let a = wilder_atr(&[2.0, 3.0, 4.0], &[1.0, 1.0, 2.0], &[1.5, 2.0, 3.0], 2);
        assert!(a[0].is_nan());
        // TR = [1, 2, 2] -> ewm alpha .5: 1, 1.5, 1.75
        assert!((a[1] - 1.5).abs() < 1e-12 && (a[2] - 1.75).abs() < 1e-12);
    }

    #[test]
    fn double7_uses_inclusive_rolling_extrema() {
        let close = [vec![5.0; 200], vec![10.0, 9.0, 9.0, 11.0, 11.0]].concat();
        let panel = Panel {
            dates: (0..close.len())
                .map(|index| {
                    crate::data::parse_date("2024-01-01").unwrap()
                        + chrono::Duration::days(index as i64)
                })
                .collect(),
            symbols: vec!["X".into()],
            open: vec![close.clone()],
            high: vec![close.clone()],
            low: vec![close.clone()],
            close: vec![close],
        };
        let cfg = Config {
            lookback: 3,
            min_history: 1,
            sma_window: 200,
            ..crate::config::load_config(
                "s001_rsi2_connors",
                Some(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/configs/s001_rsi2_connors.yaml"
                )),
                &crate::config::Cli {
                    data_dir: None,
                    symbols: None,
                },
                None,
            )
            .unwrap()
        };
        let sig = compute_double7_signals(&panel, &cfg);
        assert!(sig.signal[0][202]);
        assert!(sig.exit_now[0][204]);
        assert_eq!(sig.indicator_name, "close");
    }
}
