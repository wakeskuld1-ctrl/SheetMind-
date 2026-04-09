// 2026-03-31 CST: 这里建立 stock 模块边界，原因是股票导入、同步和技术面咨询已经不再属于通用分析底座。
// 目的：把股票业务域单独收口，后续新增指标、行情同步和技术咨询一律从这里扩展，不再反向挂回 foundation。
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
// 2026-04-09 CST: 这里挂入主席正式裁决模块，原因是 Task 1 要把“最终正式动作”从投委会线中拆出，
// 目的：让 stock 域内拥有独立的主席裁决入口，后续 package / verify / audit 都能沿这条线继续扩展。
#[path = "security_chair_resolution.rs"]
pub mod security_chair_resolution;
// 2026-04-09 CST: 这里挂入正式特征快照模块，原因是 Task 2 要把 feature_snapshot 底座提升为独立正式对象；
// 目的：让后续 forward_outcome / training / refit 都能沿统一快照入口演进。
#[path = "security_feature_snapshot.rs"]
pub mod security_feature_snapshot;
// 2026-04-09 CST: 这里挂入正式未来标签回填模块，原因是 Task 3 要把 forward_outcome 纳入证券主链的正式对象边界；
// 目的：让后续训练、回算与复盘都沿 stock 域统一扩展，而不是在外层重复拼装未来标签逻辑。
#[path = "security_forward_outcome.rs"]
pub mod security_forward_outcome;
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
