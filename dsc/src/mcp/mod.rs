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

pub mod list_resources;

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
#[tokio::main(flavor = "multi_thread")]
pub async fn start_mcp_server() -> Result<(), McpError> {
    let service = match McpServer::new().serve(stdio()).await {
        Ok(service) => service,
        Err(err) => {
            tracing::error!(error = %err, "Failed to start MCP server");
            return Err(McpError::internal_error(t!("mcp.mod.failedToInitialize", error = err.to_string()), None));
        }
    };

    match service.waiting().await {
        Ok(_) => {
            tracing::info!("{}", t!("mcp.mod.serverStopped"));
        }
        Err(err) => {
            tracing::error!("{}", t!("mcp.mod.failedToWait", error = err));
        }
    }

    Ok(())
}
