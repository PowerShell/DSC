// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// DSC artifact kind constants
// ---------------------------------------------------------------------------

/// Artifact type media type for DSC schemas.
pub const ARTIFACT_TYPE_SCHEMA: &str = "application/vnd.microsoft.dsc.schema.v1+json";
/// Artifact type media type for DSC resources.
pub const ARTIFACT_TYPE_RESOURCE: &str = "application/vnd.microsoft.dsc.resource.v1+tar+gzip";
/// Artifact type media type for DSC configurations.
pub const ARTIFACT_TYPE_CONFIGURATION: &str =
    "application/vnd.microsoft.dsc.configuration.v1+yaml";
/// Artifact type media type for DSC extensions.
pub const ARTIFACT_TYPE_EXTENSION: &str = "application/vnd.microsoft.dsc.extension.v1+tar+gzip";
/// Artifact type media type for DSC adapters.
pub const ARTIFACT_TYPE_ADAPTER: &str = "application/vnd.microsoft.dsc.adapter.v1+tar+gzip";
/// Artifact type media type for DSC bundles (OCI index grouping related artifacts).
pub const ARTIFACT_TYPE_BUNDLE: &str = "application/vnd.microsoft.dsc.bundle.v1+json";

/// DSC annotation keys
pub const ANNOTATION_KIND: &str = "com.microsoft.dsc.kind";
pub const ANNOTATION_NAME: &str = "com.microsoft.dsc.name";
pub const ANNOTATION_VERSION: &str = "com.microsoft.dsc.version";
pub const ANNOTATION_PUBLISHER: &str = "com.microsoft.dsc.publisher";
pub const ANNOTATION_RESOURCE_ID: &str = "com.microsoft.dsc.resource.id";
pub const ANNOTATION_RESOURCE_VERSION: &str = "com.microsoft.dsc.resource.version";

/// OCI media type constants
pub const MEDIA_TYPE_OCI_MANIFEST: &str = "application/vnd.oci.image.manifest.v1+json";
pub const MEDIA_TYPE_OCI_INDEX: &str = "application/vnd.oci.image.index.v1+json";

// ---------------------------------------------------------------------------
// Discovery input types
// ---------------------------------------------------------------------------

/// The kind of DSC artifact to discover.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ArtifactKind {
    Schema,
    Configuration,
    Resource,
    Extension,
    Adapter,
    Bundle,
}

impl ArtifactKind {
    /// Returns the `com.microsoft.dsc.kind` annotation value for this kind.
    #[must_use]
    pub fn annotation_value(&self) -> &'static str {
        match self {
            ArtifactKind::Schema => "schema",
            ArtifactKind::Configuration => "configuration",
            ArtifactKind::Resource => "resource",
            ArtifactKind::Extension => "extension",
            ArtifactKind::Adapter => "adapter",
            ArtifactKind::Bundle => "bundle",
        }
    }

    /// Returns the artifact type media type for this kind.
    #[must_use]
    pub fn artifact_type(&self) -> &'static str {
        match self {
            ArtifactKind::Schema => ARTIFACT_TYPE_SCHEMA,
            ArtifactKind::Configuration => ARTIFACT_TYPE_CONFIGURATION,
            ArtifactKind::Resource => ARTIFACT_TYPE_RESOURCE,
            ArtifactKind::Extension => ARTIFACT_TYPE_EXTENSION,
            ArtifactKind::Adapter => ARTIFACT_TYPE_ADAPTER,
            ArtifactKind::Bundle => ARTIFACT_TYPE_BUNDLE,
        }
    }
}

/// Platform filter for selecting platform-specific manifest variants from an OCI index.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct PlatformFilter {
    /// Target operating system (e.g. `"windows"`, `"linux"`, `"darwin"`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub os: Option<String>,

    /// Target CPU architecture (e.g. `"amd64"`, `"arm64"`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub architecture: Option<String>,
}

/// A registry repository reference to query during discovery.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RegistryReference {
    /// Registry reference: either a full repository path or just the registry host.
    ///
    /// Examples:
    /// - `ghcr.io/contoso/dsc-resources`  (specific repository, all tags)
    /// - `ghcr.io/contoso/dsc-resources:v2.0.0`  (specific repository with tag)
    /// - `orasdscartifacts.azurecr.io`  (entire registry, discovers all repositories)
    /// - `registry.io:5000`  (registry with custom port, discovers all repositories)
    pub reference: String,
}

/// Top-level JSON input consumed from STDIN by the `discover` subcommand.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct DiscoveryInput {
    /// Registry repositories to query.
    #[serde(default)]
    pub registries: Vec<RegistryReference>,

    /// Restrict results to a specific artifact kind.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kind: Option<ArtifactKind>,

    /// Restrict results to a specific version string (exact match).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,

    /// Restrict results to a specific platform variant (used when an OCI index is encountered).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub platform: Option<PlatformFilter>,
}

// ---------------------------------------------------------------------------
// OCI distribution spec types
// ---------------------------------------------------------------------------

/// Descriptor for an OCI object (config, layer, or manifest entry).
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OciDescriptor {
    pub media_type: String,
    pub digest: String,
    pub size: i64,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub annotations: Option<HashMap<String, String>>,
}

/// OCI image manifest (single-platform artifact).
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OciManifest {
    pub schema_version: u32,

    #[serde(default)]
    pub media_type: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub artifact_type: Option<String>,

    pub config: OciDescriptor,

    pub layers: Vec<OciDescriptor>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub annotations: Option<HashMap<String, String>>,
}

/// Platform descriptor for an entry inside an OCI index.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OciPlatform {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub os: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub architecture: Option<String>,
}

/// Descriptor for an entry inside an OCI image index.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OciIndexEntry {
    pub media_type: String,
    pub digest: String,
    pub size: i64,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub platform: Option<OciPlatform>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub annotations: Option<HashMap<String, String>>,
}

/// OCI image index (multi-platform or bundle).
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OciIndex {
    pub schema_version: u32,

    #[serde(default)]
    pub media_type: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub artifact_type: Option<String>,

    pub manifests: Vec<OciIndexEntry>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub annotations: Option<HashMap<String, String>>,
}

/// Represents either a single OCI manifest or an OCI index returned by the registry.
#[derive(Debug, Clone)]
pub enum OciManifestOrIndex {
    Manifest(OciManifest),
    Index(OciIndex),
}

// ---------------------------------------------------------------------------
// Discovery output type
// ---------------------------------------------------------------------------

/// A discovered DSC artifact returned as a JSON line on STDOUT.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiscoveredArtifact {
    /// Full registry repository reference (e.g. `ghcr.io/contoso/dsc-resources`).
    pub repository: String,

    /// Tag that identifies this specific version.
    pub tag: String,

    /// Content-addressable digest of the manifest.
    pub digest: String,

    /// OCI `artifactType` media type.
    pub artifact_type: String,

    /// Value of the `com.microsoft.dsc.kind` annotation, when present.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,

    /// Artifact name derived from DSC annotations.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// Artifact version derived from DSC annotations.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,

    /// Platform metadata, when the artifact was selected from an OCI index.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub platform: Option<OciPlatform>,

    /// Layer descriptors containing the distributable payload.
    pub layers: Vec<OciDescriptor>,

    /// All manifest-level annotations.
    pub annotations: HashMap<String, String>,
}

// ---------------------------------------------------------------------------
// OCI auth helper types (internal)
// ---------------------------------------------------------------------------

/// Response from an OCI registry token endpoint.
/// Different registries use either `token` or `access_token` as the key.
#[derive(Debug, Deserialize)]
pub struct TokenResponse {
    pub token: Option<String>,
    pub access_token: Option<String>,
}

impl TokenResponse {
    /// Returns the bearer token string regardless of which field the registry used.
    #[must_use]
    pub fn bearer_token(&self) -> Option<&str> {
        self.token
            .as_deref()
            .or(self.access_token.as_deref())
    }
}

/// Parsed fields from a `WWW-Authenticate: Bearer` header.
#[derive(Debug, Default)]
pub struct BearerChallenge {
    pub realm: String,
    pub service: Option<String>,
    pub scope: Option<String>,
}
