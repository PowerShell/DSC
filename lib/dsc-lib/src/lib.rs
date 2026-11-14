// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::discovery::{command_discovery::ImportedManifest, discovery_trait::DiscoveryFilter};
use crate::discovery::discovery_trait::DiscoveryKind;
use crate::progress::ProgressFormat;
use crate::types::TypeNameFilter;

use dscerror::DscError;
use resources::resource_invoke::ResourceInvoke;
use rust_i18n::i18n;

pub mod actions;
pub mod configure;
pub mod diagnostics;
pub mod discovery;
pub mod dscerror;
pub mod dscresources;
pub mod extensions;
pub mod functions;
pub mod lint;
pub mod parser;
pub mod plugins;
pub mod progress;
pub mod resources;
pub mod schemas;
pub mod settings;
pub mod types;
pub mod util;

// Define module for unit tests and stubs
#[cfg(test)]
#[macro_use]
pub(crate) mod tests;

i18n!("locales", fallback = "en-us");

pub struct DscManager {
    discovery: discovery::Discovery,
}

impl DscManager {
    /// Create a new `DscManager` instance.
    ///
    /// # Errors
    ///
    /// This function will return an error if the underlying discovery fails.
    ///
    #[must_use]
    pub fn new() -> Self {
        Self {
            discovery: discovery::Discovery::new(),
        }
    }

    /// Find a resource by name.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the resource to find, can have wildcards.
    ///
    pub fn find_resource(&mut self, filter: &DiscoveryFilter) -> Result<Option<Box<dyn ResourceInvoke>>, DscError> {
        self.discovery.find_resource(filter)
    }

    pub fn list_available(
        &mut self,
        kind: &DiscoveryKind,
        type_name_filter: &TypeNameFilter,
        adapter_name_filter: Option<&TypeNameFilter>,
        progress_format: ProgressFormat
    ) -> Vec<ImportedManifest> {
        self.discovery.list_available(kind, type_name_filter, adapter_name_filter, progress_format)
    }

    pub fn find_resources(&mut self, required_resource_types: &[DiscoveryFilter], progress_format: ProgressFormat) -> Result<(), DscError> {
        self.discovery.find_resources(required_resource_types, progress_format)
    }
}

impl Default for DscManager {
    fn default() -> Self {
        Self::new()
    }
}
