// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::server::mcp_server::McpServer;
use dsc_lib::{configure::context::Context, parser::Statement};
use rmcp::{ErrorData as McpError, Json, tool, tool_router, handler::server::wrapper::Parameters};
use rust_i18n::t;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::task;

// This wrapper is needed as rmcp does not support directly returning a `Value` type
#[derive(Serialize, JsonSchema)]
#[serde(untagged)]
pub enum ExpressionResult {
    Value(Value),
}

#[derive(Serialize, JsonSchema)]
pub struct ExpressionResponse {
    pub result: ExpressionResult,
}

#[derive(Deserialize, JsonSchema)]
pub struct ExpressionRequest {
    #[schemars(description = "The DSC expression to invoke")]
    pub expression: String,
}

#[tool_router(router = invoke_dsc_expression_router, vis = "pub")]
impl McpServer {
    #[tool(
        description = "Invoke a DSC expression.",
        annotations(
            title = "Invoke a DSC expression",
            read_only_hint = false,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = true,
        )
    )]
    pub async fn invoke_dsc_expression(&self, Parameters(ExpressionRequest { expression }): Parameters<ExpressionRequest>) -> Result<Json<ExpressionResponse>, McpError> {
        let result = task::spawn_blocking(move || {
            let mut statement = Statement::new().map_err(|e| McpError::internal_error(t!("server.invoke_dsc_expression.parserInitializationFailed", error = e), None))?;
            let result = statement.parse_and_execute(&expression, &Context::new())
                .map_err(|e| McpError::invalid_request(t!("server.invoke_dsc_expression.expressionEvaluationFailed", expression = expression, error = e), None))?;
            Ok(ExpressionResponse { result: ExpressionResult::Value(result) })
        }).await.map_err(|e| McpError::internal_error(e.to_string(), None))??;

        Ok(Json(result))
    }
}
