# 发现记录

## 2026-04-08 Metadata Validator 补充发现

- `MetadataValidator` 当前是严格的节点级校验器，而不是 repository 级批量审计器；这条边界已经被测试和阶段文档共同钉住。
- 多 concept 节点当前采用“字段兼容性逐 concept 校验、required 按并集处理”的语义，这能覆盖最小治理闭环，但后续如果出现 concept inheritance，语义需要重新显式设计。
- `KnowledgeNode.metadata` 仍然是 `BTreeMap<String, String>`，所以当前类型校验本质上是字符串解析校验，而不是强类型存储；这适合当前阶段，但 schema versioning 之后可能需要更强演进策略。
- 当前 validator 已经把 schema registry 从“定义层”推进到了“执行层”，但还没有形成 repository 批量扫描、错误聚合统计和自动修复能力。

## 2026-04-08 Schema Versioning 第一阶段补充发现

- `MetadataSchema` 当前已经不再是“无版本 registry”，而是具备正式 `schema_version` 的治理对象。
- 第一阶段兼容性语义被明确收口为“精确版本匹配”，这保证了契约清晰，但也意味着当前还不具备任何跨版本自动兼容能力。
- 由于当前只做 version contract，不做 migration，所以 `schema_version` 现在主要承担“声明”和“拒绝不合法版本”的职责，而不是“自动演进”职责。
- 后续一旦进入 `deprecated / replaced_by / alias`，就需要重新审视字段级与 schema 级 versioning 的职责边界。

## 2026-04-08 Migration Contract 第一阶段补充发现

- `MetadataFieldDefinition` 当前已经从“静态字段定义”推进到“带演进语义的字段治理对象”，这意味着 metadata schema 开始具备正式的字段生命周期表达力。
- 当前 alias 只停留在 contract 层注册和冲突校验，还没有进入任何解析路径，所以它现在是“治理信息”，不是“执行行为”。
- 当前 `replaced_by` 只是声明式关系，不会触发任何自动字段重写；后续如果要进入 executor，必须先明确冲突策略和覆盖优先级。
- 本轮为解除测试阻塞，顺手修复了 `security_decision_briefing.rs` 的一个浮点字面量推断问题；这不是本轮主线能力，但属于继续执行 TDD 所必需的编译修复。

## 当前代码事实

- `src/ops/security_committee_vote.rs` 仍是 5 席投票实现，且所有席位都在同一进程内构造。
- `tests/security_committee_vote_cli.rs` 已新增红测，要求 `committee_engine == "seven_seat_committee_v3"` 且每席 `execution_mode == "child_process"`。
- `src/tools/contracts.rs` 当前 `ToolRequest` 只有 `Deserialize`，`ToolResponse` 只有 `Serialize`，不够支撑内部子进程通过 JSON 往返。
- `src/tools/dispatcher.rs` 与 `src/tools/dispatcher/stock_ops.rs` 是 CLI 正式入口分发链，适合挂内部 seat agent。
- `security_decision_briefing` 会直接调用 `security_committee_vote`，所以 vote 结果合同升级时要兼容 briefing 使用场景。

## 实现判断

- 正式 CLI 路径可以通过子进程调用当前二进制来证明“席位独立执行”。
- 直接函数测试路径若无法解析到 CLI 可执行文件，需要回退到进程内 seat agent，否则测试 harness 下会失效。
- 七席设计应落到现有 `security_committee_vote` 合同中，而不是再造新的 committee tool。

## 2026-04-08 补充发现

- 直接函数测试的 `current_exe()` 指向的是测试 harness，而不是 `excel_skill.exe`，所以需要从邻近 `target/debug` 路径回推正式二进制。
- `briefing` 内嵌 vote 与“重新调用一次 formal vote”的稳定业务语义应一致，但 `process_id / execution_instance_id` 属于每次独立执行的动态证据，不能再做整对象全等。
- 当前“独立证明”最直接的正式证据链是：
  - `committee_engine == "seven_seat_committee_v3"`
  - `votes.len() == 7`
  - 每席 `execution_mode == "child_process"`
  - 7 个 `process_id` 唯一
  - 7 个 `execution_instance_id` 唯一

## 2026-04-08 Foundation 补充发现

- `src/ops/foundation/` 当前已经具备最小知识导航闭环，但它仍是独立 foundation 内核，不应误判为“完整知识库”或“已接入证券分析主链”。
- 当前闭环顺序已经稳定为：`ontology_schema -> ontology_store -> capability_router -> roaming_engine -> retrieval_engine -> evidence_assembler -> navigation_pipeline`。
- `capability_router` 当前采用“短语优先、token 回退”的最小规则，适合 phase 1，但尚未支持更复杂的语义归一化与 metadata 过滤。
- `roaming_engine` 当前是受限 BFS，只支持 relation-type 白名单、`max_depth` 与 `max_concepts` 三类预算控制。
- `retrieval_engine` 当前只做候选域内关键词交集评分，尚无向量检索、重排序与持久化索引。
- `evidence_assembler` 当前能稳定装配 route、path、hits、citations、summary，但 summary 仍是零依赖模板文本，不是更强的摘要系统。
- Windows 环境下 foundation 相关 `cargo test` 偶发会被残留 `excel_skill.exe` 或 `cargo` 进程锁住，触发 `os error 5`；跑测试前先清残留进程是必要操作。

## 2026-04-08 Foundation Phase 2 第一阶段补充发现

- `KnowledgeBundle` 已把 ontology 与 graph 原始数据统一收口成标准知识包，foundation 后续若要支持导入导出，应优先围绕 bundle 扩展，而不是直接序列化内存 store。
- `KnowledgeRepository` 当前已经提供最小的构建校验、JSON 落盘与读回能力，但它还只是“文件级标准仓储”，不是“完整入库系统”。
- `MetadataFilter` 当前只支持 exact-match，且过滤目标固定为 `KnowledgeNode.metadata`；这足以支撑通用标准能力阶段，但还不适合复杂业务检索。
- `KnowledgeNode.metadata` 现在是 foundation 标准节点模型的一部分，这意味着后续业务域若要接入 foundation，应该先把域属性映射到 metadata，而不是另起一套节点外过滤结构。
- 当前 foundation 的真实边界应表述为“最小导航闭环 + 标准包/仓储 + metadata 精确过滤”，不能再写成“已经有完整知识库内容”。

## 2026-04-08 Foundation Knowledge Ingestion 补充发现

- `knowledge_ingestion` 现在已经提供两条标准导入路径：完整 `KnowledgeBundle` JSON 与单文件 tagged-record JSONL；这意味着 foundation 已经具备“标准文件 -> 标准包/仓储”的最小入口。
- JSONL 路径当前依赖 `bundle_header` 提供 `schema_version`，并把 `concept / relation / node / edge` 全部收在同一个文件里；这比目录型导入更轻，但也意味着后续如果要做大规模入库，可能还需要进一步拆分布局。
- 当前 JSONL 错误边界已经能定位到具体 `line_number`，这对手工维护知识文件很重要；后续扩展校验时应继续保留这类“可诊断性优先”的设计。
- `knowledge_ingestion` 最终仍复用 `KnowledgeRepository::new()` 做一致性校验，而不是自己复制 duplicate node id 等规则；这条边界应该继续保持，避免导入层和仓储层规则分叉。

## 2026-04-08 Foundation Metadata Filter 扩展发现

- `MetadataFilter` 现在已经从“单字段 exact-match”扩展为“多字段 AND + 可选 concept scope”，但仍然保持在通用标准层，没有引入 DSL。
- 当前 concept scope 的语义是“节点 concept_ids 与过滤器 scope 有任一交集即可”，这适合当前阶段；如果后续要支持更复杂语义，应明确是 AND 还是 OR，而不是隐式扩展。
- 当前组合过滤顺序本质上是“exact-match AND concept-scope”，这已经足以支撑 foundation 通用能力；后续不应在没有批准的情况下顺手扩成 OR/NOT 或模糊匹配。

## 2026-04-08 Foundation Repository Layout 补充发现

- `KnowledgeRepository` 当前已经支持两种保存方式：单文件 `save_to_path()` 与标准布局目录 `save_to_layout_dir()`；目录布局最小契约固定为 `bundle.json + repository.manifest.json`。
- 现阶段 manifest 只承载 `layout_version / bundle_file / schema_version / counts`，这足够做最小布局标准，但还不足以支撑更复杂版本迁移或索引发现。
- 当前写入路径已经统一走“同目录 staging 写入 -> 再替换正式文件”的最小安全策略，这比直接覆盖写更稳，但还没有做到更完整的跨平台强原子保证。

## 2026-04-08 Metadata Schema Registry 补充发现

- `MetadataSchema` 现在已经把 metadata 从“节点上的 `BTreeMap<String, String>`”提升到“字段定义注册表 + concept 绑定策略”两个正式对象。
- `ConceptMetadataPolicy` 当前的 `allows_field()` 语义是“allowed 或 required 任一命中即可”，这对当前阶段是合理的，因为 required 本身就是 allowed 的更严格子集。
- 当前 registry 已经能挡住“policy 引用未知字段”这类最基本错误，但还没有把这些规则真正应用到 `KnowledgeNode.metadata` 校验上，因此它现在仍是“治理定义层”，不是“执行校验层”。
## 2026-04-10 Validator Linkage 补充发现

<!-- 2026-04-10 CST：追加 validator linkage 阶段发现。原因：foundation metadata 治理已经从 contract 层推进到 validator 执行层。目的：把本轮沉淀下来的语义边界写清楚，避免后续 AI 误把 compatibility 语义、repository audit 与 executor 混为一谈。 -->

- `MetadataValidator` 现在已经不再只是消费 `required / disallowed / allowed_values / value_type` 的静态 registry，而是正式进入“字段演进信号也会影响节点级校验结果”的阶段。
- alias 的当前语义已经固定为“提示式兼容”而不是“自动重写”：
  - 会输出 `AliasFieldUsed`
  - 但不会因为使用 alias 就额外制造 required / disallowed 误报
- deprecated 的当前语义已经固定为“提示 + 替代建议”而不是“自动迁移”：
  - 会输出 `DeprecatedFieldUsed`
  - 如果存在 `replaced_by`，则返回推荐替代字段
- 当前 required 兼容语义已经被明确扩展到：
  - alias 命中 required canonical 字段
  - deprecated 字段通过 `replaced_by` 命中 required 字段
- 这意味着 foundation 元数据治理当前已经从“定义层 + contract 层”推进到“定义层 + contract 层 + 节点级执行层”，但仍然没有进入：
  - repository 批量审计
  - 自动 rewrite
  - migration executor

## 2026-04-10 Repository-Level Audit 补充发现

<!-- 2026-04-10 CST：追加 repository-level audit 阶段发现。原因：foundation 已经从节点 validator 推进到仓库级审计。目的：写清楚当前审计层的真正职责、hygiene diagnostics 的边界，以及什么还没有做。 -->

- `RepositoryMetadataAudit` 当前已经把 foundation metadata 治理从“节点级执行层”推进到“仓库级聚合观察层”，但它仍然是只读审计器，不是修复器。
- 当前 `issue_type_counts` 的聚合键直接复用 `MetadataValidationIssue` 变体名，这让报告实现更轻，但也意味着后续如果要做对外稳定协议，可能还需要单独的 audit issue kind 层。
- 当前 `concept_issue_counts` 只统计真正带 `concept_id` 的问题，这个口径是刻意收窄的：
  - 不给 `AliasFieldUsed`
  - 不给 `DeprecatedFieldUsed`
  - 不给 `InvalidAllowedValue / InvalidValueType`
  强行虚构 concept 归属
- 当前 evidence hygiene diagnostics 只做到最小显式规则：
  - 同一 `source_ref + locator` 跨节点重复
  - 空白 `locator`
  - 空白 `source_ref`
- 这意味着 foundation 当前已经具备：
  - registry
  - validator
  - repository audit
  但仍然没有进入：
  - 更复杂的 evidence quality scoring
  - 自动 rewrite
  - migration executor

## 2026-04-10 Findings: Foundation Evidence Hygiene Diagnostics Expansion
- 本轮只扩展 foundation repository audit 的只读诊断层，没有触碰 migration executor、自动 rewrite 或证券业务链。
- `WeakLocator` 当前口径：`Blank` 或 `TooShort(<3)`。
- `WeakSourceRef` 当前口径：`Blank`、`TooShort(<4)`、`MissingNamespace(不含冒号)`。
- 尚未覆盖的后续能力主要有 3 个方向：更细 locator 结构规则、更细 source_ref 规范规则、审计报告分级/摘要。
