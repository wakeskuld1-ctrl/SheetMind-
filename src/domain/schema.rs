use serde::Serialize;

// 2026-03-21: 这里定义 schema 状态枚举，目的是把“是否允许继续计算”从隐式约定提升为强约束的领域语义。
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum SchemaState {
    // 2026-03-21: 默认先进入待确认状态，目的是降低复杂表头误判后连锁传播的风险。
    Pending,
    // 2026-03-21: 预留已确认状态，目的是为后续 header inference 成功后的放行流程提供清晰标记。
    Confirmed,
}

impl SchemaState {
    // 2026-03-21: 提供统一确认判断方法，目的是避免各模块自己用模式匹配散写门禁逻辑。
    pub fn is_confirmed(&self) -> bool {
        matches!(self, Self::Confirmed)
    }
}

// 2026-03-21: 这里统一输出 schema 状态标签，目的是让 CLI/Tool 层和测试共享同一套状态文案，避免重复映射。
pub fn infer_schema_state_label(state: &SchemaState) -> &'static str {
    match state {
        SchemaState::Pending => "pending",
        SchemaState::Confirmed => "confirmed",
    }
}

// 2026-03-21: 这里定义表头推断置信度，目的是把“自动通过”与“必须确认”的判断依据固化为显式状态而不是零散布尔值。
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum ConfidenceLevel {
    // 2026-03-21: 高置信度代表可以自动放行，目的是在简单表格场景下减少额外确认成本。
    High,
    // 2026-03-21: 中置信度代表存在结构风险，目的是让问答界面先与用户确认再继续后续处理。
    Medium,
    // 2026-03-21: 低置信度代表结构判断很不可靠，目的是阻止错误推断继续向后传播。
    Low,
}

impl ConfidenceLevel {
    // 2026-03-21: 统一暴露高置信度判断，目的是让推断结果和 Tool 协议共享同一套自动放行逻辑。
    pub fn is_high(&self) -> bool {
        matches!(self, Self::High)
    }
}

// 2026-03-21: 这里定义单列的表头路径信息，目的是在内部保留多层表头语义的同时，对外暴露稳定 canonical 名称。
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct HeaderColumn {
    // 2026-03-21: 保存原始表头层级片段，目的是给后续预览映射和人工确认提供完整上下文。
    pub header_path: Vec<String>,
    // 2026-03-21: 暴露标准列名，目的是让后续表处理 Tool 只依赖稳定列名而不直接依赖原始 Excel 表头形态。
    pub canonical_name: String,
}

// 2026-03-21: 这里定义表头推断结果，目的是把列映射、置信度和 schema 门禁一次性返回给上层 Tool。
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct HeaderInference {
    // 2026-03-21: 保存识别后的列信息，目的是作为后续 DataFrame 列名与确认界面的统一输入。
    pub columns: Vec<HeaderColumn>,
    // 2026-03-21: 保存当前判断置信度，目的是决定是否自动确认 schema。
    pub confidence: ConfidenceLevel,
    // 2026-03-21: 直接给出 schema 状态，目的是让调用方无需重复写“高置信度即确认”的散落逻辑。
    pub schema_state: SchemaState,
    // 2026-03-21: 保存推断使用的表头行数，目的是帮助后续表区域确认与调试复杂 Excel。
    pub header_row_count: usize,
    // 2026-03-21: 记录数据开始行号，目的是让 DataFrame 加载层能正确跳过标题行和多层表头。
    pub data_start_row_index: usize,
}
