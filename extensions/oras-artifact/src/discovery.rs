// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use std::collections::HashMap;

use reqwest::{
    blocking::{Client, Response},
    header::{ACCEPT, AUTHORIZATION, WWW_AUTHENTICATE},
    StatusCode, Url,
};
use tracing::{debug, trace, warn};

use crate::{
    error::DiscoveryError,
    models::{
        ArtifactKind, BearerChallenge, DiscoveredArtifact, DiscoveryInput, OciDescriptor,
        OciIndex, OciIndexEntry, OciManifest, OciManifestOrIndex, OciPlatform, PlatformFilter,
        TokenResponse, ANNOTATION_KIND, ANNOTATION_NAME, ANNOTATION_PUBLISHER,
        ANNOTATION_RESOURCE_ID, ANNOTATION_RESOURCE_VERSION, ANNOTATION_VERSION,
        MEDIA_TYPE_OCI_INDEX, MEDIA_TYPE_OCI_MANIFEST,
    },
};

// Accept header value that asks the registry for either an OCI index or a single manifest.
const ACCEPT_OCI: &str = "application/vnd.oci.image.index.v1+json, application/vnd.oci.image.manifest.v1+json";

// Environment variable that callers may pre-populate with a Bearer token.
const ENV_REGISTRY_TOKEN: &str = "ORAS_REGISTRY_TOKEN";

/// Parses a `WWW-Authenticate: Bearer` header value into its named parameters.
fn parse_bearer_challenge(header: &str) -> BearerChallenge {
    let stripped = header.trim_start_matches("Bearer").trim();
    let mut challenge = BearerChallenge::default();

    for part in stripped.split(',') {
        let part = part.trim();
        if let Some((key, value)) = part.split_once('=') {
            let value = value.trim_matches('"');
            match key.trim() {
                "realm" => challenge.realm = value.to_string(),
                "service" => challenge.service = Some(value.to_string()),
                "scope" => challenge.scope = Some(value.to_string()),
                _ => {}
            }
        }
    }

    challenge
}

/// Parsed components of a registry reference string.
#[derive(Debug)]
struct ParsedReference {
    /// Hostname (and optional port) of the registry, e.g. `ghcr.io`.
    host: String,
    /// Repository path inside the registry, e.g. `contoso/dsc-resources`.
    /// `None` when the reference is registry-only (triggers catalog discovery).
    repository: Option<String>,
    /// Tag component when provided; `None` means "all tags".
    tag: Option<String>,
}

impl ParsedReference {
    fn parse(reference: &str) -> Result<Self, DiscoveryError> {
        // Split off an optional tag (everything after the last ':' that is not part of the host).
        let (without_tag, tag) = if let Some(at_colon) = reference.rfind(':') {
            // Make sure this ':' is not part of the hostname (e.g. `registry.io:5000/repo`).
            // A colon in the host portion appears before the first '/'.
            let first_slash = reference.find('/').unwrap_or(reference.len());
            if at_colon > first_slash {
                (&reference[..at_colon], Some(reference[at_colon + 1..].to_string()))
            } else {
                (reference, None)
            }
        } else {
            (reference, None)
        };

        if without_tag.is_empty() {
            return Err(DiscoveryError::InvalidReference {
                reference: reference.to_string(),
                message: "registry host must not be empty".to_string(),
            });
        }

        // If there is no '/', the reference is registry-only (catalog discovery mode).
        let (host, repository) = if let Some(slash_pos) = without_tag.find('/') {
            let host = without_tag[..slash_pos].to_string();
            let repo = without_tag[slash_pos + 1..].to_string();
            if host.is_empty() || repo.is_empty() {
                return Err(DiscoveryError::InvalidReference {
                    reference: reference.to_string(),
                    message: "host and repository must not be empty".to_string(),
                });
            }
            (host, Some(repo))
        } else {
            (without_tag.to_string(), None)
        };

        Ok(ParsedReference { host, repository, tag })
    }

    /// Returns the base URL for the OCI distribution API for a specific repository.
    fn api_base_for(&self, repo: &str) -> String {
        format!("https://{}/v2/{}", self.host, repo)
    }

    /// Returns the base URL for the OCI distribution API using the stored repository.
    /// Panics if called on a registry-only reference.
    fn api_base(&self) -> String {
        self.api_base_for(
            self.repository
                .as_deref()
                .expect("api_base() called on a registry-only ParsedReference"),
        )
    }

    /// Returns the base URL for registry-level API calls (e.g. `_catalog`).
    fn registry_base(&self) -> String {
        format!("https://{}", self.host)
    }
}

/// HTTP client that communicates with an OCI-compliant container registry.
pub struct RegistryClient {
    client: Client,
    parsed: ParsedReference,
    /// Bearer token acquired during authentication.
    token: Option<String>,
}

impl RegistryClient {
    /// Creates a new client for the given registry reference.
    ///
    /// # Errors
    /// Returns [`DiscoveryError::InvalidReference`] if `reference` cannot be parsed.
    pub fn new(reference: &str) -> Result<Self, DiscoveryError> {
        let parsed = ParsedReference::parse(reference)?;
        let client = Client::builder()
            .user_agent("oras-artifact-discover/0.1.0")
            .build()
            .unwrap_or_default();

        // Honour a pre-supplied token from the environment.
        let token = std::env::var(ENV_REGISTRY_TOKEN).ok();

        Ok(RegistryClient { client, parsed, token })
    }

    // -----------------------------------------------------------------------
    // Private helpers
    // -----------------------------------------------------------------------

    /// Performs a GET request to `url` with the given `accept` header, handling
    /// `401` responses by fetching a Bearer token and retrying once.
    /// Set `catalog_scope` to true when accessing registry-level endpoints like `/_catalog`.
    fn get_authenticated_internal(
        &mut self,
        url: &str,
        accept: &str,
        catalog_scope: bool,
    ) -> Result<Response, DiscoveryError> {
        let response = self
            .client
            .get(url)
            .header(ACCEPT, accept)
            .bearer_auth_opt(self.token.as_deref())
            .send()
            .map_err(|e| DiscoveryError::HttpRequest {
                context: url.to_string(),
                source: e,
            })?;

        if response.status() == StatusCode::UNAUTHORIZED {
            // Attempt to authenticate using the WWW-Authenticate challenge.
            // Always re-authenticate even if we have an existing token — the current
            // token may have been issued with a different scope (e.g. catalog vs repository).
            if let Some(www_auth) = response.headers().get(WWW_AUTHENTICATE) {
                let challenge_str = www_auth.to_str().unwrap_or("").to_string();
                debug!("Received WWW-Authenticate: {challenge_str}");
                self.token = None; // clear stale / wrong-scope token before re-authenticating
                self.authenticate(&challenge_str, catalog_scope)?;

                // Retry with the newly acquired token.
                return self
                    .client
                    .get(url)
                    .header(ACCEPT, accept)
                    .bearer_auth_opt(self.token.as_deref())
                    .send()
                    .map_err(|e| DiscoveryError::HttpRequest {
                        context: url.to_string(),
                        source: e,
                    });
            } else {
                // No WWW-Authenticate challenge; return the 401 for the caller to handle.
                debug!("Received 401 without WWW-Authenticate header; repository may require credentials");
                return Ok(response);
            }
        }

        Ok(response)
    }

    /// Convenience wrapper for non-catalog endpoints.
    fn get_authenticated(
        &mut self,
        url: &str,
        accept: &str,
    ) -> Result<Response, DiscoveryError> {
        self.get_authenticated_internal(url, accept, false)
    }

    /// Convenience wrapper for catalog endpoints that need `registry:catalog:*` scope.
    fn get_authenticated_catalog(
        &mut self,
        url: &str,
        accept: &str,
    ) -> Result<Response, DiscoveryError> {
        self.get_authenticated_internal(url, accept, true)
    }

    /// Fetches a Bearer token using the challenge parameters from `www_authenticate`.
    /// If `catalog_scope` is true, appends the catalog scope to the token request.
    fn authenticate(&mut self, www_authenticate: &str, catalog_scope: bool) -> Result<(), DiscoveryError> {
        let challenge = parse_bearer_challenge(www_authenticate);

        if challenge.realm.is_empty() {
            return Err(DiscoveryError::Auth {
                registry: self.parsed.host.clone(),
                message: "WWW-Authenticate header missing realm".to_string(),
            });
        }

        debug!("Fetching token from realm: {}", challenge.realm);

        let mut token_url = Url::parse(&challenge.realm).map_err(|e| DiscoveryError::Auth {
            registry: self.parsed.host.clone(),
            message: format!("Invalid token endpoint URL '{}': {e}", challenge.realm),
        })?;

        {
            let mut pairs = token_url.query_pairs_mut();
            if let Some(service) = &challenge.service {
                pairs.append_pair("service", service);
            }
            // Use explicit scopes matching PSResourceGet's behaviour rather than
            // echoing the challenge scope (which may duplicate what we append below).
            if catalog_scope {
                // registry-level catalog access
                pairs.append_pair("scope", "registry:catalog:*");
            } else {
                // repository-level operations: wildcard repo scope + catalog scope
                pairs.append_pair("scope", "repository:*:*");
                pairs.append_pair("scope", "registry:catalog:*");
            }
        }

        let response = self.client.get(token_url).send().map_err(|e| DiscoveryError::Auth {
            registry: self.parsed.host.clone(),
            message: format!("Failed to reach token endpoint: {e}"),
        })?;

        if !response.status().is_success() {
            return Err(DiscoveryError::Auth {
                registry: self.parsed.host.clone(),
                message: format!(
                    "Token endpoint returned HTTP {}",
                    response.status().as_u16()
                ),
            });
        }

        let body = response.text().map_err(|e| DiscoveryError::Auth {
            registry: self.parsed.host.clone(),
            message: format!("Failed to read token response: {e}"),
        })?;

        let token_response: TokenResponse = serde_json::from_str(&body).map_err(|e| DiscoveryError::Auth {
            registry: self.parsed.host.clone(),
            message: format!("Failed to parse token response: {e}"),
        })?;

        let token = token_response.bearer_token().ok_or_else(|| DiscoveryError::Auth {
            registry: self.parsed.host.clone(),
            message: "Token response contained no token".to_string(),
        })?;

        self.token = Some(token.to_string());
        debug!("Successfully acquired bearer token");
        Ok(())
    }

    /// Returns the full list of tags available in the given repository.
    fn list_tags(&mut self, repo: &str) -> Result<Vec<String>, DiscoveryError> {
        let url = format!("{}/tags/list", self.parsed.api_base_for(repo));
        debug!("Fetching tags from: {url}");

        let response = self.get_authenticated(&url, "application/json")?;

        if response.status() == StatusCode::NOT_FOUND {
            return Err(DiscoveryError::NotFound {
                reference: format!("{}/{}", self.parsed.host, repo),
            });
        }

        // Handle 401 responses without authentication challenge.
        if response.status() == StatusCode::UNAUTHORIZED {
            warn!("Received 401 Unauthorized from {url}; returning empty tags");
            return Ok(Vec::new());
        }

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let body = response.text().unwrap_or_default();
            return Err(DiscoveryError::HttpStatus {
                status,
                url,
                body,
            });
        }

        #[derive(serde::Deserialize)]
        struct TagsResponse {
            tags: Option<Vec<String>>,
        }

        let body = response.text().map_err(|e| DiscoveryError::HttpRequest {
            context: url.clone(),
            source: e,
        })?;

        let tags_resp: TagsResponse = serde_json::from_str(&body).map_err(|e| DiscoveryError::ManifestParse {
            url,
            source: e,
        })?;

        Ok(tags_resp.tags.unwrap_or_default())
    }

    /// Returns the full list of repositories in the registry (registry-level catalog).
    fn list_repositories(&mut self) -> Result<Vec<String>, DiscoveryError> {
        // ACR (and some other registries) return 404 — not 401 — when /_catalog is
        // requested without a token, effectively hiding the endpoint.  PSResourceGet
        // works around this by pre-authenticating via the /v2/ ping endpoint, which
        // always issues a Bearer challenge, before calling /_catalog.
        // We do the same: if we have no token yet, hit /v2/ first to acquire one.
        if self.token.is_none() {
            self.ensure_catalog_token()?;
        }

        let url = format!("{}/_catalog", self.parsed.registry_base());
        debug!("Fetching catalog from: {url}");

        let response = self.get_authenticated_catalog(&url, "application/json")?;

        // If we still get 404 the registry genuinely doesn't implement /_catalog.
        if response.status() == StatusCode::NOT_FOUND {
            warn!(
                "Registry '{}' does not support the _catalog endpoint; \
                 specify a repository path (e.g. '{}/my-repo') to discover artifacts",
                self.parsed.host, self.parsed.host
            );
            return Ok(Vec::new());
        }

        if response.status() == StatusCode::UNAUTHORIZED {
            warn!("Received 401 Unauthorized from {url}; returning empty catalog");
            return Ok(Vec::new());
        }

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let body = response.text().unwrap_or_default();
            return Err(DiscoveryError::HttpStatus { status, url, body });
        }

        #[derive(serde::Deserialize)]
        struct CatalogResponse {
            repositories: Option<Vec<String>>,
        }

        let body = response.text().map_err(|e| DiscoveryError::HttpRequest {
            context: url.clone(),
            source: e,
        })?;

        let catalog_resp: CatalogResponse = serde_json::from_str(&body).map_err(|e| DiscoveryError::ManifestParse {
            url,
            source: e,
        })?;

        let repos = catalog_resp.repositories.unwrap_or_default();
        if repos.is_empty() {
            warn!("No repositories found in registry {}", self.parsed.host);
        }
        Ok(repos)
    }

    /// Pre-authenticates for catalog access by issuing a HEAD request to the
    /// `/v2/` ping endpoint, which always returns a Bearer challenge.
    /// This mirrors PSResourceGet's `IsContainerRegistryUnauthenticated()` pattern:
    /// ACR returns 404 (not 401) for unauthenticated `/_catalog` requests, so we
    /// must obtain a token before attempting the catalog call.
    fn ensure_catalog_token(&mut self) -> Result<(), DiscoveryError> {
        let ping_url = format!("{}/v2/", self.parsed.registry_base());
        debug!("Pre-authenticating via registry ping: {ping_url}");

        let response = self
            .client
            .head(&ping_url)
            .send()
            .map_err(|e| DiscoveryError::HttpRequest {
                context: ping_url.clone(),
                source: e,
            })?;

        if response.status() == StatusCode::UNAUTHORIZED {
            if let Some(www_auth) = response.headers().get(WWW_AUTHENTICATE) {
                let challenge_str = www_auth.to_str().unwrap_or("").to_string();
                debug!("Ping challenge: {challenge_str}");
                self.authenticate(&challenge_str, true)?;
                debug!("Catalog token acquired via ping endpoint");
            }
        }
        // If /v2/ returns 200 (public registry) or anything else, proceed without token.
        Ok(())
    }

    /// Fetches the manifest (or index) for `reference` (tag or digest).
    fn fetch_manifest(&mut self, reference: &str) -> Result<(String, OciManifestOrIndex), DiscoveryError> {
        let url = format!("{}/manifests/{}", self.parsed.api_base(), reference);
        debug!("Fetching manifest: {url}");

        let response = self.get_authenticated(&url, ACCEPT_OCI)?;

        if response.status() == StatusCode::NOT_FOUND {
            return Err(DiscoveryError::NotFound {
                reference: format!(
                    "{}/{}:{}",
                    self.parsed.host, self.parsed.repository.as_deref().unwrap_or(""), reference
                ),
            });
        }

        // Handle 401 responses - return as error so caller can skip this reference
        if response.status() == StatusCode::UNAUTHORIZED {
            let body = response.text().unwrap_or_default();
            warn!("Received 401 Unauthorized from {url} for reference {reference}");
            return Err(DiscoveryError::HttpStatus {
                status: StatusCode::UNAUTHORIZED.as_u16(),
                url,
                body,
            });
        }

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let body = response.text().unwrap_or_default();
            return Err(DiscoveryError::HttpStatus { status, url, body });
        }

        // Capture the content digest from the response header if present.
        let digest = response
            .headers()
            .get("Docker-Content-Digest")
            .and_then(|v| v.to_str().ok())
            .unwrap_or(reference)
            .to_string();

        let content_type = response
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("")
            .to_string();

        let body = response.text().map_err(|e| DiscoveryError::HttpRequest {
            context: url.clone(),
            source: e,
        })?;

        trace!("Manifest response content-type: {content_type}");

        let manifest_or_index = if content_type.contains(MEDIA_TYPE_OCI_INDEX)
            || body.contains(r#""manifests""#)
        {
            let index: OciIndex =
                serde_json::from_str(&body).map_err(|e| DiscoveryError::ManifestParse {
                    url: url.clone(),
                    source: e,
                })?;
            OciManifestOrIndex::Index(index)
        } else {
            let manifest: OciManifest =
                serde_json::from_str(&body).map_err(|e| DiscoveryError::ManifestParse {
                    url,
                    source: e,
                })?;
            OciManifestOrIndex::Manifest(manifest)
        };

        Ok((digest, manifest_or_index))
    }

    // -----------------------------------------------------------------------
    // Validation helpers
    // -----------------------------------------------------------------------

    fn validate_manifest(
        manifest: &OciManifest,
        reference: &str,
    ) -> Result<(), DiscoveryError> {
        if manifest.schema_version == 0 {
            return Err(DiscoveryError::ManifestValidation {
                reference: reference.to_string(),
                message: "schemaVersion is missing or zero".to_string(),
            });
        }

        if manifest.config.media_type.is_empty() {
            return Err(DiscoveryError::ManifestValidation {
                reference: reference.to_string(),
                message: "config.mediaType is required".to_string(),
            });
        }

        if manifest.layers.is_empty() {
            return Err(DiscoveryError::ManifestValidation {
                reference: reference.to_string(),
                message: "manifest must contain at least one layer".to_string(),
            });
        }

        if manifest
            .media_type
            .as_str()
            .ne(MEDIA_TYPE_OCI_MANIFEST)
            && !manifest.media_type.is_empty()
        {
            return Err(DiscoveryError::ManifestValidation {
                reference: reference.to_string(),
                message: format!(
                    "manifest mediaType must be '{MEDIA_TYPE_OCI_MANIFEST}' but was '{}'",
                    manifest.media_type
                ),
            });
        }

        Ok(())
    }

    fn validate_dsc_annotations(
        manifest: &OciManifest,
        requested_kind: Option<&ArtifactKind>,
        reference: &str,
    ) -> Result<(), DiscoveryError> {
        let annotations = manifest.annotations.as_ref().ok_or_else(|| {
            DiscoveryError::ManifestValidation {
                reference: reference.to_string(),
                message: "manifest annotations are required for DSC artifacts".to_string(),
            }
        })?;

        let kind_value = annotations
            .get(ANNOTATION_KIND)
            .map(String::as_str)
            .or_else(|| requested_kind.map(ArtifactKind::annotation_value));

        let Some(kind_value) = kind_value else {
            // If no kind was provided and no kind annotation exists, defer to filters.
            return Ok(());
        };

        let Some(artifact_type) = manifest.artifact_type.as_deref() else {
            return Err(DiscoveryError::ManifestValidation {
                reference: reference.to_string(),
                message: "artifactType is required for DSC artifacts".to_string(),
            });
        };

        let expected_kind = match kind_value {
            "schema" => ArtifactKind::Schema,
            "configuration" => ArtifactKind::Configuration,
            "resource" => ArtifactKind::Resource,
            "extension" => ArtifactKind::Extension,
            "adapter" => ArtifactKind::Adapter,
            "bundle" => ArtifactKind::Bundle,
            other => {
                return Err(DiscoveryError::ManifestValidation {
                    reference: reference.to_string(),
                    message: format!("unsupported DSC kind annotation '{other}'"),
                });
            }
        };

        let expected_type = expected_kind.artifact_type();
        if artifact_type != expected_type {
            return Err(DiscoveryError::ManifestValidation {
                reference: reference.to_string(),
                message: format!(
                    "artifactType '{artifact_type}' does not align with kind '{kind_value}' (expected '{expected_type}')"
                ),
            });
        }

        match expected_kind {
            ArtifactKind::Schema
            | ArtifactKind::Configuration
            | ArtifactKind::Extension
            | ArtifactKind::Adapter => {
                if !annotations.contains_key(ANNOTATION_NAME) {
                    return Err(DiscoveryError::ManifestValidation {
                        reference: reference.to_string(),
                        message: format!("missing required annotation '{ANNOTATION_NAME}'"),
                    });
                }
                if !annotations.contains_key(ANNOTATION_VERSION) {
                    return Err(DiscoveryError::ManifestValidation {
                        reference: reference.to_string(),
                        message: format!("missing required annotation '{ANNOTATION_VERSION}'"),
                    });
                }
            }
            ArtifactKind::Resource => {
                if !annotations.contains_key(ANNOTATION_RESOURCE_ID) {
                    return Err(DiscoveryError::ManifestValidation {
                        reference: reference.to_string(),
                        message: format!("missing required annotation '{ANNOTATION_RESOURCE_ID}'"),
                    });
                }
                if !annotations.contains_key(ANNOTATION_VERSION)
                    && !annotations.contains_key(ANNOTATION_RESOURCE_VERSION)
                {
                    return Err(DiscoveryError::ManifestValidation {
                        reference: reference.to_string(),
                        message: format!(
                            "missing required annotation '{ANNOTATION_VERSION}' or '{ANNOTATION_RESOURCE_VERSION}'"
                        ),
                    });
                }
            }
            ArtifactKind::Bundle => {}
        }

        Ok(())
    }

    // -----------------------------------------------------------------------
    // Filtering helpers
    // -----------------------------------------------------------------------

    /// Returns `true` when `manifest` matches the requested `kind` and `version` filters.
    fn manifest_matches_filters(
        manifest: &OciManifest,
        kind: Option<&ArtifactKind>,
        version: Option<&str>,
    ) -> bool {
        let annotations = manifest.annotations.as_ref();

        // Artifact type filter via annotation
        if let Some(k) = kind {
            let expected_kind = k.annotation_value();
            let artifact_kind = annotations
                .and_then(|a| a.get(ANNOTATION_KIND))
                .map(String::as_str)
                .unwrap_or("");

            // Also accept matching via artifactType media type.
            let artifact_type_matches = manifest
                .artifact_type
                .as_deref()
                .map_or(false, |at| at == k.artifact_type());

            if artifact_kind != expected_kind && !artifact_type_matches {
                return false;
            }
        }

        // Version filter
        if let Some(v) = version {
            let manifest_version = annotations
                .and_then(|a| a.get(ANNOTATION_VERSION))
                .or_else(|| annotations.and_then(|a| a.get(ANNOTATION_RESOURCE_VERSION)))
                .map(String::as_str)
                .unwrap_or("");
            if manifest_version != v {
                return false;
            }
        }

        true
    }

    /// Selects matching manifests from an OCI index, applying platform and kind/version filters.
    ///
    /// Fetches the actual manifest for each qualifying index entry.
    fn process_index(
        &mut self,
        index: &OciIndex,
        tag: &str,
        platform: Option<&PlatformFilter>,
        kind: Option<&ArtifactKind>,
        version: Option<&str>,
    ) -> Result<Vec<DiscoveredArtifact>, DiscoveryError> {
        let mut results = Vec::new();
        let repo_ref = format!("{}/{}", self.parsed.host, self.parsed.repository.as_deref().unwrap_or(""));
        let mut platform_candidate_found = platform.is_none();

        for entry in &index.manifests {
            // Platform filter
            if let Some(pf) = platform {
                if !platform_matches(entry, pf) {
                    continue;
                }
                platform_candidate_found = true;
            }

            // Fetch the individual manifest for this entry.
            let (digest, moi) = match self.fetch_manifest(&entry.digest) {
                Ok(m) => m,
                Err(e) => {
                    warn!("Skipping index entry {}: {e}", entry.digest);
                    continue;
                }
            };

            if let OciManifestOrIndex::Manifest(manifest) = moi {
                if let Err(e) = Self::validate_manifest(&manifest, &entry.digest) {
                    warn!("Validation failed for {}: {e}", entry.digest);
                    continue;
                }

                if let Err(e) = Self::validate_dsc_annotations(&manifest, kind, &entry.digest) {
                    warn!("DSC validation failed for {}: {e}", entry.digest);
                    continue;
                }

                if !Self::manifest_matches_filters(&manifest, kind, version) {
                    continue;
                }

                results.push(build_artifact(
                    &repo_ref,
                    tag,
                    &digest,
                    &manifest,
                    entry.platform.clone(),
                ));
            }
        }

        if !platform_candidate_found {
            let os = platform.and_then(|p| p.os.clone());
            let arch = platform.and_then(|p| p.architecture.clone());
            return Err(DiscoveryError::PlatformNotFound {
                reference: format!("{repo_ref}:{tag}"),
                os,
                arch,
            });
        }

        Ok(results)
    }

    // -----------------------------------------------------------------------
    // Public API
    // -----------------------------------------------------------------------

    /// Runs Phase 1 discovery for this registry using the given `input` filters.
    ///
    /// Outputs one [`DiscoveredArtifact`] per matching OCI artifact.
    ///
    /// # Errors
    /// Returns [`DiscoveryError`] on registry communication, authentication, or
    /// manifest validation failures.
    pub fn discover(
        &mut self,
        input: &DiscoveryInput,
    ) -> Result<Vec<DiscoveredArtifact>, DiscoveryError> {
        // Determine which repositories to scan.
        let repos: Vec<String> = if let Some(repo) = self.parsed.repository.clone() {
            vec![repo]
        } else {
            debug!("No repository specified; listing all repositories in {}", self.parsed.host);
            self.list_repositories()?
        };

        let mut all_artifacts: Vec<DiscoveredArtifact> = Vec::new();

        for repo in &repos {
            debug!("Scanning repository '{}/{repo}'", self.parsed.host);
            let repo_artifacts = self.discover_repo(repo, input)?;
            all_artifacts.extend(repo_artifacts);
        }

        Ok(all_artifacts)
    }

    /// Discovers artifacts in a specific repository.
    fn discover_repo(
        &mut self,
        repo: &str,
        input: &DiscoveryInput,
    ) -> Result<Vec<DiscoveredArtifact>, DiscoveryError> {
        // Override the client's parsed repository for this call by temporarily
        // adjusting api_base via the repo argument passed down to each method.
        let tags: Vec<String> = if let Some(tag) = &self.parsed.tag.clone() {
            vec![tag.clone()]
        } else {
            debug!("No tag specified; listing all tags for {}/{repo}", self.parsed.host);
            let tags = self.list_tags(repo)?;
            if tags.is_empty() {
                warn!("No tags found for {}/{repo}", self.parsed.host);
            }
            tags
        };

        let mut all_artifacts: Vec<DiscoveredArtifact> = Vec::new();
        let repo_ref = format!("{}/{}", self.parsed.host, repo);

        for tag in &tags {
            debug!("Processing tag '{tag}' in {repo_ref}");
            // Temporarily set the repository so fetch_manifest uses the right URL.
            let prev_repo = self.parsed.repository.replace(repo.to_string());
            let result = self.fetch_manifest(tag);
            self.parsed.repository = prev_repo;

            let (digest, moi) = match result {
                Ok(m) => m,
                Err(DiscoveryError::NotFound { .. }) => {
                    warn!("Tag '{tag}' not found in {repo_ref}; skipping");
                    continue;
                }
                Err(e) => return Err(e),
            };

            match moi {
                OciManifestOrIndex::Manifest(manifest) => {
                    if let Err(e) = Self::validate_manifest(&manifest, &format!("{repo_ref}:{tag}")) {
                        warn!("Validation failed for {repo_ref}:{tag} – {e}");
                        continue;
                    }

                    if let Err(e) = Self::validate_dsc_annotations(
                        &manifest,
                        input.kind.as_ref(),
                        &format!("{repo_ref}:{tag}"),
                    ) {
                        warn!("DSC validation failed for {repo_ref}:{tag} – {e}");
                        continue;
                    }

                    if !Self::manifest_matches_filters(
                        &manifest,
                        input.kind.as_ref(),
                        input.version.as_deref(),
                    ) {
                        continue;
                    }

                    all_artifacts.push(build_artifact(
                        &repo_ref,
                        tag,
                        &digest,
                        &manifest,
                        None,
                    ));
                }

                OciManifestOrIndex::Index(index) => {
                    debug!("Tag '{tag}' is an OCI index with {} entries", index.manifests.len());
                    let platform_filter = input.platform.as_ref();
                    // Temporarily set the repository for process_index.
                    let prev_repo = self.parsed.repository.replace(repo.to_string());
                    let result = self.process_index(
                        &index,
                        tag,
                        platform_filter,
                        input.kind.as_ref(),
                        input.version.as_deref(),
                    );
                    self.parsed.repository = prev_repo;

                    let mut artifacts = result?;

                    if artifacts.is_empty() && platform_filter.is_some() {
                        let pf = platform_filter.unwrap();
                        warn!(
                            "No platform variant matched for {repo_ref}:{tag} (os={:?}, arch={:?})",
                            pf.os, pf.architecture
                        );
                    }

                    all_artifacts.append(&mut artifacts);
                }
            }
        }

        Ok(all_artifacts)
    }
}

// ---------------------------------------------------------------------------
// Internal free functions
// ---------------------------------------------------------------------------

/// Returns `true` when an index entry's platform satisfies the requested filter.
fn platform_matches(entry: &OciIndexEntry, filter: &PlatformFilter) -> bool {
    let platform = match &entry.platform {
        Some(p) => p,
        None => {
            // If no platform is declared on the entry, also check annotations.
            return entry
                .annotations
                .as_ref()
                .map_or(true, |_| filter.os.is_none() && filter.architecture.is_none());
        }
    };

    if let Some(os) = &filter.os {
        if platform.os.as_deref() != Some(os.as_str()) {
            return false;
        }
    }

    if let Some(arch) = &filter.architecture {
        if platform.architecture.as_deref() != Some(arch.as_str()) {
            return false;
        }
    }

    true
}

/// Constructs a [`DiscoveredArtifact`] from a resolved OCI manifest.
fn build_artifact(
    repository: &str,
    tag: &str,
    digest: &str,
    manifest: &OciManifest,
    platform: Option<OciPlatform>,
) -> DiscoveredArtifact {
    let annotations: HashMap<String, String> = manifest
        .annotations
        .clone()
        .unwrap_or_default();

    let artifact_type = manifest
        .artifact_type
        .clone()
        .unwrap_or_else(|| manifest.config.media_type.clone());

    let kind = annotations.get(ANNOTATION_KIND).cloned();

    let name = annotations
        .get(ANNOTATION_NAME)
        .or_else(|| annotations.get(ANNOTATION_RESOURCE_ID))
        .or_else(|| annotations.get(ANNOTATION_PUBLISHER))
        .cloned();

    let version = annotations
        .get(ANNOTATION_VERSION)
        .or_else(|| annotations.get(ANNOTATION_RESOURCE_VERSION))
        .cloned();

    // Collect layer descriptors, cloning only the required fields.
    let layers: Vec<OciDescriptor> = manifest.layers.clone();

    DiscoveredArtifact {
        repository: repository.to_string(),
        tag: tag.to_string(),
        digest: digest.to_string(),
        artifact_type,
        kind,
        name,
        version,
        platform,
        layers,
        annotations,
    }
}

// ---------------------------------------------------------------------------
// Extension trait for optional bearer auth
// ---------------------------------------------------------------------------

trait RequestBuilderExt {
    fn bearer_auth_opt(self, token: Option<&str>) -> Self;
}

impl RequestBuilderExt for reqwest::blocking::RequestBuilder {
    fn bearer_auth_opt(self, token: Option<&str>) -> Self {
        if let Some(t) = token {
            self.header(AUTHORIZATION, format!("Bearer {t}"))
        } else {
            self
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    fn sample_layer() -> OciDescriptor {
        OciDescriptor {
            media_type: "application/tar+gzip".to_string(),
            digest: "sha256:aaaaaaaa".to_string(),
            size: 42,
            annotations: None,
        }
    }

    fn sample_resource_manifest() -> OciManifest {
        let mut annotations = HashMap::new();
        annotations.insert(ANNOTATION_KIND.to_string(), "resource".to_string());
        annotations.insert(
            ANNOTATION_RESOURCE_ID.to_string(),
            "Contoso/MyResource".to_string(),
        );
        annotations.insert(ANNOTATION_VERSION.to_string(), "2.0.0".to_string());

        OciManifest {
            schema_version: 2,
            media_type: MEDIA_TYPE_OCI_MANIFEST.to_string(),
            artifact_type: Some(ArtifactKind::Resource.artifact_type().to_string()),
            config: OciDescriptor {
                media_type: "application/vnd.oci.empty.v1+json".to_string(),
                digest: "sha256:bbbbbbbb".to_string(),
                size: 2,
                annotations: None,
            },
            layers: vec![sample_layer()],
            annotations: Some(annotations),
        }
    }

    #[test]
    fn single_manifest_matches_kind_and_version_filters() {
        let manifest = sample_resource_manifest();
        assert!(RegistryClient::manifest_matches_filters(
            &manifest,
            Some(&ArtifactKind::Resource),
            Some("2.0.0")
        ));
    }

    #[test]
    fn no_match_when_version_differs() {
        let manifest = sample_resource_manifest();
        assert!(!RegistryClient::manifest_matches_filters(
            &manifest,
            Some(&ArtifactKind::Resource),
            Some("9.9.9")
        ));
    }

    #[test]
    fn index_platform_selection_matches_expected_variant() {
        let entry = OciIndexEntry {
            media_type: MEDIA_TYPE_OCI_MANIFEST.to_string(),
            digest: "sha256:cccccccc".to_string(),
            size: 100,
            platform: Some(OciPlatform {
                os: Some("windows".to_string()),
                architecture: Some("amd64".to_string()),
            }),
            annotations: None,
        };

        let filter = PlatformFilter {
            os: Some("windows".to_string()),
            architecture: Some("amd64".to_string()),
        };

        assert!(platform_matches(&entry, &filter));
    }

    #[test]
    fn annotation_validation_rejects_missing_required_resource_id() {
        let mut manifest = sample_resource_manifest();
        let mut annotations = manifest.annotations.clone().unwrap_or_default();
        annotations.remove(ANNOTATION_RESOURCE_ID);
        manifest.annotations = Some(annotations);

        let result = RegistryClient::validate_dsc_annotations(
            &manifest,
            Some(&ArtifactKind::Resource),
            "test/reference",
        );

        assert!(result.is_err());
    }

    #[test]
    fn malformed_manifest_json_fails_to_parse() {
        let raw = "{\"schemaVersion\":2,\"mediaType\":\"application/vnd.oci.image.manifest.v1+json\"";
        let result = serde_json::from_str::<OciManifest>(raw);
        assert!(result.is_err());
    }
}
