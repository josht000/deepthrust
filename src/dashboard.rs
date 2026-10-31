//! Nine-panel dashboard (pure-Rust `plotters` SVG backend), laid out like the Python
//! `framework/dashboard.py` PNG: A cumulative log return, B key metrics, C partition
//! decomposition, D drawdown, E calendar years, F trade returns, G holding period,
//! H exit reasons, I open positions.

use chrono::{Datelike, NaiveDate};
use plotters::coord::Shift;
use plotters::prelude::*;
use plotters::style::text_anchor::{HPos, Pos, VPos};
use plotters::style::FontStyle;
use std::collections::BTreeSet;
use std::path::Path;

use crate::partition::Row;
use crate::stats;

type Area<'a> = DrawingArea<SVGBackend<'a>, Shift>;

const W: i32 = 2000;
const H: i32 = 2200;
const FILL_COLORS: [(u8, u8, u8); 4] = [
    (31, 119, 180),
    (214, 39, 40),
    (44, 160, 44),
    (148, 103, 189),
];
const BENCH_COLORS: [(u8, u8, u8); 4] = [
    (245, 133, 24),
    (84, 162, 75),
    (228, 87, 86),
    (178, 121, 162),
];
const GRID: RGBColor = RGBColor(221, 221, 221);
const GRAY: RGBColor = RGBColor(136, 136, 136);

pub struct TradeIn {
    pub ret: f64,
    pub hold: usize,
    pub reason: String,
}

pub struct FillIn {
    pub name: String,
    pub r_day: Vec<f64>,
    pub f_day: Vec<f64>,
    pub count: Vec<f64>,
    /// Mean daily log return of the random-pick pool baseline.
    pub random_pool: Vec<f64>,
    pub trades: Vec<TradeIn>,
}

pub struct ChartInput<'a> {
    pub title: &'a str,
    pub caption: &'a str,
    pub note: &'a str,
    pub dates: &'a [NaiveDate],
    pub first_row: usize,
    pub split_row: usize,
    pub fills: &'a [FillIn],
    /// Equal-weight liquid-universe daily returns (the "trend" reference).
    pub b_day: &'a [f64],
    pub benchmarks: &'a [(String, Vec<f64>)],
    /// (fill, period, stats) in results-frame order.
    pub frame: &'a [(String, String, Row)],
}

pub fn cumsum(v: impl Iterator<Item = f64>) -> Vec<f64> {
    let mut acc = 0.0;
    v.map(|x| {
        acc += x;
        acc
    })
    .collect()
}

fn e<E: std::fmt::Display>(x: E) -> String {
    x.to_string()
}

fn rgb(c: (u8, u8, u8)) -> RGBColor {
    RGBColor(c.0, c.1, c.2)
}

fn fill_color(i: usize) -> RGBColor {
    rgb(FILL_COLORS[i % FILL_COLORS.len()])
}

fn title_size(text: &str, width: i32, base: i32) -> i32 {
    let fit = width as f64 / (text.chars().count().max(1) as f64 * 0.56);
    fit.min(base as f64).max(8.0) as i32
}

fn text(size: i32, mono: bool, color: RGBColor, h: HPos, v: VPos) -> TextStyle<'static> {
    let family = if mono {
        FontFamily::Monospace
    } else {
        FontFamily::SansSerif
    };
    FontDesc::new(family, size as f64, FontStyle::Normal)
        .color(&color)
        .pos(Pos::new(h, v))
}

macro_rules! chart {
    ($area:expr, $title:expr, $w:expr, $xr:expr, $yr:expr) => {{
        let t: String = $title.to_string();
        let size = title_size(&t, $w, 17);
        ChartBuilder::on($area)
            .margin(8)
            .x_label_area_size(40)
            .y_label_area_size(70)
            .caption(t, ("sans-serif", size))
            .build_cartesian_2d($xr, $yr)
            .map_err(e)?
    }};
}

macro_rules! mesh {
    ($chart:expr, $xl:expr, $yl:expr, $xdesc:expr, $ydesc:expr, $xfmt:expr, $yfmt:expr) => {
        $chart
            .configure_mesh()
            .x_labels($xl)
            .y_labels($yl)
            .x_desc($xdesc)
            .y_desc($ydesc)
            .x_label_formatter($xfmt)
            .y_label_formatter($yfmt)
            .label_style(("sans-serif", 14))
            .axis_desc_style(("sans-serif", 14))
            .bold_line_style(GRID)
            .light_line_style(TRANSPARENT)
            .draw()
            .map_err(e)?
    };
}

macro_rules! legend {
    ($chart:expr, $pos:expr, $size:expr) => {
        $chart
            .configure_series_labels()
            .position($pos)
            .background_style(WHITE.mix(0.85))
            .border_style(RGBColor(190, 190, 190))
            .label_font(("sans-serif", $size))
            .draw()
            .map_err(e)?
    };
}

fn dates_formatter(dates: &[NaiveDate]) -> impl Fn(&usize) -> String + '_ {
    move |i| {
        dates
            .get(*i)
            .map(|d| d.format("%Y-%m").to_string())
            .unwrap_or_default()
    }
}

fn category_formatter(labels: Vec<String>) -> impl Fn(&f64) -> String {
    move |x| {
        let r = x.round();
        if (x - r).abs() < 1e-6 && r >= 0.0 && (r as usize) < labels.len() {
            labels[r as usize].clone()
        } else {
            String::new()
        }
    }
}

fn pad_range(lo: f64, hi: f64, frac: f64) -> (f64, f64) {
    let (lo, hi) = if lo.is_finite() && hi.is_finite() && lo < hi {
        (lo, hi)
    } else {
        (-0.1, 0.1)
    };
    let p = frac * (hi - lo);
    (lo - p, hi + p)
}

fn finite_extent<'a>(vals: impl Iterator<Item = &'a f64>) -> (f64, f64) {
    vals.filter(|v| v.is_finite())
        .fold((f64::MAX, f64::MIN), |(lo, hi), v| (lo.min(*v), hi.max(*v)))
}

fn fmt_opt(v: f64, f: impl Fn(f64) -> String) -> String {
    if v.is_finite() {
        f(v)
    } else {
        "n/a".into()
    }
}

const METRIC_ROWS: [(&str, &str); 17] = [
    ("trades", "n trades"),
    ("total_log", "total (log)"),
    ("trend", "trend (log)"),
    ("random_pick_pool", "random pool"),
    ("bias", "bias"),
    ("skill", "skill"),
    ("skill_pct", "skill % of total"),
    ("skill_p", "p(skill<=0)"),
    ("sharpe", "Sharpe"),
    ("pf", "profit factor"),
    ("pf_ci_low", "PF bootstrap low"),
    ("pf_ci_high", "PF bootstrap high"),
    ("win_pct", "win rate %"),
    ("cagr_pct", "CAGR %"),
    ("max_dd_pct", "max DD %"),
    ("avg_positions", "avg positions"),
    ("avg_fraction_invested", "avg invested"),
];

fn metric_text(key: &str, v: f64) -> String {
    fmt_opt(v, |v| match key {
        "trades" | "avg_positions" => format!("{v:.0}"),
        "total_log" | "trend" | "random_pick_pool" | "skill" => format!("{v:+.3}"),
        "bias" => format!("{v:.3}"),
        "skill_pct" | "avg_fraction_invested" => format!("{:.0}%", v * 100.0),
        "skill_p" | "pf" | "pf_ci_low" | "pf_ci_high" => format!("{v:.2}"),
        "sharpe" => format!("{v:+.2}"),
        "win_pct" | "max_dd_pct" => format!("{v:.1}"),
        "cagr_pct" => format!("{v:+.1}"),
        _ => format!("{v}"),
    })
}

fn panel_a(area: &Area, c: &ChartInput, w: i32) -> Result<(), String> {
    let (lo, hi) = (c.first_row, c.dates.len() - 1);
    // (label, values, color, dash (size, spacing), line width)
    let mut series: Vec<(String, Vec<f64>, RGBColor, Option<(i32, i32)>, u32)> = vec![];
    for (i, f) in c.fills.iter().enumerate() {
        let strat = cumsum(f.r_day.iter().map(|r| r.ln_1p()));
        series.push((
            format!("strategy [{}] net of costs (end {:+.2})", f.name, strat[hi]),
            strat,
            fill_color(i),
            None,
            2,
        ));
        let rnd = cumsum(f.random_pool.iter().copied());
        series.push((
            format!("random-pick pool mean [{}] (end {:+.2})", f.name, rnd[hi]),
            rnd,
            fill_color(i),
            Some((2, 3)),
            1,
        ));
    }
    if let Some(r) = c.fills.last() {
        let trend = cumsum(r.f_day.iter().zip(c.b_day).map(|(f, b)| f * b.ln_1p()));
        series.push((
            format!(
                "trend = exposure-weighted equal-weight liquid universe [{}] (end {:+.2})",
                r.name, trend[hi]
            ),
            trend,
            RGBColor(85, 85, 85),
            Some((7, 4)),
            2,
        ));
    }
    for (i, (name, rets)) in c.benchmarks.iter().enumerate() {
        let path = cumsum(rets.iter().map(|r| r.max(-0.999999).ln_1p()));
        series.push((
            format!("benchmark {name} (end {:+.2})", path[hi]),
            path,
            rgb(BENCH_COLORS[i % BENCH_COLORS.len()]),
            Some((9, 3)),
            2,
        ));
    }
    let (mut ymin, mut ymax) = (f64::MAX, f64::MIN);
    for (_, v, ..) in &series {
        let (a, b) = finite_extent(v[lo..=hi].iter());
        ymin = ymin.min(a);
        ymax = ymax.max(b);
    }
    let (y0, y1) = pad_range(ymin, ymax, 0.05);
    let split = c.split_row.min(hi).max(lo);
    let mut ch = chart!(
        area,
        "A. Cumulative log return: strategy, partition baselines, and external benchmarks",
        w,
        lo..hi,
        y0..y1
    );
    mesh!(
        ch,
        8,
        10,
        "session date",
        "cumulative log return",
        &dates_formatter(c.dates),
        &|v: &f64| format!("{v:.1}")
    );
    ch.draw_series(std::iter::once(Rectangle::new(
        [(lo, y0), (split, y1)],
        RGBColor(232, 240, 254).mix(0.5).filled(),
    )))
    .map_err(e)?;
    ch.draw_series(std::iter::once(Rectangle::new(
        [(split, y0), (hi, y1)],
        RGBColor(253, 236, 234).mix(0.5).filled(),
    )))
    .map_err(e)?;
    ch.draw_series(LineSeries::new([(split, y0), (split, y1)], GRAY))
        .map_err(e)?;
    for (label, vals, col, dash, lw) in series {
        let pts: Vec<(usize, f64)> = (lo..=hi).map(|i| (i, vals[i])).collect();
        let style = col.stroke_width(lw);
        // legend swatch: solid, or three dashes for broken lines
        let gap = if dash.is_some() { 3 } else { 0 };
        let glyph = move |(x, y): (i32, i32)| {
            EmptyElement::at((x, y))
                + PathElement::new(vec![(0, 0), (6, 0)], style)
                + PathElement::new(vec![(6 + gap, 0), (14, 0)], style)
                + PathElement::new(vec![(14 + gap, 0), (22, 0)], style)
        };
        match dash {
            None => ch
                .draw_series(LineSeries::new(pts, style))
                .map_err(e)?
                .label(label)
                .legend(glyph),
            Some((size, spacing)) => ch
                .draw_series(DashedLineSeries::new(pts, size, spacing, style))
                .map_err(e)?
                .label(label)
                .legend(glyph),
        };
    }
    ch.draw_series(std::iter::once(Text::new(
        " DEV (selection period)",
        (lo, y1),
        text(15, false, RGBColor(26, 79, 156), HPos::Left, VPos::Top),
    )))
    .map_err(e)?;
    ch.draw_series(std::iter::once(Text::new(
        " TEST (locked out-of-sample)",
        (split, y1),
        text(15, false, RGBColor(163, 38, 32), HPos::Left, VPos::Top),
    )))
    .map_err(e)?;
    legend!(ch, SeriesLabelPosition::LowerLeft, 12);
    Ok(())
}

fn panel_b(area: &Area, c: &ChartInput, w: i32) -> Result<(), String> {
    area.draw(&Text::new(
        "B. Key metrics (log returns, research scale, net of costs)",
        (w / 2, 12),
        text(16, false, BLACK, HPos::Center, VPos::Top),
    ))
    .map_err(e)?;
    // Each cell sits at a fixed x: SVG collapses runs of spaces, so columns cannot be space-padded.
    let cw = 6.7;
    let label_w = (18.0 * cw) as i32 + 8;
    let col_w = (16.0 * cw) as i32;
    let (x0, y0, dy) = (8, 48, 21);
    let mono = |h| text(11, true, BLACK, h, VPos::Top);
    area.draw(&Text::new("metric", (x0, y0), mono(HPos::Left)))
        .map_err(e)?;
    for (j, (fill, period, _)) in c.frame.iter().enumerate() {
        let head: String = fill.chars().take(9).collect();
        area.draw(&Text::new(
            format!("{head}|{period}"),
            (label_w + (j as i32 + 1) * col_w, y0),
            mono(HPos::Right),
        ))
        .map_err(e)?;
    }
    for (k, (key, label)) in METRIC_ROWS.iter().enumerate() {
        let y = y0 + dy * (k as i32 + 1);
        area.draw(&Text::new(*label, (x0, y), mono(HPos::Left)))
            .map_err(e)?;
        for (j, (_, _, row)) in c.frame.iter().enumerate() {
            area.draw(&Text::new(
                metric_text(key, row.get(key)),
                (label_w + (j as i32 + 1) * col_w, y),
                mono(HPos::Right),
            ))
            .map_err(e)?;
        }
    }
    Ok(())
}

fn panel_c(area: &Area, c: &ChartInput, w: i32, h: i32) -> Result<(), String> {
    let n = c.frame.len();
    let (mut lo, mut hi) = (0.0f64, 0.0f64);
    for (_, _, r) in c.frame {
        let (mut pos, mut neg) = (0.0, 0.0);
        for k in ["trend", "skill", "bias"] {
            let v = r.get(k);
            if !v.is_finite() {
                continue;
            }
            if v >= 0.0 {
                pos += v;
            } else {
                neg += v;
            }
        }
        let tot = r.get("total_log");
        let tot = if tot.is_finite() { tot } else { 0.0 };
        lo = lo.min(neg).min(tot);
        hi = hi.max(pos).max(tot);
    }
    let (y0, y1) = pad_range(lo, hi, 0.18);
    let unit = (y1 - y0) / (h as f64 - 130.0); // data units per pixel
    let labels: Vec<String> = c.frame.iter().map(|(f, p, _)| format!("{f} {p}")).collect();
    let mut ch = chart!(
        area,
        "C. Partition test: total = trend + skill + bias (log return per period; dev bias = selection haircut, test bias = 0)",
        w,
        -0.6f64..(n as f64 - 0.4),
        y0..y1
    );
    mesh!(
        ch,
        n + 2,
        8,
        "",
        "log return over period",
        &category_formatter(labels),
        &|v: &f64| format!("{v:.2}")
    );
    ch.draw_series(LineSeries::new([(-0.6, 0.0), (n as f64 - 0.4, 0.0)], GRAY))
        .map_err(e)?;
    let parts = [
        ("trend", RGBColor(68, 114, 196)),
        ("skill", RGBColor(112, 173, 71)),
        ("bias", RGBColor(217, 83, 79)),
    ];
    for (i, (_, _, r)) in c.frame.iter().enumerate() {
        let x = i as f64;
        let (mut pos, mut neg) = (0.0f64, 0.0f64);
        for (name, col) in parts {
            let v = r.get(name);
            if !v.is_finite() {
                continue;
            }
            let base = if v >= 0.0 { pos } else { neg };
            let rect = Rectangle::new([(x - 0.3, base), (x + 0.3, base + v)], col.filled());
            let s = ch.draw_series(std::iter::once(rect)).map_err(e)?;
            if i == 0 {
                s.label(name).legend(move |(lx, ly)| {
                    Rectangle::new([(lx, ly - 6), (lx + 22, ly + 6)], col.filled())
                });
            }
            if v.abs() > 0.03 {
                let mid = base + v / 2.0;
                let st = text(13, false, WHITE, HPos::Center, VPos::Center);
                ch.draw_series(std::iter::once(Text::new(
                    name,
                    (x, mid + 7.0 * unit),
                    st.clone(),
                )))
                .map_err(e)?;
                ch.draw_series(std::iter::once(Text::new(
                    format!("{v:+.2}"),
                    (x, mid - 7.0 * unit),
                    st,
                )))
                .map_err(e)?;
            }
            if v >= 0.0 {
                pos += v;
            } else {
                neg += v;
            }
        }
        let tot = r.get("total_log");
        if tot.is_finite() {
            let dot = Circle::new((x, tot), 7, BLACK.filled());
            let s = ch.draw_series(std::iter::once(dot)).map_err(e)?;
            if i == 0 {
                s.label("total")
                    .legend(|(lx, ly)| Circle::new((lx + 11, ly), 6, BLACK.filled()));
            }
            ch.draw_series(std::iter::once(Text::new(
                format!("total {tot:+.2}"),
                (x + 0.33, tot),
                text(12, false, BLACK, HPos::Left, VPos::Center),
            )))
            .map_err(e)?;
        }
        let st = text(12, false, BLACK, HPos::Center, VPos::Bottom);
        ch.draw_series(std::iter::once(Text::new(
            format!(
                "skill {} of total",
                fmt_opt(r.get("skill_pct"), |v| format!("{:.0}%", v * 100.0))
            ),
            (x, y0 + 16.0 * unit),
            st.clone(),
        )))
        .map_err(e)?;
        ch.draw_series(std::iter::once(Text::new(
            format!("p={}", fmt_opt(r.get("skill_p"), |v| format!("{v:.2}"))),
            (x, y0 + 2.0 * unit),
            st,
        )))
        .map_err(e)?;
    }
    legend!(ch, SeriesLabelPosition::UpperRight, 12);
    Ok(())
}

fn panel_d(area: &Area, c: &ChartInput, w: i32) -> Result<(), String> {
    let (lo, hi) = (c.first_row, c.dates.len() - 1);
    let mut curves = vec![];
    for f in c.fills {
        let mut eq = 1.0;
        let mut peak = f64::MIN;
        let dd: Vec<f64> = f.r_day[lo..]
            .iter()
            .map(|r| {
                eq *= 1.0 + r;
                peak = peak.max(eq);
                100.0 * (eq / peak - 1.0)
            })
            .collect();
        curves.push(dd);
    }
    let (mn, _) = finite_extent(curves.iter().flatten());
    let (y0, y1) = pad_range(mn.min(-0.001), 0.0, 0.05);
    let mut ch = chart!(
        area,
        "D. Drawdown of the strategy equity curve",
        w,
        lo..hi,
        y0..y1
    );
    mesh!(
        ch,
        8,
        8,
        "session date",
        "drawdown from peak, %",
        &dates_formatter(c.dates),
        &|v: &f64| format!("{v:.0}")
    );
    let split = c.split_row.min(hi).max(lo);
    ch.draw_series(LineSeries::new([(split, y0), (split, y1)], GRAY))
        .map_err(e)?;
    for (i, (f, dd)) in c.fills.iter().zip(&curves).enumerate() {
        let col = fill_color(i);
        let worst = dd.iter().cloned().fold(0.0, f64::min);
        ch.draw_series(LineSeries::new(
            dd.iter().enumerate().map(|(k, v)| (lo + k, *v)),
            col.stroke_width(1),
        ))
        .map_err(e)?
        .label(format!("{} (max {worst:.1}%)", f.name))
        .legend(move |(x, y)| PathElement::new(vec![(x, y), (x + 22, y)], col.stroke_width(2)));
    }
    legend!(ch, SeriesLabelPosition::LowerLeft, 12);
    Ok(())
}

fn panel_e(area: &Area, c: &ChartInput, w: i32) -> Result<(), String> {
    let first = c.first_row;
    let years: Vec<i32> = c.dates[first..]
        .iter()
        .map(|d| d.year())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    let nf = c.fills.len().max(1);
    let split_year = c.dates[c.split_row.min(c.dates.len() - 1)].year();
    let vals: Vec<Vec<f64>> = c
        .fills
        .iter()
        .map(|f| {
            years
                .iter()
                .map(|y| {
                    let s: f64 = (first..c.dates.len())
                        .filter(|&i| c.dates[i].year() == *y)
                        .map(|i| f.r_day[i].ln_1p())
                        .sum();
                    100.0 * s.exp_m1()
                })
                .collect()
        })
        .collect();
    let (mn, mx) = finite_extent(vals.iter().flatten());
    let (mut y0, mut y1) = (mn.min(0.0), mx.max(0.0));
    let span = (y1 - y0).max(1.0);
    y0 -= if mn < 0.0 { 0.08 * span } else { 0.0 };
    y1 += 0.08 * span;
    let labels: Vec<String> = years
        .iter()
        .map(|y| format!("{y}{}", if *y >= split_year { " (test)" } else { "" }))
        .collect();
    let n = years.len();
    let mut ch = chart!(
        area,
        "E. Calendar-year return of the strategy account (percent, net of costs)",
        w,
        -0.6f64..(n as f64 - 0.4),
        y0..y1
    );
    mesh!(
        ch,
        n + 2,
        8,
        "",
        "return, %",
        &category_formatter(labels),
        &|v: &f64| format!("{v:.0}")
    );
    ch.draw_series(LineSeries::new([(-0.6, 0.0), (n as f64 - 0.4, 0.0)], GRAY))
        .map_err(e)?;
    let bw = 0.8 / nf as f64;
    for (i, f) in c.fills.iter().enumerate() {
        let col = fill_color(i);
        let bars = vals[i].iter().enumerate().map(|(k, v)| {
            let x0 = k as f64 - 0.4 + i as f64 * bw;
            Rectangle::new([(x0, 0.0), (x0 + bw, *v)], col.filled())
        });
        ch.draw_series(bars)
            .map_err(e)?
            .label(f.name.clone())
            .legend(move |(x, y)| Rectangle::new([(x, y - 6), (x + 22, y + 6)], col.filled()));
        let value_labels = vals[i].iter().enumerate().map(|(k, v)| {
            let x = k as f64 - 0.4 + (i as f64 + 0.5) * bw;
            let vp = if *v >= 0.0 { VPos::Bottom } else { VPos::Top };
            Text::new(
                format!("{v:.0}"),
                (x, *v),
                text(10, false, BLACK, HPos::Center, vp),
            )
        });
        ch.draw_series(value_labels).map_err(e)?;
    }
    legend!(ch, SeriesLabelPosition::UpperLeft, 12);
    Ok(())
}

fn panel_f(area: &Area, c: &ChartInput, w: i32) -> Result<(), String> {
    let mut hists = vec![];
    for f in c.fills {
        let mut counts = [0u32; 50];
        for t in &f.trades {
            let v = (t.ret * 100.0).clamp(-25.0, 25.0);
            counts[((v + 25.0).floor() as usize).min(49)] += 1;
        }
        hists.push(counts);
    }
    let ymax = hists.iter().flatten().copied().max().unwrap_or(1).max(1) as f64 * 1.05;
    let mut ch = chart!(
        area,
        "F. Net return per trade (clipped to +-25%)",
        w,
        -25f64..25f64,
        0f64..ymax
    );
    mesh!(
        ch,
        11,
        8,
        "trade return, %",
        "number of trades",
        &|v: &f64| format!("{v:.0}"),
        &|v: &f64| format!("{v:.0}")
    );
    for (i, f) in c.fills.iter().enumerate() {
        let col = fill_color(i);
        let rets: Vec<f64> = f.trades.iter().map(|t| t.ret * 100.0).collect();
        let n = rets.len();
        let (mean, med, win) = if n == 0 {
            (f64::NAN, f64::NAN, f64::NAN)
        } else {
            (
                stats::mean(&rets),
                stats::median(&rets),
                rets.iter().filter(|v| **v > 0.0).count() as f64 / n as f64,
            )
        };
        let bars = hists[i].iter().enumerate().map(|(k, cnt)| {
            let x0 = -25.0 + k as f64;
            Rectangle::new([(x0, 0.0), (x0 + 1.0, *cnt as f64)], col.mix(0.5).filled())
        });
        ch.draw_series(bars)
            .map_err(e)?
            .label(format!(
                "{}: n={n}, mean {mean:+.2}%, median {med:+.2}%, win {:.0}%",
                f.name,
                win * 100.0
            ))
            .legend(move |(x, y)| {
                Rectangle::new([(x, y - 6), (x + 22, y + 6)], col.mix(0.5).filled())
            });
    }
    ch.draw_series(LineSeries::new([(0.0, 0.0), (0.0, ymax)], BLACK))
        .map_err(e)?;
    legend!(ch, SeriesLabelPosition::UpperRight, 11);
    Ok(())
}

/// Grouped bars over integer categories: one bar per fill, centred on each category.
#[allow(clippy::too_many_arguments)]
fn grouped_bars(
    area: &Area,
    c: &ChartInput,
    w: i32,
    title: &str,
    xdesc: &str,
    labels: Vec<String>,
    values: Vec<Vec<f64>>,
    legend_names: Vec<String>,
    value_labels: bool,
) -> Result<(), String> {
    let n = labels.len();
    let ymax = values.iter().flatten().cloned().fold(1.0, f64::max) * 1.12;
    let mut ch = chart!(area, title, w, -0.6f64..(n as f64 - 0.4), 0f64..ymax);
    mesh!(
        ch,
        n + 2,
        8,
        xdesc,
        "number of trades",
        &category_formatter(labels),
        &|v: &f64| format!("{v:.0}")
    );
    let bw = 0.8 / c.fills.len().max(1) as f64;
    for (i, name) in legend_names.into_iter().enumerate() {
        let col = fill_color(i);
        let bars = values[i].iter().enumerate().map(|(k, v)| {
            let x0 = k as f64 - 0.4 + i as f64 * bw;
            Rectangle::new([(x0, 0.0), (x0 + bw, *v)], col.filled())
        });
        ch.draw_series(bars)
            .map_err(e)?
            .label(name)
            .legend(move |(x, y)| Rectangle::new([(x, y - 6), (x + 22, y + 6)], col.filled()));
        if value_labels {
            let tx = values[i].iter().enumerate().map(|(k, v)| {
                let x = k as f64 - 0.4 + (i as f64 + 0.5) * bw;
                Text::new(
                    format!("{v:.0}"),
                    (x, *v),
                    text(10, false, BLACK, HPos::Center, VPos::Bottom),
                )
            });
            ch.draw_series(tx).map_err(e)?;
        }
    }
    legend!(ch, SeriesLabelPosition::UpperRight, 12);
    Ok(())
}

fn panel_g(area: &Area, c: &ChartInput, w: i32) -> Result<(), String> {
    let max_hold = c
        .fills
        .iter()
        .flat_map(|f| f.trades.iter().map(|t| t.hold))
        .max()
        .unwrap_or(0);
    let mut values = vec![];
    let mut names = vec![];
    for f in c.fills {
        let mut counts = vec![0.0; max_hold + 1];
        for t in &f.trades {
            counts[t.hold] += 1.0;
        }
        let mean = if f.trades.is_empty() {
            f64::NAN
        } else {
            f.trades.iter().map(|t| t.hold as f64).sum::<f64>() / f.trades.len() as f64
        };
        values.push(counts);
        names.push(format!("{} (mean {mean:.1})", f.name));
    }
    let labels = (0..=max_hold).map(|h| h.to_string()).collect();
    grouped_bars(
        area,
        c,
        w,
        "G. Holding period",
        "sessions held (entry to exit)",
        labels,
        values,
        names,
        false,
    )
}

fn panel_h(area: &Area, c: &ChartInput, w: i32) -> Result<(), String> {
    let reasons: Vec<String> = c
        .fills
        .iter()
        .flat_map(|f| f.trades.iter().map(|t| t.reason.clone()))
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    let values = c
        .fills
        .iter()
        .map(|f| {
            reasons
                .iter()
                .map(|r| f.trades.iter().filter(|t| &t.reason == r).count() as f64)
                .collect()
        })
        .collect();
    let names = c.fills.iter().map(|f| f.name.clone()).collect();
    grouped_bars(
        area,
        c,
        w,
        "H. Exit reason (resid = signal recovery, cap = max hold, stop/target = price rules)",
        "",
        reasons,
        values,
        names,
        true,
    )
}

fn panel_i(area: &Area, c: &ChartInput, w: i32) -> Result<(), String> {
    let (lo, hi) = (c.first_row, c.dates.len() - 1);
    let curves: Vec<Vec<f64>> = c
        .fills
        .iter()
        .map(|f| {
            let v = &f.count[lo..];
            (0..v.len())
                .map(|k| stats::mean(&v[(k + 1).saturating_sub(20)..=k]))
                .collect()
        })
        .collect();
    let (_, mx) = finite_extent(curves.iter().flatten());
    let top = mx.max(1.0) * 1.08;
    let mut ch = chart!(
        area,
        "I. Open positions (20-session average)",
        w,
        lo..hi,
        0f64..top
    );
    mesh!(
        ch,
        8,
        8,
        "session date",
        "number of open positions",
        &dates_formatter(c.dates),
        &|v: &f64| format!("{v:.1}")
    );
    let split = c.split_row.min(hi).max(lo);
    ch.draw_series(LineSeries::new([(split, 0.0), (split, top)], GRAY))
        .map_err(e)?;
    for (i, (f, v)) in c.fills.iter().zip(&curves).enumerate() {
        let col = fill_color(i);
        ch.draw_series(LineSeries::new(
            v.iter().enumerate().map(|(k, x)| (lo + k, *x)),
            col.stroke_width(1),
        ))
        .map_err(e)?
        .label(f.name.clone())
        .legend(move |(x, y)| PathElement::new(vec![(x, y), (x + 22, y)], col.stroke_width(2)));
    }
    legend!(ch, SeriesLabelPosition::UpperRight, 12);
    Ok(())
}

pub fn draw(path: &Path, c: &ChartInput) -> Result<(), String> {
    if c.dates.len() < 2 || c.first_row >= c.dates.len() - 1 || c.fills.is_empty() {
        return Err("not enough sessions to chart".into());
    }
    let root = SVGBackend::new(path, (W as u32, H as u32)).into_drawing_area();
    root.fill(&WHITE).map_err(e)?;
    root.draw(&Text::new(
        c.title,
        (W / 2, 14),
        FontDesc::new(FontFamily::SansSerif, 26.0, FontStyle::Bold)
            .color(&BLACK)
            .pos(Pos::new(HPos::Center, VPos::Top)),
    ))
    .map_err(e)?;
    root.draw(&Text::new(
        c.caption,
        (W / 2, 56),
        text(12, true, RGBColor(51, 51, 51), HPos::Center, VPos::Top),
    ))
    .map_err(e)?;
    if !c.note.is_empty() {
        root.draw(&Text::new(
            c.note,
            (W / 2, H - 30),
            text(13, false, RGBColor(85, 85, 85), HPos::Center, VPos::Top),
        ))
        .map_err(e)?;
    }
    let (top, bottom, left, right) = (90, 50, 10, 10);
    let cw = (W - left - right) / 3;
    let ch = (H - top - bottom) / 4;
    let cell = |col: i32, row: i32, span: i32| {
        root.clone().shrink(
            (left + col * cw, top + row * ch),
            ((cw * span - 6) as u32, (ch - 6) as u32),
        )
    };
    panel_a(&cell(0, 0, 2), c, cw * 2)?;
    panel_b(&cell(2, 0, 1), c, cw)?;
    panel_c(&cell(0, 1, 2), c, cw * 2, ch)?;
    panel_d(&cell(2, 1, 1), c, cw)?;
    panel_e(&cell(0, 2, 2), c, cw * 2)?;
    panel_f(&cell(2, 2, 1), c, cw)?;
    panel_g(&cell(0, 3, 1), c, cw)?;
    panel_h(&cell(1, 3, 1), c, cw)?;
    panel_i(&cell(2, 3, 1), c, cw)?;
    root.present().map_err(e)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cumsum_accumulates_from_the_start() {
        let c = cumsum([0.5, 0.1, 0.1].into_iter());
        assert_eq!(c[0], 0.5);
        assert!((c[2] - 0.7).abs() < 1e-12);
    }

    #[test]
    fn category_formatter_labels_only_integer_ticks() {
        let f = category_formatter(vec!["a".into(), "b".into()]);
        assert_eq!(f(&1.0), "b");
        assert_eq!(f(&0.5), "");
        assert_eq!(f(&5.0), "");
    }
}
