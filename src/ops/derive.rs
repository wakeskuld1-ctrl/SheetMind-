use chrono::{NaiveDate, NaiveDateTime};
use polars::prelude::{DataFrame, NamedFrom, Series};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::domain::handles::TableHandle;
use crate::frame::loader::LoadedTable;

// 2026-03-22: 这里定义派生字段规格，目的是把打标、分桶和评分统一收敛到一个可复用 Tool 协议里。
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum DerivationSpec {
    CaseWhen {
        output_column: String,
        rules: Vec<CaseWhenRule>,
        else_value: String,
    },
    Bucketize {
        source_column: String,
        output_column: String,
        buckets: Vec<BucketRule>,
        else_value: String,
    },
    ScoreRules {
        output_column: String,
        #[serde(default)]
        default_score: i64,
        rules: Vec<ScoreRule>,
    },
    DateBucketize {
        source_column: String,
        output_column: String,
        buckets: Vec<DateBucketRule>,
        else_value: String,
    },
    Template {
        output_column: String,
        template: String,
    },
}

// 2026-03-22: 这里定义条件打标规则，目的是让经营分析里的优先级、标签和推荐原因先支持最常见的 if-else 模式。
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct CaseWhenRule {
    pub when: DerivePredicate,
    pub value: String,
}

// 2026-03-22: 这里定义分桶规则，目的是让金额段、活跃度段、收入段这些业务标签可以稳定下沉到 Tool 层。
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct BucketRule {
    pub label: String,
    pub min_inclusive: Option<f64>,
    pub max_exclusive: Option<f64>,
}

// 2026-03-22: 这里定义评分规则，目的是让多规则累计打分先有一个最小可用的确定性计算入口。
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct ScoreRule {
    pub when: DerivePredicate,
    pub score: i64,
}

// 2026-03-22: 这里定义派生条件，目的是让打标和评分共享同一套规则判断表达。
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct DeriveCondition {
    pub column: String,
    pub operator: DeriveOperator,
    pub value: String,
}

// 2026-03-23: 这里新增条件组，目的是在不引入完整表达式引擎的前提下先覆盖 all/any 两类高频规则组合。
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct DeriveConditionGroup {
    pub mode: LogicalMode,
    pub conditions: Vec<DeriveCondition>,
}

// 2026-03-23: 这里用非标记联合兼容旧单条件与新条件组，目的是保证原有 JSON 契约不被破坏。
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(untagged)]
pub enum DerivePredicate {
    Condition(DeriveCondition),
    Group(DeriveConditionGroup),
}

// 2026-03-23: 这里定义条件组模式，目的是让 Tool 层用稳定枚举表达 and/or 语义。
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LogicalMode {
    All,
    Any,
}

// 2026-03-22: 这里定义最小操作符集合，目的是先覆盖分层、阈值判断和等值标签的主路径。
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DeriveOperator {
    Equals,
    Gte,
    Gt,
    Lte,
    Lt,
}

// 2026-03-23: 这里新增日期分段规则，目的是让旺季/淡季、季度、活动期等标签稳定下沉到 Tool 层。
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct DateBucketRule {
    pub label: String,
    pub start_inclusive: Option<String>,
    pub end_exclusive: Option<String>,
}

// 2026-03-22: 这里定义派生错误，目的是把缺列、空规格和规则解析失败统一转换成可读中文错误。
#[derive(Debug, Error)]
pub enum DeriveError {
    #[error("derive_columns 至少需要一个 derivation")]
    EmptyDerivations,
    #[error("找不到列: {0}")]
    MissingColumn(String),
    #[error("派生规则无效: {0}")]
    InvalidRule(String),
    #[error("无法写回派生列 `{column}`: {message}")]
    WriteColumn { column: String, message: String },
}

// 2026-03-22: 这里执行派生字段主入口，目的是把规则型中间表生成能力稳定下沉到 Rust Tool 层。
pub fn derive_columns(
    loaded: &LoadedTable,
    derivations: &[DerivationSpec],
) -> Result<LoadedTable, DeriveError> {
    if derivations.is_empty() {
        return Err(DeriveError::EmptyDerivations);
    }

    let mut dataframe = loaded.dataframe.clone();

    for derivation in derivations {
        match derivation {
            DerivationSpec::CaseWhen {
                output_column,
                rules,
                else_value,
            } => {
                let values = (0..dataframe.height())
                    .map(|row_index| {
                        rules
                            .iter()
                            .find(|rule| {
                                evaluate_predicate(&dataframe, row_index, &rule.when)
                                    .unwrap_or(false)
                            })
                            .map(|rule| rule.value.clone())
                            .unwrap_or_else(|| else_value.clone())
                    })
                    .collect::<Vec<_>>();
                upsert_series(
                    &mut dataframe,
                    Series::new(output_column.clone().into(), values),
                    output_column,
                )?;
            }
            DerivationSpec::Bucketize {
                source_column,
                output_column,
                buckets,
                else_value,
            } => {
                ensure_column_exists(&dataframe, source_column)?;
                let values = (0..dataframe.height())
                    .map(|row_index| {
                        let current = numeric_value(&dataframe, source_column, row_index)
                            .ok()
                            .flatten();
                        match current {
                            Some(current) => buckets
                                .iter()
                                .find(|bucket| bucket_matches(bucket, current))
                                .map(|bucket| bucket.label.clone())
                                .unwrap_or_else(|| else_value.clone()),
                            None => else_value.clone(),
                        }
                    })
                    .collect::<Vec<_>>();
                upsert_series(
                    &mut dataframe,
                    Series::new(output_column.clone().into(), values),
                    output_column,
                )?;
            }
            DerivationSpec::ScoreRules {
                output_column,
                default_score,
                rules,
            } => {
                let values = (0..dataframe.height())
                    .map(|row_index| {
                        rules.iter().fold(*default_score, |acc, rule| {
                            if evaluate_predicate(&dataframe, row_index, &rule.when)
                                .unwrap_or(false)
                            {
                                acc + rule.score
                            } else {
                                acc
                            }
                        })
                    })
                    .collect::<Vec<_>>();
                upsert_series(
                    &mut dataframe,
                    Series::new(output_column.clone().into(), values),
                    output_column,
                )?;
            }
            DerivationSpec::DateBucketize {
                source_column,
                output_column,
                buckets,
                else_value,
            } => {
                ensure_column_exists(&dataframe, source_column)?;
                let values = (0..dataframe.height())
                    .map(|row_index| {
                        let current = date_value(&dataframe, source_column, row_index)
                            .ok()
                            .flatten();
                        match current {
                            Some(current) => buckets
                                .iter()
                                .find(|bucket| date_bucket_matches(bucket, current).unwrap_or(false))
                                .map(|bucket| bucket.label.clone())
                                .unwrap_or_else(|| else_value.clone()),
                            None => else_value.clone(),
                        }
                    })
                    .collect::<Vec<_>>();
                upsert_series(
                    &mut dataframe,
                    Series::new(output_column.clone().into(), values),
                    output_column,
                )?;
            }
            DerivationSpec::Template {
                output_column,
                template,
            } => {
                let values = (0..dataframe.height())
                    .map(|row_index| render_template(&dataframe, row_index, template))
                    .collect::<Result<Vec<_>, DeriveError>>()?;
                upsert_series(
                    &mut dataframe,
                    Series::new(output_column.clone().into(), values),
                    output_column,
                )?;
            }
        }
    }

    let handle = TableHandle::new_confirmed(
        loaded.handle.source_path(),
        loaded.handle.sheet_name(),
        dataframe
            .get_column_names()
            .iter()
            .map(|name| name.to_string())
            .collect(),
    );

    Ok(LoadedTable { handle, dataframe })
}

// 2026-03-23: 这里统一判断单条件或条件组，目的是让打标和评分在最小扩展下支持 all/any 组合。
fn evaluate_predicate(
    dataframe: &DataFrame,
    row_index: usize,
    predicate: &DerivePredicate,
) -> Result<bool, DeriveError> {
    match predicate {
        DerivePredicate::Condition(condition) => evaluate_condition(dataframe, row_index, condition),
        DerivePredicate::Group(group) => match group.mode {
            LogicalMode::All => group
                .conditions
                .iter()
                .map(|condition| evaluate_condition(dataframe, row_index, condition))
                .try_fold(true, |acc, result| result.map(|matched| acc && matched)),
            LogicalMode::Any => group
                .conditions
                .iter()
                .map(|condition| evaluate_condition(dataframe, row_index, condition))
                .try_fold(false, |acc, result| result.map(|matched| acc || matched)),
        },
    }
}

// 2026-03-22: 这里统一判断规则条件，目的是让打标和评分复用同一套表达式解释逻辑。
fn evaluate_condition(
    dataframe: &DataFrame,
    row_index: usize,
    condition: &DeriveCondition,
) -> Result<bool, DeriveError> {
    ensure_column_exists(dataframe, &condition.column)?;
    let series = dataframe
        .column(&condition.column)
        .map_err(|_| DeriveError::MissingColumn(condition.column.clone()))?
        .as_materialized_series();

    match condition.operator {
        DeriveOperator::Equals => Ok(series
            .str_value(row_index)
            .map(|value| value.as_ref() == condition.value)
            .unwrap_or(false)),
        DeriveOperator::Gte => {
            compare_numeric(series, row_index, &condition.value, |left, right| {
                left >= right
            })
        }
        DeriveOperator::Gt => {
            compare_numeric(series, row_index, &condition.value, |left, right| {
                left > right
            })
        }
        DeriveOperator::Lte => {
            compare_numeric(series, row_index, &condition.value, |left, right| {
                left <= right
            })
        }
        DeriveOperator::Lt => {
            compare_numeric(series, row_index, &condition.value, |left, right| {
                left < right
            })
        }
    }
}

// 2026-03-22: 这里集中做数值比较，目的是让阈值判断全部走同一条解析和兜底路径。
fn compare_numeric(
    series: &polars::prelude::Series,
    row_index: usize,
    expected: &str,
    comparator: impl Fn(f64, f64) -> bool,
) -> Result<bool, DeriveError> {
    let Some(current) = series
        .str_value(row_index)
        .ok()
        .and_then(|value| value.parse::<f64>().ok())
    else {
        return Ok(false);
    };
    let expected = expected.parse::<f64>().map_err(|error| {
        DeriveError::InvalidRule(format!("无法把 `{expected}` 解析为数值条件: {error}"))
    })?;
    Ok(comparator(current, expected))
}

// 2026-03-22: 这里从表中读取数值，目的是让数值分桶统一复用同一套解析逻辑。
fn numeric_value(
    dataframe: &DataFrame,
    column: &str,
    row_index: usize,
) -> Result<Option<f64>, DeriveError> {
    let series = dataframe
        .column(column)
        .map_err(|_| DeriveError::MissingColumn(column.to_string()))?
        .as_materialized_series();
    Ok(series
        .str_value(row_index)
        .ok()
        .and_then(|value| value.parse::<f64>().ok()))
}

// 2026-03-22: 这里统一判断某个数值是否落入桶内，目的是把金额段、活跃度段等规则保持可测试。
fn bucket_matches(bucket: &BucketRule, current: f64) -> bool {
    let lower_ok = bucket.min_inclusive.is_none_or(|lower| current >= lower);
    let upper_ok = bucket.max_exclusive.is_none_or(|upper| current < upper);
    lower_ok && upper_ok
}

// 2026-03-23: 这里统一读取日期值，目的是让日期分段可直接消费常见日期与日期时间文本。
fn date_value(
    dataframe: &DataFrame,
    column: &str,
    row_index: usize,
) -> Result<Option<NaiveDate>, DeriveError> {
    let series = dataframe
        .column(column)
        .map_err(|_| DeriveError::MissingColumn(column.to_string()))?
        .as_materialized_series();
    Ok(series
        .str_value(row_index)
        .ok()
        .and_then(|value| parse_date_like(value.as_ref()).ok()))
}

// 2026-03-23: 这里统一判断日期是否落入分段，目的是让季度、旺季等规则保持可测试。
fn date_bucket_matches(bucket: &DateBucketRule, current: NaiveDate) -> Result<bool, DeriveError> {
    let lower_ok = match &bucket.start_inclusive {
        Some(start) => current >= parse_date_like(start)?,
        None => true,
    };
    let upper_ok = match &bucket.end_exclusive {
        Some(end) => current < parse_date_like(end)?,
        None => true,
    };
    Ok(lower_ok && upper_ok)
}

// 2026-03-23: 这里把模板列渲染单独收口，目的是让“推荐原因/说明文本”生成保持简单稳定。
fn render_template(
    dataframe: &DataFrame,
    row_index: usize,
    template: &str,
) -> Result<String, DeriveError> {
    let mut rendered = String::new();
    let mut cursor = 0usize;
    while let Some(start_offset) = template[cursor..].find('{') {
        let start = cursor + start_offset;
        rendered.push_str(&template[cursor..start]);
        let Some(end_offset) = template[start + 1..].find('}') else {
            rendered.push_str(&template[start..]);
            return Ok(rendered);
        };
        let end = start + 1 + end_offset;
        let column_name = &template[start + 1..end];
        ensure_column_exists(dataframe, column_name)?;
        let series = dataframe
            .column(column_name)
            .map_err(|_| DeriveError::MissingColumn(column_name.to_string()))?
            .as_materialized_series();
        let value = series
            .str_value(row_index)
            .map(|item| item.into_owned())
            .unwrap_or_default();
        rendered.push_str(&value);
        cursor = end + 1;
    }
    rendered.push_str(&template[cursor..]);
    Ok(rendered)
}

// 2026-03-23: 这里统一解析常见日期/日期时间文本，目的是让日期分段不依赖单独的预处理 Tool 也能工作。
fn parse_date_like(value: &str) -> Result<NaiveDate, DeriveError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(DeriveError::InvalidRule("日期值不能为空".to_string()));
    }

    if let Ok(date) = NaiveDate::parse_from_str(trimmed, "%Y-%m-%d") {
        return Ok(date);
    }
    if let Ok(date) = NaiveDate::parse_from_str(trimmed, "%Y/%m/%d") {
        return Ok(date);
    }
    if let Ok(date_time) = NaiveDateTime::parse_from_str(trimmed, "%Y-%m-%d %H:%M:%S") {
        return Ok(date_time.date());
    }
    if let Ok(date_time) = NaiveDateTime::parse_from_str(trimmed, "%Y-%m-%d %H:%M") {
        return Ok(date_time.date());
    }
    if let Ok(date_time) = NaiveDateTime::parse_from_str(trimmed, "%Y-%m-%dT%H:%M:%S") {
        return Ok(date_time.date());
    }

    Err(DeriveError::InvalidRule(format!(
        "无法把 `{trimmed}` 解析为日期"
    )))
}

// 2026-03-22: 这里统一写回派生列，目的是支持“新增列”和“覆盖同名派生列”两种情况而不让 dispatcher 关心细节。
fn upsert_series(
    dataframe: &mut DataFrame,
    series: Series,
    column_name: &str,
) -> Result<(), DeriveError> {
    if let Some(column_index) = dataframe.get_column_index(column_name) {
        dataframe
            .replace_column(column_index, series)
            .map_err(|error| DeriveError::WriteColumn {
                column: column_name.to_string(),
                message: error.to_string(),
            })?;
    } else {
        dataframe
            .with_column(series)
            .map_err(|error| DeriveError::WriteColumn {
                column: column_name.to_string(),
                message: error.to_string(),
            })?;
    }
    Ok(())
}

// 2026-03-22: 这里统一校验列存在性，目的是让规则错误尽早在 Tool 层显式暴露出来。
fn ensure_column_exists(dataframe: &DataFrame, column: &str) -> Result<(), DeriveError> {
    if dataframe.get_column_index(column).is_none() {
        return Err(DeriveError::MissingColumn(column.to_string()));
    }
    Ok(())
}
