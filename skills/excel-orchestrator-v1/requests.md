# Excel Orchestrator V1 跨层交接模板

这个文件不是底层 Tool 模板集合，而是总入口层的跨层交接模板。

使用原则：

- 总入口 Skill 不直接做计算
- 总入口的主要动作是“判断当前层 -> 交给子 Skill”
- 跨层时优先保留 `table_ref`
- 每轮开始优先先读 `get_session_state`
- 需要显式修正阶段、目标、选列时，再写 `update_session_state`

## 入口恢复场景

适用场景：

- 用户第一次打开工作簿就失败
- 当前更像路径格式问题，而不是文件内容问题
- 当前更像中文路径兼容问题，而不是文件不存在

总入口应表达成：

- 当前理解：这是入口恢复问题，旧会话状态或第一次尝试的路径不适合直接继续复用
- 当前状态：当前失败更像路径格式或路径兼容问题，不急着判断文件内容异常
- 下一步动作：切回 `table-processing-v1`，先恢复文件入口，再继续表处理

禁止项：

- 不要在总入口层直接判断 sheet
- 不要在总入口层直接做统计摘要
- 不要在总入口层把路径问题说成文件损坏

## 进入表处理层

适用场景：

- 没有 `table_ref`
- 用户处于“先看看 / 先整理 / 先合表”阶段

推荐先读：

```json
{
  "tool": "get_session_state",
  "args": {
    "session_id": "default"
  }
}
```

总入口应表达成：

- 当前理解：你现在还在整理表的阶段
- 当前状态：还没有可复用的确认态
- 下一步动作：先进入 `table-processing-v1`

如果这一步是因为首次打开失败而退回表处理层，还应补一句：

- 当前状态：这次失败更像文件入口问题，我会先恢复路径，再继续读工作簿

## 进入分析建模层

适用场景：

- 已有 `table_ref`
- 用户想看统计摘要、观察诊断、回归、聚类

总入口应表达成：

- 当前理解：你现在要继续做分析或建模
- 当前状态：已存在可复用确认态 `table_ref`
- 下一步动作：进入 `analysis-modeling-v1`

推荐交接状态：

```json
{
  "current_stage": "analysis_modeling",
  "schema_status": "confirmed",
  "active_table_ref": "table_1234567890",
  "last_user_goal": "先看统计摘要"
}
```

如果总入口需要在真正进入分析建模前先显式改写状态，可写：

```json
{
  "tool": "update_session_state",
  "args": {
    "session_id": "default",
    "current_stage": "analysis_modeling",
    "schema_status": "confirmed",
    "active_table_ref": "table_1234567890",
    "last_user_goal": "先看统计摘要"
  }
}
```

## 进入决策助手层

适用场景：

- 已有 `table_ref`
- 用户想知道“下一步该做什么”

总入口应表达成：

- 当前理解：你现在更需要优先级建议，而不是直接算模型
- 当前状态：已存在可复用确认态 `table_ref`
- 下一步动作：进入 `decision-assistant-v1`

推荐交接状态：

```json
{
  "current_stage": "decision_assistant",
  "schema_status": "confirmed",
  "active_table_ref": "table_1234567890",
  "last_user_goal": "下一步该做什么"
}
```

如果已经进入 `decision_assistant` Tool，则允许直接依赖 Tool 自动把阶段推进为 `decision_assistant`，不要求总入口重复写一次状态。

## 从决策助手切回分析建模

适用场景：

- 用户前面在问“下一步该做什么”
- 现在改成“直接帮我做回归 / 聚类”

推荐交接状态：

```json
{
  "current_stage": "analysis_modeling",
  "schema_status": "confirmed",
  "active_table_ref": "table_1234567890",
  "last_user_goal": "直接做聚类"
}
```

## 禁止项

- 不要在总入口层自己拼底层 Tool JSON
- 不要在总入口层自己做统计、建模、业务结论
- 不要在已有 `table_ref` 时强制用户重新确认表头

## 2026-03-23 兼容补充
- 如果总入口已经拿到 `file_ref` 与 `sheets`，后续交接优先让下层按 `file_ref + sheet_index` 继续。
- 面向用户时统一说“第几个 Sheet”，不要再要求用户重复输入中文 Sheet 名。
- 如果需要复制到 ASCII 临时路径，必须先征求用户确认。

## 2026-03-26 cross-layer template: stopped_join_risk_threshold

### Trigger payload (from runtime)

```json
{
  "execution_status": "stopped_join_risk_threshold",
  "stopped_at_step_id": "step_2",
  "executed_steps": [
    {
      "step_id": "step_2",
      "action": "join_preflight",
      "join_risk_guard_breaches": [
        "left_unmatched_row_count=28 exceeds max_left_unmatched_rows=10"
      ]
    }
  ]
}
```

### Orchestrator response template

- Current understanding: execution paused by safety guard, not by parser/runtime crash.
- Current status: join preflight indicates risk above threshold.
- Next action: ask user to choose safe cleanup first or explicit threshold increase and rerun.

### Optional state write before routing to cleanup

```json
{
  "tool": "update_session_state",
  "args": {
    "session_id": "default",
    "current_stage": "table_processing",
    "last_user_goal": "resolve join risk threshold stop"
  }
}
```

### Follow-up rerun request template (only after explicit user confirmation)

```json
{
  "tool": "execute_multi_table_plan",
  "args": {
    "plan": {"steps": []},
    "auto_confirm_join": true,
    "max_left_unmatched_rows": 30,
    "max_right_unmatched_rows": 30,
    "max_left_duplicate_keys": 10,
    "max_right_duplicate_keys": 10
  }
}
```

## 2026-03-26 cross-layer template: stopped_missing_result_bindings

### Trigger payload (from runtime)

```json
{
  "execution_status": "stopped_missing_result_bindings",
  "stopped_at_step_id": "step_3",
  "stop_reason": "step `step_3` missing result_ref_bindings: step_2_result"
}
```

### Orchestrator response template

- Current understanding: plan execution paused because required prior-step output handle is missing.
- Current status: this is a controlled dependency stop, not a runtime crash.
- Next action: route to table-processing flow to restore chain context and re-run from the blocked step.

### Optional state write before routing

```json
{
  "tool": "update_session_state",
  "args": {
    "session_id": "default",
    "current_stage": "table_processing",
    "last_user_goal": "resolve missing result bindings"
  }
}
```

## 2026-03-26 cross-layer template: failed with unknown failure diagnostics

### Trigger payload (from runtime)

```json
{
  "execution_status": "failed",
  "stopped_at_step_id": "step_1",
  "stop_reason": "join_tables 缺少 left 参数",
  "failure_diagnostics": {
    "failure_class": "unknown_runtime_failure",
    "fallback_route": "table_processing_diagnostics",
    "fallback_message": "Execution encountered an unclassified runtime/tool failure. Route to table-processing diagnostics before retry.",
    "default_template": "resume_execution",
    "next_template_on_success": "resume_full_chain",
    "failed_step_id": "step_1",
    "failed_action": "join_tables",
    "failed_tool": "join_tables",
    "raw_error": "join_tables 缺少 left 参数",
    "state_synced": true,
    "recovery_templates": {
      "update_session_state": {
        "tool": "update_session_state",
        "args": {
          "session_id": "default",
          "current_stage": "table_processing",
          "last_user_goal": "resolve unknown multi-table failure at step_1"
        }
      },
      "resume_execution": {
        "tool": "execute_multi_table_plan",
        "args": {
          "session_id": "default",
          "plan": {"steps": [{"step_id": "step_1"}]},
          "auto_confirm_join": false,
          "stop_after_step_id": "step_1",
          "result_ref_bindings": {}
        }
      },
      "resume_full_chain": {
        "tool": "execute_multi_table_plan",
        "args": {
          "session_id": "default",
          "plan": {"steps": [{"step_id": "step_1"}]},
          "auto_confirm_join": false,
          "result_ref_bindings": {}
        }
      }
    }
  }
}
```

### Orchestrator response template

- Current understanding: execution failed in an unclassified runtime/tool branch rather than a controlled preflight gate.
- Current status: fallback route is `table_processing_diagnostics`; blocked step/action is provided in `failure_diagnostics`.
- Next action: route to table-processing diagnostics first, then use `recovery_templates.resume_execution` after blocked-step inputs are fixed.

### Follow-up payload after blocked-step replay success

When replay call (`stop_after_step_id=<blocked_step>`) succeeds, runtime returns:

```json
{
  "execution_status": "stopped_after_step_id",
  "continuation_templates": {
    "default_template": "resume_full_chain",
    "resume_full_chain": {
      "tool": "execute_multi_table_plan",
      "args": {
        "session_id": "default",
        "plan": {"steps": [{"step_id": "step_1"}]},
        "auto_confirm_join": false,
        "result_ref_bindings": {}
      }
    }
  }
}
```

### Single-call macro recovery template

```json
{
  "tool": "recover_multi_table_failure",
  "args": {
    "failure_diagnostics": {},
    "continue_after_replay": true
  }
}
```

Use this when you want runtime to execute:
1) blocked-step replay, then
2) full-chain continuation,
in one call.
