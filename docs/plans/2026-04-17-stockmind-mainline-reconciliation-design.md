# StockMind Mainline Reconciliation Design

> **For Claude:** 先以线上 `StockMind/main` 为主流程创建独立工程，再将本地 `Excel_Skill` 中仍有价值的能力按清单逐项迁移。

**Goal:** 以远端 `StockMind/main` 为唯一主基线，新建独立工程并建立“旧工程能力盘点 -> 新工程择优迁移”的工作流。

**Architecture:** 本次不在旧仓库内继续叠加开发，而是新建 `D:\Rust\StockMind` 独立工程。远端 `StockMind/main` 作为主线，旧工程 `D:\Rust\Excel_Skill` 仅作为功能来源仓库，通过差异盘点和分批迁移把有价值的 Rust 能力、测试和文档整理进新工程。

**Tech Stack:** Git, GitHub, PowerShell, Rust, Cargo, Markdown

---

## 背景

- 用户已经将产品拆分为新的独立仓库：`https://github.com/wakeskuld1-ctrl/StockMind`
- 远端当前公开主分支为 `main`
- 本地旧工程 `D:\Rust\Excel_Skill` 仍有大量历史功能与临时测试产物，不能继续作为主流程仓库

## 核心原则

1. 线上仓库优先
   - 所有后续开发以 `StockMind/main` 为主
   - 本地旧工程不再作为主线，只保留参考价值

2. 独立工程隔离
   - 新工程单独放在 `D:\Rust\StockMind`
   - 避免旧工程脏工作区和临时夹具影响新仓库

3. 迁移优先于重构
   - 先做能力盘点
   - 再决定哪些功能应直接迁移、哪些要适配迁移、哪些不再保留
   - 非必要不做大重构

4. 小批量推进
   - 先建立新工程和整合分支
   - 再按能力模块逐步迁移
   - 每一批迁移都能单独验证

## 本次任务目标

### 第一阶段

- 在本地创建独立工程目录 `D:\Rust\StockMind`
- 克隆远端 `StockMind/main`
- 创建整合分支，作为后续迁移工作分支

### 第二阶段

- 盘点旧工程中的关键能力，识别是否值得迁移
- 重点关注：
  - `security_position_plan`
  - `security_chair_resolution`
  - `security_decision_submit_approval`
  - `security_direction_first_training_run`
  - `src/ops/foundation/` 下已成熟能力

### 第三阶段

- 输出首轮迁移清单
- 将能力分为：
  - 可直接迁移
  - 需适配后迁移
  - 暂不迁移

## 不采用的方案

### 不采用“在旧工程里直接加 remote 然后合”

原因：
- 旧工程工作区很脏
- 有大量 `tests/runtime_fixtures/...` 临时目录
- 会进一步混淆旧主线和新主线

### 不采用“一次性自动 merge 两仓库”

原因：
- 新旧仓库已经分拆
- 自动 merge 很容易制造大量噪音冲突
- 我们当前真正需要的是“能力甄别”，不是“历史强拼接”

## 输出物

- 新独立工程目录：`D:\Rust\StockMind`
- 新工程整合分支
- 首轮差异盘点结论
- 后续迁移清单

## 风险

- 旧工程中有些能力已经依赖旧目录结构，迁移时可能需要重新落位
- 旧工程里存在测试运行生成的临时夹具，盘点时要避免把这些当成正式资产
- 新仓库的结构可能已经领先旧工程很多，部分旧能力可能已经被重写或替代

## 成功标准

- 本地存在一个干净可用的 `StockMind` 独立工程
- 新工程基于远端 `main` 创建了可继续工作的整合分支
- 已完成首轮关键能力盘点，并能清楚回答“本地哪些功能值得合并”
