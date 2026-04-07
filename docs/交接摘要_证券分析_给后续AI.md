<!-- 2026-04-01 CST: 新增这份证券分析专用交接文档，原因是现有总交接手册仍以 2026-03-31 的 Excel/GUI 主线为主；目的是把 2026-04-01 之后的证券分析产品链、日期规则、数据源、验证口径和续做建议单独沉淀给后续 AI。 -->
# 证券分析交接摘要（给后续 AI）

更新日期：2026-04-01

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

## 7. 已完成验证

- `cargo test --test security_analysis_contextual_cli -- --nocapture --test-threads=1`
- `cargo test --test security_analysis_fullstack_cli -- --nocapture --test-threads=1`
- `cargo test --test integration_tool_contract -- --nocapture --test-threads=1`
- `cargo fmt --all`

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

## 10. 接手时先看这些文件

- `skills/security-analysis-v1/SKILL.md`
- `src/ops/technical_consultation_basic.rs`
- `src/ops/security_analysis_contextual.rs`
- `src/ops/security_analysis_fullstack.rs`
- `tests/security_analysis_contextual_cli.rs`
- `tests/security_analysis_fullstack_cli.rs`
- `docs/acceptance/2026-04-01-security-analysis-contextual-v1.md`
- `docs/acceptance/2026-04-01-security-analysis-fullstack-v1.md`
- `docs/交接摘要_给后续AI.md`
- `CHANGELOG_TASK.MD`

## 11. 一句话结论

当前证券分析主线已经不是“单证券技术面 demo”，而是一个能跑通 `技术面 -> 环境 -> 财报/公告 -> 综合结论` 的本地产品链；后续 AI 要做的是沿这条链继续加深，而不是重开一套分析架构。
## 12. 2026-04-08 七席委员会 V3 补充

- 当前正式投决主链：
  - `security_decision_briefing`
  - `security_committee_vote`
- 不要恢复旧 `security_decision_committee` 主链，也不要再造第二套正式投决入口。
- `security_committee_vote` 当前已升级为：
  - `committee_engine = "seven_seat_committee_v3"`
  - `6 名审议委员 + 1 名风控委员`
  - 固定席位：
    - `chair`
    - `fundamental_reviewer`
    - `technical_reviewer`
    - `event_reviewer`
    - `valuation_reviewer`
    - `execution_reviewer`
    - `risk_officer`
- “如何证明独立”当前依赖的正式证据字段：
  - 每席 `execution_mode == "child_process"`
  - 每席都有真实 `process_id`
  - 每席都有唯一 `execution_instance_id`
  - 同一轮 vote 的 7 个 `process_id` 唯一
  - 同一轮 vote 的 7 个 `execution_instance_id` 唯一
- 关键代码入口：
  - `src/ops/security_committee_vote.rs`
  - `src/tools/dispatcher.rs`
  - `src/tools/dispatcher/stock_ops.rs`
  - `tests/security_committee_vote_cli.rs`
  - `tests/security_analysis_resonance_cli.rs`
- 本轮特别修过的坑：
  - integration test 里的直接函数调用，`current_exe()` 不一定是 `excel_skill.exe`
  - 需要从测试 harness 邻近目录回推正式二进制，才能让 direct call 也复用真实 child process 路径
  - `briefing` 内嵌 vote 与重新执行 vote 的稳定语义应一致，但 `process_id / execution_instance_id` 不能再做全等比较
- 下次接手建议先读：
  - `docs/plans/2026-04-08-security-committee-vote-seven-seat-design.md`
  - `docs/plans/2026-04-08-security-committee-vote-seven-seat.md`
  - `src/ops/security_committee_vote.rs`
  - `tests/security_committee_vote_cli.rs`
  - `tests/security_analysis_resonance_cli.rs`
