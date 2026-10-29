// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::server::mcp_server::McpServer;
use dsc_lib::{
    DscManager, configure::config_doc::SecurityContextKind, discovery::{command_discovery::ImportedManifest, discovery_trait::DiscoveryKind},
    progress::ProgressFormat, types::{FullyQualifiedTypeName, TypeNameFilter}
};
use rmcp::{ErrorData as McpError, Json, tool, tool_router};
use schemars::JsonSchema;
use serde::Serialize;
use serde_json::{Map, Value};
use tokio::task;

#[derive(Serialize, JsonSchema)]
pub struct ActionListResult {
    pub actions: Vec<ActionSummary>,
}

#[derive(Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ActionSummary {
    pub r#type: FullyQualifiedTypeName,
    pub input_schema: Option<Map<String, Value>>,
    pub output_schema: Option<Map<String, Value>>,
    pub require_security_context: Option<SecurityContextKind>,
    pub description: Option<String>,
}

#[tool_router(router = list_dsc_actions_router, vis = "pub")]
impl McpServer {
    #[tool(
description = "List summary of all DSC actions available on the local machine",
        annotations(
            title = "Enumerate all available DSC actions on the local machine returning name and description.",
            read_only_hint = true,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = true,
        )
    )]
    pub async fn list_dsc_actions(&self) -> Result<Json<ActionListResult>, McpError> {
        let result = task::spawn_blocking(move || {
            let mut dsc = DscManager::new();
            let mut actions = Vec::<ActionSummary>::new();
            for action in dsc.list_available(&DiscoveryKind::Action, &TypeNameFilter::default(), None, ProgressFormat::None) {
                if let ImportedManifest::Action(action) = action {
                    let input_schema = action.input_schema.clone();
                    let output_schema = action.output_schema.clone();

                    let summary = ActionSummary {
                        r#type: action.type_name.clone(),
                        input_schema,
                        output_schema,
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
