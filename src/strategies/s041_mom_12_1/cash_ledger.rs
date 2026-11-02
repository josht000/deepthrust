//! Cash-account primitives for S041 execution research.

use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq)]
pub struct Position {
    pub shares: i64,
    pub average_price: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Fill {
    pub symbol: String,
    pub shares: i64,
    pub price: f64,
    pub commission: f64,
    pub realized_pnl: f64,
}

#[derive(Clone, Debug)]
pub struct Ledger {
    cash: f64,
    positions: BTreeMap<String, Position>,
    dividend_receivables: BTreeMap<String, f64>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Valuation {
    pub equity: f64,
    pub gross_exposure: f64,
    pub net_exposure: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SpinOffReceipt {
    pub symbol: String,
    pub shares: i64,
    pub cash_in_lieu: f64,
    pub realized_pnl: f64,
    pub value_weight: f64,
}

impl Ledger {
    pub fn new(initial_cash: f64) -> Result<Self, String> {
        if !initial_cash.is_finite() || initial_cash <= 0.0 {
            return Err("initial cash must be positive and finite".into());
        }
        Ok(Self {
            cash: initial_cash,
            positions: BTreeMap::new(),
            dividend_receivables: BTreeMap::new(),
        })
    }

    pub fn cash(&self) -> f64 {
        self.cash
    }

    pub fn positions(&self) -> &BTreeMap<String, Position> {
        &self.positions
    }

    pub fn value(&self, marks: &BTreeMap<String, f64>) -> Result<Valuation, String> {
        let mut net = 0.0;
        let mut gross = 0.0;
        for (symbol, position) in &self.positions {
            let price = *marks
                .get(symbol)
                .ok_or_else(|| format!("missing mark for {symbol}"))?;
            validate_price(price)?;
            let value = position.shares as f64 * price;
            net += value;
            gross += value.abs();
        }
        let equity = self.cash + net + self.dividend_receivables.values().sum::<f64>();
        if !equity.is_finite() || !gross.is_finite() {
            return Err("nonfinite account valuation".into());
        }
        Ok(Valuation {
            equity,
            gross_exposure: gross,
            net_exposure: net,
        })
    }

    /// Execute against an observed price; liquidity must be known before the fill.
    pub fn execute(
        &mut self,
        symbol: &str,
        requested_shares: i64,
        observed_price: f64,
        slippage_bps: f64,
        commission_per_share: f64,
        available_shares: u64,
        short_located: bool,
        marks: &BTreeMap<String, f64>,
    ) -> Result<Fill, String> {
        if symbol.is_empty() || requested_shares == 0 || requested_shares == i64::MIN {
            return Err("order requires a symbol and nonzero representable shares".into());
        }
        validate_price(observed_price)?;
        if !slippage_bps.is_finite() || !(0.0..10_000.0).contains(&slippage_bps) {
            return Err("slippage must be finite and in [0, 10000) basis points".into());
        }
        if !commission_per_share.is_finite() || commission_per_share < 0.0 {
            return Err("commission per share must be nonnegative and finite".into());
        }
        let count = requested_shares.unsigned_abs().min(available_shares);
        if count == 0 || count > i64::MAX as u64 {
            return Err("order has no available whole-share liquidity".into());
        }
        let shares = (count as i64) * requested_shares.signum();
        let previous = self.positions.get(symbol).cloned().unwrap_or(Position {
            shares: 0,
            average_price: 0.0,
        });
        let next_shares = previous
            .shares
            .checked_add(shares)
            .ok_or("share count overflow")?;
        if next_shares < previous.shares.min(0) && !short_located {
            return Err(format!("short locate is required for {symbol}"));
        }
        let price = observed_price * (1.0 + shares.signum() as f64 * slippage_bps / 10_000.0);
        validate_price(price)?;
        let commission = commission_per_share * count as f64;
        let closed = if previous.shares.signum() != shares.signum() {
            previous.shares.unsigned_abs().min(count)
        } else {
            0
        };
        let realized =
            closed as f64 * (price - previous.average_price) * previous.shares.signum() as f64
                - commission;
        let basis = if next_shares == 0 {
            0.0
        } else if previous.shares == 0 || previous.shares.signum() != next_shares.signum() {
            price
        } else if previous.shares.signum() == shares.signum() {
            (previous.average_price * previous.shares.unsigned_abs() as f64 + price * count as f64)
                / next_shares.unsigned_abs() as f64
        } else {
            previous.average_price
        };
        let mut next = self.clone();
        next.cash -= shares as f64 * price + commission;
        if next_shares == 0 {
            next.positions.remove(symbol);
        } else {
            next.positions.insert(
                symbol.to_owned(),
                Position {
                    shares: next_shares,
                    average_price: basis,
                },
            );
        }
        let mut fill_marks = marks.clone();
        fill_marks.insert(symbol.to_owned(), observed_price);
        let before = self.value(&fill_marks)?;
        let after = next.value(&fill_marks)?;
        // Existing price moves can breach the limit; only exposure-reducing trades
        // may then proceed until the account is back within the limit.
        let short_collateral = (after.gross_exposure - after.net_exposure) / 2.0;
        if (after.equity <= 0.0
            || after.gross_exposure > after.equity + 1e-8
            || next.cash < short_collateral - 1e-8)
            && after.gross_exposure >= before.gross_exposure - 1e-8
        {
            return Err("order would exceed no-leverage gross exposure limit".into());
        }
        *self = next;
        Ok(Fill {
            symbol: symbol.to_owned(),
            shares,
            price,
            commission,
            realized_pnl: realized,
        })
    }

    pub fn apply_dividend(&mut self, symbol: &str, per_share: f64) -> Result<f64, String> {
        if !per_share.is_finite() || per_share < 0.0 {
            return Err("dividend must be nonnegative and finite".into());
        }
        let amount = self
            .positions
            .get(symbol)
            .map_or(0.0, |p| p.shares as f64 * per_share);
        if !(self.cash + amount).is_finite() {
            return Err("dividend would make cash nonfinite".into());
        }
        self.cash += amount;
        Ok(amount)
    }

    /// Record ex-date entitlement without making the payment available as cash.
    pub fn accrue_dividend(
        &mut self,
        action_id: &str,
        symbol: &str,
        per_share: f64,
    ) -> Result<f64, String> {
        if action_id.is_empty() || self.dividend_receivables.contains_key(action_id) {
            return Err("dividend action ID must be nonempty and unique".into());
        }
        if !per_share.is_finite() || per_share < 0.0 {
            return Err("dividend must be nonnegative and finite".into());
        }
        let amount = self
            .positions
            .get(symbol)
            .map_or(0.0, |p| p.shares as f64 * per_share);
        if !amount.is_finite() {
            return Err("dividend entitlement is nonfinite".into());
        }
        self.dividend_receivables
            .insert(action_id.to_owned(), amount);
        Ok(amount)
    }

    pub fn settle_dividend(&mut self, action_id: &str) -> Result<f64, String> {
        let amount = *self
            .dividend_receivables
            .get(action_id)
            .ok_or_else(|| format!("unknown dividend action {action_id}"))?;
        if !(self.cash + amount).is_finite() {
            return Err("dividend payment would make cash nonfinite".into());
        }
        self.dividend_receivables.remove(action_id);
        self.cash += amount;
        Ok(amount)
    }

    /// Fractional shares resulting from a split are settled at the supplied cash-in-lieu price.
    pub fn apply_split(
        &mut self,
        symbol: &str,
        old_shares: u32,
        new_shares: u32,
        cash_in_lieu_price: f64,
    ) -> Result<f64, String> {
        if old_shares == 0 || new_shares == 0 {
            return Err("split ratio must have positive integer terms".into());
        }
        validate_price(cash_in_lieu_price)?;
        let Some(position) = self.positions.get(symbol).cloned() else {
            return Ok(0.0);
        };
        let numerator = (position.shares as i128) * i128::from(new_shares);
        let whole = numerator / i128::from(old_shares);
        let remainder = numerator % i128::from(old_shares);
        let whole = i64::try_from(whole).map_err(|_| "split share count overflow")?;
        let cash_in_lieu = remainder as f64 / old_shares as f64 * cash_in_lieu_price;
        let basis = position.average_price * old_shares as f64 / new_shares as f64;
        if !(self.cash + cash_in_lieu).is_finite() || !basis.is_finite() {
            return Err("split would produce a nonfinite account".into());
        }
        self.cash += cash_in_lieu;
        if whole == 0 {
            self.positions.remove(symbol);
        } else {
            self.positions.insert(
                symbol.to_owned(),
                Position {
                    shares: whole,
                    average_price: basis,
                },
            );
        }
        Ok(cash_in_lieu)
    }

    pub fn apply_spinoff(
        &mut self,
        source_symbol: &str,
        components: &[(String, f64, f64)],
    ) -> Result<Vec<SpinOffReceipt>, String> {
        if source_symbol.is_empty() || components.is_empty() {
            return Err("spin-off requires a source symbol and distributed assets".into());
        }
        let source = self
            .positions
            .get(source_symbol)
            .cloned()
            .ok_or_else(|| format!("no held position for spin-off source {source_symbol}"))?;
        if components
            .iter()
            .filter(|(symbol, _, _)| symbol == source_symbol)
            .count()
            != 1
        {
            return Err(format!(
                "spin-off for {source_symbol} must include the continuing source asset exactly once"
            ));
        }
        for (index, (symbol, ratio, price)) in components.iter().enumerate() {
            if symbol.is_empty()
                || !ratio.is_finite()
                || *ratio <= 0.0
                || components[..index]
                    .iter()
                    .any(|(previous, _, _)| previous == symbol)
            {
                return Err(
                    "spin-off component symbols and share ratios must be unique and positive"
                        .into(),
                );
            }
            validate_price(*price)?;
        }
        let total_value_per_source_share = components
            .iter()
            .map(|(_, ratio, price)| ratio * price)
            .sum::<f64>();
        if !total_value_per_source_share.is_finite() || total_value_per_source_share <= 0.0 {
            return Err("spin-off distribution value must be positive and finite".into());
        }

        let mut next = self.clone();
        next.positions.remove(source_symbol);
        let mut receipts = Vec::with_capacity(components.len());
        for (symbol, ratio, price) in components {
            let exact_shares = source.shares as f64 * ratio;
            if !exact_shares.is_finite()
                || exact_shares <= i64::MIN as f64
                || exact_shares >= i64::MAX as f64
            {
                return Err(format!("spin-off share count overflow for {symbol}"));
            }
            let shares = exact_shares.trunc() as i64;
            let fractional_shares = exact_shares - shares as f64;
            let cash_in_lieu = fractional_shares * price;
            let average_price = source.average_price * price / total_value_per_source_share;
            if !average_price.is_finite() || average_price <= 0.0 {
                return Err(format!(
                    "invalid allocated basis for spin-off asset {symbol}"
                ));
            }
            let mut realized_pnl = fractional_shares * (price - average_price);
            if shares != 0 {
                if let Some(previous) = next.positions.remove(symbol) {
                    let combined_shares = previous
                        .shares
                        .checked_add(shares)
                        .ok_or("spin-off share count overflow")?;
                    if previous.shares.signum() == shares.signum() {
                        let combined_basis = (previous.average_price
                            * previous.shares.unsigned_abs() as f64
                            + average_price * shares.unsigned_abs() as f64)
                            / combined_shares.unsigned_abs() as f64;
                        next.positions.insert(
                            symbol.clone(),
                            Position {
                                shares: combined_shares,
                                average_price: combined_basis,
                            },
                        );
                    } else {
                        let closed = previous.shares.unsigned_abs().min(shares.unsigned_abs());
                        realized_pnl += closed as f64
                            * (price - previous.average_price)
                            * previous.shares.signum() as f64;
                        if combined_shares != 0 {
                            let basis = if combined_shares.signum() == previous.shares.signum() {
                                previous.average_price
                            } else {
                                average_price
                            };
                            next.positions.insert(
                                symbol.clone(),
                                Position {
                                    shares: combined_shares,
                                    average_price: basis,
                                },
                            );
                        }
                    }
                } else {
                    next.positions.insert(
                        symbol.clone(),
                        Position {
                            shares,
                            average_price,
                        },
                    );
                }
            }
            next.cash += cash_in_lieu;
            if !next.cash.is_finite() || !realized_pnl.is_finite() {
                return Err("spin-off settlement produced a nonfinite account".into());
            }
            receipts.push(SpinOffReceipt {
                symbol: symbol.clone(),
                shares,
                cash_in_lieu,
                realized_pnl,
                value_weight: ratio * price / total_value_per_source_share,
            });
        }
        *self = next;
        Ok(receipts)
    }

    pub fn settle_delisting(
        &mut self,
        symbol: &str,
        last_mark: f64,
        delisting_return: f64,
    ) -> Result<f64, String> {
        validate_price(last_mark)?;
        if !delisting_return.is_finite() || delisting_return <= -1.0 {
            return Err("delisting return must be finite and greater than -100%".into());
        }
        let position = self
            .positions
            .get(symbol)
            .ok_or_else(|| format!("no held position to settle for {symbol}"))?;
        let settlement_price = last_mark * (1.0 + delisting_return);
        validate_price(settlement_price)?;
        let cash_delta = position.shares as f64 * settlement_price;
        if !(self.cash + cash_delta).is_finite() {
            return Err("delisting settlement would make cash nonfinite".into());
        }
        self.cash += cash_delta;
        self.positions.remove(symbol);
        Ok(cash_delta)
    }

    pub fn charge_borrow(
        &mut self,
        marks: &BTreeMap<String, f64>,
        annual_rate: f64,
        calendar_days: u32,
    ) -> Result<f64, String> {
        if !annual_rate.is_finite() || annual_rate < 0.0 {
            return Err("borrow rate must be nonnegative and finite".into());
        }
        let mut short_value = 0.0;
        for (symbol, position) in &self.positions {
            if position.shares < 0 {
                let price = *marks
                    .get(symbol)
                    .ok_or_else(|| format!("missing mark for {symbol}"))?;
                validate_price(price)?;
                short_value += position.shares.unsigned_abs() as f64 * price;
            }
        }
        let charge = short_value * annual_rate * calendar_days as f64 / 365.0;
        if !charge.is_finite() || !(self.cash - charge).is_finite() {
            return Err("borrow charge would make cash nonfinite".into());
        }
        self.cash -= charge;
        Ok(charge)
    }
}

fn validate_price(price: f64) -> Result<(), String> {
    if !price.is_finite() || price <= 0.0 {
        return Err("price must be positive and finite".into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn marks(symbol: &str, price: f64) -> BTreeMap<String, f64> {
        [(symbol.to_owned(), price)].into_iter().collect()
    }

    #[test]
    fn cash_no_leverage_and_partial_liquidity() {
        let mut book = Ledger::new(10_000.0).unwrap();
        let fill = book
            .execute("A", 200, 100.0, 0.0, 0.0, 50, false, &marks("A", 100.0))
            .unwrap();
        assert_eq!(fill.shares, 50);
        assert_eq!(book.cash(), 5_000.0);
        assert!(book
            .execute("A", 51, 100.0, 0.0, 0.0, 100, false, &marks("A", 100.0))
            .is_err());
        assert_eq!(book.positions()["A"].shares, 50);
        book.execute("A", 50, 100.0, 0.0, 0.0, 50, false, &marks("A", 100.0))
            .unwrap();
        assert_eq!(
            book.value(&marks("A", 100.0)).unwrap().gross_exposure,
            10_000.0
        );
    }

    #[test]
    fn short_locate_and_no_leverage_include_short_proceeds() {
        let mut book = Ledger::new(10_000.0).unwrap();
        assert!(book
            .execute("A", -1, 100.0, 0.0, 0.0, 1, false, &marks("A", 100.0))
            .is_err());
        book.execute("A", -100, 100.0, 0.0, 0.0, 100, true, &marks("A", 100.0))
            .unwrap();
        assert_eq!(book.cash(), 20_000.0);
        assert_eq!(book.value(&marks("A", 100.0)).unwrap().equity, 10_000.0);
        assert!(book
            .execute("A", -1, 100.0, 0.0, 0.0, 1, true, &marks("A", 100.0))
            .is_err());
        let fill = book
            .execute("A", 50, 90.0, 0.0, 0.0, 50, false, &marks("A", 90.0))
            .unwrap();
        assert_eq!(fill.realized_pnl, 500.0);
    }

    #[test]
    fn dividend_split_and_short_borrow_are_cash_accounted() {
        let mut book = Ledger::new(10_000.0).unwrap();
        book.execute("A", 3, 100.0, 0.0, 0.0, 3, false, &marks("A", 100.0))
            .unwrap();
        assert_eq!(book.apply_dividend("A", 2.0).unwrap(), 6.0);
        assert_eq!(book.apply_split("A", 2, 1, 200.0).unwrap(), 100.0);
        assert_eq!(book.positions()["A"].shares, 1);
        assert_eq!(book.positions()["A"].average_price, 200.0);
        let mut short = Ledger::new(10_000.0).unwrap();
        short
            .execute("A", -10, 100.0, 0.0, 0.0, 10, true, &marks("A", 100.0))
            .unwrap();
        assert_eq!(short.apply_dividend("A", 2.0).unwrap(), -20.0);
        assert!((short.charge_borrow(&marks("A", 100.0), 0.0365, 10).unwrap() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn spinoff_preserves_whole_share_account_value_and_settles_fractional_shares() {
        let mut book = Ledger::new(1_000.0).unwrap();
        book.execute("RTX", 3, 86.01, 0.0, 0.0, 3, false, &marks("RTX", 86.01))
            .unwrap();
        let before = book.value(&marks("RTX", 86.01)).unwrap().equity;
        let receipts = book
            .apply_spinoff(
                "RTX",
                &[
                    ("RTX".into(), 1.0, 51.16),
                    ("CARR".into(), 1.0, 13.75),
                    ("OTIS".into(), 0.5, 43.75),
                ],
            )
            .unwrap();
        let mut market = BTreeMap::new();
        market.insert("RTX".into(), 51.16);
        market.insert("CARR".into(), 13.75);
        market.insert("OTIS".into(), 43.75);
        let after = book.value(&market).unwrap().equity;
        assert_eq!(book.positions()["RTX"].shares, 3);
        assert_eq!(book.positions()["CARR"].shares, 3);
        assert_eq!(book.positions()["OTIS"].shares, 1);
        assert!((receipts[2].cash_in_lieu - 21.875).abs() < 1e-10);
        assert!((after - before - 2.325).abs() < 1e-8);
        let allocated_basis = book.positions()["RTX"].average_price * 3.0
            + book.positions()["CARR"].average_price * 3.0
            + book.positions()["OTIS"].average_price * 1.5;
        assert!((allocated_basis - 258.03).abs() < 1e-8);
    }

    #[test]
    fn slippage_and_commission_are_not_free() {
        let mut book = Ledger::new(10_000.0).unwrap();
        let fill = book
            .execute("A", 10, 100.0, 10.0, 0.05, 10, false, &marks("A", 100.0))
            .unwrap();
        assert!((fill.price - 100.1).abs() < 1e-10);
        assert!((book.value(&marks("A", 100.0)).unwrap().equity - 9998.5).abs() < 1e-8);
    }

    #[test]
    fn insolvent_short_may_be_liquidated_but_not_increased() {
        let mut book = Ledger::new(1_000.0).unwrap();
        book.execute("A", -10, 100.0, 0.0, 0.0, 10, true, &marks("A", 100.0))
            .unwrap();
        assert!(book
            .execute("A", -1, 250.0, 0.0, 0.0, 1, true, &marks("A", 250.0))
            .is_err());
        book.execute("A", 10, 250.0, 0.0, 0.0, 10, false, &marks("A", 250.0))
            .unwrap();
        assert!(book.positions().is_empty());
        assert_eq!(book.cash(), -500.0);
    }

    #[test]
    fn dividend_entitlement_survives_sale_but_is_not_cash_until_payment() {
        let mut book = Ledger::new(1_000.0).unwrap();
        book.execute("A", 5, 100.0, 0.0, 0.0, 5, false, &marks("A", 100.0))
            .unwrap();
        assert_eq!(book.accrue_dividend("div1", "A", 2.0).unwrap(), 10.0);
        assert_eq!(book.cash(), 500.0);
        assert_eq!(book.value(&marks("A", 98.0)).unwrap().equity, 1_000.0);
        book.execute("A", -5, 98.0, 0.0, 0.0, 5, false, &marks("A", 98.0))
            .unwrap();
        assert_eq!(book.cash(), 990.0);
        assert_eq!(book.settle_dividend("div1").unwrap(), 10.0);
        assert_eq!(book.cash(), 1_000.0);
        assert!(book.settle_dividend("div1").is_err());
    }

    #[test]
    fn delisting_settlement_closes_long_and_short_positions_at_assumed_return() {
        let mut book = Ledger::new(1_000.0).unwrap();
        book.execute("LONG", 2, 100.0, 0.0, 0.0, 2, false, &marks("LONG", 100.0))
            .unwrap();
        assert!((book.settle_delisting("LONG", 90.0, -0.30).unwrap() - 126.0).abs() < 1e-10);
        assert!(!book.positions().contains_key("LONG"));
        assert_eq!(book.cash(), 926.0);

        let mut short = Ledger::new(1_000.0).unwrap();
        short
            .execute(
                "SHORT",
                -2,
                100.0,
                0.0,
                0.0,
                2,
                true,
                &marks("SHORT", 100.0),
            )
            .unwrap();
        assert!((short.settle_delisting("SHORT", 90.0, -0.30).unwrap() + 126.0).abs() < 1e-10);
        assert!(!short.positions().contains_key("SHORT"));
        assert_eq!(short.cash(), 1074.0);
    }

    #[test]
    fn unpaid_dividend_cannot_fund_new_shares() {
        let mut book = Ledger::new(1_000.0).unwrap();
        book.execute("A", 5, 100.0, 0.0, 0.0, 5, false, &marks("A", 100.0))
            .unwrap();
        book.accrue_dividend("div1", "A", 20.0).unwrap();
        assert!(book
            .execute("A", 6, 100.0, 0.0, 0.0, 6, false, &marks("A", 100.0))
            .is_err());
        assert_eq!(book.cash(), 500.0);
    }
}
