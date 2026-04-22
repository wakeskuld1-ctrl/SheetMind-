use excel_skill::ops::betting_optimizer::{
    BettingAdjustmentEntry, BettingOptimizerEntry, BettingOptimizerRequest,
    BettingOptimizerSolution, build_optimizer_copy_text, build_optimizer_copy_texts,
    build_optimizer_summary, evaluate_current_metrics, solve_betting_adjustment,
};

fn numbered_entries(stakes: &[i64]) -> Vec<BettingOptimizerEntry> {
    stakes
        .iter()
        .enumerate()
        .map(|(index, stake)| BettingOptimizerEntry::new(format!("{:02}", index + 1), *stake))
        .collect::<Vec<_>>()
}

fn sample_request() -> BettingOptimizerRequest {
    sample_request_with_limit(1500.0, 19)
}

fn sample_request_with_limit(
    max_loss_limit: f64,
    loss_count_target: i64,
) -> BettingOptimizerRequest {
    BettingOptimizerRequest::new(
        vec![
            BettingOptimizerEntry::new("01", 40),
            BettingOptimizerEntry::new("02", 80),
            BettingOptimizerEntry::new("03", 50),
            BettingOptimizerEntry::new("04", 70),
            BettingOptimizerEntry::new("05", 70),
            BettingOptimizerEntry::new("06", 70),
            BettingOptimizerEntry::new("07", 50),
            BettingOptimizerEntry::new("08", 35),
            BettingOptimizerEntry::new("09", 10),
            BettingOptimizerEntry::new("10", 15),
            BettingOptimizerEntry::new("11", 50),
            BettingOptimizerEntry::new("12", 45),
            BettingOptimizerEntry::new("13", 70),
            BettingOptimizerEntry::new("14", 50),
            BettingOptimizerEntry::new("15", 60),
            BettingOptimizerEntry::new("16", 70),
            BettingOptimizerEntry::new("17", 80),
            BettingOptimizerEntry::new("18", 70),
            BettingOptimizerEntry::new("19", 50),
            BettingOptimizerEntry::new("20", 15),
            BettingOptimizerEntry::new("21", 60),
            BettingOptimizerEntry::new("22", 15),
            BettingOptimizerEntry::new("23", 50),
            BettingOptimizerEntry::new("24", 45),
            BettingOptimizerEntry::new("25", 70),
            BettingOptimizerEntry::new("26", 50),
            BettingOptimizerEntry::new("27", 60),
            BettingOptimizerEntry::new("28", 70),
            BettingOptimizerEntry::new("29", 80),
            BettingOptimizerEntry::new("30", 60),
            BettingOptimizerEntry::new("31", 50),
            BettingOptimizerEntry::new("32", 45),
            BettingOptimizerEntry::new("33", 0),
            BettingOptimizerEntry::new("34", 25),
            BettingOptimizerEntry::new("35", 50),
            BettingOptimizerEntry::new("36", 50),
            BettingOptimizerEntry::new("37", 40),
            BettingOptimizerEntry::new("38", 70),
            BettingOptimizerEntry::new("39", 50),
            BettingOptimizerEntry::new("40", 60),
            BettingOptimizerEntry::new("41", 50),
            BettingOptimizerEntry::new("42", 60),
            BettingOptimizerEntry::new("43", 50),
            BettingOptimizerEntry::new("44", 55),
            BettingOptimizerEntry::new("45", 0),
            BettingOptimizerEntry::new("46", 45),
            BettingOptimizerEntry::new("47", 50),
            BettingOptimizerEntry::new("48", 40),
            BettingOptimizerEntry::new("49", 40),
        ],
        47.0,
        0.02,
        max_loss_limit,
        loss_count_target,
    )
}

fn selective_adjustment_request() -> BettingOptimizerRequest {
    BettingOptimizerRequest::new(
        vec![
            BettingOptimizerEntry::new("01", 80),
            BettingOptimizerEntry::new("02", 50),
            BettingOptimizerEntry::new("03", 2),
        ],
        47.0,
        0.02,
        3500.0,
        2,
    )
}

fn selective_request_with_manual_lock(locked_stake: i64) -> BettingOptimizerRequest {
    BettingOptimizerRequest::new(
        vec![
            BettingOptimizerEntry::with_manual_constraints("01", 80, Some(locked_stake), None),
            BettingOptimizerEntry::new("02", 50),
            BettingOptimizerEntry::new("03", 2),
        ],
        47.0,
        0.02,
        3500.0,
        2,
    )
}

fn selective_request_with_refund_cap(refund_cap: i64) -> BettingOptimizerRequest {
    BettingOptimizerRequest::new(
        vec![
            BettingOptimizerEntry::with_manual_constraints("01", 80, None, Some(refund_cap)),
            BettingOptimizerEntry::new("02", 50),
            BettingOptimizerEntry::new("03", 2),
        ],
        47.0,
        0.02,
        3500.0,
        2,
    )
}

fn concentrated_request(
    concentrated_stake: i64,
    entry_count: usize,
    max_loss_limit: f64,
    loss_count_target: i64,
) -> BettingOptimizerRequest {
    let mut stakes = vec![0; entry_count];
    stakes[0] = concentrated_stake;
    BettingOptimizerRequest::new(
        numbered_entries(&stakes),
        47.0,
        0.02,
        max_loss_limit,
        loss_count_target,
    )
}

fn uniform_request(
    stake: i64,
    entry_count: usize,
    max_loss_limit: f64,
    loss_count_target: i64,
) -> BettingOptimizerRequest {
    BettingOptimizerRequest::new(
        numbered_entries(&vec![stake; entry_count]),
        47.0,
        0.02,
        max_loss_limit,
        loss_count_target,
    )
}

fn low_loss_average_request() -> BettingOptimizerRequest {
    let mut stakes = vec![60; 10];
    stakes.extend(vec![50; 39]);
    BettingOptimizerRequest::new(numbered_entries(&stakes), 47.0, 0.02, 1500.0, 19)
}

fn sparse_high_risk_request() -> BettingOptimizerRequest {
    let mut stakes = vec![200; 30];
    stakes.extend(vec![0; 19]);
    BettingOptimizerRequest::new(numbered_entries(&stakes), 47.0, 0.02, 1500.0, 19)
}

fn brute_force_best_solution_signature(
    request: &BettingOptimizerRequest,
) -> Option<(i64, i64, usize)> {
    // 2026-04-21 CST: Keep a tiny exhaustive oracle in tests because the user
    // explicitly asked us to guard against "first solution" regressions, and a
    // 4-entry brute-force case is cheap enough to validate true optimality.
    fn dfs(
        request: &BettingOptimizerRequest,
        index: usize,
        stakes: &mut Vec<i64>,
        best: &mut Option<(i64, i64, usize)>,
    ) {
        if index == request.entries.len() {
            let metrics = evaluate_current_metrics(&BettingOptimizerRequest::new(
                stakes
                    .iter()
                    .enumerate()
                    .map(|(position, stake)| {
                        BettingOptimizerEntry::new(format!("{:02}", position + 1), *stake)
                    })
                    .collect::<Vec<_>>(),
                request.payout_multiplier,
                request.rebate_rate,
                request.max_loss_limit,
                request.loss_count_target,
            ))
            .unwrap();
            if metrics.max_loss > request.max_loss_limit + 1e-9 {
                return;
            }

            let total_adjusted = metrics.total_stake;
            let gap = (metrics.loss_count as i64 - request.loss_count_target).abs();
            let signature = (gap, -total_adjusted, metrics.loss_count);

            if best
                .as_ref()
                .map(|current| signature < *current)
                .unwrap_or(true)
            {
                *best = Some(signature);
            }
            return;
        }

        let original = request.entries[index].original_stake;
        for adjusted in 0..=original {
            stakes[index] = adjusted;
            dfs(request, index + 1, stakes, best);
        }
    }

    let mut working = request
        .entries
        .iter()
        .map(|entry| entry.original_stake)
        .collect::<Vec<_>>();
    let mut best = None;
    dfs(request, 0, &mut working, &mut best);
    best.map(|(gap, negative_total, loss_count)| (-negative_total, gap, loss_count))
}

// 2026-04-21 CST: Lock the live customer workbook inputs into a regression
// fixture because the current solver is misreporting "no feasible solution"
// on this exact field case and we need a stable RED test before any fix.
fn customer_regression_request() -> BettingOptimizerRequest {
    BettingOptimizerRequest::new(
        vec![
            BettingOptimizerEntry::new("01", 12),
            BettingOptimizerEntry::new("02", 12),
            BettingOptimizerEntry::new("03", 221),
            BettingOptimizerEntry::new("04", 231),
            BettingOptimizerEntry::new("05", 312),
            BettingOptimizerEntry::new("06", 221),
            BettingOptimizerEntry::new("07", 223),
            BettingOptimizerEntry::new("08", 221),
            BettingOptimizerEntry::new("09", 221),
            BettingOptimizerEntry::new("10", 321),
            BettingOptimizerEntry::new("11", 321),
            BettingOptimizerEntry::new("12", 412),
            BettingOptimizerEntry::new("13", 43),
            BettingOptimizerEntry::new("14", 22),
            BettingOptimizerEntry::new("15", 221),
            BettingOptimizerEntry::new("16", 321),
            BettingOptimizerEntry::new("17", 221),
            BettingOptimizerEntry::new("18", 221),
            BettingOptimizerEntry::new("19", 421),
            BettingOptimizerEntry::new("20", 212),
            BettingOptimizerEntry::new("21", 334),
            BettingOptimizerEntry::new("22", 321),
            BettingOptimizerEntry::new("23", 221),
            BettingOptimizerEntry::new("24", 123),
            BettingOptimizerEntry::new("25", 32),
            BettingOptimizerEntry::new("26", 32),
            BettingOptimizerEntry::new("27", 12),
            BettingOptimizerEntry::new("28", 321),
            BettingOptimizerEntry::new("29", 221),
            BettingOptimizerEntry::new("30", 221),
            BettingOptimizerEntry::new("31", 321),
            BettingOptimizerEntry::new("32", 221),
            BettingOptimizerEntry::new("33", 331),
            BettingOptimizerEntry::new("34", 423),
            BettingOptimizerEntry::new("35", 212),
            BettingOptimizerEntry::new("36", 212),
            BettingOptimizerEntry::new("37", 121),
            BettingOptimizerEntry::new("38", 231),
            BettingOptimizerEntry::new("39", 32),
            BettingOptimizerEntry::new("40", 21),
            BettingOptimizerEntry::new("41", 221),
            BettingOptimizerEntry::new("42", 221),
            BettingOptimizerEntry::new("43", 221),
            BettingOptimizerEntry::new("44", 221),
            BettingOptimizerEntry::new("45", 312),
            BettingOptimizerEntry::new("46", 221),
            BettingOptimizerEntry::new("47", 212),
            BettingOptimizerEntry::new("48", 441),
            BettingOptimizerEntry::new("49", 121),
        ],
        47.0,
        0.02,
        1500.0,
        19,
    )
}

#[test]
fn betting_optimizer_request_keeps_integer_entry_contract() {
    let request = sample_request();

    assert_eq!(request.loss_count_target, 19);
    assert_eq!(request.entries.len(), 49);
    assert_eq!(request.entries[0].original_stake, 40);
    assert_eq!(request.entries[1].label, "02");
}

#[test]
fn betting_metrics_match_current_workbook_rules() {
    let request = sample_request();

    let metrics = evaluate_current_metrics(&request).unwrap();

    assert_eq!(metrics.total_stake, 2440);
    assert!((metrics.rebate - 48.8).abs() < 1e-9);
    assert!((metrics.payable_principal - 2391.2).abs() < 1e-9);
    assert!((metrics.max_loss - 1368.8).abs() < 1e-9);
    assert_eq!(metrics.loss_count, 19);
}

#[test]
fn optimizer_only_refunds_current_risk_numbers_on_full_sample() {
    let request = sample_request_with_limit(1000.0, 19);

    let solution = solve_betting_adjustment(&request).unwrap();

    assert_eq!(solution.total_adjusted_stake, 2413);
    assert_eq!(solution.total_refund, 27);
    assert!(solution.max_loss <= 1000.0);
    assert_eq!(solution.loss_count, 19);
    assert_eq!(solution.loss_count_gap, 0);
    assert_eq!(solution.entries[1].adjusted_stake, 71);
    assert_eq!(solution.entries[16].adjusted_stake, 71);
    assert_eq!(solution.entries[28].adjusted_stake, 71);
    assert_eq!(solution.entries[8].adjusted_stake, 10);
    assert_eq!(solution.entries[9].adjusted_stake, 15);
    assert_eq!(solution.entries[32].adjusted_stake, 0);
}

#[test]
fn optimizer_only_refunds_current_risk_numbers() {
    let request = selective_adjustment_request();

    let solution = solve_betting_adjustment(&request).unwrap();

    assert_eq!(solution.total_adjusted_stake, 129);
    assert_eq!(solution.total_refund, 3);
    assert!(solution.max_loss <= 3500.0);
    assert_eq!(solution.loss_count, 2);
    assert_eq!(solution.loss_count_gap, 0);
    assert_eq!(solution.entries[0].adjusted_stake, 77);
    assert_eq!(solution.entries[1].adjusted_stake, 50);
    assert_eq!(solution.entries[2].adjusted_stake, 2);
}

#[test]
fn optimizer_summary_mentions_limit_refund_and_focus_numbers() {
    let request = sample_request_with_limit(1000.0, 19);
    let current_metrics = evaluate_current_metrics(&request).unwrap();
    let solution = solve_betting_adjustment(&request).unwrap();

    let summary = build_optimizer_summary(&request, &current_metrics, &solution);

    assert!(summary.contains("1000"));
    assert!(summary.contains("27"));
    assert!(summary.contains("19"));
    assert!(!summary.contains("02"));
}

#[test]
fn optimizer_copy_text_uses_refund_amount_language_for_adjusted_numbers() {
    let request = sample_request_with_limit(1000.0, 19);
    let solution = solve_betting_adjustment(&request).unwrap();

    let copy_text = build_optimizer_copy_text(&solution);

    assert!(copy_text.starts_with("重点下调建议："));
    assert!(copy_text.contains("02打"));
    assert!(copy_text.contains("17打"));
    assert!(copy_text.contains("29打"));
    assert!(!copy_text.contains("调整后最大亏损"));
}

#[test]
fn optimizer_copy_text_reports_no_adjustment_when_solution_has_no_refund() {
    let request = uniform_request(30, 49, 1500.0, 0);
    let solution = solve_betting_adjustment(&request).unwrap();

    let copy_text = build_optimizer_copy_text(&solution);

    assert_eq!(copy_text, "本轮无需下调，保持当前方案即可。");
}

#[test]
fn optimizer_copy_texts_split_original_excess_small_and_large_groups() {
    // 2026-04-22 CST: Freeze the new copy-box wording because the user now
    // needs three additional operator-friendly descriptions without changing
    // solver math or the underlying refund amounts.
    let solution = BettingOptimizerSolution {
        total_original_stake: 0,
        total_adjusted_stake: 0,
        total_refund: 204,
        rebate: 0.0,
        payable_principal: 0.0,
        max_loss: 0.0,
        loss_count: 19,
        loss_count_gap: 0,
        constraint_limited: false,
        entries: vec![
            BettingAdjustmentEntry {
                label: "01".to_string(),
                original_stake: 100,
                adjusted_stake: 67,
                refund_amount: 33,
                payout_amount: 0.0,
                pnl_value: 0.0,
                is_loss_number: true,
            },
            BettingAdjustmentEntry {
                label: "02".to_string(),
                original_stake: 100,
                adjusted_stake: 52,
                refund_amount: 48,
                payout_amount: 0.0,
                pnl_value: 0.0,
                is_loss_number: true,
            },
            BettingAdjustmentEntry {
                label: "03".to_string(),
                original_stake: 100,
                adjusted_stake: 72,
                refund_amount: 28,
                payout_amount: 0.0,
                pnl_value: 0.0,
                is_loss_number: true,
            },
            BettingAdjustmentEntry {
                label: "08".to_string(),
                original_stake: 100,
                adjusted_stake: 97,
                refund_amount: 3,
                payout_amount: 0.0,
                pnl_value: 0.0,
                is_loss_number: true,
            },
            BettingAdjustmentEntry {
                label: "11".to_string(),
                original_stake: 100,
                adjusted_stake: 70,
                refund_amount: 30,
                payout_amount: 0.0,
                pnl_value: 0.0,
                is_loss_number: true,
            },
            BettingAdjustmentEntry {
                label: "49".to_string(),
                original_stake: 100,
                adjusted_stake: 35,
                refund_amount: 65,
                payout_amount: 0.0,
                pnl_value: 0.0,
                is_loss_number: true,
            },
        ],
    };

    let copy_texts = build_optimizer_copy_texts(&solution);

    assert_eq!(
        copy_texts.original,
        "重点下调建议：01打33，02打48，03打28，08打3，11打30，49打65。"
    );
    assert_eq!(
        copy_texts.large_overage,
        "大于30净额建议：01打3，02打18，49打35。"
    );
    assert_eq!(
        copy_texts.small_group,
        "小于等于30归类：03打28，08打3，11打30。"
    );
    assert_eq!(
        copy_texts.large_group,
        "大于30归类：01打33，02打48，49打65。"
    );
}

#[test]
fn optimizer_honors_manual_locked_stake_when_recomputing() {
    let request = selective_request_with_manual_lock(76);

    let solution = solve_betting_adjustment(&request).unwrap();

    assert_eq!(solution.entries[0].adjusted_stake, 76);
    assert!(!solution.constraint_limited);
}

#[test]
fn optimizer_returns_constraint_limited_result_when_refund_cap_blocks_required_refund() {
    let request = selective_request_with_refund_cap(2);

    let solution = solve_betting_adjustment(&request).unwrap();

    assert_eq!(solution.entries[0].adjusted_stake, 80);
    assert!(solution.constraint_limited);
    assert!(solution.max_loss > 3500.0);
}

#[test]
fn optimizer_handles_single_number_concentration_boundary() {
    let request = concentrated_request(1000, 49, 1500.0, 19);

    let solution = solve_betting_adjustment(&request).unwrap();

    assert_eq!(solution.total_adjusted_stake, 32);
    assert_eq!(solution.total_refund, 968);
    assert!(solution.max_loss <= 1500.0);
    assert_eq!(solution.loss_count, 1);
    assert_eq!(solution.loss_count_gap, 18);
    assert_eq!(solution.entries[0].adjusted_stake, 32);
    assert!(solution.entries.iter().skip(1).all(|entry| entry.adjusted_stake == 0));
}

#[test]
fn optimizer_returns_zero_refund_for_uniform_request_when_target_is_already_met() {
    let request = uniform_request(30, 49, 1500.0, 0);

    let current_metrics = evaluate_current_metrics(&request).unwrap();
    let solution = solve_betting_adjustment(&request).unwrap();

    assert_eq!(current_metrics.loss_count, 0);
    assert_eq!(current_metrics.max_loss, 0.0);
    assert_eq!(solution.total_refund, 0);
    assert_eq!(solution.total_adjusted_stake, current_metrics.total_stake);
    assert_eq!(solution.loss_count, 0);
    assert_eq!(solution.loss_count_gap, 0);
}

#[test]
fn optimizer_can_increase_loss_count_from_low_loss_average_distribution() {
    let request = low_loss_average_request();

    let current_metrics = evaluate_current_metrics(&request).unwrap();
    let solution = solve_betting_adjustment(&request).unwrap();

    assert!(current_metrics.loss_count < 19);
    assert!(solution.max_loss <= 1500.0);
    assert_eq!(solution.loss_count, 19);
    assert_eq!(solution.loss_count_gap, 0);
    assert_eq!(solution.total_adjusted_stake, 2397);
}

#[test]
fn optimizer_handles_sparse_high_risk_distribution_with_many_zero_rows() {
    let request = sparse_high_risk_request();

    let current_metrics = evaluate_current_metrics(&request).unwrap();
    let solution = solve_betting_adjustment(&request).unwrap();

    assert!(current_metrics.loss_count > 29);
    assert!(solution.max_loss <= 1500.0);
    assert_eq!(solution.loss_count, 19);
    assert_eq!(solution.loss_count_gap, 0);
    assert_eq!(solution.total_adjusted_stake, 1598);
    assert!(solution
        .entries
        .iter()
        .skip(30)
        .all(|entry| entry.original_stake == 0 && entry.adjusted_stake == 0));
}

#[test]
fn optimizer_matches_bruteforce_optimum_on_small_boundary_case() {
    let request = BettingOptimizerRequest::new(
        numbered_entries(&[6, 5, 4, 1]),
        47.0,
        0.02,
        120.0,
        2,
    );

    let solution = solve_betting_adjustment(&request).unwrap();
    let brute_force =
        brute_force_best_solution_signature(&request).expect("expected brute-force solution");

    assert_eq!(
        (solution.total_adjusted_stake, solution.loss_count_gap, solution.loss_count),
        brute_force
    );
}

#[test]
fn optimizer_finds_exact_target_on_customer_regression_case() {
    let request = customer_regression_request();

    let current_metrics = evaluate_current_metrics(&request).unwrap();

    assert_eq!(current_metrics.total_stake, 10564);
    assert_eq!(current_metrics.loss_count, 33);

    let solution = solve_betting_adjustment(&request).unwrap();

    assert!(solution.max_loss <= 1500.0);
    assert_eq!(solution.loss_count, 19);
    assert_eq!(solution.loss_count_gap, 0);
    assert_eq!(solution.total_adjusted_stake, 4946);
    assert_eq!(solution.total_refund, 5618);
    assert!(solution
        .entries
        .iter()
        .all(|entry| entry.adjusted_stake <= entry.original_stake));
}

#[test]
fn optimizer_rejects_refund_cap_on_non_risk_number() {
    let request = BettingOptimizerRequest::new(
        vec![
            BettingOptimizerEntry::new("01", 80),
            BettingOptimizerEntry::new("02", 50),
            BettingOptimizerEntry::with_manual_constraints("03", 2, None, Some(1)),
        ],
        47.0,
        0.02,
        3500.0,
        2,
    );

    let error = match solve_betting_adjustment(&request) {
        Ok(_) => panic!("expected non-risk refund cap to be rejected"),
        Err(error) => error,
    };

    assert!(error.to_string().contains("非风险号码"));
}
