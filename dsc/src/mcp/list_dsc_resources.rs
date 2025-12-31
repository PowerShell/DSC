// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::mcp::mcp_server::McpServer;
use dsc_lib::{
    DscManager, discovery::{
        command_discovery::ImportedManifest::Resource,
        discovery_trait::{DiscoveryFilter, DiscoveryKind},
    }, dscresources::resource_manifest::Kind, progress::ProgressFormat, types::FullyQualifiedTypeName
};
use rmcp::{ErrorData as McpError, Json, tool, tool_router, handler::server::wrapper::Parameters};
use rust_i18n::t;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use tokio::task;

#[derive(Serialize, JsonSchema)]
pub struct ResourceListResult {
    pub resources: Vec<ResourceSummary>,
}

#[derive(Serialize, JsonSchema)]
pub struct ResourceSummary {
    pub r#type: FullyQualifiedTypeName,
    pub kind: Kind,
    pub description: Option<String>,
    #[serde(rename = "requireAdapter")]
    pub require_adapter: Option<FullyQualifiedTypeName>,
}

#[derive(Deserialize, JsonSchema)]
pub struct ListResourcesRequest {
    #[schemars(description = "Filter adapted resources to only those requiring the specified adapter type.  If not specified, all non-adapted resources are returned.")]
    pub adapter: Option<String>,
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
    pub async fn list_dsc_resources(&self, Parameters(ListResourcesRequest { adapter }): Parameters<ListResourcesRequest>) -> Result<Json<ResourceListResult>, McpError> {
        let result = task::spawn_blocking(move || {
            let mut dsc = DscManager::new();
            let adapter_filter = match adapter {
                Some(adapter) => {
                    if let Some(resource) = dsc.find_resource(&DiscoveryFilter::new(&adapter, None, None)) {
                        if resource.kind != Kind::Adapter {
                            return Err(McpError::invalid_params(t!("mcp.list_dsc_resources.resourceNotAdapter", adapter = adapter), None));
                        }
                        adapter
                    } else {
                        return Err(McpError::invalid_params(t!("mcp.list_dsc_resources.adapterNotFound", adapter = adapter), None));
                    }
                },
                None => String::new(),
            };
            let mut resources = BTreeMap::<String, ResourceSummary>::new();
            for resource in dsc.list_available(&DiscoveryKind::Resource, "*", &adapter_filter, ProgressFormat::None) {
                if let Resource(resource) = resource {
                    let summary = ResourceSummary {
                        r#type: resource.type_name.clone(),
                        kind: resource.kind.clone(),
                        description: resource.description.clone(),
                        require_adapter: resource.require_adapter,
                    };
                    resources.insert(resource.type_name.to_lowercase(), summary);
                }
            }
            Ok(ResourceListResult { resources: resources.into_values().collect() })
        }).await.map_err(|e| McpError::internal_error(e.to_string(), None))??;

        Ok(Json(result))
    }
}
