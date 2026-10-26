// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use std::process::Command;

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

#[test]
fn invalid_arguments_return_json_diagnostics() {
    for args in [vec!["invalid"], vec!["get"], vec!["export", "--unexpected"]] {
        let output = Command::new(env!("CARGO_BIN_EXE_csp"))
            .args(args)
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(1));
        assert!(output.stdout.is_empty());
        let error: serde_json::Value = serde_json::from_slice(&output.stderr).unwrap();
        assert!(error["error"].as_str().is_some_and(|s| !s.is_empty()));
    }
}
