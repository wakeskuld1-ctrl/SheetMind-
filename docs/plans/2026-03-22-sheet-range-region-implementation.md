# Sheet Range / Region Tool Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 新增 `inspect_sheet_range` 与 `load_table_region`，让 Excel 表处理层具备“先探查区域，再显式加载区域”的保守能力。

**Architecture:** 新增一个 Excel 区域模块负责 used range 扫描与 A1 区域解析，再新增一个 frame 区域装载模块负责把显式区域转成 `LoadedTable`。`load_table_region` 第一版返回 `result_ref`，避免扩大 `table_ref` 持久化边界。

**Tech Stack:** Rust 2024、calamine、polars、serde、现有 CLI JSON Tool 调度架构

---

### Task 1: `inspect_sheet_range`

**Files:**
- Create: `D:/Rust/Excel_Skill/src/excel/sheet_range.rs`
- Modify: `D:/Rust/Excel_Skill/src/excel/mod.rs`
- Modify: `D:/Rust/Excel_Skill/src/tools/contracts.rs`
- Modify: `D:/Rust/Excel_Skill/src/tools/dispatcher.rs`
- Modify: `D:/Rust/Excel_Skill/tests/integration_open_workbook.rs`
- Modify: `D:/Rust/Excel_Skill/tests/integration_cli_json.rs`

**Step 1: Write the failing test**
- 增加 runtime 生成工作簿测试，覆盖“表从 `B3` 开始”的场景。
- 增加 CLI catalog / JSON 响应测试。

**Step 2: Run test to verify it fails**
Run: `cargo test inspect_sheet_range --test integration_open_workbook --test integration_cli_json -v`
Expected: FAIL

**Step 3: Write minimal implementation**
- 扫描非空单元格边界
- 生成 Excel 风格 `used_range`
- 返回 used range 内前 N 行样本

**Step 4: Run test to verify it passes**
Run: `cargo test inspect_sheet_range --test integration_open_workbook --test integration_cli_json -v`
Expected: PASS

### Task 2: `load_table_region`

**Files:**
- Create: `D:/Rust/Excel_Skill/src/frame/region_loader.rs`
- Modify: `D:/Rust/Excel_Skill/src/frame/mod.rs`
- Modify: `D:/Rust/Excel_Skill/src/excel/header_inference.rs`
- Modify: `D:/Rust/Excel_Skill/src/tools/contracts.rs`
- Modify: `D:/Rust/Excel_Skill/src/tools/dispatcher.rs`
- Modify: `D:/Rust/Excel_Skill/tests/integration_frame.rs`
- Modify: `D:/Rust/Excel_Skill/tests/integration_cli_json.rs`

**Step 1: Write the failing test**
- 增加 frame 测试：显式区域加载成功、`header_row_count=2` 的多层表头成功、非法区域报错。
- 增加 CLI 测试：catalog 暴露、返回 preview + `result_ref`。

**Step 2: Run test to verify it fails**
Run: `cargo test load_table_region --test integration_frame --test integration_cli_json -v`
Expected: FAIL

**Step 3: Write minimal implementation**
- 解析 Excel 区域
- 裁切所选矩形
- 依据 `header_row_count` 生成 canonical 列名
- 把数据行装成 `LoadedTable`
- 复用现有 `result_ref` 持久化响应链

**Step 4: Run test to verify it passes**
Run: `cargo test load_table_region --test integration_frame --test integration_cli_json -v`
Expected: PASS

### Task 3: 全量验证与任务日志

**Files:**
- Modify: `D:/Rust/Excel_Skill/task_plan.md`
- Modify: `D:/Rust/Excel_Skill/findings.md`
- Modify: `D:/Rust/Excel_Skill/progress.md`
- Modify: `D:/Rust/Excel_Skill/.trae/CHANGELOG_TASK.md`

**Step 1: Run verification**
Run: `cargo fmt`
Run: `cargo test -v`
Run: `cargo build --release -v`
Expected: PASS

**Step 2: Append logs**
- 更新 planning files
- 追加 task journal，只新增不改历史
