pub mod cache;
pub mod command_discovery;
pub mod discovery_trait;
pub mod powershell_discovery;

use crate::discovery::discovery_trait::ResourceDiscovery;
use crate::dscresources::dscresource::DscResource;
use wildmatch::WildMatch;

pub struct Discovery {
    resources: Vec<DscResource>,
}

impl Discovery {
    pub fn new() -> Discovery {
        let discovery_types: Vec<Box<dyn ResourceDiscovery>> = vec![
            Box::new(command_discovery::CommandDiscovery::new()),
            Box::new(powershell_discovery::PowerShellDiscovery::new()),
        ];

        let mut resources: Vec<DscResource> = Vec::new();

        for discovery_type in discovery_types {
            let discovered_resources = discovery_type.discover();
            for resource in discovered_resources {
                resources.push(resource);
            }
        }

        Discovery {
            resources,
        }
    }

    pub fn find_resource(&self, resource_name: &String) -> Option<&DscResource> {

        self.resources.iter().find(|resource| resource.name == *resource_name)
    }
}
