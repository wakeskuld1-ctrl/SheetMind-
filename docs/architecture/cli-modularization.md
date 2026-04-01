# CLI Modularization Architecture

## Goal

The CLI remains a single binary entry point, but its internal routing and support logic are split into cohesive modules.

This refactor keeps the external contract stable for:

- CLI tool names
- JSON request/response shapes
- `table_ref`, `result_ref`, and `workbook_ref` semantics
- session-stage transitions used by Skills

## Stable External Contract

The public boundary is still:

- `ToolRequest`
- `ToolResponse`
- `dispatcher::dispatch()`

Skills and higher-level wrappers should continue to treat the CLI as a tool registry plus JSON protocol. They should not depend on internal Rust module paths.

## Module Layout

### Public tool modules

- `src/tools/catalog.rs`
  - Owns the canonical ordered tool registry
  - Prevents catalog drift across CLI entry points and tests

- `src/tools/contracts.rs`
  - Defines `ToolRequest` and `ToolResponse`
  - Builds catalog responses from the central registry

- `src/tools/session.rs`
  - Owns session-state sync and active-handle updates
  - Keeps session-side effects out of individual dispatch handlers

- `src/tools/results.rs`
  - Owns result persistence and `result_ref` response shaping
  - Centralizes lineage/source-ref collection

- `src/tools/sources.rs`
  - Owns source resolution for path, `file_ref`, `table_ref`, and `result_ref`
  - Encapsulates confirmation gating and nested source loading

### Dispatcher submodules

- `src/tools/dispatcher.rs`
  - Thin router only
  - Dispatches by registered tool name into focused submodules

- `src/tools/dispatcher/workbook_io.rs`
  - Workbook open/list/inspect/load flows
  - header/schema confirmation flows
  - session-state read/write
  - export handlers

- `src/tools/dispatcher/single_table.rs`
  - Single-input table transforms
  - preview, filter, cast, derive, group, pivot, sort, rename, lookup, fill, format, etc.

- `src/tools/dispatcher/multi_table.rs`
  - Multi-table orchestration
  - join, append, workbook composition, table-link suggestion, workflow suggestion

- `src/tools/dispatcher/analysis_ops.rs`
  - Analysis and modeling handlers
  - summarize, analyze, stat summary, regression, clustering, decision assistant

- `src/tools/dispatcher/shared.rs`
  - Small shared parsing/casting helpers used across dispatcher submodules
  - Intentionally narrow to avoid rebuilding a god module

## Dependency Direction

The intended dependency flow is:

`dispatcher submodules -> sources/results/session/shared -> frame/runtime/ops/domain`

Key rules:

- Submodules may call shared helper modules, but should not depend on sibling handler internals.
- Session updates should go through `tools::session`.
- Result persistence should go through `tools::results`.
- Source resolution should go through `tools::sources`.
- New handlers should not add business logic back into `dispatcher.rs`.

## Adding a New Tool

When adding a new tool:

1. Register its name in `src/tools/catalog.rs`
2. Route it in `src/tools/dispatcher.rs`
3. Implement it in the closest cohesive submodule
4. Reuse `sources`, `results`, and `session` helpers instead of duplicating logic
5. Add or update integration tests that lock the external JSON contract

Placement guidance:

- Workbook/session/export behavior: `workbook_io.rs`
- Single-table transformation: `single_table.rs`
- Multi-table orchestration: `multi_table.rs`
- Analysis/modeling/decision output: `analysis_ops.rs`
- Cross-cutting helper logic: only if genuinely reusable, in `shared.rs`, `sources.rs`, `results.rs`, or `session.rs`

## Testing Strategy

Contract safety is currently enforced by:

- registry/catalog tests
- CLI JSON integration tests
- frame/ops integration tests
- full `cargo test --quiet`

The most important invariant is that internal refactors must not change Skill-facing behavior.

## Non-Goals

This refactor does not change:

- the binary-first delivery model
- end-user tool names
- the persisted handle model
- the session memory model

It is an internal modularization for maintainability, reviewability, and safer future extension.
