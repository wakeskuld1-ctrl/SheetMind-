use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::ops::foundation::metadata_schema::{
    ConceptMetadataPolicy, MetadataFieldDefinition, MetadataSchema, MetadataSchemaError,
};
use crate::ops::stock::security_decision_briefing::PositionPlan;
use crate::tools::catalog;

// 2026-04-08 CST: 这里给 ToolRequest 补序列化能力，原因是七席委员会子进程需要把内部 tool 请求重新编码后写入 CLI stdin；
// 目的：让正式 dispatcher 合同既能接收外部请求，也能被投决会内部 seat agent 复用，避免再造第二套进程内协议。
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ToolRequest {
    pub tool: String,
    #[serde(default)]
    pub args: Value,
}

// 2026-04-08 CST: 这里给 ToolResponse 补反序列化能力，原因是七席委员会父进程需要把子进程返回的 JSON 安全回读成正式响应；
// 目的：确保独立执行证明链仍沿用现有 ToolResponse 合同，而不是在 committee 内部额外拼装弱类型 JSON。
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ToolResponse {
    pub status: String,
    pub data: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl ToolResponse {
    pub fn ok(data: Value) -> Self {
        Self {
            status: "ok".to_string(),
            data,
            error: None,
        }
    }

    // 2026-04-02 CST: 这里补一个强类型序列化入口，原因是 security_decision_briefing 后续会引入更厚的结构化响应，不适合在每个 dispatcher 分支重复手写 `json!(result)`；
    // 目的：让 Tool 层可以直接复用 serde 序列化结果，统一合同输出路径并减少重复样板。
    pub fn ok_serialized<T: Serialize>(data: &T) -> Self {
        let serialized =
            serde_json::to_value(data).expect("tool response serialization should succeed");
        Self::ok(serialized)
    }

    pub fn needs_confirmation(data: Value) -> Self {
        Self {
            status: "needs_confirmation".to_string(),
            data,
            error: None,
        }
    }

    pub fn error(message: impl Into<String>) -> Self {
        Self {
            status: "error".to_string(),
            data: json!({}),
            error: Some(message.into()),
        }
    }

    pub fn tool_catalog() -> Self {
        Self::ok(json!({
            "tool_catalog": catalog::tool_names(),
            // 2026-03-31 CST: 这里把分模块目录一并暴露给外部，原因是当前项目已经明确分成 foundation / stock 两个能力域。
            // 目的：在不破坏原有平铺 tool_catalog 契约的前提下，为 AI、GUI 和后续路由提供稳定的模块边界元数据。
            "tool_catalog_modules": {
                "foundation": catalog::foundation_tool_names(),
                "stock": catalog::stock_tool_names(),
            }
        }))
    }
}

// 2026-04-08 CST: 这里新增仓位计划记录请求合同，原因是证券主链后续要把 briefing 子层 `position_plan`
// 正式升级成可引用对象；目的：先把 decision/approval/evidence 绑定和计划快照字段固定下来，便于后续 Tool 与 runtime 实现复用同一份类型。
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct SecurityPositionPlanRecordRequest {
    pub decision_ref: String,
    pub approval_ref: String,
    pub evidence_version: String,
    pub symbol: String,
    pub analysis_date: String,
    pub position_plan: PositionPlan,
}

// 2026-04-08 CST: 这里新增仓位计划记录响应合同，原因是 Task 1 红测要求先锁定正式 record 对象的最小字段骨架；
// 目的：让后续 position_plan_record Tool 即使还没接持久化，也必须先沿稳定的结构化合同对外返回引用与核心仓位边界。
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct SecurityPositionPlanRecordResult {
    pub position_plan_ref: String,
    pub decision_ref: String,
    pub approval_ref: String,
    pub evidence_version: String,
    pub symbol: String,
    pub analysis_date: String,
    pub position_action: String,
    pub starter_position_pct: f64,
    pub max_position_pct: f64,
    pub position_plan: PositionPlan,
}

impl SecurityPositionPlanRecordResult {
    // 2026-04-08 CST: 这里补最小构造辅助函数，原因是 Task 1 只需要先证明 record 合同可由 briefing 仓位计划稳定投影；
    // 目的：把 `position_action / starter_position_pct / max_position_pct` 的提取收口到单点，避免后续 Tool 与测试夹具各自再拼一次。
    pub fn from_position_plan(
        position_plan_ref: String,
        request: SecurityPositionPlanRecordRequest,
    ) -> Self {
        let (position_action, starter_position_pct, max_position_pct) =
            request.position_plan.record_projection();

        Self {
            position_plan_ref,
            decision_ref: request.decision_ref,
            approval_ref: request.approval_ref,
            evidence_version: request.evidence_version,
            symbol: request.symbol,
            analysis_date: request.analysis_date,
            position_action: position_action.to_string(),
            starter_position_pct,
            max_position_pct,
            position_plan: request.position_plan,
        }
    }
}

// 2026-04-08 CST: 这里新增调仓事件类型枚举，原因是 Task 3 需要先把 position_adjustment_event 的动作口径固定成正式合同，
// 目的：让后续 Tool、审批简报和投后复盘都复用同一套事件类型，而不是各自手写字符串造成命名漂移。
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PositionAdjustmentEventType {
    Build,
    Add,
    Reduce,
    Exit,
    RiskUpdate,
}

// 2026-04-08 CST: 这里新增调仓事件与计划一致性的枚举，原因是后续事件不只要记录“做了什么”，还要记录“是否按计划执行”，
// 目的：把 on_plan / justified_deviation / off_plan 固化成统一字段，方便后续复盘直接汇总偏离质量。
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PositionPlanAlignment {
    OnPlan,
    JustifiedDeviation,
    OffPlan,
}

// 2026-04-08 CST: 这里新增调仓事件请求合同，原因是 Task 4 的正式 Tool 需要有可复用的最小输入边界，
// 目的：先把 decision / approval / evidence / plan_ref 与仓位变化数据捆在一个正式类型里，避免后续 dispatcher 和测试重复拼字段。
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct SecurityRecordPositionAdjustmentRequest {
    pub decision_ref: String,
    pub approval_ref: String,
    pub evidence_version: String,
    pub position_plan_ref: String,
    pub symbol: String,
    pub event_type: PositionAdjustmentEventType,
    pub event_date: String,
    pub before_position_pct: f64,
    pub after_position_pct: f64,
    pub trigger_reason: String,
    pub plan_alignment: PositionPlanAlignment,
}

// 2026-04-08 CST: 这里新增调仓事件响应合同，原因是 Task 3 红测要求先锁定正式 event 对象的最小输出字段，
// 目的：让多次调仓记录后续能够稳定回指 position_plan / decision / approval / evidence，并为 post_trade_review 提供统一输入骨架。
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct SecurityRecordPositionAdjustmentResult {
    pub adjustment_event_ref: String,
    pub decision_ref: String,
    pub approval_ref: String,
    pub evidence_version: String,
    pub position_plan_ref: String,
    pub symbol: String,
    pub event_type: PositionAdjustmentEventType,
    pub event_date: String,
    pub before_position_pct: f64,
    pub after_position_pct: f64,
    pub trigger_reason: String,
    pub plan_alignment: PositionPlanAlignment,
}

impl SecurityRecordPositionAdjustmentResult {
    // 2026-04-08 CST: 这里补最小构造辅助函数，原因是后续 Task 4 会从正式请求对象稳定投影出事件记录，
    // 目的：把字段搬运集中在单点，避免 Tool 实现、测试和未来审批封装各自复制一份映射逻辑。
    pub fn from_request(
        adjustment_event_ref: String,
        request: SecurityRecordPositionAdjustmentRequest,
    ) -> Self {
        Self {
            adjustment_event_ref,
            decision_ref: request.decision_ref,
            approval_ref: request.approval_ref,
            evidence_version: request.evidence_version,
            position_plan_ref: request.position_plan_ref,
            symbol: request.symbol,
            event_type: request.event_type,
            event_date: request.event_date,
            before_position_pct: request.before_position_pct,
            after_position_pct: request.after_position_pct,
            trigger_reason: request.trigger_reason,
            plan_alignment: request.plan_alignment,
        }
    }
}

// 2026-04-10 CST: 这里新增条件复核触发类型合同，原因是 condition_review 新模块已经进入证券主链，
// 目的：把人工复核、收盘复核、事件复核、数据过期复核收口为统一可序列化枚举，避免 CLI/Tool 各自手写字符串。
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SecurityConditionReviewTriggerType {
    ManualReview,
    EndOfDayReview,
    EventReview,
    DataStalenessReview,
}

// 2026-04-10 CST: 这里新增条件复核后续动作合同，原因是复核结果需要正式驱动后续研究、上会和执行控制，
// 目的：让 condition_review 输出能被 approval / position_plan / review 主链稳定消费，而不是继续靠自然语言判断。
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SecurityConditionReviewFollowUpAction {
    KeepPlan,
    UpdatePositionPlan,
    ReopenResearch,
    ReopenCommittee,
    FreezeExecution,
}

// 2026-04-08 CST: 这里新增投后复盘总结果枚举，原因是 Task 5 需要先把“整体复盘结论”收口成稳定合同；
// 目的：避免后续 Tool、审批简报与复盘报告各自手写 validated / mixed / invalidated，导致口径漂移。
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PostTradeReviewOutcome {
    Validated,
    Mixed,
    Invalidated,
}

// 2026-04-08 CST: 这里新增复盘维度强弱枚举，原因是决策准确度、执行质量、风控质量三层都要复用同一套等级口径；
// 目的：让投后复盘在不同维度上保持统一语义，后续统计与翻译层也能稳定复用。
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PostTradeReviewDimension {
    Strong,
    Acceptable,
    Weak,
}

// 2026-04-08 CST: 这里新增投后复盘请求合同，原因是 Task 6 的正式 Tool 需要先有统一输入边界；
// 目的：把 plan / decision / approval / evidence / adjustment_event_refs 绑定在同一份正式请求里，避免后续 dispatcher 与 runtime 各自拼装字段。
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct SecurityPostTradeReviewRequest {
    pub decision_ref: String,
    pub approval_ref: String,
    pub evidence_version: String,
    pub position_plan_ref: String,
    pub symbol: String,
    pub analysis_date: String,
    pub adjustment_event_refs: Vec<String>,
}

// 2026-04-08 CST: 这里新增投后复盘响应合同，原因是 Task 5 需要先把复盘正式对象的最小输出字段钉住；
// 目的：确保后续 security_post_trade_review 落地后，外层 Skill / AI / 审批链拿到的是统一可回指、可审计、可复盘的结构化对象。
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct SecurityPostTradeReviewResult {
    pub post_trade_review_ref: String,
    pub position_plan_ref: String,
    pub decision_ref: String,
    pub approval_ref: String,
    pub evidence_version: String,
    pub symbol: String,
    pub analysis_date: String,
    pub adjustment_event_refs: Vec<String>,
    pub review_outcome: PostTradeReviewOutcome,
    pub decision_accuracy: PostTradeReviewDimension,
    pub execution_quality: PostTradeReviewDimension,
    pub risk_control_quality: PostTradeReviewDimension,
    pub correction_actions: Vec<String>,
    pub next_cycle_guidance: Vec<String>,
}

impl SecurityPostTradeReviewResult {
    // 2026-04-08 CST: 这里补正式复盘结果构造辅助函数，原因是 Task 6 会基于已落盘的计划与调仓事件聚合生成复盘结论；
    // 目的：把最终结果装配集中在单点，避免 Tool 实现、测试与后续审计出口重复搬运字段。
    pub fn assemble(
        post_trade_review_ref: String,
        request: &SecurityPostTradeReviewRequest,
        review_outcome: PostTradeReviewOutcome,
        decision_accuracy: PostTradeReviewDimension,
        execution_quality: PostTradeReviewDimension,
        risk_control_quality: PostTradeReviewDimension,
        correction_actions: Vec<String>,
        next_cycle_guidance: Vec<String>,
    ) -> Self {
        Self {
            post_trade_review_ref,
            position_plan_ref: request.position_plan_ref.clone(),
            decision_ref: request.decision_ref.clone(),
            approval_ref: request.approval_ref.clone(),
            evidence_version: request.evidence_version.clone(),
            symbol: request.symbol.clone(),
            analysis_date: request.analysis_date.clone(),
            adjustment_event_refs: request.adjustment_event_refs.clone(),
            review_outcome,
            decision_accuracy,
            execution_quality,
            risk_control_quality,
            correction_actions,
            next_cycle_guidance,
        }
    }
}

// 2026-04-10 CST: 这里新增可序列化 metadata schema contract，原因是内部 MetadataSchema 带有运行时索引，
// 不能直接作为 CLI JSON 输入；目的：把 foundation repository audit 的外部输入限定为声明式 schema 载荷。
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct MetadataSchemaContract {
    pub schema_version: String,
    pub fields: Vec<MetadataFieldDefinition>,
    pub concept_policies: Vec<ConceptMetadataPolicy>,
}

impl MetadataSchemaContract {
    // 2026-04-10 CST: 这里把 contract -> runtime schema 的装配收口到合同层，原因是 dispatcher 不应重复拼接 schema 构造逻辑，
    // 目的：让 CLI、后续 Skill 与其它 tool 接入都复用同一套 schema 装配边界。
    pub fn build_schema(&self) -> Result<MetadataSchema, MetadataSchemaError> {
        MetadataSchema::new_with_version(
            self.schema_version.clone(),
            self.fields.clone(),
            self.concept_policies.clone(),
        )
    }
}

// 2026-04-10 CST: 这里新增 foundation repository metadata audit 请求合同，原因是方案A要求把 repository 级治理入口正式工具化，
// 目的：把工具输入边界最小化到“layout dir + metadata schema”，先不引入报告导出和自动修复。
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct FoundationRepositoryMetadataAuditRequest {
    pub repository_layout_dir: String,
    pub metadata_schema: MetadataSchemaContract,
}

// 2026-04-10 CST: 这里定义可序列化的 repository audit issue 合同，原因是内部审计 issue 枚举更适合 Rust 侧治理，
// 但 CLI 输出需要稳定、扁平、可被外部直接消费；目的：统一对外的 issue kind 与字段载荷结构。
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct FoundationRepositoryMetadataAuditIssue {
    pub kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub node_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub concept_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub field_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub canonical_field_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub replaced_by: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repository_schema_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata_schema_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub actual_value: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expected_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_values: Option<Vec<String>>,
}

// 2026-04-10 CST: 这里定义 foundation repository metadata audit 结果合同，原因是 repository 层当前只暴露 Rust 内部 report，
// 还缺正式 Tool 输出对象；目的：为 CLI / Skill 提供稳定的 issue_count / is_clean / issues 审计结果。
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct FoundationRepositoryMetadataAuditResult {
    pub repository_layout_dir: String,
    pub repository_schema_version: String,
    pub metadata_schema_version: String,
    pub issue_count: usize,
    pub is_clean: bool,
    pub issues: Vec<FoundationRepositoryMetadataAuditIssue>,
}

impl FoundationRepositoryMetadataAuditResult {
    // 2026-04-10 CST: 这里集中封装审计结果构造，原因是 issue_count 与 is_clean 都属于 report 的派生字段，
    // 目的：避免 dispatcher、测试和后续调用方各自重复计算这些标准字段。
    pub fn new(
        repository_layout_dir: impl Into<String>,
        repository_schema_version: impl Into<String>,
        metadata_schema_version: impl Into<String>,
        issues: Vec<FoundationRepositoryMetadataAuditIssue>,
    ) -> Self {
        Self {
            repository_layout_dir: repository_layout_dir.into(),
            repository_schema_version: repository_schema_version.into(),
            metadata_schema_version: metadata_schema_version.into(),
            issue_count: issues.len(),
            is_clean: issues.is_empty(),
            issues,
        }
    }
}

// 2026-04-10 CST: 这里新增 foundation repository metadata audit gate 结果合同，原因是方案A第二阶段不再只输出“审计报告”，
// 还要输出“是否允许继续流转”的消费层判断；目的：把 gate_passed、阻塞问题与非阻塞问题固定成通用标准能力，而不是让调用方各自重写判定。
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct FoundationRepositoryMetadataAuditGateResult {
    pub repository_layout_dir: String,
    pub repository_schema_version: String,
    pub metadata_schema_version: String,
    pub gate_passed: bool,
    pub blocking_issue_count: usize,
    pub non_blocking_issue_count: usize,
    pub blocking_issues: Vec<FoundationRepositoryMetadataAuditIssue>,
    pub non_blocking_issues: Vec<FoundationRepositoryMetadataAuditIssue>,
}

impl FoundationRepositoryMetadataAuditGateResult {
    // 2026-04-10 CST: 这里集中封装 gate 结果构造，原因是 gate_passed 与两类 issue_count 都是阻塞分级的派生字段，
    // 目的：避免 dispatcher、测试和后续消费层重复手写计数与放行判定，保持 gate 语义单点收口。
    pub fn new(
        repository_layout_dir: impl Into<String>,
        repository_schema_version: impl Into<String>,
        metadata_schema_version: impl Into<String>,
        blocking_issues: Vec<FoundationRepositoryMetadataAuditIssue>,
        non_blocking_issues: Vec<FoundationRepositoryMetadataAuditIssue>,
    ) -> Self {
        Self {
            repository_layout_dir: repository_layout_dir.into(),
            repository_schema_version: repository_schema_version.into(),
            metadata_schema_version: metadata_schema_version.into(),
            gate_passed: blocking_issues.is_empty(),
            blocking_issue_count: blocking_issues.len(),
            non_blocking_issue_count: non_blocking_issues.len(),
            blocking_issues,
            non_blocking_issues,
        }
    }
}

// 2026-04-10 CST: 这里新增 foundation repository metadata audit batch 请求合同，原因是 A1 第一刀要把单仓库审计提升为批次级入口，
// 目的：把批处理输入边界固定为“多个 repository layout dir + 一份共用 metadata schema”，先不扩到每仓库独立 schema。
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct FoundationRepositoryMetadataAuditBatchRequest {
    pub repository_layout_dirs: Vec<String>,
    pub metadata_schema: MetadataSchemaContract,
}

// 2026-04-10 CST: 这里新增 foundation repository metadata audit batch 结果合同，原因是 A1 需要批次摘要与逐仓库结果双层输出，
// 目的：为后续导入链接入保留稳定的批次统计边界，而不是让调用方自己汇总多次 gate 结果。
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct FoundationRepositoryMetadataAuditBatchResult {
    pub total_repository_count: usize,
    pub passed_repository_count: usize,
    pub failed_repository_count: usize,
    pub blocking_issue_count_total: usize,
    pub non_blocking_issue_count_total: usize,
    pub repositories: Vec<FoundationRepositoryMetadataAuditGateResult>,
}

impl FoundationRepositoryMetadataAuditBatchResult {
    // 2026-04-10 CST: 这里集中封装 batch 结果构造，原因是批次计数都属于逐仓库 gate 结果的派生字段，
    // 目的：避免 dispatcher 和后续消费层重复手写 passed/failed 与 issue 总数汇总逻辑。
    pub fn new(repositories: Vec<FoundationRepositoryMetadataAuditGateResult>) -> Self {
        let total_repository_count = repositories.len();
        let passed_repository_count = repositories.iter().filter(|item| item.gate_passed).count();
        let failed_repository_count =
            total_repository_count.saturating_sub(passed_repository_count);
        let blocking_issue_count_total = repositories
            .iter()
            .map(|item| item.blocking_issue_count)
            .sum();
        let non_blocking_issue_count_total = repositories
            .iter()
            .map(|item| item.non_blocking_issue_count)
            .sum();

        Self {
            total_repository_count,
            passed_repository_count,
            failed_repository_count,
            blocking_issue_count_total,
            non_blocking_issue_count_total,
            repositories,
        }
    }
}

// 2026-04-10 CST: 这里新增 foundation repository import gate 请求合同，原因是方案B1要把 batch 审计结果提升为“导入接入层”正式入口，
// 目的：继续沿用“多个 repository layout dir + 一份共用 metadata schema”的最小输入边界，不在消费层提前扩成对象持久化或每仓库独立 schema。
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct FoundationRepositoryImportGateRequest {
    pub repository_layout_dirs: Vec<String>,
    pub metadata_schema: MetadataSchemaContract,
}

// 2026-04-10 CST: 这里新增 foundation repository import gate 结果合同，原因是方案B1的目标不是重复暴露 batch 原始摘要，
// 目的：而是把“accepted / rejected 列表 + 阻塞原因汇总 + 下一阶段是否允许继续”收口成上层可直接消费的标准能力。
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct FoundationRepositoryImportGateResult {
    pub next_stage_allowed: bool,
    pub all_repositories_accepted: bool,
    pub accepted_repository_count: usize,
    pub rejected_repository_count: usize,
    pub blocking_issue_count_total: usize,
    pub non_blocking_issue_count_total: usize,
    pub blocking_issue_kind_summary: Vec<String>,
    pub accepted_repositories: Vec<FoundationRepositoryMetadataAuditGateResult>,
    pub rejected_repositories: Vec<FoundationRepositoryMetadataAuditGateResult>,
}

impl FoundationRepositoryImportGateResult {
    // 2026-04-10 CST: 这里集中封装导入接入 gate 结果构造，原因是 next_stage_allowed、accepted/rejected 分流与阻塞原因汇总
    // 都属于 batch gate 结果的派生语义；目的：避免 dispatcher 和后续调用方各自重复解释同一批治理结果。
    pub fn new(repositories: Vec<FoundationRepositoryMetadataAuditGateResult>) -> Self {
        let mut accepted_repositories = Vec::new();
        let mut rejected_repositories = Vec::new();
        let mut blocking_issue_kind_summary = BTreeSet::new();
        let mut blocking_issue_count_total = 0usize;
        let mut non_blocking_issue_count_total = 0usize;

        for repository in repositories {
            blocking_issue_count_total += repository.blocking_issue_count;
            non_blocking_issue_count_total += repository.non_blocking_issue_count;
            for issue in &repository.blocking_issues {
                blocking_issue_kind_summary.insert(issue.kind.clone());
            }

            if repository.gate_passed {
                accepted_repositories.push(repository);
            } else {
                rejected_repositories.push(repository);
            }
        }

        let accepted_repository_count = accepted_repositories.len();
        let rejected_repository_count = rejected_repositories.len();

        Self {
            next_stage_allowed: accepted_repository_count > 0,
            all_repositories_accepted: rejected_repository_count == 0,
            accepted_repository_count,
            rejected_repository_count,
            blocking_issue_count_total,
            non_blocking_issue_count_total,
            blocking_issue_kind_summary: blocking_issue_kind_summary.into_iter().collect(),
            accepted_repositories,
            rejected_repositories,
        }
    }
}
