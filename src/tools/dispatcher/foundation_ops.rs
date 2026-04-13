use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use serde_json::Value;

use crate::ops::foundation::capability_router::NavigationRequest;
use crate::ops::foundation::knowledge_ingestion::{
    KnowledgeIngestionError, load_repository_from_json_path, load_repository_from_jsonl_path,
};

// 2026-04-13 CST: 这里新增 foundation design skeleton 的正式 dispatcher 入口，原因是方案C要求把
// “开发前先做设计骨架”从规则提升为正式 Tool；
// 目的：让上层能稳定拿到 summary、Mermaid 图和 warning，而不是只依赖自由文本设计说明。
pub(super) fn dispatch_foundation_design_skeleton(args: Value) -> ToolResponse {
    let request = match serde_json::from_value::<FoundationDesignSkeletonRequest>(args) {
        Ok(request) => request,
        Err(error) => return ToolResponse::error(format!("request parsing failed: {error}")),
    };

    match build_foundation_design_skeleton_result(&request) {
        Ok(result) => ToolResponse::ok_serialized(&result),
        Err(error) => ToolResponse::error(error),
    }
}

// 2026-04-13 CST: 这里新增 foundation design gap audit 的正式 dispatcher 入口，原因是方案C要求设计骨架
// 与 graphify 现状图联动，形成“设计 vs 成品”的标准审计入口；
// 目的：让上层稳定消费差距结果，而不是每次人工读 graph.json 对比。
pub(super) fn dispatch_foundation_design_gap_audit(args: Value) -> ToolResponse {
    let request = match serde_json::from_value::<FoundationDesignGapAuditRequest>(args) {
        Ok(request) => request,
        Err(error) => return ToolResponse::error(format!("request parsing failed: {error}")),
    };

    match build_foundation_design_gap_audit_result(&request) {
        Ok(result) => ToolResponse::ok_serialized(&result),
        Err(error) => ToolResponse::error(error),
    }
}
use crate::ops::foundation::knowledge_repository::{
    KnowledgeRepository, MetadataRepositoryAuditIssue,
};
use crate::ops::foundation::metadata_schema::MetadataSchema;
use crate::ops::foundation::metadata_validator::MetadataValidationIssue;
use crate::ops::foundation::navigation_pipeline::{NavigationPipeline, NavigationPipelineError};
use crate::ops::foundation::roaming_engine::RoamingPlan;
use crate::tools::contracts::{
    FoundationDesignGapAuditRequest, FoundationDesignGapAuditResult, FoundationDesignGapCheck,
    FoundationDesignInterfaceContract, FoundationDesignLayerContract,
    FoundationDesignMethodContract, FoundationDesignModuleContract,
    FoundationDesignSkeletonRequest, FoundationDesignSkeletonResult,
    FoundationDesignVisualArtifacts, FoundationNavigationHit, FoundationNavigationRequest,
    FoundationNavigationResult, FoundationNavigationRoamingStep,
    FoundationRepositoryImportGateRequest, FoundationRepositoryImportGateResult,
    FoundationRepositoryMetadataAuditBatchRequest, FoundationRepositoryMetadataAuditBatchResult,
    FoundationRepositoryMetadataAuditExportRequest, FoundationRepositoryMetadataAuditExportResult,
    FoundationRepositoryMetadataAuditGateResult, FoundationRepositoryMetadataAuditIssue,
    FoundationRepositoryMetadataAuditRequest, FoundationRepositoryMetadataAuditResult,
    MetadataSchemaContract, ToolResponse,
};

// 2026-04-13 CST: 这里新增 foundation navigation 的正式 dispatcher 入口，原因是 B1 的目标是把现有 route/roam/retrieve/assemble
// 内核收口成可调用的通用 Tool，而不是继续停留在模块级闭环；目的：让 CLI / Skill 直接消费知识漫游主线。
pub(super) fn dispatch_foundation_navigation(args: Value) -> ToolResponse {
    let request = match serde_json::from_value::<FoundationNavigationRequest>(args) {
        Ok(request) => request,
        Err(error) => return ToolResponse::error(format!("request parsing failed: {error}")),
    };

    let ontology_store = match request.knowledge_bundle.to_ontology_store() {
        Ok(store) => store,
        Err(error) => {
            return ToolResponse::error(format!("ontology schema build failed: {error:?}"));
        }
    };
    let graph_store = request.knowledge_bundle.to_graph_store();
    let roaming_plan_template = RoamingPlan::new(vec![])
        .with_allowed_relation_types(request.allowed_relation_types.clone())
        .with_max_depth(request.max_depth)
        .with_max_concepts(request.max_concepts);
    let pipeline = NavigationPipeline::new(ontology_store, graph_store, roaming_plan_template);
    let evidence = match pipeline.run(&NavigationRequest::new(request.question.clone())) {
        Ok(evidence) => evidence,
        Err(error) => return map_navigation_pipeline_error(request.question, error),
    };

    let roaming_path = evidence
        .roaming_path
        .into_iter()
        .map(|step| {
            FoundationNavigationRoamingStep::new(
                step.from_concept_id,
                step.to_concept_id,
                step.relation_type,
                step.depth,
            )
        })
        .collect();
    let hits = evidence
        .hits
        .into_iter()
        .map(|hit| FoundationNavigationHit::new(hit.node_id, hit.score, hit.evidence_refs))
        .collect();
    let result = FoundationNavigationResult::new(
        evidence.route.matched_concept_ids,
        roaming_path,
        hits,
        evidence.citations,
        evidence.summary,
    );

    ToolResponse::ok_serialized(&result)
}

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

// 2026-04-13 CST: 这里新增 foundation repository metadata audit export 分发入口，原因是方案A本轮交付的是
// “schema_path + bundle_path -> stable DTO” 的文件输入边界，而不是替换既有 layout_dir 审计入口；
// 目的：复用当前已有 ingestion + repository audit 主线，让外部直接消费标准文件产物。
pub(super) fn dispatch_foundation_repository_metadata_audit_export(args: Value) -> ToolResponse {
    let request =
        match serde_json::from_value::<FoundationRepositoryMetadataAuditExportRequest>(args) {
            Ok(request) => request,
            Err(error) => return ToolResponse::error(format!("request parsing failed: {error}")),
        };

    let schema_path = match normalize_non_blank_path(request.schema_path, "schema_path") {
        Ok(path) => path,
        Err(error_response) => return error_response,
    };
    let bundle_path = match normalize_non_blank_path(request.bundle_path, "bundle_path") {
        Ok(path) => path,
        Err(error_response) => return error_response,
    };

    let metadata_schema = match load_metadata_schema_from_json_path(Path::new(&schema_path)) {
        Ok(metadata_schema) => metadata_schema,
        Err(error_response) => return error_response,
    };
    let (bundle_format, repository) =
        match load_repository_from_bundle_path(Path::new(&bundle_path)) {
            Ok(result) => result,
            Err(error_response) => return error_response,
        };
    let audit_result = execute_foundation_repository_metadata_audit_for_loaded_repository(
        repository,
        &metadata_schema,
    );

    ToolResponse::ok_serialized(&FoundationRepositoryMetadataAuditExportResult::new(
        schema_path,
        bundle_path,
        bundle_format,
        audit_result.repository_schema_version,
        audit_result.metadata_schema_version,
        audit_result.issues,
    ))
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
    Ok(
        execute_foundation_repository_metadata_audit_for_loaded_repository(
            repository,
            metadata_schema,
        )
        .with_repository_layout_dir(repository_layout_dir),
    )
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

impl FoundationRepositoryMetadataAuditExecution {
    // 2026-04-13 CST: 这里补一个 layout dir 回填 helper，原因是 export tool 与 layout dir tool 共用同一份
    // repository audit 执行结果，但只有 layout dir 边界需要携带目录字段；
    // 目的：避免为了复用逻辑复制两套 issue 映射与汇总代码。
    fn with_repository_layout_dir(mut self, repository_layout_dir: String) -> Self {
        self.repository_layout_dir = repository_layout_dir;
        self
    }
}

// 2026-04-13 CST: 这里抽取“已加载 repository -> audit execution” helper，原因是 export tool 与原有 layout dir tool
// 共享同一条 repository 审计主线；
// 目的：在不改 audit 语义的前提下复用 issue 映射逻辑，避免两条边界各自复制一份实现。
fn execute_foundation_repository_metadata_audit_for_loaded_repository(
    repository: KnowledgeRepository,
    metadata_schema: &MetadataSchema,
) -> FoundationRepositoryMetadataAuditExecution {
    let issues = repository
        .audit_metadata(metadata_schema)
        .issues
        .into_iter()
        .map(map_repository_audit_issue)
        .collect::<Vec<_>>();

    FoundationRepositoryMetadataAuditExecution {
        repository_layout_dir: String::new(),
        repository_schema_version: repository.bundle().schema_version.clone(),
        metadata_schema_version: metadata_schema.schema_version.clone(),
        issues,
    }
}

// 2026-04-13 CST: 这里集中做字符串路径 trim + fail-fast，原因是方案边界明确要求 dispatcher 层先拦截空白路径，
// 目的：避免把空白参数交给底层文件系统后产生不稳定、不可读的错误信息。
fn normalize_non_blank_path(raw_path: String, field_name: &str) -> Result<String, ToolResponse> {
    let normalized = raw_path.trim().to_string();
    if normalized.is_empty() {
        return Err(ToolResponse::error(format!("{field_name} cannot be blank")));
    }

    Ok(normalized)
}

// 2026-04-13 CST: 这里集中装载 metadata schema 文件，原因是 export tool 的 schema 输入边界是 JSON 文件，
// 目的：先把文件读取、JSON 反序列化与 schema build 失败拆清楚，再复用现有 MetadataSchemaContract 构建主线。
fn load_metadata_schema_from_json_path(path: &Path) -> Result<MetadataSchema, ToolResponse> {
    let raw = std::fs::read_to_string(path)
        .map_err(|error| ToolResponse::error(format!("metadata schema read failed: {error}")))?;
    let contract = serde_json::from_str::<MetadataSchemaContract>(&raw).map_err(|error| {
        ToolResponse::error(format!("metadata schema deserialize failed: {error}"))
    })?;

    contract
        .build_schema()
        .map_err(|error| ToolResponse::error(format!("metadata schema build failed: {error:?}")))
}

// 2026-04-13 CST: 这里集中按扩展名分流 bundle 文件装载，原因是方案边界要求 `bundle_path` 同时支持 `.json` 与 `.jsonl`，
// 目的：在 dispatcher 边界显式固定受支持格式，避免上层误以为任意文件后缀都可直接导入。
fn load_repository_from_bundle_path(
    path: &Path,
) -> Result<(String, KnowledgeRepository), ToolResponse> {
    match path.extension().and_then(|extension| extension.to_str()) {
        Some("json") => Ok((
            "json".to_string(),
            load_repository_from_json_path(path).map_err(map_repository_ingestion_error(
                "repository bundle json load failed",
            ))?,
        )),
        Some("jsonl") => Ok((
            "jsonl".to_string(),
            load_repository_from_jsonl_path(path).map_err(map_repository_ingestion_error(
                "repository bundle jsonl load failed",
            ))?,
        )),
        _ => Err(ToolResponse::error(
            "bundle_path must end with .json or .jsonl",
        )),
    }
}

// 2026-04-13 CST: 这里集中映射 ingestion 错误，原因是 export tool 复用了标准 bundle 导入主线，
// 目的：保留底层失败语义，同时给 CLI 返回更容易定位是哪个装载阶段出错的上下文。
fn map_repository_ingestion_error(
    context: &'static str,
) -> impl FnOnce(KnowledgeIngestionError) -> ToolResponse {
    move |error| ToolResponse::error(format!("{context}: {error:?}"))
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

// 2026-04-13 CST: 这里集中映射 foundation navigation pipeline 错误，原因是 Tool 层需要稳定暴露“哪一环失败”的可读错误，
// 目的：避免把内部错误类型直接暴露给外部，又保留 route/roam/retrieve 三段的故障边界。
fn map_navigation_pipeline_error(question: String, error: NavigationPipelineError) -> ToolResponse {
    match error {
        NavigationPipelineError::RouteFailed { .. } => ToolResponse::error(format!(
            "foundation navigation route failed for question: {question}"
        )),
        NavigationPipelineError::RoamFailed { .. } => ToolResponse::error(format!(
            "foundation navigation roam failed for question: {question}"
        )),
        NavigationPipelineError::RetrieveFailed { .. } => ToolResponse::error(format!(
            "foundation navigation retrieve failed for question: {question}"
        )),
    }
}

// 2026-04-13 CST: 这里集中构建 design skeleton 结果，原因是 dispatcher 不应内联层级校验、
// Mermaid 生成与 warning 汇总逻辑；
// 目的：把设计骨架的最小标准输出稳定收口在单点。
// 2026-04-13 CST: 这里集中构建 design skeleton 结果，原因是 dispatcher 不应内联校验、
// warning 汇总和辅助可视化拼装；
// 2026-04-13 CST 追加：按用户要求把 JSON 结构作为主边界，Mermaid 降级为可选 visuals 辅助块。
fn build_foundation_design_skeleton_result(
    request: &FoundationDesignSkeletonRequest,
) -> Result<FoundationDesignSkeletonResult, String> {
    validate_foundation_design_request(request)?;
    let warnings = collect_foundation_design_warnings(request);
    let summary = format!(
        "Feature `{}` covers {} layer(s), {} module(s), {} interface(s), {} method(s), {} test scenario(s).",
        request.feature_name,
        request.layers.len(),
        request.modules.len(),
        request.interfaces.len(),
        request.methods.len(),
        request.test_scenarios.len(),
    );

    Ok(FoundationDesignSkeletonResult {
        feature_name: request.feature_name.clone(),
        summary,
        warnings,
        layer_count: request.layers.len(),
        module_count: request.modules.len(),
        interface_count: request.interfaces.len(),
        method_count: request.methods.len(),
        test_scenario_count: request.test_scenarios.len(),
        // 2026-04-13 CST: 这里继续保留 Mermaid，是为了交接和人工扫图时可直接复用；
        // 目的：不影响 JSON-first 的主边界，同时避免重新单做一套图转换工具。
        visuals: Some(FoundationDesignVisualArtifacts {
            layer_diagram_mermaid: Some(build_layer_diagram_mermaid(&request.layers)),
            dependency_diagram_mermaid: Some(build_dependency_diagram_mermaid(&request.modules)),
            interface_diagram_mermaid: Some(build_interface_diagram_mermaid(
                &request.modules,
                &request.interfaces,
                &request.methods,
            )),
        }),
    })
}

// 2026-04-13 CST: 这里集中构建 design gap audit 结果，原因是差距审计需要复用 design skeleton 校验、
// graphify 图谱发现、节点标准化与逐项命中比对；
// 目的：让“设计 vs 成品”审计逻辑保持单点实现，避免 dispatcher 和测试各自拼装。
fn build_foundation_design_gap_audit_result(
    request: &FoundationDesignGapAuditRequest,
) -> Result<FoundationDesignGapAuditResult, String> {
    validate_foundation_design_request(&request.skeleton)?;
    let graph_path = resolve_foundation_design_graph_path(request.graph_path.as_deref())?;
    let graph_nodes = load_graph_nodes_from_path(&graph_path)?;
    let graph_index = build_graph_match_index(&graph_nodes);

    let module_checks = request
        .skeleton
        .modules
        .iter()
        .map(|module| build_module_gap_check(module, &graph_index))
        .collect::<Vec<_>>();
    let interface_checks = request
        .skeleton
        .interfaces
        .iter()
        .map(|interface| {
            build_named_gap_check(
                interface.id.clone(),
                interface.label.clone(),
                interface_match_keys(interface),
                &graph_index,
            )
        })
        .collect::<Vec<_>>();
    let method_checks = request
        .skeleton
        .methods
        .iter()
        .map(|method| {
            build_named_gap_check(
                method.id.clone(),
                method.label.clone(),
                method_match_keys(method),
                &graph_index,
            )
        })
        .collect::<Vec<_>>();

    let missing_modules = module_checks
        .iter()
        .filter(|check| !check.matched)
        .map(|check| check.design_id.clone())
        .collect::<Vec<_>>();
    let missing_interfaces = interface_checks
        .iter()
        .filter(|check| !check.matched)
        .map(|check| check.design_id.clone())
        .collect::<Vec<_>>();
    let missing_methods = method_checks
        .iter()
        .filter(|check| !check.matched)
        .map(|check| check.design_id.clone())
        .collect::<Vec<_>>();

    let mut warnings = collect_foundation_design_warnings(&request.skeleton);
    if graph_nodes.is_empty() {
        warnings.push("graph.json contains zero nodes".to_string());
    }
    if !missing_modules.is_empty() {
        warnings.push(format!(
            "missing modules in implementation graph: {}",
            missing_modules.join(", ")
        ));
    }
    if !missing_interfaces.is_empty() {
        warnings.push(format!(
            "missing interfaces in implementation graph: {}",
            missing_interfaces.join(", ")
        ));
    }
    if !missing_methods.is_empty() {
        warnings.push(format!(
            "missing methods in implementation graph: {}",
            missing_methods.join(", ")
        ));
    }

    Ok(FoundationDesignGapAuditResult {
        feature_name: request.skeleton.feature_name.clone(),
        graph_path: graph_path.to_string_lossy().to_string(),
        matched_module_count: module_checks.iter().filter(|check| check.matched).count(),
        matched_interface_count: interface_checks
            .iter()
            .filter(|check| check.matched)
            .count(),
        matched_method_count: method_checks.iter().filter(|check| check.matched).count(),
        module_checks,
        interface_checks,
        method_checks,
        missing_modules,
        missing_interfaces,
        missing_methods,
        warnings,
    })
}

// 2026-04-13 CST: 这里集中校验 design skeleton 请求，原因是所有设计相关 Tool 都依赖同一套最小边界约束；
// 目的：尽早拦截空 feature 名称、重复 id 与悬空引用，避免后续 Mermaid 或 gap audit 输出失真。
fn validate_foundation_design_request(
    request: &FoundationDesignSkeletonRequest,
) -> Result<(), String> {
    if request.feature_name.trim().is_empty() {
        return Err("feature_name cannot be blank".to_string());
    }
    if request.objective.trim().is_empty() {
        return Err("objective cannot be blank".to_string());
    }

    ensure_unique_design_ids(request.layers.iter().map(|item| item.id.as_str()), "layer")?;
    ensure_unique_design_ids(
        request.modules.iter().map(|item| item.id.as_str()),
        "module",
    )?;
    ensure_unique_design_ids(
        request.interfaces.iter().map(|item| item.id.as_str()),
        "interface",
    )?;
    ensure_unique_design_ids(
        request.methods.iter().map(|item| item.id.as_str()),
        "method",
    )?;

    let layer_ids = request
        .layers
        .iter()
        .map(|item| item.id.as_str())
        .collect::<BTreeSet<_>>();
    let module_ids = request
        .modules
        .iter()
        .map(|item| item.id.as_str())
        .collect::<BTreeSet<_>>();
    let interface_ids = request
        .interfaces
        .iter()
        .map(|item| item.id.as_str())
        .collect::<BTreeSet<_>>();

    for layer in &request.layers {
        for dependency in &layer.depends_on {
            if !layer_ids.contains(dependency.as_str()) {
                return Err(format!(
                    "layer `{}` depends_on unknown layer `{dependency}`",
                    layer.id
                ));
            }
        }
    }
    for module in &request.modules {
        if !layer_ids.contains(module.layer_id.as_str()) {
            return Err(format!(
                "module `{}` references unknown layer `{}`",
                module.id, module.layer_id
            ));
        }
        for dependency in &module.depends_on {
            if !module_ids.contains(dependency.as_str()) {
                return Err(format!(
                    "module `{}` depends_on unknown module `{dependency}`",
                    module.id
                ));
            }
        }
    }
    for interface in &request.interfaces {
        if !module_ids.contains(interface.module_id.as_str()) {
            return Err(format!(
                "interface `{}` references unknown module `{}`",
                interface.id, interface.module_id
            ));
        }
    }
    for method in &request.methods {
        if !interface_ids.contains(method.interface_id.as_str()) {
            return Err(format!(
                "method `{}` references unknown interface `{}`",
                method.id, method.interface_id
            ));
        }
    }

    Ok(())
}

// 2026-04-13 CST: 这里统一做设计 warning 收集，原因是部分问题不应该直接 fail-fast，
// 例如空 success criteria、缺 source_files、缺测试场景；
// 目的：让 Skeleton Tool 和 gap audit Tool 共享同一套软性提醒语义。
fn collect_foundation_design_warnings(request: &FoundationDesignSkeletonRequest) -> Vec<String> {
    let mut warnings = Vec::new();

    if request.success_criteria.is_empty() {
        warnings.push("success_criteria is empty".to_string());
    }
    if request.test_scenarios.is_empty() {
        warnings.push("test_scenarios is empty".to_string());
    }
    for module in &request.modules {
        if module.source_files.is_empty() {
            warnings.push(format!("module `{}` has no source_files", module.id));
        }
    }

    warnings
}

fn ensure_unique_design_ids<'a>(
    ids: impl Iterator<Item = &'a str>,
    kind: &str,
) -> Result<(), String> {
    let mut seen = BTreeSet::new();
    for id in ids {
        if id.trim().is_empty() {
            return Err(format!("{kind} id cannot be blank"));
        }
        if !seen.insert(id.to_string()) {
            return Err(format!("duplicate {kind} id `{id}`"));
        }
    }

    Ok(())
}

fn build_layer_diagram_mermaid(layers: &[FoundationDesignLayerContract]) -> String {
    let mut lines = vec!["flowchart TD".to_string()];
    for layer in layers {
        lines.push(format!(
            "    {}[\"{}\"]",
            mermaid_id(&layer.id),
            escape_mermaid_label(&layer.label)
        ));
    }
    for layer in layers {
        for dependency in &layer.depends_on {
            lines.push(format!(
                "    {} --> {}",
                mermaid_id(&layer.id),
                mermaid_id(dependency)
            ));
        }
    }
    lines.join("\n")
}

fn build_dependency_diagram_mermaid(modules: &[FoundationDesignModuleContract]) -> String {
    let mut lines = vec!["flowchart LR".to_string()];
    for module in modules {
        lines.push(format!(
            "    {}[\"{}\"]",
            mermaid_id(&module.id),
            escape_mermaid_label(&format!("{} ({})", module.label, module.layer_id))
        ));
    }
    for module in modules {
        for dependency in &module.depends_on {
            lines.push(format!(
                "    {} --> {}",
                mermaid_id(&module.id),
                mermaid_id(dependency)
            ));
        }
    }
    lines.join("\n")
}

fn build_interface_diagram_mermaid(
    modules: &[FoundationDesignModuleContract],
    interfaces: &[FoundationDesignInterfaceContract],
    methods: &[FoundationDesignMethodContract],
) -> String {
    let mut lines = vec!["flowchart TD".to_string()];
    for module in modules {
        lines.push(format!(
            "    {}[\"{}\"]",
            mermaid_id(&module.id),
            escape_mermaid_label(&module.label)
        ));
    }
    for interface in interfaces {
        let interface_node_id = format!("iface_{}", mermaid_id(&interface.id));
        lines.push(format!(
            "    {}[\"{}: {}\"]",
            interface_node_id,
            escape_mermaid_label(&interface.kind),
            escape_mermaid_label(&interface.label)
        ));
        lines.push(format!(
            "    {} --> {}",
            mermaid_id(&interface.module_id),
            interface_node_id
        ));
    }
    for method in methods {
        let method_node_id = format!("method_{}", mermaid_id(&method.id));
        lines.push(format!(
            "    {}[\"{}\"]",
            method_node_id,
            escape_mermaid_label(&method.label)
        ));
        lines.push(format!(
            "    iface_{} --> {}",
            mermaid_id(&method.interface_id),
            method_node_id
        ));
    }
    lines.join("\n")
}

fn mermaid_id(value: &str) -> String {
    value
        .chars()
        .map(|ch| if ch.is_ascii_alphanumeric() { ch } else { '_' })
        .collect()
}

fn escape_mermaid_label(value: &str) -> String {
    value.replace('"', "'")
}

fn resolve_foundation_design_graph_path(graph_path: Option<&str>) -> Result<PathBuf, String> {
    if let Some(graph_path) = graph_path {
        let normalized = graph_path.trim();
        if normalized.is_empty() {
            return Err("graph_path cannot be blank".to_string());
        }
        return Ok(PathBuf::from(normalized));
    }

    discover_latest_src_graph_path().ok_or_else(|| {
        "graph_path missing and no graphify-out/src-map-*/graph.json could be discovered"
            .to_string()
    })
}

fn discover_latest_src_graph_path() -> Option<PathBuf> {
    let graphify_out_dir = Path::new("graphify-out");
    let mut candidates = fs::read_dir(graphify_out_dir)
        .ok()?
        .filter_map(Result::ok)
        .filter_map(|entry| {
            let path = entry.path();
            let name = path.file_name()?.to_str()?.to_string();
            if !path.is_dir() || !name.starts_with("src-map-") {
                return None;
            }
            let graph_path = path.join("graph.json");
            if graph_path.exists() {
                Some(graph_path)
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    candidates.sort();
    candidates.pop()
}

fn load_graph_nodes_from_path(path: &Path) -> Result<Vec<GraphNodeMatch>, String> {
    let raw =
        fs::read_to_string(path).map_err(|error| format!("graph json read failed: {error}"))?;
    let payload = serde_json::from_str::<Value>(&raw)
        .map_err(|error| format!("graph json deserialize failed: {error}"))?;
    let nodes = payload
        .get("nodes")
        .and_then(Value::as_array)
        .ok_or_else(|| "graph json missing `nodes` array".to_string())?;

    Ok(nodes
        .iter()
        .map(|node| GraphNodeMatch {
            id: node
                .get("id")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string(),
            label: node
                .get("label")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string(),
            source_file: node
                .get("source_file")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string(),
        })
        .collect())
}

fn build_graph_match_index(nodes: &[GraphNodeMatch]) -> GraphMatchIndex {
    let mut token_to_labels = BTreeMap::<String, BTreeSet<String>>::new();
    for node in nodes {
        for token in graph_node_match_tokens(node) {
            token_to_labels
                .entry(token)
                .or_default()
                .insert(node.label.clone());
        }
    }

    GraphMatchIndex { token_to_labels }
}

fn build_module_gap_check(
    module: &FoundationDesignModuleContract,
    graph_index: &GraphMatchIndex,
) -> FoundationDesignGapCheck {
    let mut match_keys = vec![
        normalize_design_token(&module.id),
        normalize_design_token(&module.label),
    ];
    for source_file in &module.source_files {
        match_keys.push(normalize_design_token(source_file));
        if let Some(file_name) = Path::new(source_file)
            .file_name()
            .and_then(|name| name.to_str())
        {
            match_keys.push(normalize_design_token(file_name));
        }
        if let Some(stem) = Path::new(source_file)
            .file_stem()
            .and_then(|stem| stem.to_str())
        {
            match_keys.push(normalize_design_token(stem));
        }
    }

    build_named_gap_check(
        module.id.clone(),
        module.label.clone(),
        match_keys,
        graph_index,
    )
}

fn build_named_gap_check(
    design_id: String,
    design_label: String,
    match_keys: Vec<String>,
    graph_index: &GraphMatchIndex,
) -> FoundationDesignGapCheck {
    let mut matched_labels = BTreeSet::new();
    for key in match_keys.into_iter().filter(|item| !item.is_empty()) {
        if let Some(labels) = graph_index.token_to_labels.get(&key) {
            matched_labels.extend(labels.iter().cloned());
        }
    }

    FoundationDesignGapCheck {
        design_id,
        design_label,
        matched: !matched_labels.is_empty(),
        matched_node_labels: matched_labels.into_iter().collect(),
    }
}

fn interface_match_keys(interface: &FoundationDesignInterfaceContract) -> Vec<String> {
    vec![
        normalize_design_token(&interface.id),
        normalize_design_token(&interface.label),
        normalize_design_token(&format!("{}{}", interface.kind, interface.label)),
    ]
}

fn method_match_keys(method: &FoundationDesignMethodContract) -> Vec<String> {
    let normalized_label = normalize_design_token(&method.label);
    let normalized_without_parens = normalize_design_token(method.label.trim_end_matches("()"));
    vec![
        normalize_design_token(&method.id),
        normalized_label.clone(),
        normalized_without_parens.clone(),
        normalize_design_token(&format!("{}()", normalized_without_parens)),
    ]
}

fn normalize_design_token(value: &str) -> String {
    value
        .to_ascii_lowercase()
        .replace('\\', "/")
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric())
        .collect()
}

fn graph_node_match_tokens(node: &GraphNodeMatch) -> BTreeSet<String> {
    let mut tokens = BTreeSet::new();
    tokens.insert(normalize_design_token(&node.id));
    tokens.insert(normalize_design_token(&node.label));
    tokens.insert(normalize_design_token(&node.source_file));

    if let Some(file_name) = Path::new(&node.source_file)
        .file_name()
        .and_then(|name| name.to_str())
    {
        tokens.insert(normalize_design_token(file_name));
    }
    if let Some(stem) = Path::new(&node.source_file)
        .file_stem()
        .and_then(|stem| stem.to_str())
    {
        tokens.insert(normalize_design_token(stem));
    }
    if let Some(label_without_parens) = node.label.strip_suffix("()") {
        tokens.insert(normalize_design_token(label_without_parens));
    }

    tokens
}

struct GraphNodeMatch {
    id: String,
    label: String,
    source_file: String,
}

struct GraphMatchIndex {
    token_to_labels: BTreeMap<String, BTreeSet<String>>,
}
