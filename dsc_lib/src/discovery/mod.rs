// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

pub mod command_discovery;
pub mod discovery_trait;

use crate::discovery::discovery_trait::{DiscoveryKind, ResourceDiscovery};
use crate::extensions::dscextension::DscExtension;
use crate::{dscresources::dscresource::DscResource, dscerror::DscError, progress::ProgressFormat};
use std::collections::BTreeMap;
use command_discovery::ManifestResource;
use tracing::{debug, error};

#[derive(Clone)]
pub struct Discovery {
    pub resources: BTreeMap<String, DscResource>,
    pub extensions: BTreeMap<String, DscExtension>,
}

impl Discovery {
    /// Create a new `Discovery` instance.
    ///
    /// # Errors
    ///
    /// This function will return an error if the underlying instance creation fails.
    ///
    pub fn new() -> Result<Self, DscError> {
        Ok(Self {
            resources: BTreeMap::new(),
            extensions: BTreeMap::new(),
        })
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
    pub fn list_available(&mut self, kind: &DiscoveryKind, type_name_filter: &str, adapter_name_filter: &str, progress_format: ProgressFormat) -> Vec<ManifestResource> {
        let discovery_types: Vec<Box<dyn ResourceDiscovery>> = vec![
            Box::new(command_discovery::CommandDiscovery::new(progress_format)),
        ];

        let mut resources: Vec<ManifestResource> = Vec::new();

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
        }

        resources
    }

    #[must_use]
    pub fn find_resource(&self, type_name: &str) -> Option<&DscResource> {
        self.resources.get(&type_name.to_lowercase())
    }

    /// Find resources based on the required resource types.
    ///
    /// # Arguments
    ///
    /// * `required_resource_types` - The required resource types.
    pub fn find_resources(&mut self, required_resource_types: &[String], progress_format: ProgressFormat) {
        let discovery_types: Vec<Box<dyn ResourceDiscovery>> = vec![
            Box::new(command_discovery::CommandDiscovery::new(progress_format)),
        ];
        let mut remaining_required_resource_types = required_resource_types.to_owned();
        for mut discovery_type in discovery_types {

            let discovered_resources = match discovery_type.find_resources(&remaining_required_resource_types) {
                Ok(value) => value,
                Err(err) => {
                    error!("{err}");
                    continue;
                }
            };

            for resource in discovered_resources {
                self.resources.insert(resource.0.clone(), resource.1);
                remaining_required_resource_types.retain(|x| *x != resource.0);
            };
        }
    }
}

fn convert_wildcard_to_regex(wildcard: &str) -> String {
    let mut regex = wildcard.to_string().replace('.', "\\.").replace('?', ".").replace('*', ".*?");
    regex.insert(0, '^');
    regex.push('$');
    regex
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_wildcard_to_regex() {
        let wildcard = "*";
        let regex = convert_wildcard_to_regex(wildcard);
        assert_eq!(regex, "^.*?$");

        let wildcard = "File";
        let regex = convert_wildcard_to_regex(wildcard);
        assert_eq!(regex, "^File$");

        let wildcard = "r*";
        let regex = convert_wildcard_to_regex(wildcard);
        assert_eq!(regex, "^r.*?$");
    }
}

impl Default for Discovery {
    fn default() -> Self {
        Self::new().unwrap()
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
