// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use tonic::{transport::Server, Request, Response, Status};

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

        // TODO: Implement actual resource creation/update logic
        Err(Status::unimplemented("CreateOrUpdate not yet implemented"))
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

async fn start_bicep_server_async(addr: impl Into<std::net::SocketAddr>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr = addr.into();

    tracing::info!("Starting Bicep gRPC server on {addr}");

    let route_guide = BicepExtensionService;
    let svc = BicepExtensionServer::new(route_guide);

    Server::builder().add_service(svc).serve(addr).await?;

    Ok(())
}

/// Synchronous wrapper to start the Bicep gRPC server
///
/// # Errors
///
/// This function will return an error if the Bicep server fails to start or if the tokio runtime cannot be created.
pub fn start_bicep_server() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let rt = tokio::runtime::Runtime::new()?;

    // Default to localhost:50051 (standard gRPC port)
    let addr: std::net::SocketAddr = "127.0.0.1:50051".parse()?;

    rt.block_on(start_bicep_server_async(addr))?;
    Ok(())
}
