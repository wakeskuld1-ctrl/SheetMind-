# Excel Orchestrator V1 Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build `excel-orchestrator-v1` as a top-level routing Skill that maintains a minimal session state summary and routes users into table processing, analysis-modeling, or decision-assistant flows without doing any computation itself.

**Architecture:** Keep the existing three Skills intact and add a fourth top-level Skill that defines the routing rules, state-summary protocol, and cross-layer handoff conventions. The orchestrator will treat `table_ref` as the primary confirmed-state handle and will only fall back to `path + sheet` when no confirmed handle exists.

**Tech Stack:** Markdown Skills, existing Skill validation script, existing acceptance-dialogue pattern, Rust Tool layer contracts.

---

### Task 1: Create orchestrator acceptance cases first

**Files:**
- Create: `D:/Rust/Excel_Skill/skills/excel-orchestrator-v1/acceptance-dialogues.md`
- Create: `D:/Rust/Excel_Skill/skills/excel-orchestrator-v1/cases.md`

**Step 1: Write the failing acceptance coverage**
- Add dialogue cases for:
  - user only says “先看看这个 Excel” -> route to table-processing
  - user has confirmed `table_ref` and asks “先看统计” -> route to analysis-modeling
  - user has confirmed `table_ref` and asks “下一步该做什么” -> route to decision-assistant
  - user asks for modeling without confirmed state -> route back to table-processing first

**Step 2: Review the cases and verify each one maps to exactly one layer**

Run:

```powershell
Get-Content D:/Rust/Excel_Skill/skills/excel-orchestrator-v1/cases.md
Get-Content D:/Rust/Excel_Skill/skills/excel-orchestrator-v1/acceptance-dialogues.md
```

Expected:
- Every case clearly identifies the target layer and the reason.

### Task 2: Write the orchestrator Skill document

**Files:**
- Create: `D:/Rust/Excel_Skill/skills/excel-orchestrator-v1/SKILL.md`
- Create: `D:/Rust/Excel_Skill/skills/excel-orchestrator-v1/requests.md`

**Step 1: Write the minimal Skill body**
- Describe:
  - when this Skill triggers
  - what state it tracks conceptually
  - how it routes to the three child Skills
  - why `table_ref` is the preferred cross-layer handle

**Step 2: Keep the Skill narrow**
- Do not duplicate the full logic of child Skills.
- Do not introduce computation or business decision rules.
- Only include high-level routing and state-summary rules.

**Step 3: Validate structure**

Run:

```powershell
# 开发期结构校验（仅研发辅助，不属于客户运行依赖）
python -X utf8 C:/Users/wakes/.codex/skills/.system/skill-creator/scripts/quick_validate.py D:/Rust/Excel_Skill/skills/excel-orchestrator-v1
```

Expected:
- PASS

### Task 3: Define local-memory-runtime-v1 design

**Files:**
- Create: `D:/Rust/Excel_Skill/docs/plans/2026-03-22-local-memory-runtime-v1-design.md`

**Step 1: Document the local memory requirements**
- State clearly that memory must be:
  - local
  - independent from LLM context
  - outside Skill files
  - suitable for binary packaging

**Step 2: Recommend a concrete storage design**
- Prefer SQLite as the V1/V2 baseline.
- Define the minimal tables:
  - `sessions`
  - `table_refs`
  - `session_state`
  - `model_contexts`
  - `event_logs`
  - `user_preferences`

**Step 3: Document integration points**
- Explain how orchestrator and child Skills will conceptually read/write through this runtime later.

### Task 4: Planning file and journal closure

**Files:**
- Modify: `D:/Rust/Excel_Skill/task_plan.md`
- Modify: `D:/Rust/Excel_Skill/findings.md`
- Modify: `D:/Rust/Excel_Skill/progress.md`
- Modify: `D:/Rust/Excel_Skill/.trae/CHANGELOG_TASK.md`

**Step 1: Update planning files**
- Record:
  - orchestrator design decision
  - local memory independence decision
  - why SQLite is the preferred direction

**Step 2: Append task journal**
- Add a new entry covering:
  - orchestrator design doc
  - orchestrator implementation plan
  - local memory design doc

**Step 3: Final verification**

Run:

```powershell
# 开发期结构校验（仅研发辅助，不属于客户运行依赖）
python -X utf8 C:/Users/wakes/.codex/skills/.system/skill-creator/scripts/quick_validate.py D:/Rust/Excel_Skill/skills/excel-orchestrator-v1
```

Expected:
- PASS
