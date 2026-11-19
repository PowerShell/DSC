// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use std::process::Command;

use test_case::test_case;

#[cfg(windows)]
use is_elevated::is_elevated;

#[test]
fn help_lists_all_operations() {
    let output = Command::new(env!("CARGO_BIN_EXE_csp"))
        .arg("--help")
        .output()
        .unwrap();
    assert!(output.status.success());
    let help = String::from_utf8(output.stdout).unwrap();
    for operation in ["list", "get", "set", "test", "export"] {
        assert!(help.contains(operation), "{help}");
    }
}

#[test_case(vec!["invalid"]; "invalid command")]
#[test_case(vec!["get"]; "missing arguments for get")]
#[test_case(vec!["export", "--unexpected"]; "unexpected argument for export")]
fn invalid_arguments_return_json_diagnostics(args: Vec<&str>) {
    let output = Command::new(env!("CARGO_BIN_EXE_csp"))
        .args(&args)
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    let error: serde_json::Value = serde_json::from_slice(&output.stderr).unwrap();
    assert!(error["error"].as_str().is_some_and(|s| !s.is_empty()));
}

#[cfg(windows)]
fn invoke_with_stdin(operation: &str, input: &[u8]) -> std::process::Output {
    use std::io::Write;
    use std::process::Stdio;

    let mut child = Command::new(env!("CARGO_BIN_EXE_csp"))
        .args([
            operation,
            "--resource-type",
            "Microsoft.Windows.CSP/MDM_Policy_User_Config01_Authentication02",
            "--input",
            "-",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(input).unwrap();
    child.wait_with_output().unwrap()
}

#[cfg(windows)]
#[test_case("get"; "for get operation")]
#[test_case("set"; "for set operation")]
#[test_case("test"; "for test operation")]
#[test_case("export"; "for export operation")]
fn accepts_large_json_payloads_over_stdin(operation: &str) {
    if is_elevated() {
        let input = serde_json::json!({"Payload": "x".repeat(70_000)}).to_string();
        assert!(input.encode_utf16().count() > 32_767);
        let output = invoke_with_stdin(operation, input.as_bytes());
        assert_eq!(output.status.code(), Some(2), "{operation}: {output:?}");
        assert!(output.stdout.is_empty());
        let error: serde_json::Value = serde_json::from_slice(&output.stderr).unwrap();
        assert_eq!(error["error"], "Unknown or output-only property 'Payload'.");
    } else {
        eprintln!("Skipping test - current context isn't elevated.");
    }
}

#[cfg(windows)]
#[test_case(b"{".as_slice(), "Invalid JSON input:"; "invalid JSON input")]
#[test_case(b"".as_slice(), "Invalid JSON input:"; "empty JSON input")]
#[test_case(b"\xff".as_slice(), "Could not read JSON input from stdin:"; "unreadable JSON input")]
fn reports_invalid_or_unreadable_stdin_as_json_errors(input: &[u8], expected: &str) {
    if is_elevated() {
        let output = invoke_with_stdin("get", input);
        assert_eq!(output.status.code(), Some(2));
        assert!(output.stdout.is_empty());
        let error: serde_json::Value = serde_json::from_slice(&output.stderr).unwrap();
        assert!(error["error"].as_str().unwrap().starts_with(expected));
    } else {
        eprintln!("Skipping test - current context isn't elevated.");
    }
}

#[test_case("get"; "get operation")]
#[test_case("set"; "set operation")]
#[test_case("test"; "test operation")]
#[test_case("export"; "export operation")]
fn manifest_sends_operation_payloads_over_stdin(operation: &str) {
    let manifest: serde_json::Value =
        serde_json::from_str(include_str!("../csp.dsc.resource.json")).unwrap();
    assert_eq!(manifest[operation]["input"], "stdin");
    assert_eq!(
        manifest[operation]["args"],
        serde_json::json!([
            operation,
            {"resourceTypeArg": "--resource-type"},
            "--input",
            "-"
        ])
    );
}
