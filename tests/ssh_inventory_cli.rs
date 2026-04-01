mod common;

use serde_json::json;

use crate::common::run_cli_with_json;

#[test]
fn tool_catalog_includes_ssh_inventory() {
    let output = run_cli_with_json("");

    // 2026-03-28 14:36 CST: 修改原因和目的：先锁目录可发现性，避免只实现 SSH 内核却忘了把安全受限入口注册到正式 Tool 清单。
    assert!(
        output["data"]["tool_catalog"]
            .as_array()
            .expect("tool catalog should be an array")
            .iter()
            .any(|tool| tool == "ssh_inventory")
    );
}

#[test]
fn ssh_inventory_rejects_non_whitelisted_command() {
    let request = json!({
        "tool": "ssh_inventory",
        "args": {
            "host": "127.0.0.1",
            "port": 22,
            "username": "readonly",
            "commands": ["uname -a"],
            "validate_only": true
        }
    });

    let output = run_cli_with_json(&request.to_string());

    // 2026-03-28 14:36 CST: 修改原因和目的：锁定“非白名单命令直接拒绝”的安全边界，避免后续为了方便调试把 SSH 工具放大成任意命令执行器。
    assert_eq!(output["status"], "error");
    assert!(
        output["error"]
            .as_str()
            .expect("error message should exist")
            .contains("whitelist")
    );
}

#[test]
fn ssh_inventory_rejects_shell_control_operators() {
    let request = json!({
        "tool": "ssh_inventory",
        "args": {
            "host": "127.0.0.1",
            "port": 22,
            "username": "readonly",
            "commands": ["ps -ef; cat /etc/passwd"],
            "validate_only": true
        }
    });

    let output = run_cli_with_json(&request.to_string());

    // 2026-03-28 14:36 CST: 修改原因和目的：锁定 shell 分隔符与拼接攻击的拒绝路径，防止用户输入借 SSH 工具突破只读采集范围。
    assert_eq!(output["status"], "error");
    assert!(
        output["error"]
            .as_str()
            .expect("error message should exist")
            .contains("unsafe")
    );
}

#[test]
fn ssh_inventory_accepts_validate_only_whitelisted_commands() {
    let request = json!({
        "tool": "ssh_inventory",
        "args": {
            "host": "127.0.0.1",
            "port": 22,
            "username": "readonly",
            "commands": ["ps -ef", "top -b -n 1", "nproc", "free -m"],
            "validate_only": true
        }
    });

    let output = run_cli_with_json(&request.to_string());

    // 2026-03-28 14:36 CST: 修改原因和目的：先锁住白名单命令在不发起真实 SSH 的情况下也能被预校验通过，后续前端或 Skill 可以先做安全预检再决定是否连远端。
    assert_eq!(output["status"], "ok");
    assert_eq!(output["data"]["validation_mode"], "validate_only");
    assert_eq!(
        output["data"]["allowed_commands"]
            .as_array()
            .expect("allowed commands should exist")
            .len(),
        4
    );
}

#[test]
fn ssh_inventory_returns_stable_error_when_connection_fails() {
    let request = json!({
        "tool": "ssh_inventory",
        "args": {
            "host": "127.0.0.1",
            "port": 1,
            "username": "readonly",
            "commands": ["ps -ef"]
        }
    });

    let output = run_cli_with_json(&request.to_string());

    // 2026-03-28 14:36 CST: 修改原因和目的：锁定真实连通失败时的稳定错误出口，避免 SSH 故障污染容量分析主链路或泄漏不必要的底层细节。
    assert_eq!(output["status"], "error");
    assert!(
        output["error"]
            .as_str()
            .expect("error message should exist")
            .contains("ssh")
    );
}
