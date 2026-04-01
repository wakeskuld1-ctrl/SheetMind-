use calamine::{Data, Reader, open_workbook_auto};
use std::collections::HashSet;
use thiserror::Error;

use crate::domain::schema::{ConfidenceLevel, HeaderColumn, HeaderInference, SchemaState};

// 2026-03-21: ?????????????????????????????????????
#[derive(Debug, Error)]
pub enum HeaderInferenceError {
    // 2026-03-21: ????????????????????????????????????
    #[error("???????: {0}")]
    OpenWorkbook(String),
    // 2026-03-21: ?????????????????????????????????
    #[error("??????: {0}")]
    MissingSheet(String),
    // 2026-03-21: ?????????????????????????????
    #[error("????????????")]
    EmptySheet,
}

// 2026-03-21: ????????????? DataFrame ???????????????????????
pub fn infer_header_schema(
    path: &str,
    sheet_name: &str,
) -> Result<HeaderInference, HeaderInferenceError> {
    let mut workbook = open_workbook_auto(path)
        .map_err(|error| HeaderInferenceError::OpenWorkbook(error.to_string()))?;
    let range = workbook
        .worksheet_range(sheet_name)
        .map_err(|error| HeaderInferenceError::OpenWorkbook(error.to_string()))?;

    if range.get_size().0 == 0 || range.get_size().1 == 0 {
        return Err(HeaderInferenceError::EmptySheet);
    }

    let rows: Vec<Vec<String>> = range
        .rows()
        .map(|row| row.iter().map(normalize_cell).collect())
        .collect();

    let non_empty_rows: Vec<(usize, Vec<String>)> = rows
        .into_iter()
        .enumerate()
        .filter(|(_, row)| row.iter().any(|cell| !cell.is_empty()))
        .collect();

    if non_empty_rows.is_empty() {
        return Err(HeaderInferenceError::EmptySheet);
    }

    let first_index = non_empty_rows[0].0;
    let mut header_row_indices: Vec<usize> = Vec::new();
    let mut header_rows: Vec<Vec<String>> = Vec::new();
    let mut confidence = ConfidenceLevel::High;

    if is_single_title_row(&non_empty_rows[0].1) && non_empty_rows.len() > 1 {
        confidence = ConfidenceLevel::Medium;
        let second_index = non_empty_rows[1].0;
        if second_index > first_index + 1 {
            confidence = ConfidenceLevel::Medium;
        }
        header_row_indices.push(second_index);
        header_rows.push(non_empty_rows[1].1.clone());
    } else {
        header_row_indices.push(non_empty_rows[0].0);
        header_rows.push(non_empty_rows[0].1.clone());
        if non_empty_rows.len() > 1 && row_looks_like_header(&non_empty_rows[1].1) {
            header_row_indices.push(non_empty_rows[1].0);
            header_rows.push(non_empty_rows[1].1.clone());
        }
        if non_empty_rows.len() > 2 && row_looks_like_header(&non_empty_rows[2].1) {
            header_row_indices.push(non_empty_rows[2].0);
            header_rows.push(non_empty_rows[2].1.clone());
        }
    }

    let width = header_rows.iter().map(|row| row.len()).max().unwrap_or(0);
    let mut requires_confirmation = false;
    let raw_columns = (0..width)
        .map(|column_index| {
            let path = header_rows
                .iter()
                .filter_map(|row| row.get(column_index))
                .map(|value| value.trim())
                .filter(|value| !value.is_empty())
                .map(|value| value.to_string())
                .collect::<Vec<_>>();

            let canonical_name = canonicalize_path(&path, column_index);
            if canonical_name.requires_confirmation {
                // 2026-03-22: ???????????????????????????????
                requires_confirmation = true;
            }

            HeaderColumn {
                header_path: path,
                canonical_name: canonical_name.name,
            }
        })
        .collect::<Vec<_>>();
    let (columns, adjusted_for_uniqueness) = ensure_unique_canonical_names(raw_columns);
    if adjusted_for_uniqueness {
        // 2026-03-22: ????????????????????????? schema ???????
        requires_confirmation = true;
    }

    if requires_confirmation && confidence.is_high() {
        // 2026-03-22: ???????/???????????????????????????????????
        confidence = ConfidenceLevel::Medium;
    }

    let schema_state = if confidence.is_high() {
        SchemaState::Confirmed
    } else {
        SchemaState::Pending
    };
    let data_start_row_index = header_row_indices.last().copied().unwrap_or(0) + 1;

    Ok(HeaderInference {
        columns,
        confidence,
        schema_state,
        header_row_count: header_rows.len(),
        data_start_row_index,
    })
}

fn normalize_cell(cell: &Data) -> String {
    match cell {
        Data::Empty => String::new(),
        _ => cell.to_string().trim().to_string(),
    }
}

// 2026-03-22: 这里补充显式表头路径规范化入口，目的是让区域加载也能复用和整表推断一致的 canonical 列名规则。
pub(crate) fn canonicalize_header_paths(
    header_paths: Vec<Vec<String>>,
) -> (Vec<HeaderColumn>, bool) {
    let mut requires_confirmation = false;
    let raw_columns = header_paths
        .into_iter()
        .enumerate()
        .map(|(column_index, path)| {
            let canonical_name = canonicalize_path(&path, column_index);
            if canonical_name.requires_confirmation {
                requires_confirmation = true;
            }

            HeaderColumn {
                header_path: path,
                canonical_name: canonical_name.name,
            }
        })
        .collect::<Vec<_>>();
    let (columns, adjusted_for_uniqueness) = ensure_unique_canonical_names(raw_columns);
    if adjusted_for_uniqueness {
        requires_confirmation = true;
    }

    (columns, requires_confirmation)
}

fn is_single_title_row(row: &[String]) -> bool {
    row.iter().filter(|cell| !cell.is_empty()).count() == 1
}

fn row_looks_like_header(row: &[String]) -> bool {
    let non_empty = row
        .iter()
        .filter(|cell| !cell.is_empty())
        .collect::<Vec<_>>();
    !non_empty.is_empty() && non_empty.iter().all(|cell| !looks_like_number(cell))
}

fn looks_like_number(value: &str) -> bool {
    value.parse::<f64>().is_ok()
}

// 2026-03-22: ?????? canonical ????????????????????????????????
struct CanonicalName {
    name: String,
    requires_confirmation: bool,
}

fn canonicalize_path(path: &[String], column_index: usize) -> CanonicalName {
    if path.is_empty() {
        return CanonicalName {
            name: format!("column_{}", column_index + 1),
            requires_confirmation: true,
        };
    }

    let normalized = path
        .iter()
        .map(|segment| {
            segment
                .chars()
                .map(|character| {
                    if character.is_ascii_alphanumeric() {
                        character.to_ascii_lowercase()
                    } else {
                        '_'
                    }
                })
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("_")
        .split('_')
        .filter(|segment| !segment.is_empty())
        .collect::<Vec<_>>()
        .join("_");

    if normalized.is_empty() {
        // 2026-03-22: ????????? ASCII ???????????????? Polars ???????????
        CanonicalName {
            name: format!("column_{}", column_index + 1),
            requires_confirmation: true,
        }
    } else {
        CanonicalName {
            name: normalized,
            requires_confirmation: false,
        }
    }
}

fn ensure_unique_canonical_names(columns: Vec<HeaderColumn>) -> (Vec<HeaderColumn>, bool) {
    let mut used_names = HashSet::<String>::new();
    let mut adjusted = false;
    let mut unique_columns = Vec::with_capacity(columns.len());

    for mut column in columns {
        let base_name = column.canonical_name.clone();
        let mut candidate = base_name.clone();
        let mut suffix_index = 2usize;

        while used_names.contains(&candidate) {
            candidate = format!("{base_name}_{suffix_index}");
            suffix_index += 1;
        }

        if candidate != base_name {
            // 2026-03-22: ????? canonical ??????????????????????????????????
            adjusted = true;
            column.canonical_name = candidate.clone();
        }

        used_names.insert(candidate);
        unique_columns.push(column);
    }

    (unique_columns, adjusted)
}
