//! The trade engine. Turns an entry instruction into a filled trade on
//! the 1-minute bar stream, applying the spec's fill rules:
//!   - a target (limit) needs a tick through the level; fills at limit
//!   - stops fill at stop +- 1 tick slippage, or at the bar's open when
//!     it gaps through (stop & gap slippage)
//!   - if a stop and target are both inside one bar, the stop hit first
//!   - market entries fill at the next bar's open (1 tick slippage)
//!   - limit entries fill on touch, at the limit (0 slippage, user rule)
//!   - time exits at the Nth trigger bar's close; roll exits at the last
//!     bar before a roll
//!   - one position at a time; optional half size
//! Costs (decision #8): $2.00/side commission; slippage only on
//! market/stop fills.

use crate::pal::types::{Bar, Market};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    Long,
    Short,
}

impl Direction {
    pub fn sign(self) -> f64 {
        match self {
            Direction::Long => 1.0,
            Direction::Short => -1.0,
        }
    }
    pub fn other(self) -> Direction {
        match self {
            Direction::Long => Direction::Short,
            Direction::Short => Direction::Long,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Entry {
    /// limit at `price`; fills when touched (low<=p<=high for longs)
    Limit(f64),
    /// market at the next bar's open after the signal
    Market,
    /// buy-stop at `price` (fills 1 tick through on touch, gap price on gap)
    StopEntry(f64),
}

#[derive(Clone, Copy, Debug)]
pub enum Target {
    /// fixed price target (already includes any reference math)
    Fixed(f64),
    /// R multiple of (entry - stop) risk, resolved at fill
    RMult(f64),
    /// no target: run to time/roll/end
    None,
}

#[derive(Clone, Copy, Debug)]
pub struct OrderSpec {
    pub dir: Direction,
    pub entry: Entry,
    /// hard stop price (already includes any ATR buffer)
    pub stop: f64,
    pub target: Target,
    /// session minutes to hold before time exit; 0 = none
    pub time_exit_minutes: u32,
    /// cancel an unfilled order after this many session minutes; 0=never
    pub cancel_after_minutes: u32,
    /// size multiplier (1.0 or 0.5)
    pub size: f64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExitReason {
    Target,
    Stop,
    Time,
    Roll,
    EndOfData,
    Cancelled,
}

#[derive(Clone, Debug)]
pub struct Trade {
    pub dir: Direction,
    pub entry_px: f64,
    pub exit_px: f64,
    pub entry_bar: usize,
    pub exit_bar: usize,
    pub entry_session: i32,
    pub exit_session: i32,
    pub reason: ExitReason,
    pub pnl_usd: f64,
    pub r_multiple: f64,
}

/// Context per 1-minute bar needed by the engine.
#[derive(Clone, Copy, Debug)]
pub struct BarCtx {
    /// index of the session (0-based within the run)
    pub session: i32,
    /// cumulative session minutes: session * 1380 + minute index.
    /// drives time exits and cancel windows in wall-clock terms.
    pub minute_global: u64,
    /// true when this bar is inside the 60-minute pre-roll no-entry zone
    pub pre_roll: bool,
    /// true when this is the last bar before the roll boundary: an open
    /// trade must close at this bar's close
    pub last_before_roll: bool,
}

pub struct CostModel {
    pub commission_per_side: f64,
    pub tick_size: f64,
    pub tick_value: f64,
}

impl CostModel {
    pub fn for_market(m: Market) -> CostModel {
        CostModel {
            commission_per_side: m.commission_per_side(),
            tick_size: m.tick_size(),
            tick_value: m.tick_value(),
        }
    }
}

/// Run one order over the bar stream starting at `signal_bar` (the bar
/// whose close produced the signal — entries act on LATER bars only).
/// Returns the filled trade, or a cancelled trade when the order never
/// fills before expiry/roll/end.
pub fn run_order(
    bars: &[Bar],
    ctx: &[BarCtx],
    signal_bar: usize,
    spec: &OrderSpec,
    costs: &CostModel,
) -> Trade {
    let tick = costs.tick_size;
    let dir = spec.dir.sign();
    let mut filled = false;
    let mut entry_px = 0.0;
    let mut entry_bar = 0usize;
    let mut target_px = f64::NAN;
    let live_minute = ctx[signal_bar].minute_global;

    let mut exit_px = f64::NAN;
    let mut exit_bar = signal_bar;
    let mut reason = ExitReason::EndOfData;

    for i in signal_bar + 1..bars.len() {
        let b = &bars[i];
        if !filled {
            // roll: unfilled orders die at the roll boundary
            if ctx[i].last_before_roll {
                return cancel(signal_bar, i, spec.dir);
            }
            if spec.cancel_after_minutes > 0
                && ctx[i].minute_global >= live_minute + spec.cancel_after_minutes as u64
            {
                return cancel(signal_bar, i, spec.dir);
            }
            // no new fills inside the pre-roll window
            if ctx[i].pre_roll {
                continue;
            }
            match spec.entry {
                Entry::Limit(p) => {
                    let touched = if spec.dir == Direction::Long {
                        b.low <= p
                    } else {
                        b.high >= p
                    };
                    if touched {
                        filled = true;
                        entry_px = p; // 0 slippage on limits (user rule)
                        entry_bar = i;
                    }
                }
                Entry::Market => {
                    filled = true;
                    entry_px = b.open + dir * tick; // 1 tick adverse
                    entry_bar = i;
                }
                Entry::StopEntry(p) => {
                    let through = if spec.dir == Direction::Long {
                        b.high >= p
                    } else {
                        b.low <= p
                    };
                    if through {
                        // gap through -> fill at open when open is worse
                        let gapped = if spec.dir == Direction::Long {
                            b.open > p
                        } else {
                            b.open < p
                        };
                        entry_px = if gapped { b.open } else { p + dir * tick };
                        entry_bar = i;
                        filled = true;
                    }
                }
            }
            if filled {
                // resolve R-multiple targets at fill
                target_px = match spec.target {
                    Target::Fixed(t) => t,
                    Target::RMult(r) => {
                        let risk = (entry_px - spec.stop).abs();
                        if risk > 0.0 {
                            entry_px + dir * r * risk
                        } else {
                            f64::NAN
                        }
                    }
                    Target::None => f64::NAN,
                };
                // conservative same-bar resolution: a fill inside this
                // bar can be stopped by its remaining range, but never
                // reaches target in the same bar
                let stop_hit = if spec.dir == Direction::Long {
                    b.low <= spec.stop
                } else {
                    b.high >= spec.stop
                };
                if stop_hit {
                    let gapped = if spec.dir == Direction::Long {
                        b.open < spec.stop
                    } else {
                        b.open > spec.stop
                    };
                    exit_px = if gapped { b.open } else { spec.stop - dir * tick };
                    exit_bar = i;
                    reason = ExitReason::Stop;
                    break;
                }
            }
        } else {
            // stop + target in same bar -> stop first (spec)
            let stop_hit = if spec.dir == Direction::Long {
                b.low <= spec.stop
            } else {
                b.high >= spec.stop
            };
            if stop_hit {
                let gapped = if spec.dir == Direction::Long {
                    b.open < spec.stop
                } else {
                    b.open > spec.stop
                };
                exit_px = if gapped {
                    b.open
                } else {
                    spec.stop - dir * tick
                };
                exit_bar = i;
                reason = ExitReason::Stop;
                break;
            }
            if target_px.is_finite() {
                // a target needs a tick through the level
                let target_hit = if spec.dir == Direction::Long {
                    b.high >= target_px + tick
                } else {
                    b.low <= target_px - tick
                };
                if target_hit {
                    exit_px = target_px; // limit fill, 0 slippage
                    exit_bar = i;
                    reason = ExitReason::Target;
                    break;
                }
            }
            if ctx[i].last_before_roll {
                exit_px = b.close;
                exit_bar = i;
                reason = ExitReason::Roll;
                break;
            }
            if spec.time_exit_minutes > 0
                && ctx[i].minute_global >= ctx[entry_bar].minute_global + spec.time_exit_minutes as u64
            {
                exit_px = b.close;
                exit_bar = i;
                reason = ExitReason::Time;
                break;
            }
        }
    }
    if !filled {
        return cancel(signal_bar, exit_bar, spec.dir);
    }
    if !exit_px.is_finite() {
        exit_px = bars[bars.len() - 1].close;
        exit_bar = bars.len() - 1;
        reason = ExitReason::EndOfData;
    }

    let gross = (exit_px - entry_px) * dir / tick * costs.tick_value * spec.size;
    let pnl = gross - 2.0 * costs.commission_per_side * spec.size;
    let risk = (entry_px - spec.stop).abs();
    let r_multiple = if risk > 0.0 {
        (exit_px - entry_px) * dir / risk
    } else {
        0.0
    };
    Trade {
        dir: spec.dir,
        entry_px,
        exit_px,
        entry_bar,
        exit_bar,
        entry_session: ctx[entry_bar].session,
        exit_session: ctx[exit_bar].session,
        reason,
        pnl_usd: pnl,
        r_multiple,
    }
}

fn cancel(signal_bar: usize, at: usize, dir: Direction) -> Trade {
    Trade {
        dir,
        entry_px: f64::NAN,
        exit_px: f64::NAN,
        entry_bar: signal_bar,
        exit_bar: at,
        entry_session: -1,
        exit_session: -1,
        reason: ExitReason::Cancelled,
        pnl_usd: 0.0,
        r_multiple: 0.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn b(o: f64, h: f64, l: f64, c: f64) -> Bar {
        Bar {
            ts_utc: chrono::DateTime::from_timestamp(0, 0).unwrap(),
            open: o,
            high: h,
            low: l,
            close: c,
            volume: 1,
        }
    }

    fn flat_ctx(n: usize) -> Vec<BarCtx> {
        (0..n)
            .map(|i| BarCtx {
                session: 0,
                minute_global: i as u64,
                pre_roll: false,
                last_before_roll: false,
            })
            .collect()
    }

    fn costs() -> CostModel {
        CostModel::for_market(Market::Es)
    }

    #[test]
    fn long_limit_fills_on_touch_at_limit() {
        let bars = vec![b(100., 100., 100., 100.), b(101., 101., 99., 100.)];
        let spec = OrderSpec {
            dir: Direction::Long,
            entry: Entry::Limit(99.5),
            stop: 98.0,
            target: Target::Fixed(102.0),
            time_exit_minutes: 0,
            cancel_after_minutes: 0,
            size: 1.0,
        };
        let t = run_order(&bars, &flat_ctx(bars.len()), 0, &spec, &costs());
        assert_eq!(t.entry_px, 99.5);
    }

    #[test]
    fn target_needs_tick_through() {
        let bars = vec![
            b(100., 100., 100., 100.),
            b(100., 102., 99., 100.), // touches 102.0 exactly: no fill (needs 102.25)
            b(100., 103., 100., 100.), // ticks through: fills at 102.0
        ];
        let spec = OrderSpec {
            dir: Direction::Long,
            entry: Entry::Limit(100.0),
            stop: 98.0,
            target: Target::Fixed(102.0),
            time_exit_minutes: 0,
            cancel_after_minutes: 0,
            size: 1.0,
        };
        let t = run_order(&bars, &flat_ctx(bars.len()), 0, &spec, &costs());
        assert_eq!(t.reason, ExitReason::Target);
        assert_eq!(t.exit_bar, 2);
        assert_eq!(t.exit_px, 102.0);
    }

    #[test]
    fn stop_first_when_both_in_bar() {
        // bar 2 contains both stop (97.5) and target (102.0) -> stop wins
        let bars = vec![
            b(100., 100., 100., 100.),
            b(100., 103., 97., 98.),
        ];
        let spec = OrderSpec {
            dir: Direction::Long,
            entry: Entry::Market,
            stop: 97.5,
            target: Target::Fixed(102.0),
            time_exit_minutes: 0,
            cancel_after_minutes: 0,
            size: 1.0,
        };
        let t = run_order(&bars, &flat_ctx(bars.len()), 0, &spec, &costs());
        assert_eq!(t.reason, ExitReason::Stop);
    }

    #[test]
    fn gap_through_stop_fills_at_open() {
        let bars = vec![
            b(100., 100., 100., 100.),
            b(100., 101., 100., 100.),
            b(96., 96., 95., 95.), // opens through 98.0 stop
        ];
        let spec = OrderSpec {
            dir: Direction::Long,
            entry: Entry::Market,
            stop: 98.0,
            target: Target::Fixed(110.0),
            time_exit_minutes: 0,
            cancel_after_minutes: 0,
            size: 1.0,
        };
        let t = run_order(&bars, &flat_ctx(bars.len()), 0, &spec, &costs());
        assert_eq!(t.reason, ExitReason::Stop);
        assert_eq!(t.exit_px, 96.0);
    }

    #[test]
    fn stop_slippage_one_tick() {
        let bars = vec![
            b(100., 100., 100., 100.),
            b(100., 100., 97., 98.), // trades through 98.0 without gapping
        ];
        let spec = OrderSpec {
            dir: Direction::Long,
            entry: Entry::Limit(100.0),
            stop: 98.0,
            target: Target::Fixed(110.0),
            time_exit_minutes: 0,
            cancel_after_minutes: 0,
            size: 1.0,
        };
        let t = run_order(&bars, &flat_ctx(bars.len()), 0, &spec, &costs());
        assert_eq!(t.exit_px, 97.75); // 98.0 - 1 tick
    }

    #[test]
    fn mirrored_short() {
        let bars = vec![
            b(100., 100., 100., 100.),
            b(99., 100., 98., 99.), // fills short limit at 99.5? no: high=100 >=99.5 yes
            b(99., 99., 97., 97.),  // trades through target 97.5-0.25=97.25? low 97 <=97.25: target
        ];
        let spec = OrderSpec {
            dir: Direction::Short,
            entry: Entry::Limit(99.5),
            stop: 101.0,
            target: Target::Fixed(97.5),
            time_exit_minutes: 0,
            cancel_after_minutes: 0,
            size: 1.0,
        };
        let t = run_order(&bars, &flat_ctx(bars.len()), 0, &spec, &costs());
        assert_eq!(t.entry_px, 99.5);
        assert_eq!(t.reason, ExitReason::Target);
        assert!(t.pnl_usd > 0.0);
    }

    #[test]
    fn roll_closes_at_last_bar() {
        let mut ctx = flat_ctx(3);
        ctx[2].last_before_roll = true;
        let bars = vec![
            b(100., 100., 100., 100.),
            b(100., 101., 99., 100.),
            b(101., 102., 100., 101.),
        ];
        let spec = OrderSpec {
            dir: Direction::Long,
            entry: Entry::Market,
            stop: 90.0,
            target: Target::Fixed(200.0),
            time_exit_minutes: 0,
            cancel_after_minutes: 0,
            size: 1.0,
        };
        let t = run_order(&bars, &ctx, 0, &spec, &costs());
        assert_eq!(t.reason, ExitReason::Roll);
        assert_eq!(t.exit_px, 101.0);
    }

    #[test]
    fn time_exit() {
        let bars = vec![
            b(100., 100., 100., 100.),
            b(100., 101., 99., 100.),
            b(100., 101., 99., 100.),
            b(100., 101., 99., 99.),
        ];
        let spec = OrderSpec {
            dir: Direction::Long,
            entry: Entry::Limit(100.0),
            stop: 90.0,
            target: Target::Fixed(200.0),
            time_exit_minutes: 2,
            cancel_after_minutes: 0,
            size: 1.0,
        };
        let t = run_order(&bars, &flat_ctx(bars.len()), 0, &spec, &costs());
        assert_eq!(t.reason, ExitReason::Time);
        assert_eq!(t.exit_bar, 3);
    }

    #[test]
    fn unfilled_limit_cancels() {
        let bars = vec![
            b(100., 100., 100., 100.),
            b(100., 101., 99., 100.),
            b(100., 101., 99., 100.),
        ];
        let spec = OrderSpec {
            dir: Direction::Long,
            entry: Entry::Limit(95.0), // never touched
            stop: 90.0,
            target: Target::Fixed(200.0),
            time_exit_minutes: 0,
            cancel_after_minutes: 0,
            size: 1.0,
        };
        let mut s2 = spec;
        s2.cancel_after_minutes = 1;
        // plain limit (no cancel) just runs to end unfilled
        let t = run_order(&bars, &flat_ctx(bars.len()), 0, &spec, &costs());
        assert_eq!(t.reason, ExitReason::Cancelled);
        let t2 = run_order(&bars, &flat_ctx(bars.len()), 0, &s2, &costs());
        assert_eq!(t2.reason, ExitReason::Cancelled);
    }

    #[test]
    fn pnl_includes_costs() {
        // long 1 ES: entry 100, exit 104 -> 16 ticks * $12.5 - $4 = $196
        let bars = vec![
            b(100., 100., 100., 100.),
            b(100., 104., 100., 104.),
            b(104., 105., 104., 104.),
        ];
        let spec = OrderSpec {
            dir: Direction::Long,
            entry: Entry::Limit(100.0),
            stop: 90.0,
            target: Target::Fixed(104.0),
            time_exit_minutes: 0,
            cancel_after_minutes: 0,
            size: 1.0,
        };
        let t = run_order(&bars, &flat_ctx(bars.len()), 0, &spec, &costs());
        assert!((t.pnl_usd - 196.0).abs() < 1e-9);
        // half size halves gross and commission
        let mut s2 = spec;
        s2.size = 0.5;
        let t2 = run_order(&bars, &flat_ctx(bars.len()), 0, &s2, &costs());
        assert!((t2.pnl_usd - 98.0).abs() < 1e-9);
    }
}
