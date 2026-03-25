use serde_json::{Value, json};

use crate::ops::analyze::analyze_table;
use crate::ops::cluster_kmeans::cluster_kmeans;
use crate::ops::decision_assistant::decision_assistant;
use crate::ops::linear_regression::linear_regression;
use crate::ops::logistic_regression::logistic_regression;
use crate::ops::stat_summary::stat_summary;
use crate::ops::summary::summarize_table;
use crate::runtime::local_memory::SessionStage;
use crate::tools::contracts::ToolResponse;
use crate::tools::session;
use crate::tools::sources;

use super::shared::{apply_optional_casts, parse_casts, parse_missing_strategy, string_array};

pub(super) fn dispatch_summarize_table(args: Value) -> ToolResponse {
    let requested_columns = string_array(&args, "columns");
    let top_k = args
        .get("top_k")
        .and_then(|value| value.as_u64())
        .unwrap_or(5) as usize;
    let casts = match parse_casts(&args, "casts", "summarize_table") {
        Ok(casts) => casts,
        Err(response) => return response,
    };

    match sources::load_table_for_analysis(&args, "summarize_table") {
        Ok(sources::OperationLoad::NeedsConfirmation(response)) => response,
        Ok(sources::OperationLoad::Loaded(loaded)) => match apply_optional_casts(loaded, &casts) {
            Ok(prepared_loaded) => {
                match summarize_table(&prepared_loaded, &requested_columns, top_k) {
                    Ok(summaries) => {
                        if let Err(response) = session::sync_loaded_table_state(
                            &args,
                            &prepared_loaded,
                            SessionStage::AnalysisModeling,
                            "analysis completed",
                            "summarize_table",
                            "analysis_completed",
                        ) {
                            return response;
                        }
                        ToolResponse::ok(json!({
                            "row_count": prepared_loaded.dataframe.height(),
                            "summaries": summaries,
                        }))
                    }
                    Err(error) => ToolResponse::error(error.to_string()),
                }
            }
            Err(error) => ToolResponse::error(error),
        },
        Err(response) => response,
    }
}

pub(super) fn dispatch_analyze_table(args: Value) -> ToolResponse {
    let requested_columns = string_array(&args, "columns");
    let top_k = args
        .get("top_k")
        .and_then(|value| value.as_u64())
        .unwrap_or(5) as usize;
    let casts = match parse_casts(&args, "casts", "analyze_table") {
        Ok(casts) => casts,
        Err(response) => return response,
    };

    match sources::load_table_for_analysis(&args, "analyze_table") {
        Ok(sources::OperationLoad::NeedsConfirmation(response)) => response,
        Ok(sources::OperationLoad::Loaded(loaded)) => match apply_optional_casts(loaded, &casts) {
            Ok(prepared_loaded) => {
                let result = analyze_table(&prepared_loaded, &requested_columns, top_k);
                if let Err(response) = session::sync_loaded_table_state(
                    &args,
                    &prepared_loaded,
                    SessionStage::AnalysisModeling,
                    "analysis completed",
                    "analyze_table",
                    "analysis_completed",
                ) {
                    return response;
                }
                ToolResponse::ok(json!(result))
            }
            Err(error) => ToolResponse::error(error),
        },
        Err(response) => response,
    }
}

pub(super) fn dispatch_stat_summary(args: Value) -> ToolResponse {
    let requested_columns = string_array(&args, "columns");
    let top_k = args
        .get("top_k")
        .and_then(|value| value.as_u64())
        .unwrap_or(5) as usize;
    let casts = match parse_casts(&args, "casts", "stat_summary") {
        Ok(casts) => casts,
        Err(response) => return response,
    };

    match sources::load_table_for_analysis(&args, "stat_summary") {
        Ok(sources::OperationLoad::NeedsConfirmation(response)) => response,
        Ok(sources::OperationLoad::Loaded(loaded)) => match apply_optional_casts(loaded, &casts) {
            Ok(prepared_loaded) => {
                match stat_summary(&prepared_loaded, &requested_columns, top_k) {
                    Ok(result) => {
                        if let Err(response) = session::sync_loaded_table_state(
                            &args,
                            &prepared_loaded,
                            SessionStage::AnalysisModeling,
                            "analysis completed",
                            "stat_summary",
                            "analysis_completed",
                        ) {
                            return response;
                        }
                        ToolResponse::ok(json!(result))
                    }
                    Err(error) => ToolResponse::error(error.to_string()),
                }
            }
            Err(error) => ToolResponse::error(error),
        },
        Err(response) => response,
    }
}

pub(super) fn dispatch_linear_regression(args: Value) -> ToolResponse {
    let Some(features_value) = args.get("features").and_then(|value| value.as_array()) else {
        return ToolResponse::error("invalid request parameters");
    };
    let Some(target) = args.get("target").and_then(|value| value.as_str()) else {
        return ToolResponse::error("invalid request parameters");
    };
    let features = features_value
        .iter()
        .filter_map(|value| value.as_str())
        .collect::<Vec<_>>();
    let intercept = args
        .get("intercept")
        .and_then(|value| value.as_bool())
        .unwrap_or(true);
    let missing_strategy = match parse_missing_strategy(&args, "linear_regression") {
        Ok(strategy) => strategy,
        Err(response) => return response,
    };
    let casts = match parse_casts(&args, "casts", "linear_regression") {
        Ok(casts) => casts,
        Err(response) => return response,
    };

    match sources::load_table_for_analysis(&args, "linear_regression") {
        Ok(sources::OperationLoad::NeedsConfirmation(response)) => response,
        Ok(sources::OperationLoad::Loaded(loaded)) => match apply_optional_casts(loaded, &casts) {
            Ok(prepared_loaded) => match linear_regression(
                &prepared_loaded,
                &features,
                target,
                intercept,
                missing_strategy,
            ) {
                Ok(result) => {
                    if let Err(response) = session::sync_loaded_table_state(
                        &args,
                        &prepared_loaded,
                        SessionStage::AnalysisModeling,
                        "modeling completed",
                        "linear_regression",
                        "modeling_completed",
                    ) {
                        return response;
                    }
                    ToolResponse::ok(json!(result))
                }
                Err(error) => ToolResponse::error(error.to_string()),
            },
            Err(error) => ToolResponse::error(error),
        },
        Err(response) => response,
    }
}

pub(super) fn dispatch_logistic_regression(args: Value) -> ToolResponse {
    let Some(features_value) = args.get("features").and_then(|value| value.as_array()) else {
        return ToolResponse::error("invalid request parameters");
    };
    let Some(target) = args.get("target").and_then(|value| value.as_str()) else {
        return ToolResponse::error("invalid request parameters");
    };
    let features = features_value
        .iter()
        .filter_map(|value| value.as_str())
        .collect::<Vec<_>>();
    let intercept = args
        .get("intercept")
        .and_then(|value| value.as_bool())
        .unwrap_or(true);
    let positive_label = args.get("positive_label").and_then(|value| value.as_str());
    let missing_strategy = match parse_missing_strategy(&args, "logistic_regression") {
        Ok(strategy) => strategy,
        Err(response) => return response,
    };
    let casts = match parse_casts(&args, "casts", "logistic_regression") {
        Ok(casts) => casts,
        Err(response) => return response,
    };

    match sources::load_table_for_analysis(&args, "logistic_regression") {
        Ok(sources::OperationLoad::NeedsConfirmation(response)) => response,
        Ok(sources::OperationLoad::Loaded(loaded)) => match apply_optional_casts(loaded, &casts) {
            Ok(prepared_loaded) => match logistic_regression(
                &prepared_loaded,
                &features,
                target,
                intercept,
                missing_strategy,
                positive_label,
            ) {
                Ok(result) => {
                    if let Err(response) = session::sync_loaded_table_state(
                        &args,
                        &prepared_loaded,
                        SessionStage::AnalysisModeling,
                        "modeling completed",
                        "logistic_regression",
                        "modeling_completed",
                    ) {
                        return response;
                    }
                    ToolResponse::ok(json!(result))
                }
                Err(error) => ToolResponse::error(error.to_string()),
            },
            Err(error) => ToolResponse::error(error),
        },
        Err(response) => response,
    }
}

pub(super) fn dispatch_cluster_kmeans(args: Value) -> ToolResponse {
    let Some(features_value) = args.get("features").and_then(|value| value.as_array()) else {
        return ToolResponse::error("invalid request parameters");
    };
    let Some(cluster_count) = args.get("cluster_count").and_then(|value| value.as_u64()) else {
        return ToolResponse::error("invalid request parameters");
    };
    let features = features_value
        .iter()
        .filter_map(|value| value.as_str())
        .collect::<Vec<_>>();
    let max_iterations = args
        .get("max_iterations")
        .and_then(|value| value.as_u64())
        .unwrap_or(100) as usize;
    let missing_strategy = match parse_missing_strategy(&args, "cluster_kmeans") {
        Ok(strategy) => strategy,
        Err(response) => return response,
    };
    let casts = match parse_casts(&args, "casts", "cluster_kmeans") {
        Ok(casts) => casts,
        Err(response) => return response,
    };

    match sources::load_table_for_analysis(&args, "cluster_kmeans") {
        Ok(sources::OperationLoad::NeedsConfirmation(response)) => response,
        Ok(sources::OperationLoad::Loaded(loaded)) => match apply_optional_casts(loaded, &casts) {
            Ok(prepared_loaded) => match cluster_kmeans(
                &prepared_loaded,
                &features,
                cluster_count as usize,
                max_iterations,
                missing_strategy,
            ) {
                Ok(result) => {
                    if let Err(response) = session::sync_loaded_table_state(
                        &args,
                        &prepared_loaded,
                        SessionStage::AnalysisModeling,
                        "modeling completed",
                        "cluster_kmeans",
                        "modeling_completed",
                    ) {
                        return response;
                    }
                    ToolResponse::ok(json!(result))
                }
                Err(error) => ToolResponse::error(error.to_string()),
            },
            Err(error) => ToolResponse::error(error),
        },
        Err(response) => response,
    }
}

pub(super) fn dispatch_decision_assistant(args: Value) -> ToolResponse {
    let requested_columns = string_array(&args, "columns");
    let top_k = args
        .get("top_k")
        .and_then(|value| value.as_u64())
        .unwrap_or(5) as usize;
    let casts = match parse_casts(&args, "casts", "decision_assistant") {
        Ok(casts) => casts,
        Err(response) => return response,
    };

    match sources::load_table_for_analysis(&args, "decision_assistant") {
        Ok(sources::OperationLoad::NeedsConfirmation(response)) => response,
        Ok(sources::OperationLoad::Loaded(loaded)) => match apply_optional_casts(loaded, &casts) {
            Ok(prepared_loaded) => {
                match decision_assistant(&prepared_loaded, &requested_columns, top_k) {
                    Ok(result) => {
                        if let Err(response) = session::sync_loaded_table_state(
                            &args,
                            &prepared_loaded,
                            SessionStage::DecisionAssistant,
                            "decision assistant completed",
                            "decision_assistant",
                            "decision_assistant_completed",
                        ) {
                            return response;
                        }
                        ToolResponse::ok(json!(result))
                    }
                    Err(error) => ToolResponse::error(error.to_string()),
                }
            }
            Err(error) => ToolResponse::error(error),
        },
        Err(response) => response,
    }
}
