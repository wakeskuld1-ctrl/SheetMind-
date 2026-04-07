# Warning Scenario Turning Point Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 为 `04_经营预警` 与 `05_未来场景预测` 增加多拐点预测主线，让老板能同时看到预警演化、情景分化和主拐点。

**Architecture:** 先用测试锁定 `04/05` 页新增的多拐点文本合同，再扩展 `WarningSignal` 与 `ScenarioForecast` 的交付字段，最后最小改造写表逻辑与真实 Excel 回读验证。

**Tech Stack:** Python, pytest, openpyxl, xlsxwriter

---

### Task 1: 为 04/05 页写失败测试

**Files:**
- Modify: `D:\Rust\Excel_Skill\tests\test_boss_report_workbook.py`

**Step 1: Write the failing test**

- 新增一个测试，锁定 `04_经营预警` 包含：
  - `预警时间轴`
  - `预计灯色变化`
  - `预计主拐点`
- 锁定 `05_未来场景预测` 包含：
  - `利润拐点`
  - `毛利率拐点`
  - `灯色拐点`
  - `动作拐点`
  - `老板主拐点`

**Step 2: Run test to verify it fails**

Run: `python -m pytest tests\test_boss_report_workbook.py::test_build_boss_report_workbook_adds_multi_turning_points_to_warning_and_scenarios -q`

Expected: FAIL，因为当前 `04/05` 页还没有多拐点合同。

### Task 2: 扩展样例数据结构

**Files:**
- Modify: `D:\Rust\Excel_Skill\tests\test_boss_report_workbook.py`
- Modify: `D:\Rust\Excel_Skill\tools\boss_report_workbook_v3_impl.py`

**Step 1: Keep the new failing test red**

- 先不改写表逻辑，只让数据结构能承接新增合同。

**Step 2: Write minimal implementation**

- 扩展 `WarningSignal`，增加：
  - `expected_status_change`
  - `expected_main_turning_point`
- 扩展 `ScenarioForecast`，增加：
  - `profit_turning_point_period`
  - `margin_turning_point_period`
  - `status_turning_point_period`
  - `action_turning_point_period`
  - `main_turning_point_period`

**Step 3: Run test to verify it still fails correctly**

Run: `python -m pytest tests\test_boss_report_workbook.py::test_build_boss_report_workbook_adds_multi_turning_points_to_warning_and_scenarios -q`

Expected: 仍然 FAIL，但失败点转到页面文本缺失。

### Task 3: 改造 04_经营预警 页

**Files:**
- Modify: `D:\Rust\Excel_Skill\tools\boss_report_workbook_v3_impl.py`

**Step 1: Implement minimal page changes**

- 在 `04_经营预警` 增加：
  - `预警时间轴`
  - `预计灯色变化`
  - `预计主拐点`
  - 预警演化表

**Step 2: Run targeted test**

Run: `python -m pytest tests\test_boss_report_workbook.py::test_build_boss_report_workbook_adds_multi_turning_points_to_warning_and_scenarios -q`

Expected: 如果 `05` 页仍未完成，则继续 FAIL 在场景页。

### Task 4: 改造 05_未来场景预测 页

**Files:**
- Modify: `D:\Rust\Excel_Skill\tools\boss_report_workbook_v3_impl.py`

**Step 1: Implement minimal page changes**

- 为每个情景补：
  - `利润拐点`
  - `毛利率拐点`
  - `灯色拐点`
  - `动作拐点`
  - `老板主拐点`
- 在预测表中保留：
  - `预测数据`
  - `拐点标记`
- 必要时把不同拐点类型写入同一个表的标记列中。

**Step 2: Run targeted test**

Run: `python -m pytest tests\test_boss_report_workbook.py::test_build_boss_report_workbook_adds_multi_turning_points_to_warning_and_scenarios -q`

Expected: PASS

### Task 5: 运行全量验证

**Files:**
- Modify: `D:\Rust\Excel_Skill\.trae\CHANGELOG_TASK.md`

**Step 1: Run all workbook tests**

Run: `python -m pytest tests\test_boss_report_workbook.py -q`

Expected: PASS

**Step 2: Run syntax verification**

Run: `python -m py_compile tools\boss_report_workbook.py tools\boss_report_workbook_v3_impl.py tests\test_boss_report_workbook.py`

Expected: PASS

**Step 3: Generate real workbook**

Run: `python tools\boss_report_workbook.py --output "D:\Excel测试\第3天作业-业绩诊断_老板汇报版_策略矩阵版.xlsx"`

Expected: 文件生成成功

**Step 4: Read back the real workbook**

- 验证 `04` 页与 `05` 页出现新增多拐点字段
- 验证情景对象与拐点文本落盘

**Step 5: Append task journal**

- 追加 `.trae/CHANGELOG_TASK.md`
