use calamine::{Data, Reader, open_workbook_auto};
use rust_xlsxwriter::{
    Button, ConditionalFormatCell, ConditionalFormatCellRule, Format, FormatAlign, FormatBorder,
    Workbook, XlsxError,
};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use thiserror::Error;
use zip::ZipArchive;

use crate::ops::betting_optimizer::{
    BettingMetrics, BettingOptimizerEntry, BettingOptimizerRequest, BettingOptimizerSolution,
    build_optimizer_copy_texts,
};

pub const CURRENT_SHEET_NAME: &str = "计算器";
pub const SUGGESTION_SHEET_NAME: &str = "优化建议";
pub const SUGGESTION_ROUND_SHEET_PREFIX: &str = "优化建议_第";
const LEGACY_CURRENT_SHEET_NAME: &str = "当前盘面";
const ENTRY_COUNT: usize = 49;
const TARGET_LABEL_COL: u16 = 32;
const TARGET_VALUE_COL: u16 = 33;
const MAX_LOSS_TARGET_ROW: u32 = 10;
const LOSS_COUNT_TARGET_ROW: u32 = 11;
const ACTION_ROW: u32 = 12;
const STATUS_ROW: u32 = 13;
const RESULT_META_LABEL_COL: u16 = 9;
const RESULT_META_VALUE_COL: u16 = 10;
const RESULT_COPY_LABEL_COL: u16 = 13;
const RESULT_COPY_VALUE_START_COL: u16 = 13;
const RESULT_COPY_VALUE_END_COL: u16 = 16;
const RESULT_SOURCE_SHEET_ROW: u32 = 1;
const RESULT_ROUND_NAME_ROW: u32 = 2;
const RESULT_MAX_LOSS_TARGET_ROW: u32 = 3;
const RESULT_LOSS_COUNT_TARGET_ROW: u32 = 4;
const RESULT_ACTION_ROW: u32 = 5;
const RESULT_STATUS_ROW: u32 = 6;
const RESULT_COPY_LABEL_ROW: u32 = 7;
const RESULT_COPY_VALUE_START_ROW: u32 = 8;
const RESULT_COPY_VALUE_END_ROW: u32 = 9;
const RESULT_LARGE_OVERAGE_LABEL_ROW: u32 = 10;
const RESULT_LARGE_OVERAGE_VALUE_START_ROW: u32 = 11;
const RESULT_LARGE_OVERAGE_VALUE_END_ROW: u32 = 12;
const RESULT_SMALL_GROUP_LABEL_ROW: u32 = 13;
const RESULT_SMALL_GROUP_VALUE_START_ROW: u32 = 14;
const RESULT_SMALL_GROUP_VALUE_END_ROW: u32 = 15;
const RESULT_LARGE_GROUP_LABEL_ROW: u32 = 16;
const RESULT_LARGE_GROUP_VALUE_START_ROW: u32 = 17;
const RESULT_LARGE_GROUP_VALUE_END_ROW: u32 = 18;
const RESULT_DETAIL_HEADER_ROW: u32 = 7;
const RESULT_DETAIL_START_ROW: u32 = 8;
const RESULT_NEXT_BASELINE_COL: u16 = 8;
const RESULT_MANUAL_LOCK_COL: u16 = 9;
const RESULT_REFUND_CAP_COL: u16 = 10;
const RESULT_MIN_RETAINED_COL: u16 = 11;
const RESULT_CONSTRAINT_STATUS_COL: u16 = 12;
const TOTAL_ROW: u32 = 11;
const TOP_HEADER_ROW: u32 = 0;
const BOTTOM_HEADER_ROW: u32 = 5;

const TOP_ROWS: [u32; 4] = [1, 2, 3, 4];
const BOTTOM_ROWS_LONG: [u32; 5] = [6, 7, 8, 9, 10];
const BOTTOM_ROWS_SHORT: [u32; 4] = [6, 7, 8, 9];

const SNAKE_LABELS: [&str; 4] = ["02", "14", "26", "38"];
const RABBIT_LABELS: [&str; 4] = ["04", "16", "28", "40"];
const OX_LABELS: [&str; 4] = ["06", "18", "30", "42"];
const PIG_LABELS: [&str; 4] = ["08", "20", "32", "44"];
const ROOSTER_LABELS: [&str; 4] = ["10", "22", "34", "46"];
const GOAT_LABELS: [&str; 4] = ["12", "24", "36", "48"];
const HORSE_LABELS: [&str; 5] = ["01", "13", "25", "37", "49"];
const DRAGON_LABELS: [&str; 4] = ["03", "15", "27", "39"];
const TIGER_LABELS: [&str; 4] = ["05", "17", "29", "41"];
const RAT_LABELS: [&str; 4] = ["07", "19", "31", "43"];
const DOG_LABELS: [&str; 4] = ["09", "21", "33", "45"];
const MONKEY_LABELS: [&str; 4] = ["11", "23", "35", "47"];

#[derive(Clone, Copy)]
struct GroupLayout {
    parity_label: Option<&'static str>,
    parity_col: Option<u16>,
    animal_label: &'static str,
    animal_col: u16,
    header_row: u32,
    entry_rows: &'static [u32],
    labels: &'static [&'static str],
}

impl GroupLayout {
    fn stake_col(self) -> u16 {
        self.animal_col + 1
    }

    fn payout_col(self) -> u16 {
        self.animal_col + 2
    }

    fn pnl_col(self) -> u16 {
        self.animal_col + 3
    }

    fn advice_col(self) -> u16 {
        self.animal_col + 4
    }

    fn placeholder_row(self) -> Option<u32> {
        if self.header_row == BOTTOM_HEADER_ROW && self.entry_rows.len() == 4 {
            Some(10)
        } else {
            None
        }
    }
}

#[derive(Clone, Copy)]
struct EntrySlot {
    label: &'static str,
    row: u32,
    stake_col: u16,
    pnl_col: u16,
}

pub struct BettingWorkbookContract {
    pub request: BettingOptimizerRequest,
    pub source_sheet_name: String,
    pub source_round_index: Option<usize>,
}

struct SolutionSheetContext<'a> {
    current_metrics: &'a BettingMetrics,
    solution: &'a BettingOptimizerSolution,
    summary: &'a str,
    original_copy_text: &'a str,
    large_overage_copy_text: &'a str,
    small_group_copy_text: &'a str,
    large_group_copy_text: &'a str,
    source_sheet_name: &'a str,
}

#[derive(Debug, Error)]
pub enum BettingWorkbookBridgeError {
    #[error("failed to write betting workbook template: {0}")]
    WriteTemplate(String),
    #[error("failed to open betting workbook: {0}")]
    OpenWorkbook(String),
    #[error("betting workbook is missing sheet: {0}")]
    MissingSheet(String),
    #[error("betting workbook cell is invalid: {0}")]
    InvalidCell(String),
    #[error("betting workbook is missing embedded vba project")]
    MissingVbaProject,
    #[error("failed to read workbook zip entry: {0}")]
    ReadZipEntry(String),
}

struct CurrentSheetFormats {
    parity_header: Format,
    animal_header: Format,
    header: Format,
    advice_header: Format,
    input_value: Format,
    display_value: Format,
    summary_label: Format,
    summary_value: Format,
    summary_label_highlight: Format,
    required_label: Format,
    required_value: Format,
    note_label: Format,
    note_value: Format,
    slash_placeholder: Format,
}

fn group_layouts() -> [GroupLayout; 12] {
    [
        GroupLayout {
            parity_label: Some("双"),
            parity_col: Some(0),
            animal_label: "蛇",
            animal_col: 1,
            header_row: TOP_HEADER_ROW,
            entry_rows: &TOP_ROWS,
            labels: &SNAKE_LABELS,
        },
        GroupLayout {
            parity_label: None,
            parity_col: None,
            animal_label: "兔",
            animal_col: 6,
            header_row: TOP_HEADER_ROW,
            entry_rows: &TOP_ROWS,
            labels: &RABBIT_LABELS,
        },
        GroupLayout {
            parity_label: None,
            parity_col: None,
            animal_label: "牛",
            animal_col: 11,
            header_row: TOP_HEADER_ROW,
            entry_rows: &TOP_ROWS,
            labels: &OX_LABELS,
        },
        GroupLayout {
            parity_label: None,
            parity_col: None,
            animal_label: "猪",
            animal_col: 16,
            header_row: TOP_HEADER_ROW,
            entry_rows: &TOP_ROWS,
            labels: &PIG_LABELS,
        },
        GroupLayout {
            parity_label: None,
            parity_col: None,
            animal_label: "鸡",
            animal_col: 21,
            header_row: TOP_HEADER_ROW,
            entry_rows: &TOP_ROWS,
            labels: &ROOSTER_LABELS,
        },
        GroupLayout {
            parity_label: None,
            parity_col: None,
            animal_label: "羊",
            animal_col: 26,
            header_row: TOP_HEADER_ROW,
            entry_rows: &TOP_ROWS,
            labels: &GOAT_LABELS,
        },
        GroupLayout {
            parity_label: Some("单"),
            parity_col: Some(0),
            animal_label: "马",
            animal_col: 1,
            header_row: BOTTOM_HEADER_ROW,
            entry_rows: &BOTTOM_ROWS_LONG,
            labels: &HORSE_LABELS,
        },
        GroupLayout {
            parity_label: None,
            parity_col: None,
            animal_label: "龙",
            animal_col: 6,
            header_row: BOTTOM_HEADER_ROW,
            entry_rows: &BOTTOM_ROWS_SHORT,
            labels: &DRAGON_LABELS,
        },
        GroupLayout {
            parity_label: None,
            parity_col: None,
            animal_label: "虎",
            animal_col: 11,
            header_row: BOTTOM_HEADER_ROW,
            entry_rows: &BOTTOM_ROWS_SHORT,
            labels: &TIGER_LABELS,
        },
        GroupLayout {
            parity_label: None,
            parity_col: None,
            animal_label: "鼠",
            animal_col: 16,
            header_row: BOTTOM_HEADER_ROW,
            entry_rows: &BOTTOM_ROWS_SHORT,
            labels: &RAT_LABELS,
        },
        GroupLayout {
            parity_label: None,
            parity_col: None,
            animal_label: "狗",
            animal_col: 21,
            header_row: BOTTOM_HEADER_ROW,
            entry_rows: &BOTTOM_ROWS_SHORT,
            labels: &DOG_LABELS,
        },
        GroupLayout {
            parity_label: None,
            parity_col: None,
            animal_label: "猴",
            animal_col: 26,
            header_row: BOTTOM_HEADER_ROW,
            entry_rows: &BOTTOM_ROWS_SHORT,
            labels: &MONKEY_LABELS,
        },
    ]
}

fn entry_slots() -> Vec<EntrySlot> {
    let mut slots = Vec::with_capacity(ENTRY_COUNT);
    for group in group_layouts() {
        for (index, label) in group.labels.iter().enumerate() {
            slots.push(EntrySlot {
                label,
                row: group.entry_rows[index],
                stake_col: group.stake_col(),
                pnl_col: group.pnl_col(),
            });
        }
    }
    slots
}

fn build_current_sheet_formats() -> CurrentSheetFormats {
    // 2026-04-20 CST: Restore the operator-facing workbook shell to the
    // original calculator-style page so field usage stays familiar while the
    // Rust solver continues to own the risk-control calculation.
    let centered = Format::new()
        .set_align(FormatAlign::Center)
        .set_align(FormatAlign::VerticalCenter)
        .set_border(FormatBorder::Thin);
    let centered_medium = centered
        .clone()
        .set_border_top(FormatBorder::Medium)
        .set_border_left(FormatBorder::Medium);

    CurrentSheetFormats {
        parity_header: Format::new()
            .set_align(FormatAlign::Center)
            .set_align(FormatAlign::VerticalCenter)
            .set_background_color("FFF2CC")
            .set_border(FormatBorder::Medium)
            .set_bold(),
        animal_header: Format::new()
            .set_align(FormatAlign::Center)
            .set_align(FormatAlign::VerticalCenter)
            .set_background_color("1F1F1F")
            .set_font_color("FFFFFF")
            .set_border(FormatBorder::Medium)
            .set_bold(),
        header: Format::new()
            .set_align(FormatAlign::Center)
            .set_align(FormatAlign::VerticalCenter)
            .set_background_color("D9E2F3")
            .set_border(FormatBorder::Thin)
            .set_text_wrap()
            .set_bold(),
        advice_header: Format::new()
            .set_align(FormatAlign::Center)
            .set_align(FormatAlign::VerticalCenter)
            .set_background_color("92D050")
            .set_border(FormatBorder::Thin)
            .set_text_wrap()
            .set_bold(),
        input_value: Format::new()
            .set_align(FormatAlign::Center)
            .set_align(FormatAlign::VerticalCenter)
            .set_background_color("FFF2CC")
            .set_border(FormatBorder::Thin),
        display_value: centered.clone().set_num_format("0.00"),
        summary_label: Format::new()
            .set_align(FormatAlign::Center)
            .set_align(FormatAlign::VerticalCenter)
            .set_background_color("E2F0D9")
            .set_border(FormatBorder::Thin)
            .set_text_wrap()
            .set_bold(),
        summary_value: Format::new()
            .set_align(FormatAlign::Center)
            .set_align(FormatAlign::VerticalCenter)
            .set_border(FormatBorder::Thin)
            .set_num_format("0.00"),
        summary_label_highlight: Format::new()
            .set_align(FormatAlign::Center)
            .set_align(FormatAlign::VerticalCenter)
            .set_background_color("FF6666")
            .set_font_color("FFFFFF")
            .set_border(FormatBorder::Thin)
            .set_text_wrap()
            .set_bold(),
        required_label: Format::new()
            .set_align(FormatAlign::Center)
            .set_align(FormatAlign::VerticalCenter)
            .set_background_color("FCE4D6")
            .set_border(FormatBorder::Thin)
            .set_text_wrap()
            .set_bold(),
        required_value: Format::new()
            .set_align(FormatAlign::Center)
            .set_align(FormatAlign::VerticalCenter)
            .set_background_color("FFF2CC")
            .set_border(FormatBorder::Thin)
            .set_num_format("0"),
        note_label: Format::new()
            .set_align(FormatAlign::Center)
            .set_align(FormatAlign::VerticalCenter)
            .set_background_color("D9EAD3")
            .set_border(FormatBorder::Thin)
            .set_text_wrap()
            .set_bold(),
        note_value: centered_medium,
        slash_placeholder: Format::new()
            .set_align(FormatAlign::Center)
            .set_align(FormatAlign::VerticalCenter)
            .set_border(FormatBorder::Thin)
            .set_font_color("808080"),
    }
}

fn column_name(mut column: u16) -> String {
    let mut name = String::new();
    loop {
        name.insert(0, (b'A' + (column % 26) as u8) as char);
        if column < 26 {
            break;
        }
        column = (column / 26) - 1;
    }
    name
}

fn cell_ref(row: u32, column: u16) -> String {
    format!("{}{}", column_name(column), row + 1)
}

fn sum_formula(refs: &[String]) -> String {
    format!("=SUM({})", refs.join(","))
}

fn max_formula(refs: &[String]) -> String {
    format!("=MAX({})", refs.join(","))
}

fn min_formula(refs: &[String]) -> String {
    format!("=MIN({})", refs.join(","))
}

fn positive_count_formula(refs: &[String]) -> String {
    let parts = refs
        .iter()
        .map(|cell| format!("IF({cell}>0,1,0)"))
        .collect::<Vec<_>>();
    format!("=SUM({})", parts.join(","))
}

// 2026-04-20 CST: Add a dedicated xlsm template writer because the delivery
// path now depends on a stable two-sheet workbook shape with embedded VBA, and
// we need one governed workbook contract before wiring the solver binary.
pub fn write_betting_template_xlsm(
    output_path: &str,
    vba_project_path: &str,
) -> Result<(), BettingWorkbookBridgeError> {
    let request = default_template_request();
    write_betting_template_xlsm_with_request(output_path, vba_project_path, &request)
}

pub fn write_betting_template_xlsm_with_request(
    output_path: &str,
    vba_project_path: &str,
    request: &BettingOptimizerRequest,
) -> Result<(), BettingWorkbookBridgeError> {
    write_workbook(output_path, vba_project_path, request, None)
}

pub fn load_betting_workbook_contract(
    path: &str,
) -> Result<BettingWorkbookContract, BettingWorkbookBridgeError> {
    load_betting_workbook_contract_from_sheet(path, None)
}

pub fn load_betting_workbook_contract_from_sheet(
    path: &str,
    source_sheet_name: Option<&str>,
) -> Result<BettingWorkbookContract, BettingWorkbookBridgeError> {
    let mut workbook = open_workbook_auto(path)
        .map_err(|error| BettingWorkbookBridgeError::OpenWorkbook(error.to_string()))?;
    let selected_sheet_name = resolve_selected_sheet_name(&mut workbook, source_sheet_name)?;
    let range = workbook
        .worksheet_range(&selected_sheet_name)
        .map_err(|error| BettingWorkbookBridgeError::OpenWorkbook(error.to_string()))?;

    if selected_sheet_name == CURRENT_SHEET_NAME || selected_sheet_name == LEGACY_CURRENT_SHEET_NAME
    {
        build_contract_from_current_sheet(&selected_sheet_name, &range)
    } else if let Some(round_index) = parse_round_sheet_index(&selected_sheet_name) {
        build_contract_from_round_sheet(&selected_sheet_name, round_index, &range)
    } else {
        Err(BettingWorkbookBridgeError::MissingSheet(selected_sheet_name))
    }
}

fn resolve_selected_sheet_name<RS: std::io::Read + std::io::Seek>(
    workbook: &mut calamine::Sheets<RS>,
    requested_sheet_name: Option<&str>,
) -> Result<String, BettingWorkbookBridgeError> {
    // 2026-04-21 CST: Allow explicit source-sheet selection because approved
    // multi-round debugging now needs the solver to re-enter from a result
    // sheet instead of always assuming the first calculator page.
    if let Some(sheet_name) = requested_sheet_name {
        return workbook
            .worksheet_range(sheet_name)
            .map(|_| sheet_name.to_string())
            .map_err(|_| BettingWorkbookBridgeError::MissingSheet(sheet_name.to_string()));
    }

    for sheet_name in [CURRENT_SHEET_NAME, LEGACY_CURRENT_SHEET_NAME] {
        if workbook.worksheet_range(sheet_name).is_ok() {
            return Ok(sheet_name.to_string());
        }
    }

    Err(BettingWorkbookBridgeError::MissingSheet(
        CURRENT_SHEET_NAME.to_string(),
    ))
}

fn build_contract_from_current_sheet(
    selected_sheet_name: &str,
    range: &calamine::Range<Data>,
) -> Result<BettingWorkbookContract, BettingWorkbookBridgeError> {
    let stake_by_label = entry_slots()
        .into_iter()
        .map(|slot| {
            (
                slot.label.to_string(),
                integer_cell(range, slot.row, slot.stake_col).unwrap_or(0),
            )
        })
        .collect::<HashMap<_, _>>();
    let entries = (1..=ENTRY_COUNT)
        .map(|index| {
            let label = format!("{index:02}");
            BettingOptimizerEntry::new(label.clone(), *stake_by_label.get(&label).unwrap_or(&0))
        })
        .collect::<Vec<_>>();

    let max_loss_limit =
        float_cell(range, MAX_LOSS_TARGET_ROW, TARGET_VALUE_COL).ok_or_else(|| {
            BettingWorkbookBridgeError::InvalidCell("AH11 should contain max loss target".to_string())
        })?;
    let loss_count_target =
        integer_cell(range, LOSS_COUNT_TARGET_ROW, TARGET_VALUE_COL).ok_or_else(|| {
            BettingWorkbookBridgeError::InvalidCell(
                "AH12 should contain loss count target".to_string(),
            )
        })?;

    Ok(BettingWorkbookContract {
        request: BettingOptimizerRequest::new(entries, 47.0, 0.02, max_loss_limit, loss_count_target),
        source_sheet_name: selected_sheet_name.to_string(),
        source_round_index: None,
    })
}

fn build_contract_from_round_sheet(
    selected_sheet_name: &str,
    round_index: usize,
    range: &calamine::Range<Data>,
) -> Result<BettingWorkbookContract, BettingWorkbookBridgeError> {
    // 2026-04-21 CST: Parse the result sheet as the next-round input contract
    // because approved iterative tuning now happens directly on prior result
    // pages and must remain integer-only and traceable.
    let mut entries = Vec::with_capacity(ENTRY_COUNT);
    for index in 0..ENTRY_COUNT {
        let row = RESULT_DETAIL_START_ROW + index as u32;
        let label = string_cell(range, row, 0)
            .unwrap_or_else(|| format!("{:02}", index + 1))
            .trim()
            .to_string();
        let stake = integer_cell(range, row, RESULT_NEXT_BASELINE_COL)
            .or_else(|| integer_cell(range, row, 2))
            .ok_or_else(|| {
                BettingWorkbookBridgeError::InvalidCell(format!(
                    "I{} should contain next-round baseline stake",
                    row + 1
                ))
            })?;
        let manual_locked_stake = integer_cell(range, row, RESULT_MANUAL_LOCK_COL);
        let manual_refund_cap = integer_cell(range, row, RESULT_REFUND_CAP_COL);
        if manual_locked_stake.is_some() && manual_refund_cap.is_some() {
            return Err(BettingWorkbookBridgeError::InvalidCell(format!(
                "第{}行同一行不能同时填写手工锁定下轮下注额和本轮最多可退款金额",
                row + 1
            )));
        }

        entries.push(BettingOptimizerEntry::with_manual_constraints(
            label,
            stake,
            manual_locked_stake,
            manual_refund_cap,
        ));
    }

    let max_loss_limit =
        float_cell(range, RESULT_MAX_LOSS_TARGET_ROW, RESULT_META_VALUE_COL).ok_or_else(|| {
            BettingWorkbookBridgeError::InvalidCell("K4 should contain max loss target".to_string())
        })?;
    let loss_count_target = integer_cell(range, RESULT_LOSS_COUNT_TARGET_ROW, RESULT_META_VALUE_COL)
        .ok_or_else(|| {
            BettingWorkbookBridgeError::InvalidCell(
                "K5 should contain loss count target".to_string(),
            )
        })?;

    Ok(BettingWorkbookContract {
        request: BettingOptimizerRequest::new(entries, 47.0, 0.02, max_loss_limit, loss_count_target),
        source_sheet_name: selected_sheet_name.to_string(),
        source_round_index: Some(round_index),
    })
}

fn parse_round_sheet_index(sheet_name: &str) -> Option<usize> {
    sheet_name
        .strip_prefix(SUGGESTION_ROUND_SHEET_PREFIX)?
        .strip_suffix("轮")?
        .parse::<usize>()
        .ok()
}

pub fn write_betting_workbook_solution_xlsm(
    template_path: &str,
    output_path: &str,
    request: &BettingOptimizerRequest,
    current_metrics: &BettingMetrics,
    solution: &BettingOptimizerSolution,
    summary: &str,
) -> Result<(), BettingWorkbookBridgeError> {
    let default_contract = BettingWorkbookContract {
        request: BettingOptimizerRequest::new(
            request
                .entries
                .iter()
                .map(|entry| BettingOptimizerEntry::new(entry.label.clone(), entry.original_stake))
                .collect::<Vec<_>>(),
            request.payout_multiplier,
            request.rebate_rate,
            request.max_loss_limit,
            request.loss_count_target,
        ),
        source_sheet_name: CURRENT_SHEET_NAME.to_string(),
        source_round_index: None,
    };
    write_betting_workbook_solution_xlsm_from_contract(
        template_path,
        output_path,
        &default_contract,
        current_metrics,
        solution,
        summary,
    )
}

pub fn write_betting_workbook_solution_xlsm_from_contract(
    template_path: &str,
    output_path: &str,
    contract: &BettingWorkbookContract,
    current_metrics: &BettingMetrics,
    solution: &BettingOptimizerSolution,
    summary: &str,
) -> Result<(), BettingWorkbookBridgeError> {
    let temp_vba_path = extract_embedded_vba_project(template_path)?;
    let copy_texts = build_optimizer_copy_texts(solution);
    let solution_context = SolutionSheetContext {
        current_metrics,
        solution,
        summary,
        original_copy_text: &copy_texts.original,
        large_overage_copy_text: &copy_texts.large_overage,
        small_group_copy_text: &copy_texts.small_group,
        large_group_copy_text: &copy_texts.large_group,
        source_sheet_name: &contract.source_sheet_name,
    };
    let result = write_workbook(
        output_path,
        temp_vba_path.to_string_lossy().as_ref(),
        &contract.request,
        Some(&solution_context),
    );
    let _ = fs::remove_file(temp_vba_path);
    result
}

fn write_workbook(
    output_path: &str,
    vba_project_path: &str,
    request: &BettingOptimizerRequest,
    solution_payload: Option<&SolutionSheetContext<'_>>,
) -> Result<(), BettingWorkbookBridgeError> {
    let mut workbook = Workbook::new();
    workbook
        .add_vba_project(vba_project_path)
        .map_err(|error| BettingWorkbookBridgeError::WriteTemplate(error.to_string()))?;

    write_current_sheet(&mut workbook, request)?;
    write_suggestion_sheet(&mut workbook, request, solution_payload)?;

    workbook.save(output_path).map_err(map_xlsx_error)
}

fn write_current_sheet(
    workbook: &mut Workbook,
    request: &BettingOptimizerRequest,
) -> Result<(), BettingWorkbookBridgeError> {
    let sheet = workbook.add_worksheet();
    sheet.set_name(CURRENT_SHEET_NAME).map_err(map_xlsx_error)?;
    let formats = build_current_sheet_formats();
    let stake_lookup = request
        .entries
        .iter()
        .map(|entry| (entry.label.clone(), entry.original_stake))
        .collect::<HashMap<_, _>>();

    let column_widths = [
        (0, 3.33),
        (1, 4.33),
        (2, 10.50),
        (3, 8.00),
        (4, 8.00),
        (5, 7.50),
        (6, 4.33),
        (7, 10.50),
        (8, 8.00),
        (9, 8.00),
        (10, 7.50),
        (11, 4.33),
        (12, 10.50),
        (13, 8.00),
        (14, 8.00),
        (15, 7.50),
        (16, 4.33),
        (17, 10.50),
        (18, 8.00),
        (19, 8.00),
        (20, 7.50),
        (21, 4.33),
        (22, 10.50),
        (23, 8.00),
        (24, 8.00),
        (25, 7.50),
        (26, 4.33),
        (27, 10.50),
        (28, 8.00),
        (29, 8.00),
        (30, 7.50),
        (31, 13.0),
        (32, 18.67),
        (33, 13.0),
    ];
    for (column, width) in column_widths {
        sheet
            .set_column_width(column, width)
            .map_err(map_xlsx_error)?;
    }

    let row_heights = [
        (0, 31.95),
        (1, 34.95),
        (2, 34.95),
        (3, 34.95),
        (4, 34.95),
        (5, 37.05),
        (6, 34.95),
        (7, 34.95),
        (8, 34.95),
        (9, 34.95),
        (10, 30.0),
        (13, 28.05),
    ];
    for (row, height) in row_heights {
        sheet.set_row_height(row, height).map_err(map_xlsx_error)?;
    }

    let slots = entry_slots();
    let total_stake_cell = cell_ref(TOTAL_ROW, 3);
    let payable_principal_cell = cell_ref(TOTAL_ROW, 14);
    let stake_refs = slots
        .iter()
        .map(|slot| cell_ref(slot.row, slot.stake_col))
        .collect::<Vec<_>>();
    let pnl_refs = slots
        .iter()
        .map(|slot| cell_ref(slot.row, slot.pnl_col))
        .collect::<Vec<_>>();
    let risk_conditional_format_style = Format::new()
        .set_font_color("9C0006")
        .set_background_color("FFC7CE");
    let risk_conditional_format = ConditionalFormatCell::new()
        .set_rule(ConditionalFormatCellRule::GreaterThan(0))
        .set_format(&risk_conditional_format_style);

    for group in group_layouts() {
        if let (Some(label), Some(column)) = (group.parity_label, group.parity_col) {
            sheet
                .write_string_with_format(group.header_row, column, label, &formats.parity_header)
                .map_err(map_xlsx_error)?;
        }

        sheet
            .write_string_with_format(
                group.header_row,
                group.animal_col,
                group.animal_label,
                &formats.animal_header,
            )
            .map_err(map_xlsx_error)?;
        sheet
            .write_string_with_format(
                group.header_row,
                group.stake_col(),
                "特下注额（需要填写）",
                &formats.header,
            )
            .map_err(map_xlsx_error)?;
        sheet
            .write_string_with_format(
                group.header_row,
                group.payout_col(),
                "赔付额",
                &formats.header,
            )
            .map_err(map_xlsx_error)?;
        sheet
            .write_string_with_format(group.header_row, group.pnl_col(), "亏损额", &formats.header)
            .map_err(map_xlsx_error)?;
        sheet
            .write_string_with_format(
                group.header_row,
                group.advice_col(),
                "建议修改",
                &formats.advice_header,
            )
            .map_err(map_xlsx_error)?;

        for (index, label) in group.labels.iter().enumerate() {
            let row = group.entry_rows[index];
            let stake_col = group.stake_col();
            let payout_col = group.payout_col();
            let pnl_col = group.pnl_col();
            let advice_col = group.advice_col();
            let stake_value = *stake_lookup.get(*label).unwrap_or(&0) as f64;
            let stake_ref = cell_ref(row, stake_col);
            let payout_ref = cell_ref(row, payout_col);

            sheet
                .write_string_with_format(row, group.animal_col, *label, &formats.display_value)
                .map_err(map_xlsx_error)?;
            sheet
                .write_number_with_format(row, stake_col, stake_value, &formats.input_value)
                .map_err(map_xlsx_error)?;
            sheet
                .write_formula_with_format(
                    row,
                    payout_col,
                    format!("={stake_ref}*{}", request.payout_multiplier).as_str(),
                    &formats.display_value,
                )
                .map_err(map_xlsx_error)?;
            sheet
                .write_formula_with_format(
                    row,
                    pnl_col,
                    format!("={payout_ref}-{payable_principal_cell}").as_str(),
                    &formats.display_value,
                )
                .map_err(map_xlsx_error)?;
            sheet
                .write_string_with_format(row, advice_col, "", &formats.display_value)
                .map_err(map_xlsx_error)?;
        }

        if let Some(row) = group.placeholder_row() {
            for column in [
                group.animal_col,
                group.stake_col(),
                group.payout_col(),
                group.pnl_col(),
                group.advice_col(),
            ] {
                sheet
                    .write_string_with_format(row, column, "/", &formats.slash_placeholder)
                    .map_err(map_xlsx_error)?;
            }
        }

        sheet
            .add_conditional_format(
                group.entry_rows[0],
                group.pnl_col(),
                *group.entry_rows.last().unwrap(),
                group.pnl_col(),
                &risk_conditional_format,
            )
            .map_err(map_xlsx_error)?;
    }

    sheet
        .write_string_with_format(TOTAL_ROW, 1, "总下注额度", &formats.summary_label)
        .map_err(map_xlsx_error)?;
    sheet
        .write_formula_with_format(
            TOTAL_ROW,
            3,
            sum_formula(&stake_refs).as_str(),
            &formats.summary_value,
        )
        .map_err(map_xlsx_error)?;
    sheet
        .write_string_with_format(TOTAL_ROW, 8, "返水", &formats.summary_label)
        .map_err(map_xlsx_error)?;
    sheet
        .write_formula_with_format(
            TOTAL_ROW,
            9,
            format!("={total_stake_cell}*{}", request.rebate_rate).as_str(),
            &formats.summary_value,
        )
        .map_err(map_xlsx_error)?;
    sheet
        .write_string_with_format(TOTAL_ROW, 13, "可赔付本金", &formats.summary_label)
        .map_err(map_xlsx_error)?;
    sheet
        .write_formula_with_format(
            TOTAL_ROW,
            14,
            format!("={total_stake_cell}-{}", cell_ref(TOTAL_ROW, 9)).as_str(),
            &formats.summary_value,
        )
        .map_err(map_xlsx_error)?;

    sheet
        .write_string_with_format(
            0,
            TARGET_LABEL_COL,
            "当前亏损最大值",
            &formats.summary_label_highlight,
        )
        .map_err(map_xlsx_error)?;
    sheet
        .write_formula_with_format(
            1,
            TARGET_LABEL_COL,
            max_formula(&pnl_refs).as_str(),
            &formats.summary_value,
        )
        .map_err(map_xlsx_error)?;
    sheet
        .write_string_with_format(
            2,
            TARGET_LABEL_COL,
            "当前亏损号码数",
            &formats.summary_label,
        )
        .map_err(map_xlsx_error)?;
    sheet
        .write_formula_with_format(
            3,
            TARGET_LABEL_COL,
            positive_count_formula(&pnl_refs).as_str(),
            &formats.summary_value,
        )
        .map_err(map_xlsx_error)?;
    sheet
        .write_string_with_format(
            4,
            TARGET_LABEL_COL,
            "当前可赔付本金",
            &formats.summary_label,
        )
        .map_err(map_xlsx_error)?;
    sheet
        .write_formula_with_format(
            5,
            TARGET_LABEL_COL,
            format!("={payable_principal_cell}").as_str(),
            &formats.summary_value,
        )
        .map_err(map_xlsx_error)?;
    sheet
        .write_string_with_format(6, TARGET_LABEL_COL, "当前最大盈利值", &formats.note_label)
        .map_err(map_xlsx_error)?;
    sheet
        .write_formula_with_format(
            7,
            TARGET_LABEL_COL,
            min_formula(&pnl_refs).as_str(),
            &formats.summary_value,
        )
        .map_err(map_xlsx_error)?;

    sheet
        .write_string_with_format(
            MAX_LOSS_TARGET_ROW,
            TARGET_LABEL_COL,
            "最大亏损目标值（需要填写）",
            &formats.required_label,
        )
        .map_err(map_xlsx_error)?;
    sheet
        .write_number_with_format(
            MAX_LOSS_TARGET_ROW,
            TARGET_VALUE_COL,
            request.max_loss_limit,
            &formats.required_value,
        )
        .map_err(map_xlsx_error)?;
    sheet
        .write_string_with_format(
            LOSS_COUNT_TARGET_ROW,
            TARGET_LABEL_COL,
            "目标亏损号码数（需要填写）",
            &formats.required_label,
        )
        .map_err(map_xlsx_error)?;
    sheet
        .write_number_with_format(
            LOSS_COUNT_TARGET_ROW,
            TARGET_VALUE_COL,
            request.loss_count_target as f64,
            &formats.required_value,
        )
        .map_err(map_xlsx_error)?;
    sheet
        .write_string_with_format(
            ACTION_ROW,
            TARGET_LABEL_COL,
            "业务操作",
            &formats.note_label,
        )
        .map_err(map_xlsx_error)?;
    sheet
        .write_string_with_format(
            STATUS_ROW,
            TARGET_LABEL_COL,
            "求解状态",
            &formats.note_label,
        )
        .map_err(map_xlsx_error)?;
    sheet
        .write_string_with_format(STATUS_ROW, TARGET_VALUE_COL, "待测算", &formats.note_value)
        .map_err(map_xlsx_error)?;

    let button = Button::new()
        .set_caption("测算并生成建议")
        .set_macro("RunBettingSolverFromActiveSheet")
        .set_width(160)
        .set_height(32);
    sheet
        .insert_button(ACTION_ROW, TARGET_VALUE_COL, &button)
        .map_err(map_xlsx_error)?;

    Ok(())
}

// 2026-04-22 CST: Keep each operator copy variant in its own row block
// because the user now needs four independent copy boxes while preserving
// the existing column layout and manual-adjustment table structure.
fn write_result_copy_block(
    sheet: &mut rust_xlsxwriter::Worksheet,
    label_row: u32,
    value_start_row: u32,
    value_end_row: u32,
    label_text: &str,
    value_text: &str,
    section_header: &Format,
    detail_text: &Format,
) -> Result<(), BettingWorkbookBridgeError> {
    sheet
        .write_string_with_format(label_row, RESULT_COPY_LABEL_COL, label_text, section_header)
        .map_err(map_xlsx_error)?;
    sheet
        .merge_range(
            value_start_row,
            RESULT_COPY_VALUE_START_COL,
            value_end_row,
            RESULT_COPY_VALUE_END_COL,
            value_text,
            detail_text,
        )
        .map_err(map_xlsx_error)?;
    Ok(())
}

fn write_suggestion_sheet(
    workbook: &mut Workbook,
    request: &BettingOptimizerRequest,
    solution_payload: Option<&SolutionSheetContext<'_>>,
) -> Result<(), BettingWorkbookBridgeError> {
    let sheet = workbook.add_worksheet();
    let sheet_name = solution_payload
        .map(|context| next_round_sheet_name(context.source_sheet_name))
        .unwrap_or_else(|| SUGGESTION_SHEET_NAME.to_string());
    sheet.set_name(&sheet_name).map_err(map_xlsx_error)?;
    for column in 0..=RESULT_CONSTRAINT_STATUS_COL {
        sheet.set_column_width(column, 16).map_err(map_xlsx_error)?;
    }
    sheet.set_column_width(0, 18).map_err(map_xlsx_error)?;
    sheet.set_column_width(1, 14).map_err(map_xlsx_error)?;
    sheet.set_column_width(2, 14).map_err(map_xlsx_error)?;
    sheet.set_column_width(3, 12).map_err(map_xlsx_error)?;
    sheet.set_column_width(4, 14).map_err(map_xlsx_error)?;
    sheet.set_column_width(5, 14).map_err(map_xlsx_error)?;
    sheet.set_column_width(6, 12).map_err(map_xlsx_error)?;
    sheet.set_column_width(7, 14).map_err(map_xlsx_error)?;
    sheet.set_column_width(8, 18).map_err(map_xlsx_error)?;
    sheet.set_column_width(9, 14).map_err(map_xlsx_error)?;
    sheet.set_column_width(10, 18).map_err(map_xlsx_error)?;
    sheet.set_column_width(11, 18).map_err(map_xlsx_error)?;
    sheet.set_column_width(12, 18).map_err(map_xlsx_error)?;
    // 2026-04-21 CST: Reserve a wide copy-text area to the right of the
    // manual-constraint table so every generated round keeps the same
    // operator-facing refund sentence without overlapping editable columns.
    sheet.set_column_width(13, 18).map_err(map_xlsx_error)?;
    sheet.set_column_width(14, 18).map_err(map_xlsx_error)?;
    sheet.set_column_width(15, 18).map_err(map_xlsx_error)?;
    sheet.set_column_width(16, 18).map_err(map_xlsx_error)?;

    let title_format = Format::new()
        .set_align(FormatAlign::Center)
        .set_align(FormatAlign::VerticalCenter)
        .set_background_color("D9EAD3")
        .set_border(FormatBorder::Medium)
        .set_bold();
    let section_header = Format::new()
        .set_align(FormatAlign::Center)
        .set_align(FormatAlign::VerticalCenter)
        .set_background_color("D9E2F3")
        .set_border(FormatBorder::Thin)
        .set_bold();
    let metric_label = Format::new()
        .set_align(FormatAlign::Center)
        .set_align(FormatAlign::VerticalCenter)
        .set_background_color("E2F0D9")
        .set_border(FormatBorder::Thin)
        .set_bold();
    let metric_value = Format::new()
        .set_align(FormatAlign::Center)
        .set_align(FormatAlign::VerticalCenter)
        .set_border(FormatBorder::Thin)
        .set_num_format("0.00");
    let detail_header = Format::new()
        .set_align(FormatAlign::Center)
        .set_align(FormatAlign::VerticalCenter)
        .set_background_color("92D050")
        .set_border(FormatBorder::Thin)
        .set_text_wrap()
        .set_bold();
    let detail_text = Format::new()
        .set_align(FormatAlign::Center)
        .set_align(FormatAlign::VerticalCenter)
        .set_border(FormatBorder::Thin);
    let input_number = Format::new()
        .set_align(FormatAlign::Center)
        .set_align(FormatAlign::VerticalCenter)
        .set_background_color("FFF2CC")
        .set_border(FormatBorder::Thin)
        .set_num_format("0");
    let detail_number = Format::new()
        .set_align(FormatAlign::Center)
        .set_align(FormatAlign::VerticalCenter)
        .set_border(FormatBorder::Thin)
        .set_num_format("0.00");
    let detail_number_highlight = Format::new()
        .set_align(FormatAlign::Center)
        .set_align(FormatAlign::VerticalCenter)
        .set_border(FormatBorder::Thin)
        .set_num_format("0.00")
        .set_font_color("9C0006")
        .set_background_color("FFC7CE");

    sheet
        .write_string_with_format(0, 0, "优化摘要", &title_format)
        .map_err(map_xlsx_error)?;

    if let Some(context) = solution_payload {
        let current_metrics = context.current_metrics;
        let solution = context.solution;
        let summary = context.summary;
        let original_copy_text = context.original_copy_text;
        let large_overage_copy_text = context.large_overage_copy_text;
        let small_group_copy_text = context.small_group_copy_text;
        let large_group_copy_text = context.large_group_copy_text;
        sheet
            .merge_range(1, 0, 2, 7, summary, &section_header)
            .map_err(map_xlsx_error)?;
        // 2026-04-21 CST: Keep a dedicated copy block in the result-sheet meta
        // area so every generated round exposes the same operator-facing refund
        // sentence without shifting the established metric/detail table rows.
        // 2026-04-21 CST: Move the copy block to a dedicated far-right area
        // because rows 7+ in columns J:M are reused by the manual-constraint
        // table, and keeping the operator copy text there causes it to be
        // overwritten from round 2 onward.
        write_result_copy_block(
            sheet,
            RESULT_COPY_LABEL_ROW,
            RESULT_COPY_VALUE_START_ROW,
            RESULT_COPY_VALUE_END_ROW,
            "可复制描述（原始）",
            original_copy_text,
            &section_header,
            &detail_text,
        )?;
        write_result_copy_block(
            sheet,
            RESULT_LARGE_OVERAGE_LABEL_ROW,
            RESULT_LARGE_OVERAGE_VALUE_START_ROW,
            RESULT_LARGE_OVERAGE_VALUE_END_ROW,
            "可复制描述（大于30净额）",
            large_overage_copy_text,
            &section_header,
            &detail_text,
        )?;
        write_result_copy_block(
            sheet,
            RESULT_SMALL_GROUP_LABEL_ROW,
            RESULT_SMALL_GROUP_VALUE_START_ROW,
            RESULT_SMALL_GROUP_VALUE_END_ROW,
            "可复制描述（小于等于30归类）",
            small_group_copy_text,
            &section_header,
            &detail_text,
        )?;
        write_result_copy_block(
            sheet,
            RESULT_LARGE_GROUP_LABEL_ROW,
            RESULT_LARGE_GROUP_VALUE_START_ROW,
            RESULT_LARGE_GROUP_VALUE_END_ROW,
            "可复制描述（大于30归类）",
            large_group_copy_text,
            &section_header,
            &detail_text,
        )?;
        sheet
            .write_string_with_format(
                RESULT_SOURCE_SHEET_ROW,
                RESULT_META_LABEL_COL,
                "来源页",
                &metric_label,
            )
            .map_err(map_xlsx_error)?;
        sheet
            .write_string_with_format(
                RESULT_SOURCE_SHEET_ROW,
                RESULT_META_VALUE_COL,
                context.source_sheet_name,
                &detail_text,
            )
            .map_err(map_xlsx_error)?;
        sheet
            .write_string_with_format(
                RESULT_ROUND_NAME_ROW,
                RESULT_META_LABEL_COL,
                "本轮结果页",
                &metric_label,
            )
            .map_err(map_xlsx_error)?;
        sheet
            .write_string_with_format(
                RESULT_ROUND_NAME_ROW,
                RESULT_META_VALUE_COL,
                &sheet_name,
                &detail_text,
            )
            .map_err(map_xlsx_error)?;
        sheet
            .write_string_with_format(
                RESULT_MAX_LOSS_TARGET_ROW,
                RESULT_META_LABEL_COL,
                "最大计划亏损目标",
                &metric_label,
            )
            .map_err(map_xlsx_error)?;
        sheet
            .write_number_with_format(
                RESULT_MAX_LOSS_TARGET_ROW,
                RESULT_META_VALUE_COL,
                request.max_loss_limit,
                &input_number,
            )
            .map_err(map_xlsx_error)?;
        sheet
            .write_string_with_format(
                RESULT_LOSS_COUNT_TARGET_ROW,
                RESULT_META_LABEL_COL,
                "目标亏损号码数",
                &metric_label,
            )
            .map_err(map_xlsx_error)?;
        sheet
            .write_number_with_format(
                RESULT_LOSS_COUNT_TARGET_ROW,
                RESULT_META_VALUE_COL,
                request.loss_count_target as f64,
                &input_number,
            )
            .map_err(map_xlsx_error)?;
        sheet
            .write_string_with_format(
                RESULT_ACTION_ROW,
                RESULT_META_LABEL_COL,
                "业务操作",
                &metric_label,
            )
            .map_err(map_xlsx_error)?;
        sheet
            .write_string_with_format(
                RESULT_STATUS_ROW,
                RESULT_META_LABEL_COL,
                "求解状态",
                &metric_label,
            )
            .map_err(map_xlsx_error)?;
        sheet
            .write_string_with_format(
                RESULT_STATUS_ROW,
                RESULT_META_VALUE_COL,
                result_sheet_status_text(solution),
                &detail_text,
            )
            .map_err(map_xlsx_error)?;
        let button = Button::new()
            .set_caption("基于本页再次测算")
            .set_macro("RunBettingSolverFromActiveSheet")
            .set_width(160)
            .set_height(28);
        sheet
            .insert_button(RESULT_ACTION_ROW, RESULT_META_VALUE_COL, &button)
            .map_err(map_xlsx_error)?;
        sheet
            .write_string_with_format(3, 0, "指标", &section_header)
            .map_err(map_xlsx_error)?;
        sheet
            .write_string_with_format(3, 1, "当前", &section_header)
            .map_err(map_xlsx_error)?;
        sheet
            .write_string_with_format(3, 2, "建议", &section_header)
            .map_err(map_xlsx_error)?;
        sheet
            .write_string_with_format(4, 0, "总下注额", &metric_label)
            .map_err(map_xlsx_error)?;
        sheet
            .write_number_with_format(4, 1, current_metrics.total_stake as f64, &metric_value)
            .map_err(map_xlsx_error)?;
        sheet
            .write_number_with_format(4, 2, solution.total_adjusted_stake as f64, &metric_value)
            .map_err(map_xlsx_error)?;
        sheet
            .write_string_with_format(5, 0, "总退款额", &metric_label)
            .map_err(map_xlsx_error)?;
        sheet
            .write_number_with_format(5, 1, 0.0, &metric_value)
            .map_err(map_xlsx_error)?;
        sheet
            .write_number_with_format(5, 2, solution.total_refund as f64, &metric_value)
            .map_err(map_xlsx_error)?;
        sheet
            .write_string_with_format(6, 0, "最大亏损", &metric_label)
            .map_err(map_xlsx_error)?;
        sheet
            .write_number_with_format(6, 1, current_metrics.max_loss, &metric_value)
            .map_err(map_xlsx_error)?;
        sheet
            .write_number_with_format(6, 2, solution.max_loss, &metric_value)
            .map_err(map_xlsx_error)?;
        sheet
            .write_string_with_format(RESULT_DETAIL_HEADER_ROW, 0, "号码", &detail_header)
            .map_err(map_xlsx_error)?;
        sheet
            .write_string_with_format(RESULT_DETAIL_HEADER_ROW, 1, "当前基线下注额", &detail_header)
            .map_err(map_xlsx_error)?;
        sheet
            .write_string_with_format(RESULT_DETAIL_HEADER_ROW, 2, "建议下注额", &detail_header)
            .map_err(map_xlsx_error)?;
        sheet
            .write_string_with_format(RESULT_DETAIL_HEADER_ROW, 3, "退款额", &detail_header)
            .map_err(map_xlsx_error)?;
        sheet
            .write_string_with_format(RESULT_DETAIL_HEADER_ROW, 4, "当前盈亏额", &detail_header)
            .map_err(map_xlsx_error)?;
        sheet
            .write_string_with_format(RESULT_DETAIL_HEADER_ROW, 5, "调整后盈亏额", &detail_header)
            .map_err(map_xlsx_error)?;
        sheet
            .write_string_with_format(RESULT_DETAIL_HEADER_ROW, 6, "当前是否风险", &detail_header)
            .map_err(map_xlsx_error)?;
        sheet
            .write_string_with_format(RESULT_DETAIL_HEADER_ROW, 7, "是否建议下调", &detail_header)
            .map_err(map_xlsx_error)?;
        // 2026-04-21 CST: Extend the result-sheet contract with append-only
        // manual-adjustment columns now, because field operators need one
        // governed place to enter next-round constraints before we wire the
        // solver/parser logic in later TDD steps.
        sheet
            .write_string_with_format(
                RESULT_DETAIL_HEADER_ROW,
                RESULT_NEXT_BASELINE_COL,
                "下轮基线下注额（需要填写）",
                &detail_header,
            )
            .map_err(map_xlsx_error)?;
        sheet
            .write_string_with_format(
                RESULT_DETAIL_HEADER_ROW,
                RESULT_MANUAL_LOCK_COL,
                "手工锁定下轮下注额（需要填写，可留空）",
                &detail_header,
            )
            .map_err(map_xlsx_error)?;
        sheet
            .write_string_with_format(
                RESULT_DETAIL_HEADER_ROW,
                RESULT_REFUND_CAP_COL,
                "本轮最多可退款金额（需要填写，可留空）",
                &detail_header,
            )
            .map_err(map_xlsx_error)?;
        sheet
            .write_string_with_format(
                RESULT_DETAIL_HEADER_ROW,
                RESULT_MIN_RETAINED_COL,
                "对应最低保留下注额",
                &detail_header,
            )
            .map_err(map_xlsx_error)?;
        sheet
            .write_string_with_format(
                RESULT_DETAIL_HEADER_ROW,
                RESULT_CONSTRAINT_STATUS_COL,
                "人工约束状态",
                &detail_header,
            )
            .map_err(map_xlsx_error)?;

        for (index, entry) in solution.entries.iter().enumerate() {
            let row = RESULT_DETAIL_START_ROW + index as u32;
            let current_pnl = request.entries[index].original_stake as f64
                * request.payout_multiplier
                - current_metrics.payable_principal;
            let adjusted_value_format = if entry.refund_amount > 0 {
                &detail_number_highlight
            } else {
                &detail_number
            };
            let current_loss_format = if current_pnl > 0.0 {
                &detail_number_highlight
            } else {
                &detail_number
            };
            let adjusted_loss_format = if entry.pnl_value > 0.0 {
                &detail_number_highlight
            } else {
                &detail_number
            };
            sheet
                .write_string_with_format(row, 0, &entry.label, &detail_text)
                .map_err(map_xlsx_error)?;
            sheet
                .write_number_with_format(row, 1, entry.original_stake as f64, &detail_number)
                .map_err(map_xlsx_error)?;
            sheet
                .write_number_with_format(
                    row,
                    2,
                    entry.adjusted_stake as f64,
                    adjusted_value_format,
                )
                .map_err(map_xlsx_error)?;
            sheet
                .write_number_with_format(
                    row,
                    3,
                    entry.refund_amount as f64,
                    adjusted_value_format,
                )
                .map_err(map_xlsx_error)?;
            sheet
                .write_number_with_format(row, 4, current_pnl, current_loss_format)
                .map_err(map_xlsx_error)?;
            sheet
                .write_number_with_format(row, 5, entry.pnl_value, adjusted_loss_format)
                .map_err(map_xlsx_error)?;
            sheet
                .write_string_with_format(
                    row,
                    6,
                    if current_pnl > 0.0 { "是" } else { "否" },
                    &detail_text,
                )
                .map_err(map_xlsx_error)?;
            sheet
                .write_string_with_format(
                    row,
                    7,
                    if entry.refund_amount > 0 {
                        "是"
                    } else {
                        "否"
                    },
                    &detail_text,
                )
                .map_err(map_xlsx_error)?;
            sheet
                .write_number_with_format(
                    row,
                    RESULT_NEXT_BASELINE_COL,
                    entry.adjusted_stake as f64,
                    &input_number,
                )
                .map_err(map_xlsx_error)?;
            sheet
                .write_blank(row, RESULT_MANUAL_LOCK_COL, &input_number)
                .map_err(map_xlsx_error)?;
            sheet
                .write_blank(row, RESULT_REFUND_CAP_COL, &input_number)
                .map_err(map_xlsx_error)?;
            if let Some(refund_cap) = request.entries[index].manual_refund_cap {
                sheet
                    .write_number_with_format(
                        row,
                        RESULT_MIN_RETAINED_COL,
                        (request.entries[index].original_stake - refund_cap) as f64,
                        &detail_number,
                    )
                    .map_err(map_xlsx_error)?;
            } else {
                sheet
                    .write_blank(row, RESULT_MIN_RETAINED_COL, &detail_number)
                    .map_err(map_xlsx_error)?;
            }
            let constraint_status = if solution.constraint_limited
                && (request.entries[index].manual_locked_stake.is_some()
                    || request.entries[index].manual_refund_cap.is_some())
            {
                "人工约束导致目标未完全达成"
            } else if request.entries[index].manual_locked_stake.is_some() {
                "已锁定"
            } else if request.entries[index].manual_refund_cap.is_some() {
                "已设置退款上限"
            } else {
                "未设置"
            };
            sheet
                .write_string_with_format(
                    row,
                    RESULT_CONSTRAINT_STATUS_COL,
                    constraint_status,
                    &detail_text,
                )
                .map_err(map_xlsx_error)?;
        }
    } else {
        sheet
            .merge_range(
                1,
                0,
                2,
                7,
                "请在第一页录入数据后点击测算。",
                &section_header,
            )
            .map_err(map_xlsx_error)?;
    }

    Ok(())
}

fn default_template_request() -> BettingOptimizerRequest {
    let entries = (1..=ENTRY_COUNT)
        .map(|index| BettingOptimizerEntry::new(format!("{index:02}"), 0))
        .collect::<Vec<_>>();
    BettingOptimizerRequest::new(entries, 47.0, 0.02, 1500.0, 19)
}

fn integer_cell(range: &calamine::Range<Data>, row: u32, column: u16) -> Option<i64> {
    let value = range.get_value((row, column as u32))?;
    match value {
        Data::Int(number) => Some(*number),
        Data::Float(number) => {
            if (*number - number.round()).abs() < 1e-9 {
                Some(number.round() as i64)
            } else {
                None
            }
        }
        Data::String(text) => text.trim().parse::<i64>().ok(),
        _ => None,
    }
}

fn string_cell(range: &calamine::Range<Data>, row: u32, column: u16) -> Option<String> {
    let value = range.get_value((row, column as u32))?;
    match value {
        Data::String(text) => Some(text.clone()),
        Data::Int(number) => Some(number.to_string()),
        Data::Float(number) => Some(number.to_string()),
        _ => None,
    }
}

fn float_cell(range: &calamine::Range<Data>, row: u32, column: u16) -> Option<f64> {
    let value = range.get_value((row, column as u32))?;
    match value {
        Data::Int(number) => Some(*number as f64),
        Data::Float(number) => Some(*number),
        Data::String(text) => text.trim().parse::<f64>().ok(),
        _ => None,
    }
}

fn next_round_sheet_name(source_sheet_name: &str) -> String {
    let next_round = parse_round_sheet_index(source_sheet_name).unwrap_or(0) + 1;
    format!("{SUGGESTION_ROUND_SHEET_PREFIX}{next_round}轮")
}

// 2026-04-21 CST: Surface the completed-solve state in the result sheet status
// block so operators can immediately distinguish normal output from
// constraint-limited output before starting the next manual adjustment round.
fn result_sheet_status_text(solution: &BettingOptimizerSolution) -> &'static str {
    if solution.constraint_limited {
        "人工约束导致目标未完全达成，可继续在本页调整后再次测算"
    } else {
        "本轮测算完成，可在本页填写人工约束后再次测算"
    }
}

fn extract_embedded_vba_project(
    template_path: &str,
) -> Result<PathBuf, BettingWorkbookBridgeError> {
    let file = File::open(template_path)
        .map_err(|error| BettingWorkbookBridgeError::ReadZipEntry(error.to_string()))?;
    let mut archive = ZipArchive::new(file)
        .map_err(|error| BettingWorkbookBridgeError::ReadZipEntry(error.to_string()))?;
    let mut entry = archive
        .by_name("xl/vbaProject.bin")
        .map_err(|_| BettingWorkbookBridgeError::MissingVbaProject)?;

    let output_dir = Path::new(template_path)
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join(".betting_solver_tmp");
    fs::create_dir_all(&output_dir)
        .map_err(|error| BettingWorkbookBridgeError::ReadZipEntry(error.to_string()))?;
    let output_path = output_dir.join(format!(
        "vbaProject_{}.bin",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|error| BettingWorkbookBridgeError::ReadZipEntry(error.to_string()))?
            .as_nanos()
    ));

    let mut output_file = File::create(&output_path)
        .map_err(|error| BettingWorkbookBridgeError::ReadZipEntry(error.to_string()))?;
    let mut bytes = Vec::new();
    entry
        .read_to_end(&mut bytes)
        .map_err(|error| BettingWorkbookBridgeError::ReadZipEntry(error.to_string()))?;
    output_file
        .write_all(&bytes)
        .map_err(|error| BettingWorkbookBridgeError::ReadZipEntry(error.to_string()))?;

    Ok(output_path)
}

fn map_xlsx_error(error: XlsxError) -> BettingWorkbookBridgeError {
    BettingWorkbookBridgeError::WriteTemplate(error.to_string())
}
