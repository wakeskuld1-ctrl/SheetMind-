# Path Recovery Skill Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Improve the orchestrator and table-processing Skills so they can correctly explain and recover from Windows path format errors and Chinese-path compatibility failures.

**Architecture:** Keep the change at the Skill/documentation layer in this round. Add one shared design document, then update the orchestrator and table-processing rules, request guidance, case mappings, and acceptance dialogues so both layers use the same recovery order and the same user-facing wording.

**Tech Stack:** Markdown Skill files、UTF-8 text updates、开发期结构校验脚本（仅研发辅助，不属于客户运行依赖）。

---

### Task 1: Write the path-recovery design document

**Files:**
- Create: `D:/Rust/Excel_Skill/docs/plans/2026-03-22-path-recovery-skill-design.md`

**Step 1: Document the two failure classes**
- Windows path syntax errors
- Chinese-path compatibility failures

**Step 2: Document the recovery order**
- First path-format correction
- Then ASCII temp-copy fallback
- Only then file-unreadable conclusion

### Task 2: Update the orchestrator Skill

**Files:**
- Modify: `D:/Rust/Excel_Skill/skills/excel-orchestrator-v1/SKILL.md`
- Modify: `D:/Rust/Excel_Skill/skills/excel-orchestrator-v1/requests.md`
- Modify: `D:/Rust/Excel_Skill/skills/excel-orchestrator-v1/cases.md`
- Modify: `D:/Rust/Excel_Skill/skills/excel-orchestrator-v1/acceptance-dialogues.md`

**Step 1: Add entry-recovery routing rules**
- Explain when the orchestrator should classify the issue as a file-entry recovery problem.

**Step 2: Add wording guidance**
- Use the user-provided three-part explanations as the target wording style.

**Step 3: Add at least one acceptance scene**
- A failed first open caused by path syntax.

### Task 3: Update the table-processing Skill

**Files:**
- Modify: `D:/Rust/Excel_Skill/skills/table-processing-v1/SKILL.md`
- Modify: `D:/Rust/Excel_Skill/skills/table-processing-v1/requests.md`
- Modify: `D:/Rust/Excel_Skill/skills/table-processing-v1/cases.md`
- Modify: `D:/Rust/Excel_Skill/skills/table-processing-v1/acceptance-dialogues.md`

**Step 1: Add recovery-order rules**
- Correct path format first.
- Then try the ASCII-copy fallback for Chinese-path compatibility.

**Step 2: Add request/template guidance**
- Make it explicit that path recovery is a host-level prep step, not an Excel computation Tool.

**Step 3: Add acceptance scenes**
- Path-format correction
- Chinese-path ASCII temp-copy fallback

### Task 4: Validate and journal

**Files:**
- Modify: `D:/Rust/Excel_Skill/task_plan.md`
- Modify: `D:/Rust/Excel_Skill/findings.md`
- Modify: `D:/Rust/Excel_Skill/progress.md`
- Modify: `D:/Rust/Excel_Skill/.trae/CHANGELOG_TASK.md`

**Step 1: Run UTF-8 reads**
- Read back the modified files with a UTF-8-safe development helper.

**Step 2: Run Skill validation**
- Validate both `excel-orchestrator-v1` and `table-processing-v1`.

**Step 3: Record the change**
- Update plan files and task journal with the new path-recovery rules.
