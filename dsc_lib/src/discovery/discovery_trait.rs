// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::{dscerror::DscError, dscresources::dscresource::DscResource};
use std::collections::BTreeMap;

use super::command_discovery::ManifestResource;

#[derive(PartialEq)]
pub enum DiscoveryKind {
    Resource,
    Extension,
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
    fn list_available(&mut self, kind: &DiscoveryKind, type_name_filter: &str, adapter_name_filter: &str) -> Result<BTreeMap<String, Vec<ManifestResource>>, DscError>;

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
    fn find_resources(&mut self, required_resource_types: &[String]) -> Result<BTreeMap<String, DscResource>, DscError>;
}
