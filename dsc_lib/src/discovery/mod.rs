// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

mod command_discovery;
mod discovery_trait;

use crate::discovery::discovery_trait::ResourceDiscovery;
use crate::{dscresources::dscresource::DscResource, dscerror::DscError};
use std::collections::BTreeMap;
use tracing::error;

pub struct Discovery {
    pub resources: BTreeMap<String, DscResource>,
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
        })
    }

    /// List operation.
    #[allow(clippy::missing_panics_doc)] // false positive in clippy; this function will never panic
    pub fn list_available_resources(&mut self, type_name_filter: &str, adapter_name_filter: &str) -> Vec<DscResource> {
        let discovery_types: Vec<Box<dyn ResourceDiscovery>> = vec![
            Box::new(command_discovery::CommandDiscovery::new()),
        ];

        let mut resources: Vec<DscResource> = Vec::new();

        for mut discovery_type in discovery_types {

            let discovered_resources = match discovery_type.list_available_resources(type_name_filter, adapter_name_filter) {
                Ok(value) => value,
                Err(err) => {
                    error!("{err}");
                    continue;
                }
            };

            for resource in discovered_resources {
                    resources.push(resource.1);
            };
        }

        resources
    }

    #[must_use]
    pub fn find_resource(&self, type_name: &str) -> Option<&DscResource> {
        self.resources.get(type_name)
    }

    pub fn discover_resources(&mut self, required_resource_types: &[String]) {

        let discovery_types: Vec<Box<dyn ResourceDiscovery>> = vec![
            Box::new(command_discovery::CommandDiscovery::new()),
        ];

        let mut remaining_required_resource_types = required_resource_types.to_owned();
        for mut discovery_type in discovery_types {

            let discovered_resources = match discovery_type.discover_resources(&remaining_required_resource_types) {
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
