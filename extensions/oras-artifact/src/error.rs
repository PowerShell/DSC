// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use thiserror::Error;

/// All errors that can occur during ORAS artifact discovery.
#[derive(Debug, Error)]
pub enum DiscoveryError {
    /// An HTTP transport or status error from the registry.
    #[error("Registry request failed for '{context}': {source}")]
    HttpRequest {
        context: String,
        #[source]
        source: reqwest::Error,
    },

    /// The registry returned an unexpected HTTP status code.
    #[error("Registry returned HTTP {status} for '{url}': {body}")]
    HttpStatus {
        status: u16,
        url: String,
        body: String,
    },

    /// Authentication with the registry failed.
    #[error("Authentication error for registry '{registry}': {message}")]
    Auth { registry: String, message: String },

    /// A manifest or index response could not be parsed as valid JSON.
    #[error("Failed to parse manifest JSON from '{url}': {source}")]
    ManifestParse {
        url: String,
        #[source]
        source: serde_json::Error,
    },

    /// A manifest is missing required OCI or DSC fields.
    #[error("Manifest validation failed for '{reference}': {message}")]
    ManifestValidation { reference: String, message: String },

    /// The requested artifact (tag) was not found in the repository.
    #[error("Artifact not found: '{reference}'")]
    NotFound { reference: String },

    /// No manifest variant in an OCI index matched the requested platform.
    #[error(
        "No platform variant found for '{reference}': requested os={os:?} arch={arch:?}"
    )]
    PlatformNotFound {
        reference: String,
        os: Option<String>,
        arch: Option<String>,
    },

    /// An invalid registry reference was provided.
    #[error("Invalid registry reference '{reference}': {message}")]
    InvalidReference { reference: String, message: String },
}
