# Task Plan

## 2026-04-13 Position Plan Sizing Layer Stage 2

### Goal
- 在不再做大重构的前提下，把第二阶段“给多少合适”正式接入 `position_plan + chair + submit_approval` 三条链，并统一对外呈现。

### Scope
- 在 `src/ops/security_position_plan.rs` 中补齐共享 sizing builder、overlay 和正式字段映射。
- 在 `src/ops/security_chair_resolution.rs` 中补齐 `target_gross_pct / sizing_grade / sizing_reason`。
- 在 `src/ops/security_decision_submit_approval.rs` 中接入 sizing overlay，并统一 `approval_brief.entry_summary`。

### Current Status
- [x] 第二阶段 sizing 规则对象、共享 builder 与 overlay 已落地。
- [x] `chair_resolution` 已同步暴露 sizing 结果。
- [x] `submit_approval` 已在 scorecard/master_scorecard 后刷新 sizing 层。
- [x] 聚焦红测已全部转绿。

### Acceptance Criteria
- [x] `position_plan` 包含 `target_gross_pct / sizing_grade / sizing_reason / sizing_risk_flags`
- [x] `chair_resolution` 包含 `target_gross_pct / sizing_grade / sizing_reason`
- [x] `approval_brief.entry_summary` 统一展示 `entry grade / target / 首仓 / max / reason`
- [x] `watch` 固定为 1% 观察仓且 `allow_add = false`
- [x] `blocked` 固定为 0 仓
- [x] `pilot_long / standard_long` 分别落到 6% / 12% 目标仓

### Phases
1. 复现第二阶段红测与缺失编译口
2. 补共享 sizing builder 与 position_plan 映射
3. 接入 submit_approval / chair_resolution
4. 运行聚焦验证并补 task journal

### Errors Encountered
| Error | Attempt | Resolution |
|-------|---------|------------|
| ready case 的 `plan_status` 被 sizing helper 意外改成 `probe_only` | 1 | 保持 sizing 层只刷新仓位能力，不再改原有 approval 状态语义 |

## 2026-04-12 Foundation Hygiene Export DTO V1

### Goal
- 在不改动 `RepositoryMetadataAudit` 内部聚合主流程的前提下，补一层稳定的 `v1` 对外 DTO，让后续 AI、上层封装和未来界面层不再直接耦合内部 report 结构。
### Scope
- 为 foundation repository hygiene audit 增加版本化导出 DTO。
- 锁定 summary / grouped views / reason views 的导出契约。
- 保持当前内部模型与既有语义不变，只增加单向导出层。
### Current Status
- 用户已批准 `方案 C`。
- 当前 foundation 主线已经锁定了：
  - 大仓库稳定性
  - repeated reason 边界语义
- 本轮准备进入 DTO 契约层的 TDD 切片。
### Acceptance Criteria
- [ ] 新增 `RepositoryMetadataAuditExportDtoV1` 及其必要子 DTO。
- [ ] `RepositoryMetadataAuditReport` 能稳定转换为 `v1` DTO。
- [ ] 导出 DTO 保持当前 summary / views / reason views 的既有语义与排序。
- [ ] 基础 fixture、大样本 fixture、边界 fixture 的 DTO 导出测试全部通过。
- [ ] foundation 回归测试通过。
### Phases
1. 写设计文档与实施计划
2. 先补 DTO 导出红测
3. 最小实现 DTO 与转换层
4. 跑 DTO 聚焦测试转绿
5. 跑 foundation 回归
6. 补 handoff 与 task journal
### Errors Encountered
| Error | Attempt | Resolution |
|-------|---------|------------|
| None yet | 0 | Design and plan stage |

## 2026-04-12 ETF 训练链路体检

### Goal
- 在不改动主架构的前提下，确认 ETF / 证券训练链路是否已经具备“进入提准阶段”的基础，并定位当前最主要的精度瓶颈。

### Scope
- 核对 ETF 主线关键 CLI 测试是否已通过。
- 核对训练 / refit / shadow evaluation / promotion 是否形成闭环。
- 判断瓶颈更偏向样本、标签、特征、聚合还是运行时模型接入。

### Current Status
- A1 已完成首个 ETF 端到端训练回归，并修复路径事件头标签契约错误。
- `security_external_proxy_history_import / feature_snapshot / master_scorecard / chair_resolution` 关键 ETF 主链测试已通过。
- `security_scorecard_training / security_scorecard_refit / security_shadow_evaluation / security_model_promotion` 关键训练治理测试已通过。
- 训练体检进入“瓶颈归因”阶段，重点确认 ETF 端到端训练覆盖、样本厚度与运行时接模路径。

## Goal
- 一次性推进证券主链到“只有通过明确验收门槛才算可实盘”的阶段。
- 当前最小阻断点是 latest `2026-04-10` 运行时的 ETF `feature_incomplete`，需要先修掉 validation/pool 链漏导 ETF 池级代理历史的问题，再继续推进整包剩余事项。

## Acceptance Criteria
- [x] latest runtime 中 `cross_border_etf / equity_etf` 不再因为缺少 ETF 代理历史而出现 `feature_incomplete`
- [x] pooled holdout 与 latest chair 结果能在同一代理历史口径下对齐
- [x] validation/pool 流程会自动导入 ETF 池级代理历史
- [x] 如果 ETF validation slice 缺少对应池级代理历史，流程会显式失败
- [x] `513180.SH / 515790.SH` latest raw 与 summary 同步更新，`score_status = ready`
- [ ] 只有全部主链门槛通过，才声明“可以做实盘”

## Phases
1. 确认 latest `feature_incomplete` 的真实根因
2. 补红测锁定 ETF 池级代理历史自动导入与缺失显式失败
3. 修复 validation/pool 流程并回归
4. 导入真实池级代理历史并重跑 latest chair
5. 继续推进整包剩余验收项

## Errors Encountered
| Error | Attempt | Resolution |
|-------|---------|------------|
| latest raw 仍为 `feature_incomplete`，但 summary 假阳性显示 `ready` | 1 | 继续向下追查 raw feature snapshot，确认真实问题是 ETF 池级代理历史未正式导入 governed external proxy 库 |
## 2026-04-12 Regression Head Precision Scheme C

### Goal
- 在不改主架构的前提下，把回归头从“门槛偏松 + 预测偏脆”推进到“治理更严 + 预测更稳”的下一阶段。

### Scope
- 收紧 `security_scorecard_training` 中回归头 readiness。
- 增加回归头相对基线的质量度量。
- 对回归 bin 预测值做一次最小幅度的 baseline shrinkage。

### Current Status
- 用户已批准 `方案 C`。
- 已完成测试锁定、最小实现与关键回归验证。
- 本轮没有扩散到 runtime registry 或训练大改。

### Acceptance Criteria
- [x] 回归头训练输出包含 `baseline_rmse` 与 `rmse_improvement_vs_baseline`
- [x] readiness 输出包含 `regression_quality_status`
- [x] 回归头只有在优于基线且方向性达标时才进入 `shadow_candidate_ready`
- [x] 薄 support 的回归 bin 会向全局 baseline 收缩
- [x] `security_scorecard_training_cli / security_master_scorecard_cli / security_chair_resolution_cli` 关键回归通过

### Phases
1. 先补红测锁定新的回归治理契约
2. 实现回归 bin shrinkage 与基线对比指标
3. 收紧回归 readiness 门槛
4. 跑关键回归并更新交接记忆

## 2026-04-12 Direction-First Training Run Slice

### Goal
- 在不改训练主链架构的前提下，补一个“方向优先、回归次优”的正式训练编排入口。
- 让后续 7 小时长训不再依赖 shell 临时循环，而是走可发现、可落盘、可续跑的工具入口。

### Scope
- 新增 `security_direction_first_training_run`
- 接入 `catalog + dispatcher + stock dispatcher + ops export`
- 支持：
  - 读取现有 registry 做排序
  - 桥接 training request 直接生成 registry 后再排序
- 输出一份 `stage_summary`

### Current Status
- [x] 红测已补齐并验证失败原因正确
- [x] 最小实现已完成
- [x] catalog 与 dispatcher 已接通
- [x] 排序单测与训练 CLI 回归已通过

### Acceptance Criteria
- [x] `tool_catalog` 能发现 `security_direction_first_training_run`
- [x] 方向指标强、回归指标弱的候选，仍能在 survivor 里获胜
- [x] 新工具能把排序摘要落盘
- [x] 新工具支持 registry source 与 training request source
- [ ] 基于真实多池子、多 horizon 的正式 7 小时请求完成组装
- [ ] 长训 checkpoint 记录模板补到 execution notes

### Phases
1. 补齐红测导入并确认真正 RED
2. 新增方向优先训练 op 与排序逻辑
3. 接入 dispatcher / catalog
4. 跑聚焦验证与 `security_scorecard_training_cli` 全回归
5. 进入真实长训请求组装
## 2026-04-13 Direction-First 7h Long Run Execution

### Goal
- 在不继续重构的前提下，直接启动正式的方向优先长训，并把运行证据固定下来，保证 7 小时后可以验收。

### Scope
- 生成正式请求 JSON
- 生成执行说明
- 启动后台进程
- 完成首轮健康检查

### Current Status
- [x] 正式请求 JSON 已生成
- [x] 执行说明已生成
- [x] 后台进程已启动
- [x] 首轮健康检查已完成
- [ ] 最终 `stage_summary` 待进程结束后生成
- [ ] 40 个候选的最终排序待进程结束后确认

### Acceptance Criteria
- [x] 进程不是秒退
- [x] runtime root 下开始生成候选目录与 artifact
- [x] PID、日志路径、请求路径已落文档
- [ ] 最终 stdout 返回 `security_direction_first_training_run` 成功结果
- [ ] 最终 stage summary 可用于后续 AI/人工验收

### Phases
1. 生成 40 候选正式请求
2. 落执行说明
3. 启动后台 EXE
4. 检查存活与落盘
5. 等待最终产物完成后验收
