// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::mcp::McpServer;
use dsc_lib::{
    DscManager,
    discovery::{
        command_discovery::ImportedManifest,
        discovery_trait::DiscoveryKind,
    },
    progress::ProgressFormat,
};
use rmcp::{ErrorData as McpError, Json, tool, tool_router};
use schemars::JsonSchema;
use serde::{Serialize, Deserialize};
use tokio::task;

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct ResourceListResult {
    pub resources: Vec<ImportedManifest>,
}

#[tool_router]
impl McpServer {
    #[must_use]
    pub fn new() -> Self {
        Self {
            tool_router: Self::tool_router()
        }
    }

    #[tool(
        description = "List all DSC resources available on the local machine",
        annotations(
            title = "Enumerate all available DSC resources on the local machine",
            read_only_hint = true,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = true,
        )
    )]
    async fn list_dsc_resources(&self) -> Result<Json<ResourceListResult>, McpError> {
        let result = task::spawn_blocking(move || {
            let mut dsc = DscManager::new();
            let mut resources = Vec::new();
            for resource in dsc.list_available(&DiscoveryKind::Resource, "*", "", ProgressFormat::None) {
                resources.push(resource);
            }
            ResourceListResult { resources }
        }).await.map_err(|e| McpError::internal_error(e.to_string(), None))?;

        Ok(Json(result))
    }
}
