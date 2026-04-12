# P7 Automatic Proxy Backfill And Promotion Hardening Implementation Plan

> **For Claude/Codex:** REQUIRED SUB-SKILLS: use `test-driven-development`, `systematic-debugging`, and append `.trae/CHANGELOG_TASK.md` after completion.

**Goal:** Upgrade the securities mainline so historical proxy backfill becomes a governed promotion-evidence source and `champion` promotion depends on stronger multi-window stability evidence.

**Architecture:** Reuse the existing `security_external_proxy_backfill -> security_history_expansion -> security_shadow_evaluation -> security_model_promotion -> security_decision_submit_approval` chain instead of introducing a parallel lifecycle.

---

## Task 1: Lock governed backfill coverage with failing tests

**Files**
- Modify: `D:\Rust\Excel_Skill\tests\security_external_proxy_backfill_cli.rs`
- Modify: `D:\Rust\Excel_Skill\tests\security_history_expansion_cli.rs`

**Red tests**
- backfill result should expose reusable coverage fields such as:
  - `covered_proxy_fields`
  - `covered_dates`
  - `covered_symbol_count`
  - `coverage_tier`
- history expansion should accept governed backfill references and surface actual imported coverage evidence

## Task 2: Lock multi-window shadow evidence with failing tests

**Files**
- Modify: `D:\Rust\Excel_Skill\tests\security_shadow_evaluation_cli.rs`

**Red tests**
- shadow evaluation should emit:
  - `shadow_window_count`
  - `oot_stability_status`
  - `window_consistency_status`
  - `promotion_evidence_notes[]`
- champion recommendation should require strong multi-window evidence

## Task 3: Lock harder champion promotion with failing tests

**Files**
- Modify: `D:\Rust\Excel_Skill\tests\security_model_promotion_cli.rs`

**Red tests**
- champion promotion should fail when OOT/window evidence is still thin
- champion promotion should pass only when repeated shadow + stable windows + ready backfill coverage all hold

## Task 4: Lock approval/package promotion-evidence summary with failing tests

**Files**
- Modify: `D:\Rust\Excel_Skill\tests\security_decision_submit_approval_cli.rs`

**Red tests**
- approval brief and decision package should expose:
  - `shadow_window_count`
  - `oot_stability_status`
  - `window_consistency_status`
  - `promotion_evidence_notes`
- non-full-release grades should remain downgraded even if one evidence dimension is green

## Task 5: Implement the strengthened promotion-evidence chain

**Files**
- Modify: `D:\Rust\Excel_Skill\src\ops\security_external_proxy_backfill.rs`
- Modify: `D:\Rust\Excel_Skill\src\ops\security_history_expansion.rs`
- Modify: `D:\Rust\Excel_Skill\src\ops\security_shadow_evaluation.rs`
- Modify: `D:\Rust\Excel_Skill\src\ops\security_model_promotion.rs`
- Modify: `D:\Rust\Excel_Skill\src\ops\security_decision_approval_brief.rs`
- Modify: `D:\Rust\Excel_Skill\src\ops\security_decision_package.rs`
- Modify: `D:\Rust\Excel_Skill\src\ops\security_decision_submit_approval.rs`

**Implementation targets**
- publish governed backfill coverage signals
- let history expansion consume governed backfill evidence
- add shadow multi-window / OOT evidence
- harden champion promotion against thin window evidence
- expose stronger promotion evidence in approval/package consumers

## Task 6: Run focused regressions and close the phase

**Verification**
```powershell
cargo test --test security_external_proxy_backfill_cli -- --nocapture
cargo test --test security_history_expansion_cli -- --nocapture
cargo test --test security_shadow_evaluation_cli -- --nocapture
cargo test --test security_model_promotion_cli -- --nocapture
cargo test --test security_decision_submit_approval_cli -- --nocapture
cargo test --test security_feature_snapshot_cli -- --nocapture
cargo test --test security_scorecard_refit_cli -- --nocapture
```

**Closeout**
- append `.trae/CHANGELOG_TASK.md`
- summarize what changed, residual risks, and next suggested tests
