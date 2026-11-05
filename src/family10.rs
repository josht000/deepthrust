//! Family 10 candlestick and bar pattern studies.

use std::collections::BTreeMap;
use std::path::Path;

use chrono::NaiveDate;

use crate::config::Config;
use crate::data::{self, Panel};
use crate::family2::blank_signals;
use crate::family4::{benchmark_returns, rebalance_days};
use crate::signals::Signals;

pub const S103_NAME: &str = "s103_yield_curve_rotation";
pub const S103_TITLE: &str = "S103 Yield-Curve Financials/Utilities Rotation";

fn s103_symbol_index(panel: &Panel, name: &str) -> Result<usize, String> {
    panel
        .symbols
        .iter()
        .position(|symbol| symbol == name)
        .ok_or_else(|| format!("{name} is required in the S103 universe"))
}

/// FRED fredgraph.csv: `observation_date,<SERIES>` with "." for missing days.
fn read_fred_series(path: &Path) -> Result<BTreeMap<NaiveDate, f64>, String> {
    let mut reader = csv::Reader::from_path(path)
        .map_err(|error| format!("{}: {error}", path.display()))?;
    let mut series = BTreeMap::new();
    for (row, result) in reader.records().enumerate() {
        let record =
            result.map_err(|error| format!("{} row {}: {error}", path.display(), row + 2))?;
        let date = data::parse_date(record.get(0).unwrap_or_default().trim())
            .map_err(|error| format!("{} row {}: {error}", path.display(), row + 2))?;
        let value = record.get(1).unwrap_or_default().trim();
        if value.is_empty() || value == "." {
            continue;
        }
        let value = value.parse::<f64>().map_err(|error| {
            format!("{} row {}: invalid value {value:?}: {error}", path.display(), row + 2)
        })?;
        series.insert(date, value);
    }
    if series.is_empty() {
        return Err(format!("{}: no observations", path.display()));
    }
    Ok(series)
}

/// Last-observation-carried-forward alignment of a daily FRED series to the panel
/// calendar (covers bond-market holidays on stock trading days).
fn align_fred(series: &BTreeMap<NaiveDate, f64>, dates: &[NaiveDate]) -> Vec<f64> {
    let mut aligned = vec![f64::NAN; dates.len()];
    let mut last = f64::NAN;
    let mut iter = series.iter().peekable();
    for (day, date) in dates.iter().enumerate() {
        while let Some(&(&obs_date, &value)) = iter.peek() {
            if obs_date <= *date {
                last = value;
                iter.next();
            } else {
                break;
            }
        }
        aligned[day] = last;
    }
    aligned
}

fn banded(change: f64, band: f64) -> f64 {
    if !change.is_finite() || change.abs() <= band {
        0.0
    } else {
        change.signum()
    }
}

// S103: rotate financial_symbol (XLF) vs utility_symbol (XLU) on yield-curve moves.
// `separate`: banded 10Y and 2Y changes vote separately (slope vote + 10Y-level
// vote); `slope`: banded change of the 10Y-2Y spread; `level`: T10Y2Y level vs
// threshold. XLF when the curve signal favours steepening beyond the dead band,
// else XLU (`band_mode: binary`), or hold the previous leg inside the band
// (`band_mode: hold`).
pub fn compute_s103_signals(panel: &Panel, cfg: &Config) -> Result<Signals, String> {
    if !matches!(cfg.signal_mode.as_str(), "separate" | "slope" | "level") {
        return Err(format!(
            "S103 signal_mode must be separate|slope|level, got {}",
            cfg.signal_mode
        ));
    }
    if !matches!(cfg.band_mode.as_str(), "binary" | "hold") {
        return Err(format!(
            "S103 band_mode must be binary|hold, got {}",
            cfg.band_mode
        ));
    }
    if cfg.fred_dir.trim().is_empty() {
        return Err("S103 requires fred_dir with DGS10.csv, DGS2.csv, T10Y2Y.csv".into());
    }
    if cfg.rate_change_window == 0 {
        return Err("S103 rate_change_window must be > 0".into());
    }
    let fin = s103_symbol_index(panel, &cfg.financial_symbol)?;
    let util = s103_symbol_index(panel, &cfg.utility_symbol)?;
    let root = Path::new(&cfg.fred_dir);
    let dgs10 = align_fred(&read_fred_series(&root.join("DGS10.csv"))?, &panel.dates);
    let dgs2 = align_fred(&read_fred_series(&root.join("DGS2.csv"))?, &panel.dates);
    let t10y2y = if cfg.signal_mode == "level" {
        align_fred(&read_fred_series(&root.join("T10Y2Y.csv"))?, &panel.dates)
    } else {
        Vec::new()
    };
    let n = panel.dates.len();
    let w = cfg.rate_change_window;
    let band = cfg.rate_dead_band;
    let binary = cfg.band_mode == "binary";
    let mut result = blank_signals(panel, "yield_curve_vote");
    let mut weights = vec![vec![0.0; n]; panel.symbols.len()];
    let mut hold_fin = false;
    for &day in &rebalance_days(panel, &cfg.rebalance)? {
        result.tradable[fin][day] = panel.close[fin][day].is_finite();
        result.tradable[util][day] = panel.close[util][day].is_finite();
        let (d10, d2) = if day >= w {
            (dgs10[day] - dgs10[day - w], dgs2[day] - dgs2[day - w])
        } else {
            (f64::NAN, f64::NAN)
        };
        let vote = match cfg.signal_mode.as_str() {
            "slope" => banded(d10 - d2, band),
            "level" => {
                if t10y2y[day].is_finite() {
                    banded(t10y2y[day] - cfg.level_threshold, band)
                } else {
                    0.0
                }
            }
            _ => banded(d10 - d2, band) + banded(d10, band),
        };
        result.rsi[fin][day] = if cfg.signal_mode == "level" {
            t10y2y[day]
        } else {
            d10 - d2
        };
        let target = if vote > 0.0 {
            fin
        } else if vote < 0.0 || binary || !hold_fin {
            util
        } else {
            fin
        };
        hold_fin = target == fin;
        weights[target][day] = 1.0;
        result.signal[target][day] = true;
    }
    result.trade_weights = Some(weights);
    result.benchmark_returns = benchmark_returns(panel, "SPY");
    Ok(result)
}
