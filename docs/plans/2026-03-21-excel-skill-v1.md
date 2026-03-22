# Excel Skill V1 Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build a no-deployment Rust Excel processing engine that ships as a single binary, reads Excel into in-memory DataFrames, exposes table-processing tools through stable JSON interfaces, and allows a Skill layer to orchestrate those tools without embedding business logic.

**Architecture:** V1 uses a layered Rust design: Excel I/O and schema inference feed a Polars-backed table engine, atomic operations are wrapped as tool contracts, and a thin CLI/JSON bridge exposes the capabilities to Skills. Complex headers use a dual-track schema model: preserve hierarchical header metadata internally while exposing stable canonical column names externally; downstream tools only run after schema confirmation.

**Tech Stack:** Rust, Cargo, `polars`, `calamine`, `rust_xlsxwriter`, `serde`, `serde_json`, `thiserror`, `tracing`, `assert_cmd`, `insta`, fixture-based integration tests.

---

## Delivery Decision

**Recommended binary shape:** single executable CLI that accepts JSON requests and returns JSON responses.

- Why this first:
  - no Python/Node/runtime deployment burden for Excel users
  - easy for Skill to call as a local Tool adapter
  - packaging is simple: unzip and run one `.exe`
- Deferred option:
  - later add daemon mode or local HTTP mode for hot session reuse without changing the internal engine API

## Project Layout

- Create: `Cargo.toml`
- Create: `src/main.rs`
- Create: `src/lib.rs`
- Create: `src/domain/mod.rs`
- Create: `src/domain/handles.rs`
- Create: `src/domain/schema.rs`
- Create: `src/domain/result.rs`
- Create: `src/excel/mod.rs`
- Create: `src/excel/reader.rs`
- Create: `src/excel/regions.rs`
- Create: `src/excel/header_inference.rs`
- Create: `src/frame/mod.rs`
- Create: `src/frame/registry.rs`
- Create: `src/frame/conversion.rs`
- Create: `src/ops/mod.rs`
- Create: `src/ops/select.rs`
- Create: `src/ops/filter.rs`
- Create: `src/ops/group.rs`
- Create: `src/ops/pivot.rs`
- Create: `src/ops/export.rs`
- Create: `src/tools/mod.rs`
- Create: `src/tools/contracts.rs`
- Create: `src/tools/dispatcher.rs`
- Create: `src/tasks/mod.rs`
- Create: `src/tasks/normalize_table.rs`
- Create: `src/tasks/build_summary_report.rs`
- Create: `src/trace/mod.rs`
- Create: `src/trace/execution.rs`
- Create: `tests/fixtures/`
- Create: `tests/integration_open_workbook.rs`
- Create: `tests/integration_header_schema.rs`
- Create: `tests/integration_table_ops.rs`
- Create: `tests/integration_cli_json.rs`
- Create: `tests/common/mod.rs`
- Create: `docs/contracts/tool-schema.md`

## Constraints To Preserve

- Skill only orchestrates; no computation logic in Skill prompts.
- All calculations must execute in Rust engine code.
- Every downstream table Tool requires a confirmed schema.
- V1 prefers strictness over convenience: medium/low-confidence header inference must request confirmation.
- Output must be deployment-light: produce a single binary artifact for end users.
- Bugfix flow must remain TDD: reproduce with a failing test first, then fix.

### Task 1: Scaffold the binary-first project skeleton

**Files:**
- Create: `D:/Rust/Excel_Skill/Cargo.toml`
- Create: `D:/Rust/Excel_Skill/src/main.rs`
- Create: `D:/Rust/Excel_Skill/src/lib.rs`
- Create: `D:/Rust/Excel_Skill/src/domain/mod.rs`
- Create: `D:/Rust/Excel_Skill/tests/common/mod.rs`

**Step 1: Write the failing smoke test**

```rust
#[test]
fn cli_without_args_returns_json_help() {
    let mut cmd = assert_cmd::Command::cargo_bin("excel_skill").unwrap();
    cmd.assert().success().stdout(predicates::str::contains("tool_catalog"));
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test cli_without_args_returns_json_help -v`
Expected: FAIL because binary and CLI response are not implemented.

**Step 3: Write minimal implementation**

```rust
fn main() {
    println!("{\"tool_catalog\":[]}");
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test cli_without_args_returns_json_help -v`
Expected: PASS.

**Step 5: Commit**

```bash
git add Cargo.toml src/main.rs src/lib.rs tests/common/mod.rs
git commit -m "feat: scaffold binary-first excel skill"
```

If `git` is not initialized yet, initialize the repository first or defer this step until repository setup is complete.

### Task 2: Define domain handles, schema state, and execution result contracts

**Files:**
- Create: `D:/Rust/Excel_Skill/src/domain/handles.rs`
- Create: `D:/Rust/Excel_Skill/src/domain/schema.rs`
- Create: `D:/Rust/Excel_Skill/src/domain/result.rs`
- Test: `D:/Rust/Excel_Skill/tests/integration_open_workbook.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn schema_state_blocks_table_operations_until_confirmed() {
    let table = TableHandle::new_pending("sales.xlsx", "Sheet1");
    assert!(!table.schema_state().is_confirmed());
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test schema_state_blocks_table_operations_until_confirmed -v`
Expected: FAIL because `TableHandle` and `SchemaState` do not exist.

**Step 3: Write minimal implementation**

```rust
pub enum SchemaState {
    Pending,
    Confirmed,
}

impl SchemaState {
    pub fn is_confirmed(&self) -> bool {
        matches!(self, Self::Confirmed)
    }
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test schema_state_blocks_table_operations_until_confirmed -v`
Expected: PASS.

**Step 5: Commit**

```bash
git add src/domain/handles.rs src/domain/schema.rs src/domain/result.rs tests/integration_open_workbook.rs
git commit -m "feat: add domain contracts for schema-gated tables"
```

### Task 3: Add Excel workbook discovery and sheet listing

**Files:**
- Create: `D:/Rust/Excel_Skill/src/excel/mod.rs`
- Create: `D:/Rust/Excel_Skill/src/excel/reader.rs`
- Test: `D:/Rust/Excel_Skill/tests/integration_open_workbook.rs`
- Fixture: `D:/Rust/Excel_Skill/tests/fixtures/basic-sales.xlsx`

**Step 1: Write the failing test**

```rust
#[test]
fn open_workbook_lists_visible_sheets() {
    let response = open_workbook("tests/fixtures/basic-sales.xlsx").unwrap();
    assert_eq!(response.sheet_names, vec!["Sales", "Lookup"]);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test open_workbook_lists_visible_sheets -v`
Expected: FAIL because Excel reader is missing.

**Step 3: Write minimal implementation**

```rust
pub fn open_workbook(path: &str) -> Result<WorkbookSummary, ExcelError> {
    let workbook = calamine::open_workbook_auto(path)?;
    Ok(WorkbookSummary::from_workbook(workbook))
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test open_workbook_lists_visible_sheets -v`
Expected: PASS.

**Step 5: Commit**

```bash
git add src/excel/mod.rs src/excel/reader.rs tests/integration_open_workbook.rs tests/fixtures/basic-sales.xlsx
git commit -m "feat: add workbook discovery"
```

### Task 4: Detect sheet regions and infer header schema with confidence

**Files:**
- Create: `D:/Rust/Excel_Skill/src/excel/regions.rs`
- Create: `D:/Rust/Excel_Skill/src/excel/header_inference.rs`
- Modify: `D:/Rust/Excel_Skill/src/domain/schema.rs`
- Test: `D:/Rust/Excel_Skill/tests/integration_header_schema.rs`
- Fixture: `D:/Rust/Excel_Skill/tests/fixtures/multi-header-sales.xlsx`
- Fixture: `D:/Rust/Excel_Skill/tests/fixtures/title-gap-header.xlsx`

**Step 1: Write the failing tests**

```rust
#[test]
fn infer_multi_row_header_builds_canonical_columns() {
    let result = infer_header_schema("tests/fixtures/multi-header-sales.xlsx", "Report").unwrap();
    assert_eq!(result.columns[0].canonical_name, "region_east_sales");
    assert!(result.confidence.is_high());
}

#[test]
fn low_confidence_header_stays_unconfirmed() {
    let result = infer_header_schema("tests/fixtures/title-gap-header.xlsx", "Sheet1").unwrap();
    assert!(!result.schema_state.is_confirmed());
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test integration_header_schema -v`
Expected: FAIL because region detection and header inference are not implemented.

**Step 3: Write minimal implementation**

```rust
pub fn infer_header_schema(...) -> Result<HeaderInferenceResult, SchemaError> {
    // Detect occupied ranges, score candidate header rows, and build canonical names.
    todo!()
}
```

**Step 4: Run tests to verify they pass**

Run: `cargo test integration_header_schema -v`
Expected: PASS with both confidence branches covered.

**Step 5: Commit**

```bash
git add src/excel/regions.rs src/excel/header_inference.rs src/domain/schema.rs tests/integration_header_schema.rs tests/fixtures/multi-header-sales.xlsx tests/fixtures/title-gap-header.xlsx
git commit -m "feat: add header inference with confidence gating"
```

### Task 5: Convert confirmed tables into Polars-backed in-memory handles

**Files:**
- Create: `D:/Rust/Excel_Skill/src/frame/mod.rs`
- Create: `D:/Rust/Excel_Skill/src/frame/registry.rs`
- Create: `D:/Rust/Excel_Skill/src/frame/conversion.rs`
- Modify: `D:/Rust/Excel_Skill/src/domain/handles.rs`
- Test: `D:/Rust/Excel_Skill/tests/integration_table_ops.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn confirmed_schema_creates_table_handle_with_dataframe() {
    let table = load_confirmed_table("tests/fixtures/basic-sales.xlsx", "Sales").unwrap();
    assert_eq!(table.row_count(), 12);
    assert!(table.dataframe().is_some());
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test confirmed_schema_creates_table_handle_with_dataframe -v`
Expected: FAIL because registry and conversion are missing.

**Step 3: Write minimal implementation**

```rust
pub fn load_confirmed_table(...) -> Result<TableHandle, FrameError> {
    // Reject pending schema, then build a DataFrame using canonical column names.
    todo!()
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test confirmed_schema_creates_table_handle_with_dataframe -v`
Expected: PASS.

**Step 5: Commit**

```bash
git add src/frame/mod.rs src/frame/registry.rs src/frame/conversion.rs src/domain/handles.rs tests/integration_table_ops.rs
git commit -m "feat: add polars-backed table registry"
```

### Task 6: Implement atomic table tools with schema-safe validation

**Files:**
- Create: `D:/Rust/Excel_Skill/src/ops/mod.rs`
- Create: `D:/Rust/Excel_Skill/src/ops/select.rs`
- Create: `D:/Rust/Excel_Skill/src/ops/filter.rs`
- Create: `D:/Rust/Excel_Skill/src/ops/group.rs`
- Create: `D:/Rust/Excel_Skill/src/ops/pivot.rs`
- Create: `D:/Rust/Excel_Skill/src/ops/export.rs`
- Test: `D:/Rust/Excel_Skill/tests/integration_table_ops.rs`

**Step 1: Write the failing tests**

```rust
#[test]
fn group_and_aggregate_sums_sales_by_region() {
    let result = group_and_aggregate(sample_table(), &["region"], &[sum("sales")]).unwrap();
    assert_eq!(result.value_for("East", "sales_sum"), 1200);
}

#[test]
fn pivot_table_creates_month_columns() {
    let result = pivot_table(sample_table(), "month", "sales", sum()).unwrap();
    assert!(result.column_names().contains(&"2026-01".to_string()));
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test integration_table_ops -v`
Expected: FAIL because atomic operations are not implemented.

**Step 3: Write minimal implementation**

```rust
pub fn group_and_aggregate(...) -> Result<TableHandle, OpError> {
    // Validate canonical columns and delegate to Polars lazy/group-by expressions.
    todo!()
}
```

**Step 4: Run tests to verify they pass**

Run: `cargo test integration_table_ops -v`
Expected: PASS.

**Step 5: Commit**

```bash
git add src/ops/mod.rs src/ops/select.rs src/ops/filter.rs src/ops/group.rs src/ops/pivot.rs src/ops/export.rs tests/integration_table_ops.rs
git commit -m "feat: add atomic table processing tools"
```

### Task 7: Define stable JSON tool contracts and dispatcher

**Files:**
- Create: `D:/Rust/Excel_Skill/src/tools/mod.rs`
- Create: `D:/Rust/Excel_Skill/src/tools/contracts.rs`
- Create: `D:/Rust/Excel_Skill/src/tools/dispatcher.rs`
- Create: `D:/Rust/Excel_Skill/docs/contracts/tool-schema.md`
- Test: `D:/Rust/Excel_Skill/tests/integration_cli_json.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn cli_dispatches_open_workbook_json_request() {
    let request = serde_json::json!({
        "tool": "open_workbook",
        "args": { "path": "tests/fixtures/basic-sales.xlsx" }
    });
    let output = run_cli(request);
    assert_eq!(output["status"], "ok");
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test cli_dispatches_open_workbook_json_request -v`
Expected: FAIL because JSON contracts and dispatcher do not exist.

**Step 3: Write minimal implementation**

```rust
pub fn dispatch(request: ToolRequest) -> ToolResponse {
    match request.tool.as_str() {
        "open_workbook" => dispatch_open_workbook(request.args),
        _ => ToolResponse::unsupported(request.tool),
    }
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test cli_dispatches_open_workbook_json_request -v`
Expected: PASS.

**Step 5: Commit**

```bash
git add src/tools/mod.rs src/tools/contracts.rs src/tools/dispatcher.rs docs/contracts/tool-schema.md tests/integration_cli_json.rs
git commit -m "feat: add json tool contracts and dispatcher"
```

### Task 8: Add task-level tools for normalization and summary reporting

**Files:**
- Create: `D:/Rust/Excel_Skill/src/tasks/mod.rs`
- Create: `D:/Rust/Excel_Skill/src/tasks/normalize_table.rs`
- Create: `D:/Rust/Excel_Skill/src/tasks/build_summary_report.rs`
- Modify: `D:/Rust/Excel_Skill/src/tools/dispatcher.rs`
- Test: `D:/Rust/Excel_Skill/tests/integration_cli_json.rs`

**Step 1: Write the failing tests**

```rust
#[test]
fn normalize_table_standardizes_headers_and_types() {
    let output = run_cli(json!({
        "tool": "normalize_table",
        "args": { "path": "tests/fixtures/title-gap-header.xlsx", "sheet": "Sheet1" }
    }));
    assert_eq!(output["status"], "needs_confirmation");
}

#[test]
fn build_summary_report_exports_excel_file() {
    let output = run_cli(json!({
        "tool": "build_summary_report",
        "args": { "path": "tests/fixtures/basic-sales.xlsx", "sheet": "Sales", "group_by": ["region"], "metrics": ["sales:sum"] }
    }));
    assert_eq!(output["status"], "ok");
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test integration_cli_json -v`
Expected: FAIL because task-level tools are not implemented.

**Step 3: Write minimal implementation**

```rust
pub fn build_summary_report(args: SummaryArgs) -> Result<ToolResponse, TaskError> {
    // Reuse lower-level operations directly, not via nested tool API calls.
    todo!()
}
```

**Step 4: Run tests to verify they pass**

Run: `cargo test integration_cli_json -v`
Expected: PASS.

**Step 5: Commit**

```bash
git add src/tasks/mod.rs src/tasks/normalize_table.rs src/tasks/build_summary_report.rs src/tools/dispatcher.rs tests/integration_cli_json.rs
git commit -m "feat: add task-level excel tools"
```

### Task 9: Add execution tracing, error taxonomy, and user-safe confirmations

**Files:**
- Create: `D:/Rust/Excel_Skill/src/trace/mod.rs`
- Create: `D:/Rust/Excel_Skill/src/trace/execution.rs`
- Modify: `D:/Rust/Excel_Skill/src/domain/result.rs`
- Modify: `D:/Rust/Excel_Skill/src/tools/contracts.rs`
- Test: `D:/Rust/Excel_Skill/tests/integration_cli_json.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn low_confidence_schema_returns_confirmation_payload() {
    let output = run_cli(json!({
        "tool": "normalize_table",
        "args": { "path": "tests/fixtures/title-gap-header.xlsx", "sheet": "Sheet1" }
    }));
    assert_eq!(output["status"], "needs_confirmation");
    assert!(output["trace"].is_array());
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test low_confidence_schema_returns_confirmation_payload -v`
Expected: FAIL because trace and confirmation envelope are incomplete.

**Step 3: Write minimal implementation**

```rust
pub struct ExecutionTrace {
    pub steps: Vec<TraceStep>,
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test low_confidence_schema_returns_confirmation_payload -v`
Expected: PASS.

**Step 5: Commit**

```bash
git add src/trace/mod.rs src/trace/execution.rs src/domain/result.rs src/tools/contracts.rs tests/integration_cli_json.rs
git commit -m "feat: add trace and confirmation responses"
```

### Task 10: Package and verify the no-deployment binary experience

**Files:**
- Modify: `D:/Rust/Excel_Skill/Cargo.toml`
- Create: `D:/Rust/Excel_Skill/README.md`
- Create: `D:/Rust/Excel_Skill/scripts/package.ps1`
- Test: `D:/Rust/Excel_Skill/tests/integration_cli_json.rs`

**Step 1: Write the failing packaging check**

```powershell
$binary = "target/release/excel_skill.exe"
if (-not (Test-Path $binary)) { throw "binary missing" }
```

**Step 2: Run verification to show packaging is incomplete**

Run: `cargo build --release`
Expected: first build may fail until all dependencies and features are correctly configured.

**Step 3: Write minimal implementation**

```powershell
cargo build --release
Compress-Archive -Path target/release/excel_skill.exe -DestinationPath dist/excel-skill-win64.zip -Force
```

**Step 4: Run packaging verification**

Run: `powershell -ExecutionPolicy Bypass -File scripts/package.ps1`
Expected: PASS and produce a zip containing a single executable plus minimal docs.

**Step 5: Commit**

```bash
git add Cargo.toml README.md scripts/package.ps1
git commit -m "chore: package single-binary release"
```

## Risks To Watch

- Header inference may overfit early fixtures and fail on merged cells or decorative report titles.
- Polars type casting may differ from Excel user expectations for dates, percentages, and mixed text-number columns.
- Excel write-back may preserve data only, not workbook formatting; this must be explicit in V1 docs.
- Large workbooks may require lazy loading or chunking later; V1 should document expected memory behavior.
- JSON tool contracts must remain stable once the Skill starts depending on them.

## Test Matrix

- Single-row header workbook
- Multi-row header workbook
- Title row + blank row + header row workbook
- Multiple tables in one sheet
- Missing values, duplicate rows, mixed numeric/text columns
- Group aggregation, pivot, export roundtrip
- CLI JSON success, validation failure, and confirmation-needed flows
- Release build and packaged single-binary smoke run on a clean machine

## Suggested First Tool Catalog

- `open_workbook`
- `list_sheets`
- `detect_sheet_regions`
- `infer_header_schema`
- `apply_header_schema`
- `inspect_table`
- `select_columns`
- `filter_rows`
- `sort_rows`
- `rename_columns`
- `cast_column_types`
- `fill_missing_values`
- `drop_missing_values`
- `deduplicate_rows`
- `create_derived_column`
- `group_and_aggregate`
- `join_tables`
- `pivot_table`
- `unpivot_table`
- `preview_table`
- `export_excel`
- `export_csv`
- `normalize_table`
- `build_summary_report`

## Binary Distribution Options

1. **Single CLI executable (recommended)**
   - Pros: easiest distribution, easiest local Skill invocation, no service lifecycle
   - Cons: each invocation is a cold start, in-memory state must be reloaded or persisted explicitly
2. **Local daemon executable**
   - Pros: keeps workbook/table state warm, faster repeated operations
   - Cons: user must manage a background process, troubleshooting is harder for non-technical Excel users
3. **Embedded host plugin**
   - Pros: best UX inside a future app shell
   - Cons: highest integration cost, least suitable for V1 validation

Use option 1 for V1. Design internal modules so option 2 can be added later without changing tool contracts.
