use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

#[derive(Debug, Clone, Deserialize)]
pub struct ToolRequest {
    pub tool: String,
    #[serde(default)]
    pub args: Value,
}

#[derive(Debug, Clone, Serialize)]
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
            "tool_catalog": [
                "open_workbook",
                // 2026-03-22: 这里暴露 sheet 列表探查入口，目的是让上层先拿到工作簿结构再决定后续区域或多表动作。
                "list_sheets",
                // 2026-03-22: 这里暴露区域探查入口，目的是让上层先看到 Sheet 里的有效矩形区域再决定下一步。
                "inspect_sheet_range",
                // 2026-03-22: 这里暴露显式区域加载入口，目的是让上层把确认过的 range 装成可继续分析的结果表。
                "load_table_region",
                "normalize_table",
                "apply_header_schema",
                // 2026-03-22: 这里暴露会话状态读取入口，目的是让 orchestrator 能先读取本地独立记忆层再决定路由。
                "get_session_state",
                // 2026-03-22: 这里暴露会话状态写入入口，目的是让 orchestrator 能显式更新当前阶段、目标与激活句柄。
                "update_session_state",
                "preview_table",
                "select_columns",
                // 2026-03-23: 这里暴露文本标准化入口，目的是让上层在 join / lookup 前先做稳定清洗。
                "normalize_text_columns",
                // 2026-03-23: 这里暴露列改名入口，目的是让字段口径统一成为显式 Tool 能力。
                "rename_columns",
                // 2026-03-22: 这里暴露通用补空入口，目的是补齐不依赖 lookup 的缺失值处理基础能力。
                "fill_missing_values",
                // 2026-03-22: 这里暴露通用去重入口，目的是补齐整行与子集列去重的基础表处理能力。
                "distinct_rows",
                // 2026-03-22: 这里暴露按业务键去重入口，目的是补齐“保留最新/最早记录”这类主键级去重能力。
                "deduplicate_by_key",
                // 2026-03-22: 这里暴露导出前整理入口，目的是把列顺序和客户可见表头提前沉淀成独立能力。
                "format_table_for_export",
                // 2026-03-23: 这里暴露 lookup 回填入口，目的是补齐主数据补值的基础能力。
                "fill_missing_from_lookup",
                // 2026-03-23: 这里暴露日期时间标准化入口，目的是让时间口径清洗正式进入基础 Tool 目录。
                "parse_datetime_columns",
                // 2026-03-23: 这里暴露轻量查值入口，目的是给 Excel 用户提供更贴近 VLOOKUP/XLOOKUP 的基础能力。
                "lookup_values",
                // 2026-03-23: 这里暴露窗口计算入口，目的是把排名、累计值等分析桥接能力纳入基础 Tool 目录。
                "window_calculation",
                "filter_rows",
                "cast_column_types",
                "derive_columns",
                "group_and_aggregate",
                // 2026-03-23: 这里暴露透视入口，目的是让 Excel 用户熟悉的宽表分析直接进入基础 Tool 目录。
                "pivot_table",
                "sort_rows",
                "top_n",
                // 2026-03-22: 这里暴露 workbook 草稿组装入口，目的是让多张结果表先拼成可复用的多 Sheet 输出计划。
                "compose_workbook",
                // 2026-03-23: 这里暴露结果交付模板入口，目的是让上层直接生成标准汇报 workbook 草稿，而不是手工串多个输出 Tool。
                "report_delivery",
                // 2026-03-23: 这里暴露独立图表草稿入口，目的是把图表构建从 report_delivery 里拆出来，形成可复用的 chart_ref 能力。
                "build_chart",
                // 2026-03-23: 这里暴露图表图片导出入口，目的是让独立图表能力先形成客户可见的最小交付闭环。
                "export_chart_image",
                "export_csv",
                "export_excel",
                // 2026-03-22: 这里暴露多 Sheet 工作簿导出入口，目的是把 workbook_ref 真正落成标准 xlsx 文件。
                "export_excel_workbook",
                "join_tables",
                // 2026-03-21: 这里暴露多表关系建议入口，目的是让 Skill 能先发现并调用显性关联候选能力。
                "suggest_table_links",
                // 2026-03-22: 这里暴露多表流程建议入口，目的是让 Skill 能先判断应该追加、关联还是继续确认。
                "suggest_table_workflow",
                // 2026-03-22: 这里暴露多表顺序建议入口，目的是让 Skill 能先拿到多张表的处理步骤计划。
                "suggest_multi_table_plan",
                "append_tables",
                "summarize_table",
                "analyze_table",
                "stat_summary",
                // 2026-03-25: 这里暴露相关性分析入口，原因是统计诊断层开始补“目标列与候选特征列的关系排序”；目的是让上层 Skill 可以先推荐“看哪些字段最相关”。
                "correlation_analysis",
                // 2026-03-25: 这里暴露异常值检测入口，原因是统计诊断层第二批要先把可疑极端记录显式标出来；目的是让上层 Skill 能推荐“先看异常值”。
                "outlier_detection",
                // 2026-03-25: 这里暴露分布分析入口，原因是统计诊断层第二批还需要补“看分布形态”的观察能力；目的是让上层 Skill 能推荐“再看分布是否偏态”。
                "distribution_analysis",
                // 2026-03-25: 这里暴露趋势分析入口，原因是统计诊断层要进一步回答时间上的整体走向；目的是让上层 Skill 可以自然推荐“再看趋势”。
                "trend_analysis",
                "linear_regression",
                "logistic_regression",
                "cluster_kmeans",
                "decision_assistant"
            ]
        }))
    }
}
