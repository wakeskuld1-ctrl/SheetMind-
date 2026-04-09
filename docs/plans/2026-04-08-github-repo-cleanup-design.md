# GitHub Repo Cleanup Design

## Goal

Make the GitHub repository reflect the real current product direction: `SheetMind / Excel_Skill`, Rust-first, CLI-first, foundation-first.

## Scope

- Rewrite the repository homepage entry in `README.md`.
- Repair the top-level onboarding entry in `AI_START_HERE.md`.
- Add one repository governance note for directory and branch meaning.
- Clean only local stale branches that are already invalid or superseded.

## Non-Goals

- Do not delete valid remote branches in this round.
- Do not refactor business code.
- Do not rewrite historical plans in bulk.
- Do not try to remove all Python-origin traces from the repository in one pass.

## Design

### 1. GitHub homepage

`README.md` should stop presenting the repository as `TradingAgents`.
It should instead explain:

- what the repository is now
- what the current active direction is
- where to start reading
- what parts are active vs historical

### 2. AI / engineer onboarding entry

`AI_START_HERE.md` should become a clean UTF-8 Chinese entry file with a short read order and frozen architecture rules.

### 3. Branch clarity

This round only cleans local stale branches:

- branch with gone upstream and already superseded elsewhere
- branch with gone upstream and already merged

Remote branches stay unchanged in this round to avoid destructive cleanup.

### 4. Governance note

Add one compact repo note to explain:

- root directory responsibilities
- active lines vs historical lines
- branch naming expectations

## Expected Result

- GitHub homepage no longer misleads visitors.
- New AI sessions can find the correct entry path immediately.
- Local branch list is cleaner and easier to reason about.
- The repository becomes easier to present externally without risky structural churn.
