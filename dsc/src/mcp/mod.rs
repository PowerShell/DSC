// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use rmcp::{
    ErrorData as McpError,
    handler::server::tool::ToolRouter,
    model::{InitializeResult, InitializeRequestParam, ServerCapabilities, ServerInfo},
    service::{RequestContext, RoleServer},
    ServerHandler,
    ServiceExt,
    tool_handler,
    transport::stdio,
};
use rust_i18n::t;

pub mod list_dsc_resources;

#[derive(Debug, Clone)]
pub struct McpServer {
    tool_router: ToolRouter<Self>
}

impl Default for McpServer {
    fn default() -> Self {
        Self::new()
    }
}

#[tool_handler]
impl ServerHandler for McpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            capabilities: ServerCapabilities::builder()
                .enable_tools()
                .build(),
            instructions: Some(t!("mcp.mod.instructions").to_string()),
            ..Default::default()
        }
    }

    async fn initialize(&self, _request: InitializeRequestParam, _context: RequestContext<RoleServer>) -> Result<InitializeResult, McpError> {
        Ok(self.get_info())
    }
}

/// This function initializes and starts the MCP server, handling any errors that may occur.
///
/// # Errors
///
/// This function will return an error if the MCP server fails to start.
pub async fn start_mcp_server_async() -> Result<(), McpError> {
    // Initialize the MCP server
    let server = McpServer::new();

    // Try to create the service with proper error handling
    let service = server.serve(stdio()).await
        .map_err(|err|  McpError::internal_error(t!("mcp.mod.failedToInitialize", error = err.to_string()), None))?;

    // Wait for the service to complete with proper error handling
    service.waiting().await
        .map_err(|err| McpError::internal_error(t!("mcp.mod.serverWaitFailed", error = err.to_string()), None))?;

    tracing::info!("{}", t!("mcp.mod.serverStopped"));
    Ok(())
}

/// Synchronous wrapper to start the MCP server
///
/// # Errors
///
/// This function will return an error if the MCP server fails to start or if the tokio runtime cannot be created.
pub fn start_mcp_server() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let rt = tokio::runtime::Runtime::new()
        .map_err(|e| McpError::internal_error(t!("mcp.mod.failedToCreateRuntime", error = e.to_string()), None))?;

    rt.block_on(start_mcp_server_async())
        .map_err(|e| McpError::internal_error(t!("mcp.mod.failedToStart", error = e.to_string()), None))?;
    Ok(())
}
