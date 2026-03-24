use std::fs;
use std::path::PathBuf;
use std::time::UNIX_EPOCH;

use polars::prelude::DataFrame;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::frame::result_ref_store::{PersistedResultDataset, ResultRefStoreError};
use crate::runtime_paths::workspace_runtime_dir;

// 2026-03-23: 这里定义可持久化的图表类型，原因是独立 chart_ref 需要脱离 report_delivery 存储；目的是让图表构建与图表交付形成独立闭环。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PersistedChartType {
    Column,
    Line,
    Pie,
    Scatter,
}

// 2026-03-23: 这里定义图表系列规格，原因是独立图表 Tool 需要支持单系列和多系列；目的是让 chart_ref 能稳定表达客户可见的图例语义。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PersistedChartSeriesSpec {
    pub value_column: String,
    #[serde(default)]
    pub name: Option<String>,
}

// 2026-03-23: 这里定义图表草稿持久化结构，原因是 build_chart 产物需要跨请求复用；目的是让 export_chart_image 和后续 report_delivery 共享同一份图表规格。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PersistedChartDraft {
    pub chart_ref: String,
    pub produced_by: String,
    #[serde(default)]
    pub source_refs: Vec<String>,
    pub chart_type: PersistedChartType,
    #[serde(default)]
    pub title: Option<String>,
    pub category_column: String,
    #[serde(default)]
    pub x_axis_name: Option<String>,
    #[serde(default)]
    pub y_axis_name: Option<String>,
    #[serde(default)]
    pub show_legend: bool,
    #[serde(default)]
    pub width: u32,
    #[serde(default)]
    pub height: u32,
    pub series: Vec<PersistedChartSeriesSpec>,
    pub dataset: PersistedResultDataset,
}

impl PersistedChartDraft {
    // 2026-03-23: 这里从 DataFrame 构造 chart_ref 草稿，原因是独立图表能力要冻结构图所需的数据快照；目的是避免导出图表时再回退到原始 Excel 或上游 result_ref。
    pub fn from_dataframe(
        chart_ref: &str,
        produced_by: &str,
        source_refs: Vec<String>,
        dataframe: &DataFrame,
        chart_type: PersistedChartType,
        title: Option<String>,
        category_column: &str,
        series: Vec<PersistedChartSeriesSpec>,
    ) -> Result<Self, ChartRefStoreError> {
        Self::from_dataframe_with_layout(
            chart_ref,
            produced_by,
            source_refs,
            dataframe,
            chart_type,
            title,
            category_column,
            None,
            None,
            false,
            900,
            520,
            series,
        )
    }

    // 2026-03-23: 这里补布局元数据入口，原因是后续 SVG 导出与 workbook 复用都需要统一尺寸；目的是让 chart_ref 先把图和数据一起稳定冻结下来。
    #[allow(clippy::too_many_arguments)]
    pub fn from_dataframe_with_layout(
        chart_ref: &str,
        produced_by: &str,
        source_refs: Vec<String>,
        dataframe: &DataFrame,
        chart_type: PersistedChartType,
        title: Option<String>,
        category_column: &str,
        x_axis_name: Option<String>,
        y_axis_name: Option<String>,
        show_legend: bool,
        width: u32,
        height: u32,
        series: Vec<PersistedChartSeriesSpec>,
    ) -> Result<Self, ChartRefStoreError> {
        if chart_ref.trim().is_empty() {
            return Err(ChartRefStoreError::InvalidChartSpec(
                "chart_ref 不能为空".to_string(),
            ));
        }
        if category_column.trim().is_empty() {
            return Err(ChartRefStoreError::InvalidChartSpec(
                "category_column 不能为空".to_string(),
            ));
        }
        if series.is_empty() {
            return Err(ChartRefStoreError::InvalidChartSpec(
                "series 至少需要一个数值列".to_string(),
            ));
        }
        if matches!(chart_type, PersistedChartType::Pie) && series.len() != 1 {
            return Err(ChartRefStoreError::InvalidChartSpec(
                "pie 图暂时只支持单个数值列".to_string(),
            ));
        }
        if dataframe.column(category_column).is_err() {
            return Err(ChartRefStoreError::MissingChartColumn(
                category_column.to_string(),
            ));
        }
        for item in &series {
            if item.value_column.trim().is_empty() {
                return Err(ChartRefStoreError::InvalidChartSpec(
                    "value_column 不能为空".to_string(),
                ));
            }
            if dataframe.column(&item.value_column).is_err() {
                return Err(ChartRefStoreError::MissingChartColumn(
                    item.value_column.clone(),
                ));
            }
        }

        let dataset = PersistedResultDataset::from_dataframe(
            "__chart_ref__",
            produced_by,
            source_refs.clone(),
            dataframe,
        )
        .map_err(ChartRefStoreError::SnapshotChart)?;

        Ok(Self {
            chart_ref: chart_ref.to_string(),
            produced_by: produced_by.to_string(),
            source_refs,
            chart_type,
            title,
            category_column: category_column.to_string(),
            x_axis_name,
            y_axis_name,
            show_legend,
            width,
            height,
            series,
            dataset,
        })
    }

    // 2026-03-23: 这里把图表草稿恢复成 DataFrame，原因是 SVG 导出和后续交付都需要读取冻结后的数据；目的是让 chart_ref 复用 result_ref 的稳定回放能力。
    pub fn to_dataframe(&self) -> Result<DataFrame, ChartRefStoreError> {
        self.dataset
            .to_dataframe()
            .map_err(ChartRefStoreError::RestoreChart)
    }
}

// 2026-03-23: 这里定义 chart_ref 存储错误，原因是需要把图表规格错误、文件错误和快照错误分层返回；目的是给上层稳定中文报错。
#[derive(Debug, Error)]
pub enum ChartRefStoreError {
    #[error("chart_ref 规格不合法: {0}")]
    InvalidChartSpec(String),
    #[error("chart_ref 引用的列不存在: {0}")]
    MissingChartColumn(String),
    #[error("无法为 chart_ref 创建存储目录: {0}")]
    CreateStoreDir(String),
    #[error("无法保存 chart_ref `{chart_ref}`: {message}")]
    SaveChart { chart_ref: String, message: String },
    #[error("无法读取 chart_ref `{chart_ref}`: {message}")]
    LoadChart { chart_ref: String, message: String },
    #[error("无法快照图表数据: {0}")]
    SnapshotChart(ResultRefStoreError),
    #[error("无法恢复图表数据: {0}")]
    RestoreChart(ResultRefStoreError),
}

// 2026-03-23: 这里定义 chart_ref 文件存储入口，原因是独立图表 Tool 需要和 result_ref/workbook_ref 一样支持跨请求复用；目的是统一本地二进制能力的句柄体验。
#[derive(Debug, Clone)]
pub struct ChartDraftStore {
    root_dir: PathBuf,
}

impl ChartDraftStore {
    pub fn new(root_dir: PathBuf) -> Self {
        Self { root_dir }
    }

    // 2026-03-23: 这里提供默认 chart_ref 落盘目录，原因是 build_chart/export_chart_image 要共享同一位置；目的是保持句柄存储规则稳定。
    pub fn workspace_default() -> Result<Self, ChartRefStoreError> {
        let runtime_dir = workspace_runtime_dir().map_err(ChartRefStoreError::CreateStoreDir)?;
        Ok(Self::new(runtime_dir.join("chart_refs")))
    }

    // 2026-03-23: 这里统一生成 chart_ref，原因是需要和 table_ref/result_ref/workbook_ref 明确区分；目的是便于调试与会话状态展示。
    pub fn create_chart_ref() -> String {
        let timestamp = UNIX_EPOCH
            .elapsed()
            .map(|duration| duration.as_nanos())
            .unwrap_or_default();
        format!("chart_{}_{}", std::process::id(), timestamp)
    }

    pub fn save(&self, draft: &PersistedChartDraft) -> Result<(), ChartRefStoreError> {
        fs::create_dir_all(&self.root_dir)
            .map_err(|error| ChartRefStoreError::CreateStoreDir(error.to_string()))?;
        let path = self.path_for(&draft.chart_ref);
        let content =
            serde_json::to_vec_pretty(draft).map_err(|error| ChartRefStoreError::SaveChart {
                chart_ref: draft.chart_ref.clone(),
                message: error.to_string(),
            })?;
        fs::write(&path, content).map_err(|error| ChartRefStoreError::SaveChart {
            chart_ref: draft.chart_ref.clone(),
            message: error.to_string(),
        })
    }

    pub fn load(&self, chart_ref: &str) -> Result<PersistedChartDraft, ChartRefStoreError> {
        let path = self.path_for(chart_ref);
        let content = fs::read(&path).map_err(|error| ChartRefStoreError::LoadChart {
            chart_ref: chart_ref.to_string(),
            message: error.to_string(),
        })?;
        serde_json::from_slice::<PersistedChartDraft>(&content).map_err(|error| {
            ChartRefStoreError::LoadChart {
                chart_ref: chart_ref.to_string(),
                message: error.to_string(),
            }
        })
    }

    fn path_for(&self, chart_ref: &str) -> PathBuf {
        self.root_dir.join(format!("{chart_ref}.json"))
    }
}
