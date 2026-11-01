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
                borrow_cost_per_bar: 0.0,
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
                borrow_cost_per_bar: 0.0,
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
    let trades = rebalance_trades(panel, sig, cfg, fill, cfg.breadth)?;
    Ok(weight_rebalance_trades(trades, cfg.book_slots))
}

fn weight_rebalance_trades(
    mut trades: Vec<crate::engine::Trade>,
    slots: usize,
) -> Vec<crate::engine::Trade> {
    let mut counts = std::collections::BTreeMap::<usize, usize>::new();
    for trade in &trades {
        *counts.entry(trade.signal_idx).or_default() += 1;
    }
    for trade in &mut trades {
        let weight = 1.0 / counts[&trade.signal_idx].max(slots).max(1) as f64;
        trade.weight = weight;
        trade.entry_turnover = weight;
        trade.exit_turnover = weight;
        trade.weighted = true;
    }
    trades
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
            if entry > last
                || (!is_open && entry + 1 == n_rows)
                || (is_open && !open[entry].is_finite())
            {
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
                borrow_cost_per_bar: 0.0,
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

pub fn compute_s032_signals(panel: &Panel, cfg: &Config) -> Result<Signals, String> {
    if cfg.entry_window == 0 || cfg.universe_top_n == 0 {
        return Err("S032 requires positive high-window and universe sizes".into());
    }
    if !matches!(cfg.selection_mode.as_str(), "rank" | "floor") {
        return Err(format!(
            "unknown S032 selection_mode {:?}",
            cfg.selection_mode
        ));
    }
    if !cfg.ratio_floor.is_finite() || !(0.0..=1.0).contains(&cfg.ratio_floor) {
        return Err("S032 ratio_floor must be finite and in [0, 1]".into());
    }
    let columns: Vec<_> = (0..panel.symbols.len()).collect();
    let ranks = if cfg.rank_cache_dir.trim().is_empty() {
        crate::data::liquidity_ranks(
            panel,
            &columns,
            cfg.rank_window,
            cfg.rank_min_observations,
            cfg.min_price,
            cfg.min_history,
        )?
    } else {
        crate::data::read_liquid_rank_cache(&cfg.rank_cache_dir, &panel.dates, &panel.symbols)?
    };
    let n_rows = panel.dates.len();
    let mut proximity = vec![vec![f64::NAN; n_rows]; panel.symbols.len()];
    let mut signal = vec![vec![false; n_rows]; panel.symbols.len()];
    let mut tradable = vec![vec![false; n_rows]; panel.symbols.len()];
    for symbol in 0..panel.symbols.len() {
        let close = &panel.close[symbol];
        let high = rolling_extreme(close, cfg.entry_window, false);
        for day in 0..n_rows {
            if !close[day].is_finite()
                || !ranks[symbol][day].is_finite()
                || ranks[symbol][day] > cfg.universe_top_n as f64
                || !high[day].is_finite()
                || high[day] <= 0.0
            {
                continue;
            }
            tradable[symbol][day] = true;
            let ratio = close[day] / high[day];
            if cfg.selection_mode != "floor" || ratio >= cfg.ratio_floor {
                proximity[symbol][day] = ratio;
                signal[symbol][day] = true;
            }
        }
    }
    Ok(Signals {
        rsi: proximity,
        indicator_name: format!("close_to_{}d_high", cfg.entry_window),
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

fn monthly_rebalance_rows(panel: &Panel) -> Vec<usize> {
    (0..panel.dates.len())
        .filter(|&day| {
            day + 1 == panel.dates.len()
                || panel.dates[day].year() != panel.dates[day + 1].year()
                || panel.dates[day].month() != panel.dates[day + 1].month()
        })
        .collect()
}

fn cross_sectional_decile_breadth(sig: &Signals, cfg: &Config) -> Result<usize, String> {
    if !cfg.decile.is_finite() || !(0.0..=1.0).contains(&cfg.decile) || cfg.decile == 0.0 {
        return Err("cross-sectional decile must be in (0, 1]".into());
    }
    let n_rows = sig.rsi.first().map_or(0, Vec::len);
    let mut eligible: Vec<_> = (0..n_rows)
        .map(|day| {
            (0..sig.rsi.len())
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
    Ok(cfg.min_breadth.max((median * cfg.decile) as usize))
}

pub fn s032_trades(
    panel: &Panel,
    sig: &Signals,
    cfg: &Config,
    fill: &str,
) -> Result<Vec<crate::engine::Trade>, String> {
    if cfg.rebalance != "monthly" {
        return Err(format!(
            "S032 currently supports monthly rebalancing, got {:?}",
            cfg.rebalance
        ));
    }
    if cfg.holding_months == 0 {
        return Err("S032 holding_months must be greater than zero".into());
    }
    if cfg.holding_months == 1 && cfg.formation_skip_months == 0 {
        let breadth = if cfg.selection_mode == "floor" {
            usize::MAX
        } else {
            cross_sectional_decile_breadth(sig, cfg)?
        };
        return rebalance_trades(panel, sig, cfg, fill, breadth);
    }
    if !matches!(fill, "close" | "next_open") {
        return Err(format!("S032 does not support fill {fill:?}"));
    }
    let rows = monthly_rebalance_rows(panel);
    if rows.is_empty() {
        return Ok(Vec::new());
    }
    let breadth = if cfg.selection_mode == "floor" {
        usize::MAX
    } else {
        cross_sectional_decile_breadth(sig, cfg)?
    };
    let mut trades = Vec::new();
    for (period, &formation_day) in rows.iter().enumerate() {
        let entry_period = period + cfg.formation_skip_months;
        if entry_period >= rows.len() {
            continue;
        }
        let exit_period = entry_period.saturating_add(cfg.holding_months);
        let censored = exit_period >= rows.len();
        let exit_day = if censored {
            *rows.last().unwrap()
        } else {
            rows[exit_period]
        };
        let mut picks: Vec<_> = (0..panel.symbols.len())
            .filter(|&symbol| {
                sig.tradable[symbol][formation_day] && sig.rsi[symbol][formation_day].is_finite()
            })
            .collect();
        picks.sort_by(|&a, &b| {
            sig.rsi[b][formation_day]
                .total_cmp(&sig.rsi[a][formation_day])
                .then_with(|| panel.symbols[a].cmp(&panel.symbols[b]))
        });
        picks.truncate(breadth);
        let entry_signal_day = rows[entry_period];
        for symbol in picks {
            let close = &panel.close[symbol];
            let open = &panel.open[symbol];
            let Some(last) = close.iter().rposition(|value| value.is_finite()) else {
                continue;
            };
            let is_open = fill == "next_open";
            let entry = if is_open {
                entry_signal_day + 1
            } else {
                entry_signal_day
            };
            if entry > last || (is_open && !open[entry].is_finite()) {
                continue;
            }
            let scheduled_exit = if is_open { exit_day + 1 } else { exit_day };
            let delisted = last < scheduled_exit && last + 1 < panel.dates.len();
            let exit = if scheduled_exit <= last {
                scheduled_exit
            } else {
                last
            };
            let entry_px = if is_open {
                open[entry]
            } else {
                close[entry_signal_day]
            };
            let mut exit_px = if is_open && scheduled_exit <= last && open[exit].is_finite() {
                open[exit]
            } else {
                close[exit]
            };
            if delisted {
                exit_px *= 1.0 + cfg.delisting_return;
            }
            if !entry_px.is_finite() || !exit_px.is_finite() || exit < entry {
                continue;
            }
            let cost = cfg.cost_for_symbol(&panel.symbols[symbol]);
            trades.push(crate::engine::Trade {
                sym_idx: symbol,
                signal_idx: formation_day,
                entry_idx: entry,
                exit_idx: exit,
                entry_px,
                exit_px,
                ret: exit_px * (1.0 - cost) / (entry_px * (1.0 + cost)) - 1.0,
                ret_gross: exit_px / entry_px - 1.0,
                exit_reason: if delisted {
                    "delisted"
                } else if censored {
                    "censored"
                } else {
                    "rebalance"
                },
                hold: exit - entry,
                signal_value: sig.rsi[symbol][formation_day],
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

fn rolling_std_current(values: &[f64], window: usize) -> Vec<f64> {
    let mut result = vec![f64::NAN; values.len()];
    if window < 2 {
        return result;
    }
    let mut prefix = vec![[0.0; 3]; values.len() + 1];
    for day in 0..values.len() {
        prefix[day + 1] = prefix[day];
        if values[day].is_finite() {
            prefix[day + 1][0] += 1.0;
            prefix[day + 1][1] += values[day];
            prefix[day + 1][2] += values[day] * values[day];
        }
    }
    for day in window - 1..values.len() {
        let start = day + 1 - window;
        let count = prefix[day + 1][0] - prefix[start][0];
        if count < window as f64 {
            continue;
        }
        let sum = prefix[day + 1][1] - prefix[start][1];
        let sum_squares = prefix[day + 1][2] - prefix[start][2];
        let variance = ((sum_squares - sum * sum / count) / (count - 1.0)).max(0.0);
        result[day] = variance.sqrt();
    }
    result
}

fn s035_shrunk_betas(panel: &Panel, cfg: &Config) -> Result<Mat, String> {
    if cfg.beta_window < 2 || cfg.beta_min_obs < 2 || cfg.beta_min_obs > cfg.beta_window {
        return Err("S035 beta window and minimum observations are invalid".into());
    }
    let benchmark = panel
        .symbols
        .iter()
        .position(|symbol| symbol == &cfg.benchmark)
        .ok_or_else(|| {
            format!(
                "S035 low-beta mode requires benchmark {} in the panel",
                cfg.benchmark
            )
        })?;
    let returns: Mat = panel
        .close
        .iter()
        .map(|close| {
            (0..close.len())
                .map(|day| {
                    if day == 0
                        || !close[day].is_finite()
                        || !close[day - 1].is_finite()
                        || close[day - 1] <= 0.0
                    {
                        f64::NAN
                    } else {
                        close[day] / close[day - 1] - 1.0
                    }
                })
                .collect()
        })
        .collect();
    Ok(returns
        .iter()
        .map(|stock| {
            rolling_beta_lagged(
                stock,
                &returns[benchmark],
                cfg.beta_window,
                cfg.beta_min_obs,
            )
            .into_iter()
            .map(|beta| {
                if beta.is_finite() {
                    0.6 * beta + 0.4
                } else {
                    f64::NAN
                }
            })
            .collect()
        })
        .collect())
}

pub fn compute_s035_signals(panel: &Panel, cfg: &Config) -> Result<Signals, String> {
    if !matches!(cfg.strategy_mode.as_str(), "low_vol" | "low_beta") {
        return Err(format!(
            "unknown S035 strategy_mode {:?}",
            cfg.strategy_mode
        ));
    }
    if cfg.vol_window < 2 || cfg.universe_top_n == 0 {
        return Err("S035 requires vol_window >= 2 and a positive universe size".into());
    }
    if cfg.sector_neutral && cfg.simfin_dir.trim().is_empty() {
        return Err("S035 sector-neutral mode requires simfin_dir".into());
    }
    let columns: Vec<_> = (0..panel.symbols.len()).collect();
    let ranks = if cfg.rank_cache_dir.trim().is_empty() {
        crate::data::liquidity_ranks(
            panel,
            &columns,
            cfg.rank_window,
            cfg.rank_min_observations,
            cfg.min_price,
            cfg.min_history,
        )?
    } else {
        crate::data::read_liquid_rank_cache(&cfg.rank_cache_dir, &panel.dates, &panel.symbols)?
    };
    let beta = if cfg.strategy_mode == "low_beta" {
        Some(s035_shrunk_betas(panel, cfg)?)
    } else {
        None
    };
    let excluded: std::collections::BTreeSet<_> = crate::config::split_csv(&cfg.etf_symbols)
        .into_iter()
        .collect();
    let mut sectors = vec![None; panel.symbols.len()];
    if cfg.sector_neutral {
        let (_, groups) = simfin_groups(&panel.symbols, &cfg.simfin_dir)?;
        sectors = groups;
    }
    let n_rows = panel.dates.len();
    let mut score = vec![vec![f64::NAN; n_rows]; panel.symbols.len()];
    let mut tradable = vec![vec![false; n_rows]; panel.symbols.len()];
    for symbol in 0..panel.symbols.len() {
        if excluded.contains(&panel.symbols[symbol])
            || (cfg.strategy_mode == "low_beta" && panel.symbols[symbol] == cfg.benchmark)
        {
            continue;
        }
        let volatility = if beta.is_none() {
            let close = &panel.close[symbol];
            let returns: Vec<_> = (0..n_rows)
                .map(|day| {
                    if day == 0
                        || !close[day].is_finite()
                        || !close[day - 1].is_finite()
                        || close[day - 1] <= 0.0
                    {
                        f64::NAN
                    } else {
                        close[day] / close[day - 1] - 1.0
                    }
                })
                .collect();
            Some(rolling_std_current(&returns, cfg.vol_window))
        } else {
            None
        };
        for day in 0..n_rows {
            if !panel.close[symbol][day].is_finite()
                || !ranks[symbol][day].is_finite()
                || ranks[symbol][day] > cfg.universe_top_n as f64
                || (cfg.sector_neutral && sectors[symbol].is_none())
            {
                continue;
            }
            let value = if let Some(betas) = &beta {
                betas[symbol][day]
            } else {
                volatility.as_ref().unwrap()[day] * 252.0_f64.sqrt()
            };
            if value.is_finite() {
                tradable[symbol][day] = true;
                score[symbol][day] = -value;
            }
        }
    }
    Ok(Signals {
        rsi: score.clone(),
        indicator_name: if beta.is_some() {
            "shrunk_252d_beta".into()
        } else {
            format!("annualized_vol_{}", cfg.vol_window)
        },
        signal: score
            .iter()
            .map(|series| series.iter().map(|value| value.is_finite()).collect())
            .collect(),
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

pub fn s035_trades(
    panel: &Panel,
    sig: &Signals,
    cfg: &Config,
    fill: &str,
) -> Result<Vec<crate::engine::Trade>, String> {
    if cfg.rebalance != "monthly" {
        return Err(format!(
            "S035 currently supports monthly rebalancing, got {:?}",
            cfg.rebalance
        ));
    }
    if !cfg.sector_neutral && cfg.strategy_mode == "low_vol" {
        return rebalance_trades(
            panel,
            sig,
            cfg,
            fill,
            cross_sectional_decile_breadth(sig, cfg)?,
        );
    }
    if !matches!(fill, "close" | "next_open") {
        return Err(format!("S035 does not support fill {fill:?}"));
    }
    let rows = monthly_rebalance_rows(panel);
    if rows.is_empty() {
        return Ok(Vec::new());
    }
    let beta = if cfg.strategy_mode == "low_beta" {
        Some(s035_shrunk_betas(panel, cfg)?)
    } else {
        None
    };
    let groups = if cfg.sector_neutral {
        Some(simfin_groups(&panel.symbols, &cfg.simfin_dir)?.1)
    } else {
        None
    };
    let default_breadth = cross_sectional_decile_breadth(sig, cfg)?;
    let mut trades = Vec::new();
    for (period, &signal_day) in rows.iter().enumerate() {
        let boundary = rows
            .get(period + 1)
            .copied()
            .unwrap_or(panel.dates.len() - 1);
        let mut candidates: Vec<_> = (0..panel.symbols.len())
            .filter(|&symbol| {
                sig.tradable[symbol][signal_day] && sig.rsi[symbol][signal_day].is_finite()
            })
            .collect();
        let selected: Vec<(usize, Option<String>)> = if let Some(sectors) = &groups {
            let mut by_sector = std::collections::BTreeMap::<String, Vec<usize>>::new();
            for symbol in candidates.drain(..) {
                if let Some(sector) = &sectors[symbol] {
                    by_sector.entry(sector.clone()).or_default().push(symbol);
                }
            }
            let mut picks = Vec::new();
            for (sector, symbols) in by_sector.iter_mut() {
                symbols.sort_by(|&a, &b| {
                    sig.rsi[b][signal_day]
                        .total_cmp(&sig.rsi[a][signal_day])
                        .then_with(|| panel.symbols[a].cmp(&panel.symbols[b]))
                });
                let count = ((symbols.len() as f64 * cfg.decile).ceil() as usize).max(1);
                picks.extend(
                    symbols
                        .iter()
                        .take(count)
                        .map(|&symbol| (symbol, Some(sector.clone()))),
                );
            }
            picks
        } else {
            candidates.sort_by(|&a, &b| {
                sig.rsi[b][signal_day]
                    .total_cmp(&sig.rsi[a][signal_day])
                    .then_with(|| panel.symbols[a].cmp(&panel.symbols[b]))
            });
            candidates.truncate(default_breadth);
            candidates
                .into_iter()
                .map(|symbol| (symbol, None))
                .collect()
        };
        if selected.is_empty() {
            continue;
        }
        let sector_counts = selected.iter().fold(
            std::collections::BTreeMap::<String, usize>::new(),
            |mut counts, (_, sector)| {
                if let Some(sector) = sector {
                    *counts.entry(sector.clone()).or_default() += 1;
                }
                counts
            },
        );
        let n_sectors = sector_counts.len().max(1) as f64;
        let base_weights: Vec<_> = selected
            .iter()
            .map(|(_, sector)| {
                if let Some(sector) = sector {
                    1.0 / (n_sectors * *sector_counts.get(sector).unwrap() as f64)
                } else {
                    1.0 / selected.len() as f64
                }
            })
            .collect();
        let portfolio_beta = beta.as_ref().map(|betas| {
            selected
                .iter()
                .zip(&base_weights)
                .map(|((symbol, _), weight)| weight * betas[*symbol][signal_day])
                .sum::<f64>()
        });
        let scale = if let Some(value) = portfolio_beta {
            if !value.is_finite() || value <= 0.0 {
                continue;
            }
            1.0 / value
        } else {
            1.0
        };
        for ((symbol, _), base_weight) in selected.into_iter().zip(base_weights) {
            let close = &panel.close[symbol];
            let open = &panel.open[symbol];
            let Some(last) = close.iter().rposition(|value| value.is_finite()) else {
                continue;
            };
            let is_open = fill == "next_open";
            let entry = if is_open { signal_day + 1 } else { signal_day };
            if entry > last || (is_open && !open[entry].is_finite()) {
                continue;
            }
            let delisted = last < boundary && last + 1 < panel.dates.len();
            let mut exit = if is_open {
                boundary.saturating_add(1).min(last)
            } else {
                boundary.min(last)
            };
            let reason = if delisted {
                "delisted"
            } else if period + 1 == rows.len() {
                "censored"
            } else {
                "rebalance"
            };
            let entry_px = if is_open {
                open[entry]
            } else {
                close[signal_day]
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
            if !entry_px.is_finite() || !exit_px.is_finite() || exit < entry {
                continue;
            }
            let weight = base_weight * scale;
            let cost = cfg.cost_for_symbol(&panel.symbols[symbol]);
            let weighted = cfg.sector_neutral || beta.is_some();
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
                weight: if weighted { weight } else { 1.0 },
                entry_turnover: if weighted { weight } else { 1.0 },
                exit_turnover: if weighted { weight } else { 1.0 },
                weighted,
                borrow_cost_per_bar: 0.0,
            });
        }
    }
    trades.sort_by(|a, b| {
        (a.entry_idx, &panel.symbols[a.sym_idx]).cmp(&(b.entry_idx, &panel.symbols[b.sym_idx]))
    });
    Ok(trades)
}

pub fn compute_s041_signals(panel: &Panel, cfg: &Config) -> Result<Signals, String> {
    if cfg.mom_window == 0 || cfg.skip_window >= cfg.mom_window {
        return Err("S041 requires mom_window > skip_window >= 0".into());
    }
    if !matches!(cfg.breadth_mode.as_str(), "decile" | "top") {
        return Err(format!("unknown S041 breadth_mode {:?}", cfg.breadth_mode));
    }
    let ranks = if cfg.rank_cache_dir.trim().is_empty() {
        let columns: Vec<_> = (0..panel.symbols.len()).collect();
        crate::data::liquidity_ranks(
            panel,
            &columns,
            cfg.rank_window,
            cfg.rank_min_observations,
            cfg.min_price,
            cfg.min_history,
        )?
    } else {
        crate::data::read_liquid_rank_cache(&cfg.rank_cache_dir, &panel.dates, &panel.symbols)?
    };
    let n_rows = panel.dates.len();
    let mut score = vec![vec![f64::NAN; n_rows]; panel.symbols.len()];
    let mut signal = vec![vec![false; n_rows]; panel.symbols.len()];
    let mut tradable = vec![vec![false; n_rows]; panel.symbols.len()];
    let industry = if cfg.industry_neutral {
        Some(simfin_groups(&panel.symbols, &cfg.simfin_dir)?.0)
    } else {
        None
    };
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
        }
    }
    if let Some(industry) = industry {
        let mut members = std::collections::BTreeMap::<String, Vec<usize>>::new();
        for (symbol, group) in industry.into_iter().enumerate() {
            if let Some(group) = group {
                members.entry(group).or_default().push(symbol);
            }
        }
        for symbols in members.values() {
            for day in 0..n_rows {
                let eligible: Vec<_> = symbols
                    .iter()
                    .copied()
                    .filter(|&symbol| tradable[symbol][day] && score[symbol][day].is_finite())
                    .collect();
                if eligible.is_empty() {
                    continue;
                }
                let average = eligible
                    .iter()
                    .map(|&symbol| score[symbol][day])
                    .sum::<f64>()
                    / eligible.len() as f64;
                for symbol in eligible {
                    score[symbol][day] -= average;
                }
            }
        }
    }
    for symbol in 0..panel.symbols.len() {
        for day in 0..n_rows {
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
    let trades = rebalance_trades(panel, sig, cfg, fill, breadth)?;
    Ok(weight_rebalance_trades(trades, cfg.book_slots))
}

fn simfin_groups(
    symbols: &[String],
    root: &str,
) -> Result<(Vec<Option<String>>, Vec<Option<String>>), String> {
    if root.trim().is_empty() {
        return Err("industry/peer grouping requires simfin_dir".into());
    }
    let root = std::path::Path::new(root);
    let companies_path = root.join("us-companies.csv");
    let industries_path = root.join("industries.csv");
    let mut industries_reader = csv::ReaderBuilder::new()
        .delimiter(b';')
        .from_path(&industries_path)
        .map_err(|error| format!("{}: {error}", industries_path.display()))?;
    let industry_headers = industries_reader
        .headers()
        .map_err(|error| format!("{}: {error}", industries_path.display()))?
        .clone();
    let industry_id = industry_headers
        .iter()
        .position(|value| value == "IndustryId")
        .ok_or_else(|| format!("{}: missing IndustryId column", industries_path.display()))?;
    let industry_name = industry_headers
        .iter()
        .position(|value| value == "Industry")
        .ok_or_else(|| format!("{}: missing Industry column", industries_path.display()))?;
    let sector_name = industry_headers
        .iter()
        .position(|value| value == "Sector")
        .ok_or_else(|| format!("{}: missing Sector column", industries_path.display()))?;
    let mut group_by_id = std::collections::HashMap::new();
    for record in industries_reader.records() {
        let record = record.map_err(|error| format!("{}: {error}", industries_path.display()))?;
        if let (Some(id), Some(industry), Some(sector)) = (
            record.get(industry_id),
            record.get(industry_name),
            record.get(sector_name),
        ) {
            group_by_id.insert(id.to_string(), (industry.to_string(), sector.to_string()));
        }
    }

    let mut companies_reader = csv::ReaderBuilder::new()
        .delimiter(b';')
        .from_path(&companies_path)
        .map_err(|error| format!("{}: {error}", companies_path.display()))?;
    let company_headers = companies_reader
        .headers()
        .map_err(|error| format!("{}: {error}", companies_path.display()))?
        .clone();
    let ticker_col = company_headers
        .iter()
        .position(|value| value == "Ticker")
        .ok_or_else(|| format!("{}: missing Ticker column", companies_path.display()))?;
    let company_industry = company_headers
        .iter()
        .position(|value| value == "IndustryId")
        .ok_or_else(|| format!("{}: missing IndustryId column", companies_path.display()))?;
    let mut industry = std::collections::HashMap::new();
    let mut sector = std::collections::HashMap::new();
    for record in companies_reader.records() {
        let record = record.map_err(|error| format!("{}: {error}", companies_path.display()))?;
        let (Some(ticker), Some(id)) = (record.get(ticker_col), record.get(company_industry))
        else {
            continue;
        };
        if let Some((industry_name, sector_name)) = group_by_id.get(id) {
            let ticker = ticker.trim().to_ascii_uppercase();
            industry.insert(ticker.clone(), industry_name.clone());
            sector.insert(ticker, sector_name.clone());
        }
    }
    Ok((
        symbols
            .iter()
            .map(|symbol| industry.get(&symbol.to_ascii_uppercase()).cloned())
            .collect(),
        symbols
            .iter()
            .map(|symbol| sector.get(&symbol.to_ascii_uppercase()).cloned())
            .collect(),
    ))
}

fn peer_returns(
    returns: &[Vec<f64>],
    peer_member: &[Vec<bool>],
    groups: &[Option<String>],
    min_peers: usize,
) -> Vec<Vec<f64>> {
    let n_symbols = returns.len();
    let n_rows = returns.first().map_or(0, Vec::len);
    let mut result = vec![vec![f64::NAN; n_rows]; n_symbols];
    let mut groups_to_columns: std::collections::BTreeMap<&str, Vec<usize>> =
        std::collections::BTreeMap::new();
    for (symbol, group) in groups.iter().enumerate() {
        if let Some(group) = group.as_deref() {
            groups_to_columns.entry(group).or_default().push(symbol);
        }
    }
    for columns in groups_to_columns.values() {
        for day in 0..n_rows {
            let (mut sum, mut count) = (0.0, 0usize);
            for &symbol in columns {
                if peer_member[symbol][day] && returns[symbol][day].is_finite() {
                    sum += returns[symbol][day];
                    count += 1;
                }
            }
            for &symbol in columns {
                let own = usize::from(peer_member[symbol][day] && returns[symbol][day].is_finite());
                let peers = count - own;
                if peers >= min_peers {
                    result[symbol][day] =
                        (sum - if own == 1 { returns[symbol][day] } else { 0.0 }) / peers as f64;
                }
            }
        }
    }
    result
}

fn rolling_beta_lagged(
    stock: &[f64],
    peer: &[f64],
    window: usize,
    min_observations: usize,
) -> Vec<f64> {
    let n = stock.len();
    let mut prefix = vec![[0.0; 5]; n + 1];
    for day in 0..n {
        prefix[day + 1] = prefix[day];
        if stock[day].is_finite() && peer[day].is_finite() {
            prefix[day + 1][0] += 1.0;
            prefix[day + 1][1] += peer[day];
            prefix[day + 1][2] += stock[day];
            prefix[day + 1][3] += peer[day] * peer[day];
            prefix[day + 1][4] += peer[day] * stock[day];
        }
    }
    let mut beta = vec![f64::NAN; n];
    for day in 0..n {
        let start = (day + 1).saturating_sub(window);
        let count = prefix[day + 1][0] - prefix[start][0];
        if count < min_observations as f64 {
            continue;
        }
        let sx = prefix[day + 1][1] - prefix[start][1];
        let sy = prefix[day + 1][2] - prefix[start][2];
        let sxx = prefix[day + 1][3] - prefix[start][3];
        let sxy = prefix[day + 1][4] - prefix[start][4];
        let variance = sxx - sx * sx / count;
        if variance > 1e-12 {
            beta[day] = (sxy - sx * sy / count) / variance;
        }
    }
    let mut lagged = vec![f64::NAN; n];
    lagged[1..].copy_from_slice(&beta[..n - 1]);
    lagged
}

fn rolling_std_lagged(values: &[f64], window: usize, min_observations: usize) -> Vec<f64> {
    let n = values.len();
    let mut prefix = vec![[0.0; 3]; n + 1];
    for day in 0..n {
        prefix[day + 1] = prefix[day];
        if values[day].is_finite() {
            prefix[day + 1][0] += 1.0;
            prefix[day + 1][1] += values[day];
            prefix[day + 1][2] += values[day] * values[day];
        }
    }
    let mut result = vec![f64::NAN; n];
    for day in 1..n {
        let start = day.saturating_sub(window);
        let count = prefix[day][0] - prefix[start][0];
        if count < min_observations as f64 || count < 2.0 {
            continue;
        }
        let sum = prefix[day][1] - prefix[start][1];
        let sum_squares = prefix[day][2] - prefix[start][2];
        let variance = ((sum_squares - sum * sum / count) / (count - 1.0)).max(0.0);
        result[day] = variance.sqrt();
    }
    result
}

pub fn compute_s075_signals(panel: &Panel, cfg: &Config) -> Result<Signals, String> {
    if !matches!(cfg.peer_level.as_str(), "industry" | "sector") {
        return Err(format!("unknown S075 peer_level {:?}", cfg.peer_level));
    }
    if !matches!(cfg.signal_mode.as_str(), "raw" | "zscore")
        || !matches!(cfg.exit_mode.as_str(), "resid" | "zscore" | "fixed")
        || !matches!(cfg.regime_mode.as_str(), "none" | "ew_drawdown" | "ew_vol")
    {
        return Err("S075 has an unsupported signal, exit, or regime mode".into());
    }
    if cfg.beta_window == 0
        || cfg.beta_min_obs < 2
        || cfg.beta_min_obs > cfg.beta_window
        || cfg.resid_window == 0
        || cfg.min_peers == 0
        || cfg.universe_top_n == 0
        || cfg.peer_top_n < cfg.universe_top_n
        || cfg.z_window == 0
        || (cfg.regime_mode != "none" && cfg.regime_window < 2)
        || !(0.0..=1.0).contains(&cfg.decile)
        || cfg.decile == 0.0
    {
        return Err("S075 has invalid window, universe, peer, or decile settings".into());
    }
    let (industries, sectors) = simfin_groups(&panel.symbols, &cfg.simfin_dir)?;
    let n_symbols = panel.symbols.len();
    let n_rows = panel.dates.len();
    let columns: Vec<_> = (0..n_symbols).collect();
    let ranks = crate::data::liquidity_ranks(
        panel,
        &columns,
        cfg.rank_window,
        cfg.rank_min_observations,
        cfg.min_price,
        cfg.min_history,
    )?;
    let mut tradable = vec![vec![false; n_rows]; n_symbols];
    let mut peer_pool = vec![vec![false; n_rows]; n_symbols];
    for symbol in 0..n_symbols {
        for day in 0..n_rows {
            tradable[symbol][day] =
                ranks[symbol][day].is_finite() && ranks[symbol][day] <= cfg.universe_top_n as f64;
            peer_pool[symbol][day] =
                ranks[symbol][day].is_finite() && ranks[symbol][day] <= cfg.peer_top_n as f64;
        }
    }
    let mut returns = vec![vec![f64::NAN; n_rows]; n_symbols];
    for symbol in 0..n_symbols {
        for day in 1..n_rows {
            let (current, previous) = (panel.close[symbol][day], panel.close[symbol][day - 1]);
            if current.is_finite() && previous.is_finite() && previous > 0.0 {
                returns[symbol][day] = current / previous - 1.0;
            }
        }
    }
    let sector_peer = peer_returns(&returns, &peer_pool, &sectors, cfg.min_peers);
    let peer = if cfg.peer_level == "sector" {
        sector_peer
    } else {
        let industry_peer = peer_returns(&returns, &peer_pool, &industries, cfg.min_peers);
        (0..n_symbols)
            .map(|symbol| {
                (0..n_rows)
                    .map(|day| {
                        if industry_peer[symbol][day].is_finite() {
                            industry_peer[symbol][day]
                        } else {
                            sector_peer[symbol][day]
                        }
                    })
                    .collect()
            })
            .collect()
    };

    let mut residual = vec![vec![f64::NAN; n_rows]; n_symbols];
    let mut zscore = vec![vec![f64::NAN; n_rows]; n_symbols];
    for symbol in 0..n_symbols {
        let beta = rolling_beta_lagged(
            &returns[symbol],
            &peer[symbol],
            cfg.beta_window,
            cfg.beta_min_obs,
        );
        let daily_residual: Vec<_> = (0..n_rows)
            .map(|day| {
                if returns[symbol][day].is_finite()
                    && beta[day].is_finite()
                    && peer[symbol][day].is_finite()
                {
                    returns[symbol][day] - beta[day] * peer[symbol][day]
                } else {
                    f64::NAN
                }
            })
            .collect();
        let sigma = rolling_std_lagged(&daily_residual, cfg.z_window, 40);
        for day in cfg.resid_window..n_rows {
            let (current, previous) = (
                panel.close[symbol][day],
                panel.close[symbol][day - cfg.resid_window],
            );
            if !current.is_finite() || !previous.is_finite() || previous <= 0.0 {
                continue;
            }
            let peer_log_return: Option<f64> = (day + 1 - cfg.resid_window..=day)
                .map(|row| {
                    (peer[symbol][row].is_finite() && peer[symbol][row] > -1.0)
                        .then_some((1.0 + peer[symbol][row]).ln())
                })
                .collect::<Option<Vec<_>>>()
                .map(|values| values.iter().sum());
            let Some(peer_log_return) = peer_log_return else {
                continue;
            };
            let beta_value = beta[day];
            if !beta_value.is_finite() {
                continue;
            }
            let stock_return = current / previous - 1.0;
            let peer_return = peer_log_return.exp_m1();
            residual[symbol][day] = stock_return - beta_value * peer_return;
            if sigma[day].is_finite() && sigma[day] > 0.0 {
                zscore[symbol][day] =
                    residual[symbol][day] / (sigma[day] * (cfg.resid_window as f64).sqrt());
            }
        }
    }
    let score = if cfg.signal_mode == "zscore" {
        &zscore
    } else {
        &residual
    };
    let mut sma_ok = vec![vec![true; n_rows]; n_symbols];
    if cfg.sma_window > 0 {
        for symbol in 0..n_symbols {
            let sma = rolling_mean(&panel.close[symbol], cfg.sma_window);
            for day in 0..n_rows {
                sma_ok[symbol][day] =
                    panel.close[symbol][day].is_finite() && panel.close[symbol][day] > sma[day];
            }
        }
    }
    let mut signal = vec![vec![false; n_rows]; n_symbols];
    for day in 0..n_rows {
        let mut values: Vec<_> = (0..n_symbols)
            .filter(|&symbol| tradable[symbol][day] && score[symbol][day].is_finite())
            .map(|symbol| score[symbol][day])
            .collect();
        if values.is_empty() {
            continue;
        }
        values.sort_by(f64::total_cmp);
        let quantile_index = (values.len() - 1) as f64 * cfg.decile;
        let lower = quantile_index.floor() as usize;
        let upper = quantile_index.ceil() as usize;
        let fraction = quantile_index - lower as f64;
        let cutoff = values[lower] * (1.0 - fraction) + values[upper] * fraction;
        for symbol in 0..n_symbols {
            let min_resid_ok =
                cfg.min_abs_resid <= 0.0 || residual[symbol][day] <= -cfg.min_abs_resid;
            signal[symbol][day] = tradable[symbol][day]
                && score[symbol][day].is_finite()
                && score[symbol][day] <= cutoff
                && sma_ok[symbol][day]
                && min_resid_ok;
        }
    }
    if cfg.skip_days > 0 {
        let original = signal.clone();
        signal.fill(vec![false; n_rows]);
        for symbol in 0..n_symbols {
            for day in cfg.skip_days..n_rows {
                signal[symbol][day] = original[symbol][day - cfg.skip_days]
                    && tradable[symbol][day]
                    && sma_ok[symbol][day];
            }
        }
    }
    let mut b_day = vec![0.0; n_rows];
    for day in 0..n_rows {
        let (mut sum, mut count) = (0.0, 0usize);
        for symbol in 0..n_symbols {
            if tradable[symbol][day] && returns[symbol][day].is_finite() {
                sum += returns[symbol][day];
                count += 1;
            }
        }
        if count > 0 {
            b_day[day] = sum / count as f64;
        }
    }
    match cfg.regime_mode.as_str() {
        "none" => {}
        "ew_drawdown" => {
            for day in cfg.regime_window.saturating_sub(1)..n_rows {
                let window = &b_day[day + 1 - cfg.regime_window..=day];
                let total = window
                    .iter()
                    .map(|value| value.ln_1p())
                    .sum::<f64>()
                    .exp_m1();
                if total < -cfg.regime_threshold {
                    signal.iter_mut().for_each(|series| series[day] = false);
                }
            }
        }
        "ew_vol" => {
            for day in cfg.regime_window.saturating_sub(1)..n_rows {
                let window = &b_day[day + 1 - cfg.regime_window..=day];
                let mean = window.iter().sum::<f64>() / window.len() as f64;
                let variance = window
                    .iter()
                    .map(|value| (value - mean).powi(2))
                    .sum::<f64>()
                    / (window.len() - 1).max(1) as f64;
                if variance.sqrt() * 252.0f64.sqrt() > cfg.regime_threshold {
                    signal.iter_mut().for_each(|series| series[day] = false);
                }
            }
        }
        _ => unreachable!("regime mode validated above"),
    }
    let mut exit_now = vec![vec![false; n_rows]; n_symbols];
    if cfg.exit_mode == "resid" {
        for symbol in 0..n_symbols {
            for day in 0..n_rows {
                exit_now[symbol][day] = residual[symbol][day] > 0.0;
            }
        }
    } else if cfg.exit_mode == "zscore" {
        for symbol in 0..n_symbols {
            for day in 0..n_rows {
                exit_now[symbol][day] = zscore[symbol][day] > cfg.exit_z;
            }
        }
    }
    Ok(Signals {
        rsi: residual,
        indicator_name: "resid".into(),
        signal,
        exit_now,
        add_signal: None,
        atr: Some(zscore),
        stop_pct: None,
        tradable: tradable.clone(),
        pool: tradable,
        trade_weights: None,
        entry_mat: None,
        exit_mat: None,
        benchmark_returns: Some(b_day),
    })
}

pub fn s075_trades(
    panel: &Panel,
    sig: &Signals,
    cfg: &Config,
    fill: &str,
) -> Result<Vec<crate::engine::Trade>, String> {
    if !matches!(fill, "close" | "next_open") {
        return Err(format!("S075 does not support fill {fill:?}"));
    }
    if sig.atr.is_none() {
        return Err("S075 signals are missing residual z-scores".into());
    }
    let hold_cap = if cfg.exit_mode == "fixed" {
        cfg.fixed_hold
    } else {
        cfg.max_hold
    };
    if hold_cap == 0 {
        return Err("S075 hold cap must be greater than zero".into());
    }
    let n_rows = panel.dates.len();
    let is_open = fill == "next_open";
    let mut trades = Vec::new();
    for symbol in 0..panel.symbols.len() {
        let close = &panel.close[symbol];
        let open = &panel.open[symbol];
        let Some(last) = close.iter().rposition(|value| value.is_finite()) else {
            continue;
        };
        let cost = cfg.cost_for_symbol(&panel.symbols[symbol]);
        let mut previous_decision = None;
        for day in 0..n_rows {
            if !sig.signal[symbol][day]
                || matches!(previous_decision, Some(previous) if day <= previous)
            {
                continue;
            }
            let entry = if is_open { day + 1 } else { day };
            if entry > last || (is_open && !open[entry].is_finite()) {
                continue;
            }
            let entry_px = if is_open { open[entry] } else { close[day] };
            if !entry_px.is_finite() || entry_px <= 0.0 {
                continue;
            }
            let mut decision = day.saturating_add(hold_cap).min(last);
            let mut reason = if day + hold_cap > last {
                if last + 1 < n_rows {
                    "delisted"
                } else {
                    "censored"
                }
            } else {
                "cap"
            };
            for holding_day in (day + 1)..=day.saturating_add(hold_cap).min(last) {
                if !close[holding_day].is_finite() {
                    continue;
                }
                if cfg.stop_loss > 0.0 && close[holding_day] <= entry_px * (1.0 - cfg.stop_loss) {
                    decision = holding_day;
                    reason = "stop";
                    break;
                }
                if cfg.take_profit > 0.0 && close[holding_day] >= entry_px * (1.0 + cfg.take_profit)
                {
                    decision = holding_day;
                    reason = "target";
                    break;
                }
                if cfg.exit_mode != "fixed" && sig.exit_now[symbol][holding_day] {
                    decision = holding_day;
                    reason = if cfg.exit_mode == "zscore" {
                        "zscore"
                    } else {
                        "resid"
                    };
                    break;
                }
            }
            previous_decision = Some(decision);
            let mut exit = decision;
            while exit > day && !close[exit].is_finite() {
                exit -= 1;
            }
            let mut exit_px = close[exit];
            if is_open
                && reason != "delisted"
                && reason != "censored"
                && exit + 1 <= last
                && open[exit + 1].is_finite()
            {
                exit += 1;
                exit_px = open[exit];
            }
            if reason == "delisted" {
                exit_px *= 1.0 + cfg.delisting_return;
            }
            if !exit_px.is_finite() || exit < entry {
                continue;
            }
            trades.push(crate::engine::Trade {
                sym_idx: symbol,
                signal_idx: day,
                entry_idx: entry,
                exit_idx: exit,
                entry_px,
                exit_px,
                ret: exit_px * (1.0 - cost) / (entry_px * (1.0 + cost)) - 1.0,
                ret_gross: exit_px / entry_px - 1.0,
                exit_reason: reason,
                hold: exit - entry,
                signal_value: -sig.rsi[symbol][day],
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

fn normal_upper_tail(value: f64) -> f64 {
    let x = value.abs();
    let t = 1.0 / (1.0 + 0.231_641_9 * x);
    let density = (-0.5 * x * x).exp() / (2.0 * std::f64::consts::PI).sqrt();
    let tail = density
        * t
        * (0.319_381_53
            + t * (-0.356_563_782
                + t * (1.781_477_937 + t * (-1.821_255_978 + t * 1.330_274_429))));
    if value >= 0.0 {
        tail
    } else {
        1.0 - tail
    }
}

pub fn compute_s017_signals(
    panel: &Panel,
    cfg: &Config,
    actions: &[crate::data::CorporateAction],
) -> Result<Signals, String> {
    if !matches!(
        cfg.formation_cadence.as_str(),
        "annual" | "quarterly" | "monthly"
    ) || !matches!(
        cfg.holding_mode.as_str(),
        "overnight_only" | "full_day" | "overnight_intraday"
    ) || !matches!(cfg.extra_filter.as_str(), "none" | "negative_intraday")
    {
        return Err("S017 has an unsupported formation, holding, or filter mode".into());
    }
    if cfg.formation_window == 0
        || cfg.universe_top_n == 0
        || !cfg.fdr_q.is_finite()
        || !(0.0..=1.0).contains(&cfg.fdr_q)
        || !cfg.top_fraction.is_finite()
        || !(0.0..=1.0).contains(&cfg.top_fraction)
        || cfg.top_fraction == 0.0
    {
        return Err("S017 has invalid formation, universe, FDR, or top-fraction settings".into());
    }
    let n_symbols = panel.symbols.len();
    let n_rows = panel.dates.len();
    let benchmark_symbol = if cfg.benchmark.is_empty() {
        "SPY"
    } else {
        cfg.benchmark.as_str()
    };
    let benchmark_idx = panel
        .symbols
        .iter()
        .position(|symbol| symbol == benchmark_symbol)
        .ok_or_else(|| format!("S017 benchmark {benchmark_symbol:?} is missing from the panel"))?;
    let stock_columns: Vec<_> = (0..n_symbols)
        .filter(|&symbol| symbol != benchmark_idx)
        .collect();
    let ranks = crate::data::liquidity_ranks(
        panel,
        &stock_columns,
        cfg.rank_window,
        cfg.rank_min_observations,
        cfg.min_price,
        cfg.min_history,
    )?;
    let mut date_row = std::collections::HashMap::new();
    for (row, date) in panel.dates.iter().enumerate() {
        date_row.insert(*date, row);
    }
    let symbol_col: std::collections::HashMap<_, _> = panel
        .symbols
        .iter()
        .enumerate()
        .map(|(column, symbol)| (symbol.to_ascii_uppercase(), column))
        .collect();
    let mut dividend = vec![vec![0.0; n_rows]; n_symbols];
    let mut split_factor = vec![vec![1.0; n_rows]; n_symbols];
    for action in actions {
        let Some(&symbol) = symbol_col.get(&action.symbol.to_ascii_uppercase()) else {
            continue;
        };
        let Some(&day) = date_row.get(&action.date) else {
            continue;
        };
        match action.action_type.as_str() {
            "cash_dividend" => dividend[symbol][day] += action.rate.unwrap_or(0.0),
            "forward_split" | "reverse_split" | "unit_split" => {
                if let (Some(old_rate), Some(new_rate)) = (action.old_rate, action.new_rate) {
                    if old_rate > 0.0 && new_rate > 0.0 {
                        split_factor[symbol][day] *= new_rate / old_rate;
                    }
                }
            }
            _ => {}
        }
    }
    let mut overnight_entry = vec![vec![f64::NAN; n_rows]; n_symbols];
    let mut overnight_exit = vec![vec![f64::NAN; n_rows]; n_symbols];
    let mut overnight = vec![vec![f64::NAN; n_rows]; n_symbols];
    let mut intraday = vec![vec![f64::NAN; n_rows]; n_symbols];
    for symbol in 0..n_symbols {
        for day in 0..n_rows {
            let open = panel.open[symbol][day];
            if open.is_finite() && panel.close[symbol][day].is_finite() && open > 0.0 {
                intraday[symbol][day] = panel.close[symbol][day] / open - 1.0;
            }
            if day == 0 {
                continue;
            }
            let previous_close = panel.close[symbol][day - 1];
            if !previous_close.is_finite() || previous_close <= 0.0 {
                continue;
            }
            overnight_entry[symbol][day] = previous_close;
            let effective_open = (open + dividend[symbol][day]) * split_factor[symbol][day];
            if effective_open.is_finite() && effective_open > 0.0 {
                overnight_exit[symbol][day] = effective_open;
                overnight[symbol][day] = effective_open / previous_close - 1.0;
            }
        }
    }
    let last_rows: Vec<_> = panel
        .close
        .iter()
        .map(|close| close.iter().rposition(|value| value.is_finite()))
        .collect();
    let mut delisted = vec![vec![false; n_rows]; n_symbols];
    for day in 1..n_rows {
        for symbol in stock_columns.iter().copied() {
            if last_rows[symbol] == Some(day - 1) && overnight_entry[symbol][day].is_finite() {
                delisted[symbol][day] = true;
                overnight[symbol][day] = cfg.delisting_return;
                overnight_exit[symbol][day] =
                    overnight_entry[symbol][day] * (1.0 + cfg.delisting_return);
            }
        }
    }

    let mut formation_rows = Vec::new();
    for day in 0..n_rows {
        let month_end = day + 1 == n_rows
            || panel.dates[day].year() != panel.dates[day + 1].year()
            || panel.dates[day].month() != panel.dates[day + 1].month();
        if !month_end {
            continue;
        }
        let date = panel.dates[day];
        let eligible_month = match cfg.formation_cadence.as_str() {
            "annual" => date.month() == 12,
            "quarterly" => matches!(date.month(), 3 | 6 | 9 | 12),
            _ => true,
        };
        if !eligible_month {
            continue;
        }
        if day + 1 == n_rows {
            let next_month = if date.month() == 12 {
                chrono::NaiveDate::from_ymd_opt(date.year() + 1, 1, 1)
            } else {
                chrono::NaiveDate::from_ymd_opt(date.year(), date.month() + 1, 1)
            };
            if next_month
                .and_then(|next| next.pred_opt())
                .is_some_and(|last_day| date.day() < last_day.day().saturating_sub(3))
            {
                continue;
            }
        }
        formation_rows.push(day);
    }

    let mut selections = Vec::new();
    for formation in formation_rows {
        if formation < cfg.formation_window {
            continue;
        }
        let start = formation + 1 - cfg.formation_window;
        let eligible: Vec<_> = stock_columns
            .iter()
            .copied()
            .filter(|&symbol| {
                ranks[symbol][formation].is_finite()
                    && ranks[symbol][formation] <= cfg.universe_top_n as f64
            })
            .collect();
        let mut means = vec![f64::NAN; n_symbols];
        let mut t_stats = vec![f64::NAN; n_symbols];
        let mut p_values = vec![f64::NAN; n_symbols];
        let mut valid = Vec::new();
        for symbol in eligible {
            let sample: Vec<_> = (start..=formation)
                .map(|day| overnight[symbol][day])
                .filter(|value| value.is_finite())
                .collect();
            if sample.len() < cfg.formation_window || sample.len() < 2 {
                continue;
            }
            let mean = sample.iter().sum::<f64>() / sample.len() as f64;
            let variance = sample
                .iter()
                .map(|value| (value - mean).powi(2))
                .sum::<f64>()
                / (sample.len() - 1) as f64;
            if !variance.is_finite() || variance <= 0.0 {
                continue;
            }
            let t_stat = mean / (variance.sqrt() / (sample.len() as f64).sqrt());
            if !t_stat.is_finite() {
                continue;
            }
            means[symbol] = mean;
            t_stats[symbol] = t_stat;
            p_values[symbol] = normal_upper_tail(t_stat);
            valid.push(symbol);
        }
        if valid.is_empty() {
            continue;
        }
        let mut ordered = valid.clone();
        ordered.sort_by(|&a, &b| p_values[a].total_cmp(&p_values[b]).then_with(|| a.cmp(&b)));
        let cutoff = ordered
            .iter()
            .enumerate()
            .filter(|(rank, symbol)| {
                p_values[**symbol] <= cfg.fdr_q * (*rank + 1) as f64 / ordered.len() as f64
            })
            .map(|(_, symbol)| p_values[*symbol])
            .last();
        let Some(cutoff) = cutoff else {
            continue;
        };
        let mut by_mean = valid.clone();
        by_mean.sort_by(|&a, &b| means[a].total_cmp(&means[b]).then_with(|| a.cmp(&b)));
        let top_count = ((valid.len() as f64 * cfg.top_fraction).ceil() as usize).max(1);
        let top: std::collections::BTreeSet<_> =
            by_mean.into_iter().rev().take(top_count).collect();
        let mut selected = vec![false; n_symbols];
        for symbol in valid {
            let sample = (start..=formation)
                .filter_map(|day| {
                    intraday[symbol][day]
                        .is_finite()
                        .then_some(intraday[symbol][day])
                })
                .collect::<Vec<_>>();
            let negative_intraday =
                !sample.is_empty() && sample.iter().sum::<f64>() / (sample.len() as f64) < 0.0;
            selected[symbol] = p_values[symbol] <= cutoff
                && t_stats[symbol] > cfg.min_t_stat
                && means[symbol] > 0.0
                && top.contains(&symbol)
                && (cfg.extra_filter != "negative_intraday" || negative_intraday);
        }
        selections.push((formation, selected));
    }
    let mut membership = vec![vec![false; n_rows]; n_symbols];
    for index in 0..selections.len() {
        let (formation, selected) = &selections[index];
        let stop = selections
            .get(index + 1)
            .map(|(next, _)| *next)
            .unwrap_or(n_rows);
        for day in *formation..stop {
            for symbol in 0..n_symbols {
                membership[symbol][day] = selected[symbol];
            }
        }
    }
    let mut tradable = vec![vec![false; n_rows]; n_symbols];
    for symbol in stock_columns.iter().copied() {
        for day in 0..n_rows {
            tradable[symbol][day] =
                ranks[symbol][day].is_finite() && ranks[symbol][day] <= cfg.universe_top_n as f64;
        }
    }
    let mut trade_weights = vec![vec![0.0; n_rows]; n_symbols];
    let mut entry_mat = vec![vec![f64::NAN; n_rows]; n_symbols];
    let mut exit_mat = vec![vec![f64::NAN; n_rows]; n_symbols];
    let mut signal = vec![vec![false; n_rows]; n_symbols];
    let mut benchmark = vec![0.0; n_rows];
    for day in 1..n_rows {
        let candidates: Vec<_> = stock_columns
            .iter()
            .copied()
            .filter(|&symbol| tradable[symbol][day - 1] && overnight[symbol][day].is_finite())
            .collect();
        if candidates.is_empty() {
            continue;
        }
        let universe_overnight = candidates
            .iter()
            .map(|&symbol| overnight[symbol][day])
            .sum::<f64>()
            / candidates.len() as f64;
        let universe_intraday: Vec<_> = candidates
            .iter()
            .filter_map(|&symbol| {
                intraday[symbol][day]
                    .is_finite()
                    .then_some(intraday[symbol][day])
            })
            .collect();
        let mean_intraday = if universe_intraday.is_empty() {
            0.0
        } else {
            universe_intraday.iter().sum::<f64>() / universe_intraday.len() as f64
        };
        benchmark[day] = match cfg.holding_mode.as_str() {
            "full_day" => mean_intraday,
            "overnight_intraday" => 0.5 * (universe_overnight - mean_intraday),
            _ => universe_overnight,
        };
        let active_return: Vec<_> = candidates
            .iter()
            .copied()
            .filter(|&symbol| match cfg.holding_mode.as_str() {
                "full_day" => intraday[symbol][day].is_finite(),
                "overnight_intraday" => intraday[symbol][day].is_finite(),
                _ => overnight[symbol][day].is_finite(),
            })
            .filter(|&symbol| membership[symbol][day - 1])
            .collect();
        if active_return.is_empty() {
            continue;
        }
        let weight = 1.0 / active_return.len() as f64;
        for symbol in active_return {
            let (entry, exit) = match cfg.holding_mode.as_str() {
                "full_day" => (panel.open[symbol][day], panel.close[symbol][day]),
                "overnight_intraday" => (1.0, 1.0 + overnight[symbol][day] - intraday[symbol][day]),
                _ => (overnight_entry[symbol][day], overnight_exit[symbol][day]),
            };
            if !entry.is_finite() || !exit.is_finite() || entry <= 0.0 || exit <= 0.0 {
                continue;
            }
            signal[symbol][day - 1] = true;
            trade_weights[symbol][day] = weight;
            entry_mat[symbol][day] = entry;
            exit_mat[symbol][day] = exit;
        }
    }
    let values = vec![vec![f64::NAN; n_rows]; n_symbols];
    Ok(Signals {
        rsi: values,
        indicator_name: "overnight_return".into(),
        signal,
        exit_now: delisted,
        add_signal: None,
        atr: None,
        stop_pct: None,
        tradable: tradable.clone(),
        pool: tradable,
        trade_weights: Some(trade_weights),
        entry_mat: Some(entry_mat),
        exit_mat: Some(exit_mat),
        benchmark_returns: Some(benchmark),
    })
}

pub fn s017_trades(
    panel: &Panel,
    sig: &Signals,
    cfg: &Config,
    fill: &str,
) -> Result<Vec<crate::engine::Trade>, String> {
    if fill != "close_to_open" {
        return Err(format!("S017 requires close_to_open fills, got {fill:?}"));
    }
    let (Some(weights), Some(entries), Some(exits)) =
        (&sig.trade_weights, &sig.entry_mat, &sig.exit_mat)
    else {
        return Err("S017 signals are missing portfolio weights or price matrices".into());
    };
    let (turnover, exit_reason) = if cfg.holding_mode == "overnight_intraday" {
        (2.0, "overnight_intraday")
    } else if cfg.holding_mode == "full_day" {
        (1.0, "full_day")
    } else {
        (1.0, "overnight")
    };
    let mut trades = Vec::new();
    for symbol in 0..panel.symbols.len() {
        let cost = cfg.cost_for_symbol(&panel.symbols[symbol]);
        for day in 1..panel.dates.len() {
            let weight = weights[symbol][day];
            let (entry_px, exit_px) = (entries[symbol][day], exits[symbol][day]);
            if weight <= 0.0 || !weight.is_finite() || !entry_px.is_finite() || !exit_px.is_finite()
            {
                continue;
            }
            let gross = weight * (exit_px / entry_px - 1.0);
            trades.push(crate::engine::Trade {
                sym_idx: symbol,
                signal_idx: day - 1,
                entry_idx: day,
                exit_idx: day,
                entry_px,
                exit_px,
                ret: gross - cost * turnover * weight * 2.0,
                ret_gross: gross,
                exit_reason: if sig
                    .exit_now
                    .get(symbol)
                    .and_then(|series| series.get(day))
                    .copied()
                    .unwrap_or(false)
                {
                    "delisted"
                } else {
                    exit_reason
                },
                hold: 1,
                signal_value: f64::NAN,
                weight,
                entry_turnover: turnover * weight,
                exit_turnover: turnover * weight,
                weighted: true,
                borrow_cost_per_bar: 0.0,
            });
        }
    }
    trades.sort_by(|a, b| {
        (a.entry_idx, &panel.symbols[a.sym_idx]).cmp(&(b.entry_idx, &panel.symbols[b.sym_idx]))
    });
    Ok(trades)
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
    fn s032_monthly_proximity_selects_the_closest_names_and_supports_holding_cohorts() {
        let n = 150;
        let dates: Vec<_> = (0..n)
            .map(|day| {
                crate::data::parse_date("2024-01-01").unwrap() + chrono::Duration::days(day as i64)
            })
            .collect();
        let mut below_high = vec![100.0; n];
        for day in 1..n {
            let next_month = day + 1 == n
                || dates[day].year() != dates[day + 1].year()
                || dates[day].month() != dates[day + 1].month();
            if next_month && day > 0 {
                below_high[day - 1] = 100.0;
                below_high[day] = 80.0;
            }
        }
        let panel = Panel {
            dates,
            symbols: vec!["A".into(), "B".into()],
            open: vec![vec![100.0; n], below_high.clone()],
            high: vec![vec![100.0; n], below_high.clone()],
            low: vec![vec![100.0; n], below_high.clone()],
            close: vec![vec![100.0; n], below_high.clone()],
            volume: vec![vec![1000.0; n]; 2],
        };
        let mut cfg = test_config();
        cfg.min_history = 1;
        cfg.rank_window = 2;
        cfg.rank_min_observations = 1;
        cfg.min_price = 0.0;
        cfg.universe_top_n = 2;
        cfg.entry_window = 2;
        cfg.decile = 0.5;
        cfg.min_breadth = 1;
        cfg.book_slots = 1;
        cfg.rebalance = "monthly".into();
        cfg.selection_mode = "rank".into();
        let sig = compute_s032_signals(&panel, &cfg).unwrap();
        let trades = s032_trades(&panel, &sig, &cfg, "close").unwrap();
        assert!(!trades.is_empty());
        assert!(trades.iter().all(|trade| trade.sym_idx == 0));

        cfg.selection_mode = "floor".into();
        cfg.ratio_floor = 0.9;
        let floor_sig = compute_s032_signals(&panel, &cfg).unwrap();
        let first_formation = monthly_rebalance_rows(&panel)[0];
        assert!(floor_sig.signal[0][first_formation]);
        assert!(!floor_sig.signal[1][first_formation]);

        cfg.selection_mode = "rank".into();
        cfg.holding_months = 2;
        cfg.formation_skip_months = 1;
        let trades = s032_trades(&panel, &sig, &cfg, "close").unwrap();
        assert!(trades
            .iter()
            .any(|trade| trade.entry_idx > trade.signal_idx));
    }

    #[test]
    fn s035_selects_low_volatility_and_beta_scales_the_low_beta_variant() {
        let n = 150;
        let dates: Vec<_> = (0..n)
            .map(|day| {
                crate::data::parse_date("2024-01-01").unwrap() + chrono::Duration::days(day as i64)
            })
            .collect();
        let mut benchmark = vec![100.0; n];
        let mut volatile = vec![100.0; n];
        for day in 1..n {
            benchmark[day] = benchmark[day - 1] * if day % 2 == 0 { 1.01 } else { 0.99 };
            volatile[day] = volatile[day - 1] * if day % 2 == 0 { 1.02 } else { 0.98 };
        }
        let panel = Panel {
            dates,
            symbols: vec!["A".into(), "B".into(), "SPY".into()],
            open: vec![vec![100.0; n], volatile.clone(), benchmark.clone()],
            high: vec![vec![100.0; n], volatile.clone(), benchmark.clone()],
            low: vec![vec![100.0; n], volatile.clone(), benchmark.clone()],
            close: vec![vec![100.0; n], volatile.clone(), benchmark.clone()],
            volume: vec![vec![1000.0; n], vec![1000.0; n], vec![1.0; n]],
        };
        let mut cfg = test_config();
        cfg.min_history = 1;
        cfg.rank_window = 2;
        cfg.rank_min_observations = 1;
        cfg.min_price = 0.0;
        cfg.universe_top_n = 2;
        cfg.vol_window = 5;
        cfg.decile = 0.5;
        cfg.min_breadth = 1;
        cfg.book_slots = 1;
        cfg.rebalance = "monthly".into();
        cfg.strategy_mode = "low_vol".into();
        cfg.etf_symbols = "SPY".into();
        let sig = compute_s035_signals(&panel, &cfg).unwrap();
        let trades = s035_trades(&panel, &sig, &cfg, "close").unwrap();
        assert!(!trades.is_empty());
        assert!(trades.iter().all(|trade| trade.sym_idx == 0));

        cfg.strategy_mode = "low_beta".into();
        cfg.beta_window = 20;
        cfg.beta_min_obs = 10;
        cfg.benchmark = "SPY".into();
        let sig = compute_s035_signals(&panel, &cfg).unwrap();
        assert!(sig.rsi[0][50] > sig.rsi[1][50]);
        let trades = s035_trades(&panel, &sig, &cfg, "close").unwrap();
        assert!(trades.iter().all(|trade| trade.sym_idx == 0));
        assert!(trades.iter().all(|trade| trade.weight > 1.0));
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
        assert!(trades
            .iter()
            .all(|trade| trade.weighted && trade.weight == 1.0));
    }

    #[test]
    fn s075_uses_daily_peer_residuals_and_builds_non_overlapping_trades() {
        let root =
            std::env::temp_dir().join(format!("deepthrust_s075_simfin_{}", std::process::id()));
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(
            root.join("industries.csv"),
            "IndustryId;Industry;Sector\n1;Tech;Technology\n",
        )
        .unwrap();
        std::fs::write(
            root.join("us-companies.csv"),
            "Ticker;IndustryId\nA;1\nB;1\nC;1\nD;1\nE;1\n",
        )
        .unwrap();
        let n = 120;
        let symbols = vec!["A", "B", "C", "D", "E"];
        let mut close = vec![vec![100.0; n]; symbols.len()];
        for day in 1..n {
            for (symbol, series) in close.iter_mut().enumerate() {
                let daily = 0.001 + 0.01 * ((day * (symbol + 1)) as f64).sin();
                series[day] = series[day - 1] * (1.0 + daily);
            }
        }
        close[0][70] = close[0][69] * 0.65;
        for day in 71..n {
            close[0][day] = close[0][day - 1] * 1.002;
        }
        let mut cfg = test_config();
        cfg.simfin_dir = root.display().to_string();
        cfg.min_history = 1;
        cfg.rank_window = 10;
        cfg.rank_min_observations = 5;
        cfg.min_price = 5.0;
        cfg.universe_top_n = 5;
        cfg.peer_top_n = 5;
        cfg.min_peers = 2;
        cfg.beta_window = 20;
        cfg.beta_min_obs = 10;
        cfg.resid_window = 2;
        cfg.decile = 0.5;
        cfg.sma_window = 0;
        cfg.max_hold = 5;
        cfg.z_window = 20;
        cfg.exit_mode = "resid".into();
        let panel = Panel {
            dates: (0..n)
                .map(|day| {
                    crate::data::parse_date("2024-01-01").unwrap()
                        + chrono::Duration::days(day as i64)
                })
                .collect(),
            symbols: symbols.iter().map(|symbol| symbol.to_string()).collect(),
            open: close.clone(),
            high: close.clone(),
            low: close.clone(),
            close,
            volume: vec![vec![1000.0; n]; symbols.len()],
        };
        let sig = compute_s075_signals(&panel, &cfg).unwrap();
        assert!(sig
            .rsi
            .iter()
            .any(|series| series.iter().any(|value| value.is_finite())));
        assert!(sig
            .signal
            .iter()
            .any(|series| series.iter().any(|&value| value)));
        let trades = s075_trades(&panel, &sig, &cfg, "close").unwrap();
        assert!(!trades.is_empty());
        assert!(trades.iter().all(|trade| trade.exit_idx >= trade.entry_idx));
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn s017_forms_monthly_overnight_cohorts_and_adds_cash_dividends() {
        let n = 400;
        let mut cfg = test_config();
        cfg.benchmark = "SPY".into();
        cfg.min_history = 1;
        cfg.rank_window = 10;
        cfg.rank_min_observations = 5;
        cfg.universe_top_n = 10;
        cfg.formation_cadence = "monthly".into();
        cfg.formation_window = 60;
        cfg.min_t_stat = 2.0;
        cfg.fdr_q = 0.05;
        cfg.top_fraction = 0.20;
        cfg.holding_mode = "overnight_only".into();
        let symbols = ["A", "B", "C", "D", "E", "SPY"];
        let dates: Vec<_> = (0..n)
            .map(|day| {
                crate::data::parse_date("2020-01-01").unwrap() + chrono::Duration::days(day as i64)
            })
            .collect();
        let close = vec![vec![100.0; n]; symbols.len()];
        let mut open = vec![vec![100.0; n]; symbols.len()];
        for day in 1..n {
            open[0][day] = 100.0 * (1.002 + 0.0005 * (day as f64).sin());
            for symbol in 1..5 {
                open[symbol][day] = 100.0 * (1.0003 + 0.001 * ((day * (symbol + 1)) as f64).sin());
            }
        }
        let panel = Panel {
            dates: dates.clone(),
            symbols: symbols.iter().map(|symbol| symbol.to_string()).collect(),
            high: close.clone(),
            low: close.clone(),
            close,
            open,
            volume: vec![vec![1000.0; n]; symbols.len()],
        };
        let actions = [crate::data::CorporateAction {
            symbol: "A".into(),
            date: dates[100],
            action_type: "cash_dividend".into(),
            rate: Some(0.10),
            old_rate: None,
            new_rate: None,
        }];
        let sig = compute_s017_signals(&panel, &cfg, &actions).unwrap();
        assert!(sig.trade_weights.as_ref().unwrap()[0][100] > 0.0);
        assert!(sig.exit_mat.as_ref().unwrap()[0][100] > sig.entry_mat.as_ref().unwrap()[0][100]);
        assert!(sig.trade_weights.as_ref().unwrap()[1..5]
            .iter()
            .all(|series| series[100] == 0.0));
        let trades = s017_trades(&panel, &sig, &cfg, "close_to_open").unwrap();
        assert!(!trades.is_empty());
        assert!(trades.iter().all(|trade| trade.weighted));
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
