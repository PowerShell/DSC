// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use clap::Parser;
use dsc_lib::{
    configure::config_doc::ExecutionKind, dscresources::dscresource::Invoke, DscManager,
};
use tonic::{transport::Server, Request, Response, Status};

// Include the generated protobuf code
pub mod proto {
    tonic::include_proto!("extension");
}

use proto::bicep_extension_server::{BicepExtension, BicepExtensionServer};
use proto::{
    Empty, LocalExtensibilityOperationResponse, ResourceReference, ResourceSpecification,
    TypeFilesResponse,
};

#[derive(Debug, Default)]
pub struct BicepExtensionService;

#[tonic::async_trait]
impl BicepExtension for BicepExtensionService {
    async fn create_or_update(
        &self,
        request: Request<ResourceSpecification>,
    ) -> Result<Response<LocalExtensibilityOperationResponse>, Status> {
        let spec = request.into_inner();
        let resource_type = spec.r#type;
        let version = spec.api_version;
        let properties = spec.properties;

        tracing::debug!("CreateOrUpdate called for {resource_type}@{version:?}: {properties}");

        let mut dsc = DscManager::new();
        let Some(resource) = dsc.find_resource(&resource_type, version.as_deref()) else {
            return Err(Status::invalid_argument("Resource not found"));
        };

        let _result = match resource.set(&properties, false, &ExecutionKind::Actual) {
            Ok(res) => res,
            Err(e) => return Err(Status::internal(format!("DSC set operation failed: {e}"))),
        };

        // TODO: Use '_result'.
        let response = LocalExtensibilityOperationResponse {
            resource: Some(proto::Resource {
                r#type: resource_type,
                api_version: version,
                identifiers: String::new(),
                properties: properties,
                status: None,
            }),
            error_data: None,
        };

        Ok(Response::new(response))
    }

    async fn preview(
        &self,
        request: Request<ResourceSpecification>,
    ) -> Result<Response<LocalExtensibilityOperationResponse>, Status> {
        let spec = request.into_inner();
        let resource_type = spec.r#type;
        let version = spec.api_version;
        let properties = spec.properties;

        tracing::debug!("Preview called for {resource_type}@{version:?}: {properties}");

        let mut dsc = DscManager::new();
        let Some(resource) = dsc.find_resource(&resource_type, version.as_deref()) else {
            return Err(Status::invalid_argument("Resource not found"));
        };

        let _result = match resource.set(&properties, false, &ExecutionKind::WhatIf) {
            Ok(res) => res,
            Err(e) => {
                return Err(Status::internal(format!(
                    "DSC whatif operation failed: {e}"
                )))
            }
        };

        // TODO: Use '_result'.
        let response = LocalExtensibilityOperationResponse {
            resource: Some(proto::Resource {
                r#type: resource_type,
                api_version: version,
                identifiers: String::new(),
                properties: properties,
                status: None,
            }),
            error_data: None,
        };

        Ok(Response::new(response))
    }

    async fn get(
        &self,
        request: Request<ResourceReference>,
    ) -> Result<Response<LocalExtensibilityOperationResponse>, Status> {
        let reference = request.into_inner();
        let resource_type = reference.r#type.clone();
        let version = reference.api_version.clone();
        let identifiers = reference.identifiers.clone();

        tracing::debug!("Get called for {resource_type}@{version:?}: {identifiers}");

        let mut dsc = DscManager::new();
        let Some(resource) = dsc.find_resource(&resource_type, version.as_deref()) else {
            return Err(Status::invalid_argument("Resource not found"));
        };

        // TODO: DSC asks for 'properties' here but we only have 'identifiers' from Bicep.
        let _result = match resource.get(&identifiers) {
            Ok(res) => res,
            Err(e) => return Err(Status::internal(format!("DSC get operation failed: {e}"))),
        };

        // TODO: Use '_result'.
        let response = LocalExtensibilityOperationResponse {
            resource: Some(proto::Resource {
                r#type: resource_type,
                api_version: version,
                identifiers: identifiers,
                properties: String::new(),
                status: None,
            }),
            error_data: None,
        };

        Ok(Response::new(response))
    }

    async fn delete(
        &self,
        request: Request<ResourceReference>,
    ) -> Result<Response<LocalExtensibilityOperationResponse>, Status> {
        let reference = request.into_inner();
        let resource_type = reference.r#type.clone();
        let version = reference.api_version.clone();
        let identifiers = reference.identifiers.clone();

        tracing::debug!(
            "Delete called for {}@{:?}: {}",
            resource_type,
            version,
            identifiers
        );

        let mut dsc = DscManager::new();
        let Some(resource) = dsc.find_resource(&resource_type, version.as_deref()) else {
            return Err(Status::invalid_argument("Resource not found"));
        };

        // TODO: DSC asks for 'properties' here but we only have 'identifiers' from Bicep.
        let _result = match resource.delete(&identifiers) {
            Ok(res) => res,
            Err(e) => {
                return Err(Status::internal(format!(
                    "DSC delete operation failed: {e}"
                )))
            }
        };

        // TODO: Use '_result'.
        let response = LocalExtensibilityOperationResponse {
            resource: Some(proto::Resource {
                r#type: resource_type,
                api_version: version,
                identifiers: identifiers,
                properties: String::new(),
                status: None,
            }),
            error_data: None,
        };

        Ok(Response::new(response))
    }

    async fn get_type_files(
        &self,
        _request: Request<Empty>,
    ) -> Result<Response<TypeFilesResponse>, Status> {
        tracing::debug!("GetTypeFiles called");

        // TODO: Return actual Bicep type definitions...yet the extension already has these?
        // Perhaps this is where we can dynamically get them from the current system.
        Err(Status::unimplemented("GetTypeFiles not yet implemented"))
    }

    async fn ping(&self, _request: Request<Empty>) -> Result<Response<Empty>, Status> {
        tracing::debug!("Ping called");
        Ok(Response::new(Empty {}))
    }
}

#[derive(Parser, Debug)]
#[command(name = "dscbicep")]
#[command(about = "DSC Bicep Local Deploy Extension", long_about = None)]
struct Args {
    /// The path to the domain socket to connect on (Unix-like systems)
    #[arg(long)]
    socket: Option<String>,

    /// The named pipe to connect on (Windows)
    #[arg(long)]
    pipe: Option<String>,

    /// Wait for debugger to attach before starting
    #[arg(long)]
    wait_for_debugger: bool,
}

#[allow(unused_variables)]
async fn run_server(
    socket: Option<String>,
    pipe: Option<String>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let service = BicepExtensionService;

    #[cfg(unix)]
    if let Some(socket_path) = socket {
        use tokio::net::UnixListener;
        use tokio_stream::wrappers::UnixListenerStream;

        tracing::info!("Starting Bicep gRPC server on Unix socket: {}", socket_path);

        // Remove the socket file if it exists
        let _ = std::fs::remove_file(&socket_path);

        let uds = UnixListener::bind(&socket_path)?;
        let uds_stream = UnixListenerStream::new(uds);

        Server::builder()
            .add_service(BicepExtensionServer::new(service))
            .serve_with_incoming(uds_stream)
            .await?;

        return Ok(());
    }

    #[cfg(windows)]
    if let Some(pipe_name) = pipe {
        tracing::info!("Starting Bicep gRPC server on named pipe: {}", pipe_name);

        // TODO: Implement Windows named pipe transport
        // This requires additional dependencies and platform-specific code
        return Err("Windows named pipe support not yet implemented".into());
    }

    Err("Either --socket (Unix) or --pipe (Windows) must be specified".into())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let trace_level = std::env::var("DSC_TRACE_LEVEL")
        .ok()
        .and_then(|level| match level.to_uppercase().as_str() {
            "TRACE" => Some(tracing::Level::TRACE),
            "DEBUG" => Some(tracing::Level::DEBUG),
            "INFO" => Some(tracing::Level::INFO),
            "WARN" => Some(tracing::Level::WARN),
            "ERROR" => Some(tracing::Level::ERROR),
            _ => None,
        })
        .unwrap_or(tracing::Level::WARN);

    tracing_subscriber::fmt()
        .with_target(false)
        .with_level(true)
        .with_max_level(trace_level)
        .init();

    let args = Args::parse();

    if args.wait_for_debugger || std::env::var_os("DSC_GRPC_DEBUG").is_some() {
        tracing::warn!(
            "Press any key to continue after attaching to PID: {}",
            std::process::id()
        );
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
    }

    // Set up graceful shutdown on SIGTERM/SIGINT
    let shutdown_signal = async {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to listen for shutdown signal");
        tracing::info!("Received shutdown signal, terminating gracefully...");
    };

    tokio::select! {
        result = run_server(args.socket, args.pipe) => {
            if let Err(e) = result {
                tracing::error!("Server error: {}", e);
                return Err(e);
            }
        }
        _ = shutdown_signal => {
            tracing::info!("Shutdown complete");
        }
    }

    Ok(())
}
