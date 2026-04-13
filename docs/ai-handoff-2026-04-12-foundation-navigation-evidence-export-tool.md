# Foundation Navigation Evidence Export Tool

## Date
- 2026-04-12 CST

## Scope
- This slice stays on the `foundation / knowledge roaming / stable export DTO boundary` mainline.
- This slice does not change stock-domain behavior.
- This slice does not replace the internal `NavigationEvidence` struct.

## What Changed
- Added public tool discovery for `navigation_evidence_export_v1` in:
  - `src/tools/catalog.rs`
- Added a thin dispatcher export outlet in:
  - `src/tools/dispatcher.rs`
- Added file-backed CLI contract coverage in:
  - `tests/integration_cli_json.rs`
- Added public bad-input validation coverage for malformed `metadata_constraints` in:
  - `tests/integration_cli_json.rs`
- Added dispatcher-side fail-fast validation for malformed `metadata_constraints` in:
  - `src/tools/dispatcher.rs`
- Added public preflight validation for blank `bundle_path` and blank `question` in:
  - `src/tools/dispatcher.rs`
- Added public default-behavior coverage for omitted optional roaming-plan args in:
  - `tests/integration_cli_json.rs`

## Contract Summary
- Tool name: `navigation_evidence_export_v1`
- Input:
  - `bundle_path`
  - `question`
  - optional `seed_concept_ids`
  - optional `allowed_relation_types`
  - optional `max_depth`
  - optional `max_concepts`
  - optional `required_concept_tags`
  - optional `metadata_constraints`
- Output:
  - `NavigationEvidenceExportDtoV1` JSON payload

## Behavior Notes
- The tool accepts `bundle.json` and `bundle.jsonl` inputs through one public dispatcher boundary.
- The dispatcher rebuilds `KnowledgeRepository -> OntologyStore + GraphStore -> NavigationPipeline`.
- The dispatcher returns `pipeline.run_export_v1(...)` output directly as the public DTO payload.
- The dispatcher maps external `metadata_constraints` JSON into the internal `MetadataScope` contract locally at the public tool boundary.
- The dispatcher trims public text inputs before validation so blank `bundle_path`, blank `question`, and blank metadata `field` fail fast at the boundary.
- Supported public metadata operators are:
  - `equals`
  - `in`
  - `has_any`
  - `range`
- The current public metadata JSON shape includes:
  - `{"operator":"equals","field":"source","value":"sheet:sales"}`
  - `{"operator":"has_any","field":"channel","values":["analytics"]}`

## Why This Slice Exists
- The previous slice already created `NavigationEvidenceExportDtoV1` and `NavigationPipeline::run_export_v1(...)`.
- Higher-level AI and CLI callers still had no formal public outlet for that DTO.
- This slice moves external callers onto a stable roaming export contract instead of letting them bind to internal pipeline structs.

## Verification
- `cargo test --test integration_cli_json tool_catalog_includes_navigation_evidence_export_v1 -- --nocapture`
- `cargo test --test integration_cli_json navigation_evidence_export_v1 -- --nocapture`
- `cargo test --test evidence_assembler_unit -- --nocapture`
- `cargo test --test navigation_pipeline_integration -- --nocapture`

## Known Limits
- The public metadata contract is intentionally thin and only converts external JSON payloads into the existing internal scope model; it does not expose internal `NavigationRequest` structs.
- The current bad-input hardening covers blank `bundle_path`, blank `question`, blank metadata `field`, unsupported operators, missing `equals.value`, empty `in/has_any.values`, and boundless `range`.
- The tool still does not do schema-aware metadata field validation at the dispatcher boundary because this slice intentionally avoids introducing a new schema/registry input contract.
- Dispatcher verification still prints unrelated dead-code warnings from the existing codebase; they are not introduced by this slice.
- Any future semantic change to the returned DTO must be handled by versioning, not by mutating `v1` in place.

## Recommended Next Slice
- If foundation resumes later, the next additive slice should be optional schema-aware metadata field validation on the same public tool boundary.
- Keep the change additive by extending dispatcher-side validation or optional registry wiring without changing the success DTO contract.
