// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

pub mod command_discovery;
pub mod discovery_trait;

use crate::configure::config_doc::ResourceDiscoveryMode;
use crate::discovery::discovery_trait::{DiscoveryKind, ResourceDiscovery, DiscoveryFilter};
use crate::dscerror::DscError;
use crate::extensions::dscextension::{Capability, DscExtension};
use crate::types::{FullyQualifiedTypeName, TypeNameFilter};
use crate::{dscresources::dscresource::DscResource, progress::ProgressFormat};
use core::result::Result::Ok;
use semver::Version;
use std::collections::BTreeMap;
use command_discovery::{CommandDiscovery, ImportedManifest};
use tracing::error;

/// Defines the caching [`BTreeMap`] for discovered DSC extensions.
type DiscoveryExtensionCache = BTreeMap<FullyQualifiedTypeName, DscExtension>;
/// Defines the caching [`BTreeMap`] for discovered DSC manifests of any type.
type DiscoveryManifestCache = BTreeMap<FullyQualifiedTypeName, Vec<ImportedManifest>>;
/// Defines the caching [`BTreeMap`] for discovered DSC resources.
type DiscoveryResourceCache = BTreeMap<FullyQualifiedTypeName, Vec<DscResource>>;

#[derive(Clone)]
pub struct Discovery {
    pub resources: DiscoveryResourceCache,
    pub extensions: DiscoveryExtensionCache,
    pub refresh_cache: bool,
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
            resources: DiscoveryResourceCache::new(),
            extensions: DiscoveryExtensionCache::new(),
            refresh_cache: false,
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
    pub fn list_available(
        &mut self,
        kind: &DiscoveryKind,
        type_name_filter: &TypeNameFilter,
        adapter_name_filter: Option<&TypeNameFilter>,
        progress_format: ProgressFormat
    ) -> Vec<ImportedManifest> {
        let discovery_types: Vec<Box<dyn ResourceDiscovery>> = vec![
            Box::new(command_discovery::CommandDiscovery::new(progress_format)),
        ];

        let mut resources: BTreeMap<String, ImportedManifest> = BTreeMap::new();

        for mut discovery_type in discovery_types {

            let discovered_resources = match discovery_type.list_available(kind, type_name_filter, adapter_name_filter) {
                Ok(value) => value,
                Err(err) => {
                    error!("{err}");
                    continue;
                }
            };

            for (_resource_name, found_resources) in discovered_resources {
                'manifests: for manifest in found_resources {
                    match manifest {
                        ImportedManifest::Resource(ref resource) => {
                            let key = format!("{}@{}", resource.type_name, resource.version);
                            if resources.contains_key(&key) {
                                continue 'manifests; // if we already have this resource, we can skip it
                            } else {
                                resources.insert(key, manifest.clone());
                            }
                        },
                        ImportedManifest::Extension(ref extension) => {
                            let key = format!("{}@{}", extension.type_name, extension.version);
                            if resources.contains_key(&key) {
                                continue 'manifests; // if we already have this extension, we can skip it
                            } else {
                                resources.insert(key, manifest.clone());
                            }
                        }
                    }
                }
            };

            if let Ok(extensions) = discovery_type.get_extensions() {
                self.extensions.extend(extensions);
            }
        }

        resources.into_iter().map(|(_key, value)| value).collect::<Vec<ImportedManifest>>()
    }

    pub fn get_extensions(&mut self, capability: &Capability) -> Vec<DscExtension> {
        if self.extensions.is_empty() {
            self.list_available(&DiscoveryKind::Extension, &TypeNameFilter::default(), None, ProgressFormat::None);
        }
        self.extensions.values()
            .filter(|ext| ext.capabilities.contains(capability))
            .cloned()
            .collect()
    }

    #[must_use]
    pub fn find_resource(&mut self, filter: &DiscoveryFilter) -> Result<Option<&DscResource>, DscError> {
        if self.refresh_cache || self.resources.is_empty() {
            self.find_resources(&[filter.clone()], ProgressFormat::None)?;
        }

        let type_name = filter.resource_type();
        if let Some(resources) = self.resources.get(type_name) {
            if let Some(version_req) = filter.require_version() {
                for resource in resources {
                    if version_req.matches(&resource.version) && matches_adapter_requirement(resource, filter) {
                        return Ok(Some(resource));
                    }
                }
                Ok(None)
            } else {
                for resource in resources {
                    if matches_adapter_requirement(resource, filter) {
                        return Ok(Some(resource));
                    }
                }
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }

    /// Find resources based on the required resource types.
    ///
    /// # Arguments
    ///
    /// * `required_resource_types` - The required resource types.
    pub fn find_resources(&mut self, required_resource_types: &[DiscoveryFilter], progress_format: ProgressFormat) -> Result<(), DscError> {
        if !self.refresh_cache && !self.resources.is_empty() {
            // If resources are already discovered, no need to re-discover.
            return Ok(());
        }

        let mut command_discovery = CommandDiscovery::new(progress_format);
        if self.refresh_cache {
            self.resources.clear();
            self.extensions.clear();
            command_discovery.set_discovery_mode(&ResourceDiscoveryMode::DuringDeployment);
        }
        let discovery_types: Vec<Box<dyn ResourceDiscovery>> = vec![
            Box::new(command_discovery),
        ];
        for mut discovery_type in discovery_types {

            let discovered_resources = discovery_type.find_resources(required_resource_types)?;
            for (resource_name, resources) in discovered_resources {
                self.resources.entry(resource_name).or_default().extend(resources);
            }

            if let Ok(extensions) = discovery_type.get_extensions() {
                self.extensions.extend(extensions);
            }
        }
        Ok(())
    }
}

/// Check if a resource matches the adapter requirement specified in the filter.
///
/// # Arguments
/// * `resource` - The resource to check.
/// * `filter` - The discovery filter containing the adapter requirement.
///
/// # Returns
/// `true` if the resource matches the adapter requirement, `false` otherwise.
pub fn matches_adapter_requirement(resource: &DscResource, filter: &DiscoveryFilter) -> bool {
    if let Some(required_adapter) = filter.require_adapter() {
        if let Some(resource_adapter) = &resource.require_adapter {
            required_adapter == resource_adapter
        } else {
            false
        }
    } else {
        true
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
