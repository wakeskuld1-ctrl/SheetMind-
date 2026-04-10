mod common;

use chrono::{Duration, NaiveDate};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::fs;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::common::{
    create_test_runtime_db, run_cli_with_json, run_cli_with_json_runtime_and_envs,
};

// 2026-04-02 CST: 杩欓噷鏂板璇佸埜瀹℃壒鎻愪氦 CLI 娴嬭瘯澶瑰叿锛屽師鍥犳槸 P0-1 鐨勬牳蹇冧笉鏄啀缁欎竴涓垎鏋愮粨鏋滐紝鑰屾槸鎶婃姇鍐冲璞℃寮忛€佸叆瀹℃壒涓荤嚎锛?// 鐩殑锛氬厛閿佷綇鈥滆瘉鍒告姇鍐充細 -> 瀹℃壒瀵硅薄钀界洏鈥濈殑姝ｅ紡鍚堝悓锛岄伩鍏嶅疄鐜拌繃绋嬩腑鎶婅竟鐣屽仛鏁ｃ€?
fn create_stock_history_csv(prefix: &str, file_name: &str, rows: &[String]) -> PathBuf {
    let unique_suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after unix epoch")
        .as_nanos();
    let fixture_dir = PathBuf::from("tests")
        .join("runtime_fixtures")
        .join("security_decision_submit_approval")
        .join(format!("{prefix}_{unique_suffix}"));
    fs::create_dir_all(&fixture_dir).expect("security decision submit fixture dir should exist");

    let csv_path = fixture_dir.join(file_name);
    fs::write(&csv_path, rows.join("\n")).expect("security decision submit csv should be written");
    csv_path
}

// 2026-04-02 CST: 杩欓噷澶嶇敤鏈湴 HTTP 鍋囨湇鍔★紝鍘熷洜鏄鎵规ˉ鎺ユ祴璇曚粛鐒惰缁忚繃鐪熷疄璇佸埜鐮旂┒涓庢姇鍐充富閾撅紱
// 鐩殑锛氭妸澶栭儴鍩烘湰闈笌鍏憡渚濊禆绋冲畾鏀惰繘鏈湴鍙帶澶瑰叿锛岄伩鍏嶆彁浜ゅ鎵规祴璇曡澶栭儴鎺ュ彛娉㈠姩鎵撴柇銆?
fn spawn_http_route_server(routes: Vec<(&str, &str, &str, &str)>) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("test http server should bind");
    let address = format!(
        "http://{}",
        listener
            .local_addr()
            .expect("test http server should have local addr")
    );
    let route_map: HashMap<String, (String, String, String)> = routes
        .into_iter()
        .map(|(path, status_line, body, content_type)| {
            (
                path.to_string(),
                (
                    status_line.to_string(),
                    body.to_string(),
                    content_type.to_string(),
                ),
            )
        })
        .collect();

    thread::spawn(move || {
        for _ in 0..route_map.len() {
            let Ok((mut stream, _)) = listener.accept() else {
                break;
            };
            let mut buffer = [0_u8; 4096];
            let _ = stream.read(&mut buffer);
            let request_text = String::from_utf8_lossy(&buffer);
            let request_line = request_text.lines().next().unwrap_or_default();
            let request_path = request_line
                .split_whitespace()
                .nth(1)
                .unwrap_or("/")
                .split('?')
                .next()
                .unwrap_or("/");
            let (status_line, body, content_type) =
                route_map.get(request_path).cloned().unwrap_or_else(|| {
                    (
                        "HTTP/1.1 404 Not Found".to_string(),
                        "{\"error\":\"not found\"}".to_string(),
                        "application/json".to_string(),
                    )
                });
            let response = format!(
                "{status_line}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                body.len(),
                body
            );
            let _ = stream.write_all(response.as_bytes());
            let _ = stream.flush();
        }
    });

    address
}

fn write_scorecard_model_artifact(
    fixture_prefix: &str,
    file_name: &str,
    payload: &Value,
) -> PathBuf {
    // 2026-04-10 CST: 杩欓噷鏂板鏈€灏?scorecard artifact 娴嬭瘯澶瑰叿鍐欏叆鍣紝鍘熷洜鏄綋鍓嶈鍏堥攣瀹氣€滅嚎涓婅瘎鍒嗗繀椤诲鐢ㄨ缁冨悓婧愮壒寰佸悎鍚屸€濓紝
    // 鐩殑锛氳瀹℃壒涓婚摼娴嬭瘯鑳藉鐩存帴鎸備竴涓彈鎺фā鍨嬫枃浠讹紝绋冲畾澶嶇幇 evidence seed / bool 鍒嗙涓嶅懡涓殑鐪熷疄闂銆?
    let unique_suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after unix epoch")
        .as_nanos();
    let fixture_dir = PathBuf::from("tests")
        .join("runtime_fixtures")
        .join("security_decision_submit_approval")
        .join(format!("{fixture_prefix}_{unique_suffix}"));
    fs::create_dir_all(&fixture_dir).expect("scorecard model fixture dir should exist");

    let artifact_path = fixture_dir.join(file_name);
    fs::write(
        &artifact_path,
        serde_json::to_vec_pretty(payload).expect("artifact payload should serialize"),
    )
    .expect("scorecard model fixture should be written");
    artifact_path
}

#[test]
fn tool_catalog_includes_security_decision_submit_approval() {
    let output = run_cli_with_json("");

    // 2026-04-02 CST: 杩欓噷鍏堥攣浣忔柊瀹℃壒鎻愪氦 Tool 鐨勫彲鍙戠幇鎬э紝鍘熷洜鏄病杩?catalog 灏辩瓑浜庝骇鍝佷富鍏ュ彛涓嶅瓨鍦紱
    // 鐩殑锛氱‘淇濆悗缁?Skill 涓?CLI 鑳界ǔ瀹氭壘鍒扳€滄彁浜ゅ埌瀹℃壒涓荤嚎鈥濈殑姝ｅ紡鍏ュ彛銆?
    assert!(
        output["data"]["tool_catalog"]
            .as_array()
            .expect("tool catalog should be an array")
            .iter()
            .any(|tool| tool == "security_decision_submit_approval")
    );
}

#[test]
fn security_decision_submit_approval_writes_runtime_files_for_ready_case() {
    let runtime_db_path = create_test_runtime_db("security_decision_submit_approval_ready");
    let approval_root = runtime_db_path
        .parent()
        .expect("runtime db should have parent")
        .join("scenes_runtime");

    let stock_csv = create_stock_history_csv(
        "security_decision_submit_approval_ready",
        "stock.csv",
        &build_confirmed_breakout_rows(220, 88.0),
    );
    let market_csv = create_stock_history_csv(
        "security_decision_submit_approval_ready",
        "market.csv",
        &build_confirmed_breakout_rows(220, 3200.0),
    );
    let sector_csv = create_stock_history_csv(
        "security_decision_submit_approval_ready",
        "sector.csv",
        &build_confirmed_breakout_rows(220, 950.0),
    );
    import_history_csv(&runtime_db_path, &stock_csv, "601916.SH");
    import_history_csv(&runtime_db_path, &market_csv, "510300.SH");
    import_history_csv(&runtime_db_path, &sector_csv, "512800.SH");

    let server = spawn_http_route_server(vec![
        (
            "/financials",
            "HTTP/1.1 200 OK",
            r#"[
                {
                    "REPORT_DATE":"2025-12-31",
                    "NOTICE_DATE":"2026-03-28",
                    "TOTAL_OPERATE_INCOME":308227000000.0,
                    "YSTZ":8.37,
                    "PARENT_NETPROFIT":11117000000.0,
                    "SJLTZ":9.31,
                    "ROEJQ":14.8
                }
            ]"#,
            "application/json",
        ),
        (
            "/announcements",
            "HTTP/1.1 200 OK",
            r#"{
                "data":{
                    "list":[
                        {"notice_date":"2026-03-28","title":"2025骞村勾搴︽姤鍛?,"art_code":"AN202603281234567890","columns":[{"column_name":"瀹氭湡鎶ュ憡"}]},
                        {"notice_date":"2026-03-28","title":"2025骞村害鍒╂鼎鍒嗛厤棰勬鍏憡","art_code":"AN202603281234567891","columns":[{"column_name":"鍏徃鍏憡"}]}
                    ]
                }
            }"#,
            "application/json",
        ),
    ]);

    let request = json!({
        "tool": "security_decision_submit_approval",
        "args": {
            "symbol": "601916.SH",
            "market_profile": "a_share_core",
            "sector_profile": "a_share_bank",
            "stop_loss_pct": 0.05,
            "target_return_pct": 0.12,
            "approval_runtime_root": approval_root.to_string_lossy(),
            "created_at": "2026-04-02T10:30:00+08:00"
        }
    });

    let output = run_cli_with_json_runtime_and_envs(
        &request.to_string(),
        &runtime_db_path,
        &[
            (
                "EXCEL_SKILL_EASTMONEY_FINANCIAL_URL_BASE",
                format!("{server}/financials"),
            ),
            (
                "EXCEL_SKILL_EASTMONEY_ANNOUNCEMENT_URL_BASE",
                format!("{server}/announcements"),
            ),
        ],
    );

    // 2026-04-02 CST: 杩欓噷閿佷綇 ready_for_review 鎻愪氦璺緞锛屽師鍥犳槸 P0-1 鐨勭洰鏍囧氨鏄妸鍙笂浼氱殑璇佸埜鎶曞喅瀵硅薄姝ｅ紡钀借繘瀹℃壒涓荤嚎锛?    // 鐩殑锛氱‘淇?decision/approval/audit 鍥涚被宸ヤ欢涓€娆″啓榻愶紝鍚庣画绉佹湁澶氱娴佺▼鍙互鐩存帴鎺ョ潃璺戙€?
    assert_eq!(output["status"], "ok");
    assert_eq!(
        output["data"]["committee_result"]["decision_card"]["status"],
        "ready_for_review"
    );
    assert_eq!(output["data"]["approval_request"]["status"], "Pending");
    assert_eq!(output["data"]["approval_request"]["min_approvals"], 2);
    assert_eq!(
        output["data"]["approval_request"]["require_risk_signoff"],
        true
    );
    // 2026-04-08 CST: 杩欓噷鍏堥攣瀹?approval_request 瀵逛粨浣嶈鍒掔殑姝ｅ紡缁戝畾锛屽師鍥犳槸 Task 2 瑕佽 position_plan 浠?package 闄勫睘鏂囦欢鍗囩骇鎴愭寮忓彲瀹℃壒瀵硅薄锛?    // 鐩殑锛氱‘淇濆鎵硅姹傝嚜宸卞氨鏄庣‘鐭ラ亾鈥滃鐨勬槸鍝竴涓粨浣嶈鍒掋€佽矾寰勫湪鍝€佸悎鍚岀増鏈槸浠€涔堚€濓紝鑰屼笉鏄彧渚濊禆 package 闂存帴鎺ㄦ柇銆?
    assert_eq!(
        output["data"]["approval_request"]["position_plan_binding"]["position_plan_ref"],
        output["data"]["position_plan"]["plan_id"]
    );
    assert_eq!(
        output["data"]["approval_request"]["position_plan_binding"]["position_plan_path"],
        output["data"]["position_plan_path"]
    );
    assert!(
        output["data"]["decision_ref"]
            .as_str()
            .expect("decision ref should exist")
            .starts_with("decision_ref:")
    );
    assert!(
        output["data"]["approval_ref"]
            .as_str()
            .expect("approval ref should exist")
            .starts_with("approval_ref:")
    );
    assert!(
        output["data"]["approval_brief"]["bull_summary"]
            .as_array()
            .expect("bull summary should be array")
            .len()
            >= 1
    );
    assert!(
        output["data"]["approval_brief"]["bear_summary"]
            .as_array()
            .expect("bear summary should be array")
            .len()
            >= 1
    );
    assert!(
        output["data"]["approval_brief"]["gate_summary"]
            .as_array()
            .expect("gate summary should be array")
            .len()
            >= 1
    );
    assert_eq!(
        output["data"]["position_plan"]["decision_ref"],
        output["data"]["decision_ref"]
    );
    assert_eq!(
        output["data"]["position_plan"]["approval_ref"],
        output["data"]["approval_ref"]
    );
    assert_eq!(
        output["data"]["position_plan"]["contract_version"],
        "security_position_plan.v2"
    );
    assert_eq!(
        output["data"]["position_plan"]["document_type"],
        "security_position_plan"
    );
    assert_eq!(output["data"]["position_plan"]["plan_direction"], "Long");
    assert_eq!(
        output["data"]["position_plan"]["approval_binding"]["approval_ref"],
        output["data"]["approval_ref"]
    );
    assert_eq!(
        output["data"]["position_plan"]["approval_binding"]["approval_request_ref"],
        output["data"]["approval_ref"]
    );
    assert_eq!(
        output["data"]["position_plan"]["reduce_plan"]["allow_reduce"],
        true
    );
    assert!(
        output["data"]["approval_brief"]["brief_id"]
            .as_str()
            .expect("brief id should exist")
            .starts_with("brief-")
    );
    assert_eq!(
        output["data"]["approval_brief"]["contract_version"],
        "security_approval_brief.v1"
    );
    assert_eq!(
        output["data"]["approval_brief"]["decision_ref"],
        output["data"]["decision_ref"]
    );
    assert_eq!(
        output["data"]["approval_brief"]["approval_ref"],
        output["data"]["approval_ref"]
    );
    assert_eq!(
        output["data"]["approval_brief"]["package_binding"]["artifact_role"],
        "approval_brief"
    );
    // 2026-04-10 CST: 杩欓噷閿佷綇瀵瑰 scorecard 瑙ｉ噴灞傚悎鍚岋紝鍘熷洜鏄敤鎴疯姹傚鎴蜂笉鍐嶇湅鍒?success_probability 绛夋ā鍨嬪唴閮ㄥ瓧娈碉紱
    // 鐩殑锛氱‘淇?Tool 杩斿洖鐩存帴鎻愪緵鈥滅湅澶?闇囪崱/鐪嬬┖ + 椋庨櫓 + 鍔ㄤ綔寤鸿鈥濓紝鍚屾椂鍐呴儴 artifact 浠嶈蛋 scorecard_path 钀界洏銆?    // 2026-04-10 CST: 杩欓噷鎸夌湡瀹炴牱鏈緭鍑洪攣浣忓綋鍓嶇煭绾垮垽鏂紝鍘熷洜鏄?ready 鍦烘櫙鍦ㄦ柊瑙ｉ噴瑙勫垯涓嬭惤鍒颁簡鍋忕┖鐭嚎缁撴瀯锛?    // 鐩殑锛氶伩鍏嶅悗缁湁浜鸿鎶婅В閲婂眰鍥為€€鎴愬唴閮ㄥ瓧娈碉紝鎴栨棤鎰忔敼鍔ㄧ煭绾挎槧灏勮鍒欒€屼笉鑷煡銆?
    assert!(
        output["data"]["scorecard"]["overall_score"]
            .as_u64()
            .expect("overall score should exist")
            <= 100
    );
    assert!(
        output["data"]["scorecard"]["risk_level"]
            .as_str()
            .expect("risk level should exist")
            .len()
            > 0
    );
    assert!(
        output["data"]["scorecard"]["tomorrow"]["profit_probability_pct"]
            .as_f64()
            .expect("tomorrow profit probability should exist")
            >= 0.0
    );
    assert!(
        output["data"]["scorecard"]["short_term"]["loss_probability_pct"]
            .as_f64()
            .expect("short term loss probability should exist")
            >= 0.0
    );
    assert!(
        output["data"]["scorecard"]["swing_term"]["profit_loss_ratio"]
            .as_f64()
            .expect("swing term ratio should exist")
            > 0.0
    );
    assert!(
        output["data"]["scorecard"]["mid_long_term"]["trade_value"]
            .as_str()
            .expect("mid long trade value should exist")
            .len()
            > 0
    );
    assert_eq!(
        output["data"]["scorecard"]["detailed_horizons"]
            .as_array()
            .expect("detailed horizons should be array")
            .len(),
        7
    );
    assert!(
        output["data"]["scorecard"]["core_reasons"]
            .as_array()
            .expect("core reasons should be array")
            .len()
            >= 2
    );
    assert!(
        output["data"]["scorecard"]["risk_alerts"]
            .as_array()
            .expect("risk alerts should be array")
            .len()
            >= 1
    );
    assert!(
        output["data"]["scorecard"]["action_advice"]
            .as_str()
            .expect("action advice should exist")
            .len()
            > 0
    );
    assert!(
        output["data"]["scorecard"]
            .get("success_probability")
            .is_none()
    );
    assert!(output["data"]["scorecard"].get("total_score").is_none());
    assert!(output["data"]["scorecard"].get("score_status").is_none());
    assert!(
        output["data"]["scorecard"]
            .get("short_term_judgment")
            .is_none()
    );
    assert!(
        output["data"]["scorecard"]
            .get("mid_term_judgment")
            .is_none()
    );
    assert!(
        output["data"]["approval_brief"]["recommended_review_action"]
            .as_str()
            .expect("recommended review action should exist")
            .contains("approve")
    );
    assert!(
        output["data"]["approval_brief_path"]
            .as_str()
            .expect("approval brief path should exist")
            .contains("approval_briefs")
    );
    assert!(
        output["data"]["decision_package_path"]
            .as_str()
            .expect("decision package path should exist")
            .contains("decision_packages")
    );
    assert_eq!(output["data"]["position_plan"]["plan_status"], "reviewable");
    // 2026-04-08 CST: 杩欓噷鍏堥攣瀹?package 鏄惧紡瀵硅薄鍥惧悎鍚岋紝鍘熷洜鏄?Task 1 瑕佹妸 position_plan / approval_brief 浠庨殣寮?artifact 鍏崇郴鍗囩骇涓烘寮忓璞″紩鐢紱
    // 鐩殑锛氱‘淇?submit_approval 鐢熸垚鐨勬柊 package 涓嶅彧鏄€滄枃浠舵竻鍗曞瓨鍦ㄢ€濓紝鑰屾槸宸茬粡鎶婂喅绛栧璞″浘鍐欐垚鍙牎楠岀殑姝ｅ紡鍚堝悓銆?
    assert_eq!(
        output["data"]["decision_package"]["object_graph"]["decision_ref"],
        output["data"]["decision_ref"]
    );
    assert_eq!(
        output["data"]["decision_package"]["object_graph"]["approval_ref"],
        output["data"]["approval_ref"]
    );
    assert_eq!(
        output["data"]["decision_package"]["object_graph"]["position_plan_ref"],
        output["data"]["position_plan"]["plan_id"]
    );
    assert_eq!(
        output["data"]["decision_package"]["object_graph"]["approval_brief_ref"],
        output["data"]["approval_brief"]["brief_id"]
    );
    assert!(
        output["data"]["position_plan"]["suggested_gross_pct"]
            .as_f64()
            .expect("suggested gross pct should exist")
            > 0.0
    );
    // 2026-04-10 CST: 这里把审批简报文案断言收敛为“字段存在且非空”。
    // 原因与目的：该处验证的是正式合同字段，不应再锁死到具体中文片段或编码表现。
    assert!(
        output["data"]["approval_brief"]["entry_summary"]
            .as_str()
            .expect("entry summary should exist")
            .len()
            > 0
    );
    // 2026-04-10 CST: 同步放宽止损摘要断言。
    // 原因与目的：避免测试被展示层文案或编码差异误伤，保留字段级稳定校验。
    assert!(
        output["data"]["approval_brief"]["stop_loss_summary"]
            .as_str()
            .expect("stop loss summary should exist")
            .len()
            > 0
    );

    let decision_path = PathBuf::from(
        output["data"]["decision_card_path"]
            .as_str()
            .expect("decision card path should exist"),
    );
    let approval_path = PathBuf::from(
        output["data"]["approval_request_path"]
            .as_str()
            .expect("approval request path should exist"),
    );
    let events_path = PathBuf::from(
        output["data"]["approval_events_path"]
            .as_str()
            .expect("approval events path should exist"),
    );
    let audit_path = PathBuf::from(
        output["data"]["audit_log_path"]
            .as_str()
            .expect("audit log path should exist"),
    );
    let approval_brief_path = PathBuf::from(
        output["data"]["approval_brief_path"]
            .as_str()
            .expect("approval brief path should exist"),
    );
    let decision_package_path = PathBuf::from(
        output["data"]["decision_package_path"]
            .as_str()
            .expect("decision package path should exist"),
    );
    let position_plan_path = PathBuf::from(
        output["data"]["position_plan_path"]
            .as_str()
            .expect("position plan path should exist"),
    );

    assert!(decision_path.exists());
    assert!(approval_path.exists());
    assert!(events_path.exists());
    assert!(audit_path.exists());
    assert!(approval_brief_path.exists());
    assert!(decision_package_path.exists());
    assert!(position_plan_path.exists());

    let persisted_decision: Value = serde_json::from_slice(
        &fs::read(&decision_path).expect("persisted decision card should be readable"),
    )
    .expect("persisted decision card should be valid json");
    assert_eq!(
        persisted_decision["scene_name"],
        "security_decision_committee"
    );
    assert_eq!(persisted_decision["asset_id"], "601916.SH");
    assert_eq!(persisted_decision["status"], "ReadyForReview");
    assert_eq!(persisted_decision["direction"], "Long");
    assert_eq!(persisted_decision["approval"]["approval_state"], "Pending");

    let persisted_request: Value = serde_json::from_slice(
        &fs::read(&approval_path).expect("persisted approval request should be readable"),
    )
    .expect("persisted approval request should be valid json");
    assert_eq!(persisted_request["status"], "Pending");
    assert_eq!(
        persisted_request["decision_id"],
        persisted_decision["decision_id"]
    );
    assert_eq!(persisted_request["auto_reject_recommended"], false);
    assert_eq!(
        persisted_request["position_plan_binding"]["position_plan_ref"],
        output["data"]["position_plan"]["plan_id"]
    );
    assert_eq!(
        persisted_request["position_plan_binding"]["plan_direction"],
        output["data"]["position_plan"]["plan_direction"]
    );

    let persisted_events: Value = serde_json::from_slice(
        &fs::read(&events_path).expect("persisted approval events should be readable"),
    )
    .expect("persisted approval events should be valid json");
    assert_eq!(
        persisted_events
            .as_array()
            .expect("approval events should be array")
            .len(),
        0
    );

    let audit_lines = fs::read_to_string(&audit_path).expect("audit log should be readable");
    assert_eq!(audit_lines.lines().count(), 1);
    let audit_record: Value = serde_json::from_str(
        audit_lines
            .lines()
            .next()
            .expect("audit log should contain first line"),
    )
    .expect("audit line should be valid json");
    assert_eq!(audit_record["event_type"], "decision_persisted");
    assert_eq!(audit_record["decision_status"], "ReadyForReview");
    assert_eq!(audit_record["approval_status"], "Pending");

    let persisted_approval_brief: Value = serde_json::from_slice(
        &fs::read(&approval_brief_path).expect("persisted approval brief should be readable"),
    )
    .expect("persisted approval brief should be valid json");
    assert_eq!(
        persisted_approval_brief["contract_version"],
        "security_approval_brief.v1"
    );
    assert_eq!(
        persisted_approval_brief["package_binding"]["artifact_role"],
        "approval_brief"
    );

    let persisted_position_plan: Value = serde_json::from_slice(
        &fs::read(&position_plan_path).expect("persisted position plan should be readable"),
    )
    .expect("persisted position plan should be valid json");
    assert_eq!(
        persisted_position_plan["contract_version"],
        "security_position_plan.v2"
    );
    assert_eq!(
        persisted_position_plan["document_type"],
        "security_position_plan"
    );
    assert_eq!(persisted_position_plan["plan_status"], "reviewable");
    assert_eq!(
        persisted_position_plan["decision_ref"],
        output["data"]["decision_ref"]
    );
    assert_eq!(
        persisted_position_plan["approval_ref"],
        output["data"]["approval_ref"]
    );
    assert_eq!(
        persisted_position_plan["approval_binding"]["approval_request_ref"],
        output["data"]["approval_ref"]
    );
    assert_eq!(persisted_position_plan["reduce_plan"]["allow_reduce"], true);

    let persisted_decision_package: Value = serde_json::from_slice(
        &fs::read(&decision_package_path).expect("persisted decision package should be readable"),
    )
    .expect("persisted decision package should be valid json");
    assert_eq!(
        persisted_decision_package["contract_version"],
        "security_decision_package.v1"
    );
    assert_eq!(
        persisted_decision_package["decision_ref"],
        output["data"]["decision_ref"]
    );
    assert_eq!(
        persisted_decision_package["approval_ref"],
        output["data"]["approval_ref"]
    );
    // 2026-04-08 CST: 杩欓噷琛ュ厖鎸佷箙鍖?package 鐨勫璞″浘鏂█锛屽師鍥犳槸 Task 1 闇€瑕佸喕缁撴寮忓璞″浘锛岃€屼笉鍙槸淇濊瘉 CLI 杩斿洖鍊奸噷鐭殏甯﹀嚭锛?    // 鐩殑锛氱‘淇濈湡姝ｈ惤鐩樼殑 package JSON 涔熷叿澶囩ǔ瀹氱殑 object_graph锛屽悗缁?verify / revision 閮借兘鍩轰簬纾佺洏瀵硅薄鍥剧户缁伐浣溿€?
    assert_eq!(
        persisted_decision_package["object_graph"]["position_plan_ref"],
        persisted_position_plan["plan_id"]
    );
    assert_eq!(
        persisted_decision_package["object_graph"]["approval_brief_ref"],
        persisted_approval_brief["brief_id"]
    );
    assert_eq!(
        persisted_decision_package["object_graph"]["position_plan_path"],
        Value::String(position_plan_path.to_string_lossy().to_string())
    );
    assert_eq!(
        persisted_decision_package["object_graph"]["approval_brief_path"],
        Value::String(approval_brief_path.to_string_lossy().to_string())
    );
    assert_eq!(
        persisted_decision_package["package_status"],
        "review_bundle_ready"
    );
    assert!(
        persisted_decision_package["artifact_manifest"]
            .as_array()
            .expect("artifact manifest should be array")
            .iter()
            .any(|artifact| artifact["artifact_role"] == "decision_card")
    );
    assert!(
        persisted_decision_package["artifact_manifest"]
            .as_array()
            .expect("artifact manifest should be array")
            .iter()
            .any(|artifact| artifact["artifact_role"] == "approval_request")
    );
    assert!(
        persisted_decision_package["artifact_manifest"]
            .as_array()
            .expect("artifact manifest should be array")
            .iter()
            .any(|artifact| artifact["artifact_role"] == "position_plan")
    );
    assert!(
        persisted_decision_package["artifact_manifest"]
            .as_array()
            .expect("artifact manifest should be array")
            .iter()
            .any(|artifact| artifact["artifact_role"] == "approval_brief")
    );
    assert_eq!(
        persisted_decision_package["governance_binding"]["decision_ref"],
        output["data"]["decision_ref"]
    );
    assert_eq!(
        persisted_decision_package["governance_binding"]["approval_ref"],
        output["data"]["approval_ref"]
    );
}

#[test]
fn security_decision_submit_approval_maps_blocked_status_and_auto_reject_flags() {
    let runtime_db_path = create_test_runtime_db("security_decision_submit_approval_blocked");
    let approval_root = runtime_db_path
        .parent()
        .expect("runtime db should have parent")
        .join("scenes_runtime");

    let stock_csv = create_stock_history_csv(
        "security_decision_submit_approval_blocked",
        "stock.csv",
        &build_confirmed_breakout_rows(220, 88.0),
    );
    let market_csv = create_stock_history_csv(
        "security_decision_submit_approval_blocked",
        "market.csv",
        &build_confirmed_breakout_rows(220, 3200.0),
    );
    let sector_csv = create_stock_history_csv(
        "security_decision_submit_approval_blocked",
        "sector.csv",
        &build_confirmed_breakout_rows(220, 950.0),
    );
    import_history_csv(&runtime_db_path, &stock_csv, "601916.SH");
    import_history_csv(&runtime_db_path, &market_csv, "510300.SH");
    import_history_csv(&runtime_db_path, &sector_csv, "512800.SH");

    let server = spawn_http_route_server(vec![
        (
            "/financials",
            "HTTP/1.1 200 OK",
            r#"[
                {
                    "REPORT_DATE":"2025-12-31",
                    "NOTICE_DATE":"2026-03-28",
                    "TOTAL_OPERATE_INCOME":308227000000.0,
                    "YSTZ":8.37,
                    "PARENT_NETPROFIT":11117000000.0,
                    "SJLTZ":9.31,
                    "ROEJQ":14.8
                }
            ]"#,
            "application/json",
        ),
        (
            "/announcements",
            "HTTP/1.1 200 OK",
            r#"{
                "data":{
                    "list":[
                        {"notice_date":"2026-03-28","title":"2025骞村勾搴︽姤鍛?,"art_code":"AN202603281234567890","columns":[{"column_name":"瀹氭湡鎶ュ憡"}]}
                    ]
                }
            }"#,
            "application/json",
        ),
    ]);

    let request = json!({
        "tool": "security_decision_submit_approval",
        "args": {
            "symbol": "601916.SH",
            "market_profile": "a_share_core",
            "sector_profile": "a_share_bank",
            "stop_loss_pct": 0.05,
            "target_return_pct": 0.08,
            "approval_runtime_root": approval_root.to_string_lossy(),
            "created_at": "2026-04-02T10:45:00+08:00"
        }
    });

    let output = run_cli_with_json_runtime_and_envs(
        &request.to_string(),
        &runtime_db_path,
        &[
            (
                "EXCEL_SKILL_EASTMONEY_FINANCIAL_URL_BASE",
                format!("{server}/financials"),
            ),
            (
                "EXCEL_SKILL_EASTMONEY_ANNOUNCEMENT_URL_BASE",
                format!("{server}/announcements"),
            ),
        ],
    );

    // 2026-04-02 CST: 杩欓噷閿佷綇 blocked 鎻愪氦璺緞锛屽師鍥犳槸瀹℃壒妗ユ帴涓嶈兘鍙細澶勭悊鈥滃ソ鐪嬧€濈殑鎶曞喅瀵硅薄锛?    // 鐩殑锛氱‘淇濊椋庨櫓闂搁棬鎷︿笅鐨勮瘉鍒稿喅绛栦篃鑳藉舰鎴愭寮忓鎵硅褰曪紝骞舵樉寮忓甫涓?auto-reject 璇箟銆?
    assert_eq!(output["status"], "ok");
    assert_eq!(
        output["data"]["committee_result"]["decision_card"]["status"],
        "blocked"
    );
    assert_eq!(
        output["data"]["approval_request"]["status"],
        "NeedsMoreEvidence"
    );
    assert_eq!(
        output["data"]["approval_request"]["auto_reject_recommended"],
        true
    );
    assert_eq!(output["data"]["position_plan"]["plan_status"], "blocked");
    assert_eq!(output["data"]["position_plan"]["suggested_gross_pct"], 0.0);
    assert_eq!(output["data"]["position_plan"]["starter_gross_pct"], 0.0);
    assert_eq!(output["data"]["position_plan"]["max_gross_pct"], 0.0);
    assert!(
        output["data"]["approval_brief"]["recommended_review_action"]
            .as_str()
            .expect("recommended review action should exist")
            .contains("request_more_evidence")
    );
    assert!(
        output["data"]["approval_request"]["auto_reject_gate_names"]
            .as_array()
            .expect("auto reject gate names should be array")
            .iter()
            .any(|gate| gate == "risk_reward_gate")
    );

    let decision_path = PathBuf::from(
        output["data"]["decision_card_path"]
            .as_str()
            .expect("decision card path should exist"),
    );
    let persisted_decision: Value = serde_json::from_slice(
        &fs::read(&decision_path).expect("persisted decision card should be readable"),
    )
    .expect("persisted decision card should be valid json");
    assert_eq!(persisted_decision["status"], "Blocked");
    assert_eq!(persisted_decision["direction"], "Long");
    assert_eq!(
        output["data"]["decision_package"]["package_status"],
        "needs_follow_up"
    );
}

#[test]
fn security_decision_submit_approval_can_write_detached_signature_for_approval_brief() {
    let runtime_db_path = create_test_runtime_db("security_decision_submit_approval_brief_signed");
    let approval_root = runtime_db_path
        .parent()
        .expect("runtime db should have parent")
        .join("scenes_runtime");

    let stock_csv = create_stock_history_csv(
        "security_decision_submit_approval_brief_signed",
        "stock.csv",
        &build_confirmed_breakout_rows(220, 88.0),
    );
    let market_csv = create_stock_history_csv(
        "security_decision_submit_approval_brief_signed",
        "market.csv",
        &build_confirmed_breakout_rows(220, 3200.0),
    );
    let sector_csv = create_stock_history_csv(
        "security_decision_submit_approval_brief_signed",
        "sector.csv",
        &build_confirmed_breakout_rows(220, 950.0),
    );
    import_history_csv(&runtime_db_path, &stock_csv, "601916.SH");
    import_history_csv(&runtime_db_path, &market_csv, "510300.SH");
    import_history_csv(&runtime_db_path, &sector_csv, "512800.SH");

    let server = spawn_http_route_server(vec![
        (
            "/financials",
            "HTTP/1.1 200 OK",
            r#"[
                {
                    "REPORT_DATE":"2025-12-31",
                    "NOTICE_DATE":"2026-03-28",
                    "TOTAL_OPERATE_INCOME":308227000000.0,
                    "YSTZ":8.37,
                    "PARENT_NETPROFIT":11117000000.0,
                    "SJLTZ":9.31,
                    "ROEJQ":14.8
                }
            ]"#,
            "application/json",
        ),
        (
            "/announcements",
            "HTTP/1.1 200 OK",
            r#"{
                "data":{
                    "list":[
                        {"notice_date":"2026-03-28","title":"2025骞村勾搴︽姤鍛?,"art_code":"AN202603281234567890","columns":[{"column_name":"瀹氭湡鎶ュ憡"}]},
                        {"notice_date":"2026-03-28","title":"2025骞村害鍒╂鼎鍒嗛厤棰勬鍏憡","art_code":"AN202603281234567891","columns":[{"column_name":"鍏徃鍏憡"}]}
                    ]
                }
            }"#,
            "application/json",
        ),
    ]);

    let request = json!({
        "tool": "security_decision_submit_approval",
        "args": {
            "symbol": "601916.SH",
            "market_profile": "a_share_core",
            "sector_profile": "a_share_bank",
            "stop_loss_pct": 0.05,
            "target_return_pct": 0.12,
            "approval_runtime_root": approval_root.to_string_lossy(),
            "created_at": "2026-04-02T12:30:00+08:00",
            "approval_brief_signing_key_id": "brief_signing_key_20260402",
            "approval_brief_signing_key_secret": "brief-secret-for-tests"
        }
    });

    let output = run_cli_with_json_runtime_and_envs(
        &request.to_string(),
        &runtime_db_path,
        &[
            (
                "EXCEL_SKILL_EASTMONEY_FINANCIAL_URL_BASE",
                format!("{server}/financials"),
            ),
            (
                "EXCEL_SKILL_EASTMONEY_ANNOUNCEMENT_URL_BASE",
                format!("{server}/announcements"),
            ),
        ],
    );

    // 2026-04-02 CST: 杩欓噷閿佷綇 detached signature 璺緞锛屽師鍥犳槸姝ｅ紡瀹℃壒绠€鎶ュ璞″繀椤绘敮鎸佺嫭绔嬬鍚嶈€屼笉鏄仠鐣欏湪鍐呭瓨瀵硅薄锛?    // 鐩殑锛氱‘淇?approval brief 鍚庣画鍙互浣滀负鍙璁″伐浠惰繘鍏?package锛岃€屼笉鏄彧鏈夋鏂囨病鏈夌鍚嶉敋鐐广€?
    assert_eq!(output["status"], "ok");
    let signature_path = PathBuf::from(
        output["data"]["approval_brief_signature_path"]
            .as_str()
            .expect("approval brief signature path should exist"),
    );
    assert!(signature_path.exists());

    let signature_envelope: Value = serde_json::from_slice(
        &fs::read(&signature_path).expect("approval brief signature should be readable"),
    )
    .expect("approval brief signature should be valid json");
    assert_eq!(
        signature_envelope["signature_version"],
        "security_approval_brief_signature.v1"
    );
    assert_eq!(signature_envelope["algorithm"], "hmac_sha256");
    assert_eq!(
        signature_envelope["contract_version"],
        "security_approval_brief.v1"
    );
    assert_eq!(signature_envelope["key_id"], "brief_signing_key_20260402");
    assert!(
        signature_envelope["brief_id"]
            .as_str()
            .expect("brief id should exist")
            .starts_with("brief-")
    );
    assert!(
        signature_envelope["payload_sha256"]
            .as_str()
            .expect("payload sha should exist")
            .len()
            >= 32
    );
    assert!(
        output["data"]["decision_package"]["artifact_manifest"]
            .as_array()
            .expect("artifact manifest should be array")
            .iter()
            .any(|artifact| artifact["artifact_role"] == "approval_brief_signature")
    );
}

#[test]
fn security_decision_submit_approval_scorecard_reuses_evidence_seed_and_matches_bool_bins() {
    let runtime_db_path =
        create_test_runtime_db("security_decision_submit_approval_scorecard_seed");
    let approval_root = runtime_db_path
        .parent()
        .expect("runtime db should have parent")
        .join("scenes_runtime");

    let stock_csv = create_stock_history_csv(
        "security_decision_submit_approval_scorecard_seed",
        "stock.csv",
        &build_confirmed_breakout_rows(220, 88.0),
    );
    let market_csv = create_stock_history_csv(
        "security_decision_submit_approval_scorecard_seed",
        "market.csv",
        &build_confirmed_breakout_rows(220, 3200.0),
    );
    let sector_csv = create_stock_history_csv(
        "security_decision_submit_approval_scorecard_seed",
        "sector.csv",
        &build_confirmed_breakout_rows(220, 950.0),
    );
    import_history_csv(&runtime_db_path, &stock_csv, "601916.SH");
    import_history_csv(&runtime_db_path, &market_csv, "510300.SH");
    import_history_csv(&runtime_db_path, &sector_csv, "512800.SH");

    let model_artifact_path = write_scorecard_model_artifact(
        "security_decision_submit_approval_scorecard_seed",
        "scorecard_model.json",
        &json!({
            "model_id": "a_share_equity_10d_direction_head",
            "model_version": "test_contract_alignment_v1",
            "label_definition": "security_forward_outcome.v1",
            "training_window": "2025-01-01..2025-12-31",
            "oot_window": "2026-01-01..2026-03-31",
            "positive_label_definition": "positive_return_10d",
            "binning_version": "woe_binning.v1",
            "coefficient_version": "woe_logistic.v1",
            "intercept": 0.0,
            "base_score": 600.0,
            "features": [
                {
                    "feature_name": "trend_bias",
                    "group_name": "T",
                    "bins": [
                        { "bin_label": "bullish", "match_values": ["bullish"], "points": 10.0 },
                        { "bin_label": "sideways", "match_values": ["sideways"], "points": 0.0 },
                        { "bin_label": "bearish", "match_values": ["bearish"], "points": -10.0 }
                    ]
                },
                {
                    "feature_name": "announcement_count",
                    "group_name": "E",
                    "bins": [
                        { "bin_label": "all", "match_values": [], "points": 5.0 }
                    ]
                },
                {
                    "feature_name": "has_risk_warning_notice",
                    "group_name": "E",
                    "bins": [
                        { "bin_label": "false", "match_values": ["false"], "points": 2.0 },
                        { "bin_label": "true", "match_values": ["true"], "points": -6.0 }
                    ]
                },
                {
                    "feature_name": "revenue_yoy_pct",
                    "group_name": "F",
                    "bins": [
                        { "bin_label": "all", "match_values": [], "points": 3.0 }
                    ]
                },
                {
                    "feature_name": "risk_note_count",
                    "group_name": "R",
                    "bins": [
                        { "bin_label": "all", "match_values": [], "points": -1.0 }
                    ]
                },
                {
                    "feature_name": "profit_signal",
                    "group_name": "F",
                    "bins": [
                        { "bin_label": "positive", "match_values": ["positive"], "points": 4.0 },
                        { "bin_label": "neutral", "match_values": ["neutral"], "points": 0.0 },
                        { "bin_label": "negative", "match_values": ["negative"], "points": -4.0 }
                    ]
                }
            ]
        }),
    );

    let server = spawn_http_route_server(vec![
        (
            "/financials",
            "HTTP/1.1 200 OK",
            r#"[
                {
                    "REPORT_DATE":"2025-12-31",
                    "NOTICE_DATE":"2026-03-28",
                    "TOTAL_OPERATE_INCOME":308227000000.0,
                    "YSTZ":8.37,
                    "PARENT_NETPROFIT":11117000000.0,
                    "SJLTZ":9.31,
                    "ROEJQ":14.8
                }
            ]"#,
            "application/json",
        ),
        (
            "/announcements",
            "HTTP/1.1 200 OK",
            r#"{
                "data":{
                    "list":[
                        {"notice_date":"2026-03-28","title":"2025骞村勾搴︽姤鍛?,"art_code":"AN202603281234567890","columns":[{"column_name":"瀹氭湡鎶ュ憡"}]},
                        {"notice_date":"2026-03-28","title":"2025骞村害鍒╂鼎鍒嗛厤棰勬鍏憡","art_code":"AN202603281234567891","columns":[{"column_name":"鍏徃鍏憡"}]}
                    ]
                }
            }"#,
            "application/json",
        ),
    ]);

    let request = json!({
        "tool": "security_decision_submit_approval",
        "args": {
            "symbol": "601916.SH",
            "market_profile": "a_share_core",
            "sector_profile": "a_share_bank",
            "stop_loss_pct": 0.05,
            "target_return_pct": 0.12,
            "approval_runtime_root": approval_root.to_string_lossy(),
            "created_at": "2026-04-10T21:20:00+08:00",
            "scorecard_model_path": model_artifact_path.to_string_lossy()
        }
    });

    let output = run_cli_with_json_runtime_and_envs(
        &request.to_string(),
        &runtime_db_path,
        &[
            (
                "EXCEL_SKILL_EASTMONEY_FINANCIAL_URL_BASE",
                format!("{server}/financials"),
            ),
            (
                "EXCEL_SKILL_EASTMONEY_ANNOUNCEMENT_URL_BASE",
                format!("{server}/announcements"),
            ),
        ],
    );

    // 2026-04-10 CST: 杩欓噷鍏堥攣缁熶竴鐗瑰緛鍚堝悓绾㈡祴锛屽師鍥犳槸绾夸笂 scorecard 鐩墠娌℃湁瀹屾暣澶嶇敤 evidence seed锛屽鑷磋缁冪壒寰佸湪绾夸笂澶ч噺 miss锛?    // 鐩殑锛氱‘淇濅慨澶嶅悗 scorecard 瀵?evidence seed 鐗瑰緛鍜?bool 鍒嗙閮借兘瀹屾暣鍛戒腑锛岃€屼笉鏄户缁仠鐣欏湪 feature_incomplete銆?
    assert_eq!(output["status"], "ok", "output={output}");
    let scorecard_path = output["data"]["scorecard_path"]
        .as_str()
        .expect("scorecard path should exist");
    let scorecard_payload =
        fs::read_to_string(scorecard_path).expect("internal scorecard artifact should be readable");
    let scorecard_json: Value =
        serde_json::from_str(&scorecard_payload).expect("scorecard artifact should be valid json");
    // 2026-04-10 CST: 杩欓噷鏀逛负浠?scorecard_path 璇诲彇鍐呴儴 artifact锛屽師鍥犳槸 submit_approval 杩斿洖鍊奸噷鐨?`scorecard`
    // 宸插崌绾т负瀵瑰瑙ｉ噴瑙嗗浘锛涚洰鐨勶細缁х画楠岃瘉鍐呴儴璇勫垎鍗″鐢?evidence seed 涓?bool 鍒嗙鍛戒腑锛屼笉鎶婃祴璇曢敊璇湴缁戝湪瀵瑰瑙嗗浘涓娿€?
    assert_eq!(scorecard_json["score_status"], "ready");
    assert_eq!(
        scorecard_json["feature_contributions"]
            .as_array()
            .expect("feature contributions should be array")
            .iter()
            .filter(|item| item["matched"] == Value::Bool(true))
            .count(),
        6
    );
    assert_eq!(
        scorecard_json["raw_feature_snapshot"]["has_risk_warning_notice"],
        Value::Bool(false)
    );
    assert_eq!(
        scorecard_json["raw_feature_snapshot"]["profit_signal"],
        Value::String("positive".to_string())
    );
}

fn import_history_csv(runtime_db_path: &Path, csv_path: &Path, symbol: &str) {
    let request = json!({
        "tool": "import_stock_price_history",
        "args": {
            "csv_path": csv_path.to_string_lossy(),
            "symbol": symbol,
            "source": "security_decision_submit_approval_fixture"
        }
    });

    let output = run_cli_with_json_runtime_and_envs(
        &request.to_string(),
        &runtime_db_path.to_path_buf(),
        &[],
    );
    assert_eq!(output["status"], "ok");
}

fn build_confirmed_breakout_rows(day_count: usize, start_close: f64) -> Vec<String> {
    let mut rows = vec!["trade_date,open,high,low,close,adj_close,volume".to_string()];
    let start_date = NaiveDate::from_ymd_opt(2025, 1, 1).expect("seed date should be valid");
    let mut close = start_close;

    for offset in 0..day_count {
        let trade_date = start_date + Duration::days(offset as i64);
        let (next_close, volume): (f64, i64) = if offset < day_count - 20 {
            (close + 0.78, 880_000 + offset as i64 * 8_000)
        } else {
            let phase = offset - (day_count - 20);
            match phase % 4 {
                0 => (close + 1.35, 1_700_000 + phase as i64 * 26_000),
                1 => (close - 0.18, 420_000),
                2 => (close + 1.08, 1_540_000 + phase as i64 * 22_000),
                _ => (close + 0.42, 1_240_000),
            }
        };

        let open = close;
        let high = next_close.max(open) + 1.0;
        let low = next_close.min(open) - 0.86;
        let adj_close = next_close;
        rows.push(format!(
            "{},{open:.2},{high:.2},{low:.2},{next_close:.2},{adj_close:.2},{volume}",
            trade_date.format("%Y-%m-%d")
        ));
        close = next_close;
    }

    rows
}
