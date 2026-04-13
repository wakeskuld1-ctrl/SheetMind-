# 2026-04-12 Foundation Hygiene Stability Boundary Handoff

## Scope
- Slice: `A3-标准版`
- Track: `foundation / knowledge roaming / repository hygiene governance`
- Goal: lock the aggregation and ordering semantics of `repository_metadata_audit` under large-repository and repeated-reason boundary cases.
- Production impact: none in this slice. No production file was changed.

## What Changed
- Added large-repository stability coverage in:
  - `tests/repository_metadata_audit_unit.rs`
- Added repeated-reason boundary coverage in:
  - `tests/repository_metadata_audit_unit.rs`
- Saved design and implementation notes in:
  - `docs/plans/2026-04-12-foundation-hygiene-stability-boundary-design.md`
  - `docs/plans/2026-04-12-foundation-hygiene-stability-boundary-plan.md`

## Locked Semantics
- `hygiene_summary.weak_*_reason_counts`
  - Counts raw diagnostics by reason.
- `hygiene_reason_views.*_by_reason[*].diagnostic_count`
  - Counts deduplicated affected nodes under the current implementation.
- `hygiene_reason_views.*_by_reason[*].affected_node_count`
  - Currently matches the deduplicated node count and therefore matches `diagnostic_count`.
- `node_ids`
  - Comes from `BTreeSet`, so output order is lexicographically stable.

## Large Repository Stability Coverage
- Added fixture helper: `large_stability_repository()`
- Locked expectations include:
  - `report.total_nodes == 16`
  - `hygiene_summary.total_diagnostics == 20`
  - `severity_counts[Critical] == 4`
  - `severity_counts[Warning] == 16`
  - `affected_node_count == 16`
  - stable severity grouping counts
  - stable node-first ordering
  - stable per-reason `node_ids` ordering for weak locator and weak source ref diagnostics

## Boundary Reason Coverage
- Added fixture helper: `repeated_reason_boundary_repository()`
- Locked expectations include:
  - summary-level reason counts continue to count raw diagnostics
  - reason-view-level counts stay node-deduplicated
  - repeated reasons on the same node do not inflate reason-view node counts

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
- Do not say this slice changed `src/ops/foundation/repository_metadata_audit.rs`; it did not.
- This slice exists to prevent semantic drift while the roaming layer grows on top of foundation hygiene outputs.
- If a future task wants `reason_views[*].diagnostic_count` to mean raw diagnostic count instead of deduplicated node count, that is a deliberate contract change and the current tests must be rewritten together with the implementation.
- Repository-wide dead-code warnings from unrelated dispatcher areas still exist and were intentionally left out of this slice.
