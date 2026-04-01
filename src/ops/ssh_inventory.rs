use std::collections::BTreeMap;
use std::process::Command;

use serde::{Deserialize, Serialize};

const ALLOWED_COMMANDS: &[&str] = &[
    "ps -ef",
    "top -b -n 1",
    "nproc",
    "free -m",
    "cat /proc/cpuinfo",
    "cat /proc/meminfo",
    "hostname",
];
const UNSAFE_TOKENS: &[&str] = &[";", "&&", "||", "|", ">", "<"];

// 2026-03-28 15:20 CST: 这里定义 SSH 盘点请求，原因是需要把远程采集收口成强类型结构；
// 目的是让调用端只能在受控字段里表达主机、端口和命令清单，而不是传任意脚本。
#[derive(Debug, Clone, Deserialize)]
pub struct SshInventoryRequest {
    pub host: String,
    #[serde(default = "default_ssh_port")]
    pub port: u16,
    pub username: String,
    #[serde(default)]
    pub commands: Vec<String>,
    #[serde(default)]
    pub validate_only: bool,
    #[serde(default = "default_connect_timeout_seconds")]
    pub connect_timeout_seconds: u64,
}

// 2026-03-28 15:20 CST: 这里定义 SSH 盘点输出，原因是后续 capacity_assessment 需要消费标准化实例事实；
// 目的是把白名单校验结果、原始片段和解析后的主机事实统一返回。
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SshInventoryResult {
    pub validation_mode: String,
    pub allowed_commands: Vec<String>,
    pub host: String,
    pub username: String,
    #[serde(default)]
    pub command_outputs: BTreeMap<String, String>,
    pub inventory: InventorySnapshot,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct InventorySnapshot {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hostname: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cpu_core_count: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memory_total_mb: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub process_snapshot_excerpt: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_snapshot_excerpt: Option<String>,
}

fn default_ssh_port() -> u16 {
    22
}

fn default_connect_timeout_seconds() -> u64 {
    5
}

// 2026-03-28 15:20 CST: 这里提供 SSH 盘点主入口，原因是容量工具需要一个可选的远程事实补数来源；
// 目的是在保持只读白名单的前提下，为实例数和主机资源信息提供自动采集能力。
pub fn ssh_inventory(request: &SshInventoryRequest) -> Result<SshInventoryResult, String> {
    validate_request(request)?;

    let allowed_commands = request.commands.clone();
    if request.validate_only {
        return Ok(SshInventoryResult {
            validation_mode: "validate_only".to_string(),
            allowed_commands,
            host: request.host.clone(),
            username: request.username.clone(),
            command_outputs: BTreeMap::new(),
            inventory: InventorySnapshot::default(),
        });
    }

    let mut command_outputs = BTreeMap::new();
    for command in &request.commands {
        let output = run_remote_command(request, command)?;
        command_outputs.insert(command.clone(), output);
    }

    let inventory = build_inventory_snapshot(&command_outputs);

    Ok(SshInventoryResult {
        validation_mode: "executed".to_string(),
        allowed_commands,
        host: request.host.clone(),
        username: request.username.clone(),
        command_outputs,
        inventory,
    })
}

fn validate_request(request: &SshInventoryRequest) -> Result<(), String> {
    if request.host.trim().is_empty() {
        return Err("ssh request missing host".to_string());
    }
    if request.username.trim().is_empty() {
        return Err("ssh request missing username".to_string());
    }
    if request.commands.is_empty() {
        return Err("ssh request missing commands".to_string());
    }

    for command in &request.commands {
        validate_command(command)?;
    }
    Ok(())
}

fn validate_command(command: &str) -> Result<(), String> {
    let trimmed = command.trim();
    if trimmed.is_empty() {
        return Err("ssh unsafe command: empty command".to_string());
    }
    if UNSAFE_TOKENS.iter().any(|token| trimmed.contains(token)) {
        return Err(format!(
            "ssh unsafe command rejected by unsafe token policy: {trimmed}"
        ));
    }
    if !ALLOWED_COMMANDS.iter().any(|allowed| allowed == &trimmed) {
        return Err(format!("ssh command not in whitelist: {trimmed}"));
    }
    Ok(())
}

fn run_remote_command(request: &SshInventoryRequest, command: &str) -> Result<String, String> {
    let remote_target = format!("{}@{}", request.username.trim(), request.host.trim());
    let output = Command::new("ssh")
        .arg("-o")
        .arg("BatchMode=yes")
        .arg("-o")
        .arg(format!(
            "ConnectTimeout={}",
            request.connect_timeout_seconds
        ))
        .arg("-p")
        .arg(request.port.to_string())
        .arg(remote_target)
        .arg(command)
        .output()
        .map_err(|error| format!("ssh execution failed: {error}"))?;

    if output.status.success() {
        String::from_utf8(output.stdout)
            .map_err(|error| format!("ssh output decode failed: {error}"))
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        Err(format!("ssh command failed: {stderr}"))
    }
}

fn build_inventory_snapshot(command_outputs: &BTreeMap<String, String>) -> InventorySnapshot {
    InventorySnapshot {
        hostname: command_outputs
            .get("hostname")
            .map(|value| value.lines().next().unwrap_or("").trim().to_string())
            .filter(|value| !value.is_empty()),
        cpu_core_count: command_outputs
            .get("nproc")
            .and_then(|value| value.lines().next())
            .and_then(|value| value.trim().parse::<u64>().ok()),
        memory_total_mb: command_outputs
            .get("free -m")
            // 2026-03-28 16:15 CST: 这里改成显式闭包，原因是 `.get()` 返回的是 `&String` 而解析函数签名接收 `&str`；目的是消除接线后暴露的编译错误并保持解析逻辑不变。
            .and_then(|value| parse_memory_total_from_free_m(value)),
        process_snapshot_excerpt: command_outputs
            .get("ps -ef")
            .map(|value| take_excerpt(value, 12)),
        top_snapshot_excerpt: command_outputs
            .get("top -b -n 1")
            .map(|value| take_excerpt(value, 20)),
    }
}

fn parse_memory_total_from_free_m(raw: &str) -> Option<u64> {
    raw.lines().find_map(|line| {
        let trimmed = line.trim_start();
        if trimmed.starts_with("Mem:") {
            trimmed
                .split_whitespace()
                .nth(1)
                .and_then(|value| value.parse::<u64>().ok())
        } else {
            None
        }
    })
}

fn take_excerpt(raw: &str, max_lines: usize) -> String {
    raw.lines().take(max_lines).collect::<Vec<_>>().join("\n")
}
