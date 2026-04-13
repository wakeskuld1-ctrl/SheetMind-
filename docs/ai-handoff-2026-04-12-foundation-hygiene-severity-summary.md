# 2026-04-12 Foundation Hygiene Severity Summary Handoff

## Scope
- Mainline: `foundation`
- Layer: knowledge roaming repository governance
- This round only adds structured hygiene summary output on top of existing repository audit diagnostics.

## What Changed
- Added `RepositoryHygieneSeverity`
- Added `RepositoryEvidenceHygieneSummary`
- Added `hygiene_summary` to `RepositoryMetadataAuditReport`
- Kept existing `hygiene_diagnostics` detail output unchanged

## Summary Fields
- `total_diagnostics`
- `severity_counts`
- `diagnostic_type_counts`
- `weak_locator_reason_counts`
- `weak_source_ref_reason_counts`
- `affected_node_count`
- `has_blocking_hygiene_issue`

## Severity Mapping

### Critical
- `MissingEvidenceRef`
- `WeakSourceRef::Blank`
- `WeakSourceRef::TooShort`
- `WeakSourceRef::MissingNamespace`
- `WeakSourceRef::EntityMissing`

### Warning
- `DuplicateEvidenceRefWithinNode`
- `DuplicateEvidenceRef`
- all `WeakLocator::*`
- `WeakSourceRef::ContainsWhitespace`
- `WeakSourceRef::InvalidCharacter`
- `WeakSourceRef::UnknownNamespace`

### Info
- reserved only, not emitted by current logic

## Boundary
- This is still read-only governance reporting.
- No auto-remediation or migration behavior was introduced.
- The severity rules live locally inside `repository_metadata_audit.rs` for now.

## Verified Commands
- `cargo test --test repository_metadata_audit_unit -- --nocapture`
- `cargo test --test metadata_validator_unit -- --nocapture`
- `cargo test --test metadata_registry_unit -- --nocapture`
- `cargo test --test metadata_scope_resolver_unit -- --nocapture`
- `cargo test --test knowledge_record_unit -- --nocapture`
- `cargo test --test knowledge_graph_store_unit -- --nocapture`
- `cargo test --test ontology_schema_unit -- --nocapture`
- `cargo test --test ontology_store_unit -- --nocapture`

## Next Recommended Step
- Continue with report presentation cleanup or governance-oriented routing on top of `has_blocking_hygiene_issue` and the structured count maps.
