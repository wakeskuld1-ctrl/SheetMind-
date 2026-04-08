# Foundation Knowledge Ingestion Phase 2 Stage 2

> **For Claude/Codex:** 本文档只描述 foundation 的标准导入能力，不包含业务域字段映射、不包含目录扫描、不包含证券分析主链接入。

<!-- 2026-04-08 CST: 新增 foundation knowledge_ingestion 阶段文档。原因：当前已经从“可持久化标准包”推进到“可导入标准包/标准记录”，需要把输入契约和边界单独沉淀。目的：为后续扩展过滤和持久化布局提供统一交接入口。 -->

## 1. 目标

- 在 foundation 中补齐标准导入能力 `knowledge_ingestion`。
- 让外部文件可以通过两种标准路径进入 foundation：
  - `KnowledgeBundle` JSON
  - tagged-record JSONL
- 保持导入能力仍为通用标准能力，不引入业务域映射逻辑。

## 2. 本阶段范围

- `load_bundle_from_json_path(&Path)`
- `load_repository_from_json_path(&Path)`
- `load_bundle_from_jsonl_path(&Path)`
- `load_repository_from_jsonl_path(&Path)`
- `KnowledgeIngestionError`
  - 文件读取失败
  - JSON 反序列化失败
  - JSONL 行级反序列化失败
  - 缺少/重复 `bundle_header`
  - repository 构建失败

## 3. 标准记录 JSONL 格式

- `bundle_header`
  - 必填字段：`schema_version`
- `concept`
  - 字段：`id / name / aliases`
- `relation`
  - 字段：`from_concept_id / to_concept_id / relation_type`
- `node`
  - 字段：`id / title / body / concept_ids / metadata / evidence_refs`
- `edge`
  - 字段：`from_node_id / to_node_id / relation_type`

## 4. 非目标

- 不做目录型导入。
- 不做业务 CSV/Excel/证券数据直接转知识包。
- 不做增量合并、冲突解决或版本迁移。
- 不做高级校验 DSL。

## 5. 已完成测试

- `tests/knowledge_ingestion_unit.rs`
  - `knowledge_ingestion_loads_bundle_from_json_file`
  - `knowledge_ingestion_loads_repository_from_jsonl_records`
  - `knowledge_ingestion_reports_jsonl_line_number_for_invalid_record`
- 已通过回归：
  - `cargo test --test knowledge_ingestion_unit -- --nocapture`
  - `cargo test --test knowledge_bundle_unit --test knowledge_repository_unit --test knowledge_ingestion_unit -- --nocapture`

## 6. 当前边界

- 现在已经支持“标准包文件导入”和“标准记录文件导入”。
- 现在还不支持“原始业务文件直接入库”。
- 现在也还没有文件目录布局规范。

## 7. 下一阶段建议

1. 扩展 `MetadataFilter`
   - 支持多字段 AND 与可选 concept scope。
2. 规范持久化布局
   - 定义 bundle/repository 文件命名、目录结构与版本规则。
3. 原子写与更强校验
   - 为仓储写入补原子落盘，为 JSONL header/重复记录补更细边界校验。
