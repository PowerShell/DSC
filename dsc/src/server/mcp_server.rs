// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use rmcp::{
    ErrorData as McpError,
    handler::server::tool::ToolRouter,
    model::{Implementation, InitializeResult, InitializeRequestParams, ServerCapabilities, ServerConfig},
    service::{RequestContext, RoleServer},
    ServerHandler,
    tool_handler,
};
use rust_i18n::t;

/// The name the server reports for itself in the `initialize` result.
pub const SERVER_NAME: &str = "dsc";

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
                + Self::invoke_dsc_action_router()
                + Self::invoke_dsc_expression_router()
                + Self::invoke_dsc_function_router()
                + Self::invoke_dsc_resource_router()
                + Self::list_dsc_actions_router()
                + Self::list_dsc_functions_router()
                + Self::list_dsc_resources_router()
                + Self::show_dsc_action_router()
                + Self::show_dsc_resource_router()
                + Self::show_dsc_schema_router()
        }
    }
}

impl Default for McpServer {
    fn default() -> Self {
        Self::new()
    }
}

#[tool_handler(router = self.tool_router)]
impl ServerHandler for McpServer {
    fn get_info(&self) -> ServerConfig {
        let mut info = ServerConfig::new(
            ServerCapabilities::builder()
                .enable_tools()
                .build()
        );
        info.instructions = Some(t!("server.mod.instructions").to_string());
        // Report DSC itself instead of the MCP library, so a client can check which version of
        // DSC it's talking to before it sends an operation.
        info.server_info = Implementation::new(SERVER_NAME, env!("CARGO_PKG_VERSION"));
        info
    }

    async fn initialize(&self, _request: InitializeRequestParams, _context: RequestContext<RoleServer>) -> Result<InitializeResult, McpError> {
        Ok(self.get_info())
    }
}
