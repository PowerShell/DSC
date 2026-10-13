// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[cfg(test)]
mod invoke_command {
    use dsc_lib::{dscresources::command_resource::invoke_command, types::ExitCodesMap};

    /// Verifies that when `invoke_command` is called with `input = None`, the child process
    /// receives an immediate EOF on stdin (i.e., stdin is set to null) rather than inheriting
    /// the parent's stdin handle.
    ///
    /// This is a regression test for the hang introduced in DSC 3.2.0 when the PowerShell
    /// adapter changed from `"config": "full"` to `"config": "single"`. In single mode, the
    /// adapter's export operation is called with no input, leaving stdin unset in the previous
    /// code. Child processes that read from stdin would then block indefinitely when the parent
    /// process itself had an open stdin handle — either a TTY in a terminal or a pipe in CI.
    ///
    /// The test uses a timed async read rather than a blocking read so that the child process
    /// always exits within a bounded time. If stdin is null (the fix), `ReadAsync` completes
    /// immediately returning 0 bytes (EOF), which maps to -1. If stdin is inherited (the bug),
    /// `ReadAsync` blocks until the timeout fires and the test receives -2, which fails the
    /// assertion.
    #[test]
    fn no_input_does_not_block_on_stdin() {
        let exit_codes = ExitCodesMap::default();

        // Use PowerShell's own async timeout so the child process always exits within ~2s,
        // regardless of fix status. We never leave a hanging thread:
        //   byte:-1  → ReadAsync got EOF immediately  → stdin was null  → PASS
        //   byte:-2  → ReadAsync timed out (2 s)      → stdin was NOT null → FAIL
        let ps_command = concat!(
            "$reader = [Console]::OpenStandardInput();",
            "$buf = [byte[]]::new(1);",
            "$task = $reader.ReadAsync($buf, 0, 1);",
            "$completed = $task.Wait(2000);",
            "$b = if ($completed) { if ($task.Result -eq 0) { -1 } else { $buf[0] } } else { -2 };",
            "Write-Output \"byte:$b\""
        );

        let result = invoke_command(
            "pwsh",
            Some(vec![
                "-NonInteractive".to_string(),
                "-NoProfile".to_string(),
                "-Command".to_string(),
                ps_command.to_string(),
            ]),
            None,  // no input — the scenario that caused the hang
            None,
            None,
            &exit_codes,
        ).expect("invoke_command should succeed");

        let (exit_code, stdout, _stderr) = result;
        assert_eq!(exit_code, 0, "Command should exit 0");
        // -1 means ReadAsync got EOF immediately, confirming stdin was set to null.
        // -2 means stdin was open (inherited) and the read timed out after 2s.
        assert!(
            stdout.contains("byte:-1"),
            "Expected EOF (byte:-1) from null stdin, got: {stdout:?}\n\
             'byte:-2' means stdin was inherited from the parent rather than set to null."
        );
    }
}
