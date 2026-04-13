# Foundation Hygiene Stability Boundary Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add large-repository stability coverage and boundary-semantics coverage for foundation repository hygiene audit so future governance changes cannot silently drift sorting or aggregation behavior.

**Architecture:** Keep the current audit contract intact and extend the focused unit test file with two new fixtures: one wide fixture for large-sample stability and one narrow fixture for edge-case semantics. Only patch production code if the new red tests expose a real semantic gap.

**Tech Stack:** Rust, cargo test, foundation repository metadata audit unit tests

---

### Task 1: Add the failing large-repository stability test

**Files:**
- Modify: `D:\Rust\Excel_Skill\tests\repository_metadata_audit_unit.rs`

**Step 1: Write the failing test**

- Add one new test that builds a larger repository fixture and asserts:
  - summary counts
  - grouped severity ordering
  - grouped node ordering
  - reason-view ordering and node aggregation

**Step 2: Run test to verify it fails**

Run: `cargo test --test repository_metadata_audit_unit -- --nocapture`

Expected: FAIL because the current implementation or fixture support does not yet lock this larger behavior.

### Task 2: Add the failing boundary-semantics test

**Files:**
- Modify: `D:\Rust\Excel_Skill\tests\repository_metadata_audit_unit.rs`

**Step 1: Write the failing test**

- Add one edge-case test that builds a repeated-reason fixture and asserts:
  - reason summary counts
  - reason-view `diagnostic_count`
  - reason-view `affected_node_count`
  - deduplicated node list behavior

**Step 2: Run test to verify it fails**

Run: `cargo test --test repository_metadata_audit_unit -- --nocapture`

Expected: FAIL if current aggregation semantics are not yet explicit enough.

### Task 3: Implement the minimal production fix if needed

**Files:**
- Modify: `D:\Rust\Excel_Skill\src\ops\foundation\repository_metadata_audit.rs`
- Modify: `D:\Rust\Excel_Skill\tests\repository_metadata_audit_unit.rs`

**Step 1: Add only the minimal helper or aggregation fix**

- Patch production aggregation only if the red tests reveal:
  - unstable sorting
  - mismatched node deduplication
  - ambiguous diagnostic-vs-node counting semantics

**Step 2: Keep fixture support minimal**

- Add narrow helper builders inside the test file only as needed to avoid large copy-paste fixtures.

### Task 4: Verify green

**Files:**
- Test: `D:\Rust\Excel_Skill\tests\repository_metadata_audit_unit.rs`

**Step 1: Run focused test**

Run: `cargo test --test repository_metadata_audit_unit -- --nocapture`

Expected: PASS

### Task 5: Run foundation regression

**Files:**
- Test:
  - `D:\Rust\Excel_Skill\tests\metadata_validator_unit.rs`
  - `D:\Rust\Excel_Skill\tests\metadata_registry_unit.rs`
  - `D:\Rust\Excel_Skill\tests\metadata_scope_resolver_unit.rs`
  - `D:\Rust\Excel_Skill\tests\knowledge_record_unit.rs`
  - `D:\Rust\Excel_Skill\tests\knowledge_graph_store_unit.rs`
  - `D:\Rust\Excel_Skill\tests\ontology_schema_unit.rs`
  - `D:\Rust\Excel_Skill\tests\ontology_store_unit.rs`

**Step 1: Run regression commands**

Run:
- `cargo test --test metadata_validator_unit -- --nocapture`
- `cargo test --test metadata_registry_unit -- --nocapture`
- `cargo test --test metadata_scope_resolver_unit -- --nocapture`
- `cargo test --test knowledge_record_unit -- --nocapture`
- `cargo test --test knowledge_graph_store_unit -- --nocapture`
- `cargo test --test ontology_schema_unit -- --nocapture`
- `cargo test --test ontology_store_unit -- --nocapture`

Expected: PASS

### Task 6: Update handoff and task journal

**Files:**
- Create: `D:\Rust\Excel_Skill\docs\ai-handoff-2026-04-12-foundation-hygiene-stability-boundary.md`
- Modify: `D:\Rust\Excel_Skill\.trae\CHANGELOG_TASK.md`

**Step 1: Add handoff note**

- Record:
  - new stability fixture scope
  - boundary-semantics conclusions
  - whether production aggregation changed
  - verification commands

**Step 2: Append task journal entry**

- Append this round with the fixed task-journal structure.

### Task 7: Final verification before completion

**Files:**
- Review only

**Step 1: Re-check diff**

Run:
- `git diff -- src/ops/foundation/repository_metadata_audit.rs tests/repository_metadata_audit_unit.rs docs/plans/2026-04-12-foundation-hygiene-stability-boundary-design.md docs/plans/2026-04-12-foundation-hygiene-stability-boundary-plan.md docs/ai-handoff-2026-04-12-foundation-hygiene-stability-boundary.md .trae/CHANGELOG_TASK.md`

Expected: only stability-and-boundary task files are included for this slice.
