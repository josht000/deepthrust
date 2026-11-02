//! Chronological order and signal primitives for minute-bar execution experiments.

use std::collections::BTreeMap;

use chrono::{DateTime, Utc};

use crate::s041_cash_ledger::{Fill, Ledger};

#[derive(Clone, Debug)]
pub struct CompletedBar {
    pub symbol: String,
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
    pub close: f64,
    pub volume: u64,
}

#[derive(Clone, Debug)]
pub struct OpenQuote {
    pub symbol: String,
    pub timestamp: DateTime<Utc>,
    pub price: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Order {
    pub symbol: String,
    pub shares: i64,
    pub decided_at: DateTime<Utc>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ReplayEvent {
    OrderSubmitted {
        order: Order,
    },
    OrderCancelled {
        order: Order,
        cancelled_at: DateTime<Utc>,
    },
    OrderAdjusted {
        previous: Order,
        adjusted: Order,
        adjusted_at: DateTime<Utc>,
    },
    Filled {
        order: Order,
        fill: Fill,
        filled_at: DateTime<Utc>,
    },
}

#[derive(Clone, Copy, Debug)]
pub struct Execution {
    pub max_participation: f64,
    pub max_liquidity_age_seconds: i64,
    pub slippage_bps: f64,
    pub commission_per_share: f64,
    pub short_located: bool,
}

#[derive(Clone, Debug)]
pub struct PortfolioReplay {
    pub ledger: Ledger,
    pending: Vec<Order>,
    events: Vec<ReplayEvent>,
    last_event: Option<DateTime<Utc>>,
    used_capacity: BTreeMap<(String, DateTime<Utc>, DateTime<Utc>), (u64, u64, u64)>,
}

impl PortfolioReplay {
    pub fn new(initial_cash: f64) -> Result<Self, String> {
        Ok(Self {
            ledger: Ledger::new(initial_cash)?,
            pending: Vec::new(),
            events: Vec::new(),
            last_event: None,
            used_capacity: BTreeMap::new(),
        })
    }

    pub fn pending(&self) -> &[Order] {
        &self.pending
    }

    pub fn events(&self) -> &[ReplayEvent] {
        &self.events
    }

    pub fn submit(&mut self, order: Order) -> Result<(), String> {
        if order.symbol.is_empty() || order.shares == 0 || order.shares == i64::MIN {
            return Err("order requires a symbol and nonzero representable shares".into());
        }
        if self.last_event.is_some_and(|last| order.decided_at < last) {
            return Err("order decision precedes the last processed event".into());
        }
        self.events.push(ReplayEvent::OrderSubmitted {
            order: order.clone(),
        });
        self.pending.push(order);
        Ok(())
    }

    /// Cancel orders already submitted by the specified time for one symbol.
    pub fn cancel_pending(&mut self, symbol: &str, at: DateTime<Utc>) -> Result<usize, String> {
        if symbol.is_empty() {
            return Err("cancellation requires a symbol".into());
        }
        if self.last_event.is_some_and(|last| at < last) {
            return Err("cancellation precedes the last processed event".into());
        }
        let mut remaining = Vec::with_capacity(self.pending.len());
        let mut cancelled = Vec::new();
        for order in std::mem::take(&mut self.pending) {
            if order.symbol == symbol && order.decided_at <= at {
                cancelled.push(order);
            } else {
                remaining.push(order);
            }
        }
        let count = cancelled.len();
        self.pending = remaining;
        self.events.extend(
            cancelled
                .into_iter()
                .map(|order| ReplayEvent::OrderCancelled {
                    order,
                    cancelled_at: at,
                }),
        );
        self.last_event = Some(at);
        Ok(count)
    }

    pub fn adjust_pending_split(
        &mut self,
        symbol: &str,
        old_shares: u32,
        new_shares: u32,
        at: DateTime<Utc>,
    ) -> Result<usize, String> {
        if symbol.is_empty() || old_shares == 0 || new_shares == 0 {
            return Err("split adjustment requires a symbol and positive ratio".into());
        }
        if self.last_event.is_some_and(|last| at < last) {
            return Err("split adjustment precedes the last processed event".into());
        }
        let mut next = self.clone();
        let mut adjusted_count = 0;
        let mut remaining = Vec::with_capacity(next.pending.len());
        for mut order in std::mem::take(&mut next.pending) {
            if order.symbol != symbol {
                remaining.push(order);
                continue;
            }
            let previous = order.clone();
            let scaled =
                (i128::from(order.shares) * i128::from(new_shares)) / i128::from(old_shares);
            let scaled =
                i64::try_from(scaled).map_err(|_| "split-adjusted order overflows shares")?;
            if scaled == 0 {
                next.events.push(ReplayEvent::OrderCancelled {
                    order: previous,
                    cancelled_at: at,
                });
            } else {
                order.shares = scaled;
                next.events.push(ReplayEvent::OrderAdjusted {
                    previous,
                    adjusted: order.clone(),
                    adjusted_at: at,
                });
                remaining.push(order);
            }
            adjusted_count += 1;
        }
        next.pending = remaining;
        next.last_event = Some(at);
        *self = next;
        Ok(adjusted_count)
    }

    /// Process one quote atomically; unfilled shares remain pending.
    pub fn process_quote(
        &mut self,
        liquidity_bar: &CompletedBar,
        quote: &OpenQuote,
        execution: Execution,
        marks: &BTreeMap<String, f64>,
    ) -> Result<Vec<Fill>, String> {
        if self.last_event.is_some_and(|last| quote.timestamp < last) {
            return Err("quote precedes the last processed event".into());
        }
        validate_quote(liquidity_bar, quote, execution)?;
        let key = (
            liquidity_bar.symbol.clone(),
            liquidity_bar.start,
            liquidity_bar.end,
        );
        let limit = (liquidity_bar.volume as f64 * execution.max_participation).floor() as u64;
        let used = match self.used_capacity.get(&key) {
            Some(&(volume, prior_limit, used)) => {
                if volume != liquidity_bar.volume || prior_limit != limit {
                    return Err("liquidity bar or participation changed after use".into());
                }
                used
            }
            None => 0,
        };
        let mut available = limit
            .checked_sub(used)
            .ok_or("liquidity capacity was already exceeded")?;
        let mut next = self.clone();
        let mut remaining = Vec::new();
        let mut fills = Vec::new();
        for order in &self.pending {
            if order.symbol != quote.symbol || order.decided_at >= quote.timestamp {
                remaining.push(order.clone());
                continue;
            }
            if available == 0 {
                remaining.push(order.clone());
                continue;
            }
            let quantity =
                order.shares.unsigned_abs().min(available) as i64 * order.shares.signum();
            let clipped = Order {
                shares: quantity,
                ..order.clone()
            };
            let fill = execute_order(
                &mut next.ledger,
                &clipped,
                liquidity_bar,
                quote,
                execution,
                marks,
            )?;
            next.events.push(ReplayEvent::Filled {
                order: clipped,
                fill: fill.clone(),
                filled_at: quote.timestamp,
            });
            available -= fill.shares.unsigned_abs();
            let unfilled = order.shares - fill.shares;
            if unfilled != 0 {
                remaining.push(Order {
                    shares: unfilled,
                    ..order.clone()
                });
            }
            fills.push(fill);
        }
        next.pending = remaining;
        next.last_event = Some(quote.timestamp);
        next.used_capacity
            .insert(key, (liquidity_bar.volume, limit, limit - available));
        *self = next;
        Ok(fills)
    }
}

#[derive(Clone, Debug)]
pub struct RankedAsset {
    pub symbol: String,
    pub score: f64,
    pub close: f64,
}

#[derive(Clone, Copy, Debug)]
pub struct Allocation {
    pub long_count: usize,
    pub short_count: usize,
    pub long_fraction: f64,
    pub short_fraction: f64,
}

/// Produce close-based target quantities; exits are ordered before new entries.
/// Short availability and next-bar execution must be checked separately.
pub fn rebalance_orders(
    ledger: &Ledger,
    assets: &[RankedAsset],
    allocation: Allocation,
    marks: &BTreeMap<String, f64>,
    decided_at: DateTime<Utc>,
) -> Result<Vec<Order>, String> {
    if !allocation.long_fraction.is_finite()
        || !allocation.short_fraction.is_finite()
        || allocation.long_fraction < 0.0
        || allocation.short_fraction < 0.0
        || allocation.long_fraction + allocation.short_fraction > 1.0
        || (allocation.long_count == 0) != (allocation.long_fraction == 0.0)
        || (allocation.short_count == 0) != (allocation.short_fraction == 0.0)
        || allocation.long_count.saturating_add(allocation.short_count) > assets.len()
    {
        return Err("invalid or leveraged target allocation".into());
    }
    let equity = ledger.value(marks)?.equity;
    if equity <= 0.0 {
        return Err("cannot allocate nonpositive equity".into());
    }
    let mut ranked = assets.to_vec();
    let mut symbols = std::collections::BTreeSet::new();
    for asset in &ranked {
        if asset.symbol.is_empty()
            || !symbols.insert(asset.symbol.as_str())
            || !asset.score.is_finite()
            || !asset.close.is_finite()
            || asset.close <= 0.0
            || marks.get(&asset.symbol).copied() != Some(asset.close)
        {
            return Err(
                "ranked assets need unique symbols, finite scores, and matching close marks".into(),
            );
        }
    }
    ranked.sort_by(|a, b| {
        b.score
            .total_cmp(&a.score)
            .then_with(|| a.symbol.cmp(&b.symbol))
    });
    let mut targets = BTreeMap::<String, i64>::new();
    for (count, fraction, sign, list) in [
        (
            allocation.long_count,
            allocation.long_fraction,
            1,
            &ranked[..allocation.long_count],
        ),
        (
            allocation.short_count,
            allocation.short_fraction,
            -1,
            &ranked[ranked.len() - allocation.short_count..],
        ),
    ] {
        if count == 0 {
            continue;
        }
        for asset in list {
            let shares = (equity * fraction / count as f64 / asset.close).floor();
            if !shares.is_finite() || shares >= i64::MAX as f64 {
                return Err("target share count is out of range".into());
            }
            targets.insert(asset.symbol.clone(), shares as i64 * sign);
        }
    }
    for symbol in ledger.positions().keys() {
        if !marks.contains_key(symbol) {
            return Err(format!("missing close mark for held symbol {symbol}"));
        }
        targets.entry(symbol.clone()).or_default();
    }
    let mut reductions = Vec::new();
    let mut additions = Vec::new();
    for (symbol, desired) in targets {
        let current = ledger.positions().get(&symbol).map_or(0, |p| p.shares);
        if current == desired {
            continue;
        }
        let retained = if current.signum() == desired.signum() {
            current.signum() * current.unsigned_abs().min(desired.unsigned_abs()) as i64
        } else {
            0
        };
        if current != retained {
            reductions.push(Order {
                symbol: symbol.clone(),
                shares: retained - current,
                decided_at,
            });
        }
        if desired != retained {
            additions.push(Order {
                symbol,
                shares: desired - retained,
                decided_at,
            });
        }
    }
    reductions.extend(additions);
    Ok(reductions)
}

/// Fill at a later observed open. Capacity uses the preceding completed bar,
/// never the fill bar's as-yet-unknown volume.
pub fn execute_order(
    ledger: &mut Ledger,
    order: &Order,
    liquidity_bar: &CompletedBar,
    quote: &OpenQuote,
    execution: Execution,
    marks: &BTreeMap<String, f64>,
) -> Result<Fill, String> {
    validate_quote(liquidity_bar, quote, execution)?;
    if order.symbol != quote.symbol {
        return Err("order, liquidity bar and quote must use the same symbol".into());
    }
    if order.decided_at >= quote.timestamp {
        return Err("order decision and completed liquidity bar must precede the fill".into());
    }
    let capacity = (liquidity_bar.volume as f64 * execution.max_participation).floor() as u64;
    ledger.execute(
        &order.symbol,
        order.shares,
        quote.price,
        execution.slippage_bps,
        execution.commission_per_share,
        capacity,
        execution.short_located,
        marks,
    )
}

fn validate_quote(
    bar: &CompletedBar,
    quote: &OpenQuote,
    execution: Execution,
) -> Result<(), String> {
    if bar.symbol.is_empty() || bar.symbol != quote.symbol {
        return Err("liquidity bar and quote must use the same nonempty symbol".into());
    }
    if bar.start >= bar.end || bar.end >= quote.timestamp {
        return Err("completed liquidity bar must precede the quote".into());
    }
    let max_age = chrono::Duration::try_seconds(execution.max_liquidity_age_seconds)
        .ok_or("maximum liquidity age is out of range")?;
    if execution.max_liquidity_age_seconds <= 0
        || quote.timestamp.signed_duration_since(bar.end) > max_age
    {
        return Err("liquidity bar is stale or maximum age is invalid".into());
    }
    if !bar.close.is_finite() || bar.close <= 0.0 || !quote.price.is_finite() || quote.price <= 0.0
    {
        return Err("bar close and quote price must be positive and finite".into());
    }
    if !execution.max_participation.is_finite()
        || execution.max_participation <= 0.0
        || execution.max_participation > 1.0
    {
        return Err("participation must be in (0, 1]".into());
    }
    if !execution.slippage_bps.is_finite()
        || !(0.0..10_000.0).contains(&execution.slippage_bps)
        || !execution.commission_per_share.is_finite()
        || execution.commission_per_share < 0.0
    {
        return Err("invalid execution costs".into());
    }
    Ok(())
}

/// Divide a fixed integer target into deterministic, nearly equal installments.
pub fn installments(target: i64, count: usize) -> Result<Vec<i64>, String> {
    if target == 0 || target == i64::MIN || count == 0 || count > i64::MAX as usize {
        return Err("installments need a nonzero target and positive count".into());
    }
    let count_i64 = count as i64;
    let base = target / count_i64;
    let remainder = target % count_i64;
    Ok((0..count)
        .map(|index| {
            base + if index < remainder.unsigned_abs() as usize {
                target.signum()
            } else {
                0
            }
        })
        .collect())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SignalAction {
    Hold,
    Exit(i64),
    Add(i64),
}

/// Evaluate a completed close, so an exit or add can only be executed later.
pub fn position_action(
    shares: i64,
    average_entry: f64,
    close: f64,
    stop_fraction: f64,
    add_profit_fraction: f64,
    add_shares: i64,
    already_added: bool,
) -> Result<SignalAction, String> {
    if shares == 0
        || shares == i64::MIN
        || !average_entry.is_finite()
        || average_entry <= 0.0
        || !close.is_finite()
        || close <= 0.0
        || !stop_fraction.is_finite()
        || !(0.0..1.0).contains(&stop_fraction)
        || !add_profit_fraction.is_finite()
        || add_profit_fraction < 0.0
        || add_shares < 0
    {
        return Err("invalid position-action inputs".into());
    }
    let stop_hit = if shares > 0 {
        close <= average_entry * (1.0 - stop_fraction)
    } else {
        close >= average_entry * (1.0 + stop_fraction)
    };
    if stop_fraction > 0.0 && stop_hit {
        return Ok(SignalAction::Exit(-shares));
    }
    let profit_hit = if shares > 0 {
        close >= average_entry * (1.0 + add_profit_fraction)
    } else {
        close <= average_entry * (1.0 - add_profit_fraction)
    };
    if !already_added && add_shares > 0 && profit_hit {
        return Ok(SignalAction::Add(shares.signum() * add_shares));
    }
    Ok(SignalAction::Hold)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn time(minute: u32) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 10, 1, 13, minute, 0).unwrap()
    }

    #[test]
    fn replay_uses_completed_volume_and_next_open() {
        let mut ledger = Ledger::new(10_000.0).unwrap();
        let bar = CompletedBar {
            symbol: "A".into(),
            start: time(30),
            end: time(32),
            close: 100.0,
            volume: 100,
        };
        let quote = OpenQuote {
            symbol: "A".into(),
            timestamp: time(33),
            price: 102.0,
        };
        let mut order = Order {
            symbol: "A".into(),
            shares: 20,
            decided_at: time(32),
        };
        let config = Execution {
            max_participation: 0.1,
            max_liquidity_age_seconds: 60,
            slippage_bps: 0.0,
            commission_per_share: 0.0,
            short_located: false,
        };
        let fill =
            execute_order(&mut ledger, &order, &bar, &quote, config, &BTreeMap::new()).unwrap();
        assert_eq!(fill.shares, 10);
        assert_eq!(fill.price, 102.0);
        order.decided_at = time(33);
        assert!(
            execute_order(&mut ledger, &order, &bar, &quote, config, &BTreeMap::new()).is_err()
        );
    }

    #[test]
    fn pending_orders_share_bar_capacity_and_continue_on_later_bars() {
        let mut replay = PortfolioReplay::new(10_000.0).unwrap();
        replay
            .submit(Order {
                symbol: "A".into(),
                shares: 15,
                decided_at: time(30),
            })
            .unwrap();
        let bar = CompletedBar {
            symbol: "A".into(),
            start: time(30),
            end: time(31),
            close: 100.0,
            volume: 100,
        };
        let quote = OpenQuote {
            symbol: "A".into(),
            timestamp: time(32),
            price: 100.0,
        };
        let config = Execution {
            max_participation: 0.1,
            max_liquidity_age_seconds: 60,
            slippage_bps: 0.0,
            commission_per_share: 0.0,
            short_located: false,
        };
        let fills = replay
            .process_quote(&bar, &quote, config, &BTreeMap::new())
            .unwrap();
        assert_eq!(fills[0].shares, 10);
        assert_eq!(replay.pending()[0].shares, 5);
        assert!(replay
            .process_quote(&bar, &quote, config, &BTreeMap::new())
            .unwrap()
            .is_empty());
        let next_bar = CompletedBar {
            start: time(32),
            end: time(33),
            ..bar
        };
        let next_quote = OpenQuote {
            timestamp: time(34),
            ..quote
        };
        let marks = [("A".to_owned(), 100.0)].into_iter().collect();
        assert_eq!(
            replay
                .process_quote(&next_bar, &next_quote, config, &marks)
                .unwrap()[0]
                .shares,
            5
        );
        assert!(replay.pending().is_empty());
    }

    #[test]
    fn replay_rejects_bad_quotes_and_rolls_back_failed_fills() {
        let mut replay = PortfolioReplay::new(1_000.0).unwrap();
        let bar = CompletedBar {
            symbol: "A".into(),
            start: time(30),
            end: time(31),
            close: 100.0,
            volume: 100,
        };
        let quote = OpenQuote {
            symbol: "A".into(),
            timestamp: time(32),
            price: 100.0,
        };
        let config = Execution {
            max_participation: 0.1,
            max_liquidity_age_seconds: 60,
            slippage_bps: 0.0,
            commission_per_share: 0.0,
            short_located: false,
        };
        let wrong_symbol = OpenQuote {
            symbol: "B".into(),
            ..quote.clone()
        };
        assert!(replay
            .process_quote(&bar, &wrong_symbol, config, &BTreeMap::new())
            .is_err());
        let stale = OpenQuote {
            timestamp: time(33),
            ..quote.clone()
        };
        assert!(replay
            .process_quote(&bar, &stale, config, &BTreeMap::new())
            .is_err());
        replay
            .submit(Order {
                symbol: "A".into(),
                shares: 11,
                decided_at: time(30),
            })
            .unwrap();
        let fill = replay
            .process_quote(&bar, &quote, config, &BTreeMap::new())
            .unwrap();
        assert_eq!(fill[0].shares, 10);
        assert_eq!(replay.pending()[0].shares, 1);
        let changed_volume = CompletedBar {
            volume: 200,
            ..bar.clone()
        };
        assert!(replay
            .process_quote(&changed_volume, &quote, config, &BTreeMap::new())
            .is_err());
        let earlier = OpenQuote {
            timestamp: time(31),
            ..quote.clone()
        };
        assert!(replay
            .process_quote(&bar, &earlier, config, &BTreeMap::new())
            .is_err());
        let next_bar = CompletedBar {
            start: time(32),
            end: time(33),
            ..bar
        };
        let next_quote = OpenQuote {
            timestamp: time(34),
            ..quote
        };
        let marks = [("A".to_owned(), 100.0)].into_iter().collect();
        assert!(replay
            .process_quote(&next_bar, &next_quote, config, &marks)
            .is_err());
        assert_eq!(replay.pending()[0].shares, 1);
        assert_eq!(replay.ledger.positions()["A"].shares, 10);
    }

    #[test]
    fn cancellation_does_not_remove_other_symbols_or_future_orders() {
        let mut replay = PortfolioReplay::new(1_000.0).unwrap();
        for (symbol, minute) in [("A", 30), ("A", 32), ("B", 30)] {
            replay
                .submit(Order {
                    symbol: symbol.into(),
                    shares: 1,
                    decided_at: time(minute),
                })
                .unwrap();
        }
        assert_eq!(replay.cancel_pending("A", time(31)).unwrap(), 1);
        assert_eq!(replay.pending().len(), 2);
        assert!(replay.cancel_pending("B", time(30)).is_err());
        assert_eq!(replay.pending().len(), 2);
    }

    #[test]
    fn split_adjusts_pending_quantities_and_cancels_zero_share_orders() {
        let mut replay = PortfolioReplay::new(1_000.0).unwrap();
        replay
            .submit(Order {
                symbol: "A".into(),
                shares: 30,
                decided_at: time(30),
            })
            .unwrap();
        replay
            .submit(Order {
                symbol: "B".into(),
                shares: 1,
                decided_at: time(30),
            })
            .unwrap();
        assert_eq!(replay.adjust_pending_split("A", 1, 4, time(31)).unwrap(), 1);
        assert_eq!(replay.pending()[0].shares, 120);
        assert_eq!(replay.adjust_pending_split("B", 3, 1, time(32)).unwrap(), 1);
        assert_eq!(replay.pending().len(), 1);
        assert!(matches!(
            replay.events().last(),
            Some(ReplayEvent::OrderCancelled { order, cancelled_at })
                if order.symbol == "B" && *cancelled_at == time(32)
        ));
    }

    #[test]
    fn replay_records_submissions_partial_fills_and_cancellations() {
        let mut replay = PortfolioReplay::new(10_000.0).unwrap();
        replay
            .submit(Order {
                symbol: "A".into(),
                shares: 15,
                decided_at: time(30),
            })
            .unwrap();
        let bar = CompletedBar {
            symbol: "A".into(),
            start: time(30),
            end: time(31),
            close: 100.0,
            volume: 100,
        };
        let quote = OpenQuote {
            symbol: "A".into(),
            timestamp: time(32),
            price: 100.0,
        };
        let config = Execution {
            max_participation: 0.1,
            max_liquidity_age_seconds: 60,
            slippage_bps: 0.0,
            commission_per_share: 0.0,
            short_located: false,
        };
        replay
            .process_quote(&bar, &quote, config, &BTreeMap::new())
            .unwrap();
        assert_eq!(replay.pending()[0].shares, 5);
        assert!(matches!(
            replay.events(),
            [
                ReplayEvent::OrderSubmitted { order },
                ReplayEvent::Filled {
                    order: filled_order,
                    fill,
                    filled_at
                }
            ] if order.shares == 15
                && filled_order.shares == 10
                && fill.shares == 10
                && *filled_at == quote.timestamp
        ));
        assert_eq!(replay.cancel_pending("A", time(33)).unwrap(), 1);
        assert!(matches!(
            replay.events().last(),
            Some(ReplayEvent::OrderCancelled {
                order,
                cancelled_at
            }) if order.shares == 5 && *cancelled_at == time(33)
        ));
    }

    #[test]
    fn dca_and_stop_pyramid_use_whole_shares_and_direction() {
        assert_eq!(installments(10, 3).unwrap(), vec![4, 3, 3]);
        assert_eq!(installments(-10, 3).unwrap(), vec![-4, -3, -3]);
        assert_eq!(
            position_action(10, 100.0, 90.0, 0.05, 0.1, 3, false).unwrap(),
            SignalAction::Exit(-10)
        );
        assert_eq!(
            position_action(-10, 100.0, 90.0, 0.05, 0.1, 3, false).unwrap(),
            SignalAction::Add(-3)
        );
        assert_eq!(
            position_action(-10, 100.0, 110.0, 0.05, 0.1, 3, false).unwrap(),
            SignalAction::Exit(10)
        );
        assert_eq!(
            position_action(10, 100.0, 120.0, 0.05, 0.1, 3, true).unwrap(),
            SignalAction::Hold
        );
    }

    #[test]
    fn rebalance_uses_close_scores_whole_shares_and_sells_before_buying() {
        let mut ledger = Ledger::new(1_000.0).unwrap();
        let mut marks = BTreeMap::new();
        marks.insert("OLD".into(), 100.0);
        marks.insert("UP".into(), 30.0);
        marks.insert("DOWN".into(), 20.0);
        ledger
            .execute("OLD", 5, 100.0, 0.0, 0.0, 5, false, &marks)
            .unwrap();
        let assets = vec![
            RankedAsset {
                symbol: "UP".into(),
                score: 0.2,
                close: 30.0,
            },
            RankedAsset {
                symbol: "DOWN".into(),
                score: -0.3,
                close: 20.0,
            },
        ];
        let orders = rebalance_orders(
            &ledger,
            &assets,
            Allocation {
                long_count: 1,
                short_count: 1,
                long_fraction: 0.5,
                short_fraction: 0.5,
            },
            &marks,
            time(32),
        )
        .unwrap();
        assert_eq!((orders[0].symbol.as_str(), orders[0].shares), ("OLD", -5));
        assert_eq!((orders[1].symbol.as_str(), orders[1].shares), ("DOWN", -25));
        assert_eq!((orders[2].symbol.as_str(), orders[2].shares), ("UP", 16));
    }
}
