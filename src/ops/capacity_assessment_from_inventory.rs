use polars::prelude::DataFrame;
use serde::{Deserialize, Serialize};

use crate::domain::handles::TableHandle;
use crate::frame::loader::LoadedTable;
use crate::ops::capacity_assessment::{
    CapacityAssessmentRequest, CapacityAssessmentResult, InventoryEvidence, capacity_assessment,
};
use crate::ops::ssh_inventory::{SshInventoryRequest, SshInventoryResult, ssh_inventory};

// 2026-03-28 16:55 CST: 这里定义桥接 Tool 请求，原因是要把 SSH 盘点输入和容量分析输入解耦；目的是让桥接层只负责编排与映射，不侵入底层容量核心契约。
#[derive(Debug, Clone, Deserialize, Default)]
pub struct CapacityAssessmentFromInventoryRequest {
    #[serde(default)]
    pub inventory_request: Option<SshInventoryRequest>,
    #[serde(default)]
    pub inventory_result: Option<SshInventoryResult>,
    #[serde(default)]
    pub service_matchers: Option<ServiceMatchers>,
}

// 2026-03-28 16:55 CST: 这里定义保守实例匹配规则，原因是用户要求不要在没有显式规则时乱猜实例数；目的是把实例识别约束在可解释的 matcher 内。
#[derive(Debug, Clone, Deserialize, Default)]
pub struct ServiceMatchers {
    #[serde(default)]
    pub process_contains: Vec<String>,
    #[serde(default)]
    pub command_contains: Vec<String>,
}

// 2026-03-28 16:55 CST: 这里定义桥接 Tool 输出，原因是除了容量结论还要暴露映射明细；目的是让交付和复核都能看到 SSH 事实如何进入容量模型。
#[derive(Debug, Clone, Serialize)]
pub struct CapacityAssessmentFromInventoryResult {
    pub capacity_result: CapacityAssessmentResult,
    pub inventory_mapping: InventoryMappingSummary,
    pub mapping_confidence: MappingConfidence,
}

#[derive(Debug, Clone, Serialize)]
pub struct InventoryMappingSummary {
    pub source: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub discovered_instance_count: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub host_count: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub host_cpu_cores: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub host_memory_mb: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub matched_process_count: Option<u64>,
    #[serde(default)]
    pub matched_rules: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MappingConfidence {
    pub instance_count_confidence: String,
    pub host_fact_confidence: String,
}

// 2026-03-28 16:55 CST: 这里提供桥接主入口，原因是要把受限 SSH 盘点结果自动映射进容量模型；目的是打通“采集 -> 证据 -> 结论”的正式主链。
pub fn capacity_assessment_from_inventory(
    loaded: Option<&LoadedTable>,
    bridge_request: &CapacityAssessmentFromInventoryRequest,
    capacity_request: &CapacityAssessmentRequest,
) -> Result<CapacityAssessmentFromInventoryResult, String> {
    let inventory_result = resolve_inventory_result(bridge_request)?;
    let inventory_mapping =
        build_inventory_mapping(&inventory_result, bridge_request.service_matchers.as_ref());
    let mut derived_capacity_request = capacity_request.clone();
    derived_capacity_request.inventory_evidence = Some(InventoryEvidence {
        source: inventory_mapping.source.clone(),
        discovered_instance_count: inventory_mapping.discovered_instance_count,
        host_count: inventory_mapping.host_count,
        host_cpu_cores: inventory_mapping.host_cpu_cores,
        host_memory_mb: inventory_mapping.host_memory_mb,
    });

    let fallback_loaded = empty_loaded_table();
    let loaded = loaded.unwrap_or(&fallback_loaded);
    let capacity_result = capacity_assessment(loaded, &derived_capacity_request);
    let mapping_confidence = build_mapping_confidence(&inventory_mapping);

    Ok(CapacityAssessmentFromInventoryResult {
        capacity_result,
        inventory_mapping,
        mapping_confidence,
    })
}

fn resolve_inventory_result(
    bridge_request: &CapacityAssessmentFromInventoryRequest,
) -> Result<SshInventoryResult, String> {
    if let Some(result) = bridge_request.inventory_result.clone() {
        return Ok(result);
    }
    if let Some(request) = bridge_request.inventory_request.as_ref() {
        return ssh_inventory(request);
    }
    Err(
        "capacity_assessment_from_inventory requires inventory_request or inventory_result"
            .to_string(),
    )
}

fn empty_loaded_table() -> LoadedTable {
    LoadedTable {
        // 2026-03-28 16:55 CST: 这里构造空表上下文，原因是桥接 Tool 必须支持“只有 SSH、没有 Excel”时也能退化分析；目的是复用现有容量核心而不伪造文件来源。
        handle: TableHandle::new_confirmed("inventory://bridge", "inventory_bridge", Vec::new()),
        dataframe: DataFrame::default(),
    }
}

fn build_inventory_mapping(
    inventory_result: &SshInventoryResult,
    matchers: Option<&ServiceMatchers>,
) -> InventoryMappingSummary {
    let matched_process_count = count_matched_processes(inventory_result, matchers);
    let matched_rules = summarize_matched_rules(matchers);

    InventoryMappingSummary {
        source: "ssh_inventory".to_string(),
        discovered_instance_count: matched_process_count.filter(|count| *count > 0),
        host_count: Some(1),
        host_cpu_cores: inventory_result.inventory.cpu_core_count,
        host_memory_mb: inventory_result.inventory.memory_total_mb,
        matched_process_count,
        matched_rules,
    }
}

fn count_matched_processes(
    inventory_result: &SshInventoryResult,
    matchers: Option<&ServiceMatchers>,
) -> Option<u64> {
    let Some(matchers) = matchers else {
        return None;
    };
    if matchers.process_contains.is_empty() && matchers.command_contains.is_empty() {
        return None;
    }

    let raw_processes = inventory_result
        .command_outputs
        .get("ps -ef")
        .cloned()
        .or_else(|| inventory_result.inventory.process_snapshot_excerpt.clone())?;

    let match_count = raw_processes
        .lines()
        .filter(|line| {
            let trimmed = line.trim();
            !trimmed.is_empty()
                && (matchers
                    .process_contains
                    .iter()
                    .any(|needle| trimmed.contains(needle))
                    || matchers
                        .command_contains
                        .iter()
                        .any(|needle| trimmed.contains(needle)))
        })
        .count() as u64;

    if match_count == 0 {
        None
    } else {
        Some(match_count)
    }
}

fn summarize_matched_rules(matchers: Option<&ServiceMatchers>) -> Vec<String> {
    let Some(matchers) = matchers else {
        return Vec::new();
    };

    let mut rules = Vec::new();
    rules.extend(
        matchers
            .process_contains
            .iter()
            .map(|value| format!("process_contains:{value}")),
    );
    rules.extend(
        matchers
            .command_contains
            .iter()
            .map(|value| format!("command_contains:{value}")),
    );
    rules
}

fn build_mapping_confidence(mapping: &InventoryMappingSummary) -> MappingConfidence {
    let instance_count_confidence = if mapping.discovered_instance_count.is_some() {
        "matched_processes".to_string()
    } else {
        "host_facts_only".to_string()
    };
    let host_fact_confidence =
        if mapping.host_cpu_cores.is_some() || mapping.host_memory_mb.is_some() {
            "direct_inventory".to_string()
        } else {
            "insufficient".to_string()
        };

    MappingConfidence {
        instance_count_confidence,
        host_fact_confidence,
    }
}
