use polars::prelude::{DataFrame, NamedFrom, Series};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::frame::chart_ref_store::{PersistedChartDraft, PersistedChartType};
use crate::frame::workbook_ref_store::{
    PersistedWorkbookChartSpec, PersistedWorkbookChartType, PersistedWorkbookDraft,
    PersistedWorkbookLegendPosition, PersistedWorkbookSheetExportOptions,
    PersistedWorkbookSheetKind, WorkbookRefStoreError, WorkbookSheetInput,
};

#[derive(Debug)]
pub struct ReportDeliverySection {
    pub sheet_name: String,
    pub source_refs: Vec<String>,
    pub dataframe: DataFrame,
    // 2026-03-24: 这里补充交付格式意图，原因是 report_delivery 段内 format 已经可以描述交付偏好；目的是让 workbook 草稿在导出前就冻结这些规则。
    pub export_options: Option<PersistedWorkbookSheetExportOptions>,
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
    // 2026-03-24: 杩欓噷鎶?chart_ref 鑽夌妗ユ帴鎴?report_delivery 鍥捐〃瀵硅薄锛屽師鍥犳槸鏂规 A 瑕佺粺涓€鐙珛鍥捐〃涓?workbook 浜や粯鐨勮鏍艰〃杈撅紱鐩殑鏄 dispatcher 鍙仛瑁呴厤锛屼笉鍐嶅唴宓屽瓧娈垫槧灏勭粏鑺傘€?
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
    #[error("report_delivery 鐨?report_name 涓嶈兘涓虹┖")]
    EmptyReportName,
    #[error("report_delivery 鐨勫浘琛ㄩ〉鍚嶇О涓嶈兘涓虹┖")]
    EmptyChartSheetName,
    #[error("report_delivery 在关闭图表页时不能同时请求真实图表")]
    ChartsRequireChartSheet,
    #[error("report_delivery 鐨勫浘琛ㄥ瓧娈典笉鑳戒负绌? {0}")]
    EmptyChartColumn(String),
    #[error("report_delivery 的图表至少需要一个数值系列")]
    EmptyChartSeries,
    #[error("report_delivery 的 pie 图暂时只支持一个数值系列")]
    PieChartRequiresSingleSeries,
    #[error("report_delivery 鏃犳硶鏋勫缓 workbook 鑽夌: {0}")]
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

    // 2026-03-23: 杩欓噷鍏堟妸鎽樿椤靛拰鍒嗘瀽椤靛缁堝浐鍖栬繘 workbook 鑽夌锛屽師鍥犳槸缁撴灉浜や粯妯℃澘鐨勯鏋惰绋冲畾锛涚洰鐨勬槸璁╀笂灞傚彧鎵╁睍鍥捐〃鍏冩暟鎹紝鑰屼笉鍙嶅鏀瑰 Sheet 澹冲眰銆?
    let mut worksheets = vec![
        WorkbookSheetInput {
            sheet_name: request.summary.sheet_name,
            source_refs: request.summary.source_refs,
            dataframe: request.summary.dataframe,
            // 2026-03-24: 这里把摘要页明确标成数据页，原因是后续导出质量规则要对数据页生效；目的是让冻结首列、筛选等行为不再依赖页名猜测。
            sheet_kind: PersistedWorkbookSheetKind::DataSheet,
            export_options: request.summary.export_options,
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
            // 2026-03-24: 这里把分析页明确标成数据页，原因是分析结果通常仍是宽表浏览场景；目的是让数据页交付规则在 report_delivery 中自动落地。
            sheet_kind: PersistedWorkbookSheetKind::DataSheet,
            export_options: request.analysis.export_options,
            title: Some(request.report_name.clone()),
            subtitle: report_subtitle
                .clone()
                .or_else(|| Some("分析结果页".to_string())),
            data_start_row: 2,
        },
    ];

    if request.include_chart_sheet {
        // 2026-03-23: 杩欓噷淇濈暀鍥捐〃椤电殑鏁版嵁鍗犱綅琛紝鍘熷洜鏄浘琛ㄥ鍑哄墠浠嶉渶瑕佷竴涓ǔ瀹氱殑鐩爣 worksheet锛涚洰鐨勬槸璁╃湡瀹炲浘琛ㄥ啓鍏ヤ笌鏃犲浘琛ㄦā鏉垮叡鐢ㄥ悓涓€椤电鍏ュ彛銆?
        worksheets.push(WorkbookSheetInput {
            sheet_name: chart_sheet_name.clone(),
            source_refs: vec!["report_delivery#chart_sheet".to_string()],
            dataframe: build_chart_sheet_dataframe(&request.report_name, request.charts.len()),
            // 2026-03-24: 这里把图表页显式标成 chart_sheet，原因是图表页不该机械套用数据页的首列冻结规则；目的是给后续页类型差异化布局留出稳定抓手。
            sheet_kind: PersistedWorkbookSheetKind::ChartSheet,
            export_options: None,
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

    // 2026-03-23: 杩欓噷鎶婂鍥鹃粯璁ゅ竷灞€鎴愪袱鍒楃綉鏍硷紝鍘熷洜鏄崟鍒楃珫鎺掍細璁╃浜屽紶鍥惧緢蹇鎶樺埌瑙嗛噹澶栵紱鐩殑鏄粰绗竴鐗堜氦浠樹竴涓彲璇荤殑榛樿甯冨眬銆?
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
