// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::mcp::mcp_server::McpServer;
use dsc_lib::functions::{FunctionDefinition, FunctionDispatcher};
use dsc_lib::util::convert_wildcard_to_regex;
use regex::RegexBuilder;
use rmcp::{handler::server::wrapper::Parameters, tool, tool_router, ErrorData as McpError, Json};
use rust_i18n::t;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tokio::task;

#[derive(Serialize, JsonSchema)]
pub struct FunctionListResult {
    pub functions: Vec<FunctionDefinition>,
}

#[derive(Deserialize, JsonSchema)]
pub struct ListFunctionsRequest {
    #[schemars(description = "Optional function name to filter the list. Supports wildcard patterns (*, ?)")]
    pub function_filter: Option<String>,
}

#[tool_router(router = list_dsc_functions_router, vis = "pub")]
impl McpServer {
    #[tool(
        description = "List available DSC functions to be used in expressions with optional filtering by name pattern",
        annotations(
            title = "Enumerate all available DSC functions on the local machine returning name, category, description, and metadata.",
            read_only_hint = true,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = true,
        )
    )]
    pub async fn list_dsc_functions(
        &self,
        Parameters(ListFunctionsRequest { function_filter }): Parameters<ListFunctionsRequest>,
    ) -> Result<Json<FunctionListResult>, McpError> {
        let result = task::spawn_blocking(move || {
            let function_dispatcher = FunctionDispatcher::new();
            let mut functions = function_dispatcher.list();

            // apply filtering if function_filter is provided
            if let Some(name_pattern) = function_filter {
                let regex_str = convert_wildcard_to_regex(&name_pattern);
                let mut regex_builder = RegexBuilder::new(&regex_str);
                regex_builder.case_insensitive(true);

                let regex = regex_builder.build().map_err(|_| {
                    McpError::invalid_params(
                        t!("mcp.list_dsc_functions.invalidNamePattern", pattern = name_pattern),
                        None,
                    )
                })?;

                functions.retain(|func| regex.is_match(&func.name));
            }

            Ok(FunctionListResult { functions })
        })
        .await
        .map_err(|e| McpError::internal_error(e.to_string(), None))??;

        Ok(Json(result))
    }
}
