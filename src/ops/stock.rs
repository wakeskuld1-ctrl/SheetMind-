// 2026-03-31 CST: 这里建立 stock 模块边界，原因是股票导入、同步和技术面咨询已经不再属于通用分析底座。
// 目的：把股票业务域单独收口，后续新增指标、行情同步和技术咨询一律从这里扩展，不再反向挂回 foundation。
#[path = "eastmoney_enrichment.rs"]
pub mod eastmoney_enrichment;
#[path = "import_stock_price_history.rs"]
pub mod import_stock_price_history;
#[path = "security_analysis_contextual.rs"]
pub mod security_analysis_contextual;
#[path = "security_analysis_fullstack.rs"]
pub mod security_analysis_fullstack;
#[path = "security_approval_brief_signature.rs"]
pub mod security_approval_brief_signature;
#[path = "security_decision_approval_bridge.rs"]
pub mod security_decision_approval_bridge;
#[path = "security_decision_approval_brief.rs"]
pub mod security_decision_approval_brief;
#[path = "security_decision_card.rs"]
pub mod security_decision_card;
// 2026-04-02 CST: 这里挂入正式证券审批包模块，原因是审批工件已经分别落盘，需要新增统一 package 锚点；
// 目的：让股票业务域内的审批包构造能力和 approval brief、position plan 保持同级归属。
#[path = "security_decision_package.rs"]
pub mod security_decision_package;
#[path = "security_scorecard.rs"]
pub mod security_scorecard;
// 2026-04-11 CST: 这里挂入正式 master_scorecard 模块，原因是方案 C 已确认要把“未来几日赚钱效益总卡”
// 作为 stock 主链的一等对象，而不是只保留在设计文档里。
// 目的：让 CLI / Skill / 后续 package 能沿统一 stock 边界消费总卡能力。
#[path = "security_master_scorecard.rs"]
pub mod security_master_scorecard;
// 2026-04-09 CST: 这里挂入主席正式裁决模块，原因是 Task 1 要把“最终正式动作”从投委会线中拆出，
// 目的：让 stock 域内拥有独立的主席裁决入口，后续 package / verify / audit 都能沿这条线继续扩展。
#[path = "security_chair_resolution.rs"]
pub mod security_chair_resolution;
// 2026-04-09 CST: 这里挂入正式特征快照模块，原因是 Task 2 要把 feature_snapshot 底座提升为独立正式对象；
// 目的：让后续 forward_outcome / training / refit 都能沿统一快照入口演进。
#[path = "security_feature_snapshot.rs"]
pub mod security_feature_snapshot;
// 2026-04-12 CST: Mount the formal condition-review module, because P8 begins by
// turning intraperiod review into a first-class stock object.
// Purpose: expose a stable lifecycle review boundary for later execution and post-trade chaining.
#[path = "security_condition_review.rs"]
pub mod security_condition_review;
// 2026-04-12 CST: Mount the formal execution-record module, because P8 needs
// a first-class execution event object after condition review enters the lifecycle.
// Purpose: expose a stable stock boundary for build/add/reduce/exit style execution facts.
#[path = "security_execution_record.rs"]
pub mod security_execution_record;
// 2026-04-12 CST: Mount the formal post-trade review module, because P8 needs
// a first-class replayable review object after execution events.
// Purpose: expose a stable stock boundary for layered post-trade attribution.
#[path = "security_post_trade_review.rs"]
pub mod security_post_trade_review;
// 2026-04-09 CST: 这里挂入正式未来标签回填模块，原因是 Task 3 要把 forward_outcome 纳入证券主链的正式对象边界；
// 目的：让后续训练、回算与复盘都沿 stock 域统一扩展，而不是在外层重复拼装未来标签逻辑。
#[path = "security_forward_outcome.rs"]
pub mod security_forward_outcome;
// 2026-04-11 CST: Mount the governed historical external-proxy backfill tool,
// because P4 needs dated proxy records to become a first-class stock-domain
// capability before path-event heads can consume stable replay inputs.
// Purpose: expose one formal stock entry point for dated proxy writes and reads.
#[path = "security_external_proxy_backfill.rs"]
pub mod security_external_proxy_backfill;
// 2026-04-12 CST: Mount the file-based proxy-history import module, because
// Historical Data Phase 1 now needs one formal real-batch bridge into governed ETF
// proxy history before stronger live crawlers are hardened.
// Purpose: expose a stable stock boundary for file-to-governed proxy ingestion.
#[path = "security_external_proxy_history_import.rs"]
pub mod security_external_proxy_history_import;
// 2026-04-12 CST: Mount the governed stock fundamental-history backfill module,
// because Historical Data Phase 1 needs replayable financial snapshots to become
// a first-class stock capability instead of remaining live-fetch only.
// Purpose: expose one formal stock entry point for governed fundamental history writes.
#[path = "security_fundamental_history_backfill.rs"]
pub mod security_fundamental_history_backfill;
// 2026-04-12 CST: Mount the live stock fundamental-history backfill module, because
// Historical Data Phase 1 now needs one formal provider-to-governed bridge for
// multi-period financial history instead of shell-side record assembly.
// Purpose: expose a stable stock boundary for live financial-history ingestion.
#[path = "security_fundamental_history_live_backfill.rs"]
pub mod security_fundamental_history_live_backfill;
// 2026-04-12 CST: Mount the governed stock disclosure-history backfill module,
// because Historical Data Phase 1 needs replayable announcement history to become
// a first-class stock capability instead of remaining live-fetch only.
// Purpose: expose one formal stock entry point for governed disclosure history writes.
#[path = "security_disclosure_history_backfill.rs"]
pub mod security_disclosure_history_backfill;
// 2026-04-12 CST: Mount the live stock disclosure-history backfill module, because
// Historical Data Phase 1 now needs one formal provider-to-governed bridge for
// multi-page announcement history instead of shell-side record assembly.
// Purpose: expose a stable stock boundary for live disclosure-history ingestion.
#[path = "security_disclosure_history_live_backfill.rs"]
pub mod security_disclosure_history_live_backfill;
// 2026-04-12 CST: Mount the governed real-data validation backfill module, because
// the securities mainline now needs one formal tool that refreshes validation slices
// with live-compatible price history and disclosure context.
// Purpose: expose repeatable validation-data refresh through the stock boundary.
#[path = "security_real_data_validation_backfill.rs"]
pub mod security_real_data_validation_backfill;
// 2026-04-11 CST: Mount the governed history-expansion module, because P5 needs
// one first-class stock-domain record that explains which proxy-history windows
// are now eligible for promotion governance.
// Purpose: expose auditable history-coverage growth through the stock boundary.
#[path = "security_history_expansion.rs"]
pub mod security_history_expansion;
// 2026-04-11 CST: Mount the governed shadow-evaluation module, because P5 needs a
// stable review artifact between candidate metrics and any promotion decision.
// Purpose: keep promotion readiness explicit inside the stock mainline.
#[path = "security_shadow_evaluation.rs"]
pub mod security_shadow_evaluation;
// 2026-04-11 CST: Mount the governed model-promotion module, because P5 needs
// grade transitions to become auditable records instead of hidden registry state.
// Purpose: expose candidate/shadow/champion decisions as first-class stock objects.
#[path = "security_model_promotion.rs"]
pub mod security_model_promotion;
#[path = "security_scorecard_model_registry.rs"]
pub mod security_scorecard_model_registry;
#[path = "security_scorecard_refit_run.rs"]
pub mod security_scorecard_refit_run;
// 2026-04-09 CST: 这里挂入正式训练入口模块，原因是 Task 5 需要把离线 scorecard 训练纳入证券主链边界；
// 目的：让训练能力与 snapshot、forward_outcome、refit 处于同一 stock 域内持续演进，避免回退到脚本式管理。
#[path = "security_scorecard_training.rs"]
pub mod security_scorecard_training;
// 2026-04-02 CST: 这里挂入证券审批包版本化模块，原因是正式 decision package 已经存在，需要支持随着审批动作生成新版本；
// 目的：让股票业务域在“可生成、可校验”之后继续具备“可演进”的审批包能力。
#[path = "security_decision_package_revision.rs"]
pub mod security_decision_package_revision;
// 2026-04-08 CST: 这里挂入正式会后结论对象模块，原因是 Task 3 需要把会后治理对象纳入 stock 领域边界；
// 目的：为独立 Tool、后续 package/revision/verify 扩展提供统一归属。
#[path = "security_post_meeting_conclusion.rs"]
pub mod security_post_meeting_conclusion;
// 2026-04-08 CST: 这里挂入会后结论记录 Tool 模块，原因是要让“会后结论落盘 + 触发 revision”沿 stock 主链暴露；
// 目的：避免在 CLI/Skill 层手工串包版本化细节。
#[path = "security_decision_committee.rs"]
pub mod security_decision_committee;
#[path = "security_decision_evidence_bundle.rs"]
pub mod security_decision_evidence_bundle;
#[path = "security_decision_submit_approval.rs"]
pub mod security_decision_submit_approval;
#[path = "security_record_post_meeting_conclusion.rs"]
pub mod security_record_post_meeting_conclusion;
// 2026-04-02 CST: 这里挂入证券审批包校验模块，原因是正式 decision package 已具备治理价值，需要补系统可核验入口；
// 目的：让股票业务域在“可提交审批包”之后继续前进到“可校验审批包”阶段。
#[path = "security_decision_verify_package.rs"]
pub mod security_decision_verify_package;
#[path = "security_position_plan.rs"]
pub mod security_position_plan;
#[path = "security_risk_gates.rs"]
pub mod security_risk_gates;
#[path = "sync_stock_price_history.rs"]
pub mod sync_stock_price_history;
#[path = "technical_consultation_basic.rs"]
pub mod technical_consultation_basic;
