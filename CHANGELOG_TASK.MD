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

## 2026-03-27
### 修改内容
- `skills/analysis-modeling-v1/SKILL.md`：从 `63aba7d` 恢复较完整正文，并补回 `2026-03-26 P1 operational analytics chain`，修正历史坏字节导致的中文样例乱码。目的：恢复分析建模 Skill 的细节密度，同时保留 UTF-8 与 binary-only runtime 约束。
- `skills/excel-orchestrator-v1/SKILL.md`、`skills/table-processing-v1/SKILL.md`、`skills/decision-assistant-v1/SKILL.md`：从 Git 历史恢复较完整版本，而不是继续使用之前为过测试而精简后的短版文案。目的：恢复路由规则、异常分流和操作约束等细节沉淀。
- `skills/analysis-modeling-v1/acceptance-dialogues.md`、`skills/analysis-modeling-v1/cases.md`、`skills/excel-orchestrator-v1/acceptance-dialogues.md`、`skills/table-processing-v1/acceptance-dialogues.md`、`skills/table-processing-v1/cases.md`：恢复为历史里可读、可作为 UTF-8 正常消费的版本。目的：修复先前转码导致的对话验收稿和案例文档乱码。

### 修改原因
- 用户明确指出当前顶层 Skill 被我收得过短，担心丢掉后续还要用到的经验沉淀。
- 上一轮为快速收口主版本，优先保证了测试与边界约束，但 Skill 文案细节确实被压缩过。
- 之前对部分 support 文档做编码修复时，存在把文档“转成 UTF-8 但语义展示变脏”的风险，需要回退到历史干净版本。

### 方案还差什么
- [ ] 决定这些恢复后的 Skill 文案是否还要继续做结构化整理，还是先保持历史完整度。
- [ ] 如果后续要继续增强 Skill，建议单独起一轮“Skill 资产整理”而不是混进主版本分支清理。

### 潜在问题
- [ ] `analysis-modeling-v1` 的 `2026-03-26 P1 operational analytics chain` 是基于历史增量手工纠正坏字节后补回的；虽然测试不覆盖这段自然语言，但若你后续想严格对齐历史原稿，仍建议再做一次人工校对。
- [ ] `excel-orchestrator-v1` 与 `table-processing-v1` 的 support 文档恢复到了最近可读版本，而不是所有后续脏编码提交里的全部增量；影响范围：可能仍少量缺失更晚期但未清洗的验收话术细节。

### 关闭项
- 已通过 `cargo test --test integration_binary_only_runtime`。
- 已通过 `cargo test`：恢复 Skill 资产后，`SheetMind-` 全量测试仍然全绿。

### 记忆点（可选）
- 用户对 Skill 资产完整度敏感，后续不要为了短期过测试就默认大幅缩写 Skill 文案；优先恢复历史完整内容，再做最小修补。

## 2026-03-27
### 修改内容
- Git 主版本收口：将本地 `main` 先快进到 `origin/main`，再快进到稳定分支提交 `dc8934a`，并推送到 GitHub。目的：把“最领先且最稳定”的版本正式切成主版本。
- Git 清理：删除远端与本地的 `codex/mainline-integration`、`codex/p0-preflight-chain`、`codex/merge-cli-mod-batches`、`refactor/cli-modularization`。目的：清理已被主版本吸收的开发分支，收口仓库视图。
- Git 回退锚点：创建并推送标签 `mainline-20260327-stable`。目的：在删分支后仍保留一个明确的可回退稳定落点。
- 跨仓门禁验证：重新从正确工作目录执行 `SheetMind-Scenes` contract gate 与 `SheetMind-` cross-repo gate。目的：确认之前的失败来自调用环境差异，而不是主版本代码回退。

### 修改原因
- 用户要求把主版本切清楚，并清理掉其他版本，确保仓库结构和后续协作入口足够清晰。
- 在删除分支前，需要先把稳定主线明确落到 `main`，否则远端仍会保留多个并行候选版本。
- 前一次 `SheetMind-Scenes` gate 失败被确认是从错误目录调用 `cargo` 引起，需要先验证跨仓门禁全绿后再删分支。

### 方案还差什么
- [ ] 如果后续还要继续推进 `SheetMind-Scenes` 的私有演进，需要单独决定它自己的版本管理方式；当前它仍不是 Git 仓库。
- [ ] 如果之后需要对外发布说明，可再补一份“主版本切换说明”到 README 或发布日志。

### 潜在问题
- [ ] 已删除的开发分支不再保留为分支名入口；虽然已有标签 `mainline-20260327-stable` 可回退，但如果后续有人还按旧分支名协作，需要先通知切换到 `main`。
- [ ] `SheetMind-Scenes` 的 gate 成功依赖从正确仓库根目录运行，以便加载 `.cargo/config.toml`；如果后续从错误目录直接调用 `cargo`，仍可能再次触发默认源/证书问题。

### 关闭项
- 已执行 `powershell -ExecutionPolicy Bypass -File E:\Excel\SheetMind-Scenes\scripts\run_contract_gates.ps1 -Suite all -LowMemory`：通过。
- 已执行 `powershell -ExecutionPolicy Bypass -File E:\Excel\SheetMind-\scripts\run_cross_repo_gates.ps1 -Suite all -LowMemory`：通过。
- 已完成 `main` 推送：`origin/main -> dc8934a`。
- 已完成远端与本地分支清理，当前远端仅保留 `main`。

### 记忆点（可选）
- 在 Windows PowerShell 下不要把需要串行依赖的 Git 命令并行执行，也不要直接用 `&&`；切分支、合并、推送要分步做，否则容易出现状态判断错位。
- 处理 Rust 子仓 gate 问题时，先确认 `cargo` 是不是从正确仓库根目录执行，避免把 `.cargo/config.toml` 没生效误判成代码故障。
