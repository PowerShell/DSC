// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::{dscerror::DscError, extensions::dscextension::DscExtension, dscresources::dscresource::DscResource};
use std::collections::BTreeMap;
use super::command_discovery::ImportedManifest;

#[derive(Debug, PartialEq)]
pub enum DiscoveryKind {
    Resource,
    Extension,
}

#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub struct DiscoveryFilter {
    resource_type: String,
    version: Option<String>,
}

impl DiscoveryFilter {
    pub fn new(resource_type: String, version: Option<String>) -> Self {
        // The semver crate uses caret (meaning compatible) by default instead of exact if not specified
        // If the first character is a number, then we prefix with =
        let version = match version {
            Some(v) if v.chars().next().map_or(false, |c| c.is_ascii_digit()) => Some(format!("={v}")),
            other => other,
        };
        Self {
            resource_type: resource_type.to_lowercase(),
            version,
        }
    }

    pub fn resource_type(&self) -> &str {
        &self.resource_type
    }

    pub fn version(&self) -> Option<&String> {
        self.version.as_ref()
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
    fn discover(&mut self, kind: &DiscoveryKind, filter: &str) -> Result<(), DscError>;

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
    fn discover_adapted_resources(&mut self, name_filter: &str, adapter_filter: &str) -> Result<(), DscError>;

    /// List available resources based on the provided filters.
    ///
    /// # Arguments
    ///
    /// * `kind` - The kind of discovery (e.g., Resource).
    /// * `type_name_filter` - The filter for the resource type name.
    /// * `adapter_name_filter` - The filter for the adapter name (only applies to resources).
    ///
    /// # Returns
    ///
    /// A result containing a map of resource names to their corresponding `ManifestResource` instances.
    ///
    /// # Errors
    ///
    /// This function will return an error if the underlying discovery fails.
    fn list_available(&mut self, kind: &DiscoveryKind, type_name_filter: &str, adapter_name_filter: &str) -> Result<BTreeMap<String, Vec<ImportedManifest>>, DscError>;

    /// Find resources based on the required resource types.
    /// This is not applicable for extensions.
    ///
    /// # Arguments
    ///
    /// * `required_resource_types` - A slice of strings representing the required resource types.
    ///
    /// # Returns
    ///
    /// A result containing a map of resource names to their corresponding `DscResource` instances.
    ///
    /// # Errors
    ///
    /// This function will return an error if the underlying discovery fails.
    fn find_resources(&mut self, required_resource_types: &[DiscoveryFilter]) -> Result<BTreeMap<String, Vec<DscResource>>, DscError>;

    /// Get the available extensions.
    ///
    /// # Returns
    ///
    /// A result containing a map of extension names to their corresponding `DscExtension` instances.
    ///
    /// # Errors
    ///
    /// This function will return an error if the underlying discovery fails.
    fn get_extensions(&mut self) -> Result<BTreeMap<String, DscExtension>, DscError>;
}
