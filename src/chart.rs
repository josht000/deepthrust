//! Dashboard chart (pure-Rust `plotters` SVG backend): strategy vs benchmark cumulative log returns,
//! invested fraction, and the headline partition metrics.

use chrono::NaiveDate;
use plotters::prelude::*;
use std::path::Path;

pub struct Series {
    pub label: String,
    pub values: Vec<f64>,
}

pub struct ChartInput<'a> {
    pub title: &'a str,
    pub caption: &'a str,
    pub dates: &'a [NaiveDate],
    pub first_row: usize,
    pub split_row: usize,
    /// Cumulative log-return series (index-aligned with `dates`).
    pub curves: Vec<Series>,
    pub invested: Vec<f64>,
    pub metrics: Vec<String>,
}

pub fn cumulative_log(r: &[f64], first_row: usize) -> Vec<f64> {
    let mut acc = 0.0;
    r.iter()
        .enumerate()
        .map(|(i, v)| {
            if i >= first_row {
                acc += v.ln_1p();
            }
            acc
        })
        .collect()
}

pub fn draw(path: &Path, c: &ChartInput) -> Result<(), String> {
    if c.dates.len() < 2 || c.first_row >= c.dates.len() - 1 {
        return Err("not enough sessions to chart".into());
    }
    let root = SVGBackend::new(path, (1400, 900)).into_drawing_area();
    root.fill(&WHITE).map_err(|e| e.to_string())?;
    let (top, rest) = root.split_vertically(90);
    let (mid, bottom) = rest.split_vertically(560);
    top.draw(&Text::new(
        c.title.to_string(),
        (20, 12),
        ("sans-serif", 24).into_font(),
    ))
    .map_err(|e| e.to_string())?;
    top.draw(&Text::new(
        c.caption.to_string(),
        (20, 48),
        ("sans-serif", 13).into_font(),
    ))
    .map_err(|e| e.to_string())?;

    let (lo, hi) = (c.first_row, c.dates.len() - 1);
    let (mut ymin, mut ymax) = (f64::MAX, f64::MIN);
    for s in &c.curves {
        for v in &s.values[lo..=hi] {
            if v.is_finite() {
                ymin = ymin.min(*v);
                ymax = ymax.max(*v);
            }
        }
    }
    if !ymin.is_finite() || ymin >= ymax {
        ymin = -0.1;
        ymax = 0.1;
    }
    let pad = 0.05 * (ymax - ymin);
    let mut chart = ChartBuilder::on(&mid)
        .margin(15)
        .x_label_area_size(30)
        .y_label_area_size(60)
        .caption(
            "Cumulative log return (net of costs) vs benchmark",
            ("sans-serif", 16),
        )
        .build_cartesian_2d(lo..hi, (ymin - pad)..(ymax + pad))
        .map_err(|e| e.to_string())?;
    let dates = c.dates;
    chart
        .configure_mesh()
        .x_label_formatter(&|i| {
            dates
                .get(*i)
                .map(|d| d.format("%Y-%m").to_string())
                .unwrap_or_default()
        })
        .y_desc("cum. log return")
        .draw()
        .map_err(|e| e.to_string())?;
    let palette = [
        RGBColor(31, 119, 180),
        RGBColor(214, 39, 40),
        RGBColor(44, 160, 44),
        RGBColor(148, 103, 189),
    ];
    for (k, s) in c.curves.iter().enumerate() {
        let color = palette[k % palette.len()];
        chart
            .draw_series(LineSeries::new(
                (lo..=hi)
                    .filter(|&i| s.values[i].is_finite())
                    .map(|i| (i, s.values[i])),
                color.stroke_width(2),
            ))
            .map_err(|e| e.to_string())?
            .label(s.label.clone())
            .legend(move |(x, y)| {
                PathElement::new(vec![(x, y), (x + 20, y)], color.stroke_width(2))
            });
    }
    if c.split_row > lo && c.split_row < hi {
        chart
            .draw_series(LineSeries::new(
                [(c.split_row, ymin - pad), (c.split_row, ymax + pad)],
                BLACK.mix(0.6),
            ))
            .map_err(|e| e.to_string())?
            .label("dev | test split")
            .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], BLACK.mix(0.6)));
    }
    chart
        .configure_series_labels()
        .background_style(WHITE.mix(0.8))
        .border_style(BLACK)
        .position(SeriesLabelPosition::UpperLeft)
        .draw()
        .map_err(|e| e.to_string())?;

    let (left, right) = bottom.split_horizontally(900);
    let mut inv = ChartBuilder::on(&left)
        .margin(15)
        .x_label_area_size(30)
        .y_label_area_size(60)
        .caption("Fraction invested", ("sans-serif", 14))
        .build_cartesian_2d(lo..hi, 0f64..1.05)
        .map_err(|e| e.to_string())?;
    inv.configure_mesh()
        .x_label_formatter(&|i| {
            dates
                .get(*i)
                .map(|d| d.format("%Y").to_string())
                .unwrap_or_default()
        })
        .draw()
        .map_err(|e| e.to_string())?;
    inv.draw_series(LineSeries::new(
        (lo..=hi).map(|i| (i, c.invested[i])),
        RGBColor(31, 119, 180),
    ))
    .map_err(|e| e.to_string())?;
    for (k, line) in c.metrics.iter().enumerate() {
        right
            .draw(&Text::new(
                line.clone(),
                (10, 20 + 18 * k as i32),
                ("monospace", 12).into_font(),
            ))
            .map_err(|e| e.to_string())?;
    }
    root.present().map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cumulative_log_starts_at_first_row() {
        let c = cumulative_log(&[0.5, 0.1, 0.1], 1);
        assert_eq!(c[0], 0.0);
        assert!((c[2] - 2.0 * 0.1f64.ln_1p()).abs() < 1e-12);
    }
}
