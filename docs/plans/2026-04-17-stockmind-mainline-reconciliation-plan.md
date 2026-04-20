# StockMind Mainline Reconciliation Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 在独立新工程中以 `StockMind/main` 为主线，建立本地旧工程能力的首轮迁移与盘点流程。

**Architecture:** 先在 `D:\Rust\StockMind` 创建全新本地仓库副本，再在新仓库上建立整合分支。随后将 `D:\Rust\Excel_Skill` 当作参考源，聚焦关键 Rust 能力模块做结构与功能差异盘点，形成可执行迁移清单。

**Tech Stack:** Git, GitHub, PowerShell, Rust, Cargo, Markdown

---

### Task 1: 建立独立工程基线

**Files:**
- Create: `D:\Rust\StockMind\...`
- Test: Git 状态与分支检查命令输出

**Step 1: 检查目标目录是否为空缺**

Run: `Test-Path 'D:\Rust\StockMind'`
Expected: `False`

**Step 2: 克隆远端仓库**

Run: `git clone https://github.com/wakeskuld1-ctrl/StockMind.git D:\Rust\StockMind`
Expected: clone 成功且本地默认分支可用

**Step 3: 确认默认分支与远端状态**

Run: `git branch --show-current`
Expected: `main`

Run: `git remote -v`
Expected: `origin` 指向 `https://github.com/wakeskuld1-ctrl/StockMind.git`

**Step 4: 创建整合分支**

Run: `git checkout -b codex/reconcile-local-features-20260417`
Expected: 新分支创建成功

**Step 5: 记录当前状态**

- 将新工程路径、当前分支、初始提交状态写入会话记录文件

### Task 2: 采集新旧工程关键结构差异

**Files:**
- Modify: `findings.md`
- Modify: `progress.md`

**Step 1: 检查新工程顶层结构**

Run: `Get-ChildItem -Force 'D:\Rust\StockMind'`
Expected: 得到新仓库目录结构

**Step 2: 检查旧工程关键源码目录**

Run: `Get-ChildItem -Force 'D:\Rust\Excel_Skill\src\ops'`
Expected: 得到旧工程 ops 能力分布

**Step 3: 对照关键模块是否存在**

重点比对：
- `security_position_plan`
- `security_chair_resolution`
- `security_decision_submit_approval`
- `security_direction_first_training_run`
- `foundation`

**Step 4: 写入首轮发现**

- 记录“新仓已有 / 旧仓独有 / 命名变化 / 结构变化”

### Task 3: 输出首轮迁移清单

**Files:**
- Modify: `findings.md`
- Modify: `progress.md`
- Modify: `task_plan.md`

**Step 1: 将能力分为三类**

- 可直接迁移
- 需适配后迁移
- 暂不迁移

**Step 2: 说明判断依据**

- 是否已在新仓存在
- 是否强依赖旧目录结构
- 是否属于临时实验能力
- 是否已有可复用测试

**Step 3: 给出下一批实际迁移建议**

优先级默认：
1. 决策链核心能力
2. 训练编排入口
3. foundation 能力

### Task 4: 向用户回报并等待下一步迁移批准

**Files:**
- 无代码改动

**Step 1: 汇报新工程已建立**

内容包括：
- 新工程路径
- 当前整合分支
- 是否已成功拉下远端主线

**Step 2: 汇报差异盘点结果**

内容包括：
- 哪些功能已经在线上仓中存在
- 哪些是旧工程独有
- 哪些建议优先迁移

**Step 3: 明确下一步选项**

- 继续做首批能力迁移
- 先做目录/模块映射
- 先做测试与夹具梳理
