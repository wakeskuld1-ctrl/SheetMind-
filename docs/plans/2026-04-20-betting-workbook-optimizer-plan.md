# Betting Workbook Optimizer Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build a stable-delivery Excel risk-control optimizer that keeps business input on the first sheet, computes the optimal integer-only downward adjustment plan with a Rust solver, and writes the recommendation view to the second sheet.

**Architecture:** Use a hybrid delivery shape: `xlsm` for UI and button entry, Rust `solver.exe` for parsing inputs, solving the integer optimization problem, and producing the adjustment output. Treat workbook layout, optimization model, and delivery bridge as separate modules so workbook UX and solver correctness can evolve independently.

**Tech Stack:** Rust (`calamine`, existing CLI/tooling, new integer-solver dependency if needed), minimal VBA button shell, Excel `.xlsm`, integration tests with `assert_cmd`, workbook fixture checks with `calamine`/`zip`.

---

### Task 0: Re-freeze workbook UI contract around the approved original-style page

**Files:**
- Modify: `docs/plans/2026-04-20-betting-workbook-optimizer-design.md`
- Modify: `docs/plans/2026-04-20-betting-workbook-optimizer-plan.md`
- Test: `tests/betting_workbook_bridge_cli.rs`

**Step 1: Write the failing workbook-layout test**

```rust
#[test]
fn betting_template_writer_restores_original_style_calculator_layout() {
    let output_path = create_test_output_path("betting_optimizer_original_layout", "xlsm");
    write_betting_template_xlsm(output_path.to_str().unwrap(), VBA_PROJECT_PATH).unwrap();
    let mut workbook = open_workbook_auto(&output_path).unwrap();
    let range = workbook.worksheet_range("计算器").unwrap();
    assert_eq!(range.get_value((0, 0)).unwrap().to_string(), "双");
    assert_eq!(range.get_value((0, 2)).unwrap().to_string(), "特下注额（需要填写）");
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --test betting_workbook_bridge_cli betting_template_writer_restores_original_style_calculator_layout -- --nocapture`

Expected: FAIL because the current template still writes a vertical program table.

**Step 3: Update the approved workbook contract docs**

```markdown
- sheet 1 name = `计算器`
- restore original grouped layout
- mark manual input fields as `（需要填写）`
```

**Step 4: Run test to verify it still fails for the right reason**

Run: `cargo test --test betting_workbook_bridge_cli betting_template_writer_restores_original_style_calculator_layout -- --nocapture`

Expected: FAIL only because implementation is not updated yet.

**Step 5: Commit**

```bash
git add docs/plans/2026-04-20-betting-workbook-optimizer-design.md docs/plans/2026-04-20-betting-workbook-optimizer-plan.md tests/betting_workbook_bridge_cli.rs
git commit -m "docs: freeze original-style betting workbook ui contract"
```

### Task 1: Freeze workbook contract and sample fixture

**Files:**
- Create: `docs/plans/2026-04-20-betting-workbook-optimizer-design.md`
- Create: `tests/fixtures/betting_optimizer/sample_input_contract.json`
- Create: `tests/fixtures/betting_optimizer/sample_expected_layout.json`
- Test: `tests/betting_workbook_contract_cli.rs`

**Step 1: Write the failing contract test**

```rust
#[test]
fn betting_workbook_contract_fixture_is_complete() {
    let fixture = std::fs::read_to_string(
        "tests/fixtures/betting_optimizer/sample_input_contract.json"
    ).unwrap();
    let json: serde_json::Value = serde_json::from_str(&fixture).unwrap();
    assert!(json["numbers"].as_array().unwrap().len() >= 49);
    assert!(json["targets"]["max_loss"].is_number());
    assert!(json["targets"]["loss_count_target"].is_number());
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --test betting_workbook_contract_cli betting_workbook_contract_fixture_is_complete -- --nocapture`

Expected: FAIL because the fixture file does not exist yet.

**Step 3: Add the minimal fixture files**

```json
{
  "numbers": [],
  "targets": {
    "max_loss": 1500,
    "loss_count_target": 19
  }
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test --test betting_workbook_contract_cli betting_workbook_contract_fixture_is_complete -- --nocapture`

Expected: PASS

**Step 5: Commit**

```bash
git add docs/plans/2026-04-20-betting-workbook-optimizer-design.md tests/fixtures/betting_optimizer tests/betting_workbook_contract_cli.rs
git commit -m "test: freeze betting workbook input contract"
```

### Task 2: Add core optimization domain types

**Files:**
- Create: `src/ops/betting_optimizer.rs`
- Modify: `src/ops/mod.rs`
- Modify: `src/lib.rs`
- Test: `tests/betting_optimizer_unit.rs`

**Step 1: Write the failing domain test**

```rust
#[test]
fn betting_optimizer_request_requires_integer_targets() {
    let request = BettingOptimizerRequest::new(vec![("02".into(), 80)], 47.0, 0.02, 1500.0, 19);
    assert_eq!(request.loss_count_target, 19);
    assert_eq!(request.entries[0].original_stake, 80);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --test betting_optimizer_unit betting_optimizer_request_requires_integer_targets -- --nocapture`

Expected: FAIL because `BettingOptimizerRequest` does not exist.

**Step 3: Write minimal implementation**

```rust
pub struct BettingOptimizerEntry {
    pub label: String,
    pub original_stake: i64,
}

pub struct BettingOptimizerRequest {
    pub entries: Vec<BettingOptimizerEntry>,
    pub payout_multiplier: f64,
    pub rebate_rate: f64,
    pub max_loss_limit: f64,
    pub loss_count_target: i64,
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test --test betting_optimizer_unit betting_optimizer_request_requires_integer_targets -- --nocapture`

Expected: PASS

**Step 5: Commit**

```bash
git add src/ops/betting_optimizer.rs src/ops/mod.rs src/lib.rs tests/betting_optimizer_unit.rs
git commit -m "feat: add betting optimizer domain contract"
```

### Task 3: Lock current-state math before optimization

**Files:**
- Modify: `src/ops/betting_optimizer.rs`
- Test: `tests/betting_optimizer_unit.rs`

**Step 1: Write the failing math test**

```rust
#[test]
fn betting_metrics_match_current_workbook_rules() {
    let request = sample_request();
    let metrics = evaluate_current_metrics(&request).unwrap();
    assert_eq!(metrics.total_stake, 2440);
    assert!((metrics.rebate - 48.8).abs() < 1e-6);
    assert!((metrics.payable_principal - 2391.2).abs() < 1e-6);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --test betting_optimizer_unit betting_metrics_match_current_workbook_rules -- --nocapture`

Expected: FAIL because `evaluate_current_metrics` does not exist.

**Step 3: Write minimal implementation**

```rust
pub fn evaluate_current_metrics(
    request: &BettingOptimizerRequest,
) -> Result<BettingMetrics, BettingOptimizerError> {
    // sum stakes, compute rebate, payable principal, per-number pnl
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test --test betting_optimizer_unit betting_metrics_match_current_workbook_rules -- --nocapture`

Expected: PASS

**Step 5: Commit**

```bash
git add src/ops/betting_optimizer.rs tests/betting_optimizer_unit.rs
git commit -m "feat: freeze betting workbook math rules"
```

### Task 4: Add xlsm bridge spike for stable macro-preserving update

**Files:**
- Create: `src/ops/betting_workbook_bridge.rs`
- Modify: `src/ops/mod.rs`
- Test: `tests/betting_workbook_bridge_cli.rs`
- Fixture: `tests/fixtures/betting_optimizer/template.xlsm`

**Step 1: Write the failing bridge spike test**

```rust
#[test]
fn workbook_bridge_can_load_template_targets_and_write_second_sheet_artifacts() {
    let fixture = "tests/fixtures/betting_optimizer/template.xlsm";
    let workbook = load_betting_workbook_contract(fixture).unwrap();
    assert_eq!(workbook.sheet_names[0], "当前盘面");
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --test betting_workbook_bridge_cli workbook_bridge_can_load_template_targets_and_write_second_sheet_artifacts -- --nocapture`

Expected: FAIL because the bridge module and fixture do not exist.

**Step 3: Write minimal implementation**

```rust
pub fn load_betting_workbook_contract(path: &str) -> Result<BettingWorkbookContract, BettingWorkbookBridgeError> {
    // verify sheet names, locate target cells, locate number grid
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test --test betting_workbook_bridge_cli workbook_bridge_can_load_template_targets_and_write_second_sheet_artifacts -- --nocapture`

Expected: PASS

**Step 5: Commit**

```bash
git add src/ops/betting_workbook_bridge.rs src/ops/mod.rs tests/betting_workbook_bridge_cli.rs tests/fixtures/betting_optimizer/template.xlsm
git commit -m "test: spike betting workbook xlsm bridge"
```

### Task 5: Add exact integer optimization engine

**Files:**
- Modify: `Cargo.toml`
- Modify: `src/ops/betting_optimizer.rs`
- Test: `tests/betting_optimizer_unit.rs`

**Step 1: Write the failing optimality test**

```rust
#[test]
fn optimizer_finds_min_refund_solution_under_max_loss_limit() {
    let request = sample_request();
    let solution = solve_betting_adjustment(&request).unwrap();
    assert!(solution.max_loss <= 1500.0);
    assert!(solution.total_refund >= 0.0);
    assert!(solution.entries.iter().all(|entry| entry.new_stake <= entry.original_stake));
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --test betting_optimizer_unit optimizer_finds_min_refund_solution_under_max_loss_limit -- --nocapture`

Expected: FAIL because `solve_betting_adjustment` does not exist.

**Step 3: Write minimal implementation**

```rust
pub fn solve_betting_adjustment(
    request: &BettingOptimizerRequest,
) -> Result<BettingOptimizerSolution, BettingOptimizerError> {
    // exact integer optimization:
    // primary objective = maximize retained stake
    // secondary objective = minimize abs(loss_count - target)
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test --test betting_optimizer_unit optimizer_finds_min_refund_solution_under_max_loss_limit -- --nocapture`

Expected: PASS

**Step 5: Commit**

```bash
git add Cargo.toml src/ops/betting_optimizer.rs tests/betting_optimizer_unit.rs
git commit -m "feat: add exact betting adjustment solver"
```

### Task 6: Lock secondary objective semantics

**Files:**
- Modify: `src/ops/betting_optimizer.rs`
- Test: `tests/betting_optimizer_unit.rs`

**Step 1: Write the failing secondary-objective test**

```rust
#[test]
fn optimizer_prefers_loss_count_closest_to_target_among_same_refund_solutions() {
    let request = sample_tie_break_request();
    let solution = solve_betting_adjustment(&request).unwrap();
    assert_eq!(solution.loss_count_gap, 0);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --test betting_optimizer_unit optimizer_prefers_loss_count_closest_to_target_among_same_refund_solutions -- --nocapture`

Expected: FAIL because tie-break semantics are not implemented.

**Step 3: Write minimal implementation**

```rust
// enforce lexicographic optimization rather than weighted approximation
```

**Step 4: Run test to verify it passes**

Run: `cargo test --test betting_optimizer_unit optimizer_prefers_loss_count_closest_to_target_among_same_refund_solutions -- --nocapture`

Expected: PASS

**Step 5: Commit**

```bash
git add src/ops/betting_optimizer.rs tests/betting_optimizer_unit.rs
git commit -m "feat: enforce betting optimizer tie-break objective"
```

### Task 7: Add recommendation summary builder

**Files:**
- Modify: `src/ops/betting_optimizer.rs`
- Test: `tests/betting_optimizer_unit.rs`

**Step 1: Write the failing summary test**

```rust
#[test]
fn optimizer_summary_mentions_limit_refund_and_focus_numbers() {
    let summary = build_optimizer_summary(&sample_solution());
    assert!(summary.contains("最大亏损"));
    assert!(summary.contains("退款"));
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --test betting_optimizer_unit optimizer_summary_mentions_limit_refund_and_focus_numbers -- --nocapture`

Expected: FAIL because `build_optimizer_summary` does not exist.

**Step 3: Write minimal implementation**

```rust
pub fn build_optimizer_summary(solution: &BettingOptimizerSolution) -> String {
    format!("当前方案最大亏损为 ...")
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test --test betting_optimizer_unit optimizer_summary_mentions_limit_refund_and_focus_numbers -- --nocapture`

Expected: PASS

**Step 5: Commit**

```bash
git add src/ops/betting_optimizer.rs tests/betting_optimizer_unit.rs
git commit -m "feat: add betting optimizer summary builder"
```

### Task 8: Add dedicated workbook solver binary

**Files:**
- Create: `src/bin/betting_solver.rs`
- Modify: `Cargo.toml`
- Test: `tests/betting_solver_cli.rs`

**Step 1: Write the failing CLI smoke test**

```rust
#[test]
fn betting_solver_returns_nonzero_when_workbook_path_is_missing() {
    let mut cmd = assert_cmd::Command::cargo_bin("betting_solver").unwrap();
    cmd.assert().failure();
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --test betting_solver_cli betting_solver_returns_nonzero_when_workbook_path_is_missing -- --nocapture`

Expected: FAIL because the binary does not exist.

**Step 3: Write minimal implementation**

```rust
fn main() {
    // parse workbook path, run bridge + solver + writer, print status
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test --test betting_solver_cli betting_solver_returns_nonzero_when_workbook_path_is_missing -- --nocapture`

Expected: PASS

**Step 5: Commit**

```bash
git add Cargo.toml src/bin/betting_solver.rs tests/betting_solver_cli.rs
git commit -m "feat: add betting solver binary"
```

### Task 9: Wire workbook output into second-sheet contract

**Files:**
- Modify: `src/ops/betting_workbook_bridge.rs`
- Modify: `src/bin/betting_solver.rs`
- Test: `tests/betting_workbook_bridge_cli.rs`

**Step 1: Write the failing write-back test**

```rust
#[test]
fn solver_writes_summary_and_adjustment_rows_into_second_sheet() {
    let output_path = run_solver_against_fixture();
    let workbook = calamine::open_workbook_auto(&output_path).unwrap();
    assert!(sheet_contains(&workbook, "优化建议", "总退款金额"));
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --test betting_workbook_bridge_cli solver_writes_summary_and_adjustment_rows_into_second_sheet -- --nocapture`

Expected: FAIL because second-sheet writing is not implemented.

**Step 3: Write minimal implementation**

```rust
pub fn write_optimizer_output(
    workbook_path: &str,
    solution: &BettingOptimizerSolution,
) -> Result<(), BettingWorkbookBridgeError> {
    // write summary block, summary text, detail rows into 优化建议
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test --test betting_workbook_bridge_cli solver_writes_summary_and_adjustment_rows_into_second_sheet -- --nocapture`

Expected: PASS

**Step 5: Commit**

```bash
git add src/ops/betting_workbook_bridge.rs src/bin/betting_solver.rs tests/betting_workbook_bridge_cli.rs
git commit -m "feat: write betting optimizer results into workbook"
```

### Task 10: Add tool entry or packaging helper for future automation

**Files:**
- Modify: `src/tools/catalog.rs`
- Modify: `src/tools/dispatcher.rs`
- Create: `src/ops/betting_workbook_packaging.rs`
- Test: `tests/betting_solver_cli.rs`

**Step 1: Write the failing discoverability test**

```rust
#[test]
fn tool_catalog_includes_betting_workbook_packaging() {
    let output = common::run_cli_with_empty_input();
    assert!(output["data"]["tool_catalog"].to_string().contains("betting_workbook_packaging"));
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --test betting_solver_cli tool_catalog_includes_betting_workbook_packaging -- --nocapture`

Expected: FAIL because the tool does not exist.

**Step 3: Write minimal implementation**

```rust
// package template workbook + solver executable into a stable output directory
```

**Step 4: Run test to verify it passes**

Run: `cargo test --test betting_solver_cli tool_catalog_includes_betting_workbook_packaging -- --nocapture`

Expected: PASS

**Step 5: Commit**

```bash
git add src/tools/catalog.rs src/tools/dispatcher.rs src/ops/betting_workbook_packaging.rs tests/betting_solver_cli.rs
git commit -m "feat: expose betting workbook packaging entry"
```

### Task 11: Add thin VBA template integration notes and packaging docs

**Files:**
- Create: `docs/plans/2026-04-20-betting-workbook-optimizer-handoff.md`
- Create: `assets/excel_templates/README.md`
- Modify: `README.md`
- Test: `tests/betting_solver_cli.rs`

**Step 1: Write the failing packaging expectation test**

```rust
#[test]
fn packaging_docs_describe_xlsm_and_solver_pair() {
    let readme = std::fs::read_to_string("assets/excel_templates/README.md").unwrap();
    assert!(readme.contains("betting_solver.exe"));
    assert!(readme.contains(".xlsm"));
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --test betting_solver_cli packaging_docs_describe_xlsm_and_solver_pair -- --nocapture`

Expected: FAIL because the README file does not exist.

**Step 3: Write minimal implementation**

```markdown
Place `betting_solver.exe` beside the delivered workbook.
The VBA button only shells out to the executable and does not contain optimization logic.
```

**Step 4: Run test to verify it passes**

Run: `cargo test --test betting_solver_cli packaging_docs_describe_xlsm_and_solver_pair -- --nocapture`

Expected: PASS

**Step 5: Commit**

```bash
git add docs/plans/2026-04-20-betting-workbook-optimizer-handoff.md assets/excel_templates/README.md README.md tests/betting_solver_cli.rs
git commit -m "docs: add betting workbook packaging notes"
```

### Task 12: Full verification and delivery rehearsal

**Files:**
- Modify: `progress.md`
- Modify: `findings.md`
- Modify: `task_plan.md`
- Test: `tests/betting_optimizer_unit.rs`
- Test: `tests/betting_workbook_bridge_cli.rs`
- Test: `tests/betting_solver_cli.rs`

**Step 1: Run focused unit tests**

Run: `cargo test --test betting_optimizer_unit -- --nocapture`

Expected: PASS

**Step 2: Run workbook bridge tests**

Run: `cargo test --test betting_workbook_bridge_cli -- --nocapture`

Expected: PASS

**Step 3: Run solver binary tests**

Run: `cargo test --test betting_solver_cli -- --nocapture`

Expected: PASS

**Step 4: Rehearse one packaged sample**

Run: `cargo run --bin betting_solver -- "tests/fixtures/betting_optimizer/template.xlsm"`

Expected: solver exits successfully and the sample workbook contains updated `优化建议` content.

**Step 5: Record results and commit**

```bash
git add progress.md findings.md task_plan.md
git commit -m "test: verify betting workbook optimizer delivery flow"
```
