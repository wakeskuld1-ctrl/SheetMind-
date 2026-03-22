# Workbook Theme Redaction Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Re-theme the desensitized workbooks so filenames, sheet names, first-row titles, and sample business semantics all align to one fictional insurer split into property and life insurance divisions.

**Architecture:** Extend the existing Excel desensitization utility with workbook-level theme metadata. Each workbook job will define a target filename, themed sheet-name mapping, first-row header/title aliases, and a product-domain profile so fake rows stay consistent with property insurance, life insurance, or management-center semantics.

**Tech Stack:** Python 3.13, `openpyxl`, Excel COM via `pywin32`, `unittest`

---

### Task 1: Lock the new theme behavior with failing tests

**Files:**
- Modify: `D:/Rust/Excel_Skill/tests/test_excel_desensitize.py`
- Modify: `D:/Rust/Excel_Skill/tools/excel_desensitize.py`

**Step 1: Write the failing test**

Add tests that prove:
- themed workbook processing renames target sheets
- first-row headers/titles are rewritten to property/life/ops terminology
- generated rows for property workbook use property-style products
- generated rows for life workbook use life-style products

**Step 2: Run test to verify it fails**

Run: `python -m unittest tests.test_excel_desensitize -v`
Expected: FAIL because themed metadata rewriting does not exist yet.

**Step 3: Write minimal implementation**

Implement:
- workbook theme config
- first-row alias mapping
- sheet rename helper
- workbook-kind-specific fake data pools

**Step 4: Run test to verify it passes**

Run: `python -m unittest tests.test_excel_desensitize -v`
Expected: PASS

### Task 2: Rebuild the three customer deliverables with the new theme

**Files:**
- Modify: `D:/Rust/Excel_Skill/tools/excel_desensitize.py`

**Step 1: Execute themed export**

Run the utility against the original three source files with the new fictional insurer naming.

**Step 2: Verify workbook metadata**

Open the outputs and confirm:
- filenames match the new naming convention
- sheet names are changed
- row 1 reflects the new terminology

**Step 3: Verify data semantics**

Check that:
- property workbook rows look like property insurance business
- life workbook rows look like life insurance business
- processor workbook uses neutral management terminology

### Task 3: Journal the completion

**Files:**
- Modify: `D:/Rust/Excel_Skill/.trae/CHANGELOG_TASK.md`

**Step 1: Append task journal entry**

Record the re-theme change, why it was needed, verification evidence, and any remaining caveats.
