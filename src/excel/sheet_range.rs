use calamine::{Data, Range, Reader, open_workbook_auto};
use serde::Serialize;
use thiserror::Error;

// 2026-03-22: 这里定义显式矩形区域，目的是把 Excel 风格区域坐标统一收口成 inspect/load 共用的数据结构。
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ExcelRegion {
    pub start_row: usize,
    pub start_col: usize,
    pub end_row: usize,
    pub end_col: usize,
}

impl ExcelRegion {
    // 2026-03-22: 这里提供 Excel 风格区域字符串输出，目的是让 Tool 响应直接返回用户熟悉的 A1:C10 表达。
    pub fn to_range_string(&self) -> String {
        format!(
            "{}{}:{}{}",
            column_number_to_letters(self.start_col),
            self.start_row,
            column_number_to_letters(self.end_col),
            self.end_row
        )
    }

    // 2026-03-22: 这里计算区域高度，目的是统一给 inspect/load 生成行数估计与边界校验。
    pub fn row_count(&self) -> usize {
        self.end_row - self.start_row + 1
    }

    // 2026-03-22: 这里计算区域宽度，目的是统一给 inspect/load 生成列数估计与边界校验。
    pub fn column_count(&self) -> usize {
        self.end_col - self.start_col + 1
    }
}

// 2026-03-22: 这里定义样本行结构，目的是让上层既能看到原始行号，也能看到区域内原始值。
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SheetSampleRow {
    pub row_number: usize,
    pub values: Vec<String>,
}

// 2026-03-22: 这里定义区域探查响应，目的是把 used range、估计值与样本行打包成稳定 JSON 结构。
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SheetRangeInspection {
    pub path: String,
    pub sheet: String,
    pub used_range: String,
    pub start_row: usize,
    pub start_col: usize,
    pub end_row: usize,
    pub end_col: usize,
    pub row_count_estimate: usize,
    pub column_count_estimate: usize,
    pub sample_rows: Vec<SheetSampleRow>,
}

// 2026-03-22: 这里定义区域探查/解析错误，目的是给 dispatcher 返回稳定、可解释的中文错误信息。
#[derive(Debug, Error)]
pub enum SheetRangeError {
    #[error("无法打开工作簿: {0}")]
    OpenWorkbook(String),
    #[error("无法读取工作表: {0}")]
    ReadSheet(String),
    #[error("工作表为空，未识别到有效区域")]
    EmptySheet,
    #[error("区域格式无效: {0}")]
    InvalidRegion(String),
}

// 2026-03-22: 这里实现工作表区域探查，目的是先给出保守的 used range 与样本，避免直接自动猜表头。
pub fn inspect_sheet_range(
    path: &str,
    sheet_name: &str,
    sample_limit: usize,
) -> Result<SheetRangeInspection, SheetRangeError> {
    let mut workbook = open_workbook_auto(path)
        .map_err(|error| SheetRangeError::OpenWorkbook(error.to_string()))?;
    let range = workbook
        .worksheet_range(sheet_name)
        .map_err(|error| SheetRangeError::ReadSheet(error.to_string()))?;
    let used_region = detect_used_region(&range).ok_or(SheetRangeError::EmptySheet)?;
    let sample_rows = collect_sample_rows(&range, &used_region, sample_limit);

    Ok(SheetRangeInspection {
        path: path.to_string(),
        sheet: sheet_name.to_string(),
        used_range: used_region.to_range_string(),
        start_row: used_region.start_row,
        start_col: used_region.start_col,
        end_row: used_region.end_row,
        end_col: used_region.end_col,
        row_count_estimate: used_region.row_count(),
        column_count_estimate: used_region.column_count(),
        sample_rows,
    })
}

// 2026-03-22: 这里解析 Excel 区域字符串，目的是让 load_table_region 能显式接受 A1:C10 这类输入。
pub fn parse_excel_region(input: &str) -> Result<ExcelRegion, SheetRangeError> {
    let normalized = input.trim().to_ascii_uppercase();
    let Some((left, right)) = normalized.split_once(':') else {
        return Err(SheetRangeError::InvalidRegion(input.to_string()));
    };
    let (start_col, start_row) = parse_cell_reference(left)?;
    let (end_col, end_row) = parse_cell_reference(right)?;

    if start_row == 0 || start_col == 0 || end_row == 0 || end_col == 0 {
        return Err(SheetRangeError::InvalidRegion(input.to_string()));
    }
    if start_row > end_row || start_col > end_col {
        return Err(SheetRangeError::InvalidRegion(input.to_string()));
    }

    Ok(ExcelRegion {
        start_row,
        start_col,
        end_row,
        end_col,
    })
}

// 2026-03-22: 这里抽出单元格字符串化逻辑，目的是让 inspect/load 对空单元格判断保持一致。
pub(crate) fn normalize_cell(cell: &Data) -> String {
    match cell {
        Data::Empty => String::new(),
        _ => cell.to_string().trim().to_string(),
    }
}

// 2026-03-22: 这里统一把 Excel 1-based 区域映射到 calamine 0-based 访问索引，目的是避免不同模块各自换算。
pub(crate) fn region_cell_value(
    range: &Range<Data>,
    region: &ExcelRegion,
    row_offset: usize,
    column_offset: usize,
) -> String {
    let row_index = region.start_row + row_offset - 1;
    let column_index = region.start_col + column_offset - 1;
    range
        .get_value((row_index as u32 - 1, column_index as u32 - 1))
        .map(normalize_cell)
        .unwrap_or_default()
}

// 2026-03-22: 这里扫描非空边界，目的是只把真正有内容的矩形区域识别出来。
pub(crate) fn detect_used_region(range: &Range<Data>) -> Option<ExcelRegion> {
    let (sheet_start_row, sheet_start_col) = range.start().unwrap_or((0, 0));
    let mut min_row = usize::MAX;
    let mut min_col = usize::MAX;
    let mut max_row = 0usize;
    let mut max_col = 0usize;
    let mut found_non_empty = false;

    for (row_index, column_index, cell) in range.cells() {
        if normalize_cell(cell).is_empty() {
            continue;
        }

        found_non_empty = true;
        let row_number = sheet_start_row as usize + row_index + 1;
        let column_number = sheet_start_col as usize + column_index + 1;
        min_row = min_row.min(row_number);
        min_col = min_col.min(column_number);
        max_row = max_row.max(row_number);
        max_col = max_col.max(column_number);
    }

    found_non_empty.then_some(ExcelRegion {
        start_row: min_row,
        start_col: min_col,
        end_row: max_row,
        end_col: max_col,
    })
}

fn collect_sample_rows(
    range: &Range<Data>,
    region: &ExcelRegion,
    sample_limit: usize,
) -> Vec<SheetSampleRow> {
    let bounded_limit = sample_limit.max(1).min(region.row_count());

    (0..bounded_limit)
        .map(|row_offset| SheetSampleRow {
            row_number: region.start_row + row_offset,
            values: (0..region.column_count())
                .map(|column_offset| {
                    region_cell_value(range, region, row_offset + 1, column_offset + 1)
                })
                .collect(),
        })
        .collect()
}

fn parse_cell_reference(input: &str) -> Result<(usize, usize), SheetRangeError> {
    if input.is_empty() {
        return Err(SheetRangeError::InvalidRegion(input.to_string()));
    }

    let mut letters = String::new();
    let mut digits = String::new();
    for character in input.chars() {
        if character.is_ascii_alphabetic() && digits.is_empty() {
            letters.push(character);
        } else if character.is_ascii_digit() {
            digits.push(character);
        } else {
            return Err(SheetRangeError::InvalidRegion(input.to_string()));
        }
    }

    if letters.is_empty() || digits.is_empty() {
        return Err(SheetRangeError::InvalidRegion(input.to_string()));
    }

    let column = letters_to_column_number(&letters)?;
    let row = digits
        .parse::<usize>()
        .map_err(|_| SheetRangeError::InvalidRegion(input.to_string()))?;

    Ok((column, row))
}

fn letters_to_column_number(input: &str) -> Result<usize, SheetRangeError> {
    let mut result = 0usize;

    for character in input.chars() {
        if !character.is_ascii_uppercase() {
            return Err(SheetRangeError::InvalidRegion(input.to_string()));
        }
        result = result * 26 + (character as usize - 'A' as usize + 1);
    }

    Ok(result)
}

fn column_number_to_letters(mut column: usize) -> String {
    let mut letters = String::new();

    while column > 0 {
        let remainder = (column - 1) % 26;
        letters.insert(0, (b'A' + remainder as u8) as char);
        column = (column - 1) / 26;
    }

    letters
}
