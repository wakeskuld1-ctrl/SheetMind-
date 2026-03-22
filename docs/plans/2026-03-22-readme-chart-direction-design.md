# README Chart Direction Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add a chart-capability direction section to the GitHub README and align the roadmap wording with the project's binary-first positioning.

**Architecture:** This is a documentation-only change. The README will gain a small standalone chart-direction section, while the roadmap and next-stage wording will mention binary-first chart generation without overpromising chart editing for existing customer workbooks.

**Tech Stack:** Markdown, UTF-8 text editing

---

### Task 1: Inspect the README insertion points

**Files:**
- Modify: `D:/Rust/Excel_Skill/README.md`

**Step 1: Read the roadmap and next-stage sections**

Run: `Get-Content -Path 'D:/Rust/Excel_Skill/README.md' -Encoding UTF8 -Tail 140`

Expected: the current roadmap and next-stage sections are visible in UTF-8 text.

**Step 2: Decide the insertion point**

Insert the standalone chart section between `Roadmap / 路线图` and `Next Stage / 下一阶段`.

**Step 3: Define the wording**

Keep the wording limited to:
- binary-first chart generation
- common charts such as line, pie, column, and scatter
- chart output built from confirmed tables and modeling results
- no promise yet for editing existing charts inside customer files

### Task 2: Update the README

**Files:**
- Modify: `D:/Rust/Excel_Skill/README.md`

**Step 1: Add roadmap bullets**

Add chart-generation direction into near-term and longer-term roadmap bullets.

**Step 2: Add a standalone section**

Add `## Chart Capability Direction / 图表能力方向` with Chinese and English subsections.

**Step 3: Align the next-stage wording**

Mention chart output as part of the next-stage product path while keeping the current scope honest.

### Task 3: Verify and journal

**Files:**
- Modify: `D:/Rust/Excel_Skill/.trae/CHANGELOG_TASK.md`

**Step 1: Verify the README**

Run: `Select-String -Path 'D:/Rust/Excel_Skill/README.md' -Pattern 'Chart Capability Direction','图表能力方向','line','pie','column','scatter'`

Expected: the new section and chart keywords are present.

**Step 2: Append the task journal**

Append a new dated entry to `D:/Rust/Excel_Skill/.trae/CHANGELOG_TASK.md` describing the README chart-direction update and GitHub copy support.
