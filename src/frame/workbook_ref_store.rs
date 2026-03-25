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
    // 2026-03-24: 这里显式记录 sheet 的交付类型，原因是导出层下一轮要按“数据页 / 图表页”分流冻结、筛选和列宽规则；目的是避免 export 层继续通过名称猜测页面用途。
    pub sheet_kind: PersistedWorkbookSheetKind,
    // 2026-03-24: 这里补充 sheet 级导出意图快照，原因是 report_delivery 段内 format 已经允许声明交付偏好；目的是让显式数字格式等能力跟随 workbook_ref 进入最终导出。
    pub export_options: Option<PersistedWorkbookSheetExportOptions>,
    pub title: Option<String>,
    pub subtitle: Option<String>,
    pub data_start_row: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PersistedWorkbookSheetKind {
    DataSheet,
    ChartSheet,
}

fn default_sheet_kind() -> PersistedWorkbookSheetKind {
    PersistedWorkbookSheetKind::DataSheet
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PersistedWorkbookNumberFormatKind {
    Currency,
    Percent,
    Integer,
    Decimal2,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PersistedWorkbookColumnNumberFormatRule {
    pub column: String,
    pub kind: PersistedWorkbookNumberFormatKind,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PersistedWorkbookConditionalFormatKind {
    NegativeRed,
    NullWarning,
    DuplicateWarn,
    HighValueHighlight,
    PercentLowWarn,
    BetweenWarn,
    CompositeDuplicateWarn,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PersistedWorkbookColumnConditionalFormatRule {
    #[serde(default)]
    pub column: String,
    #[serde(default)]
    pub columns: Vec<String>,
    pub kind: PersistedWorkbookConditionalFormatKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub threshold: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min_threshold: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_threshold: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PersistedWorkbookSheetExportOptions {
    #[serde(default)]
    pub number_formats: Vec<PersistedWorkbookColumnNumberFormatRule>,
    #[serde(default)]
    pub conditional_formats: Vec<PersistedWorkbookColumnConditionalFormatRule>,
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
    // 2026-03-24: 这里把 sheet_kind 持久化进 workbook 草稿，原因是结果交付层需要在导出时稳定复用页类型语义；目的是让“冻结首列只对数据页生效”这类规则能跨请求保持一致。
    #[serde(default = "default_sheet_kind")]
    pub sheet_kind: PersistedWorkbookSheetKind,
    // 2026-03-24: 这里把导出意图一并冻结进 sheet，原因是 workbook 草稿需要承载“数据 + 布局 + 格式意图”；目的是让导出阶段不必回看上层调用参数。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub export_options: Option<PersistedWorkbookSheetExportOptions>,
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
    // 2026-03-22: 杩欓噷浠?DataFrame 蹇収鍗曞紶 sheet锛屽師鍥犳槸 compose_workbook / report_delivery 閮介渶瑕佸厛鍐荤粨涓棿缁撴灉锛涚洰鐨勬槸璁╁鍑哄眰鍙潰瀵圭ǔ瀹氱殑鏈湴鑽夌缁撴瀯銆?
    pub fn from_dataframe(
        sheet_name: &str,
        source_refs: Vec<String>,
        dataframe: &DataFrame,
    ) -> Result<Self, WorkbookRefStoreError> {
        Self::from_dataframe_with_layout(
            sheet_name,
            source_refs,
            dataframe,
            PersistedWorkbookSheetKind::DataSheet,
            None,
            None,
            None,
            0,
        )
    }

    // 2026-03-23: 杩欓噷琛ュ厖 sheet 绾ф爣棰樹笌璧峰琛屽竷灞€锛屽師鍥犳槸缁撴灉浜や粯妯℃澘绗簩闃舵闇€瑕佸湪鏁版嵁琛ㄤ笂鏂逛繚鐣欐眹鎶ユ爣棰樺尯锛涚洰鐨勬槸璁?workbook 鑽夌鑳界ǔ瀹氳〃杈锯€滃睍绀哄竷灞€鈥濆拰鈥滄暟鎹唴瀹光€濄€?
    pub fn from_dataframe_with_layout(
        sheet_name: &str,
        source_refs: Vec<String>,
        dataframe: &DataFrame,
        sheet_kind: PersistedWorkbookSheetKind,
        export_options: Option<PersistedWorkbookSheetExportOptions>,
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
            sheet_kind,
            export_options,
            title,
            subtitle,
            data_start_row,
            row_count: dataset.row_count,
            columns: dataset.columns,
        })
    }

    // 2026-03-22: 杩欓噷鎶婃寔涔呭寲 sheet 鎭㈠鍥?DataFrame锛屽師鍥犳槸瀵煎嚭闃舵浠嶅鐢ㄧ粺涓€鍐欏崟鍏冩牸閫昏緫锛涚洰鐨勬槸閬垮厤 workbook 鑽夌鍜岀粨鏋滈泦璧颁袱濂楁暟鎹仮澶嶈矾寰勩€?
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

    // 2026-03-23: 杩欓噷鎵╁睍 workbook 鑽夌鏀寔鍥捐〃鍏冩暟鎹紝鍘熷洜鏄粨鏋滀氦浠樺眰绗簩杞渶瑕佹妸鈥滅湡瀹炲浘琛ㄢ€濅綔涓鸿崏绋跨殑涓€閮ㄥ垎鎸佷箙鍖栵紱鐩殑鏄 report_delivery -> workbook_ref -> export_excel_workbook 褰㈡垚闂幆銆?
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
                    "sheet_name 涓嶈兘涓虹┖".to_string(),
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
                worksheet.sheet_kind,
                worksheet.export_options,
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
                "chart.target_sheet_name 涓嶈兘涓虹┖".to_string(),
            ));
        }
        if chart.data_sheet_name.trim().is_empty() {
            return Err(WorkbookRefStoreError::InvalidChartSpec(
                "chart.data_sheet_name 涓嶈兘涓虹┖".to_string(),
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
                "chart.series 鑷冲皯闇€瑕佷竴涓暟鍊煎垪".to_string(),
            ));
        }
        if chart.chart_type == PersistedWorkbookChartType::Pie && series.len() != 1 {
            return Err(WorkbookRefStoreError::InvalidChartSpec(
                "pie 鍥炬殏鏃跺彧鏀寔鍗曚釜鏁板€煎垪".to_string(),
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
                    "chart_style 蹇呴』鍦?1 鍒?48 涔嬮棿".to_string(),
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
    #[error("compose_workbook 鑷冲皯闇€瑕佷竴寮?worksheet")]
    EmptyWorksheets,
    #[error("compose_workbook 鐨?sheet_name 涓嶅悎娉? {0}")]
    InvalidSheetName(String),
    #[error("compose_workbook 瀛樺湪閲嶅 sheet_name: {0}")]
    DuplicateSheetName(String),
    #[error("workbook 鍥捐〃閰嶇疆涓嶅悎娉? {0}")]
    InvalidChartSpec(String),
    #[error("workbook 鍥捐〃寮曠敤鐨?sheet 涓嶅瓨鍦? {0}")]
    MissingChartSheet(String),
    #[error("workbook 鍥捐〃寮曠敤鐨勫垪涓嶅瓨鍦? {sheet_name}.{column_name}")]
    MissingChartColumn {
        sheet_name: String,
        column_name: String,
    },
    #[error("鏃犳硶涓?workbook 鑽夌鍒涘缓瀛樺偍鐩綍: {0}")]
    CreateStoreDir(String),
    #[error("鏃犳硶淇濆瓨 workbook_ref `{workbook_ref}`: {message}")]
    SaveWorkbook {
        workbook_ref: String,
        message: String,
    },
    #[error("鏃犳硶璇诲彇 workbook_ref `{workbook_ref}`: {message}")]
    LoadWorkbook {
        workbook_ref: String,
        message: String,
    },
    #[error("鏃犳硶蹇収 worksheet: {0}")]
    SnapshotSheet(ResultRefStoreError),
    #[error("鏃犳硶鎭㈠ workbook 鑽夌涓殑 worksheet: {0}")]
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

    // 2026-03-22: 杩欓噷缁熶竴淇濆瓨 workbook 鑽夌锛屽師鍥犳槸 compose/export/report_delivery 閮介€氳繃 workbook_ref 涓茶仈锛涚洰鐨勬槸鎶婂 Sheet 涓庡浘琛ㄥ厓鏁版嵁涓€璧疯惤鐩橈紝渚夸簬鍚庣画澶嶆斁涓庡璁°€?
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


