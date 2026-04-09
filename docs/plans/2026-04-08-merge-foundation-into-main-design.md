# Merge Foundation Into Main Design

## Goal

Safely merge the already committed foundation navigation work into `main` without dragging the current dirty worktree state into the merge.

## Why This Needs Isolation

The current workspace contains unrelated uncommitted changes:

- repository cleanup docs
- security line changes
- tool dispatcher changes
- runtime fixture additions

Switching branches directly in this workspace would risk mixing unfinished work into the `main` merge.

## Chosen Approach

Use an isolated Git worktree based on `main`.

Inside that clean worktree:

1. merge `codex/foundation-navigation-kernel`
2. resolve only `README.md` to the current project identity
3. verify merge result
4. leave the current dirty workspace untouched

## Scope

- Merge committed branch history only
- Update `README.md` on `main`
- Record execution notes and task journal

## Non-Goals

- Do not merge current uncommitted security changes
- Do not clean remote branches in this round
- Do not rewrite all historical repo structure in one pass

## Expected Result

- `main` contains the committed foundation navigation kernel
- `README.md` on `main` clearly describes `SheetMind / Excel_Skill`
- the current dirty workspace remains intact for later review
