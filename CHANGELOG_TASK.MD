## 2026-03-27
### 修改内容
- `tests/test_disclosure_store.py`：新增披露数据契约与 SQLite 存储的失败测试，覆盖代码标准化、去重键稳定性、事件与快照回读，目的是真正按先测后改推进 M3-1。
- `tradingagents/dataflows/disclosure_models.py`：新增 `DisclosureAttachment`、`SourceSnapshot`、`DisclosureEvent` 以及 `normalize_disclosure_ticker`、`build_disclosure_dedupe_key`，目的是真正建立 A 股/港股公告的统一内部契约。
- `tradingagents/dataflows/disclosure_store.py`：新增基于标准库 `sqlite3` 的 `DisclosureStore`，支持建表、幂等写入、主键读取，目的是真正把披露事件和快照元数据持久化到本地 SQLite。
- `tradingagents/dataflows/disclosure_store.py`：追加显式关闭 SQLite 连接的修复，目的是真正解决 Windows 下临时数据库文件被锁定、测试目录无法清理的问题。
- `task_plan.md`、`findings.md`、`progress.md`：更新为本次 M3-1 会话记录，目的是真正把目标、发现、验证和环境问题沉淀下来。

### 修改原因
- M3 已确定先做 A 股/港股免费官方披露数据层，当前仓库还没有公告/财报/公司事件的统一契约。
- 用户明确要求抓数据不能用大模型、不能用 Token，因此需要先建立独立于 LLM 的抓取落盘基础设施。
- 方案 2 已批准，需要在首轮同时具备统一数据模型和本地 SQLite 持久化能力。
- 发现 `sqlite3.Connection` 的上下文管理不会自动关闭连接，必须修复，否则 Windows 环境下回放测试不稳定。

### 方案还差什么?
- [ ] 还未实现巨潮资讯列表抓取与规范化。
- [ ] 还未实现上交所公告抓取与沪市交叉校验。
- [ ] 还未实现 HKEXnews 抓取与港股公告规范化。
- [ ] 还未把披露数据层接入现有 `tradingagents.dataflows.interface` 路由。
- [ ] 还未加入真实快照文件落盘与离线回放样本集。

### 潜在问题
- [ ] 当前 `dedupe_key` 只按“代码 + 标题 + 小时桶 + 文件名”保守合并，后续跨源样本增多后可能需要调优。
- [ ] SQLite 当前采用 JSON 文本列保存附件和快照元数据，若后续查询维度变多，可能需要拆表。
- [ ] 现有仓库部分测试在本机环境会经由 `langchain_openai -> transformers -> torch` 触发 DLL 初始化失败，本轮未修该环境问题。

### 关闭项
- 已通过 `python -m pytest tests/test_disclosure_store.py`。
- 已通过 `python -m compileall tradingagents/dataflows/disclosure_models.py tradingagents/dataflows/disclosure_store.py`。
- 已复现并修复 Windows 下 SQLite 临时文件句柄未释放问题。
- 已执行 `python -m pytest tests/test_ticker_symbol_handling.py`，结果为环境侧 `torch c10.dll` 初始化失败，非本次披露数据改动直接导致。

### 记忆点（可选）
- 用户要求全过程中文响应，且中文内容需按 UTF-8 处理。
- 编码前必须先给多个方案并等批准；本次已按批准的方案 2 实施。
- 遇到 Bug 先写能复现的测试，再修到通过；本次 SQLite 文件锁问题已按此规则处理。
- 每次改代码后都要补 `CHANGELOG_TASK.MD` 风格日志，不能省略。

## 2026-03-27
### 修改内容
- `tests/test_cninfo_disclosure.py`：新增巨潮证券定位、快照落盘、详情映射的失败测试，目的是真正把 M3-2 的在线抓取入口按先测后改落下来。
- `tradingagents/dataflows/disclosure_cninfo.py`：新增 `CninfoSecurityRef`、`CninfoDisclosureClient`、快照写入、公告详情映射、在线抓取并入库流程，目的是真正打通巨潮资讯 `列表抓取 + 详情抓取 + 附件链接提取 + SQLite 落盘`。
- `tests/test_disclosure_store.py`：追加北交所 `.BJ` 规范化失败测试，目的是真正防止巨潮板块映射扩展后 ticker 规则漏分支。
- `tradingagents/dataflows/disclosure_models.py`：追加 `CN-BJ/BSE/.BJ` 规范化支持，目的是真正补齐 A 股公告统一代码规则。
- `tradingagents/dataflows/data_cache/disclosure/disclosures.sqlite3`：执行了一次真实巨潮抓取并写入样本数据。
- `tradingagents/dataflows/data_cache/disclosure_snapshots/cninfo/...`：写入 lookup/list/detail 原始 JSON 快照，目的是真正支持后续离线回放和结构漂移排查。

### 修改原因
- 用户已批准“方案 2 先抓”，需要在保留基础设施的同时，尽快把第一条真实 A 股抓取链路跑通。
- 巨潮当前页面虽然是前端渲染，但其 JS 仍暴露稳定 JSON 接口，适合第一版正式接入。
- 北交所代码在巨潮板块映射里属于真实输入面，如果不补 `.BJ`，后续 A 股抓取会留下隐藏故障点。

### 方案还差什么?
- [ ] 还未实现巨潮按关键词/日期的全市场公告搜索抓取，目前先支持单 ticker 历史公告分页。
- [ ] 还未把巨潮事件抓取接入现有 `tradingagents.dataflows.interface`。
- [ ] 还未加入 PDF 正文抽取；当前首版先保存附件链接和详情元数据。
- [ ] 还未实现上交所交叉校验与港股 HKEXnews 抓取。

### 潜在问题
- [ ] 巨潮 `hisAnnouncement/query` 当前更适合“单证券历史分页”，如果后面要做全市场日期窗口抓取，可能还要补另一条搜索接口。
- [ ] 当前 `ingest_stock_disclosures()` 采用串行抓详情，请求量大时可能需要限流、重试和增量断点。
- [ ] `announcementContent` 在 PDF 公告场景通常为空，后续如果要做正文分析，需要补 PDF 文本抽取或 OCR 策略。

### 关闭项
- 已通过 `python -m pytest tests/test_disclosure_store.py tests/test_cninfo_disclosure.py`。
- 已通过 `python -m compileall tradingagents/dataflows/disclosure_models.py tradingagents/dataflows/disclosure_cninfo.py tradingagents/dataflows/disclosure_store.py`。
- 已执行真实抓取：`600519` 在 `2026-03-01` 到 `2026-03-27` 窗口成功入库 2 条公告。
- 已确认 SQLite 中存在 `cninfo-1225009431`、`cninfo-1224994303` 两条事件记录。

### 记忆点（可选）
- 用户当前优先级是“先把基础打好再真实抓数”，因此先做单源可验证链路，再补双源校验。
- 用户明确要求抓数链路不能用大模型、不能用 Token；本次巨潮抓取全程只用了普通 HTTP + JSON + SQLite。

## 2026-03-27
### 修改内容
- `tests/test_sse_disclosure_verifier.py`：新增上交所 JSONP 解析、公告行标准化、与巨潮已入库事件比对的失败测试，目的是按先测后改把 M3-3 的沪市校验源钉住。
- `tradingagents/dataflows/disclosure_sse_verifier.py`：新增 `SseAnnouncementVerifier`、`parse_sse_jsonp_payload`、`normalize_sse_bulletin`、`compare_sse_bulletins_with_store`，目的是真正打通“上交所官方列表 -> 规范化 -> 与巨潮事件比对”的第一条校验链路。
- `tradingagents/dataflows/disclosure_store.py`：新增 `list_events_by_ticker_and_date_range()`，目的是真正给上交所校验提供按代码和日期窗口取巨潮事件的能力。
- 执行了一次真实上交所校验：以 `600519` 和 `2026-03-01` 到 `2026-03-27` 时间窗比对上交所官方公告与巨潮已入库公告。

### 修改原因
- 用户已批准“方案 1”，本轮目标是先把上交所作为校验源接进来，而不是立即做双源正式入库。
- 上交所最新公告页背后已有稳定官方接口，适合先做低风险、可核对的交叉校验。
- 巨潮已经有真实入库结果，现在最有价值的是验证沪市公告是否存在漏抓或标题偏差。

### 方案还差什么?
- [ ] 还未把上交所原始响应做快照落盘。
- [ ] 还未把上交所校验结果固化成可复跑的报告文件或命令入口。
- [ ] 还未实现上交所结果升级为正式 `DisclosureEvent` 并入 SQLite。
- [ ] 还未实现港股 HKEXnews 抓取与校验。

### 潜在问题
- [ ] 上交所接口当前返回 JSONP，若官方改成 JSON 或调整回调包装，解析器需要同步更新。
- [ ] 当前匹配规则按“代码 + 日期 + 标题”保守对齐，若后续出现同日同题多版本公告，可能需要加入 URL 或类型辅助匹配。
- [ ] 目前只覆盖沪市 `stockType=1` 主板口径，若后续校验科创板或其他板块，需要补参数映射和样本测试。

### 关闭项
- 已通过 `python -m pytest tests/test_sse_disclosure_verifier.py`。
- 已通过 `python -m pytest tests/test_disclosure_store.py tests/test_cninfo_disclosure.py tests/test_sse_disclosure_verifier.py`。
- 已通过 `python -m compileall tradingagents/dataflows/disclosure_sse_verifier.py tradingagents/dataflows/disclosure_store.py`。
- 已执行真实校验：`600519` 在 `2026-03-01` 到 `2026-03-27` 窗口内，上交所 2 条公告与巨潮 2 条入库事件全部匹配，`SSE_ONLY=0`，`CNINFO_ONLY=0`。

### 记忆点（可选）
- 用户当前接受“先做上交所校验源，不做正式双源入库”的推进方式，因此这轮以验证价值优先，不扩范围。

## 2026-03-27
### 修改内容
- `tests/test_disclosure_runner.py`：新增 M3-4 统一入口失败测试，覆盖运行时路径固定、`run_summary.json` 落盘、CLI `run` 子命令输出摘要，目的是先把未来免环境打包需要的入口行为钉死。
- `tradingagents/disclosure_runner.py`：补齐统一编排入口，集中负责运行时目录构建、CNInfo 入库调用、SSE 校验调用、摘要对象组装与 `run_summary.json` 输出，目的是把抓取细节和未来交付入口隔离开。
- `cli/disclosure.py`：追加根回调并保留 `run` 子命令形态，目的是修复 Typer 在单命令场景下把 `run` 吞成默认命令、导致统一入口调用不稳定的问题。
- `task_plan.md`、`progress.md`、`findings.md`：追加 M3-4 完成记录与 CLI 折叠问题结论，目的是把这次统一入口收敛的事实沉淀下来。

### 修改原因
- 用户已批准按方案 1 推进 M3-4，本轮目标是先把“统一入口、固定输出、便于后续免环境打包”的编排层做稳，而不是直接扩到打包器。
- 现有失败测试已经复现 CLI 问题：调用 `disclosure run ...` 时被 Typer 判定为多余参数；如果不先修这个入口问题，后续打包后的普通用户调用方式会不稳定。
- 需要把 SQLite 路径、快照路径、摘要报告路径收敛到单一入口，避免后面增加桌面版、便携版或 exe 包装时修改抓取器内部实现。

### 方案还差什么?
- [ ] 还未增加真正面向普通用户的一键启动包装层，目前仍然是 Python CLI 入口先收敛。
- [ ] 还未把 SSE 原始响应快照和校验报告拆成独立文件产物，当前先固化统一摘要报告。
- [ ] 还未开始港股 HKEXnews 免费官方披露接入。

### 潜在问题
- [ ] 当前统一入口在校验阶段默认按 `CN-SH` 规范化 ticker，更适合上交所样本；后续扩到深市或港股时，需要补市场参数或自动判别逻辑。
- [ ] `run_summary.json` 当前只输出聚合计数；如果后续需要审计级问题定位，可能还要补充 `sse_only` / `cninfo_only` 明细文件。
- [ ] 统一入口已为未来免环境打包做了边界收敛，但真正“无需 Python 环境”的交付仍需要后续选定 PyInstaller/Nuitka 等方案并做一次打包验证。

### 关闭项
- 已通过 `python -m pytest tests/test_disclosure_runner.py`。
- 已通过 `python -m pytest tests/test_disclosure_store.py tests/test_cninfo_disclosure.py tests/test_sse_disclosure_verifier.py tests/test_disclosure_runner.py`。
- 已复现并修复 `cli.disclosure` 在单命令 Typer 应用下把 `run` 视为多余参数的问题。

### 记忆点（可选）
- 用户要求“先给方案、批准后再改代码”，本轮继续按已批准的方案 1 执行，不擅自扩范围。
- 用户希望最终交付降低门槛，不要求普通用户安装完整 Python 环境；因此实现上要优先收敛统一入口和固定产物路径。

## 2026-04-09
### 修改内容
- `src/ops/foundation/ontology_schema.rs`：为 `OntologyConcept` 增加 `tags` 与 `with_tag()`，并把 lookup 索引升级为“一个 lookup key 对应多个 concept 候选”，目的是让 foundation 层开始支持同词多概念而不是强制全局唯一。
- `src/ops/foundation/ontology_store.rs`：新增 `find_concept_ids()`，目的是让 router 可以通过 store 读取 schema 的多候选结果，而不复制底层索引逻辑。
- `src/ops/foundation/capability_router.rs`：为 `NavigationRequest` 增加 `required_concept_tags` 与 `with_required_concept_tags()`，并在 phrase-first 路由流程中加入候选标签过滤，目的是给 foundation 路由补齐最小 scope/metadata 约束能力。
- `tests/ontology_schema_unit.rs`、`tests/capability_router_unit.rs`：先补“重复 lookup key 多候选”和“按标签消歧路由”的红测，再补最小实现转绿，目的是按 TDD 锁住这轮 foundation 路由升级。

### 修改原因
- 用户确认继续按方案A推进 foundation 路由主线，本轮重点是补“约束下的路由”，而不是继续扩业务域或 UI。
- 现有 schema 把 lookup key 视为全局唯一，无法表达“同一个短语在不同语义域下对应不同概念”的基础事实。
- 需要让 route 在不引入复杂 DSL 的前提下，先具备一个最小、稳定、可回归的 scope 约束入口。

### 方案还差什么?
- [ ] 还未把标签约束继续扩到 roaming plan 或 navigation pipeline；当前只先落在 router 输入层。
- [ ] 还未补更丰富的 metadata 约束，例如 source/time/type 等；本轮只做 concept tag 这一种最小约束。
- [ ] 还未补 pipeline 级别的“同词不同域”集成测试；当前先把 schema/store/router 三层打稳。

### 潜在问题
- [ ] 当前同一 lookup key 的多个概念候选默认按 schema 声明顺序返回；如果未来需要更复杂的优先级，可能还要补显式排序规则。
- [ ] 当前标签过滤只支持“任一标签命中即可”，还不支持 must-match-all 或更复杂布尔逻辑。
- [ ] 这轮仍然是轻量标签约束，不包含真正的上下文推理或语义消歧，复杂问题仍可能需要后续 roaming/retrieval 配合。

### 关闭项?
- 已通过 `cargo test --test ontology_schema_unit -- --nocapture`。
- 已通过 `cargo test --test capability_router_unit -- --nocapture`。
- 已通过 `cargo test --test ontology_store_unit -- --nocapture`。
- 已按 TDD 先新增“多候选 lookup”和“按标签消歧”测试，再补实现转绿。

### 记忆点（可选）
- 用户当前认可的推进方式是“foundation 主线逐层增强”，因此遇到同词消歧问题时，应优先在通用路由约束层补能力，而不是先在某个业务适配层做特判。

## 2026-04-09
### 修改内容
- `src/ops/foundation/capability_router.rs`：为 `CapabilityRoute` 增加 `matched_terms`，并把内部匹配流程改为同时保留“命中概念 id + 命中短语”，目的是让 foundation 路由结果具备最小可解释性，后续 trace/调试不必反推命中原因。
- `src/ops/foundation/ontology_schema.rs`：把 lookup 归一化从“仅 trim + lowercase”升级为“标点/连字符/下划线折叠为空格后统一归一化”，目的是让 `gross-margin`、`gross_margin`、`gross margin` 这类文本格式差异不再阻断同一概念定位。
- `tests/capability_router_unit.rs`：先补红测，再验证 `matched_terms` 回执和标点归一化行为，目的是按 TDD 把 foundation 路由这轮最小增量锁进可回归测试。

### 修改原因
- 用户确认继续按方案A推进 foundation 路由主线，而不是继续扩业务域能力。
- 当前 capability router 只能返回 concept id，缺少最小解释信息，后续做导航 trace 或交接时仍然容易反复追问“到底命中了哪段文本”。
- 当前 ontology lookup 规则对格式差异不够稳健，通用知识源里的连字符、下划线、空格变体会导致本应等价的短语匹配失败。

### 方案还差什么?
- [ ] 还未把 `matched_terms` 进一步上提到更完整的 navigation pipeline trace；当前只先补到 router 结果层。
- [ ] 还未引入 metadata filter、capability constraints 或更强 query normalization；这轮仅收口基础短语路由与 lookup 归一化。
- [ ] 还未补更高层的 pipeline 集成回归；当前先保证 router 与 schema 两层行为稳定。

### 潜在问题
- [ ] 当前归一化会把大部分非字母数字字符都折叠为空格；若未来需要保留某些特殊符号的区分语义，可能需要引入更细粒度规则。
- [ ] `matched_terms` 当前记录的是路由时命中的归一化短语，而不是原始问题片段；如果后续要用于面向终端用户展示，可能还需要补原文 span 信息。
- [ ] 这轮仍是规则型 route，不包含同义词扩展、上下文消歧和 metadata 约束，复杂问题仍可能命中不足。

### 关闭项?
- 已通过 `cargo test --test capability_router_unit -- --nocapture`。
- 已通过 `cargo test --test ontology_schema_unit -- --nocapture`。
- 已按 TDD 先新增 `router_normalizes_phrase_punctuation_variants` 红测，再补实现转绿。

### 记忆点（可选）
- 用户当前主线是“继续做通用标准能力的 foundation 路由”，不是业务域流程推进；后续若继续往下做，应优先补 route / roam / retrieve / evidence 这条通用内核。

## 2026-04-09
### 修改内容
- `AI_START_HERE.md`：补充“`security_*` / `stock_*` / 审批链 / scorecard / training 仅属于业务适配层，不代表当前项目主线”的硬边界说明，目的是让新 AI 在最短入口就先按通用能力主线理解仓库。
- `docs/ai-memory/project-baseline.md`：新增 mainline directory rule，并把 `security/*`、`stock/*`、approval / committee / scorecard 明确归类为 adapter-side work，目的是把“通用标准能力主线”写成可复用的项目备忘。
- `docs/ai-handoff/AI_HANDOFF_MANUAL.md`：新增 mainline directory guardrails，并强化 `security_*` 不属于 foundation mainline 的交接规则，目的是减少未来 AI 再次把业务域进展误读为项目主线的时间成本。

### 修改原因
- 用户明确要求“不做业务化能力，只做通用化的标准能力”，并要求把这条边界写进交接手册和项目备忘，避免后续会话重复花时间重新校准主线。
- 当前仓库与 `codex/foundation-navigation-kernel` 分支里同时存在 foundation 与 security 相关内容，如果没有硬性文字约束，新的 AI 很容易把并行证券轨道误判成主线方向。
- 需要把“foundation-first，业务域仅为 adapter-side track”的口径同时固化到入口文档、项目备忘和 handoff 手册三处，形成一致入口。

### 方案还差什么?
- [ ] 还未把同样的目录边界规则进一步下沉到专门的 architecture 文档；当前先在入口、备忘、交接手册三处收口。
- [ ] 还未对 `security_*` 目录做真实迁移或重命名；本轮只固化文档边界，不改目录结构。
- [ ] 还未把这组文档改动提交或推送到远端分支。

### 潜在问题
- [ ] 即使文档边界已经写清，只要仓库目录仍混放大量 `security_*` 文件，新 AI 仍可能先被文件名吸引而误判主线，需要后续配合目录治理进一步收敛。
- [ ] 当前 `AI_HANDOFF_MANUAL.md` 仍保留 security parallel track 状态说明；若未来该轨道继续扩张但不同步更新分类说明，仍可能带来“主线/并行线”混读风险。
- [ ] 本轮为文档治理修改，无法直接阻止错误开发，只能降低后续交接和理解成本。

### 关闭项?
- 已完成入口文档、项目备忘、交接手册三处口径统一。
- 已通过 `git diff -- AI_START_HERE.md docs/ai-memory/project-baseline.md docs/ai-handoff/AI_HANDOFF_MANUAL.md` 人工核对改动内容与边界表述一致。
- [ ] 未执行自动化测试：本轮仅修改文档与交接约束，未变更运行时代码。

### 记忆点（可选）
- 用户再次纠正“不要做业务化能力，只做通用化标准能力”；以后看到 `security_*` 活跃改动时，必须默认判定为业务适配层或并行领域轨道，而不是项目主线。

## 2026-03-27
### 修改内容
- `tests/test_disclosure_runner.py`：追加 M3-5 市场路由失败测试，覆盖 `600519 -> SSE`、`000001 -> SZSE`、`430047 -> BSE` 的路由判定，以及非沪市样本在 `verify_sse=True` 时不得误调用上交所校验器，目的是先把 runner 的市场分流行为钉死。
- `tradingagents/disclosure_runner.py`：新增 `DisclosureMarketRoute`、`resolve_disclosure_market_route()`、`_build_market_route()`，并让 `DisclosureRunSummary` 输出 `market_key`、`exchange`、`verification_source`，目的是移除统一入口对 `CN-SH` 的硬编码假设。
- `tradingagents/disclosure_runner.py`：调整 `run_disclosure_pipeline()`，仅在路由到 `SSE` 时才启用 `SseAnnouncementVerifier`，其余市场先保持“可抓取、可落盘、校验跳过”的稳定行为，避免深市/北交所样本误走上交所校验逻辑。
- `task_plan.md`、`progress.md`、`findings.md`：追加 M3-5 完成记录与新发现，目的是让后续 AI 能直接看出“市场感知路由已完成，但非沪市校验器尚未接入”。

### 修改原因
- 用户批准按方案 2 推进，当前目标是先修正架构层面的市场路由问题，而不是继续扩新数据源。
- 现有 runner 在校验阶段把所有 ticker 都按 `CN-SH` 规范化，这会让深市和北交所代码得到错误的规范化结果，并存在误用 SSE 校验器的风险。
- 在深市/北交所官方校验链路尚未实现前，最稳妥的策略是先把“识别对、只在支持的市场启用校验器”落实到统一入口。

### 方案还差什么?
- [ ] 还未实现深交所官方披露校验器，因此 `CN-SZ` 路由当前只做抓取与落盘，不做官方交叉校验。
- [ ] 还未实现北交所或港股官方披露校验器，因此 `CN-BJ`、`HK` 仍处于“路由已识别、校验未启用”的阶段。
- [ ] 还未把市场路由信息在 CLI 文本输出中显式打印，当前先写入 `run_summary.json`。

### 潜在问题
- [ ] 当前路由判定优先依赖代码后缀和常见前缀规则，若后续接入更复杂证券类型，可能需要引入源级元数据确认。
- [ ] `verification_source=None` 目前代表“当前无可用官方校验器”，后续若增加多市场校验器，需要补更细的状态字段，避免和“用户主动关闭校验”混淆。
- [ ] 这次修复只解决了 runner 的市场分流问题，并没有新增深市/北交所真实校验覆盖率。

### 关闭项
- 已通过 `python -m pytest tests/test_disclosure_runner.py`。
- 已通过 `python -m pytest tests/test_disclosure_store.py tests/test_cninfo_disclosure.py tests/test_sse_disclosure_verifier.py tests/test_disclosure_runner.py`。
- 已复现并修复“统一 runner 在校验阶段硬编码 `CN-SH` 导致非沪市样本路由错误”的问题。

### 记忆点（可选）
- 用户当前接受“先把架构边界修正，再决定是否扩新源或新校验器”的推进方式，因此这轮只做市场感知路由，不额外扩范围。
- 用户希望后续能继续推送给其他 AI 接续，所以记录文件和 `CHANGELOG_TASK.MD` 必须持续保持可交接状态。

## 2026-03-27
### 修改内容
- `docs/plans/2026-03-27-m3-next-steps.md`：新增独立 roadmap 文档，整理当前 M3 已完成边界、M3-6 之后的优先级、M4 打包方向、测试建议、风险与交接说明，目的是让其他 AI 不需要重新梳理上下文就能继续推进。
- `task_plan.md`：补充 roadmap 文档引用，目的是让主任务计划和独立规划文档形成双向入口。
- `progress.md`：记录“已新增独立 roadmap 供后续接续”的事实，目的是把今天的收口动作显式沉淀下来。
- `findings.md`：补充“披露数据线已进入多阶段子系统，需要独立规划文档”的结论，目的是说明为什么现在值得单独维护 roadmap。

### 修改原因
- 用户明确表示今天先不继续开发，而是要把后续计划和规划一起整理好，并为后续推送和 AI 接续做准备。
- 目前披露数据线已经从单一功能演进成带有多阶段路线图的独立子系统，仅靠开发日志已不足以高效承接后续工作。
- 需要把“下一步该做什么、为什么这样排、哪些东西还没做”从会话语境中抽离成稳定文档。

### 方案还差什么?
- [ ] 还未把 roadmap 内容同步到仓库 README 或专门的 disclosure 文档索引页。
- [ ] 还未完成远端 GitHub 推送；这取决于当前机器账号是否具备仓库写权限。
- [ ] 还未决定下一轮是先做 M3-6 明细校验报告，还是直接开 SZSE 官方校验器。

### 潜在问题
- [ ] roadmap 是基于 2026-03-27 当天状态整理的，如果后续分支继续开发但不更新该文档，交接信息会过时。
- [ ] 当前远端写权限此前已验证存在 `403` 风险，本次推送可能仍然失败。

### 关闭项
- [ ] 未执行测试：本轮为文档与交接收口修改，未改动运行时代码或测试逻辑。
- 已补齐独立 roadmap：`docs/plans/2026-03-27-m3-next-steps.md`。

### 记忆点（可选）
- 用户今天的目标是“先收口、补后续计划、再尝试推 GitHub”，不是继续扩功能。
- 当用户强调“方便其他 AI 接续”时，优先提供独立 roadmap，而不只是在日志文件里零散追加说明。
## 2026-04-09
### 修改内容
- `src/ops/foundation/navigation_pipeline.rs`：新增 foundation 通用导航主线的最小 `NavigationPipeline` 实现，串联 `route -> roam -> retrieve -> assemble`，并统一暴露 `NavigationPipelineError`，目的是补齐“问题到证据”的正式内核入口。
- `src/ops/foundation/navigation_pipeline.rs`：新增基于 `RoamingPlan` 模板种子的路由收敛逻辑，目的是真正让 pipeline 只围绕当前计划允许的主种子概念导航，避免修饰性概念把集成主链带偏。
- `src/ops/foundation.rs`：挂接 `navigation_pipeline` 模块导出，目的是让 integration test 和后续上层调用都通过 foundation 正式入口接入，而不是继续依赖缺失模块。

### 修改原因
- 已存在的 `tests/navigation_pipeline_integration.rs` 先于源码定义了 `NavigationPipeline` 契约，导致当前 foundation 主线在集成层直接编译失败，必须先补最小实现。
- 用户已多次明确当前只推进 foundation 通用标准能力，不做业务化能力；因此这轮应优先补齐 foundation 编排入口，而不是转去扩展业务链路。
- 集成测试对 `show sales trend` 的期望是只围绕 `revenue` 作为导航主种子运行，说明 pipeline 层需要在 router 多概念识别之后再做一次计划约束收敛。

### 方案还差什么?
- [ ] 还未把 `required_concept_tags` 正式打通到 pipeline 级集成测试；当前只先补齐最小编排入口。
- [ ] 还未增加 `RoamFailed` / `RetrievalFailed` 的专门集成测试；当前先覆盖了 happy path 与 route error。
- [ ] 还未把 `matched_terms`、模板种子约束等信息上提为更完整的 trace / telemetry 输出。

### 潜在问题
- [ ] 当前 pipeline 的“模板种子收敛”依赖 `RoamingPlan::seed_concept_ids` 作为允许的主种子集合；如果后续该字段被重新定义为“默认种子”而非“计划边界”，需要同步调整这段约束逻辑。
- [ ] 当前 `NavigationPipelineError` 只保留最小错误语义，没有透传内部细节；如果后续需要更强诊断能力，可能要补 `source error` 或阶段上下文。
- [ ] 当前只验证了内存态 foundation 样本；若后续图谱规模增大或概念冲突变多，需要补更多多概念、多标签、多跳场景测试。

### 关闭项?
- 已先复现并确认红测：`cargo test --test navigation_pipeline_integration -- --nocapture`，失败原因为 `foundation::navigation_pipeline` 模块缺失。
- 已通过 `cargo test --test navigation_pipeline_integration -- --nocapture`。
- 已通过 `cargo test --test capability_router_unit -- --nocapture`。
- 已通过 `cargo test --test ontology_schema_unit -- --nocapture`。
- 已通过 `cargo test --test ontology_store_unit -- --nocapture`。

### 记忆点（可选）
- 用户当前主线是补 foundation 通用导航内核，任何“集成失败”都优先判断是不是 foundation 主线入口缺失，而不是先跳去业务域修补。
## 2026-04-09
### 修改内容
- `tests/navigation_pipeline_integration.rs`：新增 `navigation_pipeline_prefers_tag_constrained_domain_evidence`，构造 `margin` 在 `finance` / `ui` 双域重名的纯内存集成样本，并通过 `with_required_concept_tags(vec!["finance"])` 验证 pipeline 最终命中正确 concept、node 与 citation。

### 修改原因
- 前一轮虽然已经把 `required_concept_tags` 接入了 `NavigationRequest` 与 `CapabilityRouter`，但还缺少 pipeline 级证据证明，无法确认标签约束真的穿过 foundation 主链。
- 用户当前推进的是 foundation 通用标准能力，因此需要把“同词不同域”的约束验证上提到集成层，而不是停留在 router 局部单测。
- 按 TDD 流程，这轮先补红测；实际执行时该测试一次通过，反向证明现有 pipeline 已经具备标签约束闭环能力，因此无需再改生产代码。

### 方案还差什么?
- [ ] 还未补 `required_concept_tags` 与模板种子收敛同时发生冲突时的集成测试，例如“模板只允许某主种子，但标签约束筛出另一组候选”的边界场景。
- [ ] 还未补 `RoamFailed` / `RetrievalFailed` 在带标签约束条件下的失败用例。
- [ ] 还未把标签命中的解释信息上提到更显式的 trace / telemetry 结构中。

### 潜在问题
- [ ] 当前 pipeline 级标签验证依赖纯内存 fixture；如果后续 graph fixture 增加更多共享 alias 节点，可能需要补多跳和多命中排序场景。
- [ ] 当前只验证单标签 `finance`；多标签 OR 语义是否满足未来预期，还没有集成级覆盖。
- [ ] 由于这轮红测直接通过，说明现有行为已满足当前契约；如果后续有人调整模板种子收敛逻辑，可能会无意破坏这个闭环，需要持续保留该测试。

### 关闭项?
- 已通过 `cargo test --test navigation_pipeline_integration -- --nocapture`。
- 已通过 `cargo test --test capability_router_unit -- --nocapture`。
- 已通过 `cargo test --test ontology_schema_unit -- --nocapture`。
- 已通过 `cargo test --test ontology_store_unit -- --nocapture`。
- 已按 TDD 先补 pipeline 级测试，再确认现有实现无需修复。

### 记忆点（可选）
- `required_concept_tags` 现在不只是 router 层能力，已经被 pipeline 集成测试证明能稳定影响 foundation 主链最终证据命中；后续不要重复把它当作“尚未闭环”的未完成项。
## 2026-04-09
### 修改内容
- `tests/navigation_pipeline_integration.rs`：新增 `navigation_pipeline_prefers_request_tags_over_conflicting_template_seeds`，构造“模板只给 `layout_margin`，但请求显式要求 `finance` 标签”的冲突样本，验证 pipeline 最终仍保留 `gross_margin` 路由并命中 finance 侧 evidence。

### 修改原因
- foundation pipeline 现在同时存在 request 级 `required_concept_tags` 与 plan 模板种子收敛两层约束，如果不明确二者冲突时的优先级，后续重构很容易把显式请求 scope 吞掉。
- 用户当前推进的是通用标准能力边界收敛，因此需要把“标签优先于冲突模板默认种子”的语义显式固定在集成测试里，而不是留给实现细节隐式决定。
- 按 TDD 流程先补边界测试；该测试一次通过，说明当前实现已经满足“请求标签优先，模板仅在不冲突时收敛”的预期契约。

### 方案还差什么?
- [ ] 还未补“多标签组合 + 冲突模板种子”的集成测试，例如 `finance + reporting` 组合场景。
- [ ] 还未补带冲突模板种子的 `RoamFailed` / `RetrievalFailed` 边界测试。
- [ ] 还未把这条优先级规则整理进更正式的 foundation 设计文档；当前先用测试和任务日志钉死行为。

### 潜在问题
- [ ] 当前优先级规则主要依赖 `constrain_route_to_template_seeds()` 中“若收敛后为空则回退原 route”的语义；后续若有人重写这段逻辑，可能无意改变契约。
- [ ] 当前只验证单个冲突 alias `margin`；更复杂的多词短语冲突还没有集成级覆盖。
- [ ] 本轮没有新增生产代码，因此若未来出现行为变化，首先应怀疑模板种子收敛逻辑而不是标签过滤逻辑本身。

### 关闭项?
- 已通过 `cargo test --test navigation_pipeline_integration -- --nocapture`。
- 已按 TDD 先补冲突边界测试，再确认现有实现无需修复。

### 记忆点（可选）
- foundation pipeline 当前已被测试明确约束为：`required_concept_tags` 属于更强的请求级 scope，模板种子只在不冲突时做收敛，不能覆盖显式标签约束。
## 2026-04-09
### 修改内容
- `tests/navigation_pipeline_integration.rs`：新增 `navigation_pipeline_surfaces_roaming_error_for_missing_seed_concepts` 与 `navigation_pipeline_surfaces_retrieval_error_when_scope_has_no_matching_evidence`，分别钉死 pipeline 在漫游种子失效和证据检索落空时的统一错误边界。
- `tests/navigation_pipeline_integration.rs`：修正 retrieval miss 样本正文，去掉与查询词 `margin` / `trend` 的意外交集，目的是真正只验证 retrieval 阶段空命中，而不是被样本噪音误伤。
- `src/ops/foundation/navigation_pipeline.rs`：调整 `constrain_route_to_template_seeds()`，在“无显式 `required_concept_tags`”时允许模板默认种子成为主收敛来源；若存在显式标签约束且模板收敛为空，则继续保留请求路由结果。目的是同时满足“标签优先”与“模板默认种子在无显式 scope 时有效”两条通用契约。

### 修改原因
- 新增的失败边界红测暴露出两点：`RetrievalFailed` 样本不够纯，`RoamFailed` 在当前实现下几乎不可达，因为模板默认种子在无显式标签时没有真正参与优先收敛。
- foundation 主线需要稳定表达 `route / roam / retrieve` 三阶段失败边界，否则上层无法基于阶段语义做可解释反馈。
- 用户当前推进的是通用标准能力，所以这轮应补的是“失败边界契约”而不是更复杂的业务策略。

### 方案还差什么?
- [ ] 还未补多标签组合条件下的 `RoamFailed` / `RetrievalFailed` 集成测试。
- [ ] 还未补“模板默认种子有效但图谱多跳后为空”的更复杂漫游失败场景。
- [ ] 还未把新的模板收敛优先级写入正式 foundation 设计文档；当前先由测试和日志约束行为。

### 潜在问题
- [ ] 当前模板默认种子在无显式标签约束时会覆盖路由结果；如果后续希望模板只作为建议而非强约束，需要重新审视这条规则。
- [ ] `RoamFailed` 当前回传的是实际参与漫游的模板种子列表，而不是原始 route concept ids；上层若想同时看到两者，需要后续补更细的诊断字段。
- [ ] 这轮仍主要覆盖单跳、内存态 fixture；真实大图谱上的性能和排序行为还没有被验证。

### 关闭项?
- 已通过 `cargo test --test navigation_pipeline_integration -- --nocapture`。
- 已通过 `cargo test --test capability_router_unit -- --nocapture`。
- 已通过 `cargo test --test ontology_schema_unit -- --nocapture`。
- 已通过 `cargo test --test ontology_store_unit -- --nocapture`。
- 已按 TDD 先补失败边界红测，再做最小生产修复直到转绿。

### 记忆点（可选）
- foundation pipeline 当前规则已被测试固定为：
- 有显式 `required_concept_tags` 时，请求级标签约束优先于模板默认种子。
- 无显式标签约束时，模板默认种子可以作为主收敛来源，并可能触发真实的 `RoamFailed` 边界。
## 2026-04-09
### 修改内容
- `tests/navigation_pipeline_integration.rs`：新增 `navigation_pipeline_prefers_intersection_when_template_and_route_partially_overlap`，构造 `route = revenue + gross_margin`、模板默认种子 = `gross_margin + ghost_seed` 的部分重叠样本，验证 pipeline 最终只保留交集 `gross_margin`。

### 修改原因
- foundation pipeline 前面已经分别钉死了“显式标签优先”和“无显式标签时模板默认种子可生效”，但还缺少“模板与 route 部分重叠”这一中间分支的集成级证明。
- 用户选择了方案C，因此需要把三层优先级完整闭环：标签优先；部分重叠取交集；完全不重叠时模板默认种子生效。
- 按 TDD 先补边界测试；测试直接通过，说明现有实现已经满足方案C，无需再调整生产代码。

### 方案还差什么?
- [ ] 还未补“多标签 + 部分重叠模板”组合场景的集成测试。
- [ ] 还未补部分重叠场景下多跳 roaming path 是否仍然只围绕交集 seeds 扩展的验证。
- [ ] 还未把这组优先级规则沉淀到更正式的 foundation 设计文档；当前先由测试和日志固定。

### 潜在问题
- [ ] 当前“部分重叠取交集”只验证了单一交集 seed；多个交集 seed 的排序和保留顺序还未显式覆盖。
- [ ] 当前样本是纯内存小图；真实图谱里如果节点分数相近，仍可能需要更细的排序测试。
- [ ] 由于这轮未改生产代码，后续若行为变化，优先检查 `constrain_route_to_template_seeds()` 是否被重构破坏。

### 关闭项?
- 已通过 `cargo test --test navigation_pipeline_integration -- --nocapture`。
- 已通过 `cargo test --test capability_router_unit -- --nocapture`。
- 已通过 `cargo test --test ontology_schema_unit -- --nocapture`。
- 已通过 `cargo test --test ontology_store_unit -- --nocapture`。
- 已按 TDD 先补“部分重叠取交集”集成测试，再确认现有实现无需修复。

### 记忆点（可选）
- foundation pipeline 当前规则已被测试完整固定为：显式标签优先；无显式标签且模板与 route 部分重叠时取交集；无交集时模板默认种子生效。
## 2026-04-09
### 修改内容
- `tests/navigation_pipeline_integration.rs`：新增 `navigation_pipeline_keeps_multiple_intersection_seeds_in_route_order`，验证当模板默认种子与 route 存在多个交集时，最终保留的 concept ids 按 route 命中顺序输出。
- `tests/navigation_pipeline_integration.rs`：新增 `navigation_pipeline_roams_only_from_kept_intersection_seeds`，验证多交集场景下的 roaming path 只围绕最终保留的交集 seeds 扩展，不会把模板中未保留的 `ghost_seed` 带入主链。

### 修改原因
- foundation pipeline 之前已经钉死了“单交集取交集”，但还缺少多个交集 seeds 时的顺序与 path 约束，否则后续排序或 roaming 重构容易引入不稳定行为。
- 用户明确要求继续收紧“多个交集 seed 的顺序与保留规则”，因此这轮要把主线最关键的剩余通用规则补成集成测试，而不是继续扩更大范围。
- 按 TDD 先补红测；两条测试直接通过，说明当前实现已经稳定满足“多个交集按 route 顺序保留，roaming 只围绕保留 seeds 扩展”的契约。

### 方案还差什么?
- [ ] 还未补“多个交集 seed + 多标签约束”复合场景测试。
- [ ] 还未补多个交集 seed 同时存在多跳 relation 时的 path 深度与顺序测试。
- [ ] 还未把这组规则整理成正式 foundation 设计文档或 handoff 规则摘要。

### 潜在问题
- [ ] 当前多个交集顺序是通过 route 顺序间接保证的；如果未来 router 的 phrase-first / token fallback 顺序被改，pipeline 输出顺序也会随之变化。
- [ ] 当前 roaming path 测试只覆盖两条一步关系；更复杂图谱中的广度优先顺序还没有专门测试。
- [ ] 这轮没有改生产代码，若后续行为漂移，优先检查 route 顺序和 `constrain_route_to_template_seeds()` 的组合行为。

### 关闭项?
- 已通过 `cargo test --test navigation_pipeline_integration -- --nocapture`。
- 已通过 `cargo test --test capability_router_unit -- --nocapture`。
- 已通过 `cargo test --test ontology_schema_unit -- --nocapture`。
- 已通过 `cargo test --test ontology_store_unit -- --nocapture`。
- 已按 TDD 先补“多交集顺序”和“多交集 roaming path”测试，再确认现有实现无需修复。

### 记忆点（可选）
- foundation pipeline 当前已被测试固定为：多个交集 seeds 按 route 命中顺序保留；roaming 只围绕最终保留的 seeds 扩展，模板中未保留的默认种子不能混入主链。
## 2026-04-09
### 修改内容
- `tests/navigation_pipeline_integration.rs`：新增 `navigation_pipeline_keeps_tag_filtered_intersection_seeds_in_route_order`，补齐“多个交集 seed + 单标签约束”的复合集成测试，目的是确认 `required_concept_tags = ["finance"]` 与模板交集规则叠加后，主链仍只保留 `gross_margin -> revenue` 这组有效 seeds。
- `tests/navigation_pipeline_integration.rs`：新增 `navigation_pipeline_keeps_multi_tag_intersection_seeds_and_evidence`，补齐“多个交集 seed + 多标签约束”的复合集成测试，目的是确认多标签输入下 `route -> roam -> retrieve -> assemble` 仍输出稳定的 concept、path、hits 与 citations。
- `tests/navigation_pipeline_integration.rs`：新增 `sample_tag_filtered_multi_overlap_navigation_pipeline` 与 `sample_multi_tag_filtered_overlap_navigation_pipeline` 两个纯内存 fixture，目的是把“同词异域候选 + 模板默认 seeds + 标签约束 + 多交集顺序”收敛到可重复回归的 foundation 样本里。
- `tests/navigation_pipeline_integration.rs`：修正 `navigation_pipeline_keeps_multi_tag_intersection_seeds_and_evidence` 的命中顺序断言，目的是对齐当前 retrieval 在同分时按 `node_id` 升序排序的通用契约，而不是误把 route 顺序当成 hits 顺序。

### 修改原因
- 当前 foundation 主线虽然已经分别覆盖了“多交集 seed 保留规则”和“标签约束路由规则”，但还缺少二者叠加后的集成级闭环，继续推进前必须把复合边界钉死。
- 用户已明确当前只做通用化标准能力，不做业务化能力，因此这一轮应继续补 foundation 主线的通用边界，而不是扩展任何 `security_* / stock_* / scorecard / training` 链路。
- 按 TDD 流程，这轮先补红测；实际执行时第二条新测试首次失败，暴露的是断言对 hits 排序契约理解偏差，而不是生产逻辑错误，因此只做测试预期修正，不改生产代码。

### 方案还差什么?
- [ ] 还未补“多个交集 seed + 多标签约束 + 冲突模板 seeds”三者同时出现时的集成测试。
- [ ] 还未补带标签约束条件下的 `RoamFailed` / `RetrievalFailed` 复合失败用例。
- [ ] 还未把这组复合边界规则同步沉淀到更正式的 foundation 设计文档，只先补到了测试与任务日志。

### 潜在问题
- [ ] 当前多标签过滤语义仍是“命中任一 required tag 即保留”；如果后续预期改成 AND 语义，现有测试和实现都需要一起调整。
- [ ] 当前 `hits` 顺序受 retrieval 评分与 `node_id` 次排序共同影响，不能直接等同于 route 顺序；后续新增断言时容易再次误判。
- [ ] 本轮仍是纯内存 fixture，尚未覆盖更大图谱、更多共享 alias 或多跳 path 下的复合排序抖动。

### 关闭项
- 已先补复合集成测试并确认红测：`cargo test --test navigation_pipeline_integration -- --nocapture`
- 已通过 `cargo test --test navigation_pipeline_integration -- --nocapture`
- 已通过 `cargo test --test capability_router_unit -- --nocapture`
- 已通过 `cargo test --test ontology_schema_unit -- --nocapture`
- 已通过 `cargo test --test ontology_store_unit -- --nocapture`

### 记忆点（可选）
- `route` 顺序与 `hits` 顺序不是同一层契约；当前 retrieval 在同分场景下按 `node_id` 升序稳定排序，后续写集成断言不能再把二者混用。
- 用户当前主线仍然是 foundation 通用导航内核的边界收敛，继续推进时优先补复合边界与文档沉淀，不要偏回业务链路。

## 2026-04-09
### 修改内容
- `tests/navigation_pipeline_integration.rs`：新增 `navigation_pipeline_surfaces_router_error_when_required_tags_filter_all_candidates`，补齐“显式标签把原本可匹配候选全部过滤掉”时的复合失败边界，目的是确认 `required_concept_tags` 在失败路径里同样会触发统一 `RouteFailed` 契约。
- `tests/navigation_pipeline_integration.rs`：新增 `navigation_pipeline_surfaces_retrieval_error_when_tagged_route_overrides_conflicting_template`，补齐“显式标签压住冲突模板后仍因无证据而失败”时的复合失败边界，目的是确认失败阶段会稳定落在 `RetrievalFailed`，而不是被模板冲突误导。
- `tests/navigation_pipeline_integration.rs`：新增 `sample_conflicting_seed_retrieval_miss_navigation_pipeline` 纯内存 fixture，目的是把“标签优先于冲突模板”的规则继续压测到失败路径，而不只停留在 happy path。

### 修改原因
- 当前 foundation 主线已经覆盖了标签约束与模板冲突的成功路径，但还缺少对应的失败路径护栏，继续推进前需要先把失败语义收紧。
- 我先核查了可达性，确认“显式标签约束 + 冲突模板”在当前契约下不会触发 `RoamFailed`：因为模板一旦与显式 tag 冲突，pipeline 会保留 route，而 route 中的 concept 必然来自 ontology store，漫游种子不会凭空失效。
- 按 TDD 流程，这轮先补两条失败测试；实际执行时两条测试都直接通过，说明当前生产实现已满足这组契约，因此本轮没有新增生产代码改动。

### 方案还差什么?
- [ ] 还未补“多标签约束 + 冲突模板 seeds + 失败边界”三者同时出现时的集成测试。
- [ ] 还未把“显式标签约束 + 冲突模板”下 `RoamFailed` 不可达这一架构事实整理进更正式的 foundation 设计文档。
- [ ] 还未补更大图谱、多跳 relation 下的标签失败场景测试。

### 潜在问题
- [ ] 当前“显式标签约束 + 冲突模板”不会触发 `RoamFailed` 是依赖现有 pipeline 契约成立的；如果未来有人改动 `constrain_route_to_template_seeds()` 的优先级，这个判断需要重新验证。
- [ ] 当前标签失败边界仍主要依赖纯内存 fixture，尚未覆盖更复杂的共享 alias 与多跳 path 情况。
- [ ] 这轮没有生产代码改动，如果后续出现失败语义漂移，应优先检查测试覆盖是否遗漏组合场景，而不是默认实现本身有新逻辑。

### 关闭项
- 已通过 `cargo test --test navigation_pipeline_integration navigation_pipeline_surfaces_router_error_when_required_tags_filter_all_candidates -- --nocapture`
- 已通过 `cargo test --test navigation_pipeline_integration navigation_pipeline_surfaces_retrieval_error_when_tagged_route_overrides_conflicting_template -- --nocapture`
- 已通过 `cargo test --test navigation_pipeline_integration -- --nocapture`
- 已通过 `cargo test --test capability_router_unit -- --nocapture`
- 已通过 `cargo test --test ontology_schema_unit -- --nocapture`
- 已通过 `cargo test --test ontology_store_unit -- --nocapture`

### 记忆点（可选）
- “显式标签约束 + 冲突模板”在当前通用导航契约下是可以产生 `RouteFailed` 和 `RetrievalFailed` 的，但不会自然产生 `RoamFailed`；后续不要再把这个不可达分支当成待补红测。
- foundation 主线下一步更适合继续补“多标签 + 冲突模板 + 失败边界”的三重组合，而不是回头去扩业务链路。
## 2026-04-09
### 修改内容
- `src/ops/foundation/metadata_constraint.rs`：新增 `MetadataConstraint::{Equals, In, HasAny, Range}` 与 `matches/matches_all`，并修正 `Range` 边界比较为 `&str` 对齐，目的是真正形成 foundation 通用 metadata 约束模型，而不是散落在 retrieval 里的临时字段判断。
- `src/ops/foundation/knowledge_record.rs`：为 `KnowledgeNode` 增加 `metadata` 容器、`MetadataFieldValue::{Text, TextList}` 以及 `with_metadata_text/with_metadata_values` 等写入与匹配辅助，目的是给 MetadataConstraint 提供标准节点级承载。
- `src/ops/foundation/capability_router.rs`：为 `NavigationRequest` 增加 `metadata_constraints` 与 `with_metadata_constraints(...)`，目的是让 metadata 约束以正式请求输入进入 foundation 主线。
- `src/ops/foundation/retrieval_engine.rs`：将 `retrieve(...)` 扩为接收 `metadata_constraints`，并在打分前执行 metadata 过滤，目的是把本轮方案B第一阶段明确收敛到 `NavigationRequest -> Retrieval`。
- `src/ops/foundation/navigation_pipeline.rs`：把 `request.metadata_constraints` 透传到 retrieval，目的是打通 metadata 约束在通用导航主线中的正式链路。
- `src/ops/foundation.rs`：导出 `metadata_constraint` 模块，目的是让该能力成为 foundation 一等公开模块。
- `tests/knowledge_record_unit.rs`、`tests/retrieval_engine_unit.rs`、`tests/navigation_pipeline_integration.rs`：先补 metadata 红测，再微调夹具文本与排序断言，目的是避免新增 metadata 专用样本误伤旧的 scoped retrieval / pipeline 契约。

### 修改原因
- 用户已明确本轮只推进 foundation 通用标准能力，并批准“方案B：先把 MetadataConstraint 做成一等模型，第一轮只正式接入 NavigationRequest -> Retrieval，不改 RoamingPlan 语义”。
- 红测已经先定义了三类边界：节点级 metadata 承载、retrieval 的 metadata 成功/失败过滤、pipeline 中 metadata 约束透传；必须补齐最小生产实现才能闭环。
- 原有 retrieval / pipeline 夹具在加入 metadata 样本后出现了“附加节点意外命中旧 query”的噪音；若不收敛夹具，会把测试失败误解成主链行为回归。

### 方案还差什么?
- [ ] 还未把 `MetadataConstraint` 正式接入 roaming 收敛、scope trace 或 roaming path 解释信息；本轮明确不做。
- [ ] 还未补 `In/HasAny` 在 pipeline 集成层的专项用例；当前只验证了 `Equals` 与 `Range` 的最小闭环。
- [ ] 还未把 metadata 约束的边界规则沉淀进更正式的 foundation 设计文档；目前先落在测试与任务日志中。

### 潜在问题
- [ ] 当前 `Range` 仍是字符串区间语义，只适用于 ISO 日期或其他可按字典序比较的字段；若后续接入非规范日期/数值字符串，可能出现比较歧义。
- [ ] 当前 metadata 只在 retrieval 前过滤节点，不参与 route / roam 收敛；若上游误以为 metadata 会影响 seed 选择，可能产生预期偏差。
- [ ] `cargo test` 过程中仍存在既有 `dead_code` warnings；本轮未处理这些告警，它们不影响当前 metadata 能力闭环，但会继续出现在回归输出中。

### 关闭项
- 已先复现并确认红测失败：`cargo test --test knowledge_record_unit -- --nocapture`、`cargo test --test retrieval_engine_unit -- --nocapture`、`cargo test --test navigation_pipeline_integration -- --nocapture`
- 已通过 `cargo test --test knowledge_record_unit -- --nocapture`
- 已通过 `cargo test --test retrieval_engine_unit -- --nocapture`
- 已通过 `cargo test --test navigation_pipeline_integration -- --nocapture`
- 已通过 `cargo test --test capability_router_unit --test ontology_schema_unit --test ontology_store_unit -- --nocapture`

### 记忆点（可选）
- foundation 当前主线是“通用 metadata 约束先收敛到 request -> retrieval”，不要再次默认把 metadata 自动扩进 roaming 语义。
- 当测试夹具因新增通用能力样本而变丰富时，优先先判断是不是夹具噪音污染了旧 query，再决定是否修改生产逻辑，不要把样本副作用误判成主链回归。
## 2026-04-09
### 修改内容
- `src/ops/foundation/metadata_constraint.rs`：新增 `MetadataScope` 标准模型，提供 `new/from_constraints/as_slice/matches`，目的是把 metadata 从 retrieval 的额外参数提升为 foundation 主线共享 scope 合同。
- `src/ops/foundation/capability_router.rs`：将 `NavigationRequest` 的 metadata 输入承载从裸 `Vec<MetadataConstraint>` 收敛为 `MetadataScope`，同时保留 `with_metadata_constraints(...)` 入口，目的是不破坏现有调用方式但完成标准模型升级。
- `src/ops/foundation/roaming_engine.rs`：为 `RoamingPlan` 与 `CandidateScope` 增加 `metadata_scope`，新增 `with_metadata_scope(...)`，并在 `roam(...)` 输出中保留该 scope，目的是让 metadata 正式经过 `RoamingPlan -> CandidateScope` 这一段主线合同。
- `src/ops/foundation/retrieval_engine.rs`：移除 `retrieve(...)` 的独立 metadata 参数，改为只消费 `scope.metadata_scope`，目的是让 retrieval 不再绕开统一 scope 合同。
- `src/ops/foundation/navigation_pipeline.rs`：在运行时构造 roaming plan 时注入 `request.metadata_scope`，并让 retrieval 直接消费 scope，目的是打通 `NavigationRequest -> RoamingPlan -> CandidateScope -> Retrieval` 的通用 metadata 主线。
- `tests/roaming_engine_unit.rs`：新增 `roaming_engine_preserves_metadata_scope_in_candidate_scope` 红绿测试，目的是钉死 roaming 至少要保留 metadata scope。
- `tests/retrieval_engine_unit.rs`：将 metadata 相关断言改为“由 `CandidateScope` 携带 metadata scope”后再执行 retrieval，目的是钉死 retrieval 的新合同。
- `tests/evidence_assembler_unit.rs`：对齐 `CandidateScope` 与 `CapabilityRoute` 的最新结构，目的是避免 assembler 继续依赖过时样本。

### 修改原因
- 用户已批准方案B第二阶段，要求继续推进通用化标准能力，而不是业务化字段特判；这一步最关键的是先把 metadata 正式提升为 route 之后主线共享的 scope 合同。
- roaming 当前只依赖 ontology store，看不到 graph node metadata；因此本轮合理边界不是“直接做节点级 metadata 漫游过滤”，而是先把 metadata 作为 `plan/scope` 的一部分稳定流过主线。
- 红测已经暴露出旧合同的问题：`RetrievalEngine` 仍依赖独立 metadata 参数，`RoamingPlan/CandidateScope` 还无法表达 metadata scope，这会阻塞后续真正的元数据收敛与解释能力。

### 方案还差什么?
- [ ] 还未让 metadata 真正参与 roaming 扩展收敛；当前只是先成为 roaming plan 与 candidate scope 的正式组成部分。
- [ ] 还未补 `MetadataConstraint::In` / `HasAny` 在 pipeline 集成层的专项测试与样本。
- [ ] 还未把 metadata scope 的解释信息输出到 trace / diagnostics；当前只有合同打通，没有显式解释面。

### 潜在问题
- [ ] 当前 `MetadataScope` 只是统一主线合同，不等于 roaming 已能理解节点级 metadata；若上游把这两件事混为一谈，容易出现能力预期偏差。
- [ ] 当前 `Range` 仍是字符串区间语义，只适合 ISO 日期或其他字典序稳定字段；若后续塞入非规范日期/数值文本，会出现比较歧义。
- [ ] 本轮仍有仓库既有 `dead_code` warnings，未处理这些告警；它们不影响当前闭环，但会继续出现在测试输出中。

### 关闭项
- 已先复现并确认红测失败：`cargo test --test roaming_engine_unit --test retrieval_engine_unit --test evidence_assembler_unit -- --nocapture`
- 已通过 `cargo test --test roaming_engine_unit --test retrieval_engine_unit --test evidence_assembler_unit -- --nocapture`
- 已通过 `cargo test --test knowledge_record_unit --test capability_router_unit --test ontology_schema_unit --test ontology_store_unit --test navigation_pipeline_integration -- --nocapture`

### 记忆点（可选）
- “metadata 进入 roaming 主线合同”不等于“roaming 已具备节点级 metadata 过滤能力”；后续不要再把这两个阶段混写成一件事。
- 用户当前持续强调的是通用标准能力，所以优先顺序应是先把 contract 做正，再逐步补 filtering / trace / metadata 管理，而不是回头做业务化分支。
## 2026-04-09
### 修改内容
- `src/ops/foundation/ontology_schema.rs`：为 `OntologyConcept` 新增通用 `metadata` 容器，以及 `with_metadata_text(...)`、`with_metadata_values(...)` 写入口，目的是让 concept 与 node 共享同一套 metadata value 模型，正式承接 concept-level metadata 收敛。
- `src/ops/foundation/metadata_constraint.rs`：补充 `MetadataScope::matches_metadata(...)`、`MetadataConstraint::field()`、`MetadataConstraint::matches_metadata(...)` 与 `matches_all_metadata(...)`，目的是把 metadata 匹配能力从仅绑定 `KnowledgeNode` 升级为可复用于 concept / node 的 foundation 通用合同。
- `src/ops/foundation/metadata_scope_resolver.rs`：新增通用 `MetadataScopeResolver`，提供 `constrain_concept_ids(...)` 与 concept 侧字段适用性判定，目的是把 concept-level metadata 收敛沉到独立标准能力层，而不是散落在 pipeline 或 roaming 分支中。
- `src/ops/foundation/ontology_store.rs`：新增 `concepts()` 只读访问入口，目的是让 resolver 在不越过 store 边界的前提下判断某个 metadata 字段是否真实存在于 concept 层。
- `src/ops/foundation/navigation_pipeline.rs`：在 template seed 收敛之后新增 metadata-aware route 收敛，并在 concept 被全部过滤时稳定返回 `RouteFailed`，目的是让 metadata 真正影响 `route -> roam` 主线，而不只停留在 retrieval。
- `src/ops/foundation/roaming_engine.rs`：在 seed 入队和 neighbor 扩展时复用 `MetadataScopeResolver` 过滤 concept ids，并补 `ontology_store()` 只读访问器，目的是让 roaming 对 concept metadata 收敛保持与 pipeline 一致的标准语义。
- `src/ops/foundation.rs`：导出 `metadata_scope_resolver` 模块，目的是把 metadata-aware concept 收敛正式挂入 foundation 公共模块边界。
- `tests/metadata_scope_resolver_unit.rs`、`tests/navigation_pipeline_integration.rs`：先用红测钉住 `In` / `HasAny` 的 concept-level metadata 收敛，再实现生产代码，目的是继续按 TDD 完成这轮通用能力闭环。

### 修改原因
- 用户已批准继续走方案B，且明确只做 foundation 通用标准能力，不做业务化链路，因此本轮必须把 metadata 从“主线合同化”继续推进到“真实参与 concept-level 收敛”。
- 上一轮虽然已经完成 `NavigationRequest -> RoamingPlan -> CandidateScope -> Retrieval` 的 metadata 合同接线，但 roaming 之前的 concept 收敛仍未真正消费 concept metadata，导致 `MetadataScope` 还没有形成完整的通用闭环。
- 同一份 `MetadataScope` 现在同时约束 concept 与 node；如果不显式区分“哪些字段在 concept 层存在”，像只存在于 node 的 `source` 一类条件会把 route 错误过滤空，所以需要单独补 resolver 适用性判断。

### 方案还差什么？
- [ ] 还未补 concept metadata 全部过滤空时的专项集成红测；当前已通过实现把该边界稳定映射为 `RouteFailed`，但还缺单独钉死该语义的测试名例。
- [ ] 还未补更复杂多跳图下的 metadata-aware roaming 扩展测试；当前覆盖了 seed 收敛与一跳 path 主线，但还没有覆盖更深层 BFS 场景。
- [ ] 还未把 metadata 字段治理、字段注册、字段解释等“本体论元数据管理”继续往上推进；本轮只完成 foundation 通用收敛内核，不做更上层管理面。

### 潜在问题
- [ ] 当前 concept-level metadata 收敛只会消费“在 concept 层真实存在的字段”；如果后续有人误以为任意 metadata 字段都会同时作用于 route / roam，就会产生能力预期偏差。
- [ ] `MetadataScopeResolver` 目前按 store 中 concept metadata 字段是否存在来判断约束是否适用于 concept；如果未来需要更细粒度的“按命中候选集而不是全 store”判定，可能还要继续细化策略。
- [ ] `cargo test` 过程仍会输出仓库现有的 `dead_code` warnings；本轮未处理这些历史告警，它们不影响当前 metadata 主线闭环，但会继续出现在回归输出里。

### 关闭项
- 已先复现红测失败：`cargo test --test metadata_scope_resolver_unit --test navigation_pipeline_integration -- --nocapture`
- 已通过 `cargo test --test metadata_scope_resolver_unit --test navigation_pipeline_integration -- --nocapture`
- 已通过 `cargo test --test knowledge_record_unit --test capability_router_unit --test ontology_schema_unit --test ontology_store_unit --test roaming_engine_unit --test retrieval_engine_unit --test evidence_assembler_unit --test navigation_pipeline_integration -- --nocapture`

### 记忆点（可选）
- 同一份 `MetadataScope` 现在同时服务 concept-level 收敛和 node-level retrieval；后续继续推进时，不要再把“只存在于 node 的 metadata 字段”默认成也会参与 route / roam 过滤。
- 用户当前主线仍然是 foundation 通用能力与本体元数据能力，不要把后续工作偏回 `security_* / stock_* / scorecard / training` 之类业务链路。
## 2026-04-09
### 修改内容
- `src/ops/foundation/metadata_registry.rs`：新增通用 `MetadataRegistry`、`MetadataFieldTarget`、`MetadataConstraintOperator` 与字段定义模型，目的是把 metadata 字段名、适用层级、支持操作符从隐式推断升级成显式注册的标准能力。
- `src/ops/foundation/metadata_constraint.rs`：新增 `operator()`、`constraints_for_registered_target(...)` 与 `matches_metadata_for_registered_target(...)`，目的是让同一份 `MetadataScope` 能按 registry 显式区分 concept 侧与 node 侧应消费的约束子集。
- `src/ops/foundation/metadata_scope_resolver.rs`：新增 `constrain_concept_ids_with_registry(...)`，目的是让 concept-level metadata 收敛从“扫现有 key 猜字段适用性”升级成“按注册表显式判断 concept 目标字段”。
- `src/ops/foundation/roaming_engine.rs`：新增带 registry 的构造器，并让 seed 与 neighbor 收敛在 registry 存在时走显式字段目录，目的是让 roaming 也使用统一的 metadata 字段管理合同。
- `src/ops/foundation/retrieval_engine.rs`：新增 `retrieve_with_metadata_registry(...)`，目的是让 retrieval 在 registry 模式下只消费 node-target metadata 字段，避免 concept-only 字段误过滤节点命中。
- `src/ops/foundation/navigation_pipeline.rs`：新增 `new_with_metadata_registry(...)`，并在 route / roam / retrieve 三段统一接入 registry 感知路径，目的是让 foundation 主线共享同一份 metadata 字段目录。
- `src/ops/foundation.rs`：导出 `metadata_registry` 模块，目的是把元数据字段目录正式纳入 foundation 标准能力边界。
- `tests/metadata_registry_unit.rs`：新增 registry 单测，钉死字段目标层级与支持操作符的标准声明能力。
- `tests/navigation_pipeline_integration.rs`：新增 `navigation_pipeline_keeps_hits_when_concept_only_metadata_is_registered`，钉死 concept-only 字段只影响 concept 收敛，不会在 retrieval 侧误过滤 node evidence。
- `docs/ai-handoff/AI_HANDOFF_MANUAL.md`、`docs/ai-memory/project-baseline.md`：补充“metadata 字段管理应走显式 registry，且字段必须声明作用层级”的长期约束，目的是减少后续 AI 会话重复花时间重新对齐。

### 修改原因
- 用户已批准继续走方案B，并明确当前主线是“通用的本体元数据管理”，不是业务链路扩展，因此需要先把 metadata 字段管理收敛成标准能力。
- 上一轮 concept-level metadata 收敛仍有一个隐含问题：字段适用层级靠实际数据里有没有某个 key 来猜，这在 concept-only / node-only 字段并存时不够稳定，也不够可治理。
- 当前最关键的基础能力是把“字段属于 concept、属于 node、还是同时属于两者”显式固定下来，否则同一份 `MetadataScope` 会在不同阶段产生歧义。

### 方案还差什么？
- [ ] 还未补 retrieval 层针对 `retrieve_with_metadata_registry(...)` 的专项单测；本轮已用 pipeline 集成红测钉住主线语义，但还缺更细粒度的执行器单测。
- [ ] 还未补“registry 已注册但 concept/node 当前未携带该字段”时的专项边界测试，当前先只覆盖 concept-only 字段不污染 retrieval 的核心路径。
- [ ] 还未继续推进更上层的 metadata 字段治理面，例如字段版本、字段分组、字段解释、字段审计输出；本轮只完成 foundation 标准目录第一阶段。

### 潜在问题
- [ ] 当前 registry 模式下，未注册字段会被对应目标层级忽略；如果后续希望改成“未注册即报错”或“未注册即拒绝执行”，还需要再补校验策略和测试。
- [ ] 当前 operator 支持关系已进入 registry，但本轮只把它用于筛选适用约束，尚未把“不支持的 operator 是否报错”收敛成正式错误合同。
- [ ] 仓库现有 `dead_code` warnings 仍然存在，本轮未处理这些历史告警；它们不影响当前 metadata 字段目录主线，但会继续出现在回归输出里。

### 关闭项
- 已先复现红测失败：`cargo test --test metadata_registry_unit --test navigation_pipeline_integration -- --nocapture`
- 已通过 `cargo test --test metadata_registry_unit --test navigation_pipeline_integration -- --nocapture`
- 已通过 `cargo test --test metadata_registry_unit --test metadata_scope_resolver_unit --test knowledge_record_unit --test capability_router_unit --test ontology_schema_unit --test ontology_store_unit --test roaming_engine_unit --test retrieval_engine_unit --test evidence_assembler_unit --test navigation_pipeline_integration -- --nocapture`

### 记忆点（可选）
- metadata 字段目录现在已经进入 foundation 标准能力边界；后续不要再把 concept/node 字段适用层级写成“靠数据里有没有这个 key 猜”。
- concept-only 字段与 node-only 字段必须显式区分，否则同一份 `MetadataScope` 会在 route / roam / retrieve 三段产生错误的交叉过滤。

## 2026-04-09
### 修改内容
- `tests/retrieval_engine_unit.rs`：新增 registry 模式下 `concept-only` 字段忽略、未注册字段报错、node-target 不支持 operator 报错三条单测，目的是把 retrieval 执行器的字段目录合同从集成层下沉到单测层钉死。
- `tests/metadata_scope_resolver_unit.rs`：新增 concept resolver 在 registry 模式下对未注册字段与不支持 operator 的红测，目的是让 route / roam 的 concept-level 收敛也受同一份字段治理合同约束。
- `tests/navigation_pipeline_integration.rs`：新增 pipeline 对未注册字段与 node-target 非法 operator 的透传测试，并补 `sample_node_only_metadata_registry_navigation_pipeline()` 夹具，目的是验证主线会显式暴露 registry 错误而不是吞成空结果。
- `src/ops/foundation/metadata_registry.rs`：新增 `MetadataRegistryError` 与 `has_field(...)`，目的是把 registry 模式下的字段治理错误收敛成共享错误模型。
- `src/ops/foundation/metadata_constraint.rs`：把 `constraints_for_registered_target(...)` 与 `matches_metadata_for_registered_target(...)` 升级为返回 `Result` 的校验路径，目的是让同一份 `MetadataScope` 在 target-aware 切片时既能忽略异 target 字段，也能拒绝未注册字段和非法 operator。
- `src/ops/foundation/metadata_scope_resolver.rs`：让 `constrain_concept_ids_with_registry(...)` 返回 `Result`，目的是把 registry 模式的 concept-level 错误合同正式带到 route / roam 入口。
- `src/ops/foundation/retrieval_engine.rs`：新增 `RetrievalEngineError::InvalidMetadataConstraint(...)`，并在 registry 模式下先校验 node-target 约束再执行打分，目的是区分“约束非法”和“合法但无命中证据”。
- `src/ops/foundation/roaming_engine.rs`：新增 `RoamingEngineError::InvalidMetadataConstraint(...)` 并透传 concept-level registry 校验错误，目的是保证直接调用 roaming 也遵守相同字段目录合同。
- `src/ops/foundation/navigation_pipeline.rs`：新增 `NavigationPipelineError::InvalidMetadataConstraint(...)`，并让 route / roam / retrieve 三段统一透传 registry 错误，目的是把字段治理失败正式提升为 foundation 主线级可解释错误。
- `docs/ai-handoff/AI_HANDOFF_MANUAL.md`、`docs/ai-memory/project-baseline.md`：补充“registry 模式下未注册字段和不支持 operator 必须 fail fast”的长期约束，目的是减少后续 AI 会话再次把错误合同退化成静默忽略。

### 修改原因
- 用户已批准继续走方案B，当前目标仍是 foundation 的通用本体元数据管理，不做任何业务化字段或业务链路扩展。
- 上一轮虽然完成了 metadata registry 第一阶段，但 `CHANGELOG_TASK.MD` 中明确剩余的尾项正是 retrieval 专项单测与“不支持 operator 是否报错”的正式合同；如果不补，registry 仍只有目录声明没有执行约束。
- 当前最关键的通用化能力不是新增字段，而是把“字段未注册”和“operator 超出合同”与“正常无命中”清晰分离，避免上层把治理缺失误判成检索失败。

### 方案还差什么?
- [ ] 还未补 `metadata_registry_unit.rs` 对 `has_field(...)` 与错误模型的更细单测；本轮先优先补齐执行链路红绿闭环。
- [ ] 还未推进 metadata 字段治理的更上层能力，例如字段分组、字段解释、字段审计输出、字段版本/兼容策略。
- [ ] 还未决定未来是否需要“严格 registry 模式”和“兼容 registry 模式”的显式运行时开关；当前行为是只要启用 registry 就走 fail-fast。

### 潜在问题
- [ ] 当前 fail-fast 语义只在“启用 registry”时成立；旧的空 registry 兼容路径仍保持原有宽松语义，后续如果两种模式混用需要注意解释差异。
- [ ] 这次新增了新的错误分支 `InvalidMetadataConstraint(...)`；如果未来上层调用方开始依赖 pipeline 错误枚举，需要同步处理这个新增分支。
- [ ] 仓库内历史 `dead_code` warnings 仍然存在，本轮未处理这些非 foundation metadata 主线问题。

### 关闭项
- 已先复现红测失败：`cargo test --test retrieval_engine_unit --test metadata_scope_resolver_unit --test navigation_pipeline_integration -- --nocapture`
- 已通过定向绿测：`cargo test --test retrieval_engine_unit --test metadata_scope_resolver_unit --test navigation_pipeline_integration -- --nocapture`
- 已通过 foundation 回归：`cargo test --test metadata_registry_unit --test metadata_scope_resolver_unit --test knowledge_record_unit --test capability_router_unit --test ontology_schema_unit --test ontology_store_unit --test roaming_engine_unit --test retrieval_engine_unit --test evidence_assembler_unit --test navigation_pipeline_integration -- --nocapture`

### 记忆点（可选）
- 启用 metadata registry 以后，不要再把“未注册字段”或“operator 不支持”处理成静默忽略；这两类情况现在属于 foundation 主线的正式错误合同。
- concept-only / node-only 的 target 切片语义依然保留：异 target 字段应被忽略，但只要字段落到当前 target，就必须同时接受 registry 的 operator 合同校验。

## 2026-04-09
### 修改内容
- `src/ops/foundation/metadata_registry.rs`：新增 `MetadataFieldGroup`、`MetadataFieldGovernance`、`MetadataRegistryValidationIssueKind`、`MetadataRegistryValidationIssue`、`MetadataRegistryValidationSummary`，并为 `MetadataFieldDefinition` 增加治理属性容器，目的是把 metadata 字段目录从“可注册”推进到“可治理、可校验”。
- `src/ops/foundation/metadata_registry.rs`：新增 `register_text_field_with_governance(...)`、`register_text_list_field_with_governance(...)`、`definition(...)`、`definitions(...)`、`fields_in_group(...)`、`validate_governance()`，目的是提供字段治理属性声明、读取和质量摘要的标准入口，同时保持旧注册 API 兼容。
- `tests/metadata_registry_unit.rs`：新增字段治理属性读写、按 group 读取、治理摘要输出三组红绿测试，目的是钉死方案B2第一轮的标准能力边界。
- `docs/ai-handoff/AI_HANDOFF_MANUAL.md`、`docs/ai-memory/project-baseline.md`：补充“字段定义应携带治理属性”和“registry 应输出结构化治理摘要”的长期约束，目的是减少后续 AI 会话重复对齐成本。

### 修改原因
- 用户已批准继续走方案B2，本轮目标是通用 metadata 字段治理，而不是继续扩展 route / roam / retrieve 的业务行为。
- 当前 registry 虽然已经有 target/operator 合同，但还缺“字段属于哪一组、字段含义是什么、为什么落在该 target、是否废弃”的治理层信息；如果不补，这条本体元数据主线仍然停留在执行期目录层。
- 方案B2要求不只是保存治理属性，还要给出结构化校验摘要，否则字段治理质量仍然只能靠人工逐项检查。

### 方案还差什么?
- [ ] 还未把 `validate_governance()` 接到单独的审计输出模块；本轮只先提供 foundation 内部可复用的摘要结构。
- [ ] 还未加入字段别名、替代字段、兼容说明等版本演进能力；这些属于后续字段治理第三阶段。
- [ ] 还未把 group 体系扩展到更细粒度分类；本轮先只保留 `Identity / Classification / Source / Time / Governance` 五个通用分组。

### 潜在问题
- [ ] 当前治理属性仍是可选的，旧注册 API 继续允许缺省；因此 `validate_governance()` 在过渡阶段会持续报告缺失项，这是有意保留的兼容行为。
- [ ] `MetadataFieldGroup` 目前是通用最小枚举；如果后续分类扩得过快，可能需要重新梳理分组边界，避免过细导致反复调整。
- [ ] 本轮只新增了 foundation 内部的治理摘要模型，没有新增 CLI 或导出层，因此外部消费方还不能直接拿到格式化报告。

### 关闭项
- 已先复现红测失败：`cargo test --test metadata_registry_unit -- --nocapture`
- 已通过最小绿测：`cargo test --test metadata_registry_unit -- --nocapture`
- 已通过回归验证：`cargo test --test metadata_registry_unit --test metadata_scope_resolver_unit --test retrieval_engine_unit --test navigation_pipeline_integration -- --nocapture`

### 记忆点（可选）
- 方案B2这一轮的主线不是新增业务字段，而是把 metadata 字段本体补成“可分组、可解释、可校验”的标准治理对象。
- 旧的 `register_text_field(...) / register_text_list_field(...)` 仍然保留兼容路径；新治理能力通过带 governance 的注册入口和 `validate_governance()` 渐进接入，不要硬切旧调用。

## 2026-04-09
### 修改内容
- `src/ops/foundation/metadata_registry.rs`：在 `MetadataFieldGovernance` 中新增 `aliases`、`replaced_by`、`compatibility_note`，并补 `with_aliases(...)`、`with_replaced_by(...)`、`with_compatibility_note(...)`，目的是把字段兼容关系正式纳入 metadata 字段治理模型。
- `src/ops/foundation/metadata_registry.rs`：新增 `MetadataRegistryCompatibilityEntry`、`MetadataRegistryAliasMapping`、`MetadataRegistryCompatibilitySummary`，并补 `compatibility_for(...)`、`compatibility_summary()` 与内部 replacement chain 解析逻辑，目的是让 registry 能稳定输出单字段兼容视图和全量兼容映射表。
- `src/ops/foundation/metadata_registry.rs`：扩展 `MetadataRegistryValidationIssueKind`，新增 `DeprecatedWithoutReplacement`、`DeprecatedWithoutCompatibilityNote`、`ReplacedByMissingField` 三类治理问题，目的是把兼容关系缺口纳入结构化治理校验摘要。
- `tests/metadata_registry_unit.rs`：新增兼容摘要与替代链路红绿测试，以及兼容关系治理缺口红绿测试，目的是钉死方案B-2的字段兼容骨架和兼容摘要边界。
- `docs/ai-handoff/AI_HANDOFF_MANUAL.md`、`docs/ai-memory/project-baseline.md`：补充“字段治理可携带兼容属性，registry 应输出兼容摘要与替代链路”的长期约束，目的是减少后续 AI 会话重复收敛边界的成本。

### 修改原因
- 用户已批准继续走方案B-2，本轮目标是通用 metadata 字段兼容治理，而不是新增业务字段或修改执行链路。
- 上一轮已经完成字段治理属性与治理摘要，但还缺 alias、替代字段和兼容说明这一层兼容骨架；如果不补，字段治理仍然只覆盖“当前状态”，无法承载版本演进和历史字段兼容。
- 方案B-2要求不仅能描述兼容关系，还要能输出结构化兼容摘要，并把兼容缺口纳入校验结果，否则仍然只能靠人工阅读 registry 判断迁移路径是否完整。

### 方案还差什么?
- [ ] 还未提供独立的 compatibility audit/export 模块；本轮只把兼容摘要能力收敛在 `MetadataRegistry` 内部。
- [ ] 还未加入 alias 冲突、替代链路循环等更强校验规则；本轮先只覆盖缺替代目标、缺兼容说明和缺替代字段这三类最小兼容问题。
- [ ] 还未把兼容摘要接到 CLI、report 或其他外部可见输出层；当前仍是 foundation 内部标准能力。

### 潜在问题
- [ ] 当前 replacement chain 解析会在发现重复字段时停止，避免循环卡死；但本轮还没有把“替代链路循环”显式输出成治理问题。
- [ ] alias 映射目前按 alias 排序输出，适合测试和审计；如果未来需要按 canonical field 分组展示，摘要结构可能还要再扩一层视图。
- [ ] 旧注册 API 仍允许完全不声明兼容属性，因此过渡阶段的 registry 仍会在 `validate_governance()` 中持续暴露兼容治理缺口，这是有意保留的渐进接入行为。

### 关闭项
- 已先复现红测失败：`cargo test --test metadata_registry_unit -- --nocapture`
- 已通过最小绿测：`cargo test --test metadata_registry_unit -- --nocapture`
- 已通过回归验证：`cargo test --test metadata_registry_unit --test metadata_scope_resolver_unit --test retrieval_engine_unit --test navigation_pipeline_integration -- --nocapture`

### 记忆点（可选）
- 兼容骨架这一轮仍然只属于“目录治理能力”，不要把 alias/replaced_by 误解成已经进入 route / roam / retrieve 的运行时规范化逻辑。
- 对废弃字段来说，`deprecated_reason` 现在不再足够；如果没有 `replaced_by` 或 `compatibility_note`，应视为治理不完整并进入结构化校验摘要。
## 2026-04-09
### 修改内容
- `src/ops/foundation/metadata_registry.rs`：新增 `MetadataRegistryFieldResolution`、`MetadataRegistryFieldResolutionBatch`、`resolve_field(...)` 与 `normalize_fields(...)`，目的是把字段别名收敛能力从静态 compatibility summary 提升为可直接消费的目录解析能力。
- `tests/metadata_registry_unit.rs`：新增 canonical field resolver 与 batch normalization 红绿测试，覆盖直命中、alias 命中、废弃字段替代链回传、未知字段保留与输入顺序稳定，目的是钉住方案 B3 的标准能力边界。
- `docs/ai-handoff/AI_HANDOFF_MANUAL.md`、`docs/ai-memory/project-baseline.md`：补充 canonical resolution 与 batch normalization 的长期约束，目的是减少后续 AI 会话再次把目录治理能力误收敛成运行时自动规范化。

### 修改原因
- 用户已批准继续走方案 B3，本轮目标仍然是 foundation 的通用 metadata 目录治理，不做任何业务化字段或业务链路扩展。
- 上一轮已经完成 governance summary 与 compatibility summary，但目录层还缺少“把输入字段名稳定解析到 canonical field”的直接消费能力；如果不补，后续工具仍要手工拼接 alias/replacement 信息。
- 这轮能力必须继续停留在 registry 目录层，而不能偷渡成 route / roam / retrieve 的运行时字段重写逻辑，所以需要同时把边界写回交接文档。

### 方案还差什么?
- [ ] 还未加入 alias 冲突、replacement chain 循环、多字段汇聚到同一替代字段等更强治理校验；当前只完成 canonical resolver 与批量规范化。
- [ ] 还未提供 batch normalization 的聚合摘要视图，例如按 canonical field 聚合、未知字段清单、命中统计等目录工具能力。
- [ ] 还未把 resolver/batch normalization 接到独立 audit/export 模块；当前仍是 foundation 内部标准能力。

### 潜在问题
- [ ] 当前 canonical resolver 仍按 registry 现有兼容关系工作；如果后续出现 alias 冲突，现有结构化结果还不能主动暴露冲突诊断。
- [ ] `normalize_fields(...)` 现在强调“不丢字段”和“保序”，如果后续有人把它误改成只返回 canonical 字段集合，会破坏目录治理的可审计性。
- [ ] 这轮没有把目录解析能力接入运行主链；如果上层误以为 alias 已自动参与检索或漫游，会产生能力预期偏差。

### 关闭项?
- 已通过 `cargo test --test metadata_registry_unit -- --nocapture`
- 已通过 `cargo test --test metadata_registry_unit --test metadata_scope_resolver_unit --test retrieval_engine_unit --test navigation_pipeline_integration -- --nocapture`

### 记忆点（可选）
- canonical field resolver / batch normalization 现在属于 metadata registry 的目录治理能力，不要再把它们误写成 foundation 主链上的自动字段规范化行为。
- 方案 B3 的关键合同是：保留输入顺序、保留未知字段、同时返回 alias 命中和 replacement chain；后续若继续扩展目录工具，不能丢掉这三个约束。
## 2026-04-09
### 修改内容
- `tests/metadata_registry_unit.rs`：新增 alias 冲突、replacement cycle、共享 replacement target 歧义三组红绿测试，目的是先把方案A要求的三类兼容治理校验边界钉死在单测层。
- `src/ops/foundation/metadata_registry.rs`：扩展 `MetadataRegistryValidationIssueKind`，新增 `AliasConflict`、`ReplacementCycle`、`AmbiguousReplacementTarget` 三类治理问题；同时补 `alias_owners()`、`replacement_target_sources()`、`has_replacement_cycle()` 等目录级校验辅助逻辑，目的是让 `validate_governance()` 能稳定输出这三类结构化治理问题。
- `docs/ai-handoff/AI_HANDOFF_MANUAL.md`、`docs/ai-memory/project-baseline.md`：补充“alias 冲突 / 替代链循环 / 共享替代目标歧义必须显式报出”的长期约束，目的是避免后续 AI 把目录冲突继续误当成隐式兼容行为。

### 修改原因
- 用户已批准继续走方案A，本轮目标是把 metadata registry 的兼容治理校验补齐，不扩目录工具视图，也不碰 route / roam / retrieve 的运行语义。
- 上一轮虽然已经完成 canonical resolver 与 batch normalization，但兼容治理仍缺少三类硬校验：alias 冲突、replacement chain 循环、多个字段汇聚到同一替代目标的歧义。
- 当前 `replacement_chain_for(...)` 只会在内部避免循环卡死，这不足以支撑审计与交接；必须把“内部保护”提升为“显式治理问题”。

### 方案还差什么?
- [ ] 还未把 alias 冲突的具体冲突详情结构化输出，例如冲突 alias 名称和参与字段列表；当前治理摘要只先报告字段级问题。
- [ ] 还未把 replacement cycle 的完整环路路径导出成审计视图；当前只先提供布尔型治理告警。
- [ ] 还未把共享 replacement target 的歧义整理成更强的目录工具摘要或独立 audit/export 模块；当前仍停留在 validation summary 层。

### 潜在问题
- [ ] 当前 `AliasConflict` / `AmbiguousReplacementTarget` 只进入治理摘要，不会阻止 `resolve_field(...)` 继续按现有目录遍历逻辑解析；如果后续需要更强 fail-fast，还要再补运行前校验策略。
- [ ] `ReplacementCycle` 现在会对落入循环链的字段逐个报出问题；如果后续希望按“完整循环组”展示，摘要模型还需要扩结构。
- [ ] 这轮仍然不改变 registry 目录能力与主链执行之间的边界；如果上层误以为新增治理校验已经自动影响运行时字段解析顺序，会产生能力预期偏差。

### 关闭项?
- 已先复现红测失败：`cargo test --test metadata_registry_unit -- --nocapture`
- 已通过 `cargo test --test metadata_registry_unit -- --nocapture`
- 已通过 `cargo test --test metadata_registry_unit --test metadata_scope_resolver_unit --test retrieval_engine_unit --test navigation_pipeline_integration -- --nocapture`

### 记忆点（可选）
- 方案A这一轮补的是“兼容治理校验”，不是“运行时解析策略”；后续不要把 `AliasConflict` / `ReplacementCycle` / `AmbiguousReplacementTarget` 误写成已经进入主链的 fail-fast 执行逻辑。
- 当前 metadata registry 的兼容治理主线已经补齐到：compatibility summary、canonical resolver、batch normalization、三类兼容冲突校验；下一步若继续，优先看目录工具增强或 audit/export，而不是回头扩业务能力。
## 2026-04-09
### 修改内容
- `tests/metadata_registry_unit.rs`：新增 `metadata_registry_batch_normalization_summary_reports_batch_counts` 红绿测试，覆盖 batch 规范化后的总输入数、直命中数、alias 命中数、未识别数、废弃字段数，目的是钉死方案B1的最小摘要合同。
- `src/ops/foundation/metadata_registry.rs`：新增 `MetadataRegistryFieldResolutionBatchSummary` 与 `MetadataRegistryFieldResolutionBatch::summary()`，目的是在保留 `normalize_fields(...)` 明细结果的前提下，补一层可直接消费的批量统计摘要。
- `docs/ai-handoff/AI_HANDOFF_MANUAL.md`、`docs/ai-memory/project-baseline.md`：补充“batch normalization 可输出轻量统计摘要”的长期约束，目的是避免后续 AI 把这轮能力误写成未知字段清单或 canonical 聚合视图。

### 修改原因
- 用户已批准继续走方案B1，本轮目标是目录工具增强里的“批量规范化摘要”，不是新增治理校验，也不是扩展 audit/export。
- 当前 `normalize_fields(...)` 已能稳定输出逐字段明细，但调用方若只关心“这一批大概解析成什么样”，仍然要重复手写计数逻辑。
- 方案B1必须保持最小边界，所以这轮只补 count summary，不提前做未知字段清单或按 canonical field 聚合。

### 方案还差什么?
- [ ] 还未输出未知字段清单；如果后续继续走方案B2，再补结构化 unknown list 视图。
- [ ] 还未输出按 canonical field 聚合的归并视图；如果后续继续走方案B3，再补 canonical aggregation。
- [ ] 还未把 batch summary 接到独立 audit/export 模块；当前仍是 metadata registry 内部目录工具能力。

### 潜在问题
- [ ] 当前 `summary()` 只给出计数，不返回具体字段列表；如果上层把它误当成可审计明细，会发现信息不够。
- [ ] `deprecated_hits` 现在统计的是“本批输入里最终命中的解析结果是否已废弃”；如果未来需要区分“直命中废弃字段”和“alias 命中废弃字段”，摘要模型还要再扩。
- [ ] 这轮仍然不改变 `normalize_fields(...)` 的明细语义和主链执行边界；如果有人误以为新增 summary 会驱动运行时行为变化，会产生能力预期偏差。

### 关闭项?
- 已先复现红测失败：`cargo test --test metadata_registry_unit -- --nocapture`
- 已通过 `cargo test --test metadata_registry_unit -- --nocapture`
- 已通过 `cargo test --test metadata_registry_unit --test metadata_scope_resolver_unit --test retrieval_engine_unit --test navigation_pipeline_integration -- --nocapture`

### 记忆点（可选）
- 方案B1这一轮只补“批量统计摘要”，不要再把它误写成“未知字段清单”或“canonical 聚合视图”；那两块分别属于后续 B2 / B3。
- `normalize_fields(...)` 的关键合同仍然是保序和不丢字段，`summary()` 只是读取这份明细后的轻量统计层，不能反过来替代明细输出。
## 2026-04-09
### 修改内容
- `tests/metadata_registry_unit.rs`：新增 `metadata_registry_batch_normalization_lists_unknown_fields_in_input_order` 红绿测试，覆盖未知字段清单的输入顺序、重复未知字段保留、输入位置回传，目的是钉死方案B2的最小目录视图合同。
- `src/ops/foundation/metadata_registry.rs`：新增 `MetadataRegistryUnknownFieldEntry` 与 `MetadataRegistryFieldResolutionBatch::unknown_fields()`，目的是在 batch normalization 明细与 count summary 之外，再补一层结构化 unknown-field list 视图。
- `docs/ai-handoff/AI_HANDOFF_MANUAL.md`、`docs/ai-memory/project-baseline.md`：补充“unknown-field list 应保序、保留重复项、带输入位置”的长期约束，目的是避免后续 AI 把未知字段清单误做成去重集合或无序导出。

### 修改原因
- 用户已批准继续走方案B2，本轮目标是目录工具增强里的“未知字段清单视图”，不是再补治理校验，也不是提前做 canonical 聚合。
- 当前 `summary()` 只能告诉上层“有多少未识别字段”，但不能回答“哪些字段未识别、它们在输入批次中的什么位置出现”，调用方仍需自行二次过滤 unresolved entries。
- 方案B2必须保持最小边界，因此这轮只补结构化 unknown-field list，不扩成 canonical aggregation 或更复杂 audit report。

### 方案还差什么?
- [ ] 还未提供未知字段去重视图；当前有意保留重复项，便于审计输入批次中的真实出现次数。
- [ ] 还未提供未知字段按出现频次或按位置分组的更强目录工具摘要；当前只给最小 ordered list。
- [ ] 还未进入 canonical field 聚合视图；如果后续继续走方案B3，再补按 canonical field 的归并能力。

### 潜在问题
- [ ] `unknown_fields()` 现在返回的是有序重复清单，不是去重集合；如果上层误把它当成唯一字段列表，统计口径会偏大。
- [ ] 当前输入位置使用的是 batch 中的 0-based index；如果后续要直接面向终端展示，可能还需要额外封装成人类可读位置格式。
- [ ] 这轮依然不改变 `normalize_fields(...)` 的目录语义和主链执行边界；如果有人误以为 unknown-field list 会驱动运行时拒绝逻辑，会产生能力预期偏差。

### 关闭项?
- 已先复现红测失败：`cargo test --test metadata_registry_unit -- --nocapture`
- 已通过 `cargo test --test metadata_registry_unit -- --nocapture`
- 已通过 `cargo test --test metadata_registry_unit --test metadata_scope_resolver_unit --test retrieval_engine_unit --test navigation_pipeline_integration -- --nocapture`

### 记忆点（可选）
- 方案B2这一轮补的是“未知字段清单视图”，不是“未知字段去重集合”；后续不要把重复项静默折叠掉。
- `unknown_fields()` 现在属于 batch normalization 的审计辅助层，和 `summary()` 一样都是读取明细的目录工具，不是主链执行策略。
## 2026-04-09
### 修改内容
- `tests/metadata_registry_unit.rs`：新增 `metadata_registry_batch_normalization_groups_entries_by_canonical_field` 红绿测试，约束 `canonical_aggregations()` 只聚合已解析字段、保留 canonical 首次出现顺序，并区分 `direct_hits / alias_hits / deprecated_hits`，目的是把方案 B3 的目录工具边界先钉死。
- `src/ops/foundation/metadata_registry.rs`：新增 `MetadataRegistryCanonicalFieldAggregationEntry` 与 `MetadataRegistryFieldResolutionBatch::canonical_aggregations()`，目的是真正提供按 canonical field 汇总的只读目录视图，同时不改变 `normalize_fields(...)`、`summary()`、`unknown_fields()` 既有合同。
- `docs/ai-handoff/AI_HANDOFF_MANUAL.md`、`docs/ai-memory/project-baseline.md`：补充 canonical aggregation 的长期约束，明确未知字段不进入聚合、聚合顺序取 canonical 首次出现顺序、该能力仍属于目录治理层，目的是减少后续 AI 会话重复收敛边界。

### 修改原因
- 用户已批准继续执行方案 `B3`，这一轮目标是补齐 metadata registry 的“按 canonical field 聚合视图”，不是扩业务字段，也不是修改 route / roam / retrieve 运行语义。
- 在已有 `resolve_field(...)`、`normalize_fields(...)`、`summary()`、`unknown_fields()` 的基础上，调用方如果要做 canonical 归并仍需重复遍历 batch 明细；这部分属于标准目录工具能力，适合下沉到 registry。
- 这轮能力必须继续保持在 foundation 的目录治理层，因此实现只读聚合视图，不做自动规范化、不接入主链执行。

### 方案还差什么?
- [ ] 还未提供 canonical aggregation 的更强审计视图，例如输入位置列表、按 canonical field 的去重统计或导出接口；本轮只补最小聚合结构。
- [ ] 还未把 canonical aggregation 接入独立 audit/export 模块；当前仍是 `MetadataRegistryFieldResolutionBatch` 的目录工具方法。
- [ ] 还未把 alias 冲突等治理问题直接与 aggregation 关联展示；当前治理校验和聚合视图仍保持解耦。

### 潜在问题
- [ ] `canonical_aggregations()` 目前以 canonical 首次出现顺序输出；如果后续有人改成按字段名排序，会破坏这轮确定下来的目录审计口径。
- [ ] 未知字段当前被刻意排除在 canonical 聚合之外；如果上层误把聚合结果当成“全量输入覆盖视图”，会漏看 unresolved 输入，需要继续配合 `unknown_fields()` 一起使用。
- [ ] `deprecated_hits` 统计的是聚合内命中的解析结果是否已废弃，不区分 direct 命中废弃字段还是 alias 命中废弃字段；如果后续需要更细颗粒度，还要扩模型。

### 关闭项
- 已先复现红测失败：`cargo test --test metadata_registry_unit -- --nocapture`
- 已通过最小绿测：`cargo test --test metadata_registry_unit -- --nocapture`
- 已通过回归验证：`cargo test --test metadata_registry_unit --test metadata_scope_resolver_unit --test retrieval_engine_unit --test navigation_pipeline_integration -- --nocapture`

### 记忆点（可选）
- 方案 B3 这一轮补的是“canonical 聚合视图”，不是“未知字段清单”也不是“运行时自动规范化”；未知字段仍然只能通过 `unknown_fields()` 显式读取。
- `canonical_aggregations()` 的合同是：只聚合已解析字段、保留 canonical 首次出现顺序、保留同一 canonical 下的原始输入顺序；后续扩展不能为了摘要方便而破坏这三个约束。
## 2026-04-09
### 修改内容
- `tests/metadata_registry_unit.rs`：新增 `metadata_registry_audit_report_combines_registry_and_batch_views` 红绿测试，约束 `audit_fields(...)` 必须统一输出 governance summary、compatibility summary、field-resolution batch、batch summary、unknown-field list 与 canonical aggregation，目的是把方案A的标准 audit/export 结果对象先钉死。
- `src/ops/foundation/metadata_registry.rs`：新增 `MetadataRegistryAuditReport` 与 `MetadataRegistry::audit_fields(...)`，并在实现过程中重建整份 `metadata_registry.rs` 为干净版本，目的有两层：一是补齐方案A的统一审计视图，二是恢复前面文本层损坏后被连行注释污染的 registry 文件。
- `src/ops/foundation/metadata_registry.rs`：在重建文件时保留并恢复既有 contracts，包括 `validate_governance()`、`compatibility_summary()`、`resolve_field()`、`normalize_fields()`、`summary()`、`unknown_fields()`、`canonical_aggregations()`，目的是确保方案A是在既有目录治理能力上做收口，而不是另起一套行为。
- `docs/ai-handoff/AI_HANDOFF_MANUAL.md`、`docs/ai-memory/project-baseline.md`：补充“统一 audit/export 对象”的长期约束，明确该对象仍属于目录治理层，不进入 route / roam / retrieve 运行语义，目的是减少后续 AI 会话反复收敛这条边界的成本。

### 修改原因
- 用户已批准执行方案A，本轮目标是把 metadata registry 现有的治理摘要、兼容摘要和批量目录工具输出收敛成一个标准对象，而不是继续扩业务字段或修改主链执行逻辑。
- 在已有 `validate_governance()`、`compatibility_summary()`、`summary()`、`unknown_fields()`、`canonical_aggregations()` 的基础上，上层若要做 audit/export 仍需手工拼装多份结果；这部分应沉淀为 foundation 标准能力。
- 这轮中途发现 `metadata_registry.rs` 受文本层污染后出现“注释吞掉代码声明”的结构损坏；与其继续做脆弱的局部修补，重建文件并按现有测试回归验证是更稳的恢复方式。

### 方案还差什么?
- [ ] 还未提供不带输入批次的 registry-only 审计对象；当前 `audit_fields(...)` 仍围绕“给定字段输入批次”的目录审计场景。
- [ ] 还未把统一 audit/export 对象接入独立 CLI、report 或导出模块；当前仍是 foundation 内部标准能力。
- [ ] 还未给统一 audit/export 对象增加更细粒度冲突明细，例如 alias 冲突参与字段列表、replacement cycle 完整环路、shared replacement target 来源列表。

### 潜在问题
- [ ] `audit_fields(...)` 当前同时带 `field_resolution_batch` 和它的派生摘要视图；如果后续有人只维护摘要而忘了同步明细合同，容易出现导出结果和明细不一致的问题。
- [ ] 本轮通过“整文件重建”恢复了受损的 `metadata_registry.rs`；后续如果再对这类带历史隐藏换行的文件做原始文本替换，仍可能重演注释吞代码的问题。
- [ ] `audit_fields(...)` 仍属于目录治理只读视图；如果上层误把它当成主链入口或运行时自动规范化接口，会产生能力预期偏差。

### 关闭项
- 已先复现红测失败：`cargo test --test metadata_registry_unit -- --nocapture`
- 已通过最小绿测：`cargo test --test metadata_registry_unit -- --nocapture`
- 已通过回归验证：`cargo test --test metadata_registry_unit --test metadata_scope_resolver_unit --test retrieval_engine_unit --test navigation_pipeline_integration -- --nocapture`

### 记忆点（可选）
- 方案A这一轮补的是“统一 audit/export 对象”，不是把 registry 逻辑接进 route / roam / retrieve 主链；统一对象只是目录治理收口层。
- 如果文件里存在历史“注释和代码之间有隐藏换行”的文本状态，不要再直接用原始全文替换方式做局部文本编辑；优先用结构化重建或更小颗粒度补丁，避免再次把声明压进注释。
