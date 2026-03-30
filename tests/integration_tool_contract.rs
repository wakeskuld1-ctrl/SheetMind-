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

#[test]
fn tool_catalog_response_exposes_foundation_and_stock_groups() {
    let response = ToolResponse::tool_catalog();
    let foundation = response.data["tool_catalog_modules"]["foundation"]
        .as_array()
        .expect("foundation tool catalog should be an array")
        .iter()
        .map(|value| value.as_str().expect("tool name should be a string"))
        .collect::<Vec<_>>();
    let stock = response.data["tool_catalog_modules"]["stock"]
        .as_array()
        .expect("stock tool catalog should be an array")
        .iter()
        .map(|value| value.as_str().expect("tool name should be a string"))
        .collect::<Vec<_>>();

    assert_eq!(foundation, excel_skill::tools::catalog::foundation_tool_names());
    assert_eq!(stock, excel_skill::tools::catalog::stock_tool_names());
    assert!(stock.contains(&"technical_consultation_basic"));
    assert!(!foundation.contains(&"technical_consultation_basic"));
}

#[test]
fn grouped_tool_catalog_matches_flat_catalog_without_overlap() {
    let foundation = excel_skill::tools::catalog::foundation_tool_names();
    let stock = excel_skill::tools::catalog::stock_tool_names();
    let mut combined = foundation
        .iter()
        .chain(stock.iter())
        .copied()
        .collect::<Vec<_>>();
    combined.sort_unstable();

    let mut flat = excel_skill::tools::catalog::tool_names().to_vec();
    flat.sort_unstable();

    assert_eq!(combined, flat);
    for tool_name in stock {
        assert!(
            !foundation.contains(tool_name),
            "tool `{tool_name}` should not overlap between foundation and stock groups"
        );
    }
}
