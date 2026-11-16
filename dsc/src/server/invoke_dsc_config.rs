// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::server::mcp_server::McpServer;
use crate::util::{
    current_dsc_version, EXIT_DSC_ERROR, EXIT_INVALID_ARGS, EXIT_INVALID_INPUT, EXIT_JSON_ERROR,
    EXIT_VALIDATION_FAILED,
};
use dsc_lib::{
    configure::{
        config_doc::{Configuration, ExecutionKind},
        config_result::{
            ConfigurationExportResult, ConfigurationGetResult, ConfigurationSetResult,
            ConfigurationTestResult,
        },
        Configurator,
    },
    dscerror::DscError,
    progress::ProgressFormat,
};
use rmcp::{handler::server::wrapper::Parameters, tool, tool_router, ErrorData as McpError, Json};
use rust_i18n::t;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tokio::task;

/// The member of a tool error's `data` object that carries the equivalent CLI exit code.
pub const EXIT_CODE_DATA_KEY: &str = "exitCode";

#[derive(Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum ConfigOperation {
    Get,
    Set,
    Test,
    Export,
}

#[derive(Serialize, JsonSchema)]
#[serde(untagged)]
pub enum ConfigOperationResult {
    GetResult(Box<ConfigurationGetResult>),
    SetResult(Box<ConfigurationSetResult>),
    TestResult(Box<ConfigurationTestResult>),
    ExportResult(Box<ConfigurationExportResult>),
}

#[derive(Serialize, JsonSchema)]
pub struct InvokeDscConfigResponse {
    pub result: ConfigOperationResult,
}

#[derive(Deserialize, JsonSchema)]
pub struct InvokeDscConfigRequest {
    #[schemars(description = "The operation to perform on the DSC configuration")]
    pub operation: ConfigOperation,
    #[schemars(description = "The DSC configuration document as a YAML string")]
    pub configuration: String,
    #[schemars(
        description = "Optional parameters to pass to the configuration as a JSON or YAML string, with or without a top-level 'parameters' wrapper"
    )]
    pub parameters: Option<String>,
    #[schemars(
        description = "When true and operation is 'set', simulate the change (what-if / dry-run) instead of applying it. The result includes 'metadata.Microsoft.DSC.executionType' = 'whatIf'. Only valid with the 'set' operation."
    )]
    #[serde(default)]
    pub what_if: Option<bool>,
}

/// Build the `data` member of a tool error.
///
/// The data tells a client which exit code the `dsc` CLI returns when the same failure happens
/// in a local run, so a remote client can exit with the same code.
///
/// # Arguments
///
/// * `exit_code` - The exit code the CLI uses for the failure.
///
/// # Returns
///
/// The JSON object to attach to the error.
fn exit_code_data(exit_code: u8) -> Option<Value> {
    Some(json!({ EXIT_CODE_DATA_KEY: exit_code }))
}

/// Convert a failed configuration operation into a tool error.
///
/// # Arguments
///
/// * `operation` - The operation that failed.
/// * `error` - The error returned by the configurator.
///
/// # Returns
///
/// An internal error carrying the exit code that `dsc config <operation>` returns for `error`.
fn operation_error(operation: &ConfigOperation, error: &DscError) -> McpError {
    let exit_code = match (operation, error) {
        (ConfigOperation::Test, DscError::Schema(_)) => EXIT_VALIDATION_FAILED,
        _ => EXIT_DSC_ERROR,
    };
    McpError::internal_error(error.to_string(), exit_code_data(exit_code))
}

#[tool_router(router = invoke_dsc_config_router, vis = "pub")]
impl McpServer {
    #[tool(
        description = "Invoke a DSC configuration operation (Get, Set, Test, Export) with optional parameters. Set 'what_if' to true to preview a Set without applying changes.",
        annotations(
            title = "Invoke a DSC configuration operation (Get, Set, Test, Export) with optional parameters and what-if support",
            read_only_hint = false,
            destructive_hint = true,
            idempotent_hint = true,
            open_world_hint = true,
        )
    )]
    pub async fn invoke_dsc_config(
        &self,
        Parameters(InvokeDscConfigRequest {
            operation,
            configuration,
            parameters,
            what_if,
        }): Parameters<InvokeDscConfigRequest>,
    ) -> Result<Json<InvokeDscConfigResponse>, McpError> {
        let result = task::spawn_blocking(move || {
            let config: Configuration = match serde_yaml::from_str::<serde_yaml::Value>(&configuration) {
                Ok(yaml_value) => match serde_json::to_value(yaml_value) {
                    Ok(json_value) => match serde_json::from_value(json_value) {
                        Ok(config) => config,
                        Err(e) => {
                            return Err(McpError::invalid_request(
                                format!(
                                    "{}: {e}",
                                    t!("server.invoke_dsc_config.invalidConfiguration")
                                ),
                                exit_code_data(EXIT_DSC_ERROR),
                            ))
                        }
                    },
                    Err(e) => {
                        return Err(McpError::invalid_request(
                            format!(
                                "{}: {e}",
                                t!("server.invoke_dsc_config.failedConvertJson")
                            ),
                            exit_code_data(EXIT_INVALID_INPUT),
                        ))
                    }
                },
                Err(e) => {
                    return Err(McpError::invalid_request(
                        format!(
                            "{}: {e}",
                            t!("server.invoke_dsc_config.invalidConfiguration")
                        ),
                        exit_code_data(EXIT_INVALID_INPUT),
                    ))
                }
            };

            let config_json = match serde_json::to_string(&config) {
                Ok(json) => json,
                Err(e) => {
                    return Err(McpError::internal_error(
                        format!("{}: {e}", t!("server.invoke_dsc_config.failedSerialize")),
                        exit_code_data(EXIT_JSON_ERROR),
                    ))
                }
            };

            let mut configurator = match Configurator::new_with_dsc_version(&config_json, ProgressFormat::None, current_dsc_version()) {
                Ok(configurator) => configurator,
                Err(e) => return Err(McpError::internal_error(e.to_string(), exit_code_data(EXIT_DSC_ERROR))),
            };

            if what_if.unwrap_or(false) {
                if !matches!(operation, ConfigOperation::Set) {
                    return Err(McpError::invalid_params(
                        t!("server.invoke_dsc_config.whatIfOnlySet"),
                        exit_code_data(EXIT_INVALID_ARGS),
                    ));
                }
                configurator.context.execution_type = ExecutionKind::WhatIf;
            }

            let parameters_value: Option<serde_json::Value> = if let Some(params_str) = parameters {
                let params_json = match serde_yaml::from_str::<serde_yaml::Value>(&params_str) {
                    Ok(yaml) => match serde_json::to_value(yaml) {
                        Ok(json) => json,
                        Err(e) => {
                            return Err(McpError::invalid_request(
                                format!(
                                    "{}: {e}",
                                    t!("server.invoke_dsc_config.failedConvertJson")
                                ),
                                exit_code_data(EXIT_DSC_ERROR),
                            ))
                        }
                    },
                    Err(e) => {
                        return Err(McpError::invalid_request(
                            format!(
                                "{}: {e}",
                                t!("server.invoke_dsc_config.invalidParameters")
                            ),
                            exit_code_data(EXIT_INVALID_INPUT),
                        ))
                    }
                };

                if params_json
                    .get("parameters")
                    .is_some_and(serde_json::Value::is_object)
                {
                    Some(params_json)
                } else {
                    Some(serde_json::json!({
                        "parameters": params_json
                    }))
                }
            } else {
                None
            };

            if let Err(e) = configurator.set_context(parameters_value.as_ref()) {
                return Err(McpError::invalid_request(
                    format!("{}: {e}", t!("server.invoke_dsc_config.failedSetParameters")),
                    exit_code_data(EXIT_INVALID_INPUT),
                ));
            }

            match operation {
                ConfigOperation::Get => {
                    let result = match configurator.invoke_get() {
                        Ok(res) => res,
                        Err(e) => return Err(operation_error(&operation, &e)),
                    };
                    Ok(ConfigOperationResult::GetResult(Box::new(result)))
                }
                ConfigOperation::Set => {
                    let result = match configurator.invoke_set(false) {
                        Ok(res) => res,
                        Err(e) => return Err(operation_error(&operation, &e)),
                    };
                    Ok(ConfigOperationResult::SetResult(Box::new(result)))
                }
                ConfigOperation::Test => {
                    let result = match configurator.invoke_test() {
                        Ok(res) => res,
                        Err(e) => return Err(operation_error(&operation, &e)),
                    };
                    Ok(ConfigOperationResult::TestResult(Box::new(result)))
                }
                ConfigOperation::Export => {
                    let result = match configurator.invoke_export() {
                        Ok(res) => res,
                        Err(e) => return Err(operation_error(&operation, &e)),
                    };
                    Ok(ConfigOperationResult::ExportResult(Box::new(result)))
                }
            }
        })
        .await
        .map_err(|e| McpError::internal_error(e.to_string(), exit_code_data(EXIT_DSC_ERROR)))??;

        Ok(Json(InvokeDscConfigResponse { result }))
    }
}
