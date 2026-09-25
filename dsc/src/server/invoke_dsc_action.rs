// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::server::mcp_server::McpServer;
use dsc_lib::{
    DscManager, configure::config_doc::ExecutionKind,
    discovery::{
        discovery_trait::DiscoveryFilter,
        DscResourceKind,
    },
    types::FullyQualifiedTypeName
};
use rmcp::{ErrorData as McpError, Json, tool, tool_router, handler::server::wrapper::Parameters};
use rust_i18n::t;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::task;

#[derive(Serialize, JsonSchema)]
pub struct InvokeDscActionResponse {
    pub result: Value,
}

#[derive(Deserialize, JsonSchema)]
pub struct InvokeDscActionRequest {
    #[schemars(description = "The type name of the DSC action to invoke")]
    pub r#type: FullyQualifiedTypeName,
    #[schemars(description = "The properties to pass to the DSC action as JSON.  Must match the resource JSON schema from `show_dsc_action` tool.")]
    pub properties_json: Option<String>,
    #[schemars(description = "When true and operation is 'set' or 'delete', simulate the change (what-if / dry-run) instead of applying it. Resources without native what-if support return a synthetic result derived from 'test'. Only valid with the 'set' and 'delete' operations.")]
    #[serde(default)]
    pub what_if: Option<bool>,
}

#[tool_router(router = invoke_dsc_action_router, vis = "pub")]
impl McpServer {
    #[tool(
        description = "Invoke a DSC action operation (Get, Set, Test, Export, Delete) with specified properties in JSON format. Set 'what_if' to true to preview a Set or Delete without applying changes.",
        annotations(
            title = "Invoke a DSC action operation (Get, Set, Test, Export, Delete) with specified properties in JSON format and what-if support",
            read_only_hint = false,
            destructive_hint = true,
            idempotent_hint = true,
            open_world_hint = true,
        )
    )]
    pub async fn invoke_dsc_action(&self, Parameters(InvokeDscActionRequest { r#type: type_name, properties_json, what_if }): Parameters<InvokeDscActionRequest>) -> Result<Json<InvokeDscActionResponse>, McpError> {
        let result = task::spawn_blocking(move || {
            let execution_kind = match what_if {
                Some(true) => ExecutionKind::WhatIf,
                _ => ExecutionKind::Actual,
            };
            let mut dsc = DscManager::new();
            let Some(resource) = dsc.find_resource(&DiscoveryFilter::new(&type_name, None, None)).unwrap_or(None) else {
                return Err(McpError::invalid_request(t!("server.invoke_dsc_action.actionNotFound", resource = type_name), None));
            };
            let action = match resource {
                DscResourceKind::Action(action) => action,
                DscResourceKind::Resource(_) => return Err(McpError::invalid_request(t!("server.invoke_dsc_action.resourceNotSupported", resource = type_name), None)),
            };
            match action.invoke(properties_json.as_deref(), &execution_kind) {
                Ok(result) => Ok(result),
                Err(e) => Err(McpError::internal_error(e.to_string(), None)),
            }
        }).await.map_err(|e| McpError::internal_error(e.to_string(), None))??;

        Ok(Json(InvokeDscActionResponse { result }))
    }
}
