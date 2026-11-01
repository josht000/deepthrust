//! Daily-panel signal calculations for the registered strategy definitions.

use crate::config::Config;
use crate::data::Panel;
use chrono::Datelike;

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

fn rolling_median(x: &[f64], w: usize) -> Vec<f64> {
    let width = w.max(1);
    let mut out = vec![f64::NAN; x.len()];
    for t in (width - 1)..x.len() {
        let mut window = x[t + 1 - width..=t].to_vec();
        if window.iter().all(|value| value.is_finite()) {
            window.sort_by(f64::total_cmp);
            let mid = width / 2;
            out[t] = if width % 2 == 0 {
                (window[mid - 1] + window[mid]) / 2.0
            } else {
                window[mid]
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
    pub trade_weights: Option<Mat>,
    pub entry_mat: Option<Mat>,
    pub exit_mat: Option<Mat>,
    pub benchmark_returns: Option<Vec<f64>>,
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
        trade_weights: None,
        entry_mat: None,
        exit_mat: None,
        benchmark_returns: None,
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
        trade_weights: None,
        entry_mat: None,
        exit_mat: None,
        benchmark_returns: None,
    }
}

pub fn compute_s019_signals(panel: &Panel, cfg: &Config) -> Result<Signals, String> {
    if cfg.gap_volume_window == 0 || cfg.gap_max_hold == 0 {
        return Err("S019 requires positive gap_volume_window and gap_max_hold".into());
    }
    if !cfg.gap_atr_multiple.is_finite()
        || cfg.gap_atr_multiple <= 0.0
        || !cfg.gap_volume_multiple.is_finite()
        || cfg.gap_volume_multiple <= 0.0
        || !cfg.gap_reward_risk.is_finite()
        || cfg.gap_reward_risk <= 0.0
    {
        return Err("S019 gap and risk parameters must be positive and finite".into());
    }
    if !matches!(cfg.gap_volume_mode.as_str(), "high" | "low") {
        return Err(format!(
            "unknown S019 gap_volume_mode {:?}",
            cfg.gap_volume_mode
        ));
    }
    if !matches!(cfg.gap_stop_mode.as_str(), "prior_close" | "gap_low") {
        return Err(format!(
            "unknown S019 gap_stop_mode {:?}",
            cfg.gap_stop_mode
        ));
    }
    if panel.volume.len() != panel.symbols.len()
        || panel
            .volume
            .iter()
            .any(|volume| volume.len() != panel.dates.len())
    {
        return Err("S019 volume data does not match the daily panel".into());
    }

    let mut values = vec![vec![f64::NAN; panel.dates.len()]; panel.symbols.len()];
    let mut signal = vec![vec![false; panel.dates.len()]; panel.symbols.len()];
    let mut tradable = vec![vec![false; panel.dates.len()]; panel.symbols.len()];
    for symbol in 0..panel.symbols.len() {
        let (close, open, high, low, volume) = (
            &panel.close[symbol],
            &panel.open[symbol],
            &panel.high[symbol],
            &panel.low[symbol],
            &panel.volume[symbol],
        );
        let atr = wilder_atr(high, low, close, 20);
        let median_volume = rolling_median(volume, cfg.gap_volume_window);
        for day in 1..panel.dates.len() {
            let previous_close = close[day - 1];
            let current_volume = volume[day];
            if !previous_close.is_finite()
                || previous_close <= 0.0
                || !open[day].is_finite()
                || !close[day].is_finite()
                || !current_volume.is_finite()
                || !median_volume[day - 1].is_finite()
                || median_volume[day - 1] <= 0.0
                || !atr[day - 1].is_finite()
                || atr[day - 1] <= 0.0
            {
                continue;
            }
            tradable[symbol][day] = true;
            let gap = open[day] - previous_close;
            let relative_volume = current_volume / median_volume[day - 1];
            let volume_match = if cfg.gap_volume_mode == "high" {
                relative_volume > cfg.gap_volume_multiple
            } else {
                relative_volume < cfg.gap_volume_multiple
            };
            if gap > cfg.gap_atr_multiple * atr[day - 1] && volume_match {
                values[symbol][day] = gap / atr[day - 1];
                signal[symbol][day] = true;
            }
        }
    }

    Ok(Signals {
        rsi: values,
        indicator_name: "gap_atr".into(),
        signal,
        exit_now: vec![vec![false; panel.dates.len()]; panel.symbols.len()],
        add_signal: None,
        atr: None,
        stop_pct: None,
        tradable: tradable.clone(),
        pool: tradable,
        trade_weights: None,
        entry_mat: None,
        exit_mat: None,
        benchmark_returns: None,
    })
}

pub fn s019_trades(
    panel: &Panel,
    sig: &Signals,
    cfg: &Config,
    fill: &str,
) -> Result<Vec<crate::engine::Trade>, String> {
    if fill != "close" {
        return Err(format!("S019 requires close fills, got {fill:?}"));
    }
    let mut trades = Vec::new();
    for symbol in 0..panel.symbols.len() {
        let (open, high, low, close) = (
            &panel.open[symbol],
            &panel.high[symbol],
            &panel.low[symbol],
            &panel.close[symbol],
        );
        let last = close.iter().rposition(|value| value.is_finite());
        let Some(last) = last else { continue };
        let cost = cfg.cost_for_symbol(&panel.symbols[symbol]);
        let mut previous_exit = None;
        for day in 1..panel.dates.len() {
            if !sig.signal[symbol][day]
                || last <= day
                || matches!(previous_exit, Some(exit) if day <= exit)
            {
                continue;
            }
            let entry_px = close[day];
            let prior_close = close[day - 1];
            let stop = if cfg.gap_stop_mode == "gap_low" {
                low[day]
            } else {
                prior_close
            };
            let risk = entry_px - stop;
            if !entry_px.is_finite() || !stop.is_finite() || risk <= 0.0 {
                continue;
            }
            let target = entry_px + cfg.gap_reward_risk * risk;
            let timeout = day.saturating_add(cfg.gap_max_hold).min(last);
            let mut exit = timeout;
            let mut exit_px = close[timeout];
            let mut reason = if timeout < day + cfg.gap_max_hold {
                "censored"
            } else {
                "time"
            };
            for session in (day + 1)..=timeout {
                if !open[session].is_finite()
                    || !high[session].is_finite()
                    || !low[session].is_finite()
                    || !close[session].is_finite()
                {
                    continue;
                }
                if open[session] <= stop {
                    exit = session;
                    exit_px = open[session];
                    reason = "stop_gap";
                    break;
                }
                if low[session] <= stop {
                    exit = session;
                    exit_px = stop;
                    reason = "stop";
                    break;
                }
                if open[session] >= target {
                    exit = session;
                    exit_px = open[session];
                    reason = "target_gap";
                    break;
                }
                if high[session] >= target {
                    exit = session;
                    exit_px = target;
                    reason = "target";
                    break;
                }
            }
            if !exit_px.is_finite() {
                continue;
            }
            let gross = exit_px / entry_px - 1.0;
            trades.push(crate::engine::Trade {
                sym_idx: symbol,
                signal_idx: day,
                entry_idx: day,
                exit_idx: exit,
                entry_px,
                exit_px,
                ret: (exit_px * (1.0 - cost) / (entry_px * (1.0 + cost))) - 1.0,
                ret_gross: gross,
                exit_reason: reason,
                hold: exit - day,
                signal_value: sig.rsi[symbol][day],
                weight: 1.0,
                entry_turnover: 1.0,
                exit_turnover: 1.0,
                weighted: false,
            });
            previous_exit = Some(exit);
        }
    }
    trades.sort_by(|a, b| {
        (a.entry_idx, &panel.symbols[a.sym_idx]).cmp(&(b.entry_idx, &panel.symbols[b.sym_idx]))
    });
    Ok(trades)
}

pub fn compute_s021_signals(panel: &Panel, cfg: &Config) -> Result<Signals, String> {
    if panel.symbols.len() != 1 {
        return Err(format!(
            "{} expects exactly one symbol, got {}",
            cfg.strategy,
            panel.symbols.len()
        ));
    }
    if !matches!(cfg.participation_filter.as_str(), "none" | "sma50") {
        return Err(format!(
            "unknown S021 participation_filter {:?}",
            cfg.participation_filter
        ));
    }

    let close = &panel.close[0];
    let open = &panel.open[0];
    let n = close.len();
    let mut entry = vec![vec![f64::NAN; n]];
    let mut exit = vec![vec![f64::NAN; n]];
    let mut overnight = vec![f64::NAN; n];
    let mut signal = vec![vec![false; n]];
    let mut tradable = vec![vec![false; n]];
    let mut pool = vec![vec![false; n]];
    let mut trade_weights = vec![vec![0.0; n]];
    let values = vec![vec![f64::NAN; n]];
    let sma = rolling_mean(close, 50);

    for day in 1..n {
        let (entry_px, exit_px) = (close[day - 1], open[day]);
        if !entry_px.is_finite() || entry_px <= 0.0 || !exit_px.is_finite() {
            continue;
        }
        entry[0][day] = entry_px;
        exit[0][day] = exit_px;
        overnight[day] = exit_px / entry_px - 1.0;
        pool[0][day - 1] = true;
        tradable[0][day - 1] = true;
        let allowed = cfg.participation_filter == "none"
            || (close[day - 1].is_finite() && close[day - 1] > sma[day - 1]);
        signal[0][day - 1] = allowed;
        if allowed {
            trade_weights[0][day] = 1.0;
        }
    }

    Ok(Signals {
        rsi: values,
        indicator_name: "overnight_return".into(),
        signal,
        exit_now: vec![vec![false; n]],
        add_signal: None,
        atr: None,
        stop_pct: None,
        tradable,
        pool,
        trade_weights: Some(trade_weights),
        entry_mat: Some(entry),
        exit_mat: Some(exit),
        benchmark_returns: Some(
            overnight
                .iter()
                .map(|value| if value.is_finite() { *value } else { 0.0 })
                .collect(),
        ),
    })
}

pub fn compute_s105_signals(panel: &Panel, cfg: &Config) -> Result<Signals, String> {
    if panel.symbols.len() != 1 {
        return Err(format!(
            "{} expects exactly one symbol, got {}",
            cfg.strategy,
            panel.symbols.len()
        ));
    }
    if cfg.vol_window < 2 || !cfg.target_vol.is_finite() || cfg.target_vol <= 0.0 {
        return Err("S105 requires vol_window >= 2 and a positive target_vol".into());
    }
    if !cfg.max_leverage.is_finite()
        || cfg.max_leverage <= 0.0
        || !cfg.resize_band.is_finite()
        || cfg.resize_band < 0.0
    {
        return Err("S105 requires positive max_leverage and nonnegative resize_band".into());
    }

    let close = &panel.close[0];
    let open = &panel.open[0];
    let n = close.len();
    let mut entry = vec![vec![f64::NAN; n]];
    let mut exit = vec![vec![f64::NAN; n]];
    let mut overnight = vec![f64::NAN; n];
    let mut weights = vec![vec![0.0; n]];
    let mut signal = vec![vec![false; n]];
    let mut tradable = vec![vec![false; n]];
    let mut pool = vec![vec![false; n]];
    let values = vec![vec![f64::NAN; n]];
    let (mut current, window) = (0.0, cfg.vol_window);

    for day in 1..n {
        let (entry_px, exit_px) = (close[day - 1], open[day]);
        if !entry_px.is_finite() || entry_px <= 0.0 || !exit_px.is_finite() {
            continue;
        }
        entry[0][day] = entry_px;
        exit[0][day] = exit_px;
        overnight[day] = exit_px / entry_px - 1.0;
        if day <= window {
            continue;
        }
        let sample = &overnight[day - window..day];
        if sample.iter().any(|value| !value.is_finite()) {
            continue;
        }
        let mean = sample.iter().sum::<f64>() / window as f64;
        let variance = sample
            .iter()
            .map(|value| (value - mean).powi(2))
            .sum::<f64>()
            / (window - 1).max(1) as f64;
        let annual_vol = variance.sqrt() * 252.0f64.sqrt();
        if !annual_vol.is_finite() || annual_vol <= 0.0 {
            continue;
        }
        let desired = cfg.max_leverage.min(cfg.target_vol / annual_vol);
        if current <= 0.0
            || cfg.resize_band <= 0.0
            || (desired / current - 1.0).abs() > cfg.resize_band
        {
            current = desired;
        }
        weights[0][day] = current;
        signal[0][day - 1] = current > 0.0;
        tradable[0][day - 1] = true;
        pool[0][day - 1] = true;
    }
    Ok(Signals {
        rsi: values,
        indicator_name: "overnight_return".into(),
        signal,
        exit_now: vec![vec![false; n]],
        add_signal: None,
        atr: None,
        stop_pct: None,
        tradable,
        pool,
        trade_weights: Some(weights),
        entry_mat: Some(entry),
        exit_mat: Some(exit),
        benchmark_returns: Some(
            overnight
                .iter()
                .map(|value| if value.is_finite() { *value } else { 0.0 })
                .collect(),
        ),
    })
}

pub fn overnight_trades(
    panel: &Panel,
    sig: &Signals,
    cfg: &Config,
    fill: &str,
) -> Result<Vec<crate::engine::Trade>, String> {
    if fill != "close_to_open" {
        return Err(format!(
            "overnight strategies require the close_to_open fill, got {fill:?}"
        ));
    }
    let (Some(weights), Some(entries), Some(exits)) =
        (&sig.trade_weights, &sig.entry_mat, &sig.exit_mat)
    else {
        return Err("overnight signals are missing trade weights or price matrices".into());
    };
    let mut trades = Vec::new();
    for symbol in 0..panel.symbols.len() {
        let cost = cfg.cost_for_symbol(&panel.symbols[symbol]);
        for day in 1..panel.dates.len() {
            let weight = weights[symbol][day];
            let entry_px = entries[symbol][day];
            let exit_px = exits[symbol][day];
            if weight <= 0.0 || !weight.is_finite() || !entry_px.is_finite() || !exit_px.is_finite()
            {
                continue;
            }
            let gross = exit_px / entry_px - 1.0;
            trades.push(crate::engine::Trade {
                sym_idx: symbol,
                signal_idx: day - 1,
                entry_idx: day,
                exit_idx: day,
                entry_px,
                exit_px,
                ret: weight * gross - 2.0 * cost * weight,
                ret_gross: weight * gross,
                exit_reason: "overnight",
                hold: 1,
                signal_value: f64::NAN,
                weight,
                entry_turnover: weight,
                exit_turnover: weight,
                weighted: true,
            });
        }
    }
    Ok(trades)
}

pub fn compute_s042_signals(panel: &Panel, cfg: &Config) -> Result<Signals, String> {
    let windows: &[usize] = match cfg.signal.as_str() {
        "mom12" => &[252],
        "blend13" => &[21, 63],
        "rankavg" => &[21, 63, 126, 189, 252],
        other => return Err(format!("unknown S042 signal {other:?}")),
    };
    if !matches!(cfg.rebalance.as_str(), "monthly" | "weekly") {
        return Err(format!(
            "unknown S042 rebalance frequency {:?}",
            cfg.rebalance
        ));
    }
    if cfg.breadth == 0 {
        return Err("S042 breadth must be greater than zero".into());
    }

    let n_symbols = panel.symbols.len();
    let n_rows = panel.dates.len();
    let mut returns = Vec::with_capacity(windows.len());
    for &window in windows {
        let mut values = vec![vec![f64::NAN; n_rows]; n_symbols];
        for (symbol, close) in panel.close.iter().enumerate() {
            for day in window..n_rows {
                let (current, prior) = (close[day], close[day - window]);
                if current.is_finite() && prior.is_finite() && prior > 0.0 {
                    values[symbol][day] = current / prior - 1.0;
                }
            }
        }
        returns.push(values);
    }

    let mut score = vec![vec![f64::NAN; n_rows]; n_symbols];
    if cfg.signal == "rankavg" {
        for day in 0..n_rows {
            let mut summed_ranks = vec![0.0; n_symbols];
            let mut rank_count = vec![0usize; n_symbols];
            for values in &returns {
                let mut ranked: Vec<_> = (0..n_symbols)
                    .filter_map(|symbol| {
                        values[symbol][day]
                            .is_finite()
                            .then_some((symbol, values[symbol][day]))
                    })
                    .collect();
                ranked.sort_by(|(col_a, value_a), (col_b, value_b)| {
                    value_b.total_cmp(value_a).then_with(|| col_a.cmp(col_b))
                });
                for (rank, (symbol, _)) in ranked.into_iter().enumerate() {
                    summed_ranks[symbol] += rank as f64;
                    rank_count[symbol] += 1;
                }
            }
            for symbol in 0..n_symbols {
                if rank_count[symbol] == windows.len() {
                    score[symbol][day] = -(summed_ranks[symbol] / rank_count[symbol] as f64);
                }
            }
        }
    } else {
        for symbol in 0..n_symbols {
            for day in 0..n_rows {
                let values: Option<Vec<f64>> = returns
                    .iter()
                    .map(|matrix| {
                        matrix[symbol][day]
                            .is_finite()
                            .then_some(matrix[symbol][day])
                    })
                    .collect();
                if let Some(values) = values {
                    score[symbol][day] = values.iter().sum::<f64>() / values.len() as f64;
                }
            }
        }
    }

    let mut tradable = vec![vec![false; n_rows]; n_symbols];
    for (symbol, close) in panel.close.iter().enumerate() {
        let mut history = 0;
        for day in 0..n_rows {
            if close[day].is_finite() {
                history += 1;
            }
            tradable[symbol][day] = close[day].is_finite() && history >= cfg.min_history;
        }
    }
    let mut signal = vec![vec![false; n_rows]; n_symbols];
    let mut rebalance_rows = Vec::new();
    for day in 0..n_rows {
        let is_end = day + 1 == n_rows
            || if cfg.rebalance == "monthly" {
                let current = panel.dates[day];
                let next = panel.dates[day + 1];
                current.year() != next.year() || current.month() != next.month()
            } else {
                panel.dates[day].iso_week() != panel.dates[day + 1].iso_week()
            };
        if is_end {
            rebalance_rows.push(day);
            for symbol in 0..n_symbols {
                signal[symbol][day] = tradable[symbol][day] && score[symbol][day].is_finite();
            }
        }
    }

    Ok(Signals {
        rsi: score,
        indicator_name: "momentum_score".into(),
        signal,
        exit_now: vec![vec![false; n_rows]; n_symbols],
        add_signal: None,
        atr: None,
        stop_pct: None,
        tradable: tradable.clone(),
        pool: tradable,
        trade_weights: None,
        entry_mat: None,
        exit_mat: None,
        benchmark_returns: None,
    })
}

pub fn s042_trades(
    panel: &Panel,
    sig: &Signals,
    cfg: &Config,
    fill: &str,
) -> Result<Vec<crate::engine::Trade>, String> {
    rebalance_trades(panel, sig, cfg, fill, cfg.breadth)
}

fn rebalance_trades(
    panel: &Panel,
    sig: &Signals,
    cfg: &Config,
    fill: &str,
    breadth: usize,
) -> Result<Vec<crate::engine::Trade>, String> {
    if !matches!(fill, "close" | "next_open") {
        return Err(format!("{} does not support fill {fill:?}", cfg.strategy));
    }
    let n_rows = panel.dates.len();
    if n_rows == 0 {
        return Ok(Vec::new());
    }
    if sig.rsi.len() != panel.symbols.len()
        || sig.tradable.len() != panel.symbols.len()
        || sig.rsi.iter().any(|series| series.len() != n_rows)
        || sig.tradable.iter().any(|series| series.len() != n_rows)
    {
        return Err("rebalance signal dimensions do not match the daily panel".into());
    }
    let rebalance_rows: Vec<_> = (0..n_rows)
        .filter(|&day| {
            day + 1 == n_rows
                || if cfg.rebalance == "monthly" {
                    let current = panel.dates[day];
                    let next = panel.dates[day + 1];
                    current.year() != next.year() || current.month() != next.month()
                } else {
                    panel.dates[day].iso_week() != panel.dates[day + 1].iso_week()
                }
        })
        .collect();
    let mut boundaries = rebalance_rows.clone();
    boundaries.push(n_rows - 1);
    let mut trades = Vec::new();
    for (period, &signal_day) in rebalance_rows.iter().enumerate() {
        let boundary = boundaries[period + 1];
        let mut picks: Vec<_> = (0..panel.symbols.len())
            .filter(|&symbol| {
                sig.tradable[symbol][signal_day] && sig.rsi[symbol][signal_day].is_finite()
            })
            .collect();
        picks.sort_by(|&a, &b| {
            sig.rsi[b][signal_day]
                .total_cmp(&sig.rsi[a][signal_day])
                .then_with(|| panel.symbols[a].cmp(&panel.symbols[b]))
        });
        picks.truncate(breadth);
        for symbol in picks {
            let close = &panel.close[symbol];
            let open = &panel.open[symbol];
            let last = close.iter().rposition(|value| value.is_finite());
            let Some(last) = last else { continue };
            let is_open = fill == "next_open";
            let entry = if is_open { signal_day + 1 } else { signal_day };
            if entry > last || (is_open && !open[entry].is_finite()) {
                continue;
            }
            let entry_px = if is_open {
                open[entry]
            } else {
                close[signal_day]
            };
            let delisted = last < boundary;
            let mut exit = if is_open {
                boundary.saturating_add(1).min(last)
            } else {
                boundary.min(last)
            };
            let reason = if delisted {
                if last + 1 < n_rows {
                    "delisted"
                } else {
                    "censored"
                }
            } else if period + 1 == rebalance_rows.len() {
                "censored"
            } else {
                "rebalance"
            };
            let mut exit_px = if is_open && !delisted && open[exit].is_finite() {
                open[exit]
            } else {
                close[exit]
            };
            while exit > entry && !exit_px.is_finite() {
                exit -= 1;
                exit_px = close[exit];
            }
            if reason == "delisted" {
                exit_px *= 1.0 + cfg.delisting_return;
            }
            if exit < entry || !entry_px.is_finite() || !exit_px.is_finite() {
                continue;
            }
            let cost = cfg.cost_for_symbol(&panel.symbols[symbol]);
            trades.push(crate::engine::Trade {
                sym_idx: symbol,
                signal_idx: signal_day,
                entry_idx: entry,
                exit_idx: exit,
                entry_px,
                exit_px,
                ret: exit_px * (1.0 - cost) / (entry_px * (1.0 + cost)) - 1.0,
                ret_gross: exit_px / entry_px - 1.0,
                exit_reason: reason,
                hold: exit - entry,
                signal_value: sig.rsi[symbol][signal_day],
                weight: 1.0,
                entry_turnover: 1.0,
                exit_turnover: 1.0,
                weighted: false,
            });
        }
    }
    trades.sort_by(|a, b| {
        (a.entry_idx, &panel.symbols[a.sym_idx]).cmp(&(b.entry_idx, &panel.symbols[b.sym_idx]))
    });
    Ok(trades)
}

pub fn compute_s033_signals(panel: &Panel, cfg: &Config) -> Result<Signals, String> {
    if cfg.entry_window == 0 || cfg.exit_window == 0 {
        return Err("S033 breakout windows must be greater than zero".into());
    }
    let etf_symbols = crate::config::split_csv(&cfg.etf_symbols);
    if etf_symbols.is_empty() {
        return Err("S033 requires etf_symbols to identify the ETF universe".into());
    }
    let etf_columns: Vec<_> = panel
        .symbols
        .iter()
        .enumerate()
        .filter_map(|(column, symbol)| etf_symbols.contains(symbol).then_some(column))
        .collect();
    if etf_columns.len() != etf_symbols.len() {
        let missing: Vec<_> = etf_symbols
            .iter()
            .filter(|symbol| !panel.symbols.contains(symbol))
            .cloned()
            .collect();
        return Err(format!(
            "S033 ETF symbols missing from panel: {}",
            missing.join(",")
        ));
    }
    let etf_set: std::collections::BTreeSet<_> = etf_columns.iter().copied().collect();
    let stock_columns: Vec<_> = (0..panel.symbols.len())
        .filter(|column| !etf_set.contains(column))
        .collect();
    let ranks = if cfg.include_stocks && !stock_columns.is_empty() {
        crate::data::liquidity_ranks(
            panel,
            &stock_columns,
            cfg.rank_window,
            cfg.rank_min_observations,
            cfg.min_price,
            cfg.min_history,
        )?
    } else {
        vec![vec![f64::NAN; panel.dates.len()]; panel.symbols.len()]
    };

    let n_rows = panel.dates.len();
    let mut values = vec![vec![f64::NAN; n_rows]; panel.symbols.len()];
    let mut signal = vec![vec![false; n_rows]; panel.symbols.len()];
    let mut exit_now = vec![vec![false; n_rows]; panel.symbols.len()];
    let mut tradable = vec![vec![false; n_rows]; panel.symbols.len()];
    for symbol in 0..panel.symbols.len() {
        let close = &panel.close[symbol];
        let high = rolling_extreme(close, cfg.entry_window, false);
        let low = rolling_extreme(close, cfg.exit_window, true);
        let mut history = 0;
        for day in 0..n_rows {
            if close[day].is_finite() {
                history += 1;
            }
            if !close[day].is_finite() || history < cfg.min_history {
                continue;
            }
            let is_etf = etf_set.contains(&symbol);
            let in_stock_universe = cfg.include_stocks
                && !is_etf
                && ranks[symbol][day].is_finite()
                && ranks[symbol][day] <= cfg.universe_top_n as f64;
            if !is_etf && !in_stock_universe {
                continue;
            }
            tradable[symbol][day] = true;
            exit_now[symbol][day] = close[day] <= low[day];
            if high[day].is_finite() && close[day] >= high[day] {
                signal[symbol][day] = true;
                values[symbol][day] = close[day] / high[day] - 1.0;
            }
        }
    }
    Ok(Signals {
        rsi: values,
        indicator_name: "breakout_distance".into(),
        signal,
        exit_now,
        add_signal: None,
        atr: None,
        stop_pct: None,
        tradable: tradable.clone(),
        pool: tradable,
        trade_weights: None,
        entry_mat: None,
        exit_mat: None,
        benchmark_returns: None,
    })
}

pub fn s033_trades(
    panel: &Panel,
    sig: &Signals,
    cfg: &Config,
    fill: &str,
) -> Result<Vec<crate::engine::Trade>, String> {
    let params = crate::engine::SwingParams {
        signal: &sig.signal,
        exit_now: &sig.exit_now,
        add_signal: None,
        max_units: 1,
        stop_atr: None,
        stop_pct: None,
        stop_mult: 0.0,
        signal_value: &sig.rsi,
    };
    let mut trades = crate::engine::swing_trades(panel, &params, cfg, fill);
    trades.retain(|trade| trade.hold > 0);
    Ok(trades)
}

pub fn compute_s041_signals(panel: &Panel, cfg: &Config) -> Result<Signals, String> {
    if cfg.industry_neutral {
        return Err("S041 industry-neutral mode requires a historical industry map, which is not configured".into());
    }
    if cfg.mom_window == 0 || cfg.skip_window >= cfg.mom_window {
        return Err("S041 requires mom_window > skip_window >= 0".into());
    }
    if !matches!(cfg.breadth_mode.as_str(), "decile" | "top") {
        return Err(format!("unknown S041 breadth_mode {:?}", cfg.breadth_mode));
    }
    let columns: Vec<_> = (0..panel.symbols.len()).collect();
    let ranks = crate::data::liquidity_ranks(
        panel,
        &columns,
        cfg.rank_window,
        cfg.rank_min_observations,
        cfg.min_price,
        cfg.min_history,
    )?;
    let n_rows = panel.dates.len();
    let mut score = vec![vec![f64::NAN; n_rows]; panel.symbols.len()];
    let mut signal = vec![vec![false; n_rows]; panel.symbols.len()];
    let mut tradable = vec![vec![false; n_rows]; panel.symbols.len()];
    for symbol in 0..panel.symbols.len() {
        let close = &panel.close[symbol];
        for day in cfg.mom_window..n_rows {
            let (current, prior) = (close[day - cfg.skip_window], close[day - cfg.mom_window]);
            if current.is_finite() && prior.is_finite() && prior > 0.0 {
                score[symbol][day] = current / prior - 1.0;
            }
        }
        for day in 0..n_rows {
            tradable[symbol][day] = panel.close[symbol][day].is_finite()
                && ranks[symbol][day].is_finite()
                && ranks[symbol][day] <= cfg.universe_top_n as f64;
            signal[symbol][day] = tradable[symbol][day] && score[symbol][day].is_finite();
        }
    }
    Ok(Signals {
        rsi: score,
        indicator_name: format!("mom{}_skip{}", cfg.mom_window, cfg.skip_window),
        signal,
        exit_now: vec![vec![false; n_rows]; panel.symbols.len()],
        add_signal: None,
        atr: None,
        stop_pct: None,
        tradable: tradable.clone(),
        pool: tradable,
        trade_weights: None,
        entry_mat: None,
        exit_mat: None,
        benchmark_returns: None,
    })
}

pub fn s041_trades(
    panel: &Panel,
    sig: &Signals,
    cfg: &Config,
    fill: &str,
) -> Result<Vec<crate::engine::Trade>, String> {
    if cfg.rebalance != "monthly" {
        return Err(format!(
            "S041 currently supports monthly rebalancing, got {:?}",
            cfg.rebalance
        ));
    }
    let breadth = if cfg.breadth_mode == "top" {
        cfg.breadth
    } else {
        if !cfg.decile.is_finite() || !(0.0..=1.0).contains(&cfg.decile) || cfg.decile == 0.0 {
            return Err("S041 decile must be in (0, 1]".into());
        }
        let mut eligible: Vec<_> = (0..panel.dates.len())
            .filter(|&day| {
                let date = panel.dates[day];
                day + 1 == panel.dates.len()
                    || date.year() != panel.dates[day + 1].year()
                    || date.month() != panel.dates[day + 1].month()
            })
            .map(|day| {
                (0..panel.symbols.len())
                    .filter(|&symbol| sig.tradable[symbol][day] && sig.rsi[symbol][day].is_finite())
                    .count()
            })
            .filter(|count| *count > 0)
            .collect();
        eligible.sort_unstable();
        let median = if eligible.is_empty() {
            0.0
        } else {
            let mid = eligible.len() / 2;
            if eligible.len() % 2 == 0 {
                (eligible[mid - 1] + eligible[mid]) as f64 / 2.0
            } else {
                eligible[mid] as f64
            }
        };
        cfg.min_breadth.max((median * cfg.decile) as usize)
    };
    if breadth == 0 {
        return Err("S041 breadth must be greater than zero".into());
    }
    rebalance_trades(panel, sig, cfg, fill, breadth)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> Config {
        let path = format!(
            "{}/configs/s001_rsi2_connors.yaml",
            env!("CARGO_MANIFEST_DIR").replace('\\', "/")
        );
        crate::config::load_config(
            "s001_rsi2_connors",
            Some(&path),
            &crate::config::Cli {
                data_dir: None,
                symbols: None,
            },
            None,
        )
        .unwrap()
    }

    #[test]
    fn s042_monthly_rotation_uses_trailing_returns_and_selects_top_k() {
        let n = 320;
        let mut cfg = test_config();
        cfg.min_history = 1;
        cfg.breadth = 1;
        cfg.signal = "mom12".into();
        cfg.rebalance = "monthly".into();
        cfg.delisting_return = -0.30;
        let mut a = vec![100.0; n];
        for value in a.iter_mut().skip(252) {
            *value = 200.0;
        }
        let panel = Panel {
            dates: (0..n)
                .map(|day| {
                    crate::data::parse_date("2023-01-01").unwrap()
                        + chrono::Duration::days(day as i64)
                })
                .collect(),
            symbols: vec!["A".into(), "B".into()],
            open: vec![a.clone(), vec![100.0; n]],
            high: vec![a.clone(), vec![100.0; n]],
            low: vec![a.clone(), vec![100.0; n]],
            close: vec![a, vec![100.0; n]],
            volume: vec![vec![1000.0; n]; 2],
        };
        let sig = compute_s042_signals(&panel, &cfg).unwrap();
        assert_eq!(sig.rsi[0][252], 1.0);
        assert_eq!(sig.rsi[1][252], 0.0);
        let trades = s042_trades(&panel, &sig, &cfg, "close").unwrap();
        assert!(!trades.is_empty());
        assert!(trades.iter().all(|trade| trade.sym_idx == 0));
        assert!(trades
            .windows(2)
            .all(|pair| pair[0].signal_idx < pair[1].signal_idx));
    }

    #[test]
    fn s033_breakout_limits_stocks_by_prior_liquidity_rank() {
        let mut cfg = test_config();
        cfg.etf_symbols = "ETF".into();
        cfg.include_stocks = true;
        cfg.universe_top_n = 1;
        cfg.rank_window = 2;
        cfg.rank_min_observations = 1;
        cfg.min_price = 5.0;
        cfg.min_history = 1;
        cfg.entry_window = 2;
        cfg.exit_window = 2;
        let panel = Panel {
            dates: (0..5)
                .map(|day| {
                    crate::data::parse_date("2024-01-01").unwrap()
                        + chrono::Duration::days(day as i64)
                })
                .collect(),
            symbols: vec!["ETF".into(), "STOCK".into(), "LOW".into()],
            open: vec![vec![10.0; 5], vec![10.0; 5], vec![2.0; 5]],
            high: vec![vec![11.0; 5], vec![12.0; 5], vec![2.0; 5]],
            low: vec![vec![9.0; 5], vec![9.0; 5], vec![2.0; 5]],
            close: vec![
                vec![10.0, 10.0, 10.0, 10.0, 10.0],
                vec![10.0, 10.0, 11.0, 12.0, 12.0],
                vec![2.0; 5],
            ],
            volume: vec![vec![10.0; 5], vec![10.0; 5], vec![1000.0; 5]],
        };
        let sig = compute_s033_signals(&panel, &cfg).unwrap();
        assert!(sig.signal[1][2]);
        assert!(sig.tradable[0][2]);
        assert!(!sig.tradable[2][2]);
    }

    #[test]
    fn s041_uses_skipped_momentum_and_monthly_top_selection() {
        let n = 120;
        let mut cfg = test_config();
        cfg.min_history = 1;
        cfg.rank_window = 2;
        cfg.rank_min_observations = 1;
        cfg.min_price = 0.0;
        cfg.universe_top_n = 2;
        cfg.mom_window = 3;
        cfg.skip_window = 1;
        cfg.breadth_mode = "top".into();
        cfg.breadth = 1;
        cfg.book_slots = 1;
        cfg.rebalance = "monthly".into();
        let mut a = vec![10.0; n];
        for day in 1..n {
            a[day] = a[day - 1] * 1.01;
        }
        let panel = Panel {
            dates: (0..n)
                .map(|day| {
                    crate::data::parse_date("2024-01-01").unwrap()
                        + chrono::Duration::days(day as i64)
                })
                .collect(),
            symbols: vec!["A".into(), "B".into()],
            open: vec![a.clone(), vec![20.0; n]],
            high: vec![a.clone(), vec![20.0; n]],
            low: vec![a.clone(), vec![20.0; n]],
            close: vec![a, vec![20.0; n]],
            volume: vec![vec![1000.0; n]; 2],
        };
        let sig = compute_s041_signals(&panel, &cfg).unwrap();
        assert!(sig.rsi[0][3] > 0.0);
        assert_eq!(sig.rsi[1][3], 0.0);
        let trades = s041_trades(&panel, &sig, &cfg, "close").unwrap();
        assert!(!trades.is_empty());
        assert!(trades.iter().all(|trade| trade.sym_idx == 0));
    }

    fn strategy_config(name: &str) -> Config {
        let path = format!(
            "{}/configs/{name}.yaml",
            env!("CARGO_MANIFEST_DIR").replace('\\', "/")
        );
        crate::config::load_config(
            name,
            Some(&path),
            &crate::config::Cli {
                data_dir: None,
                symbols: None,
            },
            None,
        )
        .unwrap()
    }

    fn s019_panel(next_day_low: f64, next_day_high: f64) -> Panel {
        let n = 23;
        let mut open = vec![100.0; n];
        let mut high = vec![101.0; n];
        let mut low = vec![99.0; n];
        let mut close = vec![100.0; n];
        let mut volume = vec![100.0; n];
        open[20] = 103.0;
        high[20] = 104.0;
        low[20] = 102.0;
        close[20] = 102.5;
        volume[20] = 250.0;
        open[21] = 103.0;
        high[21] = next_day_high;
        low[21] = next_day_low;
        close[21] = 105.0;
        Panel {
            dates: (0..n)
                .map(|row| {
                    crate::data::parse_date("2024-01-01").unwrap()
                        + chrono::Duration::days(row as i64)
                })
                .collect(),
            symbols: vec!["X".into()],
            open: vec![open],
            high: vec![high],
            low: vec![low],
            close: vec![close],
            volume: vec![volume],
        }
    }

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
    fn s019_requires_prior_atr_and_completed_day_volume() {
        let panel = s019_panel(101.0, 108.0);
        let cfg = strategy_config("s019_gap_and_go");
        let sig = compute_s019_signals(&panel, &cfg).unwrap();
        assert!(sig.signal[0][20]);
        assert!((sig.rsi[0][20] - 1.5).abs() < 1e-12);
        assert!(!sig.signal[0][19]);

        let mut threshold_cfg = cfg;
        threshold_cfg.gap_volume_multiple = 2.5;
        let threshold = compute_s019_signals(&panel, &threshold_cfg).unwrap();
        assert!(!threshold.signal[0][20]);
    }

    #[test]
    fn s019_exits_at_target_and_resolves_same_bar_stop_target_conservatively() {
        let cfg = strategy_config("s019_gap_and_go");
        let panel = s019_panel(101.0, 108.0);
        let sig = compute_s019_signals(&panel, &cfg).unwrap();
        let trades = s019_trades(&panel, &sig, &cfg, "close").unwrap();
        assert_eq!(trades.len(), 1);
        assert_eq!(trades[0].exit_reason, "target");
        assert_eq!(trades[0].exit_idx, 21);
        assert!((trades[0].exit_px - 107.5).abs() < 1e-12);

        let panel = s019_panel(99.0, 110.0);
        let sig = compute_s019_signals(&panel, &cfg).unwrap();
        let trades = s019_trades(&panel, &sig, &cfg, "close").unwrap();
        assert_eq!(trades[0].exit_reason, "stop");
        assert_eq!(trades[0].exit_px, 100.0);
    }

    #[test]
    fn s019_does_not_overlap_positions_in_the_same_symbol() {
        let panel = s019_panel(101.0, 108.0);
        let mut cfg = strategy_config("s019_gap_and_go");
        cfg.gap_reward_risk = 100.0;
        let mut sig = compute_s019_signals(&panel, &cfg).unwrap();
        sig.signal[0][21] = true;
        let trades = s019_trades(&panel, &sig, &cfg, "close").unwrap();
        assert_eq!(trades.len(), 1);
        assert_eq!(trades[0].exit_idx, 22);
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
            volume: vec![vec![1.0; 205]],
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

    #[test]
    fn s021_uses_close_to_next_open_and_only_completed_sma_data() {
        let n = 60;
        let close: Vec<f64> = (0..n).map(|row| 100.0 + row as f64).collect();
        let mut open = close.clone();
        for day in 1..n {
            open[day] = close[day - 1] * 1.01;
        }
        let panel = Panel {
            dates: (0..n)
                .map(|row| {
                    crate::data::parse_date("2024-01-01").unwrap()
                        + chrono::Duration::days(row as i64)
                })
                .collect(),
            symbols: vec!["SPY".into()],
            open: vec![open],
            high: vec![close.clone()],
            low: vec![close.clone()],
            close: vec![close],
            volume: vec![vec![1.0; n]],
        };
        let mut cfg = strategy_config("s021_index_overnight");
        cfg.participation_filter = "none".into();
        let unfiltered = compute_s021_signals(&panel, &cfg).unwrap();
        assert!(unfiltered.signal[0][0]);
        assert_eq!(unfiltered.trade_weights.as_ref().unwrap()[0][1], 1.0);
        assert!((unfiltered.benchmark_returns.as_ref().unwrap()[1] - 0.01).abs() < 1e-12);
        assert_eq!(unfiltered.entry_mat.as_ref().unwrap()[0][1], 100.0);
        assert_eq!(unfiltered.exit_mat.as_ref().unwrap()[0][1], 101.0);

        cfg.participation_filter = "sma50".into();
        let filtered = compute_s021_signals(&panel, &cfg).unwrap();
        assert!(!filtered.signal[0][48]);
        assert!(filtered.signal[0][49]);
    }

    #[test]
    fn s105_sizes_from_trailing_overnight_volatility() {
        let n = 35;
        let close = vec![100.0; n];
        let mut open = close.clone();
        for day in 1..n {
            let ret = if day % 2 == 0 { 0.01 } else { -0.01 };
            open[day] = close[day - 1] * (1.0 + ret);
        }
        let panel = Panel {
            dates: (0..n)
                .map(|row| {
                    crate::data::parse_date("2024-01-01").unwrap()
                        + chrono::Duration::days(row as i64)
                })
                .collect(),
            symbols: vec!["SPY".into()],
            open: vec![open.clone()],
            high: vec![close.clone()],
            low: vec![close.clone()],
            close: vec![close],
            volume: vec![vec![1.0; n]],
        };
        let cfg = strategy_config("s105_overnight_carry");
        let sig = compute_s105_signals(&panel, &cfg).unwrap();
        let sample: Vec<f64> = (1..=20).map(|day| open[day] / 100.0 - 1.0).collect();
        let mean = sample.iter().sum::<f64>() / sample.len() as f64;
        let std = (sample
            .iter()
            .map(|value| (value - mean).powi(2))
            .sum::<f64>()
            / (sample.len() - 1) as f64)
            .sqrt();
        let expected = (cfg.target_vol / (std * 252.0f64.sqrt())).min(cfg.max_leverage);
        let weights = sig.trade_weights.as_ref().unwrap();
        assert_eq!(weights[0][20], 0.0);
        assert!((weights[0][21] - expected).abs() < 1e-12);

        let mut changed = panel.clone();
        for day in 22..n {
            changed.open[0][day] *= 1.5;
        }
        let future = compute_s105_signals(&changed, &cfg).unwrap();
        assert_eq!(&weights[0][..22], &future.trade_weights.unwrap()[0][..22]);
    }
}
