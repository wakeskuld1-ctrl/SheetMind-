# 2026-04-12 Foundation Hygiene Export DTO V1 Handoff

## Scope
- Slice: `Foundation Hygiene Export DTO V1`
- Track: `foundation / knowledge roaming / repository hygiene governance`
- Goal: add one stable `v1` export DTO for `RepositoryMetadataAudit` so downstream AI and upper-layer callers stop depending on the internal report shape directly.

## What Changed
- Added versioned export DTO modeling in:
  - `src/ops/foundation/repository_metadata_audit.rs`
- Added one-way conversion entry point:
  - `RepositoryMetadataAuditExportDtoV1::from_report(&RepositoryMetadataAuditReport)`
- Added DTO export coverage in:
  - `tests/repository_metadata_audit_unit.rs`
- Saved design and implementation notes in:
  - `docs/plans/2026-04-12-foundation-hygiene-export-dto-v1-design.md`
  - `docs/plans/2026-04-12-foundation-hygiene-export-dto-v1-plan.md`

## Contract Notes
- This slice does not replace `RepositoryMetadataAuditReport`.
- The internal report remains the foundation internal model.
- `RepositoryMetadataAuditExportDtoV1` is the stable external contract layer for current foundation hygiene output.
- The conversion is one-way and in-memory only.

## Semantics Intentionally Mirrored
- `hygiene_summary.weak_*_reason_counts`
  - counts raw diagnostics by reason
- `hygiene_reason_views.*_by_reason[*].diagnostic_count`
  - currently counts deduplicated affected nodes
- `hygiene_reason_views.*_by_reason[*].affected_node_count`
  - currently matches the deduplicated node count
- `node_ids`
  - remains lexicographically stable

## DTO Coverage Added
- Base contract export test:
  - `dto_export_v1_mirrors_foundation_hygiene_contract`
- Large-repository ordering stability export test:
  - `dto_export_v1_keeps_large_repository_ordering_stable`
- Repeated-reason boundary export test:
  - `dto_export_v1_preserves_reason_count_semantics_on_repeated_reasons`

## Verification
- `cargo test --test repository_metadata_audit_unit -- --nocapture`
- `cargo test --test metadata_validator_unit -- --nocapture`
- `cargo test --test metadata_registry_unit -- --nocapture`
- `cargo test --test metadata_scope_resolver_unit -- --nocapture`
- `cargo test --test knowledge_record_unit -- --nocapture`
- `cargo test --test knowledge_graph_store_unit -- --nocapture`
- `cargo test --test ontology_schema_unit -- --nocapture`
- `cargo test --test ontology_store_unit -- --nocapture`

## Notes For Next AI
- Do not refactor the internal report just because `v1` now exists.
- New external callers should prefer `RepositoryMetadataAuditExportDtoV1` over direct report access.
- If future work changes reason-count semantics, both report tests and DTO tests must move together.
- Unrelated dispatcher dead-code warnings still exist and were intentionally left outside this slice.
