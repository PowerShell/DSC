// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use clap::ValueEnum;
use jsonschema::Validator;
use rust_i18n::t;
use serde::Deserialize;
use serde_json::{Map, Value};
use std::{collections::HashMap, env, path::Path, process::Stdio};
use crate::configure::{config_doc::ExecutionKind, config_result::{ResourceGetResult, ResourceTestResult}};
use crate::dscerror::DscError;
use super::{dscresource::{get_diff, redact}, invoke_result::{ExportResult, GetResult, ResolveResult, SetResult, TestResult, ValidateResult, ResourceGetResponse, ResourceSetResponse, ResourceTestResponse, get_in_desired_state}, resource_manifest::{ArgKind, InputKind, Kind, ResourceManifest, ReturnKind, SchemaKind}};
use tracing::{error, warn, info, debug, trace};
use tokio::{io::{AsyncBufReadExt, AsyncWriteExt, BufReader}, process::Command};
use which::which;

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
pub fn invoke_get(resource: &ResourceManifest, cwd: &str, filter: &str, target_resource: Option<&str>) -> Result<GetResult, DscError> {
    debug!("{}", t!("dscresources.commandResource.invokeGet", resource = &resource.resource_type));
    let mut command_input = CommandInput { env: None, stdin: None };
    let Some(get) = &resource.get else {
        return Err(DscError::NotImplemented("get".to_string()));
    };
    let resource_type = match target_resource {
        Some(r) => r,
        None => &resource.resource_type,
    };
    let args = process_args(get.args.as_ref(), filter, resource_type);
    if !filter.is_empty() {
        verify_json(resource, cwd, filter)?;
        command_input = get_command_input(get.input.as_ref(), filter)?;
    }

    info!("{}", t!("dscresources.commandResource.invokeGetUsing", resource = &resource.resource_type, executable = &get.executable));
    let (_exit_code, stdout, stderr) = invoke_command(&get.executable, args, command_input.stdin.as_deref(), Some(cwd), command_input.env, resource.exit_codes.as_ref())?;
    if resource.kind == Some(Kind::Resource) {
        debug!("{}", t!("dscresources.commandResource.verifyOutputUsing", resource = &resource.resource_type, executable = &get.executable));
        verify_json(resource, cwd, &stdout)?;
    }

    let result: GetResult = if let Ok(group_response) = serde_json::from_str::<Vec<ResourceGetResult>>(&stdout) {
        trace!("{}", t!("dscresources.commandResource.groupGetResponse", response = &group_response : {:?}));
        GetResult::Group(group_response)
    } else {
        let result: Value = match serde_json::from_str(&stdout) {
            Ok(r) => {r},
            Err(err) => {
                return Err(DscError::Operation(t!("dscresources.commandResource.failedParseJson", executable = &get.executable, stdout = stdout, stderr = stderr, err = err).to_string()))
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
pub fn invoke_set(resource: &ResourceManifest, cwd: &str, desired: &str, skip_test: bool, execution_type: &ExecutionKind, target_resource: Option<&str>) -> Result<SetResult, DscError> {
    debug!("{}", t!("dscresources.commandResource.invokeSet", resource = &resource.resource_type));
    let operation_type: String;
    let mut is_synthetic_what_if = false;
    let set_method = match execution_type {
        ExecutionKind::Actual => {
            operation_type = "set".to_string();
            &resource.set
        },
        ExecutionKind::WhatIf => {
            operation_type = "whatif".to_string();
            if resource.what_if.is_none() {
                is_synthetic_what_if = true;
                &resource.set
            } else {
                &resource.what_if
            }
        }
    };
    let Some(set) = set_method else {
        return Err(DscError::NotImplemented("set".to_string()));
    };
    verify_json(resource, cwd, desired)?;

    // if resource doesn't implement a pre-test, we execute test first to see if a set is needed
    if !skip_test && set.pre_test != Some(true) {
        info!("{}", t!("dscresources.commandResource.noPretest", resource = &resource.resource_type));
        let test_result = invoke_test(resource, cwd, desired, target_resource)?;
        if is_synthetic_what_if {
            return Ok(test_result.into());
        }
        let (in_desired_state, actual_state) = match &test_result {
            TestResult::Group(group_response) => {
                let in_desired_state = get_in_desired_state(&test_result);
                let mut result_array: Vec<Value> = Vec::new();
                for result in group_response {
                    result_array.push(serde_json::to_value(result)?);
                }
                (in_desired_state, Value::from(result_array))
            },
            TestResult::Resource(response) => {
                (response.in_desired_state, response.actual_state.clone())
            }
        };

        if in_desired_state && execution_type == &ExecutionKind::Actual {
            let before_state = redact(&serde_json::from_str(desired)?);
            return Ok(SetResult::Resource(ResourceSetResponse{
                before_state,
                after_state: actual_state,
                changed_properties: None,
            }));
        }
    }

    if is_synthetic_what_if {
        return Err(DscError::NotImplemented(t!("dscresources.commandResource.syntheticWhatIf").to_string()));
    }

    let Some(get) = &resource.get else {
        return Err(DscError::NotImplemented("get".to_string()));
    };
    let resource_type = match target_resource {
        Some(r) => r,
        None => &resource.resource_type,
    };
    let args = process_args(get.args.as_ref(), desired, resource_type);
    let command_input = get_command_input(get.input.as_ref(), desired)?;

    info!("{}", t!("dscresources.commandResource.setGetCurrent", resource = &resource.resource_type, executable = &get.executable));
    let (exit_code, stdout, stderr) = invoke_command(&get.executable, args, command_input.stdin.as_deref(), Some(cwd), command_input.env, resource.exit_codes.as_ref())?;

    if resource.kind == Some(Kind::Resource) {
        debug!("{}", t!("dscresources.commandResource.setVerifyGet", resource = &resource.resource_type, executable = &get.executable));
        verify_json(resource, cwd, &stdout)?;
    }

    let pre_state_value: Value = if exit_code == 0 {
        serde_json::from_str(&stdout)?
    }
    else {
        return Err(DscError::Command(resource.resource_type.clone(), exit_code, stderr));
    };
    let mut pre_state = if pre_state_value.is_object() {
        let mut pre_state_map: Map<String, Value> = serde_json::from_value(pre_state_value)?;

        // if the resource is an adapter, then the `get` will return a `result`, but a full `set` expects the before state to be `resources`
        if resource.kind == Some(Kind::Adapter) && pre_state_map.contains_key("result") && !pre_state_map.contains_key("resources") {
            pre_state_map.insert("resources".to_string(), pre_state_map["result"].clone());
            pre_state_map.remove("result");
        }
        serde_json::to_value(pre_state_map)?
    } else {
        pre_state_value
    };

    let mut env: Option<HashMap<String, String>> = None;
    let mut input_desired: Option<&str> = None;
    let args = process_args(set.args.as_ref(), desired, resource_type);
    match &set.input {
        Some(InputKind::Env) => {
            env = Some(json_to_hashmap(desired)?);
        },
        Some(InputKind::Stdin) => {
            input_desired = Some(desired);
        },
        None => {
            // leave input as none
        },
    }

    info!("Invoking {} '{}' using '{}'", operation_type, &resource.resource_type, &set.executable);
    let (exit_code, stdout, stderr) = invoke_command(&set.executable, args, input_desired, Some(cwd), env, resource.exit_codes.as_ref())?;

    match set.returns {
        Some(ReturnKind::State) => {

            if resource.kind == Some(Kind::Resource) {
                debug!("{}", t!("dscresources.commandResource.setVerifyOutput", operation = operation_type, resource = &resource.resource_type, executable = &set.executable));
                verify_json(resource, cwd, &stdout)?;
            }

            let actual_value: Value = match serde_json::from_str(&stdout){
                Result::Ok(r) => {r},
                Result::Err(err) => {
                    return Err(DscError::Operation(t!("dscresources.commandResource.failedParseJson", executable = &get.executable, stdout = stdout, stderr = stderr, err = err).to_string()))
                }
            };

            // for changed_properties, we compare post state to pre state
            let diff_properties = get_diff( &actual_value, &pre_state);
            pre_state = redact(&pre_state);
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
                return Err(DscError::Command(resource.resource_type.clone(), exit_code, t!("dscresources.commandResource.setUnexpectedOutput").to_string()));
            };
            let actual_value: Value = serde_json::from_str(actual_line)?;
            // TODO: need schema for diff_properties to validate against
            let Some(diff_line) = lines.next() else {
                return Err(DscError::Command(resource.resource_type.clone(), exit_code, t!("dscresources.commandResource.setUnexpectedDiff").to_string()));
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
            let get_result = invoke_get(resource, cwd, desired, target_resource)?;
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
            pre_state = redact(&pre_state);
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
pub fn invoke_test(resource: &ResourceManifest, cwd: &str, expected: &str, target_resource: Option<&str>) -> Result<TestResult, DscError> {
    debug!("{}", t!("dscresources.commandResource.invokeTest", resource = &resource.resource_type));
    let Some(test) = &resource.test else {
        info!("{}", t!("dscresources.commandResource.testSyntheticTest", resource = &resource.resource_type));
        return invoke_synthetic_test(resource, cwd, expected, target_resource);
    };

    verify_json(resource, cwd, expected)?;

    let resource_type = match target_resource {
        Some(r) => r,
        None => &resource.resource_type,
    };
    let args = process_args(test.args.as_ref(), expected, resource_type);
    let command_input = get_command_input(test.input.as_ref(), expected)?;

    info!("{}", t!("dscresources.commandResource.invokeTestUsing", resource = &resource.resource_type, executable = &test.executable));
    let (exit_code, stdout, stderr) = invoke_command(&test.executable, args, command_input.stdin.as_deref(), Some(cwd), command_input.env, resource.exit_codes.as_ref())?;

    if resource.kind == Some(Kind::Resource) {
        debug!("{}", t!("dscresources.commandResource.testVerifyOutput", resource = &resource.resource_type, executable = &test.executable));
        verify_json(resource, cwd, &stdout)?;
    }

    if resource.kind == Some(Kind::Importer) {
        debug!("{}", t!("dscresources.commandResource.testGroupTestResponse"));
        let group_test_response: Vec<ResourceTestResult> = serde_json::from_str(&stdout)?;
        return Ok(TestResult::Group(group_test_response));
    }

    let mut expected_value: Value = serde_json::from_str(expected)?;
    match test.returns {
        Some(ReturnKind::State) => {
            let actual_value: Value = match serde_json::from_str(&stdout){
                Result::Ok(r) => {r},
                Result::Err(err) => {
                    return Err(DscError::Operation(t!("dscresources.commandResource.failedParseJson", executable = &test.executable, stdout = stdout, stderr = stderr, err = err).to_string()))
                }
            };
            let in_desired_state = get_desired_state(&actual_value)?;
            let diff_properties = get_diff(&expected_value, &actual_value);
            expected_value = redact(&expected_value);
            Ok(TestResult::Resource(ResourceTestResponse {
                desired_state: expected_value,
                actual_state: actual_value,
                in_desired_state: in_desired_state.unwrap_or(diff_properties.is_empty()),
                diff_properties,
            }))
        },
        Some(ReturnKind::StateAndDiff) => {
            // command should be returning actual state as a JSON line and a list of properties that differ as separate JSON line
            let mut lines = stdout.lines();
            let Some(actual_value) = lines.next() else {
                return Err(DscError::Command(resource.resource_type.clone(), exit_code, t!("dscresources.commandResource.testNoActualState").to_string()));
            };
            let actual_value: Value = serde_json::from_str(actual_value)?;
            let Some(diff_properties) = lines.next() else {
                return Err(DscError::Command(resource.resource_type.clone(), exit_code, t!("dscresources.commandResource.testNoDiff").to_string()));
            };
            let diff_properties: Vec<String> = serde_json::from_str(diff_properties)?;
            expected_value = redact(&expected_value);
            let in_desired_state = get_desired_state(&actual_value)?;
            Ok(TestResult::Resource(ResourceTestResponse {
                desired_state: expected_value,
                actual_state: actual_value,
                in_desired_state: in_desired_state.unwrap_or(diff_properties.is_empty()),
                diff_properties,
            }))
        },
        None => {
            // perform a get and compare the result to the expected state
            let get_result = invoke_get(resource, cwd, expected, target_resource)?;
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
            expected_value = redact(&expected_value);
            Ok(TestResult::Resource(ResourceTestResponse {
                desired_state: expected_value,
                actual_state,
                in_desired_state: diff_properties.is_empty(),
                diff_properties,
            }))
        },
    }
}

fn get_desired_state(actual: &Value) -> Result<Option<bool>, DscError> {
    // if actual state contains _inDesiredState, we use that to determine if the resource is in desired state
    let mut in_desired_state: Option<bool> = None;
    if let Some(in_desired_state_value) = actual.get("_inDesiredState") {
        if let Some(desired_state) = in_desired_state_value.as_bool() {
            in_desired_state = Some(desired_state);
        } else {
            return Err(DscError::Operation(t!("dscresources.commandResource.inDesiredStateNotBool").to_string()));
        }
    }
    Ok(in_desired_state)
}

fn invoke_synthetic_test(resource: &ResourceManifest, cwd: &str, expected: &str, target_resource: Option<&str>) -> Result<TestResult, DscError> {
    let get_result = invoke_get(resource, cwd, expected, target_resource)?;
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
    let expected_value: Value = serde_json::from_str(expected)?;
    let diff_properties = get_diff(&expected_value, &actual_state);
    Ok(TestResult::Resource(ResourceTestResponse {
        desired_state: expected_value,
        actual_state,
        in_desired_state: diff_properties.is_empty(),
        diff_properties,
    }))
}

/// Invoke the delete operation against a command resource.
///
/// # Arguments
///
/// * `resource` - The resource manifest for the command resource.
/// * `cwd` - The current working directory.
/// * `filter` - The filter to apply to the resource in JSON.
///
/// # Errors
///
/// Error is returned if the underlying command returns a non-zero exit code.
pub fn invoke_delete(resource: &ResourceManifest, cwd: &str, filter: &str, target_resource: Option<&str>) -> Result<(), DscError> {
    let Some(delete) = &resource.delete else {
        return Err(DscError::NotImplemented("delete".to_string()));
    };

    verify_json(resource, cwd, filter)?;

    let resource_type = match target_resource {
        Some(r) => r,
        None => &resource.resource_type,
    };
    let args = process_args(delete.args.as_ref(), filter, resource_type);
    let command_input = get_command_input(delete.input.as_ref(), filter)?;

    info!("{}", t!("dscresources.commandResource.invokeDeleteUsing", resource = resource_type, executable = &delete.executable));
    let (_exit_code, _stdout, _stderr) = invoke_command(&delete.executable, args, command_input.stdin.as_deref(), Some(cwd), command_input.env, resource.exit_codes.as_ref())?;

    Ok(())
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
pub fn invoke_validate(resource: &ResourceManifest, cwd: &str, config: &str, target_resource: Option<&str>) -> Result<ValidateResult, DscError> {
    trace!("{}", t!("dscresources.commandResource.invokeValidateConfig", resource = &resource.resource_type, config = &config));
    // TODO: use schema to validate config if validate is not implemented
    let Some(validate) = resource.validate.as_ref() else {
        return Err(DscError::NotImplemented("validate".to_string()));
    };

    let resource_type = match target_resource {
        Some(r) => r,
        None => &resource.resource_type,
    };
    let args = process_args(validate.args.as_ref(), config, resource_type);
    let command_input = get_command_input(validate.input.as_ref(), config)?;

    info!("{}", t!("dscresources.commandResource.invokeValidateUsing", resource = resource_type, executable = &validate.executable));
    let (_exit_code, stdout, _stderr) = invoke_command(&validate.executable, args, command_input.stdin.as_deref(), Some(cwd), command_input.env, resource.exit_codes.as_ref())?;
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
            let (_exit_code, stdout, _stderr) = invoke_command(&command.executable, command.args.clone(), None, Some(cwd), None, resource.exit_codes.as_ref())?;
            Ok(stdout)
        },
        SchemaKind::Embedded(ref schema) => {
            let json = serde_json::to_string(schema)?;
            Ok(json)
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
pub fn invoke_export(resource: &ResourceManifest, cwd: &str, input: Option<&str>, target_resource: Option<&str>) -> Result<ExportResult, DscError> {
    let Some(export) = resource.export.as_ref() else {
        // see if get is supported and use that instead
        if resource.get.is_some() {
            info!("{}", t!("dscresources.commandResource.exportNotSupportedUsingGet", resource = &resource.resource_type));
            let get_result = invoke_get(resource, cwd, input.unwrap_or(""), target_resource)?;
            let mut instances: Vec<Value> = Vec::new();
            match get_result {
                GetResult::Group(group_response) => {
                    for result in group_response {
                        instances.push(serde_json::to_value(result)?);
                    }
                },
                GetResult::Resource(response) => {
                    instances.push(response.actual_state);
                }
            }
            return Ok(ExportResult {
                actual_state: instances,
            });
        }
        // if neither export nor get is supported, return an error
        return Err(DscError::Operation(t!("dscresources.commandResource.exportNotSupported", resource = &resource.resource_type).to_string()))
    };

    let mut command_input: CommandInput = CommandInput { env: None, stdin: None };
    let args: Option<Vec<String>>;
    let resource_type = match target_resource {
        Some(r) => r,
        None => &resource.resource_type,
    };
    if let Some(input) = input {
        if !input.is_empty() {
            verify_json(resource, cwd, input)?;

            command_input = get_command_input(export.input.as_ref(), input)?;
        }

        args = process_args(export.args.as_ref(), input, resource_type);
    } else {
        args = process_args(export.args.as_ref(), "", resource_type);
    }

    let (_exit_code, stdout, stderr) = invoke_command(&export.executable, args, command_input.stdin.as_deref(), Some(cwd), command_input.env, resource.exit_codes.as_ref())?;
    let mut instances: Vec<Value> = Vec::new();
    for line in stdout.lines()
    {
        let instance: Value = match serde_json::from_str(line){
            Result::Ok(r) => {r},
            Result::Err(err) => {
                return Err(DscError::Operation(t!("dscresources.commandResource.failedParseJson", executable = &export.executable, stdout = stdout, stderr = stderr, err = err).to_string()))
            }
        };
        if resource.kind == Some(Kind::Resource) {
            debug!("{}", t!("dscresources.commandResource.exportVerifyOutput", resource = &resource.resource_type, executable = &export.executable));
            verify_json(resource, cwd, line)?;
        }
        instances.push(instance);
    }

    Ok(ExportResult {
        actual_state: instances,
    })
}

/// Invoke the resolve operation on a resource
///
/// # Arguments
///
/// * `resource` - The resource manifest
/// * `cwd` - The current working directory
/// * `input` - Input to the command
///
/// # Returns
///
/// * `ResolveResult` - The result of the resolve operation
///
/// # Errors
///
/// Error returned if the resource does not successfully resolve the input
pub fn invoke_resolve(resource: &ResourceManifest, cwd: &str, input: &str) -> Result<ResolveResult, DscError> {
    let Some(resolve) = &resource.resolve else {
        return Err(DscError::Operation(t!("dscresources.commandResource.resolveNotSupported", resource = &resource.resource_type).to_string()));
    };

    let args = process_args(resolve.args.as_ref(), input, &resource.resource_type);
    let command_input = get_command_input(resolve.input.as_ref(), input)?;

    info!("{}", t!("dscresources.commandResource.invokeResolveUsing", resource = &resource.resource_type, executable = &resolve.executable));
    let (_exit_code, stdout, _stderr) = invoke_command(&resolve.executable, args, command_input.stdin.as_deref(), Some(cwd), command_input.env, resource.exit_codes.as_ref())?;
    let result: ResolveResult = serde_json::from_str(&stdout)?;
    Ok(result)
}

/// Asynchronously invoke a command and return the exit code, stdout, and stderr.
///
/// # Arguments
///
/// * `executable` - The command to execute
/// * `args` - Optional arguments to pass to the command
/// * `input` - Optional input to pass to the command
/// * `cwd` - Optional working directory to execute the command in
/// * `env` - Optional environment variable mappings to add or update
/// * `exit_codes` - Optional descriptions of exit codes
///
/// # Errors
///
/// Error is returned if the command fails to execute or stdin/stdout/stderr cannot be opened.
///
async fn run_process_async(executable: &str, args: Option<Vec<String>>, input: Option<&str>, cwd: Option<&str>, env: Option<HashMap<String, String>>, exit_codes: Option<&HashMap<i32, String>>) -> Result<(i32, String, String), DscError> {

    // use somewhat large initial buffer to avoid early string reallocations;
    // the value is based on list result of largest of built-in adapters - WMI adapter ~500KB
    const INITIAL_BUFFER_CAPACITY: usize = 1024*1024;

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

    let mut child = match command.spawn() {
        Ok(c) => c,
        Err(e) => {
            return Err(DscError::CommandOperation(e.to_string(), executable.to_string()))
        }
    };

    let Some(stdout) = child.stdout.take() else {
        return Err(DscError::CommandOperation(t!("dscresources.commandResource.processChildStdout").to_string(), executable.to_string()));
    };
    let Some(stderr) = child.stderr.take() else {
        return Err(DscError::CommandOperation(t!("dscresources.commandResource.processChildStderr").to_string(), executable.to_string()));
    };
    let mut stdout_reader = BufReader::new(stdout).lines();
    let mut stderr_reader = BufReader::new(stderr).lines();

    if let Some(input) = input {
        trace!("Writing to command STDIN: {input}");
        let Some(mut stdin) = child.stdin.take() else {
            return Err(DscError::CommandOperation(t!("dscresources.commandResource.processChildStdin").to_string(), executable.to_string()));
        };
        if stdin.write_all(input.as_bytes()).await.is_err() {
            return Err(DscError::CommandOperation(t!("dscresources.commandResource.processWriteStdin").to_string(), executable.to_string()));
        }
        drop(stdin);
    }

    let Some(child_id) = child.id() else {
        return Err(DscError::CommandOperation(t!("dscresources.commandResource.processChildId").to_string(), executable.to_string()));
    };

    let child_task = tokio::spawn(async move {
        child.wait().await
    });

    let stdout_task = tokio::spawn(async move {
        let mut stdout_result = String::with_capacity(INITIAL_BUFFER_CAPACITY);
        while let Ok(Some(line)) = stdout_reader.next_line().await {
            stdout_result.push_str(&line);
            stdout_result.push('\n');
        }
        stdout_result
    });

    let stderr_task = tokio::spawn(async move {
        let mut filtered_stderr = String::with_capacity(INITIAL_BUFFER_CAPACITY);
        while let Ok(Some(stderr_line)) = stderr_reader.next_line().await {
            let filtered_stderr_line = log_stderr_line(&child_id, &stderr_line);
            if !filtered_stderr_line.is_empty() {
                filtered_stderr.push_str(filtered_stderr_line);
                filtered_stderr.push('\n');
            }
        }
        filtered_stderr
    });

    let exit_code = child_task.await.unwrap()?.code();
    let stdout_result = stdout_task.await.unwrap();
    let stderr_result = stderr_task.await.unwrap();

    if let Some(code) = exit_code {
        debug!("{}", t!("dscresources.commandResource.processChildExit", executable = executable, id = child_id, code = code));

        if code != 0 {
            if let Some(exit_codes) = exit_codes {
                if let Some(error_message) = exit_codes.get(&code) {
                    return Err(DscError::CommandExitFromManifest(executable.to_string(), code, error_message.to_string()));
                }
            }
            return Err(DscError::Command(executable.to_string(), code, stderr_result));
        }

        Ok((code, stdout_result, stderr_result))
    } else {
        debug!("{}", t!("dscresources.commandResource.processChildTerminated", executable = executable, id = child_id));
        Err(DscError::CommandOperation(t!("dscresources.commandResource.processTerminated").to_string(), executable.to_string()))
    }
}

fn convert_hashmap_string_keys_to_i32(input: Option<&HashMap<String, String>>) -> Result<Option<HashMap<i32, String>>, DscError> {
    if input.is_none() {
        return Ok(None);
    }

    let mut output: HashMap<i32, String> = HashMap::new();
    for (key, value) in input.unwrap() {
        if let Ok(key_int) = key.parse::<i32>() {
            output.insert(key_int, value.clone());
        } else {
            return Err(DscError::NotSupported(t!("util.invalidExitCodeKey", key = key).to_string()));
        }
    }
    Ok(Some(output))
}

/// Invoke a command and return the exit code, stdout, and stderr.
///
/// # Arguments
///
/// * `executable` - The command to execute
/// * `args` - Optional arguments to pass to the command
/// * `input` - Optional input to pass to the command
/// * `cwd` - Optional working directory to execute the command in
/// * `env` - Optional environment variable mappings to add or update
/// * `exit_codes` - Optional descriptions of exit codes
///
/// # Errors
///
/// Error is returned if the command fails to execute or stdin/stdout/stderr cannot be opened.
///
/// # Panics
///
/// Will panic if tokio runtime can't be created.
///
#[allow(clippy::implicit_hasher)]
pub fn invoke_command(executable: &str, args: Option<Vec<String>>, input: Option<&str>, cwd: Option<&str>, env: Option<HashMap<String, String>>, exit_codes: Option<&HashMap<String, String>>) -> Result<(i32, String, String), DscError> {
    let exit_codes = convert_hashmap_string_keys_to_i32(exit_codes)?;
    let mut executable = executable.to_string();
    if which(&executable).is_err() && !Path::new(&executable).is_absolute() && cwd.is_some() {
        if let Some(cwd) = cwd {
            let cwd_path = Path::new(cwd);
            let executable_path = cwd_path.join(&executable);
            if !executable_path.exists() {
                return Err(DscError::CommandOperation(t!("dscresources.commandResource.executableNotFound", executable = executable_path.display()).to_string(), executable.to_string()));
            }
            executable = executable_path.to_string_lossy().to_string();
        }
    }

    tokio::runtime::Builder::new_multi_thread().enable_all().build().unwrap().block_on(
        async {
            trace!("{}", t!("dscresources.commandResource.commandInvoke", executable = executable, args = args : {:?}));
            if let Some(cwd) = cwd {
                trace!("{}", t!("dscresources.commandResource.commandCwd", cwd = cwd));
            }

            match run_process_async(&executable, args, input, cwd, env, exit_codes.as_ref()).await {
                Ok((code, stdout, stderr)) => {
                    Ok((code, stdout, stderr))
                },
                Err(err) => {
                    error!("{}", t!("dscresources.commandResource.runProcessError", executable = executable, error = err));
                    Err(err)
                }
            }
        }
    )
}

/// Process the arguments for a command resource.
///
/// # Arguments
///
/// * `args` - The arguments to process
/// * `value` - The value to use for JSON input arguments
///
/// # Returns
///
/// A vector of strings representing the processed arguments
pub fn process_args(args: Option<&Vec<ArgKind>>, input: &str, resource_type: &str) -> Option<Vec<String>> {
    let Some(arg_values) = args else {
        debug!("{}", t!("dscresources.commandResource.noArgs"));
        return None;
    };

    let mut processed_args = Vec::<String>::new();
    for arg in arg_values {
        match arg {
            ArgKind::String(s) => {
                processed_args.push(s.clone());
            },
            ArgKind::Json { json_input_arg, mandatory } => {
                if input.is_empty() && *mandatory != Some(true) {
                    continue;
                }

                processed_args.push(json_input_arg.clone());
                processed_args.push(input.to_string());
            },
            ArgKind::ResourceType { resource_type_arg } => {
                processed_args.push(resource_type_arg.clone());
                processed_args.push(resource_type.to_string());
            }
        }
    }

    Some(processed_args)
}

struct CommandInput {
    env: Option<HashMap<String, String>>,
    stdin: Option<String>,
}

fn get_command_input(input_kind: Option<&InputKind>, input: &str) -> Result<CommandInput, DscError> {
    let mut env: Option<HashMap<String, String>> = None;
    let mut stdin: Option<String> = None;
    match input_kind {
        Some(InputKind::Env) => {
            debug!("{}", t!("dscresources.commandResource.parseAsEnvVars"));
            env = Some(json_to_hashmap(input)?);
        },
        Some(InputKind::Stdin) => {
            debug!("{}", t!("dscresources.commandResource.parseAsStdin"));
            stdin = Some(input.to_string());
        },
        None => {
            debug!("{}", t!("dscresources.commandResource.noInput"));
            // leave input as none
        },
    }

    Ok(CommandInput {
        env,
        stdin,
    })
}

fn verify_json(resource: &ResourceManifest, cwd: &str, json: &str) -> Result<(), DscError> {

    debug!("{}", t!("dscresources.commandResource.verifyJson", resource = resource.resource_type));

    // see if resource implements validate
    if resource.validate.is_some() {
        trace!("{}", t!("dscresources.commandResource.validateJson", json = json));
        let result = invoke_validate(resource, cwd, json, None)?;
        if result.valid {
            return Ok(());
        }

        return Err(DscError::Validation(t!("dscresources.commandResource.resourceInvalidJson").to_string()));
    }

    // otherwise, use schema validation
    let schema = get_schema(resource, cwd)?;
    let schema: Value = serde_json::from_str(&schema)?;
    let compiled_schema = match Validator::new(&schema) {
        Ok(schema) => schema,
        Err(e) => {
            return Err(DscError::Schema(e.to_string()));
        },
    };
    let json: Value = serde_json::from_str(json)?;
    if let Err(err) = compiled_schema.validate(&json) {
        return Err(DscError::Schema(err.to_string()));
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
                                return Err(DscError::Operation(t!("dscresources.commandResource.invalidArrayKey", key = key).to_string()));
                            },
                        }
                    }
                    map.insert(key, array.join(","));
                },
                Value::Null => {
                    // ignore null values
                },
                Value::Object(_) => {
                    return Err(DscError::Operation(t!("dscresources.commandResource.invalidKey", key = key).to_string()));
                },
            }
        }
    }
    Ok(map)
}

/// Log output from a process as traces.
///
/// # Arguments
///
/// * `process_name` - The name of the process
/// * `process_id` - The ID of the process
/// * `trace_line` - The stderr line from the process
pub fn log_stderr_line<'a>(process_id: &u32, trace_line: &'a str) -> &'a str
{
    if !trace_line.is_empty()
    {
        if let Ok(trace_object) = serde_json::from_str::<Trace>(trace_line) {
            let mut include_target = trace_object.level == TraceLevel::Debug || trace_object.level == TraceLevel::Trace;
            let target = if let Some(t) = trace_object.target.as_deref() { t } else {
                include_target = false;
                ""
            };
            let line_number = if let Some(l) = trace_object.line_number { l } else {
                include_target = false;
                0
            };
            let trace_message = if include_target {
                format!("PID {process_id}: {target}: {line_number}: {}", trace_object.fields.message)
            } else {
                format!("PID {process_id}: {}", trace_object.fields.message)
            };
            match trace_object.level {
                TraceLevel::Error => {
                    error!(trace_message);
                },
                TraceLevel::Warn => {
                    warn!(trace_message);
                },
                TraceLevel::Info => {
                    info!(trace_message);
                },
                TraceLevel::Debug => {
                    debug!(trace_message);
                },
                TraceLevel::Trace => {
                    trace!(trace_message);
                },
            }
        }
        else if let Ok(json_obj) = serde_json::from_str::<Value>(trace_line) {
            if let Some(msg) = json_obj.get("error") {
                error!("PID {process_id}: {}", msg.as_str().unwrap_or_default());
            } else if let Some(msg) = json_obj.get("warn") {
                warn!("PID {process_id}: {}", msg.as_str().unwrap_or_default());
            } else if let Some(msg) = json_obj.get("info") {
                info!("PID {process_id}: {}", msg.as_str().unwrap_or_default());
            } else if let Some(msg) = json_obj.get("debug") {
                debug!("PID {process_id}: {}", msg.as_str().unwrap_or_default());
            } else if let Some(msg) = json_obj.get("trace") {
                trace!("PID {process_id}: {}", msg.as_str().unwrap_or_default());
            } else {
                // the line is a valid json, but not one of standard trace lines - return it as filtered stderr_line
                trace!("PID {process_id}: {trace_line}");
                return trace_line;
            }
        } else {
            // the line is not a valid json - return it as filtered stderr_line
            trace!("PID {process_id}: {}", trace_line);
            return trace_line;
        }
    }

    ""
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, ValueEnum)]
pub enum TraceLevel {
    #[serde(rename = "ERROR")]
    Error,
    #[serde(rename = "WARN")]
    Warn,
    #[serde(rename = "INFO")]
    Info,
    #[serde(rename = "DEBUG")]
    Debug,
    #[serde(rename = "TRACE")]
    Trace,
}

#[derive(Deserialize)]
struct Fields {
    message: String,
}

#[derive(Deserialize)]
struct Trace {
    #[serde(rename = "timestamp")]
    _timestamp: String,
    level: TraceLevel,
    fields: Fields,
    target: Option<String>,
    // `tracing` crate uses snake_case, but we can allow for camelCase to be consistent with JSON
    #[serde(alias = "lineNumber")]
    line_number: Option<u32>,
    #[serde(rename = "span")]
    _span: Option<HashMap<String, Value>>,
    #[serde(rename = "spans")]
    _spans: Option<Vec<HashMap<String, Value>>>,
}
