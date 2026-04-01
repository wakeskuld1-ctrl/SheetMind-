# Skill TableRef Bridge + Decision Assistant V1 Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Finish the V1 upper-layer route by making Skills default to `table_ref`, tightening logistic-regression precheck guidance, adding a decision-assistant Skill, and running a real-file end-to-end acceptance round.

**Architecture:** Reuse the already-landed persistent `table_ref` bridge as the handoff contract between table-processing and analysis-modeling. Keep Tool computation in Rust, and let Skill docs/routing own only intent recognition, non-technical questioning, and result explanation. Where low-IT guidance is still too weak, add the smallest possible Tool/test reinforcement instead of inventing Skill-side computation.

**Tech Stack:** Rust CLI, serde_json, markdown Skills, acceptance artifacts, existing integration tests.

---

### Task 1: Lock Step 1 with tests and route docs

**Files:**
- Modify: `D:/Rust/Excel_Skill/tests/integration_cli_json.rs`
- Modify: `D:/Rust/Excel_Skill/skills/table-processing-v1/SKILL.md`
- Modify: `D:/Rust/Excel_Skill/skills/analysis-modeling-v1/SKILL.md`
- Modify: `D:/Rust/Excel_Skill/skills/analysis-modeling-v1/requests.md`
- Modify: `D:/Rust/Excel_Skill/skills/analysis-modeling-v1/acceptance-dialogues.md`

**Step 1: Write the failing test**
- Add a CLI integration test proving `decision_assistant` can also consume a reusable `table_ref` after `apply_header_schema`.
- Add or update acceptance text that explicitly requires “表处理层确认态 -> table_ref -> 分析建模层 / 决策助手层复用”.

**Step 2: Run test to verify it fails or expose the current gap**

Run:

```powershell
cargo test decision_assistant_accepts_table_ref_from_apply_header_schema --test integration_cli_json -- --exact
```

Expected:
- If it already passes, treat it as a verification gap and keep the test.
- If it fails, use the failure to drive the minimal fix.

**Step 3: Write minimal implementation / doc update**
- Update Skill docs so the default narrative becomes:
  - table-processing confirms structure
  - returns `table_ref`
  - analysis-modeling and decision-assistant prefer `table_ref`
  - only fall back to `path + sheet` when no `table_ref` exists

**Step 4: Re-run the test**

```powershell
cargo test decision_assistant_accepts_table_ref_from_apply_header_schema --test integration_cli_json -- --exact
```

Expected:
- PASS

### Task 2: Tighten logistic-regression precheck guidance with TDD

**Files:**
- Modify: `D:/Rust/Excel_Skill/tests/integration_cli_json.rs`
- Modify: `D:/Rust/Excel_Skill/src/ops/model_prep.rs`
- Modify: `D:/Rust/Excel_Skill/skills/analysis-modeling-v1/SKILL.md`
- Modify: `D:/Rust/Excel_Skill/skills/analysis-modeling-v1/requests.md`
- Modify: `D:/Rust/Excel_Skill/skills/analysis-modeling-v1/acceptance-dialogues.md`

**Step 1: Write the failing test**
- Add a CLI test proving single-class logistic targets return a readable, non-technical action-oriented error.

**Step 2: Run test to verify it fails**

Run:

```powershell
cargo test logistic_regression_reports_single_class_target_with_actionable_guidance --test integration_cli_json -- --exact
```

Expected:
- FAIL because the current error is readable but not yet guidance-rich enough.

**Step 3: Write minimal implementation**
- Keep computation in Tool.
- Improve the single-class / positive-label related error text only enough to tell the user what to do next.
- Sync Skill docs so they explicitly ask for target, positive label, and suggest checking target distribution first when needed.

**Step 4: Re-run the test**

```powershell
cargo test logistic_regression_reports_single_class_target_with_actionable_guidance --test integration_cli_json -- --exact
```

Expected:
- PASS

### Task 3: Add decision-assistant Skill V1

**Files:**
- Create: `D:/Rust/Excel_Skill/skills/decision-assistant-v1/SKILL.md`
- Create: `D:/Rust/Excel_Skill/skills/decision-assistant-v1/requests.md`
- Create: `D:/Rust/Excel_Skill/skills/decision-assistant-v1/cases.md`
- Create: `D:/Rust/Excel_Skill/skills/decision-assistant-v1/acceptance-dialogues.md`

**Step 1: Write the baseline acceptance cases**
- Define scenarios for:
  - 质量诊断优先
  - 有阻塞风险时不直接建议建模
  - 可继续建模时建议下一步 Tool
  - 基于 `table_ref` 进入决策助手

**Step 2: Write minimal Skill docs**
- Keep it route-only, no computation.
- Use non-technical Chinese for low-IT users.
- Reuse `decision_assistant` Tool as the main high-level entry.

**Step 3: Validate Skill structure**

Run:

```powershell
# 开发期结构校验（仅研发辅助，不属于客户运行依赖）
python -X utf8 C:/Users/wakes/.codex/skills/.system/skill-creator/scripts/quick_validate.py D:/Rust/Excel_Skill/skills/decision-assistant-v1
```

Expected:
- PASS

### Task 4: Real-file V1 acceptance round

**Files:**
- Create or Modify: `D:/Rust/Excel_Skill/docs/acceptance/2026-03-22-v1-final-e2e-real-file.md`
- Create or Modify: `D:/Rust/Excel_Skill/docs/acceptance/artifacts/2026-03-22-v1-final-e2e-real-file/*`
- Modify: `D:/Rust/Excel_Skill/task_plan.md`
- Modify: `D:/Rust/Excel_Skill/findings.md`
- Modify: `D:/Rust/Excel_Skill/progress.md`
- Modify: `D:/Rust/Excel_Skill/.trae/CHANGELOG_TASK.md`

**Step 1: Re-run real workbook flow**
- Use `D:/Excel测试/新疆客户/2026文旅体台账.xlsx`.
- Walk through:
  - table-processing confirmation
  - `apply_header_schema` -> `table_ref`
  - analysis-modeling by `table_ref`
  - decision-assistant by `table_ref`

**Step 2: Save request/response artifacts**
- Keep every request and response as JSON.
- Record what the Skill should ask, what Tool actually returns, and where V1 still stops.

**Step 3: Final verification**

Run:

```powershell
cargo test -v
```

Expected:
- All tests pass
