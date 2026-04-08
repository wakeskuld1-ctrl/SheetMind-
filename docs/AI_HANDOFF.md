# AI 交接手册

<!-- 2026-04-02 CST: 新增统一 AI 交接手册。原因：仓库里已有两份历史交接摘要，内容分散且有旧主线残留。目的：把当前正式主链、门禁规则、接手顺序和维护要求集中到一份权威手册。 -->

## 1. 当前应该怎么理解这个仓库

这个仓库当前最重要的正式产品链是证券分析链，而不是早期的 Excel/GUI 叙事。后续 AI 接手时，默认应优先从证券分析、briefing、投决会、Skill 门禁和合同测试这条线继续推进，除非用户明确要求处理其他模块。

## 2. 当前正式主链

### 2.1 分析层

- `technical_consultation_basic`
  - 单证券技术面分析。
- `security_analysis_contextual`
  - 证券 + 大盘 + 板块环境共振分析。
- `security_analysis_fullstack`
  - 技术面 + 财务快照 + 公告摘要 + 行业上下文的综合分析。
- `signal_outcome_research`
  - 信号结果研究与历史类比沉淀。
  - 现在已向上游正式输出赔率层所需的数值摘要：`win/loss/flat rate`、`avg win/loss return`、`payoff_ratio`、`expectancy`。

### 2.2 决策层

- `security_decision_briefing`
  - 当前统一证券决策入口。
  - 输出结构化 briefing、`committee_payload`、默认 `committee_recommendations`。
  - 现在已新增 `odds_brief` 与 `position_plan`，并把同源摘要同步进 `committee_payload.odds_digest / position_digest`。
- `security_committee_vote`
  - 正式投决会入口。
  - 只消费 briefing 产出的同一份 `committee_payload`。

### 2.2.1 执行与复盘层

<!-- 2026-04-08 CST: 追加证券执行与复盘层交接。原因：Task 6 已把仓位计划、调仓事件和投后复盘打通成正式 Tool 链，如果手册仍只写 briefing/vote，后续 AI 会把执行链误判成未建设或继续留在对话层。目的：明确证券主链已经具备单标的最小执行闭环。 -->

- `security_position_plan_record`
  - 把 `security_decision_briefing.position_plan` 落成正式计划对象。
  - 生成 `position_plan_ref`，并与 `decision_ref / approval_ref / evidence_version` 对齐。
- `security_record_position_adjustment`
  - 记录围绕同一 `position_plan_ref` 的实际调仓事件。
  - 输出 `adjustment_event_ref / before_position_pct / after_position_pct / trigger_reason / plan_alignment`。
- `security_post_trade_review`
  - 只消费 `position_plan_ref + adjustment_event_refs`。
  - 会回读正式计划与事件对象，做引用一致性、日期顺序和仓位衔接校验，并输出结构化复盘结论。
- `src/runtime/security_execution_store.rs`
  - 当前执行层正式事实存储。
  - 负责 `position_plan` 与 `adjustment_event` 的落盘和按 ref 回读。

### 2.3 默认投决语义

- `standard`
  - 普通个股分析报告默认投决建议。
- `strict`
  - 涉及金额与买卖动作的严格交易建议。
- `advisory`
  - 已有持仓判断与持仓处置建议。

### 2.4 Foundation 通用知识标准能力（Phase 2 第一阶段）

<!-- 2026-04-08 CST: 更新 foundation 接手说明。原因：当前仓库已经在 navigation kernel 之上补齐标准知识包、标准仓储与 metadata 精确过滤，如果仍按 Phase 1 口径交接，后续 AI 会误判缺口和边界。目的：明确 foundation 线当前只做通用标准能力，不做业务化知识库。 -->

- 代码位置：`src/ops/foundation/`
- 当前闭环：`ontology_schema -> ontology_store -> capability_router -> roaming_engine -> retrieval_engine -> evidence_assembler -> navigation_pipeline`
- 当前已完成：
  - `KnowledgeBundle`
    - 标准知识包结构，统一承载 ontology 与 graph 原始数据。
  - `KnowledgeRepository`
    - 标准仓储入口，支持构建、单文件保存/读回、标准布局目录保存/读回和重建查询视图。
  - `MetadataFilter`
    - 支持基于 `node.metadata` 的 exact-match、多字段 AND 与可选 `concept scope` 过滤。
  - `knowledge_ingestion`
    - 支持标准 bundle JSON 与 tagged-record JSONL 导入，并可直接构建 `KnowledgeRepository`。
- 当前已新增：
  - `MetadataSchema`
    - 支持 metadata 字段定义、字段类型与 allowed values 注册。
  - `ConceptMetadataPolicy`
    - 支持 concept 允许字段和 required 字段绑定。
- 当前状态：已具备 `question -> NavigationEvidence` 内存闭环 + 标准包/仓储最小持久化闭环 + 标准 JSON/JSONL 导入 + 标准布局目录，但仍是独立 foundation 通用内核。
- 当前未接入：`security_analysis_*`、`security_decision_briefing`、`security_committee_vote`
- 当前未完成：知识入库流水线、持久化目录布局与索引、向量检索、高级 metadata filter、证券分析适配层

### 2.4.1 2026-04-08 CST 补充纠偏
<!-- 2026-04-08 CST: 追加 metadata validator 阶段补充。原因：本轮已完成节点级 validator，而上一版接手口径仍停留在 schema registry。目的：让后续 AI 直接看到当前治理层已经进入执行校验阶段。 -->

- `MetadataValidator` 已完成，并已导出到 `foundation` 模块边界
- 当前 foundation 已具备节点级 metadata 结构化校验能力：
  - required 字段
  - concept policy 缺失
  - disallowed field
  - allowed values
  - value type
  - multi-concept compatibility
- 当前 foundation 线下一优先级应更新为：
  - `schema versioning / migration`
  - repository 级批量 metadata 校验
  - 更强 registry 治理对象

### 2.4.2 2026-04-08 CST Schema Versioning 第一阶段补充
<!-- 2026-04-08 CST: 追加 schema versioning 第一阶段补充。原因：本轮已把 MetadataSchema 从“无版本 registry”推进到“有正式版本契约的 registry”。目的：让后续 AI 接手时直接从 migration contract 往下接。 -->

- `MetadataSchema` 已新增正式字段 `schema_version`
- 当前默认版本为 `metadata-schema:v1`
- 当前已新增：
  - `MetadataSchema::new_with_version(...)`
  - `MetadataSchema::is_compatible_with(...)`
  - `MetadataSchemaError::InvalidSchemaVersion`
- 当前 versioning 仍停留在第一阶段：
  - 只做精确版本匹配
  - 不做 migration 执行
  - 不做 deprecated / replaced_by / alias

### 2.4.3 2026-04-08 CST Migration Contract 第一阶段补充
<!-- 2026-04-08 CST: 追加 migration contract 第一阶段补充。原因：本轮已正式建模字段演进对象。目的：让后续 AI 直接从 validator 联动或 repository 审计继续推进。 -->

- `MetadataFieldDefinition` 已正式承载：
  - `deprecated`
  - `replaced_by`
  - `aliases`
- 当前 schema 构建期已经会拦截：
  - unknown replacement target
  - self replacement target
  - alias conflict
- 当前仍未进入：
  - alias 执行层解析
  - deprecated validator 联动
  - 自动迁移执行器

## 3. 绝对不要再走回头路的点

<!-- 2026-04-02 CST: 新增“禁止回退”部分。原因：近期多轮纠偏都集中在入口分叉、事实包分叉和旧主线误读。目的：给后续 AI 一个明确的禁区列表。 -->

- 不要绕过正式 Tool 主链回退成自由拼装分析。
- 不要直接手工构造 `committee_payload`。
- 不要让报告使用一套事实、投决会再使用另一套事实。
- 不要再把正式仓位计划、调仓记录和投后复盘只留在对话文本里，而不落成可回读对象。
- 不要只把能力做成内部函数而不接 `catalog`、`dispatcher`、Skill 和回归测试。
- 不要把旧的 Excel/GUI 文档误当成当前项目主线。
- 不要把历史切片测试通过误写成“整仓全绿”。

## 4. 当前必须知道的入口文件

- `src/ops/security_decision_briefing.rs`
- `src/ops/security_committee_vote.rs`
- `src/ops/security_analysis_contextual.rs`
- `src/ops/security_analysis_fullstack.rs`
- `src/ops/signal_outcome_research.rs`
- `src/ops/security_position_plan_record.rs`
- `src/ops/security_record_position_adjustment.rs`
- `src/ops/security_post_trade_review.rs`
- `src/tools/catalog.rs`
- `src/tools/dispatcher.rs`
- `src/tools/dispatcher/stock_ops.rs`
- `src/runtime/security_execution_store.rs`
- `skills/security-analysis-v1/SKILL.md`
- `skills/security-decision-briefing-v1/SKILL.md`
- `skills/security-committee-v1/SKILL.md`
- `tests/security_analysis_resonance_cli.rs`
- `tests/security_committee_vote_cli.rs`
- `tests/integration_tool_contract.rs`

## 5. 推荐接手顺序

1. 先看 [README.md](/E:/TradingAgents/TradingAgents/README.md)。
2. 再看本手册，确认当前主链和禁区。
3. 再看 `CHANGELOG_TASK.MD` 最近两到三段，确认最新修改、风险和记忆点。
4. 再按需求打开对应 `src/ops/...`、`tests/...` 和 `skills/...` 文件。
5. 真要新增能力时，优先补测试，再改实现，再补文档和任务日志。

## 6. 当前文档同步规则

- 用户每次要求修改功能后，都要评估是否同步更新：
  - `README.md`
  - `docs/AI_HANDOFF.md`
  - `CHANGELOG_TASK.MD`
- 如果是正式 Tool、Skill、流程门禁、合同字段变更，通常三者都要更新。
- 如果只是局部实现细节变更，至少要补 `CHANGELOG_TASK.MD`。

## 7. 最近已确认的状态

### 7.1 投决会链路

- `security_committee_vote` 已被正式 catalog 收录并可通过 dispatcher 调用。
- 显式 `tool_catalog` 请求与空输入目录返回已经统一。
- `security_decision_briefing -> committee_payload -> security_committee_vote` 已有端到端回归。

### 7.2 默认投决建议

- `security_decision_briefing` 默认内嵌 `committee_recommendations`。
- `standard / strict / advisory` 三种模式已一并准备。
- 默认建议复用正式 `security_committee_vote` 结果，不是手写摘要。

### 7.2.1 赔率层与仓位层

- `signal_outcome_research_summary` 现在已补齐赔率系统 V1 所需数值字段，不需要再新开平行复盘模块。
- `security_decision_briefing` 现在会正式输出：
  - `odds_brief`
  - `position_plan`
- `committee_payload` 现在同步承载：
  - `odds_digest`
  - `position_digest`
- 当前规则仍是 V1 轻量分档版：
  - 赔率层按 `win_rate / payoff_ratio / expectancy / sample_count` 分档。
  - 仓位层按 `odds_grade + historical_confidence + resonance_score + execution_plan` 分档。
- 当前边界：
  - 不做 Kelly 全公式。
  - 不做组合级相关性管理。
  - 不做 ETF/海外资产专用赔率模板。

### 7.2.2 仓位计划、调仓记录与投后复盘闭环

<!-- 2026-04-08 CST: 追加 Task 6 状态收口。原因：证券主链已经从“只给仓位建议”推进到“正式计划对象 -> 多次调仓事件 -> 投后复盘聚合”的单标的闭环。目的：让后续 AI 从当前闭环上继续接审批、审计和收益结果，而不是回退到临时文本记录。 -->

- `security_position_plan_record` 已正式落地：
  - 可把 briefing 中的 `position_plan` 落成正式对象
  - 会返回 `position_plan_ref`
  - 会绑定 `decision_ref / approval_ref / evidence_version`
- `security_record_position_adjustment` 已正式落地：
  - 可围绕同一 `position_plan_ref` 连续记录多次事件
  - 会返回 `adjustment_event_ref`
  - 会保留 `before_position_pct / after_position_pct / trigger_reason / plan_alignment`
- `security_post_trade_review` 已正式落地：
  - 可只凭 `position_plan_ref + adjustment_event_refs` 执行复盘
  - 不要求调用方重复提交完整计划和完整事件对象
  - 当前会做：
    - `position_plan_ref` 一致性校验
    - `symbol / decision_ref / approval_ref / evidence_version` 一致性校验
    - `event_date` 顺序校验
    - 相邻事件 `after_position_pct -> before_position_pct` 仓位衔接校验
    - 轻规则复盘维度聚合
- 当前闭环边界已明确为：
  - 单标的
  - 多次调仓
  - 正式 ref 回读
  - 最小一致性校验
  - 结构化投后复盘
- 当前还没完成的下一层资产化能力：
  - 复盘结果尚未正式装订进审批简报对象或 decision package
  - 尚未接入收益表现、赔率表现、信号结果研究层
  - 尚未覆盖更细粒度盘中执行日志、滑点或多版本计划并存

### 7.3 Foundation Phase 2 第一阶段状态

- `src/ops/foundation/` 已完成 ontology、route、roam、retrieve、assemble、pipeline 七段最小实现，并补齐 `knowledge_bundle`、`knowledge_repository` 两个标准能力模块。
- 对应回归已覆盖：`ontology_schema_unit`、`ontology_store_unit`、`knowledge_record_unit`、`knowledge_graph_store_unit`、`capability_router_unit`、`roaming_engine_unit`、`retrieval_engine_unit`、`evidence_assembler_unit`、`navigation_pipeline_integration`、`knowledge_bundle_unit`、`knowledge_repository_unit`、`knowledge_ingestion_unit`。
- 当前 `KnowledgeNode` 已统一携带 `metadata`，foundation 可通过 `MetadataFilter` 做 exact-match 过滤。
- 当前 `MetadataFilter` 已支持多字段 AND 与可选 `concept scope`，过滤能力仍保持在通用标准层，不包含 DSL。
- 当前 `knowledge_ingestion` 已支持两类标准输入：完整 `KnowledgeBundle` JSON 与单文件 tagged-record JSONL。
- 当前 `KnowledgeRepository` 已支持标准布局目录：`bundle.json + repository.manifest.json`。
- 当前 `MetadataSchema` 已开始把 metadata 从“字符串 map”提升为“字段注册 + concept 绑定”的正式管理对象。
- 当前应把它理解为“可复用通用标准能力”，而不是“已接入证券分析主链的完整知识库系统”。

## 8. 当前已知风险

- `security_decision_briefing` 现在会额外生成三次 vote 结果，后续若 vote 逻辑继续变重，要注意耗时。
- 仓库里仍然存在较多历史 `dead_code` warning，这次没有混做清理。
- 当前 Skill 门禁主要是流程文档级约束，若有人绕过 Skill 直接拼 Tool，仍需要测试和 review 兜底。
- foundation 当前虽然已具备标准包、标准仓储、标准布局目录、metadata 多字段 AND / concept scope 过滤、标准 JSON/JSONL 导入和 metadata schema registry，但仍缺 validator、versioning / migration、原始业务数据入库流水线、更完整的持久化索引/目录布局与向量检索，不能把它误读成“完整知识库”。
- Windows 下 `cargo test` 偶发会受残留 `excel_skill.exe` / `cargo` 进程占用影响，出现 `os error 5`；长跑测试前应先清残留进程。
- `security_execution_store` 当前使用独立 SQLite 文件 `security_execution.db`；如果未来要求统一进单一 runtime.db，需要单独迁移方案。
- 当前同日同类型调仓事件仍可能发生 ref 冲突；若进入“同一日多次 reduce/add”场景，需要补版本号或序号策略。
- 当前 `security_post_trade_review` 仍是轻规则复盘，尚未纳入真实收益、赔率兑现和更细的执行质量指标。

## 9. 后续优先级建议

- 优先阅读 [2026-04-08-closed-loop-investment-research-roadmap.md](/E:/TradingAgents/TradingAgents/docs/plans/2026-04-08-closed-loop-investment-research-roadmap.md)，后续证券主链默认按“闭环优先”推进，而不是继续深挖投决会制度层。
- 继续推进赔率/仓位时，先看 [2026-04-08-odds-position-system-design.md](/E:/TradingAgents/TradingAgents/docs/plans/2026-04-08-odds-position-system-design.md)，不要脱离 `signal_outcome_research` 另起平行研究模块。
- 继续推进执行闭环时，先看 [2026-04-08-security-post-trade-review-position-management.md](/E:/TradingAgents/TradingAgents/docs/plans/2026-04-08-security-post-trade-review-position-management.md)，不要把正式计划/调仓/复盘重新退回对话层。
- 证券主链下一阶段的正式优先级应为：`复盘结果资产化/审批绑定 -> 信号结果研究层接入复盘 -> 更细粒度执行日志 -> 组合层仓位管理 -> 市场结构层 -> 技术面平衡计分卡 -> 深层信息面增强`。
- `security_committee_vote` 当前可视为“最小可用收口版”已完成，后续只做必要收口，不再默认吞噬主线资源。
- `signal_outcome_research` 应被视为后续最重要的正式扩展点，因为赔率、胜率、最大回撤和投后复盘都应从这条线长出来。
- 先补更强的 `strict / advisory` 端到端回归。
- 再补授权开启时的 `tool_catalog` 目录门禁回归。
- 如果后续报告模板要进一步产品化，再把三类 committee 建议更明确地落进中文报告排版层。
- 如果继续推进 foundation 线，下一优先级应是“metadata validator + schema versioning”先于“直接接证券分析主链”，否则元数据管理层仍然缺最关键的治理闭环。
- Foundation 线后续默认只做通用标准能力，除非用户重新明确批准，否则不要自行扩展业务化知识库能力。

### 9.1 2026-04-08 CST Foundation 更新

- `MetadataValidator` 已经把 `MetadataSchema + ConceptMetadataPolicy` 从“治理定义层”推进到了“节点执行层”
- 当前风险描述应更新为：foundation 仍然不等于“完整知识库”，但它已经不再缺 validator，真正缺的是 versioning / migration 和更完整的批量治理入口

### 9.2 2026-04-08 CST Foundation 下一阶段候选
<!-- 2026-04-08 CST: 追加 foundation 下一阶段候选方案。原因：migration contract 落地后，后续实现路径已经出现分叉。目的：给后续 AI 一个明确的默认推荐顺序，避免误入业务化或过早做执行器。 -->

- 候选方案 A：`Validator` 联动
  - 让 `deprecated / aliases / replaced_by` 进入节点级 `MetadataValidationIssue`
  - 这是当前默认推荐路线，因为它与现有 `MetadataValidator` 最连续、改动面最小
- 候选方案 B：Repository-Level Audit
  - 做整库批量扫描，输出 deprecated 字段使用清单与可迁移清单
  - 适合放在方案 A 之后，基于节点级 issue 再做聚合
- 候选方案 C：Migration Executor
  - 仅在后续继续获批时再进入 dry-run / rewrite 级执行层
  - 当前不要提前跳到自动重写 metadata
- 当前推荐顺序：
  - 先方案 A
  - 再视需要进入方案 B
  - 最后才考虑方案 C
