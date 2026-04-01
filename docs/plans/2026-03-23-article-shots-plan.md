# Article Shots Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 生成 4 张可用于文章发布的非白屏截图，覆盖 AI 对话与 Excel 经营分析结果。

**Architecture:** 先生成本地 HTML 对话页并用浏览器截图，再用 Excel 前台窗口定位关键页签与区域，通过屏幕抓取导出 PNG。每张截图都在导出后立即校验尺寸与可见内容，避免再次出现空白图。

**Tech Stack:** PowerShell、HTML/CSS、Excel COM、浏览器截图链路

---

### Task 1: 准备计划与产物目录

**Files:**
- Create: `D:\Rust\Excel_Skill\docs\plans\2026-03-23-article-shots-design.md`
- Create: `D:\Rust\Excel_Skill\docs\plans\2026-03-23-article-shots-plan.md`
- Modify/Create: `D:\Rust\Excel_Skill\.excel_skill_runtime\output\article_shots\`

**Step 1: 确认目录存在**
Run: `Test-Path 'D:\Rust\Excel_Skill\.excel_skill_runtime\output\article_shots'`
Expected: `True`

**Step 2: 如有旧白图，保留但重新输出新图名**
Run: `Get-ChildItem 'D:\Rust\Excel_Skill\.excel_skill_runtime\output\article_shots'`
Expected: 可见旧 PNG，后续不覆盖验证前文件。

### Task 2: 生成对话页

**Files:**
- Create: `D:\Rust\Excel_Skill\.excel_skill_runtime\output\article_shots\story_demo.html`

**Step 1: 写入自问自答内容**
包含“老板提问 / AI 拆解 / 关键结论 / 下一步动作”四段。

**Step 2: 浏览器打开并截图**
Run: 使用浏览器截图链路输出 `D:\Rust\Excel_Skill\.excel_skill_runtime\output\article_shots\07_对话开场.png`
Expected: 页面可见中文对话气泡与结论卡片。

### Task 3: 重做 Excel 真实截图

**Files:**
- Source: `D:\Rust\Excel_Skill\.excel_skill_runtime\output\个险长期险_渠道按月收入透视表_含同比总计.xlsx`
- Source: `D:\Rust\Excel_Skill\.excel_skill_runtime\output\2026渠道目标拆解与客户维系建议.xlsx`
- Create: `D:\Rust\Excel_Skill\.excel_skill_runtime\output\article_shots\08_渠道同比分析_真实截图.png`
- Create: `D:\Rust\Excel_Skill\.excel_skill_runtime\output\article_shots\09_春节错月目标_真实截图.png`
- Create: `D:\Rust\Excel_Skill\.excel_skill_runtime\output\article_shots\10_重点客户建议_真实截图.png`

**Step 1: 前台打开目标工作簿并激活 sheet**
`渠道同比分析`、`春节错月版目标`、`经营建议`

**Step 2: 设置合适缩放与显示区域**
确保标题、表头和至少一屏核心数据可见。

**Step 3: 抓取窗口区域**
Expected: 不是白图，能看到 Excel 网格、中文标题、数据。

### Task 4: 校验与回报

**Files:**
- Modify: `D:\Rust\Excel_Skill\.trae\CHANGELOG_TASK.md`

**Step 1: 校验截图尺寸与抽样像素**
Expected: 不为纯白。

**Step 2: 人工查看至少 2 张图**
Expected: 文字清晰、结构可辨识。

**Step 3: 记录任务日志并向用户反馈**
Expected: 说明产物路径、截图内容、残留风险。
