# AI Handoff - 2026-04-21 Betting Upload

## Project Goal
- Deliver a stable Excel betting optimizer workflow with VBA-triggered solving, operator-visible logs, multi-round adjustment sheets, and customer-facing packaged outputs.

## Current Confirmed Behavior
- Live customer regression case `10564 / 1500 / 19` now solves successfully instead of returning `NoFeasibleSolution`.
- Boundary coverage now includes concentrated betting, uniform already-safe betting, sparse high-risk rows with many zeros, low-loss-to-target adjustment, and a small brute-force optimality cross-check.
- CLI next-round solving now discovers the actual generated result sheet name from workbook metadata instead of relying on a brittle encoded literal.

## Start Here Next Time
- Read this file and `docs/execution-notes-2026-04-21-betting-upload.md`.
- Then inspect:
  - `src/ops/betting_optimizer.rs`
  - `src/ops/betting_workbook_bridge.rs`
  - `tests/betting_optimizer_unit.rs`
  - `tests/betting_solver_cli.rs`
- Rerun:
  - `cargo test --test betting_optimizer_unit -- --nocapture`
  - `cargo test --test betting_solver_cli -- --nocapture`
  - `cargo test --test betting_workbook_bridge_cli -- --nocapture`

## Key Inputs and Outputs
- Customer delivery package:
  - `outputs/客户交付包_2026-04-21/`
- Engineering delivery package:
  - `outputs/betting_optimizer_delivery/`
- Current task history:
  - `.trae/CHANGELOG_TASK.md`
- Approved design/process context:
  - `docs/plans/2026-04-20-betting-workbook-optimizer-design.md`
  - `docs/plans/2026-04-21-betting-boundary-regression-design.md`
  - `docs/plans/2026-04-21-betting-manual-adjustment-constraints-design.md`

## Easy Mistakes To Avoid
- Do not revert user-created generated fixtures or outputs just to make the tree smaller.
- Do not assume an encoded Chinese sheet literal is stable in tests; prefer reading the actual workbook metadata.
- Do not claim the optimizer is fixed without rerunning the three betting test suites above.

## Known Open Risks
- `betting_workbook_bridge_cli` still has 3 legacy ignored tests tied to unstable encoded fixture literals.
- Repository-wide cargo warnings in unrelated modules are still present and may distract future verification runs.

## Upload Context
- This branch upload intentionally includes generated delivery artifacts because the user explicitly requested both compiled and uncompiled materials to be pushed.
