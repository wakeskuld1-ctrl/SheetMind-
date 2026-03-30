use polars::prelude::{DataFrame, NamedFrom, Series};
use serde::{Deserialize, Serialize};

use crate::frame::loader::LoadedTable;
use crate::frame::workbook_ref_store::{
    PersistedWorkbookChartSeriesSpec, PersistedWorkbookChartSpec, PersistedWorkbookChartType,
    PersistedWorkbookLegendPosition, PersistedWorkbookSheetKind, WorkbookDraftStore,
    WorkbookSheetInput,
};
use crate::ops::diagnostics_report::{
    DiagnosticsReportRequest, DiagnosticsReportResult, diagnostics_report,
};
use crate::ops::export::export_excel_workbook;

// 2026-03-29 00:08 CST：这里定义组合诊断 Excel 报表请求，原因是这轮要把“组合诊断参数 + workbook 交付参数”收口成正式高层 Tool；
// 目的：让调用方一次请求就能完成 diagnostics_report 与 workbook/xlsx 交付，而不是手工拼装多层调用。
#[derive(Debug, Clone, Deserialize)]
pub struct DiagnosticsReportExcelReportRequest {
    pub report_name: String,
    #[serde(default)]
    pub report_subtitle: Option<String>,
    #[serde(default = "default_summary_sheet_name")]
    pub summary_sheet_name: String,
    #[serde(default = "default_overview_sheet_name")]
    pub overview_sheet_name: String,
    #[serde(default = "default_detail_sheet_name")]
    pub detail_sheet_name: String,
    #[serde(default = "default_trend_sheet_name")]
    pub trend_sheet_name: String,
    #[serde(default = "default_chart_sheet_name")]
    pub chart_sheet_name: String,
    #[serde(default = "default_include_chart_sheet")]
    pub include_chart_sheet: bool,
    #[serde(default)]
    pub output_path: Option<String>,
    #[serde(flatten)]
    pub diagnostics_request: DiagnosticsReportRequest,
}

// 2026-03-29 00:08 CST：这里定义高层 Excel 交付结果，原因是上层既要消费 workbook 交付句柄，也要继续拿到组合诊断结果；
// 目的：把“业务结果 + 交付结果”放在同一个响应里，减少后续 AI 或 CLI 的二次查询与状态拼接。
#[derive(Debug, Clone, Serialize)]
pub struct DiagnosticsReportExcelReportResult {
    pub diagnostics_result: DiagnosticsReportResult,
    pub workbook_ref: String,
    pub sheet_names: Vec<String>,
    pub format: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_path: Option<String>,
}

fn default_summary_sheet_name() -> String {
    "执行摘要".to_string()
}

fn default_overview_sheet_name() -> String {
    "诊断概览".to_string()
}

fn default_detail_sheet_name() -> String {
    "相关性与异常".to_string()
}

fn default_trend_sheet_name() -> String {
    "分布与趋势".to_string()
}

fn default_chart_sheet_name() -> String {
    "图表摘要".to_string()
}

fn default_include_chart_sheet() -> bool {
    true
}

// 2026-03-29 00:08 CST：这里提供高层 workbook 交付主入口，原因是 diagnostics_report 已经具备统一 JSON 合同，下一步自然要形成 Rust 原生 Excel 交付；
// 目的：继续沿现有 workbook draft 与 export 主线扩展，而不是重开新的交付架构。
pub fn diagnostics_report_excel_report(
    loaded: &LoadedTable,
    request: &DiagnosticsReportExcelReportRequest,
) -> Result<DiagnosticsReportExcelReportResult, String> {
    if request.report_name.trim().is_empty() {
        return Err("diagnostics_report_excel_report 缺少 report_name".to_string());
    }
    if matches!(request.output_path.as_deref(), Some(path) if path.trim().is_empty()) {
        return Err("diagnostics_report_excel_report 的 output_path 不能为空".to_string());
    }

    let diagnostics_result = diagnostics_report(loaded, &request.diagnostics_request)
        .map_err(|error| error.to_string())?;

    let workbook_ref = WorkbookDraftStore::create_workbook_ref();
    let source_refs = source_refs_for_delivery(loaded);
    let mut sheet_names = vec![
        request.summary_sheet_name.clone(),
        request.overview_sheet_name.clone(),
        request.detail_sheet_name.clone(),
        request.trend_sheet_name.clone(),
    ];
    let mut worksheets = vec![
        WorkbookSheetInput {
            sheet_name: request.summary_sheet_name.clone(),
            source_refs: source_refs.clone(),
            dataframe: build_summary_dataframe(&diagnostics_result, request),
            // 2026-03-29 00:08 CST：这里把执行摘要页标成数据页，原因是第一版虽然是管理摘要，但仍要保留稳定的表格导出行为；
            // 目的：先保证 workbook 交付稳定，再把图表和更复杂样式留给后续切片。
            sheet_kind: PersistedWorkbookSheetKind::DataSheet,
            export_options: None,
            title: Some(request.report_name.clone()),
            subtitle: Some(resolve_sheet_subtitle(
                request.report_subtitle.as_deref(),
                "组合诊断执行摘要",
            )),
            data_start_row: 2,
        },
        WorkbookSheetInput {
            sheet_name: request.overview_sheet_name.clone(),
            source_refs: source_refs.clone(),
            dataframe: build_overview_dataframe(&diagnostics_result),
            // 2026-03-29 00:08 CST：这里单独落诊断概览页，原因是业务方需要一眼看到各 section 的可用状态；
            // 目的：把降级信息显式留在 workbook 里，避免只能从 JSON warnings 阅读。
            sheet_kind: PersistedWorkbookSheetKind::DataSheet,
            export_options: None,
            title: Some(request.report_name.clone()),
            subtitle: Some("各诊断分段状态概览".to_string()),
            data_start_row: 2,
        },
        WorkbookSheetInput {
            sheet_name: request.detail_sheet_name.clone(),
            source_refs: source_refs.clone(),
            dataframe: build_correlation_outlier_dataframe(&diagnostics_result),
            // 2026-03-29 00:08 CST：这里把相关性与异常集中成一页，原因是这两块更接近“问题结构”视角；
            // 目的：让第一版 workbook 至少能稳定交付最关键的结构化观察，而不要求调用方继续自己拼表。
            sheet_kind: PersistedWorkbookSheetKind::DataSheet,
            export_options: None,
            title: Some(request.report_name.clone()),
            subtitle: Some("相关性与异常值观察".to_string()),
            data_start_row: 2,
        },
        WorkbookSheetInput {
            sheet_name: request.trend_sheet_name.clone(),
            source_refs: source_refs.clone(),
            dataframe: build_distribution_trend_dataframe(&diagnostics_result),
            // 2026-03-29 00:08 CST：这里把分布与趋势集中成一页，原因是这两块更接近“形态与变化”视角；
            // 目的：用最小改动把统计诊断结果转成管理层可读的 workbook 页面。
            sheet_kind: PersistedWorkbookSheetKind::DataSheet,
            export_options: None,
            title: Some(request.report_name.clone()),
            subtitle: Some("分布与趋势观察".to_string()),
            data_start_row: 2,
        },
    ];

    let charts = if request.include_chart_sheet {
        // 2026-03-29 00:49 CST：这里把图表页做成“数据源页 + 图表承载页”，原因是现有 workbook chart 导出要求图表引用正式 sheet；
        // 目的：用最小改动把相关性、异常占比和趋势曲线接进现有导出主线，而不是额外再造隐藏 sheet 架构。
        let chart_dataframe = build_chart_sheet_dataframe(&diagnostics_result);
        let chart_specs = build_chart_specs(&diagnostics_result, &request.chart_sheet_name);
        sheet_names.push(request.chart_sheet_name.clone());
        worksheets.push(WorkbookSheetInput {
            sheet_name: request.chart_sheet_name.clone(),
            source_refs,
            dataframe: chart_dataframe,
            sheet_kind: PersistedWorkbookSheetKind::ChartSheet,
            export_options: None,
            title: Some(request.report_name.clone()),
            subtitle: Some("图表摘要与管理观察".to_string()),
            data_start_row: 0,
        });
        chart_specs
    } else {
        vec![]
    };

    let draft =
        crate::frame::workbook_ref_store::PersistedWorkbookDraft::from_sheet_inputs_with_charts(
            &workbook_ref,
            worksheets,
            charts,
        )
        .map_err(|error| error.to_string())?;
    let store = WorkbookDraftStore::workspace_default().map_err(|error| error.to_string())?;
    store.save(&draft).map_err(|error| error.to_string())?;

    if let Some(output_path) = request.output_path.as_deref() {
        export_excel_workbook(&draft, output_path).map_err(|error| error.to_string())?;
    }

    Ok(DiagnosticsReportExcelReportResult {
        diagnostics_result,
        workbook_ref,
        sheet_names,
        format: if request.output_path.is_some() {
            "xlsx".to_string()
        } else {
            "workbook_ref".to_string()
        },
        output_path: request.output_path.clone(),
    })
}

fn source_refs_for_delivery(loaded: &LoadedTable) -> Vec<String> {
    vec![
        format!(
            "{}#{}",
            loaded.handle.source_path(),
            loaded.handle.sheet_name()
        ),
        "diagnostics_report_excel_report".to_string(),
    ]
}

fn resolve_sheet_subtitle(report_subtitle: Option<&str>, fallback: &str) -> String {
    report_subtitle
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(fallback)
        .to_string()
}

fn build_summary_dataframe(
    result: &DiagnosticsReportResult,
    request: &DiagnosticsReportExcelReportRequest,
) -> DataFrame {
    let risk_level = resolve_risk_level(result);
    let decision_readiness = resolve_decision_readiness(result);
    let priority_direction = result
        .recommended_actions
        .first()
        .cloned()
        .unwrap_or_else(|| "建议先检查字段配置与数据质量后再继续诊断。".to_string());
    // 2026-03-28 22:08 CST：这里补分析承接字段，原因是当前执行摘要已经能表达“结果如何”，但还不能稳定表达“下一步怎么接”；
    // 目的：继续沿现有 workbook 交付层增强，把复核、补数、建模建议直接写进摘要页，方便 AI 或人工顺着当前结论继续推进。
    let review_recommendation = resolve_review_recommendation(result);
    let data_completion_recommendation = resolve_data_completion_recommendation(result, request);
    let modeling_recommendation = resolve_modeling_recommendation(result, request);
    let suggested_tool = resolve_suggested_tool(result, request);
    let suggested_target_field = resolve_suggested_target_field(request);
    let suggested_time_field = resolve_suggested_time_field(request);
    let main_blocker = resolve_main_blocker(result, request);
    let next_stage_condition = resolve_next_stage_condition(result, request);
    let mut item = vec![
        "report_name".to_string(),
        "report_status".to_string(),
        "总体风险等级".to_string(),
        "可直接决策".to_string(),
        "优先处理方向".to_string(),
        "复核建议".to_string(),
        "补数建议".to_string(),
        "建模建议".to_string(),
        "建议优先工具".to_string(),
        "建议目标字段".to_string(),
        "建议时间字段".to_string(),
        "当前主要阻塞项".to_string(),
        "进入下一步前需满足条件".to_string(),
        "row_count".to_string(),
        "section_count".to_string(),
        "available_section_count".to_string(),
        "决策结论".to_string(),
    ];
    let mut value = vec![
        result.report_name.clone(),
        result.report_status.clone(),
        risk_level,
        decision_readiness,
        priority_direction,
        review_recommendation,
        data_completion_recommendation,
        modeling_recommendation,
        suggested_tool,
        suggested_target_field,
        suggested_time_field,
        main_blocker,
        next_stage_condition,
        result.row_count.to_string(),
        result.section_count.to_string(),
        result.available_section_count.to_string(),
        result.overall_summary.clone(),
    ];

    for finding in &result.key_findings {
        item.push("关键发现".to_string());
        value.push(finding.clone());
    }
    for action in &result.recommended_actions {
        item.push("建议动作".to_string());
        value.push(action.clone());
    }
    for warning in &result.warnings {
        item.push("降级提醒".to_string());
        value.push(warning.clone());
    }

    DataFrame::new(vec![
        Series::new("item".into(), item).into(),
        Series::new("value".into(), value).into(),
    ])
    .expect("summary dataframe should be valid")
}

fn build_overview_dataframe(result: &DiagnosticsReportResult) -> DataFrame {
    let keys = result
        .sections
        .iter()
        .map(|section| section.key.clone())
        .collect::<Vec<_>>();
    let titles = result
        .sections
        .iter()
        .map(|section| section.title.clone())
        .collect::<Vec<_>>();
    let statuses = result
        .sections
        .iter()
        .map(|section| section.status.clone())
        .collect::<Vec<_>>();
    let summaries = result
        .sections
        .iter()
        .map(|section| section.summary.clone())
        .collect::<Vec<_>>();

    DataFrame::new(vec![
        Series::new("section_key".into(), keys).into(),
        Series::new("section_title".into(), titles).into(),
        Series::new("section_status".into(), statuses).into(),
        Series::new("section_summary".into(), summaries).into(),
    ])
    .expect("overview dataframe should be valid")
}

fn build_correlation_outlier_dataframe(result: &DiagnosticsReportResult) -> DataFrame {
    let mut section = Vec::new();
    let mut metric = Vec::new();
    let mut value = Vec::new();

    if let Some(correlation) = result.correlation_section.as_ref() {
        for item in &correlation.top_positive {
            section.push("correlation".to_string());
            metric.push(format!("top_positive:{}", item.feature_column));
            value.push(format!("{:.4}", item.coefficient));
        }
        for item in &correlation.top_negative {
            section.push("correlation".to_string());
            metric.push(format!("top_negative:{}", item.feature_column));
            value.push(format!("{:.4}", item.coefficient));
        }
        if correlation.top_positive.is_empty() && correlation.top_negative.is_empty() {
            section.push("correlation".to_string());
            metric.push("summary".to_string());
            value.push(correlation.human_summary.overall.clone());
        }
    } else {
        section.push("correlation".to_string());
        metric.push("status".to_string());
        value.push("unavailable".to_string());
    }

    if let Some(outlier) = result.outlier_section.as_ref() {
        for summary in &outlier.outlier_summaries {
            section.push("outlier".to_string());
            metric.push(format!("{}:outlier_ratio", summary.column));
            value.push(format!("{:.2}%", summary.outlier_ratio * 100.0));
        }
        if outlier.outlier_summaries.is_empty() {
            section.push("outlier".to_string());
            metric.push("summary".to_string());
            value.push(outlier.human_summary.overall.clone());
        }
    } else {
        section.push("outlier".to_string());
        metric.push("status".to_string());
        value.push("unavailable".to_string());
    }

    DataFrame::new(vec![
        Series::new("section".into(), section).into(),
        Series::new("metric".into(), metric).into(),
        Series::new("value".into(), value).into(),
    ])
    .expect("correlation and outlier dataframe should be valid")
}

fn build_distribution_trend_dataframe(result: &DiagnosticsReportResult) -> DataFrame {
    let mut section = Vec::new();
    let mut metric = Vec::new();
    let mut value = Vec::new();

    if let Some(distribution) = result.distribution_section.as_ref() {
        section.push("distribution".to_string());
        metric.push("column".to_string());
        value.push(distribution.column.clone());
        section.push("distribution".to_string());
        metric.push("median".to_string());
        value.push(format!("{:.4}", distribution.distribution_summary.median));
        section.push("distribution".to_string());
        metric.push("skewness".to_string());
        value.push(format!("{:.4}", distribution.distribution_summary.skewness));
    } else {
        section.push("distribution".to_string());
        metric.push("status".to_string());
        value.push("unavailable".to_string());
    }

    if let Some(trend) = result.trend_section.as_ref() {
        section.push("trend".to_string());
        metric.push("direction".to_string());
        value.push(trend.direction.clone());
        section.push("trend".to_string());
        metric.push("absolute_change".to_string());
        value.push(format!("{:.4}", trend.absolute_change));
        section.push("trend".to_string());
        metric.push("change_rate".to_string());
        value.push(format!("{:.2}%", trend.change_rate * 100.0));
    } else {
        section.push("trend".to_string());
        metric.push("status".to_string());
        value.push("unavailable".to_string());
    }

    DataFrame::new(vec![
        Series::new("section".into(), section).into(),
        Series::new("metric".into(), metric).into(),
        Series::new("value".into(), value).into(),
    ])
    .expect("distribution and trend dataframe should be valid")
}

fn resolve_risk_level(result: &DiagnosticsReportResult) -> String {
    // 2026-03-29 00:49 CST：这里先用轻规则给管理摘要补风险等级，原因是方案3第一步要求把摘要页从“字段罗列”升级成“管理判断”；
    // 目的：先交付可解释、稳定的风险口径，而不是过早引入新的复杂评分器。
    if result.report_status == "degraded" || !result.warnings.is_empty() {
        "高".to_string()
    } else if result.available_section_count >= 3 && result.key_findings.len() <= 3 {
        "中".to_string()
    } else {
        "低".to_string()
    }
}

fn resolve_decision_readiness(result: &DiagnosticsReportResult) -> String {
    if result.report_status == "ok" && result.warnings.is_empty() {
        "是".to_string()
    } else {
        "否（建议复核）".to_string()
    }
}

fn resolve_review_recommendation(result: &DiagnosticsReportResult) -> String {
    // 2026-03-28 22:08 CST：这里优先用 warning 和降级状态生成复核建议，原因是分析承接型摘要首先要回答“现在是不是应该回头看数据或配置”；
    // 目的：让摘要页在高风险场景下稳定落到“先复核”，避免后续 AI 或人工被误导到直接建模。
    if result.report_status == "degraded" || !result.warnings.is_empty() {
        "建议立即复核".to_string()
    } else if result.available_section_count < result.section_count {
        "建议重点复核".to_string()
    } else {
        "可进入下一步前抽样复核".to_string()
    }
}

fn resolve_data_completion_recommendation(
    result: &DiagnosticsReportResult,
    request: &DiagnosticsReportExcelReportRequest,
) -> String {
    if result.available_section_count < result.section_count || has_missing_handoff_fields(request)
    {
        "建议优先补数".to_string()
    } else if result.available_section_count >= 3 {
        "当前可不优先补数".to_string()
    } else {
        "建议按需补数".to_string()
    }
}

fn resolve_modeling_recommendation(
    result: &DiagnosticsReportResult,
    request: &DiagnosticsReportExcelReportRequest,
) -> String {
    if result.report_status == "degraded" || !result.warnings.is_empty() {
        "暂不建议进入建模".to_string()
    } else if has_missing_handoff_fields(request) {
        "建议先补齐目标字段或时间字段后再评估建模".to_string()
    } else if request.diagnostics_request.correlation.is_some()
        && request.diagnostics_request.trend.is_some()
        && result.available_section_count >= 3
    {
        "可以进入下一步建模准备".to_string()
    } else {
        "建议先完成诊断补齐后再评估建模".to_string()
    }
}

fn resolve_suggested_tool(
    result: &DiagnosticsReportResult,
    request: &DiagnosticsReportExcelReportRequest,
) -> String {
    if result.report_status == "ok"
        && result.warnings.is_empty()
        && !has_missing_handoff_fields(request)
        && request.diagnostics_request.correlation.is_some()
        && request.diagnostics_request.trend.is_some()
    {
        "linear_regression".to_string()
    } else {
        "diagnostics_report".to_string()
    }
}

fn resolve_suggested_target_field(request: &DiagnosticsReportExcelReportRequest) -> String {
    request
        .diagnostics_request
        .correlation
        .as_ref()
        .map(|correlation| correlation.target_column.trim())
        .filter(|value| !value.is_empty())
        .unwrap_or("未配置（建议先补目标字段）")
        .to_string()
}

fn resolve_suggested_time_field(request: &DiagnosticsReportExcelReportRequest) -> String {
    request
        .diagnostics_request
        .trend
        .as_ref()
        .map(|trend| trend.time_column.trim())
        .filter(|value| !value.is_empty())
        .unwrap_or("未配置（建议先补时间字段）")
        .to_string()
}

fn resolve_main_blocker(
    result: &DiagnosticsReportResult,
    request: &DiagnosticsReportExcelReportRequest,
) -> String {
    if let Some(warning) = result.warnings.first() {
        return warning.clone();
    }

    if has_missing_handoff_fields(request) {
        return "目标字段或时间字段尚未配置完整".to_string();
    }

    let missing_sections = result
        .sections
        .iter()
        .filter(|section| section.status != "available")
        .map(|section| section.key.clone())
        .collect::<Vec<_>>();
    if !missing_sections.is_empty() {
        return format!("部分诊断分段仍不可用：{}", missing_sections.join(" / "));
    }

    "当前主要阻塞项较少，可继续下一步分析".to_string()
}

fn resolve_next_stage_condition(
    result: &DiagnosticsReportResult,
    request: &DiagnosticsReportExcelReportRequest,
) -> String {
    if let Some(warning) = result.warnings.first() {
        return format!("先处理 {} 后再进入下一步分析", warning);
    }

    if has_missing_handoff_fields(request) {
        return "先补齐目标字段或时间字段后再进入下一步".to_string();
    }

    if result.available_section_count < result.section_count {
        return "先补齐缺失诊断分段后再进入下一步".to_string();
    }

    "已满足进入下一步分析的基础条件".to_string()
}

fn has_missing_handoff_fields(request: &DiagnosticsReportExcelReportRequest) -> bool {
    request
        .diagnostics_request
        .correlation
        .as_ref()
        .map(|correlation| correlation.target_column.trim().is_empty())
        .unwrap_or(true)
        || request
            .diagnostics_request
            .trend
            .as_ref()
            .map(|trend| trend.time_column.trim().is_empty())
            .unwrap_or(true)
}

fn build_chart_sheet_dataframe(result: &DiagnosticsReportResult) -> DataFrame {
    let correlation_items = collect_correlation_chart_items(result);
    let outlier_items = collect_outlier_ratio_chart_items(result);
    let outlier_count_items = collect_outlier_count_chart_items(result);
    let distribution_items = collect_distribution_chart_items(result);
    let trend_points = result
        .trend_section
        .as_ref()
        .map(|section| section.points.clone())
        .unwrap_or_default();
    let max_len = correlation_items
        .len()
        .max(outlier_items.len())
        .max(outlier_count_items.len())
        .max(distribution_items.len())
        .max(trend_points.len())
        .max(1);

    let mut correlation_label = Vec::with_capacity(max_len);
    let mut correlation_value = Vec::with_capacity(max_len);
    let mut outlier_label = Vec::with_capacity(max_len);
    let mut outlier_value = Vec::with_capacity(max_len);
    let mut outlier_count = Vec::with_capacity(max_len);
    let mut distribution_label = Vec::with_capacity(max_len);
    let mut distribution_count = Vec::with_capacity(max_len);
    let mut distribution_ratio = Vec::with_capacity(max_len);
    let mut trend_time = Vec::with_capacity(max_len);
    let mut trend_value = Vec::with_capacity(max_len);

    for index in 0..max_len {
        let correlation_item = correlation_items.get(index);
        correlation_label.push(correlation_item.map(|item| item.0.clone()));
        correlation_value.push(correlation_item.map(|item| item.1));

        let outlier_item = outlier_items.get(index);
        outlier_label.push(outlier_item.map(|item| item.0.clone()));
        outlier_value.push(outlier_item.map(|item| item.1));
        let outlier_count_item = outlier_count_items.get(index);
        outlier_count.push(outlier_count_item.map(|item| item.1));

        let distribution_item = distribution_items.get(index);
        distribution_label.push(distribution_item.map(|item| item.0.clone()));
        distribution_count.push(distribution_item.map(|item| item.1));
        distribution_ratio.push(distribution_item.map(|item| item.2));

        let trend_item = trend_points.get(index);
        trend_time.push(trend_item.map(|item| item.time.clone()));
        trend_value.push(trend_item.map(|item| item.value));
    }

    DataFrame::new(vec![
        Series::new("相关性标签".into(), correlation_label).into(),
        Series::new("相关性系数".into(), correlation_value).into(),
        Series::new("异常列".into(), outlier_label).into(),
        Series::new("异常占比".into(), outlier_value).into(),
        Series::new("异常数".into(), outlier_count).into(),
        Series::new("分布区间".into(), distribution_label).into(),
        Series::new("分布计数".into(), distribution_count).into(),
        Series::new("分布占比".into(), distribution_ratio).into(),
        Series::new("趋势时间".into(), trend_time).into(),
        Series::new("趋势值".into(), trend_value).into(),
    ])
    .expect("chart sheet dataframe should be valid")
}

fn collect_correlation_chart_items(result: &DiagnosticsReportResult) -> Vec<(String, f64)> {
    let mut items = Vec::new();
    if let Some(correlation) = result.correlation_section.as_ref() {
        for item in correlation.top_positive.iter().take(3) {
            items.push((format!("正相关:{}", item.feature_column), item.coefficient));
        }
        for item in correlation.top_negative.iter().take(3) {
            items.push((format!("负相关:{}", item.feature_column), item.coefficient));
        }
    }
    items
}

fn collect_outlier_ratio_chart_items(result: &DiagnosticsReportResult) -> Vec<(String, f64)> {
    result
        .outlier_section
        .as_ref()
        .map(|outlier| {
            outlier
                .outlier_summaries
                .iter()
                .map(|item| (item.column.clone(), item.outlier_ratio))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
}

fn collect_outlier_count_chart_items(result: &DiagnosticsReportResult) -> Vec<(String, f64)> {
    let mut items = result
        .outlier_section
        .as_ref()
        .map(|outlier| {
            outlier
                .outlier_summaries
                .iter()
                .map(|item| (item.column.clone(), item.outlier_count as f64))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    items.sort_by(|left, right| {
        right
            .1
            .total_cmp(&left.1)
            .then_with(|| left.0.cmp(&right.0))
    });
    items
}

fn collect_distribution_chart_items(result: &DiagnosticsReportResult) -> Vec<(String, f64, f64)> {
    result
        .distribution_section
        .as_ref()
        .map(|distribution| {
            distribution
                .bins
                .iter()
                .map(|item| (item.label.clone(), item.count as f64, item.ratio))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
}

fn build_chart_specs(
    result: &DiagnosticsReportResult,
    chart_sheet_name: &str,
) -> Vec<PersistedWorkbookChartSpec> {
    let mut charts = Vec::new();

    if !collect_correlation_chart_items(result).is_empty() {
        charts.push(PersistedWorkbookChartSpec {
            chart_ref: None,
            source_refs: vec!["diagnostics_report_excel_report#correlation_chart".to_string()],
            chart_type: PersistedWorkbookChartType::Column,
            target_sheet_name: chart_sheet_name.to_string(),
            data_sheet_name: chart_sheet_name.to_string(),
            category_column: "相关性标签".to_string(),
            value_column: "相关性系数".to_string(),
            title: Some("相关性 Top 对比".to_string()),
            series: vec![PersistedWorkbookChartSeriesSpec {
                value_column: "相关性系数".to_string(),
                name: Some("相关性系数".to_string()),
            }],
            show_legend: false,
            legend_position: Some(PersistedWorkbookLegendPosition::TopRight),
            chart_style: Some(10),
            x_axis_name: Some("特征项".to_string()),
            y_axis_name: Some("相关系数".to_string()),
            // 2026-03-29 00:49 CST：这里把图放到右侧区域，原因是图表页左侧仍保留数据列供导出和回查；
            // 目的：在不引入隐藏 sheet 的前提下，避免图表直接压住数据源表。
            anchor_row: 1,
            anchor_col: 8,
        });
    }

    if !collect_outlier_ratio_chart_items(result).is_empty() {
        charts.push(PersistedWorkbookChartSpec {
            chart_ref: None,
            source_refs: vec!["diagnostics_report_excel_report#outlier_chart".to_string()],
            chart_type: PersistedWorkbookChartType::Column,
            target_sheet_name: chart_sheet_name.to_string(),
            data_sheet_name: chart_sheet_name.to_string(),
            category_column: "异常列".to_string(),
            value_column: "异常占比".to_string(),
            title: Some("异常占比对比".to_string()),
            series: vec![PersistedWorkbookChartSeriesSpec {
                value_column: "异常占比".to_string(),
                name: Some("异常占比".to_string()),
            }],
            show_legend: false,
            legend_position: Some(PersistedWorkbookLegendPosition::TopRight),
            chart_style: Some(11),
            x_axis_name: Some("列".to_string()),
            y_axis_name: Some("异常占比".to_string()),
            anchor_row: 17,
            anchor_col: 8,
        });
    }

    if !collect_distribution_chart_items(result).is_empty() {
        charts.push(PersistedWorkbookChartSpec {
            chart_ref: None,
            source_refs: vec!["diagnostics_report_excel_report#distribution_chart".to_string()],
            chart_type: PersistedWorkbookChartType::Column,
            target_sheet_name: chart_sheet_name.to_string(),
            data_sheet_name: chart_sheet_name.to_string(),
            category_column: "分布区间".to_string(),
            value_column: "分布计数".to_string(),
            title: Some("分布区间计数".to_string()),
            series: vec![PersistedWorkbookChartSeriesSpec {
                value_column: "分布计数".to_string(),
                name: Some("分布计数".to_string()),
            }],
            show_legend: false,
            legend_position: Some(PersistedWorkbookLegendPosition::TopRight),
            chart_style: Some(13),
            x_axis_name: Some("区间".to_string()),
            y_axis_name: Some("计数".to_string()),
            anchor_row: 1,
            anchor_col: 16,
        });
    }

    if !collect_outlier_count_chart_items(result).is_empty() {
        charts.push(PersistedWorkbookChartSpec {
            chart_ref: None,
            source_refs: vec!["diagnostics_report_excel_report#outlier_top_chart".to_string()],
            chart_type: PersistedWorkbookChartType::Column,
            target_sheet_name: chart_sheet_name.to_string(),
            data_sheet_name: chart_sheet_name.to_string(),
            category_column: "异常列".to_string(),
            value_column: "异常数".to_string(),
            title: Some("异常 Top 计数".to_string()),
            series: vec![PersistedWorkbookChartSeriesSpec {
                value_column: "异常数".to_string(),
                name: Some("异常数".to_string()),
            }],
            show_legend: false,
            legend_position: Some(PersistedWorkbookLegendPosition::TopRight),
            chart_style: Some(14),
            x_axis_name: Some("列".to_string()),
            y_axis_name: Some("异常点数".to_string()),
            anchor_row: 17,
            anchor_col: 16,
        });
    }

    if result
        .trend_section
        .as_ref()
        .map(|section| !section.points.is_empty())
        .unwrap_or(false)
    {
        charts.push(PersistedWorkbookChartSpec {
            chart_ref: None,
            source_refs: vec!["diagnostics_report_excel_report#trend_chart".to_string()],
            chart_type: PersistedWorkbookChartType::Line,
            target_sheet_name: chart_sheet_name.to_string(),
            data_sheet_name: chart_sheet_name.to_string(),
            category_column: "趋势时间".to_string(),
            value_column: "趋势值".to_string(),
            title: Some("趋势变化".to_string()),
            series: vec![PersistedWorkbookChartSeriesSpec {
                value_column: "趋势值".to_string(),
                name: Some("趋势值".to_string()),
            }],
            show_legend: false,
            legend_position: Some(PersistedWorkbookLegendPosition::TopRight),
            chart_style: Some(12),
            x_axis_name: Some("时间".to_string()),
            y_axis_name: Some("数值".to_string()),
            anchor_row: 33,
            anchor_col: 8,
        });
    }

    charts
}
