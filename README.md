# SheetMind / Excel_Skill

`SheetMind / Excel_Skill` is a Rust-first Excel analysis and foundation-capability repository.

The current priority is not to keep expanding scattered business scenarios. The priority is to stabilize a reusable base that can support table processing, analysis, report delivery, and future domain extensions through a cleaner architecture.

## Current Product Direction

The current mainline is:

- Rust / EXE / CLI-first delivery
- foundation-first evolution
- reusable Excel-oriented table, analysis, and report capabilities
- business-agnostic navigation flow: `ontology-lite -> roaming -> retrieval -> evidence assembly`

Python historical materials may still exist in the repository, but they are not the current product mainline.

## Read First

If you are a new AI session or a new engineer, start here:

1. [AI_START_HERE.md](./AI_START_HERE.md)
2. [docs/ai-memory/project-baseline.md](./docs/ai-memory/project-baseline.md)
3. [docs/ai-handoff/AI_HANDOFF_MANUAL.md](./docs/ai-handoff/AI_HANDOFF_MANUAL.md)
4. [docs/execution-notes-2026-04-07-foundation-navigation-kernel.md](./docs/execution-notes-2026-04-07-foundation-navigation-kernel.md)
5. [docs/architecture/cli-modularization.md](./docs/architecture/cli-modularization.md)

## What This Repository Is For

This repository currently carries four closely related capability lines:

### 1. Excel and Table Processing

- workbook open and table loading
- preview, filtering, sorting, grouping, and append/join flows
- structured source references such as `file_ref`, `table_ref`, `result_ref`, and `workbook_ref`

### 2. Analysis and Modeling

- statistical summary and diagnostics
- regression and clustering workflows
- report-oriented result shaping

### 3. Runtime and Delivery

- session-aware local execution
- structured CLI JSON contracts
- workbook/report export pipeline

### 4. Foundation Navigation Kernel

The current foundation work is moving along this fixed order:

`ontology-lite -> roaming -> retrieval -> evidence assembly`

Current implemented foundation modules include:

- `ontology_schema`
- `ontology_store`
- `knowledge_record`
- `knowledge_graph_store`
- `capability_router`
- `roaming_engine`
- `retrieval_engine`

Do not treat retrieval as the system entry point, and do not restart architecture refactors without clear proof that the current baseline cannot carry the next requirement.

## Repository Structure

- `src/`: Rust application, CLI, runtime, tools, and foundation modules
- `tests/`: Rust integration tests, unit tests, and runtime fixtures
- `docs/`: plans, execution notes, acceptance records, and handoff materials
- `skills/`: project-local skills and orchestration assets
- `cli/`: historical CLI-related Python materials
- `tradingagents/`: historical imported source materials, not the current product identity

## Development Rules

- Default to the Rust mainline
- Keep foundation and application concerns separated
- Prefer TDD and small verifiable steps
- Do not re-architect by default when extending capabilities
- Keep AI handoff materials continuously usable

## Common Verification

### Check branch and worktree status

```bash
git status --short --branch
git branch -vv
```

### Foundation regression sweep

```bash
cargo test --test ontology_schema_unit --test ontology_store_unit --test knowledge_record_unit --test knowledge_graph_store_unit --test capability_router_unit --test roaming_engine_unit --test retrieval_engine_unit -- --nocapture
```

## Not the Current Priority

This stage is not primarily about:

- GUI-first intelligent Q&A
- binding foundation to one single business domain
- restarting large-scale architecture refactors for isolated scenarios
- making cloud models a hard dependency

## Note for Future AI Sessions

Do not trust the oldest historical naming in the repository as the current truth.

Read the handoff and baseline documents first, confirm the active mainline, then decide whether a change belongs to the foundation layer or to a future extension project.
