// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use clap::Parser;
use dsc_lib::{
    configure::config_doc::ExecutionKind,
    discovery::discovery_trait::DiscoveryFilter,
    dscresources::{
        dscresource::Invoke,
        invoke_result::{GetResult, SetResult},
    },
    DscManager,
};
use rust_i18n::{i18n, t};
use std::{env, io, process};
use tonic::{transport::Server, Request, Response, Status};

// Include the generated protobuf code
pub mod proto {
    tonic::include_proto!("extension");
    pub(crate) const FILE_DESCRIPTOR_SET: &[u8] = tonic::include_file_descriptor_set!("bicep");
}

use proto::bicep_extension_server::{BicepExtension, BicepExtensionServer};
use proto::{
    Empty, LocalExtensibilityOperationResponse, ResourceReference, ResourceSpecification,
    TypeFilesResponse,
};

i18n!("locales", fallback = "en-us");

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

        tracing::debug!(
            "{}",
            t!(
                "bicep.functionCalled",
                function = "CreateOrUpdate",
                resourceType = resource_type,
                version = format!("{version:?}"),
                properties = properties
            )
        );

        let mut dsc = DscManager::new();
        let Some(resource) = dsc
            .find_resource(&DiscoveryFilter::new(
                &resource_type,
                version.as_deref(),
                None,
            ))
            .unwrap_or(None)
        else {
            return Err(Status::not_found(
                t!("dscerror.resourceNotFound").to_string(),
            ));
        };

        let SetResult::Resource(result) = resource
            .set(&properties, false, &ExecutionKind::Actual)
            .map_err(|e| Status::aborted(e.to_string()))?
        else {
            return Err(Status::unimplemented(
                t!("dscerror.notSupported").to_string(),
            ));
        };

        Ok(Response::new(LocalExtensibilityOperationResponse {
            resource: Some(proto::Resource {
                r#type: resource_type,
                api_version: version,
                identifiers: properties,
                properties: result.after_state.to_string(),
                status: None,
            }),
            error_data: None,
        }))
    }

    async fn preview(
        &self,
        request: Request<ResourceSpecification>,
    ) -> Result<Response<LocalExtensibilityOperationResponse>, Status> {
        let spec = request.into_inner();
        let resource_type = spec.r#type;
        let version = spec.api_version;
        let properties = spec.properties;

        tracing::debug!(
            "{}",
            t!(
                "bicep.functionCalled",
                function = "Preview",
                resourceType = resource_type,
                version = format!("{version:?}"),
                properties = properties
            )
        );

        let mut dsc = DscManager::new();
        let Some(resource) = dsc
            .find_resource(&DiscoveryFilter::new(
                &resource_type,
                version.as_deref(),
                None,
            ))
            .unwrap_or(None)
        else {
            return Err(Status::not_found(
                t!("dscerror.resourceNotFound").to_string(),
            ));
        };

        let SetResult::Resource(result) = resource
            .set(&properties, false, &ExecutionKind::WhatIf)
            .map_err(|e| Status::aborted(e.to_string()))?
        else {
            return Err(Status::unimplemented(
                t!("dscerror.notSupported").to_string(),
            ));
        };

        Ok(Response::new(LocalExtensibilityOperationResponse {
            resource: Some(proto::Resource {
                r#type: resource_type,
                api_version: version,
                identifiers: properties,
                properties: result.after_state.to_string(),
                status: None,
            }),
            error_data: None,
        }))
    }

    async fn get(
        &self,
        request: Request<ResourceReference>,
    ) -> Result<Response<LocalExtensibilityOperationResponse>, Status> {
        let reference = request.into_inner();
        let resource_type = reference.r#type.clone();
        let version = reference.api_version.clone();
        let identifiers = reference.identifiers.clone();

        tracing::debug!(
            "{}",
            t!(
                "bicep.functionCalled",
                function = "Get",
                resourceType = resource_type,
                version = format!("{version:?}"),
                properties = identifiers
            )
        );

        let mut dsc = DscManager::new();
        let Some(resource) = dsc
            .find_resource(&DiscoveryFilter::new(
                &resource_type,
                version.as_deref(),
                None,
            ))
            .unwrap_or(None)
        else {
            return Err(Status::not_found(
                t!("dscerror.resourceNotFound").to_string(),
            ));
        };

        let GetResult::Resource(result) = resource
            .get(&identifiers)
            .map_err(|e| Status::aborted(e.to_string()))?
        else {
            return Err(Status::unimplemented(
                t!("dscerror.notSupported").to_string(),
            ));
        };

        Ok(Response::new(LocalExtensibilityOperationResponse {
            resource: Some(proto::Resource {
                r#type: resource_type,
                api_version: version,
                identifiers: identifiers,
                properties: result.actual_state.to_string(),
                status: None,
            }),
            error_data: None,
        }))
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
            "{}",
            t!(
                "bicep.functionCalled",
                function = "Delete",
                resourceType = resource_type,
                version = format!("{version:?}"),
                properties = identifiers
            )
        );

        let mut dsc = DscManager::new();
        let Some(resource) = dsc
            .find_resource(&DiscoveryFilter::new(
                &resource_type,
                version.as_deref(),
                None,
            ))
            .unwrap_or(None)
        else {
            return Err(Status::not_found(
                t!("dscerror.resourceNotFound").to_string(),
            ));
        };

        resource
            .delete(&identifiers, &ExecutionKind::Actual)
            .map_err(|e| Status::aborted(e.to_string()))?;

        Ok(Response::new(LocalExtensibilityOperationResponse {
            resource: Some(proto::Resource {
                r#type: resource_type,
                api_version: version,
                identifiers: identifiers,
                properties: "{}".to_string(),
                status: None,
            }),
            error_data: None,
        }))
    }

    async fn get_type_files(
        &self,
        _request: Request<Empty>,
    ) -> Result<Response<TypeFilesResponse>, Status> {
        // TODO: Dynamically return type definitions for DSC resources found on the system.
        Err(Status::unimplemented(
            t!("dscerror.notImplemented").to_string(),
        ))
    }

    async fn ping(&self, _request: Request<Empty>) -> Result<Response<Empty>, Status> {
        Ok(Response::new(Empty {}))
    }
}

#[derive(Parser, Debug)]
#[command(name = "dsc-bicep-ext")]
#[command(about = "DSC Bicep Local Deploy Extension", long_about = None)]
struct Args {
    /// The path to the domain socket to connect on (Unix-like systems)
    #[arg(long)]
    socket: Option<String>,

    /// The named pipe to connect on (Windows)
    #[arg(long)]
    pipe: Option<String>,

    /// The HTTP address to listen on (e.g., 127.0.0.1:50051)
    #[arg(long)]
    http: Option<String>,

    /// Wait for debugger to attach before starting
    #[arg(long)]
    wait_for_debugger: bool,
}

#[allow(unused_variables)]
async fn run_server(
    socket: Option<String>,
    pipe: Option<String>,
    http: Option<String>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let service = BicepExtensionService;

    #[cfg(unix)]
    if let Some(socket_path) = socket {
        use tokio::net::UnixListener;
        use tokio_stream::wrappers::UnixListenerStream;

        tracing::info!(
            "{}",
            t!(
                "bicep.serverStarting",
                transport = "socket",
                address = socket_path
            )
        );

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
        // TODO: This named pipe code is messy and honestly mostly generated. It
        // does work, but most of the problem lies in minimal Windows support
        // inside the Tokio library (and no support for UDS).
        use std::pin::Pin;
        use std::task::{Context, Poll};
        use tokio::io::{AsyncRead, AsyncWrite};
        use tokio::net::windows::named_pipe::ServerOptions;
        use tonic::transport::server::Connected;

        // Wrapper to implement Connected trait for NamedPipeServer
        struct NamedPipeConnection(tokio::net::windows::named_pipe::NamedPipeServer);

        impl Connected for NamedPipeConnection {
            type ConnectInfo = ();

            fn connect_info(&self) -> Self::ConnectInfo {
                ()
            }
        }

        impl AsyncRead for NamedPipeConnection {
            fn poll_read(
                mut self: Pin<&mut Self>,
                cx: &mut Context<'_>,
                buf: &mut tokio::io::ReadBuf<'_>,
            ) -> Poll<std::io::Result<()>> {
                Pin::new(&mut self.0).poll_read(cx, buf)
            }
        }

        impl AsyncWrite for NamedPipeConnection {
            fn poll_write(
                mut self: Pin<&mut Self>,
                cx: &mut Context<'_>,
                buf: &[u8],
            ) -> Poll<std::io::Result<usize>> {
                Pin::new(&mut self.0).poll_write(cx, buf)
            }

            fn poll_flush(
                mut self: Pin<&mut Self>,
                cx: &mut Context<'_>,
            ) -> Poll<std::io::Result<()>> {
                Pin::new(&mut self.0).poll_flush(cx)
            }

            fn poll_shutdown(
                mut self: Pin<&mut Self>,
                cx: &mut Context<'_>,
            ) -> Poll<std::io::Result<()>> {
                Pin::new(&mut self.0).poll_shutdown(cx)
            }
        }

        // Windows named pipes must be in the format \\.\pipe\{name}
        let full_pipe_path = format!(r"\\.\pipe\{}", pipe_name);
        tracing::info!(
            "{}",
            t!(
                "bicep.serverStarting",
                transport = "named pipe",
                address = full_pipe_path
            )
        );

        // Create a stream that accepts connections on the named pipe
        let incoming = async_stream::stream! {
            // Track whether this is the first instance
            let mut is_first = true;

            loop {
                let pipe = if is_first {
                    ServerOptions::new()
                        .first_pipe_instance(true)
                        .create(&full_pipe_path)
                } else {
                    ServerOptions::new()
                        .create(&full_pipe_path)
                };

                let server = match pipe {
                    Ok(server) => server,
                    Err(e) => {
                        tracing::error!("{}", t!("bicep.serverError", error = e.to_string()));
                        break;
                    }
                };

                is_first = false;

                match server.connect().await {
                    Ok(()) => {
                        yield Ok::<_, std::io::Error>(NamedPipeConnection(server));
                    }
                    Err(e) => {
                        tracing::error!("{}", t!("bicep.serverError", error = e.to_string()));
                        break;
                    }
                }
            }
        };

        Server::builder()
            .add_service(BicepExtensionServer::new(service))
            .serve_with_incoming(incoming)
            .await?;

        return Ok(());
    }

    // Default to HTTP server on [::1]:50051 if no transport specified
    let addr = http.unwrap_or_else(|| "[::1]:50051".to_string());
    tracing::info!(
        "{}",
        t!(
            "bicep.serverStarting",
            transport = "HTTP",
            address = addr.to_string()
        )
    );

    let reflection_service = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(proto::FILE_DESCRIPTOR_SET)
        .build_v1()
        .unwrap();

    Server::builder()
        .add_service(reflection_service)
        .add_service(BicepExtensionServer::new(service))
        .serve(addr.parse()?)
        .await?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let trace_level = env::var("DSC_TRACE_LEVEL")
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

    if args.wait_for_debugger
        || env::var_os("DSC_GRPC_DEBUG").is_some_and(|v| v.eq_ignore_ascii_case("true"))
    {
        tracing::warn!("{}", t!("bicep.waitForDebugger", pid = process::id()));
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
    }

    // Set up graceful shutdown on SIGTERM/SIGINT
    let shutdown_signal = async {
        tokio::signal::ctrl_c().await.unwrap();
    };

    tokio::select! {
        result = run_server(args.socket, args.pipe, args.http) => {
            if let Err(e) = result {
                tracing::error!("{}", t!("bicep.serverError", error = e.to_string()));
                return Err(e);
            }
        }
        _ = shutdown_signal => {
        }
    }

    Ok(())
}
