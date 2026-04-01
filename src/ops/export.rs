use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use polars::prelude::{AnyValue, DataFrame, DataType};
use rust_xlsxwriter::{
    Chart, ChartLegendPosition, ChartType, ConditionalFormatBlank, ConditionalFormatCell,
    ConditionalFormatCellRule, ConditionalFormatDuplicate, ConditionalFormatFormula, Format,
    FormatAlign, Workbook, Worksheet,
};
use thiserror::Error;

use crate::frame::loader::LoadedTable;
use crate::frame::workbook_ref_store::{
    PersistedWorkbookChartSeriesSpec, PersistedWorkbookChartSpec, PersistedWorkbookChartType,
    PersistedWorkbookColumnConditionalFormatRule, PersistedWorkbookConditionalFormatKind,
    PersistedWorkbookDraft, PersistedWorkbookLegendPosition, PersistedWorkbookNumberFormatKind,
    PersistedWorkbookSheet, PersistedWorkbookSheetKind,
};
use crate::ops::excel_chart_writer::{
    WorksheetBinding as ChartWorksheetBinding,
    insert_charts_into_workbook as write_charts_to_workbook,
};

#[derive(Debug, Error)]
pub enum ExportError {
    #[error("鐎电厧鍤捄顖氱窞缂傚搫鐨悥鍓佹窗瑜? {0}")]
    MissingParentDirectory(String),
    #[error("閺冪姵纭堕崚娑樼紦鐎电厧鍤惄顔肩秿: {0}")]
    CreateOutputDir(String),
    #[error("閺冪姵纭堕崘娆忓毉 CSV: {0}")]
    WriteCsv(String),
    #[error("閺冪姵纭堕崘娆忓毉 Excel: {0}")]
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
            // 2026-03-23: 鏉╂瑩鍣烽幐澶屾埂鐎圭偛宕熼崗鍐╃壐缁鐎烽崘?Excel閿涘苯甯崶鐘虫Ц鐎电厧鍤惃鍕波閺嬫粏銆冩潻妯款洣缂佈呯敾鐞氼偄顓归幋閿嬬湴閸滃被鈧線鈧繗顫嬮崪灞惧笓鎼村骏绱遍惄顔炬畱閺勵垶浼╅崗宥嗗閺堝鈧ジ鍏橀柅鈧崠鏍ㄥ灇閺傚洦婀伴妴?
            wrapped_row |=
                write_excel_cell(worksheet, column, row_index, column_index, &cell_formats)
                    .map_err(|error| ExportError::WriteExcel(error.to_string()))?;
        }
        // 2026-03-24: 鏉╂瑩鍣烽崷銊ュ礋鐞涖劌顕遍崙鐑樻娑撴椽鏆遍弬鍥ㄦ拱鐞涘矁藟娑撯偓娑擃亙绻氱€瑰牐顢戞姗堢礉閸樼喎娲滈弰顖欑矌閺?wrapText 娴犲秴褰查懗鍊燁唨妫ｆ牕鐫嗛惇瀣崳閺夈儴绻冮幍渚婄幢閻╊喚娈戦弰顖濐唨鐠囧瓨妲戦崚妤€婀?Excel 闁插本澧﹀鈧崥搴㈡纯閹恒儴绻庨崣顖濐嚢閻樿埖鈧降鈧?
        if wrapped_row {
            worksheet
                .set_row_height((row_index + 1) as u32, 36)
                .map_err(|error| ExportError::WriteExcel(error.to_string()))?;
        }
    }
    // 2026-03-24: 鏉╂瑩鍣风紒娆忓礋鐞涖劌顕遍崙楦克夐懛顏勫З缁涙盯鈧绱濋崢鐔锋礈閺勵垯绗熼崝锛勬暏閹村嘲顕遍崙鍝勬倵閺堚偓鐢瓕顫嗛崝銊ょ稊鐏忚鲸妲搁幐澶婂灙缁涙盯鈧绱遍惄顔炬畱閺勵垱濡搁崺铏诡攨閸欘垳鏁ら幀褏娲块幒銉︾焽閸掓澘绨崇仦鍌氼嚤閸戦缚鍏橀崝娑栤偓?
    apply_autofilter(
        worksheet,
        0,
        loaded.dataframe.height(),
        loaded.dataframe.width(),
    )
    .map_err(|error| ExportError::WriteExcel(error.to_string()))?;
    // 2026-03-24: 鏉╂瑩鍣烽崷銊ュ礋鐞涖劌顕遍崙鐑樻閸愯崵绮ㄧ悰銊ャ仈鐞涘矉绱濋崢鐔锋礈閺勵垰顓归幋閿嬬叀閻鏆辩悰銊︽闂団偓鐟曚礁顫愮紒鍫滅箽閻ｆ瑥鐡у▓鍏哥瑐娑撳鏋冮敍娑氭窗閻ㄥ嫭妲哥拋鈺佺唨绾偓鐎电厧鍤稊鐔峰徔婢跺洦娓剁亸蹇撳讲鐠囩粯鈧佲偓?
    apply_freeze_panes(worksheet, 0, false)
        .map_err(|error| ExportError::WriteExcel(error.to_string()))?;
    // 2026-03-24: 鏉╂瑩鍣烽崷銊ュ礋鐞涖劌顕遍崙鍝勬倵缂佺喍绔寸悰銉︽▔瀵繐鍨€规枻绱濋崢鐔锋礈閺勵垶绮拋銈咁啍鎼达箑顕梹鍨灙閸氬秴鎷版稉顓熸瀮鐎涙顔屾稉宥呭几婵傛枻绱遍惄顔炬畱閺勵垵顔€鐎电厧鍤惃鍕礋鐞涖劌绱戠粻鍗炲祮鐠囦紮绱濇稉宥呯箑鐎广垺鍩涢崘宥嗗瀹搞儲瀚嬮崚妤€顔旈妴?
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
    let mut worksheet_bindings = BTreeMap::<String, ChartWorksheetBinding>::new();

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
                // 2026-03-24: 鏉╂瑩鍣烽幎濠冪垼妫版ê灏Ο顏勬倻閸氬牆鑻熼崚鐗堟殻瀵姵鏆熼幑顔裤€冪€硅棄瀹抽敍灞藉斧閸ョ姵妲搁崣顏勫晸閸?A1 閺囨潙鍎氶弲顕€鈧碍鏆熼幑顔裤€冮敍娑氭窗閻ㄥ嫭妲搁幎?report_delivery 妞や絻绻樻稉鈧銉﹀鏉╂垟鈧粍鐪归幎銉ь焾閳ユ繆顫囬幇鐔粹偓?
                write_sheet_banner(
                    worksheet,
                    0,
                    last_title_col,
                    title,
                    &cell_formats.title_format,
                )
                .map_err(|error| ExportError::WriteExcel(error.to_string()))?;
            }
        }
        if let Some(subtitle) = worksheet_snapshot.subtitle.as_ref() {
            if !subtitle.trim().is_empty() {
                // 2026-03-24: 鏉╂瑩鍣烽幎濠傚閺嶅洭顣介崪灞剧垼妫版ü绻氶幐浣告倱閺嶉娈戝Ο顏勬倻缂佹挻鐎敍灞藉斧閸ョ姵妲搁弽鍥暯閸栨椽娓剁憰浣歌埌閹存劗菙鐎规俺顫嬬憴澶婃健閿涙稓娲伴惃鍕Ц闁灝鍘ら崜顖涚垼妫版ü绮涚€涖倝娴傞梿鎯版儰閸?A2閵?
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
                // 2026-03-23: 鏉╂瑩鍣风拋鈺侇樋 Sheet 鐎电厧鍤径宥囨暏閸氬奔绔存總妤冭閸ㄥ鍟撻崙娲偓鏄忕帆閿涘苯甯崶鐘虫Ц report_delivery 閸?compose_workbook 闁垝绱扮挧鎷岀箹闁插矉绱遍惄顔炬畱閺勵垯绻氱拠浣告禈鐞涖劍鏆熼幑顔界爱閸楁洖鍘撻弽闂寸瘍娣囨繃瀵旈崣顖濐吀缁犳娈戦弫鏉库偓鑲╄閸ㄥ鈧?
                wrapped_row |= write_excel_cell_with_row_offset(
                    worksheet,
                    column,
                    row_index,
                    column_index,
                    header_row + 1,
                    &cell_formats,
                    number_format_for_column(
                        worksheet_snapshot,
                        dataframe.get_column_names()[column_index].as_str(),
                    ),
                )
                .map_err(|error| ExportError::WriteExcel(error.to_string()))?;
            }
            if wrapped_row {
                worksheet
                    .set_row_height(header_row + 1 + row_index as u32, 36)
                    .map_err(|error| ExportError::WriteExcel(error.to_string()))?;
            }
        }
        // 2026-03-24: 鏉╂瑩鍣风紒?workbook 閸愬懏鐦″鐘恒€冪紒鐔剁閸旂姾鍤滈崝銊х摣闁绱濋崢鐔锋礈閺?compose_workbook / report_delivery 闁姤妲搁棃銏犳倻娴溿倓绮惃鍕波閺嬫粏銆冮敍娑氭窗閻ㄥ嫭妲哥拋鈺冩暏閹撮攱澧﹀鈧崡瀹犲厴缁涙冻绱濇稉宥囨暏閹靛濮╅崘宥囧仯娑撯偓闁秲鈧?
        apply_autofilter(worksheet, header_row, dataframe.height(), dataframe.width())
            .map_err(|error| ExportError::WriteExcel(error.to_string()))?;
        // 2026-03-24: 鏉╂瑩鍣烽幐澶婃倗 sheet 閻ㄥ嫭鐖ｆ０妯哄隘娑撳氦銆冩径缈犵秴缂冾喚绮烘稉鈧崘鑽ょ波缁愭鐗搁敍灞藉斧閸ョ姵妲?report_delivery 娑?compose_workbook 闁粙娓剁憰浣告躬濠婃艾濮╅弮鏈电箽閻ｆ瑨銆冩径杈剧幢閻╊喚娈戦弰顖涘Ω閸愯崵绮ㄧ憴鍕灟濞屽鍩岄崗顒€鍙℃禍銈勭帛鐏炲倽鈧奔绗夐弰顖涙殠閽€钘夋躬濡剝婢橀柅鏄忕帆闁插被鈧?
        apply_freeze_panes(
            worksheet,
            header_row,
            matches!(
                worksheet_snapshot.sheet_kind,
                PersistedWorkbookSheetKind::DataSheet
            ),
        )
        .map_err(|error| ExportError::WriteExcel(error.to_string()))?;
        // 2026-03-24: 这里在 sheet 级导出选项冻结后补写条件格式，原因是负值预警和空白提醒必须跟着 workbook 草稿稳定落地；目的是让 report_delivery 的交付偏好真正体现在成品 Excel 里。
        apply_conditional_formats(
            worksheet,
            worksheet_snapshot,
            &dataframe,
            header_row,
            &cell_formats,
        )
        .map_err(|error| ExportError::WriteExcel(error.to_string()))?;
        // 2026-03-24: 鏉╂瑩鍣风紒?workbook 閸愬懏鐦″鐘虫殶閹诡喛銆冪悰銉ㄥ殰閸斻劌鍨€规枻绱濋崢鐔锋礈閺?report_delivery / compose_workbook 闁€熻泲閸氬奔绔撮弶鈥叉唉娴犳﹢鎽肩捄顖ょ幢閻╊喚娈戦弰顖涘Ω閳ユ粌鍨€硅棄褰茬拠缁樷偓褉鈧繀绔村▎鈩冣偓褎鐭囬崚鏉垮彆閸忓崬顕遍崙鍝勭湴閵?
        apply_auto_column_widths(worksheet, &dataframe)
            .map_err(|error| ExportError::WriteExcel(error.to_string()))?;

        worksheet_bindings.insert(
            worksheet_snapshot.sheet_name.clone(),
            ChartWorksheetBinding {
                row_count: dataframe.height(),
                header_row,
                column_indices,
            },
        );
    }

    write_charts_to_workbook(&mut workbook, &worksheet_bindings, &draft.charts)
        .map_err(|error| ExportError::WriteExcel(error.to_string()))?;

    workbook
        .save(output_path)
        .map_err(|error| ExportError::WriteExcel(error.to_string()))
}

#[allow(dead_code)]
fn insert_chart_into_workbook(
    workbook: &mut Workbook,
    worksheet_bindings: &BTreeMap<String, ChartWorksheetBinding>,
    chart_spec: &PersistedWorkbookChartSpec,
) -> Result<(), ExportError> {
    let Some(data_binding) = worksheet_bindings.get(&chart_spec.data_sheet_name) else {
        return Err(ExportError::WriteExcel(format!(
            "閸ユ崘銆冮弫鐗堝祦濠?sheet 娑撳秴鐡ㄩ崷? {}",
            chart_spec.data_sheet_name
        )));
    };
    let Some(&category_col) = data_binding.column_indices.get(&chart_spec.category_column) else {
        return Err(ExportError::WriteExcel(format!(
            "閸ユ崘銆冮崚鍡欒閸掓ぞ绗夌€涙ê婀? {}.{}",
            chart_spec.data_sheet_name, chart_spec.category_column
        )));
    };
    if data_binding.row_count == 0 {
        return Err(ExportError::WriteExcel(format!(
            "閸ユ崘銆冮弫鐗堝祦濠ф劖鐥呴張澶婂讲閻劍鏆熼幑顔款攽: {}",
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
        let Some(&series_value_col) = data_binding.column_indices.get(&series_spec.value_column)
        else {
            return Err(ExportError::WriteExcel(format!(
                "閸ユ崘銆冮弫鏉库偓鐓庡灙娑撳秴鐡ㄩ崷? {}.{}",
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
        // 2026-03-23: 鏉╂瑩鍣风紒娆撴姜妤楃厧娴樼拋鍓х枂鏉炴潙鎮曠粔甯礉閸樼喎娲滈弰顖涚叴缁惧灝娴樻稉搴㈡殠閻愮懓娴橀懘杈╊瀲娑撳﹣绗呴弬鍥ф倵娴犲秹娓舵穱婵囧瘮閸欘垵顕伴敍娑氭窗閻ㄥ嫭妲搁崙蹇撶毌鐎广垺鍩涢崷?Excel 闁插奔绨╁▎陇袙闁插﹤鐡у▓鐢垫畱閹存劖婀伴妴?
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

#[allow(dead_code)]
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

#[allow(dead_code)]
fn map_chart_type(chart_type: &PersistedWorkbookChartType) -> ChartType {
    match chart_type {
        PersistedWorkbookChartType::Column => ChartType::Column,
        PersistedWorkbookChartType::Line => ChartType::Line,
        PersistedWorkbookChartType::Pie => ChartType::Pie,
        PersistedWorkbookChartType::Scatter => ChartType::Scatter,
    }
}

#[allow(dead_code)]
fn map_legend_position(position: &PersistedWorkbookLegendPosition) -> ChartLegendPosition {
    match position {
        PersistedWorkbookLegendPosition::Top => ChartLegendPosition::Top,
        PersistedWorkbookLegendPosition::Bottom => ChartLegendPosition::Bottom,
        PersistedWorkbookLegendPosition::Left => ChartLegendPosition::Left,
        PersistedWorkbookLegendPosition::Right => ChartLegendPosition::Right,
        PersistedWorkbookLegendPosition::TopRight => ChartLegendPosition::TopRight,
    }
}

#[allow(dead_code)]
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
    write_excel_cell_with_row_offset(worksheet, column, row_index, column_index, 1, formats, None)
}

fn write_excel_cell_with_row_offset(
    worksheet: &mut rust_xlsxwriter::Worksheet,
    column: &polars::prelude::Column,
    row_index: usize,
    column_index: usize,
    row_offset: u32,
    formats: &ExportCellFormats,
    number_format: Option<PersistedWorkbookNumberFormatKind>,
) -> Result<bool, rust_xlsxwriter::XlsxError> {
    let row = row_offset + row_index as u32;
    let col = column_index as u16;
    let series = column.as_materialized_series();

    match series.get(row_index) {
        Ok(AnyValue::Null) => Ok(false),
        Ok(AnyValue::Int8(value)) => {
            worksheet.write_number_with_format(
                row,
                col,
                value as f64,
                resolve_numeric_format(formats, number_format.as_ref(), true),
            )?;
            Ok(false)
        }
        Ok(AnyValue::Int16(value)) => {
            worksheet.write_number_with_format(
                row,
                col,
                value as f64,
                resolve_numeric_format(formats, number_format.as_ref(), true),
            )?;
            Ok(false)
        }
        Ok(AnyValue::Int32(value)) => {
            worksheet.write_number_with_format(
                row,
                col,
                value as f64,
                resolve_numeric_format(formats, number_format.as_ref(), true),
            )?;
            Ok(false)
        }
        Ok(AnyValue::Int64(value)) => {
            worksheet.write_number_with_format(
                row,
                col,
                value as f64,
                resolve_numeric_format(formats, number_format.as_ref(), true),
            )?;
            Ok(false)
        }
        Ok(AnyValue::UInt8(value)) => {
            worksheet.write_number_with_format(
                row,
                col,
                value as f64,
                resolve_numeric_format(formats, number_format.as_ref(), true),
            )?;
            Ok(false)
        }
        Ok(AnyValue::UInt16(value)) => {
            worksheet.write_number_with_format(
                row,
                col,
                value as f64,
                resolve_numeric_format(formats, number_format.as_ref(), true),
            )?;
            Ok(false)
        }
        Ok(AnyValue::UInt32(value)) => {
            worksheet.write_number_with_format(
                row,
                col,
                value as f64,
                resolve_numeric_format(formats, number_format.as_ref(), true),
            )?;
            Ok(false)
        }
        Ok(AnyValue::UInt64(value)) => {
            worksheet.write_number_with_format(
                row,
                col,
                value as f64,
                resolve_numeric_format(formats, number_format.as_ref(), true),
            )?;
            Ok(false)
        }
        Ok(AnyValue::Float32(value)) => {
            worksheet.write_number_with_format(
                row,
                col,
                value as f64,
                resolve_numeric_format(formats, number_format.as_ref(), false),
            )?;
            Ok(false)
        }
        Ok(AnyValue::Float64(value)) => {
            worksheet.write_number_with_format(
                row,
                col,
                value,
                resolve_numeric_format(formats, number_format.as_ref(), false),
            )?;
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
                worksheet.write_string_with_format(
                    row,
                    col,
                    value.as_ref(),
                    &formats.wrapped_text_format,
                )?;
                Ok(true)
            } else {
                worksheet.write_string(row, col, value.as_ref())?;
                Ok(false)
            }
        }
        Err(error) => Err(rust_xlsxwriter::XlsxError::ParameterError(
            error.to_string(),
        )),
    }
}

#[derive(Debug, Clone)]
struct ExportCellFormats {
    integer_format: Format,
    float_format: Format,
    currency_format: Format,
    percent_format: Format,
    wrapped_text_format: Format,
    negative_red_format: Format,
    null_warning_format: Format,
    duplicate_warn_format: Format,
    high_value_highlight_format: Format,
    between_warn_format: Format,
    title_format: Format,
    subtitle_format: Format,
}

// 2026-03-24: ????????????????????? workbook ????????????????????????????????
fn build_export_cell_formats() -> ExportCellFormats {
    ExportCellFormats {
        integer_format: Format::new().set_num_format("#,##0"),
        float_format: Format::new().set_num_format("#,##0.00"),
        currency_format: Format::new().set_num_format("\u{00A5}#,##0.00"),
        percent_format: Format::new().set_num_format("0.00%"),
        wrapped_text_format: Format::new().set_text_wrap(),
        negative_red_format: Format::new()
            .set_font_color("9C0006")
            .set_background_color("FFC7CE"),
        null_warning_format: Format::new()
            .set_font_color("9C6500")
            .set_background_color("FFEB9C"),
        duplicate_warn_format: Format::new()
            .set_font_color("C65911")
            .set_background_color("FCE4D6"),
        high_value_highlight_format: Format::new()
            .set_font_color("006100")
            .set_background_color("C6EFCE"),
        between_warn_format: Format::new()
            .set_font_color("9C6500")
            .set_background_color("FFEB9C"),
        title_format: Format::new()
            .set_bold()
            .set_font_size(16)
            .set_align(FormatAlign::Center),
        subtitle_format: Format::new()
            .set_font_size(11)
            .set_align(FormatAlign::Center),
    }
}

// 2026-03-24: ???????????????????? workbook ????????????????????????????????????????????
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
        let mut has_wrapped_text = should_wrap_text(column_name.as_str());
        let series = column.as_materialized_series();
        for row_index in 0..dataframe.height() {
            let value = series
                .str_value(row_index)
                .map_err(|error| rust_xlsxwriter::XlsxError::ParameterError(error.to_string()))?;
            max_width = max_width.max(display_width(value.as_ref()));
            has_wrapped_text |= should_wrap_text(value.as_ref());
        }
        // 2026-03-24: 这里把列宽策略改成“按列类型分档”，原因是交付层同时存在长说明列和数值列时，统一上限会把宽表横向拉得过长；目的是让说明列保留可读性、数值列保持紧凑。
        let width = recommended_column_width(series.dtype(), max_width, has_wrapped_text) as f64;
        worksheet.set_column_width(column_index as u16, width)?;
    }
    Ok(())
}

// 2026-03-24: 这里把列宽建议单独抽出来，原因是宽表优化需要把“数据类型”和“是否长文本”两个维度一起考虑；目的是让后续条件格式、导出模板继续复用同一套列宽基线。
fn recommended_column_width(
    dtype: &DataType,
    max_content_width: usize,
    has_wrapped_text: bool,
) -> usize {
    let desired_width = max_content_width.saturating_add(2);
    if dtype.is_primitive_numeric() {
        return desired_width.clamp(10, 14);
    }
    if dtype.is_bool() {
        return desired_width.clamp(8, 10);
    }
    if dtype.is_temporal() {
        return desired_width.clamp(10, 20);
    }
    if has_wrapped_text {
        return desired_width.clamp(12, 36);
    }
    desired_width.clamp(8, 24)
}

// 2026-03-24: 鏉╂瑩鍣烽悽銊ょ箽鐎瑰牐顫夐崚娆忓灲閺傤厽妲搁崥锕€鎯庨悽銊╂毐閺傚洦婀伴幑銏ｎ攽閿涘苯甯崶鐘虫Ц鐠囧瓨妲戦崚妤呪偓姘埗娑撳秹鈧倸鎮庨弮鐘绘閹峰顔旈敍娑氭窗閻ㄥ嫭妲搁崷銊╃帛鐠併倕鍨€硅棄鎷伴崣顖濐嚢閹傜闂傛潙鍘涢崣鏍х繁娑撯偓娑擃亞菙鐎规艾閽╃悰掳鈧?
fn should_wrap_text(value: &str) -> bool {
    value.contains('\n') || display_width(value) > 32
}

// 2026-03-24: 鏉╂瑩鍣风紒鐔剁鏉堟挸鍤弽鍥暯閸栫儤铆楠炲拑绱濋崢鐔锋礈閺勵垱鐖ｆ０妯烘嫲閸擃垱鐖ｆ０姗€鍏橀棁鈧憰浣规暜閹镐讲鈧粌宕熼崚妤€鍟撻崗銉⑩偓婵嗘嫲閳ユ粌顦块崚妤伱崥鎴濇値楠炲灈鈧繀琚辩粔宥呮簚閺咁垽绱遍惄顔炬畱閺勵垰鍣虹亸?workbook 鐎电厧鍤仦鍌滄畱闁插秴顦查崚鍡樻暜閵?
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

// 2026-03-24: ???????????????? Excel Format???? workbook ??????? currency / percent ??????
// ?????????????????????????????
fn resolve_numeric_format<'a>(
    formats: &'a ExportCellFormats,
    number_format: Option<&PersistedWorkbookNumberFormatKind>,
    is_integer: bool,
) -> &'a Format {
    match number_format {
        Some(PersistedWorkbookNumberFormatKind::Currency) => &formats.currency_format,
        Some(PersistedWorkbookNumberFormatKind::Percent) => &formats.percent_format,
        Some(PersistedWorkbookNumberFormatKind::Integer) => &formats.integer_format,
        Some(PersistedWorkbookNumberFormatKind::Decimal2) => &formats.float_format,
        None if is_integer => &formats.integer_format,
        None => &formats.float_format,
    }
}

// 2026-03-24: ??????? workbook ????????????????????????????????
// ???? report_delivery ??????????? xlsx ???
fn number_format_for_column(
    worksheet_snapshot: &PersistedWorkbookSheet,
    column_name: &str,
) -> Option<PersistedWorkbookNumberFormatKind> {
    worksheet_snapshot
        .export_options
        .as_ref()
        .and_then(|options| {
            options
                .number_formats
                .iter()
                .find(|rule| rule.column == column_name)
                .map(|rule| rule.kind.clone())
        })
}

// 2026-03-24: 这里把条件格式写出收口成单独函数，原因是条件格式和单元格值写入解耦后更符合导出层 SRP；目的是让后续继续扩“异常高亮/缺失提醒”时不干扰基础写值逻辑。
fn apply_conditional_formats(
    worksheet: &mut Worksheet,
    worksheet_snapshot: &PersistedWorkbookSheet,
    dataframe: &DataFrame,
    header_row: u32,
    formats: &ExportCellFormats,
) -> Result<(), rust_xlsxwriter::XlsxError> {
    if dataframe.height() == 0 {
        return Ok(());
    }

    let Some(export_options) = worksheet_snapshot.export_options.as_ref() else {
        return Ok(());
    };

    for rule in &export_options.conditional_formats {
        if matches!(
            rule.kind,
            PersistedWorkbookConditionalFormatKind::CompositeDuplicateWarn
        ) {
            apply_conditional_format_rule(
                worksheet,
                rule,
                dataframe,
                0,
                header_row + 1,
                header_row + dataframe.height() as u32,
                formats,
            )?;
            continue;
        }
        if let Some(column_index) = dataframe
            .get_column_names()
            .iter()
            .position(|column_name| column_name.as_str() == rule.column)
        {
            apply_conditional_format_rule(
                worksheet,
                rule,
                dataframe,
                column_index as u16,
                header_row + 1,
                header_row + dataframe.height() as u32,
                formats,
            )?;
        }
    }

    Ok(())
}

// 2026-03-24: 这里把列级条件格式映射成 rust_xlsxwriter 规则，原因是 workbook 草稿只保留最小可持久化语义；目的是避免上层直接暴露 Excel 公式 DSL。
fn apply_conditional_format_rule(
    worksheet: &mut Worksheet,
    rule: &PersistedWorkbookColumnConditionalFormatRule,
    dataframe: &DataFrame,
    column_index: u16,
    first_data_row: u32,
    last_data_row: u32,
    formats: &ExportCellFormats,
) -> Result<(), rust_xlsxwriter::XlsxError> {
    match rule.kind {
        PersistedWorkbookConditionalFormatKind::NegativeRed => {
            let conditional_format = ConditionalFormatCell::new()
                .set_rule(ConditionalFormatCellRule::LessThan(0))
                .set_format(&formats.negative_red_format);
            worksheet.add_conditional_format(
                first_data_row,
                column_index,
                last_data_row,
                column_index,
                &conditional_format,
            )?;
        }
        PersistedWorkbookConditionalFormatKind::NullWarning => {
            let conditional_format =
                ConditionalFormatBlank::new().set_format(&formats.null_warning_format);
            worksheet.add_conditional_format(
                first_data_row,
                column_index,
                last_data_row,
                column_index,
                &conditional_format,
            )?;
        }
        PersistedWorkbookConditionalFormatKind::DuplicateWarn => {
            let conditional_format =
                ConditionalFormatDuplicate::new().set_format(&formats.duplicate_warn_format);
            worksheet.add_conditional_format(
                first_data_row,
                column_index,
                last_data_row,
                column_index,
                &conditional_format,
            )?;
        }
        PersistedWorkbookConditionalFormatKind::HighValueHighlight => {
            let threshold = parse_conditional_threshold(rule)?;
            let conditional_format = ConditionalFormatCell::new()
                .set_rule(ConditionalFormatCellRule::GreaterThan(threshold))
                .set_format(&formats.high_value_highlight_format);
            worksheet.add_conditional_format(
                first_data_row,
                column_index,
                last_data_row,
                column_index,
                &conditional_format,
            )?;
        }
        PersistedWorkbookConditionalFormatKind::PercentLowWarn => {
            let threshold = parse_conditional_threshold(rule)?;
            let conditional_format = ConditionalFormatCell::new()
                .set_rule(ConditionalFormatCellRule::LessThan(threshold))
                .set_format(&formats.null_warning_format);
            worksheet.add_conditional_format(
                first_data_row,
                column_index,
                last_data_row,
                column_index,
                &conditional_format,
            )?;
        }
        PersistedWorkbookConditionalFormatKind::BetweenWarn => {
            let (min_threshold, max_threshold) = parse_conditional_between_thresholds(rule)?;
            let conditional_format = ConditionalFormatCell::new()
                .set_rule(ConditionalFormatCellRule::Between(
                    min_threshold,
                    max_threshold,
                ))
                .set_format(&formats.between_warn_format);
            worksheet.add_conditional_format(
                first_data_row,
                column_index,
                last_data_row,
                column_index,
                &conditional_format,
            )?;
        }
        PersistedWorkbookConditionalFormatKind::CompositeDuplicateWarn => {
            let columns = if rule.columns.is_empty() {
                vec![rule.column.clone()]
            } else {
                rule.columns.clone()
            };
            let (first_column, last_column, formula) = build_composite_duplicate_formula(
                dataframe,
                columns.as_slice(),
                first_data_row,
                last_data_row,
            )?;
            let conditional_format = ConditionalFormatFormula::new()
                .set_rule(formula.as_str())
                .set_format(&formats.duplicate_warn_format);
            worksheet.add_conditional_format(
                first_data_row,
                first_column,
                last_data_row,
                last_column,
                &conditional_format,
            )?;
        }
    }
    Ok(())
}

// 2026-03-24: 这里把阈值解析收口成独立函数，原因是多种阈值型条件格式都会复用同一段解析逻辑；目的是避免在每个分支里重复拼装错误。
fn parse_conditional_threshold(
    rule: &PersistedWorkbookColumnConditionalFormatRule,
) -> Result<f64, rust_xlsxwriter::XlsxError> {
    let threshold = rule.threshold.as_ref().ok_or_else(|| {
        rust_xlsxwriter::XlsxError::ParameterError(format!(
            "条件格式 `{}` 缺少 threshold",
            rule.column
        ))
    })?;
    threshold.parse::<f64>().map_err(|_| {
        rust_xlsxwriter::XlsxError::ParameterError(format!(
            "条件格式 `{}` 的 threshold 不是合法数字",
            rule.column
        ))
    })
}

// 2026-03-25: 这里单独解析区间阈值，原因是 between 规则需要双阈值且错误文案应当和单阈值规则区分；目的是让范围型条件格式的参数错误更容易定位。
fn parse_conditional_between_thresholds(
    rule: &PersistedWorkbookColumnConditionalFormatRule,
) -> Result<(f64, f64), rust_xlsxwriter::XlsxError> {
    let min_threshold = rule.min_threshold.as_ref().ok_or_else(|| {
        rust_xlsxwriter::XlsxError::ParameterError(format!(
            "条件格式 `{}` 缺少 min_threshold",
            rule.column
        ))
    })?;
    let max_threshold = rule.max_threshold.as_ref().ok_or_else(|| {
        rust_xlsxwriter::XlsxError::ParameterError(format!(
            "条件格式 `{}` 缺少 max_threshold",
            rule.column
        ))
    })?;
    let min_threshold = min_threshold.parse::<f64>().map_err(|_| {
        rust_xlsxwriter::XlsxError::ParameterError(format!(
            "条件格式 `{}` 的 min_threshold 不是合法数字",
            rule.column
        ))
    })?;
    let max_threshold = max_threshold.parse::<f64>().map_err(|_| {
        rust_xlsxwriter::XlsxError::ParameterError(format!(
            "条件格式 `{}` 的 max_threshold 不是合法数字",
            rule.column
        ))
    })?;
    Ok((min_threshold, max_threshold))
}

// 2026-03-25: 这里把复合键判重公式收口成辅助函数，原因是 Excel 公式字符串拼装细节较多；目的是让多列共同判重的行为稳定可复用。
fn build_composite_duplicate_formula(
    dataframe: &DataFrame,
    columns: &[String],
    first_data_row: u32,
    last_data_row: u32,
) -> Result<(u16, u16, String), rust_xlsxwriter::XlsxError> {
    if columns.len() < 2 {
        return Err(rust_xlsxwriter::XlsxError::ParameterError(
            "composite_duplicate_warn 至少需要两个 columns".to_string(),
        ));
    }

    let mut indices = Vec::<u16>::with_capacity(columns.len());
    for column_name in columns {
        let column_index = dataframe
            .get_column_names()
            .iter()
            .position(|item| item.as_str() == column_name)
            .ok_or_else(|| {
                rust_xlsxwriter::XlsxError::ParameterError(format!(
                    "条件格式列不存在: {}",
                    column_name
                ))
            })? as u16;
        indices.push(column_index);
    }
    indices.sort_unstable();

    let first_column = *indices.first().expect("at least one column");
    let last_column = *indices.last().expect("at least one column");
    let mut formula = String::from("=COUNTIFS(");
    for (position, column_index) in indices.iter().enumerate() {
        if position > 0 {
            formula.push(',');
        }
        let column_letter = excel_column_name(*column_index);
        formula.push_str(&format!(
            "${}${}:${}${},${}${}",
            column_letter,
            first_data_row + 1,
            column_letter,
            last_data_row + 1,
            column_letter,
            first_data_row + 1
        ));
    }
    formula.push_str(")>1");
    Ok((first_column, last_column, formula))
}

// 2026-03-25: 这里补一个最小列号转 Excel 字母的辅助函数，原因是复合键条件格式要生成 A1 风格公式；目的是避免引入额外依赖或把列名转换逻辑散落在字符串拼接里。
fn excel_column_name(column_index: u16) -> String {
    let mut index = column_index as usize;
    let mut letters = Vec::new();
    loop {
        letters.push(((index % 26) as u8 + b'A') as char);
        if index < 26 {
            break;
        }
        index = index / 26 - 1;
    }
    letters.iter().rev().collect()
}

// 2026-03-24: ???????????????????? workbook ???????????????????????????????????????????
fn apply_freeze_panes(
    worksheet: &mut Worksheet,
    header_row: u32,
    freeze_first_column: bool,
) -> Result<(), rust_xlsxwriter::XlsxError> {
    let freeze_row = header_row + 1;
    let freeze_col = if freeze_first_column { 1 } else { 0 };
    worksheet.set_freeze_panes(freeze_row, freeze_col)?;
    Ok(())
}

// 2026-03-24: 鏉╂瑩鍣烽梿鍡曡厬閸愭瑨鍤滈崝銊х摣闁顫夐崚娆欑礉閸樼喎娲滈弰顖涘閺堝娼伴崥鎴滄唉娴犳娈戦弫鐗堝祦鐞涖劑鍏樼敮灞炬箿瀵偓缁犲崬宓嗛張澶岀摣闁鍏橀崝娑崇幢閻╊喚娈戦弰顖滅埠娑撯偓 compose/report/export 娑撳娼柧鎹愮熅閻ㄥ嫯銆冩径缈犳唉娴滄帊缍嬫灞烩偓?
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

// 2026-03-24: 鏉╂瑩鍣烽悽銊ょ箽鐎瑰牊妯夌粈鍝勵啍鎼达缚鍙婄粻妤€鐡х粭锕€宕版担宥忕礉閸樼喎娲滈弰顖欒厬閼昏鲸鏋冨ǎ閿嬪笓閸?Excel 闁插苯顔旀惔锕€妯婂鍌涙閺勬拝绱遍惄顔炬畱閺勵垵顔€娑擃厽鏋冪€涙顔屾稉宥勭窗閸ョ姳璐熼幐澶婄摟閼哄倹鍨ㄧ痪顖氱摟缁楋附鏆熺拋锛勭暬閼板本妲戦弰鎯т焊缁愬嫨鈧?
fn display_width(value: &str) -> usize {
    value
        .chars()
        .map(|ch| if ch.is_ascii() { 1 } else { 2 })
        .sum::<usize>()
}
