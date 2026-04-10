use serde::{Deserialize, Serialize};
use serde_json::Value;

// 2026-04-09 CST: 这里保持 verify 直接消费正式 package，原因是 Task 9 要把投后复盘挂进同一治理闭环；
// 目的：让 package 校验同时覆盖 post_meeting 和 post_trade_review，而不是只检查前半段对象。
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct SecurityDecisionVerifyPackageRequest {
    pub package: Value,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct SecurityDecisionVerifyPackageCheck {
    pub check_name: String,
    pub status: String,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct SecurityDecisionVerifyPackageIssue {
    pub severity: String,
    pub code: String,
    pub detail: String,
    pub suggested_fix: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct SecurityDecisionVerifyPackageResult {
    pub verification_status: String,
    pub checks: Vec<SecurityDecisionVerifyPackageCheck>,
    pub issues: Vec<SecurityDecisionVerifyPackageIssue>,
    pub verification_summary: String,
}

pub fn security_decision_verify_package(
    request: &SecurityDecisionVerifyPackageRequest,
) -> SecurityDecisionVerifyPackageResult {
    let package = &request.package;
    let symbol = string_field(package, "symbol");
    let analysis_date = string_field(package, "analysis_date");
    let post_meeting_id = nested_string_field(
        package,
        &["post_meeting_conclusion", "post_meeting_conclusion_id"],
    );
    let post_trade_review_id = nested_string_field(package, &["post_trade_review", "review_id"]);
    let execution_journal_id =
        nested_string_field(package, &["execution_journal", "execution_journal_id"]);
    let execution_record_id =
        nested_string_field(package, &["execution_record", "execution_record_id"]);

    let graph_has_post_meeting =
        object_graph_contains(package, "post_meeting_conclusion", Some(&post_meeting_id));
    let manifest_has_post_meeting = manifest_contains(
        package,
        "security_post_meeting_conclusion",
        Some(&post_meeting_id),
    );
    let graph_has_post_trade_review =
        object_graph_contains(package, "post_trade_review", Some(&post_trade_review_id));
    let manifest_has_post_trade_review = manifest_contains(
        package,
        "security_post_trade_review",
        Some(&post_trade_review_id),
    );
    let post_trade_review_refs_aligned = post_trade_review_refs_align(package);
    let graph_has_execution_journal =
        object_graph_contains(package, "execution_journal", Some(&execution_journal_id));
    let manifest_has_execution_journal = manifest_contains(
        package,
        "security_execution_journal",
        Some(&execution_journal_id),
    );
    let execution_journal_refs_aligned = execution_journal_refs_align(package);
    let graph_has_execution_record =
        object_graph_contains(package, "execution_record", Some(&execution_record_id));
    let manifest_has_execution_record = manifest_contains(
        package,
        "security_execution_record",
        Some(&execution_record_id),
    );
    let execution_record_refs_aligned = execution_record_refs_align(package);
    let graph_has_chair = object_graph_contains(package, "chair_resolution", None);
    let manifest_has_chair = manifest_contains(package, "security_chair_resolution", None);
    let analysis_date_aligned = manifest_dates_align(package, &analysis_date);
    let symbol_aligned = manifest_symbols_align(package, &symbol);

    let mut checks = Vec::new();
    let mut issues = Vec::new();

    push_check(
        &mut checks,
        "object_graph_post_meeting_bound",
        graph_has_post_meeting,
        "检查 object_graph 是否已挂 post_meeting_conclusion。",
    );
    if !graph_has_post_meeting {
        issues.push(SecurityDecisionVerifyPackageIssue {
            severity: "high".to_string(),
            code: "missing_post_meeting_conclusion".to_string(),
            detail: "decision_package.object_graph 缺少 post_meeting_conclusion 节点。".to_string(),
            suggested_fix: "补挂 post_meeting_conclusion 到 object_graph".to_string(),
        });
    }

    push_check(
        &mut checks,
        "artifact_manifest_post_meeting_bound",
        manifest_has_post_meeting,
        "检查 artifact_manifest 是否已登记 security_post_meeting_conclusion。",
    );
    if !manifest_has_post_meeting {
        issues.push(SecurityDecisionVerifyPackageIssue {
            severity: "high".to_string(),
            code: "missing_post_meeting_conclusion".to_string(),
            detail:
                "decision_package.artifact_manifest 缺少 security_post_meeting_conclusion 条目。"
                    .to_string(),
            suggested_fix: "补挂 post_meeting_conclusion 到 artifact_manifest".to_string(),
        });
    }

    push_check(
        &mut checks,
        "object_graph_post_trade_review_bound",
        graph_has_post_trade_review,
        "检查 object_graph 是否已挂 post_trade_review 节点。",
    );
    if !graph_has_post_trade_review {
        issues.push(SecurityDecisionVerifyPackageIssue {
            severity: "high".to_string(),
            code: "missing_post_trade_review".to_string(),
            detail: "decision_package.object_graph 缺少 post_trade_review 节点。".to_string(),
            suggested_fix: "补挂 post_trade_review 到 object_graph".to_string(),
        });
    }

    push_check(
        &mut checks,
        "artifact_manifest_post_trade_review_bound",
        manifest_has_post_trade_review,
        "检查 artifact_manifest 是否已登记 security_post_trade_review。",
    );
    if !manifest_has_post_trade_review {
        issues.push(SecurityDecisionVerifyPackageIssue {
            severity: "high".to_string(),
            code: "missing_post_trade_review".to_string(),
            detail: "decision_package.artifact_manifest 缺少 security_post_trade_review 条目。"
                .to_string(),
            suggested_fix: "补挂 post_trade_review 到 artifact_manifest".to_string(),
        });
    }

    push_check(
        &mut checks,
        "object_graph_execution_journal_bound",
        graph_has_execution_journal,
        "检查 object_graph 是否已挂 execution_journal 节点。",
    );
    if !graph_has_execution_journal {
        issues.push(SecurityDecisionVerifyPackageIssue {
            severity: "high".to_string(),
            code: "missing_execution_journal".to_string(),
            detail: "decision_package.object_graph 缺少 execution_journal 节点。".to_string(),
            suggested_fix: "补挂 execution_journal 到 object_graph".to_string(),
        });
    }

    push_check(
        &mut checks,
        "artifact_manifest_execution_journal_bound",
        manifest_has_execution_journal,
        "检查 artifact_manifest 是否已登记 security_execution_journal。",
    );
    if !manifest_has_execution_journal {
        issues.push(SecurityDecisionVerifyPackageIssue {
            severity: "high".to_string(),
            code: "missing_execution_journal".to_string(),
            detail: "decision_package.artifact_manifest 缺少 security_execution_journal 条目。"
                .to_string(),
            suggested_fix: "补挂 execution_journal 到 artifact_manifest".to_string(),
        });
    }

    push_check(
        &mut checks,
        "execution_journal_refs_consistent",
        execution_journal_refs_aligned,
        "检查 execution_journal 是否与 execution_record / post_trade_review / position_plan / snapshot / outcome 保持同源引用。",
    );
    if !execution_journal_refs_aligned {
        issues.push(SecurityDecisionVerifyPackageIssue {
            severity: "medium".to_string(),
            code: "execution_journal_ref_misaligned".to_string(),
            detail: "execution_journal 与 execution_record / post_trade_review / position_plan / snapshot / outcome 的引用不一致。"
                .to_string(),
            suggested_fix:
                "重新绑定 execution_journal 与 execution_record / post_trade_review / position_plan / snapshot / outcome 的引用"
                    .to_string(),
        });
    }

    push_check(
        &mut checks,
        "object_graph_execution_record_bound",
        graph_has_execution_record,
        "检查 object_graph 是否已挂 execution_record 节点。",
    );
    if !graph_has_execution_record {
        issues.push(SecurityDecisionVerifyPackageIssue {
            severity: "high".to_string(),
            code: "missing_execution_record".to_string(),
            detail: "decision_package.object_graph 缺少 execution_record 节点。".to_string(),
            suggested_fix: "补挂 execution_record 到 object_graph".to_string(),
        });
    }

    push_check(
        &mut checks,
        "artifact_manifest_execution_record_bound",
        manifest_has_execution_record,
        "检查 artifact_manifest 是否已登记 security_execution_record。",
    );
    if !manifest_has_execution_record {
        issues.push(SecurityDecisionVerifyPackageIssue {
            severity: "high".to_string(),
            code: "missing_execution_record".to_string(),
            detail: "decision_package.artifact_manifest 缺少 security_execution_record 条目。"
                .to_string(),
            suggested_fix: "补挂 execution_record 到 artifact_manifest".to_string(),
        });
    }

    push_check(
        &mut checks,
        "execution_record_refs_consistent",
        execution_record_refs_aligned,
        "检查 execution_record 是否与 post_trade_review / position_plan / snapshot / outcome 保持同源引用。",
    );
    if !execution_record_refs_aligned {
        issues.push(SecurityDecisionVerifyPackageIssue {
            severity: "medium".to_string(),
            code: "execution_record_ref_misaligned".to_string(),
            detail: "execution_record 与 post_trade_review / position_plan / snapshot / outcome 的引用不一致。"
                .to_string(),
            suggested_fix:
                "重新绑定 execution_record 与 post_trade_review / position_plan / snapshot / outcome 的引用"
                    .to_string(),
        });
    }

    push_check(
        &mut checks,
        "post_trade_review_refs_consistent",
        post_trade_review_refs_aligned,
        "检查 post_trade_review 是否与 position_plan / snapshot / outcome 保持同源引用。",
    );
    if !post_trade_review_refs_aligned {
        issues.push(SecurityDecisionVerifyPackageIssue {
            severity: "medium".to_string(),
            code: "post_trade_review_ref_misaligned".to_string(),
            detail: "post_trade_review 的 position_plan_ref / snapshot_ref / outcome_ref 与底层结果不一致。"
                .to_string(),
            suggested_fix: "重新绑定 post_trade_review 的 position_plan_ref / snapshot_ref / outcome_ref"
                .to_string(),
        });
    }

    push_check(
        &mut checks,
        "chair_resolution_bound",
        graph_has_chair && manifest_has_chair,
        "检查 chair_resolution 是否仍保留为 package 中的正式上游对象。",
    );
    if !(graph_has_chair && manifest_has_chair) {
        issues.push(SecurityDecisionVerifyPackageIssue {
            severity: "medium".to_string(),
            code: "missing_chair_resolution_binding".to_string(),
            detail: "chair_resolution 缺少 object_graph 或 artifact_manifest 挂载。".to_string(),
            suggested_fix: "补挂 chair_resolution 到 object_graph 与 artifact_manifest".to_string(),
        });
    }

    push_check(
        &mut checks,
        "analysis_date_consistent",
        analysis_date_aligned,
        "检查 manifest 条目的 analysis_date 是否与 package 主分析日一致。",
    );
    if !analysis_date_aligned {
        issues.push(SecurityDecisionVerifyPackageIssue {
            severity: "medium".to_string(),
            code: "analysis_date_misaligned".to_string(),
            detail: "artifact_manifest 内存在与 package.analysis_date 不一致的条目。".to_string(),
            suggested_fix: "统一 artifact_manifest 的 analysis_date".to_string(),
        });
    }

    push_check(
        &mut checks,
        "symbol_consistent",
        symbol_aligned,
        "检查 manifest 条目的 symbol 是否与 package.symbol 一致。",
    );
    if !symbol_aligned {
        issues.push(SecurityDecisionVerifyPackageIssue {
            severity: "medium".to_string(),
            code: "symbol_misaligned".to_string(),
            detail: "artifact_manifest 内存在与 package.symbol 不一致的条目。".to_string(),
            suggested_fix: "统一 artifact_manifest 的 symbol".to_string(),
        });
    }

    let verification_status = if issues.is_empty() {
        "passed"
    } else {
        "failed"
    };
    let verification_summary = if issues.is_empty() {
        "decision package 校验通过，会后结论与投后复盘都已完成正式挂接。".to_string()
    } else {
        format!(
            "decision package 校验失败，共发现 {} 个问题，需要先修补后再作为后续治理输入。",
            issues.len()
        )
    };

    SecurityDecisionVerifyPackageResult {
        verification_status: verification_status.to_string(),
        checks,
        issues,
        verification_summary,
    }
}

fn push_check(
    checks: &mut Vec<SecurityDecisionVerifyPackageCheck>,
    name: &str,
    passed: bool,
    detail: &str,
) {
    checks.push(SecurityDecisionVerifyPackageCheck {
        check_name: name.to_string(),
        status: if passed { "passed" } else { "failed" }.to_string(),
        detail: detail.to_string(),
    });
}

fn string_field(value: &Value, field_name: &str) -> String {
    value
        .get(field_name)
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string()
}

fn nested_string_field(value: &Value, path: &[&str]) -> String {
    let mut current = value;
    for segment in path {
        let Some(next) = current.get(*segment) else {
            return String::new();
        };
        current = next;
    }
    current.as_str().unwrap_or_default().to_string()
}

fn object_graph_contains(package: &Value, object_type: &str, object_id: Option<&str>) -> bool {
    package
        .get("object_graph")
        .and_then(Value::as_array)
        .map(|items| {
            items.iter().any(|item| {
                item.get("object_type").and_then(Value::as_str) == Some(object_type)
                    && object_id
                        .filter(|expected| !expected.is_empty())
                        .map(|expected| {
                            item.get("object_id").and_then(Value::as_str) == Some(expected)
                        })
                        .unwrap_or(true)
            })
        })
        .unwrap_or(false)
}

fn manifest_contains(package: &Value, document_type: &str, artifact_id: Option<&str>) -> bool {
    package
        .get("artifact_manifest")
        .and_then(Value::as_array)
        .map(|items| {
            items.iter().any(|item| {
                item.get("document_type").and_then(Value::as_str) == Some(document_type)
                    && artifact_id
                        .filter(|expected| !expected.is_empty())
                        .map(|expected| {
                            item.get("artifact_id").and_then(Value::as_str) == Some(expected)
                        })
                        .unwrap_or(true)
            })
        })
        .unwrap_or(false)
}

fn manifest_dates_align(package: &Value, expected_analysis_date: &str) -> bool {
    package
        .get("artifact_manifest")
        .and_then(Value::as_array)
        .map(|items| {
            items.iter().all(|item| {
                item.get("analysis_date").and_then(Value::as_str) == Some(expected_analysis_date)
            })
        })
        .unwrap_or(false)
}

fn manifest_symbols_align(package: &Value, expected_symbol: &str) -> bool {
    package
        .get("artifact_manifest")
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .all(|item| item.get("symbol").and_then(Value::as_str) == Some(expected_symbol))
        })
        .unwrap_or(false)
}

fn post_trade_review_refs_align(package: &Value) -> bool {
    let Some(post_trade_review) = package.get("post_trade_review") else {
        return false;
    };
    let expected_position_plan_ref = nested_string_field(
        package,
        &[
            "post_trade_review_result",
            "position_plan_result",
            "position_plan_document",
            "position_plan_id",
        ],
    );
    let expected_snapshot_ref = nested_string_field(
        package,
        &[
            "post_trade_review_result",
            "forward_outcome_result",
            "snapshot",
            "snapshot_id",
        ],
    );
    let expected_outcome_ref = nested_string_field(
        package,
        &[
            "post_trade_review_result",
            "forward_outcome_result",
            "selected_outcome",
            "outcome_id",
        ],
    );

    !expected_position_plan_ref.is_empty()
        && !expected_snapshot_ref.is_empty()
        && !expected_outcome_ref.is_empty()
        && post_trade_review
            .get("position_plan_ref")
            .and_then(Value::as_str)
            == Some(expected_position_plan_ref.as_str())
        && post_trade_review
            .get("snapshot_ref")
            .and_then(Value::as_str)
            == Some(expected_snapshot_ref.as_str())
        && post_trade_review.get("outcome_ref").and_then(Value::as_str)
            == Some(expected_outcome_ref.as_str())
}

fn execution_record_refs_align(package: &Value) -> bool {
    let Some(post_trade_review) = package.get("post_trade_review") else {
        return false;
    };
    let Some(execution_record) = package.get("execution_record") else {
        return false;
    };
    let expected_execution_record_ref =
        nested_string_field(package, &["execution_record", "execution_record_id"]);
    let expected_position_plan_ref =
        nested_string_field(package, &["post_trade_review", "position_plan_ref"]);
    let expected_snapshot_ref =
        nested_string_field(package, &["post_trade_review", "snapshot_ref"]);
    let expected_outcome_ref = nested_string_field(package, &["post_trade_review", "outcome_ref"]);

    !expected_execution_record_ref.is_empty()
        && !expected_position_plan_ref.is_empty()
        && !expected_snapshot_ref.is_empty()
        && !expected_outcome_ref.is_empty()
        && post_trade_review
            .get("execution_record_ref")
            .and_then(Value::as_str)
            == Some(expected_execution_record_ref.as_str())
        && execution_record
            .get("position_plan_ref")
            .and_then(Value::as_str)
            == Some(expected_position_plan_ref.as_str())
        && execution_record.get("snapshot_ref").and_then(Value::as_str)
            == Some(expected_snapshot_ref.as_str())
        && execution_record.get("outcome_ref").and_then(Value::as_str)
            == Some(expected_outcome_ref.as_str())
}

fn execution_journal_refs_align(package: &Value) -> bool {
    let Some(post_trade_review) = package.get("post_trade_review") else {
        return false;
    };
    let Some(execution_journal) = package.get("execution_journal") else {
        return false;
    };
    let Some(execution_record) = package.get("execution_record") else {
        return false;
    };
    let expected_execution_journal_ref =
        nested_string_field(package, &["execution_journal", "execution_journal_id"]);
    let expected_position_plan_ref =
        nested_string_field(package, &["post_trade_review", "position_plan_ref"]);
    let expected_snapshot_ref =
        nested_string_field(package, &["post_trade_review", "snapshot_ref"]);
    let expected_outcome_ref = nested_string_field(package, &["post_trade_review", "outcome_ref"]);

    !expected_execution_journal_ref.is_empty()
        && !expected_position_plan_ref.is_empty()
        && !expected_snapshot_ref.is_empty()
        && !expected_outcome_ref.is_empty()
        && post_trade_review
            .get("execution_journal_ref")
            .and_then(Value::as_str)
            == Some(expected_execution_journal_ref.as_str())
        && execution_record
            .get("execution_journal_ref")
            .and_then(Value::as_str)
            == Some(expected_execution_journal_ref.as_str())
        && execution_journal
            .get("position_plan_ref")
            .and_then(Value::as_str)
            == Some(expected_position_plan_ref.as_str())
        && execution_journal
            .get("snapshot_ref")
            .and_then(Value::as_str)
            == Some(expected_snapshot_ref.as_str())
        && execution_journal.get("outcome_ref").and_then(Value::as_str)
            == Some(expected_outcome_ref.as_str())
}
