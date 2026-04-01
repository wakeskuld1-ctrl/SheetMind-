use polars::prelude::{DataFrame, SortMultipleOptions};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::domain::handles::TableHandle;
use crate::frame::loader::LoadedTable;

// 2026-03-21: 这里定义单个聚合请求，目的是把分析层的聚合意图稳定映射成 DataFrame 可执行的结构化输入。
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct AggregationSpec {
    // 2026-03-21: 指定目标列名，目的是让用户或 Skill 明确知道对哪一列做统计聚合。
    pub column: String,
    // 2026-03-21: 指定聚合算子，目的是让 V1 先聚焦最常见的汇总分析动作。
    pub operator: AggregationOperator,
}

// 2026-03-21: 这里定义 V1 支持的聚合算子，目的是优先覆盖多维分析入口最常见的指标统计。
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AggregationOperator {
    // 2026-03-21: 计数是最基础指标，目的是支持频次分析和记录量统计。
    Count,
    // 2026-03-21: 求和是金额、销量等场景的核心指标，目的是支持最常见业务汇总。
    Sum,
    // 2026-03-21: 均值是分析层的常见入口，目的是为回归和趋势分析提供基础指标。
    Mean,
    // 2026-03-21: 最小值可用于区间和异常分析，目的是支持基础分布洞察。
    Min,
    // 2026-03-21: 最大值可用于峰值分析，目的是支持业务监控和极值检查。
    Max,
}

// 2026-03-21: 这里定义分组聚合错误，目的是把空请求、缺列和聚合执行失败分开暴露给上层。
#[derive(Debug, Error)]
pub enum GroupError {
    // 2026-03-21: 不允许空分组列，目的是避免把“整体汇总”和“分组汇总”混成一类语义。
    #[error("group_and_aggregate 至少需要一个 group_by 列")]
    EmptyGroupBy,
    // 2026-03-21: 不允许空聚合计划，目的是避免返回只有分组键没有指标的结果表。
    #[error("group_and_aggregate 至少需要一个聚合定义")]
    EmptyAggregations,
    // 2026-03-21: 列不存在时返回明确错误，目的是帮助用户快速修正字段名。
    #[error("找不到列: {0}")]
    MissingColumn(String),
    // 2026-03-21: 聚合失败时统一包装底层错误，目的是让 Tool 层拿到稳定错误语义。
    #[error("无法完成分组聚合: {0}")]
    AggregateFrame(String),
}

// 2026-03-21: 这里执行显式分组聚合，目的是把表处理推进到真正的多维分析入口。
pub fn group_and_aggregate(
    loaded: &LoadedTable,
    group_by: &[&str],
    aggregations: &[AggregationSpec],
) -> Result<LoadedTable, GroupError> {
    if group_by.is_empty() {
        return Err(GroupError::EmptyGroupBy);
    }
    if aggregations.is_empty() {
        return Err(GroupError::EmptyAggregations);
    }

    ensure_columns_exist(&loaded.dataframe, group_by)?;
    for aggregation in aggregations {
        ensure_columns_exist(&loaded.dataframe, &[aggregation.column.as_str()])?;
    }

    let mut grouped_frame: Option<DataFrame> = None;

    for aggregation in aggregations {
        let aggregated = aggregate_once(&loaded.dataframe, group_by, aggregation)?;
        let sorted = aggregated
            .sort(
                group_by.iter().copied().collect::<Vec<_>>(),
                SortMultipleOptions::default().with_maintain_order(true),
            )
            .map_err(|error| GroupError::AggregateFrame(error.to_string()))?;

        grouped_frame = Some(match grouped_frame {
            Some(current) => {
                // 2026-03-21: 后续聚合结果只拼接指标列，目的是避免重复附加分组键导致结果表结构混乱。
                let extra_columns = sorted.get_columns()[group_by.len()..].to_vec();
                current
                    .hstack(&extra_columns)
                    .map_err(|error| GroupError::AggregateFrame(error.to_string()))?
            }
            None => sorted,
        });
    }

    let dataframe = grouped_frame.expect("aggregations is checked as non-empty");
    let handle = TableHandle::new_confirmed(
        loaded.handle.source_path(),
        loaded.handle.sheet_name(),
        dataframe
            .get_column_names_str()
            .into_iter()
            .map(|name| name.to_string())
            .collect(),
    );

    Ok(LoadedTable { handle, dataframe })
}

// 2026-03-21: 这里校验列存在性，目的是在真正聚合前先返回更友好的缺列错误。
fn ensure_columns_exist(dataframe: &DataFrame, columns: &[&str]) -> Result<(), GroupError> {
    for column in columns {
        if dataframe.column(column).is_err() {
            return Err(GroupError::MissingColumn((*column).to_string()));
        }
    }
    Ok(())
}

// 2026-03-21: 这里抽出单个聚合执行逻辑，目的是把多聚合拼接流程和单聚合实现解耦，符合 SRP。
#[allow(deprecated)]
fn aggregate_once(
    dataframe: &DataFrame,
    group_by: &[&str],
    aggregation: &AggregationSpec,
) -> Result<DataFrame, GroupError> {
    let grouped = dataframe
        .group_by(group_by.iter().copied())
        .map_err(|error| GroupError::AggregateFrame(error.to_string()))?;
    let selected = grouped.select([aggregation.column.as_str()]);

    match aggregation.operator {
        AggregationOperator::Count => selected.count(),
        AggregationOperator::Sum => selected.sum(),
        AggregationOperator::Mean => selected.mean(),
        AggregationOperator::Min => selected.min(),
        AggregationOperator::Max => selected.max(),
    }
    .map_err(|error| GroupError::AggregateFrame(error.to_string()))
}
