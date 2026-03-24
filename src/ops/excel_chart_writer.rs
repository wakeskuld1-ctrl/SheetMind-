use std::collections::BTreeMap;

use rust_xlsxwriter::{Chart, ChartLegendPosition, ChartType, Workbook};
use thiserror::Error;

use crate::frame::workbook_ref_store::{
    PersistedWorkbookChartSeriesSpec, PersistedWorkbookChartSpec, PersistedWorkbookChartType,
    PersistedWorkbookLegendPosition,
};

#[derive(Debug, Clone)]
pub(crate) struct WorksheetBinding {
    pub(crate) row_count: usize,
    pub(crate) header_row: u32,
    pub(crate) column_indices: BTreeMap<String, usize>,
}

#[derive(Debug, Error)]
pub(crate) enum ExcelChartWriterError {
    #[error("图表数据源 sheet 不存在: {0}")]
    MissingDataSheet(String),
    #[error("图表分类列不存在: {0}.{1}")]
    MissingCategoryColumn(String, String),
    #[error("图表数据源没有可用数据行: {0}")]
    EmptyDataSheet(String),
    #[error("图表数值列不存在: {0}.{1}")]
    MissingValueColumn(String, String),
    #[error("{0}")]
    WriteWorkbook(String),
}

pub(crate) fn insert_charts_into_workbook(
    workbook: &mut Workbook,
    worksheet_bindings: &BTreeMap<String, WorksheetBinding>,
    chart_specs: &[PersistedWorkbookChartSpec],
) -> Result<(), ExcelChartWriterError> {
    for chart_spec in chart_specs {
        insert_chart_into_workbook(workbook, worksheet_bindings, chart_spec)?;
    }
    Ok(())
}

fn insert_chart_into_workbook(
    workbook: &mut Workbook,
    worksheet_bindings: &BTreeMap<String, WorksheetBinding>,
    chart_spec: &PersistedWorkbookChartSpec,
) -> Result<(), ExcelChartWriterError> {
    let Some(data_binding) = worksheet_bindings.get(&chart_spec.data_sheet_name) else {
        return Err(ExcelChartWriterError::MissingDataSheet(
            chart_spec.data_sheet_name.clone(),
        ));
    };
    let Some(&category_col) = data_binding.column_indices.get(&chart_spec.category_column) else {
        return Err(ExcelChartWriterError::MissingCategoryColumn(
            chart_spec.data_sheet_name.clone(),
            chart_spec.category_column.clone(),
        ));
    };
    if data_binding.row_count == 0 {
        return Err(ExcelChartWriterError::EmptyDataSheet(
            chart_spec.data_sheet_name.clone(),
        ));
    }

    let first_data_row = data_binding.header_row + 1;
    let last_row = data_binding.header_row + data_binding.row_count as u32;
    let category_col = category_col as u16;
    let mut chart = Chart::new(map_chart_type(&chart_spec.chart_type));
    if let Some(style) = chart_spec.chart_style {
        chart.set_style(style);
    }
    for series_spec in normalized_chart_series(chart_spec) {
        let Some(&series_value_col) = data_binding.column_indices.get(&series_spec.value_column)
        else {
            return Err(ExcelChartWriterError::MissingValueColumn(
                chart_spec.data_sheet_name.clone(),
                series_spec.value_column.clone(),
            ));
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
        .map_err(|error| ExcelChartWriterError::WriteWorkbook(error.to_string()))?;
    worksheet
        .insert_chart(chart_spec.anchor_row, chart_spec.anchor_col, &chart)
        .map_err(|error| ExcelChartWriterError::WriteWorkbook(error.to_string()))?;

    Ok(())
}

fn normalized_chart_series(
    chart_spec: &PersistedWorkbookChartSpec,
) -> Vec<PersistedWorkbookChartSeriesSpec> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalized_chart_series_falls_back_to_value_column() {
        let chart_spec = PersistedWorkbookChartSpec {
            chart_ref: None,
            source_refs: vec![],
            chart_type: PersistedWorkbookChartType::Column,
            target_sheet_name: "Charts".to_string(),
            data_sheet_name: "Data".to_string(),
            category_column: "month".to_string(),
            value_column: "revenue".to_string(),
            title: None,
            series: vec![],
            show_legend: false,
            legend_position: None,
            chart_style: None,
            x_axis_name: None,
            y_axis_name: None,
            anchor_row: 1,
            anchor_col: 0,
        };

        let series = normalized_chart_series(&chart_spec);

        assert_eq!(series.len(), 1);
        assert_eq!(series[0].value_column, "revenue");
        assert_eq!(series[0].name, None);
    }

    #[test]
    fn insert_charts_into_workbook_rejects_missing_data_sheet() {
        let mut workbook = Workbook::new();
        let chart_spec = PersistedWorkbookChartSpec {
            chart_ref: None,
            source_refs: vec![],
            chart_type: PersistedWorkbookChartType::Column,
            target_sheet_name: "Charts".to_string(),
            data_sheet_name: "Missing".to_string(),
            category_column: "month".to_string(),
            value_column: "revenue".to_string(),
            title: None,
            series: vec![],
            show_legend: false,
            legend_position: None,
            chart_style: None,
            x_axis_name: None,
            y_axis_name: None,
            anchor_row: 1,
            anchor_col: 0,
        };

        let result = insert_charts_into_workbook(&mut workbook, &BTreeMap::new(), &[chart_spec]);

        assert!(matches!(
            result,
            Err(ExcelChartWriterError::MissingDataSheet(sheet)) if sheet == "Missing"
        ));
    }
}
