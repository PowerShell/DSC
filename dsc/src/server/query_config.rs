// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::server::mcp_server::McpServer;
use crate::util::parse_input_to_json_value;
use dsc_lib::{configure::{config_doc::Configuration}};
use rmcp::{handler::server::wrapper::Parameters, tool, tool_router, ErrorData as McpError, Json};
use rust_i18n::t;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::task;

#[derive(Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum QueryKind {
    MetaData
}

#[derive(Serialize, JsonSchema)]
pub struct QueryDscConfigResponse {
    pub result: Value,
}

#[derive(Deserialize, JsonSchema)]
pub struct QueryDscConfigRequest {
    #[schemars(description = "The type of query to perform on the DSC configuration")]
    pub query_kind: QueryKind,
    #[schemars(description = "The DSC configuration document as a YAML string")]
    pub configuration: String,
}

#[tool_router(router = query_dsc_config_router, vis = "pub")]
impl McpServer {
    #[tool(
        description = "Query information about a DSC configuration",
        annotations(
            title = "Query information about a DSC configuration",
            read_only_hint = true,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = false,
        )
    )]
    pub async fn query_dsc_config(
        &self,
        Parameters(QueryDscConfigRequest {
            query_kind,
            configuration,
        }): Parameters<QueryDscConfigRequest>,
    ) -> Result<Json<QueryDscConfigResponse>, McpError> {
        let result = task::spawn_blocking(move || {
            let config_value = match parse_input_to_json_value(&configuration, "Configuration") {
                Ok(value) => value,
                Err(e) => {
                    return Err(McpError::invalid_request(
                        t!("server.query_dsc_config.invalidConfiguration"),
                        Some(serde_json::Value::String(e.to_string())),
                    ));
                }
            };
            let Ok(config) = serde_json::from_value::<Configuration>(config_value) else {
                return Err(McpError::internal_error(t!("server.query_dsc_config.jsonConversion"), None));
            };

            match query_kind {
                QueryKind::MetaData => {
                    if let Some(mut metadata) = config.metadata {
                        // remove the Microsoft.DSC property
                        metadata.microsoft = None;
                        let result = match serde_json::to_value(metadata) {
                            Ok(value) => value,
                            Err(e) => return Err(McpError::internal_error(e.to_string(), None)),
                        };
                        Ok(Json(QueryDscConfigResponse {
                            result
                        }))
                    } else {
                        Ok(Json(QueryDscConfigResponse {
                            result: Value::Null,
                        }))
                    }
                }
            }
        })
        .await
        .map_err(|e| McpError::internal_error(e.to_string(), None))??;

        Ok(result)
    }
}
