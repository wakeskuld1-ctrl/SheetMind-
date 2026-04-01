# Findings
## 2026-04-01（方案 B-1 Fullstack 信息面总入口 V1）
### 关键发现
- 用户真正要的不是“继续手工外查信息面”，而是一个能直接进入产品主链的统一总 Tool；因此继续扩 `security_analysis_contextual` 会让它职责变重，单独新开 `security_analysis_fullstack` 更稳。
- 对首版信息面来说，最有价值、且最适合走免费公开源的不是全量新闻，而是“最新财报快照 + 最近公告摘要”。
- 免费信息源天然会抖动，所以总 Tool 的错误边界必须是“技术主链失败才中断，信息面失败只降级”，否则产品稳定性会被第三方波动拖垮。
### 本轮决策
- 新增 `security_analysis_fullstack` 独立上层 Tool，不把财报和公告语义回灌到 `technical_consultation_basic` 或 `security_analysis_contextual`。
- 财报层先收口到“最近报告期 + 核心同比指标 + ROE + 利润信号”，公告层先收口到“最近公告列表 + 关键词摘要 + 风险关键词”。
- 信息源失败时返回结构完整的 unavailable 对象，而不是直接把整个 Tool 打成 error。
### 原因
- 这样既能最快把“全面证券分析”推进到产品可用状态，也保住了底层技术 Tool 的 SRP 边界。
- 首版先把最小高价值信息面打通，比一开始就做新闻、资金、研报全量抓取更符合当前阶段的交付性。
### 风险提示
- 当前财报和公告仍是在线聚合，没有持久化层，重复查询会继续受第三方源稳定性影响。
- 行业层当前仍是代理环境，不代表已经接入了真正的行业景气数据库；后续如果要做更深的行业分析，仍需要单独数据源与映射规则。
## 2026-04-01（方案 B 综合证券分析 V1）
### 关键发现
- 综合 Tool 真正缺的不是继续往底层堆信号，而是三个交付层缺口：逆风合同、代理配置入口、最小可用文档。
- `market_symbol / sector_symbol` 如果继续要求每次都显式传，真实使用门槛仍然偏高；但直接做全量行业自动映射又会把范围拉大，因此 profile 入口是当前最稳的中间态。
- 对综合 Tool 来说，`headwind` 不能只存在于代码分支，必须有正式红测，否则环境逆向语义很容易在后续调整中漂移。
### 本轮决策
- 把综合 Tool 请求升级为“显式 symbol 或 profile 二选一”，先只内置最小 A 股 profile。
- 错误路径不依赖解析层默认报错，而是在业务层明确返回“缺少大盘代理配置/板块代理配置”。
- 文档不强行插入现有 README 乱码段落，先单独落在 `docs/acceptance/2026-04-01-security-analysis-contextual-v1.md`，避免低价值的编码噪音修改。
### 原因
- 这样能最快把综合证券分析从“能跑”推进到“能交付、能试用、能续做”。
- 同时仍然保持 `technical_consultation_basic` 单一职责不被破坏。
### 风险提示
- 当前 profile 太少，还不足以覆盖更多 A 股行业或港股场景。
- 如果后续要引入信息面或资金面，必须继续保持在上层 Tool 聚合，不要回塞到底层技术面模块。
## 2026-04-01（方案 A-7 上层综合证券分析 Tool MVP）
### 关键发现
- 用户要补的是“上层综合证券分析”，不是继续把大盘、板块语义塞回 `technical_consultation_basic`；最稳的做法是新增独立 Tool，在上层复用三次底层技术面结果。
- 现有 `technical_consultation_basic` 已经足够作为三层聚合的底座，当前缺的是统一入口与环境共振语义，而不是再发明一套新指标。
- 真实红测表明，MVP 阶段只锁 `tailwind` 和“个股仍等待时保持 `mixed`”两条主路径，就足以把综合 Tool 正式挂上主链。
### 本轮决策
- 新增 `security_analysis_contextual` 独立模块，不改 `technical_consultation_basic` 主逻辑。
- 聚合层只做方向映射和环境结论，先不接新闻、公告、资金面、基本面。
- 继续沿用真实 `CSV -> import_stock_price_history -> technical_consultation_basic -> contextual tool` 链路，而不是伪造 JSON。
### 原因
- 这样可以保持 stock 域 SRP 清晰：底层 Tool 负责单证券技术面，上层 Tool 负责多层环境聚合。
- 先用代理符号做大盘与板块环境，能以最小成本把“更全面的证券分析”真正跑起来。
### 风险提示
- 当前 `headwind` 只在代码规则里存在，还缺显式红测；后续如果要扩展综合结论，建议先补这条样本。
- 上层环境仍依赖用户提供或外部预选的代理符号，如果代理选错，综合结论也会偏。
## 2026-04-01（方案 A-6 外层合同空头对称样本）
### 关键发现
- 在补完 `bullish_continuation` 与 `range_wait` 后，外层合同矩阵还差 `bearish_continuation` 的对称样本；如果不补，空头路径仍主要依赖 CLI 专项回归证明。
- 当前生产实现本身已经支持空头延续语义，外层缺的是“工具层正式合同样本”，不是业务逻辑能力。
### 本轮决策
- 只在 `tests/integration_tool_contract.rs` 增加空头对称样本与断言，不改生产代码。
- 继续沿用真实导数链路，而不是为了省事直接拼装响应 JSON。
### 原因
- 这一步的目标是把“能不能真实对外稳定返回”做成对称矩阵，而不是重复开发已存在的能力。
- 在并行改动背景下，优先补合同样本比继续挪动主逻辑更稳。
### 风险提示
- 当前主线仍缺整仓级回归；局部测试通过不等于所有并行改动合起来一定没问题。
- 目前还缺最小使用说明，外部使用者仍需要从测试样本理解输入输出语义。
## 2026-04-01（方案 A-5 外层合同收口）
### 关键发现
- `tests/integration_tool_contract.rs` 在这轮之前只锁了 tool catalog 分组与扁平目录，没有任何一条回归验证 `technical_consultation_basic` 的真实返回合同。
- 当前 `consultation_conclusion` 虽然已经在 CLI 专项回归里补完整，但如果外层合同文件不锁，后续仍可能出现“内部切片是对的，工具层返回却被改漏”的问题。
### 本轮决策
- 不改生产代码，只在 `integration_tool_contract.rs` 增加最小外层回归，覆盖 `bullish_continuation` 与 `range_wait` 两个边界明确、语义差异大的样本。
- 继续走真实 `CSV -> import_stock_price_history -> technical_consultation_basic` 链路，避免外层合同测试退化成手工拼接 JSON。
### 原因
- 这轮目标是验证“上层调用方真正看到的合同”，不是重复内部分类逻辑；因此最合适的是在工具合同文件直接跑真实链路。
- 选 `bullish_continuation + range_wait` 两端样本，可以最小代价覆盖“方向性延续”和“中性等待”两种最关键的外层消费路径。
### 风险提示
- 外层合同目前还没锁 `confirmed_resistance_retest_hold / confirmed_support_retest_reject` 这类 continuation 子分支的细粒度风险文案。
- 当前仍缺面向调用方的正式文档说明；测试虽然能兜底回归，但不能替代公开合同说明。
## 2026-04-01（方案 A-4 延续态与等待态证据矩阵）
### 关键发现
- `bullish_continuation / bearish_continuation` 原先虽然已有 `headline` 和通用 breakout 理由，但 `rationale` 里没有显式告诉上层“当前已经进入延续剧本，接下来最该盯的是突破后回踩 / 跌破后反抽”。
- `range_wait` 原先只有通用区间描述，缺少“趋势强度不足，所以当前不适合抢方向”的正式合同表达；这会让上层调用方还得自己把 `sideways + weak` 再翻译一遍。
### 本轮决策
- 不动 `classify_consultation_bias()`、`classify_consultation_confidence()` 和 breakout 主分类，只增强 `build_consultation_rationale()` 与 `build_consultation_risk_flags()`。
- 用现有 CLI 测试样本补红测，而不是新造入口，确保这轮收口继续沿着 `technical_consultation_basic` 主链完成。
### 原因
- 方案 A 这轮要收的是“组合结论证据层”，不是再发明新标签；因此最稳的路径就是把已有 `bias` 的解释与风险提示补完整。
- 如果延续态和等待态仍停留在 `headline` 或通用 breakout 描述，后续 Skill / GUI / 其他 AI 仍要自己推断“现在为什么该跟、为什么该等”。
### 风险提示
- 当前外层 `integration_tool_contract` 还没补跑，`consultation_conclusion` 的最终合同目前仍主要锁在 CLI 回归。
- Windows 环境下 `target` 产物较大时仍可能再次触发页面文件不足；这属于环境级风险，不是本轮逻辑回归失败。
## 2026-04-01（方案 A-3 陷阱态证据矩阵）
### 关键发现
- `bull_trap_risk / bear_trap_risk` 原先已经能表达“假突破/假跌破已形成”，但 `rationale / risk_flags` 里没有显式告诉上层“原有突破/跌破延续判断已经失效”。
- 这会让上层调用方虽然知道当前是 trap risk，却还要自己再推断“是不是该从 continuation 剧本里退出”。
### 本轮决策
- 不动 `classify_consultation_bias()`、`classify_consultation_confidence()` 和 breakout 主分类，只增强 `build_consultation_rationale()` 与 `build_consultation_risk_flags()`。
- 为双边 trap risk 增加“原延续判断失效”的专属解释与风险标记。
### 原因
- 方案 A 当前目标是把 `consultation_conclusion` 做成真正可执行的证券分析合同，陷阱态不仅要说明“发生了失败结构”，还要说明“原方向剧本应当暂停”。
- 如果 trap risk 只停留在“假突破/假跌破”事实层，上层策略依然要自己重做一次“是否还按 continuation 理解”的判断。
### 风险提示
- 当前 trap risk 增强主要覆盖“延续判断失效”这一层，若后续要接更强的策略控制，还可以继续补“修复扩散风险/反向加速风险”等更细粒度断言。
- 本轮仍未补外层 `integration_tool_contract`，trap risk 的证据层目前只在 CLI 主回归中锁住。

## 2026-04-01（方案 A-2 组合结论证据矩阵）
### 关键发现
- `bullish_range_watch / bearish_range_watch` 原先只有 `headline` 能表达“等待哪一侧关键位确认”，`rationale` 仍停留在通用 `range_bound` 兜底语句，证据层对上层调用方不够直接。
- `bullish_confirmation_watch / bearish_confirmation_watch` 原先虽然有“观察阶段”的 `headline / rationale`，但 `risk_flags` 没有显式告诉上层“当前最该防的是确认还没完成”。
### 本轮决策
- 不动 `classify_consultation_bias()`、`classify_consultation_confidence()` 和 breakout 主分类，只增强 `build_consultation_rationale()` 与 `build_consultation_risk_flags()`。
- 为双边 `range_watch` 增加方向化理由，为双边 `confirmation_watch` 与 `range_watch` 增加“尚待确认”类结构化风险标记。
### 原因
- 方案 A 当前目标是把 `consultation_conclusion` 做成可直接消费的证券分析合同，而不是让上层继续从 breakout/headline 二次猜测当前缺的到底是哪一步确认。
- 如果证据层仍停留在通用兜底描述，后续 Skill / GUI / 其他 AI 即使拿到 `bias`，也还要自己重组一遍“当前在等什么”的理由和风险。
### 风险提示
- 当前新增的风险标记主要覆盖 `range_watch / confirmation_watch`，`continuation / trap_risk / range_wait` 仍然可以继续补更细粒度的矩阵断言。
- 本轮仍未补外层 `integration_tool_contract`，组合结论证据层目前只在 CLI 主回归中锁住。

## 2026-04-01（方案 A-1 偏多区间观察态闭环）
### 关键发现
- `build_bullish_range_watch_rows(220)` 原来的最近 20 根尾盘是“缓慢抬高”的单边结构，这会在最近 4 根内持续制造新的 breakout anchor，导致样本被 `classify_retest_watch_signal()` 误识别为 `resistance_retest_watch`。
- 这次失败不是 `consultation_conclusion` 主逻辑回退，而是测试夹具和“多根回踩观察态”识别规则发生了语义重叠。
### 本轮决策
- 不调整 `technical_consultation_basic` 的生产逻辑，只修改 `build_bullish_range_watch_rows()` 的尾部样本形态。
- 把尾部改成“先冲高、后高位箱体”，让最近 4 根不再构成新的 breakout anchor，同时仍保留长期偏多、近期等待上破的真实结构。
### 原因
- 方案 A 当前目标是补齐 `consultation_conclusion` 的样本矩阵，而不是为了让测试过掉去稀释既有关键位识别规则。
- 如果把这种失败直接归因到主逻辑并去改分类函数，反而会把已经稳定的 `resistance_retest_watch` 识别边界改坏。
### 风险提示
- 其他“区间观察态”夹具如果尾部继续采用单调抬升/下压的写法，后续仍可能再次误触发 `retest_watch` 多根锚点。
- 当前只锁住了 `bias / confidence / headline` 的最小合同，`rationale / risk_flags` 仍值得继续补专项断言。

## 2026-04-01（方案 A 组合结论收口）
### 关键发现
- `consultation_conclusion` 的第一版 `confidence` 评分如果只做线性加减，会把“价格结构已 confirmed，但量能偏弱”的样本直接压成 `low`，这与 `breakout_signal` 已经明确确认结构完成的语义冲突。
- `build_confirmed_breakout_rows(220)` 这组既有回归样本本身就同时满足两件事：`breakout_signal = confirmed_resistance_breakout`，但 `volume_confirmation = weakening`；这正好说明价格结构与量能维度必须继续并列表达，不能互相覆盖。
### 本轮决策
- 不重写整套 `confidence` 评分权重，而是在 `classify_consultation_confidence()` 内给 `confirmed_* / failed_*` 这类已完成结构增加最低 `medium` 置信度地板。
- 保持 `high / medium / low` 的上限判断仍由趋势强度、量能、动量、RSRS 和背离共同决定，只修复“结构已完成却被误杀成 low”的下限问题。
### 原因
- 这条主线的目标是输出更稳定的证券分析合同，而不是把结构确认重新退化成“只有量能确认才算可信”。
- 如果 confirmed 结构能被单个辅助维度直接打回 `low`，上层 Skill / GUI 会被迫重新解读 `breakout_signal`，等于破坏本轮方案 A 想建立的组合结论层。
### 风险提示
- 当前只锁住了多头 confirmed 样本的 `confidence` 边界；空头 confirmed、trap risk 和区间态虽然现有回归未报错，但仍建议继续补专项断言，避免后续评分规则继续漂移。

## 2026-03-31（多根回踩补充）
### 关键发现
- 仅依赖“前一根突破/跌破、当前一根确认”的两根 K 线口径，会把真实存在的 2~4 根回踩磨位样本误判为 `range_bound`。
- 多根结构不需要新字段；只要保留旧关键位锚点，并确保中间 bars 没有明显走坏，就可以继续落到既有 `confirmed_* / failed_* / watch` 语义。

### 本轮决策
- 在最近 `4` 根内扫描更早的突破/跌破锚点，复用该锚点对应的旧关键位作为后续多根结构的判断基准。
- 中间 bars 继续使用统一灰区约束：多头侧不允许明显跌破 `level - buffer`，空头侧不允许明显上破 `level + buffer`。
- 保持对外合同不变，仍由 `breakout_signal` 表达结构结论，避免扩字段带来上层适配成本。

### 原因
- 这条主线目标是把日线关键位结构做成稳定可消费的咨询合同，而不是做高频执行模型。
- 多根回踩/反抽是日线样本里的常见节奏，如果不补这层，`summary / recommended_actions / watch_points` 会在真实样本里频繁退化成区间态。

### 风险提示
- 当前回看窗口固定为 `4` 根，若后续样本存在更长时间的横向磨位，仍可能需要继续扩窗口或参数化。
- 当前多根结构仍依赖收盘价关键位口径，若未来改成影线极值口径，现有样本与文案都要一起调整。


## 2026-03-31（补充）
### 关键发现
- 如果价格只是刚回到旧关键位附近，直接判成 `confirmed_resistance_retest_hold / confirmed_support_retest_reject` 或 `failed_*`，会把“正在测试关键位”的样本过早定性。
- `retest_watch` 需要一个和价格水平相关的灰区口径，否则低波动品种与高波动品种会被同一条硬阈值误伤。

### 本轮决策
- 为旧关键位附近引入灰区 buffer：`max(snapshot.atr_14 * 0.25, 0.15)`。
- 当价格仍停留在旧关键位灰区附近时，分别输出 `resistance_retest_watch` 或 `support_retest_watch`，表示回踩/反抽仍在观察中。
- `confirmed_resistance_retest_hold / confirmed_support_retest_reject` 只有在价格重新站稳或压回，并且相对旧关键位拉开明显距离后才触发。
- `failed_resistance_breakout / failed_support_breakdown` 也同步收紧为“反向离开旧关键位灰区并拉开距离”后才触发。

### 原因
- 当前主线是日线级 `technical_consultation_basic` 结构化咨询，不应该把单根贴边波动直接翻译成完成态。
- 灰区 buffer 能让“观察态”和“确认态/失败态”之间有稳定的合同边界，减少 summary、actions 和 watch_points 的语义跳变。

### 风险提示
- 当前 buffer 仍是固定公式，若后续覆盖极低价股或超高价股，可能还需要进一步参数化或做比例/绝对值混合校准。
- 当前模型依然是“最近两根 K 线 + 旧关键位”最小口径，多根回踩/多根反抽还没有纳入识别。

## 2026-03-31
### 关键发现
- `build_confirmed_breakout_rows(220)` 对应的真实样本输出中：
- `trend_bias = bullish`
- `trend_strength = strong`
- `volume_confirmation = weakening`
- 这说明“关键位有效突破”不应和“放量确认”强绑为同一字段，否则会与现有 `volume_confirmation` 语义重叠并互相冲突。

### 本轮决策
- `breakout_signal` 采用价格结构口径：
- `confirmed_resistance_breakout`
- `confirmed_support_breakdown`
- `resistance_breakout_watch`
- `support_breakdown_watch`
- `confirmed_resistance_retest_hold`
- `confirmed_support_retest_reject`
- `failed_resistance_breakout`
- `failed_support_breakdown`
- `range_bound`
- `support_level_20 / resistance_level_20` 采用“排除最新一根 K 线后的近 20 日收盘极值”，而不是影线高低点。
- “假突破 / 假跌破”不靠当前快照单点判断，而是使用“前一根相对前序关键位的位置 + 当前一根是否收回当前关键位”的二阶段口径。
- “回踩确认 / 反抽受压”不要求当前一根继续创新高/创新低，而是使用“前一根已越过旧关键位 + 当前一根仍站在旧关键位正确一侧，但已回到当前窗口内”的三阶段口径。

### 原因
- 当前主线是 `technical_consultation_basic` 的结构化咨询，不是高频交易执行系统。
- 用收盘关键位比用影线关键位更稳，更贴合“有效突破/有效跌破”的日线咨询口径。
- 量能已经有独立字段，拆成双维度更利于上层 AI 组合解释。

### 风险提示
- `breakout_signal` 已补到“回踩确认 / 反抽受压”，但还没有继续细分“retest_watch”观察态。
- 当前窗口固定为 `20` 日，适合这条主线的最小版本，但不一定适合所有证券或周期。
