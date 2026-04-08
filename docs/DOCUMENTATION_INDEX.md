# 文档索引

<!-- 2026-04-02 CST: 新增文档索引。原因：docs 目录已包含计划、验收、历史交接和执行记录，入口分散。目的：为开发者与 AI 提供一份可维护的导航清单。 -->

## 1. 必看入口

- [README.md](/E:/TradingAgents/TradingAgents/README.md)
  - 项目级入口说明。
- [AI_HANDOFF.md](/E:/TradingAgents/TradingAgents/docs/AI_HANDOFF.md)
  - 后续 AI / 开发者接手手册。
- [DOC_UPDATE_POLICY.md](/E:/TradingAgents/TradingAgents/docs/DOC_UPDATE_POLICY.md)
  - 文档更新规则。
- [CHANGELOG_TASK.MD](/E:/TradingAgents/TradingAgents/CHANGELOG_TASK.MD)
  - 任务日志与记忆点沉淀。

## 2. docs 目录说明

- `docs/acceptance/`
  - 分阶段验收文档。
- `docs/plans/`
  - 计划类文档与路线拆解。
- [plans/2026-04-08-closed-loop-investment-research-roadmap.md](/E:/TradingAgents/TradingAgents/docs/plans/2026-04-08-closed-loop-investment-research-roadmap.md)
  - 证券分析主链从“投前分析”升级到“赔率系统 + 仓位管理 + 投后复盘 + 市场结构 + 深层信息面”的正式路线图。
- [plans/2026-04-08-odds-position-system-design.md](/E:/TradingAgents/TradingAgents/docs/plans/2026-04-08-odds-position-system-design.md)
  - 基于既有 `signal_outcome_research` 的赔率系统与仓位管理设计，明确研究层复用边界、briefing 顶层新字段和 committee 同源摘要方案。
- [plans/2026-04-08-foundation-navigation-kernel-phase1.md](/E:/TradingAgents/TradingAgents/docs/plans/2026-04-08-foundation-navigation-kernel-phase1.md)
  - foundation 最小导航内核阶段计划。
- [plans/2026-04-08-foundation-standard-capabilities-phase2.md](/E:/TradingAgents/TradingAgents/docs/plans/2026-04-08-foundation-standard-capabilities-phase2.md)
  - foundation Phase 2 第一阶段通用标准能力计划与交接文档。
- [plans/2026-04-08-foundation-knowledge-ingestion-phase2-stage2.md](/E:/TradingAgents/TradingAgents/docs/plans/2026-04-08-foundation-knowledge-ingestion-phase2-stage2.md)
  - foundation Phase 2 第二阶段 knowledge_ingestion 计划与交接文档。
- [plans/2026-04-08-foundation-metadata-filter-phase2-stage3.md](/E:/TradingAgents/TradingAgents/docs/plans/2026-04-08-foundation-metadata-filter-phase2-stage3.md)
  - foundation Phase 2 第三阶段 metadata filter 扩展计划与交接文档。
- [plans/2026-04-08-foundation-repository-layout-phase2-stage4.md](/E:/TradingAgents/TradingAgents/docs/plans/2026-04-08-foundation-repository-layout-phase2-stage4.md)
  - foundation Phase 2 第四阶段 repository layout 标准化计划与交接文档。
- [plans/2026-04-08-foundation-metadata-schema-registry-phase3-stage1.md](/E:/TradingAgents/TradingAgents/docs/plans/2026-04-08-foundation-metadata-schema-registry-phase3-stage1.md)
  - foundation Phase 3 第一阶段 metadata schema registry 计划与交接文档。
- [plans/2026-04-08-foundation-metadata-validator-phase3-stage2.md](/E:/TradingAgents/TradingAgents/docs/plans/2026-04-08-foundation-metadata-validator-phase3-stage2.md)
  - foundation Phase 3 第二阶段 metadata validator 计划与交接文档。
- [plans/2026-04-08-foundation-metadata-schema-versioning-phase3-stage3.md](/E:/TradingAgents/TradingAgents/docs/plans/2026-04-08-foundation-metadata-schema-versioning-phase3-stage3.md)
  - foundation Phase 3 第三阶段 metadata schema versioning 计划与交接文档。
- [plans/2026-04-08-foundation-metadata-migration-contract-phase3-stage4.md](/E:/TradingAgents/TradingAgents/docs/plans/2026-04-08-foundation-metadata-migration-contract-phase3-stage4.md)
  - foundation Phase 3 第四阶段 metadata migration contract 计划与交接文档。
- `docs/architecture/`
  - 架构类说明。
- `docs/marketing/`
  - 市场或对外表达材料。
- `docs/execution-notes-2026-03-30.md`
  - 历史执行记录，可作为背景补充，不是当前权威入口。

## 3. 历史交接文档处理方式

- `docs/交接摘要_给后续AI.md`
  - 现在保留为跳转页，不再作为主维护文档。
- `docs/交接摘要_证券分析_给后续AI.md`
  - 现在保留为跳转页，不再单独扩展。

## 4. 维护原则

- 新功能如果改变正式主链、流程门禁、Tool 合同或推荐入口，先更新权威入口文档，再考虑是否补历史说明。
- 旧文档可保留，但必须明确它是不是“当前权威入口”。
- 如果用户要求“随时更新”，默认指的是：
  - `README.md`
  - `docs/AI_HANDOFF.md`
  - `CHANGELOG_TASK.MD`
