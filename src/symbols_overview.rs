//! Compact per-symbol comparison of a strategy batch.

use chrono::NaiveDate;
use plotters::prelude::*;
use std::collections::BTreeSet;
use std::path::Path;

const WIDTH: u32 = 1800;
const TILE_H: u32 = 230;
const MAX_TILES: usize = 24;

#[derive(Clone, Debug)]
pub struct SymbolStats {
    pub dev_total: f64,
    pub test_total: f64,
    pub dev_excess: f64,
    pub test_excess: f64,
    pub dev_sharpe: f64,
    pub test_sharpe: f64,
    pub dev_pf: f64,
    pub test_pf: f64,
    pub dev_trades: usize,
    pub test_trades: usize,
    pub dev_win_pct: f64,
    pub test_win_pct: f64,
    pub dev_max_dd: f64,
    pub test_max_dd: f64,
    pub flags: String,
}

#[derive(Clone, Debug)]
pub struct SymbolSeries {
    pub symbol: String,
    pub source: String,
    pub strategy_r: Vec<f64>,
    pub buy_hold_r: Vec<f64>,
    pub benchmark_r: Option<Vec<f64>>,
    pub stats: SymbolStats,
}

#[derive(Clone, Debug)]
pub struct VariantSeries {
    pub name: String,
    pub dev_skill: f64,
    pub symbols: Vec<SymbolSeries>,
}

#[derive(Clone, Debug)]
pub struct Overview {
    pub dates: Vec<NaiveDate>,
    pub split_row: usize,
    pub variants: Vec<VariantSeries>,
}

fn cumulative_log(returns: &[f64]) -> Vec<f64> {
    let mut total = 0.0;
    returns
        .iter()
        .map(|r| {
            total += r.max(-0.999_999).ln_1p();
            total
        })
        .collect()
}

impl Overview {
    pub fn write(&self, svg_path: &Path, csv_path: &Path) -> Result<(), String> {
        if self.variants.is_empty() {
            return Err("symbol overview needs at least one baseline or variant".into());
        }
        self.write_csv(csv_path)?;
        self.write_svg(svg_path)
    }

    fn write_csv(&self, path: &Path) -> Result<(), String> {
        let mut csv = String::from(
            "variant,symbol,source,dev_total_log,test_total_log,dev_excess_vs_buy_hold,test_excess_vs_buy_hold,dev_sharpe,test_sharpe,dev_pf,test_pf,dev_trades,test_trades,dev_win_pct,test_win_pct,dev_max_dd,test_max_dd,flags\n",
        );
        for variant in &self.variants {
            for symbol in &variant.symbols {
                let s = &symbol.stats;
                csv += &format!(
                    "{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{}\n",
                    variant.name,
                    symbol.symbol,
                    symbol.source,
                    s.dev_total,
                    s.test_total,
                    s.dev_excess,
                    s.test_excess,
                    s.dev_sharpe,
                    s.test_sharpe,
                    s.dev_pf,
                    s.test_pf,
                    s.dev_trades,
                    s.test_trades,
                    s.dev_win_pct,
                    s.test_win_pct,
                    s.dev_max_dd,
                    s.test_max_dd,
                    s.flags
                );
            }
        }
        std::fs::write(path, csv).map_err(|e| format!("{}: {e}", path.display()))
    }

    fn write_svg(&self, path: &Path) -> Result<(), String> {
        if self.dates.len() < 2 {
            return Err("symbol overview needs at least two sessions".into());
        }
        let Some(reference) = self
            .variants
            .iter()
            .find(|v| v.name == "baseline")
            .or(self.variants.first())
        else {
            return Err("symbol overview has no reference variant".into());
        };
        let mut symbols: Vec<&SymbolSeries> = reference.symbols.iter().collect();
        if symbols.len() > MAX_TILES {
            let mut ranked = symbols.clone();
            ranked.sort_by(|a, b| a.stats.dev_excess.total_cmp(&b.stats.dev_excess));
            let mut keep: BTreeSet<&str> = BTreeSet::new();
            for s in ranked.iter().take(MAX_TILES / 2) {
                keep.insert(&s.symbol);
            }
            for s in ranked.iter().rev().take(MAX_TILES / 2) {
                keep.insert(&s.symbol);
            }
            symbols.retain(|s| keep.contains(s.symbol.as_str()));
            symbols.sort_by(|a, b| {
                b.stats
                    .dev_excess
                    .total_cmp(&a.stats.dev_excess)
                    .then_with(|| a.symbol.cmp(&b.symbol))
            });
        }
        let selected = self
            .variants
            .iter()
            .filter(|variant| variant.name != "baseline")
            .filter(|variant| variant.dev_skill.is_finite())
            .max_by(|a, b| a.dev_skill.total_cmp(&b.dev_skill))
            .unwrap_or(reference);
        let mut extent = Vec::new();
        for symbol in &symbols {
            for variant in [reference, selected] {
                if let Some(series) = variant
                    .symbols
                    .iter()
                    .find(|series| series.symbol == symbol.symbol)
                {
                    extent.extend(cumulative_log(&series.strategy_r));
                    if let Some(benchmark) = &series.benchmark_r {
                        extent.extend(cumulative_log(benchmark));
                    }
                }
            }
            extent.extend(cumulative_log(&symbol.buy_hold_r));
        }
        let low = extent.iter().copied().fold(f64::INFINITY, f64::min);
        let high = extent.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        let (low, high) = if low < high {
            (low - (high - low) * 0.08, high + (high - low) * 0.08)
        } else {
            (low - 0.1, high + 0.1)
        };
        let rows = symbols.len().div_ceil(5).max(1);
        let height = (rows as u32 * TILE_H + 100).max(TILE_H + 100);
        let root = SVGBackend::new(path, (WIDTH, height)).into_drawing_area();
        root.fill(&WHITE).map_err(|e| e.to_string())?;
        root.draw(&Text::new(
            "Per-symbol returns | thick = baseline, green = best dev variant, dashed = symbol buy-and-hold, orange = benchmark",
            (18, 24),
            ("sans-serif", 19).into_font(),
        ))
        .map_err(|e| e.to_string())?;
        root.draw(&Text::new(
            format!(
                "dev < {}; selected variant on dev only: {}",
                self.dates[self.split_row.min(self.dates.len() - 1)],
                selected.name
            ),
            (18, 51),
            ("sans-serif", 15).into_font().color(&RGBColor(90, 90, 90)),
        ))
        .map_err(|e| e.to_string())?;
        let (_, grid) = root.split_vertically(65);
        let areas = grid.split_evenly((rows, 5));
        for (area, reference_symbol) in areas.iter().zip(symbols) {
            let Some(baseline) = reference
                .symbols
                .iter()
                .find(|s| s.symbol == reference_symbol.symbol)
            else {
                continue;
            };
            let Some(variant_symbol) = selected
                .symbols
                .iter()
                .find(|s| s.symbol == reference_symbol.symbol)
            else {
                continue;
            };
            draw_tile(
                area,
                self,
                baseline,
                variant_symbol,
                selected.name == reference.name,
                (low, high),
            )?;
        }
        root.present().map_err(|e| e.to_string())
    }
}

fn draw_tile(
    area: &DrawingArea<SVGBackend<'_>, plotters::coord::Shift>,
    overview: &Overview,
    baseline: &SymbolSeries,
    selected: &SymbolSeries,
    same_variant: bool,
    y_range: (f64, f64),
) -> Result<(), String> {
    let n = overview.dates.len();
    if n < 2 {
        return Err("symbol overview needs at least two sessions".into());
    }
    let base = cumulative_log(&baseline.strategy_r);
    let best = cumulative_log(&selected.strategy_r);
    let buy_hold = cumulative_log(&baseline.buy_hold_r);
    let benchmark = baseline.benchmark_r.as_ref().map(|r| cumulative_log(r));
    let (low, high) = y_range;
    let (plot_area, stats_area) = area.split_vertically(178);
    let mut chart = ChartBuilder::on(&plot_area)
        .margin(7)
        .caption(
            format!("{} [{}]", baseline.symbol, baseline.source),
            ("sans-serif", 16),
        )
        .x_label_area_size(25)
        .y_label_area_size(42)
        .build_cartesian_2d(0usize..(n - 1), low..high)
        .map_err(|e| e.to_string())?;
    chart
        .configure_mesh()
        .x_labels(4)
        .y_labels(4)
        .x_label_formatter(&|x| {
            overview
                .dates
                .get(*x)
                .map(|d| d.format("%Y").to_string())
                .unwrap_or_default()
        })
        .y_label_formatter(&|y| format!("{y:.1}"))
        .label_style(("sans-serif", 10))
        .light_line_style(TRANSPARENT)
        .draw()
        .map_err(|e| e.to_string())?;
    let split = overview.split_row.min(n - 1);
    chart
        .draw_series(std::iter::once(PathElement::new(
            vec![(split, low), (split, high)],
            RGBColor(120, 120, 120).stroke_width(1),
        )))
        .map_err(|e| e.to_string())?;
    draw_line(&mut chart, &base, RGBColor(31, 119, 180), 3, "baseline")?;
    if !same_variant {
        draw_line(&mut chart, &best, RGBColor(44, 160, 44), 2, "best dev")?;
    }
    draw_dashed_line(
        &mut chart,
        &buy_hold,
        RGBColor(70, 70, 70),
        1,
        "buy-and-hold",
    )?;
    if let Some(benchmark) = benchmark {
        draw_dashed_line(
            &mut chart,
            &benchmark,
            RGBColor(255, 127, 14),
            1,
            "benchmark",
        )?;
    }
    let dev = &baseline.stats;
    let test = &selected.stats;
    let line1 = format!(
        "base dev/test log {:+.2}/{:+.2} | PF {:.2}/{:.2} | n {}/{}",
        dev.dev_total, dev.test_total, dev.dev_pf, dev.test_pf, dev.dev_trades, dev.test_trades
    );
    let line2 = format!(
        "{} dev/test excess {:+.2}/{:+.2} | Sharpe {:.2}/{:.2}",
        if same_variant { "base" } else { "best" },
        test.dev_excess,
        test.test_excess,
        test.dev_sharpe,
        test.test_sharpe
    );
    let line3 = format!(
        "maxDD base {:+.1}/{:+.1}% | best {:+.1}/{:+.1}%",
        dev.dev_max_dd * 100.0,
        dev.test_max_dd * 100.0,
        test.dev_max_dd * 100.0,
        test.test_max_dd * 100.0
    );
    stats_area
        .draw(&Text::new(
            line1,
            (10, 1),
            ("sans-serif", 9).into_font().color(&RGBColor(35, 35, 35)),
        ))
        .map_err(|e| e.to_string())?;
    stats_area
        .draw(&Text::new(
            line2,
            (10, 14),
            ("sans-serif", 9).into_font().color(&RGBColor(35, 35, 35)),
        ))
        .map_err(|e| e.to_string())?;
    stats_area
        .draw(&Text::new(
            line3,
            (10, 27),
            ("sans-serif", 9).into_font().color(&RGBColor(35, 35, 35)),
        ))
        .map_err(|e| e.to_string())?;
    if !test.flags.is_empty() {
        stats_area
            .draw(&Text::new(
                test.flags.chars().take(55).collect::<String>(),
                (10, 40),
                ("sans-serif", 9).into_font().color(&RGBColor(170, 45, 40)),
            ))
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

fn draw_dashed_line<DB: DrawingBackend>(
    chart: &mut ChartContext<
        '_,
        DB,
        Cartesian2d<
            plotters::coord::types::RangedCoordusize,
            plotters::coord::types::RangedCoordf64,
        >,
    >,
    values: &[f64],
    color: RGBColor,
    width: u32,
    label: &str,
) -> Result<(), String> {
    chart
        .draw_series(DashedLineSeries::new(
            values.iter().enumerate().map(|(i, v)| (i, *v)),
            4,
            3,
            color.stroke_width(width),
        ))
        .map_err(|e| e.to_string())?
        .label(label)
        .legend(move |(x, y)| {
            PathElement::new(vec![(x, y), (x + 15, y)], color.stroke_width(width))
        });
    Ok(())
}

fn draw_line<DB: DrawingBackend>(
    chart: &mut ChartContext<
        '_,
        DB,
        Cartesian2d<
            plotters::coord::types::RangedCoordusize,
            plotters::coord::types::RangedCoordf64,
        >,
    >,
    values: &[f64],
    color: RGBColor,
    width: u32,
    label: &str,
) -> Result<(), String> {
    chart
        .draw_series(LineSeries::new(
            values.iter().enumerate().map(|(i, v)| (i, *v)),
            color.stroke_width(width),
        ))
        .map_err(|e| e.to_string())?
        .label(label)
        .legend(move |(x, y)| {
            PathElement::new(vec![(x, y), (x + 15, y)], color.stroke_width(width))
        });
    Ok(())
}
