// 2026-03-31 CST: 这里建立 foundation 模块边界，原因是底座能力与股票业务能力已经开始在 catalog / dispatcher 层串台。
// 目的：把 Excel、统计诊断、建模、报表与容量评估明确归到通用底座域，后续新增通用能力默认先进入这里。
#[path = "analyze.rs"]
pub mod analyze;
#[path = "append.rs"]
pub mod append;
#[path = "capacity_assessment.rs"]
pub mod capacity_assessment;
#[path = "capacity_assessment_excel_report.rs"]
pub mod capacity_assessment_excel_report;
#[path = "capacity_assessment_from_inventory.rs"]
pub mod capacity_assessment_from_inventory;
#[path = "cast.rs"]
pub mod cast;
#[path = "chart_svg.rs"]
pub mod chart_svg;
#[path = "cluster_kmeans.rs"]
pub mod cluster_kmeans;
#[path = "correlation_analysis.rs"]
pub mod correlation_analysis;
#[path = "decision_assistant.rs"]
pub mod decision_assistant;
#[path = "deduplicate_by_key.rs"]
pub mod deduplicate_by_key;
#[path = "derive.rs"]
pub mod derive;
#[path = "diagnostics_report.rs"]
pub mod diagnostics_report;
#[path = "diagnostics_report_excel_report.rs"]
pub mod diagnostics_report_excel_report;
#[path = "distinct_rows.rs"]
pub mod distinct_rows;
#[path = "distribution_analysis.rs"]
pub mod distribution_analysis;
#[path = "excel_chart_writer.rs"]
pub mod excel_chart_writer;
#[path = "export.rs"]
pub mod export;
#[path = "fill_lookup.rs"]
pub mod fill_lookup;
#[path = "fill_missing_values.rs"]
pub mod fill_missing_values;
#[path = "filter.rs"]
pub mod filter;
#[path = "format_table_for_export.rs"]
pub mod format_table_for_export;
#[path = "group.rs"]
pub mod group;
#[path = "join.rs"]
pub mod join;
#[path = "linear_regression.rs"]
pub mod linear_regression;
#[path = "logistic_regression.rs"]
pub mod logistic_regression;
#[path = "lookup_values.rs"]
pub mod lookup_values;
#[path = "model_output.rs"]
pub mod model_output;
#[path = "model_prep.rs"]
pub mod model_prep;
#[path = "multi_table_plan.rs"]
pub mod multi_table_plan;
#[path = "normalize_text.rs"]
pub mod normalize_text;
#[path = "outlier_detection.rs"]
pub mod outlier_detection;
#[path = "parse_datetime.rs"]
pub mod parse_datetime;
#[path = "pivot.rs"]
pub mod pivot;
#[path = "preview.rs"]
pub mod preview;
#[path = "rename.rs"]
pub mod rename;
#[path = "report_delivery.rs"]
pub mod report_delivery;
#[path = "select.rs"]
pub mod select;
#[path = "semantic.rs"]
pub mod semantic;
#[path = "sort.rs"]
pub mod sort;
#[path = "ssh_inventory.rs"]
pub mod ssh_inventory;
#[path = "stat_summary.rs"]
pub mod stat_summary;
#[path = "summary.rs"]
pub mod summary;
#[path = "table_links.rs"]
pub mod table_links;
#[path = "table_workflow.rs"]
pub mod table_workflow;
#[path = "top_n.rs"]
pub mod top_n;
#[path = "trend_analysis.rs"]
pub mod trend_analysis;
#[path = "window.rs"]
pub mod window;

// 2026-04-07 CST: 这里新增 foundation 导航内核子模块入口，原因是当前项目已经明确要在 foundation 内建立
// “ontology-lite -> roaming -> retrieval -> evidence” 的业务无关底座，而不是继续把语义能力混进现有业务模块。
// 目的：先用显式路径把新内核稳定挂到 `foundation.rs + src/ops/foundation/` 结构上，避免与现有 `foundation.rs`
// 单文件模块入口冲突，同时为后续 TDD 分阶段落地留出清晰边界。
#[path = "foundation/ontology_schema.rs"]
pub mod ontology_schema;
// 2026-04-07 CST: 这里挂接 ontology store，原因是 schema 定义和查询存取职责需要分离。
// 目的：让后续概念索引、别名索引和关系读取先有独立落点，不把查询逻辑塞回 schema 本身。
#[path = "foundation/ontology_store.rs"]
pub mod ontology_store;
// 2026-04-07 CST: 这里挂接 knowledge record，原因是知识节点、知识边、证据引用属于独立数据模型层。
// 目的：让知识图谱数据模型先有稳定命名空间，后续不会污染现有 Excel/分析能力模块。
#[path = "foundation/knowledge_record.rs"]
pub mod knowledge_record;
// 2026-04-09 CST: 这里挂接 MetadataConstraint 标准模型，原因是方案B第一阶段已经确认 metadata 要作为 foundation 通用能力正式入链，
// 但当前范围只收敛到 NavigationRequest -> Retrieval，不扩展到 roaming 语义。
// 目的：让 metadata 约束以一等模块对外暴露，避免调用方继续走临时字段或业务化私有接口。
#[path = "foundation/metadata_constraint.rs"]
pub mod metadata_constraint;
// 2026-04-09 CST: 这里挂接 metadata scope resolver，原因是方案B已经进入 concept-level metadata 收敛阶段，
// 需要一个独立于 pipeline / roaming 的通用模块来承接 concept ids 过滤语义。
// 目的：把 metadata-aware concept 收敛固化为 foundation 标准能力模块，而不是临时散落实现。
#[path = "foundation/metadata_scope_resolver.rs"]
pub mod metadata_scope_resolver;
// 2026-04-09 CST: 这里挂接 metadata registry，原因是方案B第一阶段要把字段名、适用目标与支持操作符从隐式推断
// 升级成显式注册的标准能力。
// 目的：为后续通用元数据管理、本体元数据治理与字段审计提供统一目录模块。
#[path = "foundation/metadata_registry.rs"]
pub mod metadata_registry;
// 2026-04-07 CST: 这里挂接 knowledge graph store，原因是图谱存储查询与 record 定义不是同一职责。
// 目的：后续可以先做纯内存查询，再无痛替换为其他存储实现。
#[path = "foundation/knowledge_graph_store.rs"]
pub mod knowledge_graph_store;
// 2026-04-07 CST: 这里挂接 capability router，原因是“问题 -> 种子概念/能力”需要单独收口。
// 目的：把导航入口从一开始就固定为独立模块，不让 retrieval 反客为主。
#[path = "foundation/capability_router.rs"]
pub mod capability_router;
// 2026-04-07 CST: 这里挂接 roaming engine，原因是知识漫游是当前 foundation 主链的中间核心。
// 目的：后续深度限制、关系限制和候选域收敛都在这里演进，而不是分散到 store 或 retrieval。
#[path = "foundation/roaming_engine.rs"]
pub mod roaming_engine;
// 2026-04-07 CST: 这里挂接 retrieval engine，原因是检索在当前设计里只是候选域内执行器。
// 目的：先从模块边界上强调 retrieval 不是系统入口，后续实现也更不容易跑偏。
#[path = "foundation/retrieval_engine.rs"]
pub mod retrieval_engine;
// 2026-04-07 CST: 这里挂接 evidence assembler，原因是最终证据装配需要和检索、漫游解耦。
// 目的：为后续统一输出导航路径、命中结果和证据引用提供单独落点。
#[path = "foundation/evidence_assembler.rs"]
pub mod evidence_assembler;
// 2026-04-09 CST: 这里挂接 navigation pipeline，原因是 foundation 已经具备 route、roam、retrieve、assemble
// 四段能力，但还缺少正式的统一入口模块。
// 目的：把“问题到证据”的主线编排明确收敛在 foundation 内核中，而不是散落在测试或未来业务层里。
#[path = "foundation/navigation_pipeline.rs"]
pub mod navigation_pipeline;
