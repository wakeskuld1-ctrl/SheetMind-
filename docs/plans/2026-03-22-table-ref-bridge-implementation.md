# Table Ref Bridge Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build a persistent `table_ref` bridge so table-processing confirmation state can be reused by analysis-modeling tools without re-running schema inference.

**Architecture:** Keep existing `path + sheet` requests working, and add a durable `table_ref` path. `apply_header_schema` will persist a confirmed table reference record to disk; analysis-modeling tools will accept `table_ref` and load confirmed schema from that record instead of calling `infer_header_schema(...)` again. The reference store will validate source file metadata to avoid silently using stale confirmation results after the workbook changes.

**Tech Stack:** Rust, serde/serde_json, std::fs, existing calamine + polars tool pipeline, existing CLI integration tests.

---

### Task 1: Add failing tests for persistent `table_ref`

**Files:**
- Modify: `D:\Rust\Excel_Skill\tests\integration_cli_json.rs`
- Modify: `D:\Rust\Excel_Skill\tests\integration_registry.rs`
- Test: `D:\Rust\Excel_Skill\tests\integration_cli_json.rs`
- Test: `D:\Rust\Excel_Skill\tests\integration_registry.rs`

**Step 1: Write failing tests**
- Add a CLI test proving `apply_header_schema` returns a reusable `table_ref`.
- Add a CLI test proving `stat_summary` accepts `table_ref` and no longer returns `needs_confirmation` on an originally ambiguous sheet.
- Add a CLI test proving one model tool accepts `table_ref` and reaches actual model execution.
- Add a registry/store test proving a stored table reference can round-trip through disk.

**Step 2: Run tests to verify they fail**

Run:

```powershell
cargo test apply_header_schema_returns_reusable_table_ref --test integration_cli_json -- --exact
cargo test stat_summary_accepts_table_ref_from_apply_header_schema --test integration_cli_json -- --exact
cargo test linear_regression_accepts_table_ref_from_apply_header_schema --test integration_cli_json -- --exact
cargo test stored_table_ref_round_trips_through_disk --test integration_registry -- --exact
```

Expected:
- FAIL because `table_ref` does not exist yet.

### Task 2: Introduce persistent table reference types and store

**Files:**
- Create: `D:\Rust\Excel_Skill\src\frame\table_ref_store.rs`
- Modify: `D:\Rust\Excel_Skill\src\frame\mod.rs`
- Modify: `D:\Rust\Excel_Skill\src\domain\handles.rs`
- Modify: `D:\Rust\Excel_Skill\src\domain\schema.rs`
- Test: `D:\Rust\Excel_Skill\tests\integration_registry.rs`

**Step 1: Write minimal persistent record design**
- Define a serializable confirmed table reference record containing:
  - `table_ref`
  - `source_path`
  - `sheet_name`
  - `columns`
  - `header_row_count`
  - `data_start_row_index`
  - source file metadata needed for staleness checks

**Step 2: Implement disk-backed store**
- Persist records under a dedicated runtime directory inside the workspace.
- Support save/load operations and a simple “source file changed” validation.

**Step 3: Run narrow tests**

Run:

```powershell
cargo test stored_table_ref_round_trips_through_disk --test integration_registry -- --exact
```

Expected:
- PASS

### Task 3: Make `apply_header_schema` emit durable `table_ref`

**Files:**
- Modify: `D:\Rust\Excel_Skill\src\tools\dispatcher.rs`
- Modify: `D:\Rust\Excel_Skill\src\frame\loader.rs`
- Test: `D:\Rust\Excel_Skill\tests\integration_cli_json.rs`

**Step 1: Implement minimal production code**
- When `apply_header_schema` succeeds:
  - keep current confirmed loading validation
  - persist a confirmed table reference
  - return `table_ref` in JSON while preserving current response fields

**Step 2: Run targeted test**

Run:

```powershell
cargo test apply_header_schema_returns_reusable_table_ref --test integration_cli_json -- --exact
```

Expected:
- PASS

### Task 4: Teach analysis/modeling tools to accept `table_ref`

**Files:**
- Modify: `D:\Rust\Excel_Skill\src\tools\dispatcher.rs`
- Modify: `D:\Rust\Excel_Skill\src\frame\loader.rs`
- Test: `D:\Rust\Excel_Skill\tests\integration_cli_json.rs`

**Step 1: Extend unified load path**
- Refactor current load path so tools can resolve input from either:
  - `table_ref`
  - or legacy `path + sheet`
- When `table_ref` is present:
  - load persisted confirmed schema
  - validate source metadata
  - load the sheet with confirmed schema directly
  - skip `infer_header_schema(...)`

**Step 2: Run targeted tests**

Run:

```powershell
cargo test stat_summary_accepts_table_ref_from_apply_header_schema --test integration_cli_json -- --exact
cargo test linear_regression_accepts_table_ref_from_apply_header_schema --test integration_cli_json -- --exact
```

Expected:
- PASS

### Task 5: Add stale-reference protection and compatibility checks

**Files:**
- Modify: `D:\Rust\Excel_Skill\src\frame\table_ref_store.rs`
- Modify: `D:\Rust\Excel_Skill\src\tools\dispatcher.rs`
- Test: `D:\Rust\Excel_Skill\tests\integration_cli_json.rs`

**Step 1: Add failing test**
- Add a test proving invalid or stale `table_ref` returns a clear error instead of silently falling back.

**Step 2: Implement minimal protection**
- If `table_ref` cannot be loaded or source metadata changed:
  - return a clear error asking the user to reconfirm the sheet

**Step 3: Run targeted test**

Run:

```powershell
cargo test stat_summary_rejects_stale_table_ref --test integration_cli_json -- --exact
```

Expected:
- PASS

### Task 6: Real-file regression re-test

**Files:**
- Modify: `D:\Rust\Excel_Skill\docs\acceptance\2026-03-22-analysis-modeling-skill-e2e-real-file.md`
- Modify: `D:\Rust\Excel_Skill\docs\acceptance\artifacts\2026-03-22-analysis-modeling-skill-e2e-real-file\*`
- Modify: `D:\Rust\Excel_Skill\findings.md`
- Modify: `D:\Rust\Excel_Skill\progress.md`

**Step 1: Re-run the real workbook flow**
- Use `apply_header_schema` to obtain a real `table_ref`.
- Re-run `stat_summary` and analysis/modeling calls through `table_ref`.
- Save fresh JSON artifacts.

**Step 2: Update acceptance record**
- Record what moved from `needs_confirmation` to actual execution.
- Record any remaining blockers honestly.

**Step 3: Final verification**

Run:

```powershell
cargo test -v
```

Expected:
- All tests pass
