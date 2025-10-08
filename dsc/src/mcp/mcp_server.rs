// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use rmcp::{
    handler::server::tool::ToolRouter,
    model::{InitializeRequestParam, InitializeResult, ServerCapabilities, ServerInfo},
    service::{RequestContext, RoleServer},
    tool_handler, ErrorData as McpError, ServerHandler,
};
use rust_i18n::t;

#[derive(Debug, Clone)]
pub struct McpServer {
    tool_router: ToolRouter<Self>,
}

impl McpServer {
    #[must_use]
    pub fn new() -> Self {
        Self {
            tool_router: Self::invoke_dsc_resource_router()
                + Self::list_dsc_functions_router()
                + Self::list_dsc_resources_router()
                + Self::show_dsc_resource_router(),
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
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            instructions: Some(t!("mcp.mod.instructions").to_string()),
            ..Default::default()
        }
    }

    async fn initialize(
        &self,
        _request: InitializeRequestParam,
        _context: RequestContext<RoleServer>,
    ) -> Result<InitializeResult, McpError> {
        Ok(self.get_info())
    }
}
