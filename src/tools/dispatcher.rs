use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use serde::Deserialize;
use serde_json::{Value, json};

use crate::domain::handles::TableHandle;
use crate::domain::schema::{ConfidenceLevel, HeaderInference, infer_schema_state_label};
use crate::excel::header_inference::infer_header_schema;
use crate::excel::reader::{list_sheets, open_workbook};
use crate::excel::sheet_range::inspect_sheet_range;
use crate::frame::chart_ref_store::{
    ChartDraftStore, PersistedChartDraft, PersistedChartSeriesSpec, PersistedChartType,
};
use crate::frame::loader::{LoadedTable, load_confirmed_table, load_table_from_table_ref};
use crate::frame::region_loader::load_table_region;
use crate::frame::result_ref_store::{PersistedResultDataset, ResultRefStore};
use crate::frame::source_file_ref_store::{PersistedSourceFileRef, SourceFileRefStore};
use crate::frame::table_ref_store::{PersistedTableRef, TableRefStore};
use crate::frame::workbook_ref_store::{
    PersistedWorkbookColumnConditionalFormatRule, PersistedWorkbookColumnNumberFormatRule,
    PersistedWorkbookDraft, PersistedWorkbookSheetExportOptions, PersistedWorkbookSheetKind,
    WorkbookDraftStore, WorkbookSheetInput,
};
use crate::ops::analyze::analyze_table;
use crate::ops::append::append_tables;
use crate::ops::cast::{CastColumnSpec, cast_column_types, summarize_column_types};
use crate::ops::chart_svg::render_chart_svg;
use crate::ops::cluster_kmeans::cluster_kmeans;
use crate::ops::correlation_analysis::correlation_analysis;
use crate::ops::decision_assistant::decision_assistant;
use crate::ops::deduplicate_by_key::{DeduplicateKeep, OrderSpec, deduplicate_by_key};
use crate::ops::derive::{DerivationSpec, derive_columns};
use crate::ops::distinct_rows::{DistinctKeep, distinct_rows};
use crate::ops::distribution_analysis::distribution_analysis;
use crate::ops::export::{export_csv, export_excel, export_excel_workbook};
use crate::ops::fill_lookup::{FillLookupRule, fill_missing_from_lookup_by_keys};
use crate::ops::fill_missing_values::{FillMissingRule, fill_missing_values};
use crate::ops::filter::{FilterCondition, filter_rows};
use crate::ops::format_table_for_export::{ExportFormatOptions, format_table_for_export};
use crate::ops::group::{AggregationSpec, group_and_aggregate};
use crate::ops::join::{JoinKeepMode, join_tables};
use crate::ops::linear_regression::linear_regression;
use crate::ops::logistic_regression::logistic_regression;
use crate::ops::lookup_values::{LookupSelect, lookup_values_by_keys};
use crate::ops::model_prep::MissingStrategy;
use crate::ops::multi_table_plan::suggest_multi_table_plan;
use crate::ops::normalize_text::{NormalizeTextRule, normalize_text_columns};
use crate::ops::outlier_detection::{OutlierDetectionMethod, outlier_detection};
use crate::ops::parse_datetime::{ParseDateTimeRule, parse_datetime_columns};
use crate::ops::pivot::{PivotAggregation, pivot_table};
use crate::ops::preview::preview_table;
use crate::ops::rename::{RenameColumnMapping, rename_columns};
use crate::ops::report_delivery::{
    ReportDeliveryChart, ReportDeliveryChartSeries, ReportDeliveryChartType,
    ReportDeliveryLegendPosition, ReportDeliveryRequest, ReportDeliverySection,
    build_report_delivery_draft, chart_ref_to_report_delivery_chart,
};
use crate::ops::select::select_columns;
use crate::ops::sort::{SortSpec, sort_rows};
use crate::ops::stat_summary::stat_summary;
use crate::ops::summary::summarize_table;
use crate::ops::table_links::suggest_table_links;
use crate::ops::table_workflow::suggest_table_workflow;
use crate::ops::top_n::top_n_rows;
use crate::ops::trend_analysis::trend_analysis;
use crate::ops::window::{WindowCalculation, WindowOrderSpec, window_calculation};
use crate::runtime::local_memory::{
    EventLogInput, LocalMemoryRuntime, SchemaStatus, SessionStage, SessionStatePatch,
};
use crate::tools::contracts::{ToolRequest, ToolResponse};
mod analysis_ops;
mod multi_table;
mod shared;
mod single_table;
mod stock_ops;
mod workbook_io;

// 2026-03-22: 杩欓噷闆嗕腑鍒嗗彂 Tool 璇锋眰锛岀洰鐨勬槸璁?CLI 鍙礋璐?JSON 鏀跺彂锛岃€屾妸鍏蜂綋鑳藉姏涓嬫矇鍒板悇鑷搷浣滃眰銆?
pub fn dispatch(request: ToolRequest) -> ToolResponse {
    match request.tool.as_str() {
        // 2026-03-25: 鏉╂瑩鍣烽弫鐗堝祦鐏炴墧顥呴梽鍛€瑰﹤鎮庨崝鐔诲厴鐟曚浇妞傞梿鍡氬敶娑擃厽鏋冩稉锟芥惔鏃堢秼閸欏倹鏆熼幍锟界痪?compose/report/chart 閻ㄥ嫭鏆熺紒鍕剁礉閻╊喚娈戦弰顖濐唨璇ユ牸鐞涘本鍎撮惄顔捐厬缂佹挻鐎幒銉ュ弳鐞涘本鏆熼敍宀勫暱閸欏倹鏆熺拠锟?
        "open_workbook" => workbook_io::dispatch_open_workbook(request.args),
        // 2026-03-22: 杩欓噷鎺ュ叆鐙珛 list_sheets 鍏ュ彛锛岀洰鐨勬槸鎶婂伐浣滅翱缁撴瀯鎺㈡煡浠?open_workbook 涓樉寮忔媶鎴愭爣鍑?I/O Tool銆?
        "list_sheets" => workbook_io::dispatch_list_sheets(request.args),
        "inspect_sheet_range" => workbook_io::dispatch_inspect_sheet_range(request.args),
        "load_table_region" => workbook_io::dispatch_load_table_region(request.args),
        "normalize_table" => workbook_io::dispatch_normalize_table(request.args),
        "apply_header_schema" => workbook_io::dispatch_apply_header_schema(request.args),
        // 2026-03-22: 杩欓噷鎺ュ叆浼氳瘽鐘舵€佽鍙栧叆鍙ｏ紝鐩殑鏄鎬诲叆鍙?Skill 鑳藉厛浠庢湰鍦扮嫭绔嬭蹇嗗眰鎭㈠褰撳墠涓婁笅鏂囥€?
        "get_session_state" => workbook_io::dispatch_get_session_state(request.args),
        // 2026-03-22: 杩欓噷鎺ュ叆浼氳瘽鐘舵€佸啓鍏ュ叆鍙ｏ紝鐩殑鏄鎬诲叆鍙?Skill 鑳芥樉寮忕淮鎶ゅ綋鍓嶉樁娈点€佺洰鏍囧拰婵€娲诲彞鏌勩€?
        "update_session_state" => workbook_io::dispatch_update_session_state(request.args),
        "preview_table" => single_table::dispatch_preview_table(request.args),
        "select_columns" => single_table::dispatch_select_columns(request.args),
        "normalize_text_columns" => single_table::dispatch_normalize_text_columns(request.args),
        "rename_columns" => single_table::dispatch_rename_columns(request.args),
        "fill_missing_values" => single_table::dispatch_fill_missing_values(request.args),
        "distinct_rows" => single_table::dispatch_distinct_rows(request.args),
        "deduplicate_by_key" => single_table::dispatch_deduplicate_by_key(request.args),
        "format_table_for_export" => single_table::dispatch_format_table_for_export(request.args),
        "fill_missing_from_lookup" => single_table::dispatch_fill_missing_from_lookup(request.args),
        "parse_datetime_columns" => single_table::dispatch_parse_datetime_columns(request.args),
        "lookup_values" => single_table::dispatch_lookup_values(request.args),
        "window_calculation" => single_table::dispatch_window_calculation(request.args),
        "filter_rows" => single_table::dispatch_filter_rows(request.args),
        "cast_column_types" => single_table::dispatch_cast_column_types(request.args),
        "derive_columns" => single_table::dispatch_derive_columns(request.args),
        "group_and_aggregate" => single_table::dispatch_group_and_aggregate(request.args),
        "pivot_table" => single_table::dispatch_pivot_table(request.args),
        "sort_rows" => single_table::dispatch_sort_rows(request.args),
        "top_n" => single_table::dispatch_top_n(request.args),
        "compose_workbook" => dispatch_compose_workbook(request.args),
        "report_delivery" => dispatch_report_delivery(request.args),
        "build_chart" => dispatch_build_chart(request.args),
        "export_chart_image" => dispatch_export_chart_image(request.args),
        "export_csv" => workbook_io::dispatch_export_csv(request.args),
        "export_excel" => workbook_io::dispatch_export_excel(request.args),
        "export_excel_workbook" => workbook_io::dispatch_export_excel_workbook(request.args),
        "join_tables" => multi_table::dispatch_join_tables(request.args),
        // 2026-03-21: 杩欓噷鎺ュ叆鏄炬€у叧鑱斿€欓€夊垎鍙戯紝鐩殑鏄 CLI 鍏堢粰涓婂眰杩斿洖鈥滃厛寤鸿銆佸啀鎵ц鈥濈殑澶氳〃鍏ュ彛銆?
        "suggest_table_links" => multi_table::dispatch_suggest_table_links(request.args),
        // 2026-03-22: 杩欓噷鎺ュ叆澶氳〃娴佺▼寤鸿鍒嗗彂锛岀洰鐨勬槸鍏堝垽鏂洿鍍忚拷鍔犺繕鏄叧鑱旓紝鍐嶄氦缁欎笂灞?Skill 鍋氱‘璁ゃ€?
        "suggest_table_workflow" => multi_table::dispatch_suggest_table_workflow(request.args),
        // 2026-03-22: 杩欓噷鎺ュ叆澶氳〃椤哄簭寤鸿鍒嗗彂锛岀洰鐨勬槸璁?CLI 鐩存帴杩斿洖鈥滃厛杩藉姞銆佸啀鍏宠仈鈥濈殑淇濆畧璁″垝姝ラ銆?
        "suggest_multi_table_plan" => multi_table::dispatch_suggest_multi_table_plan(request.args),
        "append_tables" => multi_table::dispatch_append_tables(request.args),
        "summarize_table" => analysis_ops::dispatch_summarize_table(request.args),
        "analyze_table" => analysis_ops::dispatch_analyze_table(request.args),
        "stat_summary" => analysis_ops::dispatch_stat_summary(request.args),
        "correlation_analysis" => analysis_ops::dispatch_correlation_analysis(request.args),
        "outlier_detection" => analysis_ops::dispatch_outlier_detection(request.args),
        "distribution_analysis" => analysis_ops::dispatch_distribution_analysis(request.args),
        "trend_analysis" => analysis_ops::dispatch_trend_analysis(request.args),
        // 2026-03-31 CST: 这里把股票技术面 Tool 明确切到 stock dispatcher，原因是股票业务域不应继续挂在通用分析 dispatcher 上。
        // 目的：先在分发入口建立 foundation / stock 边界，避免后续继续把股票能力写回 analysis_ops。
        "technical_consultation_basic" => {
            stock_ops::dispatch_technical_consultation_basic(request.args)
        }
        // 2026-04-01 CST: 这里接入上层综合证券分析 Tool，原因是方案 A 已确认需要在主分发层正式暴露“个股 + 大盘 + 板块”的统一入口。
        // 目的：让 CLI / Skill 直接消费聚合后的证券分析结论，而不是在外层手工串三次技术面 Tool。
        "security_analysis_contextual" => {
            stock_ops::dispatch_security_analysis_contextual(request.args)
        }
        // 2026-04-01 CST: 这里接入 fullstack 总 Tool，原因是方案 1 已确认要把技术与信息面统一暴露到产品主链；
        // 目的：让 CLI / Skill 直接消费完整证券分析结果，而不是在外层继续拼财报和公告抓取。
        "security_analysis_fullstack" => {
            stock_ops::dispatch_security_analysis_fullstack(request.args)
        }
        // 2026-03-31 CST: 这里把股票历史导入 Tool 切到 stock dispatcher，原因是股票历史导入属于股票业务域而不是通用分析域。
        // 目的：把 “CSV -> SQLite” 明确收口到 stock 模块，和 foundation 底座入口隔离开。
        // 2026-04-01 CST: 这里接入证券投决证据包 Tool，原因是方案 B 需要先把研究链冻结成统一 evidence bundle；
        // 目的：让 CLI / Skill 在单次对话里也能确保正反方共享同源证据，而不是各自直接读取研究 Tool。
        "security_decision_evidence_bundle" => {
            stock_ops::dispatch_security_decision_evidence_bundle(request.args)
        }
        // 2026-04-01 CST: 这里接入证券投决会总入口 Tool，原因是用户已经明确要求研究链上面补一个双立场投决层；
        // 目的：正式暴露“证据冻结 + 正反方 + 闸门 + 投决卡”的统一产品入口。
        "security_decision_committee" => {
            stock_ops::dispatch_security_decision_committee(request.args)
        }
        "security_committee_member_agent" => {
            stock_ops::dispatch_security_committee_member_agent(request.args)
        }
        // 2026-04-09 CST: 这里把主席正式裁决 Tool 接入主 dispatcher，原因是 Task 1 要让主席线成为独立可路由能力；
        // 目的：让最终正式决议对象能够像 committee / scorecard 一样被 CLI 和 Skill 正式调用。
        "security_chair_resolution" => stock_ops::dispatch_security_chair_resolution(request.args),
        // 2026-04-12 CST: Route the formal condition-review tool through the main
        // dispatcher, because P8 begins the execution loop by formalizing review triggers.
        // Purpose: keep lifecycle review on the same public path as approval and scorecards.
        "security_condition_review" => stock_ops::dispatch_security_condition_review(request.args),
        // 2026-04-12 CST: Route the formal execution-record tool through the main
        // dispatcher, because P8 turns execution events into first-class replayable artifacts.
        // Purpose: keep execution facts on the same public stock path as reviews and approvals.
        "security_execution_record" => stock_ops::dispatch_security_execution_record(request.args),
        // 2026-04-12 CST: Route the formal post-trade review tool through the main
        // dispatcher, because P8 closes the execution loop with layered replayable review artifacts.
        // Purpose: keep post-trade review on the same public stock path as the rest of the lifecycle.
        "security_post_trade_review" => {
            stock_ops::dispatch_security_post_trade_review(request.args)
        }
        // 2026-04-09 CST: 这里把特征快照 Tool 接入主 dispatcher，原因是 Task 2 要让训练底座对象成为正式可路由能力；
        // 目的：让 feature_snapshot 不再只是内部辅助逻辑，而是 CLI / Skill 可直接调用的正式入口。
        "security_feature_snapshot" => stock_ops::dispatch_security_feature_snapshot(request.args),
        // 2026-04-09 CST: 这里把未来标签回填 Tool 接入主 dispatcher，原因是 Task 3 要让 forward_outcome 成为主链一等能力；
        // 目的：让 CLI / Skill 可以直接路由到 snapshot 绑定的多期限标签结果，而不是外层手工拼调用链。
        "security_external_proxy_backfill" => {
            // 2026-04-11 CST: Route the dated external-proxy backfill tool through the
            // main dispatcher, because governed historical proxy ingestion now belongs
            // to the same public stock tool chain as feature snapshots and approvals.
            // Purpose: let CLI and Skills persist dated proxy rows without hidden helpers.
            stock_ops::dispatch_security_external_proxy_backfill(request.args)
        }
        "security_external_proxy_history_import" => {
            // 2026-04-12 CST: Route the file-based proxy-history import tool through the
            // main dispatcher, because real ETF proxy batches now belong to the same
            // public stock chain as other governed history tools.
            // Purpose: keep proxy-history import off ad-hoc shell paths.
            stock_ops::dispatch_security_external_proxy_history_import(request.args)
        }
        "security_fundamental_history_backfill" => {
            // 2026-04-12 CST: Route the governed stock fundamental-history backfill
            // tool through the main dispatcher, because replayable financial history
            // now belongs to the same public stock chain as price and proxy backfill.
            // Purpose: keep stock financial-history writes off ad-hoc shell flows.
            stock_ops::dispatch_security_fundamental_history_backfill(request.args)
        }
        "security_fundamental_history_live_backfill" => {
            // 2026-04-12 CST: Route the live governed financial-history tool through the
            // main dispatcher, because multi-period provider imports now belong to the
            // same public stock chain as other governed history tools.
            // Purpose: keep live financial-history import off ad-hoc shell paths.
            stock_ops::dispatch_security_fundamental_history_live_backfill(request.args)
        }
        "security_disclosure_history_backfill" => {
            // 2026-04-12 CST: Route the governed stock disclosure-history backfill
            // tool through the main dispatcher, because replayable announcement history
            // now belongs to the same public stock chain as price and proxy backfill.
            // Purpose: keep stock disclosure-history writes off ad-hoc shell flows.
            stock_ops::dispatch_security_disclosure_history_backfill(request.args)
        }
        "security_disclosure_history_live_backfill" => {
            // 2026-04-12 CST: Route the live governed disclosure-history tool through the
            // main dispatcher, because multi-page provider imports now belong to the
            // same public stock chain as other governed history tools.
            // Purpose: keep live disclosure-history import off ad-hoc shell paths.
            stock_ops::dispatch_security_disclosure_history_live_backfill(request.args)
        }
        "security_real_data_validation_backfill" => {
            // 2026-04-12 CST: Route the governed real-data validation backfill tool
            // through the main dispatcher, because validation-slice refresh now belongs
            // to the same public stock chain as lifecycle replay and proxy backfill.
            // Purpose: keep live-compatible validation refresh off ad-hoc shell paths.
            stock_ops::dispatch_security_real_data_validation_backfill(request.args)
        }
        "security_history_expansion" => {
            stock_ops::dispatch_security_history_expansion(request.args)
        }
        "security_shadow_evaluation" => {
            stock_ops::dispatch_security_shadow_evaluation(request.args)
        }
        "security_model_promotion" => stock_ops::dispatch_security_model_promotion(request.args),
        "security_forward_outcome" => stock_ops::dispatch_security_forward_outcome(request.args),
        // 2026-04-11 CST: 这里把 master_scorecard Tool 接入主 dispatcher，原因是方案 C 已确认要让
        // “未来几日赚钱效益总卡”成为正式主链能力。
        // 目的：让 CLI / Skill 直接获得 committee + scorecard + master_scorecard 的统一输出。
        "security_master_scorecard" => stock_ops::dispatch_security_master_scorecard(request.args),
        "security_scorecard_refit" => stock_ops::dispatch_security_scorecard_refit(request.args),
        // 2026-04-09 CST: 这里把正式 scorecard training Tool 接入主 dispatcher，原因是 Task 5 需要让训练入口成为一等可路由能力；
        // 目的：让训练主链可以像 snapshot/forward_outcome/refit 一样被 CLI 和 Skill 正式调度。
        "security_scorecard_training" => {
            stock_ops::dispatch_security_scorecard_training(request.args)
        }
        // 2026-04-02 CST: 这里接入证券审批提交总入口 Tool，原因是用户已经批准把证券投决结果正式送进审批治理主线；
        // 目的：让 CLI / Skill 可以一次完成“投决 + 提交审批”，不再停留在单纯研究建议。
        "security_decision_submit_approval" => {
            stock_ops::dispatch_security_decision_submit_approval(request.args)
        }
        // 2026-04-02 CST: 这里接入证券审批包校验总入口 Tool，原因是用户已经批准把 decision package 从“可生成”推进到“可核验”；
        // 目的：让 CLI / Skill 可以正式调用 package verify，而不是在外层手工读文件比哈希。
        "security_decision_verify_package" => {
            stock_ops::dispatch_security_decision_verify_package(request.args)
        }
        // 2026-04-02 CST: 这里接入证券审批包版本化总入口 Tool，原因是用户已经批准让 decision package 跟随审批动作生成新版本；
        // 目的：让 CLI / Skill 可以正式调用 package revision，而不是在外层手工复制和重算 package。
        "security_decision_package_revision" => {
            stock_ops::dispatch_security_decision_package_revision(request.args)
        }
        // 2026-04-08 CST: 这里把会后结论记录 Tool 接入主 dispatcher，原因是红测要求从 CLI 主入口即可调用该能力；
        // 目的：让“会后结论正式落盘”成为一等治理入口，而不是外层脚本拼装动作。
        "security_record_post_meeting_conclusion" => {
            stock_ops::dispatch_security_record_post_meeting_conclusion(request.args)
        }
        "import_stock_price_history" => {
            stock_ops::dispatch_import_stock_price_history(request.args)
        }
        // 2026-03-31 CST: 这里把股票历史 HTTP 同步 Tool 切到 stock dispatcher，原因是联网行情补数已经是股票域专属能力。
        // 目的：让 provider、日期和技术咨询前置数据准备继续停留在 stock 模块里独立演进。
        "sync_stock_price_history" => stock_ops::dispatch_sync_stock_price_history(request.args),
        // 2026-03-28 23:54 CST: 这里把统计诊断组合 Tool 接入主 dispatcher，原因是高层组合能力也必须沿现有 Rust Tool 主链暴露；
        // 目的是让统一 JSON 诊断包可以和其它 Tool 一样被 CLI 与后续编排直接调用。
        "diagnostics_report" => analysis_ops::dispatch_diagnostics_report(request.args),
        // 2026-03-29 00:08 CST：这里把组合诊断 Excel 报表 Tool 接入 dispatcher，原因是 workbook-first 交付也必须沿现有 Rust Tool 主链暴露；
        // 目的：让 CLI 和后续自动化能直接调用“组合诊断 -> workbook/xlsx”的最终交付入口。
        "diagnostics_report_excel_report" => {
            analysis_ops::dispatch_diagnostics_report_excel_report(request.args)
        }
        // 2026-03-28 10:42 CST: 这里把容量评估接入主 dispatcher，原因是要沿用现有 Tool 分发骨架；目的是让 CLI/自动化链路都能直接调用新场景能力。
        "capacity_assessment" => analysis_ops::dispatch_capacity_assessment(request.args),
        // 2026-03-28 16:55 CST: 这里把容量桥接 Tool 接入主 dispatcher，原因是要让 SSH 盘点结果可以直接转成容量分析；目的是减少调用方手工做证据映射。
        "capacity_assessment_from_inventory" => {
            analysis_ops::dispatch_capacity_assessment_from_inventory(request.args)
        }
        // 2026-03-28 16:12 CST: 这里把受限 SSH 盘点接入主 dispatcher，原因是 CLI 与自动化调用需要统一走正式 Tool 分发；目的是让安全白名单采集能被外部直接触达。
        // 2026-03-28 22:19 CST: 这里把容量评估 Excel 报表 Tool 接入主 dispatcher，原因是用户要“一步出 Excel”的正式入口；
        // 目的是让 CLI、自动化和后续 Skill 都能直接调用最终交付能力。
        "capacity_assessment_excel_report" => {
            analysis_ops::dispatch_capacity_assessment_excel_report(request.args)
        }
        "ssh_inventory" => analysis_ops::dispatch_ssh_inventory(request.args),
        "linear_regression" => analysis_ops::dispatch_linear_regression(request.args),
        "logistic_regression" => analysis_ops::dispatch_logistic_regression(request.args),
        "cluster_kmeans" => analysis_ops::dispatch_cluster_kmeans(request.args),
        "decision_assistant" => analysis_ops::dispatch_decision_assistant(request.args),
        _ => ToolResponse::error(format!("鏆備笉鏀寔鐨?tool: {}", request.tool)),
    }
}

fn dispatch_open_workbook(args: Value) -> ToolResponse {
    let Some(path) = args.get("path").and_then(|value| value.as_str()) else {
        // 2026-03-23: 杩欓噷鏀跺彛 open_workbook 缂哄弬鎶ラ敊锛屽師鍥犳槸鍘嗗彶涔辩爜瀵艰嚧 UTF-8 鏂囨鍥炲綊澶辫触锛涚洰鐨勬槸淇濊瘉 CLI 瀵规櫘閫氱敤鎴疯緭鍑虹ǔ瀹氬彲璇荤殑涓枃鎻愮ず銆?
        return ToolResponse::error("open_workbook 缺少 path 参数");
    };

    match open_workbook(path) {
        // 2026-03-23: 杩欓噷鎶?open_workbook 鎴愬姛缁撴灉缁熶竴鍗囩骇鎴愬甫 file_ref 鐨勫搷搴旓紝鍘熷洜鏄悗缁祦绋嬮渶瑕佹寜鈥滅鍑犱釜 Sheet鈥濈户缁紱鐩殑鏄湪涓嶇Щ闄ゆ棫 path/sheet 瀛楁鐨勫墠鎻愪笅琛ラ綈绋冲Ε鍏ュ彛銆?
        Ok(summary) => build_opened_file_response(&args, summary),
        Err(error) => ToolResponse::error(error.to_string()),
    }
}

fn dispatch_list_sheets(args: Value) -> ToolResponse {
    let Some(path) = args.get("path").and_then(|value| value.as_str()) else {
        return ToolResponse::error("list_sheets 缺少 path 参数");
    };

    match list_sheets(path) {
        // 2026-03-23: 杩欓噷璁?list_sheets 涓?open_workbook 澶嶇敤鍚屼竴濂?file_ref 鍝嶅簲缁撴瀯锛屽師鍥犳槸涓よ€呴兘灞炰簬宸ヤ綔绨挎帰鏌ュ叆鍙ｏ紱鐩殑鏄鍚庣画 Tool 閮借兘缁х画鎸?Sheet 绱㈠紩涓叉帴銆?
        Ok(summary) => build_opened_file_response(&args, summary),
        Err(error) => ToolResponse::error(error.to_string()),
    }
}

fn dispatch_inspect_sheet_range(args: Value) -> ToolResponse {
    let source = match resolve_sheet_source(&args, "inspect_sheet_range") {
        Ok(source) => source,
        Err(response) => return response,
    };
    let sample_rows = args
        .get("sample_rows")
        .and_then(|value| value.as_u64())
        .unwrap_or(5) as usize;

    match inspect_sheet_range(&source.path, &source.sheet_name, sample_rows) {
        Ok(inspection) => ToolResponse::ok(json!(inspection)),
        Err(error) => ToolResponse::error(error.to_string()),
    }
}

fn dispatch_load_table_region(args: Value) -> ToolResponse {
    let source = match resolve_sheet_source(&args, "load_table_region") {
        Ok(source) => source,
        Err(response) => return response,
    };
    let Some(region) = args.get("range").and_then(|value| value.as_str()) else {
        return ToolResponse::error("load_table_region 缺少 range 参数");
    };
    let header_row_count = args
        .get("header_row_count")
        .and_then(|value| value.as_u64())
        .unwrap_or(1) as usize;

    match load_table_region(&source.path, &source.sheet_name, region, header_row_count) {
        Ok(loaded) => {
            let table_ref = TableRefStore::create_table_ref();
            // 2026-03-22: 杩欓噷鎶婃樉寮忓尯鍩熺粨鏋滃悓姝ユ矇娣€涓?table_ref锛岀洰鐨勬槸璁╁悗缁?preview銆佸垎鏋愬拰瀵煎嚭閮借兘澶嶇敤鍚屼竴涓‘璁ゆ€佸彞鏌勩€?
            let persisted = match PersistedTableRef::from_region(
                table_ref.clone(),
                &source.path,
                &source.sheet_name,
                region,
                loaded.handle.columns().to_vec(),
                header_row_count,
            ) {
                Ok(persisted) => persisted,
                Err(error) => return ToolResponse::error(error.to_string()),
            };
            let store = match TableRefStore::workspace_default() {
                Ok(store) => store,
                Err(error) => return ToolResponse::error(error.to_string()),
            };
            if let Err(error) = store.save(&persisted) {
                return ToolResponse::error(error.to_string());
            }
            if let Err(response) =
                sync_confirmed_table_state(&args, &persisted, "鏌ョ湅鍖哄煙鍔犺浇缁撴灉")
            {
                return response;
            }

            match preview_table(&loaded.dataframe, 5) {
                Ok(preview) => respond_with_result_dataset(
                    "load_table_region",
                    &args,
                    &loaded,
                    json!({
                        "path": source.path,
                        "sheet": source.sheet_name,
                        "range": region,
                        "header_row_count": header_row_count,
                        "table_ref": persisted.table_ref,
                        "columns": preview.columns,
                        "rows": preview.rows,
                        "row_count": loaded.dataframe.height(),
                    }),
                ),
                Err(error) => ToolResponse::error(error.to_string()),
            }
        }
        Err(error) => ToolResponse::error(error.to_string()),
    }
}

fn dispatch_normalize_table(args: Value) -> ToolResponse {
    let source = match resolve_sheet_source(&args, "normalize_table") {
        Ok(source) => source,
        Err(response) => return response,
    };

    match infer_header_schema(&source.path, &source.sheet_name) {
        Ok(inference) => build_inference_response(&source.sheet_name, inference),
        Err(error) => ToolResponse::error(error.to_string()),
    }
}

fn dispatch_apply_header_schema(args: Value) -> ToolResponse {
    let source = match resolve_sheet_source(&args, "apply_header_schema") {
        Ok(source) => source,
        Err(response) => return response,
    };

    match infer_header_schema(&source.path, &source.sheet_name) {
        Ok(inference) => {
            // 2026-03-22: 杩欓噷鎶婃帹鏂粨鏋滄彁鍗囦负 confirmed 鍚庡啀钀界洏 table_ref锛岀洰鐨勬槸璁╄〃澶寸‘璁ゆ€佽兘绋冲畾杩涘叆鍚庣画澶氭闂幆銆?
            let forced_inference = HeaderInference {
                columns: inference.columns.clone(),
                confidence: ConfidenceLevel::High,
                schema_state: crate::domain::schema::SchemaState::Confirmed,
                header_row_count: inference.header_row_count,
                data_start_row_index: inference.data_start_row_index,
            };

            match load_confirmed_table(&source.path, &source.sheet_name, &forced_inference) {
                Ok(loaded) => {
                    let row_count = loaded.dataframe.height();
                    let table_ref = TableRefStore::create_table_ref();
                    let persisted = match PersistedTableRef::from_confirmed_schema(
                        table_ref.clone(),
                        &source.path,
                        &source.sheet_name,
                        &forced_inference,
                    ) {
                        Ok(persisted) => persisted,
                        Err(error) => return ToolResponse::error(error.to_string()),
                    };
                    let store = match TableRefStore::workspace_default() {
                        Ok(store) => store,
                        Err(error) => return ToolResponse::error(error.to_string()),
                    };
                    if let Err(error) = store.save(&persisted) {
                        return ToolResponse::error(error.to_string());
                    }
                    if let Err(response) = sync_confirmed_table_state(
                        &args,
                        &persisted,
                        // 2026-03-24: 这里修复本轮必经路径上的坏字符串，原因是历史乱码吞掉了结束引号并连带破坏整文件语法；目的是恢复 apply_header_schema 这条基线路径的可编译状态。
                        "查看确认后的表",
                    ) {
                        return response;
                    }

                    ToolResponse::ok(json!({
                        "table_id": table_ref,
                        "table_ref": persisted.table_ref,
                        "schema_state": infer_schema_state_label(loaded.handle.schema_state()),
                        "sheet": loaded.handle.sheet_name(),
                        "columns": forced_inference.columns,
                        "row_count": row_count,
                    }))
                }
                Err(error) => ToolResponse::error(error.to_string()),
            }
        }
        Err(error) => ToolResponse::error(error.to_string()),
    }
}

// 2026-03-23: 杩欓噷琛ュ洖浼氳瘽鐘舵€佹洿鏂板叆鍙傜粨鏋勶紝鍘熷洜鏄紪璇戦樆濉炴毚闇插嚭璇ュ畾涔夌己澶憋紱鐩殑鏄仮澶?update_session_state 鍒嗗彂閾捐矾骞惰鍚庣画娴嬭瘯鑳界户缁帹杩涖€?
#[derive(Debug, Deserialize, Default)]
struct UpdateSessionStateInput {
    session_id: Option<String>,
    current_workbook: Option<String>,
    current_sheet: Option<String>,
    current_file_ref: Option<String>,
    current_sheet_index: Option<usize>,
    current_stage: Option<SessionStage>,
    schema_status: Option<SchemaStatus>,
    active_table_ref: Option<String>,
    active_handle_ref: Option<String>,
    // 2026-03-23: 杩欓噷琛?active_handle_kind 鍏ュ弬锛屽師鍥犳槸鏈疆瑕佹妸鏈€鏂版縺娲诲彞鏌勭殑绫诲瀷鏄惧紡钀藉埌 session_state锛涚洰鐨勬槸璁?update_session_state 涔熻兘鍙備笌澶氭闂幆娴嬭瘯涓庢墜宸ヨ皟璇曘€?
    active_handle_kind: Option<String>,
    last_user_goal: Option<String>,
    selected_columns: Option<Vec<String>>,
}

fn dispatch_get_session_state(args: Value) -> ToolResponse {
    let runtime = match memory_runtime() {
        Ok(runtime) => runtime,
        Err(response) => return response,
    };
    let session_id = session_id_from_args(&args);

    match runtime.get_session_state(&session_id) {
        Ok(state) => ToolResponse::ok(build_session_state_response(&state)),
        Err(error) => ToolResponse::error(error.to_string()),
    }
}

fn dispatch_update_session_state(args: Value) -> ToolResponse {
    let payload = match serde_json::from_value::<UpdateSessionStateInput>(args.clone()) {
        Ok(payload) => payload,
        Err(error) => {
            return ToolResponse::error(format!("update_session_state 参数解析失败: {error}"));
        }
    };
    let runtime = match memory_runtime() {
        Ok(runtime) => runtime,
        Err(response) => return response,
    };
    let session_id = payload.session_id.unwrap_or_else(|| "default".to_string());
    let patch = SessionStatePatch {
        current_workbook: payload.current_workbook,
        current_sheet: payload.current_sheet,
        current_file_ref: payload.current_file_ref,
        current_sheet_index: payload.current_sheet_index,
        current_stage: payload.current_stage,
        schema_status: payload.schema_status,
        // 2026-03-23: 杩欓噷淇濈暀 active_table_ref 鐨勫吋瀹瑰厹搴曪紝鍘熷洜鏄棫璇锋眰鍙兘鍙紶 active_handle_ref锛涚洰鐨勬槸鍦ㄥ垎绂绘柊鏃ц涔夊悗锛屼粛璁╄€佽姹傜户缁鍒版棫瀛楁锛岃€屾柊璇锋眰鍦ㄦ樉寮忎紶鍏?active_table_ref 鏃朵粛鍙繚鐣欏垎绂昏涔夈€?
        active_table_ref: payload
            .active_table_ref
            .or_else(|| payload.active_handle_ref.clone()),
        active_handle_ref: payload.active_handle_ref,
        active_handle_kind: payload.active_handle_kind,
        last_user_goal: payload.last_user_goal,
        selected_columns: payload.selected_columns,
    };

    match runtime.update_session_state(&session_id, &patch) {
        Ok(state) => {
            let _ = runtime.append_event(
                &session_id,
                &EventLogInput {
                    event_type: "session_state_updated".to_string(),
                    stage: Some(state.current_stage.clone()),
                    tool_name: Some("update_session_state".to_string()),
                    status: "ok".to_string(),
                    // 2026-03-24: 这里修复本轮实际碰到的坏字符串，原因是历史乱码把 closing quote 吃掉后会连带破坏整个 dispatcher 的语法解析；目的是先恢复本轮图表桥接开发所需的可编译基线。
                    message: Some("总入口显式更新会话状态".to_string()),
                },
            );
            ToolResponse::ok(build_session_state_response(&state))
        }
        Err(error) => ToolResponse::error(error.to_string()),
    }
}

// 2026-03-22: 杩欓噷涓轰細璇濈姸鎬佽ˉ鍏呮縺娲诲彞鏌勬憳瑕侊紝鐩殑鏄湪淇濈暀 active_table_ref 鍏煎瀛楁鐨勫悓鏃讹紝瀵逛笂灞?Skill 鏆撮湶鏇存竻鏅扮殑婵€娲讳笂涓嬫枃銆?
fn build_session_state_response(state: &crate::runtime::local_memory::SessionState) -> Value {
    let mut payload = json!(state);
    if let Some(object) = payload.as_object_mut() {
        // 2026-03-23: 杩欓噷浼樺厛鏆撮湶鏄惧紡婵€娲诲彞鏌勶紝鍘熷洜鏄柟妗圔瑕佹眰浼氳瘽鐘舵€佺洿鎺ユ寚鍚戝綋鍓嶆渶鏂扮粨鏋滐紱鐩殑鏄湪淇濈暀 active_table_ref 鍏煎瀛楁鐨勫悓鏃讹紝鎶婃渶鏂?handle 璇箟绋冲畾鏆撮湶缁欎笂灞?Skill銆?
        let active_handle_ref = state
            .active_handle_ref
            .clone()
            .or_else(|| state.active_table_ref.clone());
        let active_handle_kind = state.active_handle_kind.clone().or_else(|| {
            active_handle_ref
                .as_deref()
                .map(classify_handle_kind)
                .map(str::to_string)
        });
        object.insert(
            "active_handle_ref".to_string(),
            active_handle_ref
                .clone()
                .map(Value::String)
                .unwrap_or(Value::Null),
        );
        object.insert(
            "active_handle".to_string(),
            active_handle_ref
                .as_ref()
                .map(|reference| {
                    let kind = active_handle_kind
                        .clone()
                        .unwrap_or_else(|| classify_handle_kind(reference).to_string());
                    json!({
                        "ref": reference,
                        "kind": kind,
                    })
                })
                .unwrap_or(Value::Null),
        );
    }
    payload
}

// 2026-03-22: 杩欓噷鎸夊彞鏌勫墠缂€鎺ㄦ柇婵€娲诲璞＄被鍨嬶紝鐩殑鏄 table_ref/result_ref/workbook_ref 鍦ㄤ細璇濇憳瑕侀噷涓€鐪煎彲杈ㄣ€?
fn classify_handle_kind(reference: &str) -> &'static str {
    if reference.starts_with("result_") {
        return "result_ref";
    }
    if reference.starts_with("table_") {
        return "table_ref";
    }
    if reference.starts_with("workbook_") {
        return "workbook_ref";
    }
    if reference.starts_with("chart_") {
        return "chart_ref";
    }
    "unknown"
}

fn dispatch_preview_table(args: Value) -> ToolResponse {
    let limit = args
        .get("limit")
        .and_then(|value| value.as_u64())
        .unwrap_or(5) as usize;

    match load_table_for_tool(&args, "preview_table") {
        Ok(OperationLoad::NeedsConfirmation(response)) => response,
        Ok(OperationLoad::Loaded(loaded)) => preview_loaded_table(&loaded, limit),
        Err(response) => response,
    }
}

fn dispatch_select_columns(args: Value) -> ToolResponse {
    let Some(columns) = args.get("columns").and_then(|value| value.as_array()) else {
        return ToolResponse::error("select_columns 缂哄皯 columns 鍙傛暟");
    };
    let requested_columns = columns
        .iter()
        .filter_map(|value| value.as_str())
        .collect::<Vec<_>>();

    match load_table_for_tool(&args, "select_columns") {
        Ok(OperationLoad::NeedsConfirmation(response)) => response,
        Ok(OperationLoad::Loaded(loaded)) => match select_columns(&loaded, &requested_columns) {
            Ok(selected) => respond_with_result_dataset(
                "select_columns",
                &args,
                &selected,
                json!({
                    "columns": selected.handle.columns(),
                    "row_count": selected.dataframe.height(),
                }),
            ),
            Err(error) => ToolResponse::error(error.to_string()),
        },
        Err(response) => response,
    }
}

fn dispatch_filter_rows(args: Value) -> ToolResponse {
    let Some(conditions_value) = args.get("conditions") else {
        return ToolResponse::error("filter_rows 缂哄皯 conditions 鍙傛暟");
    };
    let conditions = match serde_json::from_value::<Vec<FilterCondition>>(conditions_value.clone())
    {
        Ok(conditions) => conditions,
        Err(error) => {
            return ToolResponse::error(format!("filter_rows 鏉′欢瑙ｆ瀽澶辫触: {error}"));
        }
    };

    match load_table_for_tool(&args, "filter_rows") {
        Ok(OperationLoad::NeedsConfirmation(response)) => response,
        Ok(OperationLoad::Loaded(loaded)) => match filter_rows(&loaded, &conditions) {
            Ok(filtered) => respond_with_preview_and_result_ref("filter_rows", &args, &filtered, 5),
            Err(error) => ToolResponse::error(error.to_string()),
        },
        Err(response) => response,
    }
}

fn dispatch_cast_column_types(args: Value) -> ToolResponse {
    let Some(casts_value) = args.get("casts") else {
        return ToolResponse::error("cast_column_types 缂哄皯 casts 鍙傛暟");
    };
    let casts = match serde_json::from_value::<Vec<CastColumnSpec>>(casts_value.clone()) {
        Ok(casts) => casts,
        Err(error) => {
            return ToolResponse::error(format!("cast_column_types 鍙傛暟瑙ｆ瀽澶辫触: {error}"));
        }
    };

    match load_table_for_tool(&args, "cast_column_types") {
        Ok(OperationLoad::NeedsConfirmation(response)) => response,
        Ok(OperationLoad::Loaded(loaded)) => match cast_column_types(&loaded, &casts) {
            Ok(casted) => respond_with_result_dataset(
                "cast_column_types",
                &args,
                &casted,
                json!({
                    "columns": casted.handle.columns(),
                    "column_types": summarize_column_types(&casted.dataframe),
                    "row_count": casted.dataframe.height(),
                }),
            ),
            Err(error) => ToolResponse::error(error.to_string()),
        },
        Err(response) => response,
    }
}

fn dispatch_derive_columns(args: Value) -> ToolResponse {
    let Some(derivations_value) = args.get("derivations") else {
        return ToolResponse::error("derive_columns 缂哄皯 derivations 鍙傛暟");
    };
    let derivations = match serde_json::from_value::<Vec<DerivationSpec>>(derivations_value.clone())
    {
        Ok(derivations) => derivations,
        Err(error) => {
            return ToolResponse::error(format!("derive_columns 鍙傛暟瑙ｆ瀽澶辫触: {error}"));
        }
    };
    let casts = match parse_casts(&args, "casts", "derive_columns") {
        Ok(casts) => casts,
        Err(response) => return response,
    };

    match load_table_for_tool(&args, "derive_columns") {
        Ok(OperationLoad::NeedsConfirmation(response)) => response,
        Ok(OperationLoad::Loaded(loaded)) => match apply_optional_casts(loaded, &casts) {
            Ok(prepared_loaded) => match derive_columns(&prepared_loaded, &derivations) {
                Ok(derived) => respond_with_preview_and_result_ref(
                    "derive_columns",
                    &args,
                    &derived,
                    derived.dataframe.height(),
                ),
                Err(error) => ToolResponse::error(error.to_string()),
            },
            Err(error) => ToolResponse::error(error),
        },
        Err(response) => response,
    }
}

fn dispatch_group_and_aggregate(args: Value) -> ToolResponse {
    let Some(group_by_value) = args.get("group_by").and_then(|value| value.as_array()) else {
        return ToolResponse::error("group_and_aggregate 缂哄皯 group_by 鍙傛暟");
    };
    let Some(aggregations_value) = args.get("aggregations") else {
        return ToolResponse::error("group_and_aggregate 缂哄皯 aggregations 鍙傛暟");
    };
    let group_by = group_by_value
        .iter()
        .filter_map(|value| value.as_str())
        .collect::<Vec<_>>();
    let aggregations =
        match serde_json::from_value::<Vec<AggregationSpec>>(aggregations_value.clone()) {
            Ok(aggregations) => aggregations,
            Err(error) => {
                return ToolResponse::error(format!(
                    "group_and_aggregate 鍙傛暟瑙ｆ瀽澶辫触: {error}"
                ));
            }
        };
    let casts = match parse_casts(&args, "casts", "group_and_aggregate") {
        Ok(casts) => casts,
        Err(response) => return response,
    };

    match load_table_for_tool(&args, "group_and_aggregate") {
        Ok(OperationLoad::NeedsConfirmation(response)) => response,
        Ok(OperationLoad::Loaded(loaded)) => match apply_optional_casts(loaded, &casts) {
            Ok(prepared_loaded) => {
                match group_and_aggregate(&prepared_loaded, &group_by, &aggregations) {
                    Ok(grouped) => respond_with_preview_and_result_ref(
                        "group_and_aggregate",
                        &args,
                        &grouped,
                        grouped.dataframe.height(),
                    ),
                    Err(error) => ToolResponse::error(error.to_string()),
                }
            }
            Err(error) => ToolResponse::error(error),
        },
        Err(response) => response,
    }
}

// 2026-03-23: 杩欓噷鎺ュ叆閫忚鍒嗗彂锛岀洰鐨勬槸鎶?Excel 鐢ㄦ埛鐔熸倝鐨?pivot 鑳藉姏绾冲叆鐜版湁鍗曡〃 Tool 閾惧紡浣撻獙銆?
fn dispatch_pivot_table(args: Value) -> ToolResponse {
    let rows = string_array(&args, "rows");
    let columns = string_array(&args, "columns");
    let values = string_array(&args, "values");
    let Some(aggregation_value) = args.get("aggregation") else {
        return ToolResponse::error("pivot_table 缂哄皯 aggregation 鍙傛暟");
    };
    let aggregation = match serde_json::from_value::<PivotAggregation>(aggregation_value.clone()) {
        Ok(aggregation) => aggregation,
        Err(error) => {
            return ToolResponse::error(format!("pivot_table 鍙傛暟瑙ｆ瀽澶辫触: {error}"));
        }
    };
    let casts = match parse_casts(&args, "casts", "pivot_table") {
        Ok(casts) => casts,
        Err(response) => return response,
    };

    match load_table_for_tool(&args, "pivot_table") {
        Ok(OperationLoad::NeedsConfirmation(response)) => response,
        Ok(OperationLoad::Loaded(loaded)) => match apply_optional_casts(loaded, &casts) {
            Ok(prepared_loaded) => {
                match pivot_table(&prepared_loaded, &rows, &columns, &values, aggregation) {
                    Ok(pivoted) => {
                        respond_with_preview_and_result_ref("pivot_table", &args, &pivoted, 20)
                    }
                    Err(error) => ToolResponse::error(error.to_string()),
                }
            }
            Err(error) => ToolResponse::error(error),
        },
        Err(response) => response,
    }
}

fn dispatch_sort_rows(args: Value) -> ToolResponse {
    let Some(sorts_value) = args.get("sorts") else {
        return ToolResponse::error("sort_rows 缂哄皯 sorts 鍙傛暟");
    };
    let limit = args
        .get("limit")
        .and_then(|value| value.as_u64())
        .unwrap_or(5) as usize;
    let sorts = match serde_json::from_value::<Vec<SortSpec>>(sorts_value.clone()) {
        Ok(sorts) => sorts,
        Err(error) => return ToolResponse::error(format!("sort_rows 鍙傛暟瑙ｆ瀽澶辫触: {error}")),
    };
    let casts = match parse_casts(&args, "casts", "sort_rows") {
        Ok(casts) => casts,
        Err(response) => return response,
    };

    match load_table_for_tool(&args, "sort_rows") {
        Ok(OperationLoad::NeedsConfirmation(response)) => response,
        Ok(OperationLoad::Loaded(loaded)) => match apply_optional_casts(loaded, &casts) {
            Ok(prepared_loaded) => match sort_rows(&prepared_loaded, &sorts) {
                Ok(sorted) => {
                    respond_with_preview_and_result_ref("sort_rows", &args, &sorted, limit)
                }
                Err(error) => ToolResponse::error(error.to_string()),
            },
            Err(error) => ToolResponse::error(error),
        },
        Err(response) => response,
    }
}

fn dispatch_top_n(args: Value) -> ToolResponse {
    let Some(sorts_value) = args.get("sorts") else {
        return ToolResponse::error("top_n 缂哄皯 sorts 鍙傛暟");
    };
    let Some(n) = args.get("n").and_then(|value| value.as_u64()) else {
        return ToolResponse::error("top_n 缂哄皯 n 鍙傛暟");
    };
    let sorts = match serde_json::from_value::<Vec<SortSpec>>(sorts_value.clone()) {
        Ok(sorts) => sorts,
        Err(error) => return ToolResponse::error(format!("top_n 鍙傛暟瑙ｆ瀽澶辫触: {error}")),
    };
    let casts = match parse_casts(&args, "casts", "top_n") {
        Ok(casts) => casts,
        Err(response) => return response,
    };

    match load_table_for_tool(&args, "top_n") {
        Ok(OperationLoad::NeedsConfirmation(response)) => response,
        Ok(OperationLoad::Loaded(loaded)) => match apply_optional_casts(loaded, &casts) {
            Ok(prepared_loaded) => match top_n_rows(&prepared_loaded, &sorts, n as usize) {
                Ok(top_rows) => respond_with_preview_and_result_ref(
                    "top_n",
                    &args,
                    &top_rows,
                    top_rows.dataframe.height(),
                ),
                Err(error) => ToolResponse::error(error.to_string()),
            },
            Err(error) => ToolResponse::error(error),
        },
        Err(response) => response,
    }
}

#[derive(Debug, Deserialize)]
struct ComposeWorkbookWorksheetArg {
    sheet_name: String,
    #[serde(default)]
    format: Option<ExportFormatOptions>,
    source: NestedTableSource,
}

#[derive(Debug, Deserialize)]
struct ReportDeliverySectionArg {
    #[serde(default)]
    sheet_name: Option<String>,
    #[serde(default)]
    format: Option<ExportFormatOptions>,
    source: NestedTableSource,
}

#[derive(Debug, Deserialize)]
struct ReportDeliveryChartArg {
    #[serde(default)]
    chart_ref: Option<String>,
    #[serde(default)]
    chart_type: Option<ReportDeliveryChartType>,
    #[serde(default)]
    title: Option<String>,
    #[serde(default)]
    category_column: Option<String>,
    #[serde(default)]
    value_column: Option<String>,
    #[serde(default)]
    series: Vec<ReportDeliveryChartSeriesArg>,
    #[serde(default)]
    show_legend: bool,
    #[serde(default)]
    legend_position: Option<ReportDeliveryLegendPosition>,
    #[serde(default)]
    chart_style: Option<u8>,
    #[serde(default)]
    x_axis_name: Option<String>,
    #[serde(default)]
    y_axis_name: Option<String>,
    #[serde(default)]
    anchor_row: Option<u32>,
    #[serde(default)]
    anchor_col: Option<u16>,
}

#[derive(Debug, Deserialize)]
struct ReportDeliveryChartSeriesArg {
    value_column: String,
    #[serde(default)]
    name: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ReportDeliveryArgs {
    report_name: Option<String>,
    #[serde(default)]
    report_subtitle: Option<String>,
    summary: ReportDeliverySectionArg,
    analysis: ReportDeliverySectionArg,
    #[serde(default = "default_true")]
    include_chart_sheet: bool,
    #[serde(default)]
    chart_sheet_name: Option<String>,
    #[serde(default)]
    charts: Vec<ReportDeliveryChartArg>,
}

#[derive(Debug, Deserialize)]
struct BuildChartSeriesArg {
    value_column: String,
    #[serde(default)]
    name: Option<String>,
}

#[derive(Debug, Deserialize)]
struct BuildChartArgs {
    source: NestedTableSource,
    chart_type: PersistedChartType,
    #[serde(default)]
    title: Option<String>,
    category_column: String,
    #[serde(default)]
    value_column: Option<String>,
    #[serde(default)]
    series: Vec<BuildChartSeriesArg>,
    #[serde(default)]
    x_axis_name: Option<String>,
    #[serde(default)]
    y_axis_name: Option<String>,
    #[serde(default)]
    show_legend: bool,
    #[serde(default)]
    width: Option<u32>,
    #[serde(default)]
    height: Option<u32>,
}

#[derive(Debug, Deserialize)]
struct ExportChartImageArgs {
    chart_ref: String,
    output_path: String,
}

fn default_true() -> bool {
    true
}

// 2026-03-22: 杩欓噷鎺ュ叆 workbook 鑽夌缁勮鍒嗗彂锛岀洰鐨勬槸鎶婂寮犺〃蹇収瑁呴厤鎴愪竴涓彲澶嶇敤鐨勫 Sheet 浜や粯鍙ユ焺銆?
fn dispatch_compose_workbook(args: Value) -> ToolResponse {
    let Some(worksheets_value) = args.get("worksheets") else {
        return ToolResponse::error("compose_workbook 缺少 worksheets 参数");
    };
    let worksheet_args = match serde_json::from_value::<Vec<ComposeWorkbookWorksheetArg>>(
        worksheets_value.clone(),
    ) {
        Ok(worksheet_args) => worksheet_args,
        Err(error) => {
            return ToolResponse::error(format!(
                "compose_workbook 的 worksheets 参数解析失败: {error}"
            ));
        }
    };

    let mut sheet_inputs = Vec::<WorkbookSheetInput>::with_capacity(worksheet_args.len());
    for worksheet_arg in worksheet_args {
        let loaded = match load_nested_table_source_from_parsed(
            &worksheet_arg.source,
            "compose_workbook",
            &worksheet_arg.sheet_name,
        ) {
            Ok(OperationLoad::NeedsConfirmation(response)) => return response,
            Ok(OperationLoad::Loaded(loaded)) => loaded,
            Err(response) => return response,
        };
        // 2026-03-24: 这里让 compose_workbook 直接复用导出整理规则，原因是基础多表组装入口也需要承接列整理与条件格式声明；目的是避免用户必须绕行 report_delivery 才能拿到完整交付能力。
        let loaded =
            match apply_report_delivery_section_format(loaded, worksheet_arg.format.as_ref()) {
                Ok(loaded) => loaded,
                Err(response) => return response,
            };
        // 2026-03-24: 这里提前冻结 compose_workbook 的 sheet 级导出意图，原因是 workbook_ref 需要把条件格式与数字格式一起带到最终导出层；目的是让低层入口与高层模板入口能力对齐。
        let export_options =
            match build_sheet_export_options(&loaded, worksheet_arg.format.as_ref()) {
                Ok(options) => options,
                Err(error) => return ToolResponse::error(error.to_string()),
            };
        sheet_inputs.push(WorkbookSheetInput {
            sheet_name: worksheet_arg.sheet_name,
            source_refs: source_refs_from_nested_source(&worksheet_arg.source),
            dataframe: loaded.dataframe,
            // 2026-03-24: 这里把 compose_workbook 生成的普通结果页默认标成数据页，原因是导出层需要稳定套用数据页规则；目的是避免 export 再反推页面用途。
            sheet_kind: PersistedWorkbookSheetKind::DataSheet,
            export_options,
            title: None,
            subtitle: None,
            data_start_row: 0,
        });
    }

    let workbook_ref = WorkbookDraftStore::create_workbook_ref();
    let draft = match PersistedWorkbookDraft::from_sheet_inputs(&workbook_ref, sheet_inputs) {
        Ok(draft) => draft,
        Err(error) => return ToolResponse::error(error.to_string()),
    };
    let store = match WorkbookDraftStore::workspace_default() {
        Ok(store) => store,
        Err(error) => return ToolResponse::error(error.to_string()),
    };
    if let Err(error) = store.save(&draft) {
        return ToolResponse::error(error.to_string());
    }
    if let Err(response) =
        sync_output_handle_state(&args, &workbook_ref, "workbook_ref", "compose_workbook")
    {
        return response;
    }

    ToolResponse::ok(json!({
        "workbook_ref": workbook_ref,
        "sheet_count": draft.worksheets.len(),
        "sheet_names": draft
            .worksheets
            .iter()
            .map(|worksheet| worksheet.sheet_name.clone())
            .collect::<Vec<_>>(),
    }))
}

// 2026-03-23: 杩欓噷鎺ュ叆缁撴灉浜や粯妯℃澘鍒嗗彂锛屽師鍥犳槸 V2-P2 闇€瑕佷竴涓嫭绔嬩簬 compose_workbook 鐨勪笂灞傛眹鎶ュ叆鍙ｃ€?
// 2026-03-23: 鐩殑鏄厛鎶娾€滄憳瑕侀〉 / 鍒嗘瀽缁撴灉椤?/ 鍥捐〃椤垫ā鏉?-> workbook_ref鈥濇寮忔矇娣€鎴愮嫭绔?Tool銆?
fn dispatch_report_delivery(args: Value) -> ToolResponse {
    let delivery_args = match serde_json::from_value::<ReportDeliveryArgs>(args.clone()) {
        Ok(delivery_args) => delivery_args,
        Err(error) => {
            // 2026-03-24: 这里收口 report_delivery 的参数解析错误文案，原因是本轮继续扩 chart_ref 桥接输入；目的是避免新入口沿用历史乱码报错。
            return ToolResponse::error(format!("report_delivery 参数解析失败: {error}"));
        }
    };

    let summary_loaded = match load_nested_table_source_from_parsed(
        &delivery_args.summary.source,
        "report_delivery",
        "summary",
    ) {
        Ok(OperationLoad::NeedsConfirmation(response)) => return response,
        Ok(OperationLoad::Loaded(loaded)) => loaded,
        Err(response) => return response,
    };
    // 2026-03-24: 这里允许 report_delivery 直接承接 summary 段的导出整理规则，原因是结果交付层不该强迫上层先手工串一次 format_table_for_export；目的是把“整理 -> 交付”压进单个高层入口。
    let summary_loaded = match apply_report_delivery_section_format(
        summary_loaded,
        delivery_args.summary.format.as_ref(),
    ) {
        Ok(loaded) => loaded,
        Err(response) => return response,
    };
    let analysis_loaded = match load_nested_table_source_from_parsed(
        &delivery_args.analysis.source,
        "report_delivery",
        "analysis",
    ) {
        Ok(OperationLoad::NeedsConfirmation(response)) => return response,
        Ok(OperationLoad::Loaded(loaded)) => loaded,
        Err(response) => return response,
    };
    // 2026-03-24: 这里同样给 analysis 段接入复用格式整理，原因是分析结果页常常需要在交付前重排列顺序和改业务列名；目的是减少上层 Skill 的样板编排。
    let analysis_loaded = match apply_report_delivery_section_format(
        analysis_loaded,
        delivery_args.analysis.format.as_ref(),
    ) {
        Ok(loaded) => loaded,
        Err(response) => return response,
    };
    let summary_source_refs = source_refs_from_nested_source(&delivery_args.summary.source);
    let analysis_source_refs = source_refs_from_nested_source(&delivery_args.analysis.source);
    let charts = match resolve_report_delivery_charts(
        delivery_args.charts,
        &analysis_loaded.dataframe,
        &analysis_source_refs,
    ) {
        Ok(charts) => charts,
        Err(response) => return response,
    };

    let workbook_ref = WorkbookDraftStore::create_workbook_ref();
    let report_name = delivery_args
        .report_name
        .unwrap_or_else(|| "标准分析汇报".to_string());
    // 2026-03-24: 这里先把段级导出意图单独算出来，原因是 dispatch_report_delivery 返回 ToolResponse，不能在 struct literal 里直接用 `?`；
    // 目的是保持错误出口稳定，同时避免对 loaded 发生先 move 后借用。
    let summary_export_options =
        match build_sheet_export_options(&summary_loaded, delivery_args.summary.format.as_ref()) {
            Ok(options) => options,
            Err(error) => return ToolResponse::error(error.to_string()),
        };
    // 2026-03-24: 这里同样提前计算 analysis 段的导出意图，原因是要让 currency / percent 规则在 move dataframe 之前完成校验；
    // 目的是保证 report_delivery 草稿组装时既能通过编译，也能保留列级格式元数据。
    let analysis_export_options = match build_sheet_export_options(
        &analysis_loaded,
        delivery_args.analysis.format.as_ref(),
    ) {
        Ok(options) => options,
        Err(error) => return ToolResponse::error(error.to_string()),
    };
    let draft = match build_report_delivery_draft(
        &workbook_ref,
        ReportDeliveryRequest {
            report_name: report_name.clone(),
            report_subtitle: delivery_args.report_subtitle,
            summary: ReportDeliverySection {
                sheet_name: delivery_args
                    .summary
                    .sheet_name
                    .unwrap_or_else(|| "摘要页".to_string()),
                source_refs: summary_source_refs,
                dataframe: summary_loaded.dataframe,
                export_options: summary_export_options,
            },
            analysis: ReportDeliverySection {
                sheet_name: delivery_args
                    .analysis
                    .sheet_name
                    .unwrap_or_else(|| "分析结果页".to_string()),
                source_refs: analysis_source_refs,
                dataframe: analysis_loaded.dataframe,
                export_options: analysis_export_options,
            },
            include_chart_sheet: delivery_args.include_chart_sheet,
            chart_sheet_name: delivery_args
                .chart_sheet_name
                .unwrap_or_else(|| "图表页".to_string()),
            charts,
        },
    ) {
        Ok(draft) => draft,
        Err(error) => return ToolResponse::error(error.to_string()),
    };

    let store = match WorkbookDraftStore::workspace_default() {
        Ok(store) => store,
        Err(error) => return ToolResponse::error(error.to_string()),
    };
    if let Err(error) = store.save(&draft) {
        return ToolResponse::error(error.to_string());
    }
    if let Err(response) =
        sync_output_handle_state(&args, &workbook_ref, "workbook_ref", "report_delivery")
    {
        return response;
    }

    ToolResponse::ok(json!({
        "workbook_ref": workbook_ref,
        "report_name": report_name,
        "template": "standard_report_v2",
        "sheet_count": draft.worksheets.len(),
        "chart_count": draft.charts.len(),
        "sheet_names": draft
            .worksheets
            .iter()
            .map(|worksheet| worksheet.sheet_name.clone())
            .collect::<Vec<_>>(),
    }))
}

// 2026-03-24: 这里把 report_delivery 段内的格式整理收口成单独辅助函数，原因是 summary / analysis 两段都要复用同一条规则；目的是保持 dispatcher 内部装配逻辑符合 SRP，避免重复分支。
fn apply_report_delivery_section_format(
    loaded: LoadedTable,
    format: Option<&ExportFormatOptions>,
) -> Result<LoadedTable, ToolResponse> {
    let Some(format) = format else {
        return Ok(loaded);
    };
    format_table_for_export(&loaded, format).map_err(|error| ToolResponse::error(error.to_string()))
}

// 2026-03-24: 这里把可跨请求持久化的导出意图从段内 format 中抽出来，原因是列整理先落到 DataFrame，而数字格式等规则还要进入 workbook_ref；
// 目的是让 export_excel_workbook 直接消费稳定草稿，而不是回看上层请求体。
fn build_sheet_export_options(
    loaded: &LoadedTable,
    format: Option<&ExportFormatOptions>,
) -> Result<
    Option<PersistedWorkbookSheetExportOptions>,
    crate::ops::format_table_for_export::FormatTableForExportError,
> {
    let Some(format) = format else {
        return Ok(None);
    };

    let number_formats = normalize_sheet_number_formats(loaded, &format.number_formats)?;
    let conditional_formats =
        normalize_sheet_conditional_formats(loaded, &format.conditional_formats)?;
    if number_formats.is_empty() && conditional_formats.is_empty() {
        return Ok(None);
    }

    Ok(Some(PersistedWorkbookSheetExportOptions {
        number_formats,
        conditional_formats,
    }))
}

// 2026-03-24: 这里校验数字格式规则仍然指向格式整理后的可见列，原因是列顺序和列名可能已在 format_table_for_export 中被调整；
// 目的是避免 workbook 草稿写入无效列规则，直到导出阶段才报晚错。
fn normalize_sheet_number_formats(
    loaded: &LoadedTable,
    rules: &[PersistedWorkbookColumnNumberFormatRule],
) -> Result<
    Vec<PersistedWorkbookColumnNumberFormatRule>,
    crate::ops::format_table_for_export::FormatTableForExportError,
> {
    let mut normalized = Vec::with_capacity(rules.len());
    for rule in rules {
        if loaded.dataframe.column(&rule.column).is_err() {
            return Err(
                crate::ops::format_table_for_export::FormatTableForExportError::MissingColumn(
                    rule.column.clone(),
                ),
            );
        }
        normalized.push(rule.clone());
    }
    Ok(normalized)
}

// 2026-03-24: 这里校验条件格式规则仍然指向整理后的可见列，原因是 report_delivery 段内可能已经做过重命名或裁剪；目的是避免导出阶段才发现规则挂在不存在的列上。
fn normalize_sheet_conditional_formats(
    loaded: &LoadedTable,
    rules: &[PersistedWorkbookColumnConditionalFormatRule],
) -> Result<
    Vec<PersistedWorkbookColumnConditionalFormatRule>,
    crate::ops::format_table_for_export::FormatTableForExportError,
> {
    let mut normalized = Vec::with_capacity(rules.len());
    for rule in rules {
        let column = if requires_single_conditional_column(rule) {
            Some(resolve_conditional_column(loaded, &rule.column)?)
        } else {
            None
        };
        // 2026-03-24: 这里在进入 workbook 草稿前先校验条件格式参数，原因是阈值类规则缺少 threshold 或挂到非数值列上时，导出阶段报错会过晚；目的是把错误前移到 Tool 调用边界。
        match rule.kind {
            crate::frame::workbook_ref_store::PersistedWorkbookConditionalFormatKind::NegativeRed
            | crate::frame::workbook_ref_store::PersistedWorkbookConditionalFormatKind::NullWarning
            | crate::frame::workbook_ref_store::PersistedWorkbookConditionalFormatKind::DuplicateWarn => {}
            crate::frame::workbook_ref_store::PersistedWorkbookConditionalFormatKind::HighValueHighlight
            | crate::frame::workbook_ref_store::PersistedWorkbookConditionalFormatKind::PercentLowWarn => {
                let threshold = rule.threshold.as_ref().ok_or_else(|| {
                    crate::ops::format_table_for_export::FormatTableForExportError::BuildFrame(
                        format!("conditional_format `{}` 缺少 threshold", rule.column),
                    )
                })?;
                threshold.parse::<f64>().map_err(|_| {
                    crate::ops::format_table_for_export::FormatTableForExportError::BuildFrame(
                        format!("conditional_format `{}` 的 threshold 不是合法数字", rule.column),
                    )
                })?;
                if !column.expect("column checked").dtype().is_primitive_numeric() {
                    return Err(
                        crate::ops::format_table_for_export::FormatTableForExportError::BuildFrame(
                            format!("conditional_format `{}` 只能用于数值列", rule.column),
                        ),
                    );
                }
            }
            crate::frame::workbook_ref_store::PersistedWorkbookConditionalFormatKind::BetweenWarn => {
                let min_threshold = rule.min_threshold.as_ref().ok_or_else(|| {
                    crate::ops::format_table_for_export::FormatTableForExportError::BuildFrame(
                        format!("conditional_format `{}` 缺少 min_threshold", rule.column),
                    )
                })?;
                let max_threshold = rule.max_threshold.as_ref().ok_or_else(|| {
                    crate::ops::format_table_for_export::FormatTableForExportError::BuildFrame(
                        format!("conditional_format `{}` 缺少 max_threshold", rule.column),
                    )
                })?;
                min_threshold.parse::<f64>().map_err(|_| {
                    crate::ops::format_table_for_export::FormatTableForExportError::BuildFrame(
                        format!("conditional_format `{}` 的 min_threshold 不是合法数字", rule.column),
                    )
                })?;
                max_threshold.parse::<f64>().map_err(|_| {
                    crate::ops::format_table_for_export::FormatTableForExportError::BuildFrame(
                        format!("conditional_format `{}` 的 max_threshold 不是合法数字", rule.column),
                    )
                })?;
                if !column.expect("column checked").dtype().is_primitive_numeric() {
                    return Err(
                        crate::ops::format_table_for_export::FormatTableForExportError::BuildFrame(
                            format!("conditional_format `{}` 只能用于数值列", rule.column),
                        ),
                    );
                }
            }
            crate::frame::workbook_ref_store::PersistedWorkbookConditionalFormatKind::CompositeDuplicateWarn => {
                if rule.columns.len() < 2 {
                    return Err(
                        crate::ops::format_table_for_export::FormatTableForExportError::BuildFrame(
                            "composite_duplicate_warn 至少需要两个 columns".to_string(),
                        ),
                    );
                }
                for column_name in &rule.columns {
                    let _ = resolve_conditional_column(loaded, column_name)?;
                }
            }
        }
        normalized.push(rule.clone());
    }
    Ok(normalized)
}

// 2026-03-25: 这里把“是否需要单列入口”单独判断，原因是复合键判重会改走多列校验分支；目的是让单列规则和多列规则的边界更清晰。
fn requires_single_conditional_column(rule: &PersistedWorkbookColumnConditionalFormatRule) -> bool {
    !matches!(
        rule.kind,
        crate::frame::workbook_ref_store::PersistedWorkbookConditionalFormatKind::CompositeDuplicateWarn
    )
}

// 2026-03-25: 这里统一解析条件格式引用列，原因是多个规则分支都需要做同一套缺列校验；目的是避免错误文案和校验边界分散。
fn resolve_conditional_column<'a>(
    loaded: &'a LoadedTable,
    column_name: &str,
) -> Result<
    &'a polars::prelude::Column,
    crate::ops::format_table_for_export::FormatTableForExportError,
> {
    loaded.dataframe.column(column_name).map_err(|_| {
        crate::ops::format_table_for_export::FormatTableForExportError::MissingColumn(
            column_name.to_string(),
        )
    })
}

fn resolve_report_delivery_charts(
    chart_args: Vec<ReportDeliveryChartArg>,
    analysis_dataframe: &polars::prelude::DataFrame,
    analysis_source_refs: &[String],
) -> Result<Vec<ReportDeliveryChart>, ToolResponse> {
    chart_args
        .into_iter()
        .map(|chart| resolve_report_delivery_chart(chart, analysis_dataframe, analysis_source_refs))
        .collect()
}

fn resolve_report_delivery_chart(
    chart: ReportDeliveryChartArg,
    analysis_dataframe: &polars::prelude::DataFrame,
    analysis_source_refs: &[String],
) -> Result<ReportDeliveryChart, ToolResponse> {
    if let Some(chart_ref) = chart.chart_ref.as_ref() {
        let store = ChartDraftStore::workspace_default()
            .map_err(|error| ToolResponse::error(error.to_string()))?;
        let draft = store
            .load(chart_ref)
            .map_err(|error| ToolResponse::error(error.to_string()))?;
        if !chart_ref_matches_analysis(&draft, analysis_dataframe, analysis_source_refs) {
            return Err(ToolResponse::error(
                "report_delivery 的 chart_ref 与 analysis 数据不一致",
            ));
        }
        return Ok(chart_ref_to_report_delivery_chart(&draft));
    }

    let chart_type = chart
        .chart_type
        .ok_or_else(|| ToolResponse::error("report_delivery 的 chart_type 不能为空"))?;
    Ok(ReportDeliveryChart {
        chart_ref: None,
        source_refs: vec![],
        chart_type,
        title: chart.title,
        category_column: chart.category_column.unwrap_or_default(),
        value_column: chart.value_column.unwrap_or_default(),
        series: chart
            .series
            .into_iter()
            .map(|series| ReportDeliveryChartSeries {
                value_column: series.value_column,
                name: series.name,
            })
            .collect(),
        show_legend: chart.show_legend,
        legend_position: chart.legend_position,
        chart_style: chart.chart_style,
        x_axis_name: chart.x_axis_name,
        y_axis_name: chart.y_axis_name,
        anchor_row: chart.anchor_row,
        anchor_col: chart.anchor_col,
    })
}

fn chart_ref_matches_analysis(
    draft: &PersistedChartDraft,
    analysis_dataframe: &polars::prelude::DataFrame,
    analysis_source_refs: &[String],
) -> bool {
    if analysis_dataframe.column(&draft.category_column).is_err() {
        return false;
    }
    if draft
        .series
        .iter()
        .any(|item| analysis_dataframe.column(&item.value_column).is_err())
    {
        return false;
    }
    if draft
        .source_refs
        .iter()
        .any(|source_ref| analysis_source_refs.iter().any(|item| item == source_ref))
    {
        return true;
    }
    draft.dataset.row_count == analysis_dataframe.height()
}
fn dispatch_build_chart(args: Value) -> ToolResponse {
    let chart_args = match serde_json::from_value::<BuildChartArgs>(args.clone()) {
        Ok(chart_args) => chart_args,
        Err(error) => {
            // 2026-03-24: 这里收口 build_chart 的参数解析错误文案，原因是本轮继续扩展图表桥接链路；目的是避免历史乱码继续扩散到图表入口。
            return ToolResponse::error(format!("build_chart 参数解析失败: {error}"));
        }
    };

    let loaded =
        match load_nested_table_source_from_parsed(&chart_args.source, "build_chart", "source") {
            Ok(OperationLoad::NeedsConfirmation(response)) => return response,
            Ok(OperationLoad::Loaded(loaded)) => loaded,
            Err(response) => return response,
        };

    let mut series = chart_args
        .series
        .into_iter()
        .map(|item| PersistedChartSeriesSpec {
            value_column: item.value_column,
            name: item.name,
        })
        .collect::<Vec<_>>();
    if series.is_empty() {
        if let Some(value_column) = chart_args.value_column.as_ref() {
            if !value_column.trim().is_empty() {
                series.push(PersistedChartSeriesSpec {
                    value_column: value_column.clone(),
                    name: None,
                });
            }
        }
    }
    if series.is_empty() {
        // 2026-03-24: 这里明确要求至少一个数值系列，原因是图表不能在没有 Y 值的情况下继续生成；目的是让上层尽早补问而不是导出空图。
        return ToolResponse::error("build_chart 至少需要一个数值系列");
    }

    let chart_ref = ChartDraftStore::create_chart_ref();
    let draft = match PersistedChartDraft::from_dataframe_with_layout(
        &chart_ref,
        "build_chart",
        source_refs_from_nested_source(&chart_args.source),
        &loaded.dataframe,
        chart_args.chart_type.clone(),
        chart_args.title.clone(),
        &chart_args.category_column,
        chart_args.x_axis_name.clone(),
        chart_args.y_axis_name.clone(),
        chart_args.show_legend,
        chart_args.width.unwrap_or(900),
        chart_args.height.unwrap_or(520),
        series.clone(),
    ) {
        Ok(draft) => draft,
        Err(error) => return ToolResponse::error(error.to_string()),
    };

    let store = match ChartDraftStore::workspace_default() {
        Ok(store) => store,
        Err(error) => return ToolResponse::error(error.to_string()),
    };
    if let Err(error) = store.save(&draft) {
        return ToolResponse::error(error.to_string());
    }
    if let Err(response) = sync_output_handle_state(&args, &chart_ref, "chart_ref", "build_chart") {
        return response;
    }

    ToolResponse::ok(json!({
        "chart_ref": chart_ref,
        "chart_type": serde_json::to_value(&chart_args.chart_type).unwrap_or_else(|_| Value::String("unknown".to_string())),
        "title": chart_args.title,
        "category_column": chart_args.category_column,
        "series_count": series.len(),
        "row_count": loaded.dataframe.height(),
    }))
}
fn dispatch_export_chart_image(args: Value) -> ToolResponse {
    let export_args = match serde_json::from_value::<ExportChartImageArgs>(args.clone()) {
        Ok(export_args) => export_args,
        Err(error) => {
            return ToolResponse::error(format!("export_chart_image 鍙傛暟瑙ｆ瀽澶辫触: {error}"));
        }
    };

    if export_args.chart_ref.trim().is_empty() {
        // 2026-03-24: 这里收口当前图表导出入口的缺参提示，原因是本轮正在打通 chart_ref 到交付层；目的是让图表桥接相关错误对上层更可读。
        return ToolResponse::error("export_chart_image 缺少 chart_ref 参数");
    }
    if export_args.output_path.trim().is_empty() {
        return ToolResponse::error("export_chart_image 缺少 output_path 参数");
    }
    if !export_args
        .output_path
        .to_ascii_lowercase()
        .ends_with(".svg")
    {
        // 2026-03-24: 这里把导出格式继续收口为 SVG，原因是当前纯 Rust 二进制链路里 SVG 最稳定；目的是先保证图表交付闭环，不引入额外运行时依赖。
        return ToolResponse::error("export_chart_image 目前只支持导出 svg");
    }

    let store = match ChartDraftStore::workspace_default() {
        Ok(store) => store,
        Err(error) => return ToolResponse::error(error.to_string()),
    };
    let draft = match store.load(&export_args.chart_ref) {
        Ok(draft) => draft,
        Err(error) => return ToolResponse::error(error.to_string()),
    };
    let svg = match render_chart_svg(&draft) {
        Ok(svg) => svg,
        Err(error) => return ToolResponse::error(error.to_string()),
    };

    if let Some(parent) = Path::new(&export_args.output_path).parent() {
        if let Err(error) = fs::create_dir_all(parent) {
            return ToolResponse::error(format!("无法创建图表导出目录: {error}"));
        }
    }
    if let Err(error) = fs::write(&export_args.output_path, svg.as_bytes()) {
        return ToolResponse::error(format!("无法写出图表 SVG: {error}"));
    }
    if let Err(response) = sync_output_handle_state(
        &args,
        &export_args.chart_ref,
        "chart_ref",
        "export_chart_image",
    ) {
        return response;
    }

    ToolResponse::ok(json!({
        "chart_ref": export_args.chart_ref,
        "output_path": export_args.output_path,
        "format": "svg",
    }))
}

fn dispatch_export_csv(args: Value) -> ToolResponse {
    let Some(output_path) = args.get("output_path").and_then(|value| value.as_str()) else {
        return ToolResponse::error("export_csv 缂哄皯 output_path 鍙傛暟");
    };

    match load_table_for_tool(&args, "export_csv") {
        Ok(OperationLoad::NeedsConfirmation(response)) => response,
        Ok(OperationLoad::Loaded(loaded)) => match export_csv(&loaded, output_path) {
            Ok(()) => ToolResponse::ok(json!({
                "output_path": output_path,
                "row_count": loaded.dataframe.height(),
                "column_count": loaded.dataframe.width(),
                "format": "csv",
            })),
            Err(error) => ToolResponse::error(error.to_string()),
        },
        Err(response) => response,
    }
}

fn dispatch_export_excel(args: Value) -> ToolResponse {
    let Some(output_path) = args.get("output_path").and_then(|value| value.as_str()) else {
        return ToolResponse::error("export_excel 缂哄皯 output_path 鍙傛暟");
    };
    let sheet_name = args
        .get("sheet_name")
        .and_then(|value| value.as_str())
        .unwrap_or("Report");

    match load_table_for_tool(&args, "export_excel") {
        Ok(OperationLoad::NeedsConfirmation(response)) => response,
        Ok(OperationLoad::Loaded(loaded)) => match export_excel(&loaded, output_path, sheet_name) {
            Ok(()) => ToolResponse::ok(json!({
                "output_path": output_path,
                "sheet_name": sheet_name,
                "row_count": loaded.dataframe.height(),
                "column_count": loaded.dataframe.width(),
                "format": "xlsx",
            })),
            Err(error) => ToolResponse::error(error.to_string()),
        },
        Err(response) => response,
    }
}

// 2026-03-22: 杩欓噷鎺ュ叆澶?Sheet workbook 瀵煎嚭鍒嗗彂锛岀洰鐨勬槸璁?compose_workbook 浜у嚭鐨?workbook_ref 鍙互鐪熸钀芥垚 xlsx 鏂囦欢銆?
fn dispatch_export_excel_workbook(args: Value) -> ToolResponse {
    let Some(workbook_ref) = args.get("workbook_ref").and_then(|value| value.as_str()) else {
        return ToolResponse::error("export_excel_workbook 缂哄皯 workbook_ref 鍙傛暟");
    };
    let Some(output_path) = args.get("output_path").and_then(|value| value.as_str()) else {
        return ToolResponse::error("export_excel_workbook 缂哄皯 output_path 鍙傛暟");
    };

    let store = match WorkbookDraftStore::workspace_default() {
        Ok(store) => store,
        Err(error) => return ToolResponse::error(error.to_string()),
    };
    let draft = match store.load(workbook_ref) {
        Ok(draft) => draft,
        Err(error) => return ToolResponse::error(error.to_string()),
    };

    match export_excel_workbook(&draft, output_path) {
        Ok(()) => ToolResponse::ok(json!({
            "workbook_ref": workbook_ref,
            "output_path": output_path,
            "sheet_count": draft.worksheets.len(),
            "sheet_names": draft
                .worksheets
                .iter()
                .map(|worksheet| worksheet.sheet_name.clone())
                .collect::<Vec<_>>(),
            "format": "xlsx",
        })),
        Err(error) => ToolResponse::error(error.to_string()),
    }
}

fn dispatch_join_tables(args: Value) -> ToolResponse {
    let Some(left_value) = args.get("left") else {
        return ToolResponse::error("join_tables 缺少 left 参数");
    };
    let Some(right_value) = args.get("right") else {
        return ToolResponse::error("join_tables 缺少 right 参数");
    };
    let Some(left_on) = args.get("left_on").and_then(|value| value.as_str()) else {
        return ToolResponse::error("join_tables 缺少 left_on 参数");
    };
    let Some(right_on) = args.get("right_on").and_then(|value| value.as_str()) else {
        return ToolResponse::error("join_tables 缺少 right_on 参数");
    };
    let limit = args
        .get("limit")
        .and_then(|value| value.as_u64())
        .unwrap_or(5) as usize;

    let keep_mode = match args.get("keep_mode") {
        Some(mode_value) => match serde_json::from_value::<JoinKeepMode>(mode_value.clone()) {
            Ok(mode) => mode,
            Err(error) => {
                return ToolResponse::error(format!(
                    "join_tables 鐨?keep_mode 鍙傛暟瑙ｆ瀽澶辫触: {error}"
                ));
            }
        },
        None => JoinKeepMode::MatchedOnly,
    };
    let left_casts = match parse_casts(&args, "left_casts", "join_tables") {
        Ok(casts) => casts,
        Err(response) => return response,
    };
    let right_casts = match parse_casts(&args, "right_casts", "join_tables") {
        Ok(casts) => casts,
        Err(response) => return response,
    };

    // 2026-03-22: 杩欓噷鎶?join 鐨勫乏鍙宠緭鍏ョ粺涓€鏀跺彛鍒板祵濂楁潵婧愯В鏋愬櫒锛岀洰鐨勬槸璁╂樉鎬у叧鑱旀棦鑳藉悆鍘熷琛紝涔熻兘鐩存帴鍚?table_ref/result_ref銆?
    match load_nested_table_source(left_value, "join_tables", "left") {
        Ok(OperationLoad::NeedsConfirmation(response)) => response,
        Ok(OperationLoad::Loaded(left_loaded)) => {
            match load_nested_table_source(right_value, "join_tables", "right") {
                Ok(OperationLoad::NeedsConfirmation(response)) => response,
                Ok(OperationLoad::Loaded(right_loaded)) => {
                    let prepared_left = apply_optional_casts(left_loaded, &left_casts);
                    let prepared_right = apply_optional_casts(right_loaded, &right_casts);

                    match (prepared_left, prepared_right) {
                        (Ok(prepared_left), Ok(prepared_right)) => {
                            match join_tables(
                                &prepared_left,
                                &prepared_right,
                                left_on,
                                right_on,
                                keep_mode,
                            ) {
                                Ok(joined) => respond_with_preview_and_result_ref(
                                    "join_tables",
                                    &args,
                                    &joined,
                                    limit,
                                ),
                                Err(error) => ToolResponse::error(error.to_string()),
                            }
                        }
                        (Err(error), _) | (_, Err(error)) => ToolResponse::error(error),
                    }
                }
                Err(response) => response,
            }
        }
        Err(response) => response,
    }
}

// 2026-03-23: 杩欓噷鎺ュ叆鏂囨湰鏍囧噯鍖栧垎鍙戯紝鐩殑鏄妸 join / lookup 鍓嶇殑鏂囨湰娓呮礂娌夋穩鎴愮粺涓€鍗曡〃 Tool 鍏ュ彛銆?
fn dispatch_normalize_text_columns(args: Value) -> ToolResponse {
    let Some(rules_value) = args.get("rules") else {
        return ToolResponse::error("normalize_text_columns 缂哄皯 rules 鍙傛暟");
    };
    let rules = match serde_json::from_value::<Vec<NormalizeTextRule>>(rules_value.clone()) {
        Ok(rules) => rules,
        Err(error) => {
            return ToolResponse::error(format!(
                "normalize_text_columns 鍙傛暟瑙ｆ瀽澶辫触: {error}"
            ));
        }
    };

    match load_table_for_tool(&args, "normalize_text_columns") {
        Ok(OperationLoad::NeedsConfirmation(response)) => response,
        Ok(OperationLoad::Loaded(loaded)) => match normalize_text_columns(&loaded, &rules) {
            Ok(normalized) => {
                respond_with_preview_and_result_ref("normalize_text_columns", &args, &normalized, 5)
            }
            Err(error) => ToolResponse::error(error.to_string()),
        },
        Err(response) => response,
    }
}

// 2026-03-23: 杩欓噷鎺ュ叆鏃ユ湡鏃堕棿鏍囧噯鍖栧垎鍙戯紝鐩殑鏄鏃堕棿鍒楁竻娲楃户缁部鐢ㄧ幇鏈?result_ref 閾惧紡澶嶇敤浣撻獙銆?
fn dispatch_parse_datetime_columns(args: Value) -> ToolResponse {
    let Some(rules_value) = args.get("rules") else {
        return ToolResponse::error("parse_datetime_columns 缂哄皯 rules 鍙傛暟");
    };
    let rules = match serde_json::from_value::<Vec<ParseDateTimeRule>>(rules_value.clone()) {
        Ok(rules) => rules,
        Err(error) => {
            return ToolResponse::error(format!(
                "parse_datetime_columns 鍙傛暟瑙ｆ瀽澶辫触: {error}"
            ));
        }
    };

    match load_table_for_tool(&args, "parse_datetime_columns") {
        Ok(OperationLoad::NeedsConfirmation(response)) => response,
        Ok(OperationLoad::Loaded(loaded)) => match parse_datetime_columns(&loaded, &rules) {
            Ok(parsed) => {
                respond_with_preview_and_result_ref("parse_datetime_columns", &args, &parsed, 20)
            }
            Err(error) => ToolResponse::error(error.to_string()),
        },
        Err(response) => response,
    }
}

// 2026-03-23: 杩欓噷鎺ュ叆杞婚噺鏌ュ€煎垎鍙戯紝鐩殑鏄涓昏〃涓嶅彉琛岀殑甯﹀垪鍦烘櫙娌跨敤鐜版湁鍙屾潵婧愪笌 result_ref 閾惧紡浣撻獙銆?
fn dispatch_lookup_values(args: Value) -> ToolResponse {
    let Some(base_value) = args.get("base") else {
        return ToolResponse::error("lookup_values 缂哄皯 base 鍙傛暟");
    };
    let Some(lookup_value) = args.get("lookup") else {
        return ToolResponse::error("lookup_values 缂哄皯 lookup 鍙傛暟");
    };
    let base_keys = match parse_lookup_key_args(&args, "base_on", "base_keys", "lookup_values") {
        Ok(keys) => keys,
        Err(response) => return response,
    };
    let lookup_keys =
        match parse_lookup_key_args(&args, "lookup_on", "lookup_keys", "lookup_values") {
            Ok(keys) => keys,
            Err(response) => return response,
        };
    let Some(selects_value) = args.get("selects") else {
        return ToolResponse::error("lookup_values 缂哄皯 selects 鍙傛暟");
    };
    let selects = match serde_json::from_value::<Vec<LookupSelect>>(selects_value.clone()) {
        Ok(selects) => selects,
        Err(error) => {
            return ToolResponse::error(format!("lookup_values 鍙傛暟瑙ｆ瀽澶辫触: {error}"));
        }
    };

    match load_nested_table_source(base_value, "lookup_values", "base") {
        Ok(OperationLoad::NeedsConfirmation(response)) => response,
        Ok(OperationLoad::Loaded(base_loaded)) => {
            match load_nested_table_source(lookup_value, "lookup_values", "lookup") {
                Ok(OperationLoad::NeedsConfirmation(response)) => response,
                Ok(OperationLoad::Loaded(lookup_loaded)) => {
                    let base_key_refs = base_keys.iter().map(String::as_str).collect::<Vec<_>>();
                    let lookup_key_refs =
                        lookup_keys.iter().map(String::as_str).collect::<Vec<_>>();
                    match lookup_values_by_keys(
                        &base_loaded,
                        &lookup_loaded,
                        &base_key_refs,
                        &lookup_key_refs,
                        &selects,
                    ) {
                        Ok(looked_up) => respond_with_preview_and_result_ref(
                            "lookup_values",
                            &args,
                            &looked_up,
                            20,
                        ),
                        Err(error) => ToolResponse::error(error.to_string()),
                    }
                }
                Err(response) => response,
            }
        }
        Err(response) => response,
    }
}

// 2026-03-23: 杩欓噷鎺ュ叆绐楀彛璁＄畻鍒嗗彂锛岀洰鐨勬槸璁╃粍鍐呭簭鍙枫€佹帓鍚嶅拰绱鍊兼部鐢ㄥ崟琛?Tool 鐨勭粺涓€杈撳叆涓?result_ref 浣撻獙銆?
fn dispatch_window_calculation(args: Value) -> ToolResponse {
    let Some(order_by_value) = args.get("order_by") else {
        return ToolResponse::error("window_calculation 缂哄皯 order_by 鍙傛暟");
    };
    let Some(calculations_value) = args.get("calculations") else {
        return ToolResponse::error("window_calculation 缂哄皯 calculations 鍙傛暟");
    };
    let partition_by = string_array(&args, "partition_by");
    let order_by = match serde_json::from_value::<Vec<WindowOrderSpec>>(order_by_value.clone()) {
        Ok(order_by) => order_by,
        Err(error) => {
            return ToolResponse::error(format!("window_calculation 鍙傛暟瑙ｆ瀽澶辫触: {error}"));
        }
    };
    let calculations =
        match serde_json::from_value::<Vec<WindowCalculation>>(calculations_value.clone()) {
            Ok(calculations) => calculations,
            Err(error) => {
                return ToolResponse::error(format!(
                    "window_calculation 鍙傛暟瑙ｆ瀽澶辫触: {error}"
                ));
            }
        };
    let casts = match parse_casts(&args, "casts", "window_calculation") {
        Ok(casts) => casts,
        Err(response) => return response,
    };

    match load_table_for_tool(&args, "window_calculation") {
        Ok(OperationLoad::NeedsConfirmation(response)) => response,
        Ok(OperationLoad::Loaded(loaded)) => match apply_optional_casts(loaded, &casts) {
            Ok(prepared_loaded) => {
                match window_calculation(&prepared_loaded, &partition_by, &order_by, &calculations)
                {
                    Ok(calculated) => respond_with_preview_and_result_ref(
                        "window_calculation",
                        &args,
                        &calculated,
                        20,
                    ),
                    Err(error) => ToolResponse::error(error.to_string()),
                }
            }
            Err(error) => ToolResponse::error(error),
        },
        Err(response) => response,
    }
}

// 2026-03-23: 杩欓噷鎺ュ叆鍒楁敼鍚嶅垎鍙戯紝鐩殑鏄瀛楁鍙ｅ緞缁熶竴缁х画娌跨敤鐜版湁 result_ref 閾惧紡澶嶇敤浣撻獙銆?
fn dispatch_rename_columns(args: Value) -> ToolResponse {
    let Some(mappings_value) = args.get("mappings") else {
        return ToolResponse::error("rename_columns 缂哄皯 mappings 鍙傛暟");
    };
    let mappings = match serde_json::from_value::<Vec<RenameColumnMapping>>(mappings_value.clone())
    {
        Ok(mappings) => mappings,
        Err(error) => {
            return ToolResponse::error(format!("rename_columns 鍙傛暟瑙ｆ瀽澶辫触: {error}"));
        }
    };

    match load_table_for_tool(&args, "rename_columns") {
        Ok(OperationLoad::NeedsConfirmation(response)) => response,
        Ok(OperationLoad::Loaded(loaded)) => match rename_columns(&loaded, &mappings) {
            Ok(renamed) => {
                respond_with_preview_and_result_ref("rename_columns", &args, &renamed, 5)
            }
            Err(error) => ToolResponse::error(error.to_string()),
        },
        Err(response) => response,
    }
}

// 2026-03-22: 杩欓噷鎺ュ叆閫氱敤琛ョ┖鍒嗗彂锛岀洰鐨勬槸璁╁父閲忚ˉ绌恒€佽ˉ闆跺拰鍓嶅€煎～琛ユ部鐢ㄧ幇鏈夊崟琛?Tool 閾惧紡浣撻獙銆?
fn dispatch_fill_missing_values(args: Value) -> ToolResponse {
    let Some(rules_value) = args.get("rules") else {
        return ToolResponse::error("fill_missing_values 缂哄皯 rules 鍙傛暟");
    };
    let rules = match serde_json::from_value::<Vec<FillMissingRule>>(rules_value.clone()) {
        Ok(rules) => rules,
        Err(error) => {
            return ToolResponse::error(format!("fill_missing_values 鍙傛暟瑙ｆ瀽澶辫触: {error}"));
        }
    };

    match load_table_for_tool(&args, "fill_missing_values") {
        Ok(OperationLoad::NeedsConfirmation(response)) => response,
        Ok(OperationLoad::Loaded(loaded)) => match fill_missing_values(&loaded, &rules) {
            Ok(filled) => {
                respond_with_preview_and_result_ref("fill_missing_values", &args, &filled, 20)
            }
            Err(error) => ToolResponse::error(error.to_string()),
        },
        Err(response) => response,
    }
}

// 2026-03-22: 杩欓噷鎺ュ叆閫氱敤鍘婚噸鍒嗗彂锛岀洰鐨勬槸璁╂暣琛屽幓閲嶅拰鎸夊瓙闆嗗垪鍘婚噸娌跨敤鐜版湁鍗曡〃 Tool 鐨勯摼寮忎綋楠屻€?
fn dispatch_distinct_rows(args: Value) -> ToolResponse {
    let subset = string_array(&args, "subset");
    let keep = match args.get("keep") {
        Some(value) => match serde_json::from_value::<DistinctKeep>(value.clone()) {
            Ok(keep) => keep,
            Err(error) => {
                return ToolResponse::error(format!("distinct_rows 鍙傛暟瑙ｆ瀽澶辫触: {error}"));
            }
        },
        None => DistinctKeep::First,
    };

    match load_table_for_tool(&args, "distinct_rows") {
        Ok(OperationLoad::NeedsConfirmation(response)) => response,
        Ok(OperationLoad::Loaded(loaded)) => match distinct_rows(&loaded, &subset, keep) {
            Ok(distincted) => {
                respond_with_preview_and_result_ref("distinct_rows", &args, &distincted, 20)
            }
            Err(error) => ToolResponse::error(error.to_string()),
        },
        Err(response) => response,
    }
}

// 2026-03-22: 杩欓噷鎺ュ叆鎸変笟鍔￠敭鍘婚噸鍒嗗彂锛岀洰鐨勬槸璁┾€滃厛鎸夋帓搴忚鍒欓€夋渶鏂?鏈€鏃╄褰曗€濈殑鐪熷疄涓氬姟娓呮礂鍦烘櫙杩涘叆缁熶竴 Tool 鍏ュ彛銆?
fn dispatch_deduplicate_by_key(args: Value) -> ToolResponse {
    let keys = string_array(&args, "keys");
    let order_by = match args.get("order_by") {
        Some(value) => match serde_json::from_value::<Vec<OrderSpec>>(value.clone()) {
            Ok(order_by) => order_by,
            Err(error) => {
                return ToolResponse::error(format!(
                    "deduplicate_by_key 鍙傛暟瑙ｆ瀽澶辫触: {error}"
                ));
            }
        },
        None => Vec::new(),
    };
    let keep = match args.get("keep") {
        Some(value) => match serde_json::from_value::<DeduplicateKeep>(value.clone()) {
            Ok(keep) => keep,
            Err(error) => {
                return ToolResponse::error(format!(
                    "deduplicate_by_key 鍙傛暟瑙ｆ瀽澶辫触: {error}"
                ));
            }
        },
        None => DeduplicateKeep::First,
    };

    match load_table_for_tool(&args, "deduplicate_by_key") {
        Ok(OperationLoad::NeedsConfirmation(response)) => response,
        Ok(OperationLoad::Loaded(loaded)) => {
            match deduplicate_by_key(&loaded, &keys, &order_by, keep) {
                Ok(deduplicated) => respond_with_preview_and_result_ref(
                    "deduplicate_by_key",
                    &args,
                    &deduplicated,
                    20,
                ),
                Err(error) => ToolResponse::error(error.to_string()),
            }
        }
        Err(response) => response,
    }
}

// 2026-03-22: 杩欓噷鎺ュ叆瀵煎嚭鍓嶆暣鐞嗗垎鍙戯紝鐩殑鏄鍒楅『搴忋€佽〃澶村埆鍚嶅拰杈撳嚭瑁佸壀娌跨敤鐜版湁鍗曡〃 Tool 鐨勯摼寮忎綋楠屻€?
fn dispatch_format_table_for_export(args: Value) -> ToolResponse {
    let options = match serde_json::from_value::<ExportFormatOptions>(args.clone()) {
        Ok(options) => options,
        Err(error) => {
            return ToolResponse::error(format!(
                "format_table_for_export 鍙傛暟瑙ｆ瀽澶辫触: {error}"
            ));
        }
    };

    match load_table_for_tool(&args, "format_table_for_export") {
        Ok(OperationLoad::NeedsConfirmation(response)) => response,
        Ok(OperationLoad::Loaded(loaded)) => match format_table_for_export(&loaded, &options) {
            Ok(formatted) => respond_with_preview_and_result_ref(
                "format_table_for_export",
                &args,
                &formatted,
                20,
            ),
            Err(error) => ToolResponse::error(error.to_string()),
        },
        Err(response) => response,
    }
}

// 2026-03-23: 杩欓噷鎺ュ叆 lookup 鍥炲～鍒嗗彂锛岀洰鐨勬槸璁?base / lookup 鍙屾潵婧愬満鏅篃鑳藉鐢ㄧ粺涓€ JSON Tool 鍏ュ彛銆?
fn dispatch_fill_missing_from_lookup(args: Value) -> ToolResponse {
    let Some(base_value) = args.get("base") else {
        return ToolResponse::error("fill_missing_from_lookup 缂哄皯 base 鍙傛暟");
    };
    let Some(lookup_value) = args.get("lookup") else {
        return ToolResponse::error("fill_missing_from_lookup 缂哄皯 lookup 鍙傛暟");
    };
    let base_keys =
        match parse_lookup_key_args(&args, "base_on", "base_keys", "fill_missing_from_lookup") {
            Ok(keys) => keys,
            Err(response) => return response,
        };
    let lookup_keys = match parse_lookup_key_args(
        &args,
        "lookup_on",
        "lookup_keys",
        "fill_missing_from_lookup",
    ) {
        Ok(keys) => keys,
        Err(response) => return response,
    };
    let Some(fills_value) = args.get("fills") else {
        return ToolResponse::error("fill_missing_from_lookup 缂哄皯 fills 鍙傛暟");
    };
    let fills = match serde_json::from_value::<Vec<FillLookupRule>>(fills_value.clone()) {
        Ok(fills) => fills,
        Err(error) => {
            return ToolResponse::error(format!(
                "fill_missing_from_lookup 鍙傛暟瑙ｆ瀽澶辫触: {error}"
            ));
        }
    };

    match load_nested_table_source(base_value, "fill_missing_from_lookup", "base") {
        Ok(OperationLoad::NeedsConfirmation(response)) => response,
        Ok(OperationLoad::Loaded(base_loaded)) => {
            match load_nested_table_source(lookup_value, "fill_missing_from_lookup", "lookup") {
                Ok(OperationLoad::NeedsConfirmation(response)) => response,
                Ok(OperationLoad::Loaded(lookup_loaded)) => {
                    let base_key_refs = base_keys.iter().map(String::as_str).collect::<Vec<_>>();
                    let lookup_key_refs =
                        lookup_keys.iter().map(String::as_str).collect::<Vec<_>>();
                    match fill_missing_from_lookup_by_keys(
                        &base_loaded,
                        &lookup_loaded,
                        &base_key_refs,
                        &lookup_key_refs,
                        &fills,
                    ) {
                        Ok(filled) => respond_with_preview_and_result_ref(
                            "fill_missing_from_lookup",
                            &args,
                            &filled,
                            5,
                        ),
                        Err(error) => ToolResponse::error(error.to_string()),
                    }
                }
                Err(response) => response,
            }
        }
        Err(response) => response,
    }
}

fn dispatch_suggest_table_links(args: Value) -> ToolResponse {
    let Some(left_value) = args.get("left") else {
        return ToolResponse::error("suggest_table_links 缂哄皯 left 鍙傛暟");
    };
    let Some(right_value) = args.get("right") else {
        return ToolResponse::error("suggest_table_links 缂哄皯 right 鍙傛暟");
    };
    // 2026-03-21: 杩欓噷闄愬埗鍊欓€夋暟閲忓彲閰嶇疆锛岀洰鐨勬槸璁╅棶绛旂晫闈㈠厛鐪嬫渶绋崇殑灏戦噺寤鸿锛岄伩鍏嶄竴娆¤繑鍥炶繃澶氬櫔澹般€?
    let max_candidates = args
        .get("max_candidates")
        .and_then(|value| value.as_u64())
        .unwrap_or(3) as usize;

    let left_casts = match parse_casts(&args, "left_casts", "suggest_table_links") {
        Ok(casts) => casts,
        Err(response) => return response,
    };
    let right_casts = match parse_casts(&args, "right_casts", "suggest_table_links") {
        Ok(casts) => casts,
        Err(response) => return response,
    };

    // 2026-03-23: 杩欓噷鎶婂叧绯诲缓璁眰涔熷崌绾т负宓屽鏉ユ簮杈撳叆锛岀洰鐨勬槸璁╂樉鎬у叧鑱斿€欓€夊彲浠ョ洿鎺ユ秷璐?table_ref 鍜?result_ref銆?
    match load_nested_table_source(left_value, "suggest_table_links", "left") {
        Ok(OperationLoad::NeedsConfirmation(response)) => response,
        Ok(OperationLoad::Loaded(left_loaded)) => {
            match load_nested_table_source(right_value, "suggest_table_links", "right") {
                Ok(OperationLoad::NeedsConfirmation(response)) => response,
                Ok(OperationLoad::Loaded(right_loaded)) => {
                    let prepared_left = apply_optional_casts(left_loaded, &left_casts);
                    let prepared_right = apply_optional_casts(right_loaded, &right_casts);

                    match (prepared_left, prepared_right) {
                        (Ok(prepared_left), Ok(prepared_right)) => {
                            match suggest_table_links(
                                &prepared_left,
                                &prepared_right,
                                max_candidates,
                            ) {
                                Ok(result) => ToolResponse::ok(json!(result)),
                                Err(error) => ToolResponse::error(error.to_string()),
                            }
                        }
                        (Err(error), _) | (_, Err(error)) => ToolResponse::error(error),
                    }
                }
                Err(response) => response,
            }
        }
        Err(response) => response,
    }
}

fn dispatch_suggest_table_workflow(args: Value) -> ToolResponse {
    let Some(left_value) = args.get("left") else {
        return ToolResponse::error("suggest_table_workflow 缂哄皯 left 鍙傛暟");
    };
    let Some(right_value) = args.get("right") else {
        return ToolResponse::error("suggest_table_workflow 缂哄皯 right 鍙傛暟");
    };
    // 2026-03-22: 杩欓噷闄愬埗鏈€澶氳繑鍥炲灏戜釜鍏宠仈鍊欓€夛紝鐩殑鏄宸ヤ綔娴佸眰杈撳嚭淇濇寔绮剧畝锛屼紭鍏堝憟鐜版渶绋崇殑灏戦噺寤鸿銆?
    let max_link_candidates = args
        .get("max_link_candidates")
        .and_then(|value| value.as_u64())
        .unwrap_or(3) as usize;

    let left_casts = match parse_casts(&args, "left_casts", "suggest_table_workflow") {
        Ok(casts) => casts,
        Err(response) => return response,
    };
    let right_casts = match parse_casts(&args, "right_casts", "suggest_table_workflow") {
        Ok(casts) => casts,
        Err(response) => return response,
    };

    // 2026-03-23: 杩欓噷鎶婂伐浣滄祦寤鸿灞備篃鍗囩骇涓哄祵濂楁潵婧愯緭鍏ワ紝鐩殑鏄寤鸿璋冪敤楠ㄦ灦鑳芥壙鎺ヤ笂娓稿彞鏌勮€屼笉閫€鍖栧洖鍘熷璺緞銆?
    match load_nested_table_source(left_value, "suggest_table_workflow", "left") {
        Ok(OperationLoad::NeedsConfirmation(response)) => response,
        Ok(OperationLoad::Loaded(left_loaded)) => {
            match load_nested_table_source(right_value, "suggest_table_workflow", "right") {
                Ok(OperationLoad::NeedsConfirmation(response)) => response,
                Ok(OperationLoad::Loaded(right_loaded)) => {
                    let prepared_left = apply_optional_casts(left_loaded, &left_casts);
                    let prepared_right = apply_optional_casts(right_loaded, &right_casts);

                    match (prepared_left, prepared_right) {
                        (Ok(prepared_left), Ok(prepared_right)) => {
                            match suggest_table_workflow(
                                &prepared_left,
                                &prepared_right,
                                max_link_candidates,
                            ) {
                                Ok(result) => {
                                    let mut payload = json!(result);
                                    // 2026-03-23: 杩欓噷鎶婂缓璁皟鐢ㄤ腑鐨勬潵婧愰鏋舵敼鍥炵敤鎴峰師濮嬭緭鍏ワ紝鐩殑鏄伩鍏?table_ref/result_ref 鍦ㄥ缓璁眰閫€鍖栧洖 path+sheet銆?
                                    rewrite_workflow_suggested_tool_call_sources(
                                        &mut payload,
                                        left_value.clone(),
                                        right_value.clone(),
                                    );
                                    ToolResponse::ok(payload)
                                }
                                Err(error) => ToolResponse::error(error.to_string()),
                            }
                        }
                        (Err(error), _) | (_, Err(error)) => ToolResponse::error(error),
                    }
                }
                Err(response) => response,
            }
        }
        Err(response) => response,
    }
}

fn dispatch_suggest_multi_table_plan(args: Value) -> ToolResponse {
    #[derive(Debug, Deserialize)]
    struct MultiPlanTableInput {
        path: Option<String>,
        sheet: Option<String>,
        table_ref: Option<String>,
        result_ref: Option<String>,
        alias: Option<String>,
    }

    let Some(tables_value) = args.get("tables") else {
        return ToolResponse::error("suggest_multi_table_plan 缂哄皯 tables 鍙傛暟");
    };
    let table_inputs =
        match serde_json::from_value::<Vec<MultiPlanTableInput>>(tables_value.clone()) {
            Ok(inputs) => inputs,
            Err(error) => {
                return ToolResponse::error(format!(
                    "suggest_multi_table_plan 鐨?tables 鍙傛暟瑙ｆ瀽澶辫触: {error}"
                ));
            }
        };
    let max_link_candidates = args
        .get("max_link_candidates")
        .and_then(|value| value.as_u64())
        .unwrap_or(3) as usize;

    let mut loaded_tables = Vec::<(String, LoadedTable)>::new();
    let mut source_payloads = BTreeMap::<String, Value>::new();
    for (index, input) in table_inputs.into_iter().enumerate() {
        let table_ref = input
            .alias
            .unwrap_or_else(|| format!("table_{}", index + 1));
        let source = NestedTableSource {
            path: input.path,
            sheet: input.sheet,
            // 2026-03-23: 杩欓噷鏄惧紡琛ラ綈 file_ref/sheet_index 缂虹渷鍊硷紝鍘熷洜鏄?NestedTableSource 宸叉墿灞曟柊鍏ュ彛锛涚洰鐨勬槸淇濇寔鏃х殑澶氳〃璁″垝杈撳叆浠嶅彲绋冲畾鏋勯€犮€?
            file_ref: None,
            sheet_index: None,
            table_ref: input.table_ref,
            result_ref: input.result_ref,
        };
        let source_payload = nested_source_payload(&source);
        // 2026-03-23: 杩欓噷鎶婂琛ㄨ鍒掑櫒鐨勬瘡涓師濮嬭緭鍏ユ潵婧愬厛缂撳瓨涓嬫潵锛岀洰鐨勬槸鍚庨潰鎶婂缓璁皟鐢ㄩ鏋舵仮澶嶆垚鐢ㄦ埛鏈€鍒濅紶鍏ョ殑鏉ユ簮绫诲瀷銆?
        source_payloads.insert(table_ref.clone(), source_payload);
        match load_nested_table_source_from_parsed(&source, "suggest_multi_table_plan", "tables") {
            Ok(OperationLoad::NeedsConfirmation(response)) => return response,
            Ok(OperationLoad::Loaded(loaded)) => loaded_tables.push((table_ref, loaded)),
            Err(response) => return response,
        }
    }

    match suggest_multi_table_plan(loaded_tables, max_link_candidates) {
        Ok(result) => {
            let mut payload = json!(result);
            rewrite_multi_table_plan_suggested_tool_call_sources(&mut payload, &source_payloads);
            ToolResponse::ok(payload)
        }
        Err(error) => ToolResponse::error(error.to_string()),
    }
}

fn dispatch_append_tables(args: Value) -> ToolResponse {
    let Some(top_value) = args.get("top") else {
        return ToolResponse::error("append_tables 缂哄皯 top 鍙傛暟");
    };
    let Some(bottom_value) = args.get("bottom") else {
        return ToolResponse::error("append_tables 缂哄皯 bottom 鍙傛暟");
    };
    let limit = args
        .get("limit")
        .and_then(|value| value.as_u64())
        .unwrap_or(5) as usize;

    // 2026-03-22: 杩欓噷鎶?append 鐨勪笂涓嬭緭鍏ョ粺涓€鏀跺彛鍒板祵濂楁潵婧愯В鏋愬櫒锛岀洰鐨勬槸璁┾€滃厛杩藉姞鍐嶅叧鑱斺€濋摼璺兘鐩存帴娑堣垂涓婁竴浠?result_ref銆?
    match load_nested_table_source(top_value, "append_tables", "top") {
        Ok(OperationLoad::NeedsConfirmation(response)) => response,
        Ok(OperationLoad::Loaded(top_loaded)) => {
            match load_nested_table_source(bottom_value, "append_tables", "bottom") {
                Ok(OperationLoad::NeedsConfirmation(response)) => response,
                Ok(OperationLoad::Loaded(bottom_loaded)) => {
                    match append_tables(&top_loaded, &bottom_loaded) {
                        Ok(appended) => respond_with_preview_and_result_ref(
                            "append_tables",
                            &args,
                            &appended,
                            limit,
                        ),
                        Err(error) => ToolResponse::error(error.to_string()),
                    }
                }
                Err(response) => response,
            }
        }
        Err(response) => response,
    }
}

fn dispatch_summarize_table(args: Value) -> ToolResponse {
    let requested_columns = string_array(&args, "columns");
    let top_k = args
        .get("top_k")
        .and_then(|value| value.as_u64())
        .unwrap_or(5) as usize;
    let casts = match parse_casts(&args, "casts", "summarize_table") {
        Ok(casts) => casts,
        Err(response) => return response,
    };

    match load_table_for_analysis(&args, "summarize_table") {
        Ok(OperationLoad::NeedsConfirmation(response)) => response,
        Ok(OperationLoad::Loaded(loaded)) => match apply_optional_casts(loaded, &casts) {
            Ok(prepared_loaded) => {
                match summarize_table(&prepared_loaded, &requested_columns, top_k) {
                    Ok(summaries) => {
                        if let Err(response) = sync_loaded_table_state(
                            &args,
                            &prepared_loaded,
                            SessionStage::AnalysisModeling,
                            "鏌ョ湅缁熻鎽樿",
                            "summarize_table",
                            "analysis_completed",
                        ) {
                            return response;
                        }
                        ToolResponse::ok(json!({
                            "row_count": prepared_loaded.dataframe.height(),
                            "summaries": summaries,
                        }))
                    }
                    Err(error) => ToolResponse::error(error.to_string()),
                }
            }
            Err(error) => ToolResponse::error(error),
        },
        Err(response) => response,
    }
}

fn dispatch_analyze_table(args: Value) -> ToolResponse {
    let requested_columns = string_array(&args, "columns");
    let top_k = args
        .get("top_k")
        .and_then(|value| value.as_u64())
        .unwrap_or(5) as usize;
    let casts = match parse_casts(&args, "casts", "analyze_table") {
        Ok(casts) => casts,
        Err(response) => return response,
    };

    match load_table_for_analysis(&args, "analyze_table") {
        Ok(OperationLoad::NeedsConfirmation(response)) => response,
        Ok(OperationLoad::Loaded(loaded)) => match apply_optional_casts(loaded, &casts) {
            Ok(prepared_loaded) => {
                let result = analyze_table(&prepared_loaded, &requested_columns, top_k);
                if let Err(response) = sync_loaded_table_state(
                    &args,
                    &prepared_loaded,
                    SessionStage::AnalysisModeling,
                    "鏌ョ湅鍒嗘瀽璇婃柇",
                    "analyze_table",
                    "analysis_completed",
                ) {
                    return response;
                }
                ToolResponse::ok(json!(result))
            }
            Err(error) => ToolResponse::error(error),
        },
        Err(response) => response,
    }
}

fn dispatch_stat_summary(args: Value) -> ToolResponse {
    let requested_columns = string_array(&args, "columns");
    let top_k = args
        .get("top_k")
        .and_then(|value| value.as_u64())
        .unwrap_or(5) as usize;
    let casts = match parse_casts(&args, "casts", "stat_summary") {
        Ok(casts) => casts,
        Err(response) => return response,
    };

    match load_table_for_analysis(&args, "stat_summary") {
        Ok(OperationLoad::NeedsConfirmation(response)) => response,
        Ok(OperationLoad::Loaded(loaded)) => match apply_optional_casts(loaded, &casts) {
            Ok(prepared_loaded) => {
                match stat_summary(&prepared_loaded, &requested_columns, top_k) {
                    Ok(result) => {
                        if let Err(response) = sync_loaded_table_state(
                            &args,
                            &prepared_loaded,
                            SessionStage::AnalysisModeling,
                            "鏌ョ湅缁熻鎽樿",
                            "stat_summary",
                            "analysis_completed",
                        ) {
                            return response;
                        }
                        ToolResponse::ok(json!(result))
                    }
                    Err(error) => ToolResponse::error(error.to_string()),
                }
            }
            Err(error) => ToolResponse::error(error),
        },
        Err(response) => response,
    }
}

fn dispatch_correlation_analysis(args: Value) -> ToolResponse {
    let Some(target_column) = args.get("target_column").and_then(|value| value.as_str()) else {
        return ToolResponse::error("correlation_analysis 缺少 target_column 参数");
    };
    let feature_columns = string_array(&args, "feature_columns");
    let casts = match parse_casts(&args, "casts", "correlation_analysis") {
        Ok(casts) => casts,
        Err(response) => return response,
    };

    match load_table_for_analysis(&args, "correlation_analysis") {
        Ok(OperationLoad::NeedsConfirmation(response)) => response,
        Ok(OperationLoad::Loaded(loaded)) => match apply_optional_casts(loaded, &casts) {
            Ok(prepared_loaded) => {
                match correlation_analysis(&prepared_loaded, target_column, &feature_columns) {
                    Ok(result) => {
                        if let Err(response) = sync_loaded_table_state(
                            &args,
                            &prepared_loaded,
                            SessionStage::AnalysisModeling,
                            "查看相关性分析",
                            "correlation_analysis",
                            "analysis_completed",
                        ) {
                            return response;
                        }
                        ToolResponse::ok(json!(result))
                    }
                    Err(error) => ToolResponse::error(error.to_string()),
                }
            }
            Err(error) => ToolResponse::error(error),
        },
        Err(response) => response,
    }
}

fn dispatch_outlier_detection(args: Value) -> ToolResponse {
    let columns = string_array(&args, "columns");
    let method = match args.get("method") {
        Some(value) => match serde_json::from_value::<OutlierDetectionMethod>(value.clone()) {
            Ok(method) => method,
            Err(error) => {
                return ToolResponse::error(format!(
                    "outlier_detection 的 method 参数解析失败: {error}"
                ));
            }
        },
        None => OutlierDetectionMethod::Iqr,
    };
    let casts = match parse_casts(&args, "casts", "outlier_detection") {
        Ok(casts) => casts,
        Err(response) => return response,
    };

    match load_table_for_analysis(&args, "outlier_detection") {
        Ok(OperationLoad::NeedsConfirmation(response)) => response,
        Ok(OperationLoad::Loaded(loaded)) => match apply_optional_casts(loaded, &casts) {
            Ok(prepared_loaded) => match outlier_detection(&prepared_loaded, &columns, method) {
                Ok((flagged_loaded, result)) => {
                    if let Err(response) = sync_loaded_table_state(
                        &args,
                        &prepared_loaded,
                        SessionStage::AnalysisModeling,
                        "查看异常值诊断",
                        "outlier_detection",
                        "analysis_completed",
                    ) {
                        return response;
                    }
                    respond_with_result_dataset(
                        "outlier_detection",
                        &args,
                        &flagged_loaded,
                        json!({
                            "method": result.method,
                            "row_count": result.row_count,
                            "outlier_summaries": result.outlier_summaries,
                            "human_summary": result.human_summary,
                            "columns": preview_table(&flagged_loaded.dataframe, 20).map(|preview| preview.columns).unwrap_or_default(),
                            "rows": preview_table(&flagged_loaded.dataframe, 20).map(|preview| preview.rows).unwrap_or_default()
                        }),
                    )
                }
                Err(error) => ToolResponse::error(error.to_string()),
            },
            Err(error) => ToolResponse::error(error),
        },
        Err(response) => response,
    }
}

fn dispatch_distribution_analysis(args: Value) -> ToolResponse {
    let Some(column) = args.get("column").and_then(|value| value.as_str()) else {
        return ToolResponse::error("distribution_analysis 缺少 column 参数");
    };
    let bins = args
        .get("bins")
        .and_then(|value| value.as_u64())
        .unwrap_or(10) as usize;
    let casts = match parse_casts(&args, "casts", "distribution_analysis") {
        Ok(casts) => casts,
        Err(response) => return response,
    };

    match load_table_for_analysis(&args, "distribution_analysis") {
        Ok(OperationLoad::NeedsConfirmation(response)) => response,
        Ok(OperationLoad::Loaded(loaded)) => match apply_optional_casts(loaded, &casts) {
            Ok(prepared_loaded) => match distribution_analysis(&prepared_loaded, column, bins) {
                Ok(result) => {
                    if let Err(response) = sync_loaded_table_state(
                        &args,
                        &prepared_loaded,
                        SessionStage::AnalysisModeling,
                        "查看分布分析",
                        "distribution_analysis",
                        "analysis_completed",
                    ) {
                        return response;
                    }
                    ToolResponse::ok(json!(result))
                }
                Err(error) => ToolResponse::error(error.to_string()),
            },
            Err(error) => ToolResponse::error(error),
        },
        Err(response) => response,
    }
}

fn dispatch_trend_analysis(args: Value) -> ToolResponse {
    let Some(time_column) = args.get("time_column").and_then(|value| value.as_str()) else {
        return ToolResponse::error("trend_analysis 缺少 time_column 参数");
    };
    let Some(value_column) = args.get("value_column").and_then(|value| value.as_str()) else {
        return ToolResponse::error("trend_analysis 缺少 value_column 参数");
    };
    let casts = match parse_casts(&args, "casts", "trend_analysis") {
        Ok(casts) => casts,
        Err(response) => return response,
    };

    match load_table_for_analysis(&args, "trend_analysis") {
        Ok(OperationLoad::NeedsConfirmation(response)) => response,
        Ok(OperationLoad::Loaded(loaded)) => match apply_optional_casts(loaded, &casts) {
            Ok(prepared_loaded) => {
                match trend_analysis(&prepared_loaded, time_column, value_column) {
                    Ok(result) => {
                        if let Err(response) = sync_loaded_table_state(
                            &args,
                            &prepared_loaded,
                            SessionStage::AnalysisModeling,
                            "查看趋势分析",
                            "trend_analysis",
                            "analysis_completed",
                        ) {
                            return response;
                        }
                        ToolResponse::ok(json!(result))
                    }
                    Err(error) => ToolResponse::error(error.to_string()),
                }
            }
            Err(error) => ToolResponse::error(error),
        },
        Err(response) => response,
    }
}

fn dispatch_linear_regression(args: Value) -> ToolResponse {
    let Some(features_value) = args.get("features").and_then(|value| value.as_array()) else {
        return ToolResponse::error("linear_regression 缂哄皯 features 鍙傛暟");
    };
    let Some(target) = args.get("target").and_then(|value| value.as_str()) else {
        return ToolResponse::error("linear_regression 缂哄皯 target 鍙傛暟");
    };
    let features = features_value
        .iter()
        .filter_map(|value| value.as_str())
        .collect::<Vec<_>>();
    let intercept = args
        .get("intercept")
        .and_then(|value| value.as_bool())
        .unwrap_or(true);
    let missing_strategy = match parse_missing_strategy(&args, "linear_regression") {
        Ok(strategy) => strategy,
        Err(response) => return response,
    };
    let casts = match parse_casts(&args, "casts", "linear_regression") {
        Ok(casts) => casts,
        Err(response) => return response,
    };

    match load_table_for_analysis(&args, "linear_regression") {
        Ok(OperationLoad::NeedsConfirmation(response)) => response,
        Ok(OperationLoad::Loaded(loaded)) => match apply_optional_casts(loaded, &casts) {
            Ok(prepared_loaded) => match linear_regression(
                &prepared_loaded,
                &features,
                target,
                intercept,
                missing_strategy,
            ) {
                Ok(result) => {
                    if let Err(response) = sync_loaded_table_state(
                        &args,
                        &prepared_loaded,
                        SessionStage::AnalysisModeling,
                        // 2026-03-24: 这里补回线性回归阶段目标文案，原因是历史乱码再次吞掉结束引号；目的是恢复 modeling 路径编译并保留会话状态摘要。
                        "执行线性回归分析",
                        "linear_regression",
                        "modeling_completed",
                    ) {
                        return response;
                    }
                    ToolResponse::ok(json!(result))
                }
                Err(error) => ToolResponse::error(error.to_string()),
            },
            Err(error) => ToolResponse::error(error),
        },
        Err(response) => response,
    }
}

fn dispatch_logistic_regression(args: Value) -> ToolResponse {
    let Some(features_value) = args.get("features").and_then(|value| value.as_array()) else {
        return ToolResponse::error("logistic_regression 缂哄皯 features 鍙傛暟");
    };
    let Some(target) = args.get("target").and_then(|value| value.as_str()) else {
        return ToolResponse::error("logistic_regression 缂哄皯 target 鍙傛暟");
    };
    let features = features_value
        .iter()
        .filter_map(|value| value.as_str())
        .collect::<Vec<_>>();
    let intercept = args
        .get("intercept")
        .and_then(|value| value.as_bool())
        .unwrap_or(true);
    let positive_label = args.get("positive_label").and_then(|value| value.as_str());
    let missing_strategy = match parse_missing_strategy(&args, "logistic_regression") {
        Ok(strategy) => strategy,
        Err(response) => return response,
    };
    let casts = match parse_casts(&args, "casts", "logistic_regression") {
        Ok(casts) => casts,
        Err(response) => return response,
    };

    match load_table_for_analysis(&args, "logistic_regression") {
        Ok(OperationLoad::NeedsConfirmation(response)) => response,
        Ok(OperationLoad::Loaded(loaded)) => match apply_optional_casts(loaded, &casts) {
            Ok(prepared_loaded) => match logistic_regression(
                &prepared_loaded,
                &features,
                target,
                intercept,
                missing_strategy,
                positive_label,
            ) {
                Ok(result) => {
                    if let Err(response) = sync_loaded_table_state(
                        &args,
                        &prepared_loaded,
                        SessionStage::AnalysisModeling,
                        "閹笛嗩攽闁槒绶崶鐐茬秺",
                        "logistic_regression",
                        "modeling_completed",
                    ) {
                        return response;
                    }
                    ToolResponse::ok(json!(result))
                }
                Err(error) => ToolResponse::error(error.to_string()),
            },
            Err(error) => ToolResponse::error(error),
        },
        Err(response) => response,
    }
}
fn dispatch_cluster_kmeans(args: Value) -> ToolResponse {
    let Some(features_value) = args.get("features").and_then(|value| value.as_array()) else {
        return ToolResponse::error("cluster_kmeans 缂哄皯 features 鍙傛暟");
    };
    let Some(cluster_count) = args.get("cluster_count").and_then(|value| value.as_u64()) else {
        return ToolResponse::error("cluster_kmeans 缂哄皯 cluster_count 鍙傛暟");
    };
    let features = features_value
        .iter()
        .filter_map(|value| value.as_str())
        .collect::<Vec<_>>();
    let max_iterations = args
        .get("max_iterations")
        .and_then(|value| value.as_u64())
        .unwrap_or(100) as usize;
    let missing_strategy = match parse_missing_strategy(&args, "cluster_kmeans") {
        Ok(strategy) => strategy,
        Err(response) => return response,
    };
    let casts = match parse_casts(&args, "casts", "cluster_kmeans") {
        Ok(casts) => casts,
        Err(response) => return response,
    };

    match load_table_for_analysis(&args, "cluster_kmeans") {
        Ok(OperationLoad::NeedsConfirmation(response)) => response,
        Ok(OperationLoad::Loaded(loaded)) => match apply_optional_casts(loaded, &casts) {
            Ok(prepared_loaded) => match cluster_kmeans(
                &prepared_loaded,
                &features,
                cluster_count as usize,
                max_iterations,
                missing_strategy,
            ) {
                Ok(result) => {
                    if let Err(response) = sync_loaded_table_state(
                        &args,
                        &prepared_loaded,
                        SessionStage::AnalysisModeling,
                        "鎵ц鑱氱被鍒嗘瀽",
                        "cluster_kmeans",
                        "modeling_completed",
                    ) {
                        return response;
                    }
                    ToolResponse::ok(json!(result))
                }
                Err(error) => ToolResponse::error(error.to_string()),
            },
            Err(error) => ToolResponse::error(error),
        },
        Err(response) => response,
    }
}

fn dispatch_decision_assistant(args: Value) -> ToolResponse {
    let requested_columns = string_array(&args, "columns");
    let top_k = args
        .get("top_k")
        .and_then(|value| value.as_u64())
        .unwrap_or(5) as usize;
    let casts = match parse_casts(&args, "casts", "decision_assistant") {
        Ok(casts) => casts,
        Err(response) => return response,
    };

    match load_table_for_analysis(&args, "decision_assistant") {
        Ok(OperationLoad::NeedsConfirmation(response)) => response,
        Ok(OperationLoad::Loaded(loaded)) => match apply_optional_casts(loaded, &casts) {
            Ok(prepared_loaded) => {
                match decision_assistant(&prepared_loaded, &requested_columns, top_k) {
                    Ok(result) => {
                        if let Err(response) = sync_loaded_table_state(
                            &args,
                            &prepared_loaded,
                            SessionStage::DecisionAssistant,
                            // 2026-03-24: 这里补回决策助手阶段目标文案，原因是坏字符串导致后半段文件持续落在未闭合字符串里；目的是恢复后续辅助函数区段的正常解析。
                            "获取下一步决策建议",
                            "decision_assistant",
                            "decision_assistant_completed",
                        ) {
                            return response;
                        }
                        ToolResponse::ok(json!(result))
                    }
                    Err(error) => ToolResponse::error(error.to_string()),
                }
            }
            Err(error) => ToolResponse::error(error),
        },
        Err(response) => response,
    }
}

enum OperationLoad {
    NeedsConfirmation(ToolResponse),
    Loaded(LoadedTable),
}

#[derive(Debug, Deserialize)]
struct NestedTableSource {
    path: Option<String>,
    sheet: Option<String>,
    // 2026-03-23: 杩欓噷鏂板 file_ref锛屽師鍥犳槸鍚庣画澶氳〃鍦烘櫙涔熷彲鑳介渶瑕佸鐢ㄢ€滃凡鎵撳紑鏂囦欢 + 绗嚑涓?Sheet鈥濓紱鐩殑鏄宓屽鏉ユ簮涓庨《灞傛潵婧愪繚鎸佸悓涓€濂楃ǔ濡ュ叆鍙ｃ€?
    file_ref: Option<String>,
    // 2026-03-23: 杩欓噷鏂板 sheet_index锛屽師鍥犳槸 file_ref 妯″紡涓嬩笉鑳藉啀瑕佹眰涓婂眰閲嶅浼?Sheet 鍚嶏紱鐩殑鏄澶氳〃鏉ユ簮涔熻兘鎸夆€滅鍑犱釜 Sheet鈥濈户缁€?
    sheet_index: Option<usize>,
    table_ref: Option<String>,
    result_ref: Option<String>,
}

// 2026-03-23: 杩欓噷瀹氫箟鍗曡〃鏉ユ簮瑙ｆ瀽缁撴灉锛屽師鍥犳槸 dispatcher 闇€瑕佹妸 path+sheet 涓?file_ref+sheet_index 涓ゅ鍏ュ彛缁熶竴鍒板悓涓€鍐呴儴缁撴瀯锛涚洰鐨勬槸鏂板鏂板叆鍙ｆ椂涓嶇牬鍧忔棫鍒嗘敮銆?
struct ResolvedSheetSource {
    path: String,
    sheet_name: String,
}

fn load_sheet_for_operation(path: &str, sheet: &str) -> Result<OperationLoad, String> {
    match infer_header_schema(path, sheet) {
        Ok(inference) => {
            if !matches!(inference.confidence, ConfidenceLevel::High) {
                return Ok(OperationLoad::NeedsConfirmation(build_inference_response(
                    sheet, inference,
                )));
            }

            load_confirmed_table(path, sheet, &inference)
                .map(OperationLoad::Loaded)
                .map_err(|error| error.to_string())
        }
        Err(error) => Err(error.to_string()),
    }
}

// 2026-03-22: 杩欓噷涓?join/append 绛夊琛?Tool 缁熶竴瑙ｆ瀽 path銆乼able_ref銆乺esult_ref 涓夌鏉ユ簮銆?
fn load_nested_table_source(
    value: &Value,
    tool: &str,
    field_name: &str,
) -> Result<OperationLoad, ToolResponse> {
    let source = parse_nested_table_source(value, tool, field_name)?;
    load_nested_table_source_from_parsed(&source, tool, field_name)
}

// 2026-03-23: 杩欓噷鎶婂祵濂楁潵婧愯В鏋愭媶鎴愮嫭绔嬫楠わ紝鐩殑鏄鈥滃厛鍔犺浇鈥濅笌鈥滀繚鐣欏師濮嬫潵婧愰鏋垛€濅袱绫婚渶姹傚鐢ㄥ悓涓€濂楀弬鏁版牎楠屻€?
fn parse_nested_table_source(
    value: &Value,
    tool: &str,
    field_name: &str,
) -> Result<NestedTableSource, ToolResponse> {
    serde_json::from_value::<NestedTableSource>(value.clone()).map_err(|error| {
        ToolResponse::error(format!(
            "{tool} 鐨?{field_name} 鍙傛暟瑙ｆ瀽澶辫触: {error}"
        ))
    })
}

// 2026-03-23: 杩欓噷澶嶇敤宸茶В鏋愭潵婧愮户缁杞借〃锛岀洰鐨勬槸璁╁琛ㄨ鍒掑櫒鏃㈣兘璇诲彇鏁版嵁锛屼篃鑳藉悓姝ユ嬁鍒板師濮嬫潵婧愬畾涔夈€?
fn load_nested_table_source_from_parsed(
    source: &NestedTableSource,
    tool: &str,
    field_name: &str,
) -> Result<OperationLoad, ToolResponse> {
    if let Some(result_ref) = source.result_ref.as_deref() {
        return load_result_from_ref(result_ref).map_err(ToolResponse::error);
    }

    if let Some(table_ref) = source.table_ref.as_deref() {
        return load_table_from_ref(table_ref).map_err(ToolResponse::error);
    }

    if let Some(file_ref) = source.file_ref.as_deref() {
        let Some(sheet_index) = source.sheet_index else {
            return Err(ToolResponse::error(format!(
                "{tool} 鐨?{field_name} 缂哄皯 sheet_index 鍙傛暟"
            )));
        };
        let resolved = resolve_sheet_source_from_file_ref(file_ref, sheet_index)
            .map_err(ToolResponse::error)?;
        return load_sheet_for_operation(&resolved.path, &resolved.sheet_name)
            .map_err(ToolResponse::error);
    }

    match (source.path.as_deref(), source.sheet.as_deref()) {
        (Some(path), Some(sheet)) => {
            load_sheet_for_operation(path, sheet).map_err(ToolResponse::error)
        }
        (Some(_), None) => Err(ToolResponse::error(format!(
            "{tool} 鐨?{field_name} 缂哄皯 sheet 鍙傛暟"
        ))),
        (None, Some(_)) => Err(ToolResponse::error(format!(
            "{tool} 鐨?{field_name} 缂哄皯 path 鍙傛暟"
        ))),
        (None, None) => Err(ToolResponse::error(format!(
            "{tool} 鐨?{field_name} 闇€瑕佹彁渚?path+sheet銆乫ile_ref+sheet_index銆乼able_ref 鎴?result_ref"
        ))),
    }
}

// 2026-03-23: 杩欓噷鎶婃潵婧愬畾涔夐噸鏂板帇鎴愭渶灏?JSON 楠ㄦ灦锛岀洰鐨勬槸璁╁缓璁皟鐢ㄨ兘鍘熸牱淇濈暀鐢ㄦ埛杈撳叆鐨勬潵婧愮被鍨嬭€屼笉娣峰叆 alias 绛夎鍒掑瓧娈点€?
fn nested_source_payload(source: &NestedTableSource) -> Value {
    if let Some(table_ref) = source.table_ref.as_ref() {
        return json!({ "table_ref": table_ref });
    }
    if let Some(result_ref) = source.result_ref.as_ref() {
        return json!({ "result_ref": result_ref });
    }
    if let (Some(file_ref), Some(sheet_index)) = (source.file_ref.as_ref(), source.sheet_index) {
        return json!({
            "file_ref": file_ref,
            "sheet_index": sheet_index,
        });
    }
    if let (Some(path), Some(sheet)) = (source.path.as_ref(), source.sheet.as_ref()) {
        return json!({
            "path": path,
            "sheet": sheet,
        });
    }

    json!({})
}

// 2026-03-22: 杩欓噷鎶婂崟涓祵濂楁潵婧愮炕璇戞垚琛€缂樺紩鐢ㄥ垪琛紝鐩殑鏄 workbook 鑽夌涔熻兘淇濈暀姣忓紶 sheet 鐨勪笂娓告潵婧愯鏄庛€?
fn source_refs_from_nested_source(source: &NestedTableSource) -> Vec<String> {
    if let Some(table_ref) = source.table_ref.as_ref() {
        return vec![table_ref.clone()];
    }
    if let Some(result_ref) = source.result_ref.as_ref() {
        return vec![result_ref.clone()];
    }
    if let (Some(file_ref), Some(sheet_index)) = (source.file_ref.as_ref(), source.sheet_index) {
        return vec![format!("{file_ref}#{sheet_index}")];
    }
    if let (Some(path), Some(sheet)) = (source.path.as_ref(), source.sheet.as_ref()) {
        return vec![format!("{path}#{sheet}")];
    }
    Vec::new()
}

// 2026-03-22: 杩欓噷涓哄垎鏋愬缓妯″眰缁熶竴鎺ュ叆鍙屽叆鍙ｏ紝鐩殑鏄 Tool 鍚屾椂鍏煎鏃х殑 path+sheet 鍜屾柊鐨?table_ref銆?
fn load_table_for_analysis(args: &Value, tool: &str) -> Result<OperationLoad, ToolResponse> {
    load_table_for_tool(args, tool)
}

// 2026-03-22: 杩欓噷涓哄崟琛ㄧ被 Tool 缁熶竴鎺ュ叆涓夌杈撳叆锛岀洰鐨勬槸璁╄〃澶勭悊灞傚拰鍒嗘瀽灞傞兘鑳藉鐢?path+sheet銆乼able_ref銆乺esult_ref銆?
fn load_table_for_tool(args: &Value, tool: &str) -> Result<OperationLoad, ToolResponse> {
    if let Some(result_ref) = args.get("result_ref").and_then(|value| value.as_str()) {
        return load_result_from_ref(result_ref).map_err(ToolResponse::error);
    }

    if let Some(table_ref) = args.get("table_ref").and_then(|value| value.as_str()) {
        return load_table_from_ref(table_ref).map_err(ToolResponse::error);
    }

    let source = resolve_sheet_source(args, tool)?;
    load_sheet_for_operation(&source.path, &source.sheet_name).map_err(ToolResponse::error)
}

// 2026-03-22: 杩欓噷浠?table_ref 鎭㈠鎸佷箙鍖栫‘璁ゆ€侊紝鐩殑鏄鍒嗘瀽寤烘ā灞傝烦杩囬噸澶?schema 鎺ㄦ柇銆?
fn load_table_from_ref(table_ref: &str) -> Result<OperationLoad, String> {
    let store = TableRefStore::workspace_default().map_err(|error| error.to_string())?;
    let persisted = store.load(table_ref).map_err(|error| error.to_string())?;
    load_table_from_table_ref(&persisted)
        .map(OperationLoad::Loaded)
        .map_err(|error| error.to_string())
}

// 2026-03-22: 杩欓噷浠?result_ref 鎭㈠涓棿缁撴灉闆嗭紝鐩殑鏄璺ㄨ姹傞摼寮忓垎鏋愬彲浠ョ洿鎺ユ秷璐逛笂涓€姝ョ粨鏋滆€屼笉蹇呭洖閫€鍒板師濮?Excel銆?
fn load_result_from_ref(result_ref: &str) -> Result<OperationLoad, String> {
    let store = ResultRefStore::workspace_default().map_err(|error| error.to_string())?;
    let persisted = store.load(result_ref).map_err(|error| error.to_string())?;
    let dataframe = persisted
        .to_dataframe()
        .map_err(|error| error.to_string())?;
    let handle = TableHandle::new_confirmed(
        format!("result://{result_ref}"),
        persisted.produced_by.clone(),
        persisted
            .columns
            .iter()
            .map(|column| column.name.clone())
            .collect(),
    );

    Ok(OperationLoad::Loaded(LoadedTable { handle, dataframe }))
}

// 2026-03-23: 杩欓噷鎶?open_workbook/list_sheets 鎴愬姛缁撴灉缁熶竴鍗囩骇鎴愬甫 file_ref 鐨勫搷搴旓紝鍘熷洜鏄悗缁祦绋嬮渶瑕佹寜鈥滅鍑犱釜 Sheet鈥濈户缁紱鐩殑鏄湪涓嶇Щ闄ゆ棫瀛楁鐨勫墠鎻愪笅澧為噺琛ラ綈绋冲Ε鍏ュ彛銆?
fn build_opened_file_response(
    args: &Value,
    summary: crate::excel::reader::WorkbookSummary,
) -> ToolResponse {
    let original_path = args
        .get("original_path")
        .and_then(|value| value.as_str())
        .unwrap_or(summary.path.as_str());
    let recovery_applied = args
        .get("recovery_applied")
        .and_then(|value| value.as_bool())
        .unwrap_or(original_path != summary.path);
    let file_ref = SourceFileRefStore::create_file_ref();
    let persisted = match PersistedSourceFileRef::from_opened_file(
        file_ref.clone(),
        original_path,
        &summary.path,
        &summary.sheet_names,
        recovery_applied,
    ) {
        Ok(persisted) => persisted,
        Err(error) => return ToolResponse::error(error.to_string()),
    };
    let store = match SourceFileRefStore::workspace_default() {
        Ok(store) => store,
        Err(error) => return ToolResponse::error(error.to_string()),
    };
    if let Err(error) = store.save(&persisted) {
        return ToolResponse::error(error.to_string());
    }

    ToolResponse::ok(json!({
        "path": summary.path,
        "original_path": persisted.original_path,
        "working_path": persisted.working_path,
        "recovery_applied": persisted.recovery_applied,
        "file_ref": persisted.file_ref,
        "sheet_names": summary.sheet_names,
        "sheets": persisted.sheets,
    }))
}

// 2026-03-23: 杩欓噷缁熶竴瑙ｆ瀽鍗曡〃鏉ユ簮锛屽師鍥犳槸椤跺眰 Tool 闇€瑕佸悓鏃跺吋瀹?path+sheet 涓?file_ref+sheet_index 涓ゅ鍏ュ彛锛涚洰鐨勬槸鏂板绋冲Ε鍏ュ彛鏃朵笉鐮村潖浠讳綍鏃ц皟鐢ㄣ€?
fn resolve_sheet_source(args: &Value, tool: &str) -> Result<ResolvedSheetSource, ToolResponse> {
    if let Some(file_ref) = args.get("file_ref").and_then(|value| value.as_str()) {
        let Some(sheet_index) = args.get("sheet_index").and_then(|value| value.as_u64()) else {
            return Err(ToolResponse::error(format!(
                "{tool} 缂哄皯 sheet_index 鍙傛暟锛屾垨璇锋敼浼?path + sheet"
            )));
        };
        return resolve_sheet_source_from_file_ref(file_ref, sheet_index as usize)
            .map_err(ToolResponse::error);
    }

    let Some(path) = args.get("path").and_then(|value| value.as_str()) else {
        return Err(ToolResponse::error(format!(
            "{tool} 缂哄皯 path 鍙傛暟锛屾垨璇锋敼浼?file_ref + sheet_index / table_ref"
        )));
    };
    let Some(sheet) = args.get("sheet").and_then(|value| value.as_str()) else {
        return Err(ToolResponse::error(format!(
            "{tool} 缂哄皯 sheet 鍙傛暟锛屾垨璇锋敼浼?file_ref + sheet_index / table_ref"
        )));
    };

    Ok(ResolvedSheetSource {
        path: path.to_string(),
        sheet_name: sheet.to_string(),
    })
}

// 2026-03-23: 杩欓噷鎸?file_ref + sheet_index 鎭㈠鐪熷疄鍗曡〃鏉ユ簮锛屽師鍥犳槸澶栧眰閾捐矾涓嶇ǔ瀹氫紶閫掍腑鏂?Sheet 鍚嶏紱鐩殑鏄妸涓枃鍚嶇О鎭㈠鐣欏湪 Rust 杩涚▼鍐呴儴瀹屾垚銆?
fn resolve_sheet_source_from_file_ref(
    file_ref: &str,
    sheet_index: usize,
) -> Result<ResolvedSheetSource, String> {
    let store = SourceFileRefStore::workspace_default().map_err(|error| error.to_string())?;
    let persisted = store.load(file_ref).map_err(|error| error.to_string())?;
    persisted
        .validate_source_unchanged()
        .map_err(|error| error.to_string())?;
    let sheet_name = persisted
        .sheet_name_for_index(sheet_index)
        .map_err(|error| error.to_string())?;

    Ok(ResolvedSheetSource {
        path: persisted.working_path,
        sheet_name,
    })
}

fn build_inference_response(sheet: &str, inference: HeaderInference) -> ToolResponse {
    let payload = json!({
        "sheet": sheet,
        "confidence": confidence_label(&inference.confidence),
        "header_row_count": inference.header_row_count,
        "columns": inference.columns,
        "schema_state": infer_schema_state_label(&inference.schema_state),
    });

    if matches!(inference.confidence, ConfidenceLevel::High) {
        ToolResponse::ok(payload)
    } else {
        ToolResponse::needs_confirmation(payload)
    }
}

fn confidence_label(confidence: &ConfidenceLevel) -> &'static str {
    match confidence {
        ConfidenceLevel::High => "high",
        ConfidenceLevel::Medium => "medium",
        ConfidenceLevel::Low => "low",
    }
}

fn preview_loaded_table(loaded: &LoadedTable, limit: usize) -> ToolResponse {
    match preview_table(&loaded.dataframe, limit) {
        Ok(preview) => ToolResponse::ok(json!({
            "columns": preview.columns,
            "rows": preview.rows,
            "row_count": loaded.dataframe.height(),
        })),
        Err(error) => ToolResponse::error(error.to_string()),
    }
}

// 2026-03-22: 杩欓噷涓轰細浜х敓鏂拌〃鐨勫崟琛?Tool 缁熶竴闄勫姞 result_ref锛岀洰鐨勬槸璁╃敤鎴疯兘鐩存帴鎶婂綋鍓嶇粨鏋滄帴鍒颁笅涓€姝ュ垎鏋愰噷銆?
fn respond_with_preview_and_result_ref(
    tool_name: &str,
    args: &Value,
    loaded: &LoadedTable,
    limit: usize,
) -> ToolResponse {
    match preview_table(&loaded.dataframe, limit) {
        Ok(preview) => respond_with_result_dataset(
            tool_name,
            args,
            loaded,
            json!({
                "columns": preview.columns,
                "rows": preview.rows,
                "row_count": loaded.dataframe.height(),
            }),
        ),
        Err(error) => ToolResponse::error(error.to_string()),
    }
}

// 2026-03-22: 杩欓噷缁熶竴鎶婄粨鏋滆〃钀芥垚 result_ref 骞跺洖濉埌 JSON 鍝嶅簲锛岀洰鐨勬槸鎶娾€滅湅缁撴灉鈥濆拰鈥滅户缁鐢ㄧ粨鏋溾€濆悎骞舵垚鍚屼竴娆¤皟鐢ㄤ綋楠屻€?
fn respond_with_result_dataset(
    tool_name: &str,
    args: &Value,
    loaded: &LoadedTable,
    payload: Value,
) -> ToolResponse {
    let result_ref = match persist_result_dataset(tool_name, args, loaded) {
        Ok(result_ref) => result_ref,
        Err(response) => return response,
    };
    if let Err(response) = sync_output_handle_state(args, &result_ref, "result_ref", tool_name) {
        return response;
    }

    let mut object = match payload {
        Value::Object(object) => object,
        _ => {
            return ToolResponse::error(format!(
                "{tool_name} 杩斿洖缁撴灉涓嶆槸瀵硅薄锛屾棤娉曢檮鍔?result_ref"
            ));
        }
    };
    object.insert("result_ref".to_string(), Value::String(result_ref));
    ToolResponse::ok(Value::Object(object))
}

fn string_array<'a>(args: &'a Value, field: &str) -> Vec<&'a str> {
    args.get(field)
        .and_then(|value| value.as_array())
        .map(|items| {
            items
                .iter()
                .filter_map(|value| value.as_str())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
}

// 2026-03-23: 杩欓噷鍏煎鍗曢敭涓庡鍚堥敭鍙傛暟锛岀洰鐨勬槸璁╂棫鐨?`*_on` 璋冪敤涓嶇牬鍧忥紝鍚屾椂鍏佽鏂伴摼璺樉寮忎紶澶氬垪閿€?
fn parse_lookup_key_args(
    args: &Value,
    single_field: &str,
    multi_field: &str,
    tool: &str,
) -> Result<Vec<String>, ToolResponse> {
    let single_value = args.get(single_field).and_then(|value| value.as_str());
    let multi_values = args.get(multi_field).and_then(|value| value.as_array());

    if single_value.is_some() && multi_values.is_some() {
        return Err(ToolResponse::error(format!(
            "{tool} 涓嶈鍚屾椂浼?{single_field} 鍜?{multi_field}"
        )));
    }

    if let Some(value) = single_value {
        if value.trim().is_empty() {
            return Err(ToolResponse::error(format!(
                "{tool} 缂哄皯 {single_field} 鍙傛暟"
            )));
        }
        return Ok(vec![value.to_string()]);
    }

    if let Some(values) = multi_values {
        let keys = values
            .iter()
            .filter_map(|value| value.as_str())
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
            .collect::<Vec<_>>();
        if keys.is_empty() {
            return Err(ToolResponse::error(format!(
                "{tool} 缂哄皯 {multi_field} 鍙傛暟"
            )));
        }
        return Ok(keys);
    }

    Err(ToolResponse::error(format!(
        "{tool} 缂哄皯 {single_field} 鍙傛暟"
    )))
}

fn parse_casts(args: &Value, field: &str, tool: &str) -> Result<Vec<CastColumnSpec>, ToolResponse> {
    match args.get(field) {
        Some(casts_value) => serde_json::from_value::<Vec<CastColumnSpec>>(casts_value.clone())
            .map_err(|error| {
                ToolResponse::error(format!("{tool} 鐨?{field} 鍙傛暟瑙ｆ瀽澶辫触: {error}"))
            }),
        None => Ok(Vec::new()),
    }
}

fn parse_missing_strategy(args: &Value, tool: &str) -> Result<MissingStrategy, ToolResponse> {
    match args.get("missing_strategy") {
        Some(strategy_value) => serde_json::from_value::<MissingStrategy>(strategy_value.clone())
            .map_err(|error| {
                ToolResponse::error(format!(
                    "{tool} 鐨?missing_strategy 鍙傛暟瑙ｆ瀽澶辫触: {error}"
                ))
            }),
        None => Ok(MissingStrategy::DropRows),
    }
}

fn apply_optional_casts(
    loaded: LoadedTable,
    casts: &[CastColumnSpec],
) -> Result<LoadedTable, String> {
    if casts.is_empty() {
        Ok(loaded)
    } else {
        cast_column_types(&loaded, casts).map_err(|error| error.to_string())
    }
}

// 2026-03-22: 杩欓噷缁熶竴鎸佷箙鍖栦腑闂寸粨鏋滐紝鐩殑鏄琛ㄥ鐞嗚緭鍑哄彲浠ヤ笉渚濊禆鍘熷 Excel 鍐嶆璇诲彇灏辫繘鍏ヤ笅涓€姝ュ垎鏋愩€?
fn persist_result_dataset(
    tool_name: &str,
    args: &Value,
    loaded: &LoadedTable,
) -> Result<String, ToolResponse> {
    let result_ref = ResultRefStore::create_result_ref();
    let store = ResultRefStore::workspace_default()
        .map_err(|error| ToolResponse::error(error.to_string()))?;
    let record = PersistedResultDataset::from_dataframe(
        &result_ref,
        tool_name,
        source_refs_from_args(args),
        &loaded.dataframe,
    )
    .map_err(|error| ToolResponse::error(error.to_string()))?;
    store
        .save(&record)
        .map_err(|error| ToolResponse::error(error.to_string()))?;
    Ok(result_ref)
}

// 2026-03-22: 杩欓噷缁熶竴鎶藉彇褰撳墠杈撳叆鏉ユ簮锛岀洰鐨勬槸璁?result_ref 鑳芥妸鍗曡〃鍜屽琛ㄩ摼璺噷鐨勬墍鏈変笂娓稿彞鏌勯兘璁板畬鏁淬€?
fn source_refs_from_args(args: &Value) -> Vec<String> {
    let mut refs = Vec::<String>::new();
    collect_source_refs(args, &mut refs);
    refs
}

// 2026-03-22: 杩欓噷閫掑綊鎶藉彇璇锋眰閲岀殑鏉ユ簮鍙ユ焺锛岀洰鐨勬槸璁?join/append 鐢熸垚鐨勬柊缁撴灉涔熻兘淇濈暀宸﹀彸涓よ竟涔冭嚦娣峰悎鏉ユ簮鐨勮缂樸€?
fn collect_source_refs(value: &Value, refs: &mut Vec<String>) {
    match value {
        Value::Object(map) => {
            if let Some(result_ref) = map.get("result_ref").and_then(|item| item.as_str()) {
                push_unique_source_ref(refs, result_ref.to_string());
            }
            if let Some(table_ref) = map.get("table_ref").and_then(|item| item.as_str()) {
                push_unique_source_ref(refs, table_ref.to_string());
            }
            if let (Some(path), Some(sheet)) = (
                map.get("path").and_then(|item| item.as_str()),
                map.get("sheet").and_then(|item| item.as_str()),
            ) {
                push_unique_source_ref(refs, format!("{path}#{sheet}"));
            }
            if let Some(file_ref) = map.get("file_ref").and_then(|item| item.as_str()) {
                let source_ref = map
                    .get("sheet_index")
                    .and_then(|item| item.as_u64())
                    .map(|sheet_index| format!("{file_ref}#{sheet_index}"))
                    .unwrap_or_else(|| file_ref.to_string());
                push_unique_source_ref(refs, source_ref);
            }

            for child in map.values() {
                collect_source_refs(child, refs);
            }
        }
        Value::Array(items) => {
            for item in items {
                collect_source_refs(item, refs);
            }
        }
        Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_) => {}
    }
}

// 2026-03-22: 杩欓噷鍋氶『搴忎繚鐣欏幓閲嶏紝鐩殑鏄 source_refs 鍙涓旂ǔ瀹氾紝涓嶄細鍥犱负鍚屼竴涓潵婧愰噸澶嶅嚭鐜拌€屾薄鏌撹缂樺睍绀恒€?
fn push_unique_source_ref(refs: &mut Vec<String>, candidate: String) {
    if !refs.iter().any(|existing| existing == &candidate) {
        refs.push(candidate);
    }
}

// 2026-03-23: 杩欓噷閫掑綊鏌ユ壘璇锋眰閲岀殑绗竴涓?table_ref锛屽師鍥犳槸 append/join 杩欑被澶氳〃 Tool 鐨勭ǔ瀹氬洖婧愬彞鏌勭粡甯稿祵濂楀湪 top/left/source 閲岋紱鐩殑鏄浜у嚭鏂?result_ref 鍚庝粛鑳戒繚浣忔渶杩戠‘璁ゆ€?table_ref銆?
fn first_table_ref_in_value(value: &Value) -> Option<String> {
    match value {
        Value::Object(map) => {
            if let Some(table_ref) = map.get("table_ref").and_then(|item| item.as_str()) {
                return Some(table_ref.to_string());
            }
            map.values().find_map(first_table_ref_in_value)
        }
        Value::Array(items) => items.iter().find_map(first_table_ref_in_value),
        Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_) => None,
    }
}

// 2026-03-23: 杩欓噷鍥炲～涓よ〃宸ヤ綔娴佸缓璁噷鐨勬潵婧愰鏋讹紝鐩殑鏄涓婂眰鐩存帴鎵ц寤鸿璋冪敤鏃剁户缁部鐢?table_ref/result_ref锛岃€屼笉鏄洖閫€鍒板師濮嬭矾寰勩€?
fn rewrite_workflow_suggested_tool_call_sources(
    payload: &mut Value,
    left_source: Value,
    right_source: Value,
) {
    let Some(suggested_args) = payload
        .get_mut("suggested_tool_call")
        .and_then(|call| call.get_mut("args"))
        .and_then(Value::as_object_mut)
    else {
        return;
    };

    if suggested_args.contains_key("top") && suggested_args.contains_key("bottom") {
        suggested_args.insert("top".to_string(), left_source);
        suggested_args.insert("bottom".to_string(), right_source);
        return;
    }

    if suggested_args.contains_key("left") && suggested_args.contains_key("right") {
        suggested_args.insert("left".to_string(), left_source);
        suggested_args.insert("right".to_string(), right_source);
    }
}

// 2026-03-23: 杩欓噷鍥炲～澶氳〃璁″垝鍣ㄦ瘡涓€姝ョ殑鏉ユ簮楠ㄦ灦锛岀洰鐨勬槸璁╄鍒掗噷鐨?suggested_tool_call 鑳戒繚鐣欏師濮嬫潵婧愮被鍨嬪苟缁х画寮曠敤 step_n_result銆?
fn rewrite_multi_table_plan_suggested_tool_call_sources(
    payload: &mut Value,
    source_payloads: &BTreeMap<String, Value>,
) {
    let Some(steps) = payload.get_mut("steps").and_then(Value::as_array_mut) else {
        return;
    };

    for step in steps {
        let Some(action) = step
            .get("action")
            .and_then(Value::as_str)
            .map(|value| value.to_string())
        else {
            continue;
        };
        let input_refs = step
            .get("input_refs")
            .and_then(Value::as_array)
            .map(|items| {
                items
                    .iter()
                    .filter_map(Value::as_str)
                    .map(|item| item.to_string())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        if input_refs.len() < 2 {
            continue;
        }

        let Some(suggested_args) = step
            .get_mut("suggested_tool_call")
            .and_then(|call| call.get_mut("args"))
            .and_then(Value::as_object_mut)
        else {
            continue;
        };

        let first_source = planned_source_payload(&input_refs[0], source_payloads);
        let second_source = planned_source_payload(&input_refs[1], source_payloads);

        match action.as_str() {
            "append_tables" => {
                suggested_args.insert("top".to_string(), first_source);
                suggested_args.insert("bottom".to_string(), second_source);
            }
            "join_tables" => {
                suggested_args.insert("left".to_string(), first_source);
                suggested_args.insert("right".to_string(), second_source);
            }
            _ => {}
        }
    }
}

// 2026-03-23: 杩欓噷鎶?alias 鍜?step_n_result 缁熶竴缈昏瘧鎴愬疄闄呮潵婧愰鏋讹紝鐩殑鏄澶氳〃璁″垝姝ラ鏃㈣兘鍥炴寚鍘熻〃锛屼篃鑳藉洖鎸囧墠涓€姝ョ粨鏋溿€?
fn planned_source_payload(reference: &str, source_payloads: &BTreeMap<String, Value>) -> Value {
    source_payloads
        .get(reference)
        .cloned()
        .unwrap_or_else(|| json!({ "result_ref": reference }))
}

// 2026-03-22: 杩欓噷缁熶竴鍒涘缓鏈湴璁板繂灞傚叆鍙ｏ紝鐩殑鏄 dispatcher 鎵€鏈変細璇濈姸鎬佽鍐欏叡浜悓涓€濂楄矾寰勮В鏋愪笌閿欒鍑哄彛銆?
fn memory_runtime() -> Result<LocalMemoryRuntime, ToolResponse> {
    LocalMemoryRuntime::workspace_default().map_err(|error| ToolResponse::error(error.to_string()))
}

// 2026-03-22: 杩欓噷缁熶竴瑙ｆ瀽 session_id锛岀洰鐨勬槸璁╂樉寮忓浼氳瘽鍜岄粯璁ゅ崟浼氳瘽涓ょ妯″紡閮借兘澶嶇敤鍚屼竴濂楀叆鍙ｃ€?
fn session_id_from_args(args: &Value) -> String {
    args.get("session_id")
        .and_then(|value| value.as_str())
        .unwrap_or("default")
        .to_string()
}

// 2026-03-22: 杩欓噷缁熶竴鑾峰彇鐢ㄦ埛鐩爣鍏滃簳鍊硷紝鐩殑鏄鎬诲叆鍙ｆ樉寮忓啓鍏ュ拰鍏抽敭 Tool 鑷姩鍚屾閮借兘淇濈暀鍙В閲婄殑鐩爣鎽樿銆?
fn user_goal_from_args(args: &Value, fallback: &str) -> String {
    args.get("user_goal")
        .and_then(|value| value.as_str())
        .unwrap_or(fallback)
        .to_string()
}

// 2026-03-22: 杩欓噷缁熶竴鎻愬彇褰撳墠璇锋眰鐨勫叧蹇冨垪锛岀洰鐨勬槸璁╁垎鏋愬缓妯″眰鍜屽喅绛栧眰閮借兘鎶婂垪涓婁笅鏂囧悓姝ュ埌鏈湴璁板繂灞傘€?
fn selected_columns_from_args(args: &Value) -> Option<Vec<String>> {
    let mut selected = Vec::<String>::new();

    if let Some(columns) = args.get("columns").and_then(|value| value.as_array()) {
        selected.extend(
            columns
                .iter()
                .filter_map(|value| value.as_str())
                .map(|value| value.to_string()),
        );
    }

    if let Some(features) = args.get("features").and_then(|value| value.as_array()) {
        selected.extend(
            features
                .iter()
                .filter_map(|value| value.as_str())
                .map(|value| value.to_string()),
        );
    }

    if let Some(target) = args.get("target").and_then(|value| value.as_str()) {
        if !selected.iter().any(|column| column == target) {
            selected.push(target.to_string());
        }
    }

    if selected.is_empty() {
        None
    } else {
        Some(selected)
    }
}

// 2026-03-22: 杩欓噷闆嗕腑鍚屾纭鎬?table_ref 涓庝細璇濇憳瑕侊紝鐩殑鏄妸琛ㄥ鐞嗗眰寤虹珛鐨?confirmed 鐘舵€佺洿鎺ユ矇娣€鍒版湰鍦扮嫭绔嬭蹇嗗眰銆?
fn sync_confirmed_table_state(
    args: &Value,
    persisted: &PersistedTableRef,
    fallback_goal: &str,
) -> Result<(), ToolResponse> {
    let runtime = memory_runtime()?;
    let session_id = session_id_from_args(args);
    runtime
        .mirror_table_ref(persisted)
        .map_err(|error| ToolResponse::error(error.to_string()))?;
    runtime
        .update_session_state(
            &session_id,
            &SessionStatePatch {
                current_workbook: Some(current_workbook_for_session(args, &persisted.source_path)),
                current_sheet: Some(persisted.sheet_name.clone()),
                current_file_ref: current_file_ref_from_args(args),
                current_sheet_index: current_sheet_index_from_args(args),
                current_stage: Some(SessionStage::TableProcessing),
                schema_status: Some(SchemaStatus::Confirmed),
                active_table_ref: Some(persisted.table_ref.clone()),
                active_handle_ref: None,
                active_handle_kind: None,
                last_user_goal: Some(user_goal_from_args(args, fallback_goal)),
                selected_columns: Some(persisted.columns.clone()),
            },
        )
        .map_err(|error| ToolResponse::error(error.to_string()))?;
    runtime
        .append_event(
            &session_id,
            &EventLogInput {
                event_type: "schema_confirmed".to_string(),
                stage: Some(SessionStage::TableProcessing),
                tool_name: Some("apply_header_schema".to_string()),
                status: "ok".to_string(),
                // 2026-03-24: 这里顺手收口同一文件中本轮碰到的另一处坏字符串，原因是它同样缺失 closing quote；目的是避免后续验证再次被历史编码污染阻塞。
                message: Some("确认表头后已激活 table_ref".to_string()),
            },
        )
        .map_err(|error| ToolResponse::error(error.to_string()))?;
    Ok(())
}

// 2026-03-22: 杩欓噷闆嗕腑鍚屾鍒嗘瀽/鍐崇瓥灞傜殑浼氳瘽鐘舵€侊紝鐩殑鏄鎬诲叆鍙ｄ笅涓€杞兘鐩存帴鍒ゆ柇鐢ㄦ埛宸茬粡澶勫湪鍝釜灞傜骇銆?
fn sync_loaded_table_state(
    args: &Value,
    loaded: &LoadedTable,
    stage: SessionStage,
    fallback_goal: &str,
    tool_name: &str,
    event_type: &str,
) -> Result<(), ToolResponse> {
    let runtime = memory_runtime()?;
    let session_id = session_id_from_args(args);
    let current_state = runtime
        .get_session_state(&session_id)
        .map_err(|error| ToolResponse::error(error.to_string()))?;
    let input_handle_ref = active_handle_ref_from_args(args)
        .or_else(|| first_table_ref_in_value(args))
        .or_else(|| current_state.active_handle_ref.clone());
    let stable_table_ref =
        first_table_ref_in_value(args).or(current_state.active_table_ref.clone());
    runtime
        .update_session_state(
            &session_id,
            &SessionStatePatch {
                current_workbook: Some(current_workbook_for_session(
                    args,
                    loaded.handle.source_path(),
                )),
                current_sheet: Some(loaded.handle.sheet_name().to_string()),
                current_file_ref: current_file_ref_from_args(args),
                current_sheet_index: current_sheet_index_from_args(args),
                current_stage: Some(stage.clone()),
                schema_status: Some(SchemaStatus::Confirmed),
                // 2026-03-23: 杩欓噷淇濈暀绋冲畾 table_ref锛屽師鍥犳槸绾鍙栫被 Tool 涓嶅簲鎶婄‘璁ゆ€佸洖婧愬彞鏌勮鐩栨垚 result_ref锛涚洰鐨勬槸璁?session_state 鍚屾椂璁颁綇鈥滅ǔ瀹氭潵婧愨€濆拰鈥滃綋鍓嶈鍙栧璞♀€濄€?
                active_table_ref: stable_table_ref,
                // 2026-03-23: 杩欓噷鏄惧紡鍚屾褰撳墠杈撳叆鍙ユ焺锛屽師鍥犳槸鍒嗘瀽/鍐崇瓥绫?Tool 闇€瑕佽涓婂眰鐭ラ亾鏈疆瀹為檯娑堣垂鐨勬槸鍝釜瀵硅薄锛涚洰鐨勬槸閬垮厤璇?result_ref 鍚庝細璇濅粛閿欒鍋滅暀鍦ㄦ棫 table_ref銆?
                active_handle_ref: input_handle_ref.clone(),
                // 2026-03-23: 杩欓噷鍚屾杈撳叆鍙ユ焺绫诲瀷锛屽師鍥犳槸褰撳墠璇诲彇瀵硅薄鍙兘鏄?table_ref 鎴?result_ref锛涚洰鐨勬槸璁╃姸鎬佹憳瑕佷笉鐢ㄥ啀闈犱笂灞傞噸澶嶇寽娴嬪彞鏌勭被鍒€?
                active_handle_kind: input_handle_ref
                    .as_deref()
                    .map(classify_handle_kind)
                    .map(str::to_string),
                last_user_goal: Some(user_goal_from_args(args, fallback_goal)),
                selected_columns: selected_columns_from_args(args),
            },
        )
        .map_err(|error| ToolResponse::error(error.to_string()))?;
    runtime
        .append_event(
            &session_id,
            &EventLogInput {
                event_type: event_type.to_string(),
                stage: Some(stage),
                tool_name: Some(tool_name.to_string()),
                status: "ok".to_string(),
                // 2026-03-24: 这里修复事件消息模板，原因是 closing quote 缺失会直接破坏尾部辅助函数解析；目的是让输出句柄同步日志重新稳定。
                message: Some(format!("{tool_name} 已同步当前层级状态")),
            },
        )
        .map_err(|error| ToolResponse::error(error.to_string()))?;
    Ok(())
}

// 2026-03-22: 杩欓噷缁熶竴鎶藉彇褰撳墠婵€娲诲彞鏌勶紝鐩殑鏄浼氳瘽鐘舵€佽嚦灏戣兘璁颁綇鏈疆鏄粠 table_ref 杩樻槸 result_ref 缁х画涓嬫潵鐨勩€?
fn active_handle_ref_from_args(args: &Value) -> Option<String> {
    args.get("result_ref")
        .and_then(|value| value.as_str())
        .or_else(|| args.get("table_ref").and_then(|value| value.as_str()))
        .map(|value| value.to_string())
}

// 2026-03-23: 杩欓噷缁熶竴鍦ㄤ骇鍑烘柊鍙ユ焺鍚庡悓姝ヤ細璇濈姸鎬侊紝鍘熷洜鏄柟妗圔瑕佹眰鏈€鏂?result_ref/workbook_ref 鎴愪负褰撳墠婵€娲诲璞★紱鐩殑鏄妸鈥滄垚鍔熻繑鍥炵粨鏋溾€濆拰鈥滄洿鏂版湰鍦颁細璇濇憳瑕佲€濇敹鍙ｅ埌鍚屼竴涓嚭鍙ｃ€?
fn sync_output_handle_state(
    args: &Value,
    handle_ref: &str,
    handle_kind: &str,
    tool_name: &str,
) -> Result<(), ToolResponse> {
    let runtime = memory_runtime()?;
    let session_id = session_id_from_args(args);
    let current_state = runtime
        .get_session_state(&session_id)
        .map_err(|error| ToolResponse::error(error.to_string()))?;
    let stable_table_ref =
        first_table_ref_in_value(args).or(current_state.active_table_ref.clone());

    runtime
        .update_session_state(
            &session_id,
            &SessionStatePatch {
                current_workbook: None,
                current_sheet: None,
                current_file_ref: None,
                current_sheet_index: None,
                current_stage: None,
                schema_status: None,
                active_table_ref: stable_table_ref,
                active_handle_ref: Some(handle_ref.to_string()),
                active_handle_kind: Some(handle_kind.to_string()),
                last_user_goal: None,
                selected_columns: None,
            },
        )
        .map_err(|error| ToolResponse::error(error.to_string()))?;
    runtime
        .append_event(
            &session_id,
            &EventLogInput {
                event_type: "active_handle_updated".to_string(),
                stage: None,
                tool_name: Some(tool_name.to_string()),
                status: "ok".to_string(),
                message: Some(format!("{tool_name} 宸插悓姝ユ渶鏂?{handle_kind}")),
            },
        )
        .map_err(|error| ToolResponse::error(error.to_string()))?;
    Ok(())
}

// 2026-03-23: 杩欓噷浠?file_ref 鍏ュ弬閲屾娊鍑哄綋鍓嶅伐浣滅翱鍙ユ焺锛岀洰鐨勬槸璁╁悇涓?Tool 閮借兘缁熶竴澶嶇敤浼氳瘽涓婁笅鏂囬噷鐨勫綋鍓嶆枃浠跺畾浣嶃€?
fn current_file_ref_from_args(args: &Value) -> Option<String> {
    args.get("file_ref")
        .and_then(|value| value.as_str())
        .map(|value| value.to_string())
}

// 2026-03-23: 杩欓噷浠庘€滅鍑犱釜 Sheet鈥濆叆鍙傞噷鎶藉彇 file_ref 瀵瑰簲鐨勫伐浣滆〃绱㈠紩锛岀洰鐨勬槸璁╁悗缁祦绋嬪彲浠ヤ笉閲嶅浼?Sheet 鍚嶇О銆?
fn current_sheet_index_from_args(args: &Value) -> Option<usize> {
    args.get("sheet_index")
        .and_then(|value| value.as_u64())
        .map(|value| value as usize)
}

// 2026-03-23: 杩欓噷浼樺厛浠?file_ref 鍥炲～褰撳墠宸ヤ綔绨胯矾寰勶紝鐩殑鏄浼氳瘽鐘舵€佸湪涓枃璺緞鍜?ASCII 鍓湰涔嬮棿閮借兘淇濇寔绋冲畾鍥炴斁銆?
fn current_workbook_for_session(args: &Value, fallback_path: &str) -> String {
    let Some(file_ref) = current_file_ref_from_args(args) else {
        return fallback_path.to_string();
    };
    let Ok(store) = SourceFileRefStore::workspace_default() else {
        return fallback_path.to_string();
    };
    let Ok(record) = store.load(&file_ref) else {
        return fallback_path.to_string();
    };
    record.original_path
}
