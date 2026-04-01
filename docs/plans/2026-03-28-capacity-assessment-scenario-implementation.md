# Capacity Assessment Scenario Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add an elastic operations capacity-assessment tool on top of the existing SheetMind Rust tool system so Excel users can get quantified conclusions when data is sufficient and decision guidance when data is incomplete.

**Architecture:** Implement a new analysis/modeling tool named `capacity_assessment` inside the Rust tool layer instead of building a separate engine. The tool will load a table from the existing workbook/result/table reference pathways, inspect what evidence is available, reuse existing analysis helpers where useful, and return either quantified capacity conclusions or fallback decision guidance with explicit missing-input lists.

**Tech Stack:** Rust, Polars, existing SheetMind tool dispatcher, JSON CLI contract, cargo test

---

### Task 1: Lock the tool contract with failing CLI tests

**Files:**
- Create: `D:\Rust\Excel_Skill\.worktrees\SheetMind-\tests\capacity_assessment_cli.rs`
- Reference: `D:\Rust\Excel_Skill\.worktrees\SheetMind-\tests\common\mod.rs`
- Reference: `D:\Rust\Excel_Skill\.worktrees\SheetMind-\src\tools\catalog.rs`

**Step 1: Write the failing tests**

```rust
#[test]
fn tool_catalog_includes_capacity_assessment() {
    let output = run_cli_with_json("");
    assert!(
        output["data"]["tool_catalog"]
            .as_array()
            .unwrap()
            .iter()
            .any(|tool| tool == "capacity_assessment")
    );
}

#[test]
fn capacity_assessment_returns_quantified_snapshot_conclusion() {
    let workbook_path = create_test_workbook(
        "capacity_assessment_snapshot",
        "snapshot.xlsx",
        &[(
            "Capacity",
            vec![
                vec!["service", "instances", "cpu_usage", "memory_usage"],
                vec!["api-gateway", "4", "0.82", "0.68"],
            ],
        )],
    );
    let request = json!({
        "tool": "capacity_assessment",
        "args": {
            "path": workbook_path.to_string_lossy(),
            "sheet": "Capacity",
            "instance_count_column": "instances",
            "cpu_column": "cpu_usage",
            "memory_column": "memory_usage",
            "target_cpu_utilization": 0.70,
            "target_memory_utilization": 0.75,
            "peak_multiplier": 1.20,
            "growth_rate": 0.30,
            "require_n_plus_one": true
        }
    });
    let output = run_cli_with_json(&request.to_string());
    assert_eq!(output["status"], "ok");
    assert_eq!(output["data"]["evidence_level"], "quantified");
    assert_eq!(output["data"]["capacity_status"], "insufficient");
    assert!(output["data"]["recommended_instance_count"].as_u64().unwrap() >= 6);
}

#[test]
fn capacity_assessment_returns_guidance_when_data_is_incomplete() {
    let workbook_path = create_test_workbook(
        "capacity_assessment_guidance",
        "guidance.xlsx",
        &[(
            "Capacity",
            vec![
                vec!["service", "known_issue", "current_peak_note"],
                vec!["billing", "peak lag", "double eleven burst observed"],
            ],
        )],
    );
    let request = json!({
        "tool": "capacity_assessment",
        "args": {
            "path": workbook_path.to_string_lossy(),
            "sheet": "Capacity"
        }
    });
    let output = run_cli_with_json(&request.to_string());
    assert_eq!(output["status"], "ok");
    assert_eq!(output["data"]["evidence_level"], "guidance_only");
    assert!(output["data"]["missing_inputs"].as_array().unwrap().len() >= 3);
    assert!(output["data"]["decision_guidance"]["recommended_next_step"]
        .as_str()
        .unwrap()
        .contains("补"));
}

#[test]
fn capacity_assessment_uses_history_when_time_series_is_available() {
    let workbook_path = create_test_workbook(
        "capacity_assessment_history",
        "history.xlsx",
        &[(
            "Capacity",
            vec![
                vec!["ts", "instances", "cpu_usage", "workload_qps"],
                vec!["2026-03-25 09:00", "4", "0.52", "1200"],
                vec!["2026-03-25 10:00", "4", "0.61", "1500"],
                vec!["2026-03-25 11:00", "4", "0.79", "2100"],
                vec!["2026-03-25 12:00", "4", "0.87", "2600"],
            ],
        )],
    );
    let request = json!({
        "tool": "capacity_assessment",
        "args": {
            "path": workbook_path.to_string_lossy(),
            "sheet": "Capacity",
            "time_column": "ts",
            "instance_count_column": "instances",
            "cpu_column": "cpu_usage",
            "workload_column": "workload_qps",
            "target_cpu_utilization": 0.70,
            "peak_multiplier": 1.00,
            "growth_rate": 0.10
        }
    });
    let output = run_cli_with_json(&request.to_string());
    assert_eq!(output["status"], "ok");
    assert_eq!(output["data"]["trend_observations"]["cpu"]["direction"], "upward");
    assert!(output["data"]["history_signals"]["has_time_series"].as_bool().unwrap());
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test --manifest-path .worktrees/SheetMind-/Cargo.toml --test capacity_assessment_cli -- --nocapture`
Expected: FAIL because `capacity_assessment` is not registered yet.

**Step 3: Commit**

```bash
git -C .worktrees/SheetMind- add tests/capacity_assessment_cli.rs
git -C .worktrees/SheetMind- commit -m "test: add capacity assessment scenario contract"
```

### Task 2: Implement the elastic capacity assessment core

**Files:**
- Create: `D:\Rust\Excel_Skill\.worktrees\SheetMind-\src\ops\capacity_assessment.rs`
- Modify: `D:\Rust\Excel_Skill\.worktrees\SheetMind-\src\ops\mod.rs`

**Step 1: Add the minimal data structures and red-path handling**

```rust
pub struct CapacityAssessmentResult {
    pub assistant_kind: String,
    pub evidence_level: String,
    pub capacity_status: String,
    pub recommended_instance_count: Option<u64>,
    pub missing_inputs: Vec<String>,
    pub decision_guidance: CapacityDecisionGuidance,
}
```

**Step 2: Implement the elastic evaluation rules**

```rust
pub fn capacity_assessment(
    loaded: &LoadedTable,
    request: &CapacityAssessmentRequest,
) -> Result<CapacityAssessmentResult, CapacityAssessmentError> {
    // 1. Read the newest usable row or the only row as a snapshot
    // 2. Quantify CPU and memory pressure when columns exist
    // 3. Apply peak multiplier, growth rate, and N+1 reserve
    // 4. Fall back to guidance mode when required evidence is missing
    // 5. Reuse trend_analysis / outlier_detection when time series exists
}
```

**Step 3: Run the new tests**

Run: `cargo test --manifest-path .worktrees/SheetMind-/Cargo.toml --test capacity_assessment_cli -- --nocapture`
Expected: At least one test still FAILS until dispatcher/catalog wiring is added.

**Step 4: Commit**

```bash
git -C .worktrees/SheetMind- add src/ops/capacity_assessment.rs src/ops/mod.rs
git -C .worktrees/SheetMind- commit -m "feat: add elastic capacity assessment core"
```

### Task 3: Register the tool in the catalog and dispatcher

**Files:**
- Modify: `D:\Rust\Excel_Skill\.worktrees\SheetMind-\src\tools\catalog.rs`
- Modify: `D:\Rust\Excel_Skill\.worktrees\SheetMind-\src\tools\dispatcher.rs`
- Modify: `D:\Rust\Excel_Skill\.worktrees\SheetMind-\src\tools\dispatcher\analysis_ops.rs`

**Step 1: Add the tool name to the catalog**

```rust
"capacity_assessment",
```

**Step 2: Wire dispatcher routing**

```rust
"capacity_assessment" => analysis_ops::dispatch_capacity_assessment(request.args),
```

**Step 3: Add analysis dispatcher parsing**

```rust
pub(super) fn dispatch_capacity_assessment(args: Value) -> ToolResponse {
    // load source table
    // parse optional columns and scenario assumptions
    // call capacity_assessment(...)
    // return ToolResponse::ok(json!(result))
}
```

**Step 4: Run tests**

Run: `cargo test --manifest-path .worktrees/SheetMind-/Cargo.toml --test capacity_assessment_cli -- --nocapture`
Expected: PASS

**Step 5: Commit**

```bash
git -C .worktrees/SheetMind- add src/tools/catalog.rs src/tools/dispatcher.rs src/tools/dispatcher/analysis_ops.rs
git -C .worktrees/SheetMind- commit -m "feat: expose capacity assessment tool"
```

### Task 4: Add regression coverage for graceful degradation

**Files:**
- Modify: `D:\Rust\Excel_Skill\.worktrees\SheetMind-\tests\capacity_assessment_cli.rs`

**Step 1: Add one more regression test for partial evidence**

```rust
#[test]
fn capacity_assessment_returns_range_guidance_when_only_cpu_and_instance_data_exist() {
    // ensure the tool does not error
    // ensure evidence_level becomes partial
    // ensure recommended_instance_count is still present
    // ensure missing_inputs still lists absent dimensions
}
```

**Step 2: Run the focused test file**

Run: `cargo test --manifest-path .worktrees/SheetMind-/Cargo.toml --test capacity_assessment_cli -- --nocapture`
Expected: PASS

**Step 3: Commit**

```bash
git -C .worktrees/SheetMind- add tests/capacity_assessment_cli.rs
git -C .worktrees/SheetMind- commit -m "test: cover capacity assessment degradation paths"
```

### Task 5: Verify, document, and hand off

**Files:**
- Modify: `D:\Rust\Excel_Skill\progress.md`
- Modify: `D:\Rust\Excel_Skill\findings.md`
- Modify: `D:\Rust\Excel_Skill\task_plan.md`
- Modify: `D:\Rust\Excel_Skill\.trae\CHANGELOG_TASK.md`

**Step 1: Run verification**

Run: `cargo test --manifest-path .worktrees/SheetMind-/Cargo.toml --test capacity_assessment_cli -- --nocapture`
Expected: PASS

Run: `cargo test --manifest-path .worktrees/SheetMind-/Cargo.toml --test integration_tool_contract -- --nocapture`
Expected: PASS or unchanged existing status

**Step 2: Update tracking files**

```markdown
- progress.md: append what was implemented and which cargo tests passed
- findings.md: append the key finding that capacity assessment must degrade from quantified mode to guidance mode instead of blocking on missing metrics
- task_plan.md: append a parallel task note pointing to the SheetMind worktree implementation slice
- .trae/CHANGELOG_TASK.md: append one task-journal entry only
```

**Step 3: Commit**

```bash
git add progress.md findings.md task_plan.md .trae/CHANGELOG_TASK.md
git commit -m "docs: record capacity assessment scenario progress"
```
