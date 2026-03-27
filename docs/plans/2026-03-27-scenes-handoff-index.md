# SheetMind Scenes Handoff Index

## 目的

这份文档是 `SheetMind-` 主仓上的系统级接续索引。

原因：

- `SheetMind-Scenes` 已经承载运营供应链等私有场景实现，但它不是完全独立产品。
- `SheetMind-Scenes` 当前仍直接依赖 `SheetMind-` 的内核能力、数据口径和运行时约定。
- 其他 AI 如果只看到 `SheetMind-Scenes`，但看不到主仓里的 `join_preflight`、多表执行、`result_ref`、runtime 约束，就很难稳定接续。

结论：

- `SheetMind-` 负责保存系统级边界、能力口径和接续入口。
- `SheetMind-Scenes` 负责保存私有场景实现、样例、测试和场景级日志。

## 当前状态

截至 2026-03-27，主线可以分成两部分：

### 1. 主仓 `SheetMind-`

已完成到多表链路治理：

- `join_preflight`
- `join risk guard`
- 自动执行多表计划前的风险预估
- 阈值超限时硬停

关键参考：

- [src/ops/join.rs](/E:/Excel/SheetMind-/src/ops/join.rs)
- [src/tools/dispatcher.rs](/E:/Excel/SheetMind-/src/tools/dispatcher.rs)
- [docs/acceptance/2026-03-26-p0-join-risk-threshold-trial-guide.md](/E:/Excel/SheetMind-/docs/acceptance/2026-03-26-p0-join-risk-threshold-trial-guide.md)
- [docs/acceptance/2026-03-26-p0-join-risk-threshold-e2e.md](/E:/Excel/SheetMind-/docs/acceptance/2026-03-26-p0-join-risk-threshold-e2e.md)

### 2. 场景仓 `SheetMind-Scenes`

当前已完成到运营供应链补货场景的影子运行阶段：

- `inventory_replenishment_precheck` 场景已实现
- 样本 validation harness 已完成并通过
- 规则已切到方案 B
- 标准化 CSV -> 补货 scene 输入桥接已完成
- 上游 `join_preflight` / 执行结果 JSON -> `InventoryJoinQualityEvidence` 桥接已完成
- 影子运行 summary 已完成
- 白名单自动执行接口已预留，但目前只做资格评估，不执行

关键参考：

- [E:/Excel/SheetMind-Scenes/src/scenes/inventory_replenishment_precheck/scene_contracts.rs](E:/Excel/SheetMind-Scenes/src/scenes/inventory_replenishment_precheck/scene_contracts.rs)
- [E:/Excel/SheetMind-Scenes/src/validation/inventory_replenishment.rs](E:/Excel/SheetMind-Scenes/src/validation/inventory_replenishment.rs)
- [E:/Excel/SheetMind-Scenes/src/validation/inventory_shadow.rs](E:/Excel/SheetMind-Scenes/src/validation/inventory_shadow.rs)
- [E:/Excel/SheetMind-Scenes/src/binary/command_run_inventory_shadow.rs](E:/Excel/SheetMind-Scenes/src/binary/command_run_inventory_shadow.rs)
- [E:/Excel/SheetMind-Scenes/scripts/run_inventory_shadow.ps1](E:/Excel/SheetMind-Scenes/scripts/run_inventory_shadow.ps1)
- [E:/Excel/SheetMind-Scenes/docs/examples/inventory_replenishment_shadow/README.md](E:/Excel/SheetMind-Scenes/docs/examples/inventory_replenishment_shadow/README.md)

## 依赖关系

`SheetMind-Scenes` 当前明确依赖 `SheetMind-`：

### 代码依赖

`E:/Excel/SheetMind-Scenes/Cargo.toml` 中存在：

```toml
excel_skill = { path = "../SheetMind-" }
```

这意味着 `SheetMind-Scenes` 并不是完全独立编译单元，而是直接复用主仓能力。

### 能力依赖

补货场景目前依赖主仓这些能力口径：

- `join_preflight`
- `execute_multi_table_plan`
- `join risk guard`
- `result_ref`
- runtime 产物目录与桥接方式

### 接续依赖

如果后续 AI 要继续推进补货自动化，应先理解主仓里的：

- join 风险统计字段语义
- 多表执行停止条件
- `result_ref` 产物与路径约定

然后再进入 `SheetMind-Scenes` 继续场景侧实现。

## 推荐接续顺序

后续 AI 建议按这个顺序接：

1. 先读主仓能力边界
   - `src/ops/join.rs`
   - `src/tools/dispatcher.rs`
   - `docs/acceptance/2026-03-26-p0-join-risk-threshold-trial-guide.md`

2. 再读场景仓当前补货线
   - `src/scenes/inventory_replenishment_precheck/*`
   - `src/validation/inventory_replenishment.rs`
   - `src/validation/inventory_shadow.rs`

3. 先跑现有验证
   - `cargo test --test inventory_replenishment_validation_harness -- --nocapture`
   - `cargo test --test inventory_replenishment_shadow -- --nocapture`
   - `powershell -ExecutionPolicy Bypass -File E:\Excel\SheetMind-Scenes\scripts\run_inventory_shadow.ps1`

4. 再接真实数据
   - 用真实标准化 5 张表替换 shadow sample
   - 用真实 `join_preflight` / `execute_multi_table_plan` 响应 JSON 替换 sample join bridge

## 现在不要做的事

当前不建议直接做：

- 直接自动执行补货决策
- 跳过 shadow run 直接白名单放行
- 在没有对齐主仓 join 风险语义之前重写一套新的 join 风险模型

原因：

- 现在阶段已经适合进入真实数据影子运行
- 但还不适合直接进入生产自动决策

## 下一步建议

最自然的下一步是：

1. 用真实业务数据跑第一版 shadow
2. 对比人工当前做法与 shadow summary
3. 再决定是否实现真正的白名单自动执行器

一句话总结：

当前系统已经从“补货规则验证”推进到“依赖主仓 join 治理能力的补货影子运行”阶段，接续时必须同时理解 `SheetMind-` 和 `SheetMind-Scenes` 两边，而不能只看其中一个仓库。
