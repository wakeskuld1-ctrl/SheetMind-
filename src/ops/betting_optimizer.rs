use thiserror::Error;

// 2026-04-20 CST: Add an explicit entry type for the betting optimizer because
// the workbook contract now needs one stable Rust-side representation of each
// number and its original integer stake before we introduce solving logic.
pub struct BettingOptimizerEntry {
    pub label: String,
    pub original_stake: i64,
    pub manual_locked_stake: Option<i64>,
    pub manual_refund_cap: Option<i64>,
}

impl BettingOptimizerEntry {
    pub fn new(label: impl Into<String>, original_stake: i64) -> Self {
        Self {
            label: label.into(),
            original_stake,
            manual_locked_stake: None,
            manual_refund_cap: None,
        }
    }

    // 2026-04-21 CST: Extend the entry contract with optional per-number
    // manual constraints because result-sheet re-solve must carry operator
    // lock/refund-cap intent into the Rust solver without inventing side maps.
    pub fn with_manual_constraints(
        label: impl Into<String>,
        original_stake: i64,
        manual_locked_stake: Option<i64>,
        manual_refund_cap: Option<i64>,
    ) -> Self {
        Self {
            label: label.into(),
            original_stake,
            manual_locked_stake,
            manual_refund_cap,
        }
    }
}

// 2026-04-20 CST: Add a first-class request object because the optimizer math,
// workbook bridge, and future solver binary all need to consume the same
// validated contract instead of passing raw tuples around.
pub struct BettingOptimizerRequest {
    pub entries: Vec<BettingOptimizerEntry>,
    pub payout_multiplier: f64,
    pub rebate_rate: f64,
    pub max_loss_limit: f64,
    pub loss_count_target: i64,
}

impl BettingOptimizerRequest {
    pub fn new(
        entries: Vec<BettingOptimizerEntry>,
        payout_multiplier: f64,
        rebate_rate: f64,
        max_loss_limit: f64,
        loss_count_target: i64,
    ) -> Self {
        Self {
            entries,
            payout_multiplier,
            rebate_rate,
            max_loss_limit,
            loss_count_target,
        }
    }
}

// 2026-04-20 CST: Add outcome-level metrics because the existing workbook logic
// depends on per-number pnl values, and later optimization/reporting layers
// will need to reuse the same evaluated state without recalculating ad hoc.
pub struct BettingOutcomeMetrics {
    pub label: String,
    pub original_stake: i64,
    pub payout_amount: f64,
    pub pnl_value: f64,
}

// 2026-04-20 CST: Add an explicit adjusted-entry view because the solver output,
// workbook suggestion sheet, and acceptance checks all need one traceable record
// per number after optimization rather than re-deriving refund/pnl ad hoc.
pub struct BettingAdjustmentEntry {
    pub label: String,
    pub original_stake: i64,
    pub adjusted_stake: i64,
    pub refund_amount: i64,
    pub payout_amount: f64,
    pub pnl_value: f64,
    pub is_loss_number: bool,
}

// 2026-04-20 CST: Add an aggregate metrics view because current-sheet summary
// cells and acceptance checks both need one shared source of truth for total
// stake, rebate, payable principal, max loss, and loss-number count.
pub struct BettingMetrics {
    pub total_stake: i64,
    pub rebate: f64,
    pub payable_principal: f64,
    pub max_loss: f64,
    pub loss_count: usize,
    pub outcomes: Vec<BettingOutcomeMetrics>,
}

// 2026-04-20 CST: Add a first-class solution object because the solver must
// return both the lexicographic optimization result and the row-level adjusted
// stakes that the second workbook sheet will render verbatim.
pub struct BettingOptimizerSolution {
    pub total_original_stake: i64,
    pub total_adjusted_stake: i64,
    pub total_refund: i64,
    pub rebate: f64,
    pub payable_principal: f64,
    pub max_loss: f64,
    pub loss_count: usize,
    pub loss_count_gap: i64,
    pub constraint_limited: bool,
    pub entries: Vec<BettingAdjustmentEntry>,
}

#[derive(Debug, Error)]
pub enum BettingOptimizerError {
    #[error("betting optimizer input cannot be empty")]
    EmptyEntries,
    #[error("betting optimizer stake cannot be negative: {0}")]
    NegativeStake(String),
    #[error("betting optimizer payout multiplier must be positive")]
    InvalidPayoutMultiplier,
    #[error("betting optimizer rebate rate must be in [0, 1)")]
    InvalidRebateRate,
    #[error("betting optimizer max loss limit must be non-negative")]
    InvalidMaxLossLimit,
    #[error("betting optimizer manual constraint is invalid: {0}")]
    InvalidManualConstraint(String),
    #[error("betting optimizer could not find a feasible integer solution")]
    NoFeasibleSolution,
}

// 2026-04-20 CST: Freeze the current workbook math in one function now,
// because every later optimization and workbook-output step must reuse the
// exact same payout/rebate/principal semantics the user already validated.
pub fn evaluate_current_metrics(
    request: &BettingOptimizerRequest,
) -> Result<BettingMetrics, BettingOptimizerError> {
    validate_request(request)?;

    let stakes = request
        .entries
        .iter()
        .map(|entry| entry.original_stake)
        .collect::<Vec<_>>();

    evaluate_metrics_for_stakes(request, &stakes)
}

// 2026-04-20 CST: Add the exact solver now because the approved delivery
// contract requires integer-only downward adjustments with lexicographic
// objectives, and this closed-form search avoids introducing fragile MILP deps.
pub fn solve_betting_adjustment(
    request: &BettingOptimizerRequest,
) -> Result<BettingOptimizerSolution, BettingOptimizerError> {
    let _ = validate_request(request)?;
    let effective_request = build_effective_request_with_locked_stakes(request);
    let current_metrics = evaluate_current_metrics(&effective_request)?;
    validate_manual_constraints(request, &current_metrics)?;
    // 2026-04-21 CST: Stop freezing the adjustable set to only the initially
    // risky numbers, because once total stake moves down a previously safe row
    // can become a new risk row and must remain available for downward tuning.
    let adjustable_mask = effective_request
        .entries
        .iter()
        .map(|entry| entry.original_stake > 0 && entry.manual_locked_stake.is_none())
        .collect::<Vec<_>>();
    let target_solution = solve_target_feasible_request(&effective_request, &adjustable_mask);

    match target_solution {
        Ok(solution) if refund_caps_are_respected(request, &solution) => Ok(solution),
        Ok(_) | Err(BettingOptimizerError::NoFeasibleSolution) if request_has_manual_constraints(request) => {
            build_constraint_limited_solution(&effective_request)
        }
        Ok(solution) => Ok(solution),
        Err(error) => Err(error),
    }
}

// 2026-04-20 CST: Add one stable summary builder because the workbook
// suggestion sheet must render a governed Chinese explanation instead of
// letting each caller improvise different business wording.
pub fn build_optimizer_summary(
    request: &BettingOptimizerRequest,
    current_metrics: &BettingMetrics,
    solution: &BettingOptimizerSolution,
) -> String {
    let focus_numbers = solution
        .entries
        .iter()
        .filter(|entry| entry.refund_amount > 0)
        .collect::<Vec<_>>();

    if focus_numbers.is_empty() {
        if solution.constraint_limited {
            return format!(
                "受人工约束影响，目标最大亏损{}本轮未完全达成。当前约束下给出最接近结果：总退款{}，最大亏损{}，亏损号码数为{}。",
                format_decimal(request.max_loss_limit),
                solution.total_refund,
                format_decimal(solution.max_loss),
                solution.loss_count
            );
        }
        return format!(
            "当前方案最大亏损为{}，已不高于目标{}。在仅允许下调当前风险号码且保持整数下注的前提下，本次无需退款，亏损号码数保持为{}。",
            format_decimal(current_metrics.max_loss),
            format_decimal(request.max_loss_limit),
            solution.loss_count
        );
    }

    let focus_labels = focus_numbers
        .iter()
        .map(|entry| entry.label.clone())
        .collect::<Vec<_>>()
        .join("、");

    format!(
        "{}当前方案最大亏损为{}，目标为{}。在仅允许下调当前风险号码且保持整数下注的前提下，建议总退款{}，调整后最大亏损{}，亏损号码数调整为{}。重点下调号码为：{}。",
        if solution.constraint_limited {
            "受人工约束影响，本轮结果为最接近目标的可行结果。"
        } else {
            ""
        },
        format_decimal(current_metrics.max_loss),
        format_decimal(request.max_loss_limit),
        solution.total_refund,
        format_decimal(solution.max_loss),
        solution.loss_count,
        focus_labels
    )
}

fn validate_request(request: &BettingOptimizerRequest) -> Result<i64, BettingOptimizerError> {
    if request.entries.is_empty() {
        return Err(BettingOptimizerError::EmptyEntries);
    }
    if request.payout_multiplier <= 0.0 {
        return Err(BettingOptimizerError::InvalidPayoutMultiplier);
    }
    if !(0.0..1.0).contains(&request.rebate_rate) {
        return Err(BettingOptimizerError::InvalidRebateRate);
    }
    if request.max_loss_limit < 0.0 {
        return Err(BettingOptimizerError::InvalidMaxLossLimit);
    }

    let mut total_stake = 0_i64;
    for entry in &request.entries {
        if entry.original_stake < 0 {
            return Err(BettingOptimizerError::NegativeStake(entry.label.clone()));
        }
        total_stake += entry.original_stake;
    }

    Ok(total_stake)
}

fn build_effective_request_with_locked_stakes(
    request: &BettingOptimizerRequest,
) -> BettingOptimizerRequest {
    let entries = request
        .entries
        .iter()
        .map(|entry| {
            let effective_stake = entry.manual_locked_stake.unwrap_or(entry.original_stake);
            BettingOptimizerEntry::with_manual_constraints(
                entry.label.clone(),
                effective_stake,
                entry.manual_locked_stake,
                entry.manual_refund_cap,
            )
        })
        .collect::<Vec<_>>();

    BettingOptimizerRequest::new(
        entries,
        request.payout_multiplier,
        request.rebate_rate,
        request.max_loss_limit,
        request.loss_count_target,
    )
}

fn validate_manual_constraints(
    request: &BettingOptimizerRequest,
    current_metrics: &BettingMetrics,
) -> Result<(), BettingOptimizerError> {
    for (entry, outcome) in request.entries.iter().zip(current_metrics.outcomes.iter()) {
        if let Some(locked_stake) = entry.manual_locked_stake {
            if locked_stake < 0 || locked_stake > entry.original_stake {
                return Err(BettingOptimizerError::InvalidManualConstraint(format!(
                    "{} 的手工锁定下轮下注额必须在 0 到 {} 之间",
                    entry.label, entry.original_stake
                )));
            }
        }

        if let Some(refund_cap) = entry.manual_refund_cap {
            if refund_cap < 0 || refund_cap > entry.original_stake {
                return Err(BettingOptimizerError::InvalidManualConstraint(format!(
                    "{} 的本轮最多可退款金额必须在 0 到 {} 之间",
                    entry.label, entry.original_stake
                )));
            }
            if outcome.pnl_value <= 0.0 {
                return Err(BettingOptimizerError::InvalidManualConstraint(format!(
                    "{} 是非风险号码，不能设置本轮最多可退款金额",
                    entry.label
                )));
            }
        }
    }

    Ok(())
}

fn request_has_manual_constraints(request: &BettingOptimizerRequest) -> bool {
    request.entries.iter().any(|entry| {
        entry.manual_locked_stake.is_some() || entry.manual_refund_cap.is_some()
    })
}

fn refund_caps_are_respected(
    request: &BettingOptimizerRequest,
    solution: &BettingOptimizerSolution,
) -> bool {
    request
        .entries
        .iter()
        .zip(solution.entries.iter())
        .all(|(request_entry, solved_entry)| {
            request_entry
                .manual_refund_cap
                .map(|refund_cap| solved_entry.refund_amount <= refund_cap)
                .unwrap_or(true)
        })
}

fn solve_target_feasible_request(
    request: &BettingOptimizerRequest,
    adjustable_mask: &[bool],
) -> Result<BettingOptimizerSolution, BettingOptimizerError> {
    let total_original_stake = validate_request(request)?;
    let mut best_solution = None;

    // 2026-04-21 CST: Search every feasible retained-total candidate instead of
    // returning on the first mathematically possible total, because the user
    // now needs the globally best integer solution rather than the earliest one.
    for candidate_total in (0..=total_original_stake).rev() {
        let Some(entry_caps) =
            build_entry_caps_for_total(request, candidate_total, adjustable_mask)
        else {
            continue;
        };

        let Ok(chosen_loss_count) = choose_loss_count_closest_to_target(
            request,
            candidate_total,
            &entry_caps,
            adjustable_mask,
        ) else {
            continue;
        };
        let Ok(adjusted_stakes) = construct_adjusted_stakes(
            candidate_total,
            &entry_caps,
            adjustable_mask,
            chosen_loss_count,
            request,
        ) else {
            continue;
        };
        let candidate_solution = build_solution_from_stakes(request, &adjusted_stakes, false)?;

        if best_solution
            .as_ref()
            .map(|best| is_better_solution(&candidate_solution, best))
            .unwrap_or(true)
        {
            let reached_exact_target = candidate_solution.loss_count_gap == 0;
            best_solution = Some(candidate_solution);
            if reached_exact_target {
                break;
            }
        }
    }

    best_solution.ok_or(BettingOptimizerError::NoFeasibleSolution)
}

fn build_constraint_limited_solution(
    request: &BettingOptimizerRequest,
) -> Result<BettingOptimizerSolution, BettingOptimizerError> {
    let adjusted_stakes = request
        .entries
        .iter()
        .map(|entry| entry.original_stake)
        .collect::<Vec<_>>();

    build_solution_from_stakes(request, &adjusted_stakes, true)
}

fn build_solution_from_stakes(
    request: &BettingOptimizerRequest,
    adjusted_stakes: &[i64],
    constraint_limited: bool,
) -> Result<BettingOptimizerSolution, BettingOptimizerError> {
    let total_original_stake = validate_request(request)?;
    let adjusted_metrics = evaluate_metrics_for_stakes(request, adjusted_stakes)?;
    let entries = request
        .entries
        .iter()
        .zip(adjusted_stakes.iter())
        .zip(adjusted_metrics.outcomes.iter())
        .map(
            |((entry, adjusted_stake), outcome)| BettingAdjustmentEntry {
                label: entry.label.clone(),
                original_stake: entry.original_stake,
                adjusted_stake: *adjusted_stake,
                refund_amount: entry.original_stake - *adjusted_stake,
                payout_amount: outcome.payout_amount,
                pnl_value: outcome.pnl_value,
                is_loss_number: outcome.pnl_value > 0.0,
            },
        )
        .collect::<Vec<_>>();

    Ok(BettingOptimizerSolution {
        total_original_stake,
        total_adjusted_stake: adjusted_metrics.total_stake,
        total_refund: total_original_stake - adjusted_metrics.total_stake,
        rebate: adjusted_metrics.rebate,
        payable_principal: adjusted_metrics.payable_principal,
        max_loss: adjusted_metrics.max_loss,
        loss_count: adjusted_metrics.loss_count,
        loss_count_gap: (adjusted_metrics.loss_count as i64 - request.loss_count_target).abs(),
        constraint_limited,
        entries,
    })
}

fn evaluate_metrics_for_stakes(
    request: &BettingOptimizerRequest,
    stakes: &[i64],
) -> Result<BettingMetrics, BettingOptimizerError> {
    let total_stake = stakes.iter().sum::<i64>();

    let rebate = total_stake as f64 * request.rebate_rate;
    let payable_principal = total_stake as f64 - rebate;

    let outcomes = request
        .entries
        .iter()
        .zip(stakes.iter())
        .map(|entry| {
            let payout_amount = entry.1.to_owned() as f64 * request.payout_multiplier;
            let pnl_value = payout_amount - payable_principal;
            BettingOutcomeMetrics {
                label: entry.0.label.clone(),
                original_stake: *entry.1,
                payout_amount,
                pnl_value,
            }
        })
        .collect::<Vec<_>>();

    let max_loss = outcomes
        .iter()
        .map(|outcome| outcome.pnl_value)
        .fold(f64::NEG_INFINITY, f64::max)
        .max(0.0);
    let loss_count = outcomes
        .iter()
        .filter(|outcome| outcome.pnl_value > 0.0)
        .count();

    Ok(BettingMetrics {
        total_stake,
        rebate,
        payable_principal,
        max_loss,
        loss_count,
        outcomes,
    })
}

fn build_entry_caps_for_total(
    request: &BettingOptimizerRequest,
    candidate_total: i64,
    adjustable_mask: &[bool],
) -> Option<Vec<i64>> {
    let payable_principal = candidate_total as f64 * (1.0 - request.rebate_rate);
    let per_entry_cap = compute_per_entry_cap(request, candidate_total);
    let entry_caps = request
        .entries
        .iter()
        .zip(adjustable_mask.iter())
        .map(|(entry, is_adjustable)| {
            if *is_adjustable {
                entry.original_stake.min(per_entry_cap).max(0)
            } else {
                entry.original_stake
            }
        })
        .collect::<Vec<_>>();
    let total_capacity = entry_caps.iter().sum::<i64>();
    let fixed_entries_are_safe = request
        .entries
        .iter()
        .zip(adjustable_mask.iter())
        .filter(|(_, is_adjustable)| !**is_adjustable)
        .all(|(entry, _)| {
            entry.original_stake as f64 * request.payout_multiplier - payable_principal
                <= request.max_loss_limit + 1e-9
        });

    if fixed_entries_are_safe && total_capacity >= candidate_total {
        Some(entry_caps)
    } else {
        None
    }
}

fn is_better_solution(
    candidate: &BettingOptimizerSolution,
    incumbent: &BettingOptimizerSolution,
) -> bool {
    candidate.loss_count_gap < incumbent.loss_count_gap
        || (candidate.loss_count_gap == incumbent.loss_count_gap
            && candidate.total_adjusted_stake > incumbent.total_adjusted_stake)
        || (candidate.loss_count_gap == incumbent.loss_count_gap
            && candidate.total_adjusted_stake == incumbent.total_adjusted_stake
            && candidate.loss_count < incumbent.loss_count)
}

fn compute_per_entry_cap(request: &BettingOptimizerRequest, total_adjusted_stake: i64) -> i64 {
    let payable_ratio = 1.0 - request.rebate_rate;
    let cap = (request.max_loss_limit + payable_ratio * total_adjusted_stake as f64)
        / request.payout_multiplier;

    (cap + 1e-9).floor() as i64
}

fn choose_loss_count_closest_to_target(
    request: &BettingOptimizerRequest,
    total_adjusted_stake: i64,
    entry_caps: &[i64],
    adjustable_mask: &[bool],
) -> Result<usize, BettingOptimizerError> {
    let loss_trigger = compute_loss_trigger_stake(request, total_adjusted_stake);
    let non_loss_cap = loss_trigger.saturating_sub(1);

    let mut base_non_loss_total = 0_i64;
    let mut candidate_extras = Vec::new();
    let fixed_loss_count = request
        .entries
        .iter()
        .zip(adjustable_mask.iter())
        .filter(|(_, is_adjustable)| !**is_adjustable)
        .filter(|(entry, _)| {
            entry.original_stake as f64 * request.payout_multiplier
                - total_adjusted_stake as f64 * (1.0 - request.rebate_rate)
                > 0.0
        })
        .count();

    for (&cap, is_adjustable) in entry_caps.iter().zip(adjustable_mask.iter()) {
        if *is_adjustable {
            let safe_cap = cap.min(non_loss_cap);
            base_non_loss_total += safe_cap;
            if cap >= loss_trigger {
                candidate_extras.push(cap - safe_cap);
            }
        } else {
            base_non_loss_total += cap;
        }
    }
    candidate_extras.sort_unstable_by(|left, right| right.cmp(left));

    let mut prefix_extras = Vec::with_capacity(candidate_extras.len() + 1);
    prefix_extras.push(0_i64);
    for extra in candidate_extras {
        let next = prefix_extras.last().copied().unwrap_or(0) + extra;
        prefix_extras.push(next);
    }

    let max_candidates = prefix_extras.len() - 1;
    let target = request.loss_count_target.max(0) as usize;
    let mut best_choice = None;
    let mut best_gap = i64::MAX;

    for loss_count in 0..=max_candidates {
        let min_total = loss_count as i64 * loss_trigger
            + fixed_loss_count as i64 * 0
            + request
                .entries
                .iter()
                .zip(adjustable_mask.iter())
                .filter(|(_, is_adjustable)| !**is_adjustable)
                .map(|(entry, _)| entry.original_stake)
                .sum::<i64>();
        let max_total = base_non_loss_total + prefix_extras[loss_count];
        if min_total <= total_adjusted_stake && total_adjusted_stake <= max_total {
            let actual_loss_count = fixed_loss_count + loss_count;
            let gap = (actual_loss_count as i64 - request.loss_count_target).abs();
            if gap < best_gap
                || (gap == best_gap && actual_loss_count < best_choice.unwrap_or(usize::MAX))
            {
                best_gap = gap;
                best_choice = Some(actual_loss_count);
                if actual_loss_count == target {
                    break;
                }
            }
        }
    }

    best_choice.ok_or(BettingOptimizerError::NoFeasibleSolution)
}

fn construct_adjusted_stakes(
    total_adjusted_stake: i64,
    entry_caps: &[i64],
    adjustable_mask: &[bool],
    chosen_total_loss_count: usize,
    request: &BettingOptimizerRequest,
) -> Result<Vec<i64>, BettingOptimizerError> {
    let loss_trigger = compute_loss_trigger_stake(request, total_adjusted_stake);
    let non_loss_cap = loss_trigger.saturating_sub(1);
    let fixed_loss_count = request
        .entries
        .iter()
        .zip(adjustable_mask.iter())
        .filter(|(_, is_adjustable)| !**is_adjustable)
        .filter(|(entry, _)| {
            entry.original_stake as f64 * request.payout_multiplier
                - total_adjusted_stake as f64 * (1.0 - request.rebate_rate)
                > 0.0
        })
        .count();
    let chosen_adjustable_loss_count = chosen_total_loss_count.saturating_sub(fixed_loss_count);

    let mut candidate_indices = entry_caps
        .iter()
        .enumerate()
        .filter_map(|(index, &cap)| {
            if adjustable_mask[index] && cap >= loss_trigger {
                Some((index, cap - cap.min(non_loss_cap)))
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    candidate_indices
        .sort_unstable_by(|left, right| right.1.cmp(&left.1).then_with(|| left.0.cmp(&right.0)));

    let chosen_loss_indices = candidate_indices
        .iter()
        .take(chosen_adjustable_loss_count)
        .map(|(index, _)| *index)
        .collect::<std::collections::BTreeSet<_>>();

    let mut adjusted_stakes = request
        .entries
        .iter()
        .zip(adjustable_mask.iter())
        .map(|(entry, is_adjustable)| {
            if *is_adjustable {
                0
            } else {
                entry.original_stake
            }
        })
        .collect::<Vec<_>>();
    let mut remaining = total_adjusted_stake;

    for (entry, is_adjustable) in request.entries.iter().zip(adjustable_mask.iter()) {
        if !*is_adjustable {
            remaining -= entry.original_stake;
        }
    }

    for &index in &chosen_loss_indices {
        adjusted_stakes[index] = loss_trigger;
        remaining -= loss_trigger;
    }

    for (index, &cap) in entry_caps.iter().enumerate() {
        if !adjustable_mask[index] {
            continue;
        }
        let extra_capacity = if chosen_loss_indices.contains(&index) {
            cap - loss_trigger
        } else {
            cap.min(non_loss_cap)
        };
        if extra_capacity <= 0 || remaining <= 0 {
            continue;
        }

        let allocate = extra_capacity.min(remaining);
        adjusted_stakes[index] += allocate;
        remaining -= allocate;
    }

    if remaining != 0 {
        return Err(BettingOptimizerError::NoFeasibleSolution);
    }

    Ok(adjusted_stakes)
}

fn compute_loss_trigger_stake(request: &BettingOptimizerRequest, total_adjusted_stake: i64) -> i64 {
    let payable_principal = total_adjusted_stake as f64 * (1.0 - request.rebate_rate);
    let threshold = payable_principal / request.payout_multiplier;

    threshold.floor() as i64 + 1
}

fn format_decimal(value: f64) -> String {
    let rounded = (value * 100.0).round() / 100.0;
    if (rounded - rounded.round()).abs() < 1e-9 {
        format!("{}", rounded.round() as i64)
    } else {
        format!("{rounded:.2}")
    }
}
