# Progress
## 2026-04-01（方案 B-1 Fullstack 信息面总入口 V1）
### 已完成
- 新增 `security_analysis_fullstack` 总 Tool，请求层沿用 `symbol / market_symbol / sector_symbol / market_profile / sector_profile / as_of_date / lookback_days`，并额外支持 `disclosure_limit`。
- 新 Tool 当前已经把 4 层内容聚合到一枪输出：
- `technical_context`
- `fundamental_context`
- `disclosure_context`
- `industry_context`
- `integrated_conclusion`
- 财报面首版默认走免费公开源，返回最近报告期、披露日期、营收同比、归母净利润同比、ROE 与 `profit_signal`。
- 公告面首版默认走免费公开源，返回最近公告列表、关键词摘要、风险关键词识别与公告 headline。
- 信息源失败时已做正式降级：
- `fundamental_context.status = "unavailable"`
- `disclosure_context.status = "unavailable"`
- `integrated_conclusion.stance = "technical_only"`
- 已补文档 `docs/acceptance/2026-04-01-security-analysis-fullstack-v1.md`，便于后续真实接线和其他 AI 续做。
### 验证结果
- 已通过：`cargo test --test security_analysis_fullstack_cli -- --nocapture --test-threads=1`
- 已通过：`cargo test --test integration_tool_contract -- --nocapture --test-threads=1`
- 已通过：`cargo fmt --all`
### 当前判断
- 到这一步，证券分析产品主链已经不再只是“技术面 + 环境代理”，而是具备了最小可用的“技术 + 财报 + 公告 + 行业”统一入口。
- 首版信息面仍然是在线聚合，不做本地持久化；这对快速验证产品价值足够，但对重复查询稳定性还不够。
### 阻塞/限制
- 免费信息源当前只做最小聚合，尚未沉淀到 SQLite，离线复用能力仍不足。
- 行业层当前仍以 sector proxy 为主，尚未做自动行业映射和行业景气专属数据表。
## 2026-04-01（方案 B 综合证券分析 V1）
### 已完成
- `security_analysis_contextual` 现在已经不只是最小 MVP，而是具备了可交付 V1 的三块能力：`tailwind / mixed / headwind` 三类环境语义、profile 代理配置入口、缺参错误路径。
- 请求层已支持两种调用方式：
- 显式传 `market_symbol / sector_symbol`
- 传 `market_profile / sector_profile`
- 当前内置 profile 已落地：
- `a_share_core` -> `510300.SH`
- `a_share_bank` -> `512800.SH`
- 新增文档 `docs/acceptance/2026-04-01-security-analysis-contextual-v1.md`，可直接作为真实试用的最小调用说明。
### 验证结果
- 已通过：`cargo test --test security_analysis_contextual_cli -- --nocapture --test-threads=1`
- 已通过：`cargo test --test integration_tool_contract -- --nocapture --test-threads=1`
- 已通过：`cargo fmt --all`
### 当前判断
- 到这一步，综合证券分析已经可以真实拿来试，不再需要每次手工传完整代理 symbol，也不再只有顺风和等待态两条路。
- 当前 V1 仍然只做到技术面 + 大盘代理 + 板块代理，信息面、资金面、基本面还没有接入。
### 阻塞/限制
- profile 目前只内置了 A 股核心指数和银行板块两个最小入口，覆盖面还不够广。
- README 首页还没有同步这一段文档，当前说明主要放在 `docs/acceptance` 新文档里。
## 2026-04-01（方案 A-7 上层综合证券分析 Tool MVP）
### 已完成
- 新增 `security_analysis_contextual` 上层 Tool，请求收口为 `symbol / market_symbol / sector_symbol / as_of_date / lookback_days`，结果返回 `stock_analysis / market_analysis / sector_analysis / contextual_conclusion`。
- 聚合规则先按最小 MVP 落地：个股为 `range_wait` 时输出 `mixed`；个股与大盘、板块三者同向时输出 `tailwind`；其他分歧场景先归到 `mixed/headwind`。
- 已把新 Tool 接入 `stock` 域模块、catalog 和 dispatcher，`tool_catalog` 现在可以稳定发现 `security_analysis_contextual`。
### 验证结果
- 已通过：`cargo test --test security_analysis_contextual_cli -- --nocapture --test-threads=1`
- 已通过：`cargo test --test integration_tool_contract -- --nocapture --test-threads=1`
- 已通过：`cargo fmt --all`
### 当前判断
- 到这一步，证券分析已经不再只是“单个技术面 Tool”，而是具备了最小可用的“个股 + 大盘 + 板块”综合判断入口。
- 这仍是 MVP：环境层目前依赖代理符号，不含新闻、公告、资金面和基本面。
### 阻塞/限制
- 还没有补 `headwind` 的专门红测样本，当前只锁住了顺风与等待态混合场景。
- 综合结论文案当前仍是第一版，后续如果要直接面向用户展示，建议再补对外 JSON 示例或调用说明。
## 2026-04-01（方案 A-6 外层合同空头对称样本）
### 已完成
- 在 `tests/integration_tool_contract.rs` 中新增 `build_confirmed_breakdown_rows()` 与 `technical_consultation_basic_contract_exposes_bearish_continuation_conclusion`，把外层合同从“多头延续 + 等待态”补成“多头延续 + 空头延续 + 等待态”三类主分支。
- 这轮继续沿用真实 `CSV -> import_stock_price_history -> technical_consultation_basic` 链路，没有引入手工伪造 JSON 的测试捷径。
- 新增空头对称样本后，`integration_tool_contract` 7 条测试全绿，说明现有生产实现已自然满足外层对称合同，不需要再补业务代码。
### 验证结果
- 已通过：`cargo test --test integration_tool_contract -- --nocapture --test-threads=1`
- 已通过：`cargo fmt --all`
### 当前判断
- 到这一步，`technical_consultation_basic` 的正式证券分析合同，已经在外层测试里形成“多头延续 / 空头延续 / 等待态”三端闭环。
- 现在离“真实可用”差的主要不再是字段能力，而是整仓回归和最小使用说明。
### 阻塞/限制
- 本轮仍未执行整仓级 `cargo test -- --nocapture --test-threads=1`，跨模块联动风险没有一次性清扫。
## 2026-04-01（方案 A-5 外层合同收口）
### 已完成
- 在 `tests/integration_tool_contract.rs` 中新增两条外层合同回归，分别覆盖 `bullish_continuation` 与 `range_wait`，并补了最小 CSV 夹具、导入助手和真实 runtime SQLite 链路。
- 红测补完后直接跑 `cargo test --test integration_tool_contract -- --nocapture --test-threads=1`，结果两条新回归与原有 catalog 合同一起全绿，说明外层工具合同已经自然承接前面 CLI 层的 `consultation_conclusion` 能力。
- 这轮没有新增生产代码修改，收口动作只落在 `tests/integration_tool_contract.rs` 和记录文件。
### 验证结果
- 已通过：`cargo test --test integration_tool_contract -- --nocapture --test-threads=1`
- 已通过：`cargo fmt --all`
### 当前判断
- `technical_consultation_basic` 的 `consultation_conclusion` 现在已经同时在 CLI 切片回归和外层工具合同回归中锁住，方案 A 的“内部矩阵 + 外层合同”主收口可以视为完成。
- 当前最值得继续补的，不再是主字段存在性，而是 continuation 子分支和文档层的可消费性。
### 阻塞/限制
- 本轮仍未执行整仓级 `cargo test -- --nocapture --test-threads=1`，当前验证仍聚焦在证券分析主线与外层合同文件。
## 2026-04-01（方案 A-3 陷阱态证据矩阵）
### 已完成
- 在 `tests/technical_consultation_basic_cli.rs` 中继续补 2 组陷阱态专项断言，让 `bull_trap_risk / bear_trap_risk` 不再只锁 `bias / headline / 部分 risk_flags`，而是继续锁到“原延续判断失效”的 `rationale / risk_flags`。
- 先真实复现两条失败回归：陷阱态当前只会说“发生了假突破/假跌破”，不会显式告诉上层“原有延续判断已经失效”，说明缺口在证据层而不是 trap 分类本身。
- 在 `src/ops/technical_consultation_basic.rs` 中最小增量补两处实现：`build_consultation_rationale()` 为双边 trap risk 补专属解释，`build_consultation_risk_flags()` 为双边 trap risk 补“延续判断失效”风险标记。
### 验证结果
- 已通过：`cargo test --test technical_consultation_basic_cli technical_consultation_basic_marks_failed_resistance_breakout_signal -- --exact --nocapture --test-threads=1`
- 已通过：`cargo test --test technical_consultation_basic_cli technical_consultation_basic_marks_failed_support_breakdown_signal -- --exact --nocapture --test-threads=1`
## 2026-04-01（方案 A-4 延续态与等待态证据矩阵）
### 已完成
- 在 `tests/technical_consultation_basic_cli.rs` 中先补 3 组红测断言，分别锁定 `range_wait` 的“趋势强度不足 + 等待区间重新选边”，以及 `bullish_continuation / bearish_continuation` 的“延续成立 + 后续确认风险”。
- 真实跑出 3 个失败点后，只在 `src/ops/technical_consultation_basic.rs` 里做最小实现：给 `build_consultation_rationale()` 增加 `bullish_continuation / bearish_continuation / range_wait` 专属理由，给 `build_consultation_risk_flags()` 增加对应风险提示。
- 为了绕开 Windows 页面文件导致的 `rlib mmap` 阻塞，先清掉 `target\\debug\\deps\\libexcel_skill-91d7cde4bcafddb1.rlib/.rmeta` 再重编，之后 `technical_consultation_basic_cli` 51 条回归重新全绿。
### 验证结果
- 已通过：`cargo test --test technical_consultation_basic_cli -- --nocapture --test-threads=1`
- 已通过：`cargo fmt --all`
### 当前判断
- `consultation_conclusion` 现在在 `bullish_continuation / bearish_continuation / range_wait` 上也已经具备可直接消费的 `rationale / risk_flags`，方案 A 的内部证据矩阵基本闭环。
- 本轮没有改 dispatcher、没有新开 Tool、没有触碰并行修改中的 `foundation.rs / analysis_ops.rs / integration_tool_contract.rs`。
### 阻塞/限制
- 本轮仍未执行整仓级 `cargo test -- --nocapture --test-threads=1`，当前验证范围继续聚焦在证券分析切片。
- 已通过：`cargo test --test technical_consultation_basic_cli -- --nocapture --test-threads=1`
- 已通过：`cargo fmt --all`
### 当前判断
- `consultation_conclusion` 当前在 trap risk 场景上已经不只是“结构失败”的标签，而是能进一步表达“原有突破/跌破延续剧本失效”的证券分析语义。
- 当前整套 `technical_consultation_basic_cli` 回归仍为 51 条全部通过，说明这次陷阱态证据增强没有破坏既有区间态、观察态与其他技术指标主线。
### 阻塞/限制
- 本轮没有执行整仓级 `cargo test -- --nocapture --test-threads=1`，当前验证仍聚焦在证券分析切片。

## 2026-04-01（方案 A-2 组合结论证据矩阵）
### 已完成
- 在 `tests/technical_consultation_basic_cli.rs` 中继续补 4 组组合结论专项断言，让 `bullish_range_watch / bearish_range_watch / bullish_confirmation_watch / bearish_confirmation_watch` 不再只锁 `bias / confidence / headline`，而是继续锁到 `rationale / risk_flags`。
- 先真实复现 4 条失败回归：区间等待态缺少方向化 `rationale`，确认观察态缺少“尚待确认”类 `risk_flags`，说明缺口在组合结论证据层，不在 breakout 主分类。
- 在 `src/ops/technical_consultation_basic.rs` 中最小增量补两处实现：`build_consultation_rationale()` 为双边 `range_watch` 补方向化说明，`build_consultation_risk_flags()` 为双边 `range_watch / confirmation_watch` 补结构化确认风险。
### 验证结果
- 已通过：`cargo test --test technical_consultation_basic_cli technical_consultation_basic_marks_bearish_range_watch_conclusion -- --exact --nocapture --test-threads=1`
- 已通过：`cargo test --test technical_consultation_basic_cli technical_consultation_basic_marks_bullish_range_watch_conclusion -- --exact --nocapture --test-threads=1`
- 已通过：`cargo test --test technical_consultation_basic_cli technical_consultation_basic_marks_resistance_retest_watch_signal -- --exact --nocapture --test-threads=1`
- 已通过：`cargo test --test technical_consultation_basic_cli technical_consultation_basic_marks_support_retest_watch_signal -- --exact --nocapture --test-threads=1`
- 已通过：`cargo test --test technical_consultation_basic_cli -- --nocapture --test-threads=1`
- 已通过：`cargo fmt --all`
### 当前判断
- `consultation_conclusion` 当前在“区间方向观察态”和“确认观察态”上已经不只是标签层，而是具备了可直接消费的证据层合同。
- 当前整套 `technical_consultation_basic_cli` 回归仍为 51 条全部通过，说明这次证据层增强没有破坏既有 breakout、trap risk 与其他技术指标主线。
### 阻塞/限制
- 本轮没有执行整仓级 `cargo test -- --nocapture --test-threads=1`，当前验证仍聚焦在证券分析切片。

## 2026-04-01（方案 A-1 偏多区间观察态闭环）
### 已完成
- 在 `tests/technical_consultation_basic_cli.rs` 中把 `build_bullish_range_watch_rows()` 的高位尾盘从“缓慢抬升”改成“先冲高、后横住”的强势箱体，并在代码旁追加中文时间注释，明确修改原因与目的。
- 先用 `technical_consultation_basic_marks_bullish_range_watch_conclusion` 单测复现真实失败：当前样本会落到 `resistance_retest_watch`，而不是目标的 `range_bound -> bullish_range_watch`。
- 保持生产逻辑不动，只修正测试夹具，确认这次问题来自样本构造而非 `consultation_conclusion` 主逻辑漂移。
### 验证结果
- 已通过：`cargo test --test technical_consultation_basic_cli technical_consultation_basic_marks_bullish_range_watch_conclusion -- --exact --nocapture --test-threads=1`
- 已通过：`cargo test --test technical_consultation_basic_cli -- --nocapture --test-threads=1`
- 已通过：`cargo fmt --all`
### 当前判断
- `bullish_range_watch` 与 `bearish_range_watch` 当前都已经有真链路样本，方案 A 的区间方向观察态已形成双边闭环。
- 当前整套 `technical_consultation_basic_cli` 回归为 51 条全部通过，说明这次夹具修正没有破坏既有 breakout、trap risk、区间态与其他技术指标主线。
### 阻塞/限制
- 本轮没有执行整仓级 `cargo test -- --nocapture --test-threads=1`，当前验证仍以证券分析切片回归为主。

## 2026-04-01（方案 A 组合结论收口）
### 已完成
- 在 `src/ops/technical_consultation_basic.rs` 的 `classify_consultation_confidence()` 中补上 `has_completed_structure` 收口，确保 `confirmed_* / failed_*` 这类已完成关键位结构在量能偏弱时仍保持最小 `medium` 置信度。
- 保持本轮修改继续沿 `technical_consultation_basic` 单一证券分析主线推进，没有新开 Tool、没有新增数据入口、没有把 `volume_confirmation` 与 `breakout_signal` 混成同一个字段。
- 复用了既有 CLI 失败回归 `technical_consultation_basic_marks_confirmed_resistance_breakout_signal` 作为问题复现样本，没有额外引入一次性调试代码。
### 验证结果
- 已通过：`cargo test --test technical_consultation_basic_cli technical_consultation_basic_marks_confirmed_resistance_breakout_signal -- --exact --nocapture --test-threads=1`
- 已通过：`cargo test --test technical_consultation_basic_cli -- --nocapture --test-threads=1`
- 已通过：`cargo fmt --all`
### 当前判断
- `consultation_conclusion` 当前已经从“结构定义”推进到“真实可消费合同”，至少在多头 confirmed 场景下，`bias + confidence` 的语义边界和既有 `breakout_signal / volume_confirmation` 主合同保持一致。
- 当前整套 `technical_consultation_basic_cli` 回归为 51 条全部通过，说明这次修正没有破坏既有 MFI / CCI / Williams %R / Bollinger / OBV / RSRS / breakout 主线。
### 阻塞/限制
- 整仓级 `cargo test -- --nocapture --test-threads=1` 本轮未执行；当前验证仍以证券分析切片回归为主。

## 2026-03-31（多根回踩补充）
### 已完成
- 在 `src/ops/technical_consultation_basic.rs` 中新增 `MULTI_BAR_RETEST_LOOKBACK_BARS` 与两组锚点扫描函数，把最近若干根内先突破/跌破、再经历多根磨位的结构接入 `classify_retest_watch_signal`、`classify_confirmed_retest_signal`、`classify_failed_breakout_signal`。
- 在 `tests/technical_consultation_basic_cli.rs` 中新增 `build_multi_bar_resistance_retest_hold_rows / build_multi_bar_support_retest_reject_rows` 两组夹具，以及对应两条红绿测试。
- 当前 `breakout_signal` 已能覆盖单根与多根两类关键位回踩/反抽确认，不再把这两类样本回退成 `range_bound`。

### 验证结果
- 已通过：`cargo fmt --all`
- 已通过：`cargo test --test technical_consultation_basic_cli technical_consultation_basic_marks_multi_bar_resistance_retest_hold_signal -- --nocapture --test-threads=1`
- 已通过：`cargo test --test technical_consultation_basic_cli technical_consultation_basic_marks_multi_bar_support_retest_reject_signal -- --nocapture --test-threads=1`
- 已通过：`cargo test --test technical_consultation_basic_cli -- --nocapture --test-threads=1`
- 已通过：`cargo test --test stock_price_history_import_cli -- --nocapture --test-threads=1`

### 当前判断
- 这轮增量继续维持“只在 `technical_consultation_basic` 内加深关键位语义”的边界，没有引入新的数据入口、模块或外部依赖。
- 多根结构当前只扩到确认与失效链路，仍沿用同一套 `ATR` 灰区，不和 `volume_confirmation` 做强绑定。

### 阻塞/限制
- 整仓 `cargo test -- --nocapture --test-threads=1` 本轮未再执行；当前仍沿用既有环境记录，不把它当成功能切片失败。


## 2026-03-31（补充）
### 已完成
- 在 `src/ops/technical_consultation_basic.rs` 中补上 `resistance_retest_watch / support_retest_watch` 两个 `breakout_signal` 语义，并把相关说明同步接入 `summary / recommended_actions / watch_points`。
- 在 `tests/technical_consultation_basic_cli.rs` 中新增 `build_resistance_retest_watch_rows / build_support_retest_watch_rows` 两组 CSV 样本夹具，覆盖关键位灰区内“继续观察”的真实场景。
- `breakout_signal` 当前已覆盖：`confirmed_resistance_breakout`、`confirmed_support_breakdown`、`resistance_breakout_watch`、`support_breakdown_watch`、`failed_resistance_breakout`、`failed_support_breakdown`、`confirmed_resistance_retest_hold`、`confirmed_support_retest_reject`、`resistance_retest_watch`、`support_retest_watch`、`range_bound`。

### 验证结果
- 已通过：`cargo fmt --all`
- 已通过：`cargo test --test technical_consultation_basic_cli technical_consultation_basic_marks_resistance_retest_watch_signal -- --nocapture --test-threads=1`
- 已通过：`cargo test --test technical_consultation_basic_cli technical_consultation_basic_marks_support_retest_watch_signal -- --nocapture --test-threads=1`
- 已通过：`cargo test --test technical_consultation_basic_cli -- --nocapture --test-threads=1`
- 已通过：`cargo test --test stock_price_history_import_cli -- --nocapture --test-threads=1`
- 未完成整仓绿灯：`cargo test -- --nocapture --test-threads=1`

### 当前判断
- `breakout_signal` 现在已经能区分“突破/跌破确认”“观察态”“失败态”“回踩/反抽确认态”和“区间震荡态”，关键位主线的最小结构语义已基本闭环。
- 本轮新增的 `retest_watch` 没有把量能强绑进价格结构判断，`volume_confirmation` 仍保持为独立维度，避免语义重叠。

### 阻塞/限制
- 整仓 `cargo test` 在当前 Windows 环境仍受页文件/内存问题影响，这一项继续按环境级阻塞记录，不视为本轮证券分析切片回归失败。

## 2026-03-31
### 已完成
- 在 `src/ops/technical_consultation_basic.rs` 中新增关键位快照与 `breakout_signal` 分类。
- 在 `tests/technical_consultation_basic_cli.rs` 中新增：
- `technical_consultation_basic_marks_confirmed_resistance_breakout_signal`
- `technical_consultation_basic_marks_confirmed_support_breakdown_signal`
- `technical_consultation_basic_marks_failed_resistance_breakout_signal`
- `technical_consultation_basic_marks_failed_support_breakdown_signal`
- `technical_consultation_basic_marks_confirmed_resistance_retest_hold_signal`
- `technical_consultation_basic_marks_confirmed_support_retest_reject_signal`
- 在默认成功样本中补齐 `breakout_signal / support_level_20 / resistance_level_20` 合同可见性断言。
- 把关键位判断升级为“当前关键位外侧确认 + 上一根越位后当前收回”的二阶段结构识别。
- 把关键位判断继续升级为“突破/跌破后的第一次回踩承接 / 反抽受压”三阶段结构识别。

### 验证结果
- 已通过：`cargo test --test technical_consultation_basic_cli -- --nocapture --test-threads=1`
- 已通过：`cargo test --test technical_consultation_basic_cli technical_consultation_basic_marks_failed_resistance_breakout_signal -- --nocapture --test-threads=1`
- 已通过：`cargo test --test technical_consultation_basic_cli technical_consultation_basic_marks_failed_support_breakdown_signal -- --nocapture --test-threads=1`
- 已通过：`cargo test --test technical_consultation_basic_cli technical_consultation_basic_marks_confirmed_resistance_retest_hold_signal -- --nocapture --test-threads=1`
- 已通过：`cargo test --test technical_consultation_basic_cli technical_consultation_basic_marks_confirmed_support_retest_reject_signal -- --nocapture --test-threads=1`
- 已通过：`cargo test --test stock_price_history_import_cli -- --nocapture --test-threads=1`
- 未完成整仓绿灯：`cargo test -- --nocapture --test-threads=1`

### 当前判断
- 本轮新增能力已经在证券分析主线上稳定落地，且未破坏已有 ADX / OBV / 背离 / KDJ / RSRS / MFI / CCI / Williams %R / Bollinger 回归。
- `breakout_signal` 当前是“价格结构维度”字段，量能强弱仍由既有 `volume_confirmation` 单独表达，两者不会强绑为同一判断。
- `breakout_signal` 当前已覆盖第一阶段突破、第二阶段失效、第三阶段回踩/反抽确认，但还没有“retest_watch”观察态。

### 阻塞/限制
- 整仓 `cargo test` 在当前 Windows 环境下因为页文件/内存不足触发编译器与链接阶段失败，属于环境级阻塞，不是本轮关键位功能断言失败。
