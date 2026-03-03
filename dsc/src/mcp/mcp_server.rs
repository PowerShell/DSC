// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use rmcp::{
    ErrorData as McpError,
    handler::server::tool::ToolRouter,
    model::{InitializeResult, InitializeRequestParams, ServerCapabilities, ServerInfo},
    service::{RequestContext, RoleServer},
    ServerHandler,
    tool_handler,
};
use rust_i18n::t;

#[derive(Debug, Clone)]
pub struct McpServer {
    tool_router: ToolRouter<Self>
}

impl McpServer {
    #[must_use]
    pub fn new() -> Self {
        Self {
            tool_router:
                Self::invoke_dsc_config_router()
                + Self::invoke_dsc_resource_router()
                + Self::list_dsc_functions_router()
                + Self::list_dsc_resources_router()
                + Self::show_dsc_resource_router()
        }
    }
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

    async fn initialize(&self, _request: InitializeRequestParams, _context: RequestContext<RoleServer>) -> Result<InitializeResult, McpError> {
        Ok(self.get_info())
    }
}
