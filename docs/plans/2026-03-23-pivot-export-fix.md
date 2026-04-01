# Pivot Export Fix Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 修复透视结果导出到 Excel 时的空值与数值类型问题，并在修复后生成一份按渠道的收入透视表。

**Architecture:** 保持现有调用链不变，优先通过失败测试锁定两个问题：透视结果缺失单元格不能导出为字符串 `null`，月份收入列必须以真正数值单元格写入 Excel。随后最小修改 `pivot_table` 与 `export_excel` 的写出逻辑，最后复用修复后的链路生成渠道透视表。

**Tech Stack:** Rust, Polars, rust_xlsxwriter, calamine, cargo test

---

### Task 1: 为透视导出补失败测试

**Files:**
- Modify: `D:/Rust/Excel_Skill/tests/integration_frame.rs`
- Modify: `D:/Rust/Excel_Skill/tests/integration_cli_json.rs`

**Step 1: Write the failing test**
- 增加 frame 测试，锁定透视导出后的空单元格应为空白、数值列应为数值单元格。
- 增加 CLI 测试，锁定 `pivot_table -> export_excel` 链路输出可被 Excel 直接统计。

**Step 2: Run test to verify it fails**
Run: `cargo test pivot_table_export --test integration_frame --test integration_cli_json -v`
Expected: FAIL，当前导出会把值都写成字符串，并把缺失值表现成 `null` 文本。

**Step 3: Write minimal implementation**
- 暂不写实现，此任务只到红灯确认。

**Step 4: Run test to verify it fails clearly**
Run: `cargo test pivot_table_export --test integration_frame --test integration_cli_json -v`
Expected: FAIL with expected assertion mismatch.

### Task 2: 最小修复透视与 Excel 导出类型

**Files:**
- Modify: `D:/Rust/Excel_Skill/src/ops/pivot.rs`
- Modify: `D:/Rust/Excel_Skill/src/ops/export.rs`
- Test: `D:/Rust/Excel_Skill/tests/integration_frame.rs`
- Test: `D:/Rust/Excel_Skill/tests/integration_cli_json.rs`

**Step 1: Write the failing test**
- 复用 Task 1 的失败测试。

**Step 2: Run test to verify it fails**
Run: `cargo test pivot_table_export --test integration_frame --test integration_cli_json -v`
Expected: FAIL

**Step 3: Write minimal implementation**
- 让透视结果的聚合列保留为数值/空值而不是字符串 `null`。
- 让 `export_excel` 在写单元格时按真实类型落盘：空值写空白，数值写 number，其余写 string。

**Step 4: Run test to verify it passes**
Run: `cargo test pivot_table_export --test integration_frame --test integration_cli_json -v`
Expected: PASS

### Task 3: 生成按渠道透视表

**Files:**
- No code changes expected unless发现链路缺口

**Step 1: Reuse fixed binary flow**
- 复用已确认的 `个险长期险台账` 来源。
- 按 `渠道/板块` 行、`会计期间` 列、`经营收入（元）` 求和生成透视。

**Step 2: Export deliverable**
- 导出为新的 xlsx 文件，确保空值为空白、月份收入列为数值单元格。

**Step 3: Verify output**
Run: 使用二进制导出并用工作簿读取确认内容存在。
Expected: 渠道透视表可直接在 Excel 里继续统计。
