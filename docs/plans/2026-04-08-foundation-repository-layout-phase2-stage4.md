# Foundation Repository Layout Phase 2 Stage 4

> **For Claude/Codex:** 本文档只描述 foundation 的 repository 布局标准化，不包含目录型大仓储拆分、不包含业务源数据入库。

<!-- 2026-04-08 CST: 新增 foundation repository layout 阶段文档。原因：当前 repository 已从单文件保存扩展到标准布局目录，需要单独记录布局契约、manifest 和替换式写入边界。目的：为后续继续扩展持久化标准提供统一交接入口。 -->

## 1. 目标

- 在 foundation 中补齐 repository 的最小标准布局。
- 让 repository 除单文件保存外，还支持标准布局目录保存与读回。
- 让写入路径统一切到“同目录 staging 写入 -> 再替换正式文件”的最小安全路径。

## 2. 本阶段范围

- `KnowledgeRepository::save_to_path(&Path)`
  - 改为 staging 写入后替换正式文件。
- `KnowledgeRepository::save_to_layout_dir(&Path)`
  - 输出标准布局目录。
- `KnowledgeRepository::load_from_layout_dir(&Path)`
  - 从标准布局目录重建 repository。
- 标准布局文件：
  - `bundle.json`
  - `repository.manifest.json`

## 3. Manifest 契约

- `layout_version`
- `bundle_file`
- `schema_version`
- `concept_count`
- `relation_count`
- `node_count`
- `edge_count`

## 4. 非目标

- 不做目录型拆分仓储，例如 `nodes.jsonl / edges.jsonl / concepts.json`。
- 不做索引文件。
- 不做版本迁移器。
- 不做业务数据直接入库。

## 5. 已完成测试

- `tests/knowledge_repository_unit.rs`
  - `knowledge_repository_persists_standard_layout_directory`
  - `knowledge_repository_loads_repository_from_standard_layout_directory`
- 已通过回归：
  - `cargo test --test knowledge_repository_unit -- --nocapture`
  - `cargo test --test knowledge_repository_unit --test knowledge_ingestion_unit --test knowledge_bundle_unit -- --nocapture`

## 6. 当前边界

- 现在已经支持标准布局目录。
- 现在 manifest 只记录最小版本和计数信息。
- 现在还没有更完整的目录拆分和索引规范。

## 7. 下一阶段建议

1. 更强校验
   - 增加 manifest 版本校验、布局缺文件校验、edge 引用校验。
2. 更完整布局
   - 如再次获批，再考虑把 bundle 拆成多文件目录布局。
3. 原始入库流水线
   - 继续保持在标准能力范围内，考虑把原始标准文件映射进当前布局。
