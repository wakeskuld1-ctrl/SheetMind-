# 2026-04-12 Foundation Hygiene Grouped Views Handoff

## Scope

This slice stays inside the foundation knowledge-roaming layer.

- Module: `src/ops/foundation/repository_metadata_audit.rs`
- Test: `tests/repository_metadata_audit_unit.rs`
- Goal: add grouped evidence-hygiene views on top of existing raw diagnostics and summary output
- Non-goal: no business-layer workflow, no auto-remediation, no architecture rewrite

## What Changed

`RepositoryMetadataAuditReport` now exposes three hygiene layers:

1. `hygiene_diagnostics`
2. `hygiene_summary`
3. `hygiene_views`

`hygiene_views` contains:

- `by_severity`
  - `severity`
  - `diagnostic_count`
  - `affected_node_count`
  - `node_ids`
- `by_node`
  - `node_id`
  - `highest_severity`
  - `diagnostic_count`
  - `diagnostic_type_counts`

## Rules

### Severity Mapping

- `MissingEvidenceRef` -> `Critical`
- `WeakSourceRef(Blank | TooShort | MissingNamespace | EntityMissing)` -> `Critical`
- `DuplicateEvidenceRefWithinNode` -> `Warning`
- `DuplicateEvidenceRef` -> `Warning`
- `WeakLocator(*)` -> `Warning`
- `WeakSourceRef(ContainsWhitespace | InvalidCharacter | UnknownNamespace)` -> `Warning`

### Sort Order

- `by_severity`: `Critical -> Warning -> Info`
- `by_node`:
  - higher severity first
  - then larger `diagnostic_count`
  - then `node_id` ascending

## Why This Matters

This gives downstream AI a stable routing surface:

- raw list for detailed inspection
- summary for repository-level blocking checks
- grouped views for prioritizing remediation order

That keeps the foundation output machine-friendly without pushing the module into business orchestration.

## Verification

Verified with:

- `cargo test --test repository_metadata_audit_unit -- --nocapture`
- `cargo test --test metadata_validator_unit -- --nocapture`
- `cargo test --test metadata_registry_unit -- --nocapture`
- `cargo test --test metadata_scope_resolver_unit -- --nocapture`
- `cargo test --test knowledge_record_unit -- --nocapture`
- `cargo test --test knowledge_graph_store_unit -- --nocapture`
- `cargo test --test ontology_schema_unit -- --nocapture`
- `cargo test --test ontology_store_unit -- --nocapture`

## Follow-up

Recommended next foundation slices:

- add grouped view support for diagnostic reason drill-down if AI routing needs faster triage entry points
- add export-facing DTO shaping only if an external contract truly needs a narrower response shape
- keep future work incremental; do not reopen architecture refactors for this module without a concrete failure
