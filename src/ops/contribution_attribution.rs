use std::collections::BTreeMap;

use polars::prelude::{AnyValue, DataType};
use serde::Serialize;
use thiserror::Error;

use crate::frame::loader::LoadedTable;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ContributionItem {
    pub dimension: String,
    pub baseline_value: f64,
    pub current_value: f64,
    pub contribution: f64,
    pub contribution_share: f64,
    pub direction: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ContributionHumanSummary {
    pub overall: String,
    #[serde(default)]
    pub key_points: Vec<String>,
    pub recommended_next_step: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ContributionAttributionResult {
    pub period_column: String,
    pub dimension_column: String,
    pub value_column: String,
    pub baseline_period: String,
    pub current_period: String,
    pub row_count: usize,
    pub dimension_count: usize,
    pub total_baseline: f64,
    pub total_current: f64,
    pub total_change: f64,
    #[serde(default)]
    pub top_positive: Vec<ContributionItem>,
    #[serde(default)]
    pub top_negative: Vec<ContributionItem>,
    pub human_summary: ContributionHumanSummary,
}

#[derive(Debug, Error)]
pub enum ContributionAttributionError {
    #[error("contribution_attribution requires period_column")]
    MissingPeriodColumnArg,
    #[error("contribution_attribution requires dimension_column")]
    MissingDimensionColumnArg,
    #[error("contribution_attribution requires value_column")]
    MissingValueColumnArg,
    #[error("contribution_attribution requires baseline_period")]
    MissingBaselinePeriodArg,
    #[error("contribution_attribution requires current_period")]
    MissingCurrentPeriodArg,
    #[error("column `{column}` does not exist")]
    MissingColumn { column: String },
    #[error("column `{column}` is not numeric")]
    NonNumericColumn { column: String },
    #[error("no rows found for requested baseline/current periods")]
    NoMatchingPeriodRows,
}

pub fn contribution_attribution(
    loaded: &LoadedTable,
    period_column: &str,
    dimension_column: &str,
    value_column: &str,
    baseline_period: &str,
    current_period: &str,
    top_k: usize,
) -> Result<ContributionAttributionResult, ContributionAttributionError> {
    if period_column.trim().is_empty() {
        return Err(ContributionAttributionError::MissingPeriodColumnArg);
    }
    if dimension_column.trim().is_empty() {
        return Err(ContributionAttributionError::MissingDimensionColumnArg);
    }
    if value_column.trim().is_empty() {
        return Err(ContributionAttributionError::MissingValueColumnArg);
    }
    if baseline_period.trim().is_empty() {
        return Err(ContributionAttributionError::MissingBaselinePeriodArg);
    }
    if current_period.trim().is_empty() {
        return Err(ContributionAttributionError::MissingCurrentPeriodArg);
    }

    let period_series = loaded
        .dataframe
        .column(period_column)
        .map_err(|_| ContributionAttributionError::MissingColumn {
            column: period_column.to_string(),
        })?
        .as_materialized_series()
        .clone();
    let dimension_series = loaded
        .dataframe
        .column(dimension_column)
        .map_err(|_| ContributionAttributionError::MissingColumn {
            column: dimension_column.to_string(),
        })?
        .as_materialized_series()
        .clone();
    let value_series = loaded
        .dataframe
        .column(value_column)
        .map_err(|_| ContributionAttributionError::MissingColumn {
            column: value_column.to_string(),
        })?
        .as_materialized_series()
        .cast(&DataType::Float64)
        .map_err(|_| ContributionAttributionError::NonNumericColumn {
            column: value_column.to_string(),
        })?;
    let values =
        value_series
            .f64()
            .map_err(|_| ContributionAttributionError::NonNumericColumn {
                column: value_column.to_string(),
            })?;

    let mut aggregates = BTreeMap::<String, (f64, f64)>::new();
    for ((period_any, dimension_any), value_opt) in period_series
        .iter()
        .zip(dimension_series.iter())
        .zip(values.into_iter())
    {
        let Some(value) = value_opt else {
            continue;
        };
        let Some(period) = any_value_to_label(&period_any) else {
            continue;
        };
        let Some(dimension) = any_value_to_label(&dimension_any) else {
            continue;
        };
        if period != baseline_period && period != current_period {
            continue;
        }

        let entry = aggregates.entry(dimension).or_insert((0.0, 0.0));
        if period == baseline_period {
            entry.0 += value;
        } else if period == current_period {
            entry.1 += value;
        }
    }

    if aggregates.is_empty() {
        return Err(ContributionAttributionError::NoMatchingPeriodRows);
    }

    let total_baseline = aggregates
        .values()
        .map(|(baseline, _)| *baseline)
        .sum::<f64>();
    let total_current = aggregates
        .values()
        .map(|(_, current)| *current)
        .sum::<f64>();
    let total_change = total_current - total_baseline;

    let mut items = aggregates
        .into_iter()
        .map(|(dimension, (baseline_value, current_value))| {
            let contribution = current_value - baseline_value;
            let contribution_share = if total_change.abs() <= 1e-12 {
                0.0
            } else {
                contribution / total_change
            };
            let direction = if contribution > 1e-12 {
                "positive"
            } else if contribution < -1e-12 {
                "negative"
            } else {
                "flat"
            };

            ContributionItem {
                dimension,
                baseline_value,
                current_value,
                contribution,
                contribution_share,
                direction: direction.to_string(),
            }
        })
        .collect::<Vec<_>>();

    items.sort_by(|left, right| {
        right
            .contribution
            .total_cmp(&left.contribution)
            .then_with(|| left.dimension.cmp(&right.dimension))
    });
    let top_positive = items
        .iter()
        .filter(|item| item.contribution > 0.0)
        .take(top_k.max(1))
        .cloned()
        .collect::<Vec<_>>();
    let mut top_negative = items
        .iter()
        .filter(|item| item.contribution < 0.0)
        .take(top_k.max(1))
        .cloned()
        .collect::<Vec<_>>();
    top_negative.sort_by(|left, right| {
        left.contribution
            .total_cmp(&right.contribution)
            .then_with(|| left.dimension.cmp(&right.dimension))
    });

    let human_summary = build_human_summary(
        period_column,
        dimension_column,
        value_column,
        baseline_period,
        current_period,
        total_change,
        &top_positive,
        &top_negative,
    );

    Ok(ContributionAttributionResult {
        period_column: period_column.to_string(),
        dimension_column: dimension_column.to_string(),
        value_column: value_column.to_string(),
        baseline_period: baseline_period.to_string(),
        current_period: current_period.to_string(),
        row_count: loaded.dataframe.height(),
        dimension_count: items.len(),
        total_baseline,
        total_current,
        total_change,
        top_positive,
        top_negative,
        human_summary,
    })
}

fn any_value_to_label(value: &AnyValue<'_>) -> Option<String> {
    match value {
        AnyValue::Null => None,
        AnyValue::String(text) => Some((*text).to_string()),
        AnyValue::StringOwned(text) => Some(text.as_str().to_string()),
        _ => {
            let label = value.to_string();
            if label.trim().is_empty() {
                None
            } else {
                Some(label)
            }
        }
    }
}

fn build_human_summary(
    period_column: &str,
    dimension_column: &str,
    value_column: &str,
    baseline_period: &str,
    current_period: &str,
    total_change: f64,
    top_positive: &[ContributionItem],
    top_negative: &[ContributionItem],
) -> ContributionHumanSummary {
    let overall = format!(
        "Contribution attribution on `{value_column}` from `{baseline_period}` to `{current_period}` is ready on `{dimension_column}` (period field `{period_column}`)."
    );
    let mut key_points = vec![format!("Total delta is {:.4}.", total_change)];
    if let Some(item) = top_positive.first() {
        key_points.push(format!(
            "Top positive driver: `{}` (+{:.4}, share {:.2}%).",
            item.dimension,
            item.contribution,
            item.contribution_share * 100.0
        ));
    }
    if let Some(item) = top_negative.first() {
        key_points.push(format!(
            "Top negative driver: `{}` ({:.4}, share {:.2}%).",
            item.dimension,
            item.contribution,
            item.contribution_share * 100.0
        ));
    }

    ContributionHumanSummary {
        overall,
        key_points,
        recommended_next_step:
            "Use top positive/negative drivers to prioritize action plans, then validate with scenario simulation before execution."
                .to_string(),
    }
}
