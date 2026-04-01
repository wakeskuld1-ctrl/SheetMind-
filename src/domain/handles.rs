use serde::Serialize;

use crate::domain::schema::SchemaState;

// 2026-03-21: 这里定义内存表句柄，目的是在真正接入 DataFrame 前先固化“文件/工作表/schema 状态”这组最小上下文。
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TableHandle {
    // 2026-03-21: 记录来源文件，目的是为后续 trace、错误提示与跨表关联保留来源定位信息。
    source_path: String,
    // 2026-03-21: 记录来源工作表，目的是让用户确认结构时能知道当前处理的是哪张表。
    sheet_name: String,
    // 2026-03-21: 显式保存 schema 状态，目的是从领域层就阻止未确认表结构进入后续计算。
    schema_state: SchemaState,
    // 2026-03-21: 保存确认后的 canonical 列名，目的是让后续 DataFrame 装载和 Tool 引用使用稳定列集合。
    columns: Vec<String>,
}

impl TableHandle {
    // 2026-03-21: 提供待确认句柄构造器，目的是让读取 Excel 后先进入“待确认”状态，而不是直接假设结构正确。
    pub fn new_pending(source_path: impl Into<String>, sheet_name: impl Into<String>) -> Self {
        Self {
            source_path: source_path.into(),
            sheet_name: sheet_name.into(),
            schema_state: SchemaState::Pending,
            columns: Vec::new(),
        }
    }

    // 2026-03-21: 提供已确认句柄构造器，目的是在应用 schema 后产出一个可以继续进入计算链的稳定表对象。
    pub fn new_confirmed(
        source_path: impl Into<String>,
        sheet_name: impl Into<String>,
        columns: Vec<String>,
    ) -> Self {
        Self {
            source_path: source_path.into(),
            sheet_name: sheet_name.into(),
            schema_state: SchemaState::Confirmed,
            columns,
        }
    }

    // 2026-03-21: 暴露只读 schema 状态访问器，目的是让 Tool 层做门禁判断时不需要直接改写领域状态。
    pub fn schema_state(&self) -> &SchemaState {
        &self.schema_state
    }

    // 2026-03-21: 暴露来源文件访问器，目的是便于后续 JSON 响应与审计信息引用原始输入。
    pub fn source_path(&self) -> &str {
        &self.source_path
    }

    // 2026-03-21: 暴露工作表名称访问器，目的是为后续多工作表识别和关联提示保留最小上下文。
    pub fn sheet_name(&self) -> &str {
        &self.sheet_name
    }

    // 2026-03-21: 暴露 canonical 列名访问器，目的是让后续表处理与 JSON 返回共享已确认后的列集合。
    pub fn columns(&self) -> &[String] {
        &self.columns
    }
}
