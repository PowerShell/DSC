// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::mcp::mcp_server::McpServer;
use dsc_lib::{
    DscManager,
    discovery::discovery_trait::DiscoveryFilter,
    dscresources::{
        dscresource::{Capability, Invoke},
        resource_manifest::Kind
    }, types::{FullyQualifiedTypeName, ResourceVersion},
};
use rmcp::{ErrorData as McpError, Json, tool, tool_router, handler::server::wrapper::Parameters};
use rust_i18n::t;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::task;

#[derive(Serialize, JsonSchema)]
pub struct DscResource {
    /// The namespaced name of the resource.
    #[serde(rename="type")]
    pub type_name: FullyQualifiedTypeName,
    /// The kind of resource.
    pub kind: Kind,
    /// The version of the resource.
    pub version: ResourceVersion,
    /// The capabilities of the resource.
    pub capabilities: Vec<Capability>,
    /// The description of the resource.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// The author of the resource.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema: Option<Value>,
}

#[derive(Deserialize, JsonSchema)]
pub struct ShowResourceRequest {
    #[schemars(description = "The type name of the resource to get detailed information.")]
    pub r#type: String,
}

#[tool_router(router = show_dsc_resource_router, vis = "pub")]
impl McpServer {
    #[tool(
        description = "Get detailed information including the schema for a specific DSC resource",
        annotations(
            title = "Get detailed information including the schema for a specific DSC resource",
            read_only_hint = true,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = true,
        )
    )]
    pub async fn show_dsc_resource(&self, Parameters(ShowResourceRequest { r#type }): Parameters<ShowResourceRequest>) -> Result<Json<DscResource>, McpError> {
        let result = task::spawn_blocking(move || {
            let mut dsc = DscManager::new();
            let Some(resource) = dsc.find_resource(&DiscoveryFilter::new(&r#type, None, None)).unwrap_or(None) else {
                return Err(McpError::invalid_params(t!("mcp.show_dsc_resource.resourceNotFound", type_name = r#type), None))
            };
            let schema = match resource.schema() {
                Ok(schema_str) => serde_json::from_str(&schema_str).ok(),
                Err(_) => None,
            };
            Ok(DscResource {
                type_name: resource.type_name.clone(),
                kind: resource.kind.clone(),
                version: resource.version.clone(),
                capabilities: resource.capabilities.clone(),
                description: resource.description.clone(),
                author: resource.author.clone(),
                schema,
            })
        }).await.map_err(|e| McpError::internal_error(e.to_string(), None))??;

        Ok(Json(result))
    }
}
