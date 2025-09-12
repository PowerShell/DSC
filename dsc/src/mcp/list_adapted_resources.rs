// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::mcp::mcp_server::McpServer;
use dsc_lib::{
    DscManager, discovery::{
        command_discovery::ImportedManifest::Resource,
        discovery_trait::DiscoveryKind,
    }, dscresources::resource_manifest::Kind, progress::ProgressFormat
};
use rmcp::{ErrorData as McpError, Json, tool, tool_router, handler::server::wrapper::Parameters};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use tokio::task;

#[derive(Serialize, JsonSchema)]
pub struct AdaptedResourceListResult {
    pub resources: Vec<AdaptedResourceSummary>,
}

#[derive(Serialize, JsonSchema)]
pub struct AdaptedResourceSummary {
    pub r#type: String,
    pub kind: Kind,
    pub description: Option<String>,
    #[serde(rename = "requiresAdapter")]
    pub require_adapter: String,
}

#[derive(Deserialize, JsonSchema)]
pub struct ListAdaptersRequest {
    #[schemars(description = "Filter adapted resources to only those requiring the specified adapter type.")]
    pub adapter: String,
}

#[tool_router(router = list_adapted_resources_router, vis = "pub")]
impl McpServer {
    #[tool(
        description = "List summary of all adapted DSC resources available on the local machine.  Adapted resources require an adapter to run.",
        annotations(
            title = "Enumerate all available adapted DSC resources on the local machine returning name, kind, description, and required adapter.",
            read_only_hint = true,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = true,
        )
    )]
    pub async fn list_adapted_resources(&self, Parameters(ListAdaptersRequest { adapter }): Parameters<ListAdaptersRequest>) -> Result<Json<AdaptedResourceListResult>, McpError> {
        let result = task::spawn_blocking(move || {
            let mut dsc = DscManager::new();
            let mut resources = BTreeMap::<String, AdaptedResourceSummary>::new();
            for resource in dsc.list_available(&DiscoveryKind::Resource, "*", &adapter, ProgressFormat::None) {
                if let Resource(resource) = resource {
                    if let Some(require_adapter) = resource.require_adapter.as_ref() {
                        let summary = AdaptedResourceSummary {
                            r#type: resource.type_name.clone(),
                            kind: resource.kind.clone(),
                            description: resource.description.clone(),
                            require_adapter: require_adapter.clone(),
                        };
                        resources.insert(resource.type_name.to_lowercase(), summary);
                    }
                }
            }
            AdaptedResourceListResult { resources: resources.into_values().collect() }
        }).await.map_err(|e| McpError::internal_error(e.to_string(), None))?;

        Ok(Json(result))
    }
}
