// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::server::mcp_server::McpServer;
use dsc_lib::{
    DscManager, actions::{
        action_manifest::ActionManifest,
        dscaction::get_schema,
    }, configure::config_doc::SecurityContextKind, discovery::{
        command_discovery::ImportedManifest,
        discovery_trait::{
            DiscoveryFilter,
            DiscoveryKind
        },
    }, dscresources::resource_manifest::Kind, progress::ProgressFormat, types::{
        FullyQualifiedTypeName,
        TypeNameFilter
    }
};
use rmcp::{ErrorData as McpError, Json, tool, tool_router, handler::server::wrapper::Parameters};
use rust_i18n::t;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::task;

#[derive(Serialize, JsonSchema)]
pub struct ActionListResult {
    pub actions: Vec<ActionSummary>,
}

#[derive(Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ActionSummary {
    pub r#type: FullyQualifiedTypeName,
    pub input_schema: Option<Value>,
    pub output_schema: Option<Value>,
    pub require_security_context: Option<SecurityContextKind>,
    pub description: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct ListResourcesRequest {
    #[schemars(description = "Filter adapted resources to only those requiring the specified adapter type.  If not specified, all non-adapted resources are returned.")]
    pub adapter: Option<FullyQualifiedTypeName>,
}

#[tool_router(router = list_dsc_actions_router, vis = "pub")]
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
    pub async fn list_dsc_actions(&self, Parameters(ListResourcesRequest { adapter }): Parameters<ListResourcesRequest>) -> Result<Json<ActionListResult>, McpError> {
        let result = task::spawn_blocking(move || {
            let mut dsc = DscManager::new();
            let adapter_filter = match adapter {
                Some(adapter) => {
                    if let Some(resource) = dsc.find_resource(&DiscoveryFilter::new(&adapter, None, None)).unwrap_or(None) {
                        if resource.kind != Kind::Adapter {
                            return Err(McpError::invalid_params(t!("server.list_dsc_resources.resourceNotAdapter", adapter = adapter), None));
                        }
                        Some(&TypeNameFilter::Literal(resource.type_name.clone()))
                    } else {
                        return Err(McpError::invalid_params(t!("server.list_dsc_resources.adapterNotFound", adapter = adapter), None));
                    }
                },
                None => None,
            };
            let mut actions = Vec::<ActionSummary>::new();
            for action in dsc.list_available(&DiscoveryKind::Action, &TypeNameFilter::default(), adapter_filter, ProgressFormat::None) {
                if let ImportedManifest::Action(action) = action {
                    let manifest = match serde_json::from_value::<ActionManifest>(action.manifest.clone()) {
                        Ok(manifest) => manifest,
                        Err(err) => return Err(McpError::internal_error(err.to_string(), None)),
                    };
                    let input_schema = match action.invoke.input_schema.as_ref() {
                        Some(input_schema_kind) => match get_schema(input_schema_kind, &action.directory, manifest.exit_codes.as_ref()) {
                            Ok(schema) => Some(schema),
                            Err(err) => return Err(McpError::internal_error(err.to_string(), None)),
                        },
                        None => None,
                    };
                    let output_schema = match action.invoke.output_schema.as_ref() {
                        Some(output_schema_kind) => match get_schema(output_schema_kind, &action.directory, manifest.exit_codes.as_ref()) {
                            Ok(schema) => Some(schema),
                            Err(err) => return Err(McpError::internal_error(err.to_string(), None)),
                        },
                        None => None,
                    };

                    let summary = ActionSummary {
                        r#type: action.type_name.clone(),
                        input_schema: input_schema,
                        output_schema: output_schema,
                        require_security_context: action.invoke.require_security_context.clone(),
                        description: action.description.clone(),
                    };
                    actions.push(summary);
                }
            }
            Ok(ActionListResult { actions })
        }).await.map_err(|e| McpError::internal_error(e.to_string(), None))??;

        Ok(Json(result))
    }
}
