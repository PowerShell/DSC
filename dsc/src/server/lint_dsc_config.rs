// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use std::collections::HashMap;

use crate::server::mcp_server::McpServer;
use crate::util::current_dsc_version;
use dsc_lib::{
    dscerror::DscError,
    lint::{LintError, LintOptions, LintResult, Linter, RuleLevel, RuleLevelOverride},
    progress::ProgressFormat,
};
use rmcp::{handler::server::wrapper::Parameters, tool, tool_router, ErrorData as McpError, Json};
use rust_i18n::t;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::task;

#[derive(Serialize, JsonSchema)]
pub struct LintDscConfigResponse {
    pub result: LintResult,
}

#[derive(Deserialize, JsonSchema)]
pub struct LintDscConfigRequest {
    #[schemars(description = "The DSC configuration document as a JSON or YAML string")]
    pub configuration: String,
    #[schemars(
        description = "Optional parameters to check against the configuration as a JSON or YAML string, with or without a top-level 'parameters' wrapper"
    )]
    pub parameters: Option<String>,
    #[schemars(
        description = "When false, skip resource discovery and the rules that inspect installed resources. Defaults to true."
    )]
    #[serde(default)]
    pub discovery: Option<bool>,
    #[schemars(description = "Optional rule levels for this run, keyed by rule identifier")]
    #[serde(default)]
    pub rules: Option<HashMap<String, RuleLevel>>,
}

/// Parse the parameters of a lint request into the input format `dsc config` accepts.
///
/// # Arguments
///
/// * `parameters` - The parameters as JSON or YAML, with or without a top-level `parameters` wrapper.
///
/// # Errors
///
/// Returns an invalid-request error when the text isn't valid JSON or YAML.
fn parse_parameters(parameters: &str) -> Result<Value, McpError> {
    let yaml = serde_yaml::from_str::<serde_yaml::Value>(parameters)
        .map_err(|e| McpError::invalid_request(format!("{}: {e}", t!("server.lint_dsc_config.invalidParameters")), None))?;
    let json = serde_json::to_value(yaml)
        .map_err(|e| McpError::invalid_request(format!("{}: {e}", t!("server.lint_dsc_config.failedConvertJson")), None))?;
    if json.get("parameters").is_some_and(Value::is_object) {
        Ok(json)
    } else {
        Ok(serde_json::json!({ "parameters": json }))
    }
}

/// Convert a linter error into the MCP error for it.
///
/// # Arguments
///
/// * `error` - The error from the linter.
fn lint_error(error: DscError) -> McpError {
    let message = format!("{}: {error}", t!("server.lint_dsc_config.lintFailed"));
    match error {
        DscError::Lint(LintError::UnknownRule(_)) => McpError::invalid_params(message, None),
        DscError::Lint(LintError::InvalidParameters(_)) => McpError::invalid_request(message, None),
        _ => McpError::internal_error(message, None),
    }
}

#[tool_router(router = lint_dsc_config_router, vis = "pub")]
impl McpServer {
    #[tool(
        description = "Lint a DSC configuration document without applying it. Returns diagnostics for problems such as invalid expressions, undeclared parameters, unresolved dependencies, and, with discovery, property values that don't match the resource schema.",
        annotations(
            title = "Lint a DSC configuration document",
            read_only_hint = true,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = false,
        )
    )]
    pub async fn lint_dsc_config(
        &self,
        Parameters(LintDscConfigRequest {
            configuration,
            parameters,
            discovery,
            rules,
        }): Parameters<LintDscConfigRequest>,
    ) -> Result<Json<LintDscConfigResponse>, McpError> {
        let result = task::spawn_blocking(move || {
            let parameters = parameters.as_deref().map(parse_parameters).transpose()?;
            let rule_levels = rules.unwrap_or_default().into_iter().map(|(id, level)| RuleLevelOverride { id, level }).collect();
            let options = LintOptions {
                discovery: discovery.unwrap_or(true),
                rule_levels,
                parameters,
                dsc_version: Some(current_dsc_version()),
                progress_format: ProgressFormat::None,
                ..LintOptions::default()
            };
            let mut linter = Linter::new(options).map_err(lint_error)?;
            linter.lint(&configuration).map_err(lint_error)
        })
        .await
        .map_err(|e| McpError::internal_error(e.to_string(), None))??;

        Ok(Json(LintDscConfigResponse { result }))
    }
}
