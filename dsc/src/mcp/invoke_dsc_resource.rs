// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::mcp::mcp_server::McpServer;
use dsc_lib::{
    configure::config_doc::ExecutionKind,
    dscresources::{
        dscresource::Invoke,
        invoke_result::{ExportResult, GetResult, SetResult, TestResult},
    },
    DscManager,
};
use rmcp::{handler::server::wrapper::Parameters, tool, tool_router, ErrorData as McpError, Json};
use rust_i18n::t;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tokio::task;

#[derive(Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum DscOperation {
    Get,
    Set,
    Test,
    Export,
}

#[derive(Serialize, JsonSchema)]
#[serde(untagged)]
pub enum ResourceOperationResult {
    GetResult(GetResult),
    SetResult(SetResult),
    TestResult(TestResult),
    ExportResult(ExportResult),
}

#[derive(Serialize, JsonSchema)]
pub struct InvokeDscResourceResponse {
    pub result: ResourceOperationResult,
}

#[derive(Deserialize, JsonSchema)]
pub struct InvokeDscResourceRequest {
    #[schemars(description = "The operation to perform on the DSC resource")]
    pub operation: DscOperation,
    #[schemars(description = "The type name of the DSC resource to invoke")]
    pub resource_type: String,
    #[schemars(
        description = "The properties to pass to the DSC resource as JSON.  Must match the resource JSON schema from `show_dsc_resource` tool."
    )]
    pub properties_json: String,
}

#[tool_router(router = invoke_dsc_resource_router, vis = "pub")]
impl McpServer {
    #[tool(
        description = "Invoke a DSC resource operation (Get, Set, Test, Export) with specified properties in JSON format",
        annotations(
            title = "Invoke a DSC resource operation (Get, Set, Test, Export) with specified properties in JSON format",
            read_only_hint = false,
            destructive_hint = true,
            idempotent_hint = true,
            open_world_hint = true,
        )
    )]
    pub async fn invoke_dsc_resource(
        &self,
        Parameters(InvokeDscResourceRequest {
            operation,
            resource_type,
            properties_json,
        }): Parameters<InvokeDscResourceRequest>,
    ) -> Result<Json<InvokeDscResourceResponse>, McpError> {
        let result = task::spawn_blocking(move || {
            let mut dsc = DscManager::new();
            let Some(resource) = dsc.find_resource(&resource_type, None) else {
                return Err(McpError::invalid_request(
                    t!("mcp.invoke_dsc_resource.resourceNotFound", resource = resource_type),
                    None,
                ));
            };
            match operation {
                DscOperation::Get => {
                    let result = match resource.get(&properties_json) {
                        Ok(res) => res,
                        Err(e) => return Err(McpError::internal_error(e.to_string(), None)),
                    };
                    Ok(ResourceOperationResult::GetResult(result))
                }
                DscOperation::Set => {
                    let result = match resource.set(&properties_json, false, &ExecutionKind::Actual) {
                        Ok(res) => res,
                        Err(e) => return Err(McpError::internal_error(e.to_string(), None)),
                    };
                    Ok(ResourceOperationResult::SetResult(result))
                }
                DscOperation::Test => {
                    let result = match resource.test(&properties_json) {
                        Ok(res) => res,
                        Err(e) => return Err(McpError::internal_error(e.to_string(), None)),
                    };
                    Ok(ResourceOperationResult::TestResult(result))
                }
                DscOperation::Export => {
                    let result = match resource.export(&properties_json) {
                        Ok(res) => res,
                        Err(e) => return Err(McpError::internal_error(e.to_string(), None)),
                    };
                    Ok(ResourceOperationResult::ExportResult(result))
                }
            }
        })
        .await
        .map_err(|e| McpError::internal_error(e.to_string(), None))??;

        Ok(Json(InvokeDscResourceResponse { result }))
    }
}
