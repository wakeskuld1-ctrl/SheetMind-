# AI START HERE

## 文件目的

这是一份给新 AI / 新工程师的最短入口。

目标只有两个：

1. 用最短时间进入当前真实主线
2. 避免一接手就被历史材料和旧标题带偏

## 当前项目是什么

当前仓库应理解为：

- `SheetMind / Excel_Skill`
- Rust-first
- EXE / CLI-first
- foundation-first

不要把它再理解成旧的 `TradingAgents` GitHub 首页项目。

## 必读顺序

请按下面顺序阅读：

1. [README.md](./README.md)
2. [docs/ai-memory/project-baseline.md](./docs/ai-memory/project-baseline.md)
3. [docs/ai-handoff/AI_HANDOFF_MANUAL.md](./docs/ai-handoff/AI_HANDOFF_MANUAL.md)
4. [docs/execution-notes-2026-04-07-foundation-navigation-kernel.md](./docs/execution-notes-2026-04-07-foundation-navigation-kernel.md)
5. [docs/architecture/repo-and-branch-governance.md](./docs/architecture/repo-and-branch-governance.md)

## 当前主线

当前主线不是继续无序叠业务，而是先把通用底座做稳：

1. Rust 基础能力
2. Excel / 表处理 / 分析 / 报表
3. foundation 导航内核
4. 交付、交接、验证体系

这里再补一条硬边界：

- `security_*`、`stock_*`、审批链、committee、scorecard、training 这类内容，即使存在于仓库中，也只应理解为业务适配层或并行领域轨道；
- 它们不是当前项目主线，不代表 foundation 范围已经改变；
- 任何 AI 接手时，都必须先按通用能力主线理解仓库，再决定是否进入某个业务域分支。

## 已确认的架构原则

### 1. 默认按现有架构继续推进

这轮已经定好的结构，后续默认沿着它扩展，不要每次接手都重新重构。

### 2. foundation 顺序固定

当前 foundation 主线顺序固定为：

`ontology-lite -> roaming -> retrieval -> evidence assembly`

### 3. retrieval 不是系统入口

retrieval 只是候选域内的执行阶段，不是整个系统的起点。

### 4. Rust 是产品主线

仓库里可能还保留历史 Python 相关内容，但当前产品开发主线是 Rust / EXE，不要把 Python 重新混回主交付链路。

### 5. 目录边界先于业务进展

当前应这样理解目录边界：

- `src/ops/foundation/`：只承载 ontology、roaming、retrieval、evidence 这类通用导航内核
- 通用 runtime / tooling / table / analysis / report：属于可复用标准能力
- `security_*`、`stock_*` 等：属于业务适配层，不属于 foundation 主线

如果看到大量证券相关文件，不要自动推断“当前项目主线是证券业务化”；默认判断应当仍然是“foundation-first，业务域为上层适配”。

## 开工前必须遵守

1. 用中文沟通
2. 修改前先给方案，等批准再动
3. 遇到 Bug 先补复现测试再修
4. 不要为单个场景破坏长期边界
5. 每次任务完成后补 `.trae/CHANGELOG_TASK.md`

## 当前最重要的提醒

如果你只记一句话，请记这个：

`先守住边界，再补能力；先沿现有架构推进，非必要不重构。`
