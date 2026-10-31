//! Daily-bar loading: Yahoo Finance (the Rust equivalent of the yfinance pull) with a CSV cache,
//! total-return adjusted, aligned onto a (dates x symbols) panel.

use chrono::{DateTime, NaiveDate};
use std::path::{Path, PathBuf};

use crate::timing;

#[derive(Debug, Clone)]
pub struct Bars {
    pub dates: Vec<NaiveDate>,
    pub open: Vec<f64>,
    pub high: Vec<f64>,
    pub low: Vec<f64>,
    pub close: Vec<f64>,
    pub volume: Vec<f64>,
}

/// Date-by-symbol panel; every matrix is indexed `[symbol][row]` with NaN for missing sessions.
#[derive(Debug, Clone)]
pub struct Panel {
    pub dates: Vec<NaiveDate>,
    pub symbols: Vec<String>,
    pub open: Vec<Vec<f64>>,
    pub high: Vec<Vec<f64>>,
    pub low: Vec<Vec<f64>>,
    pub close: Vec<Vec<f64>>,
}

pub fn cache_path(dir: &str, symbol: &str, suffix: &str) -> PathBuf {
    Path::new(dir).join(format!("{symbol}{suffix}"))
}

pub fn parse_date(s: &str) -> Result<NaiveDate, String> {
    NaiveDate::parse_from_str(s, "%Y-%m-%d").map_err(|e| format!("bad date {s:?}: {e}"))
}

/// Parse a Yahoo v8 chart JSON response into total-return adjusted bars.
pub fn parse_yahoo(json: &serde_json::Value) -> Result<Bars, String> {
    let res = json.pointer("/chart/result/0").ok_or_else(|| {
        format!(
            "yahoo error: {}",
            json.pointer("/chart/error")
                .map(|e| e.to_string())
                .unwrap_or_default()
        )
    })?;
    let offset = res
        .pointer("/meta/gmtoffset")
        .and_then(|v| v.as_i64())
        .unwrap_or(0);
    let ts = res
        .get("timestamp")
        .and_then(|v| v.as_array())
        .ok_or("yahoo response has no timestamps")?;
    let q = |k: &str| -> Result<&Vec<serde_json::Value>, String> {
        res.pointer(&format!("/indicators/quote/0/{k}"))
            .and_then(|v| v.as_array())
            .ok_or(format!("yahoo response missing {k}"))
    };
    let (o, h, l, c, v) = (q("open")?, q("high")?, q("low")?, q("close")?, q("volume")?);
    let adj = res
        .pointer("/indicators/adjclose/0/adjclose")
        .and_then(|v| v.as_array());
    let mut b = Bars {
        dates: vec![],
        open: vec![],
        high: vec![],
        low: vec![],
        close: vec![],
        volume: vec![],
    };
    for i in 0..ts.len() {
        let f = |a: &Vec<serde_json::Value>| a.get(i).and_then(|x| x.as_f64());
        let (Some(ts_i), Some(oi), Some(hi), Some(li), Some(ci)) =
            (ts[i].as_i64(), f(o), f(h), f(l), f(c))
        else {
            continue;
        };
        let factor = adj
            .and_then(|a| a.get(i))
            .and_then(|x| x.as_f64())
            .map(|a| a / ci)
            .unwrap_or(1.0);
        let date = DateTime::from_timestamp(ts_i + offset, 0)
            .ok_or("bad timestamp")?
            .date_naive();
        if b.dates.last() == Some(&date) {
            continue;
        }
        b.dates.push(date);
        b.open.push(oi * factor);
        b.high.push(hi * factor);
        b.low.push(li * factor);
        b.close.push(ci * factor);
        b.volume.push(f(v).unwrap_or(0.0));
    }
    Ok(b)
}

/// Download daily bars for one symbol from Yahoo Finance.
pub fn fetch_yahoo(symbol: &str, start: NaiveDate) -> Result<Bars, String> {
    let p1 = start.and_hms_opt(0, 0, 0).unwrap().and_utc().timestamp();
    let p2 = chrono::Utc::now().timestamp() + 86_400;
    let url = format!(
        "https://query1.finance.yahoo.com/v8/finance/chart/{symbol}?period1={p1}&period2={p2}&interval=1d&events=div%2Csplit&includeAdjustedClose=true"
    );
    let agent = ureq::AgentBuilder::new()
        .timeout(std::time::Duration::from_secs(30))
        .build();
    let mut last = String::new();
    for attempt in 0..3 {
        match agent
            .get(&url)
            .set("User-Agent", "Mozilla/5.0 (deepthrust)")
            .call()
        {
            Ok(r) => {
                let json: serde_json::Value = r
                    .into_json()
                    .map_err(|e| format!("{symbol}: bad json: {e}"))?;
                return parse_yahoo(&json).map_err(|e| format!("{symbol}: {e}"));
            }
            Err(e) => {
                last = e.to_string();
                std::thread::sleep(std::time::Duration::from_millis(500 * (attempt + 1)));
            }
        }
    }
    Err(format!("{symbol}: yahoo download failed: {last}"))
}

pub fn write_csv(path: &Path, b: &Bars) -> std::io::Result<()> {
    if let Some(p) = path.parent() {
        std::fs::create_dir_all(p)?;
    }
    let mut s = String::from("date,open,high,low,close,volume\n");
    for i in 0..b.dates.len() {
        s += &format!(
            "{},{},{},{},{},{}\n",
            b.dates[i], b.open[i], b.high[i], b.low[i], b.close[i], b.volume[i]
        );
    }
    std::fs::write(path, s)
}

pub fn read_csv(path: &Path) -> Result<Bars, String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let mut b = Bars {
        dates: vec![],
        open: vec![],
        high: vec![],
        low: vec![],
        close: vec![],
        volume: vec![],
    };
    for (n, line) in text.lines().enumerate().skip(1) {
        let f: Vec<&str> = line.split(',').collect();
        if f.len() < 6 {
            continue;
        }
        let num = |i: usize| {
            f[i].parse::<f64>()
                .map_err(|e| format!("{}:{}: {e}", path.display(), n + 1))
        };
        b.dates.push(parse_date(f[0])?);
        b.open.push(num(1)?);
        b.high.push(num(2)?);
        b.low.push(num(3)?);
        b.close.push(num(4)?);
        b.volume.push(num(5)?);
    }
    Ok(b)
}

/// Load one symbol: reuse the CSV cache unless `rebuild`, otherwise download and refresh the cache.
pub fn load_symbol(
    dir: &str,
    suffix: &str,
    symbol: &str,
    start: NaiveDate,
    rebuild: bool,
) -> Result<Bars, String> {
    let path = cache_path(dir, symbol, suffix);
    if !rebuild && path.exists() {
        let _g = timing::stage(&format!("read cache {symbol}"));
        return read_csv(&path);
    }
    let bars = {
        let _g = timing::stage(&format!("yahoo download {symbol}"));
        fetch_yahoo(symbol, start)?
    };
    let _g = timing::stage(&format!("write cache {symbol}"));
    write_csv(&path, &bars).map_err(|e| format!("{}: {e}", path.display()))?;
    Ok(bars)
}

/// Align per-symbol bars on the union of their dates.
pub fn build_panel(symbols: &[String], bars: &[Bars]) -> Panel {
    let mut dates: Vec<NaiveDate> = bars.iter().flat_map(|b| b.dates.iter().copied()).collect();
    dates.sort();
    dates.dedup();
    let n = dates.len();
    let mut p = Panel {
        dates,
        symbols: symbols.to_vec(),
        open: vec![vec![f64::NAN; n]; symbols.len()],
        high: vec![vec![f64::NAN; n]; symbols.len()],
        low: vec![vec![f64::NAN; n]; symbols.len()],
        close: vec![vec![f64::NAN; n]; symbols.len()],
    };
    for (j, b) in bars.iter().enumerate() {
        for (i, d) in b.dates.iter().enumerate() {
            if let Ok(r) = p.dates.binary_search(d) {
                p.open[j][r] = b.open[i];
                p.high[j][r] = b.high[i];
                p.low[j][r] = b.low[i];
                p.close[j][r] = b.close[i];
            }
        }
    }
    p
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_yahoo_and_adjusts_to_total_return() {
        let j = serde_json::json!({"chart": {"result": [{
            "meta": {"gmtoffset": -18000},
            "timestamp": [1704205800, 1704292200, 1704378600],
            "indicators": {
                "quote": [{"open": [10.0, 11.0, null], "high": [11.0, 12.0, 1.0], "low": [9.0, 10.0, 1.0],
                           "close": [10.5, 11.5, null], "volume": [100.0, 200.0, 300.0]}],
                "adjclose": [{"adjclose": [5.25, 11.5, null]}]}}]}});
        let b = parse_yahoo(&j).unwrap();
        assert_eq!(b.dates.len(), 2);
        assert_eq!(b.dates[0].to_string(), "2024-01-02");
        assert!((b.close[0] - 5.25).abs() < 1e-12 && (b.open[0] - 5.0).abs() < 1e-12);
        assert!((b.close[1] - 11.5).abs() < 1e-12);
    }

    #[test]
    fn csv_round_trip_and_panel_alignment() {
        let d = |s: &str| parse_date(s).unwrap();
        let a = Bars {
            dates: vec![d("2024-01-02"), d("2024-01-03")],
            open: vec![1.0, 2.0],
            high: vec![1.5, 2.5],
            low: vec![0.5, 1.5],
            close: vec![1.2, 2.2],
            volume: vec![1.0, 2.0],
        };
        let b = Bars {
            dates: vec![d("2024-01-03")],
            open: vec![3.0],
            high: vec![3.0],
            low: vec![3.0],
            close: vec![3.0],
            volume: vec![1.0],
        };
        let path = std::env::temp_dir().join("deepthrust_csv_test.csv");
        write_csv(&path, &a).unwrap();
        let r = read_csv(&path).unwrap();
        assert_eq!(r.dates, a.dates);
        assert_eq!(r.close, a.close);
        let p = build_panel(&["A".into(), "B".into()], &[a, b]);
        assert_eq!(p.dates.len(), 2);
        assert!(p.close[1][0].is_nan());
        assert_eq!(p.close[1][1], 3.0);
        let _ = std::fs::remove_file(path);
    }
}
