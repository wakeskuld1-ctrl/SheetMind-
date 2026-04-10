use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::ops::stock::security_decision_verify_package::{
    SecurityDecisionVerifyPackageRequest, SecurityDecisionVerifyPackageResult,
    security_decision_verify_package,
};

// 2026-04-09 CST: 这里扩展 revision 修补建议，原因是 Task 9 标准治理版不能只发现投后复盘问题，还要能给出补件动作；
// 目的：让 verify 失败后，AI / Skill / 人工都能沿统一 remediation 建议完成收口。
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct SecurityDecisionPackageRevisionRequest {
    pub package: Value,
    #[serde(default)]
    pub verification: Option<SecurityDecisionVerifyPackageResult>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct SecurityDecisionPackageRevisionResult {
    pub revision_status: String,
    pub suggested_actions: Vec<String>,
    pub missing_objects: Vec<String>,
    pub manifest_repairs: Vec<String>,
    pub revision_summary: String,
}

pub fn security_decision_package_revision(
    request: &SecurityDecisionPackageRevisionRequest,
) -> SecurityDecisionPackageRevisionResult {
    let verification = request.verification.clone().unwrap_or_else(|| {
        security_decision_verify_package(&SecurityDecisionVerifyPackageRequest {
            package: request.package.clone(),
        })
    });

    let mut suggested_actions = Vec::new();
    let mut missing_objects = Vec::new();
    let mut manifest_repairs = Vec::new();

    for issue in &verification.issues {
        match issue.code.as_str() {
            "missing_post_meeting_conclusion" => {
                push_unique(&mut missing_objects, "post_meeting_conclusion".to_string());
                if issue.suggested_fix.contains("object_graph") {
                    push_unique(
                        &mut suggested_actions,
                        "补挂 post_meeting_conclusion 到 object_graph".to_string(),
                    );
                }
                if issue.suggested_fix.contains("artifact_manifest") {
                    push_unique(
                        &mut manifest_repairs,
                        "补挂 post_meeting_conclusion 到 artifact_manifest".to_string(),
                    );
                }
            }
            "missing_post_trade_review" => {
                push_unique(&mut missing_objects, "post_trade_review".to_string());
                if issue.suggested_fix.contains("object_graph") {
                    push_unique(
                        &mut suggested_actions,
                        "补挂 post_trade_review 到 object_graph".to_string(),
                    );
                }
                if issue.suggested_fix.contains("artifact_manifest") {
                    push_unique(
                        &mut manifest_repairs,
                        "补挂 post_trade_review 到 artifact_manifest".to_string(),
                    );
                }
            }
            "missing_execution_journal" => {
                push_unique(&mut missing_objects, "execution_journal".to_string());
                if issue.suggested_fix.contains("object_graph") {
                    push_unique(
                        &mut suggested_actions,
                        "补挂 execution_journal 到 object_graph".to_string(),
                    );
                }
                if issue.suggested_fix.contains("artifact_manifest") {
                    push_unique(
                        &mut manifest_repairs,
                        "补挂 execution_journal 到 artifact_manifest".to_string(),
                    );
                }
            }
            "missing_execution_record" => {
                push_unique(&mut missing_objects, "execution_record".to_string());
                if issue.suggested_fix.contains("object_graph") {
                    push_unique(
                        &mut suggested_actions,
                        "补挂 execution_record 到 object_graph".to_string(),
                    );
                }
                if issue.suggested_fix.contains("artifact_manifest") {
                    push_unique(
                        &mut manifest_repairs,
                        "补挂 execution_record 到 artifact_manifest".to_string(),
                    );
                }
            }
            "missing_chair_resolution_binding" => {
                push_unique(&mut missing_objects, "chair_resolution".to_string());
                push_unique(
                    &mut suggested_actions,
                    "补挂 chair_resolution 到 object_graph 与 artifact_manifest".to_string(),
                );
            }
            "post_trade_review_ref_misaligned" => {
                push_unique(
                    &mut manifest_repairs,
                    "重新绑定 post_trade_review 的 position_plan_ref / snapshot_ref / outcome_ref"
                        .to_string(),
                );
            }
            "execution_journal_ref_misaligned" => {
                push_unique(
                    &mut manifest_repairs,
                    "重新绑定 execution_journal 与 execution_record / post_trade_review / position_plan / snapshot / outcome 的引用"
                        .to_string(),
                );
            }
            "execution_record_ref_misaligned" => {
                push_unique(
                    &mut manifest_repairs,
                    "重新绑定 execution_record 与 post_trade_review / position_plan / snapshot / outcome 的引用"
                        .to_string(),
                );
            }
            "analysis_date_misaligned" => {
                push_unique(
                    &mut manifest_repairs,
                    "统一 artifact_manifest 的 analysis_date".to_string(),
                );
            }
            "symbol_misaligned" => {
                push_unique(
                    &mut manifest_repairs,
                    "统一 artifact_manifest 的 symbol".to_string(),
                );
            }
            _ => {
                push_unique(&mut suggested_actions, issue.suggested_fix.clone());
            }
        }
    }

    let revision_status = if verification.verification_status == "passed" {
        "no_changes_required"
    } else {
        "repair_required"
    };
    let revision_summary = if verification.verification_status == "passed" {
        "package revision 无需补丁，当前 package 可继续进入后续治理链。".to_string()
    } else {
        format!(
            "package revision 已生成修补建议，共覆盖 {} 个校验问题。",
            verification.issues.len()
        )
    };

    SecurityDecisionPackageRevisionResult {
        revision_status: revision_status.to_string(),
        suggested_actions,
        missing_objects,
        manifest_repairs,
        revision_summary,
    }
}

fn push_unique(target: &mut Vec<String>, candidate: String) {
    if candidate.trim().is_empty() {
        return;
    }
    if !target.iter().any(|item| item == &candidate) {
        target.push(candidate);
    }
}
