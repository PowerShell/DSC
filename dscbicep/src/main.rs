// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use clap::Parser;
use tonic::{transport::Server, Request, Response, Status};
use dsc_lib::{
    configure::config_doc::ExecutionKind,
    dscresources::dscresource::Invoke,
    DscManager,
};

// Include the generated protobuf code
pub mod proto {
    tonic::include_proto!("extension");
}

use proto::bicep_extension_server::{BicepExtension, BicepExtensionServer};
use proto::{
    Empty, ResourceSpecification, ResourceReference, LocalExtensibilityOperationResponse,
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
        tracing::debug!(
            "CreateOrUpdate called for type: {}, apiVersion: {:?}",
            spec.r#type,
            spec.api_version
        );

        let mut dsc = DscManager::new();
        let Some(resource) = dsc.find_resource(&spec.r#type, None) else {
            return Err(Status::invalid_argument("Resource not found"));
        };

        let _result = match resource.set(&spec.properties, false, &ExecutionKind::Actual) {
            Ok(res) => res,
            Err(e) => return Err(Status::internal(format!("DSC set operation failed: {}", e))),
        };

        let response = LocalExtensibilityOperationResponse {
            resource: Some(proto::Resource {
                r#type: spec.r#type,
                api_version: spec.api_version,
                identifiers: String::new(),
                properties: spec.properties,
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
        tracing::debug!(
            "Preview called for type: {}, apiVersion: {:?}",
            spec.r#type,
            spec.api_version
        );

        // TODO: Implement preview/what-if logic
        Err(Status::unimplemented("Preview not yet implemented"))
    }

    async fn get(
        &self,
        request: Request<ResourceReference>,
    ) -> Result<Response<LocalExtensibilityOperationResponse>, Status> {
        let reference = request.into_inner();
        tracing::debug!(
            "Get called for type: {}, identifiers: {}",
            reference.r#type,
            reference.identifiers
        );

        // TODO: Implement resource retrieval logic
        Err(Status::unimplemented("Get not yet implemented"))
    }

    async fn delete(
        &self,
        request: Request<ResourceReference>,
    ) -> Result<Response<LocalExtensibilityOperationResponse>, Status> {
        let reference = request.into_inner();
        tracing::debug!(
            "Delete called for type: {}, identifiers: {}",
            reference.r#type,
            reference.identifiers
        );

        // TODO: Implement resource deletion logic
        Err(Status::unimplemented("Delete not yet implemented"))
    }

    async fn get_type_files(
        &self,
        _request: Request<Empty>,
    ) -> Result<Response<TypeFilesResponse>, Status> {
        tracing::debug!("GetTypeFiles called");

        // TODO: Return actual Bicep type definitions
        Err(Status::unimplemented("GetTypeFiles not yet implemented"))
    }

    async fn ping(
        &self,
        _request: Request<Empty>,
    ) -> Result<Response<Empty>, Status> {
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
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_target(false)
        .with_level(true)
        // TODO: Plumb tracing env var support.
        .with_max_level(tracing::Level::TRACE)
        .init();

    let args = Args::parse();

    // TODO: Find out if there is any actual way to get bicep local-deploy to send the --wait-for-debugger command.
    if true {
        tracing::info!("Waiting for debugger to attach...");
        tracing::info!("Press any key to continue after attaching to PID: {}", std::process::id());
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
