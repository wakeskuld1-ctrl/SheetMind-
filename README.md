# TradingAgents

<!-- 2026-04-02 CST: 重写根 README。原因：原 README 仍以旧的 SheetMind/Excel 叙述为主，已经不能准确反映当前证券分析与投决会主链。目的：为开发者、AI 与后续维护者提供统一、可执行、可更新的项目入口说明。 -->

## 1. 项目定位

TradingAgents 当前是一套以 Rust 为主的本地化分析与工具编排工程，核心目标不是做单点脚本，而是把证券分析、研究摘要、投决建议、工具发现、Skill 门禁和运行时存储串成一条可回归、可交接、可扩展的正式主链。

当前最重要的产品方向是证券分析链，而不是旧版的 Excel/GUI 叙事。GUI、报表、历史兼容模块仍然存在，但后续新增能力默认优先围绕证券分析主链落地。

## 2. 当前主链

### 2.1 证券分析主链

- `security_analysis_contextual`
  - 面向个股 + 大盘 + 板块环境共振分析。
- `security_analysis_fullstack`
  - 面向技术面、财务快照、公告摘要、行业上下文的综合分析。
- `security_decision_briefing`
  - 当前统一的证券分析决策入口，负责把分析事实收敛成 briefing。
- `security_committee_vote`
  - 正式投决会表决入口，只消费 `security_decision_briefing` 产出的同一份 `committee_payload`。

### 2.2 当前默认行为

从 2026-04-02 起，`security_decision_briefing` 默认不再只返回分析事实，还会在顶层返回 `committee_recommendations`，直接提供三类投决建议：

- `standard`
  - 面向普通个股分析报告的默认投决建议。
- `strict`
  - 面向涉及金额、买卖动作和交易建议的严格模式。
- `advisory`
  - 面向已有持仓判断和处置建议。

这些建议不是独立写的摘要，而是 briefing 内部复用正式 `security_committee_vote` 的结果生成，因此默认报告与独立投决会工具保持同口径。

### 2.2.1 仓位计划与投后复盘最小闭环

<!-- 2026-04-08 CST: 追加证券主链执行闭环状态。原因：Task 6 已把“正式仓位计划 -> 多次调仓事件 -> 投后复盘”做成可落盘、可回读、可校验的正式 Tool 链，但 README 仍停留在只写 briefing/vote 的状态。目的：让后续 AI 和开发者明确证券主链已经进入单标的执行与复盘闭环，而不是继续把仓位建议留在对话文本里。 -->

- `security_decision_briefing`
  - 继续作为仓位计划事实源，正式输出 `position_plan`，并把同源摘要同步进入 `committee_payload.position_digest`。
- `security_position_plan_record`
  - 把 briefing 中的正式仓位计划落成可引用对象，生成 `position_plan_ref`，并绑定既有 `decision_ref / approval_ref / evidence_version`。
- `security_record_position_adjustment`
  - 允许围绕同一 `position_plan_ref` 连续记录多次调仓事件，沉淀 `adjustment_event_ref`、前后仓位、触发原因与 `plan_alignment`。
- `security_post_trade_review`
  - 只消费 `position_plan_ref + adjustment_event_refs`，从执行层回读计划与事件，做一致性校验、仓位衔接校验和轻规则复盘聚合。
- `security_execution_store`
  - 当前最小执行层存储，负责正式仓位计划与调仓事件的落盘和按 ref 回读。
- 当前已完成的闭环边界：
  - 单标的
  - 多次调仓
  - 正式 ref 串联
  - 最小一致性校验
  - 结构化投后复盘
- 当前尚未完成的扩展：
  - 把复盘结果继续装订进正式审批简报或 decision package
  - 引入收益结果、赔率结果和更细粒度盘中执行日志
  - 解决“同一日同类型多次调仓”下的版本/序号冲突

### 2.3 Foundation 通用知识标准能力（Phase 2 第一阶段）

<!-- 2026-04-08 CST: 更新 foundation 阶段说明。原因：本轮已在 Phase 1 最小导航内核之上补齐标准知识包、标准仓储和 metadata 精确过滤，但 README 仍停留在“只完成 Phase 1”的口径。目的：明确 foundation 当前做的是通用标准能力，而不是业务化知识库或证券分析适配层。 -->

- 位置：`src/ops/foundation/`
- 已完成最小闭环：`ontology_schema -> ontology_store -> capability_router -> roaming_engine -> retrieval_engine -> evidence_assembler -> navigation_pipeline`
- 已新增标准能力：
  - `KnowledgeBundle`
    - 统一封装 `schema_version / concepts / relations / nodes / edges`，作为标准知识包。
  - `KnowledgeRepository`
    - 提供 `new / save_to_path / load_from_path / save_to_layout_dir / load_from_layout_dir / to_ontology_store / to_graph_store`，作为标准仓储入口。
  - `MetadataFilter`
    - 提供基于 `node.metadata` 的 exact-match 过滤，并支持多字段 AND 与可选 `concept scope`。
  - `knowledge_ingestion`
    - 提供标准 bundle JSON 导入与 tagged-record JSONL 导入，支持重建 `KnowledgeBundle / KnowledgeRepository`。
- 已新增标准布局：
  - `bundle.json`
    - 作为标准知识包文件。
  - `repository.manifest.json`
    - 记录 `layout_version / bundle_file / schema_version / counts` 等最小布局元数据。
- 已新增元数据管理能力：
  - `MetadataSchema`
    - 统一注册 metadata 字段定义、字段类型和 allowed values。
  - `ConceptMetadataPolicy`
    - 统一声明 concept 允许字段和 required 字段。
- 当前定位：纯 foundation 通用标准能力，仍然不接 `security_analysis_* / security_decision_briefing / security_committee_vote`，也不承载业务化知识入库逻辑。
- 已锁定的契约：alias/phrase route、关系白名单漫游、候选域内检索、citation 装配、统一 `question -> NavigationEvidence` 入口、标准知识包落盘读回、metadata 多字段 AND / concept scope、标准 JSON/JSONL 导入、标准布局目录与 manifest、metadata 字段注册与 concept 字段绑定。
- 未完成项：metadata validator、schema versioning / migration、原始业务数据入库流水线、更完整的持久化目录布局与索引、向量检索、与证券分析主链的适配层。

### 2.4.1 2026-04-08 CST 补充纠偏
<!-- 2026-04-08 CST: 追加 metadata validator 阶段补充。原因：本轮已完成节点级 validator，如果只保留上一轮摘要，后续接手会误判 validator 仍未落地。目的：在不抹掉历史记录的前提下，给当前 foundation 状态一个明确纠偏。 -->

- `MetadataValidator` 已完成，并已接入 `src/ops/foundation/metadata_validator.rs`
- 当前 foundation 已具备节点级 metadata 结构化校验能力：
  - required 字段
  - concept policy 缺失
  - disallowed field
  - allowed values
  - value type
  - multi-concept compatibility
- 当前真正未完成项应收口为：
  - `schema versioning / migration`
  - repository 级批量 metadata 校验
  - 原始业务数据入库流水线
  - 更完整的持久化目录布局与索引
  - 向量检索
  - 与证券分析主链的适配层

### 2.4.2 2026-04-08 CST Schema Versioning 第一阶段补充
<!-- 2026-04-08 CST: 追加 schema versioning 第一阶段补充。原因：本轮已完成 metadata schema 的正式版本契约。目的：让后续接手方明确 validator 之后已经进入 versioning 层，而不是仍停留在“无版本 registry”。 -->

- `MetadataSchema` 已新增 `schema_version`
- 当前已具备：
  - 默认版本 `metadata-schema:v1`
  - 显式版本构造 `new_with_version(...)`
  - 空白版本拒绝
  - 最小兼容判断 `is_compatible_with(...)`
- 当前真正未完成项进一步收口为：
  - `migration contract`
  - repository 级批量版本审计
  - 更强兼容规则

### 2.4.3 2026-04-08 CST Migration Contract 第一阶段补充
<!-- 2026-04-08 CST: 追加 migration contract 第一阶段补充。原因：本轮已把字段演进对象正式纳入 metadata schema。目的：让后续接手方明确 foundation 元数据管理已经进入“字段演进治理”层。 -->

- `MetadataFieldDefinition` 已新增：
  - `deprecated`
  - `replaced_by`
  - `aliases`
- 当前已具备最小构建期合法性校验：
  - replacement target 必须已注册
  - replacement target 不能自指
  - alias 不能与字段 key 冲突
  - alias 在 schema 内必须全局唯一
- 当前真正未完成项应收口为：
  - deprecated / alias 的 validator 联动
  - repository 级批量审计
  - 真正的 migration 执行器

## 3. 关键规则

<!-- 2026-04-02 CST: 新增主链规则说明。原因：近期多轮工作都在围绕 briefing/vote 的入口统一与门禁收口。目的：把后续协作中最容易走偏的地方直接写进入口文档。 -->

- 不要绕过 `security_decision_briefing` 直接手工拼投决事实包。
- 不要让分析报告和投决会使用两套不同事实来源。
- 涉及正式表决时，只允许走 `security_decision_briefing -> security_committee_vote`。
- 如果只是拿 briefing 结果出报告，默认可直接消费 `committee_recommendations`。
- 新增证券分析能力时，优先接入正式 Tool 主链、catalog、dispatcher、Skill，而不是停留在内部函数或一次性脚本。

## 4. 主要目录

- `src/`
  - 核心 Rust 代码，含 `ops/`、`tools/`、`runtime/` 等主逻辑。
- `src/ops/foundation/`
  - 2026-04-08 CST: 新增 foundation 知识导航内核目录说明。原因：方案 C 第一阶段已经形成独立子模块，需要在项目入口明确其存在。目的：让后续协作优先在此目录定位 navigation kernel 实现。
  - 包含 ontology、route、roam、retrieve、assemble、pipeline 等最小可用组件。
- `tests/`
  - CLI、合同、端到端与行为回归测试。
- `skills/`
  - 面向 AI/Agent 的流程门禁与正式使用约束。
- `docs/`
  - 交接、计划、验收、维护规则与文档索引。
- `docs/plans/2026-04-08-foundation-standard-capabilities-phase2.md`
  - 2026-04-08 CST: 新增 foundation Phase 2 第一阶段计划/交接文档。原因：本轮新增的是通用标准能力而不是业务能力，需要单独记录范围、非目标和完成状态。目的：为后续只做标准能力的继续迭代提供统一入口。
- `docs/plans/2026-04-08-foundation-knowledge-ingestion-phase2-stage2.md`
  - 2026-04-08 CST: 新增 foundation knowledge_ingestion 阶段文档。原因：标准导入能力已成为第二阶段独立能力块，需要记录输入契约、测试口径与剩余边界。目的：为后续继续扩展 ingestion 与过滤能力提供交接入口。
- `docs/plans/2026-04-08-foundation-metadata-filter-phase2-stage3.md`
  - 2026-04-08 CST: 新增 foundation metadata filter 阶段文档。原因：过滤能力已从单字段 exact-match 扩展到多字段 AND 与 concept scope，需要单独沉淀契约和边界。目的：为后续继续扩展过滤与持久化标准提供交接入口。
- `docs/plans/2026-04-08-foundation-repository-layout-phase2-stage4.md`
  - 2026-04-08 CST: 新增 foundation repository layout 阶段文档。原因：repository 已进入布局标准化阶段，需要记录标准目录、manifest 和替换式写入边界。目的：为后续继续扩展持久化标准提供交接入口。
- `docs/plans/2026-04-08-foundation-metadata-schema-registry-phase3-stage1.md`
  - 2026-04-08 CST: 新增 foundation metadata schema registry 阶段文档。原因：整体目标开始从“标准能力底座”进入“元数据管理层”，需要记录字段注册和 concept policy 契约。目的：为后续 validator 和 versioning 提供交接入口。
- `docs/plans/2026-04-08-foundation-metadata-validator-phase3-stage2.md`
  - 2026-04-08 CST: 新增 foundation metadata validator 阶段文档。原因：schema registry 已完成定义层能力，需要把约束真正执行到节点 metadata 上。目的：为后续 schema versioning 提供接手入口。
- `CHANGELOG_TASK.MD`
  - 按任务维度沉淀修改内容、风险、未完成项和记忆点。

## 5. 文档入口

- [AI_HANDOFF.md](/E:/TradingAgents/TradingAgents/docs/AI_HANDOFF.md)
  - 给后续 AI / 开发者的统一接手手册。
- [DOCUMENTATION_INDEX.md](/E:/TradingAgents/TradingAgents/docs/DOCUMENTATION_INDEX.md)
  - 当前 docs 目录的导航入口。
- [DOC_UPDATE_POLICY.md](/E:/TradingAgents/TradingAgents/docs/DOC_UPDATE_POLICY.md)
  - 文档必须如何随功能同步更新的规则。
- [CHANGELOG_TASK.MD](/E:/TradingAgents/TradingAgents/CHANGELOG_TASK.MD)
  - 每次开发任务完成后的任务日志。

## 6. 常用命令

```powershell
cargo test --test security_analysis_resonance_cli -- --nocapture
cargo test --test security_committee_vote_cli -- --nocapture
cargo test --test integration_tool_contract -- --nocapture
cargo test --test ontology_schema_unit --test ontology_store_unit --test knowledge_record_unit --test knowledge_graph_store_unit --test capability_router_unit --test roaming_engine_unit --test retrieval_engine_unit --test evidence_assembler_unit --test navigation_pipeline_integration --test knowledge_bundle_unit --test knowledge_repository_unit -- --nocapture
cargo test --test knowledge_bundle_unit --test knowledge_repository_unit --test knowledge_ingestion_unit -- --nocapture
cargo test --test knowledge_repository_unit --test knowledge_ingestion_unit --test knowledge_bundle_unit -- --nocapture
cargo test --test metadata_schema_registry_unit --test knowledge_repository_unit --test knowledge_ingestion_unit --test knowledge_bundle_unit -- --nocapture
```

如果要验证默认 briefing 是否已经内嵌三种投决建议，可优先跑：

```powershell
cargo test --test security_analysis_resonance_cli security_decision_briefing_includes_default_committee_recommendations_for_all_modes -- --nocapture
```

如果要验证当前证券主链的“仓位计划 -> 调仓记录 -> 投后复盘”最小闭环，可优先跑：

```powershell
cargo test --test security_analysis_resonance_cli security_position_plan_record_persists_briefing_plan -- --nocapture
cargo test --test security_committee_vote_cli security_record_position_adjustment_supports_multiple_events -- --nocapture
cargo test --test security_analysis_resonance_cli security_post_trade_review -- --nocapture
```

## 7. 维护约定

- 每次功能改动后，同步检查 README、AI 交接手册、任务日志是否需要更新。
- 新增正式 Tool 时，至少同步检查 `catalog`、`dispatcher`、相关 Skill、合同测试和交接文档。
- 如果用户纠正了流程或口径，要把纠正沉淀进 `CHANGELOG_TASK.MD` 的“记忆点”。
- 不要把历史旧文档当作最新权威入口，当前统一入口以本 README 和 `docs/AI_HANDOFF.md` 为准。
