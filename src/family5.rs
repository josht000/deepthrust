//! Family 5 stock cross-section and earnings-event studies.

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::Path;

use chrono::{Datelike, Duration, NaiveDate};

use crate::config::{split_csv, Config};
use crate::data::{self, Panel};
use crate::engine::{last_valid_rows, Trade};
use crate::family2::{
    blank_signals, canonical_symbol, read_announcements, read_quarterly_eps, surprises,
    trailing_sue_threshold, Announcement, ReleaseTiming,
};
use crate::family4::rebalance_days;
use crate::signals::{rolling_mean, simfin_groups, wilder_atr, Signals};

pub const S052_NAME: &str = "s052_earnings_premium";
pub const S052_TITLE: &str = "S052 Earnings Announcement Premium (Monthly)";
pub const S053_NAME: &str = "s053_pre_earnings_runup";
pub const S053_TITLE: &str = "S053 Pre-Earnings Run-Up";
pub const S054_NAME: &str = "s054_pead_surprise";
pub const S054_TITLE: &str = "S054 PEAD Earnings-Surprise Drift";
pub const S067_NAME: &str = "s067_sector_breadth_rotation";
pub const S067_TITLE: &str = "S067 Relative Sector Breadth Rotation";
pub const S078_NAME: &str = "s078_statarb_reversal";
pub const S078_TITLE: &str = "S078 Chan-Style Statistical Arbitrage Basket";

pub(crate) struct PublishRecord {
    pub date: NaiveDate,
    pub shares_basic: f64,
}

/// SimFin quarterly file: Ticker, Publish Date, Shares (Basic) per row.
pub(crate) fn read_publish_records(
    path: &str,
) -> Result<HashMap<String, Vec<PublishRecord>>, String> {
    let path = Path::new(path);
    let mut reader = csv::ReaderBuilder::new()
        .delimiter(b';')
        .from_path(path)
        .map_err(|error| format!("{}: {error}", path.display()))?;
    let headers = reader
        .headers()
        .map_err(|error| format!("{}: {error}", path.display()))?
        .clone();
    let find = |names: &[&str]| {
        headers
            .iter()
            .position(|header| names.iter().any(|name| header.eq_ignore_ascii_case(name)))
    };
    let ticker_col = find(&["ticker", "symbol"])
        .ok_or_else(|| format!("{}: missing ticker column", path.display()))?;
    let date_col = find(&["publish date", "publish_date"])
        .ok_or_else(|| format!("{}: missing publish date column", path.display()))?;
    let shares_col = find(&["shares (basic)", "shares_basic"]);
    let mut records = HashMap::<String, Vec<PublishRecord>>::new();
    for (row, result) in reader.records().enumerate() {
        let record =
            result.map_err(|error| format!("{} row {}: {error}", path.display(), row + 2))?;
        let ticker = record.get(ticker_col).unwrap_or_default().trim();
        if ticker.is_empty() {
            continue;
        }
        let date = record.get(date_col).unwrap_or_default().trim();
        let Ok(parsed) = data::parse_date(date) else { continue };
        let shares = shares_col
            .and_then(|col| record.get(col))
            .and_then(|value| value.trim().replace(',', "").parse::<f64>().ok())
            .unwrap_or(f64::NAN);
        records
            .entry(canonical_symbol(ticker))
            .or_default()
            .push(PublishRecord {
                date: parsed,
                shares_basic: shares,
            });
    }
    for values in records.values_mut() {
        values.sort_by_key(|record| record.date);
    }
    Ok(records)
}

/// Monthly volume share falling in months that contained a publish date,
/// over `lookback` calendar months ending `lag` months before `asof_day`.
fn volume_concentration(
    panel: &Panel,
    j: usize,
    asof_day: usize,
    announce_months: &BTreeSet<(i32, u32)>,
    lookback_months: i64,
    lag_months: i64,
) -> f64 {
    if lookback_months <= 0 {
        return f64::NAN;
    }
    let start = panel.dates[asof_day] - Duration::days(31 * (lag_months + lookback_months) + 15);
    let cutoff = panel.dates[asof_day] - Duration::days(31 * lag_months);
    let (mut ann_vol, mut total_vol) = (0.0, 0.0);
    for u in 0..=asof_day {
        let date = panel.dates[u];
        if date < start {
            continue;
        }
        if date > cutoff {
            break;
        }
        let vol = panel.volume[j][u];
        if !vol.is_finite() {
            continue;
        }
        total_vol += vol;
        if announce_months.contains(&(date.year(), date.month())) {
            ann_vol += vol;
        }
    }
    if total_vol <= 0.0 {
        return f64::NAN;
    }
    ann_vol / total_vol
}

/// Upper-case ticker set with at least one SimFin publish record.
pub fn publish_symbols(path: &str) -> Result<BTreeSet<String>, String> {
    let publish = read_publish_records(path)?;
    Ok(publish.keys().cloned().collect())
}

fn announce_months_before(dates: &[NaiveDate], cutoff: NaiveDate) -> BTreeSet<(i32, u32)> {
    dates
        .iter()
        .filter(|&&date| date < cutoff)
        .map(|date| (date.year(), date.month()))
        .collect()
}

fn count_reports_in_window(dates: &[NaiveDate], end: NaiveDate, days: i64) -> usize {
    dates
        .iter()
        .filter(|&&date| date < end && date >= end - Duration::days(days))
        .count()
}

pub(crate) fn load_ranks(
    panel: &Panel,
    cfg: &Config,
    columns: &[usize],
) -> Result<Vec<Vec<f64>>, String> {
    if cfg.rank_cache_dir.trim().is_empty() {
        data::liquidity_ranks(
            panel,
            columns,
            cfg.rank_window,
            cfg.rank_min_observations,
            cfg.min_price,
            cfg.min_history,
        )
    } else {
        data::read_liquid_rank_cache(&cfg.rank_cache_dir, &panel.dates, &panel.symbols)
    }
}

fn ew_stock_benchmark(panel: &Panel, tradable: &[Vec<bool>], etf_set: &BTreeSet<usize>) -> Option<Vec<f64>> {
    let n = panel.dates.len();
    let mut out = vec![0.0; n];
    for t in 1..n {
        let (mut sum, mut count) = (0.0, 0usize);
        for j in 0..panel.symbols.len() {
            if etf_set.contains(&j) {
                continue;
            }
            let (a, b) = (panel.close[j][t - 1], panel.close[j][t]);
            if tradable[j][t - 1] && a.is_finite() && a > 0.0 && b.is_finite() {
                sum += b / a - 1.0;
                count += 1;
            }
        }
        out[t] = if count > 0 { sum / count as f64 } else { 0.0 };
    }
    Some(out)
}

// ---------------- S052 earnings announcement premium ----------------

pub fn compute_s052_signals(panel: &Panel, cfg: &Config) -> Result<Signals, String> {
    if !matches!(cfg.leg_mode.as_str(), "long_only" | "long_spy_hedge" | "long_short") {
        return Err(format!("unknown S052 leg_mode {:?}", cfg.leg_mode));
    }
    if !matches!(cfg.weighting_mode.as_str(), "equal" | "value") {
        return Err(format!("unknown S052 weighting_mode {:?}", cfg.weighting_mode));
    }
    let publish = read_publish_records(&cfg.earnings_dates_file)?;
    let n = panel.dates.len();
    let spy_idx = panel.symbols.iter().position(|s| s == "SPY");
    let mut etf_set = BTreeSet::new();
    if let Some(j) = spy_idx {
        etf_set.insert(j);
    }
    let stock_cols: Vec<usize> = (0..panel.symbols.len())
        .filter(|j| !etf_set.contains(j))
        .collect();
    let ranks = load_ranks(panel, cfg, &stock_cols)?;
    let mut result = blank_signals(panel, "volume_concentration");
    for &j in &stock_cols {
        for t in 0..n {
            result.tradable[j][t] = panel.close[j][t].is_finite()
                && panel.close[j][t] >= cfg.min_price
                && ranks[j][t].is_finite()
                && ranks[j][t] <= cfg.universe_top_n as f64;
            result.pool[j][t] = result.tradable[j][t];
        }
    }
    let mut weights = vec![vec![0.0; n]; panel.symbols.len()];
    for &day in &rebalance_days(panel, &cfg.rebalance)? {
        let next_month = if day + 1 < n {
            (panel.dates[day + 1].year(), panel.dates[day + 1].month())
        } else {
            continue;
        };
        let cutoff = panel.dates[day];
        let mut scored: Vec<(usize, f64)> = Vec::new();
        for &j in &stock_cols {
            if !result.tradable[j][day] {
                continue;
            }
            let Some(records) = publish.get(&canonical_symbol(&panel.symbols[j])) else {
                continue;
            };
            let dates: Vec<NaiveDate> = records.iter().map(|r| r.date).collect();
            if count_reports_in_window(&dates, cutoff, 365) as i64 != cfg.required_reports {
                continue;
            }
            let months = announce_months_before(&dates, cutoff);
            let expected = months.iter().any(|&(_, month)| month == next_month.1);
            if expected {
                let vc = volume_concentration(
                    panel,
                    j,
                    day,
                    &months,
                    cfg.volume_lookback_months,
                    cfg.volume_lag_months,
                );
                if vc.is_finite() {
                    scored.push((j, vc));
                }
            }
        }
        scored.sort_by(|a, b| b.1.total_cmp(&a.1).then(panel.symbols[a.0].cmp(&panel.symbols[b.0])));
        let n_long = ((scored.len() as f64) * cfg.volume_quintile).ceil() as usize;
        let longs = &scored[..n_long.min(scored.len())];
        if longs.is_empty() {
            continue;
        }
        let weight_of = |j: usize| -> f64 {
            if cfg.weighting_mode == "value" {
                let shares = publish
                    .get(&canonical_symbol(&panel.symbols[j]))
                    .and_then(|records| {
                        records
                            .iter()
                            .rev()
                            .find(|record| record.date <= cutoff)
                    })
                    .map(|record| record.shares_basic)
                    .unwrap_or(f64::NAN);
                if shares.is_finite() && shares > 0.0 {
                    shares * panel.close[j][day]
                } else {
                    f64::NAN
                }
            } else {
                1.0
            }
        };
        let raw: Vec<f64> = longs.iter().map(|&(j, _)| weight_of(j)).collect();
        if raw.iter().any(|w| !w.is_finite() || *w <= 0.0) {
            let valid: Vec<f64> = raw.iter().map(|w| if w.is_finite() && *w > 0.0 { *w } else { 0.0 }).collect();
            let total: f64 = valid.iter().sum();
            if total <= 0.0 {
                continue;
            }
            for (k, &(j, vc)) in longs.iter().enumerate() {
                let w = valid[k] / total;
                if w > 0.0 {
                    weights[j][day] = w;
                    result.signal[j][day] = true;
                    result.rsi[j][day] = vc;
                }
            }
        } else {
            let total: f64 = raw.iter().sum();
            for (k, &(j, vc)) in longs.iter().enumerate() {
                weights[j][day] = raw[k] / total;
                result.signal[j][day] = true;
                result.rsi[j][day] = vc;
            }
        }
        match cfg.leg_mode.as_str() {
            "long_spy_hedge" => {
                if let Some(spy) = spy_idx {
                    weights[spy][day] = -1.0;
                }
            }
            "long_short" => {
                let non_announcers: Vec<usize> = stock_cols
                    .iter()
                    .copied()
                    .filter(|&j| result.tradable[j][day] && !longs.iter().any(|&(lj, _)| lj == j))
                    .collect();
                let n_short = non_announcers.len().max(1) as f64;
                for &j in &non_announcers {
                    weights[j][day] = -1.0 / n_short;
                }
            }
            _ => {}
        }
    }
    result.trade_weights = Some(weights);
    result.benchmark_returns = ew_stock_benchmark(panel, &result.tradable, &etf_set);
    Ok(result)
}

/// Monthly/weekly weight book with per-symbol last-row clamping and delisting
/// adjustment (family4::weighted_rebalance_trades plus delisting handling).
pub(crate) fn book_trades(
    panel: &Panel,
    sig: &Signals,
    cfg: &Config,
    fill: &str,
) -> Result<Vec<Trade>, String> {
    if !matches!(fill, "close" | "next_open") {
        return Err(format!("{} does not support fill {fill:?}", cfg.strategy));
    }
    let weights = sig
        .trade_weights
        .as_ref()
        .ok_or_else(|| format!("{} requires target weights", cfg.strategy))?;
    let days = rebalance_days(panel, &cfg.rebalance)?;
    let is_open = fill == "next_open";
    let last = last_valid_rows(&panel.close);
    let n = panel.dates.len();
    let mut trades = Vec::new();
    for (period, &signal_day) in days.iter().enumerate() {
        let entry = signal_day + usize::from(is_open);
        if entry >= n {
            continue;
        }
        let boundary = days.get(period + 1).copied().unwrap_or(n - 1);
        for symbol in 0..panel.symbols.len() {
            let weight = weights[symbol][signal_day];
            if !weight.is_finite() || weight == 0.0 {
                continue;
            }
            let Some(last_row) = last[symbol] else { continue };
            let delisted = last_row < boundary;
            let exit = if is_open {
                boundary.saturating_add(1).min(last_row)
            } else {
                boundary.min(last_row)
            };
            if exit < entry {
                continue;
            }
            let entry_px = if is_open {
                panel.open[symbol][entry]
            } else {
                panel.close[symbol][signal_day]
            };
            let mut exit_px = if is_open && !delisted && panel.open[symbol][exit].is_finite() {
                panel.open[symbol][exit]
            } else {
                panel.close[symbol][exit]
            };
            if delisted {
                exit_px *= 1.0 + cfg.delisting_return;
            }
            if !entry_px.is_finite() || entry_px <= 0.0 || !exit_px.is_finite() {
                continue;
            }
            let prior_weight = period
                .checked_sub(1)
                .map(|previous| weights[symbol][days[previous]])
                .unwrap_or(0.0);
            let next_weight = days
                .get(period + 1)
                .filter(|&&next_day| next_day + 1 < n)
                .map(|&next_day| weights[symbol][next_day])
                .unwrap_or(0.0);
            let entry_turnover = (weight - prior_weight).abs();
            let exit_turnover = if next_weight == 0.0 { weight.abs() } else { 0.0 };
            let direction = weight.signum();
            let gross = direction * (exit_px / entry_px - 1.0);
            let cost = cfg.cost_for_symbol(&panel.symbols[symbol]);
            let borrow = if weight < 0.0 && exit > entry {
                cfg.borrow_bps_annual / 10_000.0 / cfg.periods_per_year as f64
                    * (exit - entry + 1) as f64
            } else {
                0.0
            };
            trades.push(Trade {
                sym_idx: symbol,
                signal_idx: signal_day,
                entry_idx: entry,
                exit_idx: exit,
                entry_px,
                exit_px,
                ret: gross
                    - cost * (entry_turnover + exit_turnover) / weight.abs()
                    - borrow,
                ret_gross: gross,
                exit_reason: if delisted {
                    "delisted"
                } else if period + 1 == days.len() {
                    "censored"
                } else {
                    "rebalance"
                },
                hold: exit.saturating_sub(entry),
                signal_value: sig.rsi[symbol][signal_day],
                weight,
                entry_turnover,
                exit_turnover,
                weighted: true,
                borrow_cost_per_bar: if weight < 0.0 && exit > entry {
                    cfg.borrow_bps_annual / 10_000.0 / cfg.periods_per_year as f64
                } else {
                    0.0
                },
            });
        }
    }
    trades.sort_by(|a, b| {
        (a.entry_idx, &panel.symbols[a.sym_idx]).cmp(&(b.entry_idx, &panel.symbols[b.sym_idx]))
    });
    Ok(trades)
}

pub fn s052_trades(panel: &Panel, sig: &Signals, cfg: &Config, fill: &str) -> Result<Vec<Trade>, String> {
    book_trades(panel, sig, cfg, fill)
}

// ---------------- S053 pre-earnings run-up ----------------

struct RunUpEvent {
    symbol: usize,
    entry_day: usize,
    exit_day: usize, // session after the realized publish date (open fill)
    vc: f64,
}

fn expected_events(
    panel: &Panel,
    publish: &HashMap<String, Vec<NaiveDate>>,
    cfg: &Config,
) -> Vec<RunUpEvent> {
    let n = panel.dates.len();
    let mut events = Vec::new();
    let session_of = |date: NaiveDate| panel.dates.partition_point(|&d| d < date);
    for (j, symbol) in panel.symbols.iter().enumerate() {
        let Some(dates) = publish.get(&canonical_symbol(symbol)) else {
            continue;
        };
        let mut expected: Vec<NaiveDate> = dates
            .iter()
            .map(|d| *d + Duration::days(cfg.expected_shift_days))
            .collect();
        expected.sort_unstable();
        expected.dedup();
        let mut last_event = NaiveDate::default();
        for e in expected {
            if last_event != NaiveDate::default() && e - last_event < Duration::days(45) {
                continue;
            }
            last_event = e;
            let e_session = session_of(e);
            if e_session == 0 || e_session < cfg.entry_sessions_before + 1 || e_session >= n {
                continue;
            }
            let entry_day = e_session - cfg.entry_sessions_before;
            // realized announcement: first publish date within (e-15d, e+45d]
            let realized = dates
                .iter()
                .copied()
                .find(|d| *d > e - Duration::days(15) && *d <= e + Duration::days(45));
            let exit_day = realized
                .map(|d| (session_of(d) + 1).min(n - 1))
                .unwrap_or_else(|| (entry_day + cfg.max_hold).min(n - 1));
            if exit_day <= entry_day {
                continue;
            }
            events.push(RunUpEvent {
                symbol: j,
                entry_day,
                exit_day,
                vc: f64::NAN,
            });
        }
    }
    events
}

pub fn compute_s053_signals(panel: &Panel, cfg: &Config) -> Result<Signals, String> {
    let publish_records = read_publish_records(&cfg.earnings_dates_file)?;
    let publish: HashMap<String, Vec<NaiveDate>> = publish_records
        .iter()
        .map(|(symbol, records)| {
            (
                symbol.clone(),
                records.iter().map(|record| record.date).collect(),
            )
        })
        .collect();
    let n = panel.dates.len();
    let stock_cols: Vec<usize> = (0..panel.symbols.len()).collect();
    let ranks = load_ranks(panel, cfg, &stock_cols)?;
    let mut result = blank_signals(panel, "volume_concentration");
    for j in 0..panel.symbols.len() {
        for t in 0..n {
            result.tradable[j][t] = panel.close[j][t].is_finite()
                && panel.close[j][t] >= cfg.min_price
                && ranks[j][t].is_finite()
                && ranks[j][t] <= cfg.universe_top_n as f64;
            result.pool[j][t] = result.tradable[j][t];
        }
    }
    let mut events = expected_events(panel, &publish, cfg);
    // per-day cross-sectional volume-concentration tercile filter
    let mut by_day: BTreeMap<usize, Vec<usize>> = BTreeMap::new();
    for (i, event) in events.iter_mut().enumerate() {
        let j = event.symbol;
        if !result.tradable[j][event.entry_day] {
            continue;
        }
        let months = publish
            .get(&canonical_symbol(&panel.symbols[j]))
            .map(|dates| announce_months_before(dates, panel.dates[event.entry_day]))
            .unwrap_or_default();
        event.vc = volume_concentration(
            panel,
            j,
            event.entry_day,
            &months,
            cfg.volume_lookback_months,
            cfg.volume_lag_months,
        );
        if event.vc.is_finite() {
            by_day.entry(event.entry_day).or_default().push(i);
        }
    }
    for (_, idxs) in by_day {
        let mut sorted: Vec<f64> = idxs.iter().map(|&i| events[i].vc).collect();
        sorted.sort_by(f64::total_cmp);
        let cutoff = if cfg.signal_mode == "no_tercile" {
            f64::NEG_INFINITY
        } else {
            crate::family2::quantile(&mut sorted, 2.0 / 3.0)
        };
        for i in idxs {
            let event = &events[i];
            if event.vc >= cutoff {
                result.signal[event.symbol][event.entry_day] = true;
                result.rsi[event.symbol][event.entry_day] = event.vc;
                result.exit_now[event.symbol][event.exit_day] = true;
            }
        }
    }
    result.benchmark_returns = ew_stock_benchmark(panel, &result.tradable, &BTreeSet::new());
    Ok(result)
}

/// Entry at close/next_open of the signal day; exit at the open of the first
/// session after the realized announcement (captures the gap for both timings).
pub fn s053_trades(panel: &Panel, sig: &Signals, cfg: &Config, fill: &str) -> Result<Vec<Trade>, String> {
    if !matches!(fill, "close" | "next_open") {
        return Err(format!("S053 does not support fill {fill:?}"));
    }
    let n = panel.dates.len();
    let last = last_valid_rows(&panel.close);
    let mut trades = Vec::new();
    for j in 0..panel.symbols.len() {
        let Some(last_row) = last[j] else { continue };
        let cost = cfg.cost_for_symbol(&panel.symbols[j]);
        for t in 0..n {
            if !sig.signal[j][t] {
                continue;
            }
            let Some(u) = (t + 1..=last_row).find(|&u| sig.exit_now[j][u]) else {
                continue;
            };
            let (entry_idx, entry_px) = if fill == "next_open" {
                (t + 1, panel.open[j].get(t + 1).copied().unwrap_or(f64::NAN))
            } else {
                (t, panel.close[j][t])
            };
            let exit_px = if u <= last_row {
                panel.open[j][u]
            } else {
                f64::NAN
            };
            let delisted = u >= last_row && last_row + 1 < panel.dates.len();
            let exit_px = if delisted {
                panel.close[j][last_row] * (1.0 + cfg.delisting_return)
            } else {
                exit_px
            };
            let exit_idx = if delisted { last_row } else { u };
            if !entry_px.is_finite() || entry_px <= 0.0 || !exit_px.is_finite() || exit_idx <= entry_idx {
                continue;
            }
            trades.push(Trade {
                sym_idx: j,
                signal_idx: t,
                entry_idx,
                exit_idx,
                entry_px,
                exit_px,
                ret: exit_px * (1.0 - cost) / (entry_px * (1.0 + cost)) - 1.0,
                ret_gross: exit_px / entry_px - 1.0,
                exit_reason: if delisted {
                    "delisted"
                } else if u >= last_row {
                    "censored"
                } else {
                    "announcement"
                },
                hold: exit_idx - entry_idx,
                signal_value: sig.rsi[j][t],
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

// ---------------- S054 PEAD ----------------

struct PeadEvent {
    symbol: usize,
    reaction_day: usize,
    sue: f64,
    ear: f64,
    next_reaction: usize, // session of the symbol's next reaction (for early exit)
}

pub fn compute_s054_signals(panel: &Panel, cfg: &Config) -> Result<Signals, String> {
    if !matches!(cfg.surprise_mode.as_str(), "sue" | "ear" | "both") {
        return Err(format!("unknown S054 surprise_mode {:?}", cfg.surprise_mode));
    }
    if !matches!(cfg.signal_mode.as_str(), "edgar" | "amc") {
        return Err(format!("unknown S054 signal_mode {:?}", cfg.signal_mode));
    }
    let n = panel.dates.len();
    let quarters = read_quarterly_eps(Path::new(&cfg.s023_eps_file))?;
    let announcements: Vec<Announcement> = if cfg.signal_mode == "edgar" {
        if cfg.edgar_filings_file.trim().is_empty() {
            return Err("S054 edgar mode requires edgar_filings_file (8-K Item 2.02)".into());
        }
        read_announcements(Path::new(&cfg.edgar_filings_file))?
    } else {
        quarters
            .iter()
            .map(|q| Announcement {
                symbol: q.symbol.clone(),
                date: q.publish_date,
                timing: ReleaseTiming::AfterMarket,
            })
            .collect()
    };
    let surprise_vec = surprises(&quarters);
    let mut sue_by_key: HashMap<(String, NaiveDate), f64> = HashMap::new();
    for item in &surprise_vec {
        sue_by_key.insert((item.symbol.clone(), item.publish_date), item.sue);
    }
    let spy_idx = panel.symbols.iter().position(|s| s == "SPY");
    let session_of = |date: NaiveDate| panel.dates.partition_point(|&d| d < date);
    let mut events: Vec<PeadEvent> = Vec::new();
    let mut ann_dates: HashMap<usize, Vec<NaiveDate>> = HashMap::new();
    for a in &announcements {
        let symbol_idx = panel
            .symbols
            .iter()
            .position(|s| canonical_symbol(s) == a.symbol);
        let Some(j) = symbol_idx else { continue };
        ann_dates.entry(j).or_default().push(a.date);
    }
    for dates in ann_dates.values_mut() {
        dates.sort_unstable();
    }
    for a in &announcements {
        let Some(j) = panel
            .symbols
            .iter()
            .position(|s| canonical_symbol(s) == a.symbol)
        else {
            continue;
        };
        let announce_day = session_of(a.date);
        if announce_day >= n || announce_day == 0 {
            continue;
        }
        let reaction = match a.timing {
            ReleaseTiming::BeforeMarket => announce_day,
            ReleaseTiming::AfterMarket => announce_day + 1,
        };
        if reaction + 2 >= n {
            continue;
        }
        let sue = sue_by_key
            .get(&(a.symbol.clone(), a.date))
            .copied()
            .unwrap_or(f64::NAN);
        let ear = {
            let (c0, c2) = (panel.close[j][reaction - 1], panel.close[j][reaction + 1]);
            let spy = spy_idx.map(|spy| {
                panel.close[spy][reaction + 1] / panel.close[spy][reaction - 1] - 1.0
            });
            if c0.is_finite() && c0 > 0.0 && c2.is_finite() {
                c2 / c0 - 1.0 - spy.unwrap_or(0.0)
            } else {
                f64::NAN
            }
        };
        let next_reaction = ann_dates
            .get(&j)
            .and_then(|dates| dates.iter().copied().find(|&d| d > a.date))
            .map(|d| session_of(d) + 1)
            .unwrap_or(n);
        events.push(PeadEvent {
            symbol: j,
            reaction_day: reaction,
            sue,
            ear,
            next_reaction,
        });
    }
    let mut result = blank_signals(panel, "sue");
    for j in 0..panel.symbols.len() {
        for t in 0..n {
            result.tradable[j][t] = panel.close[j][t].is_finite()
                && panel.close[j][t] >= cfg.min_price;
            result.pool[j][t] = result.tradable[j][t];
        }
    }
    for event in &events {
        let (j, r) = (event.symbol, event.reaction_day);
        if !result.tradable[j][r] {
            continue;
        }
        let sue_pass = event.sue.is_finite()
            && trailing_sue_threshold(&surprise_vec, panel.dates[r])
                .is_some_and(|threshold| event.sue > threshold);
        let ear_pass = event.ear.is_finite() && event.ear > 0.0;
        let pass = match cfg.surprise_mode.as_str() {
            "ear" => ear_pass,
            "both" => sue_pass && ear_pass,
            _ => sue_pass,
        };
        if pass {
            result.signal[j][r] = true;
            result.rsi[j][r] = if cfg.surprise_mode == "ear" { event.ear } else { event.sue };
            let exit_day = (r + 1 + cfg.max_hold - 1).min(event.next_reaction.saturating_sub(1)).min(n - 1);
            if exit_day > r {
                result.exit_now[j][exit_day] = true;
            }
        }
    }
    result.benchmark_returns = ew_stock_benchmark(panel, &result.tradable, &BTreeSet::new());
    Ok(result)
}

/// Enter at the open of the first session after the reaction day; exit at the
/// earlier of `max_hold` sessions or the session before the next announcement.
pub fn s054_trades(panel: &Panel, sig: &Signals, cfg: &Config, fill: &str) -> Result<Vec<Trade>, String> {
    if !matches!(fill, "next_open" | "close") {
        return Err(format!("S054 does not support fill {fill:?}"));
    }
    let n = panel.dates.len();
    let last = last_valid_rows(&panel.close);
    let atr = panel
        .high
        .iter()
        .zip(&panel.low)
        .zip(&panel.close)
        .map(|((h, l), c)| wilder_atr(h, l, c, cfg.atr_len))
        .collect::<Vec<_>>();
    let mut trades = Vec::new();
    for j in 0..panel.symbols.len() {
        let Some(last_row) = last[j] else { continue };
        let cost = cfg.cost_for_symbol(&panel.symbols[j]);
        let mut busy_until = 0usize;
        for r in 0..n {
            if !sig.signal[j][r] || r <= busy_until {
                continue;
            }
            if cfg.gap_atr_multiple > 0.0 && r > 0 {
                let gap = (panel.open[j][r] / panel.close[j][r - 1] - 1.0).abs();
                if !atr[j][r].is_finite()
                    || !panel.close[j][r - 1].is_finite()
                    || gap >= cfg.gap_atr_multiple * atr[j][r] / panel.close[j][r - 1]
                {
                    continue;
                }
            }
            let (entry_idx, entry_px) = if fill == "next_open" {
                (r + 1, panel.open[j].get(r + 1).copied().unwrap_or(f64::NAN))
            } else {
                (r, panel.close[j][r])
            };
            let Some(u) = (r + 1..=last_row).find(|&u| sig.exit_now[j][u]) else {
                continue;
            };
            let exit_idx = u.min(last_row);
            let delisted = u >= last_row && last_row + 1 < panel.dates.len();
            let exit_px = if delisted {
                panel.close[j][last_row] * (1.0 + cfg.delisting_return)
            } else {
                panel.close[j][u]
            };
            if !entry_px.is_finite()
                || entry_px <= 0.0
                || !exit_px.is_finite()
                || exit_idx <= entry_idx
            {
                continue;
            }
            trades.push(Trade {
                sym_idx: j,
                signal_idx: r,
                entry_idx,
                exit_idx,
                entry_px,
                exit_px,
                ret: exit_px * (1.0 - cost) / (entry_px * (1.0 + cost)) - 1.0,
                ret_gross: exit_px / entry_px - 1.0,
                exit_reason: if delisted {
                    "delisted"
                } else if u < r + cfg.max_hold {
                    "next_event"
                } else {
                    "time"
                },
                hold: exit_idx - entry_idx,
                signal_value: sig.rsi[j][r],
                weight: 1.0,
                entry_turnover: 1.0,
                exit_turnover: 1.0,
                weighted: false,
                borrow_cost_per_bar: 0.0,
            });
            busy_until = exit_idx;
        }
    }
    trades.sort_by(|a, b| {
        (a.entry_idx, &panel.symbols[a.sym_idx]).cmp(&(b.entry_idx, &panel.symbols[b.sym_idx]))
    });
    Ok(trades)
}

// ---------------- S067 relative sector breadth rotation ----------------

pub fn compute_s067_signals(panel: &Panel, cfg: &Config) -> Result<Signals, String> {
    let n = panel.dates.len();
    // sector ETF columns come from the sector_etf_map values
    let map: Vec<(String, String)> = split_csv(&cfg.sector_etf_map)
        .into_iter()
        .filter_map(|entry| {
            let mut parts = entry.split(':');
            Some((parts.next()?.to_string(), parts.next()?.to_string()))
        })
        .collect();
    if map.is_empty() {
        return Err("S067 requires sector_etf_map like 'Technology:XLK,...'".into());
    }
    let etf_of: BTreeMap<String, usize> = map
        .iter()
        .filter_map(|(sector, ticker)| {
            panel
                .symbols
                .iter()
                .position(|s| s == ticker)
                .map(|idx| (sector.clone(), idx))
        })
        .collect();
    let etf_set: BTreeSet<usize> = etf_of.values().copied().collect();
    let stock_cols: Vec<usize> = (0..panel.symbols.len())
        .filter(|j| !etf_set.contains(j))
        .collect();
    let ranks = load_ranks(panel, cfg, &stock_cols)?;
    let (_, sectors) = simfin_groups(&panel.symbols, &cfg.simfin_dir)?;
    let mut result = blank_signals(panel, "sector_breadth_z");
    for j in 0..panel.symbols.len() {
        let is_etf = etf_set.contains(&j);
        for t in 0..n {
            result.tradable[j][t] = if is_etf {
                panel.close[j][t].is_finite()
            } else {
                false
            };
            result.pool[j][t] = result.tradable[j][t];
        }
    }
    // per-sector member lists restricted to mapped sectors
    let mut members: BTreeMap<String, Vec<usize>> = BTreeMap::new();
    for (j, sector) in sectors.iter().enumerate() {
        if let Some(sector) = sector {
            if etf_of.contains_key(sector) && stock_cols.contains(&j) {
                members.entry(sector.clone()).or_default().push(j);
            }
        }
    }
    let sma: Vec<Vec<f64>> = (0..panel.symbols.len())
        .map(|j| rolling_mean(&panel.close[j], cfg.sma_window))
        .collect();
    let mut weights = vec![vec![0.0; n]; panel.symbols.len()];
    for &day in &rebalance_days(panel, &cfg.rebalance)? {
        let mut breadth: Vec<(String, f64)> = Vec::new();
        for (sector, list) in &members {
            let (mut above, mut total) = (0usize, 0usize);
            for &j in list {
                let liquid = ranks[j][day].is_finite() && ranks[j][day] <= cfg.universe_top_n as f64;
                if liquid && panel.close[j][day].is_finite() && sma[j][day].is_finite() {
                    total += 1;
                    if panel.close[j][day] > sma[j][day] {
                        above += 1;
                    }
                }
            }
            if total >= cfg.min_peers {
                breadth.push((sector.clone(), above as f64 / total as f64));
            }
        }
        if breadth.len() < 3 {
            continue;
        }
        let mean = breadth.iter().map(|(_, b)| b).sum::<f64>() / breadth.len() as f64;
        let sd = (breadth
            .iter()
            .map(|(_, b)| (b - mean).powi(2))
            .sum::<f64>()
            / (breadth.len() - 1) as f64)
            .sqrt();
        if sd <= 0.0 {
            continue;
        }
        let mut z_sum = 0.0;
        let mut z_of: Vec<(String, f64)> = Vec::new();
        for (sector, b) in &breadth {
            let z = (b - mean) / sd;
            if z > 0.0 {
                z_sum += z;
            }
            z_of.push((sector.clone(), z));
        }
        for (sector, z) in &z_of {
            let Some(&etf) = etf_of.get(sector) else { continue };
            result.rsi[etf][day] = *z;
            if *z > 0.0 && z_sum > 0.0 {
                weights[etf][day] = z / z_sum;
                result.signal[etf][day] = true;
            }
        }
    }
    result.trade_weights = Some(weights);
    result.benchmark_returns = ew_stock_benchmark(panel, &result.tradable, &etf_set);
    Ok(result)
}

// ---------------- S078 stat-arb daily reversal / OU ----------------

pub fn compute_s078_signals(panel: &Panel, cfg: &Config) -> Result<Signals, String> {
    if !matches!(cfg.signal_mode.as_str(), "reversal" | "ou") {
        return Err(format!("unknown S078 signal_mode {:?}", cfg.signal_mode));
    }
    if cfg.rebalance != "daily" {
        return Err("S078 requires rebalance: daily".into());
    }
    let n = panel.dates.len();
    let stock_cols: Vec<usize> = (0..panel.symbols.len()).collect();
    let ranks = load_ranks(panel, cfg, &stock_cols)?;
    let mut result = blank_signals(panel, "reversal_score");
    // EW market return for beta estimation
    let mut mkt = vec![f64::NAN; n];
    for t in 1..n {
        let (mut sum, mut count) = (0.0, 0usize);
        for j in 0..panel.symbols.len() {
            let (a, b) = (panel.close[j][t - 1], panel.close[j][t]);
            if a.is_finite() && a > 0.0 && b.is_finite() {
                sum += b / a - 1.0;
                count += 1;
            }
        }
        if count > 0 {
            mkt[t] = sum / count as f64;
        }
    }
    for j in 0..panel.symbols.len() {
        for t in 0..n {
            result.tradable[j][t] = panel.close[j][t].is_finite()
                && panel.close[j][t] >= cfg.min_price
                && ranks[j][t].is_finite()
                && ranks[j][t] <= cfg.universe_top_n as f64;
            result.pool[j][t] = result.tradable[j][t];
        }
    }
    let mut weights = vec![vec![0.0; n]; panel.symbols.len()];
    for &day in &rebalance_days(panel, &cfg.rebalance)? {
        let eligible: Vec<usize> = (0..panel.symbols.len())
            .filter(|&j| result.tradable[j][day] && day >= 1)
            .collect();
        if cfg.signal_mode == "reversal" {
            let mut scored: Vec<(usize, f64)> = eligible
                .iter()
                .copied()
                .filter(|&j| {
                    panel.close[j][day].is_finite()
                        && panel.close[j][day - 1].is_finite()
                        && panel.close[j][day - 1] > 0.0
                })
                .map(|j| (j, -(panel.close[j][day] / panel.close[j][day - 1] - 1.0)))
                .collect();
            scored.sort_by(|a, b| b.1.total_cmp(&a.1).then(panel.symbols[a.0].cmp(&panel.symbols[b.0])));
            let k = ((scored.len() as f64) * cfg.top_fraction).max(1.0) as usize;
            let longs = &scored[..k.min(scored.len())];
            let shorts = &scored[scored.len().saturating_sub(k)..];
            if longs.is_empty() || shorts.is_empty() {
                continue;
            }
            let (mut beta_l, mut beta_s) = (1.0, 1.0);
            if cfg.beta_neutral {
                let beta_of = |j: usize| -> f64 {
                    if day + 1 < cfg.beta_window + 1 {
                        return f64::NAN;
                    }
                    let start = day + 1 - cfg.beta_window;
                    let mut xs = Vec::new();
                    let mut ys = Vec::new();
                    for u in start..=day {
                        if panel.close[j][u].is_finite()
                            && panel.close[j][u - 1].is_finite()
                            && panel.close[j][u - 1] > 0.0
                            && mkt[u].is_finite()
                        {
                            xs.push(mkt[u]);
                            ys.push(panel.close[j][u] / panel.close[j][u - 1] - 1.0);
                        }
                    }
                    if ys.len() < cfg.beta_min_obs {
                        return f64::NAN;
                    }
                    let mx = xs.iter().sum::<f64>() / xs.len() as f64;
                    let my = ys.iter().sum::<f64>() / ys.len() as f64;
                    let cov: f64 = xs.iter().zip(&ys).map(|(x, y)| (x - mx) * (y - my)).sum();
                    let var: f64 = xs.iter().map(|x| (x - mx).powi(2)).sum();
                    if var <= 0.0 { f64::NAN } else { cov / var }
                };
                let l: Vec<f64> = longs.iter().map(|&(j, _)| beta_of(j)).collect();
                let s: Vec<f64> = shorts.iter().map(|&(j, _)| beta_of(j)).collect();
                if l.iter().all(|b| b.is_finite()) && s.iter().all(|b| b.is_finite()) {
                    beta_l = l.iter().sum::<f64>() / l.len() as f64;
                    beta_s = s.iter().sum::<f64>() / s.len() as f64;
                }
            }
            let scale = if beta_l.abs() > 1e-9 { beta_s / beta_l } else { 1.0 };
            for &(j, score) in longs {
                weights[j][day] = scale / longs.len() as f64;
                result.rsi[j][day] = score;
                result.signal[j][day] = true;
            }
            for &(j, score) in shorts {
                weights[j][day] = -1.0 / shorts.len() as f64;
                result.rsi[j][day] = score;
                result.signal[j][day] = true;
            }
        } else {
            // OU stage: residual half-life vs EW market, fastest names weighted by 1/half-life
            let mut candidates: Vec<(usize, f64, f64)> = Vec::new(); // (j, half_life, resid_t)
            for &j in &eligible {
                if day + 1 < cfg.formation_window + 1 {
                    continue;
                }
                let start = day + 1 - cfg.formation_window;
                let mut resid = Vec::new();
                let mut cum_mkt = 0.0;
                for u in start..=day {
                    if !panel.close[j][u].is_finite() || !mkt[u].is_finite() {
                        break;
                    }
                    cum_mkt += mkt[u];
                    resid.push(panel.close[j][u].ln() - cum_mkt);
                }
                if resid.len() < cfg.formation_window {
                    continue;
                }
                // AR(1): resid_t = a + phi resid_{t-1}
                let (mut sx, mut sy, mut sxx, mut sxy) = (0.0, 0.0, 0.0, 0.0);
                let m = resid.len() - 1;
                for i in 0..m {
                    sx += resid[i];
                    sy += resid[i + 1];
                    sxx += resid[i] * resid[i];
                    sxy += resid[i] * resid[i + 1];
                }
                let sx2n = m as f64;
                let denom = sx2n * sxx - sx * sx;
                if denom.abs() < 1e-12 {
                    continue;
                }
                let phi = (sx2n * sxy - sx * sy) / denom;
                if phi <= 0.0 || phi >= 1.0 {
                    continue;
                }
                let half_life = -std::f64::consts::LN_2 / phi.ln();
                if !(0.25..60.0).contains(&half_life) {
                    continue;
                }
                candidates.push((j, half_life, resid[m]));
            }
            candidates.sort_by(|a, b| a.1.total_cmp(&b.1));
            let take = candidates.len().min(50);
            let inv_sum: f64 = candidates[..take].iter().map(|(_, h, _)| 1.0 / h).sum();
            if inv_sum <= 0.0 {
                continue;
            }
            for &(j, h, resid_t) in &candidates[..take] {
                let w = (1.0 / h) / inv_sum;
                let dir = if resid_t < 0.0 { 1.0 } else { -1.0 };
                weights[j][day] = dir * w;
                result.rsi[j][day] = h;
                result.signal[j][day] = true;
            }
        }
    }
    result.trade_weights = Some(weights);
    result.benchmark_returns = ew_stock_benchmark(panel, &result.tradable, &BTreeSet::new());
    Ok(result)
}
