# P6 Shadow Champion Hardening Implementation Plan

> **For Claude/Codex:** REQUIRED SUB-SKILLS: use `test-driven-development`, `systematic-debugging`, and append `.trae/CHANGELOG_TASK.md` after completion.

**Goal:** Harden the securities `candidate -> shadow -> champion` lifecycle by standardizing history coverage, tracking repeated shadow observations, strengthening champion promotion, and exposing governance summaries in approval/package outputs.

**Architecture:** Reuse the existing `security_history_expansion -> security_shadow_evaluation -> security_model_promotion -> security_decision_submit_approval` chain instead of introducing a parallel governance system.

---

## Task 1: Lock standardized history coverage with failing tests

**Files**
- Modify: `D:\Rust\Excel_Skill\tests\security_history_expansion_cli.rs`

**Red tests**
- history-expansion output should expose:
  - `coverage_tier`
  - `shadow_readiness_hint`
  - `champion_readiness_hint`
  - `proxy_field_coverage[]`

## Task 2: Lock repeated shadow observation semantics with failing tests

**Files**
- Modify: `D:\Rust\Excel_Skill\tests\security_shadow_evaluation_cli.rs`

**Red tests**
- shadow evaluation should accept prior evaluations and output:
  - `shadow_observation_count`
  - `shadow_consistency_status`
  - `promotion_blockers[]`
- champion recommendation should require repeated stable observations

## Task 3: Lock harder champion promotion with failing tests

**Files**
- Modify: `D:\Rust\Excel_Skill\tests\security_model_promotion_cli.rs`

**Red tests**
- champion promotion should succeed only when shadow evaluation carries:
  - `recommended_model_grade = champion`
  - enough observation count
  - stable consistency
  - no blockers

## Task 4: Lock approval/package governance summary with failing tests

**Files**
- Modify: `D:\Rust\Excel_Skill\tests\security_decision_submit_approval_cli.rs`

**Red tests**
- approval brief and decision package should expose:
  - `model_governance_summary`
  - `shadow_observation_count`
  - `shadow_consistency_status`
  - `promotion_blockers`
- non-full-release grades should remain downgraded

## Task 5: Implement the hardened governance chain

**Files**
- Modify: `D:\Rust\Excel_Skill\src\ops\security_history_expansion.rs`
- Modify: `D:\Rust\Excel_Skill\src\ops\security_shadow_evaluation.rs`
- Modify: `D:\Rust\Excel_Skill\src\ops\security_model_promotion.rs`
- Modify: `D:\Rust\Excel_Skill\src\ops\security_decision_approval_brief.rs`
- Modify: `D:\Rust\Excel_Skill\src\ops\security_decision_package.rs`
- Modify: `D:\Rust\Excel_Skill\src\ops\security_decision_submit_approval.rs`

**Implementation targets**
- add standardized coverage summary fields
- add prior shadow evaluation replay inputs
- harden champion promotion conditions
- expose governance summary in approval/package

## Task 6: Run focused regressions and close the phase

**Verification**
```powershell
cargo test --test security_history_expansion_cli -- --nocapture
cargo test --test security_shadow_evaluation_cli -- --nocapture
cargo test --test security_model_promotion_cli -- --nocapture
cargo test --test security_decision_submit_approval_cli -- --nocapture
cargo test --test security_scorecard_refit_cli -- --nocapture
cargo test --test security_master_scorecard_cli -- --nocapture
cargo test --test security_chair_resolution_cli -- --nocapture
```

**Closeout**
- append `.trae/CHANGELOG_TASK.md`
- summarize what changed, residual risks, and next suggested tests
