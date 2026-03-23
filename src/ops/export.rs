use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use polars::prelude::{AnyValue, DataFrame};
use rust_xlsxwriter::{Chart, ChartLegendPosition, ChartType, Format, FormatAlign, Workbook, Worksheet};
use thiserror::Error;

use crate::frame::loader::LoadedTable;
use crate::frame::workbook_ref_store::{
    PersistedWorkbookChartSeriesSpec, PersistedWorkbookChartSpec, PersistedWorkbookChartType,
    PersistedWorkbookDraft, PersistedWorkbookLegendPosition,
};

#[derive(Debug, Error)]
pub enum ExportError {
    #[error("导出路径缺少父目录: {0}")]
    MissingParentDirectory(String),
    #[error("无法创建导出目录: {0}")]
    CreateOutputDir(String),
    #[error("无法写出 CSV: {0}")]
    WriteCsv(String),
    #[error("无法写出 Excel: {0}")]
    WriteExcel(String),
}

pub fn export_csv(loaded: &LoadedTable, output_path: &str) -> Result<(), ExportError> {
    ensure_parent_dir(output_path)?;

    let mut rows = Vec::<String>::new();
    rows.push(
        loaded
            .dataframe
            .get_column_names()
            .iter()
            .map(|name| escape_csv_field(name))
            .collect::<Vec<_>>()
            .join(","),
    );

    for row_index in 0..loaded.dataframe.height() {
        let row = loaded
            .dataframe
            .get_columns()
            .iter()
            .map(|column| {
                column
                    .as_materialized_series()
                    .str_value(row_index)
                    .map(|value| escape_csv_field(value.as_ref()))
                    .map_err(|error| ExportError::WriteCsv(error.to_string()))
            })
            .collect::<Result<Vec<_>, _>>()?;
        rows.push(row.join(","));
    }

    fs::write(output_path, rows.join("\n"))
        .map_err(|error| ExportError::WriteCsv(error.to_string()))
}

pub fn export_excel(
    loaded: &LoadedTable,
    output_path: &str,
    sheet_name: &str,
) -> Result<(), ExportError> {
    ensure_parent_dir(output_path)?;

    let mut workbook = Workbook::new();
    let worksheet = workbook.add_worksheet();
    worksheet
        .set_name(sheet_name)
        .map_err(|error| ExportError::WriteExcel(error.to_string()))?;
    let cell_formats = build_export_cell_formats();

    for (column_index, column_name) in loaded.dataframe.get_column_names().iter().enumerate() {
        worksheet
            .write_string(0, column_index as u16, column_name.as_str())
            .map_err(|error| ExportError::WriteExcel(error.to_string()))?;
    }

    for row_index in 0..loaded.dataframe.height() {
        let mut wrapped_row = false;
        for (column_index, column) in loaded.dataframe.get_columns().iter().enumerate() {
            // 2026-03-23: 这里按真实单元格类型写 Excel，原因是导出的结果表还要继续被客户求和、透视和排序；目的是避免所有值都退化成文本。
            wrapped_row |= write_excel_cell(worksheet, column, row_index, column_index, &cell_formats)
                .map_err(|error| ExportError::WriteExcel(error.to_string()))?;
        }
        // 2026-03-24: 这里在单表导出时为长文本行补一个保守行高，原因是仅有 wrapText 仍可能让首屏看起来过扁；目的是让说明列在 Excel 里打开后更接近可读状态。
        if wrapped_row {
            worksheet
                .set_row_height((row_index + 1) as u32, 36)
                .map_err(|error| ExportError::WriteExcel(error.to_string()))?;
        }
    }
    // 2026-03-24: 这里给单表导出补自动筛选，原因是业务用户导出后最常见动作就是按列筛选；目的是把基础可用性直接沉到底层导出能力。
    apply_autofilter(worksheet, 0, loaded.dataframe.height(), loaded.dataframe.width())
        .map_err(|error| ExportError::WriteExcel(error.to_string()))?;
    // 2026-03-24: 这里在单表导出时冻结表头行，原因是客户查看长表时需要始终保留字段上下文；目的是让基础导出也具备最小可读性。
    apply_freeze_panes(worksheet, 0).map_err(|error| ExportError::WriteExcel(error.to_string()))?;
    // 2026-03-24: 这里在单表导出后统一补显式列宽，原因是默认宽度对长列名和中文字段不友好；目的是让导出的单表开箱即读，不必客户再手工拖列宽。
    apply_auto_column_widths(worksheet, &loaded.dataframe)
        .map_err(|error| ExportError::WriteExcel(error.to_string()))?;

    workbook
        .save(output_path)
        .map_err(|error| ExportError::WriteExcel(error.to_string()))
}

pub fn export_excel_workbook(
    draft: &PersistedWorkbookDraft,
    output_path: &str,
) -> Result<(), ExportError> {
    ensure_parent_dir(output_path)?;

    let mut workbook = Workbook::new();
    let mut worksheet_bindings = BTreeMap::<String, WorksheetBinding>::new();

    for worksheet_snapshot in &draft.worksheets {
        let dataframe = worksheet_snapshot
            .to_dataframe()
            .map_err(|error| ExportError::WriteExcel(error.to_string()))?;
        let worksheet = workbook.add_worksheet();
        worksheet
            .set_name(&worksheet_snapshot.sheet_name)
            .map_err(|error| ExportError::WriteExcel(error.to_string()))?;
        let cell_formats = build_export_cell_formats();
        let last_title_col = dataframe.width().saturating_sub(1) as u16;

        if let Some(title) = worksheet_snapshot.title.as_ref() {
            if !title.trim().is_empty() {
                // 2026-03-24: 这里把标题区横向合并到整张数据表宽度，原因是只写在 A1 更像普通数据表；目的是把 report_delivery 页进一步拉近“汇报稿”观感。
                write_sheet_banner(worksheet, 0, last_title_col, title, &cell_formats.title_format)
                    .map_err(|error| ExportError::WriteExcel(error.to_string()))?;
            }
        }
        if let Some(subtitle) = worksheet_snapshot.subtitle.as_ref() {
            if !subtitle.trim().is_empty() {
                // 2026-03-24: 这里把副标题和标题保持同样的横向结构，原因是标题区需要形成稳定视觉块；目的是避免副标题仍孤零零落在 A2。
                write_sheet_banner(
                    worksheet,
                    1,
                    last_title_col,
                    subtitle,
                    &cell_formats.subtitle_format,
                )
                .map_err(|error| ExportError::WriteExcel(error.to_string()))?;
            }
        }

        let header_row = worksheet_snapshot.data_start_row;
        let mut column_indices = BTreeMap::<String, usize>::new();
        for (column_index, column_name) in dataframe.get_column_names().iter().enumerate() {
            worksheet
                .write_string(header_row, column_index as u16, column_name.as_str())
                .map_err(|error| ExportError::WriteExcel(error.to_string()))?;
            column_indices.insert(column_name.to_string(), column_index);
        }

        for row_index in 0..dataframe.height() {
            let mut wrapped_row = false;
            for (column_index, column) in dataframe.get_columns().iter().enumerate() {
                // 2026-03-23: 这里让多 Sheet 导出复用同一套类型写出逻辑，原因是 report_delivery 和 compose_workbook 都会走这里；目的是保证图表数据源单元格也保持可计算的数值类型。
                wrapped_row |= write_excel_cell_with_row_offset(
                    worksheet,
                    column,
                    row_index,
                    column_index,
                    header_row + 1,
                    &cell_formats,
                )
                    .map_err(|error| ExportError::WriteExcel(error.to_string()))?;
            }
            if wrapped_row {
                worksheet
                    .set_row_height(header_row + 1 + row_index as u32, 36)
                    .map_err(|error| ExportError::WriteExcel(error.to_string()))?;
            }
        }
        // 2026-03-24: 这里给 workbook 内每张表统一加自动筛选，原因是 compose_workbook / report_delivery 都是面向交付的结果表；目的是让用户打开即能筛，不用手动再点一遍。
        apply_autofilter(
            worksheet,
            header_row,
            dataframe.height(),
            dataframe.width(),
        )
        .map_err(|error| ExportError::WriteExcel(error.to_string()))?;
        // 2026-03-24: 这里按各 sheet 的标题区与表头位置统一冻结窗格，原因是 report_delivery 与 compose_workbook 都需要在滚动时保留表头；目的是把冻结规则沉到公共交付层而不是散落在模板逻辑里。
        apply_freeze_panes(worksheet, header_row)
            .map_err(|error| ExportError::WriteExcel(error.to_string()))?;
        // 2026-03-24: 这里给 workbook 内每张数据表补自动列宽，原因是 report_delivery / compose_workbook 都走同一条交付链路；目的是把“列宽可读性”一次性沉到公共导出层。
        apply_auto_column_widths(worksheet, &dataframe)
            .map_err(|error| ExportError::WriteExcel(error.to_string()))?;

        worksheet_bindings.insert(
            worksheet_snapshot.sheet_name.clone(),
            WorksheetBinding {
                row_count: dataframe.height(),
                header_row,
                column_indices,
            },
        );
    }

    for chart_spec in &draft.charts {
        insert_chart_into_workbook(&mut workbook, &worksheet_bindings, chart_spec)?;
    }

    workbook
        .save(output_path)
        .map_err(|error| ExportError::WriteExcel(error.to_string()))
}

#[derive(Debug)]
struct WorksheetBinding {
    row_count: usize,
    header_row: u32,
    column_indices: BTreeMap<String, usize>,
}

fn insert_chart_into_workbook(
    workbook: &mut Workbook,
    worksheet_bindings: &BTreeMap<String, WorksheetBinding>,
    chart_spec: &PersistedWorkbookChartSpec,
) -> Result<(), ExportError> {
    let Some(data_binding) = worksheet_bindings.get(&chart_spec.data_sheet_name) else {
        return Err(ExportError::WriteExcel(format!(
            "图表数据源 sheet 不存在: {}",
            chart_spec.data_sheet_name
        )));
    };
    let Some(&category_col) = data_binding.column_indices.get(&chart_spec.category_column) else {
        return Err(ExportError::WriteExcel(format!(
            "图表分类列不存在: {}.{}",
            chart_spec.data_sheet_name, chart_spec.category_column
        )));
    };
    if data_binding.row_count == 0 {
        return Err(ExportError::WriteExcel(format!(
            "图表数据源没有可用数据行: {}",
            chart_spec.data_sheet_name
        )));
    }

    let first_data_row = data_binding.header_row + 1;
    let last_row = data_binding.header_row + data_binding.row_count as u32;
    let category_col = category_col as u16;
    let mut chart = Chart::new(map_chart_type(&chart_spec.chart_type));
    if let Some(style) = chart_spec.chart_style {
        chart.set_style(style);
    }
    for series_spec in normalized_chart_series(chart_spec) {
        let Some(&series_value_col) = data_binding.column_indices.get(&series_spec.value_column) else {
            return Err(ExportError::WriteExcel(format!(
                "图表数值列不存在: {}.{}",
                chart_spec.data_sheet_name, series_spec.value_column
            )));
        };
        let value_col = series_value_col as u16;
        let series = chart.add_series();
        series
            .set_categories((
                chart_spec.data_sheet_name.as_str(),
                first_data_row,
                category_col,
                last_row,
                category_col,
            ))
            .set_values((
                chart_spec.data_sheet_name.as_str(),
                first_data_row,
                value_col,
                last_row,
                value_col,
            ));
        if let Some(name) = series_spec.name.as_ref() {
            if !name.trim().is_empty() {
                series.set_name(name.as_str());
            }
        }
    }

    if let Some(title) = chart_spec.title.as_ref() {
        if !title.trim().is_empty() {
            chart.title().set_name(title);
        }
    }
    if chart_spec.chart_type != PersistedWorkbookChartType::Pie {
        // 2026-03-23: 这里给非饼图设置轴名称，原因是柱线图与散点图脱离上下文后仍需保持可读；目的是减少客户在 Excel 里二次解释字段的成本。
        chart.x_axis().set_name(
            chart_spec
                .x_axis_name
                .as_deref()
                .unwrap_or(&chart_spec.category_column),
        );
        chart.y_axis().set_name(
            chart_spec
                .y_axis_name
                .as_deref()
                .unwrap_or(&chart_spec.value_column),
        );
    }
    if chart_spec.show_legend {
        if let Some(position) = chart_spec.legend_position.as_ref() {
            chart.legend().set_position(map_legend_position(position));
        }
    } else {
        chart.legend().set_hidden();
    }

    let worksheet = workbook
        .worksheet_from_name(&chart_spec.target_sheet_name)
        .map_err(|error| ExportError::WriteExcel(error.to_string()))?;
    worksheet
        .insert_chart(chart_spec.anchor_row, chart_spec.anchor_col, &chart)
        .map_err(|error| ExportError::WriteExcel(error.to_string()))?;

    Ok(())
}

fn normalized_chart_series(chart_spec: &PersistedWorkbookChartSpec) -> Vec<PersistedWorkbookChartSeriesSpec> {
    if !chart_spec.series.is_empty() {
        return chart_spec.series.clone();
    }
    vec![PersistedWorkbookChartSeriesSpec {
        value_column: chart_spec.value_column.clone(),
        name: None,
    }]
}

fn map_chart_type(chart_type: &PersistedWorkbookChartType) -> ChartType {
    match chart_type {
        PersistedWorkbookChartType::Column => ChartType::Column,
        PersistedWorkbookChartType::Line => ChartType::Line,
        PersistedWorkbookChartType::Pie => ChartType::Pie,
        PersistedWorkbookChartType::Scatter => ChartType::Scatter,
    }
}

fn map_legend_position(position: &PersistedWorkbookLegendPosition) -> ChartLegendPosition {
    match position {
        PersistedWorkbookLegendPosition::Top => ChartLegendPosition::Top,
        PersistedWorkbookLegendPosition::Bottom => ChartLegendPosition::Bottom,
        PersistedWorkbookLegendPosition::Left => ChartLegendPosition::Left,
        PersistedWorkbookLegendPosition::Right => ChartLegendPosition::Right,
        PersistedWorkbookLegendPosition::TopRight => ChartLegendPosition::TopRight,
    }
}

fn ensure_parent_dir(output_path: &str) -> Result<(), ExportError> {
    let path = Path::new(output_path);
    let Some(parent) = path.parent() else {
        return Err(ExportError::MissingParentDirectory(output_path.to_string()));
    };
    fs::create_dir_all(parent).map_err(|error| ExportError::CreateOutputDir(error.to_string()))
}

fn escape_csv_field(value: &str) -> String {
    if value.contains(',') || value.contains('"') || value.contains('\n') || value.contains('\r') {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_string()
    }
}

fn write_excel_cell(
    worksheet: &mut rust_xlsxwriter::Worksheet,
    column: &polars::prelude::Column,
    row_index: usize,
    column_index: usize,
    formats: &ExportCellFormats,
) -> Result<bool, rust_xlsxwriter::XlsxError> {
    write_excel_cell_with_row_offset(worksheet, column, row_index, column_index, 1, formats)
}

fn write_excel_cell_with_row_offset(
    worksheet: &mut rust_xlsxwriter::Worksheet,
    column: &polars::prelude::Column,
    row_index: usize,
    column_index: usize,
    row_offset: u32,
    formats: &ExportCellFormats,
) -> Result<bool, rust_xlsxwriter::XlsxError> {
    let row = row_offset + row_index as u32;
    let col = column_index as u16;
    let series = column.as_materialized_series();

    match series.get(row_index) {
        Ok(AnyValue::Null) => Ok(false),
        // 2026-03-23: 这里显式把链式写入收口成 Result<(), _>，原因是当前 xlsxwriter API 在不同基础类型上返回风格不同；目的是让导出层统一处理写单元格错误。
        Ok(AnyValue::Int8(value)) => {
            worksheet.write_number_with_format(row, col, value as f64, &formats.integer_format)?;
            Ok(false)
        }
        Ok(AnyValue::Int16(value)) => {
            worksheet.write_number_with_format(row, col, value as f64, &formats.integer_format)?;
            Ok(false)
        }
        Ok(AnyValue::Int32(value)) => {
            worksheet.write_number_with_format(row, col, value as f64, &formats.integer_format)?;
            Ok(false)
        }
        Ok(AnyValue::Int64(value)) => {
            worksheet.write_number_with_format(row, col, value as f64, &formats.integer_format)?;
            Ok(false)
        }
        Ok(AnyValue::UInt8(value)) => {
            worksheet.write_number_with_format(row, col, value as f64, &formats.integer_format)?;
            Ok(false)
        }
        Ok(AnyValue::UInt16(value)) => {
            worksheet.write_number_with_format(row, col, value as f64, &formats.integer_format)?;
            Ok(false)
        }
        Ok(AnyValue::UInt32(value)) => {
            worksheet.write_number_with_format(row, col, value as f64, &formats.integer_format)?;
            Ok(false)
        }
        Ok(AnyValue::UInt64(value)) => {
            worksheet.write_number_with_format(row, col, value as f64, &formats.integer_format)?;
            Ok(false)
        }
        Ok(AnyValue::Float32(value)) => {
            worksheet.write_number_with_format(row, col, value as f64, &formats.float_format)?;
            Ok(false)
        }
        Ok(AnyValue::Float64(value)) => {
            worksheet.write_number_with_format(row, col, value, &formats.float_format)?;
            Ok(false)
        }
        Ok(AnyValue::Boolean(value)) => {
            worksheet.write_boolean(row, col, value)?;
            Ok(false)
        }
        Ok(_) => {
            let value = series
                .str_value(row_index)
                .map_err(|error| rust_xlsxwriter::XlsxError::ParameterError(error.to_string()))?;
            if should_wrap_text(value.as_ref()) {
                worksheet.write_string_with_format(row, col, value.as_ref(), &formats.wrapped_text_format)?;
                Ok(true)
            } else {
                worksheet.write_string(row, col, value.as_ref())?;
                Ok(false)
            }
        }
        Err(error) => Err(rust_xlsxwriter::XlsxError::ParameterError(error.to_string())),
    }
}

#[derive(Debug, Clone)]
struct ExportCellFormats {
    integer_format: Format,
    float_format: Format,
    wrapped_text_format: Format,
    title_format: Format,
    subtitle_format: Format,
}

// 2026-03-24: 这里集中构造导出默认格式，原因是单表导出和 workbook 导出都要复用同一套“更像报表”的默认样式；目的是在不引入新协议的前提下提升交付成品感。
fn build_export_cell_formats() -> ExportCellFormats {
    ExportCellFormats {
        integer_format: Format::new().set_num_format("#,##0"),
        float_format: Format::new().set_num_format("#,##0.00"),
        wrapped_text_format: Format::new().set_text_wrap(),
        title_format: Format::new()
            .set_bold()
            .set_font_size(16)
            .set_align(FormatAlign::Center),
        subtitle_format: Format::new()
            .set_font_size(11)
            .set_align(FormatAlign::Center),
    }
}

// 2026-03-24: 这里集中计算并写入列宽，原因是单表导出和 workbook 导出都需要同一套“按实际显示值自适应”的规则；目的是避免两条交付链各自维护一份列宽逻辑。
fn apply_auto_column_widths(
    worksheet: &mut Worksheet,
    dataframe: &DataFrame,
) -> Result<(), rust_xlsxwriter::XlsxError> {
    for (column_index, column_name) in dataframe.get_column_names().iter().enumerate() {
        let column = dataframe
            .get_columns()
            .get(column_index)
            .ok_or_else(|| rust_xlsxwriter::XlsxError::ParameterError("列索引越界".to_string()))?;
        let mut max_width = display_width(column_name.as_str());
        let series = column.as_materialized_series();
        for row_index in 0..dataframe.height() {
            let value = series
                .str_value(row_index)
                .map_err(|error| rust_xlsxwriter::XlsxError::ParameterError(error.to_string()))?;
            max_width = max_width.max(display_width(value.as_ref()));
        }
        let width = (max_width + 2).clamp(8, 48) as f64;
        worksheet.set_column_width(column_index as u16, width)?;
    }
    Ok(())
}

// 2026-03-24: 这里用保守规则判断是否启用长文本换行，原因是说明列通常不适合无限拉宽；目的是在默认列宽和可读性之间先取得一个稳定平衡。
fn should_wrap_text(value: &str) -> bool {
    value.contains('\n') || display_width(value) > 32
}

// 2026-03-24: 这里统一输出标题区横幅，原因是标题和副标题都需要支持“单列写入”和“多列横向合并”两种场景；目的是减少 workbook 导出层的重复分支。
fn write_sheet_banner(
    worksheet: &mut Worksheet,
    row: u32,
    last_col: u16,
    value: &str,
    format: &Format,
) -> Result<(), rust_xlsxwriter::XlsxError> {
    if last_col == 0 {
        worksheet.write_string_with_format(row, 0, value, format)?;
    } else {
        worksheet.merge_range(row, 0, row, last_col, value, format)?;
    }
    Ok(())
}

// 2026-03-24: 这里集中写冻结窗格规则，原因是单表导出与 workbook 导出都共享“冻结到首个数据行前”的体验要求；目的是让标题区和表头在长表滚动时保持可见。
fn apply_freeze_panes(
    worksheet: &mut Worksheet,
    header_row: u32,
) -> Result<(), rust_xlsxwriter::XlsxError> {
    let freeze_row = header_row + 1;
    worksheet.set_freeze_panes(freeze_row, 0)?;
    Ok(())
}

// 2026-03-24: 这里集中写自动筛选规则，原因是所有面向交付的数据表都希望开箱即有筛选能力；目的是统一 compose/report/export 三条链路的表头交互体验。
fn apply_autofilter(
    worksheet: &mut Worksheet,
    header_row: u32,
    row_count: usize,
    column_count: usize,
) -> Result<(), rust_xlsxwriter::XlsxError> {
    if row_count == 0 || column_count == 0 {
        return Ok(());
    }
    let last_row = header_row + row_count as u32;
    let last_col = (column_count - 1) as u16;
    worksheet.autofilter(header_row, 0, last_row, last_col)?;
    Ok(())
}

// 2026-03-24: 这里用保守显示宽度估算字符占位，原因是中英文混排在 Excel 里宽度差异明显；目的是让中文字段不会因为按字节或纯字符数计算而明显偏窄。
fn display_width(value: &str) -> usize {
    value
        .chars()
        .map(|ch| if ch.is_ascii() { 1 } else { 2 })
        .sum::<usize>()
}
