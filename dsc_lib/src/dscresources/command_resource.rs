// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use jsonschema::JSONSchema;
use serde_json::Value;
use std::{process::Command, io::{Write, Read}, process::Stdio};

use crate::dscerror::DscError;
use super::{dscresource::get_diff,resource_manifest::{ResourceManifest, ReturnKind, SchemaKind}, invoke_result::{GetResult, SetResult, TestResult, ValidateResult, ExportResult}};

pub const EXIT_PROCESS_TERMINATED: i32 = 0x102;

/// Invoke the get operation on a resource
///
/// # Arguments
///
/// * `resource` - The resource manifest
/// * `filter` - The filter to apply to the resource in JSON
///
/// # Errors
///
/// Error returned if the resource does not successfully get the current state
pub fn invoke_get(resource: &ResourceManifest, cwd: &str, filter: &str) -> Result<GetResult, DscError> {
    if !filter.is_empty() && resource.get.input.is_some() {
        verify_json(resource, cwd, filter)?;
    }

    let (exit_code, stdout, stderr) = invoke_command(&resource.get.executable, resource.get.args.clone(), Some(filter), Some(cwd))?;
    //println!("{stdout}");
    if exit_code != 0 {
        return Err(DscError::Command(resource.resource_type.clone(), exit_code, stderr));
    }

    let result: Value = match serde_json::from_str(&stdout){
        Result::Ok(r) => {r},
        Result::Err(err) => {
            return Err(DscError::Operation(format!("Failed to parse json from get {}|{}|{} -> {err}", &resource.get.executable, stdout, stderr)))
        }
    };

    Ok(GetResult {
        actual_state: result,
    })
}

/// Invoke the set operation on a resource
///
/// # Arguments
///
/// * `resource` - The resource manifest
/// * `desired` - The desired state of the resource in JSON
///
/// # Errors
///
/// Error returned if the resource does not successfully set the desired state
pub fn invoke_set(resource: &ResourceManifest, cwd: &str, desired: &str) -> Result<SetResult, DscError> {
    let Some(set) = &resource.set else {
        return Err(DscError::NotImplemented("set".to_string()));
    };
    verify_json(resource, cwd, desired)?;
    // if resource doesn't implement a pre-test, we execute test first to see if a set is needed
    if !set.pre_test.unwrap_or_default() {
        let test_result = invoke_test(resource, cwd, desired)?;
        if test_result.in_desired_state {
            return Ok(SetResult {
                before_state: test_result.desired_state,
                after_state: test_result.actual_state,
                changed_properties: None,
            });
        }
    }
    let (exit_code, stdout, stderr) = invoke_command(&resource.get.executable, resource.get.args.clone(), Some(desired), Some(cwd))?;
    let pre_state: Value = if exit_code == 0 {
        serde_json::from_str(&stdout)?
    }
    else {
        return Err(DscError::Command(resource.resource_type.clone(), exit_code, stderr));
    };
    let (exit_code, stdout, stderr) = invoke_command(&set.executable, set.args.clone(), Some(desired), Some(cwd))?;
    if exit_code != 0 {
        return Err(DscError::Command(resource.resource_type.clone(), exit_code, stderr));
    }
    match set.returns {
        Some(ReturnKind::State) => {

            let actual_value: Value = match serde_json::from_str(&stdout){
                Result::Ok(r) => {r},
                Result::Err(err) => {
                    return Err(DscError::Operation(format!("Failed to parse json from set {}|{}|{} -> {err}", &set.executable, stdout, stderr)))
                }
            };

            // for changed_properties, we compare post state to pre state
            let diff_properties = get_diff( &actual_value, &pre_state);
            Ok(SetResult {
                before_state: pre_state,
                after_state: actual_value,
                changed_properties: Some(diff_properties),
            })
        },
        Some(ReturnKind::StateAndDiff) => {
            // command should be returning actual state as a JSON line and a list of properties that differ as separate JSON line
            let mut lines = stdout.lines();
            let Some(actual_line) = lines.next() else {
                return Err(DscError::Command(resource.resource_type.clone(), exit_code, "Command did not return expected actual output".to_string()));
            };
            let actual_value: Value = serde_json::from_str(actual_line)?;
            // TODO: need schema for diff_properties to validate against
            let Some(diff_line) = lines.next() else {
                return Err(DscError::Command(resource.resource_type.clone(), exit_code, "Command did not return expected diff output".to_string()));
            };
            let diff_properties: Vec<String> = serde_json::from_str(diff_line)?;
            Ok(SetResult {
                before_state: pre_state,
                after_state: actual_value,
                changed_properties: Some(diff_properties),
            })
        },
        None => {
            // perform a get and compare the result to the expected state
            let get_result = invoke_get(resource, cwd, desired)?;
            // for changed_properties, we compare post state to pre state
            let diff_properties = get_diff( &get_result.actual_state, &pre_state);
            Ok(SetResult {
                before_state: pre_state,
                after_state: get_result.actual_state,
                changed_properties: Some(diff_properties),
            })
        },
    }
}

/// Invoke the test operation against a command resource.
///
/// # Arguments
///
/// * `resource` - The resource manifest for the command resource.
/// * `expected` - The expected state of the resource in JSON.
///
/// # Errors
///
/// Error is returned if the underlying command returns a non-zero exit code.
pub fn invoke_test(resource: &ResourceManifest, cwd: &str, expected: &str) -> Result<TestResult, DscError> {
    let Some(test) = resource.test.as_ref() else {
        return Err(DscError::NotImplemented("test".to_string()));
    };

    verify_json(resource, cwd, expected)?;
    let (exit_code, stdout, stderr) = invoke_command(&test.executable, test.args.clone(), Some(expected), Some(cwd))?;
    if exit_code != 0 {
        return Err(DscError::Command(resource.resource_type.clone(), exit_code, stderr));
    }

    let expected_value: Value = serde_json::from_str(expected)?;
    match test.returns {
        Some(ReturnKind::State) => {
            let actual_value: Value = match serde_json::from_str(&stdout){
                Result::Ok(r) => {r},
                Result::Err(err) => {
                    return Err(DscError::Operation(format!("Failed to parse json from test {}|{}|{} -> {err}", &test.executable, stdout, stderr)))
                }
            };
            let diff_properties = get_diff(&expected_value, &actual_value);
            Ok(TestResult {
                desired_state: expected_value,
                actual_state: actual_value,
                in_desired_state: diff_properties.is_empty(),
                diff_properties,
            })
        },
        Some(ReturnKind::StateAndDiff) => {
            // command should be returning actual state as a JSON line and a list of properties that differ as separate JSON line
            let mut lines = stdout.lines();
            let Some(actual_value) = lines.next() else {
                return Err(DscError::Command(resource.resource_type.clone(), exit_code, "No actual state returned".to_string()));
            };
            let actual_value: Value = serde_json::from_str(actual_value)?;
            let Some(diff_properties) = lines.next() else {
                return Err(DscError::Command(resource.resource_type.clone(), exit_code, "No diff properties returned".to_string()));
            };
            let diff_properties: Vec<String> = serde_json::from_str(diff_properties)?;
            Ok(TestResult {
                desired_state: expected_value,
                actual_state: actual_value,
                in_desired_state: diff_properties.is_empty(),
                diff_properties,
            })
        },
        None => {
            // perform a get and compare the result to the expected state
            let get_result = invoke_get(resource, cwd, expected)?;
            let diff_properties = get_diff(&expected_value, &get_result.actual_state);
            Ok(TestResult {
                desired_state: expected_value,
                actual_state: get_result.actual_state,
                in_desired_state: diff_properties.is_empty(),
                diff_properties,
            })
        },
    }
}

pub fn invoke_validate(resource: &ResourceManifest, cwd: &str, config: &str) -> Result<ValidateResult, DscError> {
    // TODO: use schema to validate config if validate is not implemented
    let Some(validate) = resource.validate.as_ref() else {
        return Err(DscError::NotImplemented("validate".to_string()));
    };

    let (exit_code, stdout, stderr) = invoke_command(&validate.executable, validate.args.clone(), Some(config), Some(cwd))?;
    if exit_code != 0 {
        return Err(DscError::Command(resource.resource_type.clone(), exit_code, stderr));
    }

    let result: ValidateResult = serde_json::from_str(&stdout)?;
    Ok(result)
}

/// Get the JSON schema for a resource
///
/// # Arguments
///
/// * `resource` - The resource manifest
///
/// # Errors
///
/// Error if schema is not available or if there is an error getting the schema
pub fn get_schema(resource: &ResourceManifest, cwd: &str) -> Result<String, DscError> {
    let Some(schema_kind) = resource.schema.as_ref() else {
        return Err(DscError::SchemaNotAvailable(resource.resource_type.clone()));
    };

    match schema_kind {
        SchemaKind::Command(ref command) => {
            let (exit_code, stdout, stderr) = invoke_command(&command.executable, command.args.clone(), None, Some(cwd))?;
            if exit_code != 0 {
                return Err(DscError::Command(resource.resource_type.clone(), exit_code, stderr));
            }
            Ok(stdout)
        },
        SchemaKind::Embedded(ref schema) => {
            let json = serde_json::to_string(schema)?;
            Ok(json)
        },
        SchemaKind::Url(ref url) => {
            // TODO: cache downloaded schemas so we don't have to download them every time
            let mut response = reqwest::blocking::get(url)?;
            if !response.status().is_success() {
                return Err(DscError::HttpStatus(response.status()));
            }

            let mut body = String::new();
            response.read_to_string(&mut body)?;
            Ok(body)
        },
    }
}

pub fn invoke_export(resource: &ResourceManifest, cwd: &str) -> Result<ExportResult, DscError> {

    let (exit_code, stdout, stderr) = invoke_command(&resource.export.clone().unwrap().executable, resource.export.clone().unwrap().args.clone(), None, Some(cwd))?;
    if exit_code != 0 {
        return Err(DscError::Command(resource.resource_type.clone(), exit_code, stderr));
    }

    let mut instances: Vec<Value> = Vec::new();
    for line in stdout.lines()
    {
        let instance: Value = match serde_json::from_str(&line){
            Result::Ok(r) => {r},
            Result::Err(err) => {
                return Err(DscError::Operation(format!("Failed to parse json from export {}|{}|{} -> {err}", &resource.export.clone().unwrap().executable, stdout, stderr)))
            }
        };
        instances.push(instance);
    }

    Ok(ExportResult {
        actual_state: instances,
    })
}

/// Invoke a command and return the exit code, stdout, and stderr.
///
/// # Arguments
///
/// * `executable` - The command to execute
/// * `args` - Optional arguments to pass to the command
/// * `input` - Optional input to pass to the command
/// * `cwd` - Optional working directory to execute the command in
///
/// # Errors
///
/// Error is returned if the command fails to execute or stdin/stdout/stderr cannot be opened.
pub fn invoke_command(executable: &str, args: Option<Vec<String>>, input: Option<&str>, cwd: Option<&str>) -> Result<(i32, String, String), DscError> {
    let mut command = Command::new(executable);
    if input.is_some() {
        command.stdin(Stdio::piped());
    }
    command.stdout(Stdio::piped());
    command.stderr(Stdio::piped());
    if let Some(args) = args {
        command.args(args);
    }
    if let Some(cwd) = cwd {
        command.current_dir(cwd);
    }

    let mut child = command.spawn()?;
    if input.is_some() {
        // pipe to child stdin in a scope so that it is dropped before we wait
        // otherwise the pipe isn't closed and the child process waits forever
        let Some(mut child_stdin) = child.stdin.take() else {
            return Err(DscError::CommandOperation("Failed to open stdin".to_string(), executable.to_string()));
        };
        child_stdin.write_all(input.unwrap_or_default().as_bytes())?;
        child_stdin.flush()?;
    }

    let Some(mut child_stdout) = child.stdout.take() else {
        return Err(DscError::CommandOperation("Failed to open stdout".to_string(), executable.to_string()));
    };
    let mut stdout_buf = Vec::new();
    child_stdout.read_to_end(&mut stdout_buf)?;

    let Some(mut child_stderr) = child.stderr.take() else {
        return Err(DscError::CommandOperation("Failed to open stderr".to_string(), executable.to_string()));
    };
    let mut stderr_buf = Vec::new();
    child_stderr.read_to_end(&mut stderr_buf)?;

    let exit_status = child.wait()?;

    let exit_code = exit_status.code().unwrap_or(EXIT_PROCESS_TERMINATED);
    let stdout = String::from_utf8_lossy(&stdout_buf).to_string();
    let stderr = String::from_utf8_lossy(&stderr_buf).to_string();
    Ok((exit_code, stdout, stderr))
}

fn verify_json(resource: &ResourceManifest, cwd: &str, json: &str) -> Result<(), DscError> {

    //TODO: add to debug stream:
    //println!("verify_json - resource_type - {}", resource.resource_type);

    //TODO: remove this after schema validation for classic PS resources is implemented
    if resource.resource_type == "DSC/PowerShellGroup" {return Ok(());}

    let schema = get_schema(resource, cwd)?;
    let schema: Value = serde_json::from_str(&schema)?;
    let compiled_schema = match JSONSchema::compile(&schema) {
        Ok(schema) => schema,
        Err(e) => {
            return Err(DscError::Schema(e.to_string()));
        },
    };
    let json: Value = serde_json::from_str(json)?;
    let result = match compiled_schema.validate(&json) {
        Ok(_) => Ok(()),
        Err(err) => {
            let mut error = String::new();
            for e in err {
                error.push_str(&format!("{e} "));
            }

            Err(DscError::Schema(error))
        },
    };
    result
}
