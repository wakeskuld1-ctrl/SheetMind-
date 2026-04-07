<!-- 2026-04-01 CST: 新增这份证券分析专用交接文档，原因是现有总交接手册仍以 2026-03-31 的 Excel/GUI 主线为主；目的是把 2026-04-01 之后的证券分析产品链、日期规则、数据源、验证口径和续做建议单独沉淀给后续 AI。 -->
# 证券分析交接摘要（给后续 AI）

更新日期：2026-04-02

## 1. 当前证券分析产品主线

- 底层技术面：
  - `src/ops/technical_consultation_basic.rs`
- 环境聚合层：
  - `src/ops/security_analysis_contextual.rs`
- 全面证券分析层：
  - `src/ops/security_analysis_fullstack.rs`
- 分发入口：
  - `src/tools/dispatcher/stock_ops.rs`
  - `src/tools/dispatcher.rs`
- 目录入口：
  - `src/tools/catalog.rs`

## 1.1 2026-04-02 已扩展成证券投前治理主线

<!-- 2026-04-02 CST: 追加证券投前治理链概览，原因是 2026-04-02 已不只是证券分析，而是把投决会、审批桥、仓位计划、审批简报、decision package、验包和版本化都接起来；目的是让后续 AI 不再把这条线误判成“只有 analysis/fullstack”的研究链。 -->
- 证据冻结：
  - `src/ops/security_decision_evidence_bundle.rs`
- 正反方投决会：
  - `src/ops/security_decision_committee.rs`
- 风险闸门：
  - `src/ops/security_risk_gates.rs`
- 投决卡：
  - `src/ops/security_decision_card.rs`
- 审批桥与正式提交：
  - `src/ops/security_decision_approval_bridge.rs`
  - `src/ops/security_decision_submit_approval.rs`
- 仓位计划：
  - `src/ops/security_position_plan.rs`
- 正式审批简报与签名：
  - `src/ops/security_decision_approval_brief.rs`
  - `src/ops/security_approval_brief_signature.rs`
- 审批包与治理校验：
  - `src/ops/security_decision_package.rs`
  - `src/ops/security_decision_verify_package.rs`
- 审批后版本化：
  - `src/ops/security_decision_package_revision.rs`
- 问答总入口 Skill：
  - `skills/security-pm-assistant-v1/SKILL.md`

## 2. 当前职责边界

- `technical_consultation_basic`
  - 只负责单证券日线技术面
  - 不要塞回大盘、板块、财报、公告语义
- `security_analysis_contextual`
  - 负责个股 + 大盘代理 + 板块代理的技术环境共振
- `security_analysis_fullstack`
  - 负责技术面 + 财报快照 + 公告摘要 + 行业上下文 + 综合结论

## 3. 日期锚定硬规则

<!-- 2026-04-01 CST: 新增这节日期规则，原因是用户明确纠正过 ETF/证券分析必须严格按当前日期收盘口径回答；目的是防止后续 AI 再混用更早日期的数据。 -->
- 证券分析默认只允许锚定“当前日期”
- 如果当前日期没有有效收盘数据，才允许退到前一个交易日
- 输出必须显式写明实际分析日期，例如：
  - `按 2026-04-01 收盘分析`
  - `当前日期无有效收盘，退至 2026-03-31 收盘分析`
- 不允许混用多个日期的数据去拼一个结论
- 如果无法核实当前日期或前一个交易日的可靠收盘数据，就明确说无法核实，不要编造价格

对应 Skill：

- `skills/security-analysis-v1/SKILL.md`

## 4. 当前数据源口径

- 行情：
  - 本地主链优先走 `import_stock_price_history` / `sync_stock_price_history` -> SQLite
  - 外查核验时只用免费、公开、无需 Token 的接口
- 财报快照：
  - 当前默认走东财公开接口
- 公告摘要：
  - 当前默认走东财公开接口
- 原则：
  - 不用大模型抓行情
  - 不用 Token
  - 免费源失败时允许降级，但必须说明 unavailable 范围

## 5. 当前可用输出层

- `technical_consultation_basic`
  - 提供 `trend_bias / trend_strength / breakout_signal / consultation_conclusion` 等结构化结果
- `security_analysis_contextual`
  - 提供 `alignment` 级别的环境判断，例如 `tailwind / mixed / headwind`
- `security_analysis_fullstack`
  - 提供：
    - `technical_context`
    - `fundamental_context`
    - `disclosure_context`
    - `industry_context`
    - `integrated_conclusion`

## 6. 当前已知降级语义

- 财报源失败：
  - `fundamental_context.status = "unavailable"`
- 公告源失败：
  - `disclosure_context.status = "unavailable"`
- 只要信息面未完整拿到：
  - `integrated_conclusion.stance = "technical_only"`

这表示当前产品优先保证“不因为免费源波动而整条链路中断”。

## 6.1 ETF 场景为什么会出现“信息面不足”

<!-- 2026-04-02 CST: 追加 ETF 信息面不足解释，原因是用户在正式投研会后追问了 159866 为什么只剩 technical_only；目的是把这个结论沉淀给下一位 AI，避免后续再把 ETF 当普通股票去抓财报和公告。 -->
- 不是“日本没有信息”，而是当前 ETF 信息面仍偏股票口径：
  - 财报层默认按个股财报快照抓取，ETF 没有营业收入/净利润语义。
  - 公告层默认按股票公告抓取，尚未优先覆盖 ETF 真正关键的：
    - 溢价风险提示
    - 停复牌/临停
    - 申赎清单
    - 跟踪误差
    - 份额变化
    - 基金运作公告
- 因而像 `159866.SZ` 这类跨境 ETF，目前很容易出现：
  - `fundamental_context.status = "unavailable"`
  - `disclosure_context.status = "unavailable"`
  - `integrated_conclusion.stance = "technical_only"`
- 后续如果要继续做 ETF 场景，优先补的是 ETF 专用信息面，而不是继续把股票财报/股票公告接口硬套上去。

## 7. 已完成验证

- `cargo test --test security_analysis_contextual_cli -- --nocapture --test-threads=1`
- `cargo test --test security_analysis_fullstack_cli -- --nocapture --test-threads=1`
- `cargo test --test integration_tool_contract -- --nocapture --test-threads=1`
- `cargo fmt --all`
- `cargo test --test security_decision_evidence_bundle_cli --test security_decision_committee_cli --test security_decision_submit_approval_cli --test security_decision_verify_package_cli --test security_decision_package_revision_cli`

补充说明：

- 2026-04-02 已验证证券投前治理切片：
  - `evidence_bundle -> committee -> submit_approval -> decision_package -> verify_package -> package_revision`
- 上述切片为绿色，但仍不是整仓全绿声明。

注意：

- 当前更多是证券分析切片级验证
- 不是整仓级全绿声明

## 8. 后续 AI 最容易犯的错

- 把信息面重新塞回 `technical_consultation_basic`
- 放着项目内 Tool 不用，回退成泛泛股评
- 没先锁“当前日期”，就混入别的交易日价格
- 把免费源 unavailable 误判成整个 Tool 不可用
- 把 Python `tradingagents/` 那套架构误当成当前本地证券分析产品主链

## 9. 建议下一个续做方向

- 先做信息面本地缓存：
  - 财报快照 SQLite
  - 公告摘要 SQLite
- 再做输出合同强化：
  - 在 Rust Tool 返回里显式补 `analysis_date`
- 再做信息面扩展：
  - 新闻面
  - 资金面
  - 一致预期

### 9.1 ETF/跨境 ETF 优先补什么

<!-- 2026-04-02 CST: 追加 ETF 续做优先级，原因是 159866 正式投研会暴露出 ETF 信息面明显缺口；目的是让后续续做优先补对地方，而不是继续堆股票式指标。 -->
- ETF 溢价/折价与 IOPV/NAV 偏离
- 跟踪误差与跟踪偏离
- 基金份额变化与申赎
- ETF 专用公告抓取与摘要
- 日元汇率、日经现货/期货、宏观事件的 ETF 场景映射
- 如果继续做“决策纠错/复盘”，优先补一个轻量 `review record`，并绑定：
  - `decision_ref`
  - `approval_ref`
  - `package_path`

## 10. 接手时先看这些文件

- `skills/security-analysis-v1/SKILL.md`
- `skills/security-pm-assistant-v1/SKILL.md`
- `src/ops/technical_consultation_basic.rs`
- `src/ops/security_analysis_contextual.rs`
- `src/ops/security_analysis_fullstack.rs`
- `src/ops/security_decision_evidence_bundle.rs`
- `src/ops/security_decision_committee.rs`
- `src/ops/security_decision_submit_approval.rs`
- `src/ops/security_decision_verify_package.rs`
- `src/ops/security_decision_package_revision.rs`
- `tests/security_analysis_contextual_cli.rs`
- `tests/security_analysis_fullstack_cli.rs`
- `tests/security_decision_evidence_bundle_cli.rs`
- `tests/security_decision_committee_cli.rs`
- `tests/security_decision_submit_approval_cli.rs`
- `tests/security_decision_verify_package_cli.rs`
- `tests/security_decision_package_revision_cli.rs`
- `docs/acceptance/2026-04-01-security-analysis-contextual-v1.md`
- `docs/acceptance/2026-04-01-security-analysis-fullstack-v1.md`
- `tests/runtime_fixtures/live_committee_159866_runtime/committee_result.json`
- `docs/交接摘要_给后续AI.md`
- `CHANGELOG_TASK.MD`

## 10.1 159866 正式投研会留档

<!-- 2026-04-02 CST: 追加 159866 正式投研会留档，原因是用户要求把“工银日经ETF 159866 + 40% 仓位 + 成本 1.466”的正式过会结果保留下来，后续要用于复盘和纠错；目的是给下一位 AI 一个可回读的真实案例。 -->
- 标的：
  - `159866.SZ`
- 分析日：
  - `2026-04-01`
- 运行工件：
  - `tests/runtime_fixtures/live_committee_159866_runtime/committee_result.json`
  - `tests/runtime_fixtures/live_committee_159866_runtime/runtime.db`
  - `tests/runtime_fixtures/live_committee_159866_runtime/stock_history.db`
- 运行口径：
  - `market_symbol = ^N225`
  - `sector_symbol = ^N225`
  - `stop_loss_pct = 0.05`
  - `target_return_pct = 0.12`
  - `min_risk_reward_ratio = 2.0`
- 关键结论：
  - `decision_card.status = needs_more_evidence`
  - `decision_card.direction = long`
  - `decision_card.position_size_suggestion = pilot`
  - `decision_card.confidence_score = 0.12`
  - 不支持把 `40%` 重仓继续视作“符合会上的仓位建议”
- 这次结论能成立的原因：
  - 风报比闸门通过
  - 但 `data_completeness_gate / market_alignment_gate / event_risk_gate` 都是 `warn`
  - 综合结论属于 `technical_only`
- 这次结论的核心限制：
  - 适合回答“当前是否支持重仓执行”
  - 不适合假装成 ETF 全信息面完备后的最终结论

## 11. 一句话结论

当前证券主线已经从“单证券技术面 demo”推进成“研究链 + 投决会 + 审批包 + 校验 + 版本化”的投前治理产品雏形；后续 AI 要做的是继续补 ETF/信息面缺口和复盘纠错能力，而不是重开一套分析架构。
