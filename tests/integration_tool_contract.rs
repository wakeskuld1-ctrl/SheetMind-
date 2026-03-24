mod common;

use excel_skill::tools::contracts::ToolResponse;

use crate::common::run_cli_with_json;

#[test]
fn cli_tool_catalog_matches_registered_tool_names() {
    let output = run_cli_with_json("");
    let actual = output["data"]["tool_catalog"]
        .as_array()
        .expect("tool catalog should be an array")
        .iter()
        .map(|value| {
            value
                .as_str()
                .expect("tool name should be a string")
                .to_string()
        })
        .collect::<Vec<_>>();

    let expected = excel_skill::tools::catalog::tool_names()
        .iter()
        .map(|name| (*name).to_string())
        .collect::<Vec<_>>();

    assert_eq!(actual, expected);
}

#[test]
fn tool_catalog_response_uses_registered_tool_names() {
    let response = ToolResponse::tool_catalog();
    let actual = response.data["tool_catalog"]
        .as_array()
        .expect("tool catalog should be an array")
        .iter()
        .map(|value| {
            value
                .as_str()
                .expect("tool name should be a string")
                .to_string()
        })
        .collect::<Vec<_>>();

    let expected = excel_skill::tools::catalog::tool_names()
        .iter()
        .map(|name| (*name).to_string())
        .collect::<Vec<_>>();

    assert_eq!(actual, expected);
}
