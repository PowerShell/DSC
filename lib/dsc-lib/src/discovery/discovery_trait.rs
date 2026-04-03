// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::{
    configure::config_doc::ResourceDiscoveryMode,
    discovery::{DiscoveryExtensionCache, DiscoveryManifestCache, DiscoveryResourceCache},
    dscerror::DscError,
    types::{FullyQualifiedTypeName, ResourceVersionReq, SemanticVersionReq, TypeNameFilter}
};

#[derive(Debug, PartialEq)]
pub enum DiscoveryKind {
    Resource,
    Extension,
}

#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub struct DiscoveryFilter {
    require_adapter: Option<FullyQualifiedTypeName>,
    r#type: FullyQualifiedTypeName,
    require_version: Option<ResourceVersionReq>,
}

impl DiscoveryFilter {
    /// Construct a [`DiscoveryFilter`] for a resource with the specified type name, optional
    /// version requirement, and optional adapter requirement.
    ///
    /// # Arguments
    ///
    /// - `type_name` - The [`FullyQualifiedTypeName`] of the resource.
    /// - `require_version` - An optional [`ResourceVersionReq`] specifying the version requirement
    ///   for the resource. The version requirement can be semantic or date-based, depending on the
    ///   resource's versioning scheme.
    /// - `require_adapter` - An optional [`FullyQualifiedTypeName`] specifying the adapter that
    ///   the resource is expected to require.
    ///
    /// # Returns
    ///
    /// A new instance of [`DiscoveryFilter`] initialized with the provided parameters.
    pub fn new(
        type_name: &FullyQualifiedTypeName,
        require_version: Option<ResourceVersionReq>,
        require_adapter: Option<FullyQualifiedTypeName>
    ) -> Self {
        Self {
            require_adapter,
            r#type: type_name.clone(),
            require_version,
        }
    }

    /// Construct a [`DiscoveryFilter`] for an extension with the specified type name and optional
    /// version requirement.
    ///
    /// # Arguments
    ///
    /// - `type_name` - The [`FullyQualifiedTypeName`] of the extension.
    /// - `require_version` - An optional [`SemanticVersionReq`] specifying the semantic version
    ///    requirement for the extension.
    ///
    /// # Returns
    ///
    /// A new instance of [`DiscoveryFilter`] initialized with the provided parameters.
    ///
    /// Note that extensions do not have an adapter requirement, so the `require_adapter` field is
    /// always set to `None`.
    pub fn new_for_extension(
        type_name: &FullyQualifiedTypeName,
        require_version: Option<SemanticVersionReq>,
    ) -> Self {
        Self {
            require_adapter: None,
            r#type: type_name.clone(),
            require_version: require_version.map(|r| r.into()),
        }
    }

    #[must_use]
    pub fn require_adapter(&self) -> Option<&FullyQualifiedTypeName> {
        self.require_adapter.as_ref()
    }

    #[must_use]
    pub fn resource_type(&self) -> &FullyQualifiedTypeName {
        &self.r#type
    }

    #[must_use]
    pub fn require_version(&self) -> Option<&ResourceVersionReq> {
        self.require_version.as_ref()
    }
}

pub trait ResourceDiscovery {
    /// Discovery method to find resources.
    ///
    /// # Arguments
    ///
    /// * `kind` - The kind of discovery (e.g., Resource).
    /// * `filter` - The filter for the resource type name.
    ///
    /// # Returns
    ///
    /// A result indicating success or failure.
    ///
    /// # Errors
    ///
    /// This function will return an error if the underlying discovery fails.
    fn discover(&mut self, kind: &DiscoveryKind, filter: &TypeNameFilter) -> Result<(), DscError>;

    /// Discover adapted resources based on the provided filters.
    ///
    /// # Arguments
    ///
    /// * `name_filter` - The filter for the resource name.
    /// * `adapter_filter` - The filter for the adapter name.
    ///
    /// # Returns
    ///
    /// A result indicating success or failure.
    ///
    /// # Errors
    ///
    /// This function will return an error if the underlying discovery fails.
    fn discover_adapted_resources(
        &mut self,
        name_filter: &TypeNameFilter,
        adapter_filter: &TypeNameFilter
    ) -> Result<(), DscError>;

    /// List available resources based on the provided filters.
    ///
    /// # Arguments
    ///
    /// - `kind` - The kind of discovery (e.g., Resource).
    /// - `type_name_filter` - The filter for the resource type name.
    /// - `adapter_name_filter` - The filter for the adapter name (only applies to resources).
    ///
    /// # Returns
    ///
    /// A result containing a map of resource names to their corresponding `ManifestResource` instances.
    ///
    /// # Errors
    ///
    /// This function will return an error if the underlying discovery fails.
    fn list_available(
        &mut self,
        kind: &DiscoveryKind,
        type_name_filter: &TypeNameFilter,
        adapter_name_filter: Option<&TypeNameFilter>
    ) -> Result<DiscoveryManifestCache, DscError>;

    /// Find resources based on the required resource types.
    /// This is not applicable for extensions.
    ///
    /// # Arguments
    ///
    /// - `required_resource_types` - A slice of `DiscoveryFilter` instances representing the
    ///   required resource types.
    ///
    /// # Returns
    ///
    /// A result containing a map of resource names to their corresponding `DscResource` instances.
    ///
    /// # Errors
    ///
    /// This function will return an error if the underlying discovery fails.
    fn find_resources(&mut self, required_resource_types: &[DiscoveryFilter]) -> Result<DiscoveryResourceCache, DscError>;

    /// Get the available extensions.
    ///
    /// # Returns
    ///
    /// A result containing a map of extension names to their corresponding `DscExtension` instances.
    ///
    /// # Errors
    ///
    /// This function will return an error if the underlying discovery fails.
    fn get_extensions(&mut self) -> Result<DiscoveryExtensionCache, DscError>;

    /// Set the discovery mode.
    ///
    /// # Arguments
    ///
    /// - `mode` - The resource discovery mode to set.
    fn set_discovery_mode(&mut self, mode: &ResourceDiscoveryMode);
}
