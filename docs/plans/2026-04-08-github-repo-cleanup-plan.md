# GitHub Repo Cleanup Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Clean the repository's GitHub-facing presentation and local stale branches without touching business logic.

**Architecture:** This round is documentation-first and governance-first. We will correct the public repository entry points, add a compact repo/branch guidance file, and only remove local branches that are clearly stale and already superseded or merged.

**Tech Stack:** Markdown, Git, PowerShell

---

### Task 1: Rewrite GitHub-facing entry files

**Files:**
- Modify: `D:\Rust\Excel_Skill\README.md`
- Modify: `D:\Rust\Excel_Skill\AI_START_HERE.md`

**Step 1: Replace outdated repository identity text**

Rewrite both files so they describe:
- `SheetMind / Excel_Skill`
- Rust / EXE / CLI-first direction
- foundation-first architecture

**Step 2: Add clean read-order links**

Ensure both files point to:
- `docs/ai-memory/project-baseline.md`
- `docs/ai-handoff/AI_HANDOFF_MANUAL.md`
- latest foundation execution note

**Step 3: Review rendered content**

Run: `Get-Content D:\Rust\Excel_Skill\README.md -TotalCount 120`
Expected: no TradingAgents title remains in the top section

### Task 2: Add repository governance note

**Files:**
- Create: `D:\Rust\Excel_Skill\docs\architecture\repo-and-branch-governance.md`

**Step 1: Write a compact governance note**

Include:
- active product direction
- root directory meaning
- branch naming expectations
- current cleanup rule: local stale branches can be removed, remote business branches stay

**Step 2: Link it from README**

Add a pointer in `README.md` so GitHub visitors can find it quickly.

### Task 3: Clean stale local branches

**Files:**
- No file edits

**Step 1: Remove merged stale local branch**

Run: `git branch -d codex/p0-preflight-chain`

**Step 2: Remove superseded stale local branch**

Run: `git branch -d codex/cli-mod-review`
If Git refuses because it is not fully merged, verify the commit is contained elsewhere and use `-D` only if necessary.

**Step 3: Re-check branch clarity**

Run: `git branch -vv`
Expected: stale local branches no longer appear

### Task 4: Verify and record

**Files:**
- Modify: `D:\Rust\Excel_Skill\.trae\CHANGELOG_TASK.md`

**Step 1: Verify repository state**

Run: `git status --short --branch`
Expected: only intended documentation changes remain in the worktree

**Step 2: Append task journal entry**

Add an append-only entry describing:
- README cleanup
- onboarding entry cleanup
- governance note
- local branch cleanup

**Step 3: Report without claiming unrun tests**

If no code tests are needed, say clearly that this round was docs/branch cleanup only.
