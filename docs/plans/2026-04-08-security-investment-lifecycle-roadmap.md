# Security Investment Lifecycle Roadmap Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 在现有证券分析 Rust 主链上，把能力从“投前治理雏形”推进到“投前、投中、投后完整闭环”，并补齐 ETF / 跨境 ETF 的专项能力。

**Architecture:** 继续沿用现有 `security_analysis_* -> decision_evidence_bundle -> decision_committee -> position_plan -> submit_approval -> verify_package -> package_revision` 单主链，不重开第二套证券决策架构。后续开发按照“先收口正式对象，再补执行层，再补复盘层，最后补 ETF 专项适配”的顺序推进，确保所有新增对象都能挂回 `decision_ref / approval_ref / package_path`。

**Tech Stack:** Rust、CLI-first、Serde JSON、文件制品落盘、现有 `src/ops/` 证券治理链、`tests/*_cli.rs` 集成测试、必要时配合 runtime fixtures / SQLite 样例数据。

---

<!-- 2026-04-08 CST: 新增证券投前/投中/投后开发路线图，原因是需要把后续工作从零散对话要求收口成可执行计划；目的是让下一位 AI 或工程师能够沿既有证券主链按阶段继续推进，而不是重复造轮子。 -->

## 当前基线

- 已存在的核心对象与命令主要集中在以下文件：
  - `src/ops/security_decision_evidence_bundle.rs`
  - `src/ops/security_decision_committee.rs`
  - `src/ops/security_position_plan.rs`
  - `src/ops/security_decision_submit_approval.rs`
  - `src/ops/security_decision_verify_package.rs`
  - `src/ops/security_decision_package.rs`
  - `src/ops/security_decision_package_revision.rs`
  - `src/ops/security_risk_gates.rs`
- 已存在的 CLI 回归测试主要包括：
  - `tests/security_decision_evidence_bundle_cli.rs`
  - `tests/security_decision_committee_cli.rs`
  - `tests/security_decision_submit_approval_cli.rs`
  - `tests/security_decision_verify_package_cli.rs`
  - `tests/security_decision_package_revision_cli.rs`
- 当前已经完成的主线重点是“投前研究治理”：研究证据收集、投决会、审批提交、审批包校验、审批包修订。
- 当前明显缺口集中在三类：
  - 投前对象之间的正式契约还没有完全冻结，尤其是仓位计划、审批简报、会后结论之间的引用关系还不够完整。
  - 投中执行没有正式对象层，缺执行策略、执行日志、触发监控和重新开会机制。
  - 投后复盘还没有正式对象层，缺结果回填、窗口评估、委员校准和 ETF 专项信息面适配。

## Phase 1: 投前闭环收口

### Task 1: 冻结正式决策对象图

**目标：** 把“研究证据包、投决会结果、仓位计划、审批简报、审批结果、审批包”之间的引用关系一次性冻结成正式对象图，避免后续执行层和复盘层接入时再回头重构。

**涉及文件：**
- 重点阅读：
  - `src/ops/security_decision_evidence_bundle.rs`
  - `src/ops/security_decision_committee.rs`
  - `src/ops/security_position_plan.rs`
  - `src/ops/security_decision_package.rs`
  - `src/ops/security_decision_submit_approval.rs`
- 预期新增或修改：
  - `src/ops/security_decision_package.rs`
  - `src/tools/contracts.rs`
  - `src/tools/catalog.rs`
  - `tests/security_decision_verify_package_cli.rs`

**交付物：**
- 明确统一的 `decision_ref / approval_ref / package_path / brief_ref / position_plan_ref` 契约字段。
- 统一对象图关系说明，确保后续新增对象都能挂到审批包或其下游对象上。
- 更新相关 CLI 输出与 JSON schema 样例。

**验收标准：**
- 审批包校验命令能验证对象图中的关键引用是否存在且一致。
- 仓位计划与审批简报都能被审批包完整引用。
- 相关 CLI 测试通过，且无新增破坏性字段漂移。

### Task 2: 把仓位计划彻底挂入审批链

**目标：** 让 `security_position_plan` 从“分析附属输出”升级为“正式可审批对象”，并和 `decision_ref / approval_ref` 建立稳定引用。

**涉及文件：**
- 重点阅读：
  - `src/ops/security_position_plan.rs`
  - `src/ops/security_decision_submit_approval.rs`
  - `src/ops/security_decision_verify_package.rs`
- 预期新增或修改：
  - `src/ops/security_position_plan.rs`
  - `src/ops/security_decision_submit_approval.rs`
  - `tests/security_decision_submit_approval_cli.rs`
  - `tests/security_decision_verify_package_cli.rs`

**交付物：**
- 位置计划正式落盘结构，明确 `position_size / add_rule / reduce_rule / stop_rule / take_profit_rule`。
- 审批提交逻辑能够携带或引用仓位计划。
- 审批校验逻辑能够检查仓位计划是否齐备、是否与决议方向一致。

**验收标准：**
- 新生成的审批对象能够稳定引用仓位计划。
- 缺失仓位计划时，校验命令应给出明确失败或警告。
- `submit approval` 和 `verify package` 两条回归链路通过。

### Task 3: 正式化审批简报与会后结论

**目标：** 产出“可单独落盘、可签名、可进入 decision package 的正式审批简报对象”，把会前摘要和会后结论一起纳入治理链。

**涉及文件：**
- 重点阅读：
  - `src/ops/security_decision_committee.rs`
  - `src/ops/security_decision_package.rs`
  - `src/ops/security_decision_package_revision.rs`
- 预期新增或修改：
  - `src/ops/security_decision_package.rs`
  - `src/ops/security_decision_package_revision.rs`
  - `tests/security_decision_package_revision_cli.rs`
  - 可选新增 `src/ops/security_decision_brief.rs`

**交付物：**
- 审批简报对象，包括结论摘要、分歧点、关键风险、执行前提。
- 会后结论对象，包括最终采纳意见、驳回原因、补证要求、修订历史。
- 审批包可携带或引用上述对象。

**验收标准：**
- 审批简报和会后结论都可独立落盘。
- 审批包修订时可以追加新的简报或结论版本。
- 版本校验和修订回归测试通过。

## Phase 2: 投中执行层

### Task 4: 新增正式执行策略对象

**目标：** 把“盘前计划 / 盘中触发 / 盘后约束”从口头建议升级为正式 `execution_strategy` 对象，承接审批通过后的执行口径。

**涉及文件：**
- 重点阅读：
  - `src/ops/security_position_plan.rs`
  - `src/ops/security_decision_package.rs`
  - `src/ops/security_risk_gates.rs`
- 预期新增：
  - `src/ops/security_execution_strategy.rs`
  - `tests/security_execution_strategy_cli.rs`

**交付物：**
- 执行策略对象，覆盖允许开仓区间、允许加减仓条件、失效条件、盘中观察点、重新评估触发器。
- 可挂接到审批包或审批结果，形成“已批决议 -> 可执行计划”闭环。

**验收标准：**
- 执行策略可落盘、可被审批包引用、可被后续执行日志引用。
- 缺失关键执行约束时，命令行能给出结构化提示。

### Task 5: 触发监控与重新开会机制

**目标：** 在投中层引入触发监控，让价格、事件、风险闸门变化能够触发“重开投研会 / 重开投决会 / 仅更新执行策略”等分流动作。

**涉及文件：**
- 重点阅读：
  - `src/ops/security_risk_gates.rs`
  - `src/ops/security_decision_committee.rs`
  - `src/ops/security_decision_package_revision.rs`
- 预期新增或修改：
  - `src/ops/security_execution_monitor.rs`
  - `src/ops/security_decision_package_revision.rs`
  - `tests/security_decision_committee_cli.rs`
  - `tests/security_decision_package_revision_cli.rs`

**交付物：**
- 触发事件对象：价格跌破、价格突破、事件冲击、数据过期、审批失效。
- 触发分流规则：仅更新策略、要求补证、重开委员会、强制风控否决。

**验收标准：**
- 输入触发事件后，系统能明确给出后续动作类型。
- 不同等级触发事件能够正确挂接到 `decision_ref / approval_ref`。

### Task 6: 新增执行日志

**目标：** 把“实际做了什么、什么时候做的、是否偏离已批计划”沉淀为正式 `execution_log` 对象，为后续复盘提供可信输入。

**涉及文件：**
- 预期新增：
  - `src/ops/security_execution_log.rs`
  - `tests/security_execution_log_cli.rs`
- 预期修改：
  - `src/ops/security_decision_package.rs`
  - `src/tools/catalog.rs`

**交付物：**
- 执行日志对象：成交时间、成交价、成交量、动作类型、触发原因、关联策略、是否偏离计划。
- 执行日志可汇总成单笔决策的投中轨迹。

**验收标准：**
- 执行日志可独立落盘。
- 执行日志可被正式关联到审批包或其子对象。
- 执行日志可作为复盘输入被稳定读取。

## Phase 3: 投后复盘层

### Task 7: 新增复盘记录对象

**目标：** 引入正式 `review_record` 对象，记录结果、归因、偏差、纠错动作，避免复盘继续停留在对话文本中。

**涉及文件：**
- 预期新增：
  - `src/ops/security_review_record.rs`
  - `tests/security_review_record_cli.rs`
- 预期修改：
  - `src/ops/security_decision_package.rs`
  - `src/ops/security_decision_package_revision.rs`

**交付物：**
- 复盘记录对象：回看窗口、收益结果、最大回撤、是否达成原目标、偏差归因、纠错建议。
- 复盘记录能够回挂 `decision_ref / approval_ref / execution_log_ref / package_path`。

**验收标准：**
- 复盘记录可独立落盘。
- 它可以被打包到 decision package 或 revision 历史中。

### Task 8: 结果回填与后验窗口

**目标：** 定义统一的后验观察窗口，把“一周、两周、一个月、除权除息后”等结果回填规则标准化，避免未来再出现分红年份、观察日错配这种决策污染。

**涉及文件：**
- 重点阅读：
  - `src/ops/security_analysis_contextual.rs`
  - `src/ops/security_analysis_fullstack.rs`
  - `skills/security-analysis-v1/SKILL.md`
- 预期新增或修改：
  - `src/ops/security_signal_outcome_backfill.rs`
  - `tests/security_signal_outcome_backfill_cli.rs`
  - `skills/security-analysis-v1/SKILL.md`

**交付物：**
- 标准后验窗口定义：T+1、1 周、2 周、1 月、重大事件后。
- 结果回填对象与计算口径说明。
- 对分红、除权、停牌、ETF 溢折价等特殊情况给出明示规则。

**验收标准：**
- 同一证券在不同窗口下的回填结果可重复生成。
- 文档和 skill 中明确禁止“拿错误年份分红数据直接参与当前年度判断”。

### Task 9: 委员与风险闸门校准

**目标：** 让七人委员会和风控委员的历史命中情况、偏差模式、否决阈值进入可校准状态，使委员会不只是输出结论，还能被复盘纠偏。

**涉及文件：**
- 重点阅读：
  - `src/ops/security_decision_committee.rs`
  - `src/ops/security_risk_gates.rs`
- 预期新增或修改：
  - `src/ops/security_committee_calibration.rs`
  - `tests/security_decision_committee_cli.rs`
  - `tests/security_risk_gates_cli.rs`

**交付物：**
- 委员校准对象：委员倾向、历史表现、常见失误、最近修正记录。
- 风险闸门校准对象：哪些 gate 过松、哪些 gate 过严、哪些需要行业或市场差异化参数。

**验收标准：**
- 复盘完成后，委员会参数或提示信息可被正式更新。
- 更新后的校准结果能在下一轮投决会被读取，但不会破坏独立席位生成逻辑。

## Phase 4: ETF / 跨境 ETF 专项补洞

### Task 10: ETF 专用信息面适配层

**目标：** 专门补齐 ETF / 跨境 ETF 的信息面与执行口径，避免继续用个股模板硬套 ETF，尤其是日经 ETF、跨境 ETF、存在溢价和汇率因素的产品。

**涉及文件：**
- 重点阅读：
  - `src/ops/security_analysis_contextual.rs`
  - `src/ops/security_analysis_fullstack.rs`
  - `src/ops/technical_consultation_basic.rs`
- 预期新增或修改：
  - `src/ops/security_etf_context_adapter.rs`
  - `tests/security_analysis_contextual_cli.rs`
  - `tests/security_analysis_fullstack_cli.rs`
  - `skills/security-analysis-v1/SKILL.md`

**交付物：**
- ETF 专项字段：IOPV/NAV 偏离、溢折价、跟踪误差、申赎份额变化、汇率映射、指数期货联动、基金公告摘要。
- ETF 专项分析口径：什么时候可以只看技术面，什么时候必须提升到信息面不充分警告。
- 跨境 ETF 决策模板：明确“标的指数、汇率、期货、基金溢价”四层映射关系。

**验收标准：**
- ETF 场景下的最终结论不会再伪装成“个股式基本面齐全”。
- 像 `159866` 这样的案例能够得到更贴近 ETF 特性的分析与会后留档。

## 推荐执行顺序

1. 先完成 Phase 1 的 Task 1-3，把投前正式对象图收口。
2. 再完成 Phase 2 的 Task 4-6，把投中执行层挂到既有审批链上。
3. 再完成 Phase 3 的 Task 7-9，把复盘与委员校准做成正式对象层。
4. 最后补 Phase 4 的 Task 10，把 ETF / 跨境 ETF 的专项缺口一次性补齐。

## 里程碑定义

- 里程碑 M1：投前正式对象全部可引用、可校验、可进审批包。
- 里程碑 M2：执行策略、触发监控、执行日志可独立落盘并挂回审批链。
- 里程碑 M3：复盘记录、结果回填、委员校准全部进入正式对象层。
- 里程碑 M4：ETF / 跨境 ETF 案例具备专用信息面适配与复盘纠错能力。

## 下次接手建议阅读顺序

1. `docs/交接摘要_证券分析_给后续AI.md`
2. `docs/plans/2026-04-08-security-investment-lifecycle-roadmap.md`
3. `src/ops/security_decision_evidence_bundle.rs`
4. `src/ops/security_decision_committee.rs`
5. `src/ops/security_position_plan.rs`
6. `src/ops/security_decision_submit_approval.rs`
7. `src/ops/security_decision_verify_package.rs`
8. `src/ops/security_decision_package_revision.rs`
