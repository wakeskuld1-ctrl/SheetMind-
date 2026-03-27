## 2026-03-27
### 修改内容
- `codex/mainline-integration`：从 `codex/p0-preflight-chain` 拉出主版本收口分支，并合并 `origin/main`，解决统计诊断相关冲突，形成提交 `521d65d`。
- `skills/analysis-modeling-v1/SKILL.md`：重写为 UTF-8 顶层技能说明，并补充 `Rust 二进制`、`不依赖 Python`、`不要求用户安装 Python` 运行时约束。目的：修复 `integration_binary_only_runtime` 守护测试并明确交付边界。
- `skills/excel-orchestrator-v1/SKILL.md`：重写为 UTF-8 顶层技能说明，并补充 binary-only runtime 约束。目的：统一总入口 Skill 的交付边界说明。
- `skills/table-processing-v1/SKILL.md`：重写为 UTF-8 顶层技能说明，并补充 binary-only runtime 约束。目的：统一表处理 Skill 的交付边界说明。
- `skills/decision-assistant-v1/SKILL.md`：重写为 UTF-8 顶层技能说明，并补充 binary-only runtime 约束。目的：统一决策层 Skill 的交付边界说明。
- `skills/analysis-modeling-v1/acceptance-dialogues.md`、`skills/analysis-modeling-v1/cases.md`、`skills/excel-orchestrator-v1/acceptance-dialogues.md`、`skills/table-processing-v1/acceptance-dialogues.md`、`skills/table-processing-v1/cases.md`：转换为可被 UTF-8 正常读取的编码。目的：消除顶层技能树中的非 UTF-8 文档风险。
- `tests/integration_cli_json.rs`：将 `suggest_table_workflow` 的两条旧断言从 `join_tables` 更新为 `join_preflight`。目的：让测试和当前“先预检再执行 join”的稳定协议一致。
- `README.md`、`scripts/run_cross_repo_gates.ps1`：新增跨仓统一门禁入口说明与执行脚本。目的：支持 `SheetMind-` 与 `SheetMind-Scenes` 一次性联跑回归。

### 修改原因
- 需要选出“功能最领先且稳定性最强”的主版本基线，并把 `main` 的唯一功能差异吸收到业务主线上。
- 全量测试暴露 `skills` 文档存在非 UTF-8 编码问题，导致 binary-only runtime 守护测试失败。
- `suggest_table_workflow` 已经改为先返回 `join_preflight`，但旧测试仍然断言 `join_tables`，导致全量测试失败。
- 本地已有跨仓门禁脚本草稿，需要纳入主版本候选并验证主仓稳定性。

### 方案还差什么
- [ ] 决定是否将 `codex/mainline-integration` 提升为新的默认主分支，或将其合并回 `main`。
- [ ] 处理 `SheetMind-Scenes` 的 contract gate 编译错误，再重新跑 `scripts/run_cross_repo_gates.ps1 -Suite all -LowMemory`。
- [ ] 在确认跨仓门禁全绿后，再删除或归档 `codex/merge-cli-mod-batches`、`refactor/cli-modularization` 等旧分支。

### 潜在问题
- [ ] `scripts/run_cross_repo_gates.ps1 -Suite all -LowMemory` 当前会在 `SheetMind-Scenes` 的 contract gate 阶段失败；触发条件是该仓继续保留现有编译错误。影响范围：跨仓统一门禁无法全绿，但 `SheetMind-` 主仓自身稳定性不受影响。
- [ ] `codex/merge-cli-mod-batches` 不是当前主线的增量优化，而是旧基线上的替代实现；若后续整分支硬并，可能回退当前多表稳定性链路与文档资产。

### 关闭项
- 已通过 `cargo test --test stat_diagnostics_cli`：8/8 通过。
- 已通过 `cargo test --test integration_cli_json execute_multi_table_plan -- --nocapture`。
- 已通过 `cargo test --test integration_cli_json join_preflight_returns_estimates_risks_and_preview_rows_in_cli -- --exact`。
- 已通过 `cargo test --test integration_binary_only_runtime`：2/2 通过。
- 已通过 `cargo test --test integration_cli_json suggest_table_workflow_recommends_join_in_cli -- --exact`。
- 已通过 `cargo test --test integration_cli_json suggest_table_workflow_preserves_nested_source_payloads_in_tool_call -- --exact`。
- 已通过 `cargo test`：`SheetMind-` 全量测试全绿。
- 已执行 `powershell -ExecutionPolicy Bypass -File E:\Excel\SheetMind-\scripts\run_cross_repo_gates.ps1 -Suite all -LowMemory`：`SheetMind-` recovery gates 全过，`SheetMind-Scenes` contract gates 因其自身编译错误失败。

### 记忆点（可选）
- 用户当前希望先收口出“最领先且最稳定”的主版本，再考虑删除其他开发分支；分支清理必须晚于统一验证，不要先删后验。
- 用户要求所有后续响应、过程说明和备注保持中文，并在代码/文档变更后补 task journal。
