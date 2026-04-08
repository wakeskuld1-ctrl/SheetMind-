// 2026-03-31 CST: 这里建立 foundation 模块边界，原因是底座能力与业务能力已经开始在 catalog / dispatcher 层串台。
// 2026-03-31 CST: 目的：把 Excel、统计诊断、建模、报表与容量评估明确归到通用底座区，后续新增通用能力默认先进入这里。
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

// 2026-04-07 CST: 这里新增 foundation 导航内核子模块入口，原因是当前项目已明确要在 foundation 内建设
// 2026-04-07 CST: “ontology-lite -> roaming -> retrieval -> evidence” 的业务无关底座，而不是继续把语义能力混进现有业务模块。
// 2026-04-07 CST: 目的：先用显式路径把新内核稳定挂到 `foundation.rs + src/ops/foundation/` 结构上，避免与既有入口冲突。
#[path = "foundation/ontology_schema.rs"]
pub mod ontology_schema;
// 2026-04-07 CST: 这里挂接 ontology store，原因是 schema 定义与查询存取职责需要分离。
// 2026-04-07 CST: 目的：让后续概念索引、别名索引和关系读取先有独立落点，不把查询逻辑塞回 schema 本身。
#[path = "foundation/ontology_store.rs"]
pub mod ontology_store;
// 2026-04-07 CST: 这里挂接 knowledge record，原因是知识节点、知识边、证据引用属于独立数据模型层。
// 2026-04-07 CST: 目的：让知识图谱数据模型先有稳定命名空间，后续不污染既有 Excel / 分析能力模块。
#[path = "foundation/knowledge_record.rs"]
pub mod knowledge_record;
// 2026-04-07 CST: 这里挂接 knowledge graph store，原因是图谱存储查询与 record 定义不是同一职责。
// 2026-04-07 CST: 目的：后续可以先做纯内存查询，再无痛替换为其他存储实现。
#[path = "foundation/knowledge_graph_store.rs"]
pub mod knowledge_graph_store;
// 2026-04-07 CST: 这里挂接 capability router，原因是“问题 -> 种子概念/能力”需要独立收口。
// 2026-04-07 CST: 目的：从一开始就把导航入口固定成独立模块，不让 retrieval 反客为主。
#[path = "foundation/capability_router.rs"]
pub mod capability_router;
// 2026-04-07 CST: 这里挂接 roaming engine，原因是知识漫游是当前 foundation 主链的中间核心。
// 2026-04-07 CST: 目的：后续深度限制、关系限制和候选域收敛都在这里演进，而不是分散到 store 或 retrieval。
#[path = "foundation/roaming_engine.rs"]
pub mod roaming_engine;
// 2026-04-07 CST: 这里挂接 retrieval engine，原因是检索在当前设计里只是候选域内执行器。
// 2026-04-07 CST: 目的：先从模块边界上强调 retrieval 不是系统入口，后续实现也更不容易跑偏。
#[path = "foundation/retrieval_engine.rs"]
pub mod retrieval_engine;
// 2026-04-07 CST: 这里挂接 evidence assembler，原因是最终证据装配需要和检索、漫游解耦。
// 2026-04-07 CST: 目的：为后续统一输出导航路径、命中结果和证据引用提供单独落点。
#[path = "foundation/evidence_assembler.rs"]
pub mod evidence_assembler;
// 2026-04-08 CST: 这里挂接 navigation pipeline，原因是 Task 8 与 Task 9 之间现在已有正式断层：
// 2026-04-08 CST: assembler 已开始承担统一输出职责，但还缺一个 foundation 内部入口把 route / roam / retrieve 串起来。
// 2026-04-08 CST: 目的：把最小闭环正式落到 foundation 命名空间，避免再次回退成测试里手工编排模块。
#[path = "foundation/navigation_pipeline.rs"]
pub mod navigation_pipeline;
