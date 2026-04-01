use std::io::{self, Read};

use encoding_rs::GBK;
use excel_skill::license::service::{LicenseService, LicenseServiceError};
use excel_skill::license::types::{
    LicenseActivateRequest, LicenseDeactivateResult, LicenseStatusRequest,
};
use excel_skill::tool_catalog_json;
use excel_skill::tools::contracts::{ToolRequest, ToolResponse};
use excel_skill::tools::dispatcher::dispatch;
use serde_json::json;

fn main() {
    let mut input_bytes = Vec::new();
    match io::stdin().read_to_end(&mut input_bytes) {
        Ok(_) => {
            let input = match decode_input_bytes(&input_bytes) {
                Ok(input) => input,
                Err(error) => {
                    print_response(ToolResponse::error(error));
                    return;
                }
            };

            if input.trim().is_empty() {
                println!("{}", tool_catalog_json());
                return;
            }

            print_response(handle_request_input(input));
        }
        Err(error) => print_response(ToolResponse::error(format!(
            "\u{8bfb}\u{53d6}\u{6807}\u{51c6}\u{8f93}\u{5165}\u{5931}\u{8d25}: {error}"
        ))),
    }
}

fn handle_request_input(input: String) -> ToolResponse {
    match serde_json::from_str::<ToolRequest>(&input) {
        Ok(request) => handle_tool_request(request),
        Err(error) => ToolResponse::error(format!(
            "\u{8bf7}\u{6c42} JSON \u{89e3}\u{6790}\u{5931}\u{8d25}: {error}"
        )),
    }
}

fn handle_tool_request(request: ToolRequest) -> ToolResponse {
    // 2026-03-29 CST: 这里把授权门禁挂在主入口，原因是 Lemon 授权属于 EXE 级别的运行前校验，
    // 目的：不侵入现有 Excel / 分析 Tool 业务实现，就能统一拦截未授权调用。
    let service = LicenseService::from_env();

    match request.tool.as_str() {
        "license_activate" => handle_license_activate(service, request.args),
        "license_status" => handle_license_status(service, request.args),
        "license_deactivate" => handle_license_deactivate(service),
        _ => match service.enforce_tool_access(&request.tool) {
            Ok(_) => dispatch(request),
            Err(error) => ToolResponse::error(error.to_string()),
        },
    }
}

fn handle_license_activate(service: LicenseService, args: serde_json::Value) -> ToolResponse {
    // 2026-03-29 CST: 这里把激活请求解析放在主入口，原因是授权工具和普通业务 Tool 不同，属于进程级控制面；
    // 目的：让激活、状态、反激活都共用同一套授权服务，而不是再塞进业务 dispatcher。
    let request = match serde_json::from_value::<LicenseActivateRequest>(args) {
        Ok(request) => request,
        Err(error) => {
            return ToolResponse::error(format!("license_activate 参数解析失败: {error}"));
        }
    };

    match service.activate(&request) {
        Ok(result) => ToolResponse::ok(json!(result)),
        Err(error) => ToolResponse::error(error.to_string()),
    }
}

fn handle_license_status(service: LicenseService, args: serde_json::Value) -> ToolResponse {
    let request = match serde_json::from_value::<LicenseStatusRequest>(args) {
        Ok(request) => request,
        Err(error) => return ToolResponse::error(format!("license_status 参数解析失败: {error}")),
    };

    match service.status(request.refresh) {
        Ok(result) => ToolResponse::ok(json!(result)),
        Err(error) => ToolResponse::error(error.to_string()),
    }
}

fn handle_license_deactivate(service: LicenseService) -> ToolResponse {
    match service.deactivate() {
        Ok(LicenseDeactivateResult {
            licensed,
            deactivated,
            message,
        }) => ToolResponse::ok(json!({
            "licensed": licensed,
            "deactivated": deactivated,
            "message": message
        })),
        Err(LicenseServiceError::Store(error)) => ToolResponse::error(error.to_string()),
        Err(LicenseServiceError::Client(error)) => ToolResponse::error(error.to_string()),
        Err(error) => ToolResponse::error(error.to_string()),
    }
}

fn decode_input_bytes(input_bytes: &[u8]) -> Result<String, String> {
    if input_bytes.is_empty() {
        return Ok(String::new());
    }

    if let Some(decoded) = decode_utf8_with_optional_bom(input_bytes) {
        return Ok(decoded);
    }

    if let Some(decoded) = decode_utf16_with_bom(input_bytes) {
        return Ok(decoded);
    }

    let (decoded, _, had_errors) = GBK.decode(input_bytes);
    if !had_errors {
        return Ok(decoded.into_owned());
    }

    Err(
        "\u{6807}\u{51c6}\u{8f93}\u{5165}\u{4e0d}\u{662f}\u{53ef}\u{8bc6}\u{522b}\u{7684} UTF-8 / UTF-16 / GBK \u{7f16}\u{7801}\u{ff0c}\u{65e0}\u{6cd5}\u{89e3}\u{6790}\u{8bf7}\u{6c42}".to_string(),
    )
}

fn decode_utf8_with_optional_bom(input_bytes: &[u8]) -> Option<String> {
    const UTF8_BOM: &[u8; 3] = b"\xEF\xBB\xBF";
    let bytes = input_bytes.strip_prefix(UTF8_BOM).unwrap_or(input_bytes);
    String::from_utf8(bytes.to_vec()).ok()
}

fn decode_utf16_with_bom(input_bytes: &[u8]) -> Option<String> {
    const UTF16_LE_BOM: &[u8; 2] = b"\xFF\xFE";
    const UTF16_BE_BOM: &[u8; 2] = b"\xFE\xFF";

    if let Some(bytes) = input_bytes.strip_prefix(UTF16_LE_BOM) {
        return decode_utf16_units(bytes, true);
    }
    if let Some(bytes) = input_bytes.strip_prefix(UTF16_BE_BOM) {
        return decode_utf16_units(bytes, false);
    }

    None
}

fn decode_utf16_units(bytes: &[u8], little_endian: bool) -> Option<String> {
    if bytes.len() % 2 != 0 {
        return None;
    }

    let units = bytes
        .chunks_exact(2)
        .map(|chunk| {
            if little_endian {
                u16::from_le_bytes([chunk[0], chunk[1]])
            } else {
                u16::from_be_bytes([chunk[0], chunk[1]])
            }
        })
        .collect::<Vec<_>>();

    String::from_utf16(&units).ok()
}

fn print_response(response: ToolResponse) {
    let payload =
        serde_json::to_string(&response).expect("tool response serialization should succeed");
    println!("{}", payload);
}
