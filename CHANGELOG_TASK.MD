## 2026-04-08
### 修改内容
- `tests/metadata_migration_contract_unit.rs`：新增 4 条红测，锁定字段演进契约注册、unknown replacement target、自指 replacement 和 alias 冲突这 4 条最小 migration contract 契约。
- `src/ops/foundation/metadata_schema.rs`：为 `MetadataFieldDefinition` 新增 `deprecated / replaced_by / aliases`，并新增对应 builder 与 `UnknownReplacementTarget / SelfReplacementTarget / DuplicateFieldAlias` 构建期错误，把字段演进正式纳入 metadata schema 治理对象。
- `src/ops/security_decision_briefing.rs`：补最小 `f64` 字面量类型标注，目的仅为解除当前工具链下的编译阻塞，保证 foundation 元数据测试能继续执行。
- `README.md`、`docs/AI_HANDOFF.md`、`docs/DOCUMENTATION_INDEX.md`、`docs/plans/2026-04-08-foundation-metadata-migration-contract-phase3-stage4.md`、`task_plan.md`、`progress.md`、`findings.md`：同步更新 foundation 当前口径，明确 migration contract 第一阶段已完成。

### 修改原因
- 用户已批准按方案A推进 `Migration Contract` 第一阶段，范围只做字段演进治理对象和构建期合法性校验，不做执行层。
- 如果没有 `deprecated / replaced_by / aliases` 这些正式契约，后续 validator 联动、repository 审计和 migration executor 都缺少统一输入对象。
- 测试过程中暴露 `security_decision_briefing.rs` 现存编译阻塞，如果不先做最小修复，连 foundation 元数据相关红绿回归都无法继续。

### 方案还差什么?
- [ ] 还没有实现 deprecated / alias 的 validator 联动。
- [ ] 还没有实现 repository 级批量演进审计。
- [ ] 还没有实现真正的 migration executor。

### 潜在问题
- [ ] 当前 alias 只做 contract 注册和冲突校验，还没有真正接入任何字段解析链路；如果上层误以为 alias 已经生效，会出现预期偏差。
- [ ] 当前 `replaced_by` 只是声明式关系，没有定义多跳替换、链式替换或冲突覆盖策略。
- [ ] Windows 下构建锁问题仍会反复出现，后续长跑回归前仍要注意清理残留 `excel_skill/cargo/rustc` 进程。

### 关闭项
- 已通过 `cargo test --test metadata_migration_contract_unit -- --nocapture`
- 已通过 `cargo test --test metadata_schema_registry_unit --test metadata_schema_versioning_unit --test metadata_migration_contract_unit --test metadata_validator_unit --test knowledge_repository_unit --test knowledge_ingestion_unit --test knowledge_bundle_unit -- --nocapture`

### 记忆点（可选）
- 用户当前主线仍是“本体论元数据管理”，所以 versioning 之后继续沿 migration contract、validator 联动、repository 审计往前走，不要跳回业务主链。

## 2026-04-08
### 修改内容
- `tests/metadata_schema_versioning_unit.rs`：新增 4 条红测，锁定默认版本、显式版本、空白版本拒绝和精确版本兼容性四条最小 versioning 契约。
- `src/ops/foundation/metadata_schema.rs`：新增 `schema_version`、`DEFAULT_METADATA_SCHEMA_VERSION`、`MetadataSchema::new_with_version(...)`、`MetadataSchema::is_compatible_with(...)` 和 `MetadataSchemaError::InvalidSchemaVersion`，把 metadata schema 推进到正式版本契约阶段。
- `README.md`、`docs/AI_HANDOFF.md`、`docs/DOCUMENTATION_INDEX.md`、`docs/plans/2026-04-08-foundation-metadata-schema-versioning-phase3-stage3.md`、`task_plan.md`、`progress.md`、`findings.md`：同步更新 foundation 当前口径，明确 versioning 第一阶段已完成，下一优先级转向 migration contract。

### 修改原因
- 用户已批准继续按方案 A 推进 `Schema Versioning` 第一阶段，而不是直接进入 migration 执行器或 repository 级批量治理。
- 如果 `MetadataSchema` 仍然没有正式版本契约，后续 deprecated / replaced_by / migration 都没有稳定锚点，validator 也无法与版本演进规则衔接。

### 方案还差什么?
- [ ] 还没有实现 migration contract。
- [ ] 还没有实现 repository 级批量版本审计。
- [ ] 还没有实现 deprecated / replaced_by / alias 等字段演进治理对象。

### 潜在问题
- [ ] 当前 `is_compatible_with(...)` 只做精确匹配，如果后续需要主版本兼容或向后兼容，现有规则需要扩展。
- [ ] 当前只校验空白版本，没有约束更细的版本格式；如果后续版本命名变散，可能需要增加格式校验。
- [ ] 当前 versioning 还没有和 repository manifest、ingestion 输入头或批量审计绑定，因此版本治理仍停留在 schema 对象层。

### 关闭项
- 已通过 `cargo test --test metadata_schema_versioning_unit -- --nocapture`
- 已通过 `cargo test --test metadata_schema_registry_unit --test metadata_schema_versioning_unit --test metadata_validator_unit --test knowledge_repository_unit --test knowledge_ingestion_unit --test knowledge_bundle_unit -- --nocapture`

### 记忆点（可选）
- 用户当前主线仍然是“本体论元数据管理”，所以 validator 之后继续沿 versioning / migration 往前走，不要中途切回业务主链或泛化到整库治理。

## 2026-04-08
### 修改内容
- `docs/plans/2026-04-08-odds-position-system-design.md`：新增“基于现有 `signal_outcome_research` 的赔率系统与仓位管理设计”，明确研究层复用边界、briefing 顶层新字段、committee 同源摘要与最小改动面。
- `src/ops/signal_outcome_research.rs`：扩展 `SignalOutcomeResearchSummaryResult`，新增 `loss/flat rate`、`avg win/loss return`、`payoff_ratio_10d`、`expectancy_10d` 等赔率层数值，并补 `average_option / ratio_option` 辅助函数。
- `src/ops/security_decision_briefing.rs`：新增 `OddsBrief`、`PositionPlan`、`committee_payload.odds_digest / position_digest`，并把 `historical_digest -> odds_brief -> position_plan` 收口成同一条装配链。
- `tests/security_analysis_resonance_cli.rs`：新增 `security_decision_briefing_exposes_odds_and_position_layers_after_analog_study` 红绿回归，并把 `committee_payload` 结构断言扩展到 `odds_digest / position_digest`。
- `tests/integration_tool_contract.rs`：更新 `security_decision_briefing` 对外合同样例，补 `odds_brief / position_plan / odds_digest / position_digest`，并添加 `#![recursion_limit = "256"]` 解决大体量 `json!` 合同样例的宏展开层级问题。
- `tests/security_committee_vote_cli.rs`：更新 `CommitteePayload` 正样本夹具，使 vote 测试继续消费完整的新事实包。
- `docs/DOCUMENTATION_INDEX.md`、`docs/AI_HANDOFF.md`：登记新设计文档，并把赔率层/仓位层已接入主链的事实写进 AI 交接手册。

### 修改原因
- 用户已明确批准“方案 B：研究层 + 决策层分层版”，要求基于既有 `signal_outcome_research` 继续实现赔率系统与仓位管理，不允许重复开发平行复盘模块。
- 如果只停留在 `historical_digest`，平台仍然只有历史研究事实，没有正式的赔率层与仓位层，闭环主线无法向投中环节推进。

### 方案还差什么?
- [ ] 还没有把 `odds_digest / position_digest` 真正接入 `security_committee_vote` 的席位规则，只是先完成同源承载。
- [ ] 还没有实现投后复盘层，把本次赔率与仓位建议和未来真实结果自动对账。
- [ ] 还没有补 ETF / 海外资产的专用赔率模板，当前仍以个股银行 study_key 为主。

### 潜在问题
- [ ] `payoff_ratio_10d` 在样本没有负收益时会合法返回 `None`，调用方不能把它误当成“字段缺失”。
- [ ] `position_plan` 当前是规则分档版，不是 Kelly、风险预算或组合级相关性管理，不能误读成完整仓位引擎。
- [ ] Windows 下长跑 `cargo test` 仍可能受到目标文件占用影响；这轮已改为串行定点回归，整文件长跑时仍需注意 `os error 5 / LNK1104`。

### 关闭项
- 已通过 `cargo test --test integration_tool_contract -- --nocapture --test-threads=1`
- 已通过 `cargo test --test security_committee_vote_cli -- --nocapture --test-threads=1`
- 已通过 `cargo test --test security_analysis_resonance_cli security_decision_briefing_exposes_available_historical_digest_after_analog_study -- --nocapture --test-threads=1`
- 已通过 `cargo test --test security_analysis_resonance_cli security_decision_briefing_exposes_odds_and_position_layers_after_analog_study -- --nocapture --test-threads=1`
- 已通过 `cargo fmt --all`

### 记忆点（可选）
- 用户已明确纠正：赔率系统和仓位管理必须建立在现有 `signal_outcome_research` 之上，不能因为要做闭环就新开第二套“历史复盘/赔率模块”。

## 2026-04-08
### 修改内容
- `tests/metadata_validator_unit.rs`：新增 5 条红测，锁定 required 字段缺失、concept policy 缺失、disallowed field、allowed values/value type 错误，以及 multi-concept compatibility 的最小执行契约。
- `src/ops/foundation/metadata_validator.rs`：新增 `MetadataValidator` 与 `MetadataValidationIssue`，把 metadata schema registry 的治理定义真正执行到 `KnowledgeNode.metadata` 节点级校验上，并统一输出结构化 issue。
- `src/ops/foundation.rs`：导出 `metadata_validator` 模块，使 validator 成为 foundation 正式模块边界的一部分。
- `README.md`、`docs/AI_HANDOFF.md`、`docs/DOCUMENTATION_INDEX.md`、`docs/plans/2026-04-08-foundation-metadata-validator-phase3-stage2.md`、`task_plan.md`、`progress.md`、`findings.md`：同步更新 foundation 当前口径，明确 metadata validator 已完成，下一优先级转为 schema versioning / migration。

### 修改原因
- 用户已批准继续按方案 B 推进“本体论元数据管理”的下一阶段，范围收口为节点级 metadata validator，而不是继续只停留在 schema registry 定义层。
- 如果只有 `MetadataSchema + ConceptMetadataPolicy` 而没有执行层 validator，元数据管理仍然缺少最关键的治理闭环，后续 versioning 也缺少稳定落点。

### 方案还差什么?
- [ ] 还没有实现 schema versioning / migration。
- [ ] 还没有实现 repository 级批量 metadata 校验。
- [ ] 还没有支持 deprecated / replaced_by / field alias / concept inheritance 等演进治理对象。

### 潜在问题
- [ ] 当前 `MetadataValidator` 只做节点级校验，不会一次性扫描整个 repository；如果上层误以为已经具备批量治理能力，会出现使用边界偏差。
- [ ] 当前 `KnowledgeNode.metadata` 仍是字符串 map，`Integer / Float / Boolean` 校验依赖字符串解析；如果后续引入更复杂类型，现有约束需要升级。
- [ ] 当前多 concept 语义是“字段逐 concept 兼容、required 取并集”；如果后续扩展 concept inheritance 或覆盖规则，这一语义可能需要重新定义。

### 关闭项
- 已通过 `cargo test --test metadata_validator_unit -- --nocapture`
- 已通过 `cargo test --test metadata_schema_registry_unit --test metadata_validator_unit --test knowledge_repository_unit --test knowledge_ingestion_unit --test knowledge_bundle_unit -- --nocapture`

### 记忆点（可选）
- 用户已明确：当前整体计划聚焦“本体论元数据管理”，优先级应沿着 `registry -> validator -> versioning` 推进，不要回到只围绕标准包/仓储层打转。

## 2026-04-08
### 修改内容
- 在 `tests/metadata_schema_registry_unit.rs` 中新增 3 条红测，分别锁定 metadata 字段定义注册、concept 允许字段/required 字段绑定，以及 concept policy 引用未知字段时的失败边界。
- 在 `src/ops/foundation/metadata_schema.rs` 中新增 `MetadataValueType`、`MetadataFieldDefinition`、`ConceptMetadataPolicy`、`MetadataSchema` 与 `MetadataSchemaError`，把 metadata 管理从“map 存值”推进到“字段注册 + concept 绑定”的正式对象层。
- 在 `src/ops/foundation.rs` 中导出 `metadata_schema` 模块，使 metadata schema registry 成为 foundation 正式模块边界的一部分。
- 同步更新 `README.md`、`docs/AI_HANDOFF.md`、`docs/DOCUMENTATION_INDEX.md`、`docs/plans/2026-04-08-foundation-metadata-schema-registry-phase3-stage1.md`、`task_plan.md`、`progress.md`、`findings.md`，把 foundation 当前状态更新为“开始进入元数据管理层”。

### 修改原因
- 用户明确表示现在说的是“整体计划里的本体论元数据管理”，而不是只补底座能力，因此需要从标准包/仓储/过滤阶段继续推进到 registry 管理层。
- 如果没有字段注册和 concept 绑定，后续 validator、versioning、migration 和治理动作都没有统一输入对象，整体元数据管理系统就无法成立。

### 方案还差什么?
- [ ] 还没有实现 metadata validator。
- [ ] 还没有实现 schema versioning / migration。
- [ ] 还没有支持 deprecated / replaced_by、field alias、concept inheritance。

### 潜在问题
- [ ] 当前 `MetadataValueType` 只是注册层类型枚举，还没有真正驱动节点 metadata 值校验。
- [ ] 当前 concept policy 只定义了 allowed/required 字段，还没有定义字段默认值、枚举覆盖或 concept 层级继承。
- [ ] 当前 registry 已经建立，但还没有和 `KnowledgeRepository` / `knowledge_ingestion` 形成执行层联动，因此仍需下一阶段 validator 收口。

### 关闭项
- 已通过 `cargo test --test metadata_schema_registry_unit -- --nocapture`

### 记忆点（可选）
- 用户明确纠正后续方向是“整体本体论元数据管理”，所以这条线接下来应该优先做 registry、validator、versioning，而不是继续只在标准包/仓储层打转。

## 2026-04-08
### 修改内容
- 在 `tests/knowledge_repository_unit.rs` 中新增 `knowledge_repository_persists_standard_layout_directory` 与 `knowledge_repository_loads_repository_from_standard_layout_directory` 两条红测，先钉住标准布局目录与 manifest 契约。
- 在 `src/ops/foundation/knowledge_repository.rs` 中新增 `save_to_layout_dir()`、`load_from_layout_dir()` 与 `KnowledgeRepositoryLayoutManifest`，把 repository 的最小标准布局固定为 `bundle.json + repository.manifest.json`。
- 在 `src/ops/foundation/knowledge_repository.rs` 中把 `save_to_path()` 切到“同目录 staging 写入 -> 再替换正式文件”的路径，用最小替换式写入降低半成品文件风险。
- 同步更新 `README.md`、`docs/AI_HANDOFF.md`、`docs/DOCUMENTATION_INDEX.md`、`docs/plans/2026-04-08-foundation-repository-layout-phase2-stage4.md`、`task_plan.md`、`progress.md`、`findings.md`，把 foundation 当前状态更新为“标准包 + 标准仓储 + 导入 + 过滤扩展 + 标准布局目录”。

### 修改原因
- 用户已批准方案 B，当前阶段需要把 repository 从“单文件可存”推进到“有正式布局标准可存可读”。
- 只有标准包、导入和过滤还不够，foundation 还需要一套最小文件布局契约，否则后续持久化继续扩展时会反复分叉。

### 方案还差什么?
- [ ] 还没有支持目录拆分仓储，例如 `nodes.jsonl / edges.jsonl / concepts.json`。
- [ ] 还没有支持索引文件与版本迁移。
- [ ] 还没有对 manifest 版本不匹配、布局缺文件等情况补更细校验。

### 潜在问题
- [ ] 当前 staging 写入虽然已经比直接覆盖更稳，但在 Windows 上仍不是更完整语义下的强原子替换。
- [ ] 当前 manifest 只记录最小计数与入口文件名，如果后续布局继续变复杂，现有元数据可能不够。
- [ ] 当前 repository 仍然没有 edge 指向不存在节点这类更细一致性校验，布局标准化并不会自动解决这类数据问题。

### 关闭项
- 已通过 `cargo test --test knowledge_repository_unit -- --nocapture`
- 已通过 `cargo test --test knowledge_repository_unit --test knowledge_ingestion_unit --test knowledge_bundle_unit -- --nocapture`

### 记忆点（可选）
- 用户已明确批准方案 B，所以这轮只做标准布局目录、manifest 和替换式写入，不要顺手扩成目录拆分仓储或更大持久化系统。

## 2026-04-08
### 修改内容
- 在 `tests/knowledge_repository_unit.rs` 中新增 `knowledge_repository_requires_all_exact_matches` 与 `knowledge_repository_limits_matches_to_concept_scope` 两条红测，分别锁定多字段 AND 与 concept scope 两个最小过滤扩展行为。
- 在 `src/ops/foundation/knowledge_repository.rs` 中扩展 `MetadataFilter`，新增 `with_concept_id()`，并把 `filtered_node_ids()` 从“只看 metadata exact-match”扩展为“metadata 多字段 AND + concept scope”的组合过滤。
- 同步更新 `README.md`、`docs/AI_HANDOFF.md`、`docs/DOCUMENTATION_INDEX.md`、`docs/plans/2026-04-08-foundation-metadata-filter-phase2-stage3.md`、`task_plan.md`、`progress.md`、`findings.md`，把 foundation 当前过滤能力更新为“多字段 AND + concept scope”。

### 修改原因
- 用户已批准方案 B，要求继续扩展 foundation 的通用过滤能力，但仍然不做 DSL、不做业务化规则。
- 单字段 exact-match 已经不够支撑当前标准能力阶段，至少需要支持“多个条件同时满足”和“按概念域先收窄”这两个最小能力。

### 方案还差什么?
- [ ] 还没有支持 OR / NOT。
- [ ] 还没有支持范围过滤、模糊匹配或正则匹配。
- [ ] 还没有对 concept scope 提供批量接口或更细布尔语义定义。

### 潜在问题
- [ ] 当前 concept scope 采用“任一 concept_id 命中即可”的语义；如果后续节点挂多个 concept，且调用方预期是更严格的 AND 语义，可能会出现预期偏差。
- [ ] 当前 `filtered_node_ids()` 内部仍使用线性 `contains()` 判断 concept scope；节点量和 scope 同时变大时，过滤成本会继续上升。
- [ ] 当前过滤器能力已经扩展，但 repository 写入仍不是原子写，后续若继续推进持久化标准，这个风险还需要单独收口。

### 关闭项
- 已通过 `cargo test --test knowledge_repository_unit -- --nocapture`

### 记忆点（可选）
- 用户已明确批准方案 B，所以这轮只做多字段 AND 与可选 concept scope，不要顺手扩成 OR/NOT、范围过滤或完整 DSL。

## 2026-04-08
### 修改内容
- 在 `tests/knowledge_ingestion_unit.rs` 中新增 3 条红测，分别锁定 `KnowledgeBundle` JSON 导入、tagged-record JSONL 导入和 JSONL 行号错误边界，保证 `knowledge_ingestion` 先由失败测试定义契约。
- 在 `src/ops/foundation/knowledge_ingestion.rs` 中新增 `KnowledgeIngestionError`、`load_bundle_from_json_path()`、`load_repository_from_json_path()`、`load_bundle_from_jsonl_path()`、`load_repository_from_jsonl_path()`，并采用 `bundle_header + concept/relation/node/edge` 的单文件 tagged-record JSONL 格式实现最小标准导入能力。
- 在 `src/ops/foundation.rs` 中导出 `knowledge_ingestion` 模块，使 foundation 标准导入能力成为正式模块边界的一部分。
- 同步更新 `README.md`、`docs/AI_HANDOFF.md`、`docs/DOCUMENTATION_INDEX.md`、`docs/plans/2026-04-08-foundation-knowledge-ingestion-phase2-stage2.md`、`task_plan.md`、`progress.md`、`findings.md`，把 foundation 当前状态更新为“标准包 + 标准仓储 + metadata 过滤 + 标准 JSON/JSONL 导入”。

### 修改原因
- 用户已明确批准方案 B，当前阶段需要在不做业务化映射的前提下，先补齐 foundation 的标准导入能力。
- 只有标准包和标准仓储还不够，foundation 还需要一条“外部文件如何进入标准包/仓储”的最小链路，否则这些标准结构仍然主要依赖手工构造样本。

### 方案还差什么?
- [ ] 还没有支持目录型导入或多文件布局导入，当前 JSONL 仍是单文件 tagged-record 格式。
- [ ] 还没有支持业务原始文件（CSV/Excel/证券数据）直接转标准包。
- [ ] 还没有扩展 `MetadataFilter` 到多字段 AND、concept scope 或更复杂过滤语义。

### 潜在问题
- [ ] 当前 JSONL 只要求存在一个 `bundle_header`，但还没有对“重复 concept id / relation 重复 / edge 指向不存在节点”等边界做更细导入级校验，主要仍依赖后续 schema/repository 构建阶段兜底。
- [ ] `KnowledgeRepository::save_to_path()` 依然不是原子写；即使导入能力已经补齐，落盘阶段仍有半成品文件风险。
- [ ] tagged-record JSONL 现在适合手工维护和小规模样本，但如果后续文件变大，单文件顺序维护成本会升高。

### 关闭项
- 已通过 `cargo test --test knowledge_ingestion_unit -- --nocapture`
- 已通过 `cargo test --test knowledge_bundle_unit --test knowledge_repository_unit --test knowledge_ingestion_unit -- --nocapture`

### 记忆点（可选）
- 用户已批准方案 B，所以当前阶段只做标准 bundle JSON 与 tagged-record JSONL 两类通用导入，不要顺手扩成目录扫描、业务 CSV 映射或证券分析适配。

## 2026-04-08
### 修改内容
- 在 `src/ops/foundation/knowledge_bundle.rs` 中新增 `KnowledgeBundle` 标准知识包结构，并补 `to_ontology_store() / to_graph_store()` 重建入口，用来把 Phase 1 的纯内存导航内核收口为可持久化、可导入导出的通用标准载体。
- 在 `src/ops/foundation/knowledge_repository.rs` 中新增 `KnowledgeRepository` 与 `MetadataFilter`，补最小仓储校验、JSON 落盘/读回和 `node.metadata` exact-match 过滤，用来形成 foundation 的标准仓储边界。
- 在 `src/ops/foundation/knowledge_record.rs`、`src/ops/foundation/ontology_schema.rs`、`src/ops/foundation.rs` 中同步补 `metadata`、`serde` 派生与模块导出，使标准包与仓储闭环可以直接复用现有 ontology/graph 查询层。
- 在 `tests/knowledge_bundle_unit.rs` 与 `tests/knowledge_repository_unit.rs` 中先写失败测试，再补最小实现转绿，锁定标准包、标准仓储和 metadata 精确过滤契约。
- 同步更新 `README.md`、`docs/AI_HANDOFF.md`、`docs/DOCUMENTATION_INDEX.md`、`docs/plans/2026-04-08-foundation-standard-capabilities-phase2.md`、`task_plan.md`、`progress.md`、`findings.md`，把 foundation 当前口径收口为“通用标准能力”，不再误写成业务化完整知识库。

### 修改原因
- 用户已明确纠正：foundation 线当前不做业务化能力，只做通用化标准能力；如果继续沿着证券分析主链写交接文档和计划，会持续误导后续实现方向。
- Phase 1 已经形成最小导航闭环，但此前还缺少标准知识包、标准仓储和 metadata 过滤契约，导致 foundation 仍停留在“能跑”而不是“可标准化复用”的状态。

### 方案还差什么?
- [ ] 还没有补 `knowledge_ingestion`，当前仍缺标准 JSON/JSONL -> `KnowledgeBundle / KnowledgeRepository` 的导入流水线。
- [ ] 还没有补持久化目录布局、版本迁移与索引规范，当前只具备单文件 JSON 仓储能力。
- [ ] 还没有扩展 `MetadataFilter` 到多字段 AND、concept scope 或更复杂过滤语义。

### 潜在问题
- [ ] `KnowledgeRepository::save_to_path()` 当前是直接全量写文件，不是原子写；若写入中断，目标文件可能留下半成品内容。
- [ ] `MetadataFilter` 目前只支持 exact-match，若上层误以为它支持模糊查询、范围过滤或业务 DSL，会出现检索预期落差。
- [ ] 当前 foundation 虽已具备标准包和标准仓储，但仍没有知识内容导入流水线；如果外部继续把它描述成“完整知识库”，会误导后续协作边界。

### 关闭项
- 已通过 `cargo test --test knowledge_bundle_unit --test knowledge_repository_unit -- --nocapture`
- 已通过 `cargo test --test ontology_schema_unit --test ontology_store_unit --test knowledge_record_unit --test knowledge_graph_store_unit --test capability_router_unit --test roaming_engine_unit --test retrieval_engine_unit --test evidence_assembler_unit --test navigation_pipeline_integration --test knowledge_bundle_unit --test knowledge_repository_unit -- --nocapture`

### 记忆点（可选）
- 用户已明确纠正：foundation 线当前只做通用化标准能力，不做业务化能力；后续若未再次获批，不要把它主动接到证券分析主链，也不要把“标准能力建设”误写成“完整知识库产品化”。

## 2026-04-01
### 修改内容
- 在 `tests/integration_tool_contract.rs` 中新增 `technical_consultation_basic_contract_exposes_bullish_continuation_conclusion / technical_consultation_basic_contract_exposes_range_wait_conclusion` 两条外层合同回归，并补充同文件内的最小 CSV 夹具、导入助手与样本生成函数，所有新增代码旁都补了中文时间注释，说明修改原因与目的。
- 同步更新 `task_plan.md`、`progress.md`、`findings.md`，把这轮“方案 A-5 外层合同收口”的测试范围、结论和后续待办写入续接记录。

### 修改原因
- `consultation_conclusion` 虽然已经在 CLI 专项回归中补齐，但 `integration_tool_contract.rs` 之前还没有任何一条测试证明工具层对外确实会返回完整字段。
- 这意味着后续如果发生工具层字段裁剪或结构漂移，现有回归无法第一时间发现，因此需要把外层合同也正式锁住。

### 方案还差什么?
- [ ] 还没有把 `consultation_conclusion` 的字段语义补进 README 或对外 JSON 示例。
- [ ] 还没有补 `confirmed_resistance_retest_hold / confirmed_support_retest_reject` 的外层 continuation 子分支合同。

### 潜在问题
- [ ] 当前外层合同只锁了 `bullish_continuation` 和 `range_wait` 两端样本，continuation 的细分回踩/反抽分支仍可能在未来文案调整时漂移。
- [ ] 本轮没有执行整仓级 `cargo test -- --nocapture --test-threads=1`，当前验证仍是主线定向回归。

### 关闭项
- 已通过 `cargo test --test integration_tool_contract -- --nocapture --test-threads=1`
- 已通过 `cargo fmt --all`

### 记忆点（可选）
- 当内部 CLI 切片已经锁住字段后，仍要在外层合同文件再锁一层真实工具返回；否则“内部正确、外层丢字段”这类问题没有防线。
## 2026-04-01
### 修改内容
- 在 `tests/technical_consultation_basic_cli.rs` 中补 `technical_consultation_basic_marks_choppy_history_as_weak_trend / technical_consultation_basic_marks_confirmed_resistance_breakout_signal / technical_consultation_basic_marks_confirmed_support_breakdown_signal` 的 `consultation_conclusion.rationale / risk_flags` 红测断言，并在断言旁补充中文时间注释，说明修改原因与目的。
- 在 `src/ops/technical_consultation_basic.rs` 中补 `build_consultation_rationale()` 对 `bullish_continuation / bearish_continuation / range_wait` 的专属解释，以及 `build_consultation_risk_flags()` 对这 3 类场景的专属风险提示，并在代码旁补充中文时间注释，说明修改原因与目的。
- 同步更新 `task_plan.md`、`progress.md`、`findings.md`，把本轮“方案 A-4 延续态与等待态证据矩阵”的失败来源、修正方式、验证结果与后续待办写入续接记录。

### 修改原因
- 新补的红测真实复现出：`bullish_continuation / bearish_continuation / range_wait` 虽然已有 `bias / headline`，但 `rationale` 缺少“延续为何成立 / 当前为何该等”的专属解释，`risk_flags` 也缺少对应的结构化风险提示。
- 这说明缺口在 `consultation_conclusion` 的证据层，而不是在 breakout 主分类或置信度主逻辑；因此应采用最小实现补齐解释层与风控层。

### 方案还差什么?
- [ ] 还没有补跑 `cargo test --test integration_tool_contract -- --nocapture --test-threads=1`，外层工具合同仍值得再锁一层。
- [ ] 还没有把 `consultation_conclusion` 的字段语义补进文档或示例 JSON，后续调用方仍主要依赖测试样例理解合同。

### 潜在问题
- [ ] Windows 环境下 `target\\debug\\deps` 的 `rlib` 较大时，仍可能再次触发页面文件不足导致的 `mmap` 失败；本轮是通过清理单个 `libexcel_skill-*.rlib/.rmeta` 后重编绕开。
- [ ] 当前风险提示使用固定中文短句，后续如果要接更细粒度的策略分流，可能还需要继续细分成 breakout 后回踩、跌破后反抽、区间等待失效等更细语义。

### 关闭项
- 已通过 `cargo test --test technical_consultation_basic_cli -- --nocapture --test-threads=1`
- 已通过 `cargo fmt --all`

### 记忆点（可选）
- 当 `consultation_conclusion` 已经上升为正式证券分析合同后，`headline` 不能是唯一解释层；`rationale / risk_flags` 也必须显式写出“当前延续剧本的确认点”或“当前为什么只能等待”。
## 2026-04-01
### 修改内容
- 在 `src/ops/technical_consultation_basic.rs` 中修正 `classify_consultation_confidence()`，为 `confirmed_* / failed_*` 这类已完成关键位结构补上最低 `medium` 置信度地板，并在代码旁补充中文时间注释，说明修改原因与目的。
- 同步更新 `task_plan.md`、`progress.md`、`findings.md`，把本轮“方案 A 组合结论收口”的问题来源、决策方式、验证结果与后续待办写入续接记录。
### 修改原因
- 既有失败回归 `technical_consultation_basic_marks_confirmed_resistance_breakout_signal` 已经真实复现：`confirmed_resistance_breakout` 场景下，新加的 `consultation_conclusion.confidence` 被错误压成 `low`。
- 这说明 `breakout_signal` 的价格结构确认语义不能被 `volume_confirmation = weakening` 直接覆盖，否则上层合同会出现“结构已确认，但组合结论像未确认”的冲突。
### 方案还差什么
- [ ] 还没有补齐空头 confirmed、trap risk、range/watch 各类 `consultation_conclusion.confidence` 的专项矩阵测试。
- [ ] 还没有把 `consultation_conclusion` 加进 README 或对外 JSON 示例，后续其他调用方仍需要从测试里找合同样例。
### 潜在问题
- [ ] 当前采用的是“completed structure 最低 `medium`”收口，如果后续再加入更强的负向辅助指标，可能需要重新评估是否要细分出更低级别或风险覆盖规则。
- [ ] 本轮没有执行整仓级 `cargo test -- --nocapture --test-threads=1`，若后续合并更多并行改动，仍建议补一次全量回归。
### 关闭项
- 已通过：`cargo test --test technical_consultation_basic_cli technical_consultation_basic_marks_confirmed_resistance_breakout_signal -- --exact --nocapture --test-threads=1`
- 已通过：`cargo test --test technical_consultation_basic_cli -- --nocapture --test-threads=1`
- 已通过：`cargo fmt --all`
### 记忆点（可选）
- 已确认结构完成的证券分析样本，允许因量能偏弱从 `high` 降到 `medium`，但不应直接降成 `low`；否则会破坏 `breakout_signal` 与 `consultation_conclusion` 的合同一致性。

## 2026-04-01
### 修改内容
- 在 `tests/technical_consultation_basic_cli.rs` 中调整 `build_bullish_range_watch_rows()` 的最近 20 根尾部夹具，把“缓慢抬升”改成“先冲高、后高位箱体”，并在代码旁补充中文时间注释，说明修改原因与目的。
- 同步更新 `task_plan.md`、`progress.md`、`findings.md`，把本轮“偏多区间观察态闭环”的失败原因、样本修正方式、验证结果与后续待办写入续接记录。
### 修改原因
- 新增回归 `technical_consultation_basic_marks_bullish_range_watch_conclusion` 先真实复现：原样本被误判为 `resistance_retest_watch`，没有落到目标的 `range_bound -> bullish_range_watch` 路径。
- 这说明问题出在夹具尾部形态与“多根回踩观察态”规则重叠，而不是 `consultation_conclusion` 的主分类逻辑回退。
### 方案还差什么
- [ ] 还没有补齐 `bullish_range_watch / bearish_range_watch` 的 `rationale / risk_flags` 专项断言矩阵。
- [ ] 还没有补跑 `integration_tool_contract`，外层合同链路的回归还可以再加一层保险。
### 潜在问题
- [ ] 其他区间观察态夹具如果继续采用单调抬升或单调下压的尾部写法，后续仍可能误触发 `retest_watch` 多根锚点。
- [ ] 当前验证集中在证券分析切片，没有覆盖整仓级 `cargo test -- --nocapture --test-threads=1`。
### 关闭项
- 已通过：`cargo test --test technical_consultation_basic_cli technical_consultation_basic_marks_bullish_range_watch_conclusion -- --exact --nocapture --test-threads=1`
- 已通过：`cargo test --test technical_consultation_basic_cli -- --nocapture --test-threads=1`
- 已通过：`cargo fmt --all`
### 记忆点（可选）
- 区间观察态样本的尾部如果仍保持单边推进，容易与多根 `retest_watch` 锚点扫描重叠；要优先用“先冲高/探低，再箱体整理”的夹具形态表达等待态。

## 2026-04-01
### 修改内容
- 在 `tests/technical_consultation_basic_cli.rs` 中继续补 4 组 `consultation_conclusion.rationale / risk_flags` 断言，覆盖 `bullish_range_watch`、`bearish_range_watch`、`bullish_confirmation_watch`、`bearish_confirmation_watch`。
- 在 `src/ops/technical_consultation_basic.rs` 中最小增量补 `build_consultation_rationale()` 与 `build_consultation_risk_flags()`：为双边区间等待态追加方向化理由，为双边观察态追加“尚待确认”风险标记，并在代码旁补充中文时间注释，说明修改原因与目的。
- 同步更新 `task_plan.md`、`progress.md`、`findings.md`，把本轮“组合结论证据矩阵”失败原因、修正方式、验证结果与后续待办写入续接记录。
### 修改原因
- 新增断言先真实复现：`range_watch` 只有通用区间理由，`confirmation_watch` 缺少“尚待确认”类风险标记，说明 `consultation_conclusion` 的证据层仍不够完整。
- 这说明当前缺口不在 bias/confidence 主分类，而在上层证券分析合同对“当前在等什么确认、最该防什么风险”的直接表达不足。
### 方案还差什么
- [ ] 还没有把 `bull_trap_risk / bear_trap_risk / bullish_continuation / bearish_continuation / range_wait` 也补成同等级的 `rationale / risk_flags` 精确断言矩阵。
- [ ] 还没有补跑 `cargo test --test integration_tool_contract -- --nocapture --test-threads=1`。
### 潜在问题
- [ ] 当前新增的“尚待确认”风险标记仍是固定短语，后续如果要对接更细的策略层，可能需要继续细分成 breakout_watch / retest_watch / range_watch 不同粒度。
- [ ] 当前验证仍聚焦在证券分析切片，没有覆盖整仓级 `cargo test -- --nocapture --test-threads=1`。
### 关闭项
- 已通过：`cargo test --test technical_consultation_basic_cli technical_consultation_basic_marks_bearish_range_watch_conclusion -- --exact --nocapture --test-threads=1`
- 已通过：`cargo test --test technical_consultation_basic_cli technical_consultation_basic_marks_bullish_range_watch_conclusion -- --exact --nocapture --test-threads=1`
- 已通过：`cargo test --test technical_consultation_basic_cli technical_consultation_basic_marks_resistance_retest_watch_signal -- --exact --nocapture --test-threads=1`
- 已通过：`cargo test --test technical_consultation_basic_cli technical_consultation_basic_marks_support_retest_watch_signal -- --exact --nocapture --test-threads=1`
- 已通过：`cargo test --test technical_consultation_basic_cli -- --nocapture --test-threads=1`
- 已通过：`cargo fmt --all`
### 记忆点（可选）
- 当 `consultation_conclusion` 已经上提成正式证券分析合同后，`headline` 不能是唯一的“解释层”；`rationale / risk_flags` 也必须明确写出“当前在等什么确认”。

## 2026-04-01
### 修改内容
- 在 `tests/technical_consultation_basic_cli.rs` 中继续补 2 组陷阱态 `consultation_conclusion.rationale / risk_flags` 断言，覆盖 `bull_trap_risk` 与 `bear_trap_risk`。
- 在 `src/ops/technical_consultation_basic.rs` 中最小增量补 `build_consultation_rationale()` 与 `build_consultation_risk_flags()`：为双边 trap risk 追加“原延续判断失效”的专属解释与风险标记，并在代码旁补充中文时间注释，说明修改原因与目的。
- 同步更新 `task_plan.md`、`progress.md`、`findings.md`，把本轮“陷阱态证据矩阵”失败原因、修正方式、验证结果与后续待办写入续接记录。
### 修改原因
- 新增断言先真实复现：trap risk 当前只会说明“假突破/假跌破已经形成”，不会明确告诉上层“原有突破/跌破延续判断已经失效”。
- 这说明当前缺口不在 trap 分类本身，而在证券分析合同对“原剧本已经失效”这层可执行语义的表达不足。
### 方案还差什么
- [ ] 还没有把 `bullish_continuation / bearish_continuation / range_wait` 也补成同等级的 `rationale / risk_flags` 精确断言矩阵。
- [ ] 还没有补跑 `cargo test --test integration_tool_contract -- --nocapture --test-threads=1`。
### 潜在问题
- [ ] 当前新增的 trap risk 风险标记仍聚焦“延续判断失效”，后续如果接策略控制层，可能还需要继续细分成修复扩散、反向加速等更细语义。
- [ ] 当前验证仍聚焦在证券分析切片，没有覆盖整仓级 `cargo test -- --nocapture --test-threads=1`。
### 关闭项
- 已通过：`cargo test --test technical_consultation_basic_cli technical_consultation_basic_marks_failed_resistance_breakout_signal -- --exact --nocapture --test-threads=1`
- 已通过：`cargo test --test technical_consultation_basic_cli technical_consultation_basic_marks_failed_support_breakdown_signal -- --exact --nocapture --test-threads=1`
- 已通过：`cargo test --test technical_consultation_basic_cli -- --nocapture --test-threads=1`
- 已通过：`cargo fmt --all`
### 记忆点（可选）
- 陷阱态如果只写“假突破/假跌破已形成”，对上层还不够；还必须显式告诉调用方“原延续剧本已经失效”，这样后续策略层才不需要再自行推断。
## 2026-04-01
### 修改内容
- 在 `tests/integration_tool_contract.rs` 中新增 `build_confirmed_breakdown_rows()` 与 `technical_consultation_basic_contract_exposes_bearish_continuation_conclusion`，把外层工具合同从“多头延续 + 等待态”补成“多头延续 + 空头延续 + 等待态”三类主分支，并在代码旁补中文时间注释说明修改原因与目的。
- 同步更新 `task_plan.md`、`progress.md`、`findings.md`，把“方案 A-6 外层合同空头对称样本”的补齐情况、验证结果和后续待办写入续接记录。

### 修改原因
- 当前生产实现已经支持 `bearish_continuation`，但外层合同文件还缺对应的正式对称样本；这会让空头路径继续主要依赖 CLI 专项回归。
- 这一步的目标是补齐外层合同矩阵，而不是继续改动已稳定的生产逻辑。

### 方案还差什么?
- [ ] 还没有执行整仓级 `cargo test -- --nocapture --test-threads=1`。
- [ ] 还没有补最小使用说明或对外 JSON 示例。

### 潜在问题
- [ ] 当前验证仍以证券分析主线切片为主，无法完全替代整仓级联动回归。
- [ ] 如果后续继续调整 `headline / rationale / risk_flags` 文案，没有公开文档时仍容易让外部调用方跟不上合同语义变化。

### 关闭项
- 已通过 `cargo test --test integration_tool_contract -- --nocapture --test-threads=1`
- 已通过 `cargo fmt --all`

### 记忆点（可选）
- 合同样本矩阵不要只锁“最顺手的一边”；多头和空头都要有外层对称样本，否则后续续接容易误判为空头路径尚未正式可用。
## 2026-04-01
### 修改内容
- `src/ops/security_analysis_contextual.rs`：新增上层综合证券分析 Tool，复用 3 次 `technical_consultation_basic` 输出个股、大盘、板块分析，并聚合 `contextual_conclusion`；代码旁已补中文时间注释，说明修改原因和目的。
- `src/ops/stock.rs`、`src/ops/mod.rs`：暴露 `security_analysis_contextual` 模块，保持 stock 域边界清晰。
- `src/tools/dispatcher/stock_ops.rs`、`src/tools/dispatcher.rs`、`src/tools/catalog.rs`：接入新 Tool 的 dispatcher 与 catalog，使 CLI 可以发现和调用该能力。
- `tests/security_analysis_contextual_cli.rs`：新增 3 条综合 Tool 合同回归，覆盖 tool catalog 可发现、三层同向时 `tailwind`、个股等待态时保持 `mixed`。
- `task_plan.md`、`progress.md`、`findings.md`：补充本轮 A-7 续做记录，沉淀当前完成点、风险与后续计划。

### 修改原因
- 用户已明确要求不要改 `technical_consultation_basic` 边界，而是在上层新增一个 Tool，把个股技术面、大盘环境、板块环境合成统一证券分析入口。
- 当前真实可用能力缺口不在指标计算，而在“环境联动的正式合同”，因此优先补上层聚合而不是继续堆底层信号。

### 方案还差什么？
- [ ] 还没有补 `headwind` 的正式红测样本。
- [ ] 还没有补对外 README / JSON 示例，真实调用仍主要参考测试文件。
- [ ] 还没有接入信息面、资金面、基本面维度。

### 潜在问题
- [ ] 如果 `market_symbol / sector_symbol` 选取不合理，上层综合结论会被代理符号误导。
- [ ] 当前聚合规则仍是 MVP，复杂分歧场景先归并成 `mixed/headwind`，后续可能还要细分。

### 关闭项
- 已通过 `cargo test --test security_analysis_contextual_cli -- --nocapture --test-threads=1`
- 已通过 `cargo test --test integration_tool_contract -- --nocapture --test-threads=1`
- 已通过 `cargo fmt --all`

### 记忆点（可选）
- 用户明确纠正过：综合证券分析要在 `technical_consultation_basic` 上层新增 Tool，不要把大盘、板块语义继续塞回底层技术面模块。
## 2026-04-01
### 修改内容
- `tests/security_analysis_contextual_cli.rs`：新增 `headwind`、profile 入口、缺代理配置错误路径 3 组综合 Tool 回归测试，并在代码旁补中文时间注释说明修改原因与目的。
- `src/ops/security_analysis_contextual.rs`：重写综合证券分析模块，请求支持 `market_symbol / sector_symbol` 与 `market_profile / sector_profile` 二选一；新增缺参与不支持 profile 的业务层错误；细化 `headwind` 结论。
- `docs/acceptance/2026-04-01-security-analysis-contextual-v1.md`：新增综合证券分析 V1 文档，补最小调用示例、profile 清单、alignment 语义与当前边界。
- `task_plan.md`、`progress.md`、`findings.md`：补齐本轮方案 B 的续做记录，沉淀完成点、限制与后续计划。

### 修改原因
- 用户已要求“先全部做完再测”，因此需要把综合 Tool 从 MVP 补到可交付 V1，而不是停在顺风/等待态的最小实现。
- 当前真实使用门槛主要卡在代理 symbol 必须手填、逆风路径缺合同、调用文档不足，因此优先收这三块。

### 方案还差什么？
- [ ] 还没有补更多行业 profile，也没有覆盖港股代理 profile。
- [ ] 还没有把综合证券分析说明同步到 README 首页。
- [ ] 还没有接入信息面、资金面、基本面维度。

### 潜在问题
- [ ] 当前 profile 只有 `a_share_core / a_share_bank`，真实使用时如果行业不匹配，仍需手工传代理 symbol。
- [ ] 如果后续继续扩综合结论，需继续补正式红测，避免 `headwind / mixed` 规则漂移。

### 关闭项
- 已通过 `cargo test --test security_analysis_contextual_cli -- --nocapture --test-threads=1`
- 已通过 `cargo test --test integration_tool_contract -- --nocapture --test-threads=1`
- 已通过 `cargo fmt --all`

### 记忆点（可选）
- 用户已经多次明确：综合证券分析要在上层聚合推进，不要为了“更全面”就回头污染 `technical_consultation_basic` 的职责边界。
## 2026-04-01
### 修改内容
- `tests/security_analysis_fullstack_cli.rs`：新增 `tool_catalog` 可发现性、信息面成功聚合、信息源失败降级 3 条 fullstack 总 Tool 回归测试；测试继续沿用 `CSV -> import_stock_price_history -> SQLite -> Tool` 主链，并新增按路径返回不同响应的本地 HTTP 假服务。代码旁已补中文时间注释，说明修改原因与目的。
- `src/ops/security_analysis_fullstack.rs`：新增 `security_analysis_fullstack` 上层总 Tool，聚合 `security_analysis_contextual`、财报快照、公告摘要、行业上下文与综合 `integrated_conclusion`；新增财报/公告 unavailable 降级对象；代码旁已补中文时间注释，说明修改原因与目的。
- `src/ops/stock.rs`、`src/ops/mod.rs`：暴露 `security_analysis_fullstack` 模块，保持 stock 域边界清晰。
- `src/tools/dispatcher/stock_ops.rs`、`src/tools/dispatcher.rs`、`src/tools/catalog.rs`：接入 `security_analysis_fullstack` 的 dispatcher 与 catalog，确保 CLI / Skill / tool_catalog 可发现并可调用。
- `docs/acceptance/2026-04-01-security-analysis-fullstack-v1.md`：新增 fullstack V1 验收文档，补调用样例、信息源边界、降级规则与验收命令。
- `task_plan.md`、`progress.md`、`findings.md`：同步补本轮 fullstack 信息面总入口的续做记录，沉淀完成点、限制与后续计划。

### 修改原因
- 用户已明确要求把大盘、行业、财报、公告并进产品主链，而不是继续依赖手工外查或把信息面语义塞回底层技术面 Tool。
- 当前最稳的演进方式是新增独立上层总 Tool，把技术主链与信息面聚合在一起，同时允许第三方免费源失败时优雅降级，避免产品整体不可用。

### 方案还差什么?
- [ ] 还没有把财报与公告抓取沉淀到本地 SQLite，当前仍是在线聚合。
- [ ] 还没有接入新闻面、资金面与一致预期。
- [ ] 还没有扩更多行业 profile 与自动行业映射。

### 潜在问题
- [ ] 免费信息源可能发生限流、结构漂移或短时不可达；当前虽然已做 unavailable 降级，但重复查询稳定性仍受第三方源影响。
- [ ] 财报字段当前依赖东财返回键名兼容映射，若线上字段体系变动，需要补充别名或改解析规则。
- [ ] 行业层当前主要还是 sector proxy 技术环境，不应被误解成已经具备完整行业景气数据库。

### 关闭项
- 已通过 `cargo test --test security_analysis_fullstack_cli -- --nocapture --test-threads=1`
- 已通过 `cargo test --test integration_tool_contract -- --nocapture --test-threads=1`
- 已通过 `cargo fmt --all`

### 记忆点（可选）
- 用户再次纠正后需要记住：证券分析产品的目标是“优先走我们自己的 Tool 主链给全面分析”，不是临时给一段泛化股评；信息面要继续在上层新 Tool 聚合，不要回灌到底层技术面 Tool。
## 2026-04-01
### 修改内容
- 在 `docs/交接摘要_给后续AI.md` 文件头追加 2026-04-01 证券分析主线补充，集中说明 `technical_consultation_basic / security_analysis_contextual / security_analysis_fullstack` 的当前职责、日期锚定规则和接手入口。
- 新增 `docs/交接摘要_证券分析_给后续AI.md`，把证券分析产品主链、数据源约束、降级语义、验证口径、常见误区和续做建议单独沉淀成专用 AI 交接文档。
- 在 `docs_tradingagents_agent_prompt_data_notes.md` 文件头补仓库边界提醒，明确 Python `tradingagents/` 与本地 SheetMind 证券分析主链不是同一条产品链路。
- 在 `README.md` 文件头补证券分析入口索引，方便 GitHub 访客和后续 AI 快速定位证券分析 Skill、验收文档与交接文档。

### 修改原因
- 用户这轮要求不仅要推 GitHub，还要“完善 AI 交接手册”，而现有总交接文档仍以 2026-03-31 的 Excel / GUI 主线为主，缺少 2026-04-01 之后证券分析主线的集中说明。
- 仓库同时存在 Python TradingAgents 和本地 SheetMind 证券分析链，若不显式写边界，后续 AI 很容易串台。

### 方案还差什么?
- [ ] 还没有把证券分析 `analysis_date` 规则下沉到 Rust Tool 返回合同里，当前仍主要依赖 Skill 和文档约束。
- [ ] 还没有补 README 内部更深入的证券分析调用示例，目前先提供的是入口索引级说明。

### 潜在问题
- [ ] `docs/交接摘要_给后续AI.md` 和 `README.md` 存在历史编码噪声，当前采用文件头增量追加的方式规避风险；后续如要全面整理编码，建议单独开任务，不要顺手清洗。
- [ ] 当前文档已能指导接手，但如果后续证券分析主链继续快速扩展，仍需要定期同步手册，避免再次滞后。

### 关闭项
- 已新增 `docs/交接摘要_证券分析_给后续AI.md`
- 已更新 `docs/交接摘要_给后续AI.md`
- 已更新 `docs_tradingagents_agent_prompt_data_notes.md`
- 已更新 `README.md`
- [ ] 未执行自动化测试：本轮新增为文档与 Skill 交接增强，没有改动 Rust 运行逻辑

### 记忆点（可选）
- 当仓库里同时存在多条产品线时，AI 交接手册必须先写清楚“哪条是当前用户要继续推进的主链”，否则后续很容易把无关架构拉进来。

## 2026-04-01
### 修改内容
- 在 `skills/security-analysis-v1/SKILL.md` 中新增证券分析专用 Skill，固化“证券分析默认锚定当前日期、若当前日期无有效收盘则退前一个交易日、输出必须显式写明实际分析日期”的硬规则，并同步写入“优先走 `technical_consultation_basic / security_analysis_contextual / security_analysis_fullstack` 主链”的约束。

### 修改原因
- 用户明确纠正：这条规则不能只靠口头记忆，必须写进项目 Skill，供后续 AI 和协作者继续沿同一证券分析口径工作。
- 之前证券分析回答存在混入非锚定日期数据的风险，需要把“先锁日期、再下结论”升级为项目级规范。

### 方案还差什么?
- [ ] 还没有把 `security-analysis-v1` 接入仓库内更显眼的导航位置，例如 README 或证券分析使用文档。
- [ ] 还没有给证券分析主链补自动化测试，去验证输出层确实会显式带出“实际分析日期”。

### 潜在问题
- [ ] 当前只是新增 Skill 规范文件，还没有在 Rust Tool 返回合同里强制产出 `analysis_date` 字段；如果后续调用方忽略 Skill 约束，仍可能在应用层漏写日期。
- [ ] 证券分析 Skill 是新目录，若仓库中存在额外的 Skill 索引清单而未同步，后续协作者可能不容易第一时间发现它。

### 关闭项
- 已新增 `skills/security-analysis-v1/SKILL.md`
- 已用公开免费行情接口核验 `159866` 在 `2026-04-01` 的收盘数据，可据此按新规则重算用户问题
- [ ] 未执行自动化测试：本次改动为 Skill 文档新增，尚未补程序侧自动化校验

### 记忆点（可选）
- 证券分析不能混日期；默认只看当前日期，拿不到当前日期有效收盘时才退前一个交易日，而且输出必须明确写出实际分析日期。

## 2026-04-02
### 修改内容
- `E:\TradingAgents\TradingAgents\src\runtime\signal_outcome_store.rs`：补充 `security_signal_tags` 与 `security_signal_analog_studies` 的正式读写能力，新增 `summary_payload` 持久化字段与轻量 schema 迁移助手。修改原因：要把“银行板块共振 + 核心技术相似”的历史研究沉成正式资产，而不是临时现算。目的：让后续 briefing、committee 与外层 Agent 读取同一份历史摘要。
- `E:\TradingAgents\TradingAgents\src\ops\signal_outcome_research.rs`：新增 `study_security_signal_analogs`、`signal_outcome_research_summary`、快照标签生成、相似度计算与摘要格式化逻辑，并在 `record_security_signal_snapshot` 中把标签一并落库。修改原因：用户明确要求银行体系历史研究要把 MACD、RSRS 与共振一起做。目的：形成可解释、可复用、可落库的历史相似研究主链。
- `E:\TradingAgents\TradingAgents\src\ops\security_decision_briefing.rs`：把历史研究摘要正式接回 `committee_payload.historical_digest`，不再固定输出 `unavailable`。修改原因：用户要求咨询口径与投决口径看到的信息必须一致。目的：让 briefing 直接携带历史相似样本数、10 日胜率、预期收益区间与回撤区间。
- `E:\TradingAgents\TradingAgents\src\tools\dispatcher\stock_ops.rs`、`E:\TradingAgents\TradingAgents\src\tools\dispatcher.rs`：补齐 `study_security_signal_analogs` 与 `signal_outcome_research_summary` 的 dispatcher 分发入口。修改原因：用户强调所有动作都要走 Tool/Skill 主链。目的：让历史研究能力成为正式可发现 Tool。
- `E:\TradingAgents\TradingAgents\tests\security_analysis_resonance_cli.rs`：先补两条红测，再落实现。新增“银行体系历史相似研究产出摘要”与“briefing 接入 historical_digest”两条回归测试，并构建银行板块/宏观代理/多股票快照夹具。修改原因：发现 BUG 或新增能力时必须先用测试锁边界。目的：确保研究层与 briefing 层的行为稳定可回归。

### 修改原因
- 用户明确纠正过：这类历史研究不该只看中信银行单只股票，而应覆盖银行体系，并把银行板块共振与 MACD、RSRS 等核心技术状态一起纳入。
- 用户明确纠正过：结论可信度说明必须用中文解释业务含义，不能只抛英文或内部指标名，因此本次采用“标签重合 + 核心数值接近”的可解释相似度路径。

### 方案还差什么?
- [ ] 目前 `study_key` 默认先固定为 `bank_resonance_core_technical_v1`，后续还需要扩展成可按行业模板自动切换的通用方案。
- [ ] 当前历史摘要主要输出 10 日收益/回撤区间，后续如要服务更完整投决，需要继续补 20 日或更长窗口的正式摘要展示。
- [ ] 现在银行体系历史研究夹具已打通，但本地真实 runtime 里还没有完整银行股池历史，需要后续补真实样本库再跑全量研究。

### 潜在问题
- [ ] 当前相似度算法是第一版可解释方案，核心依赖标签重合与少量指标尺度归一；如果后续字段分布发生明显漂移，可能需要重新校准各指标 scale。
- [ ] `summary_payload` 通过 JSON 持久化，若后续摘要结构扩展较大，需要注意版本兼容与旧库迁移。
- [ ] 这两条新测试是真实 Tool 链路夹具，执行时间较长；后续若继续扩展，建议拆出更轻量的 store/ops 单测，避免整体回归越来越慢。

### 关闭项
- 已通过 `cargo fmt`
- 已通过 `cargo test --test security_analysis_resonance_cli study_security_signal_analogs_builds_bank_core_technical_summary`
- 已通过 `cargo test --test security_analysis_resonance_cli security_decision_briefing_exposes_available_historical_digest_after_analog_study`

### 记忆点（可选）
- 用户明确要求：历史相似研究不是“中信银行个股复盘工具”，而是“银行板块状态库 + 银行个股响应库”的平台方向。
- 用户明确要求：MACD、RSRS 这些技术状态必须和共振一起做，不接受只做板块共振、不做技术形态分层的粗粒度研究。
### 修改内容
- `Cargo.toml`、`Cargo.lock`：新增 `regex` 依赖。原因：A3 方案首版需要解析新浪公开 HTML 页面中的财报指标表和公告 datelist。目的：在不引入更重 HTML 解析栈的前提下，先把多源降级链闭环落地。
- `src/ops/security_analysis_fullstack.rs`：将原先只依赖东财的财报/公告抓取重写为“三层 provider 链”。原因：用户现场已确认东财在本机网络下存在稳定 TLS 失败，单源实现会把 `fullstack` 频繁打回 `technical_only`。目的：按 `东财 -> 可插拔官方备源 -> 新浪公开页` 的顺序自动降级，并保留原有请求/响应合同。
- `src/ops/security_analysis_fullstack.rs`：新增新浪财报页和新浪公告页的 HTML 解析、官方 JSON 备源入口、统一 URL 构造与 provider 失败聚合。原因：需要让 fullstack 在主源故障时仍返回可消费的信息面。目的：降低对单一外部源的依赖，并给后续接入真正官方站点保留插口。
- `src/ops/security_analysis_fullstack.rs`：调整 `classify_fundamental_signal` 为“允许部分同比指标缺失时仍可给出正负判断”。原因：银行股等页面常只有净利润增速、ROE 等关键项，若坚持双字段齐备会把有效备源误判成 `unknown`。目的：让备源在关键指标足够时真正参与综合结论，而不是被无意义地退化。
- `tests/security_analysis_fullstack_cli.rs`：先补两条红测，再补齐所有 provider 失败的回归场景。原因：本轮属于信息面降级 bugfix/能力升级，必须先用测试锁住“东财失败但新浪接住”和“三层优先级”行为。目的：防止后续 provider 链继续演化时把降级顺序和 `technical_only` 触发条件改坏。

### 修改原因
- 用户批准执行 A3 方案后，核心目标已经从“解释东财为什么失败”转成“即使东财失败也要继续完成 fullstack 信息面分析”，因此需要把单源实现改成真正的多源降级。
- 当前仓库的证券分析必须优先走项目内 Tool/Skill 主链；既然 `security_analysis_fullstack` 是产品主入口，就应该在这里兜住外部源波动，而不是把缺口推给用户手工补资料。

### 方案还差什么？
- [ ] 还没有把“官方备源”接成真正默认可用的交易所/IR 抓取实现；当前首版先保留为可插拔 JSON 入口，方便后续按站点差异继续接。
- [ ] 还没有补更大范围的集成回归，例如 `cargo test -- --nocapture --test-threads=1` 全量仓库测试。
- [ ] 还没有把多源 provider 的来源说明同步到外部文档或 README。

### 潜在问题
- [ ] 新浪页面属于 HTML 抓取，若其表格标签或 datelist 结构改版，解析规则仍可能失效。
- [ ] 官方备源当前是可配置 JSON 入口，若后续直接接交易所/IR 页面，仍需补站点级测试夹具。
- [ ] `regex` 解析目前只覆盖首版所需字段；若后续要扩更多财报字段，可能需要升级为更稳的 DOM 解析方案。

### 关闭项
- 已通过 `cargo test --test security_analysis_fullstack_cli`
- 已通过 `cargo fmt`

### 记忆点（可选）
- 用户再次纠正后需要固定记住：做这个仓库内定义过的证券分析任务时，必须优先走项目里的 Tool/Skill 主链，不能绕过本地工具直接凭外部网页做口头分析。

## 2026-04-02
### 修改内容
- `tests/security_analysis_fullstack_cli.rs`：新增 `security_analysis_fullstack_recognizes_real_chinese_sina_financial_labels` 与 `security_analysis_fullstack_recognizes_mojibake_sina_financial_labels_from_live_page` 两条回归，并补充“真实中文标签夹具 / 线上乱码标签夹具”。原因：用户现场确认 `601998.SH` 的新浪财报页可以访问，但 `fundamental_context` 仍被误判为 `unavailable`。目的：先用红测锁住“页面可达但标签失真时仍要识别财报”的真实 bug，再驱动实现修复。
- `src/ops/security_analysis_fullstack.rs`：保留原新浪财报解析不动，新增 `fetch_fundamental_from_sina_resilient`，并把财报 fallback 调用切到新分支。原因：旧函数周边存在历史乱码注释噪声，直接重写风险更高。目的：用小补丁方式新增一条“日期行识别 + typecode 锚点 + 标签归一化”的稳健解析链，降低对页面文案编码状态的依赖。
- `src/ops/security_analysis_fullstack.rs`：新增 `ParsedHtmlRow / parse_html_rows_with_raw / normalize_sina_financial_label / is_sina_report_period_row / is_sina_financial_metric_row / looks_like_report_date` 等辅助函数。原因：线上新浪财报页在当前链路下会出现乱码标签，但 `financialratios44 / 59 / 65` 等结构锚点仍稳定。目的：让净利润同比、ROE、净利润与报告期即使在乱码页面里也能继续被识别。
### 修改原因
- 这轮不是数据源不可用，而是“新浪财报页能打开，但程序没把真实页面里的关键财报行识别出来”，导致 `security_analysis_fullstack` 对 `601998.SH` 继续退化成 `technical_only`。
- 用户明确追问“这个还拿不到嘛”，说明这已经从实现能力问题变成了现场可用性问题；因此需要优先修正仓库主链，而不是继续让用户手工补财报。
### 方案还差什么？
- [ ] 现在只修了新浪财报 fallback，东财财报主源的 TLS 问题还没有根治；后续如果网络环境恢复，仍建议把主源握手问题单独排查。
- [ ] 现在的稳健识别依赖 `financialratios` typecode，如果新浪未来改掉这一层链接结构，还需要继续补新夹具和解析规则。
- [ ] 公告标题里仍有历史编码噪声，后续若要提升用户可读性，还需要再单独处理新浪公告标题解码。
### 潜在问题
- [ ] 银行股场景下，新浪页常缺“营收同比”这类字段，所以当前财报结论仍可能带 `财报关键同比指标不完整` 风险提示；这属于数据源原始缺口，不是本次解析失败。
- [ ] `fetch_fundamental_from_sina` 旧函数仍保留在文件里作为历史路径，后续如果继续清理 `security_analysis_fullstack.rs`，可以考虑在单独任务里收敛掉旧实现。
### 关闭项
- 已通过：`cargo test --test security_analysis_fullstack_cli security_analysis_fullstack_recognizes_mojibake_sina_financial_labels_from_live_page -- --exact --nocapture`
- 已通过：`cargo test --test security_analysis_fullstack_cli -- --nocapture --test-threads=1`
- 已通过：`cargo fmt`
- 已现场复跑 `601998.SH`：`fundamental_context.status=available`，`source=sina_financial_guideline`，`integrated_conclusion.stance=watchful_positive`
### 记忆点（可选）
- 证券分析主链里遇到“页面能打开但工具仍判空”的问题时，不能只盯中文标签文本；要优先检查真实页面里是否还有更稳定的结构锚点，例如 `typecode`、日期列形状或固定链接模式。

## 2026-04-02
### 修改内容
- `tests/security_analysis_resonance_cli.rs`：新增共振平台首批红测，覆盖 4 个正式入口的可发现性，以及“因子注册 -> 因子序列写库 -> 事件标签写库 -> 共振分析 -> 快照落库”的完整主链。原因：用户批准按方案 3 先做平台底层，而不是继续停留在一次性分析文案。目的：用真实 `CSV -> SQLite -> Tool` 链路把平台底层合同钉住，防止后续实现只停在内部函数级别。
- `src/runtime/security_resonance_store.rs`：新增共振平台 SQLite Store，落地 `resonance_factor_registry / resonance_factor_series / resonance_event_tags / security_resonance_snapshots` 四类表，并提供因子注册、序列写入、事件写入、候选因子读取、快照替换等能力。原因：用户明确要求“算了以后写到数据库里，后边把相关性强的拉出来评估”。目的：先把平台研究底座搭好，让后续扩因子、做评估和加 Skill 都有统一数据基础。
- `src/ops/security_analysis_resonance.rs`：新增 `register_resonance_factor / append_resonance_factor_series / append_resonance_event_tags / security_analysis_resonance` 主模块；分析入口复用 `security_analysis_fullstack` 作为基础判断，再叠加板块代理、市场代理、注册因子与事件标签，输出 `top_positive_resonances / top_negative_resonances / event_overrides / resonance_score / action_bias / driver_summary`。原因：用户明确要求国际与行业证券分析不能只看技术面，要显式把共振带出来。目的：把“基础分析 + 共振研究”连成第一版正式产品链。
- `src/runtime/mod.rs`、`src/ops/stock.rs`、`src/ops/mod.rs`：导出共振平台 runtime/ops 模块。原因：新平台已经进入正式 stock 主线。目的：保持 crate 边界清晰，同时兼容现有 `crate::ops::...` 调用路径。
- `src/tools/catalog.rs`、`src/tools/dispatcher.rs`、`src/tools/dispatcher/stock_ops.rs`：把 `register_resonance_factor / append_resonance_factor_series / append_resonance_event_tags / security_analysis_resonance` 接入 tool catalog 与 stock dispatcher。原因：平台不是内部实验代码，必须成为 CLI / Skill 可发现、可调用的正式入口。目的：让后续研究与分析都能沿现有 Tool 主链推进。
### 修改原因
- 用户这轮明确纠正：共振体系不能先靠“默认因子文案”硬写，而应该优先做成可计算、可落库、可评估的平台；同时事件标签必须进入第一版，而不是后补。
- 用户还明确强调：A 股和国际资产都需要关注共振，但不能机械地把一套静态模板硬套所有市场；更合理的路径是先把传统行业的共振研究底座做好，再在上层分析里显式带出真正起作用的驱动。
### 方案还差什么？
- [ ] 现在第一版已经把平台骨架跑通，但还没有补 `evaluate_security_resonance` 这类更偏研究型的独立评估入口，当前评估与分析聚合仍绑定在 `security_analysis_resonance` 里。
- [ ] 还没有把“行业模板候选因子池”正式沉淀成默认注册脚本或配置，目前平台底层已经支持注册，但实际模板填充仍需后续继续建设。
- [ ] 还没有把事件标签升级成更细的结构化分类体系，例如地缘、政策、运价、制裁等子类维度，目前首版先锁“可写入、可参与评分、可进入结果”。
- [ ] 还没有补全量仓库回归，只跑了本次新平台专项测试和 `integration_tool_contract`。
### 潜在问题
- [ ] 第一版共振评分仍属于“可解释、可扩展的启发式量化”，并不是回测收敛后的最终权重；后续随着因子库增大，评分权重和领先/滞后规则仍需要继续校准。
- [ ] 事件标签当前已入库并参与总分，但尚未形成“统计显著性”长期评估表；若后续要区分“真驱动事件”和“叙事噪声”，还需要继续补事件有效性回溯。
- [ ] 当前板块/市场代理快照会与注册因子一起写入 `security_resonance_snapshots`，后续若因子量快速增大，可能需要进一步补索引或拆查询视图。
- [ ] 现有 `dispatcher.rs` 仍有较多历史 `dead_code` warning，这轮没有顺手清理，避免和平台底层任务混做。
### 关闭项
- 已通过 `cargo test --test security_analysis_resonance_cli -- --nocapture`
- 已通过 `cargo test --test integration_tool_contract -- --nocapture`
- 已通过 `cargo fmt`
- 已把共振平台 4 个正式 Tool 接入 `tool_catalog_modules.stock`
- 已验证共振分析会把因子注册、序列、事件标签和快照真正落到 `security_resonance.db`
### 记忆点（可选）
- 用户再次纠正后需要固定记住：共振体系不能先靠“默认因子文案”拍脑袋给结论，应该优先做成“候选因子可注册、可计算、可落库、可评估”的平台。
- 用户明确要求：事件标签不是装饰信息，必须进入第一版平台，并参与共振评估而不是只留在说明文字里。
- 用户当前策略偏好是“先把底层平台打扎实，再往上搭 Skill 和更丰富的分析层”，所以遇到类似量化/研究型需求时，应优先考虑平台化地基而不是只补一个上层结论 Tool。
## 2026-04-02
### 修改内容
- `src/ops/security_analysis_resonance.rs`：恢复并重写第二阶段共振主模块，新增 `bootstrap_resonance_template_factors`、`evaluate_security_resonance`，并把模板池初始化、独立评估、隐式板块代理、事件覆盖、负向风险回填、快照落库整合到统一评估主链中。
- `src/tools/catalog.rs`：补充 `bootstrap_resonance_template_factors`、`evaluate_security_resonance` 到 stock tool catalog 与平铺 catalog，确保 Agent/Skill 可发现。
- `src/tools/dispatcher/stock_ops.rs`：新增两个新 Tool 的请求解析与分发实现，接入 stock dispatcher。
- `src/tools/dispatcher.rs`：补充主 dispatcher 路由，使 CLI / Agent / Skill 可以正式调用第二阶段新入口。

### 修改原因
- 用户已批准第二阶段走方案 B：只保留通过 Agent 操作 Skill 的主链，不引入方案 C 的额外平台管理层。
- 第二阶段要求把“传统行业候选因子池初始化”和“独立研究评估入口”落到底层，并复用到最终 fullstack 共振分析中。
- 用户明确要求共振逻辑不能只给正向文案，必须能把背离风险、行业基本盘和事件覆盖显式带出来。

### 方案还差什么?
- [ ] 继续把更多传统行业模板扩到航运、煤炭、有色、银行之外的板块，并补真实外部因子同步链路。
- [ ] 为独立评估入口补更多“近期背离优先级”“事件叠加权重”回归测试，减少后续参数漂移风险。

### 潜在问题
- [ ] 当前“最强背离风险回填”仍是启发式规则，若后续加入更多候选因子，需要再做参数校准与评估基线。
- [ ] 隐式板块代理目前依赖传入 `sector_symbol` 自动补位，若未来模板要求多代理或跨市场代理，需要扩展解析策略。
- [ ] 当前测试通过但编译仍有既有 dispatcher `dead_code` warning，本次未处理，需要后续统一清理。

### 关闭项
- 已通过 `cargo test --test security_analysis_resonance_cli -- --nocapture` 验证第二阶段红测转绿。
- 已通过 `cargo test --test integration_tool_contract -- --nocapture` 验证工具目录与主分发契约未回归。
- 已执行 `cargo fmt` 完成格式化。

### 记忆点（可选）
- 用户明确要求：所有操作都通过 Agent 操作 Skill 主链执行，不需要方案 C 这类额外平台管理层。
- 用户强调：共振结果不仅要给正向驱动，也要显式暴露背离风险与行业基本盘，避免只剩单边利多叙事。
## 2026-04-02
### 修改内容
- `src/ops/security_analysis_resonance.rs`：扩展 `bank` 模板候选因子池，新增 `real_estate_proxy`、`resident_consumption_proxy`、`loan_growth_proxy`、`pmi_proxy`。原因：用户明确指出银行共振不能只看板块、国债收益率和信用利差。目的：把地产、消费、贷款、PMI 纳入正式可注册、可计算、可落库、可评估的银行宏观共振底层。
- `tests/security_analysis_resonance_cli.rs`：先补红测再实现，新增银行模板注册与银行宏观因子评估两条回归测试，并加入 `601998.SH` / `512800.SH` 的银行夹具及正负向宏观序列。原因：本轮属于行为扩展，必须先锁住“银行模板 >= 7 因子”与“评估结果不只剩 sector proxy”。目的：保证后续继续扩因子时不会退化回“只会算板块相关性”。

### 修改原因
- 用户再次纠正：银行股的“共振”必须覆盖国债收益率、信用利差、地产、居民消费、贷款、PMI，不然不能叫共振。
- 当前平台虽然已经有共振底座，但 `bank` 模板只有 3 个因子，导致 `601998.SH` 的评估结果缺乏完整宏观基本盘表达。

### 方案还差什么?
- [ ] 当前新增的地产、消费、贷款、PMI 仍以代理序列入口为主，后续还需要接入真实宏观数据同步链路。
- [ ] 还没有把银行事件标签细化到地产政策、信贷投放、居民消费修复等更细颗粒度分类。
- [ ] 还没有基于真实历史样本回溯这些银行宏观因子的长期稳定性与权重校准。

### 潜在问题
- [ ] 如果后续真实宏观数据频率为月度而个股为日频，当前直接对齐收益率的逻辑可能需要补频率映射或插值规则。
- [ ] `loan_growth_proxy`、`pmi_proxy` 目前是平台级代理入口，不代表已经完成真实金融统计口径接入，分析层仍需注明来源性质。
- [ ] 现有仓库仍有历史 `dead_code` warning，本轮没有顺手清理，避免与银行共振扩容任务混做。

### 关闭项
- 已通过 `cargo test --test security_analysis_resonance_cli -- --nocapture`
- 已通过 `cargo test --test integration_tool_contract -- --nocapture`
- 已通过 `cargo fmt`
- 已验证银行模板注册因子数扩展到 7 个以上，且银行评估结果能带出宏观因子，不再只剩 `bank_sector_proxy`

### 记忆点（可选）
- 用户明确要求：证券分析必须优先走项目内 Tool/Skill 主链，不能脱离工具自由发挥。
- 用户明确要求：银行共振不能只做市场相关性，必须把利率、信用、地产、消费、贷款、PMI 这些基本盘维度正式纳入。
## 2026-04-02
### 修改内容
- `docs/plans/2026-04-02-security-decision-briefing-implementation.md`：新增统一证券决策底稿实施方案。原因：用户明确要求把“标准交付物”沉成 Tool，而不是停留在模板说明。目的：把咨询场景、投决会场景、后续 Agent 表决统一到同一份 briefing 底稿上，并把 Skill 作为基础门禁。

### 修改原因
- 用户明确要求：咨询和投决会看到的信息必须完全一致，不能形成两套事实口径。
- 用户明确要求：基础面、技术面、RSRS、共振面都要纳入统一交付，不允许只做“说明文档式模板”。
- 用户明确要求：Skill 不是可选说明，而是流程门禁，必须约束所有证券分析先走统一 briefing Tool。

### 方案还差什么?
- [ ] 还没有开始代码实现 `security_decision_briefing` Tool，本次先完成正式实施方案与任务拆解。
- [ ] 还没有实现 `committee_payload` 与后续 `security_committee_vote` 的真实代码合同。
- [ ] 还没有把新的 briefing Skill 门禁写入仓库技能目录。

### 潜在问题
- [ ] 如果后续直接跳过 briefing Tool 继续扩上层分析 Skill，会再次回到双口径问题，因此实现阶段必须优先做 Tool 合同。
- [ ] 现有技能链里已经存在 `security-analysis-v1`，后续新增 Skill 时需要避免职责重叠和路由冲突。

### 关闭项
- 已新增实施方案：`docs/plans/2026-04-02-security-decision-briefing-implementation.md`
- [ ] 未执行自动化测试：本次新增的是实施方案文档，尚未进入代码实现阶段

### 记忆点（可选）
- 用户明确要求：标准交付物不应只是模板，而要沉成统一 Tool，并服务咨询与投决会两个场景。
- 用户明确要求：Skill 要作为基础门禁，强制所有证券分析先走统一 briefing Tool，再允许 Agent 做判断或表决。
## 2026-04-02
### 修改内容
- `docs/plans/2026-04-02-signal-outcome-research-platform-implementation.md`：新增“信号结果研究平台”实施方案。原因：用户批准走方案C，不只做轻量 forward returns，而是把快照落库、未来收益回填、历史相似信号统计、briefing 接入和 committee 研究输入一次规划完整。目的：把证券分析从“解释当前指标”升级成“当前指标 + 历史验证 + 投决输入”的研究平台。

### 修改原因
- 用户明确要求：类似指标历史上应该出现过，系统不仅要看当前，还要抓未来一段时间内的涨跌表现。
- 用户明确要求：这块不应只做成辅助说明，而应作为平台能力，最终服务咨询、标准交付物和投决会。

### 方案还差什么?
- [ ] 还没有进入代码实现阶段，本次先完成方案C的正式任务拆解。
- [ ] 还没有定义 `signal_outcome_store`、`record_security_signal_snapshot`、`backfill_security_signal_outcomes` 等真实代码合同。
- [ ] 还没有把 research layer 接入 briefing 和 committee payload。

### 潜在问题
- [ ] 如果第一版相似信号算法做得过重，容易拖慢交付；方案里已限定先用“标签重合 + 结构分组”。
- [ ] forward returns 与 analog summary 属于历史统计结果，后续必须明确样本量和局限，不然容易被误用成确定性预测。

### 关闭项
- 已新增实施方案：`docs/plans/2026-04-02-signal-outcome-research-platform-implementation.md`
- [ ] 未执行自动化测试：本次新增的是实施方案文档，尚未进入代码实现阶段

### 记忆点（可选）
- 用户明确要求：证券指标平台不仅要解释“现在”，还要验证“历史上类似状态后面通常怎么走”。
- 用户明确要求：这类历史研究能力也要纳入统一 Tool/Skill 主链，而不是零散外挂分析。
## 2026-04-02
### 修改内容
- `tests/integration_tool_contract.rs`：新增 signal outcome research 合同红测，并修正 `security_decision_briefing` 合同测试的序列化写法。原因：方案 C 第一批要先锁 Tool 可发现性和 research contract 边界。目的：先把研究平台合同钉住，避免后续字段与入口漂移。
- `src/runtime/signal_outcome_store.rs`：新增信号研究 SQLite schema，并补充 snapshot 写入/读取、最新 snapshot 查询和 forward returns 批量写入。原因：用户要求把当前指标和后续结果做成可沉淀的研究资产。目的：为 Task 2/3 提供统一的研究底层。
- `src/ops/signal_outcome_research.rs`：新增 `record_security_signal_snapshot` 和 `backfill_security_signal_outcomes`，前者复用 `security_analysis_resonance` 生成完整技术面 + 共振快照并落库，后者回填 `1/3/5/10/20` 日 `forward return / max drawdown / max runup`。原因：方案 C 第一批要先做“快照 + future outcomes”的最小闭环。目的：把技术面、RSRS、共振层和动作偏向通过正式 Tool 主链沉淀为研究结果。
- `src/runtime/stock_history_store.rs`：新增 `load_rows_after`，专门用于读取 snapshot 日之后的未来窗口数据。原因：forward returns 回填必须拉取“快照日之后”的日线时序。目的：保持历史数据读取仍在 runtime store 层统一收口。
- `src/ops/stock.rs`、`src/ops/mod.rs`、`src/tools/dispatcher.rs`、`src/tools/dispatcher/stock_ops.rs`：把 `signal_outcome_research` 模块、`record_security_signal_snapshot` 和 `backfill_security_signal_outcomes` 接入 stock 主链。原因：用户明确要求所有研究与咨询都必须经过 Tool/Skill 主链执行。目的：把研究平台第一批能力变成可调用、可回归的正式 Tool。
- `tests/security_analysis_resonance_cli.rs`：新增 `record_security_signal_snapshot_persists_full_indicator_state` 和 `backfill_security_signal_outcomes_records_forward_metrics` 两条红绿回归，真实验证 `CSV -> SQLite -> resonance/fullstack -> signal_outcome_store` 主链。原因：用户要求高阶指标、RSRS、共振与动作偏向必须真正落库，并能延伸未来收益研究。目的：确保 Task 2/3 不是假 JSON 或内存结果，而是完整平台链路。

### 修改原因
- 用户明确要求：证券分析不只给当下结论，还要回答“历史上类似信号后面通常怎么走”，因此必须先把 snapshot 和 future outcomes 平台化。
- 用户多次纠正：RSRS、共振层、动作偏向不能在统一底稿里丢失，而且要等数据真正写入数据库后再谈后续 research 层扩展。

### 方案还差什么？
- [ ] 还没有实现 `study_security_signal_analogs` 和 `signal_outcome_research_summary`，目前只完成了 Task 1-3 的 `snapshot + forward returns` 闭环。
- [ ] 还没有把 research layer 接入 `security_decision_briefing` 和 committee payload，目前咨询层还没有消费本次新增的历史结果层。
- [ ] 还没有给 snapshot 生成 research tags，第一版相似信号检索还没有开始实现。

### 潜在问题
- [ ] `backfill_security_signal_outcomes` 第一版是固定 `1/3/5/10/20` 日窗口和 close-based 最大回撤/上冲，后续若要做更细粒度风险，还需要引入 high/low 口径校准。
- [ ] `indicator_digest` 第一版是 key=value 的可读字符串，利于审计和简单检索，但还不是最终的相似度检索结构。
- [ ] 仓库现有 `dispatcher.rs` 仍有较多历史 `dead_code` warning，本次未与研究平台任务混做清理。

### 关闭项
- 已通过 `cargo fmt`
- 已通过 `cargo test --test integration_tool_contract -- --nocapture`
- 已通过 `cargo test --test security_analysis_resonance_cli -- --nocapture`
- 已通过 `cargo test --test security_analysis_resonance_cli record_security_signal_snapshot_persists_full_indicator_state -- --nocapture`
- 已通过 `cargo test --test security_analysis_resonance_cli backfill_security_signal_outcomes_records_forward_metrics -- --nocapture`

### 记忆点（可选）
- 用户明确要求：这类量化研究底层要先做成平台，后面再搭 briefing / committee / Skill，不要先做一份上层说明模板。
- 用户明确要求：不要脱离项目内 Tool/Skill 主链自由发挥，本轮实现必须继续沿现有 dispatcher 和 runtime 主链扩展。
## 2026-04-02
### 修改内容
- `src/ops/security_decision_briefing.rs`：新增统一 `security_decision_briefing` 交付层，复用 `security_analysis_resonance` 输出基础面、技术面、共振面、执行阈值与 `committee_payload`。原因：把咨询场景与投决会场景收敛到同一份事实合同。目的：避免上层 Agent/Skill 再各自拼装不同口径的证券结论。
- `src/tools/contracts.rs`、`tests/integration_tool_contract.rs`：补 `security_decision_briefing` 顶层字段合同与 `ToolResponse.ok_serialized` 支撑。原因：先锁对外 JSON 合同，防止 dispatcher/catalog 接入后字段漂移。目的：让 CLI / Agent / Skill 都拿到同一份稳定响应结构。
- `src/tools/catalog.rs`、`src/tools/dispatcher.rs`、`src/tools/dispatcher/stock_ops.rs`：把 `security_decision_briefing` 接入 catalog 与 stock dispatcher。原因：该能力已经不是内部函数，必须成为正式 Tool。目的：保证后续 Skill 门禁和上层工作流能直接发现并调用。
- `tests/security_analysis_resonance_cli.rs`：补 briefing assembler、执行阈值、committee payload 回归测试，并继续沿用真实 `CSV -> SQLite -> Tool` 链路验收。原因：本轮属于行为新增，必须先用红绿测试锁住输出。目的：防止后续继续扩展 briefing 时破坏现有合同。
- `skills/security-analysis-v1/SKILL.md`、`skills/security-decision-briefing-v1/SKILL.md`：重写旧 Skill 门禁并新增 briefing 专属 Skill。原因：用户明确要求 Skill 不是说明文档，而是统一入口约束。目的：强制证券分析先过 briefing Tool，再进入后续判断或表决。
- `src/runtime/signal_outcome_store.rs`：为 `SecuritySignalForwardReturnRow` 补 `Serialize`。原因：全量回归时 `BackfillSecuritySignalOutcomesResult` 需要序列化 forward returns。目的：消除 signal outcome 研究层对 briefing 回归的编译阻塞。

### 修改原因
- 用户明确要求按顺序落地 `security_decision_briefing`：先红测合同，再做 assembler，再补执行层，再接 catalog/dispatcher，最后上 Skill 门禁，不能跳步骤。
- 用户明确要求咨询输出与投决会输出必须完全同口径，不能再出现“分析一份、表决另一份”的双事实链路。
- 用户明确要求任务完成后补任务日志，沉淀修改内容、风险、验证和记忆点，便于后续续做与回归。

### 方案还差什么?
- [ ] 还没有为 `security_decision_briefing` 补单独的 CLI smoke 场景测试；当前已通过 catalog/dispatcher 合同回归覆盖正式入口，但还可以再补一条端到端直接调用样例。
- [ ] `fundamental_brief / technical_brief / resonance_brief` 目前仍以 `serde_json::Value` 方式锁合同，后续如需更强静态约束，还可以继续下沉为强类型结构。

### 潜在问题
- [ ] `execution_plan` 当前依赖 `resistance_level_20`、`volume_ratio_20`、`ema_10`、`boll_middle`、`sma_50`、`atr_14` 等指标；若后续换市场或样本不足导致底层指标缺失，阈值解释可能需要补降级策略。
- [ ] `committee_payload.evidence_version` 当前采用 `security-decision-briefing:{symbol}:{analysis_date}:v1` 拼接规则；后续如果 briefing 合同继续扩展，记得同步升级版本而不是静默改字段。
- [ ] 仓库当前仍存在较多历史 `dead_code` warning，本轮没有顺手清理；它们不影响 briefing 功能，但会继续增加编译噪音。

### 关闭项
- 已通过 `cargo test --test integration_tool_contract security_decision_briefing_contract_exposes_required_top_level_fields -- --nocapture`
- 已通过 `cargo test --test security_analysis_resonance_cli security_decision_briefing_exposes_full_indicator_and_resonance_layers -- --nocapture`
- 已通过 `cargo test --test security_analysis_resonance_cli security_decision_briefing_exposes_execution_thresholds -- --nocapture`
- 已通过 `cargo test --test integration_tool_contract security_decision_briefing_is_cataloged -- --nocapture`
- 已通过 `cargo test --test security_analysis_resonance_cli security_decision_briefing_exposes_committee_payload -- --nocapture`
- 已通过 `cargo test --test security_analysis_resonance_cli -- --nocapture`
- 已通过 `cargo test --test integration_tool_contract -- --nocapture`

### 记忆点（可选）
- 用户明确要求：这条线要严格按既定顺序落地，先锁合同与红测，再做 assembler、执行层、catalog/dispatcher、Skill 门禁，不能为了省事跳步骤。
- 用户明确要求：任务做完后要立即补 `task-journal`，把修改内容、风险、未完项和验证结果沉淀到 `CHANGELOG_TASK.MD`。
## 2026-04-02
### 修改内容
- `src/ops/sync_template_resonance_factors.rs`：新增模板级共振因子同步主链，补 `SyncTemplateResonanceFactorsRequest/Result`、银行模板代理配方、代理序列变换与落库逻辑。原因：用户要求先把银行宏观共振平台底座做扎实，再谈中信银行最终结论。目的：把“模板因子补数”从临时分析动作沉成正式 Tool。
- `src/runtime/stock_history_store.rs`：新增 `load_rows_in_range`。原因：模板因子同步需要在同一时间窗口对齐多个代理 symbol。目的：继续把日期过滤与升序读取收口在 runtime store，而不是散落到上层逻辑。
- `src/ops/stock.rs`、`src/ops/mod.rs`、`src/tools/dispatcher.rs`、`src/tools/dispatcher/stock_ops.rs`、`src/tools/catalog.rs`：把 `sync_template_resonance_factors` 接入 stock 主链、dispatcher 与工具目录。原因：用户明确要求所有操作通过 Agent + Skill + Tool 主链执行。目的：让银行宏观共振补数成为正式可发现、可调用能力。
- `tests/security_analysis_resonance_cli.rs`：先补红测再实现绿测，新增目录可发现性与银行模板宏观代理写库回归测试。原因：本轮属于新行为扩展，必须先用测试锁住能力边界。目的：确保模板级补数不是一次性脚本，而是可回归平台能力。
- `runtime/security_resonance.db`：已通过新 Tool 写入银行模板 7 个宏观/代理因子序列。原因：需要让中信银行后续统一 briefing 真正使用这些因子。目的：让共振层从“只有模板定义”升级到“有真实序列可评估”。

### 修改原因
- 用户明确要求银行共振必须把国债、信用利差、地产、消费、贷款、PMI 正式纳入，而不是口头说会考虑。
- 用户明确要求分析必须优先走项目内 Tool/Skill 主链，不能脱离工具自由分析。
- 用户明确要求平台化思路先做底层，再在其上重跑中信银行结论。

### 方案还差什么?
- [ ] `loan_growth_proxy` 与 `pmi_proxy` 当前仍是平台级代理因子，后续还需要接官方宏观原始统计口径。
- [ ] 还没有把这套模板级补数扩展到石油、航运、煤炭等其他传统行业模板。
- [ ] 还没有把宏观因子同步结果进一步接入投决会专用汇总或 committee 表决层。

### 潜在问题
- [ ] 当前部分代理因子本质上是“市场代理”而非“官方宏观原始序列”，如果用户后续要求更严谨宏观解释，需要显式注明代理性质。
- [ ] `credit_risk_spread` 采用 `511260.SH / 511190.SH` 的相对强弱口径，若后续用户要改成更贴近信用利差原始定义，需要同步调整历史可比性。
- [ ] 仓库当前仍有较多历史 `dead_code` warning，本轮未顺手清理，避免和本任务混做。

### 关闭项
- 已通过 `cargo fmt`
- 已通过 `cargo test --test security_analysis_resonance_cli tool_catalog_includes_sync_template_resonance_factors`
- 已通过 `cargo test --test security_analysis_resonance_cli sync_template_resonance_factors_bank_writes_macro_proxy_series`
- 已通过真实命令 `cargo run --quiet --bin excel_skill` + `sync_template_resonance_factors` 成功补库 2082 个点
- 已通过真实命令重跑 `601998.SH` 的 `security_decision_briefing`，确认 `action_bias` 从 `add_on_strength` 收敛为 `hold_and_confirm`

### 记忆点（可选）
- 用户明确纠正过：不是“默认带共振”就行，而是要把共振真正算出来、写进数据库，再由后续分析和投决统一消费。
- 用户明确纠正过：银行股共振不能只看市场相关性，必须把利率、信用、地产、消费、贷款、PMI 这些基本盘维度正式纳入。

## 2026-04-02
### 修改内容
- `tests/integration_tool_contract.rs`：新增 `explicit_tool_catalog_request_returns_registered_tool_names` 红绿回归。原因：`security_committee_vote` 目录用例暴露出显式 `tool_catalog` 请求没有走正式路由。目的：锁住“空输入兜底”和“显式目录请求”返回同一份工具目录合同。
- `src/tools/dispatcher.rs`：补充 `tool_catalog` 正式分发分支，并在代码旁追加时间注释说明修改原因与目的。原因：此前新工具已注册进 catalog 常量，但 dispatcher 没有实际承接显式目录请求。目的：让 CLI、测试和 GUI 都能通过统一分发链发现 `security_committee_vote` 等 stock Tool。
- `src/license/service.rs`：把 `tool_catalog` 纳入公开白名单，并在代码旁追加时间注释说明修改原因与目的。原因：若授权门禁开启，显式目录请求仍可能被当成普通业务 Tool 拦截。目的：保证目录发现入口在授权前后都保持一致可访问。

### 修改原因
- 用户已批准按方案 C 处理：既修真实根因，又把目录契约通过测试稳定下来，而不是只改测试绕过问题。
- 根因已通过红测确认：显式请求 `{"tool":"tool_catalog","args":{}}` 时，返回数据里没有可解析的 `tool_catalog` 数组，说明问题在正式路由而不是注册常量。

### 方案还差什么?
- [ ] 还没有继续处理 `skills/security-committee-v1/SKILL.md` 与相关 Skill 门禁更新。
- [ ] 还没有对 `security_committee_vote` 之后的更大范围集成链做全量回归，只完成了本次目录路由相关回归。

### 潜在问题
- [ ] 若后续有人再次只维护 `main.rs` 的空输入目录兜底，而忘记同步 `dispatcher` 的显式目录分支，目录契约仍可能再次分叉。
- [ ] 当前仓库仍存在较多历史 `dead_code` warning，本次未混做清理；后续如果继续扩展 dispatcher，编译噪音仍会影响定位新问题。

### 关闭项
- 已通过 `cargo test --test integration_tool_contract explicit_tool_catalog_request_returns_registered_tool_names -- --nocapture`
- 已通过 `cargo test --test security_committee_vote_cli security_committee_vote_is_cataloged -- --nocapture`
- 已通过 `cargo test --test security_committee_vote_cli -- --nocapture`

### 记忆点（可选）
- 用户要求：发现 BUG 时先写能稳定复现问题的测试，再修复到通过为止；这次已按“先补红测，再补路由，再回归验证”执行。

## 2026-04-02
### 修改内容
- `src/ops/signal_outcome_research.rs`：为 `SecuritySignalAnalogMatch` 补充 `Deserialize` 派生，并在代码旁追加 2026-04-02 CST 的原因/目的注释。原因：`signal_outcome_research_summary` 需要把数据库里的 analog summary JSON 重新反序列化回 briefing 主链。目的：让历史研究摘要能稳定回读，而不是只能写入不能消费。
- `src/ops/security_committee_vote.rs`：为 `CommitteeMemberVote` 与 `SecurityCommitteeVoteResult` 补充 `Deserialize` 派生，并在代码旁追加 2026-04-02 CST 的原因/目的注释。原因：新增端到端 CLI 回归时，需要把 `security_committee_vote` 的结构化输出反序列化成强类型结果。目的：让投决会正式输出成为可往返、可校验、可供 GUI/自动化复用的正式合同。
- `tests/security_analysis_resonance_cli.rs`：先补红测再转绿，新增 `security_committee_vote_consumes_briefing_payload_with_historical_digest` 端到端回归。原因：用户要求投决会链路不能只在局部通过，而要确认 `security_decision_briefing -> committee_payload -> security_committee_vote` 真正打通。目的：锁住 briefing、历史研究摘要与投票结果沿同一条 CLI 主链流转的正式行为。
- `skills/security-analysis-v1/SKILL.md`：重写证券分析总 Skill 门禁，明确默认统一入口是 `security_decision_briefing`，投决会语义必须升级为 `security_decision_briefing -> security_committee_vote`。原因：旧文案仍停留在较宽泛的分析分流，没有把 briefing/vote 主链收口成强约束。目的：统一咨询口径与表决口径，禁止上层 Agent 绕过正式事实包。
- `skills/security-decision-briefing-v1/SKILL.md`：更新 briefing Skill，去掉“vote 尚未正式实现”的过时表述，补齐基于 `committee_payload` 调用 `security_committee_vote` 的正式门禁。原因：投决会 Tool 已正式落地，旧 Skill 已与真实实现脱节。目的：让上层使用方明确 briefing 是唯一事实入口，vote 是唯一正式表决入口。
- `skills/security-committee-v1/SKILL.md`：新增投决会专用 Skill，显式要求“先 briefing，后 vote；vote 只吃 committee_payload；禁止第二套事实包”。原因：用户要求把 Skill 门禁和整条链后续一起收尾，而不仅是让 Tool 能跑。目的：把投决会流程从实现能力升级为可执行、可复用、可审计的项目约束。

### 修改原因
- 用户明确要求继续把 Skill 门禁和整条链的后续收尾，不能停留在 `security_committee_vote` 可发现、可调用的局部完成状态。
- 用户明确要求发现 BUG 时先补能稳定复现问题的测试，再修复到通过为止；这次已按“先用端到端回归锁链路，再补反序列化合同，再回归验证”执行。
- 用户明确要求咨询链与投决会链使用同一份事实底稿，不能继续出现 briefing 一份、vote 又手工拼一份的双事实链路。

### 方案还差什么?
- [ ] Skill 门禁目前是文档级约束，后续如果要进一步防止误用，还可以考虑在上层调度器或 Agent 编排层加入更强的自动门禁校验。
- [ ] 当前端到端回归主要覆盖 `standard` 模式，后续还可以继续补 `strict` 与 `advisory` 模式下的正式链路测试。
- [ ] 还没有补“授权开启时显式 `tool_catalog` 仍可访问”与“空输入目录和显式 `tool_catalog` 完全一致”的更大范围目录契约回归。

### 潜在问题
- [ ] 仓库当前仍存在较多历史 `dead_code` warning，本轮没有混做清理；后续排查新问题时编译噪音仍然存在。
- [ ] `skills/security-analysis-v1/SKILL.md`、`skills/security-decision-briefing-v1/SKILL.md`、`skills/security-committee-v1/SKILL.md` 目前只是在流程层强约束，如果未来有人绕过 Skill 直接拼底层 Tool，仍需要依赖测试和代码 review 兜底。
- [ ] 端到端测试依赖真实的研究夹具链路，执行时间较长；后续如果继续扩大量化研究回归，建议分层增加更轻量的 store/ops 单测，避免总体回归进一步变慢。

### 关闭项
- 已通过 `cargo test --test security_analysis_resonance_cli security_decision_briefing_exposes_available_historical_digest_after_analog_study -- --nocapture`
- 已通过 `cargo test --test security_analysis_resonance_cli security_committee_vote_consumes_briefing_payload_with_historical_digest -- --nocapture`
- 已通过 `cargo test --test security_committee_vote_cli -- --nocapture`

### 记忆点（可选）
- 用户再次明确要求：这条线不能只做到 Tool 层可用，还要把 Skill 门禁和整条正式链路一起收尾，避免实现有了但流程约束没落地。
- 用户再次明确要求：投决会必须只消费 `security_decision_briefing` 产出的同一份 `committee_payload`，不能手工再拼第二套事实包。

## 2026-04-02
### 修改内容
- `tests/security_analysis_resonance_cli.rs`：新增 `security_decision_briefing_includes_default_committee_recommendations_for_all_modes` 红绿回归，并在测试里把 `standard / strict / advisory` 三种内嵌建议与正式 `security_committee_vote` 结果逐一比对。原因：用户明确要求普通股票分析报告默认就要带投决会建议，而不是让用户先知道要不要开投决会。目的：锁住“报告默认带投决建议”且“建议必须复用正式 vote Tool”的行为边界。
- `src/ops/security_decision_briefing.rs`：为 `SecurityDecisionBriefingResult` 新增 `committee_recommendations` 顶层字段，并补充 `CommitteeRecommendations / CommitteeRecommendationEntry` 结构。原因：需要让 briefing 直接承载默认投决建议，而不是只返回原始 `committee_payload`。目的：让上层报告在一个入口里同时看到分析事实与可执行建议。
- `src/ops/security_decision_briefing.rs`：新增 `build_committee_recommendations` 与 `build_committee_recommendation_entry`，内部统一复用 `security_committee_vote` 生成 `standard / strict / advisory` 三套结果。原因：用户强调不能再出现报告一套、投决会一套的双口径。目的：确保 briefing 默认建议与独立 vote Tool 永远同源、同规则、同结果。
- `src/ops/security_decision_briefing.rs`：为 briefing 错误类型补充 `CommitteeVote` 分支，并在新增代码旁追加 2026-04-02 CST 的原因/目的注释。原因：默认内嵌投决建议后，briefing 需要显式承接 vote 生成失败。目的：保证链路错误来源可追踪、可测试、可向上层清晰暴露。

### 修改原因
- 用户明确要求：用户并不知道什么时候该“上投决会”，所以系统应该默认在股票分析报告里给出投决会建议，而不是把流程知识转嫁给用户。
- 用户明确细化了三种模式语义：`standard` 是个股分析报告默认投决会建议，`strict` 面向涉及金额与买卖动作的严格交易建议，`advisory` 面向已有持仓判断；因此需要把三种模式一并准备进 briefing。
- 用户再次强调：投决会建议不能手写摘要，必须沿正式 Tool 主链复用，避免同一份报告和独立投决会跑出两套不同结论。

### 方案还差什么?
- [ ] 目前默认内嵌建议已经准备好三种模式，但还没有根据用户自然语言意图自动挑选“当前最该引用哪一种”；上层 Agent/报告模板后续还可以继续补一层推荐逻辑。
- [ ] 还没有在 `integration_tool_contract` 里单独锁定 `committee_recommendations` 的完整字段合同；当前主要由专项 CLI 回归覆盖。
- [ ] 还没有继续扩到更多报告模板，例如把 `strict` 和 `advisory` 的摘要段落直接写进最终中文报告排版中。

### 潜在问题
- [ ] `security_decision_briefing` 现在默认会额外跑三次 `security_committee_vote`，虽然都是内存内逻辑，但后续如果 vote 规则继续变重，需要关注 briefing 总耗时。
- [ ] 当前 `default_mode` 固定为 `standard`，这是按“普通个股分析报告”默认场景设定的；若未来存在不同报告类型，可能需要再引入更显式的报告意图字段。
- [ ] 仓库当前仍有较多历史 `dead_code` warning，本轮没有混做清理，编译噪音仍会继续存在。

### 关闭项
- 已通过 `cargo test --test security_analysis_resonance_cli security_decision_briefing_includes_default_committee_recommendations_for_all_modes -- --nocapture`
- 已通过 `cargo test --test security_analysis_resonance_cli security_committee_vote_consumes_briefing_payload_with_historical_digest -- --nocapture`
- 已通过 `cargo test --test integration_tool_contract security_decision_briefing_contract_exposes_required_top_level_fields -- --nocapture`

### 记忆点（可选）
- 用户再次明确要求：不要假设用户知道“什么时候该开投决会”，应由系统默认把投决建议准备进股票分析报告。
- 用户再次明确要求：`standard / strict / advisory` 不是技术实现细节，而是三种不同业务语义，后续任何报告或 Skill 编排都要按这个口径使用。
## 2026-04-02
### 修改内容
- `README.md`：整页重写项目入口说明，并在文档内补 2026-04-02 CST 的修改原因/目的注释。原因：旧 README 仍以历史 SheetMind/Excel 叙事为主，无法准确反映当前证券分析与投决会主链。目的：让开发者、AI 和后续维护者都从同一份权威入口理解当前项目。
- `docs/AI_HANDOFF.md`：新增统一 AI 交接手册，并在文档内补 2026-04-02 CST 的修改原因/目的注释。原因：仓库原有两份交接摘要已经碎片化，且混有旧主线残留。目的：把当前正式主链、禁止回退事项、接手顺序、风险和文档同步规则收拢成单一权威手册。
- `docs/DOCUMENTATION_INDEX.md`、`docs/DOC_UPDATE_POLICY.md`：新增文档索引与文档更新规则，并在文档内补 2026-04-02 CST 的修改原因/目的注释。原因：用户明确要求 README、日志和 AI 交接手册需要随时更新。目的：把“要不要补文档”升级为正式维护规则，并为 docs 目录提供统一导航入口。
- `docs/交接摘要_给后续AI.md`、`docs/交接摘要_证券分析_给后续AI.md`：改成跳转页，并在文档内补 2026-04-02 CST 的修改原因/目的注释。原因：继续并行维护旧摘要会让交接入口继续分叉。目的：保留历史文件名兼容性，同时把后续维护统一收口到 README 与 `docs/AI_HANDOFF.md`。

### 修改原因
- 用户明确要求把 README、日志、AI 交接手册补齐，并且这些文档需要随时更新，不能继续只靠零散口头说明。
- 当前证券分析、briefing、投决会、Skill 门禁已经形成新的正式主链，原有文档入口和旧摘要不再足够准确。
- 如果不把“权威入口”和“同步更新规则”明确写下来，后续 AI 或开发者很容易继续从过期摘要接手，导致主线理解再次偏移。

### 方案还差什么?
- [ ] 还没有把 `docs/acceptance/`、`docs/plans/` 里的历史文档逐份补“当前是否仍有效”的状态标记；本次先完成权威入口收口，避免把任务面继续拉大。
- [ ] 还没有把“提交前文档检查”做成自动脚本或 CI 规则；当前仍依赖人工按 `DOC_UPDATE_POLICY` 执行。

### 潜在问题
- [ ] 旧文档虽然已经改成跳转页，但如果后续有人继续直接编辑历史计划或摘要而不更新权威入口，信息仍可能再次分叉。
- [ ] 本次主要是文档体系重整，没有执行自动化测试；如果后续 README 中新增的命令示例发生漂移，需要通过功能回归一并校正。

### 关闭项
- [ ] 未执行自动化验证：本次仅涉及 README 与 docs 文档重整，没有改动生产代码或测试代码。

### 记忆点（可选）
- 用户再次明确要求：README、任务日志、AI 交接手册不是可选补充材料，而是需要随功能同步维护的正式资产。
## 2026-04-08
### 修改内容
- `src/ops/technical_consultation_basic.rs`：给 `TechnicalConsultationBasicResult` 新增顶层 `analysis_date` 与 `evidence_version` 字段，并在结果构造处按 `technical-consultation-basic:<symbol>:<analysis_date>:v1` 生成统一证据版本。目的：在保留 `as_of_date` 兼容的同时，把统一合同前移到技术基础层。
- `src/ops/security_analysis_contextual.rs`：给 `SecurityAnalysisContextualResult` 新增顶层 `analysis_date` 与 `evidence_version` 字段，并复用 `stock_analysis.analysis_date` 生成 `security-analysis-contextual:<symbol>:<analysis_date>:v1`。目的：让环境层直接对外暴露统一日期和证据版本。
- `src/ops/security_analysis_fullstack.rs`：给 `SecurityAnalysisFullstackResult` 新增顶层 `analysis_date` 与 `evidence_version` 字段，并复用 `technical_context.analysis_date` 生成 `security-analysis-fullstack:<symbol>:<analysis_date>:v1`。目的：让 fullstack 聚合层成为 briefing 之下的统一事实入口。
- `src/ops/security_analysis_resonance.rs`：给 `SecurityAnalysisResonanceResult` 新增顶层 `analysis_date` 与 `evidence_version` 字段，并复用 `base_analysis.analysis_date` 生成 `security-analysis-resonance:<symbol>:<analysis_date>:v1`。目的：让 resonance 顶层也能被后续链路直接消费，不再依赖嵌套字段回钻。
- `tests/integration_tool_contract.rs`、`tests/security_analysis_contextual_cli.rs`、`tests/security_analysis_fullstack_cli.rs`、`tests/security_analysis_resonance_cli.rs`：本轮继续沿用并验证已补好的红测，确保四层统一合同字段按方案 C 生效。

### 修改原因
- 用户已明确批准方案 C：本轮只做“Skill 门禁最小加固 + 证券分析主链全链统一合同最小补齐”，不扩业务判断逻辑，不做大重构。
- 当前链路中 `security_decision_briefing` 和 `security_committee_vote` 已有统一顶层 `analysis_date / evidence_version`，但更下层的 `technical/contextual/fullstack/resonance` 仍未统一，导致上层消费必须回钻嵌套结构。
- 用户明确要求发现问题先用测试锁定，再做最小修复；这轮已按“先红测，再补字段，再回归”的方式执行。

### 方案还差什么?
- [ ] `security_decision_briefing` 与 `security_committee_vote` 之间仍有一组历史测试失败，当前定位到 `committee_payload.key_risks` 为空时会触发 `IncompleteCommitteePayload("key_risks 不能为空")`；这不属于本轮统一合同字段本身，但需要后续单独收口。
- [ ] Skill 门禁当前仍主要依赖 Skill 文档与链路约束，尚未加入更强的自动化门禁执行器。
- [ ] 统一合同这轮只覆盖了 `analysis_date / evidence_version`，后续若要扩公共字段，需要先补更明确的跨层规格。

### 潜在问题
- [ ] `cargo test --test security_analysis_resonance_cli -- --nocapture --test-threads=1` 当前会暴露 4 个旧失败用例，根因不是本轮新增字段缺失，而是 briefing 到 committee 的事实包完整性要求未满足，并且首个 panic 会导致测试里的环境锁进入 `PoisonError` 连锁失败。
- [ ] 仓库仍存在较多历史 `dead_code` warnings，本轮未处理；后续排查新问题时编译输出噪音仍然较大。
- [ ] Windows 下测试超时后会残留 `cargo`/测试二进制占用，可能触发 `LNK1104` 或 `os error 5`，后续长跑测试前要先确认没有残留进程。

### 关闭项
- 已通过 `cargo test --test integration_tool_contract technical_consultation_basic_contract_exposes_bullish_continuation_conclusion -- --exact --nocapture`
- 已通过 `cargo test --test security_analysis_contextual_cli security_analysis_contextual_uses_proxy_profiles_when_symbols_are_omitted -- --exact --nocapture`
- 已通过 `cargo test --test security_analysis_fullstack_cli security_analysis_fullstack_aggregates_technical_fundamental_and_disclosures -- --exact --nocapture`
- 已通过 `cargo test --test security_analysis_resonance_cli security_analysis_resonance_exposes_unified_analysis_date_and_evidence_version -- --exact --nocapture`
- 已通过 `cargo test --test integration_tool_contract -- --nocapture --test-threads=1`
- 已通过 `cargo test --test security_analysis_contextual_cli -- --nocapture --test-threads=1`
- 已通过 `cargo test --test security_analysis_fullstack_cli -- --nocapture --test-threads=1`
- 已通过 `cargo fmt --all`
- 已定位但未关闭 `cargo test --test security_analysis_resonance_cli -- --nocapture --test-threads=1` 的历史失败：`security_decision_briefing_exposes_committee_payload` 首先因 `key_risks 不能为空` 失败，其后 3 个用例因测试环境锁 `PoisonError` 连锁失败。

### 记忆点（可选）
- 用户已明确要求：这条线先做最小、可验证的统一合同，不要借机扩业务判断或大改 Skill 架构。
- 用户再次强调：每次代码任务结束后都要补 `CHANGELOG_TASK.MD`，并把已验证命令、未完成项、潜在风险和记忆点写清楚。

### 修改内容（补充）
- `src/ops/security_decision_briefing.rs`：在 `build_committee_payload()` 前新增 `build_committee_key_risks()` 最小兜底逻辑，当上游 `integrated_conclusion.risk_flags` 为空时，优先退化到首个负向共振风险，再退化到基于 `stop_loss_price / invalidation_price` 的失效风险。目的：只修通 `briefing -> committee` 这一小段最小闭环，不扩散修改到更大的风险抽取逻辑。

### 关闭项（补充）
- 已通过 `cargo test --test security_analysis_resonance_cli security_decision_briefing_exposes_committee_payload -- --exact --nocapture`
- 已通过 `cargo test --test security_analysis_resonance_cli security_decision_briefing_exposes_execution_thresholds -- --exact --nocapture`
- 已通过 `cargo test --test security_analysis_resonance_cli security_decision_briefing_exposes_full_indicator_and_resonance_layers -- --exact --nocapture`
- 已通过 `cargo test --test security_analysis_resonance_cli security_decision_briefing_exposes_vote_ready_committee_payload -- --exact --nocapture`

### 潜在问题（补充）
- [ ] `cargo test --test security_analysis_resonance_cli -- --nocapture --test-threads=1` 在当前 Windows 环境下仍会受到 `cargo`/构建目录锁噪音干扰，导致整组回归输出不稳定；但本轮直接相关的 4 个失败用例已经逐个转绿。

### 记忆点（补充）
- 用户明确要求：先把“最小补齐”做完，再单独说明如果要做完整版本还差什么，因此这轮对 `committee_payload.key_risks` 只做最小兜底，不顺手扩展更完整的风险抽取体系。
## 2026-04-08
### 修改内容
- `src/ops/foundation.rs`：新增 `ontology_schema`、`ontology_store`、`knowledge_record`、`knowledge_graph_store`、`capability_router`、`roaming_engine`、`retrieval_engine`、`evidence_assembler`、`navigation_pipeline` 模块导出。目的：把 foundation 知识导航内核正式收口到统一入口。
- `src/ops/foundation/roaming_engine.rs`、`src/ops/foundation/retrieval_engine.rs`、`src/ops/foundation/evidence_assembler.rs`、`src/ops/foundation/navigation_pipeline.rs`：补齐方案 C 第一阶段缺失的四段最小实现。目的：形成独立的 `question -> NavigationEvidence` 最小闭环。
- `tests/roaming_engine_unit.rs`、`tests/retrieval_engine_unit.rs`、`tests/evidence_assembler_unit.rs`、`tests/navigation_pipeline_integration.rs`：按 TDD 新增红绿测试，锁定漫游约束、候选域内检索、证据装配与统一入口闭环。目的：先用测试钉住 phase 1 契约，再补最小实现。
- `README.md`、`docs/AI_HANDOFF.md`、`progress.md`、`findings.md`：同步记录 foundation 第一阶段已完成边界、未完成项、回归范围与环境风险。目的：避免后续协作把独立 navigation kernel 误读为完整知识库系统。

### 修改原因
- 用户已批准“方案 C 第一阶段”，要求先在当前仓库内补齐独立 foundation 知识导航内核，暂不接入现有证券分析主链。
- 当前仓库虽然已有 ontology、store、router 等前置组件，但 `roam -> retrieve -> assemble -> pipeline` 闭环尚未完成，无法支撑独立的知识导航链路验证。
- 用户明确要求按 TDD 推进，并在任务结束后补 `CHANGELOG_TASK.MD` 风格日志，因此本轮必须先补失败测试、再补最小实现、最后同步文档。

### 方案还差什么?
- [ ] 还没有实现知识入库流水线、metadata filter、向量检索与持久化知识库。
- [ ] 还没有把 foundation navigation kernel 适配接入 `security_analysis_* / security_decision_briefing / security_committee_vote` 主链。
- [ ] 还没有对 foundation 第一阶段补充 README 级使用示例或上层调用样例代码。

### 潜在问题
- [ ] 当前 `retrieval_engine` 仅采用关键词交集评分；当知识节点规模扩大或文本同质化增强时，命中排序可能不足以支撑复杂检索场景。
- [ ] Windows 环境下若残留 `excel_skill.exe` 或 `cargo` 进程，`cargo test` 仍可能触发 `os error 5` 文件占用失败。
- [ ] `task-journal` 技能自带 `new_entry.ps1` 在当前 PowerShell 编码环境下解析失败；后续若继续依赖该脚本，需先修复脚本编码或执行环境。

### 关闭项
- 已通过 `cargo test --test roaming_engine_unit -- --nocapture`
- 已通过 `cargo test --test retrieval_engine_unit -- --nocapture`
- 已通过 `cargo test --test evidence_assembler_unit -- --nocapture`
- 已通过 `cargo test --test navigation_pipeline_integration -- --nocapture`
- 已通过 `cargo test --test ontology_schema_unit --test ontology_store_unit --test knowledge_record_unit --test knowledge_graph_store_unit --test capability_router_unit --test roaming_engine_unit --test retrieval_engine_unit --test evidence_assembler_unit --test navigation_pipeline_integration -- --nocapture`

### 记忆点（可选）
- 用户要求：进入代码修改前必须先给多个方案并等待批准；本轮是在既有批准的“方案 C 第一阶段”前提下继续执行，后续同类任务不能跳过这一步。
- 用户要求：每次代码任务完成后都要补 `CHANGELOG_TASK.MD`，即使技能脚本失效也要按同格式手工补齐，不能省略。
## 2026-04-08
### 修改内容
- `src/ops/security_committee_vote.rs`：把 `committee_vote` 的关键风险消费入口统一收口到 `effective_key_risks(payload)`，并在事件席位、风控席位、标准条件、payload 校验里接入 `risk_breakdown -> key_risks` 兼容逻辑。目的：让标准收口版下的结构化风险合同可以直接驱动投决规则，同时保持旧摘要字段兼容。
- `tests/security_committee_vote_cli.rs`：给手工构造的 `CommitteePayload` 测试夹具补齐 `risk_breakdown`，并写入与 `key_risks` 对齐的四类风险样例。目的：修复新合同引入后测试夹具缺字段导致的编译失败，并锁住 CLI 投决输入合同。
- `src/ops/security_decision_briefing.rs`、`tests/security_analysis_resonance_cli.rs`、`tests/integration_tool_contract.rs`：沿用本轮已完成的标准收口版改动，确认 `committee_payload.risk_breakdown` 作为正式合同字段对外暴露，`key_risks` 退化为兼容摘要。目的：把 briefing 与 committee 的风险合同边界正式固定下来。

### 修改原因
- 用户已批准“标准收口版”，本轮目标不是继续做最小兜底，而是把 `committee_payload` 的风险合同从扁平 `key_risks` 升级为结构化 `risk_breakdown`，同时保证旧调用方不被打断。
- 之前 `security_decision_briefing_exposes_committee_payload` 与 `security_decision_briefing_exposes_vote_ready_committee_payload` 的失败，不是子进程机制本身有问题，而是 `committee_vote` 仍有部分逻辑直接读取旧 `payload.key_risks`，导致新合同路径未完全贯通。
- 用户要求每次代码任务结束后补 `CHANGELOG_TASK.MD`，并把修改原因、验证结果、剩余问题和记忆点写清楚；本次按该要求手工补齐日志。

### 方案还差什么?
- [ ] 目前只完成了 `briefing + committee_payload + committee_vote` 这一段的标准收口，尚未继续向更上游 `integrated_conclusion` 做彻底重构。
- [ ] 还没有为 `risk_breakdown` 增加更细的契约校验，例如四类风险项的 `severity/category/rationale` 是否满足统一枚举与非空门禁。
- [ ] 还没有跑全量 `cargo test`；本轮只回归了与标准收口版直接相关的 4 组测试，避免在现有 Windows 锁文件环境下扩大噪声面。

### 潜在问题
- [ ] `effective_key_risks(payload)` 当前采用“优先旧 `key_risks`，否则退化读取 `risk_breakdown` 每类首条 headline”的兼容策略；如果后续外部调用方同时传入两套不一致数据，仍可能产生摘要与结构化事实不一致的问题。
- [ ] 当前测试夹具里的 `risk_breakdown` 仍是最小样例，尚未覆盖“某一类风险为空、另一类风险有多条”的排序与摘要派生边界。
- [ ] Windows 环境下仍存在 `cargo`/构建目录锁等待；如果后续要跑全量回归，仍需注意 `LNK1104`、`os error 5` 或 package cache 锁等待噪声。

### 关闭项
- 已通过 `cargo test --test security_committee_vote_cli -- --nocapture`
- 已通过 `cargo test --test security_analysis_resonance_cli security_decision_briefing_exposes_committee_payload -- --exact --nocapture`
- 已通过 `cargo test --test security_analysis_resonance_cli security_decision_briefing_exposes_vote_ready_committee_payload -- --exact --nocapture`
- 已通过 `cargo test --test integration_tool_contract security_decision_briefing_contract_exposes_required_top_level_fields -- --exact --nocapture`
- 已通过 `cargo fmt --all`

### 记忆点（可选）
- 用户再次强调：这种持续主线里的代码收口要先给方案、等批准，再执行修改；即使已有前序上下文，也不能跳过“先描述方案”的环节。
- 用户再次强调：任务完成后必须调用 task-journal 思路补 `CHANGELOG_TASK.MD`，不能因为技能脚本或编码环境不稳定就省略任务日志。
## 2026-04-08
### 修改内容
- `src/ops/security_decision_briefing.rs`：补齐 `CommitteeSubjectProfile` 最小分流规则，在 `build_summary()`、`build_committee_payload()`、`build_committee_recommendations()`、`build_fundamental_risk_items()`、`build_historical_digest()` 中接入 ETF 口径分支。目的：先把 ETF 正式纳入 briefing / committee 主链的最小可落地合同。
- `src/ops/security_committee_vote.rs`：给 `build_fundamental_opinion()` 增加 ETF 专用投票分支，并新增 `is_etf_subject()`。目的：避免 ETF 因“没有个股财报”被基本面席位直接 `defer`。
- `tests/security_committee_vote_cli.rs`：补齐 `subject_profile` 测试夹具，新增 ETF 夹具与 `security_committee_vote_etf_fundamental_reviewer_uses_fund_review_semantics` 回归测试。目的：锁住 ETF 基本面席位的新语义。
- `src/ops/security_decision_briefing.rs`：新增 `infer_subject_profile()` 单元测试。目的：锁住 A 股 ETF / 个股最小识别规则。
- `runtime/stock_history.db`：通过 `excel_skill.exe` + `EXCEL_SKILL_STOCK_DB` 显式指定仓库数据库，将 `159866.SZ` 同步到 `2026-04-07`，补入 `2026-04-03`、`2026-04-07` 两条数据。目的：让本地正式仓库库与最新分析日期对齐。

### 修改原因
- 用户已批准“方案 3”，要求先把 ETF / 海外标的纳入正式分析主链的最小版本，再同步 `159866.SZ` 到 `2026-04-07` 后重跑分析。
- 当前主链的真实断点不在技术分析本身，而在 briefing / committee 仍默认按“个股财报语义”解释 ETF，导致 ETF 无法严肃进入投决口径。
- 本轮还实际发现 CLI 默认运行时库路径会落到 `E:\\TradingAgents\\.excel_skill_runtime\\stock_history.db`，如果不显式指定 `EXCEL_SKILL_STOCK_DB`，抓到的数据不会进入仓库主库。

### 方案还差什么?
- [ ] 当前只完成了 ETF 在 briefing / committee 合同层的最小分流，还没有补 ETF 专用 resonance 模板与更完整的 ETF / 海外标的研究模板池。
- [ ] `security_decision_briefing` 里的旧历史场景文案仍有一段不可达旧代码残留，本轮先用 `#[allow(unreachable_code)]` 压住告警，后续应做一次编码清理后彻底删掉旧块。
- [ ] 还没有把 ETF 主链做成完整的端到端 acceptance case，目前验证重点仍是 `committee_vote` 语义和 `technical_consultation_basic` 最新数据回归。

### 潜在问题
- [ ] 当前 `infer_subject_profile()` 仅覆盖 A 股 ETF 的最小前缀规则：`.SZ` 的 `15/16` 与 `.SH` 的 `51/56/58`；后续若接港股、美股、海外 ETF，需要扩展正式映射规则。
- [ ] `sync_stock_price_history` 本次只成功补到 `2026-04-03` 和 `2026-04-07` 两个交易日；若后续需要完整校验 4 月上旬连续行情，还要继续核对节假日与源站覆盖情况。
- [ ] 当前仓库仍存在大量历史 `dead_code` warnings，虽然与本轮功能无直接冲突，但会抬高后续排障噪声。

### 关闭项
- 已通过 `cargo fmt --all`
- 已通过 `cargo test --test security_committee_vote_cli -- --nocapture`
- 已通过 `cargo test infer_subject_profile --lib -- --nocapture`
- 已通过 `excel_skill.exe` + `sync_stock_price_history` 将 `159866.SZ` 同步到仓库数据库 `E:\\TradingAgents\\TradingAgents\\runtime\\stock_history.db`
- 已通过 `excel_skill.exe` + `technical_consultation_basic` 使用 `2026-04-07` 数据重跑 `159866.SZ`

### 记忆点（可选）
- 以后只要是“同步股票数据后立刻分析”的任务，不能把补数和分析并行跑；必须先确认 SQLite 已写入，再顺序重跑分析，否则会读到补数前快照。
- 以后用 `excel_skill.exe` 做真实抓数或分析时，要优先显式设置 `EXCEL_SKILL_STOCK_DB=E:\\TradingAgents\\TradingAgents\\runtime\\stock_history.db`，避免默认写入 `.excel_skill_runtime` 造成“看起来抓到了、实际主库没变”的假象。

## 2026-04-08
### 修改内容
- `src/ops/security_committee_vote.rs`：在 `validate_committee_payload()` 中接入 `validate_risk_breakdown_consistency()`，并新增 `derive_key_risks_from_breakdown()`。目的：把 `risk_breakdown` 提升为投决会门禁主源，拒绝分类错桶、字段缺失和摘要漂移。
- `src/ops/security_committee_vote.rs`：收紧 `effective_key_risks()` 兼容逻辑，仅允许复用已通过强一致性校验的 `key_risks`，否则统一从 `risk_breakdown` 派生。目的：保留旧字段兼容，同时取消“第二套风险事实”。
- `tests/security_committee_vote_cli.rs`：保留两条失败测试锁住“分类错桶”和“key_risks 与 risk_breakdown 不一致”场景，并把合法夹具中的 `key_risks` 同步到固定派生顺序。目的：让正负样本都符合方案 B 的新合同。

### 修改原因
- 用户已批准方案 B，要求把 `risk_breakdown` 提升为 `committee_payload` 的强一致性门禁主源，并要求 `key_risks` 必须与结构化风险拆解严格一致。
- 上一轮虽然已经补了 `risk_breakdown` 合同与失败测试，但正式实现仍停留在“旧 `key_risks` 优先”的兼容态，投决会还没有真正按新合同拦截脏 payload。
- 新门禁落地后，原有合法测试夹具仍使用旧摘要顺序；若不一起同步，会把“夹具过时”误判成“规则出错”。

### 方案还差什么?
- [ ] 当前只完成了 `committee_payload -> committee_vote` 这段的强一致性收口，还没有继续前推到 `security_decision_briefing` 的生成门禁。
- [ ] 还没有覆盖“单个桶内多条风险项”的摘要派生与排序测试；当前只锁住了每桶首条 headline 的最小规则。
- [ ] 还没有跑全量 `cargo test`；本轮只跑了与方案 B 直接相关的定向回归，避免在当前 Windows 环境下放大无关噪声。

### 潜在问题
- [ ] 当前 `derive_key_risks_from_breakdown()` 仍采用“每个桶取首条 headline”的派生策略；如果后续要引入更复杂的优先级规则，需要先补测试再改实现。
- [ ] 仓库中仍存在较多历史 `dead_code` warnings，本轮未处理；后续做全量回归时输出噪声仍会偏大。
- [ ] 当前工作区本身是脏的，存在多处用户既有改动与未跟踪文件；后续提交或推送前需要明确只收本轮相关变更。

### 关闭项
- 已通过 `cargo test --test security_committee_vote_cli -- --nocapture`
- 已通过 `cargo fmt --all`
- 已通过格式化后的二次回归 `cargo test --test security_committee_vote_cli -- --nocapture`

### 记忆点（可选）
- 用户再次强调：这种持续主线里的代码收口也必须先给方案、等批准，再执行修改。
- 用户再次强调：每次代码任务结束后都要按 `task-journal` 思路补 `CHANGELOG_TASK.MD`，不能因为历史编码脏或技能脚本不可用而省略。

## 2026-04-08
### 修改内容
- `docs/plans/2026-04-08-closed-loop-investment-research-roadmap.md`：新增“闭环投研平台路线图”正式文档。目的：把后续主线从“继续加厚投决会”收口到“赔率系统 -> 仓位管理 -> 投后复盘 -> 市场结构 -> 技术面平衡计分卡 -> 深层信息面增强”。
- `docs/DOCUMENTATION_INDEX.md`：补挂新路线图入口。目的：让后续协作者能从统一索引直接定位这份正式计划，而不是继续从零散对话里推断优先级。
- `docs/AI_HANDOFF.md`：更新后续优先级建议，明确 `security_committee_vote` 已进入最小可用收口态，下一阶段应转向闭环建设。目的：避免后续 AI 继续默认把投决会当作第一主线。

### 修改原因
- 用户明确指出：如果没有仓位管理与投后复盘，平台深度不足，不能算闭环体系。
- 当前仓库虽然已经完成“投前分析 + briefing + 投决会”基础设施，但还缺赔率、仓位和复盘三段，继续深挖投决会会偏离平台成型方向。
- 用户要求把整套后续思路写成一份正式文档并放到平台文档体系里，供后续 AI 和开发继续接续。

### 方案还差什么?
- [ ] 这次只完成了路线图文档化，还没有开始对应的业务实现，尤其是 `signal_outcome_research` 的赔率层、仓位层和复盘层。
- [ ] 还没有把“免费数据源分层”进一步拆成可执行的数据适配清单；这需要单独开下一轮方案。
- [ ] 还没有把新路线同步到 `task_plan.md / findings.md / progress.md` 这一组文件化计划体系中。

### 潜在问题
- [ ] 当前路线图里对“深层免费数据源”仍按分层策略表述，没有落到逐源核验；后续若直接实现这部分，仍可能被源稳定性卡住。
- [ ] 如果后续协作者只看旧的阶段文档而不看新路线图，仍可能继续把资源投到投决会制度层。
- [ ] 本轮只改文档，没有跑自动化测试；后续真正开始实现闭环模块时，需要重新按 TDD 建测试入口。

### 关闭项
- [ ] 未执行自动化验证：本轮仅涉及路线图、索引与交接文档更新，没有修改生产代码或测试代码。

### 记忆点（可选）
- 用户再次明确：平台是否成型，关键不在“分析更全”，而在“投前、投中、投后是否闭环”。
- 用户再次明确：仓位管理和投后复盘不是增强项，而是闭环平台的核心层。
## 2026-04-08
### 修改内容
- `docs/AI_HANDOFF.md`：追加 foundation 元数据管理下一阶段候选方案，明确方案 A / B / C 的边界、推荐顺序与默认入口。目的：把后续推进方向固化到交接文档，避免再次误入业务化链路。 <!-- 2026-04-08 CST: 本次追加交接方案说明。原因：用户要求把下一步方案一并写入仓库并推送。 -->
- `docs/plans/2026-04-08-foundation-metadata-migration-contract-phase3-stage4.md`：补充 migration contract 落地后的候选路线、优缺点与推荐路线。目的：让阶段计划文档直接承接下一轮实现。 <!-- 2026-04-08 CST: 本次追加阶段计划说明。原因：需要把候选方案正式落盘。 -->
- `task_plan.md`、`progress.md`：同步记录“先 A、后 B、最后 C”的默认顺序。目的：统一计划与进度口径，减少交接歧义。 <!-- 2026-04-08 CST: 本次同步计划与进度。原因：避免文档间口径不一致。 -->
- 本次提交范围收口为 `foundation` 相关源码、测试、交接文档与为其通过编译所需的最小修复。目的：只推送当前通用标准能力主线，不混入其它业务化脏改动。 <!-- 2026-04-08 CST: 本次记录提交范围。原因：仓库当前工作区较脏，需要明确边界。 -->

### 修改原因
- 用户明确要求先把当前进展推送回 GitHub，并把“下一步方案”一起写进仓库文档。
- 当前 `Metadata Migration Contract` 已完成第一阶段，如果不把后续默认推进顺序写清，后续接手很容易直接跳到 repository 审计或执行器，偏离当前最小治理闭环。

### 方案还差什么?
- [ ] 还没有开始实现方案 A 的 `Validator` 联动，本次仅完成方案入库与推送整理。
- [ ] 还没有做 repository 级批量审计与 migration executor，这两部分仍需后续单独获批后推进。

### 潜在问题
- [ ] 当前仓库存在大量非本轮业务线改动和未跟踪文件，如果后续提交不继续按路径收口，仍可能把不相关改动混进 foundation 主线。
- [ ] `progress.md`、`AI_HANDOFF.md` 等文档在当前 PowerShell 输出下有编码显示噪声，但文件本身按 UTF-8 维护；后续若用错误编码保存，可能造成真正乱码。

### 关闭项
- 已通过 `cargo test --test metadata_schema_registry_unit --test metadata_schema_versioning_unit --test metadata_migration_contract_unit --test metadata_validator_unit --test knowledge_repository_unit --test knowledge_ingestion_unit --test knowledge_bundle_unit -- --nocapture`

### 记忆点（可选）
- 用户再次明确要求：推送前不仅要整理当前实现，还要把“下一步做什么”的方案正式写进仓库文档，不能只停留在对话里。
## 2026-04-08
### 修改内容
- `src/ops/signal_outcome_research.rs`：先新增 `signal_outcome_research_summary_exposes_extended_analog_metrics` 回归测试，再补齐 `SignalOutcomeResearchSummaryResult` 的扩展字段与最小汇总计算逻辑。目的：修复独立 foundation 分支从混合股票分支剥离后出现的编译断点。 <!-- 2026-04-08 CST: 本次先红测后修复。原因：独立分支暴露出摘要结构体字段缺失。 -->
- `docs/AI_HANDOFF.md`：补充分支边界说明，明确 `codex/merge-cli-mod-batches` 为混合股票分支，`codex/foundation-metadata-governance` 为 foundation 独立分支。目的：避免后续继续把 foundation 改动叠到股票分支。 <!-- 2026-04-08 CST: 本次补充分支交接说明。原因：用户要求把混合分支信息补进去。 -->

### 修改原因
- 用户要求把 `bf595e6` 从股票混合分支中拆出来，优先保证 foundation 代码没有冲突、能独立成立。
- 实际验证发现：`bf595e6` 单独切出后无法编译，根因是 `security_decision_briefing` 已消费扩展版 research summary 字段，但该分支内 `signal_outcome_research` 仍停留在旧结构。

### 方案还差什么?
- [ ] 还没有开始实现方案 A 的 `Metadata Validator` 联动，本次只完成 foundation 独立分支的最小编译修复与分支边界说明。
- [ ] 还需要把当前独立分支提交并推送到远端对应的新 foundation 分支。

### 潜在问题
- [ ] 当前修复只保证 research summary 摘要字段与 briefing 消费侧契约对齐，没有继续扩展赔率/仓位系统其它行为；后续若再扩大消费字段，需要继续按 TDD 增量补齐。
- [ ] `docs/AI_HANDOFF.md` 等文件在 PowerShell 输出里仍有编码显示噪声，但本次编辑按 UTF-8 维护；后续若用错误编码保存，仍可能造成真正乱码。

### 关闭项
- 已通过 `cargo test signal_outcome_research_summary_exposes_extended_analog_metrics --lib -- --nocapture`

### 记忆点（可选）
- 用户明确要求：先把 foundation 从股票混合分支里拆干净，再继续推进后续方案；当前重点是“代码没冲突、分支边界清楚”，不是顺手继续扩股票业务能力。
