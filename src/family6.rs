//! Family 6 news-sentiment studies: S056 daily sentiment book, S058 topic-conditioned.

use std::collections::HashMap;
use std::fs::File;
use std::path::Path;

use chrono::NaiveDate;
use parquet::file::reader::{FileReader, SerializedFileReader};
use parquet::record::Field;

use crate::config::Config;
use crate::data::{self, Panel};
use crate::engine::Trade;
use crate::family2::{blank_signals, field_text};
use crate::signals::Signals;

pub const S056_NAME: &str = "s056_news_sentiment";
pub const S056_TITLE: &str = "S056 LLM Daily News Sentiment";
pub const S058_NAME: &str = "s058_topic_sentiment";
pub const S058_TITLE: &str = "S058 News-Topic Conditioned Sentiment";

pub(crate) struct SentimentRow {
    pub symbol: String,
    pub date: NaiveDate,
    pub hour: f64,
    pub score: f64,
    pub lm_score: f64,
    pub topic: String,
}

fn field_f64(field: Option<&Field>) -> f64 {
    match field {
        Some(Field::Double(v)) => *v,
        Some(Field::Float(v)) => *v as f64,
        Some(Field::Int(v)) => *v as f64,
        Some(Field::Long(v)) => *v as f64,
        _ => f64::NAN,
    }
}

/// sentiment.parquet written by tools/score_news_sentiment.py:
/// symbol, date (ET yyyy-mm-dd), hour (ET hour float), score, lm_score, topic.
pub(crate) fn read_sentiment(path: &Path) -> Result<Vec<SentimentRow>, String> {
    let file = File::open(path).map_err(|error| format!("{}: {error}", path.display()))?;
    let reader = SerializedFileReader::new(file)
        .map_err(|error| format!("{}: invalid Parquet file: {error}", path.display()))?;
    let rows = reader
        .get_row_iter(None)
        .map_err(|error| format!("{}: cannot read Parquet rows: {error}", path.display()))?;
    let mut out = Vec::new();
    for (row_index, row) in rows.enumerate() {
        let row = row.map_err(|error| format!("{} row {}: {error}", path.display(), row_index + 1))?;
        let columns: HashMap<String, &Field> = row
            .get_column_iter()
            .map(|(name, value)| (name.to_ascii_lowercase(), value))
            .collect();
        let get = |name: &str| columns.get(name).copied();
        let symbol = field_text(get("symbol"), "symbol", path, row_index + 1)?;
        if symbol.trim().is_empty() {
            continue;
        }
        let date_str = field_text(get("date"), "date", path, row_index + 1)?;
        let Ok(date) = data::parse_date(date_str.trim()) else {
            continue;
        };
        let score = field_f64(get("score"));
        if !score.is_finite() {
            continue;
        }
        out.push(SentimentRow {
            symbol: crate::family2::canonical_symbol(symbol),
            date,
            hour: field_f64(get("hour")),
            score,
            lm_score: field_f64(get("lm_score")),
            topic: field_text(get("topic"), "topic", path, row_index + 1)?
                .to_string(),
        });
    }
    Ok(out)
}

fn topic_kept(mode: &str, topic: &str) -> bool {
    match mode {
        "firm_specific" => !matches!(topic, "ratings" | "macro" | "other"),
        "no_rating_repeat" => topic != "ratings",
        _ => true,
    }
}

/// Session assignment: an item with hour < `cutoff` belongs to the first session
/// on/after its ET date; otherwise to the first session strictly after it.
fn session_of(panel: &Panel, date: NaiveDate, hour: f64, cutoff: f64) -> Option<usize> {
    if hour < cutoff {
        let idx = panel.dates.partition_point(|&d| d < date);
        (idx < panel.dates.len()).then_some(idx)
    } else {
        let idx = panel.dates.partition_point(|&d| d <= date);
        (idx < panel.dates.len()).then_some(idx)
    }
}

/// Daily per-symbol sentiment book; quintile tails at each session.
fn compute_news_signals(
    panel: &Panel,
    cfg: &Config,
    default_topic: &str,
) -> Result<Signals, String> {
    if cfg.sentiment_file.trim().is_empty() {
        return Err(format!(
            "{} requires sentiment_file (tools/score_news_sentiment.py output)",
            cfg.strategy
        ));
    }
    if !matches!(cfg.score_mode.as_str(), "finbert" | "lm") {
        return Err(format!("unknown score_mode {:?}", cfg.score_mode));
    }
    if !matches!(cfg.leg_mode.as_str(), "long" | "long_short") {
        return Err(format!("unknown leg_mode {:?}", cfg.leg_mode));
    }
    if !matches!(
        cfg.signal_mode.as_str(),
        "overnight" | "intraday_close" | "full_day"
    ) {
        return Err(format!("unknown signal_mode {:?}", cfg.signal_mode));
    }
    let topic_mode = if cfg.topic_mode.is_empty() {
        default_topic
    } else {
        cfg.topic_mode.as_str()
    };
    let rows = read_sentiment(Path::new(&cfg.sentiment_file))?;
    let n = panel.dates.len();
    let symbol_idx: HashMap<&str, usize> = panel
        .symbols
        .iter()
        .enumerate()
        .map(|(j, s)| (crate::family2::canonical_symbol(s).leak() as &str, j))
        .collect();
    // per (j, session): (sum, count)
    let mut sums: HashMap<(usize, usize), (f64, usize)> = HashMap::new();
    let cutoff = if cfg.signal_mode == "full_day" { 24.0 } else { 16.0 };
    for row in &rows {
        if !topic_kept(topic_mode, &row.topic) {
            continue;
        }
        if cfg.signal_mode == "intraday_close" && !(9.5..16.0).contains(&row.hour) {
            continue;
        }
        let Some(&j) = symbol_idx.get(row.symbol.as_str()) else {
            continue;
        };
        let Some(t) = session_of(panel, row.date, row.hour, cutoff) else {
            continue;
        };
        let value = if cfg.score_mode == "lm" { row.lm_score } else { row.score };
        if !value.is_finite() {
            continue;
        }
        let entry = sums.entry((j, t)).or_insert((0.0, 0));
        entry.0 += value;
        entry.1 += 1;
    }
    let columns: Vec<usize> = (0..panel.symbols.len()).collect();
    let ranks = if cfg.rank_cache_dir.trim().is_empty() {
        data::liquidity_ranks(
            panel,
            &columns,
            cfg.rank_window,
            cfg.rank_min_observations,
            cfg.min_price,
            cfg.min_history,
        )?
    } else {
        data::read_liquid_rank_cache(&cfg.rank_cache_dir, &panel.dates, &panel.symbols)?
    };
    let mut result = blank_signals(panel, "news_sentiment");
    let mut weights = vec![vec![0.0; n]; panel.symbols.len()];
    let mut entries = vec![vec![f64::NAN; n]; panel.symbols.len()];
    let mut exits = vec![vec![f64::NAN; n]; panel.symbols.len()];
    for j in 0..panel.symbols.len() {
        for t in 0..n {
            result.tradable[j][t] = panel.close[j][t].is_finite()
                && panel.close[j][t] >= cfg.min_price
                && ranks[j][t].is_finite()
                && ranks[j][t] <= cfg.universe_top_n as f64;
            result.pool[j][t] = result.tradable[j][t];
        }
    }
    let mut daily: Vec<Vec<(usize, f64)>> = vec![Vec::new(); n];
    for (&(j, t), &(sum, count)) in &sums {
        if count > 0 && t < n {
            result.rsi[j][t] = sum / count as f64;
            daily[t].push((j, result.rsi[j][t]));
        }
    }
    for t in 1..n {
        let eligible: Vec<(usize, f64)> = daily[t]
            .iter()
            .copied()
            .filter(|&(j, _)| result.tradable[j][t] && t + 1 < n)
            .collect();
        let k = ((eligible.len() as f64) * cfg.top_fraction).max(1.0) as usize;
        let mut sorted = eligible;
        sorted.sort_by(|a, b| b.1.total_cmp(&a.1).then(panel.symbols[a.0].cmp(&panel.symbols[b.0])));
        let longs = &sorted[..k.min(sorted.len())];
        let shorts = if cfg.leg_mode == "long_short" {
            &sorted[sorted.len().saturating_sub(k)..]
        } else {
            &[][..]
        };
        for &(j, _) in longs {
            if !panel.close[j][t].is_finite() {
                continue;
            }
            result.signal[j][t] = true;
            weights[j][t + 1] = 1.0 / longs.len() as f64;
            entries[j][t + 1] = panel.close[j][t];
            exits[j][t + 1] = if cfg.signal_mode == "intraday_close" {
                panel.close[j][t + 1]
            } else {
                panel.open[j][t + 1]
            };
        }
        for &(j, _) in shorts {
            if !panel.close[j][t].is_finite() {
                continue;
            }
            result.signal[j][t] = true;
            weights[j][t + 1] = -1.0 / shorts.len().max(1) as f64;
            entries[j][t + 1] = panel.close[j][t];
            exits[j][t + 1] = if cfg.signal_mode == "intraday_close" {
                panel.close[j][t + 1]
            } else {
                panel.open[j][t + 1]
            };
        }
    }
    result.trade_weights = Some(weights);
    result.entry_mat = Some(entries);
    result.exit_mat = Some(exits);
    Ok(result)
}

pub fn compute_s056_signals(panel: &Panel, cfg: &Config) -> Result<Signals, String> {
    compute_news_signals(panel, cfg, "all")
}

pub fn compute_s058_signals(panel: &Panel, cfg: &Config) -> Result<Signals, String> {
    compute_news_signals(panel, cfg, "firm_specific")
}

/// Overnight-weighted signed book: entry close[t] -> open[t+1] (or close for
/// intraday_close), with borrow charged on negative weights.
pub fn news_trades(panel: &Panel, sig: &Signals, cfg: &Config, fill: &str) -> Result<Vec<Trade>, String> {
    if fill != "close_to_open" {
        return Err(format!("{} supports close_to_open fill only, got {fill:?}", cfg.strategy));
    }
    let weights = sig
        .trade_weights
        .as_ref()
        .ok_or_else(|| format!("{} requires trade weights", cfg.strategy))?;
    let entries = sig
        .entry_mat
        .as_ref()
        .ok_or_else(|| format!("{} requires entry matrix", cfg.strategy))?;
    let exits = sig
        .exit_mat
        .as_ref()
        .ok_or_else(|| format!("{} requires exit matrix", cfg.strategy))?;
    let n = panel.dates.len();
    let borrow_per_bar = cfg.borrow_bps_annual / 10_000.0 / cfg.periods_per_year as f64;
    let mut trades = Vec::new();
    for j in 0..panel.symbols.len() {
        let cost = cfg.cost_for_symbol(&panel.symbols[j]);
        for day in 1..n {
            let w = weights[j][day];
            if w.abs() < 1e-9 || !w.is_finite() {
                continue;
            }
            let (entry_px, exit_px) = (entries[j][day], exits[j][day]);
            if !entry_px.is_finite() || entry_px <= 0.0 || !exit_px.is_finite() {
                continue;
            }
            let direction = w.signum();
            let gross = direction * (exit_px / entry_px - 1.0);
            let borrow = if w < 0.0 { borrow_per_bar } else { 0.0 };
            trades.push(Trade {
                sym_idx: j,
                signal_idx: day - 1,
                entry_idx: day - 1,
                exit_idx: day,
                entry_px,
                exit_px,
                ret: gross - cost * 2.0 * w.abs() - borrow,
                ret_gross: gross,
                exit_reason: "next_open",
                hold: 1,
                signal_value: sig.rsi[j][day - 1],
                weight: w,
                entry_turnover: w.abs(),
                exit_turnover: w.abs(),
                weighted: true,
                borrow_cost_per_bar: if w < 0.0 { borrow_per_bar } else { 0.0 },
                dir_sign: 1.0,
            });
        }
    }
    trades.sort_by(|a, b| {
        (a.entry_idx, &panel.symbols[a.sym_idx]).cmp(&(b.entry_idx, &panel.symbols[b.sym_idx]))
    });
    Ok(trades)
}
