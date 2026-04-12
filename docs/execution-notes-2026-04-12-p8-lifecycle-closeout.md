# 2026-04-12 P8 Lifecycle Closeout

## Scope

- Added formal lifecycle tools on the current securities mainline:
  - `security_condition_review`
  - `security_execution_record`
  - `security_post_trade_review`
- Extended package revision so lifecycle artifacts can be attached after approval-time package creation.
- Added package-visible lifecycle feedback summary for later replay and operator review.

## Current lifecycle order

1. Run `security_decision_submit_approval`
2. Run `security_condition_review` when intraperiod review is triggered
3. Run `security_execution_record` when a governed execution or freeze event happens
4. Run `security_post_trade_review` after execution review becomes available
5. Run `security_decision_package_revision` with any lifecycle artifact paths that should join the package

## Package revision attachment fields

- `condition_review_path`
- `execution_record_path`
- `post_trade_review_path`

When provided, the revision flow now:
- validates lifecycle bindings against `decision_ref / approval_ref / position_plan_ref`
- updates `decision_package.object_graph`
- appends lifecycle artifacts into `artifact_manifest`
- writes `lifecycle_governance_summary`

## Lifecycle governance summary

The package now exposes:
- `lifecycle_status`
- `condition_review_ref`
- `execution_record_ref`
- `post_trade_review_ref`
- `recommended_governance_action`
- `attribution_layers`

## Focused verification run on 2026-04-12

- `cargo test --test security_decision_package_revision_cli -- --nocapture`
- `cargo test --test security_decision_submit_approval_cli security_decision_submit_approval_writes_runtime_files_for_ready_case -- --nocapture`
- `cargo test --test security_condition_review_cli -- --nocapture`
- `cargo test --test security_execution_record_cli -- --nocapture`
- `cargo test --test security_post_trade_review_cli -- --nocapture`

## Known boundaries

- The current lifecycle attachment is package-revision driven; the initial approval package still starts without lifecycle refs.
- `AI_HANDOFF.md` currently has unresolved conflict markers in the workspace, so this note is the clean source of truth for the 2026-04-12 P8 closeout.
- Feedback has been formalized into package-visible summary fields, but training/promotion automatic consumption still belongs to later phases.
