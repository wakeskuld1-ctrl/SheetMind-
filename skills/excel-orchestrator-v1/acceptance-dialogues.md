# Excel Orchestrator V1 模拟对话验收稿

这个文档用于人工验收 `excel-orchestrator-v1`。

验收目标不是看它会不会“自由发挥”，而是看它是否稳定符合这 5 条：

1. 用户只面对一个总入口
2. 总入口会先给出当前状态摘要
3. 有 `table_ref` 时优先复用确认态
4. 没有确认态时不会跳过表处理层
5. 总入口只做路由和解释，不自己做计算

---

## 场景 0：用户要看真实文件，但第一次打开失败更像路径问题

### 用户

先看看这份台账，刚才打开失败了。

### 期望 Skill 回复

当前理解：工具层里已经存在旧会话状态，但它指向测试文件，不适合直接复用到你这份台账。  
当前状态：第一次打开工作簿失败，错误更像是 Windows 路径语法不正确，问题更像是路径格式而不是文件内容。  
下一步动作：我会先把你切回表处理层，改用 Windows 原生反斜杠路径重新打开，并把当前目标切回这份台账。  
对应路由：`table-processing-v1`

### 通过判定

- 明确这是入口恢复问题
- 明确问题更像路径格式，不是文件内容
- 明确切回表处理层处理，而不是直接开始分析

### 失败判定

- 直接说文件坏了
- 继续沿用旧测试文件状态
- 不经表处理层就直接开始统计或建模

---

## 场景 1：用户只说“先看看这个 Excel”

### 用户

先看看这个 Excel。

### 期望 Skill 回复

当前理解：你现在还处在最开始的整理阶段，想先知道这份 Excel 的结构是否清楚。  
当前状态：我还没有拿到可复用的确认态，所以这一步不能直接进入分析或决策层。  
下一步动作：我会先把你带到表处理层，先判断表头和表结构。  
对应路由：`table-processing-v1`

### 通过判定

- 明确说当前没有确认态
- 明确路由到表处理层

### 失败判定

- 直接说去做统计摘要
- 直接说去做回归 / 聚类 / 决策建议

---

## 场景 2：已有 `table_ref`，用户说“先看统计”

### 用户

这张表前面已经确认过了，先看统计。

### 期望 Skill 回复

当前理解：你现在不是要重新整理表，而是基于已经确认好的表继续看统计摘要。  
当前状态：已有可复用的 `table_ref`，所以不需要重复确认表头。  
下一步动作：我会把你切到分析建模层，优先看统计摘要或观察诊断。  
对应路由：`analysis-modeling-v1`

### 通过判定

- 明确说明复用 `table_ref`
- 没有要求重复提供 `path + sheet`

### 失败判定

- 再次要求用户确认表头
- 先退回表处理层

---

## 场景 3：用户要直接做逻辑回归，但没有确认态

### 用户

直接帮我做逻辑回归。

### 期望 Skill 回复

当前理解：你已经明确想做模型，但当前还缺少一个前提，就是这张表还没有进入已确认的可复用状态。  
当前状态：没有 `table_ref`，所以现在还不能直接进入建模。  
下一步动作：我会先把你带回表处理层，先确认表头与结构，拿到确认态后再继续进入分析建模层。  
对应路由：先 `table-processing-v1`

### 通过判定

- 明确说明为什么不能直接进模型
- 明确说明先建立确认态

### 失败判定

- 直接把用户送进 `analysis-modeling-v1`
- 跳过确认态，直接给模型参数追问

---

## 场景 4：已有 `table_ref`，用户要“下一步该做什么”

### 用户

你直接告诉我下一步该做什么。

### 期望 Skill 回复

当前理解：你现在不是要立刻执行模型，而是想先知道当前最该先处理哪一步。  
当前状态：这张表已经有确认态，可以直接进入建议层。  
下一步动作：我会把你切到决策助手层，让它先告诉你阻塞问题、优先动作和下一步建议。  
对应路由：`decision-assistant-v1`

### 通过判定

- 明确路由到决策助手层
- 不直接进入分析建模层

### 失败判定

- 直接开始建模
- 重新要求用户确认表头

---

## 场景 5：决策建议后，用户要求直接执行聚类

### 用户

好，那你直接帮我做聚类。

### 期望 Skill 回复

当前理解：你的意图已经从“先给建议”切换成“直接执行模型”。  
当前状态：已有确认态，所以不需要回表处理层。  
下一步动作：我会把你切回分析建模层，继续补齐聚类需要的字段和分组数。  
对应路由：`analysis-modeling-v1`

### 通过判定

- 明确说这是切层，不是继续停留在决策助手层
- 明确保留 `table_ref`

### 失败判定

- 继续停留在决策助手层
- 丢失确认态上下文

---

## 场景 6：文件存在，但中文路径兼容失败

### 用户

文件是有的，你继续处理就行。

### 期望 Skill 回复

当前理解：文件本身存在，但当前工具链把中文路径传给底层 Excel 读取库时出了兼容问题，不是你这份文件找不到。  
当前状态：直接读中文路径会失败，不过 PowerShell 能正常定位到文件，所以这一步更像路径兼容问题。  
下一步动作：我会先把你切回表处理层，用 ASCII 临时副本继续读取 Sheet 列表，再选出最像目标台账的 Sheet。  
对应路由：`table-processing-v1`

### 通过判定

- 明确文件存在
- 明确失败点更像中文路径兼容问题
- 明确先回表处理层做 ASCII 副本降级

### 失败判定

- 直接说文件不存在
- 跳过恢复步骤直接猜 sheet
- 继续沿用失败的中文路径直接做后续分析

## 2026-03-23 兼容补充
- 验收对话里如果提到 ASCII 临时副本，必须先出现“如果你同意”或“请先确认”这类表述。
- 一旦文件已经打开，后续对话优先改成“第几个 Sheet”的说法。
- 不要把内部引用名或内部链路细节直接讲给非技术用户。

---

## Scenario 7: controlled stop on join risk threshold

### User

Continue running the multi-table plan.

### Expected Skill reply

Current understanding: the plan did run, but runtime paused at join preflight because risk exceeded the configured safety threshold.
Current status: this is a controlled safety stop (`stopped_join_risk_threshold`), not a generic execution failure.
Next action: I will route to table-processing flow first to reduce unmatched/duplicate key risk, or if you confirm, we can rerun with higher thresholds.
Route: `table-processing-v1` by default.

### Pass criteria

- Clearly distinguishes safety stop vs system failure.
- Surfaces breach metrics in plain language.
- Does not auto-retry with higher thresholds.
- Asks user to confirm before any threshold increase rerun.

### Fail criteria

- Treats risk-threshold stop as generic crash.
- Retries automatically with looser guard.
- Skips user confirmation for threshold increase.

---

## Scenario 8: controlled stop on missing result bindings

### User

Continue the plan from where it stopped.

### Expected Skill reply

Current understanding: execution paused because a required prior-step result handle was not available for the next step.
Current status: this is `stopped_missing_result_bindings`, a controlled dependency stop rather than a system failure.
Next action: I will route to table-processing flow to restore/rebuild the missing bindings, then continue execution from the blocked step.
Route: `table-processing-v1`.

### Pass criteria

- Clearly distinguishes dependency stop vs runtime crash.
- Mentions missing alias/binding requirement in plain language.
- Routes to binding completion/replay path before rerun.

### Fail criteria

- Treats missing bindings as unknown crash.
- Attempts blind rerun without rebuilding bindings.
- Skips explanation of what is missing.

---

## Scenario 9: failed execution with unknown failure diagnostics

### User

Continue execution and fix the blocked step.

### Expected Skill reply

Current understanding: execution failed in an unclassified runtime/tool branch, not a controlled preflight stop.
Current status: `failure_diagnostics` points to `failed_step_id`/`failed_action`, fallback route is `table_processing_diagnostics`, and `recovery_templates` provides deterministic resume calls (`resume_execution` / `resume_full_chain`).
Next action: I will route to table-processing diagnostics first, fix blocked inputs, and then choose blocked-step replay or full-chain continuation.
Route: `table-processing-v1`.

### Pass criteria

- Distinguishes unknown failure vs controlled stop statuses.
- Mentions `failure_diagnostics` fields in plain language.
- Keeps diagnostics-first route before rerun.

### Fail criteria

- Routes directly to analysis/modeling.
- Treats unknown failure as completed result.
- Tries blind rerun without diagnostics.
