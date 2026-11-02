use std::collections::{BTreeMap, BTreeSet, VecDeque};

use chrono::{DateTime, Datelike, NaiveDate, TimeZone, Utc};

use crate::data::{CorporateAction, Panel};
use crate::s041_replay::{
    installments, position_action, rebalance_orders, Allocation, CompletedBar, Execution,
    OpenQuote, Order, PortfolioReplay, RankedAsset, SignalAction,
};
use crate::signals::Signals;

#[derive(Clone, Copy, Debug)]
pub struct CashExecution {
    pub initial_cash: f64,
    pub long_fraction: f64,
    pub short_fraction: f64,
    pub short_count: usize,
    pub max_participation: f64,
    pub max_liquidity_age_seconds: i64,
    pub max_pending_sessions: usize,
    pub max_stale_mark_sessions: usize,
    pub dca_installments: usize,
    pub stop_loss_fraction: f64,
    pub pyramid_profit_fraction: f64,
    pub pyramid_add_fraction: f64,
    pub slippage_bps: f64,
    pub commission_per_share: f64,
    pub annual_borrow_rate: f64,
    pub delisting_return: f64,
}

#[derive(Clone, Debug)]
pub struct SpinOffDistribution {
    pub symbol: String,
    pub shares_per_source_share: f64,
}

#[derive(Clone, Debug)]
pub struct SpinOffAction {
    pub date: NaiveDate,
    pub source_symbol: String,
    pub distributions: Vec<SpinOffDistribution>,
    pub source_url: String,
}

#[derive(Clone, Debug)]
pub struct CashDay {
    pub date: NaiveDate,
    pub cash: f64,
    pub equity: f64,
    pub gross_exposure: f64,
    pub net_exposure: f64,
    pub positions: usize,
    pub log_return: f64,
    pub stock_entries: usize,
    pub completed_round_trips: usize,
    pub winning_round_trips: usize,
}

#[derive(Clone, Debug)]
pub struct CashActionEvent {
    pub date: NaiveDate,
    pub action_id: String,
    pub symbol: String,
    pub action: String,
    pub amount: f64,
}

#[derive(Clone, Debug)]
pub struct CashTrade {
    pub date: NaiveDate,
    pub symbol: String,
    pub realized_pnl: f64,
}

#[derive(Clone, Debug)]
pub struct CashLifecycleEvent {
    pub date: NaiveDate,
    pub symbol: String,
    pub action: String,
    pub shares: i64,
}

#[derive(Clone, Debug)]
pub struct CashReplay {
    pub account: PortfolioReplay,
    pub days: Vec<CashDay>,
    pub actions: Vec<CashActionEvent>,
    pub lifecycle_events: Vec<CashLifecycleEvent>,
    pub unpaid_dividends: Vec<String>,
    pub trades: Vec<CashTrade>,
    pub rejections: Vec<(NaiveDate, String, String)>,
    pub missing_open_quotes: BTreeSet<(NaiveDate, String)>,
    pub stale_marks: BTreeSet<(NaiveDate, String)>,
    pub rebalance_dates: Vec<NaiveDate>,
    pub stock_entries: usize,
    pub completed_round_trips: usize,
    pub winning_round_trips: usize,
}

pub fn run_cash_replay(
    signals_panel: &Panel,
    exec_panel: &Panel,
    signals: &Signals,
    corporate_actions: &[CorporateAction],
    execution: CashExecution,
    breadth_mode: &str,
    breadth: usize,
    min_breadth: usize,
    decile: f64,
) -> Result<CashReplay, String> {
    run_cash_replay_with_spinoffs(
        signals_panel,
        exec_panel,
        signals,
        corporate_actions,
        &[],
        execution,
        breadth_mode,
        breadth,
        min_breadth,
        decile,
    )
}

pub fn run_cash_replay_with_spinoffs(
    signals_panel: &Panel,
    exec_panel: &Panel,
    signals: &Signals,
    corporate_actions: &[CorporateAction],
    spinoffs: &[SpinOffAction],
    execution: CashExecution,
    breadth_mode: &str,
    breadth: usize,
    min_breadth: usize,
    decile: f64,
) -> Result<CashReplay, String> {
    validate_inputs(
        signals_panel,
        exec_panel,
        signals,
        execution,
        breadth_mode,
        breadth,
        min_breadth,
        decile,
    )?;
    let mut account = PortfolioReplay::new(execution.initial_cash)?;
    let mut days = Vec::with_capacity(exec_panel.dates.len());
    let mut actions_log = Vec::new();
    let mut lifecycle_events = Vec::new();
    let mut unpaid_dividends = Vec::new();
    let mut completed_trades = Vec::new();
    let mut rejections = Vec::new();
    let mut missing_open_quotes = BTreeSet::new();
    let mut stale_marks = BTreeSet::new();
    let mut rebalance_dates = Vec::new();
    let mut last_marks = BTreeMap::<String, f64>::new();
    let mut last_bar = BTreeMap::<String, (NaiveDate, u64, f64)>::new();
    let mut stale_sessions = BTreeMap::<String, usize>::new();
    let mut dca_schedules = BTreeMap::<String, VecDeque<i64>>::new();
    let mut pyramided = BTreeSet::<String>::new();
    let mut stop_blocked = BTreeSet::<String>::new();
    let mut signal_row = BTreeMap::new();
    for (row, date) in signals_panel.dates.iter().enumerate() {
        signal_row.insert(*date, row);
    }
    let mut actions_by_date = BTreeMap::<NaiveDate, Vec<&CorporateAction>>::new();
    for action in corporate_actions {
        actions_by_date.entry(action.date).or_default().push(action);
    }
    let mut spinoffs_by_date = BTreeMap::<NaiveDate, Vec<&SpinOffAction>>::new();
    for action in spinoffs {
        spinoffs_by_date
            .entry(action.date)
            .or_default()
            .push(action);
    }
    let mut payments = BTreeMap::<NaiveDate, Vec<String>>::new();
    let mut previous_date: Option<NaiveDate> = None;
    let mut previous_equity = execution.initial_cash;
    let mut stock_entries = 0;
    let mut completed_round_trips = 0;
    let mut winning_round_trips = 0;
    let mut cycle_pnl = BTreeMap::<String, f64>::new();

    for (day, &date) in exec_panel.dates.iter().enumerate() {
        let mut day_entries = 0;
        let mut day_round_trips = 0;
        let mut day_wins = 0;
        let open_time = session_time(date, 9)?;
        let close_time = session_time(date, 16)?;
        for action_id in payments.remove(&date).unwrap_or_default() {
            let amount = account.ledger.settle_dividend(&action_id)?;
            actions_log.push(CashActionEvent {
                date,
                action_id,
                symbol: String::new(),
                action: "dividend_payment".into(),
                amount,
            });
        }

        if let Some(day_spinoffs) = spinoffs_by_date.get(&date) {
            for spinoff in day_spinoffs {
                if !account
                    .ledger
                    .positions()
                    .contains_key(&spinoff.source_symbol)
                {
                    continue;
                }
                let mut components = vec![(
                    spinoff.source_symbol.clone(),
                    1.0,
                    panel_value(exec_panel, &spinoff.source_symbol, day, "open")?,
                )];
                for distribution in &spinoff.distributions {
                    components.push((
                        distribution.symbol.clone(),
                        distribution.shares_per_source_share,
                        panel_value(exec_panel, &distribution.symbol, day, "open")?,
                    ));
                }
                let previous_cycle_pnl =
                    cycle_pnl.remove(&spinoff.source_symbol).unwrap_or_default();
                for receipt in account
                    .ledger
                    .apply_spinoff(&spinoff.source_symbol, &components)?
                {
                    let pnl_symbol = if receipt.shares == 0 {
                        spinoff.source_symbol.clone()
                    } else {
                        receipt.symbol.clone()
                    };
                    *cycle_pnl.entry(pnl_symbol).or_default() +=
                        previous_cycle_pnl * receipt.value_weight + receipt.realized_pnl;
                    lifecycle_events.push(CashLifecycleEvent {
                        date,
                        symbol: receipt.symbol.clone(),
                        action: format!("spinoff_distribution_from_{}", spinoff.source_symbol),
                        shares: receipt.shares,
                    });
                    actions_log.push(CashActionEvent {
                        date,
                        action_id: format!(
                            "spinoff-{}-{}-{date}",
                            spinoff.source_symbol, receipt.symbol
                        ),
                        symbol: receipt.symbol,
                        action: "spin_off_cash_in_lieu_at_open".into(),
                        amount: receipt.cash_in_lieu,
                    });
                }
            }
        }

        if let Some(day_actions) = actions_by_date.get(&date) {
            for action in day_actions
                .iter()
                .copied()
                .filter(|action| action.action_type.to_ascii_lowercase().contains("split"))
            {
                let held = account.ledger.positions().contains_key(&action.symbol);
                let has_pending = account
                    .pending()
                    .iter()
                    .any(|order| order.symbol == action.symbol);
                if held || has_pending {
                    let (old, new) =
                        split_ratio_terms(action.old_rate, action.new_rate, &action.symbol)?;
                    let amount = if held {
                        let price = panel_value(exec_panel, &action.symbol, day, "open")?;
                        account
                            .ledger
                            .apply_split(&action.symbol, old, new, price)?
                    } else {
                        0.0
                    };
                    account.adjust_pending_split(&action.symbol, old, new, open_time)?;
                    actions_log.push(CashActionEvent {
                        date,
                        action_id: format!("split-{}-{date}", action.symbol),
                        symbol: action.symbol.clone(),
                        action: "split".into(),
                        amount,
                    });
                }
            }
            for (action_index, action) in day_actions
                .iter()
                .copied()
                .enumerate()
                .filter(|(_, action)| action.action_type.eq_ignore_ascii_case("cash_dividend"))
            {
                if account.ledger.positions().contains_key(&action.symbol) {
                    let action_id =
                        format!("dividend-{}-{}-{action_index}", action.symbol, action.date);
                    let amount = account.ledger.accrue_dividend(
                        &action_id,
                        &action.symbol,
                        action.rate.ok_or_else(|| {
                            format!(
                                "{} dividend on {} has no per-share rate",
                                action.symbol, action.date
                            )
                        })?,
                    )?;
                    let action_name = if let Some(payment_date) = action.payment_date {
                        payments
                            .entry(payment_date)
                            .or_default()
                            .push(action_id.clone());
                        "dividend_entitlement"
                    } else {
                        unpaid_dividends.push(action_id.clone());
                        "dividend_entitlement_missing_pay_date"
                    };
                    actions_log.push(CashActionEvent {
                        date,
                        action_id,
                        symbol: action.symbol.clone(),
                        action: action_name.into(),
                        amount,
                    });
                }
            }
            for action in day_actions.iter().copied() {
                if !action.action_type.eq_ignore_ascii_case("cash_dividend")
                    && !action.action_type.to_ascii_lowercase().contains("split")
                    && account.ledger.positions().contains_key(&action.symbol)
                {
                    return Err(format!(
                        "unsupported corporate action {:?} for held symbol {} on {}",
                        action.action_type, action.symbol, date
                    ));
                }
            }
        }

        let mut open_marks = last_marks.clone();
        for (symbol_index, symbol) in exec_panel.symbols.iter().enumerate() {
            let open = exec_panel.open[symbol_index][day];
            if open.is_finite() && open > 0.0 {
                open_marks.insert(symbol.clone(), open);
            } else if account.ledger.positions().contains_key(symbol) {
                missing_open_quotes.insert((date, symbol.clone()));
            }
        }

        let pending_symbols: Vec<String> = account
            .pending()
            .iter()
            .map(|order| order.symbol.clone())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect();
        for symbol in &pending_symbols {
            let expired: Vec<Order> = account
                .pending()
                .iter()
                .filter(|order| order.symbol == *symbol)
                .filter(|order| {
                    let order_day = exec_panel
                        .dates
                        .partition_point(|order_date| *order_date < order.decided_at.date_naive());
                    day.saturating_sub(order_day) > execution.max_pending_sessions
                })
                .cloned()
                .collect();
            if !expired.is_empty() {
                account.cancel_pending(symbol, open_time)?;
            }
        }

        let pending_order_sequence: Vec<String> = account
            .pending()
            .iter()
            .map(|order| order.symbol.clone())
            .fold(Vec::new(), |mut symbols, symbol| {
                if !symbols.contains(&symbol) {
                    symbols.push(symbol);
                }
                symbols
            });
        for symbol in pending_order_sequence {
            let Some(symbol_index) = exec_panel
                .symbols
                .iter()
                .position(|candidate| candidate == &symbol)
            else {
                return Err(format!("pending order references unknown symbol {symbol}"));
            };
            let Some(open) = finite_price(exec_panel.open[symbol_index][day]) else {
                continue;
            };
            let Some((bar_date, volume, bar_close)) = last_bar.get(&symbol).copied() else {
                continue;
            };
            let liquidity_bar = CompletedBar {
                symbol: symbol.clone(),
                start: session_time(bar_date, 0)?,
                end: session_time(bar_date, 16)?,
                close: bar_close,
                volume,
            };
            let quote = OpenQuote {
                symbol: symbol.clone(),
                timestamp: open_time,
                price: open,
            };
            let execution = Execution {
                max_participation: execution.max_participation,
                max_liquidity_age_seconds: execution.max_liquidity_age_seconds,
                slippage_bps: execution.slippage_bps,
                commission_per_share: execution.commission_per_share,
                short_located: execution.short_fraction > 0.0,
            };
            let mut before = account
                .ledger
                .positions()
                .get(&symbol)
                .map_or(0, |p| p.shares);
            match account.process_quote(&liquidity_bar, &quote, execution, &open_marks) {
                Ok(fills) => {
                    for fill in fills {
                        let after = before
                            .checked_add(fill.shares)
                            .ok_or("fill position overflow")?;
                        if before == 0 || before.signum() != after.signum() {
                            let entered = usize::from(after != 0);
                            stock_entries += entered;
                            day_entries += entered;
                        }
                        if before != 0 && (after == 0 || before.signum() != after.signum()) {
                            if let Some(chunks) = dca_schedules.remove(&symbol) {
                                lifecycle_events.push(CashLifecycleEvent {
                                    date,
                                    symbol: symbol.clone(),
                                    action: "dca_schedule_cancelled_after_position_close".into(),
                                    shares: chunks.iter().sum(),
                                });
                            }
                            pyramided.remove(&symbol);
                            completed_round_trips += 1;
                            day_round_trips += 1;
                            let trade_pnl =
                                cycle_pnl.remove(&symbol).unwrap_or(0.0) + fill.realized_pnl;
                            completed_trades.push(CashTrade {
                                date,
                                symbol: symbol.clone(),
                                realized_pnl: trade_pnl,
                            });
                            if trade_pnl > 0.0 {
                                winning_round_trips += 1;
                                day_wins += 1;
                            }
                            if after != 0 {
                                let closed = before.unsigned_abs().min(fill.shares.unsigned_abs());
                                let opened = fill.shares.unsigned_abs() - closed;
                                let opening_fee = if fill.shares == 0 {
                                    0.0
                                } else {
                                    fill.commission * opened as f64
                                        / fill.shares.unsigned_abs() as f64
                                };
                                cycle_pnl.insert(symbol.clone(), -opening_fee);
                            }
                        } else {
                            *cycle_pnl.entry(symbol.clone()).or_default() += fill.realized_pnl;
                        }
                        before = after;
                    }
                }
                Err(error) => {
                    rejections.push((date, symbol.clone(), error));
                    account.cancel_pending(&symbol, open_time)?;
                    if let Some(chunks) = dca_schedules.remove(&symbol) {
                        lifecycle_events.push(CashLifecycleEvent {
                            date,
                            symbol: symbol.clone(),
                            action: "dca_schedule_cancelled_after_rejection".into(),
                            shares: chunks.iter().sum(),
                        });
                    }
                }
            }
        }

        let mut close_marks = last_marks.clone();
        let mut delisted = Vec::new();
        for (symbol_index, symbol) in exec_panel.symbols.iter().enumerate() {
            let close = exec_panel.close[symbol_index][day];
            if close.is_finite() && close > 0.0 {
                close_marks.insert(symbol.clone(), close);
                stale_sessions.remove(symbol);
            } else if account.ledger.positions().contains_key(symbol) {
                stale_marks.insert((date, symbol.clone()));
                let stale_count = stale_sessions.entry(symbol.clone()).or_default();
                *stale_count += 1;
                if *stale_count >= execution.max_stale_mark_sessions {
                    delisted.push(symbol.clone());
                }
            }
        }
        for symbol in delisted {
            let cancelled = account.cancel_pending(&symbol, close_time)?;
            if let Some(chunks) = dca_schedules.remove(&symbol) {
                lifecycle_events.push(CashLifecycleEvent {
                    date,
                    symbol: symbol.clone(),
                    action: "dca_schedule_cancelled_by_delisting".into(),
                    shares: chunks.iter().sum(),
                });
            }
            pyramided.remove(&symbol);
            let last_mark = *close_marks.get(&symbol).ok_or_else(|| {
                format!("held symbol {symbol} has no prior raw close for settlement")
            })?;
            let position = account.ledger.positions()[&symbol].clone();
            let settlement_price = last_mark * (1.0 + execution.delisting_return);
            let cash_delta =
                account
                    .ledger
                    .settle_delisting(&symbol, last_mark, execution.delisting_return)?;
            let trade_pnl = cycle_pnl.remove(&symbol).unwrap_or(0.0)
                + position.shares as f64 * (settlement_price - position.average_price);
            completed_trades.push(CashTrade {
                date,
                symbol: symbol.clone(),
                realized_pnl: trade_pnl,
            });
            completed_round_trips += 1;
            day_round_trips += 1;
            if trade_pnl > 0.0 {
                winning_round_trips += 1;
                day_wins += 1;
            }
            close_marks.remove(&symbol);
            stale_sessions.remove(&symbol);
            actions_log.push(CashActionEvent {
                date,
                action_id: format!("delisting-{symbol}-{date}"),
                symbol: symbol.clone(),
                action: "assumed_delisting_settlement".into(),
                amount: cash_delta,
            });
            if cancelled > 0 {
                actions_log.push(CashActionEvent {
                    date,
                    action_id: format!("delisting-cancelled-{symbol}-{date}"),
                    symbol,
                    action: "delisting_cancelled_pending_orders".into(),
                    amount: cancelled as f64,
                });
            }
        }
        if let Some(last_date) = previous_date {
            let elapsed_days = (date - last_date).num_days().max(1) as u32;
            account.ledger.charge_borrow(
                &close_marks,
                execution.annual_borrow_rate,
                elapsed_days,
            )?;
        }
        let valuation = account.ledger.value(&close_marks)?;
        let log_return = (valuation.equity / previous_equity).ln();
        if !log_return.is_finite() {
            return Err(format!("nonfinite portfolio return on {date}"));
        }
        days.push(CashDay {
            date,
            cash: account.ledger.cash(),
            equity: valuation.equity,
            gross_exposure: valuation.gross_exposure,
            net_exposure: valuation.net_exposure,
            positions: account.ledger.positions().len(),
            log_return,
            stock_entries: day_entries,
            completed_round_trips: day_round_trips,
            winning_round_trips: day_wins,
        });
        previous_equity = valuation.equity;
        last_marks = close_marks;

        for symbol in exec_panel.symbols.iter() {
            let (Some(position), Some(&close)) = (
                account.ledger.positions().get(symbol),
                exec_panel
                    .symbols
                    .iter()
                    .position(|candidate| candidate == symbol)
                    .map(|index| &exec_panel.close[index][day]),
            ) else {
                continue;
            };
            if !close.is_finite() || close <= 0.0 {
                continue;
            }
            let add_shares = if execution.pyramid_profit_fraction > 0.0
                && execution.pyramid_add_fraction > 0.0
            {
                (position.shares.unsigned_abs() as f64 * execution.pyramid_add_fraction)
                    .floor()
                    .max(1.0)
                    .min(i64::MAX as f64) as i64
            } else {
                0
            };
            let action = position_action(
                position.shares,
                position.average_price,
                close,
                execution.stop_loss_fraction,
                execution.pyramid_profit_fraction,
                add_shares,
                pyramided.contains(symbol),
            )?;
            match action {
                SignalAction::Exit(shares) => {
                    account.cancel_pending(symbol, close_time)?;
                    if let Some(chunks) = dca_schedules.remove(symbol) {
                        lifecycle_events.push(CashLifecycleEvent {
                            date,
                            symbol: symbol.clone(),
                            action: "dca_schedule_cancelled_by_stop".into(),
                            shares: chunks.iter().sum(),
                        });
                    }
                    pyramided.remove(symbol);
                    stop_blocked.insert(symbol.clone());
                    lifecycle_events.push(CashLifecycleEvent {
                        date,
                        symbol: symbol.clone(),
                        action: "stop_exit_submitted".into(),
                        shares,
                    });
                    account.submit(Order {
                        symbol: symbol.clone(),
                        shares,
                        decided_at: close_time,
                    })?;
                }
                SignalAction::Add(shares)
                    if !account
                        .pending()
                        .iter()
                        .any(|order| order.symbol == *symbol)
                        && !dca_schedules.contains_key(symbol) =>
                {
                    lifecycle_events.push(CashLifecycleEvent {
                        date,
                        symbol: symbol.clone(),
                        action: "pyramid_add_submitted".into(),
                        shares,
                    });
                    account.submit(Order {
                        symbol: symbol.clone(),
                        shares,
                        decided_at: close_time,
                    })?;
                    pyramided.insert(symbol.clone());
                }
                SignalAction::Hold | SignalAction::Add(_) => {}
            }
        }

        let Some(&signal_day) = signal_row.get(&date) else {
            schedule_next_installments(
                &mut account,
                &mut dca_schedules,
                &mut lifecycle_events,
                close_time,
            )?;
            for (symbol_index, symbol) in exec_panel.symbols.iter().enumerate() {
                let volume = exec_panel.volume[symbol_index][day];
                if volume.is_finite() && volume >= 0.0 {
                    if let Some(close) = finite_price(exec_panel.close[symbol_index][day]) {
                        last_bar.insert(
                            symbol.clone(),
                            (date, volume.floor().min(u64::MAX as f64) as u64, close),
                        );
                    }
                }
            }
            previous_date = Some(date);
            continue;
        };
        let is_month_end = signal_day + 1 == signals_panel.dates.len()
            || signals_panel.dates[signal_day].month()
                != signals_panel.dates[signal_day + 1].month()
            || signals_panel.dates[signal_day].year() != signals_panel.dates[signal_day + 1].year();
        let mut rebalanced = false;
        if is_month_end && day + 1 < exec_panel.dates.len() {
            rebalanced = true;
            let mut candidates: Vec<RankedAsset> = (0..signals_panel.symbols.len())
                .filter(|&symbol| {
                    signals.signal[symbol][signal_day]
                        && signals.rsi[symbol][signal_day].is_finite()
                        && !stop_blocked.contains(&signals_panel.symbols[symbol])
                })
                .map(|symbol| RankedAsset {
                    symbol: signals_panel.symbols[symbol].clone(),
                    score: signals.rsi[symbol][signal_day],
                    close: 1.0,
                })
                .collect();
            candidates.sort_by(|a, b| {
                b.score
                    .total_cmp(&a.score)
                    .then_with(|| a.symbol.cmp(&b.symbol))
            });
            let count = if breadth_mode == "top" {
                breadth
            } else {
                min_breadth.max((candidates.len() as f64 * decile).floor() as usize)
            }
            .min(candidates.len());
            let long_count = if execution.long_fraction > 0.0 {
                count
            } else {
                0
            };
            let short_count = if execution.short_fraction > 0.0 {
                execution.short_count.min(candidates.len() - long_count)
            } else {
                0
            };
            let candidate_count = candidates.len();
            candidates = candidates
                .into_iter()
                .enumerate()
                .filter(|(index, _)| *index < long_count || *index >= candidate_count - short_count)
                .map(|(_, candidate)| candidate)
                .collect();
            for candidate in &mut candidates {
                let exec_symbol = exec_panel
                    .symbols
                    .iter()
                    .position(|name| name == &candidate.symbol)
                    .ok_or_else(|| {
                        format!(
                            "selected symbol {} has no raw execution series",
                            candidate.symbol
                        )
                    })?;
                let raw_close = exec_panel.close[exec_symbol][day];
                if !raw_close.is_finite() || raw_close <= 0.0 {
                    return Err(format!(
                        "selected symbol {} has no raw close on rebalance date {date}",
                        candidate.symbol
                    ));
                }
                candidate.close = raw_close;
            }
            for order in account.pending().to_vec() {
                account.cancel_pending(&order.symbol, close_time)?;
            }
            for (symbol, chunks) in std::mem::take(&mut dca_schedules) {
                lifecycle_events.push(CashLifecycleEvent {
                    date,
                    symbol,
                    action: "dca_schedule_cancelled_at_rebalance".into(),
                    shares: chunks.iter().sum(),
                });
            }
            let marks = last_marks.clone();
            let orders = rebalance_orders(
                &account.ledger,
                &candidates,
                Allocation {
                    long_count,
                    short_count,
                    long_fraction: if long_count == 0 {
                        0.0
                    } else {
                        execution.long_fraction
                    },
                    short_fraction: if short_count == 0 {
                        0.0
                    } else {
                        execution.short_fraction
                    },
                },
                &marks,
                close_time,
            )?;
            let mut planned_positions: BTreeMap<String, i64> = account
                .ledger
                .positions()
                .iter()
                .map(|(symbol, position)| (symbol.clone(), position.shares))
                .collect();
            for order in orders {
                let current = planned_positions.get(&order.symbol).copied().unwrap_or(0);
                if order.shares > 0 && current >= 0 && execution.dca_installments > 1 {
                    let mut chunks: VecDeque<i64> =
                        installments(order.shares, execution.dca_installments)?
                            .into_iter()
                            .filter(|shares| *shares > 0)
                            .collect();
                    let first = chunks
                        .pop_front()
                        .ok_or("DCA schedule produced no positive installment")?;
                    account.submit(Order {
                        shares: first,
                        ..order.clone()
                    })?;
                    lifecycle_events.push(CashLifecycleEvent {
                        date,
                        symbol: order.symbol.clone(),
                        action: "dca_installment_submitted".into(),
                        shares: first,
                    });
                    if !chunks.is_empty() {
                        lifecycle_events.push(CashLifecycleEvent {
                            date,
                            symbol: order.symbol.clone(),
                            action: "dca_installments_scheduled".into(),
                            shares: chunks.iter().sum(),
                        });
                        dca_schedules.insert(order.symbol.clone(), chunks);
                    }
                } else {
                    account.submit(order.clone())?;
                }
                let position = planned_positions.entry(order.symbol.clone()).or_default();
                *position = position
                    .checked_add(order.shares)
                    .ok_or("planned position overflow")?;
            }
            rebalance_dates.push(date);
            stop_blocked.clear();
        }
        if !rebalanced {
            schedule_next_installments(
                &mut account,
                &mut dca_schedules,
                &mut lifecycle_events,
                close_time,
            )?;
        }
        for (symbol_index, symbol) in exec_panel.symbols.iter().enumerate() {
            let volume = exec_panel.volume[symbol_index][day];
            if volume.is_finite() && volume >= 0.0 {
                if let Some(close) = finite_price(exec_panel.close[symbol_index][day]) {
                    last_bar.insert(
                        symbol.clone(),
                        (date, volume.floor().min(u64::MAX as f64) as u64, close),
                    );
                }
            }
        }
        previous_date = Some(date);
    }

    account.ledger.value(&last_marks)?;
    Ok(CashReplay {
        account,
        days,
        actions: actions_log,
        lifecycle_events,
        unpaid_dividends,
        trades: completed_trades,
        rejections,
        missing_open_quotes,
        stale_marks,
        rebalance_dates,
        stock_entries,
        completed_round_trips,
        winning_round_trips,
    })
}

fn validate_inputs(
    signals_panel: &Panel,
    exec_panel: &Panel,
    signals: &Signals,
    execution: CashExecution,
    breadth_mode: &str,
    breadth: usize,
    min_breadth: usize,
    decile: f64,
) -> Result<(), String> {
    if signals_panel.symbols != exec_panel.symbols {
        return Err("signal and raw execution panels must have identical symbol order".into());
    }
    if signals.rsi.len() != signals_panel.symbols.len()
        || signals.signal.len() != signals_panel.symbols.len()
        || signals
            .rsi
            .iter()
            .any(|series| series.len() != signals_panel.dates.len())
        || signals
            .signal
            .iter()
            .any(|series| series.len() != signals_panel.dates.len())
    {
        return Err("S041 signal dimensions do not match the signal panel".into());
    }
    if !execution.initial_cash.is_finite()
        || execution.initial_cash <= 0.0
        || !execution.long_fraction.is_finite()
        || !execution.short_fraction.is_finite()
        || execution.long_fraction < 0.0
        || execution.short_fraction < 0.0
        || execution.long_fraction + execution.short_fraction > 1.0
        || execution.max_pending_sessions == 0
        || execution.max_stale_mark_sessions == 0
        || execution.dca_installments == 0
        || !execution.stop_loss_fraction.is_finite()
        || !(0.0..1.0).contains(&execution.stop_loss_fraction)
        || !execution.pyramid_profit_fraction.is_finite()
        || execution.pyramid_profit_fraction < 0.0
        || !execution.pyramid_add_fraction.is_finite()
        || !(0.0..=1.0).contains(&execution.pyramid_add_fraction)
        || (execution.pyramid_profit_fraction > 0.0 && execution.pyramid_add_fraction == 0.0)
        || !execution.max_participation.is_finite()
        || execution.max_participation <= 0.0
        || execution.max_participation > 1.0
        || execution.max_liquidity_age_seconds <= 0
        || !execution.slippage_bps.is_finite()
        || !(0.0..10_000.0).contains(&execution.slippage_bps)
        || !execution.commission_per_share.is_finite()
        || execution.commission_per_share < 0.0
        || !execution.annual_borrow_rate.is_finite()
        || execution.annual_borrow_rate < 0.0
        || !execution.delisting_return.is_finite()
        || execution.delisting_return <= -1.0
        || (execution.short_fraction > 0.0) != (execution.short_count > 0)
    {
        return Err("invalid cash execution settings".into());
    }
    if !matches!(breadth_mode, "top" | "decile")
        || (breadth_mode == "top" && breadth == 0)
        || min_breadth == 0
        || !decile.is_finite()
        || !(0.0..=1.0).contains(&decile)
        || (breadth_mode == "decile" && decile == 0.0)
    {
        return Err("invalid S041 breadth configuration".into());
    }
    if exec_panel.dates.is_empty()
        || exec_panel
            .dates
            .windows(2)
            .any(|dates| dates[0] >= dates[1])
        || signals_panel
            .dates
            .windows(2)
            .any(|dates| dates[0] >= dates[1])
    {
        return Err("cash replay requires strictly increasing nonempty raw execution dates".into());
    }
    Ok(())
}

fn schedule_next_installments(
    account: &mut PortfolioReplay,
    schedules: &mut BTreeMap<String, VecDeque<i64>>,
    lifecycle_events: &mut Vec<CashLifecycleEvent>,
    decided_at: DateTime<Utc>,
) -> Result<(), String> {
    let symbols: Vec<String> = schedules.keys().cloned().collect();
    for symbol in symbols {
        if account.pending().iter().any(|order| order.symbol == symbol) {
            continue;
        }
        let shares = schedules
            .get_mut(&symbol)
            .and_then(VecDeque::pop_front)
            .ok_or_else(|| format!("empty DCA schedule for {symbol}"))?;
        account.submit(Order {
            symbol: symbol.clone(),
            shares,
            decided_at,
        })?;
        lifecycle_events.push(CashLifecycleEvent {
            date: decided_at.date_naive(),
            symbol: symbol.clone(),
            action: "dca_installment_submitted".into(),
            shares,
        });
        if schedules.get(&symbol).is_some_and(VecDeque::is_empty) {
            schedules.remove(&symbol);
        }
    }
    Ok(())
}

fn finite_price(price: f64) -> Option<f64> {
    (price.is_finite() && price > 0.0).then_some(price)
}

fn panel_value(panel: &Panel, symbol: &str, day: usize, field: &str) -> Result<f64, String> {
    let index = panel
        .symbols
        .iter()
        .position(|candidate| candidate == symbol)
        .ok_or_else(|| format!("corporate action references unknown symbol {symbol}"))?;
    let value = match field {
        "open" => panel.open[index][day],
        "close" => panel.close[index][day],
        _ => return Err(format!("unsupported price field {field}")),
    };
    finite_price(value)
        .ok_or_else(|| format!("missing raw {field} for {symbol} on {}", panel.dates[day]))
}

fn split_ratio_terms(
    old_rate: Option<f64>,
    new_rate: Option<f64>,
    symbol: &str,
) -> Result<(u32, u32), String> {
    let old_rate = old_rate.ok_or_else(|| format!("split action for {symbol} lacks old rate"))?;
    let new_rate = new_rate.ok_or_else(|| format!("split action for {symbol} lacks new rate"))?;
    if !old_rate.is_finite() || old_rate <= 0.0 || !new_rate.is_finite() || new_rate <= 0.0 {
        return Err(format!(
            "invalid split rates for {symbol}: {old_rate}, {new_rate}"
        ));
    }
    let ratio = new_rate / old_rate;
    for denominator in 1..=10_000u32 {
        let numerator = (ratio * denominator as f64).round();
        if numerator >= 1.0
            && numerator <= u32::MAX as f64
            && (ratio - numerator / denominator as f64).abs() <= ratio.abs() * 1e-5 + 1e-8
        {
            return Ok((denominator, numerator as u32));
        }
    }
    Err(format!(
        "split ratio for {symbol} cannot be represented within tolerance: {ratio}"
    ))
}

// Synthetic UTC values preserve event order; daily bars do not contain exchange timestamps.
fn session_time(date: NaiveDate, hour: u32) -> Result<DateTime<Utc>, String> {
    let naive = date
        .and_hms_opt(hour, 0, 0)
        .ok_or_else(|| format!("invalid session timestamp on {date}"))?;
    Ok(Utc.from_utc_datetime(&naive))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn date(day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(2025, 1, day).unwrap()
    }

    fn panel(dates: Vec<NaiveDate>, closes: Vec<Vec<f64>>) -> Panel {
        let rows = dates.len();
        let symbols = vec!["A".into(), "B".into()];
        Panel {
            dates,
            symbols,
            open: closes
                .iter()
                .map(|series| series.iter().map(|price| price - 1.0).collect())
                .collect(),
            high: closes
                .iter()
                .map(|series| series.iter().map(|price| price + 1.0).collect())
                .collect(),
            low: closes
                .iter()
                .map(|series| series.iter().map(|price| price - 2.0).collect())
                .collect(),
            close: closes,
            volume: vec![vec![100_000.0; rows]; 2],
        }
    }

    fn signals(rows: usize) -> Signals {
        Signals {
            rsi: vec![vec![2.0; rows], vec![1.0; rows]],
            indicator_name: "test".into(),
            signal: vec![vec![true; rows], vec![true; rows]],
            exit_now: vec![vec![false; rows]; 2],
            add_signal: None,
            atr: None,
            stop_pct: None,
            tradable: vec![vec![true; rows]; 2],
            pool: vec![vec![true; rows]; 2],
            trade_weights: None,
            entry_mat: None,
            exit_mat: None,
            benchmark_returns: None,
        }
    }

    fn settings() -> CashExecution {
        CashExecution {
            initial_cash: 10_000.0,
            long_fraction: 1.0,
            short_fraction: 0.0,
            short_count: 0,
            max_participation: 1.0,
            max_liquidity_age_seconds: 7 * 24 * 60 * 60,
            max_pending_sessions: 5,
            max_stale_mark_sessions: 3,
            dca_installments: 1,
            stop_loss_fraction: 0.0,
            pyramid_profit_fraction: 0.0,
            pyramid_add_fraction: 0.25,
            slippage_bps: 0.0,
            commission_per_share: 0.0,
            annual_borrow_rate: 0.0,
            delisting_return: -0.30,
        }
    }

    #[test]
    fn cash_replay_fills_only_after_the_close_decision() {
        let dates = vec![date(31), NaiveDate::from_ymd_opt(2025, 2, 3).unwrap()];
        let signal_panel = panel(dates.clone(), vec![vec![100.0, 100.0], vec![50.0, 50.0]]);
        let mut execution_panel = signal_panel.clone();
        execution_panel.open[0][1] = 100.0;
        execution_panel.close[0][1] = 110.0;
        let output = run_cash_replay(
            &signal_panel,
            &execution_panel,
            &signals(dates.len()),
            &[],
            settings(),
            "top",
            1,
            1,
            0.10,
        )
        .unwrap();
        assert_eq!(output.rebalance_dates, vec![dates[0]]);
        assert_eq!(output.stock_entries, 1);
        assert_eq!(output.account.ledger.positions()["A"].shares, 100);
        assert!((output.days[1].equity - 11_000.0).abs() < 1e-8);
        assert!(matches!(
            output.account.events(),
            [
                crate::s041_replay::ReplayEvent::OrderSubmitted { order },
                crate::s041_replay::ReplayEvent::Filled {
                    filled_at,
                    fill,
                    ..
                }
            ] if order.decided_at.date_naive() == dates[0]
                && filled_at.date_naive() == dates[1]
                && fill.shares == 100
        ));
    }

    #[test]
    fn cash_replay_applies_spin_off_before_open_marks_without_losing_equity() {
        let dates = vec![
            date(31),
            NaiveDate::from_ymd_opt(2025, 2, 3).unwrap(),
            NaiveDate::from_ymd_opt(2025, 2, 4).unwrap(),
        ];
        let signal_panel = panel(
            dates.clone(),
            vec![vec![100.0, 100.0, 60.0], vec![50.0, 50.0, 80.0]],
        );
        let mut execution_panel = signal_panel.clone();
        execution_panel.open[0][1] = 100.0;
        execution_panel.close[0][1] = 100.0;
        execution_panel.open[0][2] = 60.0;
        execution_panel.close[0][2] = 60.0;
        execution_panel.open[1][2] = 80.0;
        execution_panel.close[1][2] = 80.0;
        let mut execution = settings();
        execution.initial_cash = 300.0;
        let spinoffs = [SpinOffAction {
            date: dates[2],
            source_symbol: "A".into(),
            distributions: vec![SpinOffDistribution {
                symbol: "B".into(),
                shares_per_source_share: 0.5,
            }],
            source_url: "https://example.com/action".into(),
        }];
        let output = run_cash_replay_with_spinoffs(
            &signal_panel,
            &execution_panel,
            &signals(dates.len()),
            &[],
            &spinoffs,
            execution,
            "top",
            1,
            1,
            0.10,
        )
        .unwrap();
        assert_eq!(output.account.ledger.positions()["A"].shares, 3);
        assert_eq!(output.account.ledger.positions()["B"].shares, 1);
        assert!((output.days[2].equity - 300.0).abs() < 1e-8);
        assert!(output.actions.iter().any(|action| {
            action.symbol == "B"
                && action.action == "spin_off_cash_in_lieu_at_open"
                && (action.amount - 40.0).abs() < 1e-8
        }));
    }

    #[test]
    fn cash_replay_accrues_dividend_entitlement_without_early_cash() {
        let dates = vec![
            date(31),
            NaiveDate::from_ymd_opt(2025, 2, 3).unwrap(),
            NaiveDate::from_ymd_opt(2025, 2, 4).unwrap(),
        ];
        let signal_panel = panel(dates.clone(), vec![vec![100.0; 3], vec![50.0; 3]]);
        let mut execution_panel = signal_panel.clone();
        execution_panel.open[0][1] = 100.0;
        execution_panel.close[0][1] = 100.0;
        execution_panel.open[0][2] = 98.0;
        execution_panel.close[0][2] = 98.0;
        let action = CorporateAction {
            symbol: "A".into(),
            date: dates[2],
            payment_date: Some(NaiveDate::from_ymd_opt(2025, 2, 10).unwrap()),
            action_type: "cash_dividend".into(),
            rate: Some(1.0),
            old_rate: None,
            new_rate: None,
        };
        let output = run_cash_replay(
            &signal_panel,
            &execution_panel,
            &signals(dates.len()),
            &[action],
            settings(),
            "top",
            1,
            1,
            0.10,
        )
        .unwrap();
        assert_eq!(output.actions.len(), 1);
        assert_eq!(output.actions[0].action, "dividend_entitlement");
        assert_eq!(output.actions[0].amount, 100.0);
        assert!(output.unpaid_dividends.is_empty());
        assert_eq!(output.account.ledger.cash(), 0.0);
        assert!((output.days[2].equity - 9_900.0).abs() < 1e-8);
    }

    #[test]
    fn cash_replay_settles_and_removes_positions_after_stale_close_threshold() {
        let dates = vec![
            date(31),
            NaiveDate::from_ymd_opt(2025, 2, 3).unwrap(),
            NaiveDate::from_ymd_opt(2025, 2, 4).unwrap(),
            NaiveDate::from_ymd_opt(2025, 2, 5).unwrap(),
            NaiveDate::from_ymd_opt(2025, 2, 6).unwrap(),
        ];
        let signal_panel = panel(
            dates.clone(),
            vec![vec![100.0; dates.len()], vec![50.0; dates.len()]],
        );
        let mut execution_panel = signal_panel.clone();
        execution_panel.open[0][1] = 100.0;
        execution_panel.close[0][1] = 100.0;
        for day in 2..dates.len() {
            execution_panel.open[0][day] = f64::NAN;
            execution_panel.close[0][day] = f64::NAN;
        }

        let output = run_cash_replay(
            &signal_panel,
            &execution_panel,
            &signals(dates.len()),
            &[],
            settings(),
            "top",
            1,
            1,
            0.10,
        )
        .unwrap();

        assert_eq!(output.stale_marks.len(), 3);
        assert!(output.account.ledger.positions().is_empty());
        assert!((output.account.ledger.cash() - 7_000.0).abs() < 1e-8);
        assert!((output.days[4].equity - 7_000.0).abs() < 1e-8);
        assert_eq!(output.trades.len(), 1);
        assert!((output.trades[0].realized_pnl + 3_000.0).abs() < 1e-8);
        assert_eq!(output.actions.len(), 1);
        assert_eq!(output.actions[0].action, "assumed_delisting_settlement");
        assert!((output.actions[0].amount - 7_000.0).abs() < 1e-8);
    }

    #[test]
    fn cash_replay_executes_dca_as_separate_next_open_orders() {
        let dates = vec![
            date(31),
            NaiveDate::from_ymd_opt(2025, 2, 3).unwrap(),
            NaiveDate::from_ymd_opt(2025, 2, 4).unwrap(),
            NaiveDate::from_ymd_opt(2025, 2, 5).unwrap(),
        ];
        let signal_panel = panel(
            dates.clone(),
            vec![vec![100.0; dates.len()], vec![50.0; dates.len()]],
        );
        let output = run_cash_replay(
            &signal_panel,
            &signal_panel,
            &signals(dates.len()),
            &[],
            CashExecution {
                dca_installments: 3,
                ..settings()
            },
            "top",
            1,
            1,
            0.10,
        )
        .unwrap();
        let fills: Vec<_> = output
            .account
            .events()
            .iter()
            .filter_map(|event| match event {
                crate::s041_replay::ReplayEvent::Filled {
                    fill, filled_at, ..
                } => Some((filled_at.date_naive(), fill.shares)),
                _ => None,
            })
            .collect();
        assert_eq!(fills, vec![(dates[1], 34), (dates[2], 33), (dates[3], 33)]);
        assert_eq!(output.account.ledger.positions()["A"].shares, 100);
    }

    #[test]
    fn cash_replay_submits_stop_exit_only_after_the_triggering_close() {
        let dates = vec![
            date(31),
            NaiveDate::from_ymd_opt(2025, 2, 3).unwrap(),
            NaiveDate::from_ymd_opt(2025, 2, 4).unwrap(),
            NaiveDate::from_ymd_opt(2025, 2, 5).unwrap(),
        ];
        let signal_panel = panel(
            dates.clone(),
            vec![vec![100.0; dates.len()], vec![50.0; dates.len()]],
        );
        let mut execution_panel = signal_panel.clone();
        execution_panel.close[0][1] = 90.0;
        let output = run_cash_replay(
            &signal_panel,
            &execution_panel,
            &signals(dates.len()),
            &[],
            CashExecution {
                dca_installments: 3,
                stop_loss_fraction: 0.05,
                ..settings()
            },
            "top",
            1,
            1,
            0.10,
        )
        .unwrap();
        assert!(output.account.ledger.positions().is_empty());
        assert!(matches!(
            output.account.events(),
            [
                crate::s041_replay::ReplayEvent::OrderSubmitted { order: entry },
                crate::s041_replay::ReplayEvent::Filled { .. },
                crate::s041_replay::ReplayEvent::OrderSubmitted { order: exit },
                crate::s041_replay::ReplayEvent::Filled { .. }
            ] if entry.shares > 0
                && exit.shares < 0
                && exit.decided_at.date_naive() == dates[1]
        ));
        assert!(output
            .lifecycle_events
            .iter()
            .any(|event| { event.action == "dca_schedule_cancelled_by_stop" && event.shares > 0 }));
    }

    #[test]
    fn cash_replay_pyramids_a_winner_at_the_next_open_once() {
        let dates = vec![
            date(31),
            NaiveDate::from_ymd_opt(2025, 2, 3).unwrap(),
            NaiveDate::from_ymd_opt(2025, 2, 4).unwrap(),
            NaiveDate::from_ymd_opt(2025, 2, 5).unwrap(),
        ];
        let signal_panel = panel(
            dates.clone(),
            vec![vec![100.0; dates.len()], vec![50.0; dates.len()]],
        );
        let mut execution_panel = signal_panel.clone();
        execution_panel.close[0][1] = 110.0;
        execution_panel.open[0][2] = 110.0;
        let output = run_cash_replay(
            &signal_panel,
            &execution_panel,
            &signals(dates.len()),
            &[],
            CashExecution {
                long_fraction: 0.5,
                pyramid_profit_fraction: 0.10,
                pyramid_add_fraction: 0.25,
                ..settings()
            },
            "top",
            1,
            1,
            0.10,
        )
        .unwrap();
        let fills: Vec<i64> = output
            .account
            .events()
            .iter()
            .filter_map(|event| match event {
                crate::s041_replay::ReplayEvent::Filled { fill, .. } => Some(fill.shares),
                _ => None,
            })
            .collect();
        assert_eq!(fills, vec![50, 12]);
        assert_eq!(output.account.ledger.positions()["A"].shares, 62);
    }

    #[test]
    fn cash_replay_liquidates_when_month_end_has_no_eligible_candidates() {
        let dates = vec![
            date(31),
            NaiveDate::from_ymd_opt(2025, 2, 3).unwrap(),
            NaiveDate::from_ymd_opt(2025, 2, 28).unwrap(),
            NaiveDate::from_ymd_opt(2025, 3, 3).unwrap(),
        ];
        let signal_panel = panel(
            dates.clone(),
            vec![vec![100.0; dates.len()], vec![50.0; dates.len()]],
        );
        let mut signal_set = signals(dates.len());
        signal_set.signal[0][2] = false;
        signal_set.signal[1][2] = false;
        let output = run_cash_replay(
            &signal_panel,
            &signal_panel,
            &signal_set,
            &[],
            settings(),
            "top",
            1,
            1,
            0.10,
        )
        .unwrap();
        assert!(output.account.ledger.positions().is_empty());
        assert_eq!(output.rebalance_dates, vec![dates[0], dates[2]]);
    }

    #[test]
    fn cash_replay_blocks_a_selected_symbol_without_raw_rebalance_price() {
        let dates = vec![date(31), NaiveDate::from_ymd_opt(2025, 2, 3).unwrap()];
        let signal_panel = panel(
            dates.clone(),
            vec![vec![100.0; dates.len()], vec![50.0; dates.len()]],
        );
        let mut execution_panel = signal_panel.clone();
        execution_panel.close[0][0] = f64::NAN;
        let error = run_cash_replay(
            &signal_panel,
            &execution_panel,
            &signals(dates.len()),
            &[],
            settings(),
            "top",
            1,
            1,
            0.10,
        )
        .unwrap_err();
        assert!(error.contains("selected symbol A has no raw close"));
    }
}
