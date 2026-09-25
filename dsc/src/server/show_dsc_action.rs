// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::server::mcp_server::McpServer;
use dsc_lib::{
    DscManager,
    actions::action_manifest::SupportedOperations,
    configure::config_doc::SecurityContextKind,
    discovery::{DscResourceKind, discovery_trait::DiscoveryFilter},
    types::{FullyQualifiedTypeName, SemanticVersion},
};
use rmcp::{ErrorData as McpError, Json, tool, tool_router, handler::server::wrapper::Parameters};
use rust_i18n::t;
use schemars::{JsonSchema, json_schema};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::task;

fn nullable_json_object_schema(_: &mut schemars::SchemaGenerator) -> schemars::Schema {
    json_schema!({"oneOf": [{"type": "null"}, {"type": "object"}]})
}

#[derive(Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct DscAction {
    /// The namespaced name of the action.
    #[serde(rename="type")]
    pub r#type: FullyQualifiedTypeName,
    /// The version of the action.
    pub version: SemanticVersion,
    /// The supported operations of the action.
    pub supported_operations: Vec<SupportedOperations>,
    /// The description of the action.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// The author of the action.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schemars(schema_with = "nullable_json_object_schema")]
    pub input_schema: Option<Value>,
    /// The output schema of the action.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schemars(schema_with = "nullable_json_object_schema")]
    pub output_schema: Option<Value>,
    /// The security context required by the action.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub require_security_context: Option<SecurityContextKind>,
}

#[derive(Deserialize, JsonSchema)]
pub struct ShowActionRequest {
    #[schemars(description = "The type name of the action to get detailed information.")]
    pub r#type: FullyQualifiedTypeName,
}

#[tool_router(router = show_dsc_action_router, vis = "pub")]
impl McpServer {
    #[tool(
        description = "Get detailed information including the schema for a specific DSC action",
        annotations(
            title = "Get detailed information including the schema for a specific DSC action",
            read_only_hint = true,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = true,
        )
    )]
    pub async fn show_dsc_action(&self, Parameters(ShowActionRequest { r#type }): Parameters<ShowActionRequest>) -> Result<Json<DscAction>, McpError> {
        let result = task::spawn_blocking(move || {
            let mut dsc = DscManager::new();
            let Some(resource) = dsc.find_resource(&DiscoveryFilter::new(&r#type, None, None)).unwrap_or(None) else {
                return Err(McpError::invalid_params(t!("server.show_dsc_action.actionNotFound", type_name = r#type), None))
            };
            let action = match resource {
                DscResourceKind::Action(action) => action,
                DscResourceKind::Resource(_) => return Err(McpError::invalid_params(t!("server.show_dsc_action.resourceNotSupported", type_name = r#type), None)),
            };
            Ok(DscAction {
                r#type: action.type_name.clone(),
                version: action.version.clone(),
                supported_operations: action.supported_operations.clone(),
                description: action.description.clone(),
                author: action.author.clone(),
                input_schema: action.input_schema.clone(),
                output_schema: action.output_schema.clone(),
                require_security_context: action.invoke.require_security_context.clone(),
            })
        }).await.map_err(|e| McpError::internal_error(e.to_string(), None))??;

        Ok(Json(result))
    }
}
