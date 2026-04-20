# Betting Workbook Color Highlighting Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add approved red highlighting so risky loss cells and adjusted recommendation cells are visually obvious in the betting workbook delivery.

**Architecture:** Sheet 1 uses Excel conditional formatting so `亏损额` cells turn red dynamically as operators change stakes. Sheet 2 uses direct cell formats because the solver output is already fixed at write time, so refund cells and positive-loss-risk profit/loss cells can be colored deterministically during workbook generation.

**Tech Stack:** Rust, `rust_xlsxwriter`, workbook XML regression tests via `zip`, existing betting workbook bridge tests.

---

### Task 1: Lock the expected workbook styling behavior with failing tests

**Files:**
- Modify: `D:\Rust\Excel_Skill\tests\betting_workbook_bridge_cli.rs`
- Test: `D:\Rust\Excel_Skill\tests\betting_workbook_bridge_cli.rs`

**Step 1: Write the failing tests**

- Add one test that generates the template workbook and asserts sheet 1 XML contains a conditional formatting rule over the `亏损额` columns.
- Add one test that generates the solved workbook and asserts style references differ for:
  - a refunded cell on sheet 2
  - a positive current/adjusted `盈亏额` cell on sheet 2

**Step 2: Run test to verify it fails**

Run: `cargo test --test betting_workbook_bridge_cli color -- --nocapture`

Expected: FAIL because the current workbook writer does not yet emit the new color rules.

### Task 2: Implement minimal workbook highlighting support

**Files:**
- Modify: `D:\Rust\Excel_Skill\src\ops\betting_workbook_bridge.rs`

**Step 1: Add sheet 1 conditional formatting**

- Introduce a reusable red risk format.
- Apply conditional formatting to each `亏损额` cell range with rule `> 0`.

**Step 2: Add sheet 2 fixed red cell formats**

- Add red-highlighted numeric/text formats for adjusted cells and risk cells.
- Use red numeric format when:
  - `refund_amount > 0`
  - `current_pnl > 0`
  - `entry.pnl_value > 0`
- Keep non-risk cells on the existing neutral format.

**Step 3: Keep code localized**

- Limit the change to workbook formatting helpers and `write_suggestion_sheet`/`write_current_sheet`.
- Avoid changing solver contract or workbook structure.

### Task 3: Verify the new behavior and refresh delivery outputs

**Files:**
- Modify: `D:\Rust\Excel_Skill\outputs\betting_optimizer_delivery\计算器s6.1_稳交付版_页面还原.xlsm`
- Modify: `D:\Rust\Excel_Skill\outputs\betting_optimizer_delivery\计算器s6.1_稳交付版_页面还原-结果.xlsm`

**Step 1: Run focused tests**

Run: `cargo test --test betting_workbook_bridge_cli -- --nocapture`

Expected: PASS with the new workbook styling checks included.

**Step 2: Rebuild delivery workbooks**

Run:

```powershell
& 'D:\Rust\Excel_Skill\target\release\betting_solver.exe' template 'D:\Rust\Excel_Skill\outputs\betting_optimizer_delivery\计算器s6.1_稳交付版_页面还原.xlsm'
& 'D:\Rust\Excel_Skill\target\release\betting_solver.exe' solve 'D:\Rust\Excel_Skill\outputs\betting_optimizer_delivery\计算器s6.1_稳交付版_页面还原.xlsm' 'D:\Rust\Excel_Skill\outputs\betting_optimizer_delivery\计算器s6.1_稳交付版_页面还原-结果.xlsm'
```

**Step 3: Spot-check final workbook evidence**

- Confirm sheet 1 still opens with conditional formatting rules.
- Confirm sheet 2 still contains `优化摘要` and the approved red-highlighted cells.
