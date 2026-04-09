# Merge Foundation Into Main Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Merge the committed foundation branch into `main` safely while keeping the current dirty workspace untouched.

**Architecture:** Use an isolated worktree checked out from `main`, merge `codex/foundation-navigation-kernel` there, refresh `README.md` to match the actual project identity, verify the merge, and record the delivery context. The original workspace stays dirty but untouched.

**Tech Stack:** Git worktrees, Markdown, PowerShell

---

### Task 1: Create isolated merge workspace

**Files:**
- No file edits

**Step 1: Create a dedicated worktree from `main`**

Run: `git worktree add C:\Users\wakes\.codex\worktrees\Excel_Skill\codex-merge-to-main-prep main`
Expected: a clean worktree is created from `main`

**Step 2: Verify the worktree is clean**

Run: `git status --short --branch`
Expected: no uncommitted changes

### Task 2: Merge committed foundation branch into main

**Files:**
- Merge branch history

**Step 1: Merge current committed branch**

Run: `git merge --no-ff codex/foundation-navigation-kernel`
Expected: merge completes, or stops only for explicit conflicts

**Step 2: Inspect the merge result**

Run: `git diff --stat ORIG_HEAD..HEAD`
Expected: foundation branch committed history is now present on `main`

### Task 3: Refresh README on merged main

**Files:**
- Modify: `D:\Rust\Excel_Skill\README.md` in the merge worktree

**Step 1: Review merged README**

Run: `Get-Content README.md -TotalCount 120`
Expected: confirm whether merge brought the outdated project identity back

**Step 2: Rewrite README if needed**

Ensure `README.md` clearly states:
- `SheetMind / Excel_Skill`
- Rust / EXE / CLI-first
- foundation-first direction
- correct read order for new readers

### Task 4: Verify and record

**Files:**
- Create or modify: `docs/execution-notes-2026-04-08-merge-foundation-into-main.md`
- Modify: `.trae/CHANGELOG_TASK.md`

**Step 1: Verify merged branch state**

Run: `git status --short --branch`
Expected: only intended merge-era docs changes remain, or clean if everything committed

**Step 2: Verify README top section**

Run: `Select-String -Path README.md -Pattern "^# SheetMind / Excel_Skill","TradingAgents"`
Expected: title matches current project identity and no misleading top-level old title remains

**Step 3: Record execution notes and task journal**

Document:
- why isolation was used
- what branch was merged
- what was intentionally excluded
- what remains outside this merge
