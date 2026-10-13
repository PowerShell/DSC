// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::server::mcp_server::McpServer;
use dsc_lib::{
    DscManager, configure::config_doc::ExecutionKind,
    discovery::discovery_trait::DiscoveryFilter,
    dscresources::{
        dscresource::Invoke,
        invoke_result::{
            DeleteResult,
            DeleteResultKind,
            ExportResult,
            GetResult,
            SetResult,
            TestResult,
        },
    }, types::FullyQualifiedTypeName
};
use rmcp::{ErrorData as McpError, Json, tool, tool_router, handler::server::wrapper::Parameters};
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
    Delete,
}

#[derive(Serialize, JsonSchema)]
#[serde(untagged)]
pub enum ResourceOperationResult {
    GetResult(GetResult),
    SetResult(SetResult),
    TestResult(TestResult),
    ExportResult(ExportResult),
    DeleteResult { success: bool },
    DeleteWhatIfResult(DeleteResult),
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
    pub resource_type: FullyQualifiedTypeName,
    #[schemars(description = "The properties to pass to the DSC resource as JSON.  Must match the resource JSON schema from `show_dsc_resource` tool.")]
    pub properties_json: String,
    #[schemars(description = "When true and operation is 'set' or 'delete', simulate the change (what-if / dry-run) instead of applying it. Resources without native what-if support return a synthetic result derived from 'test'. Only valid with the 'set' and 'delete' operations.")]
    #[serde(default)]
    pub what_if: Option<bool>,
}

#[tool_router(router = invoke_dsc_resource_router, vis = "pub")]
impl McpServer {
    #[tool(
        description = "Invoke a DSC resource operation (Get, Set, Test, Export, Delete) with specified properties in JSON format. Set 'what_if' to true to preview a Set or Delete without applying changes.",
        annotations(
            title = "Invoke a DSC resource operation (Get, Set, Test, Export, Delete) with specified properties in JSON format and what-if support",
            read_only_hint = false,
            destructive_hint = true,
            idempotent_hint = true,
            open_world_hint = true,
        )
    )]
    pub async fn invoke_dsc_resource(&self, Parameters(InvokeDscResourceRequest { operation, resource_type, properties_json, what_if }): Parameters<InvokeDscResourceRequest>) -> Result<Json<InvokeDscResourceResponse>, McpError> {
        let result = task::spawn_blocking(move || {
            let execution_kind = if what_if.unwrap_or(false) {
                if !matches!(operation, DscOperation::Set | DscOperation::Delete) {
                    return Err(McpError::invalid_params(t!("server.invoke_dsc_resource.whatIfNotSupported"), None));
                }
                ExecutionKind::WhatIf
            } else {
                ExecutionKind::Actual
            };
            let mut dsc = DscManager::new();
            let Some(resource) = dsc.find_resource(&DiscoveryFilter::new(&resource_type, None, None)).unwrap_or(None) else {
                return Err(McpError::invalid_request(t!("server.invoke_dsc_resource.resourceNotFound", resource = resource_type), None));
            };
            match operation {
                DscOperation::Get => {
                    let result = match resource.get(&properties_json) {
                        Ok(res) => res,
                        Err(e) => return Err(McpError::internal_error(e.to_string(), None)),
                    };
                    Ok(ResourceOperationResult::GetResult(result))
                },
                DscOperation::Set => {
                    let result = match resource.set(&properties_json, false, &execution_kind) {
                        Ok(res) => res,
                        Err(e) => return Err(McpError::internal_error(e.to_string(), None)),
                    };
                    Ok(ResourceOperationResult::SetResult(result))
                },
                DscOperation::Test => {
                    let result = match resource.test(&properties_json) {
                        Ok(res) => res,
                        Err(e) => return Err(McpError::internal_error(e.to_string(), None)),
                    };
                    Ok(ResourceOperationResult::TestResult(result))
                },
                DscOperation::Delete => {
                    match resource.delete(&properties_json, &execution_kind) {
                        Ok(DeleteResultKind::ResourceActual) => Ok(ResourceOperationResult::DeleteResult { success: true }),
                        Ok(DeleteResultKind::ResourceWhatIf(delete_result)) => Ok(ResourceOperationResult::DeleteWhatIfResult(delete_result)),
                        Ok(DeleteResultKind::SyntheticWhatIf(test_result)) => Ok(ResourceOperationResult::TestResult(test_result)),
                        Err(e) => Err(McpError::internal_error(e.to_string(), None)),
                    }
                },
                DscOperation::Export => {
                    let result = match resource.export(&properties_json) {
                        Ok(res) => res,
                        Err(e) => return Err(McpError::internal_error(e.to_string(), None)),
                    };
                    Ok(ResourceOperationResult::ExportResult(result))
                }
            }
        }).await.map_err(|e| McpError::internal_error(e.to_string(), None))??;

        Ok(Json(InvokeDscResourceResponse { result }))
    }
}
