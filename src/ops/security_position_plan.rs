use serde::{Deserialize, Serialize};

use crate::ops::stock::security_decision_committee::SecurityDecisionCommitteeResult;

// 2026-04-02 CST: 这里定义证券仓位计划，原因是审批对象需要从“是否可做”继续落到“准备怎么做”；
// 目的：把执行方案独立成正式对象，后续投中管理、复盘和再审批都围绕同一对象演进。
// 2026-04-08 CST: 这里补入合同头、审批绑定和 reduce_plan，原因是 Task 2 要把仓位计划升级成正式可审批对象；
// 目的：让 approval_request、package、verify 和后续执行层都能围绕统一合同消费 position_plan，而不是继续把它当作临时附属输出。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SecurityPositionPlan {
    pub contract_version: String,
    pub document_type: String,
    pub plan_id: String,
    pub decision_id: String,
    pub decision_ref: String,
    pub approval_ref: String,
    pub symbol: String,
    pub analysis_date: String,
    pub plan_direction: String,
    pub plan_status: String,
    pub risk_budget_pct: f64,
    pub suggested_gross_pct: f64,
    pub starter_gross_pct: f64,
    pub max_gross_pct: f64,
    pub entry_plan: PositionEntryPlan,
    pub add_plan: PositionAddPlan,
    pub reduce_plan: PositionReducePlan,
    pub stop_loss_plan: PositionStopLossPlan,
    pub take_profit_plan: PositionTakeProfitPlan,
    pub cancel_conditions: Vec<String>,
    pub sizing_rationale: Vec<String>,
    pub approval_binding: SecurityPositionPlanApprovalBinding,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PositionEntryPlan {
    pub entry_mode: String,
    pub trigger_condition: String,
    pub starter_gross_pct: f64,
    pub notes: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PositionAddPlan {
    pub allow_add: bool,
    pub trigger_condition: String,
    pub max_gross_pct: f64,
    pub notes: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PositionReducePlan {
    pub allow_reduce: bool,
    pub trigger_condition: String,
    pub target_gross_pct: f64,
    pub notes: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PositionStopLossPlan {
    pub stop_loss_pct: f64,
    pub hard_stop_condition: String,
    pub notes: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PositionTakeProfitPlan {
    pub first_target_pct: f64,
    pub second_target_pct: f64,
    pub partial_exit_rule: String,
    pub notes: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SecurityPositionPlanApprovalBinding {
    pub decision_ref: String,
    pub approval_ref: String,
    pub approval_request_ref: String,
    pub package_scope: String,
    pub binding_status: String,
}

// 2026-04-02 CST: 这里定义仓位计划生成输入，原因是执行计划除了 committee 结果，还必须拿到当前审批锚点；
// 目的：确保 position_plan 从第一版起就正式绑定 decision_ref / approval_ref，而不是游离在审批对象之外。
// 2026-04-08 CST: 这里补入 decision_id，原因是 Task 2 要让仓位计划能被审批链和版本链直接定位；
// 目的：避免后续 approval_request / revision / review 再从外部反推这份仓位计划属于哪次决议。
#[derive(Debug, Clone, PartialEq)]
pub struct SecurityPositionPlanBuildInput {
    pub decision_id: String,
    pub decision_ref: String,
    pub approval_ref: String,
}

// 2026-04-02 CST: 这里实现规则型仓位规划器，原因是当前阶段先要稳定生成可审批执行方案，而不是追求复杂优化；
// 目的：用可解释规则把 `blocked / needs_more_evidence / ready_for_review` 分别落成不同仓位级别。
pub fn build_security_position_plan(
    committee: &SecurityDecisionCommitteeResult,
    input: &SecurityPositionPlanBuildInput,
) -> SecurityPositionPlan {
    let status = committee.decision_card.status.as_str();
    let confidence = committee.decision_card.confidence_score;
    let warn_count = committee
        .risk_gates
        .iter()
        .filter(|gate| gate.result == "warn")
        .count();

    let (plan_status, suggested, starter, max_gross, risk_budget, allow_add) = match status {
        "blocked" => ("blocked", 0.0, 0.0, 0.0, 0.0, false),
        "needs_more_evidence" => ("probe_only", 0.05, 0.03, 0.05, 0.005, false),
        _ => {
            let mut suggested = if confidence >= 0.80 { 0.12 } else { 0.10 };
            if warn_count > 0 {
                suggested -= 0.02;
            }
            let starter = if suggested >= 0.10 { 0.06 } else { 0.05 };
            let max_gross = (suggested + 0.03_f64).min(0.15_f64);
            ("reviewable", suggested, starter, max_gross, 0.01, true)
        }
    };

    let stop_loss_pct = parse_percent(&committee.decision_card.downside_risk).unwrap_or(0.05);
    let (first_target_pct, second_target_pct) =
        parse_percent_range(&committee.decision_card.expected_return_range);
    let allow_reduce = plan_status != "blocked";
    let reduce_target_gross_pct = if plan_status == "blocked" {
        0.0
    } else if plan_status == "probe_only" {
        0.0
    } else {
        starter
    };
    let plan_direction = normalize_plan_direction(&committee.decision_card.exposure_side);

    let cancel_conditions = if plan_status == "blocked" {
        vec![
            "当前风险闸门未通过，不进入执行。".to_string(),
            committee.decision_card.final_recommendation.clone(),
        ]
    } else if plan_status == "probe_only" {
        vec![
            "补齐证据前不得扩大仓位。".to_string(),
            "若出现新增阻断性风险闸门，取消执行。".to_string(),
        ]
    } else {
        vec![
            "若跌破止损条件则取消后续加仓。".to_string(),
            "若市场或板块环境明显转逆风，则暂停执行。".to_string(),
        ]
    };

    let sizing_rationale = match plan_status {
        "blocked" => vec![
            "当前投决状态为 blocked，因此仓位计划归零。".to_string(),
            "执行计划仅保留取消条件，不生成建仓动作。".to_string(),
        ],
        "probe_only" => vec![
            "当前仅处于 needs_more_evidence，对应试探仓计划。".to_string(),
            "在补证据并重新审批前，不允许扩大仓位。".to_string(),
        ],
        _ => vec![
            format!("当前可进入审阅状态，置信度 {:.2}。", confidence),
            format!("存在 {} 个提醒闸门，已在仓位上做降档处理。", warn_count),
        ],
    };

    SecurityPositionPlan {
        contract_version: "security_position_plan.v2".to_string(),
        document_type: "security_position_plan".to_string(),
        plan_id: format!("plan-{}", committee.decision_card.decision_id),
        decision_id: input.decision_id.clone(),
        decision_ref: input.decision_ref.clone(),
        approval_ref: input.approval_ref.clone(),
        symbol: committee.symbol.clone(),
        analysis_date: committee.analysis_date.clone(),
        plan_direction,
        plan_status: plan_status.to_string(),
        risk_budget_pct: risk_budget,
        suggested_gross_pct: suggested,
        starter_gross_pct: starter,
        max_gross_pct: max_gross,
        entry_plan: PositionEntryPlan {
            entry_mode: if plan_status == "blocked" {
                "disabled".to_string()
            } else if plan_status == "probe_only" {
                "probe".to_string()
            } else {
                "staged".to_string()
            },
            trigger_condition: if plan_status == "blocked" {
                "当前不允许建仓".to_string()
            } else {
                format!("首仓 {:.1}% ，满足投决条件后执行", starter * 100.0)
            },
            starter_gross_pct: starter,
            notes: format!("首仓方案依据当前状态 {}", plan_status),
        },
        add_plan: PositionAddPlan {
            allow_add,
            trigger_condition: if allow_add {
                "回踩确认或突破延续后允许加仓".to_string()
            } else {
                "当前不允许加仓".to_string()
            },
            max_gross_pct: max_gross,
            notes: if allow_add {
                "加仓前需继续满足风险闸门要求".to_string()
            } else {
                "补证据或风险解除前禁止加仓".to_string()
            },
        },
        reduce_plan: PositionReducePlan {
            allow_reduce,
            trigger_condition: if allow_reduce {
                "达到第一目标位或市场环境转弱时允许主动减仓".to_string()
            } else {
                "当前无持仓可减".to_string()
            },
            target_gross_pct: reduce_target_gross_pct,
            notes: if allow_reduce {
                "减仓规则用于把仓位降回更稳健区间，避免只定义加仓而不定义收缩。".to_string()
            } else {
                "blocked 状态下不生成减仓动作。".to_string()
            },
        },
        stop_loss_plan: PositionStopLossPlan {
            stop_loss_pct,
            hard_stop_condition: if plan_status == "blocked" {
                "不执行，无止损动作".to_string()
            } else {
                format!("跌破 {:.1}% 风险线则执行硬止损", stop_loss_pct * 100.0)
            },
            notes: "止损线直接继承投决会风险参数".to_string(),
        },
        take_profit_plan: PositionTakeProfitPlan {
            first_target_pct,
            second_target_pct,
            partial_exit_rule: if plan_status == "blocked" {
                "不执行，无止盈动作".to_string()
            } else {
                "第一目标减仓三分之一，第二目标继续兑现".to_string()
            },
            notes: "止盈目标沿用投决卡预期收益区间".to_string(),
        },
        cancel_conditions,
        sizing_rationale,
        approval_binding: SecurityPositionPlanApprovalBinding {
            decision_ref: input.decision_ref.clone(),
            approval_ref: input.approval_ref.clone(),
            approval_request_ref: input.approval_ref.clone(),
            package_scope: "security_decision_submit_approval".to_string(),
            binding_status: "bound_to_approval_request".to_string(),
        },
    }
}

fn normalize_plan_direction(value: &str) -> String {
    match value.trim().to_ascii_lowercase().as_str() {
        "long" => "Long".to_string(),
        "short" => "Short".to_string(),
        "hedge" => "Hedge".to_string(),
        "neutral" => "NoTrade".to_string(),
        _ => "NoTrade".to_string(),
    }
}

fn parse_percent(value: &str) -> Option<f64> {
    value
        .trim()
        .trim_end_matches('%')
        .parse::<f64>()
        .ok()
        .map(|v| v / 100.0)
}

fn parse_percent_range(value: &str) -> (f64, f64) {
    let values: Vec<f64> = value
        .split('-')
        .filter_map(|part| parse_percent(part))
        .collect();
    let first = values.first().copied().unwrap_or(0.0);
    let second = values.get(1).copied().unwrap_or(first);
    (first, second)
}
