// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use jsonschema::JSONSchema;
use serde_json::Value;
use std::{collections::HashMap, env, process::Command, io::{Write, Read}, process::Stdio};
use crate::{dscerror::DscError, dscresources::invoke_result::{ResourceGetResponse, ResourceSetResponse, ResourceTestResponse}};
use crate::configure::config_result::ResourceGetResult;
use super::{dscresource::get_diff,resource_manifest::{Kind, ResourceManifest, InputKind, ReturnKind, SchemaKind}, invoke_result::{GetResult, SetResult, TestResult, ValidateResult, ExportResult}};
use tracing::{error, warn, info, debug, trace};

pub const EXIT_PROCESS_TERMINATED: i32 = 0x102;


pub fn log_resource_traces(stderr: &str)
{
    if !stderr.is_empty()
    {
        for trace_line in stderr.lines() {
            if let Result::Ok(json_obj) = serde_json::from_str::<Value>(trace_line) {
                if let Some(msg) = json_obj.get("Error") {
                    error!("{}", msg.as_str().unwrap_or_default());
                } else if let Some(msg) = json_obj.get("Warning") {
                    warn!("{}", msg.as_str().unwrap_or_default());
                } else if let Some(msg) = json_obj.get("Info") {
                    info!("{}", msg.as_str().unwrap_or_default());
                } else if let Some(msg) = json_obj.get("Debug") {
                    debug!("{}", msg.as_str().unwrap_or_default());
                } else if let Some(msg) = json_obj.get("Trace") {
                    trace!("{}", msg.as_str().unwrap_or_default());
                };
            };
        }
    }
}

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
    let input_kind = if let Some(input_kind) = &resource.get.input {
        input_kind.clone()
    }
    else {
        InputKind::Stdin
    };

    let mut env: Option<HashMap<String, String>> = None;
    let mut input_filter: Option<&str> = None;
    let mut get_args = resource.get.args.clone();
    if !filter.is_empty() {
        verify_json(resource, cwd, filter)?;

        match input_kind {
            InputKind::Env => {
                env = Some(json_to_hashmap(filter)?);
            },
            InputKind::Stdin => {
                input_filter = Some(filter);
            },
            InputKind::Arg(arg_name) => {
                replace_token(&mut get_args, &arg_name, filter)?;
            },
        }
    }

    info!("Invoking get {} using {}", &resource.resource_type, &resource.get.executable);
    let (exit_code, stdout, stderr) = invoke_command(&resource.get.executable, get_args, input_filter, Some(cwd), env)?;
    log_resource_traces(&stderr);
    if exit_code != 0 {
        return Err(DscError::Command(resource.resource_type.clone(), exit_code, stderr));
    }

    if resource.kind == Some(Kind::Resource) {
        debug!("Verifying output of get '{}' using '{}'", &resource.resource_type, &resource.get.executable);
        verify_json(resource, cwd, &stdout)?;
    }

    let result: GetResult = if let Ok(group_response) = serde_json::from_str::<Vec<ResourceGetResult>>(&stdout) {
        trace!("Group get response: {:?}", &group_response);
        GetResult::Group(group_response)
    } else {
        let result: Value = match serde_json::from_str(&stdout) {
            Ok(r) => {r},
            Err(err) => {
                return Err(DscError::Operation(format!("Failed to parse JSON from get {}|{}|{} -> {err}", &resource.get.executable, stdout, stderr)))
            }
        };
        GetResult::Resource(ResourceGetResponse{
            actual_state: result,
        })
    };

    Ok(result)
}

/// Invoke the set operation on a resource
///
/// # Arguments
///
/// * `resource` - The resource manifest
/// * `desired` - The desired state of the resource in JSON
/// * `skip_test` - If true, skip the test and directly invoke the set operation
///
/// # Errors
///
/// Error returned if the resource does not successfully set the desired state
#[allow(clippy::too_many_lines)]
pub fn invoke_set(resource: &ResourceManifest, cwd: &str, desired: &str, skip_test: bool) -> Result<SetResult, DscError> {
    let Some(set) = resource.set.as_ref() else {
        return Err(DscError::NotImplemented("set".to_string()));
    };
    verify_json(resource, cwd, desired)?;

    let mut env: Option<HashMap<String, String>> = None;
    let mut input_desired: Option<&str> = None;
    let mut args = set.args.clone();
    match &set.input {
        InputKind::Env => {
            env = Some(json_to_hashmap(desired)?);
        },
        InputKind::Stdin => {
            input_desired = Some(desired);
        },
        InputKind::Arg(arg_token) => {
            replace_token(&mut args, arg_token, desired)?;
        },
    }

    // if resource doesn't implement a pre-test, we execute test first to see if a set is needed
    if !skip_test && !set.pre_test.unwrap_or_default() {
        info!("No pretest, invoking test {}", &resource.resource_type);
        let (in_desired_state, actual_state) = match invoke_test(resource, cwd, desired)? {
            TestResult::Group(group_response) => {
                let mut result_array: Vec<Value> = Vec::new();
                for result in group_response.results {
                    result_array.push(serde_json::to_value(result)?);
                }
                (group_response.in_desired_state, Value::from(result_array))
            },
            TestResult::Resource(response) => {
                (response.in_desired_state, response.actual_state)
            }
        };

        if in_desired_state {
            return Ok(SetResult::Resource(ResourceSetResponse{
                before_state: serde_json::from_str(desired)?,
                after_state: actual_state,
                changed_properties: None,
            }));
        }
    }

    let mut get_env: Option<HashMap<String, String>> = None;
    let mut get_input: Option<&str> = None;
    let mut get_args = resource.get.args.clone();
    match &resource.get.input {
        Some(InputKind::Env) => {
            get_env = Some(json_to_hashmap(desired)?);
        },
        Some(InputKind::Stdin) => {
            get_input = Some(desired);
        },
        Some(InputKind::Arg(arg_token)) => {
            replace_token(&mut get_args, arg_token, desired)?;
        },
        None => {
            // leave input as none
        },
    }

    info!("Getting current state for set by invoking get {} using {}", &resource.resource_type, &resource.get.executable);
    let (exit_code, stdout, stderr) = invoke_command(&resource.get.executable, get_args, get_input, Some(cwd), get_env)?;
    log_resource_traces(&stderr);
    if exit_code != 0 {
        return Err(DscError::Command(resource.resource_type.clone(), exit_code, stderr));
    }

    if resource.kind == Some(Kind::Resource) {
        debug!("Verifying output of get '{}' using '{}'", &resource.resource_type, &resource.get.executable);
        verify_json(resource, cwd, &stdout)?;
    }

    let pre_state: Value = if exit_code == 0 {
        serde_json::from_str(&stdout)?
    }
    else {
        return Err(DscError::Command(resource.resource_type.clone(), exit_code, stderr));
    };

    info!("Invoking set {} using {}", &resource.resource_type, &set.executable);
    let (exit_code, stdout, stderr) = invoke_command(&set.executable, set.args.clone(), input_desired, Some(cwd), env)?;
    log_resource_traces(&stderr);
    if exit_code != 0 {
        return Err(DscError::Command(resource.resource_type.clone(), exit_code, stderr));
    }

    match set.returns {
        Some(ReturnKind::State) => {

            if resource.kind == Some(Kind::Resource) {
                debug!("Verifying output of set '{}' using '{}'", &resource.resource_type, &set.executable);
                verify_json(resource, cwd, &stdout)?;
            }

            let actual_value: Value = match serde_json::from_str(&stdout){
                Result::Ok(r) => {r},
                Result::Err(err) => {
                    return Err(DscError::Operation(format!("Failed to parse json from set {}|{}|{} -> {err}", &set.executable, stdout, stderr)))
                }
            };

            // for changed_properties, we compare post state to pre state
            let diff_properties = get_diff( &actual_value, &pre_state);
            Ok(SetResult::Resource(ResourceSetResponse{
                before_state: pre_state,
                after_state: actual_value,
                changed_properties: Some(diff_properties),
            }))
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
            Ok(SetResult::Resource(ResourceSetResponse {
                before_state: pre_state,
                after_state: actual_value,
                changed_properties: Some(diff_properties),
            }))
        },
        None => {
            // perform a get and compare the result to the expected state
            let get_result = invoke_get(resource, cwd, desired)?;
            // for changed_properties, we compare post state to pre state
            let actual_state = match get_result {
                GetResult::Group(results) => {
                    let mut result_array: Vec<Value> = Vec::new();
                    for result in results {
                        result_array.push(serde_json::to_value(result)?);
                    }
                    Value::from(result_array)
                },
                GetResult::Resource(response) => {
                    response.actual_state
                }
            };
            let diff_properties = get_diff( &actual_state, &pre_state);
            Ok(SetResult::Resource(ResourceSetResponse {
                before_state: pre_state,
                after_state: actual_state,
                changed_properties: Some(diff_properties),
            }))
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

    let mut env: Option<HashMap<String, String>> = None;
    let mut input_expected: Option<&str> = None;
    let mut args = test.args.clone();
    match &test.input {
        InputKind::Env => {
           env = Some(json_to_hashmap(expected)?);
        },
        InputKind::Stdin => {
            input_expected = Some(expected);
        },
        InputKind::Arg(arg_token) => {
            replace_token(&mut args, arg_token, expected)?;
        },
    }

    info!("Invoking test {} using {}", &resource.resource_type, &test.executable);
    let (exit_code, stdout, stderr) = invoke_command(&test.executable, args, input_expected, Some(cwd), env)?;
    log_resource_traces(&stderr);
    if exit_code != 0 {
        return Err(DscError::Command(resource.resource_type.clone(), exit_code, stderr));
    }

    if resource.kind == Some(Kind::Resource) {
        debug!("Verifying output of test '{}' using '{}'", &resource.resource_type, &test.executable);
        verify_json(resource, cwd, &stdout)?;
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
            Ok(TestResult::Resource(ResourceTestResponse {
                desired_state: expected_value,
                actual_state: actual_value,
                in_desired_state: diff_properties.is_empty(),
                diff_properties,
            }))
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
            Ok(TestResult::Resource(ResourceTestResponse {
                desired_state: expected_value,
                actual_state: actual_value,
                in_desired_state: diff_properties.is_empty(),
                diff_properties,
            }))
        },
        None => {
            // perform a get and compare the result to the expected state
            let get_result = invoke_get(resource, cwd, expected)?;
            let actual_state = match get_result {
                GetResult::Group(results) => {
                    let mut result_array: Vec<Value> = Vec::new();
                    for result in results {
                        result_array.push(serde_json::to_value(&result)?);
                    }
                    Value::from(result_array)
                },
                GetResult::Resource(response) => {
                    response.actual_state
                }
            };
            let diff_properties = get_diff( &expected_value, &actual_state);
            Ok(TestResult::Resource(ResourceTestResponse {
                desired_state: expected_value,
                actual_state,
                in_desired_state: diff_properties.is_empty(),
                diff_properties,
            }))
        },
    }
}

/// Invoke the validate operation against a command resource.
///
/// # Arguments
///
/// * `resource` - The resource manifest for the command resource.
/// * `cwd` - The current working directory.
/// * `config` - The configuration to validate in JSON.
///
/// # Returns
///
/// * `ValidateResult` - The result of the validate operation.
///
/// # Errors
///
/// Error is returned if the underlying command returns a non-zero exit code.
pub fn invoke_validate(resource: &ResourceManifest, cwd: &str, config: &str) -> Result<ValidateResult, DscError> {
    trace!("Invoking validate '{}' using: {}", &resource.resource_type, &config);
    // TODO: use schema to validate config if validate is not implemented
    let Some(validate) = resource.validate.as_ref() else {
        return Err(DscError::NotImplemented("validate".to_string()));
    };

    let (exit_code, stdout, stderr) = invoke_command(&validate.executable, validate.args.clone(), Some(config), Some(cwd), None)?;
    log_resource_traces(&stderr);
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
            let (exit_code, stdout, stderr) = invoke_command(&command.executable, command.args.clone(), None, Some(cwd), None)?;
            log_resource_traces(&stderr);
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

/// Invoke the export operation on a resource
///
/// # Arguments
///
/// * `resource` - The resource manifest
/// * `cwd` - The current working directory
/// * `input` - Input to the command
///
/// # Returns
///
/// * `ExportResult` - The result of the export operation
///
/// # Errors
///
/// Error returned if the resource does not successfully export the current state
pub fn invoke_export(resource: &ResourceManifest, cwd: &str, input: Option<&str>) -> Result<ExportResult, DscError> {

    let Some(export) = resource.export.as_ref() else {
        return Err(DscError::Operation(format!("Export is not supported by resource {}", &resource.resource_type)))
    };

    let (exit_code, stdout, stderr) = invoke_command(&export.executable, export.args.clone(), input, Some(cwd), None)?;
    log_resource_traces(&stderr);
    if exit_code != 0 {
        return Err(DscError::Command(resource.resource_type.clone(), exit_code, stderr));
    }
    let mut instances: Vec<Value> = Vec::new();
    for line in stdout.lines()
    {
        let instance: Value = match serde_json::from_str(line){
            Result::Ok(r) => {r},
            Result::Err(err) => {
                return Err(DscError::Operation(format!("Failed to parse json from export {}|{}|{} -> {err}", &export.executable, stdout, stderr)))
            }
        };
        if resource.kind == Some(Kind::Resource) {
            debug!("Verifying output of export '{}' using '{}'", &resource.resource_type, &resource.get.executable);
            verify_json(resource, cwd, line)?;
        }
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
#[allow(clippy::implicit_hasher)]
pub fn invoke_command(executable: &str, args: Option<Vec<String>>, input: Option<&str>, cwd: Option<&str>, env: Option<HashMap<String, String>>) -> Result<(i32, String, String), DscError> {
    debug!("Invoking command '{}' with args {:?}", executable, args);
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
    if let Some(env) = env {
        command.envs(env);
    }

    if executable == "dsc" && env::var("DEBUG_DSC").is_ok() {
        // remove this env var from child process as it will fail reading from keyboard to allow attaching
        command.env_remove("DEBUG_DSC");
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
    if !stdout.is_empty() {
        trace!("STDOUT returned: {}", &stdout);
    }
    if !stderr.is_empty() {
        trace!("STDERR returned: {}", &stderr);
    }
    Ok((exit_code, stdout, stderr))
}

fn replace_token(args: &mut Option<Vec<String>>, token: &str, value: &str) -> Result<(), DscError> {
    let Some(arg_values) = args else {
        return Err(DscError::Operation("No args to replace".to_string()));
    };

    let mut found = false;
    for arg in arg_values {
        if arg == token {
            found = true;
            *arg = value.to_string();
        }
    }

    if !found {
        return Err(DscError::Operation(format!("Token {token} not found in args")));
    }

    Ok(())
}

fn verify_json(resource: &ResourceManifest, cwd: &str, json: &str) -> Result<(), DscError> {

    debug!("Verify JSON for '{}'", resource.resource_type);

    // see if resource implements validate
    if resource.validate.is_some() {
        trace!("Validating against JSON: {json}");
        let result = invoke_validate(resource, cwd, json)?;
        if result.valid {
            return Ok(());
        }

        return Err(DscError::Validation("Resource reported input JSON is not valid".to_string()));
    }

    // otherwise, use schema validation
    let schema = get_schema(resource, cwd)?;
    let schema: Value = serde_json::from_str(&schema)?;
    let compiled_schema = match JSONSchema::compile(&schema) {
        Ok(schema) => schema,
        Err(e) => {
            return Err(DscError::Schema(e.to_string()));
        },
    };
    let json: Value = serde_json::from_str(json)?;
    if let Err(err) = compiled_schema.validate(&json) {
        let mut error = String::new();
        for e in err {
            error.push_str(&format!("{e} "));
        }

        return Err(DscError::Schema(error));
    }

    Ok(())
}

fn json_to_hashmap(json: &str) -> Result<HashMap<String, String>, DscError> {
    let mut map = HashMap::new();
    let json: Value = serde_json::from_str(json)?;
    if let Value::Object(obj) = json {
        for (key, value) in obj {
            match value {
                Value::String(s) => {
                    map.insert(key, s);
                },
                Value::Bool(b) => {
                    map.insert(key, b.to_string());
                },
                Value::Number(n) => {
                    map.insert(key, n.to_string());
                },
                Value::Array(a) => {
                    // only array of number or strings is supported
                    let mut array = Vec::new();
                    for v in a {
                        match v {
                            Value::String(s) => {
                                array.push(s);
                            },
                            Value::Number(n) => {
                                array.push(n.to_string());
                            },
                            _ => {
                                return Err(DscError::Operation(format!("Unsupported array value for key {key}.  Only string and number is supported.")));
                            },
                        }
                    }
                    map.insert(key, array.join(","));
                },
                Value::Null => {
                    continue;
                }
                Value::Object(_) => {
                    return Err(DscError::Operation(format!("Unsupported value for key {key}.  Only string, bool, number, and array is supported.")));
                },
            }
        }
    }
    Ok(map)
}
