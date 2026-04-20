mod common;

use std::fs;
use std::path::PathBuf;
use std::process::Command;

use crate::common::create_test_output_path;

fn contains_ascii_marker(bytes: &[u8], marker: &str) -> bool {
    bytes.windows(marker.len())
        .any(|window| window == marker.as_bytes())
}

#[test]
fn betting_vba_asset_builder_generates_formal_macro_project() {
    if !cfg!(windows) {
        return;
    }

    // 2026-04-20 CST: Reproduce the delivery-blocking VBA asset build locally
    // before fixing it, so the parser/build regression stays covered by an
    // executable check instead of a one-off manual command.
    let output_path = create_test_output_path("betting_vba_asset", "bin");
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let script_path = repo_root.join("scripts").join("build_betting_vba_asset.ps1");

    let output = Command::new("powershell")
        .args([
            "-ExecutionPolicy",
            "Bypass",
            "-File",
            script_path.to_string_lossy().as_ref(),
            "-OutputVbaProjectPath",
            output_path.to_string_lossy().as_ref(),
        ])
        .output()
        .expect("powershell should be available");

    assert!(
        output.status.success(),
        "script failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output_path.exists(), "expected generated vbaProject.bin");

    let bytes = fs::read(&output_path).expect("generated vba project should be readable");
    assert!(contains_ascii_marker(&bytes, "BettingSolverRunner"));
    assert!(contains_ascii_marker(&bytes, "SolverProgressForm"));
    assert!(contains_ascii_marker(&bytes, "RequestCancel"));
}
