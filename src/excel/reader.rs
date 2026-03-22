use calamine::{Reader, open_workbook_auto};
use serde::Serialize;
use thiserror::Error;

// 2026-03-21: 这里定义工作簿摘要，目的是向 Tool 层返回稳定、可序列化的工作簿基础结构信息。
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct WorkbookSummary {
    // 2026-03-21: 这里保留源路径，目的是让上层响应里能明确当前探查的是哪个工作簿。
    pub path: String,
    // 2026-03-21: 这里返回可见 Sheet 名称列表，目的是给后续区域探查、多表编排与用户确认提供输入。
    pub sheet_names: Vec<String>,
}

// 2026-03-21: 这里定义 Excel 读取错误，目的是把底层库错误统一收口成上层可解释的中文错误。
#[derive(Debug, Error)]
pub enum ExcelError {
    #[error("无法打开工作簿: {0}")]
    OpenWorkbook(String),
}

// 2026-03-22: 这里显式导出列出 Sheet 的能力，目的是把工作簿结构探查正式收口成独立 I/O Tool。
pub fn list_sheets(path: &str) -> Result<WorkbookSummary, ExcelError> {
    // 2026-03-21: 这里继续直接使用 calamine 读取 Excel，目的是保持纯 Rust 二进制分发目标不变。
    let workbook =
        open_workbook_auto(path).map_err(|error| ExcelError::OpenWorkbook(error.to_string()))?;

    Ok(WorkbookSummary {
        path: path.to_string(),
        // 2026-03-21: 这里复制 sheet 名称列表，目的是让返回值脱离 workbook 生命周期后仍可安全序列化。
        sheet_names: workbook.sheet_names().to_vec(),
    })
}

// 2026-03-21: 这里保留 open_workbook 兼容入口，目的是不打断现有调用方，同时复用新的 list_sheets 实现。
pub fn open_workbook(path: &str) -> Result<WorkbookSummary, ExcelError> {
    list_sheets(path)
}
