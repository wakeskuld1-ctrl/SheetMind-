# Foundation Repository Metadata Audit Export Tool

## Summary
- Added the public foundation tool `repository_metadata_audit_export_v1`.
- The tool now returns `RepositoryMetadataAuditExportDtoV1` JSON instead of exposing the internal `RepositoryMetadataAuditReport` shape directly.
- Added file-backed metadata schema loading so the tool can consume a persisted schema contract instead of in-memory constructors.
- Aligned the public repository-bundle boundary with the navigation export tool so the same export outlet now accepts JSON and JSONL bundles and rejects blank path inputs before filesystem access.

## Changed Files
- `src/ops/foundation/metadata_schema.rs`
- `src/ops/foundation/repository_metadata_audit.rs`
- `src/tools/catalog.rs`
- `src/tools/dispatcher.rs`
- `tests/integration_cli_json.rs`

## Contract
- Tool name: `repository_metadata_audit_export_v1`
- Required args:
  - `schema_path`
  - `bundle_path`
- Response:
  - `status: "ok"`
  - `data: RepositoryMetadataAuditExportDtoV1`

## Notes
- `schema_path` now points to a JSON document shape with:
  - `schema_version`
  - `fields`
  - `concept_policies`
- `bundle_path` accepts the existing `KnowledgeBundle` persisted repository shape in either `.json` or `.jsonl` form.
- The dispatcher trims `schema_path` and `bundle_path` before validation, so blank-string inputs fail fast with stable public error messages instead of OS path errors.
- The dispatcher keeps JSON vs. JSONL loader selection local to the public boundary and does not change the internal audit report semantics.
- This slice is additive only. Internal report semantics were not redesigned.

## Verified
- `cargo test --test integration_cli_json repository_metadata_audit_export_v1 -- --nocapture`
- `cargo test --test repository_metadata_audit_unit -- --nocapture`
- `cargo test --test metadata_validator_unit -- --nocapture`

## Next Recommended Step
- Keep following foundation mainline and expose the same DTO-facing boundary in any later roaming/GUI/AI entrypoints, instead of letting new callers bind to the internal report struct.
- If this tool needs more hardening later, prefer one additive dispatcher-side slice for schema-path contract validation instead of changing the audit mainline or mutating the `v1` DTO.
