// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::server::mcp_server::McpServer;
use dsc_lib::{configure::context::Context, functions::FunctionDispatcher};
use rmcp::{ErrorData as McpError, Json, handler::server::wrapper::Parameters, tool, tool_router};
use rust_i18n::t;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::task;

#[derive(Serialize, JsonSchema)]
pub struct InvokeDscFunctionResponse {
    pub result: Value,
}

#[derive(Deserialize, JsonSchema)]
pub struct InvokeDscFunctionRequest {
    #[schemars(description = "The name of the DSC function to invoke")]
    pub function_name: String,
    #[schemars(description = "The arguments to pass to the DSC function as a JSON array")]
    pub arguments: Vec<Value>,
}

#[tool_router(router = invoke_dsc_function_router, vis = "pub")]
impl McpServer {
    #[tool(
        description = "Invoke a DSC function with the specified arguments",
        annotations(
            title = "Invoke a DSC function with the specified arguments",
            read_only_hint = true,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = false,
        )
    )]
    pub async fn invoke_dsc_function(
        &self,
        Parameters(InvokeDscFunctionRequest {
            function_name,
            arguments,
        }): Parameters<InvokeDscFunctionRequest>,
    ) -> Result<Json<InvokeDscFunctionResponse>, McpError> {
        let result = task::spawn_blocking(move || {
            let function_dispatcher = FunctionDispatcher::new();
            let mut context = Context::new();
            context.dsc_version = Some(env!("CARGO_PKG_VERSION").to_string());

            function_dispatcher
                .invoke(&function_name, &arguments, &context)
                .map_err(|e| {
                    McpError::invalid_params(
                        t!(
                            "mcp.invoke_dsc_function.failedInvoke",
                            function = function_name,
                            error = e.to_string()
                        ),
                        None,
                    )
                })
        })
        .await
        .map_err(|e| McpError::internal_error(e.to_string(), None))??;

        Ok(Json(InvokeDscFunctionResponse { result }))
    }
}
