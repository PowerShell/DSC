// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::mcp::mcp_server::McpServer;
use dsc_lib::functions::{FunctionDispatcher, FunctionDefinition};
use rmcp::{ErrorData as McpError, Json, tool, tool_router, handler::server::wrapper::Parameters};
use rust_i18n::t;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use regex::RegexBuilder;
use tokio::task;

#[derive(Serialize, JsonSchema)]
pub struct FunctionListResult {
    pub functions: Vec<FunctionDefinition>,
}

#[derive(Deserialize, JsonSchema)]
pub struct ListFunctionsRequest {
    #[schemars(description = "Optional function name to filter the list. Supports wildcard patterns (*, ?)")]
    pub function_name: Option<String>,
}

fn convert_wildcard_to_regex(pattern: &str) -> String {
    let escaped = regex::escape(pattern);
    let regex_pattern = escaped
        .replace(r"\*", ".*")
        .replace(r"\?", ".");
    
    if !pattern.contains('*') && !pattern.contains('?') {
        format!("^{}$", regex_pattern)
    } else {
        regex_pattern
    }
}

#[tool_router(router = list_dsc_functions_router, vis = "pub")]
impl McpServer {
    #[tool(
        description = "List available DSC functions with optional filtering by name pattern",
        annotations(
            title = "Enumerate all available DSC functions on the local machine returning name, category, description, and metadata.",
            read_only_hint = true,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = true,
        )
    )]
    pub async fn list_dsc_functions(&self, Parameters(ListFunctionsRequest { function_name }): Parameters<ListFunctionsRequest>) -> Result<Json<FunctionListResult>, McpError> {
        let result = task::spawn_blocking(move || {
            let function_dispatcher = FunctionDispatcher::new();
            let mut functions = function_dispatcher.list();
            
            // apply filtering if function_name is provided
            if let Some(name_pattern) = function_name {
                let regex_str = convert_wildcard_to_regex(&name_pattern);
                let mut regex_builder = RegexBuilder::new(&regex_str);
                regex_builder.case_insensitive(true);
                
                let regex = regex_builder.build()
                    .map_err(|_| McpError::invalid_params(
                        t!("mcp.list_dsc_functions.invalidNamePattern", pattern = name_pattern), 
                        None
                    ))?;
                
                functions.retain(|func| regex.is_match(&func.name));
            }
            
            Ok(FunctionListResult { functions })
        }).await.map_err(|e| McpError::internal_error(e.to_string(), None))??;

        Ok(Json(result))
    }
}
