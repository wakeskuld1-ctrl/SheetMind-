# 2026-04-12 Foundation Source Ref Diagnostics Handoff

## Scope
- Mainline: `foundation`
- Layer: bottom-layer knowledge roaming repository governance
- This round only extends repository evidence hygiene diagnostics.
- This round does not touch stock/business workflows, migration executors, or auto-fix flows.

## What Changed
- Restored the `foundation` entrypoint exports for:
  - `knowledge_bundle`
  - `knowledge_ingestion`
  - `knowledge_repository`
- Extended `RepositoryWeakSourceRefReason` with:
  - `EntityMissing`
  - `ContainsWhitespace`
  - `InvalidCharacter`
  - `UnknownNamespace`
- Added minimal `source_ref` structure checks in repository metadata audit:
  - `sheet:` -> `EntityMissing`
  - `sheet: sales q1` -> `ContainsWhitespace`
  - `sheet:sales?2024` -> `InvalidCharacter`
  - `blob:sales` -> `UnknownNamespace`

## Why
- The previous workspace state had a compile break because `repository_metadata_audit` still depended on `knowledge_repository`, while `src/ops/foundation.rs` no longer re-exported that module.
- The approved task for this round was to continue the foundation evidence-hygiene line, specifically the `source_ref` structure diagnostics after the locator work had already landed.

## Validation
- `cargo test --test repository_metadata_audit_unit -- --nocapture`
- `cargo test --test metadata_validator_unit -- --nocapture`
- `cargo test --test metadata_registry_unit -- --nocapture`
- `cargo test --test metadata_scope_resolver_unit -- --nocapture`
- `cargo test --test knowledge_record_unit -- --nocapture`
- `cargo test --test knowledge_graph_store_unit -- --nocapture`
- `cargo test --test ontology_schema_unit -- --nocapture`
- `cargo test --test ontology_store_unit -- --nocapture`

## Current Boundary
- Keep this line in `foundation`.
- Treat these diagnostics as read-only governance signals.
- Do not use this task as a trigger to redesign the roaming model or move modules across namespaces.
- If the next AI continues this line, prefer:
  1. severity / summary shaping for hygiene diagnostics
  2. report presentation cleanup
  3. only then consider broader governance aggregation

## Known Notes
- The repository still emits many unrelated dead-code warnings from `src/tools/dispatcher.rs`; they were pre-existing and were not addressed in this task.
- The legacy top-level `docs/AI_HANDOFF.md` currently appears dirty/conflicted in this workspace, so this task intentionally used a dedicated handoff note instead of editing that file again.
