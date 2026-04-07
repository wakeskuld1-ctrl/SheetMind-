use polars::prelude::{DataFrame, NamedFrom, Series};
use serde::{Deserialize, Serialize};

use crate::domain::handles::TableHandle;
use crate::frame::loader::LoadedTable;
use crate::frame::workbook_ref_store::{
    PersistedWorkbookSheetKind, WorkbookDraftStore, WorkbookSheetInput,
};
use crate::ops::capacity_assessment::{CapacityAssessmentRequest, capacity_assessment};
use crate::ops::capacity_assessment_from_inventory::{
    CapacityAssessmentFromInventoryRequest, InventoryMappingSummary, MappingConfidence,
    capacity_assessment_from_inventory,
};
use crate::ops::export::export_excel_workbook;

// 2026-03-28 22:18 CST: 这里定义容量评估 Excel 报表请求，原因是这轮要把“分析参数 + SSH 桥接参数 + 交付参数”收口成单个正式 Tool；
// 目的是让调用方一次请求就能拿到 workbook_ref 或最终 xlsx，而不是继续手工串多个底层 Tool。
#[derive(Debug, Clone, Deserialize)]
pub struct CapacityAssessmentExcelReportRequest {
    pub report_name: String,
    #[serde(default)]
    pub report_subtitle: Option<String>,
    #[serde(default = "default_conclusion_sheet_name")]
    pub conclusion_sheet_name: String,
    #[serde(default = "default_resource_sheet_name")]
    pub resource_sheet_name: String,
    #[serde(default = "default_evidence_sheet_name")]
    pub evidence_sheet_name: String,
    #[serde(default = "default_action_sheet_name")]
    pub action_sheet_name: String,
    #[serde(default)]
    pub output_path: Option<String>,
    #[serde(flatten)]
    pub capacity_request: CapacityAssessmentRequest,
    #[serde(flatten)]
    pub bridge_request: CapacityAssessmentFromInventoryRequest,
}

// 2026-03-28 22:18 CST: 这里定义 Excel 报表 Tool 输出，原因是上层既要消费容量结论也要消费 workbook 交付句柄；
// 目的是把“结论 + 交付物”放在同一响应中，减少二次查询和状态拼接。
#[derive(Debug, Clone, Serialize)]
pub struct CapacityAssessmentExcelReportResult {
    pub capacity_result: crate::ops::capacity_assessment::CapacityAssessmentResult,
    pub workbook_ref: String,
    pub sheet_names: Vec<String>,
    pub format: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inventory_mapping: Option<InventoryMappingSummary>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mapping_confidence: Option<MappingConfidence>,
}

fn default_conclusion_sheet_name() -> String {
    "结论页".to_string()
}

fn default_resource_sheet_name() -> String {
    "资源测算页".to_string()
}

fn default_evidence_sheet_name() -> String {
    "证据与风险页".to_string()
}

fn default_action_sheet_name() -> String {
    "补数与行动页".to_string()
}

// 2026-03-28 22:18 CST: 这里提供高层 Excel 交付主入口，原因是用户要的是“拿数据直接出 Excel”而不是只返 JSON；
// 目的是复用现有容量分析与 workbook 导出底座，形成正式一站式交付能力。
pub fn capacity_assessment_excel_report(
    loaded: Option<&LoadedTable>,
    request: &CapacityAssessmentExcelReportRequest,
) -> Result<CapacityAssessmentExcelReportResult, String> {
    if request.report_name.trim().is_empty() {
        return Err("capacity_assessment_excel_report 缺少 report_name".to_string());
    }
    if matches!(request.output_path.as_deref(), Some(path) if path.trim().is_empty()) {
        return Err("capacity_assessment_excel_report 的 output_path 不能为空".to_string());
    }

    let (capacity_result, inventory_mapping, mapping_confidence) =
        if has_inventory_bridge_inputs(request) {
            let bridge_result = capacity_assessment_from_inventory(
                loaded,
                &request.bridge_request,
                &request.capacity_request,
            )?;
            (
                bridge_result.capacity_result,
                Some(bridge_result.inventory_mapping),
                Some(bridge_result.mapping_confidence),
            )
        } else {
            let fallback_loaded = empty_loaded_table();
            let base_loaded = loaded.unwrap_or(&fallback_loaded);
            (
                capacity_assessment(base_loaded, &request.capacity_request),
                None,
                None,
            )
        };

    let workbook_ref = WorkbookDraftStore::create_workbook_ref();
    let sheet_names = vec![
        request.conclusion_sheet_name.clone(),
        request.resource_sheet_name.clone(),
        request.evidence_sheet_name.clone(),
        request.action_sheet_name.clone(),
    ];
    let source_refs = source_refs_for_delivery(loaded, has_inventory_bridge_inputs(request));
    let worksheets = vec![
        WorkbookSheetInput {
            sheet_name: request.conclusion_sheet_name.clone(),
            source_refs: source_refs.clone(),
            dataframe: build_conclusion_dataframe(&capacity_result),
            // 2026-03-28 22:18 CST: 这里把容量结论页显式标成数据页，原因是导出层需要稳定复用标题、冻结和筛选规则；
            // 目的是让最终交付表在视觉上像汇报页，在行为上仍保持可筛选的 Excel 表结构。
            sheet_kind: PersistedWorkbookSheetKind::DataSheet,
            export_options: None,
            title: Some(request.report_name.clone()),
            subtitle: Some(resolve_sheet_subtitle(
                request.report_subtitle.as_deref(),
                "容量结论与建议",
            )),
            data_start_row: 2,
        },
        WorkbookSheetInput {
            sheet_name: request.resource_sheet_name.clone(),
            source_refs: source_refs.clone(),
            dataframe: build_resource_dataframe(&capacity_result),
            // 2026-03-28 22:18 CST: 这里单独落资源测算页，原因是资源维度结论比总评更适合表格复核；
            // 目的是把 CPU/内存等推导细节从“黑盒建议”改成“可核对交付”。
            sheet_kind: PersistedWorkbookSheetKind::DataSheet,
            export_options: None,
            title: Some(request.report_name.clone()),
            subtitle: Some("资源维度测算明细".to_string()),
            data_start_row: 2,
        },
        WorkbookSheetInput {
            sheet_name: request.evidence_sheet_name.clone(),
            source_refs: source_refs.clone(),
            dataframe: build_evidence_dataframe(
                &capacity_result,
                inventory_mapping.as_ref(),
                mapping_confidence.as_ref(),
            ),
            // 2026-03-28 22:18 CST: 这里把证据与风险独立成页，原因是用户明确要求数据不足时也要给决策思路；
            // 目的是把“凭什么这么判断”一并交付，降低结果不可解释的风险。
            sheet_kind: PersistedWorkbookSheetKind::DataSheet,
            export_options: None,
            title: Some(request.report_name.clone()),
            subtitle: Some("证据来源与风险画像".to_string()),
            data_start_row: 2,
        },
        WorkbookSheetInput {
            sheet_name: request.action_sheet_name.clone(),
            source_refs,
            dataframe: build_action_dataframe(&capacity_result),
            // 2026-03-28 22:18 CST: 这里保留补数与行动页，原因是 guidance_only / partial 场景下核心价值不只是结果数值；
            // 目的是把缺失项、补数优先级和人工决策路径直接交付给运维评审。
            sheet_kind: PersistedWorkbookSheetKind::DataSheet,
            export_options: None,
            title: Some(request.report_name.clone()),
            subtitle: Some("补数优先级与行动路径".to_string()),
            data_start_row: 2,
        },
    ];

    let draft = crate::frame::workbook_ref_store::PersistedWorkbookDraft::from_sheet_inputs(
        &workbook_ref,
        worksheets,
    )
    .map_err(|error| error.to_string())?;
    let store = WorkbookDraftStore::workspace_default().map_err(|error| error.to_string())?;
    store.save(&draft).map_err(|error| error.to_string())?;

    if let Some(output_path) = request.output_path.as_deref() {
        export_excel_workbook(&draft, output_path).map_err(|error| error.to_string())?;
    }

    Ok(CapacityAssessmentExcelReportResult {
        capacity_result,
        workbook_ref,
        sheet_names,
        format: if request.output_path.is_some() {
            "xlsx".to_string()
        } else {
            "workbook_ref".to_string()
        },
        output_path: request.output_path.clone(),
        inventory_mapping,
        mapping_confidence,
    })
}

fn has_inventory_bridge_inputs(request: &CapacityAssessmentExcelReportRequest) -> bool {
    request.bridge_request.inventory_request.is_some()
        || request.bridge_request.inventory_result.is_some()
}

fn empty_loaded_table() -> LoadedTable {
    LoadedTable {
        // 2026-03-28 22:18 CST: 这里构造空表上下文，原因是用户要求“没有 Excel 也要给决策思路”；
        // 目的是让高层 Excel 报表 Tool 也能复用既有容量降级逻辑，而不是因为缺少表源直接失败。
        handle: TableHandle::new_confirmed("report://capacity", "capacity_report", Vec::new()),
        dataframe: DataFrame::default(),
    }
}

fn source_refs_for_delivery(loaded: Option<&LoadedTable>, with_inventory: bool) -> Vec<String> {
    let mut refs = loaded
        .map(|table| {
            vec![format!(
                "{}#{}",
                table.handle.source_path(),
                table.handle.sheet_name()
            )]
        })
        .unwrap_or_default();
    if with_inventory {
        refs.push("ssh_inventory".to_string());
    }
    refs.push("capacity_assessment_excel_report".to_string());
    refs
}

fn resolve_sheet_subtitle(report_subtitle: Option<&str>, fallback: &str) -> String {
    report_subtitle
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| fallback.to_string())
}

fn build_conclusion_dataframe(
    result: &crate::ops::capacity_assessment::CapacityAssessmentResult,
) -> DataFrame {
    let mut metrics = vec![
        (
            "服务名称".to_string(),
            optional_text(result.service_risk_profile.service_name.as_deref()),
        ),
        ("证据等级".to_string(), result.evidence_level.clone()),
        ("容量状态".to_string(), result.capacity_status.clone()),
        (
            "当前实例数".to_string(),
            optional_number(result.current_instance_count),
        ),
        (
            "建议实例数".to_string(),
            optional_number(result.recommended_instance_count),
        ),
        (
            "瓶颈资源".to_string(),
            optional_text(result.limiting_resource.as_deref()),
        ),
        (
            "推荐下一步".to_string(),
            result.decision_guidance.recommended_next_step.clone(),
        ),
        ("结论摘要".to_string(), result.human_summary.overall.clone()),
    ];
    if result.history_signals.has_time_series {
        metrics.push((
            "趋势证据".to_string(),
            format!(
                "已纳入 {} 个趋势指标",
                result.history_signals.analyzed_metrics.len()
            ),
        ));
    }
    key_value_dataframe(metrics)
}

fn build_resource_dataframe(
    result: &crate::ops::capacity_assessment::CapacityAssessmentResult,
) -> DataFrame {
    let rows = if result.resource_assessments.is_empty() {
        vec![(
            "未形成量化资源结论".to_string(),
            "-".to_string(),
            "-".to_string(),
            "-".to_string(),
            "-".to_string(),
            "-".to_string(),
            "请先补齐实例数与资源利用率".to_string(),
        )]
    } else {
        result
            .resource_assessments
            .iter()
            .map(|item| {
                (
                    item.resource.clone(),
                    format!("{:.4}", item.current_utilization),
                    format!("{:.4}", item.target_utilization),
                    format!("{:.4}", item.projected_demand_multiplier),
                    format!("{:.4}", item.saturation_penalty),
                    item.required_instances.to_string(),
                    item.status.clone(),
                )
            })
            .collect::<Vec<_>>()
    };

    let mut resource = Vec::with_capacity(rows.len());
    let mut current = Vec::with_capacity(rows.len());
    let mut target = Vec::with_capacity(rows.len());
    let mut projected = Vec::with_capacity(rows.len());
    let mut penalty = Vec::with_capacity(rows.len());
    let mut required = Vec::with_capacity(rows.len());
    let mut status = Vec::with_capacity(rows.len());

    for (a, b, c, d, e, f, g) in rows {
        resource.push(a);
        current.push(b);
        target.push(c);
        projected.push(d);
        penalty.push(e);
        required.push(f);
        status.push(g);
    }

    DataFrame::new(vec![
        Series::new("资源类型".into(), resource).into(),
        Series::new("当前利用率".into(), current).into(),
        Series::new("目标利用率".into(), target).into(),
        Series::new("预测需求放大系数".into(), projected).into(),
        Series::new("饱和惩罚系数".into(), penalty).into(),
        Series::new("建议实例数".into(), required).into(),
        Series::new("状态".into(), status).into(),
    ])
    .expect("resource dataframe should build")
}

fn build_evidence_dataframe(
    result: &crate::ops::capacity_assessment::CapacityAssessmentResult,
    inventory_mapping: Option<&InventoryMappingSummary>,
    mapping_confidence: Option<&MappingConfidence>,
) -> DataFrame {
    let mut pairs = vec![
        ("模型族".to_string(), result.model_family.clone()),
        (
            "服务等级".to_string(),
            result.service_risk_profile.service_tier.clone(),
        ),
        (
            "SLA".to_string(),
            optional_text(result.service_risk_profile.sla_level.as_deref()),
        ),
        (
            "峰值模式".to_string(),
            result.service_risk_profile.peak_pattern.clone(),
        ),
        (
            "故障影响".to_string(),
            result.service_risk_profile.failure_impact.clone(),
        ),
        (
            "冗余模式".to_string(),
            result.service_risk_profile.redundancy_mode.clone(),
        ),
        (
            "扩容步长".to_string(),
            result.service_risk_profile.scaling_step.to_string(),
        ),
        (
            "证据来源".to_string(),
            comma_join_or_dash(&result.service_risk_profile.evidence_sources),
        ),
        (
            "时间序列证据".to_string(),
            if result.history_signals.has_time_series {
                format!(
                    "{} / {} 条观测",
                    comma_join_or_dash(&result.history_signals.analyzed_metrics),
                    result.history_signals.observation_count
                )
            } else {
                "无".to_string()
            },
        ),
    ];

    if let Some(observation) = result.trend_observations.cpu.as_ref() {
        pairs.push((
            "CPU 趋势".to_string(),
            format!("{} / {:.4}", observation.direction, observation.change_rate),
        ));
    }
    if let Some(observation) = result.trend_observations.memory.as_ref() {
        pairs.push((
            "内存趋势".to_string(),
            format!("{} / {:.4}", observation.direction, observation.change_rate),
        ));
    }
    if let Some(observation) = result.trend_observations.workload.as_ref() {
        pairs.push((
            "业务量趋势".to_string(),
            format!("{} / {:.4}", observation.direction, observation.change_rate),
        ));
    }
    if let Some(evidence) = result.inventory_evidence.as_ref() {
        pairs.push(("盘点来源".to_string(), evidence.source.clone()));
        pairs.push((
            "盘点实例数".to_string(),
            optional_number(evidence.discovered_instance_count),
        ));
        pairs.push(("主机数".to_string(), optional_number(evidence.host_count)));
        pairs.push((
            "主机 CPU 核数".to_string(),
            optional_number(evidence.host_cpu_cores),
        ));
        pairs.push((
            "主机内存 MB".to_string(),
            optional_number(evidence.host_memory_mb),
        ));
    }
    if let Some(mapping) = inventory_mapping {
        pairs.push((
            "匹配到的实例数".to_string(),
            optional_number(mapping.matched_process_count),
        ));
        pairs.push((
            "匹配规则".to_string(),
            if mapping.matched_rules.is_empty() {
                "-".to_string()
            } else {
                mapping.matched_rules.join(" | ")
            },
        ));
    }
    if let Some(confidence) = mapping_confidence {
        pairs.push((
            "实例映射置信度".to_string(),
            confidence.instance_count_confidence.clone(),
        ));
        pairs.push((
            "主机事实置信度".to_string(),
            confidence.host_fact_confidence.clone(),
        ));
    }

    key_value_dataframe(pairs)
}

fn build_action_dataframe(
    result: &crate::ops::capacity_assessment::CapacityAssessmentResult,
) -> DataFrame {
    let mut action_type = Vec::new();
    let mut detail = Vec::new();

    if result.missing_inputs.is_empty() {
        action_type.push("缺失输入".to_string());
        detail.push("当前无关键缺失输入".to_string());
    } else {
        for item in &result.missing_inputs {
            action_type.push("缺失输入".to_string());
            detail.push(item.clone());
        }
    }

    if result.decision_guidance.missing_input_priority.is_empty() {
        action_type.push("补数优先级".to_string());
        detail.push("当前无额外补数优先级".to_string());
    } else {
        for item in &result.decision_guidance.missing_input_priority {
            action_type.push("补数优先级".to_string());
            detail.push(item.clone());
        }
    }

    for item in &result.decision_guidance.manual_decision_path {
        action_type.push("人工决策路径".to_string());
        detail.push(item.clone());
    }

    action_type.push("置信度说明".to_string());
    detail.push(result.decision_guidance.confidence_comment.clone());
    action_type.push("推荐下一步".to_string());
    detail.push(result.decision_guidance.recommended_next_step.clone());

    DataFrame::new(vec![
        Series::new("动作类型".into(), action_type).into(),
        Series::new("说明".into(), detail).into(),
    ])
    .expect("action dataframe should build")
}

fn key_value_dataframe(pairs: Vec<(String, String)>) -> DataFrame {
    let mut metric = Vec::with_capacity(pairs.len());
    let mut value = Vec::with_capacity(pairs.len());
    for (left, right) in pairs {
        metric.push(left);
        value.push(right);
    }
    DataFrame::new(vec![
        Series::new("指标".into(), metric).into(),
        Series::new("值".into(), value).into(),
    ])
    .expect("key value dataframe should build")
}

fn optional_text(value: Option<&str>) -> String {
    value
        .map(str::trim)
        .filter(|item| !item.is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| "-".to_string())
}

fn optional_number(value: Option<u64>) -> String {
    value
        .map(|item| item.to_string())
        .unwrap_or_else(|| "-".to_string())
}

fn comma_join_or_dash(values: &[String]) -> String {
    if values.is_empty() {
        "-".to_string()
    } else {
        values.join(", ")
    }
}
