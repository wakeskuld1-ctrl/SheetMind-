# P5 Shadow Champion And History Expansion Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Upgrade the current securities training chain from governed `candidate` outputs to a governed `candidate -> shadow -> champion` lifecycle while expanding historical external-proxy coverage into a first-class auditable dependency.

**Architecture:** Reuse the existing `security_scorecard_training -> security_scorecard_refit -> security_scorecard_model_registry -> security_decision_submit_approval` chain instead of introducing a parallel promotion system. Add one governed history-expansion record, one governed shadow evaluation record, and one governed champion-promotion decision path, then make approval and decision-package consumers read model grade explicitly.

**Tech Stack:** Rust, serde/serde_json, chrono, existing securities CLI dispatcher, runtime JSON stores under `.excel_skill_runtime`, current scorecard registry/refit/approval chain, focused CLI tests under `tests/`

---

### Recommended Approach

**Approach A: Build promotion states first**

- Pros:
  - Fastest route to explicit `candidate / shadow / champion` semantics.
  - Approval chain hardening lands quickly.
- Cons:
  - Promotion remains weakly grounded if historical proxy coverage is still thin.
  - Many ETF pools will still be blocked by missing history evidence.

**Approach B: Expand historical proxies first**

- Pros:
  - Stronger data foundation before governance.
  - Promotion decisions become more credible later.
- Cons:
  - Approval chain still lacks explicit grade semantics in the short term.
  - User-visible governance progress is slower.

**Approach C: Do both in one governed phase, in sequence**

- Pros:
  - Best balance of data correctness and governance readiness.
  - Avoids reworking registry and approval contracts twice.
- Cons:
  - Broader change set than A or B alone.
  - Requires disciplined TDD and careful boundaries.

**Recommendation:** Approach C. First add governed history-expansion records, then add shadow evaluation and champion promotion records, then make approval/decision-package flows consume model grade explicitly.

---

### Task 1: Lock history-expansion and model-grade contracts with failing tests

**Files:**
- Create: `D:\Rust\Excel_Skill\tests\security_history_expansion_cli.rs`
- Modify: `D:\Rust\Excel_Skill\tests\security_decision_submit_approval_cli.rs`
- Modify: `D:\Rust\Excel_Skill\tests\security_scorecard_refit_cli.rs`

**Step 1: Write the failing tests**

- Add a CLI test proving a new governed tool can persist a history-expansion record with:
  - `market_scope`
  - `instrument_scope`
  - `instrument_subscope`
  - covered proxy fields
  - covered date ranges
- Add a refit/promotion red test proving model-grade fields can move beyond plain `candidate`.
- Add an approval red test proving non-`champion` models cannot be treated as full release-grade quant approval.

**Step 2: Run tests to verify they fail**

Run:

```powershell
cargo test --test security_history_expansion_cli -- --nocapture
cargo test --test security_scorecard_refit_cli -- --nocapture
cargo test --test security_decision_submit_approval_cli -- --nocapture
```

Expected: FAIL because there is no governed history-expansion record yet and approval/registry consumers do not yet understand `shadow/champion`.

### Task 2: Add governed historical-proxy expansion records

**Files:**
- Create: `D:\Rust\Excel_Skill\src\ops\security_history_expansion.rs`
- Modify: `D:\Rust\Excel_Skill\src\ops\stock.rs`
- Modify: `D:\Rust\Excel_Skill\src\ops\mod.rs`
- Modify: `D:\Rust\Excel_Skill\src\tools\catalog.rs`
- Modify: `D:\Rust\Excel_Skill\src\tools\dispatcher.rs`
- Modify: `D:\Rust\Excel_Skill\src\tools\dispatcher\stock_ops.rs`
- Create: `D:\Rust\Excel_Skill\tests\security_history_expansion_cli.rs`

**Step 1: Write the minimal implementation**

- Define a governed history-expansion request/result contract.
- Persist a dated expansion record keyed by:
  - market/instrument scope
  - optional instrument subscope
  - covered date range
  - covered proxy field list
- Expose the tool to the CLI catalog.

**Step 2: Run focused tests**

Run:

```powershell
cargo test --test security_history_expansion_cli -- --nocapture
```

Expected: PASS with deterministic persisted history-expansion records.

### Task 3: Extend refit/registry with model-grade semantics

**Files:**
- Modify: `D:\Rust\Excel_Skill\src\ops\security_scorecard_model_registry.rs`
- Modify: `D:\Rust\Excel_Skill\src\ops\security_scorecard_refit_run.rs`
- Modify: `D:\Rust\Excel_Skill\tests\security_scorecard_refit_cli.rs`

**Step 1: Write the minimal implementation**

- Add explicit model-grade fields:
  - `candidate`
  - `shadow`
  - `champion`
- Keep current compatibility with existing candidate runs.
- Make refit/registry results persist and expose the new grade semantics.

**Step 2: Run focused tests**

Run:

```powershell
cargo test --test security_scorecard_refit_cli -- --nocapture
```

Expected: PASS with registry/refit outputs exposing governed grade state.

### Task 4: Add shadow evaluation and promotion-decision records

**Files:**
- Create: `D:\Rust\Excel_Skill\src\ops\security_shadow_evaluation.rs`
- Create: `D:\Rust\Excel_Skill\src\ops\security_model_promotion.rs`
- Modify: `D:\Rust\Excel_Skill\src\ops\stock.rs`
- Modify: `D:\Rust\Excel_Skill\src\ops\mod.rs`
- Modify: `D:\Rust\Excel_Skill\src\tools\catalog.rs`
- Modify: `D:\Rust\Excel_Skill\src\tools\dispatcher.rs`
- Modify: `D:\Rust\Excel_Skill\src\tools\dispatcher\stock_ops.rs`
- Create: `D:\Rust\Excel_Skill\tests\security_shadow_evaluation_cli.rs`
- Create: `D:\Rust\Excel_Skill\tests\security_model_promotion_cli.rs`

**Step 1: Write the minimal implementation**

- Add a governed shadow-evaluation record that summarizes:
  - sample readiness
  - class balance
  - path-event coverage
  - proxy coverage
  - current recommended grade
- Add a promotion record that decides whether a model remains `candidate`, upgrades to `shadow`, or upgrades to `champion`.

**Step 2: Run focused tests**

Run:

```powershell
cargo test --test security_shadow_evaluation_cli -- --nocapture
cargo test --test security_model_promotion_cli -- --nocapture
```

Expected: PASS with deterministic shadow/promotion documents and stable grade outcomes.

### Task 5: Make approval and decision package consume model grade explicitly

**Files:**
- Modify: `D:\Rust\Excel_Skill\src\ops\security_decision_submit_approval.rs`
- Modify: `D:\Rust\Excel_Skill\src\ops\security_decision_approval_brief.rs`
- Modify: `D:\Rust\Excel_Skill\src\ops\security_decision_package.rs`
- Modify: `D:\Rust\Excel_Skill\tests\security_decision_submit_approval_cli.rs`

**Step 1: Write the minimal implementation**

- Let approval flow read model grade from the registry/promotion context.
- Enforce:
  - `champion`: full quant approval semantics allowed
  - `shadow`: reference-only quant context, not full release-grade
  - `candidate/unavailable`: governance-only, not release-grade
- Attach grade summary into approval brief and decision package.

**Step 2: Run focused tests**

Run:

```powershell
cargo test --test security_decision_submit_approval_cli -- --nocapture
```

Expected: PASS with grade-aware approval outcomes and package summaries.

### Task 6: Run adjacent regressions, document residual risks, and close the phase

**Files:**
- Modify: `D:\Rust\Excel_Skill\.trae\CHANGELOG_TASK.md`
- Modify if needed: `D:\Rust\Excel_Skill\docs\plans\2026-04-11-p5-shadow-champion-and-history-expansion-design.md`

**Step 1: Run adjacent regressions**

Run:

```powershell
cargo test --test security_history_expansion_cli -- --nocapture
cargo test --test security_scorecard_refit_cli -- --nocapture
cargo test --test security_shadow_evaluation_cli -- --nocapture
cargo test --test security_model_promotion_cli -- --nocapture
cargo test --test security_decision_submit_approval_cli -- --nocapture
cargo test --test security_master_scorecard_cli -- --nocapture
cargo test --test security_chair_resolution_cli -- --nocapture
```

Expected: PASS for the relevant governed chain.

**Step 2: Append task journal entry**

- Add a dated entry to `.trae/CHANGELOG_TASK.md`.
- Record:
  - what changed
  - why it changed
  - what still blocks full production-grade champion promotion
  - recommended follow-up tests

---

### Done-State Checklist

- Historical proxy expansion has a governed document and CLI tool.
- Registry/refit outputs expose explicit model-grade semantics.
- Shadow evaluation and promotion decisions are formally persisted.
- Approval and decision package flows consume model grade instead of treating all artifacts equally.
- The phase is documented and executable task-by-task without rediscovery.
