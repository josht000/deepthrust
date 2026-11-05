//! Family 9 trend and pullback systems (stocks).

use std::collections::BTreeSet;

use crate::config::{split_csv, Config};
use crate::data::Panel;
use crate::engine::{last_valid_rows, Trade};
use crate::family2::blank_signals;
use crate::signals::{rolling_mean, rolling_median, Signals};

pub const S094_NAME: &str = "s094_bollinger_squeeze";
pub const S094_TITLE: &str = "S094 Bollinger Squeeze Breakout";

/// Bollinger mid/upper/lower bands (population sd over `len` closes, NaN warm-up).
fn bb_bands(close: &[f64], len: usize, mult: f64) -> (Vec<f64>, Vec<f64>, Vec<f64>) {
    let n = close.len();
    let (mut mid, mut upper, mut lower) = (
        vec![f64::NAN; n],
        vec![f64::NAN; n],
        vec![f64::NAN; n],
    );
    let (mut sum, mut sum2, mut valid) = (0.0f64, 0.0f64, 0usize);
    for t in 0..n {
        let v = close[t];
        if v.is_finite() {
            sum += v;
            sum2 += v * v;
            valid += 1;
        }
        if t >= len {
            let old = close[t - len];
            if old.is_finite() {
                sum -= old;
                sum2 -= old * old;
                valid -= 1;
            }
        }
        if t + 1 >= len && valid == len {
            let m = sum / len as f64;
            let sd = ((sum2 - sum * m) / len as f64).max(0.0).sqrt();
            mid[t] = m;
            upper[t] = m + mult * sd;
            lower[t] = m - mult * sd;
        }
    }
    (mid, upper, lower)
}

// S094: BandWidth(20, 2sd) in the bottom decile of the last 120 sessions resolves
// directionally; trade the close outside the bands, exit on a middle-band cross.
pub fn compute_s094_signals(panel: &Panel, cfg: &Config) -> Result<Signals, String> {
    if cfg.bb_len < 2 || cfg.width_lookback < 2 || cfg.max_hold == 0 {
        return Err("S094 needs bb_len>=2, width_lookback>=2 and max_hold>0".into());
    }
    if !cfg.bb_mult.is_finite() || cfg.bb_mult <= 0.0 {
        return Err("S094 bb_mult must be positive and finite".into());
    }
    if !cfg.decile.is_finite() || cfg.decile <= 0.0 || cfg.decile > 1.0 {
        return Err("S094 decile must be in (0, 1]".into());
    }
    if !matches!(cfg.leg_mode.as_str(), "long" | "short" | "long_short") {
        return Err(format!("unknown S094 leg_mode {:?}", cfg.leg_mode));
    }
    if !matches!(cfg.exit_mode.as_str(), "mid_band" | "reentry" | "fixed") {
        return Err(format!("unknown S094 exit_mode {:?}", cfg.exit_mode));
    }
    if !matches!(cfg.gate.as_str(), "none" | "sma200") {
        return Err(format!("unknown S094 gate {:?}", cfg.gate));
    }
    let etf_names: BTreeSet<String> = split_csv(&cfg.etf_symbols).into_iter().collect();
    let stock_cols: Vec<usize> = (0..panel.symbols.len())
        .filter(|&j| !etf_names.contains(panel.symbols[j].as_str()))
        .collect();
    let ranks = if cfg.rank_cache_dir.trim().is_empty() {
        crate::data::liquidity_ranks(
            panel,
            &stock_cols,
            cfg.rank_window,
            cfg.rank_min_observations,
            cfg.min_price,
            cfg.min_history,
        )?
    } else {
        crate::data::read_liquid_rank_cache(&cfg.rank_cache_dir, &panel.dates, &panel.symbols)?
    };
    let n = panel.dates.len();
    let w = cfg.width_lookback;
    let mut result = blank_signals(panel, "bb_width_pctile");
    result.entry_mat = None;
    result.exit_mat = None;
    let mut weights = vec![vec![f64::NAN; n]; panel.symbols.len()];
    for j in 0..panel.symbols.len() {
        let close = &panel.close[j];
        let is_etf = etf_names.contains(panel.symbols[j].as_str());
        let (mid, upper, lower) = bb_bands(close, cfg.bb_len, cfg.bb_mult);
        let bandwidth: Vec<f64> = (0..n)
            .map(|t| {
                if mid[t].is_finite() && mid[t] != 0.0 {
                    (upper[t] - lower[t]) / mid[t]
                } else {
                    f64::NAN
                }
            })
            .collect();
        let medvol = if cfg.vol_confirm_mult > 0.0 {
            rolling_median(&panel.volume[j], cfg.gap_volume_window)
        } else {
            vec![f64::NAN; n]
        };
        let sma_gate = if cfg.gate == "sma200" {
            rolling_mean(close, cfg.sma_window)
        } else {
            vec![f64::NAN; n]
        };
        let mut pctile = vec![f64::NAN; n];
        let mut hist = 0usize;
        for t in 0..n {
            if close[t].is_finite() {
                hist += 1;
            }
            result.tradable[j][t] = if is_etf {
                close[t].is_finite() && hist >= cfg.min_history
            } else {
                cfg.include_stocks
                    && close[t].is_finite()
                    && close[t] >= cfg.min_price
                    && hist >= cfg.min_history
                    && ranks[j][t].is_finite()
                    && ranks[j][t] <= cfg.universe_top_n as f64
            };
            if t + 1 >= w {
                let window = &bandwidth[t + 1 - w..=t];
                if window.iter().all(|v| v.is_finite()) {
                    let count = window.iter().filter(|&&v| v <= bandwidth[t]).count();
                    pctile[t] = count as f64 / w as f64;
                }
            }
            result.rsi[j][t] = pctile[t];
            if !result.tradable[j][t] || !close[t].is_finite() || !upper[t].is_finite() {
                continue;
            }
            let squeezed = t >= cfg.squeeze_lag
                && pctile[t - cfg.squeeze_lag].is_finite()
                && pctile[t - cfg.squeeze_lag] <= cfg.decile;
            if !squeezed {
                continue;
            }
            let dir = if close[t] > upper[t] {
                if cfg.require_first_break
                    && !(t > 0
                        && close[t - 1].is_finite()
                        && upper[t - 1].is_finite()
                        && close[t - 1] <= upper[t - 1])
                {
                    continue;
                }
                1.0
            } else if close[t] < lower[t] {
                if cfg.require_first_break
                    && !(t > 0
                        && close[t - 1].is_finite()
                        && lower[t - 1].is_finite()
                        && close[t - 1] >= lower[t - 1])
                {
                    continue;
                }
                -1.0
            } else {
                continue;
            };
            if (dir > 0.0 && cfg.leg_mode == "short")
                || (dir < 0.0 && cfg.leg_mode == "long")
            {
                continue;
            }
            if cfg.gate == "sma200"
                && (!sma_gate[t].is_finite() || dir * (close[t] - sma_gate[t]) <= 0.0)
            {
                continue;
            }
            if cfg.vol_confirm_mult > 0.0
                && (!medvol[t].is_finite()
                    || !panel.volume[j][t].is_finite()
                    || panel.volume[j][t] <= cfg.vol_confirm_mult * medvol[t])
            {
                continue;
            }
            result.signal[j][t] = true;
            weights[j][t] = dir;
        }
    }
    result.trade_weights = Some(weights);
    Ok(result)
}

/// Equal-slot signed book: each entry is one 1/`book_slots` slot, shorts carry
/// negative weight and are charged `borrow_bps_annual`. `close` fills and exits
/// on the signal session; `next_open` enters open[t+1] and exits open[u+1].
pub fn s094_trades(
    panel: &Panel,
    sig: &Signals,
    cfg: &Config,
    fill: &str,
) -> Result<Vec<Trade>, String> {
    if !matches!(fill, "close" | "next_open") {
        return Err(format!("S094 does not support fill {fill:?}"));
    }
    let dir_sig = sig
        .trade_weights
        .as_ref()
        .ok_or_else(|| "S094 requires trade weights".to_string())?;
    let n = panel.dates.len();
    let last = last_valid_rows(&panel.close);
    let slot_w = 1.0 / cfg.book_slots.max(1) as f64;
    let borrow_per_bar = cfg.borrow_bps_annual / 10_000.0 / cfg.periods_per_year as f64;
    let mut trades = Vec::new();
    for j in 0..panel.symbols.len() {
        let Some(lj) = last[j] else { continue };
        if !dir_sig[j].iter().any(|v| v.is_finite()) {
            continue;
        }
        let (close, open) = (&panel.close[j], &panel.open[j]);
        let (mid, upper, lower) = bb_bands(close, cfg.bb_len, cfg.bb_mult);
        let cost = cfg.cost_for_symbol(&panel.symbols[j]);
        let mut pos: Option<(usize, usize, f64, f64)> = None; // (sig_day, entry, px, dir)
        for t in 0..=lj {
            if let Some((sd, e, ep, dir)) = pos {
                if !close[t].is_finite() {
                    continue;
                }
                let held = if fill == "next_open" { t - e + 1 } else { t - e };
                if held == 0 {
                    continue;
                }
                let mut reason: Option<&'static str> = None;
                if held >= cfg.min_hold {
                    let cross = match cfg.exit_mode.as_str() {
                        "reentry" => {
                            if dir > 0.0 {
                                close[t] < upper[t]
                            } else {
                                close[t] > lower[t]
                            }
                        }
                        "fixed" => false,
                        _ => mid[t].is_finite() && dir * (mid[t] - close[t]) > 0.0,
                    };
                    if cross {
                        reason = Some(if cfg.exit_mode == "reentry" {
                            "reentry"
                        } else {
                            "mid_band"
                        });
                    }
                }
                if reason.is_none() && held >= cfg.max_hold {
                    reason = Some("time");
                }
                if reason.is_none() && cfg.exit_mode == "fixed" && held >= cfg.fixed_hold.max(1) {
                    reason = Some("fixed");
                }
                if let Some(r) = reason {
                    let (exit_idx, px) = if fill == "close" {
                        (t, close[t])
                    } else if t + 1 <= lj && open[t + 1].is_finite() {
                        (t + 1, open[t + 1])
                    } else {
                        (t, close[t])
                    };
                    if !px.is_finite() || px <= 0.0 {
                        continue;
                    }
                    let gross = dir * (px / ep - 1.0);
                    let borrow = if dir < 0.0 {
                        borrow_per_bar * (exit_idx - e + 1) as f64
                    } else {
                        0.0
                    };
                    trades.push(Trade {
                        sym_idx: j,
                        signal_idx: sd,
                        entry_idx: e,
                        exit_idx,
                        entry_px: ep,
                        exit_px: px,
                        ret: gross - cost * 2.0 * slot_w - borrow,
                        ret_gross: gross,
                        exit_reason: r,
                        hold: exit_idx - e,
                        signal_value: sig.rsi[j][sd],
                        weight: dir * slot_w,
                        entry_turnover: slot_w,
                        exit_turnover: slot_w,
                        weighted: true,
                        borrow_cost_per_bar: if dir < 0.0 { borrow_per_bar } else { 0.0 },
                    });
                    pos = None;
                }
                continue;
            }
            let dir = dir_sig[j][t];
            if !dir.is_finite() || dir == 0.0 {
                continue;
            }
            let (e, px) = if fill == "close" {
                (t, close[t])
            } else {
                (
                    t + 1,
                    if t + 1 <= lj {
                        open[t + 1]
                    } else {
                        f64::NAN
                    },
                )
            };
            if px.is_finite() && px > 0.0 {
                pos = Some((t, e, px, dir));
            }
        }
        if let Some((sd, e, ep, dir)) = pos {
            let delisted = lj + 1 < n;
            let mut px = close[lj];
            if delisted {
                px *= 1.0 + cfg.delisting_return;
            }
            let gross = dir * (px / ep - 1.0);
            let borrow = if dir < 0.0 {
                borrow_per_bar * (lj - e + 1) as f64
            } else {
                0.0
            };
            trades.push(Trade {
                sym_idx: j,
                signal_idx: sd,
                entry_idx: e,
                exit_idx: lj,
                entry_px: ep,
                exit_px: px,
                ret: gross - cost * 2.0 * slot_w - borrow,
                ret_gross: gross,
                exit_reason: if delisted { "delisted" } else { "censored" },
                hold: lj - e,
                signal_value: sig.rsi[j][sd],
                weight: dir * slot_w,
                entry_turnover: slot_w,
                exit_turnover: slot_w,
                weighted: true,
                borrow_cost_per_bar: if dir < 0.0 { borrow_per_bar } else { 0.0 },
            });
        }
    }
    trades.sort_by(|a, b| {
        (a.entry_idx, &panel.symbols[a.sym_idx]).cmp(&(b.entry_idx, &panel.symbols[b.sym_idx]))
    });
    Ok(trades)
}
