use serde_json::Value;

use crate::dscerror::DscError;
use super::{resource_manifest::{ResourceManifest, ReturnKind}, invoke_result::{GetResult, SetResult, TestResult}};
use std::{process::Command, io::{Write, Read}, process::Stdio};

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

pub fn invoke_set(resource: &ResourceManifest, desired: &str) -> Result<SetResult, DscError> {
    // if resource doesn't implement a pre-test, we execute test first to see if a set is needed
    if !resource.set.pre_test.unwrap_or_default() {
        let test_result = invoke_test(resource, desired)?;
        if test_result.diff_properties.is_none() {
            return Ok(SetResult {
                before_state: test_result.expected_state,
                after_state: test_result.actual_state,
                changed_properties: None,
            });
        }
    }

    let pre_state: Value;
    let (exit_code, stdout, stderr) = invoke_command(&resource.get.executable, resource.get.args.clone().unwrap_or_default(), desired)?;
    if exit_code != 0 {
        return Err(DscError::Command(exit_code, stderr.to_string()));
    }
    else {
        pre_state = serde_json::from_str(&stdout)?;
    }

    let (exit_code, stdout, stderr) = invoke_command(&resource.set.executable, resource.set.args.clone().unwrap_or_default(), desired)?;
    if exit_code != 0 {
        return Err(DscError::Command(exit_code, stderr.to_string()));
    }

    match resource.set.returns {
        Some(ReturnKind::State) => {
            let actual_value: Value = serde_json::from_str(&stdout)?;
            // for changed_properties, we compare post state to pre state
            let diff_properties = get_diff( &actual_value, &pre_state);
            return Ok(SetResult {
                before_state: pre_state,
                after_state: actual_value,
                changed_properties: Some(diff_properties),
            });
        },
        Some(ReturnKind::StateAndDiff) => {
            // command should be returning actual state as a JSON line and a list of properties that differ as separate JSON line
            let mut lines = stdout.lines();
            let actual_value: Value = serde_json::from_str(lines.next().unwrap())?;
            // TODO: need schema for diff_properties to validate against
            let diff_properties: Vec<String> = serde_json::from_str(lines.next().unwrap())?;
            return Ok(SetResult {
                before_state: pre_state,
                after_state: actual_value,
                changed_properties: Some(diff_properties),
            });
        },
        None => {
            // perform a get and compare the result to the expected state
            let get_result = invoke_get(resource, desired)?;
            // for changed_properties, we compare post state to pre state
            let diff_properties = get_diff( &get_result.actual_state, &pre_state);
            return Ok(SetResult {
                before_state: pre_state,
                after_state: get_result.actual_state,
                changed_properties: Some(diff_properties),
            });
        },
    }
}

pub fn invoke_test(resource: &ResourceManifest, expected: &str) -> Result<TestResult, DscError> {
    let (exit_code, stdout, stderr) = invoke_command(&resource.test.executable, resource.test.args.clone().unwrap_or_default(), expected)?;
    if exit_code != 0 {
        return Err(DscError::Command(exit_code, stderr.to_string()));
    }

    let expected_value: Value = serde_json::from_str(expected)?;
    match resource.test.returns {
        Some(ReturnKind::State) => {
            let actual_value: Value = serde_json::from_str(&stdout)?;
            let diff_properties = get_diff(&expected_value, &actual_value);
            return Ok(TestResult {
                expected_state: expected_value,
                actual_state: actual_value,
                diff_properties: Some(diff_properties),
            });
        },
        Some(ReturnKind::StateAndDiff) => {
            // command should be returning actual state as a JSON line and a list of properties that differ as separate JSON line
            let mut lines = stdout.lines();
            let actual_value: Value = serde_json::from_str(lines.next().unwrap())?;
            let diff_properties: Vec<String> = serde_json::from_str(lines.next().unwrap())?;
            return Ok(TestResult {
                expected_state: expected_value,
                actual_state: actual_value,
                diff_properties: Some(diff_properties),
            });
        },
        None => {
            // perform a get and compare the result to the expected state
            let get_result = invoke_get(resource, expected)?;
            let diff_properties = get_diff(&expected_value, &get_result.actual_state);
            return Ok(TestResult {
                expected_state: expected_value,
                actual_state: get_result.actual_state,
                diff_properties: Some(diff_properties),
            });
        },
    }
}

fn get_diff(expected: &Value, actual: &Value) -> Vec<String> {
    let mut diff_properties: Vec<String> = Vec::new();
    if expected.is_null() {
        return diff_properties;
    }

    for (key, value) in expected.as_object().unwrap() {
        // skip meta properties
        if key.starts_with("_") || key.starts_with("$") {
            continue;
        }

        if value.is_object() {
            let sub_diff = get_diff(value, &actual[key]);
            if sub_diff.len() > 0 {
                diff_properties.push(key.to_string());
            }
        }
        else {
            match actual.as_object() {
                Some(actual_object) => {
                    if !actual_object.contains_key(key) {
                        diff_properties.push(key.to_string());
                    }
                    else {
                        if value != &actual[key] {
                            diff_properties.push(key.to_string());
                        }
                    }
                },
                None => {
                    diff_properties.push(key.to_string());
                },
            }
        }            
    }
    diff_properties
}
