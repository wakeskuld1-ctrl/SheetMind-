use std::f64::consts::PI;
use std::fmt::Write as _;

use polars::prelude::{AnyValue, Column, DataFrame};
use thiserror::Error;

use crate::frame::chart_ref_store::{PersistedChartDraft, PersistedChartType};

// 2026-03-23: Keep chart SVG rendering isolated from dispatcher so build/export can reuse one pure-Rust renderer.
#[derive(Debug, Error)]
pub enum ChartSvgError {
    #[error("\u{65e0}\u{6cd5}\u{6062}\u{590d}\u{56fe}\u{8868}\u{6570}\u{636e}: {0}")]
    RestoreChart(String),
    #[error("\u{56fe}\u{8868}\u{6ca1}\u{6709}\u{53ef}\u{7528}\u{6570}\u{636e}\u{884c}")]
    EmptyDataset,
    #[error("\u{56fe}\u{8868}\u{5bbd}\u{9ad8}\u{5fc5}\u{987b}\u{4e3a}\u{6b63}\u{6570}")]
    InvalidLayout,
    #[error("\u{56fe}\u{8868}\u{7f3a}\u{5c11}\u{5206}\u{7c7b}\u{5217}: {0}")]
    MissingCategoryColumn(String),
    #[error("\u{56fe}\u{8868}\u{7f3a}\u{5c11}\u{6570}\u{503c}\u{5217}: {0}")]
    MissingValueColumn(String),
    #[error("\u{6563}\u{70b9}\u{56fe}\u{7684} X \u{8f74}\u{5217}\u{5fc5}\u{987b}\u{662f}\u{6570}\u{503c}\u{5217}: {0}")]
    InvalidScatterXColumn(String),
    #[error("\u{56fe}\u{8868}\u{6570}\u{503c}\u{5217}\u{6ca1}\u{6709}\u{53ef}\u{7528}\u{6570}\u{503c}: {0}")]
    MissingNumericValues(String),
}

#[derive(Debug, Clone)]
struct ChartSeriesData {
    name: String,
    values: Vec<f64>,
}

#[derive(Debug, Clone, Copy)]
struct PlotArea {
    left: f64,
    top: f64,
    width: f64,
    height: f64,
}

const SERIES_COLORS: [&str; 6] = [
    "#2563eb",
    "#dc2626",
    "#16a34a",
    "#ea580c",
    "#7c3aed",
    "#0891b2",
];

pub fn render_chart_svg(draft: &PersistedChartDraft) -> Result<String, ChartSvgError> {
    if draft.width == 0 || draft.height == 0 {
        return Err(ChartSvgError::InvalidLayout);
    }

    let dataframe = draft
        .to_dataframe()
        .map_err(|error| ChartSvgError::RestoreChart(error.to_string()))?;
    if dataframe.height() == 0 {
        return Err(ChartSvgError::EmptyDataset);
    }

    let category_column = dataframe
        .column(&draft.category_column)
        .map_err(|_| ChartSvgError::MissingCategoryColumn(draft.category_column.clone()))?;
    let title = draft.title.as_deref().unwrap_or("Untitled Chart");
    let plot = build_plot_area(draft.width as f64, draft.height as f64, draft.chart_type.clone());

    let mut svg = String::new();
    write!(
        svg,
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="{width}" height="{height}" viewBox="0 0 {width} {height}" role="img" aria-label="{aria}">"##,
        width = draft.width,
        height = draft.height,
        aria = escape_xml(title),
    )
    .unwrap();
    svg.push_str(r##"<rect width="100%" height="100%" fill="#ffffff"/>"##);
    write!(
        svg,
        r##"<text x="{x}" y="28" text-anchor="middle" font-family="Segoe UI, Microsoft YaHei, sans-serif" font-size="18" font-weight="700" fill="#111827">{title}</text>"##,
        x = draft.width as f64 / 2.0,
        title = escape_xml(title),
    )
    .unwrap();

    match draft.chart_type {
        PersistedChartType::Column => {
            let categories = collect_text_values(category_column);
            let series = collect_numeric_series(draft, &dataframe)?;
            render_column_chart(&mut svg, draft, &plot, &categories, &series);
            if draft.show_legend {
                render_legend(&mut svg, draft, &series);
            }
        }
        PersistedChartType::Line => {
            let categories = collect_text_values(category_column);
            let series = collect_numeric_series(draft, &dataframe)?;
            render_line_chart(&mut svg, draft, &plot, &categories, &series);
            if draft.show_legend {
                render_legend(&mut svg, draft, &series);
            }
        }
        PersistedChartType::Pie => {
            let categories = collect_text_values(category_column);
            let series = collect_numeric_series(draft, &dataframe)?;
            render_pie_chart(&mut svg, draft, &categories, &series[0]);
        }
        PersistedChartType::Scatter => {
            let x_values = collect_numeric_column(category_column)
                .map_err(|_| ChartSvgError::InvalidScatterXColumn(draft.category_column.clone()))?;
            let series = collect_numeric_series(draft, &dataframe)?;
            render_scatter_chart(&mut svg, draft, &plot, &x_values, &series);
            if draft.show_legend {
                render_legend(&mut svg, draft, &series);
            }
        }
    }

    svg.push_str("</svg>");
    Ok(svg)
}

fn render_column_chart(
    svg: &mut String,
    draft: &PersistedChartDraft,
    plot: &PlotArea,
    categories: &[String],
    series: &[ChartSeriesData],
) {
    // 2026-03-23: Keep V1 column charts conservative: grouped bars, one shared axis, predictable export.
    render_axes(svg, draft, plot);
    let max_value = series
        .iter()
        .flat_map(|item| item.values.iter().copied())
        .fold(0.0_f64, f64::max)
        .max(1.0);
    render_y_ticks(svg, plot, max_value);

    let category_count = categories.len().max(1);
    let series_count = series.len().max(1);
    let group_width = plot.width / category_count as f64;
    let inner_gap = 8.0;
    let bar_width = ((group_width - inner_gap * 2.0) / series_count as f64).max(8.0);

    for (category_index, category) in categories.iter().enumerate() {
        let group_left = plot.left + group_width * category_index as f64;
        for (series_index, series_item) in series.iter().enumerate() {
            let value = series_item.values.get(category_index).copied().unwrap_or(0.0);
            let bar_height = plot.height * (value / max_value);
            let x = group_left + inner_gap + series_index as f64 * bar_width;
            let y = plot.top + plot.height - bar_height;
            write!(
                svg,
                r##"<rect x="{x:.2}" y="{y:.2}" width="{width:.2}" height="{height:.2}" rx="4" fill="{fill}"/>"##,
                width = bar_width * 0.86,
                height = bar_height.max(1.0),
                fill = SERIES_COLORS[series_index % SERIES_COLORS.len()],
            )
            .unwrap();
        }
        write!(
            svg,
            r##"<text x="{x:.2}" y="{y:.2}" text-anchor="middle" font-family="Segoe UI, Microsoft YaHei, sans-serif" font-size="11" fill="#374151">{label}</text>"##,
            x = group_left + group_width / 2.0,
            y = plot.top + plot.height + 20.0,
            label = escape_xml(category),
        )
        .unwrap();
    }
}

fn render_line_chart(
    svg: &mut String,
    draft: &PersistedChartDraft,
    plot: &PlotArea,
    categories: &[String],
    series: &[ChartSeriesData],
) {
    render_axes(svg, draft, plot);
    let max_value = series
        .iter()
        .flat_map(|item| item.values.iter().copied())
        .fold(0.0_f64, f64::max)
        .max(1.0);
    render_y_ticks(svg, plot, max_value);

    let step_x = if categories.len() <= 1 {
        0.0
    } else {
        plot.width / (categories.len() - 1) as f64
    };

    for (series_index, series_item) in series.iter().enumerate() {
        let mut points = String::new();
        for (index, value) in series_item.values.iter().enumerate() {
            let x = plot.left + step_x * index as f64;
            let y = plot.top + plot.height - plot.height * (*value / max_value);
            if !points.is_empty() {
                points.push(' ');
            }
            write!(points, "{x:.2},{y:.2}").unwrap();
        }
        write!(
            svg,
            r##"<polyline fill="none" stroke="{stroke}" stroke-width="3" points="{points}"/>"##,
            stroke = SERIES_COLORS[series_index % SERIES_COLORS.len()],
        )
        .unwrap();
        for (index, value) in series_item.values.iter().enumerate() {
            let x = plot.left + step_x * index as f64;
            let y = plot.top + plot.height - plot.height * (*value / max_value);
            write!(
                svg,
                r##"<circle cx="{x:.2}" cy="{y:.2}" r="4" fill="{fill}"/>"##,
                fill = SERIES_COLORS[series_index % SERIES_COLORS.len()],
            )
            .unwrap();
        }
    }

    for (index, category) in categories.iter().enumerate() {
        let x = plot.left + step_x * index as f64;
        write!(
            svg,
            r##"<text x="{x:.2}" y="{y:.2}" text-anchor="middle" font-family="Segoe UI, Microsoft YaHei, sans-serif" font-size="11" fill="#374151">{label}</text>"##,
            y = plot.top + plot.height + 20.0,
            label = escape_xml(category),
        )
        .unwrap();
    }
}

fn render_pie_chart(
    svg: &mut String,
    draft: &PersistedChartDraft,
    categories: &[String],
    series: &ChartSeriesData,
) {
    // 2026-03-23: Pie stays single-series in V1 to match build_chart validation and keep export behavior stable.
    let width = draft.width as f64;
    let height = draft.height as f64;
    let cx = width * 0.34;
    let cy = height * 0.56;
    let radius = width.min(height) * 0.24;
    let total = series.values.iter().sum::<f64>().max(1.0);
    let mut angle = -PI / 2.0;

    for (index, value) in series.values.iter().enumerate() {
        let sweep = (value / total) * PI * 2.0;
        let end_angle = angle + sweep;
        let start_x = cx + radius * angle.cos();
        let start_y = cy + radius * angle.sin();
        let end_x = cx + radius * end_angle.cos();
        let end_y = cy + radius * end_angle.sin();
        let large_arc = if sweep > PI { 1 } else { 0 };

        write!(
            svg,
            r##"<path d="M {cx:.2} {cy:.2} L {start_x:.2} {start_y:.2} A {radius:.2} {radius:.2} 0 {large_arc} 1 {end_x:.2} {end_y:.2} Z" fill="{fill}" stroke="#ffffff" stroke-width="1.5"/>"##,
            fill = SERIES_COLORS[index % SERIES_COLORS.len()],
        )
        .unwrap();

        let mid_angle = angle + sweep / 2.0;
        let label_x = cx + radius * 1.18 * mid_angle.cos();
        let label_y = cy + radius * 1.18 * mid_angle.sin();
        let percent = value / total * 100.0;
        write!(
            svg,
            r##"<text x="{x:.2}" y="{y:.2}" text-anchor="middle" font-family="Segoe UI, Microsoft YaHei, sans-serif" font-size="11" fill="#111827">{label} {percent:.1}%</text>"##,
            x = label_x,
            y = label_y,
            label = escape_xml(&categories[index]),
        )
        .unwrap();

        angle = end_angle;
    }

    if draft.show_legend {
        let legend_x = width * 0.68;
        let mut legend_y = height * 0.28;
        for (index, category) in categories.iter().enumerate() {
            write!(
                svg,
                r##"<rect x="{x:.2}" y="{y:.2}" width="12" height="12" rx="2" fill="{fill}"/>"##,
                x = legend_x,
                y = legend_y - 10.0,
                fill = SERIES_COLORS[index % SERIES_COLORS.len()],
            )
            .unwrap();
            write!(
                svg,
                r##"<text x="{x:.2}" y="{y:.2}" font-family="Segoe UI, Microsoft YaHei, sans-serif" font-size="12" fill="#374151">{label}</text>"##,
                x = legend_x + 18.0,
                y = legend_y,
                label = escape_xml(category),
            )
            .unwrap();
            legend_y += 22.0;
        }
    }
}

fn render_scatter_chart(
    svg: &mut String,
    draft: &PersistedChartDraft,
    plot: &PlotArea,
    x_values: &[f64],
    series: &[ChartSeriesData],
) {
    render_axes(svg, draft, plot);
    let min_x = x_values.iter().copied().fold(f64::INFINITY, f64::min);
    let raw_max_x = x_values.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let max_x = if (raw_max_x - min_x).abs() < f64::EPSILON {
        min_x + 1.0
    } else {
        raw_max_x
    };
    let min_y = series
        .iter()
        .flat_map(|item| item.values.iter().copied())
        .fold(f64::INFINITY, f64::min);
    let raw_max_y = series
        .iter()
        .flat_map(|item| item.values.iter().copied())
        .fold(f64::NEG_INFINITY, f64::max);
    let max_y = if (raw_max_y - min_y).abs() < f64::EPSILON {
        min_y + 1.0
    } else {
        raw_max_y
    };
    render_scatter_ticks(svg, plot, min_x, max_x, min_y, max_y);

    for (series_index, series_item) in series.iter().enumerate() {
        for (point_index, value) in series_item.values.iter().enumerate() {
            let x = scale_linear(x_values[point_index], min_x, max_x, plot.left, plot.left + plot.width);
            let y = scale_linear(*value, min_y, max_y, plot.top + plot.height, plot.top);
            write!(
                svg,
                r##"<circle cx="{x:.2}" cy="{y:.2}" r="4.5" fill="{fill}" fill-opacity="0.82"/>"##,
                fill = SERIES_COLORS[series_index % SERIES_COLORS.len()],
            )
            .unwrap();
        }
    }
}

fn render_axes(svg: &mut String, draft: &PersistedChartDraft, plot: &PlotArea) {
    write!(
        svg,
        r##"<line x1="{x1:.2}" y1="{y1:.2}" x2="{x2:.2}" y2="{y2:.2}" stroke="#9ca3af" stroke-width="1.2"/>"##,
        x1 = plot.left,
        y1 = plot.top + plot.height,
        x2 = plot.left + plot.width,
        y2 = plot.top + plot.height,
    )
    .unwrap();
    write!(
        svg,
        r##"<line x1="{x1:.2}" y1="{y1:.2}" x2="{x2:.2}" y2="{y2:.2}" stroke="#9ca3af" stroke-width="1.2"/>"##,
        x1 = plot.left,
        y1 = plot.top,
        x2 = plot.left,
        y2 = plot.top + plot.height,
    )
    .unwrap();

    let x_axis_name = draft.x_axis_name.as_deref().unwrap_or(&draft.category_column);
    let y_axis_name = draft
        .y_axis_name
        .as_deref()
        .unwrap_or(&draft.series[0].value_column);
    write!(
        svg,
        r##"<text x="{x:.2}" y="{y:.2}" text-anchor="middle" font-family="Segoe UI, Microsoft YaHei, sans-serif" font-size="12" fill="#4b5563">{label}</text>"##,
        x = plot.left + plot.width / 2.0,
        y = plot.top + plot.height + 40.0,
        label = escape_xml(x_axis_name),
    )
    .unwrap();
    write!(
        svg,
        r##"<text x="{x:.2}" y="{y:.2}" text-anchor="middle" transform="rotate(-90 {x:.2} {y:.2})" font-family="Segoe UI, Microsoft YaHei, sans-serif" font-size="12" fill="#4b5563">{label}</text>"##,
        x = plot.left - 42.0,
        y = plot.top + plot.height / 2.0,
        label = escape_xml(y_axis_name),
    )
    .unwrap();
}

fn render_y_ticks(svg: &mut String, plot: &PlotArea, max_value: f64) {
    for tick in 0..=4 {
        let ratio = tick as f64 / 4.0;
        let y = plot.top + plot.height - plot.height * ratio;
        let label = max_value * ratio;
        write!(
            svg,
            r##"<line x1="{x1:.2}" y1="{y:.2}" x2="{x2:.2}" y2="{y:.2}" stroke="#e5e7eb" stroke-width="1"/>"##,
            x1 = plot.left,
            x2 = plot.left + plot.width,
        )
        .unwrap();
        write!(
            svg,
            r##"<text x="{x:.2}" y="{text_y:.2}" text-anchor="end" font-family="Segoe UI, Microsoft YaHei, sans-serif" font-size="11" fill="#6b7280">{label:.1}</text>"##,
            x = plot.left - 8.0,
            text_y = y + 4.0,
        )
        .unwrap();
    }
}

fn render_scatter_ticks(
    svg: &mut String,
    plot: &PlotArea,
    min_x: f64,
    max_x: f64,
    min_y: f64,
    max_y: f64,
) {
    for tick in 0..=4 {
        let ratio = tick as f64 / 4.0;
        let x = plot.left + plot.width * ratio;
        let y = plot.top + plot.height - plot.height * ratio;
        let x_label = min_x + (max_x - min_x) * ratio;
        let y_label = min_y + (max_y - min_y) * ratio;
        write!(
            svg,
            r##"<line x1="{x:.2}" y1="{top:.2}" x2="{x:.2}" y2="{bottom:.2}" stroke="#f3f4f6" stroke-width="1"/>"##,
            top = plot.top,
            bottom = plot.top + plot.height,
        )
        .unwrap();
        write!(
            svg,
            r##"<line x1="{left:.2}" y1="{y:.2}" x2="{right:.2}" y2="{y:.2}" stroke="#f3f4f6" stroke-width="1"/>"##,
            left = plot.left,
            right = plot.left + plot.width,
        )
        .unwrap();
        write!(
            svg,
            r##"<text x="{x:.2}" y="{label_y:.2}" text-anchor="middle" font-family="Segoe UI, Microsoft YaHei, sans-serif" font-size="11" fill="#6b7280">{label:.1}</text>"##,
            label_y = plot.top + plot.height + 20.0,
            label = x_label,
        )
        .unwrap();
        write!(
            svg,
            r##"<text x="{x:.2}" y="{label_y:.2}" text-anchor="end" font-family="Segoe UI, Microsoft YaHei, sans-serif" font-size="11" fill="#6b7280">{label:.1}</text>"##,
            x = plot.left - 8.0,
            label_y = y + 4.0,
            label = y_label,
        )
        .unwrap();
    }
}

fn render_legend(svg: &mut String, draft: &PersistedChartDraft, series: &[ChartSeriesData]) {
    let base_x = draft.width as f64 - 160.0;
    let mut base_y = 54.0;
    for (index, series_item) in series.iter().enumerate() {
        write!(
            svg,
            r##"<rect x="{x:.2}" y="{y:.2}" width="12" height="12" rx="2" fill="{fill}"/>"##,
            x = base_x,
            y = base_y - 10.0,
            fill = SERIES_COLORS[index % SERIES_COLORS.len()],
        )
        .unwrap();
        write!(
            svg,
            r##"<text x="{x:.2}" y="{y:.2}" font-family="Segoe UI, Microsoft YaHei, sans-serif" font-size="12" fill="#374151">{label}</text>"##,
            x = base_x + 18.0,
            y = base_y,
            label = escape_xml(&series_item.name),
        )
        .unwrap();
        base_y += 20.0;
    }
}

fn build_plot_area(width: f64, height: f64, chart_type: PersistedChartType) -> PlotArea {
    if matches!(chart_type, PersistedChartType::Pie) {
        return PlotArea {
            left: width * 0.08,
            top: height * 0.16,
            width: width * 0.54,
            height: height * 0.68,
        };
    }
    PlotArea {
        left: 72.0,
        top: 64.0,
        width: (width - 104.0).max(120.0),
        height: (height - 138.0).max(80.0),
    }
}

fn collect_text_values(column: &Column) -> Vec<String> {
    (0..column.len())
        .map(|index| any_value_to_text(column.get(index).ok().as_ref()))
        .collect()
}

fn collect_numeric_series(
    draft: &PersistedChartDraft,
    dataframe: &DataFrame,
) -> Result<Vec<ChartSeriesData>, ChartSvgError> {
    let mut output = Vec::with_capacity(draft.series.len());
    for item in &draft.series {
        let column = dataframe
            .column(&item.value_column)
            .map_err(|_| ChartSvgError::MissingValueColumn(item.value_column.clone()))?;
        let values = collect_numeric_column(column)
            .map_err(|_| ChartSvgError::MissingNumericValues(item.value_column.clone()))?;
        output.push(ChartSeriesData {
            name: item.name.clone().unwrap_or_else(|| item.value_column.clone()),
            values,
        });
    }
    Ok(output)
}

fn collect_numeric_column(column: &Column) -> Result<Vec<f64>, ()> {
    let mut values = Vec::with_capacity(column.len());
    let mut saw_numeric = false;
    for index in 0..column.len() {
        let value = column.get(index).map_err(|_| ())?;
        match any_value_to_f64(&value) {
            Some(number) => {
                values.push(number);
                saw_numeric = true;
            }
            None => values.push(0.0),
        }
    }
    if !saw_numeric {
        return Err(());
    }
    Ok(values)
}

fn any_value_to_f64(value: &AnyValue<'_>) -> Option<f64> {
    match value {
        AnyValue::Int8(value) => Some(*value as f64),
        AnyValue::Int16(value) => Some(*value as f64),
        AnyValue::Int32(value) => Some(*value as f64),
        AnyValue::Int64(value) => Some(*value as f64),
        AnyValue::UInt8(value) => Some(*value as f64),
        AnyValue::UInt16(value) => Some(*value as f64),
        AnyValue::UInt32(value) => Some(*value as f64),
        AnyValue::UInt64(value) => Some(*value as f64),
        AnyValue::Float32(value) => Some(*value as f64),
        AnyValue::Float64(value) => Some(*value),
        AnyValue::String(value) => value.trim().parse::<f64>().ok(),
        AnyValue::StringOwned(value) => value.as_str().trim().parse::<f64>().ok(),
        AnyValue::Null => None,
        _ => None,
    }
}

fn any_value_to_text(value: Option<&AnyValue<'_>>) -> String {
    match value {
        Some(AnyValue::String(value)) => value.to_string(),
        Some(AnyValue::StringOwned(value)) => value.as_str().to_string(),
        Some(AnyValue::Int8(value)) => value.to_string(),
        Some(AnyValue::Int16(value)) => value.to_string(),
        Some(AnyValue::Int32(value)) => value.to_string(),
        Some(AnyValue::Int64(value)) => value.to_string(),
        Some(AnyValue::UInt8(value)) => value.to_string(),
        Some(AnyValue::UInt16(value)) => value.to_string(),
        Some(AnyValue::UInt32(value)) => value.to_string(),
        Some(AnyValue::UInt64(value)) => value.to_string(),
        Some(AnyValue::Float32(value)) => format!("{value:.2}"),
        Some(AnyValue::Float64(value)) => format!("{value:.2}"),
        Some(AnyValue::Null) | None => String::new(),
        Some(other) => other.to_string(),
    }
}

fn scale_linear(value: f64, min: f64, max: f64, start: f64, end: f64) -> f64 {
    if (max - min).abs() < f64::EPSILON {
        return (start + end) / 2.0;
    }
    start + (value - min) / (max - min) * (end - start)
}

fn escape_xml(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}
