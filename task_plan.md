# Task Plan
## 2026-04-01（方案 B-1 Fullstack 信息面总入口 V1）
### 本轮完成
- [x] 在 `tests/security_analysis_fullstack_cli.rs` 补 `tool_catalog` 可发现性、信息面成功聚合、信息源失败降级 3 组红绿测试，把“技术 + 财报 + 公告 + 行业”的总入口合同先锁住。
- [x] 新增 `src/ops/security_analysis_fullstack.rs`，在不改动 `technical_consultation_basic` 与 `security_analysis_contextual` 边界的前提下，聚合技术面、财报快照、公告摘要、行业上下文与综合结论。
- [x] 在 `src/ops/stock.rs`、`src/ops/mod.rs`、`src/tools/dispatcher/stock_ops.rs`、`src/tools/dispatcher.rs`、`src/tools/catalog.rs` 接入 `security_analysis_fullstack`，确保 CLI / Skill / catalog 主链可发现。
- [x] 新增 `docs/acceptance/2026-04-01-security-analysis-fullstack-v1.md`，补首版调用样例、信息源边界、降级规则与验收命令。
### 后续待办
- [ ] 评估是否把财报与公告抓取从“在线聚合”继续推进到本地 SQLite 持久化，避免免费源抖动影响重复查询体验。
- [ ] 评估是否继续接新闻面、资金面与一致预期，但保持它们停留在 fullstack 上层，不回灌到底层技术面 Tool。
- [ ] 评估是否补 A 股更多行业 profile，降低目前必须手工传 `sector_symbol` 的比例。
## 2026-04-01（方案 B 综合证券分析 V1）
### 本轮完成
- [x] 在 `tests/security_analysis_contextual_cli.rs` 补 `headwind`、profile 入口、缺代理配置错误路径 3 组红绿测试，把综合 Tool 从 MVP 推进到可交付 V1。
- [x] 重写 `src/ops/security_analysis_contextual.rs`，把 `market_symbol / sector_symbol` 升级为“显式 symbol 或 profile 二选一”，并补清晰的业务层错误。
- [x] 细化综合结论：三层同向为 `tailwind`，个股等待态维持 `mixed`，个股与大盘和板块双逆向时明确归入 `headwind`。
- [x] 新增 `docs/acceptance/2026-04-01-security-analysis-contextual-v1.md`，补最小调用示例、profile 清单、alignment 语义与当前边界说明。
### 后续待办
- [ ] 评估是否继续扩 profile 集合，至少补 A 股更多板块代理，避免目前只有银行板块 profile。
- [ ] 评估是否把 `security_analysis_contextual` 的文档再同步到 README 首页，方便仓库外部访客直接发现。
- [ ] 评估是否继续上探信息面、资金面、基本面，但保持综合 Tool 与底层技术面 Tool 的边界不混。
## 2026-04-01（方案 A-7 上层综合证券分析 Tool MVP）
### 本轮完成
- [x] 新增 `src/ops/security_analysis_contextual.rs`，以“三次 `technical_consultation_basic` + 上层聚合”的方式实现 `security_analysis_contextual`，先交付个股/大盘/板块三层 MVP。
- [x] 在 `src/ops/stock.rs`、`src/ops/mod.rs`、`src/tools/dispatcher/stock_ops.rs`、`src/tools/dispatcher.rs`、`src/tools/catalog.rs` 中接入新 Tool，确保 catalog、dispatcher 与 stock 域边界一致。
- [x] 跑通 `tests/security_analysis_contextual_cli.rs` 3 条红绿测试，验证 `tailwind` 与 `mixed` 两条核心路径。
- [x] 重新执行 `cargo fmt --all` 与 `cargo test --test integration_tool_contract -- --nocapture --test-threads=1`，确认新 Tool 接入没有破坏现有 stock 分组合同。
### 后续待办
- [ ] 评估是否补 `headwind` 的正式红测样本，把“个股与大盘/板块双逆向”也锁成合同。
- [ ] 评估是否给 `security_analysis_contextual` 补最小 README / JSON 示例，方便真实调用而不是只看测试。
- [ ] 评估是否继续在上层加入信息面、资金面或基本面，但保持 `technical_consultation_basic` 不扩边界。
## 2026-04-01（方案 A-6 外层合同空头对称样本）
### 本轮完成
- [x] 在 `tests/integration_tool_contract.rs` 中补 `build_confirmed_breakdown_rows()`，构造真实 `bearish_continuation` 的最小 CSV 样本，继续复用 `CSV -> import_stock_price_history -> technical_consultation_basic` 外层合同链路。
- [x] 新增 `technical_consultation_basic_contract_exposes_bearish_continuation_conclusion`，锁定 `consultation_conclusion.bias / confidence / headline / rationale / risk_flags` 的空头对称合同。
- [x] 执行 `cargo test --test integration_tool_contract -- --nocapture --test-threads=1` 与 `cargo fmt --all`，确认外层合同矩阵现已覆盖多头延续、空头延续、等待态三类主分支。
### 后续待办
- [ ] 评估是否执行整仓级 `cargo test -- --nocapture --test-threads=1`，把当前证券分析主线与并行改动一起做更大范围回归。
- [ ] 评估是否补最小使用说明或对外 JSON 示例，让真实调用不再依赖测试文件反推合同语义。
## 2026-04-01（方案 A-5 外层合同收口）
### 本轮完成
- [x] 在 `tests/integration_tool_contract.rs` 中补 `technical_consultation_basic` 的外层合同回归，覆盖 `bullish_continuation` 与 `range_wait` 两端样本，锁定 `consultation_conclusion.bias / confidence / headline / rationale / risk_flags` 的工具层可见性。
- [x] 为 `integration_tool_contract` 增加最小 CSV 夹具与导入助手，走真实 `CSV -> import_stock_price_history -> technical_consultation_basic` 链路，而不是手工伪造 JSON。
- [x] 执行 `cargo test --test integration_tool_contract -- --nocapture --test-threads=1` 与 `cargo fmt --all`，确认外层合同本身已自然满足，没有新增生产代码缺口。
### 后续待办
- [ ] 评估是否把 `consultation_conclusion` 的字段语义补进 README 或对外 JSON 示例，避免后续调用方只靠测试样例理解合同。
- [ ] 评估是否继续补 `confirmed_resistance_retest_hold / confirmed_support_retest_reject` 的外层 continuation 子分支合同，锁住回踩承接与反抽受压的细分风险文案。
## 2026-04-01（方案 A-4 延续态与等待态证据矩阵）
### 本轮完成
- [x] 先在 `tests/technical_consultation_basic_cli.rs` 里补 `technical_consultation_basic_marks_choppy_history_as_weak_trend / technical_consultation_basic_marks_confirmed_resistance_breakout_signal / technical_consultation_basic_marks_confirmed_support_breakdown_signal` 的 `rationale / risk_flags` 红测，确认缺口落在组合结论证据层，而不是 `bias / confidence` 主分类。
- [x] 在 `src/ops/technical_consultation_basic.rs` 的 `build_consultation_rationale()` 中补 `bullish_continuation / bearish_continuation / range_wait` 的专属解释，显式写出“延续已成立、后续确认点是什么”以及“趋势强度不足、先等待区间重新选边”。
- [x] 在 `src/ops/technical_consultation_basic.rs` 的 `build_consultation_risk_flags()` 中补 `bullish_continuation / bearish_continuation / range_wait` 的结构化风险提示，分别覆盖“突破后回踩”“跌破后反抽”“方向确认不足”。
- [x] 重新执行 `cargo test --test technical_consultation_basic_cli -- --nocapture --test-threads=1` 与 `cargo fmt --all`。
### 后续待办
- [x] 评估并补跑 `cargo test --test integration_tool_contract -- --nocapture --test-threads=1`，把 `consultation_conclusion` 的外层合同也锁到 `technical_consultation_basic` 链路。
- [ ] 评估是否把 `consultation_conclusion` 的字段语义补进文档或对外 JSON 示例，避免后续调用方只能从测试里反推合同。
## 2026-04-01（方案 A-3 陷阱态证据矩阵）
### 本轮完成
- [x] 先在 `technical_consultation_basic_marks_failed_resistance_breakout_signal / technical_consultation_basic_marks_failed_support_breakdown_signal` 中补 `rationale / risk_flags` 红测，验证陷阱态当前缺的是“原延续判断失效”的显式证据，而不是 bias 分类本身。
- [x] 在 `build_consultation_rationale()` 中为 `bull_trap_risk / bear_trap_risk` 补专属解释，让上层能直接知道原有突破/跌破延续剧本已经失效。
- [x] 在 `build_consultation_risk_flags()` 中为双边陷阱态补“突破延续判断失效 / 跌破延续判断失效”风险标记，不改 breakout 或 confidence 主逻辑。
- [x] 重新执行 `cargo fmt --all` 与 `cargo test --test technical_consultation_basic_cli -- --nocapture --test-threads=1`。
### 后续待办
- [ ] 继续按方案 A 评估是否把 `bullish_continuation / bearish_continuation / range_wait` 也补成同等级的 `rationale / risk_flags` 精确断言矩阵。
- [ ] 评估是否补跑 `cargo test --test integration_tool_contract -- --nocapture --test-threads=1`，把组合结论字段在外层合同再锁一遍。

## 2026-04-01（方案 A-2 组合结论证据矩阵）
### 本轮完成
- [x] 先用 `technical_consultation_basic_marks_bullish_range_watch_conclusion / technical_consultation_basic_marks_bearish_range_watch_conclusion / technical_consultation_basic_marks_resistance_retest_watch_signal / technical_consultation_basic_marks_support_retest_watch_signal` 补上 `rationale / risk_flags` 红测，而不是先改生产逻辑。
- [x] 在 `build_consultation_rationale()` 中为 `bullish_range_watch / bearish_range_watch` 补上方向化理由，让上层能直接知道“当前在等哪一侧关键位完成确认”。
- [x] 在 `build_consultation_risk_flags()` 中为 `resistance_retest_watch / support_retest_watch / bullish_range_watch / bearish_range_watch` 补上“尚待确认”类风险标记，不改 bias / confidence 分类主干。
- [x] 重新执行 `cargo fmt --all` 与 `cargo test --test technical_consultation_basic_cli -- --nocapture --test-threads=1`。
### 后续待办
- [ ] 继续按方案 A 评估是否把 `bull_trap_risk / bear_trap_risk / bullish_continuation / bearish_continuation / range_wait` 也补成同等级的 `rationale / risk_flags` 精确断言矩阵。
- [ ] 评估是否补跑 `cargo test --test integration_tool_contract -- --nocapture --test-threads=1`，把新的组合结论证据层在外层合同链路再锁一遍。

## 2026-04-01（方案 A-1 偏多区间观察态闭环）
### 本轮完成
- [x] 用新增回归 `technical_consultation_basic_marks_bullish_range_watch_conclusion` 先复现“偏多区间样本被误打成 `resistance_retest_watch`”的问题，而不是直接猜测主逻辑有错。
- [x] 仅调整 `build_bullish_range_watch_rows` 的最近 20 根尾部夹具，把“持续缓慢抬升”改成“先冲高、后高位箱体”，避免最近 4 根再次形成 breakout anchor。
- [x] 重新执行 `cargo fmt --all`、`cargo test --test technical_consultation_basic_cli technical_consultation_basic_marks_bullish_range_watch_conclusion -- --exact --nocapture --test-threads=1` 与 `cargo test --test technical_consultation_basic_cli -- --nocapture --test-threads=1`。
### 后续待办
- [ ] 继续按方案 A 补 `bullish_range_watch / bearish_range_watch` 的 `rationale / risk_flags / headline` 更细粒度断言，避免当前只锁 `bias / confidence / headline` 的最小合同。
- [ ] 评估是否补跑 `cargo test --test integration_tool_contract -- --nocapture --test-threads=1`，把组合结论字段在外层合同链路再确认一遍。

## 2026-04-01（方案 A 组合结论收口）
### 本轮完成
- [x] 复核 `technical_consultation_basic` 新增的 `consultation_conclusion` 合同是否已经真正接入 CLI 主链，而不是只停留在结构定义。
- [x] 用既有失败回归 `technical_consultation_basic_marks_confirmed_resistance_breakout_signal` 复现“confirmed 结构被误压成 low 置信度”的问题。
- [x] 最小修改 `classify_consultation_confidence`，为 `confirmed_* / failed_*` 这类已完成关键位结构补上最低 `medium` 置信度地板，不新增新模块或新入口。
- [x] 重新执行 `cargo fmt --all` 与 `cargo test --test technical_consultation_basic_cli -- --nocapture --test-threads=1`。
### 后续待办
- [ ] 继续按方案 A 梳理 `consultation_conclusion` 的样本矩阵，补齐空头观察态、陷阱态与区间态的 `confidence / rationale / risk_flags` 专项断言。
- [ ] 评估是否需要把 `consultation_conclusion` 补进文档或对外示例 JSON，方便后续 Skill / GUI / 其他 AI 直接复用。

## 2026-03-31（多根回踩补充）
### 本轮完成
- [x] 在 `technical_consultation_basic` 的既有关键位主线上补齐“多根回踩再站稳 / 多根反抽再受压”的识别能力，不新增对外模块。
- [x] 复用统一的旧关键位 + `buffer = max(snapshot.atr_14 * 0.25, 0.15)` 灰区口径，让 `watch / confirmed / failed` 三类判断都能覆盖 2~4 根磨位结构。
- [x] 补齐 CLI 回归测试：`technical_consultation_basic_marks_multi_bar_resistance_retest_hold_signal`。
- [x] 补齐 CLI 回归测试：`technical_consultation_basic_marks_multi_bar_support_retest_reject_signal`。

### 后续待办
- [ ] 评估是否继续补“多根回踩观察态 / 多根反抽观察态”的专门样本，避免未来只测确认态而漏掉灰区。
- [ ] 评估是否把 `MULTI_BAR_RETEST_LOOKBACK_BARS` 参数化，或继续保持当前固定 `4` 根窗口。
- [ ] 如继续按方案 A 推进，优先做关键位文案和样本矩阵整理，不新开证券分析模块。


## 2026-03-31（补充）
### 本轮完成
- [x] 把 `breakout_signal` 继续扩展到 `resistance_retest_watch / support_retest_watch` 观察态，补齐“正在回踩/反抽途中”和“已确认完成”之间的中间层。
- [x] 为旧关键位附近引入灰区 buffer，避免价格刚贴近旧关键位时就被过早判成 `confirmed_*` 或 `failed_*`。
- [x] 补齐 CLI 回归测试：`technical_consultation_basic_marks_resistance_retest_watch_signal`。
- [x] 补齐 CLI 回归测试：`technical_consultation_basic_marks_support_retest_watch_signal`。

### 后续待办
- [ ] 评估是否继续扩展为“多根回踩 / 多根反抽”结构，而不只依赖最近两根 K 线。
- [ ] 评估是否把 `KEY_LEVEL_LOOKBACK_DAYS` 与 retest buffer 参数化，避免不同价位区间共用同一固定口径。
- [ ] 如继续按方案 1 推进，优先补“多根回踩 / 多根反抽”的样本、文案和专项测试，不新开证券分析模块。

## 2026-03-31
### 当前任务
- [x] 方案 1：为 `technical_consultation_basic` 增加“支撑/阻力 + 突破有效性”最小能力切片。

### 本轮完成
- [x] 新增 `breakout_signal` 对外合同字段。
- [x] 新增 `indicator_snapshot.support_level_20 / resistance_level_20` 快照字段。
- [x] 在 `summary / recommended_actions / watch_points` 中接入关键位突破语义。
- [x] 补齐 CLI 回归测试：向上有效突破、向下有效跌破、默认成功合同可见性。
- [x] 把 `breakout_signal` 扩展到二阶段确认，补齐“假突破回落 / 假跌破拉回”语义。
- [x] 补齐 CLI 回归测试：`failed_resistance_breakout / failed_support_breakdown`。
- [x] 把 `breakout_signal` 扩展到三阶段确认，补齐“阻力转支撑回踩确认 / 支撑转阻力反抽受压”语义。
- [x] 补齐 CLI 回归测试：`confirmed_resistance_retest_hold / confirmed_support_retest_reject`。

### 后续待办
- [ ] 评估是否继续补“retest_watch”观察态，区分“正在回踩/反抽途中”和“已经确认完成”。
- [ ] 评估是否要把关键位窗口参数化，或继续保持固定 `20` 日窗口。
- [ ] 如要继续扩展证券分析，优先考虑“支撑转阻力/阻力转支撑的回踩确认”而不是新开模块。
