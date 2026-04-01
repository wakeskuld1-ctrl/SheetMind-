# CLI Modularization Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Refactor the internal CLI tool dispatch code into cohesive modules while keeping all external tool names, JSON request/response contracts, and CLI behavior unchanged.

**Architecture:** Keep `main.rs` as the single binary entry point and preserve `ToolRequest`/`ToolResponse` as the only public CLI protocol. Introduce a centralized tool registry plus focused helper modules for session synchronization and source resolution so `dispatcher.rs` becomes a thin routing layer rather than a god file.

**Tech Stack:** Rust 2024, serde/serde_json, rusqlite, calamine, polars, assert_cmd

---

### Task 1: Lock the Current CLI Contract

**Files:**
- Modify: `tests/integration_cli_json.rs`
- Modify: `tests/common/mod.rs`

**Step 1: Write the failing test**

Add a test that compares the CLI catalog output with a centralized registry function that does not exist yet.

```rust
#[test]
fn cli_tool_catalog_matches_registered_tool_names() {
    let output = run_cli_with_json("");
    let catalog = output["data"]["tool_catalog"].as_array().unwrap();
    let expected = excel_skill::tools::catalog::tool_names();
    assert_eq!(catalog.len(), expected.len());
    for (actual, expected_name) in catalog.iter().zip(expected.iter()) {
        assert_eq!(actual.as_str().unwrap(), *expected_name);
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --test integration_cli_json cli_tool_catalog_matches_registered_tool_names -- --exact`

Expected: FAIL because `tools::catalog` does not exist yet.

**Step 3: Write minimal implementation**

Create `src/tools/catalog.rs` with a stable ordered list of tool names and expose `tool_names()`.

**Step 4: Run test to verify it passes**

Run: `cargo test --test integration_cli_json cli_tool_catalog_matches_registered_tool_names -- --exact`

Expected: PASS

### Task 2: Centralize the Tool Catalog

**Files:**
- Create: `src/tools/catalog.rs`
- Modify: `src/tools/mod.rs`
- Modify: `src/tools/contracts.rs`
- Modify: `src/tools/dispatcher.rs`

**Step 1: Write the failing test**

Add a test proving `ToolResponse::tool_catalog()` preserves registry order and count.

```rust
#[test]
fn tool_catalog_response_uses_registry_order() {
    let response = excel_skill::tools::contracts::ToolResponse::tool_catalog();
    let catalog = response.data["tool_catalog"].as_array().unwrap();
    let expected = excel_skill::tools::catalog::tool_names();
    assert_eq!(catalog.len(), expected.len());
    assert_eq!(catalog[0].as_str().unwrap(), expected[0]);
    assert_eq!(catalog.last().unwrap().as_str().unwrap(), *expected.last().unwrap());
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --test integration_cli_json tool_catalog_response_uses_registry_order -- --exact`

Expected: FAIL until `contracts.rs` is updated to use the new registry.

**Step 3: Write minimal implementation**

Update `ToolResponse::tool_catalog()` to build JSON from `catalog::tool_names()`. Update `dispatcher.rs` to delegate route selection to the registry for at least tool existence checks / naming stability.

**Step 4: Run test to verify it passes**

Run: `cargo test --test integration_cli_json tool_catalog_response_uses_registry_order -- --exact`

Expected: PASS

### Task 3: Extract Session Sync Helpers

**Files:**
- Create: `src/tools/session.rs`
- Modify: `src/tools/mod.rs`
- Modify: `src/tools/dispatcher.rs`
- Test: `tests/integration_cli_json.rs`

**Step 1: Write the failing test**

Add a focused regression test that exercises a flow using session state after a tool call, proving behavior remains unchanged while internals move.

```rust
#[test]
fn load_table_region_still_updates_session_state_after_session_helper_extraction() {
    // Existing fixture setup pattern, then assert current_stage/schema_status/table_ref.
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --test integration_cli_json load_table_region_still_updates_session_state_after_session_helper_extraction -- --exact`

Expected: FAIL if the new helper module is not yet wired correctly.

**Step 3: Write minimal implementation**

Move session-state-related helper functions out of `dispatcher.rs` into `tools/session.rs`:
- `memory_runtime`
- `session_id_from_args`
- `user_goal_from_args`
- `selected_columns_from_args`
- `active_handle_ref_from_args`
- `current_file_ref_from_args`
- `current_sheet_index_from_args`
- `sync_confirmed_table_state`
- `sync_loaded_table_state`
- `sync_output_handle_state`

Keep signatures stable where possible.

**Step 4: Run test to verify it passes**

Run: `cargo test --test integration_cli_json load_table_region_still_updates_session_state_after_session_helper_extraction -- --exact`

Expected: PASS

### Task 4: Extract Source Resolution and Result Persistence Helpers

**Files:**
- Create: `src/tools/sources.rs`
- Create: `src/tools/results.rs`
- Modify: `src/tools/mod.rs`
- Modify: `src/tools/dispatcher.rs`
- Test: `tests/integration_cli_json.rs`

**Step 1: Write the failing test**

Add one flow test for `inspect_sheet_range` using `file_ref + sheet_index`, plus one result-producing flow that asserts `result_ref` still appears.

**Step 2: Run test to verify it fails**

Run:
- `cargo test --test integration_cli_json inspect_sheet_range_accepts_file_ref_and_sheet_index -- --exact`
- `cargo test --test integration_cli_json load_table_region_returns_preview_and_result_ref -- --exact`

Expected: At least one failure while helper extraction is incomplete.

**Step 3: Write minimal implementation**

Move these helpers out of `dispatcher.rs`:
- sheet source resolution helpers
- result persistence helpers
- source ref collection helpers

Keep `dispatcher.rs` responsible only for tool-specific orchestration.

**Step 4: Run test to verify it passes**

Run the two exact tests above again.

Expected: PASS

### Task 5: Thin the Dispatcher Surface

**Files:**
- Modify: `src/tools/dispatcher.rs`
- Optionally create: `src/tools/handlers/*.rs`
- Test: `tests/integration_cli_json.rs`, `tests/integration_open_workbook.rs`, `tests/integration_registry.rs`

**Step 1: Write the failing test**

Add a focused smoke test around one high-value tool at the start and one at the end of the catalog to protect dispatch routing.

**Step 2: Run test to verify it fails**

Run targeted `cargo test` commands for the new smoke test(s).

Expected: FAIL if routing breaks during the thinning step.

**Step 3: Write minimal implementation**

Reduce `dispatcher.rs` to:
- a routing function
- domain-level handler imports
- minimal glue only

Do not change CLI protocol, tool names, or persisted handle semantics.

**Step 4: Run test to verify it passes**

Run:
- `cargo test --test integration_cli_json -- --nocapture`
- `cargo test --test integration_open_workbook -- --nocapture`
- `cargo test --test integration_registry -- --nocapture`

Expected: PASS

### Task 6: Verification and Cleanup

**Files:**
- Modify: `docs/plans/2026-03-23-cli-modularization-plan.md`
- Modify: `progress.md`
- Modify: `findings.md`

**Step 1: Run formatting**

Run: `cargo fmt`

**Step 2: Run targeted tests**

Run the narrowest passing set available in the environment.

**Step 3: Record residual risks**

Document any tests that could not be run because of environment or dependency issues.
