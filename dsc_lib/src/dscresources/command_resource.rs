use serde_json::Value;

use crate::dscerror::DscError;
use super::{resource_manifest::ResourceManifest, invoke_result::TestResult};
use std::{process::Command, io::{Write, Read}, process::Stdio};
use crate::dscresources::invoke_result::GetResult;

pub const EXIT_PROCESS_TERMINATED: i32 = 0x102;

fn invoke_command(executable: &str, args: Vec<String>, input: &str) -> Result<(i32, String, String), DscError> {
    let mut command = Command::new(executable);
    command.stdin(Stdio::piped());
    command.stdout(Stdio::piped());
    command.stderr(Stdio::piped());
    command.args(args);

    let mut child = command.spawn()?;
    {
        // pipe to child stdin in a scope so that it is dropped before we wait
        // otherwise the pipe isn't closed and the child process waits forever
        let mut child_stdin = child.stdin.take().unwrap();
        child_stdin.write_all(input.as_bytes())?;
        child_stdin.flush()?;
    }
    let exit_status = child.wait()?;

    let mut child_stdout = child.stdout.take().unwrap();
    let mut child_stderr = child.stderr.take().unwrap();
    let mut stdout_buf = Vec::new();
    child_stdout.read_to_end(&mut stdout_buf)?;
    let mut stderr_buf = Vec::new();
    child_stderr.read_to_end(&mut stderr_buf)?;

    let exit_code = exit_status.code().unwrap_or(EXIT_PROCESS_TERMINATED) as i32;
    let stdout = String::from_utf8_lossy(&stdout_buf).to_string();
    let stderr = String::from_utf8_lossy(&stderr_buf).to_string();
    Ok((exit_code, stdout, stderr))
}

pub fn invoke_get(resource: &ResourceManifest, filter: &str) -> Result<GetResult, DscError> {
    let (exit_code, stdout, stderr) = invoke_command(&resource.get.executable, resource.get.args.clone().unwrap_or_default(), filter)?;
    if exit_code != 0 {
        return Err(DscError::Command(exit_code, stderr.to_string()));
    }

    let result: Value = serde_json::from_str(&stdout)?;
    Ok(GetResult {
        actual_state: result,
    })
}

pub fn invoke_test(resource: &ResourceManifest, expected: &str) -> Result<TestResult, DscError> {
    let (exit_code, stdout, stderr) = invoke_command(&resource.test.executable, resource.test.args.clone().unwrap_or_default(), expected)?;
    if exit_code != 0 {
        return Err(DscError::Command(exit_code, stderr.to_string()));
    }

    let expected: Value = serde_json::from_str(expected)?;
    let result: Value = serde_json::from_str(&stdout)?;
    let mut diff_properties: Vec<String> = Vec::new();
    for (key, value) in expected.as_object().unwrap() {
        if !key.starts_with("_") && result[key] != *value {
            diff_properties.push(key.to_string());
        }
    }

    Ok(TestResult {
        expected_state: expected,
        actual_state: result,
        diff_properties: Some(diff_properties),
    })
}
