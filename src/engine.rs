//! Swing-trade simulator (port of `swing_trades` in src/strategies/_engines.py).

use crate::config::Config;
use crate::data::Panel;
use crate::signals::{Mask, Mat};

#[derive(Debug, Clone)]
pub struct Trade {
    pub sym_idx: usize,
    pub signal_idx: usize,
    pub entry_idx: usize,
    pub exit_idx: usize,
    pub entry_px: f64,
    pub exit_px: f64,
    pub ret: f64,
    pub ret_gross: f64,
    pub exit_reason: &'static str,
    pub hold: usize,
    pub rsi2: f64,
}

/// Last row index with a finite close per symbol.
pub fn last_valid_rows(close: &Mat) -> Vec<Option<usize>> {
    close
        .iter()
        .map(|c| c.iter().rposition(|v| v.is_finite()))
        .collect()
}

pub struct SwingParams<'a> {
    pub signal: &'a Mask,
    pub exit_now: &'a Mask,
    pub add_signal: Option<&'a Mask>,
    pub max_units: usize,
    pub stop_atr: Option<&'a Mat>,
    pub stop_mult: f64,
    pub rsi: &'a Mat,
}

/// Per-symbol swing book: enter on `signal`, exit when `exit_now` or the stop fires.
///
/// fill="close": decisions on session t fill at close[t] (upper bound).
/// fill="next_open": signal day t fills at open[t+1]; an exit decided on session u fills at open[u+1].
pub fn swing_trades(panel: &Panel, p: &SwingParams, cfg: &Config, fill: &str) -> Vec<Trade> {
    let n_rows = panel.dates.len();
    let cost = cfg.cost();
    let is_open = fill == "next_open";
    let last = last_valid_rows(&panel.close);
    let mut rows: Vec<Trade> = vec![];
    for j in 0..panel.symbols.len() {
        if !p.signal[j].iter().any(|&s| s) {
            continue;
        }
        let (cj, oj) = (&panel.close[j], &panel.open[j]);
        let Some(lj) = last[j] else { continue };
        let mut t_prev: Option<usize> = None;
        for t in (0..n_rows).filter(|&t| p.signal[j][t]) {
            if matches!(t_prev, Some(tp) if t <= tp) {
                continue;
            }
            let entry0 = if is_open { t + 1 } else { t };
            if entry0 > lj || (is_open && !oj[entry0].is_finite()) {
                continue;
            }
            let e0_px = if is_open { oj[entry0] } else { cj[t] };
            let stop_level = p.stop_atr.and_then(|a| {
                if a[j][t].is_finite() {
                    Some(e0_px - p.stop_mult * a[j][t])
                } else {
                    None
                }
            });
            let (mut reason, mut u_exit) = ("signal", None);
            for u in (t + 1)..=lj {
                if !cj[u].is_finite() {
                    continue;
                }
                if matches!(stop_level, Some(s) if cj[u] < s) {
                    reason = "stop";
                    u_exit = Some(u);
                    break;
                }
                if p.exit_now[j][u] {
                    u_exit = Some(u);
                    break;
                }
            }
            let u_exit = match u_exit {
                Some(u) => u,
                None => {
                    reason = if lj < n_rows - 1 {
                        "delisted"
                    } else {
                        "censored"
                    };
                    lj
                }
            };
            let mut legs = vec![t];
            if let Some(a) = p.add_signal {
                for u in (t + 1)..u_exit {
                    if legs.len() >= p.max_units {
                        break;
                    }
                    if cj[u].is_finite() && a[j][u] {
                        legs.push(u);
                    }
                }
            }
            for &t_leg in &legs {
                let entry = if is_open { t_leg + 1 } else { t_leg };
                if entry > lj || (is_open && !oj[entry].is_finite()) {
                    continue;
                }
                let entry_px = if is_open { oj[entry] } else { cj[t_leg] };
                let (mut exit_idx, mut exit_px) = (u_exit, cj[u_exit]);
                if is_open
                    && reason != "delisted"
                    && reason != "censored"
                    && u_exit + 1 <= lj
                    && oj[u_exit + 1].is_finite()
                {
                    exit_idx = u_exit + 1;
                    exit_px = oj[exit_idx];
                }
                if exit_idx < entry {
                    continue;
                }
                let net = exit_px * (1.0 - cost) / (entry_px * (1.0 + cost)) - 1.0;
                rows.push(Trade {
                    sym_idx: j,
                    signal_idx: t_leg,
                    entry_idx: entry,
                    exit_idx,
                    entry_px,
                    exit_px,
                    ret: net,
                    ret_gross: exit_px / entry_px - 1.0,
                    exit_reason: reason,
                    hold: exit_idx - entry,
                    rsi2: p.rsi[j][t_leg],
                });
            }
            t_prev = Some(u_exit);
        }
    }
    rows.sort_by(|a, b| {
        (a.entry_idx, &panel.symbols[a.sym_idx]).cmp(&(b.entry_idx, &panel.symbols[b.sym_idx]))
    });
    rows
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::parse_date;

    fn cfg() -> Config {
        crate::config::load_config(
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
    }

    #[test]
    fn next_open_and_close_fills() {
        let n = 6;
        let dates = (0..n)
            .map(|i| parse_date(&format!("2024-01-{:02}", i + 2)).unwrap())
            .collect();
        let panel = Panel {
            dates,
            symbols: vec!["X".into()],
            open: vec![vec![10.0, 11.0, 12.0, 13.0, 14.0, 15.0]],
            high: vec![vec![10.0; n]],
            low: vec![vec![10.0; n]],
            close: vec![vec![10.5, 11.5, 12.5, 13.5, 14.5, 15.5]],
        };
        let mut sig = vec![false; n];
        sig[1] = true;
        let mut ex = vec![false; n];
        ex[3] = true;
        let rsi = vec![vec![5.0; n]];
        let (signal, exit_now) = (vec![sig], vec![ex]);
        let p = SwingParams {
            signal: &signal,
            exit_now: &exit_now,
            add_signal: None,
            max_units: 1,
            stop_atr: None,
            stop_mult: 0.0,
            rsi: &rsi,
        };
        let c = cfg();
        let t = swing_trades(&panel, &p, &c, "next_open");
        assert_eq!(t.len(), 1);
        assert_eq!((t[0].signal_idx, t[0].entry_idx, t[0].exit_idx), (1, 2, 4));
        assert_eq!((t[0].entry_px, t[0].exit_px), (12.0, 14.0));
        let t = swing_trades(&panel, &p, &c, "close");
        assert_eq!(
            (t[0].entry_idx, t[0].exit_idx, t[0].entry_px, t[0].exit_px),
            (1, 3, 11.5, 13.5)
        );
        assert!(t[0].ret < t[0].ret_gross);
    }
}
