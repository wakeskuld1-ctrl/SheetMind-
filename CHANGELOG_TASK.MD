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
