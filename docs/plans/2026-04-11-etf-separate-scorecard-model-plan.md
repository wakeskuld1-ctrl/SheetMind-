# ETF Separate Scorecard Model Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Split ETF and equity scorecard modeling behavior, add ETF-specific differentiating features, and prevent invalid same-score ETF comparisons from being surfaced as actionable quantitative output.

**Architecture:** Keep the committee/chair/approval governance chain shared, but branch the feature family and model validity logic by `instrument_scope`. Reuse existing technical snapshot fields so the first iteration stays within the current local-history pipeline.

**Tech Stack:** Rust, serde_json, existing security ops toolchain, CLI integration tests.

---

### Task 1: Lock the ETF collapse bug with failing tests

**Files:**
- Modify: `D:\Rust\Excel_Skill\tests\security_scorecard_training_cli.rs`
- Modify: `D:\Rust\Excel_Skill\tests\security_decision_submit_approval_cli.rs`

**Step 1: Write the failing tests**

- Add a test that proves ETF training/config/runtime currently lacks symbol-separating ETF raw features.
- Add a test that proves an ETF scorecard should downgrade when the effective ETF signal space collapses into an invalid same-score comparison.

**Step 2: Run tests to verify they fail**

Run focused test commands for the new cases and confirm the failure reason is missing ETF-specific behavior, not fixture breakage.

**Step 3: Write minimal implementation**

- Do not change more code than necessary before the red state is confirmed.

**Step 4: Re-run focused tests**

- Confirm the tests still fail for the intended reason.

**Step 5: Commit checkpoint**

- Defer git commit until the whole change is stable, but keep the TDD state explicit in notes.

### Task 2: Add ETF-specific feature seeds and training feature configs

**Files:**
- Modify: `D:\Rust\Excel_Skill\src\ops\security_decision_evidence_bundle.rs`
- Modify: `D:\Rust\Excel_Skill\src\ops\security_scorecard_training.rs`

**Step 1: Extend feature seed**

- Add ETF-ready numeric fields from `technical_context.stock_analysis.indicator_snapshot`.
- Keep comments in English with timestamp, reason, and purpose.

**Step 2: Branch training feature configs by instrument scope**

- Keep the existing equity config stable.
- Add an ETF config that includes coarse governance features plus ETF-specific numeric differentiators.

**Step 3: Verify training tests**

- Run focused scorecard training tests and confirm the ETF feature set is discoverable and parseable.

### Task 3: Add ETF runtime invalid-output guard

**Files:**
- Modify: `D:\Rust\Excel_Skill\src\ops\security_scorecard.rs`
- Modify: `D:\Rust\Excel_Skill\tests\security_decision_submit_approval_cli.rs`

**Step 1: Implement ETF validity guard**

- Detect ETF feature collapse in runtime scorecard build.
- Downgrade score status and quant signal when the output is not valid for cross-sectional comparison.

**Step 2: Keep governance output explicit**

- Add limitations text explaining why the ETF output is invalid.
- Ensure chair/approval consumers can see the downgraded status without schema drift.

**Step 3: Verify runtime tests**

- Run the new focused runtime test and adjacent approval/scorecard regressions.

### Task 4: Regression and handoff

**Files:**
- Modify: `D:\Rust\Excel_Skill\.trae\CHANGELOG_TASK.md`

**Step 1: Run regression commands**

- Focus on ETF/equity scorecard training and approval routing tests that cover the touched code paths.

**Step 2: Summarize risks**

- List remaining edge cases, especially around small ETF sample sizes and future external ETF features.

**Step 3: Append task journal**

- Add a new dated entry to `.trae/CHANGELOG_TASK.md` with changes, reason, remaining gaps, and suggested follow-up tests.
