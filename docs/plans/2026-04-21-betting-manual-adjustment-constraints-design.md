# Betting Manual Adjustment Constraints Design

## Intent
- Keep the current stable `xlsm + exe` delivery shape while adding one approved field workflow: the operator can add per-number manual adjustment constraints on a result sheet, then generate the next result sheet under those constraints.
- Preserve the already-approved append-only round model: no in-place recalculation on the current result sheet, and no overwriting historical rounds.

## Contract

### Business Contract
- Solver input remains integer-only.
- Stakes can only move downward.
- Only numbers whose `盈亏额 > 0` are treated as risky numbers and are eligible for refund optimization.
- The primary objective remains minimum total refund.
- The secondary objective remains making the count of risky numbers as close as possible to the target count.
- Manual adjustment constraints only affect the current solve request and do not silently become permanent defaults for future rounds.

### Manual Adjustment Contract
- Every result sheet may accept two optional manual inputs per number:
  - `手工锁定下轮下注额（需要填写，可留空）`
  - `本轮最多可退款金额（需要填写，可留空）`
- The two manual inputs are mutually exclusive on the same row.
- `手工锁定下轮下注额` means the next-round stake for that number is fixed and must not be changed by the solver.
- `本轮最多可退款金额` means the solver may still optimize that number, but the refund on that number must not exceed the entered cap.
- Both manual inputs must be integers.
- `手工锁定下轮下注额` must be within `[0, 当前基线下注额]`.
- `本轮最多可退款金额` must be within `[0, 当前基线下注额]`.
- For non-risk rows, `本轮最多可退款金额` is not allowed because profitable numbers do not participate in refund optimization.
- If a manual constraint makes the target impossible, the solver must return the closest feasible result and explicitly mark that the target was not fully reached because of manual constraints.

### Workbook Interaction Contract
- Sheet 1 remains the fixed operator home page.
- Result sheets remain append-only and use the format `优化建议_第N轮`.
- Manual inputs are entered on the current result sheet, but the solve result must always be written to the next result sheet.
- The current result sheet remains historical input evidence and is never recalculated in place.
- Every result sheet must expose the button `基于本页再次测算`.
- The button must read manual inputs from the active result sheet and generate `优化建议_第N+1轮`.

## Approaches Considered

### Option A1: Refund-cap only
- Core idea:
  - Add only `本轮最多可退款金额`.
- Pros:
  - Closest to the original approved scheme A.
  - The optimization model stays simple.
- Cons:
  - Operators often think in `200 -> 180` or `200 -> 10`, not in refund caps.
- Risk:
  - Higher field confusion because operators need to convert their intent.

### Option A2: Enhanced dual-input constraint model
- Core idea:
  - Add both `手工锁定下轮下注额` and `本轮最多可退款金额`, but enforce one-or-the-other on each row.
- Pros:
  - Matches real field behavior.
  - Supports both “do not reduce too much” and “I have already decided the exact next-round stake”.
- Cons:
  - Validation rules become stricter.
- Risk:
  - Operators may fill both cells unless validation and status messages are explicit.

### Option A3: Separate top-area manual adjustment panel
- Core idea:
  - Keep the detail table clean and move manual adjustment entry to a top control block.
- Pros:
  - Less clutter in the detail table.
- Cons:
  - Worse usability when multiple numbers must be adjusted.
- Risk:
  - Easier to fill the wrong number or lose row-level context.

## Approved Decision
- Use Option A2.
- Keep the additional user-approved rule: clicking solve from a result sheet never recalculates that sheet in place; it always generates the next result sheet.

## Result Sheet Design

### Existing Stable Columns To Keep
- `号码`
- `当前基线下注额`
- `建议下注额`
- `退款额`
- `当前盈亏额`
- `调整后盈亏额`
- `当前是否风险`
- `是否建议下调`

### New Columns To Add
- `手工锁定下轮下注额（需要填写，可留空）`
- `本轮最多可退款金额（需要填写，可留空）`
- `对应最低保留下注额`
- `人工约束状态`

### Input and Display Rules
- `手工锁定下轮下注额（需要填写，可留空）`
  - Editable
  - Yellow input style
  - Blank means no hard lock
- `本轮最多可退款金额（需要填写，可留空）`
  - Editable
  - Yellow input style
  - Blank means no refund cap
- `对应最低保留下注额`
  - Read-only
  - Derived from `当前基线下注额 - 本轮最多可退款金额`
  - Blank if refund cap is blank
- `人工约束状态`
  - Read-only
  - One of:
    - `未设置`
    - `已锁定`
    - `已设置退款上限`
    - `受上限约束`
    - `人工约束导致目标未完全达成`

## Solve Flow
1. The operator opens `优化建议_第N轮`.
2. The operator optionally edits:
   - `手工锁定下轮下注额（需要填写，可留空）`
   - `本轮最多可退款金额（需要填写，可留空）`
   - next-round targets in the approved target area
3. The operator clicks `基于本页再次测算`.
4. VBA validates the active sheet and calls the Rust solver with the active result sheet as the source sheet.
5. Rust reads the current result sheet as the baseline plus manual-constraint source.
6. Rust solves under:
   - downward-only constraint
   - integer-only constraint
   - optional hard-locked stake constraints
   - optional refund-cap constraints
7. Rust appends `优化建议_第N+1轮`.
8. The new sheet becomes the place where the operator sees the recomputed recommendation and can continue the next round.

## Validation Rules
- Reject if a row contains both manual inputs.
- Reject if any manual input is non-integer.
- Reject if `手工锁定下轮下注额 > 当前基线下注额`.
- Reject if `本轮最多可退款金额 > 当前基线下注额`.
- Reject if a non-risk row contains `本轮最多可退款金额`.
- Accept `0` as a valid value in either manual input.
- Show a readable workbook status message and write the same reason to the log file.

## Error Handling
- If the active result sheet is missing required columns, return a contract error and do not generate the next sheet.
- If manual inputs conflict, stop before solve and point to the exact row.
- If a hard lock or refund cap makes the target unreachable, do not fail the whole solve:
  - generate the best feasible next-round result
  - mark the solve status as constraint-limited
  - explain that the target was not fully reached because of manual constraints

## Logging Contract
- Solver logs must record:
  - source sheet name
  - target output sheet name
  - number of hard-locked rows
  - number of refund-cap rows
  - whether the solution was fully feasible or constraint-limited
- Workbook status area must show a concise summary of the same state.

## Testing Contract
- Add parser tests for the new manual-input columns on result sheets.
- Add solver tests for:
  - hard lock only
  - refund cap only
  - mixed rows with one-or-the-other
  - invalid row with both fields set
  - non-risk row with refund cap
  - next-round sheet generation under constraints
- Re-run release build and workbook generation tests before completion.

## Risks
- The current result-sheet parser is coordinate-based, so adding columns can break round-sheet loading unless the parser contract is updated together with tests.
- Dual manual-input support adds more validation branches, so field messaging must stay very explicit.
- The objective function may encounter more infeasible combinations, so the solver must distinguish “invalid input” from “valid but constrained result”.

## User Correction Memory
- Error type:
  - Solve target sheet misunderstanding.
- Trigger condition:
  - Treating a result-sheet solve as an in-place recalculation.
- Correct behavior:
  - Manual input is entered on the current result sheet, but the actual recomputed recommendation must be written to the next round sheet.
- Avoidance:
  - Keep all docs, tests, status text, and VBA entrypoints aligned to append-only next-round generation.
