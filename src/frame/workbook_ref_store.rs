use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::PathBuf;
use std::time::UNIX_EPOCH;

use polars::prelude::DataFrame;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::frame::result_ref_store::{
    PersistedResultColumn, PersistedResultDataset, ResultRefStoreError,
};
use crate::runtime_paths::workspace_runtime_dir;

pub struct WorkbookSheetInput {
    pub sheet_name: String,
    pub source_refs: Vec<String>,
    pub dataframe: DataFrame,
    pub title: Option<String>,
    pub subtitle: Option<String>,
    pub data_start_row: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PersistedWorkbookChartType {
    Column,
    Line,
    Pie,
    Scatter,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PersistedWorkbookLegendPosition {
    Top,
    Bottom,
    Left,
    Right,
    TopRight,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PersistedWorkbookChartSeriesSpec {
    pub value_column: String,
    #[serde(default)]
    pub name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PersistedWorkbookChartSpec {
    #[serde(default)]
    pub chart_ref: Option<String>,
    #[serde(default)]
    pub source_refs: Vec<String>,
    pub chart_type: PersistedWorkbookChartType,
    pub target_sheet_name: String,
    pub data_sheet_name: String,
    pub category_column: String,
    pub value_column: String,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub series: Vec<PersistedWorkbookChartSeriesSpec>,
    #[serde(default)]
    pub show_legend: bool,
    #[serde(default)]
    pub legend_position: Option<PersistedWorkbookLegendPosition>,
    #[serde(default)]
    pub chart_style: Option<u8>,
    #[serde(default)]
    pub x_axis_name: Option<String>,
    #[serde(default)]
    pub y_axis_name: Option<String>,
    #[serde(default = "default_chart_anchor_row")]
    pub anchor_row: u32,
    #[serde(default = "default_chart_anchor_col")]
    pub anchor_col: u16,
}

fn default_chart_anchor_row() -> u32 {
    1
}

fn default_chart_anchor_col() -> u16 {
    0
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PersistedWorkbookSheet {
    pub sheet_name: String,
    #[serde(default)]
    pub source_refs: Vec<String>,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub subtitle: Option<String>,
    #[serde(default)]
    pub data_start_row: u32,
    pub row_count: usize,
    pub columns: Vec<PersistedResultColumn>,
}

impl PersistedWorkbookSheet {
    // 2026-03-22: 这里从 DataFrame 快照单张 sheet，原因是 compose_workbook / report_delivery 都需要先冻结中间结果；目的是让导出层只面对稳定的本地草稿结构。
    pub fn from_dataframe(
        sheet_name: &str,
        source_refs: Vec<String>,
        dataframe: &DataFrame,
    ) -> Result<Self, WorkbookRefStoreError> {
        Self::from_dataframe_with_layout(sheet_name, source_refs, dataframe, None, None, 0)
    }

    // 2026-03-23: 这里补充 sheet 级标题与起始行布局，原因是结果交付模板第二阶段需要在数据表上方保留汇报标题区；目的是让 workbook 草稿能稳定表达“展示布局”和“数据内容”。
    pub fn from_dataframe_with_layout(
        sheet_name: &str,
        source_refs: Vec<String>,
        dataframe: &DataFrame,
        title: Option<String>,
        subtitle: Option<String>,
        data_start_row: u32,
    ) -> Result<Self, WorkbookRefStoreError> {
        let dataset = PersistedResultDataset::from_dataframe(
            "__workbook_sheet__",
            "compose_workbook",
            source_refs.clone(),
            dataframe,
        )
        .map_err(WorkbookRefStoreError::SnapshotSheet)?;

        Ok(Self {
            sheet_name: sheet_name.to_string(),
            source_refs,
            title,
            subtitle,
            data_start_row,
            row_count: dataset.row_count,
            columns: dataset.columns,
        })
    }

    // 2026-03-22: 这里把持久化 sheet 恢复回 DataFrame，原因是导出阶段仍复用统一写单元格逻辑；目的是避免 workbook 草稿和结果集走两套数据恢复路径。
    pub fn to_dataframe(&self) -> Result<DataFrame, WorkbookRefStoreError> {
        PersistedResultDataset {
            result_ref: "__workbook_sheet__".to_string(),
            produced_by: "compose_workbook".to_string(),
            source_refs: self.source_refs.clone(),
            row_count: self.row_count,
            columns: self.columns.clone(),
        }
        .to_dataframe()
        .map_err(WorkbookRefStoreError::RestoreWorkbook)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PersistedWorkbookDraft {
    pub workbook_ref: String,
    pub worksheets: Vec<PersistedWorkbookSheet>,
    #[serde(default)]
    pub charts: Vec<PersistedWorkbookChartSpec>,
}

impl PersistedWorkbookDraft {
    pub fn from_sheet_inputs(
        workbook_ref: &str,
        worksheets: Vec<WorkbookSheetInput>,
    ) -> Result<Self, WorkbookRefStoreError> {
        Self::from_sheet_inputs_with_charts(workbook_ref, worksheets, vec![])
    }

    // 2026-03-23: 这里扩展 workbook 草稿支持图表元数据，原因是结果交付层第二轮需要把“真实图表”作为草稿的一部分持久化；目的是让 report_delivery -> workbook_ref -> export_excel_workbook 形成闭环。
    pub fn from_sheet_inputs_with_charts(
        workbook_ref: &str,
        worksheets: Vec<WorkbookSheetInput>,
        charts: Vec<PersistedWorkbookChartSpec>,
    ) -> Result<Self, WorkbookRefStoreError> {
        if worksheets.is_empty() {
            return Err(WorkbookRefStoreError::EmptyWorksheets);
        }

        let mut seen = BTreeSet::<String>::new();
        let mut persisted = Vec::<PersistedWorkbookSheet>::with_capacity(worksheets.len());
        for worksheet in worksheets {
            if worksheet.sheet_name.trim().is_empty() {
                return Err(WorkbookRefStoreError::InvalidSheetName(
                    "sheet_name 不能为空".to_string(),
                ));
            }
            if !seen.insert(worksheet.sheet_name.clone()) {
                return Err(WorkbookRefStoreError::DuplicateSheetName(
                    worksheet.sheet_name.clone(),
                ));
            }
            persisted.push(PersistedWorkbookSheet::from_dataframe_with_layout(
                &worksheet.sheet_name,
                worksheet.source_refs,
                &worksheet.dataframe,
                worksheet.title,
                worksheet.subtitle,
                worksheet.data_start_row,
            )?);
        }

        validate_chart_specs(&persisted, &charts)?;

        Ok(Self {
            workbook_ref: workbook_ref.to_string(),
            worksheets: persisted,
            charts,
        })
    }
}

fn validate_chart_specs(
    worksheets: &[PersistedWorkbookSheet],
    charts: &[PersistedWorkbookChartSpec],
) -> Result<(), WorkbookRefStoreError> {
    let available_columns = worksheets
        .iter()
        .map(|worksheet| {
            let columns = worksheet
                .columns
                .iter()
                .map(|column| column.name.clone())
                .collect::<BTreeSet<_>>();
            (worksheet.sheet_name.clone(), columns)
        })
        .collect::<BTreeMap<_, _>>();

    for chart in charts {
        if chart.target_sheet_name.trim().is_empty() {
            return Err(WorkbookRefStoreError::InvalidChartSpec(
                "chart.target_sheet_name 不能为空".to_string(),
            ));
        }
        if chart.data_sheet_name.trim().is_empty() {
            return Err(WorkbookRefStoreError::InvalidChartSpec(
                "chart.data_sheet_name 不能为空".to_string(),
            ));
        }

        if !available_columns.contains_key(&chart.target_sheet_name) {
            return Err(WorkbookRefStoreError::MissingChartSheet(
                chart.target_sheet_name.clone(),
            ));
        }
        let Some(data_columns) = available_columns.get(&chart.data_sheet_name) else {
            return Err(WorkbookRefStoreError::MissingChartSheet(
                chart.data_sheet_name.clone(),
            ));
        };

        if !data_columns.contains(&chart.category_column) {
            return Err(WorkbookRefStoreError::MissingChartColumn {
                sheet_name: chart.data_sheet_name.clone(),
                column_name: chart.category_column.clone(),
            });
        }
        let series = normalized_chart_series(chart);
        if series.is_empty() {
            return Err(WorkbookRefStoreError::InvalidChartSpec(
                "chart.series 至少需要一个数值列".to_string(),
            ));
        }
        if chart.chart_type == PersistedWorkbookChartType::Pie && series.len() != 1 {
            return Err(WorkbookRefStoreError::InvalidChartSpec(
                "pie 图暂时只支持单个数值列".to_string(),
            ));
        }
        for series_item in series {
            if !data_columns.contains(&series_item.value_column) {
                return Err(WorkbookRefStoreError::MissingChartColumn {
                    sheet_name: chart.data_sheet_name.clone(),
                    column_name: series_item.value_column.clone(),
                });
            }
        }
        if !chart.value_column.trim().is_empty() && !data_columns.contains(&chart.value_column) {
            return Err(WorkbookRefStoreError::MissingChartColumn {
                sheet_name: chart.data_sheet_name.clone(),
                column_name: chart.value_column.clone(),
            });
        }
        if let Some(style) = chart.chart_style {
            if !(1..=48).contains(&style) {
                return Err(WorkbookRefStoreError::InvalidChartSpec(
                    "chart_style 必须在 1 到 48 之间".to_string(),
                ));
            }
        }
    }

    Ok(())
}

fn normalized_chart_series(
    chart: &PersistedWorkbookChartSpec,
) -> Vec<PersistedWorkbookChartSeriesSpec> {
    if !chart.series.is_empty() {
        return chart.series.clone();
    }
    if chart.value_column.trim().is_empty() {
        return vec![];
    }
    vec![PersistedWorkbookChartSeriesSpec {
        value_column: chart.value_column.clone(),
        name: None,
    }]
}

#[derive(Debug, Error)]
pub enum WorkbookRefStoreError {
    #[error("compose_workbook 至少需要一张 worksheet")]
    EmptyWorksheets,
    #[error("compose_workbook 的 sheet_name 不合法: {0}")]
    InvalidSheetName(String),
    #[error("compose_workbook 存在重复 sheet_name: {0}")]
    DuplicateSheetName(String),
    #[error("workbook 图表配置不合法: {0}")]
    InvalidChartSpec(String),
    #[error("workbook 图表引用的 sheet 不存在: {0}")]
    MissingChartSheet(String),
    #[error("workbook 图表引用的列不存在: {sheet_name}.{column_name}")]
    MissingChartColumn {
        sheet_name: String,
        column_name: String,
    },
    #[error("无法为 workbook 草稿创建存储目录: {0}")]
    CreateStoreDir(String),
    #[error("无法保存 workbook_ref `{workbook_ref}`: {message}")]
    SaveWorkbook {
        workbook_ref: String,
        message: String,
    },
    #[error("无法读取 workbook_ref `{workbook_ref}`: {message}")]
    LoadWorkbook {
        workbook_ref: String,
        message: String,
    },
    #[error("无法快照 worksheet: {0}")]
    SnapshotSheet(ResultRefStoreError),
    #[error("无法恢复 workbook 草稿中的 worksheet: {0}")]
    RestoreWorkbook(ResultRefStoreError),
}

#[derive(Debug, Clone)]
pub struct WorkbookDraftStore {
    root_dir: PathBuf,
}

impl WorkbookDraftStore {
    pub fn new(root_dir: PathBuf) -> Self {
        Self { root_dir }
    }

    pub fn workspace_default() -> Result<Self, WorkbookRefStoreError> {
        let runtime_dir = workspace_runtime_dir().map_err(WorkbookRefStoreError::CreateStoreDir)?;
        Ok(Self::new(runtime_dir.join("workbook_refs")))
    }

    pub fn create_workbook_ref() -> String {
        let timestamp = UNIX_EPOCH
            .elapsed()
            .map(|duration| duration.as_nanos())
            .unwrap_or_default();
        format!("workbook_{}_{}", std::process::id(), timestamp)
    }

    // 2026-03-22: 这里统一保存 workbook 草稿，原因是 compose/export/report_delivery 都通过 workbook_ref 串联；目的是把多 Sheet 与图表元数据一起落盘，便于后续复放与审计。
    pub fn save(&self, draft: &PersistedWorkbookDraft) -> Result<(), WorkbookRefStoreError> {
        fs::create_dir_all(&self.root_dir)
            .map_err(|error| WorkbookRefStoreError::CreateStoreDir(error.to_string()))?;
        let payload = serde_json::to_vec_pretty(draft).map_err(|error| {
            WorkbookRefStoreError::SaveWorkbook {
                workbook_ref: draft.workbook_ref.clone(),
                message: error.to_string(),
            }
        })?;
        fs::write(self.file_path(&draft.workbook_ref), payload).map_err(|error| {
            WorkbookRefStoreError::SaveWorkbook {
                workbook_ref: draft.workbook_ref.clone(),
                message: error.to_string(),
            }
        })
    }

    pub fn load(
        &self,
        workbook_ref: &str,
    ) -> Result<PersistedWorkbookDraft, WorkbookRefStoreError> {
        let payload = fs::read(self.file_path(workbook_ref)).map_err(|error| {
            WorkbookRefStoreError::LoadWorkbook {
                workbook_ref: workbook_ref.to_string(),
                message: error.to_string(),
            }
        })?;
        serde_json::from_slice(&payload).map_err(|error| WorkbookRefStoreError::LoadWorkbook {
            workbook_ref: workbook_ref.to_string(),
            message: error.to_string(),
        })
    }

    fn file_path(&self, workbook_ref: &str) -> PathBuf {
        self.root_dir.join(format!("{workbook_ref}.json"))
    }
}
