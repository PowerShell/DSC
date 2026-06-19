// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::server::mcp_server::McpServer;
use dsc_lib::{configure::context::Context, functions::FunctionDispatcher};
use rmcp::{ErrorData as McpError, Json, tool, tool_router, handler::server::wrapper::Parameters};
use rust_i18n::t;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::task;

// This wrapper is needed as rmcp does not support directly returning a `Value` type
#[derive(Deserialize, Serialize, JsonSchema)]
#[serde(untagged)]
pub enum FunctionValue {
    Value(Value),
}


#[derive(Serialize, JsonSchema)]
pub struct FunctionResponse {
    pub result: FunctionValue,
}

#[derive(Deserialize, JsonSchema)]
pub struct FunctionRequest {
    #[schemars(description = "The name of the DSC function to invoke")]
    pub function: String,
    #[schemars(description = "The parameters to pass to the DSC function as JSON array.  Must match the function JSON schema from `list_dsc_function` tool.")]
    pub parameters: FunctionValue,
}

#[tool_router(router = invoke_dsc_function_router, vis = "pub")]
impl McpServer {
    #[tool(
        description = "Invoke a DSC function with specified parameters as a JSON Array.",
        annotations(
            title = "Invoke a DSC function with specified parameters as a JSON Array",
            read_only_hint = false,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = true,
        )
    )]
    pub async fn invoke_dsc_function(&self, Parameters(FunctionRequest { function, parameters }): Parameters<FunctionRequest>) -> Result<Json<FunctionResponse>, McpError> {
        let result = task::spawn_blocking(move || {
            // if parameters is not JSON array, return error
            let parameters_array: Vec<Value> = match parameters {
                FunctionValue::Value(Value::Array(arr)) => arr,
                _ => return Err(McpError::invalid_request(t!("server.invoke_dsc_function.parametersNotArray"), None)),
            };
            let function_dispatcher = FunctionDispatcher::new();
            let result = function_dispatcher.invoke(&function, &parameters_array, &Context::new())
                .map_err(|e| McpError::invalid_request(t!("server.invoke_dsc_function.functionInvocationFailed", function = function, error = e), None))?;
            Ok(FunctionResponse { result: FunctionValue::Value(result) })
        }).await.map_err(|e| McpError::internal_error(e.to_string(), None))??;

        Ok(Json(result))
    }
}
