mod cache;
mod command_discovery;
mod discovery_trait;
mod powershell_discovery;

use crate::discovery::discovery_trait::ResourceDiscovery;
use crate::dscerror::DscError;
use crate::dscresources::dscresource::DscResource;
use regex::RegexBuilder;

pub struct Discovery {
    resources: Vec<DscResource>,
}

impl Discovery {
    pub fn new() -> Result<Discovery, DscError> {
        let discovery_types: Vec<Box<dyn ResourceDiscovery>> = vec![
            Box::new(command_discovery::CommandDiscovery::new()),
            Box::new(powershell_discovery::PowerShellDiscovery::new()),
        ];

        let mut resources: Vec<DscResource> = Vec::new();

        for mut discovery_type in discovery_types {
            discovery_type.initialize()?;
            let discovered_resources = discovery_type.discover();
            for resource in discovered_resources {
                resources.push(resource);
            }
        }

        Ok(Discovery {
            resources,
        })
    }

    pub fn find_resource(&self, name: &str) -> ResourceIterator {
        let mut regex_builder = RegexBuilder::new(convert_wildcard_to_regex(name).as_str());
        regex_builder.case_insensitive(true);
        let regex = regex_builder.build().unwrap();

        let mut resources: Vec<DscResource> = Vec::new();
        for resource in &self.resources {
            if name.is_empty() | regex.is_match(resource.name.as_str()) {
                resources.push(resource.clone());
            }
        }

        ResourceIterator::new(resources)
    }
}

fn convert_wildcard_to_regex(wildcard: &str) -> String {
    let mut regex = wildcard.to_string().replace('*', ".*").replace('?', ".");
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
        assert_eq!(regex, "^.*$");

        let wildcard = "File";
        let regex = convert_wildcard_to_regex(wildcard);
        assert_eq!(regex, "^File$");

        let wildcard = "r*";
        let regex = convert_wildcard_to_regex(wildcard);
        assert_eq!(regex, "^r.*$");
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
