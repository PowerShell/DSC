// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

pub mod command_discovery;
pub mod discovery_trait;

use crate::discovery::discovery_trait::{DiscoveryKind, ResourceDiscovery, DiscoveryFilter};
use crate::extensions::dscextension::{Capability, DscExtension};
use crate::{dscresources::dscresource::DscResource, progress::ProgressFormat};
use core::result::Result::Ok;
use semver::{Version, VersionReq};
use std::collections::BTreeMap;
use command_discovery::{CommandDiscovery, ImportedManifest};
use tracing::error;

#[derive(Clone)]
pub struct Discovery {
    pub resources: BTreeMap<String, Vec<DscResource>>,
    pub extensions: BTreeMap<String, DscExtension>,
}

impl Discovery {
    /// Create a new `Discovery` instance.
    ///
    /// # Errors
    ///
    /// This function will return an error if the underlying instance creation fails.
    ///
    #[must_use]
    pub fn new() -> Self {
        Self {
            resources: BTreeMap::new(),
            extensions: BTreeMap::new(),
        }
    }

    /// List operation for getting available resources based on the filters.
    ///
    /// # Arguments
    ///
    /// * `kind` - The kind of discovery (e.g., Resource).
    /// * `type_name_filter` - The filter for the resource type name.
    /// * `adapter_name_filter` - The filter for the adapter name.
    ///
    /// # Returns
    ///
    /// A vector of `DscResource` instances.
    pub fn list_available(&mut self, kind: &DiscoveryKind, type_name_filter: &str, adapter_name_filter: &str, progress_format: ProgressFormat) -> Vec<ImportedManifest> {
        let discovery_types: Vec<Box<dyn ResourceDiscovery>> = vec![
            Box::new(command_discovery::CommandDiscovery::new(progress_format)),
        ];

        let mut resources: Vec<ImportedManifest> = Vec::new();

        for mut discovery_type in discovery_types {

            let discovered_resources = match discovery_type.list_available(kind, type_name_filter, adapter_name_filter) {
                Ok(value) => value,
                Err(err) => {
                    error!("{err}");
                    continue;
                }
            };

            for (_resource_name, found_resources) in discovered_resources {
                for resource in found_resources {
                    resources.push(resource.clone());
                }
            };

            if let Ok(extensions) = discovery_type.get_extensions() {
                self.extensions.extend(extensions);
            }
        }

        resources
    }

    pub fn get_extensions(&mut self, capability: &Capability) -> Vec<DscExtension> {
        if self.extensions.is_empty() {
            self.list_available(&DiscoveryKind::Extension, "*", "", ProgressFormat::None);
        }
        self.extensions.values()
            .filter(|ext| ext.capabilities.contains(capability))
            .cloned()
            .collect()
    }

    #[must_use]
    pub fn find_resource(&mut self, type_name: &str, version_string: Option<&str>) -> Option<&DscResource> {
        if self.resources.is_empty() {
            let discovery_filter = DiscoveryFilter::new(type_name, version_string.map(std::string::ToString::to_string));
            self.find_resources(&[discovery_filter], ProgressFormat::None);
        }

        let type_name = type_name.to_lowercase();
        if let Some(resources) = self.resources.get(&type_name) {
            if let Some(version) = version_string {
                let version = fix_semver(version);
                if let Ok(version_req) = VersionReq::parse(&version) {
                    for resource in resources {
                        if let Ok(resource_version) = Version::parse(&resource.version) {
                            if version_req.matches(&resource_version) {
                                return Some(resource);
                            }
                        }
                    }
                    None
                } else {
                    for resource in resources {
                        if resource.version == version {
                            return Some(resource);
                        }
                    }
                    None
                }
            } else {
                resources.first()
            }
        } else {
            None
        }
    }

    /// Find resources based on the required resource types.
    ///
    /// # Arguments
    ///
    /// * `required_resource_types` - The required resource types.
    pub fn find_resources(&mut self, required_resource_types: &[DiscoveryFilter], progress_format: ProgressFormat) {
        if !self.resources.is_empty() {
            // If resources are already discovered, no need to re-discover.
            return;
        }

        let command_discovery = CommandDiscovery::new(progress_format);
        let discovery_types: Vec<Box<dyn ResourceDiscovery>> = vec![
            Box::new(command_discovery),
        ];
        for mut discovery_type in discovery_types {

            let discovered_resources = match discovery_type.find_resources(required_resource_types) {
                Ok(value) => value,
                Err(err) => {
                    error!("{err}");
                    continue;
                }
            };

            for (resource_name, resources) in discovered_resources {
                self.resources.entry(resource_name).or_default().extend(resources);
            }

            if let Ok(extensions) = discovery_type.get_extensions() {
                self.extensions.extend(extensions);
            }
        }
    }
}

/// Fix the semantic versioning requirements of a given version requirements string.
/// The `semver` crate uses caret (meaning compatible) by default instead of exact if not specified
///
/// # Parameters
/// * `version` - The version requirements string to fix.
///
/// # Returns
/// The fixed version requirements string.
#[must_use]
pub fn fix_semver(version: &str) -> String {
    // Check if is semver, then if the first character is a number, then we prefix with =
    if Version::parse(version).is_ok() && version.chars().next().is_some_and(|c| c.is_ascii_digit()) {
        return format!("={version}");
    }
    version.to_string()
}

impl Default for Discovery {
    fn default() -> Self {
        Self::new()
    }
}

pub struct ResourceIterator {
    resources: Vec<DscResource>,
    index: usize,
}

impl ResourceIterator {
    #[must_use]
    pub fn new(resources: Vec<DscResource>) -> ResourceIterator {
        ResourceIterator {
            resources,
            index: 0,
        }
    }
}

impl Iterator for ResourceIterator {
    type Item = DscResource;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.resources.len() {
            let resource = self.resources[self.index].clone();
            self.index += 1;
            Some(resource)
        } else {
            None
        }
    }
}
