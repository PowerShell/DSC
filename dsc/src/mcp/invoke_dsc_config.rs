// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::mcp::mcp_server::McpServer;
use dsc_lib::{
    configure::{
        config_doc::Configuration,
        config_result::{
            ConfigurationExportResult, ConfigurationGetResult, ConfigurationSetResult,
            ConfigurationTestResult,
        },
        Configurator,
    },
    progress::ProgressFormat,
};
use rmcp::{handler::server::wrapper::Parameters, tool, tool_router, ErrorData as McpError, Json};
use rust_i18n::t;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tokio::task;

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
    #[schemars(description = "The DSC configuration document as JSON or YAML string")]
    pub configuration: String,
    #[schemars(
        description = "Optional parameters to pass to the configuration as JSON or YAML string"
    )]
    pub parameters: Option<String>,
}

#[tool_router(router = invoke_dsc_config_router, vis = "pub")]
impl McpServer {
    #[tool(
        description = "Invoke a DSC configuration operation (Get, Set, Test, Export) with optional parameters",
        annotations(
            title = "Invoke a DSC configuration operation (Get, Set, Test, Export) with optional parameters",
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
        }): Parameters<InvokeDscConfigRequest>,
    ) -> Result<Json<InvokeDscConfigResponse>, McpError> {
        let result = task::spawn_blocking(move || {
            let config: Configuration = match serde_json::from_str(&configuration) {
                Ok(config) => config,
                Err(_) => {
                    match serde_yaml::from_str::<serde_yaml::Value>(&configuration) {
                        Ok(yaml_value) => match serde_json::to_value(yaml_value) {
                            Ok(json_value) => match serde_json::from_value(json_value) {
                                Ok(config) => config,
                                Err(e) => {
                                    return Err(McpError::invalid_request(
                                        format!(
                                            "{}: {e}",
                                            t!("mcp.invoke_dsc_config.invalidConfiguration")
                                        ),
                                        None,
                                    ))
                                }
                            },
                            Err(e) => {
                                return Err(McpError::invalid_request(
                                    format!(
                                        "{}: {e}",
                                        t!("mcp.invoke_dsc_config.failedConvertJson")
                                    ),
                                    None,
                                ))
                            }
                        },
                        Err(e) => {
                            return Err(McpError::invalid_request(
                                format!(
                                    "{}: {e}",
                                    t!("mcp.invoke_dsc_config.invalidConfiguration")
                                ),
                                None,
                            ))
                        }
                    }
                }
            };

            let config_json = match serde_json::to_string(&config) {
                Ok(json) => json,
                Err(e) => {
                    return Err(McpError::internal_error(
                        format!("{}: {e}", t!("mcp.invoke_dsc_config.failedSerialize")),
                        None,
                    ))
                }
            };

            let mut configurator = match Configurator::new(&config_json, ProgressFormat::None) {
                Ok(configurator) => configurator,
                Err(e) => return Err(McpError::internal_error(e.to_string(), None)),
            };

            configurator.context.dsc_version = Some(env!("CARGO_PKG_VERSION").to_string());

            let parameters_value: Option<serde_json::Value> = if let Some(params_str) = parameters {
                let params_json = match serde_json::from_str(&params_str) {
                    Ok(json) => json,
                    Err(_) => {
                        match serde_yaml::from_str::<serde_yaml::Value>(&params_str) {
                            Ok(yaml) => match serde_json::to_value(yaml) {
                                Ok(json) => json,
                                Err(e) => {
                                    return Err(McpError::invalid_request(
                                        format!(
                                            "{}: {e}",
                                            t!("mcp.invoke_dsc_config.failedConvertJson")
                                        ),
                                        None,
                                    ))
                                }
                            },
                            Err(e) => {
                                return Err(McpError::invalid_request(
                                    format!(
                                        "{}: {e}",
                                        t!("mcp.invoke_dsc_config.invalidParameters")
                                    ),
                                    None,
                                ))
                            }
                        }
                    }
                };

                // Wrap parameters in a "parameters" field for configurator.set_context()
                Some(serde_json::json!({
                    "parameters": params_json
                }))
            } else {
                None
            };

            if let Err(e) = configurator.set_context(parameters_value.as_ref()) {
                return Err(McpError::invalid_request(
                    format!("{}: {e}", t!("mcp.invoke_dsc_config.failedSetParameters")),
                    None,
                ));
            }

            match operation {
                ConfigOperation::Get => {
                    let result = match configurator.invoke_get() {
                        Ok(res) => res,
                        Err(e) => return Err(McpError::internal_error(e.to_string(), None)),
                    };
                    Ok(ConfigOperationResult::GetResult(Box::new(result)))
                }
                ConfigOperation::Set => {
                    let result = match configurator.invoke_set(false) {
                        Ok(res) => res,
                        Err(e) => return Err(McpError::internal_error(e.to_string(), None)),
                    };
                    Ok(ConfigOperationResult::SetResult(Box::new(result)))
                }
                ConfigOperation::Test => {
                    let result = match configurator.invoke_test() {
                        Ok(res) => res,
                        Err(e) => return Err(McpError::internal_error(e.to_string(), None)),
                    };
                    Ok(ConfigOperationResult::TestResult(Box::new(result)))
                }
                ConfigOperation::Export => {
                    let result = match configurator.invoke_export() {
                        Ok(res) => res,
                        Err(e) => return Err(McpError::internal_error(e.to_string(), None)),
                    };
                    Ok(ConfigOperationResult::ExportResult(Box::new(result)))
                }
            }
        })
        .await
        .map_err(|e| McpError::internal_error(e.to_string(), None))??;

        Ok(Json(InvokeDscConfigResponse { result }))
    }
}
