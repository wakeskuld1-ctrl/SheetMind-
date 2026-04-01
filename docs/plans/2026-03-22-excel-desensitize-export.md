# Excel Desensitize Export Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Copy the three specified workbooks into a target folder and replace sensitive business rows with logically consistent fake insurance data while preserving workbook structure.

**Architecture:** Build a standalone Python utility that copies the source files, opens workbook copies with `openpyxl`/`keep_vba`, detects the header row per target sheet, and rewrites only the data region. Use deterministic generators keyed by header names so dates, premiums, seasonality, insurer names, customer names, and people names remain internally coherent.

**Tech Stack:** Python 3.13, `openpyxl`, `unittest`, filesystem copy

---

### Task 1: Define the target behavior with tests

**Files:**
- Create: `D:/Rust/Excel_Skill/tests/test_excel_desensitize.py`
- Create: `D:/Rust/Excel_Skill/tools/excel_desensitize.py`

**Step 1: Write the failing test**

Write tests that prove:
- only requested sheets are rewritten
- header rows remain unchanged
- data rows are replaced with fake values
- generated monthly income contains low-season and high-season differences

**Step 2: Run test to verify it fails**

Run: `python -m unittest tests.test_excel_desensitize -v`
Expected: FAIL because the utility does not exist yet.

**Step 3: Write minimal implementation**

Implement:
- workbook copy helper
- target sheet selector
- header-driven fake row generator
- workbook rewrite entrypoint

**Step 4: Run test to verify it passes**

Run: `python -m unittest tests.test_excel_desensitize -v`
Expected: PASS

### Task 2: Run the utility against the real files

**Files:**
- Modify: `D:/Rust/Excel_Skill/tools/excel_desensitize.py`

**Step 1: Dry-run inspection**

Inspect source workbooks and confirm target sheet names.

**Step 2: Execute rewrite**

Run the utility to copy files into `D:/Excel测试/脱敏数据` and rewrite the selected sheets.

**Step 3: Verify outputs**

Open the output workbooks programmatically and verify:
- files exist
- sheet counts match expectations
- headers remain intact
- original-looking sensitive values are gone from rewritten rows

### Task 3: Record completion

**Files:**
- Modify: `D:/Rust/Excel_Skill/.trae/CHANGELOG_TASK.md`

**Step 1: Append task journal entry**

Record what changed, why it changed, remaining gaps, and verification status.
