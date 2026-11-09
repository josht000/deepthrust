//! Family 3 cross-sectional short-term reversal studies.

use crate::config::Config;
use crate::data::Panel;
use crate::engine::Trade;
use crate::signals::{rolling_mean, wilder_rsi, Signals};

pub const S031_NAME: &str = "s031_up_streak_fade";
pub const S031_TITLE: &str = "S031 Up-Streak Fade (5+ Days)";

// S031: `streak_days`+ consecutive up closes in the `gate` regime -> `direction`
// book; cover when `exit_mode` fires or after `max_hold` sessions.
pub fn compute_s031_signals(panel: &Panel, cfg: &Config) -> Result<Signals, String> {
    if !matches!(cfg.direction.as_str(), "short" | "long") {
        return Err(format!("S031 direction must be short|long, got {:?}", cfg.direction));
    }
    if !matches!(cfg.gate.as_str(), "sma200" | "ret21" | "none") {
        return Err(format!("S031 gate must be sma200|ret21|none, got {:?}", cfg.gate));
    }
    if !matches!(
        cfg.exit_mode.as_str(),
        "sma_or_rsi" | "sma" | "rsi" | "fixed"
    ) {
        return Err(format!(
            "S031 exit_mode must be sma_or_rsi|sma|rsi|fixed, got {:?}",
            cfg.exit_mode
        ));
    }
    if cfg.streak_days == 0
        || cfg.exit_sma == 0
        || cfg.rsi_len == 0
        || cfg.max_hold == 0
        || cfg.universe_top_n == 0
        || (cfg.gate == "sma200" && cfg.sma_window == 0)
        || (cfg.gate == "ret21" && cfg.gate_ret_window == 0)
    {
        return Err("S031 has an invalid streak, exit, gate, hold, or universe setting".into());
    }
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
    for symbol in 0..n_symbols {
        for day in 0..n_rows {
            tradable[symbol][day] =
                ranks[symbol][day].is_finite() && ranks[symbol][day] <= cfg.universe_top_n as f64;
        }
    }
    let mut streak = vec![vec![0.0; n_rows]; n_symbols];
    let mut gate_ok = vec![vec![false; n_rows]; n_symbols];
    let mut exit_now = vec![vec![false; n_rows]; n_symbols];
    for symbol in 0..n_symbols {
        let close = &panel.close[symbol];
        for day in 1..n_rows {
            if close[day].is_finite() && close[day - 1].is_finite() && close[day] > close[day - 1] {
                streak[symbol][day] = streak[symbol][day - 1] + 1.0;
            }
        }
        match cfg.gate.as_str() {
            "sma200" => {
                let sma = rolling_mean(close, cfg.sma_window);
                for day in 0..n_rows {
                    gate_ok[symbol][day] =
                        close[day].is_finite() && close[day] < sma[day];
                }
            }
            "ret21" => {
                for day in cfg.gate_ret_window..n_rows {
                    let previous = close[day - cfg.gate_ret_window];
                    gate_ok[symbol][day] = close[day].is_finite()
                        && previous.is_finite()
                        && previous > 0.0
                        && close[day] / previous - 1.0 < 0.0;
                }
            }
            _ => {
                for day in 0..n_rows {
                    gate_ok[symbol][day] = close[day].is_finite();
                }
            }
        }
        if cfg.exit_mode == "fixed" {
            continue;
        }
        let sma = rolling_mean(close, cfg.exit_sma);
        let rsi = if cfg.exit_mode != "sma" {
            wilder_rsi(close, cfg.rsi_len)?
        } else {
            vec![f64::NAN; n_rows]
        };
        for day in 0..n_rows {
            if !close[day].is_finite() {
                continue;
            }
            let sma_hit = close[day] < sma[day];
            let rsi_hit = rsi[day].is_finite() && rsi[day] < cfg.exit_rsi;
            exit_now[symbol][day] = match cfg.exit_mode.as_str() {
                "sma" => sma_hit,
                "rsi" => rsi_hit,
                _ => sma_hit || rsi_hit,
            };
        }
    }
    let mut signal = vec![vec![false; n_rows]; n_symbols];
    for symbol in 0..n_symbols {
        for day in 0..n_rows {
            signal[symbol][day] = tradable[symbol][day]
                && streak[symbol][day] >= cfg.streak_days as f64
                && gate_ok[symbol][day];
        }
    }
    if cfg.skip_days > 0 {
        let original = signal.clone();
        signal.fill(vec![false; n_rows]);
        for symbol in 0..n_symbols {
            for day in cfg.skip_days..n_rows {
                signal[symbol][day] = original[symbol][day - cfg.skip_days]
                    && tradable[symbol][day]
                    && gate_ok[symbol][day];
            }
        }
    }
    let mut b_day = vec![0.0; n_rows];
    for day in 1..n_rows {
        let (mut sum, mut count) = (0.0, 0usize);
        for symbol in 0..n_symbols {
            let (current, previous) = (panel.close[symbol][day], panel.close[symbol][day - 1]);
            if tradable[symbol][day] && current.is_finite() && previous.is_finite() && previous > 0.0
            {
                sum += current / previous - 1.0;
                count += 1;
            }
        }
        if count > 0 {
            b_day[day] = sum / count as f64;
        }
    }
    Ok(Signals {
        rsi: streak,
        indicator_name: "up_streak".into(),
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
        benchmark_returns: Some(b_day),
    })
}

pub fn s031_trades(
    panel: &Panel,
    sig: &Signals,
    cfg: &Config,
    fill: &str,
) -> Result<Vec<Trade>, String> {
    if !matches!(fill, "close" | "next_open") {
        return Err(format!("S031 does not support fill {fill:?}"));
    }
    let hold_cap = if cfg.exit_mode == "fixed" {
        cfg.fixed_hold
    } else {
        cfg.max_hold
    };
    if hold_cap == 0 {
        return Err("S031 hold cap must be greater than zero".into());
    }
    let direction = if cfg.direction == "long" { 1.0 } else { -1.0 };
    let borrow_bar = if direction < 0.0 {
        cfg.borrow_bps_annual / 10_000.0 / cfg.periods_per_year as f64
    } else {
        0.0
    };
    let n_rows = panel.dates.len();
    let is_open = fill == "next_open";
    let mut trades = Vec::new();
    for symbol in 0..panel.symbols.len() {
        let close = &panel.close[symbol];
        let open = &panel.open[symbol];
        let Some(last) = close.iter().rposition(|value| value.is_finite()) else {
            continue;
        };
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
                let adverse = close[holding_day] >= entry_px * (1.0 + cfg.stop_loss);
                let favorable = close[holding_day] <= entry_px * (1.0 - cfg.take_profit);
                let (stopped, targeted) = if direction < 0.0 {
                    (adverse, favorable)
                } else {
                    (
                        close[holding_day] <= entry_px * (1.0 - cfg.stop_loss),
                        close[holding_day] >= entry_px * (1.0 + cfg.take_profit),
                    )
                };
                if cfg.stop_loss > 0.0 && stopped {
                    decision = holding_day;
                    reason = "stop";
                    break;
                }
                if cfg.take_profit > 0.0 && targeted {
                    decision = holding_day;
                    reason = "target";
                    break;
                }
                if sig.exit_now[symbol][holding_day] {
                    decision = holding_day;
                    reason = "signal";
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
            trades.push(Trade {
                sym_idx: symbol,
                signal_idx: day,
                entry_idx: entry,
                exit_idx: exit,
                entry_px,
                exit_px,
                ret: 0.0,
                ret_gross: 0.0,
                exit_reason: reason,
                hold: exit - entry,
                signal_value: sig.rsi[symbol][day],
                weight: direction,
                entry_turnover: 0.0,
                exit_turnover: 0.0,
                weighted: true,
                borrow_cost_per_bar: borrow_bar,
                dir_sign: 1.0,
            });
        }
    }
    // Normalize each trade to 1/max(open positions, slots) of book notional so the
    // short book never exceeds 100% gross exposure on crowded signal days.
    let mut open_count = vec![0usize; n_rows];
    for trade in &trades {
        for day in trade.entry_idx..=trade.exit_idx {
            open_count[day] += 1;
        }
    }
    for trade in &mut trades {
        let concurrent = (trade.entry_idx..=trade.exit_idx)
            .map(|day| open_count[day])
            .max()
            .unwrap_or(0);
        let weight = direction / concurrent.max(cfg.book_slots) as f64;
        let cost = cfg.cost_for_symbol(&panel.symbols[trade.sym_idx]);
        let gross = trade.exit_px / trade.entry_px - 1.0;
        let borrow = borrow_bar * (trade.exit_idx - trade.entry_idx + 1) as f64;
        trade.weight = weight;
        trade.entry_turnover = weight.abs();
        trade.exit_turnover = weight.abs();
        trade.ret_gross = direction * gross;
        trade.ret = direction * gross - 2.0 * cost - borrow;
    }
    trades.sort_by(|a, b| {
        (a.entry_idx, &panel.symbols[a.sym_idx]).cmp(&(b.entry_idx, &panel.symbols[b.sym_idx]))
    });
    Ok(trades)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    fn cfg(overrides: &str) -> Config {
        let yaml = concat!(
            "partition_split_date: '2021-01-01'\n",
            "direction: short\n",
            "streak_days: 3\n",
            "gate: none\n",
            "exit_mode: fixed\n",
            "fixed_hold: 2\n",
            "max_hold: 2\n",
            "universe_top_n: 10\n",
            "rank_window: 2\n",
            "rank_min_observations: 1\n",
            "min_price: 1.0\n",
            "min_history: 1\n",
            "cost_bps_per_side: 0.0\n",
            "borrow_bps_annual: 3650.0\n",
        )
        .to_string()
            + overrides;
        serde_yaml::from_str(&yaml).expect("s031 test config")
    }

    fn panel(closes: &[Vec<f64>]) -> Panel {
        let n = closes.first().map_or(0, Vec::len);
        Panel {
            dates: (0..n)
                .map(|i| NaiveDate::from_ymd_opt(2020, 1, 1).unwrap() + chrono::Days::new(i as u64))
                .collect(),
            symbols: (0..closes.len()).map(|i| format!("S{i}")).collect(),
            open: closes
                .iter()
                .map(|c| c.iter().map(|v| v * 1.001).collect())
                .collect(),
            high: closes.to_vec(),
            low: closes.to_vec(),
            close: closes.to_vec(),
            volume: vec![vec![1e6; n]; closes.len()],
        }
    }

    #[test]
    fn streak_counts_consecutive_up_closes() {
        let panel = panel(&[vec![10.0, 11.0, 12.0, 11.0, 12.0, 13.0, 14.0]]);
        let sig = compute_s031_signals(&panel, &cfg("")).unwrap();
        assert_eq!(sig.rsi[0][2], 2.0);
        assert_eq!(sig.rsi[0][3], 0.0);
        assert_eq!(sig.rsi[0][6], 3.0);
        assert!(sig.signal[0][6]);
        assert!(!sig.signal[0][5]);
    }

    #[test]
    fn short_trade_gains_on_decline_and_charges_borrow() {
        let panel = panel(&[vec![
            10.0, 11.0, 12.0, 13.0, 12.0, 11.0, 10.0, 9.0,
        ]]);
        let sig = compute_s031_signals(&panel, &cfg("")).unwrap();
        let trades = s031_trades(&panel, &sig, &cfg(""), "close").unwrap();
        assert_eq!(trades.len(), 1);
        let t = &trades[0];
        // signal day 3 (streak 3), close entry, fixed hold 2 -> exit day 5
        assert_eq!((t.signal_idx, t.entry_idx, t.exit_idx), (3, 3, 5));
        let gross = 11.0 / 13.0 - 1.0;
        assert!((t.ret_gross + gross).abs() < 1e-12);
        let borrow = 3650.0 / 10_000.0 / 252.0 * (5 - 3 + 1) as f64;
        assert!((t.ret + gross + borrow).abs() < 1e-12);
        assert!(t.weight < 0.0 && t.weighted);
        assert!((t.borrow_cost_per_bar - 3650.0 / 10_000.0 / 252.0).abs() < 1e-15);
    }

    #[test]
    fn crowded_days_cap_gross_exposure() {
        let up = vec![10.0, 11.0, 12.0, 13.0, 12.0, 11.0, 10.0, 9.0];
        let panel = panel(&[up.clone(), up.clone(), up]);
        let sig = compute_s031_signals(&panel, &cfg("")).unwrap();
        let trades = s031_trades(&panel, &sig, &cfg("book_slots: 1\n"), "close").unwrap();
        assert_eq!(trades.len(), 3);
        let gross: f64 = trades.iter().map(|t| t.weight.abs()).sum();
        assert!((gross - 1.0).abs() < 1e-12);
    }
}
