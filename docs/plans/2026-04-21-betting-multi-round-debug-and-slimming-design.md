# Betting Workbook Multi-Round Debug And Slimming Design

## Intent
- Keep the delivered workbook stable for field use while adding one new approved capability: each result sheet can be tuned again and produce the next result sheet without overwriting history.
- Reduce `betting_solver.exe` size with the approved low-risk compile-profile route first, without changing the packaging model of `xlsm + exe`.

## Contract

### Business Contract
- Solver input remains integer-only.
- Stakes can only move downward.
- Only numbers whose `盈亏额 > 0` are considered risky and eligible for refund.
- The primary objective remains minimum total refund.
- The secondary objective remains making the count of risky numbers as close as possible to the target count.

### Workbook Contract
- Sheet 1 stays the operator home page and remains the only fixed sheet.
- First solve from sheet 1 creates `优化建议_第1轮`.
- Re-solving from `优化建议_第N轮` creates `优化建议_第N+1轮`.
- Existing result sheets are preserved and never overwritten by a later round.
- Every result sheet must clearly show:
  - current round number
  - source sheet name
  - current max-loss target
  - current loss-count target
  - summary text
  - detailed adjustment table
- Every result sheet must expose a button `基于本页再次测算`.
- Editable cells on result sheets are limited to:
  - adjusted stake baseline used for the next round
  - max loss target
  - loss count target
- Result-sheet risk and adjustment highlighting remains the approved red style.

### Solver Invocation Contract
- `say_hello` continues to mean “solve from sheet 1”.
- A new VBA macro will solve from the active result sheet.
- The solver must accept an optional source sheet name so it can read either sheet 1 or a chosen result sheet.
- Workbook write-back must append a new result sheet instead of replacing a fixed `优化建议` sheet.

## Decision

### Decision 1: Use append-only result sheets
- Reason:
  - Matches the approved workflow: second sheet tune -> third sheet result -> continue forward.
  - Preserves traceability for现场复盘.
- Consequence:
  - We need round naming, source tracking, and next-round sheet generation.

### Decision 2: Keep sheet 1 as the permanent home page
- Reason:
  - Preserves the already-approved “业务操作和现场还原放在第一页”.
  - Avoids turning sheet 1 into mutable history state.
- Consequence:
  - Multi-round tuning lives in result sheets, not by backfilling sheet 1.

### Decision 3: Support result-sheet re-solve through the same Rust binary
- Reason:
  - Keeps business logic in Rust and VBA thin.
  - Avoids duplicating parse/solve logic across VBA and Rust.
- Consequence:
  - Rust workbook bridge must learn how to parse a result sheet as the next-round baseline.

### Decision 4: Apply slimming plan A only in this round
- Reason:
  - User approved the low-risk profile-only route.
  - Fastest way to reduce size without reopening architecture.
- Consequence:
  - Add release-profile settings only; do not split crates in this task.

## Data Flow
1. User fills or reviews sheet 1 and clicks `测算并生成建议`.
2. VBA calls `betting_solver.exe solve <workbook> <temp_output> --source-sheet 计算器`.
3. Rust reads sheet 1, solves, and writes a workbook copy containing `优化建议_第1轮`.
4. VBA imports the newly created round sheet back into the current workbook.
5. User edits allowed cells on `优化建议_第1轮` and clicks `基于本页再次测算`.
6. VBA calls the same solver with `--source-sheet 优化建议_第1轮`.
7. Rust parses that sheet as the next baseline, solves again, and appends `优化建议_第2轮`.

## Sheet Design

### Sheet 1
- No structural change beyond keeping the existing target area, status area, log behavior, and button.

### Result Sheets
- Name format: `优化建议_第N轮`
- Top block:
  - `轮次`
  - `来源页`
  - `最大计划亏损目标`
  - `目标亏损号码数`
  - `业务操作`
  - `求解状态`
- Summary block:
  - current vs recommended total stake
  - current vs recommended total refund
  - current vs recommended max loss
  - current vs recommended risky-number count
  - generated explanation text
- Detail block:
  - 号码
  - 当前基线下注额（需要填写/可调整）
  - 建议下注额
  - 退款额
  - 当前盈亏额
  - 调整后盈亏额
  - 当前是否风险
  - 是否建议下调

## Error Handling
- If the source result sheet is missing required cells or has an invalid name, VBA shows a readable status and logs the exact source sheet name.
- If the next round sheet name already exists, VBA deletes only the temporary imported copy and Rust picks the next available round number.
- If a result sheet has invalid non-integer edits, solver returns a clear contract error and does not create a new sheet.

## Risks
- Result-sheet parsing is more fragile than fixed sheet 1 parsing, so we need explicit cell-address tests.
- Append-only history can create many sheets; this is acceptable in the current approved scope.
- Release-profile slimming may reduce debug ergonomics, so verification must include release build and solver smoke test.

## Acceptance
- `betting_solver.exe` release build is smaller than the previous ~52.56 MB baseline.
- Sheet 1 still generates the first result sheet successfully.
- Running solve from `优化建议_第1轮` creates `优化建议_第2轮` instead of overwriting round 1.
- Round 2 uses the edited round-1 baseline and targets.
- Existing red highlighting still works on template and result sheets.
- Macro logs and solver logs identify which source sheet was solved.
