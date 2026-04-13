# Foundation Hygiene Severity Summary Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add a structured hygiene summary to the foundation repository metadata audit so the knowledge roaming layer can consume evidence hygiene severity and aggregate counts directly.

**Architecture:** Keep the existing `hygiene_diagnostics` detail list unchanged and add a parallel `hygiene_summary` aggregation layer inside `RepositoryMetadataAuditReport`. Implement the summary locally in `repository_metadata_audit.rs` with small helper functions so the change stays inside foundation governance boundaries.

**Tech Stack:** Rust, cargo test, foundation repository metadata audit unit tests

---

### Task 1: Add the failing hygiene-summary test assertions

**Files:**
- Modify: `D:\Rust\Excel_Skill\tests\repository_metadata_audit_unit.rs`
- Test: `D:\Rust\Excel_Skill\tests\repository_metadata_audit_unit.rs`

**Step 1: Write the failing test**

- Add assertions for:
  - `report.hygiene_summary.total_diagnostics`
  - `report.hygiene_summary.severity_counts`
  - `report.hygiene_summary.diagnostic_type_counts`
  - `report.hygiene_summary.weak_locator_reason_counts`
  - `report.hygiene_summary.weak_source_ref_reason_counts`
  - `report.hygiene_summary.affected_node_count`
  - `report.hygiene_summary.has_blocking_hygiene_issue`

**Step 2: Run test to verify it fails**

Run: `cargo test --test repository_metadata_audit_unit -- --nocapture`

Expected: compile or assertion failure because `hygiene_summary` does not exist yet.

### Task 2: Implement the summary model

**Files:**
- Modify: `D:\Rust\Excel_Skill\src\ops\foundation\repository_metadata_audit.rs`

**Step 1: Write minimal implementation**

- Add:
  - `RepositoryHygieneSeverity`
  - `RepositoryEvidenceHygieneSummary`
  - `hygiene_summary` field on `RepositoryMetadataAuditReport`

**Step 2: Add helper functions**

- Add minimal helpers for:
  - severity mapping
  - diagnostic type key mapping
  - per-reason aggregation
  - affected node deduplication

**Step 3: Wire summary generation**

- Build `hygiene_summary` from `hygiene_diagnostics` inside `audit(...)`

### Task 3: Verify green

**Files:**
- Test: `D:\Rust\Excel_Skill\tests\repository_metadata_audit_unit.rs`

**Step 1: Run focused test**

Run: `cargo test --test repository_metadata_audit_unit -- --nocapture`

Expected: PASS

### Task 4: Run foundation regression

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

### Task 5: Update handoff and task journal

**Files:**
- Create: `D:\Rust\Excel_Skill\docs\ai-handoff-2026-04-12-foundation-hygiene-severity-summary.md`
- Modify: `D:\Rust\Excel_Skill\.trae\CHANGELOG_TASK.md`

**Step 1: Add handoff note**

- Record:
  - scope
  - summary model added
  - severity mapping rules
  - verification commands

**Step 2: Append task journal entry**

- Append this round’s implementation notes using the fixed task-journal structure.

### Task 6: Final verification before completion

**Files:**
- Review only

**Step 1: Re-check diff and verification evidence**

Run:
- `git diff -- src/ops/foundation/repository_metadata_audit.rs tests/repository_metadata_audit_unit.rs docs/plans/2026-04-12-foundation-hygiene-severity-summary-design.md docs/plans/2026-04-12-foundation-hygiene-severity-summary-plan.md docs/ai-handoff-2026-04-12-foundation-hygiene-severity-summary.md .trae/CHANGELOG_TASK.md`

Expected: only this task’s intended files are changed for the feature area.
