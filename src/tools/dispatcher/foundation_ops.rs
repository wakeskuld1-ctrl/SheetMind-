use std::path::Path;

use serde_json::Value;

use crate::ops::foundation::knowledge_repository::{
    KnowledgeRepository, MetadataRepositoryAuditIssue,
};
use crate::ops::foundation::metadata_schema::MetadataSchema;
use crate::ops::foundation::metadata_validator::MetadataValidationIssue;
use crate::tools::contracts::{
    FoundationRepositoryImportGateRequest, FoundationRepositoryImportGateResult,
    FoundationRepositoryMetadataAuditBatchRequest, FoundationRepositoryMetadataAuditBatchResult,
    FoundationRepositoryMetadataAuditGateResult, FoundationRepositoryMetadataAuditIssue,
    FoundationRepositoryMetadataAuditRequest, FoundationRepositoryMetadataAuditResult,
    ToolResponse,
};

// 2026-04-10 CST: 这里保留 foundation repository metadata audit 的正式 dispatcher 入口，原因是上一阶段已经把 repository 级治理能力工具化，
// 目的：继续让 CLI / Skill 通过统一入口拿到标准化审计报告，同时让后续 gate 与 batch 都建立在同一条执行链上。
pub(super) fn dispatch_foundation_repository_metadata_audit(args: Value) -> ToolResponse {
    let request = match serde_json::from_value::<FoundationRepositoryMetadataAuditRequest>(args) {
        Ok(request) => request,
        Err(error) => return ToolResponse::error(format!("request parsing failed: {error}")),
    };

    let metadata_schema = match request.metadata_schema.build_schema() {
        Ok(metadata_schema) => metadata_schema,
        Err(error) => {
            return ToolResponse::error(format!("metadata schema build failed: {error:?}"));
        }
    };
    let audit_result = match execute_foundation_repository_metadata_audit_for_repository(
        request.repository_layout_dir,
        &metadata_schema,
    ) {
        Ok(audit_result) => audit_result,
        Err(error_response) => return error_response,
    };
    let result = FoundationRepositoryMetadataAuditResult::new(
        audit_result.repository_layout_dir,
        audit_result.repository_schema_version,
        audit_result.metadata_schema_version,
        audit_result.issues,
    );

    ToolResponse::ok_serialized(&result)
}

// 2026-04-10 CST: 这里保留 foundation repository metadata audit gate 分发入口，原因是方案A当前已经进入“审计结果消费层”阶段，
// 目的：让上层编排直接消费 gate_passed 与 blocking/non_blocking 分类，而不必在外层复制通用治理规则。
pub(super) fn dispatch_foundation_repository_metadata_audit_gate(args: Value) -> ToolResponse {
    let request = match serde_json::from_value::<FoundationRepositoryMetadataAuditRequest>(args) {
        Ok(request) => request,
        Err(error) => return ToolResponse::error(format!("request parsing failed: {error}")),
    };

    let metadata_schema = match request.metadata_schema.build_schema() {
        Ok(metadata_schema) => metadata_schema,
        Err(error) => {
            return ToolResponse::error(format!("metadata schema build failed: {error:?}"));
        }
    };
    let audit_result = match execute_foundation_repository_metadata_audit_for_repository(
        request.repository_layout_dir,
        &metadata_schema,
    ) {
        Ok(audit_result) => audit_result,
        Err(error_response) => return error_response,
    };

    ToolResponse::ok_serialized(&build_foundation_repository_metadata_audit_gate_result(
        audit_result,
    ))
}

// 2026-04-10 CST: 这里新增 foundation repository metadata audit batch 分发入口，原因是 A1 已确定先做批量入口而不是直接扩到导入链，
// 目的：让上层先基于一份共用 schema 对多个 repository layout 批量执行 gate，并拿到统一批次摘要。
pub(super) fn dispatch_foundation_repository_metadata_audit_batch(args: Value) -> ToolResponse {
    let request =
        match serde_json::from_value::<FoundationRepositoryMetadataAuditBatchRequest>(args) {
            Ok(request) => request,
            Err(error) => return ToolResponse::error(format!("request parsing failed: {error}")),
        };

    let metadata_schema = match request.metadata_schema.build_schema() {
        Ok(metadata_schema) => metadata_schema,
        Err(error) => {
            return ToolResponse::error(format!("metadata schema build failed: {error:?}"));
        }
    };

    let result = match execute_foundation_repository_metadata_audit_batch(
        request.repository_layout_dirs,
        &metadata_schema,
    ) {
        Ok(result) => result,
        Err(error_response) => return error_response,
    };

    ToolResponse::ok_serialized(&result)
}

// 2026-04-10 CST: 这里新增 foundation repository import gate 分发入口，原因是方案B1要求把 batch 审计结果提升为导入接入层消费能力，
// 目的：让上层直接拿到 accepted/rejected 列表、阻塞原因汇总和 next_stage_allowed，而不是继续手工解释批量 gate 结果。
pub(super) fn dispatch_foundation_repository_import_gate(args: Value) -> ToolResponse {
    let request = match serde_json::from_value::<FoundationRepositoryImportGateRequest>(args) {
        Ok(request) => request,
        Err(error) => return ToolResponse::error(format!("request parsing failed: {error}")),
    };

    let metadata_schema = match request.metadata_schema.build_schema() {
        Ok(metadata_schema) => metadata_schema,
        Err(error) => {
            return ToolResponse::error(format!("metadata schema build failed: {error:?}"));
        }
    };
    let batch_result = match execute_foundation_repository_metadata_audit_batch(
        request.repository_layout_dirs,
        &metadata_schema,
    ) {
        Ok(result) => result,
        Err(error_response) => return error_response,
    };

    ToolResponse::ok_serialized(&FoundationRepositoryImportGateResult::new(
        batch_result.repositories,
    ))
}

// 2026-04-10 CST: 这里抽共享的单仓库 audit 执行 helper，原因是单仓库 audit、gate 与 batch 都依赖同一条 repository 装载与 issue 映射逻辑，
// 目的：让 batch 可以复用已构建的 metadata schema，避免同一批次里重复 build schema 或出现实现分叉。
fn execute_foundation_repository_metadata_audit_for_repository(
    repository_layout_dir: String,
    metadata_schema: &MetadataSchema,
) -> Result<FoundationRepositoryMetadataAuditExecution, ToolResponse> {
    let repository = KnowledgeRepository::load_from_layout_dir(Path::new(&repository_layout_dir))
        .map_err(|error| {
        ToolResponse::error(format!("repository layout load failed: {error:?}"))
    })?;
    let issues = repository
        .audit_metadata(metadata_schema)
        .issues
        .into_iter()
        .map(map_repository_audit_issue)
        .collect::<Vec<_>>();

    Ok(FoundationRepositoryMetadataAuditExecution {
        repository_layout_dir,
        repository_schema_version: repository.bundle().schema_version.clone(),
        metadata_schema_version: metadata_schema.schema_version.clone(),
        issues,
    })
}

// 2026-04-10 CST: 这里抽共享 batch 执行 helper，原因是方案A的 batch 与方案B1的 import gate 都依赖同一批逐仓库 gate 结果，
// 目的：把“批量执行 + 逐仓库 gate 结果汇总”集中到单点，避免两个入口各自重写循环逻辑后产生语义漂移。
fn execute_foundation_repository_metadata_audit_batch(
    repository_layout_dirs: Vec<String>,
    metadata_schema: &MetadataSchema,
) -> Result<FoundationRepositoryMetadataAuditBatchResult, ToolResponse> {
    let mut repositories = Vec::new();
    for repository_layout_dir in repository_layout_dirs {
        let audit_result = match execute_foundation_repository_metadata_audit_for_repository(
            repository_layout_dir,
            metadata_schema,
        ) {
            Ok(audit_result) => audit_result,
            Err(error_response) => return Err(error_response),
        };
        repositories.push(build_foundation_repository_metadata_audit_gate_result(
            audit_result,
        ));
    }

    Ok(FoundationRepositoryMetadataAuditBatchResult::new(
        repositories,
    ))
}

// 2026-04-10 CST: 这里抽共享 gate 结果构造 helper，原因是单仓库 gate 与 batch 中的逐仓库结果必须保持完全同一套分级语义，
// 目的：避免 batch 里再次手写 blocking/non_blocking 切分逻辑，导致和单仓库 gate 漂移。
fn build_foundation_repository_metadata_audit_gate_result(
    audit_result: FoundationRepositoryMetadataAuditExecution,
) -> FoundationRepositoryMetadataAuditGateResult {
    let mut blocking_issues = Vec::new();
    let mut non_blocking_issues = Vec::new();
    for issue in audit_result.issues {
        if is_blocking_repository_audit_issue(&issue) {
            blocking_issues.push(issue);
        } else {
            non_blocking_issues.push(issue);
        }
    }

    FoundationRepositoryMetadataAuditGateResult::new(
        audit_result.repository_layout_dir,
        audit_result.repository_schema_version,
        audit_result.metadata_schema_version,
        blocking_issues,
        non_blocking_issues,
    )
}

// 2026-04-10 CST: 这里集中定义 gate 的阻塞分级规则，原因是当前只做通用标准能力，不引入业务化策略或可配置优先级，
// 目的：明确 alias/deprecated 为 non-blocking，其余版本兼容、未知字段与 validator 聚合问题统一视为 blocking。
fn is_blocking_repository_audit_issue(issue: &FoundationRepositoryMetadataAuditIssue) -> bool {
    !matches!(
        issue.kind.as_str(),
        "alias_field_usage" | "deprecated_field_usage"
    )
}

// 2026-04-10 CST: 这里补共享执行结果结构，原因是 audit、gate 与 batch 都需要复用同一份装载后的上下文与 issue 列表，
// 目的：避免多返回值在多个入口间反复拆装，保持字段语义清晰并降低后续扩展成本。
struct FoundationRepositoryMetadataAuditExecution {
    repository_layout_dir: String,
    repository_schema_version: String,
    metadata_schema_version: String,
    issues: Vec<FoundationRepositoryMetadataAuditIssue>,
}

// 2026-04-10 CST: 这里集中做 repository audit issue -> 对外合同映射，原因是内部 issue 枚举更适合 Rust 侧治理表达，
// 对外则需要稳定的 kind 字段与扁平字段结构；目的：避免 CLI / Skill 直接耦合内部枚举细节。
fn map_repository_audit_issue(
    issue: MetadataRepositoryAuditIssue,
) -> FoundationRepositoryMetadataAuditIssue {
    match issue {
        MetadataRepositoryAuditIssue::IncompatibleMetadataSchemaVersion {
            repository_schema_version,
            metadata_schema_version,
        } => FoundationRepositoryMetadataAuditIssue {
            kind: "incompatible_metadata_schema_version".to_string(),
            node_id: None,
            concept_id: None,
            field_key: None,
            canonical_field_key: None,
            replaced_by: None,
            repository_schema_version: Some(repository_schema_version),
            metadata_schema_version: Some(metadata_schema_version),
            actual_value: None,
            expected_type: None,
            allowed_values: None,
        },
        MetadataRepositoryAuditIssue::UnknownMetadataField { node_id, field_key } => {
            FoundationRepositoryMetadataAuditIssue {
                kind: "unknown_metadata_field".to_string(),
                node_id: Some(node_id),
                concept_id: None,
                field_key: Some(field_key),
                canonical_field_key: None,
                replaced_by: None,
                repository_schema_version: None,
                metadata_schema_version: None,
                actual_value: None,
                expected_type: None,
                allowed_values: None,
            }
        }
        MetadataRepositoryAuditIssue::DeprecatedFieldUsage {
            node_id,
            field_key,
            replaced_by,
        } => FoundationRepositoryMetadataAuditIssue {
            kind: "deprecated_field_usage".to_string(),
            node_id: Some(node_id),
            concept_id: None,
            field_key: Some(field_key),
            canonical_field_key: None,
            replaced_by,
            repository_schema_version: None,
            metadata_schema_version: None,
            actual_value: None,
            expected_type: None,
            allowed_values: None,
        },
        MetadataRepositoryAuditIssue::AliasFieldUsage {
            node_id,
            field_key,
            canonical_field_key,
        } => FoundationRepositoryMetadataAuditIssue {
            kind: "alias_field_usage".to_string(),
            node_id: Some(node_id),
            concept_id: None,
            field_key: Some(field_key),
            canonical_field_key: Some(canonical_field_key),
            replaced_by: None,
            repository_schema_version: None,
            metadata_schema_version: None,
            actual_value: None,
            expected_type: None,
            allowed_values: None,
        },
        MetadataRepositoryAuditIssue::ValidationIssue(validation_issue) => {
            map_validation_issue(validation_issue)
        }
    }
}

// 2026-04-10 CST: 这里把 validator issue 扁平化到统一 audit issue 合同，原因是 repository audit 会聚合节点级校验问题，
// 但对外不应该再包一层内部枚举；目的：让调用方按同一套 issue 列表消费整库问题。
fn map_validation_issue(issue: MetadataValidationIssue) -> FoundationRepositoryMetadataAuditIssue {
    match issue {
        MetadataValidationIssue::MissingConceptPolicy {
            node_id,
            concept_id,
        } => FoundationRepositoryMetadataAuditIssue {
            kind: "missing_concept_policy".to_string(),
            node_id: Some(node_id),
            concept_id: Some(concept_id),
            field_key: None,
            canonical_field_key: None,
            replaced_by: None,
            repository_schema_version: None,
            metadata_schema_version: None,
            actual_value: None,
            expected_type: None,
            allowed_values: None,
        },
        MetadataValidationIssue::MissingRequiredField {
            node_id,
            concept_id,
            field_key,
        } => FoundationRepositoryMetadataAuditIssue {
            kind: "missing_required_field".to_string(),
            node_id: Some(node_id),
            concept_id: Some(concept_id),
            field_key: Some(field_key),
            canonical_field_key: None,
            replaced_by: None,
            repository_schema_version: None,
            metadata_schema_version: None,
            actual_value: None,
            expected_type: None,
            allowed_values: None,
        },
        MetadataValidationIssue::DisallowedField {
            node_id,
            concept_id,
            field_key,
        } => FoundationRepositoryMetadataAuditIssue {
            kind: "disallowed_field".to_string(),
            node_id: Some(node_id),
            concept_id: Some(concept_id),
            field_key: Some(field_key),
            canonical_field_key: None,
            replaced_by: None,
            repository_schema_version: None,
            metadata_schema_version: None,
            actual_value: None,
            expected_type: None,
            allowed_values: None,
        },
        MetadataValidationIssue::InvalidAllowedValue {
            node_id,
            field_key,
            actual_value,
            allowed_values,
        } => FoundationRepositoryMetadataAuditIssue {
            kind: "invalid_allowed_value".to_string(),
            node_id: Some(node_id),
            concept_id: None,
            field_key: Some(field_key),
            canonical_field_key: None,
            replaced_by: None,
            repository_schema_version: None,
            metadata_schema_version: None,
            actual_value: Some(actual_value),
            expected_type: None,
            allowed_values: Some(allowed_values),
        },
        MetadataValidationIssue::InvalidValueType {
            node_id,
            field_key,
            expected_type,
            actual_value,
        } => FoundationRepositoryMetadataAuditIssue {
            kind: "invalid_value_type".to_string(),
            node_id: Some(node_id),
            concept_id: None,
            field_key: Some(field_key),
            canonical_field_key: None,
            replaced_by: None,
            repository_schema_version: None,
            metadata_schema_version: None,
            actual_value: Some(actual_value),
            expected_type: Some(format!("{expected_type:?}")),
            allowed_values: None,
        },
    }
}
