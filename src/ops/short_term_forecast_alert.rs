use polars::prelude::{AnyValue, DataType};
use serde::Serialize;
use thiserror::Error;

use crate::frame::loader::LoadedTable;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ForecastStep {
    pub step_index: usize,
    pub step_label: String,
    pub forecast: f64,
    pub lower_bound: f64,
    pub upper_bound: f64,
    pub watch_level: String,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct HistoricalAlert {
    pub time: String,
    pub actual: f64,
    pub expected: f64,
    pub z_score: f64,
    pub alert_level: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ForecastHumanSummary {
    pub overall: String,
    #[serde(default)]
    pub key_points: Vec<String>,
    pub recommended_next_step: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ShortTermForecastAlertResult {
    pub time_column: String,
    pub value_column: String,
    pub row_count: usize,
    pub point_count: usize,
    pub horizon: usize,
    pub last_observed_value: f64,
    pub mean_delta: f64,
    pub residual_sigma: f64,
    #[serde(default)]
    pub forecasts: Vec<ForecastStep>,
    #[serde(default)]
    pub historical_alerts: Vec<HistoricalAlert>,
    pub human_summary: ForecastHumanSummary,
}

#[derive(Debug, Error)]
pub enum ShortTermForecastAlertError {
    #[error("short_term_forecast_alert requires time_column")]
    MissingTimeColumnArg,
    #[error("short_term_forecast_alert requires value_column")]
    MissingValueColumnArg,
    #[error("short_term_forecast_alert requires horizon >= 1")]
    InvalidHorizon,
    #[error("column `{column}` does not exist")]
    MissingColumn { column: String },
    #[error("column `{column}` is not numeric")]
    NonNumericColumn { column: String },
    #[error("not enough valid points for forecasting on `{time_column}` + `{value_column}`")]
    NotEnoughPoints {
        time_column: String,
        value_column: String,
    },
}

pub fn short_term_forecast_alert(
    loaded: &LoadedTable,
    time_column: &str,
    value_column: &str,
    horizon: usize,
    sensitivity: f64,
) -> Result<ShortTermForecastAlertResult, ShortTermForecastAlertError> {
    if time_column.trim().is_empty() {
        return Err(ShortTermForecastAlertError::MissingTimeColumnArg);
    }
    if value_column.trim().is_empty() {
        return Err(ShortTermForecastAlertError::MissingValueColumnArg);
    }
    if horizon == 0 {
        return Err(ShortTermForecastAlertError::InvalidHorizon);
    }

    let points = extract_points(loaded, time_column, value_column)?;
    if points.len() < 3 {
        return Err(ShortTermForecastAlertError::NotEnoughPoints {
            time_column: time_column.to_string(),
            value_column: value_column.to_string(),
        });
    }

    let values = points.iter().map(|(_, value)| *value).collect::<Vec<_>>();
    let last_observed_value = *values.last().unwrap_or(&0.0);
    let deltas = values
        .windows(2)
        .map(|window| window[1] - window[0])
        .collect::<Vec<_>>();
    let mean_delta = mean(&deltas);
    let residual_sigma = stddev(&deltas).max(1e-6);
    let moving_average = mean(&values[values.len().saturating_sub(3)..]);
    let exp_smooth = exponential_smoothing(&values, 0.5);

    let bounded_sensitivity = sensitivity.clamp(1.0, 5.0);
    let mut forecasts = Vec::with_capacity(horizon);
    for step in 1..=horizon {
        let step_f64 = step as f64;
        let trend_forecast = last_observed_value + mean_delta * step_f64;
        let ma_forecast = moving_average + mean_delta * step_f64 * 0.5;
        let exp_forecast = exp_smooth + mean_delta * step_f64;
        let baseline = (trend_forecast + ma_forecast + exp_forecast) / 3.0;
        let uncertainty = residual_sigma * step_f64.sqrt() * (bounded_sensitivity / 2.0);
        let lower_bound = baseline - uncertainty;
        let upper_bound = baseline + uncertainty;
        let relative_uncertainty = uncertainty / baseline.abs().max(1.0);
        let watch_level = if relative_uncertainty > 0.35 {
            "high"
        } else if relative_uncertainty > 0.2 {
            "medium"
        } else {
            "low"
        };
        let reason = match watch_level {
            "high" => "wide interval; monitor this horizon carefully",
            "medium" => "moderate interval width; keep watch",
            _ => "baseline interval is stable",
        };

        forecasts.push(ForecastStep {
            step_index: step,
            step_label: format!("t+{step}"),
            forecast: baseline,
            lower_bound,
            upper_bound,
            watch_level: watch_level.to_string(),
            reason: reason.to_string(),
        });
    }

    let mut historical_alerts = Vec::new();
    for idx in 1..values.len() {
        let expected = values[idx - 1] + mean_delta;
        let z_score = ((values[idx] - expected) / residual_sigma).abs();
        let alert_level = if z_score >= bounded_sensitivity {
            "high"
        } else if z_score >= bounded_sensitivity * 0.7 {
            "medium"
        } else {
            "low"
        };
        if alert_level != "low" {
            historical_alerts.push(HistoricalAlert {
                time: points[idx].0.clone(),
                actual: values[idx],
                expected,
                z_score,
                alert_level: alert_level.to_string(),
            });
        }
    }
    if historical_alerts.len() > 5 {
        let keep_from = historical_alerts.len() - 5;
        historical_alerts = historical_alerts.split_off(keep_from);
    }

    let human_summary = build_human_summary(
        time_column,
        value_column,
        horizon,
        last_observed_value,
        mean_delta,
        &forecasts,
        &historical_alerts,
    );

    Ok(ShortTermForecastAlertResult {
        time_column: time_column.to_string(),
        value_column: value_column.to_string(),
        row_count: loaded.dataframe.height(),
        point_count: points.len(),
        horizon,
        last_observed_value,
        mean_delta,
        residual_sigma,
        forecasts,
        historical_alerts,
        human_summary,
    })
}

fn extract_points(
    loaded: &LoadedTable,
    time_column: &str,
    value_column: &str,
) -> Result<Vec<(String, f64)>, ShortTermForecastAlertError> {
    let time_series = loaded
        .dataframe
        .column(time_column)
        .map_err(|_| ShortTermForecastAlertError::MissingColumn {
            column: time_column.to_string(),
        })?
        .as_materialized_series()
        .clone();
    let value_series = loaded
        .dataframe
        .column(value_column)
        .map_err(|_| ShortTermForecastAlertError::MissingColumn {
            column: value_column.to_string(),
        })?
        .as_materialized_series()
        .cast(&DataType::Float64)
        .map_err(|_| ShortTermForecastAlertError::NonNumericColumn {
            column: value_column.to_string(),
        })?;
    let values = value_series
        .f64()
        .map_err(|_| ShortTermForecastAlertError::NonNumericColumn {
            column: value_column.to_string(),
        })?;

    let mut points = time_series
        .iter()
        .zip(values.into_iter())
        .filter_map(|(time_value, numeric_value)| {
            numeric_value
                .and_then(|value| any_value_to_label(&time_value).map(|time| (time, value)))
        })
        .collect::<Vec<_>>();
    points.sort_by(|left, right| left.0.cmp(&right.0));
    Ok(points)
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

fn mean(values: &[f64]) -> f64 {
    if values.is_empty() {
        0.0
    } else {
        values.iter().sum::<f64>() / values.len() as f64
    }
}

fn stddev(values: &[f64]) -> f64 {
    if values.len() < 2 {
        return 0.0;
    }
    let avg = mean(values);
    let variance = values
        .iter()
        .map(|value| {
            let diff = value - avg;
            diff * diff
        })
        .sum::<f64>()
        / values.len() as f64;
    variance.sqrt()
}

fn exponential_smoothing(values: &[f64], alpha: f64) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    let mut smoothed = values[0];
    for value in values.iter().skip(1) {
        smoothed = alpha * value + (1.0 - alpha) * smoothed;
    }
    smoothed
}

fn build_human_summary(
    time_column: &str,
    value_column: &str,
    horizon: usize,
    last_observed_value: f64,
    mean_delta: f64,
    forecasts: &[ForecastStep],
    historical_alerts: &[HistoricalAlert],
) -> ForecastHumanSummary {
    let first_forecast = forecasts.first().map(|step| step.forecast).unwrap_or(0.0);
    let horizon_forecast = forecasts.last().map(|step| step.forecast).unwrap_or(0.0);
    let max_watch_level = forecasts
        .iter()
        .map(|step| step.watch_level.as_str())
        .max_by_key(|level| match *level {
            "high" => 3,
            "medium" => 2,
            _ => 1,
        })
        .unwrap_or("low");

    let overall = format!(
        "Built short-term forecast for `{value_column}` over `{time_column}` with {horizon} horizon steps; highest watch level is `{max_watch_level}`."
    );
    let mut key_points = vec![
        format!(
            "Latest observed value: {:.4}; average period delta: {:.4}.",
            last_observed_value, mean_delta
        ),
        format!(
            "Forecast moves from t+1 {:.4} to t+{} {:.4}.",
            first_forecast, horizon, horizon_forecast
        ),
    ];
    if let Some(alert) = historical_alerts.last() {
        key_points.push(format!(
            "Recent historical alert at `{}` with z-score {:.3} (level {}).",
            alert.time, alert.z_score, alert.alert_level
        ));
    }

    ForecastHumanSummary {
        overall,
        key_points,
        recommended_next_step:
            "Use this as baseline monitoring, then combine with contribution attribution when a high alert is triggered."
                .to_string(),
    }
}
