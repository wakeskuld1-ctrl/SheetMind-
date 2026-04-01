# Local Memory Runtime V1 Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build a minimal SQLite-backed local memory runtime and connect the orchestrator's lightweight session summary to real local persistence.

**Architecture:** Keep Skill files as routing protocol only, and add a Rust runtime layer that stores session state and mirrored table-ref metadata in a local SQLite database under the workspace runtime directory. Expose explicit session read/write tools for orchestrator use, and let key existing tools automatically enrich session state so cross-layer handoff can be resumed in later requests.

**Tech Stack:** Rust, `rusqlite` with bundled SQLite, existing JSON CLI contract tests, existing `table_ref` JSON store.

---

### Task 1: Define the local runtime surface

**Files:**
- Create: `D:/Rust/Excel_Skill/src/runtime/mod.rs`
- Create: `D:/Rust/Excel_Skill/src/runtime/local_memory.rs`
- Modify: `D:/Rust/Excel_Skill/src/lib.rs`
- Modify: `D:/Rust/Excel_Skill/Cargo.toml`

**Step 1: Write the failing runtime round-trip test**
- Add a test that writes session state into the runtime and reads it back.

**Step 2: Run the test to verify it fails**
Run: `cargo test runtime_persists_session_state_round_trip --test integration_registry -v`
Expected: FAIL because runtime module and APIs do not exist yet.

**Step 3: Write the minimal runtime implementation**
- Add a SQLite-backed runtime store.
- Add schema bootstrap for `sessions`, `session_state`, `table_refs`, `event_logs`.
- Keep the first implementation narrow: only session summary, mirrored table-ref metadata, and event append.

**Step 4: Run the test to verify it passes**
Run: `cargo test runtime_persists_session_state_round_trip --test integration_registry -v`
Expected: PASS.

### Task 2: Expose orchestrator session tools

**Files:**
- Modify: `D:/Rust/Excel_Skill/src/tools/contracts.rs`
- Modify: `D:/Rust/Excel_Skill/src/tools/dispatcher.rs`
- Test: `D:/Rust/Excel_Skill/tests/integration_cli_json.rs`

**Step 1: Write the failing CLI tests**
- Add `get_session_state` and `update_session_state` CLI tests.
- Verify an update in one CLI request can be read in a later request.

**Step 2: Run the targeted tests to verify they fail**
Run: `cargo test get_session_state_returns_persisted_update_from_previous_request --test integration_cli_json -v`
Expected: FAIL because the tools are not registered yet.

**Step 3: Write the minimal dispatcher support**
- Register the two new tools.
- Parse optional `session_id`, defaulting to `default`.
- Return stable JSON shaped for Skill consumption.

**Step 4: Run the targeted tests to verify they pass**
Run: `cargo test get_session_state_returns_persisted_update_from_previous_request --test integration_cli_json -v`
Expected: PASS.

### Task 3: Auto-sync key tool transitions into local memory

**Files:**
- Modify: `D:/Rust/Excel_Skill/src/tools/dispatcher.rs`
- Modify: `D:/Rust/Excel_Skill/src/frame/table_ref_store.rs`
- Test: `D:/Rust/Excel_Skill/tests/integration_cli_json.rs`

**Step 1: Write the failing bridge tests**
- Add a test proving `apply_header_schema` updates the active confirmed table state.
- Add a test proving `stat_summary(table_ref)` advances the stage to `analysis_modeling`.
- Add a test proving `decision_assistant(table_ref)` advances the stage to `decision_assistant`.

**Step 2: Run the targeted tests to verify they fail**
Run: `cargo test apply_header_schema_updates_session_state_and_active_table_ref --test integration_cli_json -v`
Expected: FAIL because existing tools do not sync runtime state yet.

**Step 3: Write the minimal sync implementation**
- Mirror saved `table_ref` metadata into SQLite when confirmation succeeds.
- Update session summary after key tool calls.
- Append lightweight event logs for these transitions.

**Step 4: Run the targeted tests to verify they pass**
Run: `cargo test apply_header_schema_updates_session_state_and_active_table_ref --test integration_cli_json -v`
Expected: PASS.

### Task 4: Skill and docs convergence

**Files:**
- Modify: `D:/Rust/Excel_Skill/skills/excel-orchestrator-v1/SKILL.md`
- Modify: `D:/Rust/Excel_Skill/skills/excel-orchestrator-v1/requests.md`
- Modify: `D:/Rust/Excel_Skill/task_plan.md`
- Modify: `D:/Rust/Excel_Skill/findings.md`
- Modify: `D:/Rust/Excel_Skill/progress.md`
- Modify: `D:/Rust/Excel_Skill/.trae/CHANGELOG_TASK.md`

**Step 1: Update the Skill docs**
- Document that the orchestrator now reads `get_session_state` first and updates state through `update_session_state` plus key tool auto-sync.

**Step 2: Run verification**
Run: `cargo test -v`
Expected: PASS.

**Step 3: Record the result**
- Update planning files and task journal with implementation, verification, and remaining follow-up.
