# Report Redesign Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 重做 2026 经营报告，修复 UTF-8 编码问题，并把结论、客户群划分、客户维系建议放到正文前部。

**Architecture:** 保留原始数据口径，重做 HTML 内容结构为“结论页 -> 客户分群 -> 客户维系建议 -> 月度目标 -> 图表附页”，再通过 Edge headless 输出 PDF。为避免再次出现乱码，报告文件直接以 UTF-8 写入本地，不再走容易污染中文的 stdin 文本拼接方式。

**Tech Stack:** PowerShell UTF-8 文件写入、Python 数据整理、HTML/CSS、Edge headless PDF

---

### Task 1: 校验客户分群口径与关键数字
**Files:**
- Read: `D:/Rust/Excel_Skill/.excel_skill_runtime/input/2025_tour_accident_income.xlsx`
- Read: `D:/Rust/Excel_Skill/.excel_skill_runtime/input/2026_tour_accident_income.xlsx`

### Task 2: 重写 HTML 成品稿结构
**Files:**
- Modify: `D:/Rust/Excel_Skill/.excel_skill_runtime/output/report_2026_business_opportunity.html`

### Task 3: 重新输出 PDF 并验证
**Files:**
- Modify: `D:/Rust/Excel_Skill/.excel_skill_runtime/output/report_2026_business_opportunity.pdf`

### Task 4: 更新过程记录
**Files:**
- Modify: `D:/Rust/Excel_Skill/task_plan.md`
- Modify: `D:/Rust/Excel_Skill/findings.md`
- Modify: `D:/Rust/Excel_Skill/progress.md`
- Modify: `D:/Rust/Excel_Skill/.trae/CHANGELOG_TASK.md`
