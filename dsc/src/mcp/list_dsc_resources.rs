// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::mcp::mcp_server::McpServer;
use dsc_lib::{
    DscManager, discovery::{
        command_discovery::ImportedManifest::Resource,
        discovery_trait::DiscoveryKind,
    }, dscresources::resource_manifest::Kind, progress::ProgressFormat
};
use rmcp::{ErrorData as McpError, Json, tool, tool_router};
use schemars::JsonSchema;
use serde::Serialize;
use std::collections::BTreeMap;
use tokio::task;

#[derive(Serialize, JsonSchema)]
pub struct ResourceListResult {
    pub resources: Vec<ResourceSummary>,
}

#[derive(Serialize, JsonSchema)]
pub struct ResourceSummary {
    pub r#type: String,
    pub kind: Kind,
    pub description: Option<String>,
}

#[tool_router(router = list_dsc_resources_router, vis = "pub")]
impl McpServer {
    #[tool(
        description = "List summary of all DSC resources available on the local machine",
        annotations(
            title = "Enumerate all available DSC resources on the local machine returning name, kind, and description.",
            read_only_hint = true,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = true,
        )
    )]
    pub async fn list_dsc_resources(&self) -> Result<Json<ResourceListResult>, McpError> {
        let result = task::spawn_blocking(move || {
            let mut dsc = DscManager::new();
            let mut resources = BTreeMap::<String, ResourceSummary>::new();
            for resource in dsc.list_available(&DiscoveryKind::Resource, "*", "", ProgressFormat::None) {
                if let Resource(resource) = resource {
                    let summary = ResourceSummary {
                        r#type: resource.type_name.clone(),
                        kind: resource.kind.clone(),
                        description: resource.description.clone(),
                    };
                    resources.insert(resource.type_name.to_lowercase(), summary);
                }
            }
            ResourceListResult { resources: resources.into_values().collect() }
        }).await.map_err(|e| McpError::internal_error(e.to_string(), None))?;

        Ok(Json(result))
    }
}
