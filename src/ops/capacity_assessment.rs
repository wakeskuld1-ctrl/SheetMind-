use polars::prelude::AnyValue;
use serde::{Deserialize, Serialize};

use crate::frame::loader::LoadedTable;
use crate::ops::trend_analysis::{TrendAnalysisResult, trend_analysis};

const INSTANCE_ALIASES: &[&str] = &[
    "instances",
    "instance_count",
    "replicas",
    "pods",
    "pod_count",
    "node_count",
];
const CPU_ALIASES: &[&str] = &["cpu_usage", "cpu_utilization", "cpu", "cpu_ratio"];
const MEMORY_ALIASES: &[&str] = &[
    "memory_usage",
    "memory_utilization",
    "memory",
    "mem_usage",
    "mem_ratio",
];
const WORKLOAD_ALIASES: &[&str] = &[
    "workload",
    "workload_qps",
    "qps",
    "rps",
    "tps",
    "traffic",
    "requests",
    "throughput",
];
const TIME_ALIASES: &[&str] = &["ts", "time", "timestamp", "datetime", "date"];

// 2026-03-28 15:05 CST: 这里扩展容量评估请求，原因是要把“场景、部署、指标、SSH盘点证据”统一纳入一个高层场景工具；
// 目的是让用户即使不给完整监控列，也能用服务逻辑和实例事实得到更有业务语义的结论。
#[derive(Debug, Clone, Deserialize)]
pub struct CapacityAssessmentRequest {
    #[serde(default)]
    pub time_column: Option<String>,
    #[serde(default)]
    pub instance_count_column: Option<String>,
    #[serde(default)]
    pub cpu_column: Option<String>,
    #[serde(default)]
    pub memory_column: Option<String>,
    #[serde(default)]
    pub workload_column: Option<String>,
    #[serde(default)]
    pub scenario_profile: Option<ScenarioProfile>,
    #[serde(default)]
    pub deployment_profile: Option<DeploymentProfile>,
    #[serde(default)]
    pub inventory_evidence: Option<InventoryEvidence>,
    #[serde(default = "default_target_cpu_utilization")]
    pub target_cpu_utilization: f64,
    #[serde(default = "default_target_memory_utilization")]
    pub target_memory_utilization: f64,
    #[serde(default = "default_peak_multiplier")]
    pub peak_multiplier: f64,
    #[serde(default = "default_growth_rate")]
    pub growth_rate: f64,
    #[serde(default)]
    pub require_n_plus_one: bool,
}

// 2026-03-28 15:05 CST: 这里定义场景画像，原因是容量判断不能只看资源指标；
// 目的是把服务等级、SLA 和峰值模式纳入风险推断，而不是把所有服务一刀切。
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct ScenarioProfile {
    #[serde(default)]
    pub service_name: Option<String>,
    #[serde(default)]
    pub service_tier: Option<String>,
    #[serde(default)]
    pub sla_level: Option<String>,
    #[serde(default)]
    pub peak_pattern: Option<String>,
    #[serde(default)]
    pub failure_impact: Option<String>,
}

// 2026-03-28 15:05 CST: 这里定义部署画像，原因是实例数、冗余模式和扩容步长常常不在监控表里；
// 目的是允许用户通过部署事实补齐容量决策所需的关键上下文。
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct DeploymentProfile {
    #[serde(default)]
    pub current_instance_count: Option<u64>,
    #[serde(default)]
    pub redundancy_mode: Option<String>,
    #[serde(default)]
    pub scaling_step: Option<u64>,
    #[serde(default)]
    pub deployment_kind: Option<String>,
    #[serde(default)]
    pub max_instance_count: Option<u64>,
}

// 2026-03-28 15:05 CST: 这里定义 SSH/外部盘点证据，原因是部署事实有时来自机器盘点而不是 Excel；
// 目的是把“自动补数”与“人工填报”统一成同一种可消费证据。
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct InventoryEvidence {
    #[serde(default)]
    pub source: String,
    #[serde(default)]
    pub discovered_instance_count: Option<u64>,
    #[serde(default)]
    pub host_count: Option<u64>,
    #[serde(default)]
    pub host_cpu_cores: Option<u64>,
    #[serde(default)]
    pub host_memory_mb: Option<u64>,
}

// 2026-03-28 15:05 CST: 这里定义输出里的服务风险画像，原因是用户最终需要知道结论受哪些业务/部署假设影响；
// 目的是把关键假设显式带回结果，方便复核和交付。
#[derive(Debug, Clone, Serialize)]
pub struct ServiceRiskProfile {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service_name: Option<String>,
    pub service_tier: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sla_level: Option<String>,
    pub peak_pattern: String,
    pub failure_impact: String,
    pub redundancy_mode: String,
    pub scaling_step: u64,
    #[serde(default)]
    pub evidence_sources: Vec<String>,
}

// 2026-03-28 15:05 CST: 这里统一定义容量输出，原因是现在要同时承载量化、部分量化和指导模式；
// 目的是让上层报表和后续 Skill 都能稳定消费同一份结构化结果。
#[derive(Debug, Clone, Serialize)]
pub struct CapacityAssessmentResult {
    pub model_family: String,
    pub evidence_level: String,
    pub capacity_status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_instance_count: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recommended_instance_count: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limiting_resource: Option<String>,
    pub resolved_columns: ResolvedColumns,
    pub service_risk_profile: ServiceRiskProfile,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inventory_evidence: Option<InventoryEvidence>,
    #[serde(default)]
    pub resource_assessments: Vec<ResourceAssessment>,
    #[serde(default)]
    pub missing_inputs: Vec<String>,
    pub history_signals: HistorySignals,
    pub trend_observations: TrendObservationSet,
    pub decision_guidance: DecisionGuidance,
    pub human_summary: CapacityHumanSummary,
}

// 2026-03-28 15:05 CST: 这里保留识别到的列名映射，原因是现场 Excel 口径经常不统一；
// 目的是把自动识别结果透明返回，方便用户核对字段口径。
#[derive(Debug, Clone, Serialize)]
pub struct ResolvedColumns {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_column: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instance_count_column: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cpu_column: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memory_column: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workload_column: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ResourceAssessment {
    pub resource: String,
    pub current_utilization: f64,
    pub target_utilization: f64,
    pub projected_demand_multiplier: f64,
    pub saturation_penalty: f64,
    pub required_instances: u64,
    pub status: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct HistorySignals {
    pub has_time_series: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_column: Option<String>,
    #[serde(default)]
    pub analyzed_metrics: Vec<String>,
    pub observation_count: usize,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct TrendObservationSet {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cpu: Option<MetricTrendObservation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memory: Option<MetricTrendObservation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workload: Option<MetricTrendObservation>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MetricTrendObservation {
    pub direction: String,
    pub change_rate: f64,
    pub point_count: usize,
    pub start_value: f64,
    pub end_value: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct DecisionGuidance {
    pub recommended_next_step: String,
    #[serde(default)]
    pub missing_input_priority: Vec<String>,
    #[serde(default)]
    pub manual_decision_path: Vec<String>,
    pub confidence_comment: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct CapacityHumanSummary {
    pub overall: String,
    #[serde(default)]
    pub key_points: Vec<String>,
    pub recommended_next_step: String,
}

#[derive(Debug, Clone)]
struct SnapshotMetrics {
    sheet_instance_count: Option<f64>,
    cpu_utilization: Option<f64>,
    memory_utilization: Option<f64>,
    workload: Option<f64>,
}

#[derive(Debug, Clone)]
struct CapacityFacts {
    current_instances: Option<f64>,
    current_instance_source: Option<String>,
    metric_resource_count: usize,
}

fn default_target_cpu_utilization() -> f64 {
    0.70
}

fn default_target_memory_utilization() -> f64 {
    0.75
}

fn default_peak_multiplier() -> f64 {
    1.00
}

fn default_growth_rate() -> f64 {
    0.00
}

// 2026-03-28 15:05 CST: 这里提供新的容量评估主入口，原因是工具已从“看指标”升级为“看场景 + 看部署 + 看指标”；
// 目的是让同一个工具既能吃 Excel 数据，也能吃 SSH 补来的实例事实。
pub fn capacity_assessment(
    loaded: &LoadedTable,
    request: &CapacityAssessmentRequest,
) -> CapacityAssessmentResult {
    let resolved_columns = resolve_columns(loaded, request);
    let snapshot = extract_latest_snapshot(loaded, &resolved_columns);
    let service_risk_profile = build_service_risk_profile(request);
    let trend_observations = build_trend_observations(loaded, &resolved_columns);
    let history_signals = build_history_signals(&resolved_columns, &trend_observations);
    let capacity_facts = resolve_capacity_facts(request, &snapshot);
    let missing_inputs = collect_missing_inputs(
        loaded,
        request,
        &resolved_columns,
        &snapshot,
        &capacity_facts,
    );
    let resource_assessments = build_resource_assessments(
        &snapshot,
        request,
        &service_risk_profile,
        &capacity_facts,
        &trend_observations,
        &history_signals,
    );
    let current_instance_count = capacity_facts.current_instances.map(round_instance_count);
    let recommended_instance_count = resource_assessments
        .iter()
        .map(|item| item.required_instances)
        .max();
    let limiting_resource = resource_assessments
        .iter()
        .max_by_key(|item| item.required_instances)
        .map(|item| item.resource.clone());
    let evidence_level = determine_evidence_level(
        &capacity_facts,
        recommended_instance_count,
        &missing_inputs,
        request,
    );

    let capacity_status = match (current_instance_count, recommended_instance_count) {
        (Some(current), Some(recommended)) if recommended > current => "insufficient".to_string(),
        (Some(_), Some(_)) => "sufficient".to_string(),
        _ => "unknown".to_string(),
    };

    let decision_guidance = match evidence_level.as_str() {
        "guidance_only" => build_guidance_only_decision(&missing_inputs, &history_signals),
        "partial" => build_partial_guidance(
            &capacity_status,
            current_instance_count,
            recommended_instance_count,
            &limiting_resource,
            &missing_inputs,
            &capacity_facts,
        ),
        _ => build_quantified_guidance(
            &capacity_status,
            current_instance_count,
            recommended_instance_count,
            &limiting_resource,
            &missing_inputs,
        ),
    };
    let human_summary = match evidence_level.as_str() {
        "guidance_only" => {
            build_guidance_only_summary(&missing_inputs, &history_signals, &decision_guidance)
        }
        "partial" => build_partial_summary(
            &capacity_status,
            current_instance_count,
            recommended_instance_count,
            &limiting_resource,
            &resource_assessments,
            &service_risk_profile,
            &decision_guidance,
        ),
        _ => build_quantified_summary(
            &capacity_status,
            current_instance_count,
            recommended_instance_count,
            &limiting_resource,
            &resource_assessments,
            &history_signals,
            &decision_guidance,
        ),
    };

    CapacityAssessmentResult {
        model_family: "elastic_rule_based_nonlinear_v2".to_string(),
        evidence_level,
        capacity_status,
        current_instance_count,
        recommended_instance_count,
        limiting_resource,
        resolved_columns,
        service_risk_profile,
        inventory_evidence: request.inventory_evidence.clone(),
        resource_assessments,
        missing_inputs,
        history_signals,
        trend_observations,
        decision_guidance,
        human_summary,
    }
}

fn resolve_columns(loaded: &LoadedTable, request: &CapacityAssessmentRequest) -> ResolvedColumns {
    let available_columns = loaded
        .dataframe
        .get_columns()
        .iter()
        .map(|column| column.name().as_str().to_string())
        .collect::<Vec<_>>();

    ResolvedColumns {
        time_column: resolve_column_name(
            request.time_column.as_deref(),
            &available_columns,
            TIME_ALIASES,
        ),
        instance_count_column: resolve_column_name(
            request.instance_count_column.as_deref(),
            &available_columns,
            INSTANCE_ALIASES,
        ),
        cpu_column: resolve_column_name(
            request.cpu_column.as_deref(),
            &available_columns,
            CPU_ALIASES,
        ),
        memory_column: resolve_column_name(
            request.memory_column.as_deref(),
            &available_columns,
            MEMORY_ALIASES,
        ),
        workload_column: resolve_column_name(
            request.workload_column.as_deref(),
            &available_columns,
            WORKLOAD_ALIASES,
        ),
    }
}

fn extract_latest_snapshot(loaded: &LoadedTable, columns: &ResolvedColumns) -> SnapshotMetrics {
    let row_count = loaded.dataframe.height();
    if row_count == 0 {
        return SnapshotMetrics {
            sheet_instance_count: None,
            cpu_utilization: None,
            memory_utilization: None,
            workload: None,
        };
    }

    let time_values = columns
        .time_column
        .as_deref()
        .map(|column| collect_text_column(loaded, column))
        .unwrap_or_default();
    let instance_values = columns
        .instance_count_column
        .as_deref()
        .map(|column| collect_numeric_column(loaded, column))
        .unwrap_or_else(|| vec![None; row_count]);
    let cpu_values = columns
        .cpu_column
        .as_deref()
        .map(|column| collect_numeric_column(loaded, column))
        .unwrap_or_else(|| vec![None; row_count]);
    let memory_values = columns
        .memory_column
        .as_deref()
        .map(|column| collect_numeric_column(loaded, column))
        .unwrap_or_else(|| vec![None; row_count]);
    let workload_values = columns
        .workload_column
        .as_deref()
        .map(|column| collect_numeric_column(loaded, column))
        .unwrap_or_else(|| vec![None; row_count]);

    let snapshot_index = select_latest_snapshot_index(
        &time_values,
        &instance_values,
        &cpu_values,
        &memory_values,
        &workload_values,
    );

    match snapshot_index {
        Some(index) => SnapshotMetrics {
            sheet_instance_count: instance_values.get(index).copied().flatten(),
            cpu_utilization: cpu_values.get(index).copied().flatten(),
            memory_utilization: memory_values.get(index).copied().flatten(),
            workload: workload_values.get(index).copied().flatten(),
        },
        None => SnapshotMetrics {
            sheet_instance_count: None,
            cpu_utilization: None,
            memory_utilization: None,
            workload: None,
        },
    }
}

fn select_latest_snapshot_index(
    time_values: &[Option<String>],
    instance_values: &[Option<f64>],
    cpu_values: &[Option<f64>],
    memory_values: &[Option<f64>],
    workload_values: &[Option<f64>],
) -> Option<usize> {
    let prefer_resource_rows = cpu_values
        .iter()
        .zip(memory_values.iter())
        .any(|(cpu, memory)| cpu.is_some() || memory.is_some());

    let mut best_index = None;
    let mut best_time = String::new();

    for index in 0..instance_values.len() {
        let has_signal = instance_values[index].is_some()
            || cpu_values[index].is_some()
            || memory_values[index].is_some()
            || workload_values[index].is_some();
        if !has_signal {
            continue;
        }
        if prefer_resource_rows && !(cpu_values[index].is_some() || memory_values[index].is_some())
        {
            continue;
        }

        let current_time = time_values
            .get(index)
            .and_then(|value| value.clone())
            .unwrap_or_else(|| format!("{index:020}"));
        if best_index.is_none() || current_time >= best_time {
            best_index = Some(index);
            best_time = current_time;
        }
    }

    best_index
}

// 2026-03-28 15:05 CST: 这里统一解析当前实例事实，原因是实例数可能来自监控表、部署画像或 SSH 盘点；
// 目的是把不同来源的实例事实统一成后续容量估算的输入。
fn resolve_capacity_facts(
    request: &CapacityAssessmentRequest,
    snapshot: &SnapshotMetrics,
) -> CapacityFacts {
    if let Some(count) = snapshot.sheet_instance_count {
        return CapacityFacts {
            current_instances: Some(count),
            current_instance_source: Some("sheet_metric".to_string()),
            metric_resource_count: metric_resource_count(snapshot),
        };
    }

    if let Some(count) = request
        .deployment_profile
        .as_ref()
        .and_then(|profile| profile.current_instance_count)
    {
        return CapacityFacts {
            current_instances: Some(count as f64),
            current_instance_source: Some("deployment_profile".to_string()),
            metric_resource_count: metric_resource_count(snapshot),
        };
    }

    if let Some(count) = request
        .inventory_evidence
        .as_ref()
        .and_then(|evidence| evidence.discovered_instance_count)
    {
        return CapacityFacts {
            current_instances: Some(count as f64),
            current_instance_source: Some("inventory_evidence".to_string()),
            metric_resource_count: metric_resource_count(snapshot),
        };
    }

    CapacityFacts {
        current_instances: None,
        current_instance_source: None,
        metric_resource_count: metric_resource_count(snapshot),
    }
}

fn metric_resource_count(snapshot: &SnapshotMetrics) -> usize {
    usize::from(snapshot.cpu_utilization.is_some())
        + usize::from(snapshot.memory_utilization.is_some())
}

fn build_service_risk_profile(request: &CapacityAssessmentRequest) -> ServiceRiskProfile {
    let service_tier = request
        .scenario_profile
        .as_ref()
        .and_then(|profile| profile.service_tier.clone())
        .unwrap_or_else(|| "standard".to_string());
    let peak_pattern = request
        .scenario_profile
        .as_ref()
        .and_then(|profile| profile.peak_pattern.clone())
        .unwrap_or_else(|| "steady".to_string());
    let failure_impact = request
        .scenario_profile
        .as_ref()
        .and_then(|profile| profile.failure_impact.clone())
        .unwrap_or_else(|| "medium".to_string());
    let redundancy_mode = request
        .deployment_profile
        .as_ref()
        .and_then(|profile| profile.redundancy_mode.clone())
        .unwrap_or_else(|| {
            if request.require_n_plus_one {
                "n_plus_one".to_string()
            } else {
                "none".to_string()
            }
        });
    let scaling_step = request
        .deployment_profile
        .as_ref()
        .and_then(|profile| profile.scaling_step)
        .unwrap_or(1)
        .max(1);
    let mut evidence_sources = Vec::new();
    if request.scenario_profile.is_some() {
        evidence_sources.push("scenario_profile".to_string());
    }
    if request.deployment_profile.is_some() {
        evidence_sources.push("deployment_profile".to_string());
    }
    if request.inventory_evidence.is_some() {
        evidence_sources.push("inventory_evidence".to_string());
    }

    ServiceRiskProfile {
        service_name: request
            .scenario_profile
            .as_ref()
            .and_then(|profile| profile.service_name.clone()),
        service_tier,
        sla_level: request
            .scenario_profile
            .as_ref()
            .and_then(|profile| profile.sla_level.clone()),
        peak_pattern,
        failure_impact,
        redundancy_mode,
        scaling_step,
        evidence_sources,
    }
}

fn build_resource_assessments(
    snapshot: &SnapshotMetrics,
    request: &CapacityAssessmentRequest,
    risk_profile: &ServiceRiskProfile,
    capacity_facts: &CapacityFacts,
    trend_observations: &TrendObservationSet,
    history_signals: &HistorySignals,
) -> Vec<ResourceAssessment> {
    let Some(current_instances) = capacity_facts.current_instances else {
        return Vec::new();
    };
    if current_instances <= 0.0 {
        return Vec::new();
    }

    let mut assessments = Vec::new();

    if let Some(cpu_utilization) = snapshot.cpu_utilization {
        assessments.push(build_single_resource_assessment(
            "cpu",
            cpu_utilization,
            request.target_cpu_utilization,
            current_instances,
            request,
            risk_profile,
            trend_observations.cpu.as_ref(),
            history_signals,
        ));
    }

    if let Some(memory_utilization) = snapshot.memory_utilization {
        assessments.push(build_single_resource_assessment(
            "memory",
            memory_utilization,
            request.target_memory_utilization,
            current_instances,
            request,
            risk_profile,
            trend_observations.memory.as_ref(),
            history_signals,
        ));
    }

    assessments
}

fn build_single_resource_assessment(
    resource: &str,
    current_utilization: f64,
    target_utilization: f64,
    current_instances: f64,
    request: &CapacityAssessmentRequest,
    risk_profile: &ServiceRiskProfile,
    trend: Option<&MetricTrendObservation>,
    history_signals: &HistorySignals,
) -> ResourceAssessment {
    let safe_target = if target_utilization <= 0.0 {
        0.70
    } else {
        target_utilization
    };
    let projected_demand_multiplier = request.peak_multiplier.max(0.20)
        * (1.0 + request.growth_rate).max(0.20)
        * scenario_pressure_multiplier(risk_profile)
        * history_pressure_multiplier(trend, history_signals);
    let saturation_penalty = saturation_penalty(current_utilization, safe_target);
    let mut required_instances =
        (current_instances * projected_demand_multiplier * saturation_penalty)
            .ceil()
            .max(1.0) as u64;

    if request.require_n_plus_one || risk_profile.redundancy_mode == "n_plus_one" {
        required_instances = required_instances.saturating_add(1);
    }

    required_instances = round_up_to_step(required_instances, risk_profile.scaling_step);

    if let Some(max_instances) = request
        .deployment_profile
        .as_ref()
        .and_then(|profile| profile.max_instance_count)
    {
        required_instances = required_instances.min(max_instances.max(1));
    }

    let current_count = round_instance_count(current_instances);
    let status = if required_instances > current_count {
        "insufficient".to_string()
    } else {
        "sufficient".to_string()
    };

    ResourceAssessment {
        resource: resource.to_string(),
        current_utilization,
        target_utilization: safe_target,
        projected_demand_multiplier,
        saturation_penalty,
        required_instances,
        status,
    }
}

fn determine_evidence_level(
    capacity_facts: &CapacityFacts,
    recommended_instance_count: Option<u64>,
    missing_inputs: &[String],
    request: &CapacityAssessmentRequest,
) -> String {
    if capacity_facts.current_instances.is_none() || recommended_instance_count.is_none() {
        return "guidance_only".to_string();
    }

    let uses_auxiliary_instance_source = matches!(
        capacity_facts.current_instance_source.as_deref(),
        Some("deployment_profile") | Some("inventory_evidence")
    );
    let has_context_profiles = request.scenario_profile.is_some()
        || request.deployment_profile.is_some()
        || request.inventory_evidence.is_some();

    if uses_auxiliary_instance_source
        || capacity_facts.metric_resource_count < 2
        || (!missing_inputs.is_empty() && has_context_profiles)
    {
        "partial".to_string()
    } else {
        "quantified".to_string()
    }
}

fn build_trend_observations(
    loaded: &LoadedTable,
    columns: &ResolvedColumns,
) -> TrendObservationSet {
    let Some(time_column) = columns.time_column.as_deref() else {
        return TrendObservationSet::default();
    };

    TrendObservationSet {
        cpu: columns
            .cpu_column
            .as_deref()
            .and_then(|column| summarize_trend(loaded, time_column, column)),
        memory: columns
            .memory_column
            .as_deref()
            .and_then(|column| summarize_trend(loaded, time_column, column)),
        workload: columns
            .workload_column
            .as_deref()
            .and_then(|column| summarize_trend(loaded, time_column, column)),
    }
}

fn build_history_signals(
    columns: &ResolvedColumns,
    trends: &TrendObservationSet,
) -> HistorySignals {
    let mut analyzed_metrics = Vec::new();
    let mut observation_count = 0usize;

    if let Some(cpu) = trends.cpu.as_ref() {
        analyzed_metrics.push("cpu".to_string());
        observation_count = observation_count.max(cpu.point_count);
    }
    if let Some(memory) = trends.memory.as_ref() {
        analyzed_metrics.push("memory".to_string());
        observation_count = observation_count.max(memory.point_count);
    }
    if let Some(workload) = trends.workload.as_ref() {
        analyzed_metrics.push("workload".to_string());
        observation_count = observation_count.max(workload.point_count);
    }

    HistorySignals {
        has_time_series: !analyzed_metrics.is_empty(),
        time_column: if analyzed_metrics.is_empty() {
            None
        } else {
            columns.time_column.clone()
        },
        analyzed_metrics,
        observation_count,
    }
}

fn collect_missing_inputs(
    loaded: &LoadedTable,
    request: &CapacityAssessmentRequest,
    columns: &ResolvedColumns,
    snapshot: &SnapshotMetrics,
    capacity_facts: &CapacityFacts,
) -> Vec<String> {
    let mut missing = Vec::new();

    if capacity_facts.current_instances.is_none() {
        missing.push(
            "补充当前实例数，可来自 instances 列、deployment_profile 或 ssh_inventory".to_string(),
        );
    }
    if columns.cpu_column.is_none() || snapshot.cpu_utilization.is_none() {
        missing.push("补充 CPU 利用率列（如 cpu_usage、cpu_utilization）".to_string());
    }
    if columns.memory_column.is_none() || snapshot.memory_utilization.is_none() {
        missing.push("补充内存利用率列（如 memory_usage、memory_utilization）".to_string());
    }
    if columns.workload_column.is_none() || snapshot.workload.is_none() {
        missing.push("补充业务量列（如 qps、rps、tps、workload）".to_string());
    }
    if request.scenario_profile.is_none() {
        missing.push("补充 scenario_profile，明确服务等级、峰值模式和故障影响".to_string());
    }
    if request.deployment_profile.is_none() && request.inventory_evidence.is_none() {
        missing.push(
            "补充 deployment_profile 或 inventory_evidence，明确冗余模式和扩容步长".to_string(),
        );
    }
    if columns.time_column.is_none()
        || !has_any_non_empty_value(loaded, columns.time_column.as_deref())
    {
        missing
            .push("补充时间列（如 ts、time、timestamp），便于识别趋势与峰值前移信号".to_string());
    }

    missing
}

fn build_quantified_guidance(
    capacity_status: &str,
    current: Option<u64>,
    recommended: Option<u64>,
    limiting_resource: &Option<String>,
    missing_inputs: &[String],
) -> DecisionGuidance {
    let current = current.unwrap_or(0);
    let recommended = recommended.unwrap_or(current);
    let bottleneck = limiting_resource
        .as_deref()
        .map(human_resource_name)
        .unwrap_or("当前瓶颈资源");
    let recommended_next_step = if capacity_status == "insufficient" {
        format!(
            "优先围绕 {bottleneck} 进行扩容校准，当前 {current} 实例建议至少提升到 {recommended} 实例，并在扩容后回看高峰时段指标。"
        )
    } else if missing_inputs.is_empty() {
        format!(
            "当前容量暂时可支撑，建议保留现有 {current} 实例并持续跟踪 {bottleneck} 的高峰水位变化。"
        )
    } else {
        format!("当前容量暂时可支撑，但仍建议补齐缺失指标后复核 {bottleneck}，避免高峰期估算偏差。")
    };

    DecisionGuidance {
        recommended_next_step,
        missing_input_priority: missing_inputs.to_vec(),
        manual_decision_path: vec![
            "先确认当前峰值实例数和资源利用率来自同一观测窗口".to_string(),
            "再确认扩容目标是否需要叠加活动、发布和容灾冗余".to_string(),
            "最后将建议实例数回填到交付报表并安排复核".to_string(),
        ],
        confidence_comment: "当前结论主要由指标证据直接量化得出，可作为本轮容量评估结论。"
            .to_string(),
    }
}

fn build_partial_guidance(
    capacity_status: &str,
    current: Option<u64>,
    recommended: Option<u64>,
    limiting_resource: &Option<String>,
    missing_inputs: &[String],
    capacity_facts: &CapacityFacts,
) -> DecisionGuidance {
    let current = current.unwrap_or(0);
    let recommended = recommended.unwrap_or(current);
    let bottleneck = limiting_resource
        .as_deref()
        .map(human_resource_name)
        .unwrap_or("当前瓶颈资源");
    let instance_source = capacity_facts
        .current_instance_source
        .clone()
        .unwrap_or_else(|| "辅助上下文".to_string());
    let recommended_next_step = if capacity_status == "insufficient" {
        format!(
            "当前结论依赖 {instance_source} 补齐的实例事实，建议优先围绕 {bottleneck} 将实例从 {current} 提升到至少 {recommended}，再补齐缺失指标复核。"
        )
    } else {
        format!(
            "当前结论依赖 {instance_source} 提供的实例事实，建议维持 {current} 实例并尽快补齐缺失指标，复核 {bottleneck} 的高峰风险。"
        )
    };

    DecisionGuidance {
        recommended_next_step,
        missing_input_priority: missing_inputs.to_vec(),
        manual_decision_path: vec![
            "先确认 deployment_profile 或 ssh_inventory 返回的实例事实是否与当前生产一致"
                .to_string(),
            "再补齐另一个核心资源指标或时间序列，缩小部分量化结论的不确定性".to_string(),
            "最后在扩容评审中标注本次结论依赖的辅助证据来源".to_string(),
        ],
        confidence_comment:
            "当前结论为部分量化：已经能给出实例建议，但仍依赖部署画像或 SSH 盘点证据补齐关键事实。"
                .to_string(),
    }
}

fn build_guidance_only_decision(
    missing_inputs: &[String],
    history_signals: &HistorySignals,
) -> DecisionGuidance {
    let mut manual_decision_path = vec![
        "先补齐当前实例数和至少一个核心资源利用率，否则无法形成最基本的容量下限".to_string(),
        "如果暂时拿不到完整历史，就先用最近一次峰值快照加业务活动倍数做保守评估".to_string(),
        "如果连峰值快照也没有，就先访谈业务峰谷规律、活动计划和变更窗口，产出风险分层结论"
            .to_string(),
    ];
    if history_signals.has_time_series {
        manual_decision_path.push(
            "已有时间序列可以先判断方向，优先补齐同时间窗下的实例事实，再把趋势证据接入量化结论"
                .to_string(),
        );
    }

    DecisionGuidance {
        recommended_next_step: format!(
            "优先补齐 {}，至少先补“当前实例数 + CPU/内存任一利用率 + 峰值业务量或活动倍数”三类输入，再做正式容量量化。",
            missing_inputs
                .first()
                .cloned()
                .unwrap_or_else(|| "关键容量输入".to_string())
        ),
        missing_input_priority: missing_inputs.to_vec(),
        manual_decision_path,
        confidence_comment:
            "当前证据不足以直接给出可靠实例数结论，但已经可以输出补数优先级和保守决策路径。"
                .to_string(),
    }
}

fn build_quantified_summary(
    capacity_status: &str,
    current: Option<u64>,
    recommended: Option<u64>,
    limiting_resource: &Option<String>,
    resource_assessments: &[ResourceAssessment],
    history_signals: &HistorySignals,
    decision_guidance: &DecisionGuidance,
) -> CapacityHumanSummary {
    let current = current.unwrap_or(0);
    let recommended = recommended.unwrap_or(current);
    let bottleneck = limiting_resource
        .as_deref()
        .map(human_resource_name)
        .unwrap_or("当前瓶颈资源");
    let overall = if capacity_status == "insufficient" {
        format!(
            "当前容量偏紧，现有 {current} 实例不足以承接目标负载，建议至少扩至 {recommended} 实例。"
        )
    } else {
        format!("当前容量总体可支撑，现有 {current} 实例暂时满足目标负载要求。")
    };
    let mut key_points = resource_assessments
        .iter()
        .map(|item| {
            format!(
                "{} 当前利用率 {:.2}，目标 {:.2}，饱和放大系数 {:.2}，建议实例数 {}。",
                human_resource_name(item.resource.as_str()),
                item.current_utilization,
                item.target_utilization,
                item.saturation_penalty,
                item.required_instances
            )
        })
        .collect::<Vec<_>>();
    key_points.push(format!("当前主要瓶颈倾向于 {bottleneck}。"));
    if history_signals.has_time_series {
        key_points.push(format!(
            "本次结论已结合时间序列证据，覆盖 {} 个趋势指标。",
            history_signals.analyzed_metrics.len()
        ));
    }

    CapacityHumanSummary {
        overall,
        key_points,
        recommended_next_step: decision_guidance.recommended_next_step.clone(),
    }
}

fn build_partial_summary(
    capacity_status: &str,
    current: Option<u64>,
    recommended: Option<u64>,
    limiting_resource: &Option<String>,
    resource_assessments: &[ResourceAssessment],
    risk_profile: &ServiceRiskProfile,
    decision_guidance: &DecisionGuidance,
) -> CapacityHumanSummary {
    let current = current.unwrap_or(0);
    let recommended = recommended.unwrap_or(current);
    let bottleneck = limiting_resource
        .as_deref()
        .map(human_resource_name)
        .unwrap_or("当前瓶颈资源");
    let overall = if capacity_status == "insufficient" {
        format!(
            "当前容量结论为部分量化：结合 {} 服务画像和部署事实判断，建议至少从 {current} 扩至 {recommended} 实例。",
            risk_profile.service_tier
        )
    } else {
        format!(
            "当前容量结论为部分量化：结合 {} 服务画像和部署事实，现有 {current} 实例暂时可支撑。",
            risk_profile.service_tier
        )
    };
    let mut key_points = resource_assessments
        .iter()
        .map(|item| {
            format!(
                "{} 当前利用率 {:.2}，按 {} 峰值模式和 {} 冗余模式折算后建议实例数 {}。",
                human_resource_name(item.resource.as_str()),
                item.current_utilization,
                risk_profile.peak_pattern,
                risk_profile.redundancy_mode,
                item.required_instances
            )
        })
        .collect::<Vec<_>>();
    key_points.push(format!("当前主要瓶颈倾向于 {bottleneck}。"));
    key_points
        .push("本次结论依赖辅助上下文补齐的实例事实，建议在正式交付中标明证据来源。".to_string());

    CapacityHumanSummary {
        overall,
        key_points,
        recommended_next_step: decision_guidance.recommended_next_step.clone(),
    }
}

fn build_guidance_only_summary(
    missing_inputs: &[String],
    history_signals: &HistorySignals,
    decision_guidance: &DecisionGuidance,
) -> CapacityHumanSummary {
    let mut key_points = missing_inputs.iter().take(5).cloned().collect::<Vec<_>>();
    if history_signals.has_time_series {
        key_points
            .push("虽然缺少完整容量字段，但已有时间序列可以先判断指标方向和波动风险。".to_string());
    } else {
        key_points.push(
            "当前缺少可直接量化的容量基础字段，只能先做补数优先级和保守决策路径判断。".to_string(),
        );
    }

    CapacityHumanSummary {
        overall: "当前输入不足以直接给出可靠的实例容量结论，但可以先输出补数优先级和保守决策路径。"
            .to_string(),
        key_points,
        recommended_next_step: decision_guidance.recommended_next_step.clone(),
    }
}

fn summarize_trend(
    loaded: &LoadedTable,
    time_column: &str,
    value_column: &str,
) -> Option<MetricTrendObservation> {
    trend_analysis(loaded, time_column, value_column)
        .ok()
        .map(metric_trend_observation)
}

fn metric_trend_observation(result: TrendAnalysisResult) -> MetricTrendObservation {
    MetricTrendObservation {
        direction: result.direction,
        change_rate: result.change_rate,
        point_count: result.point_count,
        start_value: result.start_value,
        end_value: result.end_value,
    }
}

fn scenario_pressure_multiplier(profile: &ServiceRiskProfile) -> f64 {
    let tier_factor = match profile.service_tier.as_str() {
        "core" => 1.10,
        "important" => 1.05,
        _ => 1.00,
    };
    let peak_factor = match profile.peak_pattern.as_str() {
        "burst" => 1.10,
        "seasonal" => 1.05,
        _ => 1.00,
    };
    let failure_factor = match profile.failure_impact.as_str() {
        "high" => 1.05,
        "critical" => 1.08,
        _ => 1.00,
    };

    tier_factor * peak_factor * failure_factor
}

fn history_pressure_multiplier(
    trend: Option<&MetricTrendObservation>,
    history_signals: &HistorySignals,
) -> f64 {
    let base = trend
        .map(|item| {
            if item.direction == "upward" {
                1.0 + item.change_rate.max(0.0).min(2.0).powf(1.10) * 0.25
            } else {
                1.0
            }
        })
        .unwrap_or(1.0);

    if history_signals.has_time_series {
        base.max(1.0)
    } else {
        1.0
    }
}

fn saturation_penalty(current_utilization: f64, target_utilization: f64) -> f64 {
    let ratio = (current_utilization.max(0.0) / target_utilization.max(0.05)).max(0.0);
    if ratio <= 1.0 {
        ratio
    } else {
        ratio.powf(1.35)
    }
}

fn round_instance_count(value: f64) -> u64 {
    value.round().max(0.0) as u64
}

fn round_up_to_step(value: u64, step: u64) -> u64 {
    if step <= 1 {
        value
    } else {
        value.div_ceil(step) * step
    }
}

fn resolve_column_name(
    explicit: Option<&str>,
    available_columns: &[String],
    aliases: &[&str],
) -> Option<String> {
    if let Some(column) = explicit {
        if let Some(found) = find_column_case_insensitive(available_columns, column) {
            return Some(found);
        }
    }

    aliases
        .iter()
        .find_map(|alias| find_column_case_insensitive(available_columns, alias))
        .or_else(|| {
            available_columns.iter().find_map(|candidate| {
                let normalized_candidate = normalize_column_name(candidate);
                aliases
                    .iter()
                    .any(|alias| normalized_candidate.contains(&normalize_column_name(alias)))
                    .then(|| candidate.clone())
            })
        })
}

fn find_column_case_insensitive(available_columns: &[String], target: &str) -> Option<String> {
    let normalized_target = normalize_column_name(target);
    available_columns
        .iter()
        .find(|candidate| normalize_column_name(candidate) == normalized_target)
        .cloned()
}

fn normalize_column_name(value: &str) -> String {
    value
        .trim()
        .to_ascii_lowercase()
        .replace([' ', '-', '_'], "")
}

fn collect_numeric_column(loaded: &LoadedTable, column: &str) -> Vec<Option<f64>> {
    match loaded.dataframe.column(column) {
        Ok(series) => series
            .as_materialized_series()
            .iter()
            .map(|value| any_value_to_text(&value).and_then(|text| parse_numeric_value(&text)))
            .collect(),
        Err(_) => vec![None; loaded.dataframe.height()],
    }
}

fn collect_text_column(loaded: &LoadedTable, column: &str) -> Vec<Option<String>> {
    match loaded.dataframe.column(column) {
        Ok(series) => series
            .as_materialized_series()
            .iter()
            .map(|value| any_value_to_text(&value))
            .collect(),
        Err(_) => vec![None; loaded.dataframe.height()],
    }
}

fn has_any_non_empty_value(loaded: &LoadedTable, column: Option<&str>) -> bool {
    column
        .map(|name| {
            collect_text_column(loaded, name)
                .into_iter()
                .any(|item| item.is_some())
        })
        .unwrap_or(false)
}

fn parse_numeric_value(raw: &str) -> Option<f64> {
    let normalized = raw.trim().trim_end_matches('%').replace(',', "");
    if normalized.is_empty() {
        return None;
    }
    normalized.parse::<f64>().ok().map(|value| {
        if raw.trim().ends_with('%') {
            value / 100.0
        } else {
            value
        }
    })
}

fn human_resource_name(resource: &str) -> &'static str {
    match resource {
        "cpu" => "CPU",
        "memory" => "内存",
        _ => "资源",
    }
}

fn any_value_to_text(value: &AnyValue<'_>) -> Option<String> {
    match value {
        AnyValue::Null => None,
        AnyValue::String(text) => {
            let trimmed = text.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_string())
            }
        }
        AnyValue::StringOwned(text) => {
            let trimmed = text.as_str().trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_string())
            }
        }
        _ => {
            let rendered = value.to_string();
            let trimmed = rendered.trim().trim_matches('"');
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_string())
            }
        }
    }
}
