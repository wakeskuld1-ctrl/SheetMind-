use std::collections::BTreeMap;

use polars::prelude::DataType;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::frame::loader::LoadedTable;

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct ScenarioInput {
    pub name: String,
    #[serde(default)]
    pub driver_changes: BTreeMap<String, f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct DriverElasticity {
    pub driver_column: String,
    pub elasticity: f64,
    pub paired_row_count: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ScenarioContribution {
    pub driver_column: String,
    pub delta_pct: f64,
    pub elasticity: f64,
    pub effect_pct: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ScenarioOutput {
    pub name: String,
    pub predicted_target: f64,
    pub predicted_change_pct: f64,
    pub lower_bound: f64,
    pub upper_bound: f64,
    #[serde(default)]
    pub contributions: Vec<ScenarioContribution>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ScenarioHumanSummary {
    pub overall: String,
    #[serde(default)]
    pub key_points: Vec<String>,
    pub recommended_next_step: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ScenarioSimulationResult {
    pub target_column: String,
    pub row_count: usize,
    pub baseline_target: f64,
    #[serde(default)]
    pub driver_elasticities: Vec<DriverElasticity>,
    #[serde(default)]
    pub scenarios: Vec<ScenarioOutput>,
    pub human_summary: ScenarioHumanSummary,
}

#[derive(Debug, Error)]
pub enum ScenarioSimulationError {
    #[error("scenario_simulation requires target_column")]
    MissingTargetColumnArg,
    #[error("scenario_simulation requires at least one scenario")]
    MissingScenariosArg,
    #[error("scenario_simulation requires at least one driver column")]
    MissingDriverColumns,
    #[error("scenario name cannot be empty")]
    InvalidScenarioName,
    #[error("column `{column}` does not exist")]
    MissingColumn { column: String },
    #[error("column `{column}` is not numeric")]
    NonNumericColumn { column: String },
    #[error("driver `{column}` has insufficient paired rows for elasticity")]
    NotEnoughPairedRows { column: String },
}

pub fn scenario_simulation(
    loaded: &LoadedTable,
    target_column: &str,
    driver_columns: &[String],
    scenarios: &[ScenarioInput],
) -> Result<ScenarioSimulationResult, ScenarioSimulationError> {
    if target_column.trim().is_empty() {
        return Err(ScenarioSimulationError::MissingTargetColumnArg);
    }
    if scenarios.is_empty() {
        return Err(ScenarioSimulationError::MissingScenariosArg);
    }
    if driver_columns.is_empty() {
        return Err(ScenarioSimulationError::MissingDriverColumns);
    }
    for scenario in scenarios {
        if scenario.name.trim().is_empty() {
            return Err(ScenarioSimulationError::InvalidScenarioName);
        }
    }

    let target_values = extract_numeric_column(loaded, target_column)?;
    let target_non_null = target_values
        .iter()
        .filter_map(|value| *value)
        .collect::<Vec<_>>();
    let baseline_target = mean(&target_non_null);
    let safe_baseline = baseline_target.abs().max(1e-6);

    let mut elasticity_by_driver = BTreeMap::<String, DriverElasticity>::new();
    for driver_column in driver_columns {
        let driver_values = extract_numeric_column(loaded, driver_column)?;
        let paired = target_values
            .iter()
            .zip(driver_values.iter())
            .filter_map(|(target, driver)| target.zip(*driver))
            .collect::<Vec<_>>();
        if paired.len() < 2 {
            return Err(ScenarioSimulationError::NotEnoughPairedRows {
                column: driver_column.clone(),
            });
        }

        let elasticity = calculate_elasticity(&paired).clamp(-3.0, 3.0);
        elasticity_by_driver.insert(
            driver_column.clone(),
            DriverElasticity {
                driver_column: driver_column.clone(),
                elasticity,
                paired_row_count: paired.len(),
            },
        );
    }

    let mut scenario_outputs = Vec::with_capacity(scenarios.len());
    for scenario in scenarios {
        let mut contributions = Vec::new();
        let mut predicted_change_pct = 0.0_f64;
        for (driver_column, delta_pct) in &scenario.driver_changes {
            let Some(driver_elasticity) = elasticity_by_driver.get(driver_column) else {
                continue;
            };
            let effect_pct = driver_elasticity.elasticity * *delta_pct;
            predicted_change_pct += effect_pct;
            contributions.push(ScenarioContribution {
                driver_column: driver_column.clone(),
                delta_pct: *delta_pct,
                elasticity: driver_elasticity.elasticity,
                effect_pct,
            });
        }
        contributions.sort_by(|left, right| {
            right
                .effect_pct
                .abs()
                .total_cmp(&left.effect_pct.abs())
                .then_with(|| left.driver_column.cmp(&right.driver_column))
        });

        let predicted_target = baseline_target * (1.0 + predicted_change_pct);
        let uncertainty_ratio = (0.05 + predicted_change_pct.abs() * 0.35).clamp(0.05, 0.8);
        let lower_bound = predicted_target * (1.0 - uncertainty_ratio);
        let upper_bound = predicted_target * (1.0 + uncertainty_ratio);

        scenario_outputs.push(ScenarioOutput {
            name: scenario.name.clone(),
            predicted_target,
            predicted_change_pct,
            lower_bound,
            upper_bound,
            contributions,
        });
    }

    scenario_outputs.sort_by(|left, right| {
        right
            .predicted_target
            .total_cmp(&left.predicted_target)
            .then_with(|| left.name.cmp(&right.name))
    });
    let mut driver_elasticities = elasticity_by_driver.into_values().collect::<Vec<_>>();
    driver_elasticities.sort_by(|left, right| {
        right
            .elasticity
            .abs()
            .total_cmp(&left.elasticity.abs())
            .then_with(|| left.driver_column.cmp(&right.driver_column))
    });

    let best = scenario_outputs.first();
    let worst = scenario_outputs.last();
    let human_summary = ScenarioHumanSummary {
        overall: format!(
            "Ran {} scenarios for `{}` using baseline {:.4}.",
            scenario_outputs.len(),
            target_column,
            baseline_target
        ),
        key_points: vec![
            best.map(|scenario| {
                format!(
                    "Highest projected target: `{}` -> {:.4} ({:.2}%).",
                    scenario.name,
                    scenario.predicted_target,
                    scenario.predicted_change_pct * 100.0
                )
            })
            .unwrap_or_else(|| "No scenario output available.".to_string()),
            worst
                .map(|scenario| {
                    format!(
                        "Lowest projected target: `{}` -> {:.4} ({:.2}%).",
                        scenario.name,
                        scenario.predicted_target,
                        scenario.predicted_change_pct * 100.0
                    )
                })
                .unwrap_or_else(|| "No scenario output available.".to_string()),
            format!("Baseline target reference: {:.4}.", safe_baseline),
        ],
        recommended_next_step:
            "Use contribution attribution first to validate main drivers, then finalize action with the scenario that balances upside and uncertainty."
                .to_string(),
    };

    Ok(ScenarioSimulationResult {
        target_column: target_column.to_string(),
        row_count: loaded.dataframe.height(),
        baseline_target,
        driver_elasticities,
        scenarios: scenario_outputs,
        human_summary,
    })
}

fn extract_numeric_column(
    loaded: &LoadedTable,
    column: &str,
) -> Result<Vec<Option<f64>>, ScenarioSimulationError> {
    let series = loaded
        .dataframe
        .column(column)
        .map_err(|_| ScenarioSimulationError::MissingColumn {
            column: column.to_string(),
        })?
        .as_materialized_series();
    let casted =
        series
            .cast(&DataType::Float64)
            .map_err(|_| ScenarioSimulationError::NonNumericColumn {
                column: column.to_string(),
            })?;
    let values = casted
        .f64()
        .map_err(|_| ScenarioSimulationError::NonNumericColumn {
            column: column.to_string(),
        })?
        .into_iter()
        .collect::<Vec<_>>();

    Ok(values)
}

fn calculate_elasticity(paired_values: &[(f64, f64)]) -> f64 {
    let target_mean = mean(
        &paired_values
            .iter()
            .map(|(target, _)| *target)
            .collect::<Vec<_>>(),
    );
    let driver_mean = mean(
        &paired_values
            .iter()
            .map(|(_, driver)| *driver)
            .collect::<Vec<_>>(),
    );
    if target_mean.abs() <= 1e-12 || driver_mean.abs() <= 1e-12 {
        return 0.0;
    }

    let cov = paired_values
        .iter()
        .map(|(target, driver)| (driver - driver_mean) * (target - target_mean))
        .sum::<f64>()
        / paired_values.len() as f64;
    let var_driver = paired_values
        .iter()
        .map(|(_, driver)| {
            let diff = driver - driver_mean;
            diff * diff
        })
        .sum::<f64>()
        / paired_values.len() as f64;
    if var_driver.abs() <= 1e-12 {
        return 0.0;
    }
    let beta = cov / var_driver;
    beta * (driver_mean / target_mean)
}

fn mean(values: &[f64]) -> f64 {
    if values.is_empty() {
        0.0
    } else {
        values.iter().sum::<f64>() / values.len() as f64
    }
}
