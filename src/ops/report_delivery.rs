use polars::prelude::{DataFrame, NamedFrom, Series};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::frame::chart_ref_store::{PersistedChartDraft, PersistedChartType};
use crate::frame::workbook_ref_store::{
    PersistedWorkbookChartSpec, PersistedWorkbookChartType, PersistedWorkbookDraft,
    PersistedWorkbookLegendPosition, WorkbookRefStoreError, WorkbookSheetInput,
};

#[derive(Debug)]
pub struct ReportDeliverySection {
    pub sheet_name: String,
    pub source_refs: Vec<String>,
    pub dataframe: DataFrame,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReportDeliveryChartType {
    Column,
    Line,
    Pie,
    Scatter,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReportDeliveryLegendPosition {
    Top,
    Bottom,
    Left,
    Right,
    TopRight,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReportDeliveryChartSeries {
    pub value_column: String,
    #[serde(default)]
    pub name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReportDeliveryChart {
    #[serde(default)]
    pub chart_ref: Option<String>,
    #[serde(default)]
    pub source_refs: Vec<String>,
    pub chart_type: ReportDeliveryChartType,
    #[serde(default)]
    pub title: Option<String>,
    pub category_column: String,
    #[serde(default)]
    pub value_column: String,
    #[serde(default)]
    pub series: Vec<ReportDeliveryChartSeries>,
    #[serde(default)]
    pub show_legend: bool,
    #[serde(default)]
    pub legend_position: Option<ReportDeliveryLegendPosition>,
    #[serde(default)]
    pub chart_style: Option<u8>,
    #[serde(default)]
    pub x_axis_name: Option<String>,
    #[serde(default)]
    pub y_axis_name: Option<String>,
    #[serde(default)]
    pub anchor_row: Option<u32>,
    #[serde(default)]
    pub anchor_col: Option<u16>,
}

pub fn chart_ref_to_report_delivery_chart(draft: &PersistedChartDraft) -> ReportDeliveryChart {
    // 2026-03-24: 这里把 chart_ref 草稿桥接成 report_delivery 图表对象，原因是方案 A 要统一独立图表与 workbook 交付的规格表达；目的是让 dispatcher 只做装配，不再内嵌字段映射细节。
    ReportDeliveryChart {
        chart_ref: Some(draft.chart_ref.clone()),
        source_refs: draft.source_refs.clone(),
        chart_type: map_chart_type_from_draft(&draft.chart_type),
        title: draft.title.clone(),
        category_column: draft.category_column.clone(),
        value_column: draft
            .series
            .first()
            .map(|item| item.value_column.clone())
            .unwrap_or_default(),
        series: draft
            .series
            .iter()
            .map(|item| ReportDeliveryChartSeries {
                value_column: item.value_column.clone(),
                name: item.name.clone(),
            })
            .collect(),
        show_legend: draft.show_legend,
        legend_position: None,
        chart_style: None,
        x_axis_name: draft.x_axis_name.clone(),
        y_axis_name: draft.y_axis_name.clone(),
        anchor_row: None,
        anchor_col: None,
    }
}

#[derive(Debug)]
pub struct ReportDeliveryRequest {
    pub report_name: String,
    pub report_subtitle: Option<String>,
    pub summary: ReportDeliverySection,
    pub analysis: ReportDeliverySection,
    pub include_chart_sheet: bool,
    pub chart_sheet_name: String,
    pub charts: Vec<ReportDeliveryChart>,
}

#[derive(Debug, Error)]
pub enum ReportDeliveryError {
    #[error("report_delivery 的 report_name 不能为空")]
    EmptyReportName,
    #[error("report_delivery 的图表页名称不能为空")]
    EmptyChartSheetName,
    #[error("report_delivery 在关闭图表页时不能同时请求真实图表")]
    ChartsRequireChartSheet,
    #[error("report_delivery 的图表字段不能为空: {0}")]
    EmptyChartColumn(String),
    #[error("report_delivery 的图表至少需要一个数值系列")]
    EmptyChartSeries,
    #[error("report_delivery 的 pie 图暂时只支持一个数值系列")]
    PieChartRequiresSingleSeries,
    #[error("report_delivery 无法构建 workbook 草稿: {0}")]
    BuildWorkbook(#[from] WorkbookRefStoreError),
}

pub fn build_report_delivery_draft(
    workbook_ref: &str,
    request: ReportDeliveryRequest,
) -> Result<PersistedWorkbookDraft, ReportDeliveryError> {
    if request.report_name.trim().is_empty() {
        return Err(ReportDeliveryError::EmptyReportName);
    }
    if request.include_chart_sheet && request.chart_sheet_name.trim().is_empty() {
        return Err(ReportDeliveryError::EmptyChartSheetName);
    }
    if !request.include_chart_sheet && !request.charts.is_empty() {
        return Err(ReportDeliveryError::ChartsRequireChartSheet);
    }
    validate_chart_requests(&request.charts)?;

    let analysis_sheet_name = request.analysis.sheet_name.clone();
    let chart_sheet_name = request.chart_sheet_name.clone();
    let report_subtitle = request.report_subtitle.clone();

    // 2026-03-23: 这里先把摘要页和分析页始终固化进 workbook 草稿，原因是结果交付模板的骨架要稳定；目的是让上层只扩展图表元数据，而不反复改多 Sheet 壳层。
    let mut worksheets = vec![
        WorkbookSheetInput {
            sheet_name: request.summary.sheet_name,
            source_refs: request.summary.source_refs,
            dataframe: request.summary.dataframe,
            title: Some(request.report_name.clone()),
            subtitle: report_subtitle
                .clone()
                .or_else(|| Some("摘要页".to_string())),
            data_start_row: 2,
        },
        WorkbookSheetInput {
            sheet_name: request.analysis.sheet_name,
            source_refs: request.analysis.source_refs,
            dataframe: request.analysis.dataframe,
            title: Some(request.report_name.clone()),
            subtitle: report_subtitle
                .clone()
                .or_else(|| Some("分析结果页".to_string())),
            data_start_row: 2,
        },
    ];

    if request.include_chart_sheet {
        // 2026-03-23: 这里保留图表页的数据占位表，原因是图表导出前仍需要一个稳定的目标 worksheet；目的是让真实图表写入与无图表模板共用同一页签入口。
        worksheets.push(WorkbookSheetInput {
            sheet_name: chart_sheet_name.clone(),
            source_refs: vec!["report_delivery#chart_sheet".to_string()],
            dataframe: build_chart_sheet_dataframe(&request.report_name, request.charts.len()),
            title: None,
            subtitle: None,
            data_start_row: 0,
        });
    }

    let charts = request
        .charts
        .into_iter()
        .enumerate()
        .map(|(index, chart)| {
            let series = normalize_chart_series(&chart);
            let (anchor_row, anchor_col) = resolve_chart_anchor(index, &chart);
            PersistedWorkbookChartSpec {
                chart_ref: chart.chart_ref,
                source_refs: chart.source_refs,
                chart_type: map_chart_type(chart.chart_type),
                target_sheet_name: chart_sheet_name.clone(),
                data_sheet_name: analysis_sheet_name.clone(),
                category_column: chart.category_column,
                value_column: series[0].value_column.clone(),
                title: chart.title,
                series,
                show_legend: chart.show_legend,
                legend_position: chart.legend_position.map(map_legend_position),
                chart_style: chart.chart_style,
                x_axis_name: chart.x_axis_name,
                y_axis_name: chart.y_axis_name,
                anchor_row,
                anchor_col,
            }
        })
        .collect::<Vec<_>>();

    PersistedWorkbookDraft::from_sheet_inputs_with_charts(workbook_ref, worksheets, charts)
        .map_err(Into::into)
}

fn validate_chart_requests(charts: &[ReportDeliveryChart]) -> Result<(), ReportDeliveryError> {
    for chart in charts {
        if chart.category_column.trim().is_empty() {
            return Err(ReportDeliveryError::EmptyChartColumn(
                "category_column".to_string(),
            ));
        }
        let series = normalize_chart_series(chart);
        if series.is_empty() {
            return Err(ReportDeliveryError::EmptyChartSeries);
        }
        if chart.chart_type == ReportDeliveryChartType::Pie && series.len() != 1 {
            return Err(ReportDeliveryError::PieChartRequiresSingleSeries);
        }
        for item in series {
            if item.value_column.trim().is_empty() {
                return Err(ReportDeliveryError::EmptyChartColumn(
                    "value_column".to_string(),
                ));
            }
        }
    }
    Ok(())
}

fn normalize_chart_series(
    chart: &ReportDeliveryChart,
) -> Vec<crate::frame::workbook_ref_store::PersistedWorkbookChartSeriesSpec> {
    if !chart.series.is_empty() {
        return chart
            .series
            .iter()
            .map(
                |series| crate::frame::workbook_ref_store::PersistedWorkbookChartSeriesSpec {
                    value_column: series.value_column.clone(),
                    name: series.name.clone(),
                },
            )
            .collect();
    }

    if chart.value_column.trim().is_empty() {
        return vec![];
    }

    vec![
        crate::frame::workbook_ref_store::PersistedWorkbookChartSeriesSpec {
            value_column: chart.value_column.clone(),
            name: None,
        },
    ]
}

fn resolve_chart_anchor(index: usize, chart: &ReportDeliveryChart) -> (u32, u16) {
    if chart.anchor_row.is_some() || chart.anchor_col.is_some() {
        return (chart.anchor_row.unwrap_or(1), chart.anchor_col.unwrap_or(0));
    }

    // 2026-03-23: 这里把多图默认布局成两列网格，原因是单列竖排会让第二张图很快被折到视野外；目的是给第一版交付一个可读的默认布局。
    let grid_row = (index / 2) as u32;
    let grid_col = (index % 2) as u16;
    (1 + grid_row * 16, grid_col * 8)
}

fn map_chart_type(chart_type: ReportDeliveryChartType) -> PersistedWorkbookChartType {
    match chart_type {
        ReportDeliveryChartType::Column => PersistedWorkbookChartType::Column,
        ReportDeliveryChartType::Line => PersistedWorkbookChartType::Line,
        ReportDeliveryChartType::Pie => PersistedWorkbookChartType::Pie,
        ReportDeliveryChartType::Scatter => PersistedWorkbookChartType::Scatter,
    }
}

fn map_chart_type_from_draft(chart_type: &PersistedChartType) -> ReportDeliveryChartType {
    match chart_type {
        PersistedChartType::Column => ReportDeliveryChartType::Column,
        PersistedChartType::Line => ReportDeliveryChartType::Line,
        PersistedChartType::Pie => ReportDeliveryChartType::Pie,
        PersistedChartType::Scatter => ReportDeliveryChartType::Scatter,
    }
}

fn map_legend_position(position: ReportDeliveryLegendPosition) -> PersistedWorkbookLegendPosition {
    match position {
        ReportDeliveryLegendPosition::Top => PersistedWorkbookLegendPosition::Top,
        ReportDeliveryLegendPosition::Bottom => PersistedWorkbookLegendPosition::Bottom,
        ReportDeliveryLegendPosition::Left => PersistedWorkbookLegendPosition::Left,
        ReportDeliveryLegendPosition::Right => PersistedWorkbookLegendPosition::Right,
        ReportDeliveryLegendPosition::TopRight => PersistedWorkbookLegendPosition::TopRight,
    }
}

fn build_chart_sheet_dataframe(report_name: &str, chart_count: usize) -> DataFrame {
    let status = if chart_count == 0 {
        "待接入"
    } else {
        "已生成图表位"
    };
    let note = if chart_count == 0 {
        format!("{report_name} 当前保留图表页模板，后续可继续追加真实图表")
    } else {
        format!("{report_name} 已写入 {chart_count} 张图表，请在图表页查看")
    };

    DataFrame::new(vec![
        Series::new("模块".into(), ["图表页"]).into(),
        Series::new("状态".into(), [status]).into(),
        Series::new("说明".into(), [note]).into(),
    ])
    .expect("report delivery chart sheet dataframe should build")
}
