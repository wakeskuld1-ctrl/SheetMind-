use chrono::Utc;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

// 2026-04-02 CST: 这里定义正式证券审批包文档，原因是当前审批工件虽然能各自落盘，但还缺一个统一的包级锚点；
// 目的：把 decision_card、approval_request、position_plan、approval_brief 等工件收成正式 package 合同，供后续归档、验签和导出扩展。
// 2026-04-02 CST: 这里补齐 package 合同的反序列化能力，原因是 P0-5 需要从磁盘重新读取正式审批包做核验；
// 目的：让 verify Tool 能直接按正式合同解析 package，而不是退回到松散的 Value 解析。
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct SecurityDecisionPackageDocument {
    pub package_id: String,
    pub contract_version: String,
    pub created_at: String,
    // 2026-04-02 CST: 这里补入 package 版本元数据，原因是 P0-6 要让审批包从“初始提交态”演进成正式版本链；
    // 目的：明确当前 package 属于第几版、基于哪个前版本、为什么产生以及由哪次动作触发。
    pub package_version: u32,
    pub previous_package_path: Option<String>,
    pub revision_reason: String,
    pub trigger_event_summary: String,
    pub scene_name: String,
    pub decision_id: String,
    pub decision_ref: String,
    pub approval_ref: String,
    pub symbol: String,
    pub analysis_date: String,
    pub package_status: String,
    // 2026-04-08 CST: 这里新增显式对象图绑定块，原因是 Task 1 需要把 position_plan / approval_brief 从隐式 artifact 关系升级为正式对象图合同；
    // 目的：让 package 不只知道“有哪些文件”，还知道“这些正式对象彼此如何绑定”，为后续执行层和复盘层扩展预留统一入口。
    pub object_graph: SecurityDecisionPackageObjectGraph,
    pub artifact_manifest: Vec<SecurityDecisionPackageArtifact>,
    pub governance_binding: SecurityDecisionPackageGovernanceBinding,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct SecurityDecisionPackageObjectGraph {
    pub decision_ref: String,
    pub approval_ref: String,
    pub position_plan_ref: String,
    pub approval_brief_ref: String,
    pub scorecard_ref: String,
    pub decision_card_path: String,
    pub approval_request_path: String,
    pub position_plan_path: String,
    pub approval_brief_path: String,
    pub scorecard_path: String,
}

// 2026-04-02 CST: 这里定义审批包中的工件描述，原因是 package 需要引用而不是复制每个原始对象全文；
// 目的：让调用方和后续流程能够通过 role、path、sha256、contract_version 快速定位和校验工件。
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct SecurityDecisionPackageArtifact {
    pub artifact_role: String,
    pub path: String,
    pub sha256: String,
    pub contract_version: String,
    pub required: bool,
    pub present: bool,
}

// 2026-04-02 CST: 这里定义治理绑定信息，原因是审批包不能只知道“有哪些文件”，还要知道它绑定了哪一轮证据与审批上下文；
// 目的：让 package 成为 decision_ref / approval_ref / evidence_hash / governance_hash 的统一锚点。
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct SecurityDecisionPackageGovernanceBinding {
    pub evidence_hash: String,
    pub governance_hash: String,
    pub decision_ref: String,
    pub approval_ref: String,
    pub package_scope: String,
}

// 2026-04-02 CST: 这里定义 package builder 输入，原因是 package 生成时既需要主信息，也需要外部已经算好的工件清单；
// 目的：把包对象构造和提交入口解耦，避免 submit 函数继续膨胀成“大而全的 JSON 拼装器”。
#[derive(Debug, Clone, PartialEq)]
pub struct SecurityDecisionPackageBuildInput {
    pub created_at: String,
    pub package_version: u32,
    pub previous_package_path: Option<String>,
    pub revision_reason: String,
    pub trigger_event_summary: String,
    pub scene_name: String,
    pub decision_id: String,
    pub decision_ref: String,
    pub approval_ref: String,
    pub symbol: String,
    pub analysis_date: String,
    pub decision_status: String,
    pub approval_status: String,
    // 2026-04-08 CST: 这里补入对象图构建输入，原因是 package builder 需要一次性拿到正式对象引用与路径；
    // 目的：把对象图收口在 builder，而不是让 submit / revision 在外部各自拼接，降低后续字段漂移风险。
    pub position_plan_ref: String,
    pub approval_brief_ref: String,
    pub scorecard_ref: String,
    pub decision_card_path: String,
    pub approval_request_path: String,
    pub position_plan_path: String,
    pub approval_brief_path: String,
    pub scorecard_path: String,
    pub evidence_hash: String,
    pub governance_hash: String,
    pub artifact_manifest: Vec<SecurityDecisionPackageArtifact>,
}

// 2026-04-02 CST: 这里集中构造正式审批包，原因是 package 状态、清单和治理绑定不应散落在提交入口多个临时字段里；
// 目的：把审批包变成稳定合同，后续只需要围绕这个 builder 增量扩展。
pub fn build_security_decision_package(
    input: SecurityDecisionPackageBuildInput,
) -> SecurityDecisionPackageDocument {
    SecurityDecisionPackageDocument {
        package_id: format!("pkg-{}", input.decision_id),
        contract_version: "security_decision_package.v1".to_string(),
        created_at: normalize_created_at(&input.created_at),
        package_version: input.package_version.max(1),
        previous_package_path: input.previous_package_path,
        revision_reason: input.revision_reason,
        trigger_event_summary: input.trigger_event_summary,
        scene_name: input.scene_name,
        decision_id: input.decision_id,
        decision_ref: input.decision_ref.clone(),
        approval_ref: input.approval_ref.clone(),
        symbol: input.symbol,
        analysis_date: input.analysis_date,
        package_status: derive_package_status(&input.decision_status, &input.approval_status),
        object_graph: SecurityDecisionPackageObjectGraph {
            decision_ref: input.decision_ref.clone(),
            approval_ref: input.approval_ref.clone(),
            position_plan_ref: input.position_plan_ref,
            approval_brief_ref: input.approval_brief_ref,
            scorecard_ref: input.scorecard_ref,
            decision_card_path: input.decision_card_path,
            approval_request_path: input.approval_request_path,
            position_plan_path: input.position_plan_path,
            approval_brief_path: input.approval_brief_path,
            scorecard_path: input.scorecard_path,
        },
        artifact_manifest: input.artifact_manifest,
        governance_binding: SecurityDecisionPackageGovernanceBinding {
            evidence_hash: input.evidence_hash,
            governance_hash: input.governance_hash,
            decision_ref: input.decision_ref,
            approval_ref: input.approval_ref,
            package_scope: "security_decision_submit_approval".to_string(),
        },
    }
}

// 2026-04-02 CST: 这里封装 JSON payload 的 sha256 计算，原因是审批包 manifest 需要稳定哈希但不想重复读回文件；
// 目的：直接基于落盘前 payload 生成一致摘要，减少 I/O 并保持 manifest 构造简单。
pub fn sha256_for_json_value(value: &serde_json::Value) -> Result<String, String> {
    let payload = serde_json::to_vec(value).map_err(|error| error.to_string())?;
    Ok(sha256_for_bytes(&payload))
}

// 2026-04-02 CST: 这里封装字节级 sha256，原因是 audit_log 等工件不是标准 JSON 对象数组，而是 JSONL 文本；
// 目的：让 package 既能覆盖 JSON 文件，也能覆盖文本型审计工件。
pub fn sha256_for_bytes(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

fn derive_package_status(decision_status: &str, approval_status: &str) -> String {
    match (decision_status, approval_status) {
        (_, "Approved") => "approved_bundle_ready".to_string(),
        (_, "Rejected") => "rejected_bundle_ready".to_string(),
        (_, "ApprovedWithOverride") => "approved_with_override_bundle_ready".to_string(),
        (_, "NeedsMoreEvidence") => "needs_follow_up".to_string(),
        ("blocked", _) => "needs_follow_up".to_string(),
        ("ready_for_review", "Pending") => "review_bundle_ready".to_string(),
        _ => "pending_review_materials".to_string(),
    }
}

fn normalize_created_at(value: &str) -> String {
    if value.trim().is_empty() {
        Utc::now().to_rfc3339()
    } else {
        value.trim().to_string()
    }
}
