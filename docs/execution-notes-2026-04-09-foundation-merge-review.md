# Execution Notes - 2026-04-09 Foundation Merge Review

## Scope

- This round was a merge-review and handoff closeout for `codex/foundation-merge-review`.
- The goal was not to reopen architecture design or add new product features.
- The immediate purpose was to finish the last version-consistency cleanup for this line, record the durable rules, and separate real tracked work from fresh runtime noise.

## Changes Made

- Updated [AI_HANDOFF_MANUAL.md](/C:/Users/wakes/.codex/worktrees/Excel_Skill/codex-foundation-merge-review/docs/ai-handoff/AI_HANDOFF_MANUAL.md) to record the new repository-level rule that version consistency cleanup is a one-time closeout task and must not become the default opening action for every future AI.
- Confirmed the foundation merge-repair fixes already applied in this worktree:
  - [retrieval_engine_unit.rs](/C:/Users/wakes/.codex/worktrees/Excel_Skill/codex-foundation-merge-review/tests/retrieval_engine_unit.rs) now builds `CandidateScope` fixtures with `metadata_scope`
  - [navigation_pipeline_integration.rs](/C:/Users/wakes/.codex/worktrees/Excel_Skill/codex-foundation-merge-review/tests/navigation_pipeline_integration.rs) now asserts retrieval ranking according to the real tie-break contract instead of assuming route order
  - [.trae/CHANGELOG_TASK.md](/C:/Users/wakes/.codex/worktrees/Excel_Skill/codex-foundation-merge-review/.trae/CHANGELOG_TASK.md) no longer contains leftover merge markers
- Reviewed the currently unstaged delta in [security_decision_submit_approval_cli.rs](/C:/Users/wakes/.codex/worktrees/Excel_Skill/codex-foundation-merge-review/tests/security_decision_submit_approval_cli.rs) and classified it as a real contract-alignment change rather than runtime noise:
  - old hard-coded `Long` assertions are being replaced with direction-consistency assertions
  - this matches the current governance chain, where blocked or rebased flows may no longer serialize the old fixed direction literal
- Confirmed and preserved three real test-hygiene repairs that were exposed only after runtime-noise cleanup:
  - [security_decision_committee_cli.rs](/C:/Users/wakes/.codex/worktrees/Excel_Skill/codex-foundation-merge-review/tests/security_decision_committee_cli.rs) no longer depends on an untracked local SQLite fixture and now builds its runtime database in-test
  - [security_scorecard_cli.rs](/C:/Users/wakes/.codex/worktrees/Excel_Skill/codex-foundation-merge-review/tests/security_scorecard_cli.rs) no longer depends on `tests/runtime_fixtures/local_memory/live_601916_20260408/stock_history.db` and now imports local CSV data into a fresh runtime database
  - [security_scorecard_training_cli.rs](/C:/Users/wakes/.codex/worktrees/Excel_Skill/codex-foundation-merge-review/tests/security_scorecard_training_cli.rs) now uses a non-degenerate down-trend sample so training no longer fails on `RSRS` denominator-zero regression
- Removed fresh untracked runtime-noise directories after verification so the remaining dirty state reflects tracked work plus the current documentation closeout, not test-run residue.

## Verification Run

- Ran `cargo test --test retrieval_engine_unit -- --nocapture`
  - Result: passed
- Ran `cargo test --test navigation_pipeline_integration -- --nocapture`
  - Result: passed
- Ran `cargo test --test ontology_schema_unit --test ontology_store_unit --test knowledge_record_unit --test knowledge_graph_store_unit --test capability_router_unit --test roaming_engine_unit --test retrieval_engine_unit --test evidence_assembler_unit --test navigation_pipeline_integration -- --nocapture`
  - Result: passed
- Ran `cargo test --test ontology_schema_unit --test ontology_store_unit --test knowledge_record_unit --test knowledge_graph_store_unit --test capability_router_unit --test roaming_engine_unit --test retrieval_engine_unit --test evidence_assembler_unit --test navigation_pipeline_integration --test security_chair_resolution_cli --test security_decision_committee_cli --test security_decision_evidence_bundle_cli --test security_decision_package_revision_cli --test security_decision_submit_approval_cli --test security_decision_verify_package_cli --test security_feature_snapshot_cli --test security_forward_outcome_cli --test security_post_meeting_conclusion_cli --test security_scorecard_cli --test security_scorecard_refit_cli --test security_scorecard_training_cli -- --nocapture`
  - Result: passed
- Ran `git clean -fd -- tests/runtime_fixtures/local_memory tests/runtime_fixtures/security_chair_resolution tests/runtime_fixtures/security_decision_committee tests/runtime_fixtures/security_decision_evidence_bundle tests/runtime_fixtures/security_decision_package_revision tests/runtime_fixtures/security_decision_submit_approval tests/runtime_fixtures/security_decision_verify_package tests/runtime_fixtures/security_feature_snapshot tests/runtime_fixtures/security_forward_outcome tests/runtime_fixtures/security_post_meeting_conclusion tests/runtime_fixtures/security_scorecard tests/runtime_fixtures/security_scorecard_refit tests/runtime_fixtures/security_scorecard_training`
  - Result: removed fresh untracked runtime fixture directories created during verification without touching tracked fixtures

## Remaining Risks

- The branch still contains a wide staged set that mixes foundation and adapter-side tracked work; this note does not reduce that scope by itself.
- The branch still includes many tracked fixture directories and cross-line edits by design, so final staging must still be intentional even though fresh runtime noise has now been removed.
- Existing `dead_code` warnings in `src/tools/dispatcher.rs` remain repository noise; they were present during the successful full regression and are not evidence of a new failure in this closeout round.

## Notes For Next AI

- Treat this note and the handoff manual together as the canonical record for why version-consistency cleanup must not be repeated by default.
- If Git status is dirty in the future, first distinguish between tracked feature work and untracked runtime artifacts before proposing any new cleanup.
- If security-line tests fail after a cleanup step, check fixture hygiene first:
  - first ask whether the test secretly depended on untracked local runtime state
  - then ask whether the synthetic sample itself has degenerated into an invalid indicator input
- Do not reopen merge-alignment work unless a new real trigger exists and can be named concretely.
