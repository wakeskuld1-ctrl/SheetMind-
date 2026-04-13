# 2026-04-12 Foundation Hygiene Reason Views Handoff

## Scope

This slice stays inside the foundation knowledge-roaming layer.

- Module: `src/ops/foundation/repository_metadata_audit.rs`
- Test: `tests/repository_metadata_audit_unit.rs`
- Goal: add reason-first hygiene governance views on top of existing diagnostics, summary, and grouped views
- Non-goal: no business workflow, no auto-remediation, no architecture rewrite

## What Changed

`RepositoryMetadataAuditReport` now also exposes:

- `hygiene_reason_views`
  - `weak_locator_by_reason`
  - `weak_source_ref_by_reason`

Each reason group contains:

- `reason`
- `severity`
- `diagnostic_count`
- `affected_node_count`
- `node_ids`

## Rules

### Ordering

- `weak_locator_by_reason`
  - severity rank
  - then `diagnostic_count` descending
  - then reason key ascending

- `weak_source_ref_by_reason`
  - severity rank
  - then `diagnostic_count` descending
  - then reason key ascending

### Severity

- all `WeakLocator` reason groups are `Warning`
- `WeakSourceRef` reasons:
  - `Blank`
  - `TooShort`
  - `MissingNamespace`
  - `EntityMissing`
  - map to `Critical`
  - `ContainsWhitespace`
  - `InvalidCharacter`
  - `UnknownNamespace`
  - map to `Warning`

## Why This Matters

The foundation report can now answer four different governance questions directly:

- what exact diagnostics exist
- whether the repository has blocking hygiene issues
- which nodes should be fixed first
- which weak-cause families should be fixed first

That makes the roaming foundation easier for AI to route without rebuilding one more aggregation layer outside the audit module.

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

- add drill-down export only if an external contract truly needs a slimmer output than the internal audit model
- add large-repository stability coverage if grouped governance views start handling much bigger bundles
- keep new governance views additive; do not reopen broad refactors unless a concrete contract problem appears
