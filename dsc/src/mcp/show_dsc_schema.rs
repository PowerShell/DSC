// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::{args::SchemaType, mcp::mcp_server::McpServer, util};
use rmcp::{ErrorData as McpError, Json, tool, tool_router, handler::server::wrapper::Parameters};
use schemars::{JsonSchema, json_schema};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::task;

fn json_object_schema(_: &mut schemars::SchemaGenerator) -> schemars::Schema {
    json_schema!({"type": "object"})
}

#[derive(Deserialize, JsonSchema)]
pub struct ShowSchemaRequest {
    #[schemars(description = "The schema type to retrieve the JSON schema for.")]
    pub r#type: SchemaType,
}

#[derive(Serialize, JsonSchema)]
pub struct ShowSchemaResponse {
    #[schemars(schema_with = "json_object_schema")]
    pub schema: Value,
}

#[tool_router(router = show_dsc_schema_router, vis = "pub")]
impl McpServer {
    #[tool(
        description = "Get the JSON schema for a specific part of using DSC, such as a configuration or output from an operation.",
        annotations(
            title = "Get the JSON schema for a specific part of using DSC",
            read_only_hint = true,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = true,
        )
    )]
    pub async fn show_dsc_schema(&self, Parameters(ShowSchemaRequest { r#type }): Parameters<ShowSchemaRequest>) -> Result<Json<ShowSchemaResponse>, McpError> {
        let result = task::spawn_blocking(move || {
            let schema = util::get_schema(r#type);
            Ok(ShowSchemaResponse { schema: schema.as_value().clone() })
        }).await.map_err(|e| McpError::internal_error(e.to_string(), None))??;

        Ok(Json(result))
    }
}
