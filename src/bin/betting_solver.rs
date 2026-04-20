use excel_skill::ops::betting_optimizer::{
    build_optimizer_summary, evaluate_current_metrics, solve_betting_adjustment,
};
use excel_skill::ops::betting_workbook_bridge::{
    CURRENT_SHEET_NAME, load_betting_workbook_contract, load_betting_workbook_contract_from_sheet,
    write_betting_template_xlsm, write_betting_workbook_solution_xlsm_from_contract,
};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    if args.is_empty() {
        return Err(usage());
    }

    match args[0].as_str() {
        "template" => run_template_command(&args),
        "solve" => run_solve_command(&args),
        _ => Err(usage()),
    }
}

fn run_template_command(args: &[String]) -> Result<(), String> {
    if args.len() != 2 {
        return Err(usage());
    }

    let logger = RunLogger::create_for_command("template", Path::new(&args[1]))
        .map_err(|error| format!("failed to create betting solver log: {error}"))?;
    let vba_project_path = default_vba_project_path();
    logger.log("solver_start", format!("command=template output={}", args[1]));
    write_betting_template_xlsm(&args[1], vba_project_path.to_string_lossy().as_ref()).map_err(
        |error| {
            logger.log("template_failed", error.to_string());
            error.to_string()
        },
    )?;
    logger.log("template_success", format!("output={}", args[1]));
    println!("模板已生成: {}", args[1]);
    Ok(())
}

fn run_solve_command(args: &[String]) -> Result<(), String> {
    let (input_path, output_path, source_sheet_name) = parse_solve_args(args)?;
    let output_path = output_path.unwrap_or_else(|| input_path.clone());
    let source_sheet_name = source_sheet_name.unwrap_or_else(|| CURRENT_SHEET_NAME.to_string());

    let logger = RunLogger::create_for_command("solve", Path::new(&output_path))
        .map_err(|error| format!("failed to create betting solver log: {error}"))?;
    logger.log(
        "solver_start",
        format!(
            "command=solve input={input_path} output={output_path} source_sheet={source_sheet_name}"
        ),
    );

    let workbook = if source_sheet_name == CURRENT_SHEET_NAME {
        load_betting_workbook_contract(&input_path)
    } else {
        load_betting_workbook_contract_from_sheet(&input_path, Some(&source_sheet_name))
    }
    .map_err(|error| {
        logger.log("contract_load_failed", error.to_string());
        error.to_string()
    })?;
    let manual_locked_count = workbook
        .request
        .entries
        .iter()
        .filter(|entry| entry.manual_locked_stake.is_some())
        .count();
    let refund_cap_count = workbook
        .request
        .entries
        .iter()
        .filter(|entry| entry.manual_refund_cap.is_some())
        .count();

    logger.log(
        "contract_loaded",
        format!(
            "entries={} max_loss_limit={} loss_count_target={} source_sheet={} manual_locked_count={} refund_cap_count={}",
            workbook.request.entries.len(),
            workbook.request.max_loss_limit,
            workbook.request.loss_count_target,
            workbook.source_sheet_name,
            manual_locked_count,
            refund_cap_count
        ),
    );

    let current_metrics = evaluate_current_metrics(&workbook.request).map_err(|error| {
        logger.log("current_metrics_failed", error.to_string());
        error.to_string()
    })?;
    logger.log(
        "current_metrics_ready",
        format!(
            "total_stake={} max_loss={} loss_count={}",
            current_metrics.total_stake, current_metrics.max_loss, current_metrics.loss_count
        ),
    );

    let solution = solve_betting_adjustment(&workbook.request).map_err(|error| {
        logger.log("solve_failed", error.to_string());
        error.to_string()
    })?;
    logger.log(
        "solution_ready",
        format!(
            "total_refund={} max_loss={} loss_count={} constraint_limited={}",
            solution.total_refund,
            solution.max_loss,
            solution.loss_count,
            solution.constraint_limited
        ),
    );

    let summary = build_optimizer_summary(&workbook.request, &current_metrics, &solution);
    logger.log("summary_ready", summary.clone());

    write_betting_workbook_solution_xlsm_from_contract(
        &input_path,
        &output_path,
        &workbook,
        &current_metrics,
        &solution,
        &summary,
    )
    .map_err(|error| {
        logger.log("writeback_failed", error.to_string());
        error.to_string()
    })?;

    logger.log("solve_success", format!("output={output_path}"));
    println!("优化建议已生成: {output_path}");
    Ok(())
}

fn parse_solve_args(args: &[String]) -> Result<(String, Option<String>, Option<String>), String> {
    if args.len() < 2 {
        return Err(usage());
    }

    let input_path = args[1].clone();
    let mut output_path = None;
    let mut source_sheet_name = None;
    let mut index = 2;

    while index < args.len() {
        if args[index] == "--source-sheet" {
            let Some(sheet_name) = args.get(index + 1) else {
                return Err(usage());
            };
            source_sheet_name = Some(sheet_name.clone());
            index += 2;
            continue;
        }

        if output_path.is_none() {
            output_path = Some(args[index].clone());
            index += 1;
            continue;
        }

        return Err(usage());
    }

    Ok((input_path, output_path, source_sheet_name))
}

fn default_vba_project_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("assets")
        .join("excel_templates")
        .join("betting_optimizer")
        .join("vbaProject.bin")
}

fn usage() -> String {
    "用法:\n  betting_solver template <output.xlsm>\n  betting_solver solve <input.xlsm> [output.xlsm] [--source-sheet <sheet_name>]".to_string()
}

struct RunLogger {
    path: PathBuf,
}

impl RunLogger {
    // 2026-04-20 CST: Add a persistent run log because the workbook shell needs
    // a transparent trail for field debugging, and the user explicitly asked to
    // stop treating the solver chain as a black box.
    fn create_for_command(command: &str, target_path: &Path) -> Result<Self, std::io::Error> {
        let log_dir = if let Ok(raw_dir) = std::env::var("BETTING_SOLVER_LOG_DIR") {
            PathBuf::from(raw_dir)
        } else {
            target_path
                .parent()
                .unwrap_or_else(|| Path::new("."))
                .join("logs")
        };
        fs::create_dir_all(&log_dir)?;

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis();
        let path = log_dir.join(format!("betting_solver_{command}_{timestamp}.log"));

        let logger = Self { path };
        logger.log("log_created", format!("path={}", logger.path.display()));
        Ok(logger)
    }

    fn log(&self, event: &str, message: impl AsRef<str>) {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis();
        if let Ok(mut file) = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
        {
            let _ = writeln!(file, "[{timestamp}] {event} {}", message.as_ref());
        }
    }
}
