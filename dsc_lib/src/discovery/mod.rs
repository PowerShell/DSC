// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

mod command_discovery;
mod discovery_trait;

use crate::discovery::discovery_trait::ResourceDiscovery;
use crate::dscerror::DscError;
use crate::dscresources::dscresource::{DscResource};
use regex::RegexBuilder;

pub struct Discovery {
    resources: Vec<DscResource>,
    initialized: bool,
}

impl Discovery {
    /// Create a new `Discovery` instance.
    ///
    /// # Errors
    ///
    /// This function will return an error if the underlying discovery fails.
    pub fn new() -> Result<Self, DscError> {
        Ok(Self {
            resources: Vec::new(),
            initialized: false,
        })
    }

    /// Initialize the discovery process.
    ///
    /// # Errors
    ///
    /// This function will return an error if the underlying discovery fails.
    pub fn initialize(&mut self) -> Result<(), DscError> {
        // TODO: this vec is leftover when we had multiple discovery types, we should
        //       probably just have a single command discovery type
        let discovery_types: Vec<Box<dyn ResourceDiscovery>> = vec![
            Box::new(command_discovery::CommandDiscovery::new()),
        ];

        let mut resources: Vec<DscResource> = Vec::new();

        for mut discovery_type in discovery_types {
            discovery_type.initialize()?;
            let discovered_resources = discovery_type.discover();
            for resource in discovered_resources {
                resources.push(resource.clone());
            }
        }

        self.resources = resources;
        self.initialized = true;
        Ok(())
    }

    // TODO: Need to support version?
    /// Find a resource by name.
    ///
    /// # Arguments
    ///
    /// * `type_name` - The name of the resource to find, can have wildcards.
    #[must_use]
    pub fn find_resource(&self, type_name: &str) -> ResourceIterator {
        if !self.initialized {
            return ResourceIterator::new(vec![]);
        }

        let mut regex_builder = RegexBuilder::new(convert_wildcard_to_regex(type_name).as_str());
        regex_builder.case_insensitive(true);
        let Ok(regex) = regex_builder.build() else {
            return ResourceIterator::new(vec![]);
        };

        let mut resources: Vec<DscResource> = Vec::new();
        for resource in &self.resources {
            if type_name.is_empty() | regex.is_match(resource.type_name.as_str()) {
                resources.push(resource.clone());
            }
        }

        ResourceIterator::new(resources)
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
