mod common;

use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;

use assert_cmd::Command;
use rusqlite::Connection;
use serde_json::{Value, json};

use crate::common::create_test_runtime_db;

#[derive(Debug, Clone)]
struct MockHttpResponse {
    status_code: u16,
    body: Value,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RecordedRequest {
    path: String,
    body: String,
}

struct MockLemonServer {
    base_url: String,
    requests: Arc<Mutex<Vec<RecordedRequest>>>,
    join_handle: Option<thread::JoinHandle<()>>,
}

impl MockLemonServer {
    fn start(responses: Vec<MockHttpResponse>) -> Self {
        // 2026-03-29 CST: 这里用本地 TCP 假服务模拟 Lemon Squeezy，原因是授权测试必须锁定真实 HTTP 合同，
        // 目的：让 CLI 测试覆盖激活 / 校验 / 反激活，而不依赖外网与真实商店配置。
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let requests = Arc::new(Mutex::new(Vec::<RecordedRequest>::new()));
        let requests_for_thread = Arc::clone(&requests);

        let join_handle = thread::spawn(move || {
            for response in responses {
                let (stream, _) = listener.accept().unwrap();
                handle_mock_connection(stream, response, &requests_for_thread);
            }
        });

        Self {
            base_url: format!("http://{address}"),
            requests,
            join_handle: Some(join_handle),
        }
    }

    fn recorded_requests(&self) -> Vec<RecordedRequest> {
        self.requests.lock().unwrap().clone()
    }
}

impl Drop for MockLemonServer {
    fn drop(&mut self) {
        if let Some(join_handle) = self.join_handle.take() {
            join_handle.join().unwrap();
        }
    }
}

fn handle_mock_connection(
    mut stream: TcpStream,
    response: MockHttpResponse,
    requests: &Arc<Mutex<Vec<RecordedRequest>>>,
) {
    stream
        .set_read_timeout(Some(std::time::Duration::from_secs(2)))
        .unwrap();

    let mut reader = BufReader::new(stream.try_clone().unwrap());
    let mut request_line = String::new();
    reader.read_line(&mut request_line).unwrap();

    let mut path = String::new();
    if let Some(candidate) = request_line.split_whitespace().nth(1) {
        path = candidate.to_string();
    }

    let mut content_length = 0usize;
    loop {
        let mut header_line = String::new();
        reader.read_line(&mut header_line).unwrap();
        if header_line == "\r\n" || header_line.is_empty() {
            break;
        }
        if let Some(value) = header_line.strip_prefix("Content-Length:") {
            content_length = value.trim().parse::<usize>().unwrap();
        }
    }

    let mut body_bytes = vec![0_u8; content_length];
    reader.read_exact(&mut body_bytes).unwrap();
    let body = String::from_utf8(body_bytes).unwrap();
    requests
        .lock()
        .unwrap()
        .push(RecordedRequest { path, body });

    let payload = response.body.to_string();
    let reason = match response.status_code {
        200 => "OK",
        400 => "Bad Request",
        401 => "Unauthorized",
        404 => "Not Found",
        422 => "Unprocessable Entity",
        500 => "Internal Server Error",
        _ => "OK",
    };
    let reply = format!(
        "HTTP/1.1 {} {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        response.status_code,
        reason,
        payload.len(),
        payload
    );
    stream.write_all(reply.as_bytes()).unwrap();
    stream.flush().unwrap();
}

fn run_cli_with_json_and_env(
    input: &str,
    runtime_db_path: &PathBuf,
    envs: &[(String, String)],
) -> Value {
    let mut cmd = Command::cargo_bin("excel_skill").unwrap();
    cmd.env("EXCEL_SKILL_RUNTIME_DB", runtime_db_path);
    for (key, value) in envs {
        cmd.env(key, value);
    }
    let assert = cmd.write_stdin(input).assert().success();
    let output = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    serde_json::from_str(&output).unwrap()
}

fn license_envs(base_url: &str) -> Vec<(String, String)> {
    vec![
        ("EXCEL_SKILL_LICENSE_ENFORCED".to_string(), "1".to_string()),
        (
            "EXCEL_SKILL_LEMON_BASE_URL".to_string(),
            base_url.to_string(),
        ),
        ("EXCEL_SKILL_LEMON_STORE_ID".to_string(), "1".to_string()),
        ("EXCEL_SKILL_LEMON_PRODUCT_ID".to_string(), "2".to_string()),
        ("EXCEL_SKILL_LEMON_VARIANT_ID".to_string(), "3".to_string()),
        (
            "EXCEL_SKILL_LICENSE_OFFLINE_GRACE_HOURS".to_string(),
            "168".to_string(),
        ),
    ]
}

fn activate_response() -> MockHttpResponse {
    MockHttpResponse {
        status_code: 200,
        body: json!({
            "activated": true,
            "license_key": {
                "id": 101,
                "status": "active",
                "key": "lsq-demo-key-001",
                "activation_limit": 1,
                "activation_usage": 1,
                "created_at": "2026-03-29T00:00:00Z",
                "expires_at": Value::Null
            },
            "instance": {
                "id": "inst_001",
                "name": "excel-skill-test-machine",
                "created_at": "2026-03-29T00:00:00Z"
            },
            "meta": {
                "store_id": 1,
                "product_id": 2,
                "variant_id": 3,
                "customer_email": "demo@example.com"
            }
        }),
    }
}

fn validate_response() -> MockHttpResponse {
    MockHttpResponse {
        status_code: 200,
        body: json!({
            "valid": true,
            "license_key": {
                "id": 101,
                "status": "active",
                "key": "lsq-demo-key-001",
                "activation_limit": 1,
                "activation_usage": 1,
                "created_at": "2026-03-29T00:00:00Z",
                "expires_at": Value::Null
            },
            "instance": {
                "id": "inst_001",
                "name": "excel-skill-test-machine",
                "created_at": "2026-03-29T00:00:00Z"
            },
            "meta": {
                "store_id": 1,
                "product_id": 2,
                "variant_id": 3,
                "customer_email": "demo@example.com"
            }
        }),
    }
}

fn deactivate_response() -> MockHttpResponse {
    MockHttpResponse {
        status_code: 200,
        body: json!({
            "deactivated": true,
            "license_key": {
                "id": 101,
                "status": "inactive",
                "key": "lsq-demo-key-001"
            },
            "instance": {
                "id": "inst_001",
                "name": "excel-skill-test-machine"
            },
            "meta": {
                "store_id": 1,
                "product_id": 2,
                "variant_id": 3,
                "customer_email": "demo@example.com"
            }
        }),
    }
}

#[test]
fn tool_catalog_includes_license_tools() {
    let runtime_db = create_test_runtime_db("license_tool_catalog");
    let output = run_cli_with_json_and_env("", &runtime_db, &[]);
    let tools = output["data"]["tool_catalog"].as_array().unwrap();

    // 2026-03-29 CST: 这里先锁定目录层必须公开授权工具，原因是未授权状态下用户也必须能看见激活入口，
    // 目的：避免把授权能力做成“系统明明锁住了，但又不给解锁入口”的死路。
    assert!(tools.iter().any(|tool| tool == "license_activate"));
    assert!(tools.iter().any(|tool| tool == "license_status"));
    assert!(tools.iter().any(|tool| tool == "license_deactivate"));
}

#[test]
fn protected_tool_is_blocked_when_license_is_missing() {
    let runtime_db = create_test_runtime_db("license_missing_guard");
    let envs = license_envs("http://127.0.0.1:65530");
    let request = json!({
        "tool": "open_workbook",
        "args": {
            "path": "tests/fixtures/basic-sales.xlsx"
        }
    });

    let output = run_cli_with_json_and_env(&request.to_string(), &runtime_db, &envs);

    // 2026-03-29 CST: 这里先锁定受保护工具在未授权时必须被拦截，原因是这次接 Lemon 的核心目标就是限制普通传播，
    // 目的：先把“门禁存在”这件事钉死，再去补激活与缓存细节。
    assert_eq!(output["status"], "error");
    assert!(
        output["error"]
            .as_str()
            .unwrap()
            .contains("license_activate")
    );
}

#[test]
fn license_activate_persists_state_and_allows_follow_up_tool() {
    let runtime_db = create_test_runtime_db("license_activate_ok");
    let server = MockLemonServer::start(vec![activate_response()]);
    let envs = license_envs(&server.base_url);

    let activate_request = json!({
        "tool": "license_activate",
        "args": {
            "license_key": "lsq-demo-key-001",
            "instance_name": "excel-skill-test-machine"
        }
    });
    let activate_output =
        run_cli_with_json_and_env(&activate_request.to_string(), &runtime_db, &envs);
    assert_eq!(activate_output["status"], "ok");
    assert_eq!(activate_output["data"]["licensed"], true);

    let status_request = json!({
        "tool": "license_status",
        "args": {}
    });
    let status_output = run_cli_with_json_and_env(&status_request.to_string(), &runtime_db, &envs);
    assert_eq!(status_output["status"], "ok");
    assert_eq!(status_output["data"]["licensed"], true);
    assert_eq!(status_output["data"]["instance_id"], "inst_001");

    let open_request = json!({
        "tool": "open_workbook",
        "args": {
            "path": "tests/fixtures/basic-sales.xlsx"
        }
    });
    let open_output = run_cli_with_json_and_env(&open_request.to_string(), &runtime_db, &envs);
    assert_eq!(open_output["status"], "ok");

    let connection = Connection::open(&runtime_db).unwrap();
    let saved_instance_id: String = connection
        .query_row(
            "SELECT instance_id FROM license_state WHERE singleton_id = 1",
            [],
            |row| row.get(0),
        )
        .unwrap();

    // 2026-03-29 CST: 这里锁定激活后的本地落库，原因是方案 A 的关键不是“当场激活成功”，而是“之后还能靠本地缓存放行”，
    // 目的：防止实现只把 HTTP 打通，却没有真正形成单机 EXE 可复用的授权状态。
    assert_eq!(saved_instance_id, "inst_001");

    let recorded_requests = server.recorded_requests();
    assert_eq!(recorded_requests.len(), 1);
    assert_eq!(recorded_requests[0].path, "/v1/licenses/activate");
    assert!(
        recorded_requests[0]
            .body
            .contains("license_key=lsq-demo-key-001")
    );
}

#[test]
fn stale_license_triggers_online_validate_before_protected_tool() {
    let runtime_db = create_test_runtime_db("license_validate_guard");
    let server = MockLemonServer::start(vec![activate_response(), validate_response()]);
    let mut envs = license_envs(&server.base_url);
    envs.push((
        "EXCEL_SKILL_LICENSE_VALIDATE_MAX_AGE_HOURS".to_string(),
        "0".to_string(),
    ));

    let activate_request = json!({
        "tool": "license_activate",
        "args": {
            "license_key": "lsq-demo-key-001",
            "instance_name": "excel-skill-test-machine"
        }
    });
    let activate_output =
        run_cli_with_json_and_env(&activate_request.to_string(), &runtime_db, &envs);
    assert_eq!(activate_output["status"], "ok");

    let open_request = json!({
        "tool": "open_workbook",
        "args": {
            "path": "tests/fixtures/basic-sales.xlsx"
        }
    });
    let open_output = run_cli_with_json_and_env(&open_request.to_string(), &runtime_db, &envs);
    assert_eq!(open_output["status"], "ok");

    let recorded_requests = server.recorded_requests();

    // 2026-03-29 CST: 这里把“缓存过期后自动 validate”钉成红线，原因是方案 A 不是只做一次激活，
    // 目的：确保后续真能利用 Lemon 的在线校验能力，而不是退化成一次激活后永久放行。
    assert_eq!(recorded_requests.len(), 2);
    assert_eq!(recorded_requests[0].path, "/v1/licenses/activate");
    assert_eq!(recorded_requests[1].path, "/v1/licenses/validate");
    assert!(recorded_requests[1].body.contains("instance_id=inst_001"));
}

#[test]
fn license_deactivate_clears_local_state_and_reblocks_tool() {
    let runtime_db = create_test_runtime_db("license_deactivate_guard");
    let server = MockLemonServer::start(vec![activate_response(), deactivate_response()]);
    let envs = license_envs(&server.base_url);

    let activate_request = json!({
        "tool": "license_activate",
        "args": {
            "license_key": "lsq-demo-key-001",
            "instance_name": "excel-skill-test-machine"
        }
    });
    let activate_output =
        run_cli_with_json_and_env(&activate_request.to_string(), &runtime_db, &envs);
    assert_eq!(activate_output["status"], "ok");

    let deactivate_request = json!({
        "tool": "license_deactivate",
        "args": {}
    });
    let deactivate_output =
        run_cli_with_json_and_env(&deactivate_request.to_string(), &runtime_db, &envs);
    assert_eq!(deactivate_output["status"], "ok");
    assert_eq!(deactivate_output["data"]["licensed"], false);

    let open_request = json!({
        "tool": "open_workbook",
        "args": {
            "path": "tests/fixtures/basic-sales.xlsx"
        }
    });
    let open_output = run_cli_with_json_and_env(&open_request.to_string(), &runtime_db, &envs);
    assert_eq!(open_output["status"], "error");
    assert!(
        open_output["error"]
            .as_str()
            .unwrap()
            .contains("license_activate")
    );

    let connection = Connection::open(&runtime_db).unwrap();
    let saved_rows: i64 = connection
        .query_row("SELECT COUNT(*) FROM license_state", [], |row| row.get(0))
        .unwrap_or(0);

    // 2026-03-29 CST: 这里锁定反激活必须回收本地状态，原因是如果只调了远端 deactive 却不清本地缓存，
    // 目的：就会出现“远端已停用，本地还能继续跑”的假回收。
    assert_eq!(saved_rows, 0);
}
