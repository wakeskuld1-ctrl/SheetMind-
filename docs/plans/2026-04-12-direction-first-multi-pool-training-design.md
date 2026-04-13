# Direction-First Multi-Pool Training Design

## Context
- The user wants to improve practical prediction accuracy for bank equities and ETF sub-pools.
- This round explicitly excludes the `180d` horizon.
- The target horizons are `5d`, `10d`, `15d`, and `30d`.
- The user prefers a direction-first strategy: direction quality must improve first, then regression quality can be used as a secondary optimization signal.
- The user also asked for one long training session around seven hours instead of a separate short pilot and a later formal run.

## Goal
- Run one governed multi-stage training cycle that keeps direction accuracy as the primary selection objective while using regression quality as a secondary tie-breaker.
- Cover:
  - bank equities
  - `treasury_etf`
  - `gold_etf`
  - `cross_border_etf`
  - `equity_etf`
- Produce auditable outputs that can be resumed, compared, and handed off to the next AI or operator.

## Non-Goals
- No `180d` training in this round.
- No large-scale architecture rewrite.
- No brand-new modeling framework.
- No Python-dependent business implementation.

## Strategy
- Use `direction_head` as the primary optimization target.
- Use `return_head` as a secondary quality lens after direction quality is stable enough.
- Train each market pool independently so bank equities do not dilute ETF-specific signal structure, and one ETF pool does not distort another.
- Train each horizon independently so `5d` and `30d` do not collapse into one generic parameter choice.

## Market Pools
- `a_share_bank`
- `treasury_etf`
- `gold_etf`
- `cross_border_etf`
- `equity_etf`

## Horizons
- `5`
- `10`
- `15`
- `30`

## Head Policy
- Primary head:
  - `direction_head`
- Secondary head:
  - `return_head`
- Not primary in this round:
  - `upside_first_head`
  - `stop_first_head`

Reason:
- The user asked for “accuracy” first, which maps more directly to direction quality.
- `return_head` still matters because the system should not become directionally correct while becoming numerically unstable or unusable.

## Selection Policy
### Primary ranking
1. `direction_head.test.accuracy`
2. `direction_head.test.auc`

### Secondary ranking
1. `return_head.test.directional_hit_rate`
2. `return_head.test.rmse_improvement_vs_baseline`

### Promotion rule
- A candidate cannot advance if its direction quality regresses materially.
- Regression quality only helps when direction quality is at least preserved.

## Seven-Hour Session Layout
### Stage 1: Warm Start and Safety Check
- Duration:
  - about `0-45` minutes
- Purpose:
  - combine the “pilot” into the formal run
  - confirm the data slices, runtime paths, and artifact generation are stable
  - eliminate obviously weak parameter combinations early

### Stage 2: Direction Optimization
- Duration:
  - about `45-150` minutes
- Purpose:
  - concentrate on direction quality improvement
  - compare direction-focused combinations across pools and horizons

### Stage 3: Survivor Training
- Duration:
  - about `150-330` minutes
- Purpose:
  - continue only the stronger combinations
  - bring `return_head` comparison into the decision loop without letting it override direction quality

### Stage 4: Final Convergence and Export
- Duration:
  - about `330-420` minutes
- Purpose:
  - export best artifacts
  - record summary tables
  - preserve checkpoints and runtime evidence for continuation

## Parameter Governance
- Keep parameter search incremental rather than unlimited.
- Start from a small governed candidate grid:
  - sample-plan thickness
  - train/valid/test sampling density
  - feature-family inclusion scope
  - calibration or threshold choice for direction interpretation
- Only expand if the first two stages show signal, not noise.

## Output Requirements
- Best artifact per:
  - market pool
  - horizon
  - head
- Summary table with:
  - best direction accuracy
  - best direction AUC
  - best regression directional hit rate
  - best RMSE improvement vs baseline
  - production-readiness / shadow-readiness status
- Preserved runtime paths for:
  - model artifacts
  - refit documents
  - shadow evaluation outputs
  - promotion comparison inputs if applicable
- Handoff-ready execution notes

## Failure Handling
- If a pool or horizon shows unstable direction results in Stage 1, demote it and stop spending the bulk of the seven-hour budget on it.
- If artifact generation or downstream evaluation fails, record the exact failure and continue with unaffected pools when possible.
- If direction quality improves but regression quality collapses sharply, keep the direction result but flag that pool/horizon as “direction-only provisional”.

## Risks
- Seven hours is enough for meaningful improvement, but not enough to guarantee all pools and all horizons become strong simultaneously.
- The likely bottlenecks are still:
  - sample richness
  - pool-specific separability
  - horizon-specific noise
- A good result from `a_share_bank` does not imply the ETF pools are equally strong, and vice versa.

## Expected Next Step After This Round
- If direction quality improves but remains unstable:
  - thicken sample plans and replay windows
- If direction quality is stable but runtime landing is still manual:
  - connect stronger training outputs to the runtime selection path
