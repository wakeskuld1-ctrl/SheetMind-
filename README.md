# SheetMind / Excel_Skill

`SheetMind / Excel_Skill` 是一个以 Rust 为主线的 Excel 分析与基础能力仓库。

当前仓库的核心方向不是继续扩散业务场景，而是先把通用底座做稳，尤其是：

- Rust / EXE / CLI-first 交付
- foundation 基础能力沉淀
- Excel 表处理、分析、报表、运行时与交接体系收口
- 业务无关的导航内核：`ontology-lite -> roaming -> retrieval -> evidence`

## 当前状态

当前活跃主线是 Rust 底座，不是旧的多 Agent 金融演示项目。

仓库里仍然保留了一部分历史材料和过往阶段文件，但它们不代表当前 GitHub 首页应该传达的产品身份。当前对外应以 `SheetMind / Excel_Skill` 为准。

## 先看哪里

如果你是第一次进入仓库，建议按这个顺序阅读：

1. [AI_START_HERE.md](./AI_START_HERE.md)
2. [docs/ai-memory/project-baseline.md](./docs/ai-memory/project-baseline.md)
3. [docs/ai-handoff/AI_HANDOFF_MANUAL.md](./docs/ai-handoff/AI_HANDOFF_MANUAL.md)
4. [docs/execution-notes-2026-04-07-foundation-navigation-kernel.md](./docs/execution-notes-2026-04-07-foundation-navigation-kernel.md)
5. [docs/architecture/repo-and-branch-governance.md](./docs/architecture/repo-and-branch-governance.md)

## 当前主线

### 1. Rust 基础能力

当前主仓主要承载：

- Excel / 表格数据处理能力
- 统计分析、建模、报表相关能力
- 本地运行时、会话状态、结果交付
- 可选 GUI 外壳

默认产品主线是 Rust / EXE，不把 Python 当成当前产品开发主链。

### 2. Foundation 导航内核

当前已经落地到 Task 7 的 foundation 主线包括：

- `ontology_schema`
- `ontology_store`
- `knowledge_record`
- `knowledge_graph_store`
- `capability_router`
- `roaming_engine`
- `retrieval_engine`

当前确认顺序固定为：

`ontology-lite -> roaming -> retrieval -> evidence assembly`

不要把 retrieval 当成系统入口，也不要每次接手就重新做架构重构。

## 仓库结构

主要目录职责如下：

- `src/`: Rust 主代码
- `tests/`: Rust 测试与运行时夹具
- `docs/`: 计划、验收、交接、执行记录
- `skills/`: 项目内 Skill 资产
- `cli/`: 历史 CLI 相关内容
- `tradingagents/`: 历史来源内容，当前不是产品主线

更详细的说明见：

- [docs/architecture/repo-and-branch-governance.md](./docs/architecture/repo-and-branch-governance.md)

## 开发约束

- 以 Rust 主线为准
- foundation 与业务层严格分层
- 默认按 TDD 推进
- 不在没有明确理由的情况下重复重构
- 文档与 AI handoff 必须持续可用

## 常用验证

### Foundation 主线回归

```bash
cargo test --test ontology_schema_unit --test ontology_store_unit --test knowledge_record_unit --test knowledge_graph_store_unit --test capability_router_unit --test roaming_engine_unit --test retrieval_engine_unit -- --nocapture
```

### 查看当前分支与工作区

```bash
git status --short --branch
git branch -vv
```

## 当前不做什么

这一阶段不优先做：

- GUI-first 智能问答
- 把 foundation 绑死到单一业务域
- 为了单个场景再次大规模改骨架
- 把云端模型变成硬依赖

## 备注

如果你是新的 AI 或工程师，请不要直接相信仓库里最早期的历史文本标题。先按本 README 的入口继续读，再决定是否动代码。
