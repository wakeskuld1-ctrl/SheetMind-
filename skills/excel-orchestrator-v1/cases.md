# Excel Orchestrator V1 场景映射

## 场景 0：首次打开失败更像路径格式问题

- 用户说法：先看看这个 Excel，但刚才打开失败了
- 当前状态：旧会话状态指向测试文件，当前第一次打开返回路径语法错误
- 目标层：`table-processing-v1`
- 路由原因：这是入口恢复问题，先纠正 Windows 路径格式，再继续表处理

## 场景 1：用户只想先看看 Excel

- 用户说法：先看看这个 Excel
- 当前状态：没有 `table_ref`
- 目标层：`table-processing-v1`
- 路由原因：先判断表头与结构，不能直接进入分析或决策

## 场景 2：用户想先看统计摘要

- 用户说法：先看一下统计情况
- 当前状态：已有 `table_ref`
- 目标层：`analysis-modeling-v1`
- 路由原因：已具备确认态，可以直接进入统计摘要或观察诊断

## 场景 3：用户要直接做模型

- 用户说法：帮我做逻辑回归
- 当前状态：没有 `table_ref`
- 目标层：先回 `table-processing-v1`
- 路由原因：模型入口不能跳过表头确认态

## 场景 4：用户想知道下一步先做什么

- 用户说法：你直接告诉我下一步该做什么
- 当前状态：已有 `table_ref`
- 目标层：`decision-assistant-v1`
- 路由原因：已具备确认态，适合直接进入优先级建议层

## 场景 5：用户已经在分析建模层里继续追问

- 用户说法：那你继续帮我看这几个字段能不能建模
- 当前状态：已有 `table_ref`，当前阶段是 `analysis_modeling`
- 目标层：继续留在 `analysis-modeling-v1`
- 路由原因：意图仍然是分析建模，不必切层

## 场景 6：用户在决策建议后要求直接执行模型

- 用户说法：那就直接帮我做聚类
- 当前状态：已有 `table_ref`，当前阶段是 `decision_assistant`
- 目标层：切到 `analysis-modeling-v1`
- 路由原因：从“建议下一步”切换成“真正执行模型”

## 场景 7：文件存在，但中文路径直接读取失败

- 用户说法：文件明明在，你先继续处理
- 当前状态：系统能定位文件，但 Tool 层直接读中文路径失败
- 目标层：`table-processing-v1`
- 路由原因：先走 ASCII 临时副本降级，再继续工作簿读取和表处理

## 2026-03-23 兼容补充
- 涉及中文路径恢复的案例里，要先明确这是“路径/兼容问题”，不要提前说成文件损坏。
- 一旦文件成功打开，后续案例默认按“第几个 Sheet”继续，不再要求用户重复输入中文 Sheet 名。

## Scenario 8: auto plan execution stopped by join risk threshold

- User says: continue execution, but keep it safe.
- Current status: runtime returned `execution_status=stopped_join_risk_threshold` with breach metrics.
- Target layer: `table-processing-v1` by default.
- Routing reason: this is a controlled risk gate; first action should be key cleanup / join condition correction, not blind retry.
- Alternative path: rerun with custom thresholds only after explicit user confirmation.

## Scenario 9: multi-table execution stopped by missing result bindings

- User says: continue plan execution.
- Current status: runtime returned `execution_status=stopped_missing_result_bindings` and indicates missing aliases.
- Target layer: `table-processing-v1`.
- Routing reason: this is a dependency-replay issue; chain context must be restored before continuing blocked step.
- Recovery expectation: complete missing bindings or replay from required prior step, then rerun execution.

## Scenario 10: multi-table execution failed with unknown failure diagnostics

- User says: continue and fix whatever blocked this run.
- Current status: runtime returned `execution_status=failed` with `failure_diagnostics.failure_class=unknown_runtime_failure`.
- Target layer: `table-processing-v1`.
- Routing reason: fallback route is `table_processing_diagnostics`; blocked step and tool are known but failure is unclassified.
- Recovery expectation: diagnose/fix blocked step inputs in table-processing first, then rerun from blocked step.
