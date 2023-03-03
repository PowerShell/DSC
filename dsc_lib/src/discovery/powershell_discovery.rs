use crate::discovery::discovery_trait::{ResourceDiscovery};
use crate::dscresources::dscresource::{DscResource, ImplementedAs};

pub struct PowerShellDiscovery {
    pub resources: Vec<DscResource>,
}

impl PowerShellDiscovery {
    pub fn new() -> PowerShellDiscovery {
        PowerShellDiscovery {
            resources: Vec::new(),
        }
    }
}

impl Default for PowerShellDiscovery {
    fn default() -> Self {
        Self::new()
    }
}

impl ResourceDiscovery for PowerShellDiscovery {
    fn discover(&self) -> Box<dyn Iterator<Item = DscResource>> {
        // use `Get-DscResource` to convert to config resources
        // these are just test resources
        let mut file_resource = DscResource::new();
        file_resource.name = "File".to_string();
        file_resource.implemented_as = ImplementedAs::PowerShell;
        let mut registry_resource = DscResource::new();
        registry_resource.name = "PSGet".to_string();
        registry_resource.implemented_as = ImplementedAs::PowerShell;

        let resources = vec![
            file_resource,
            registry_resource,
        ];

        Box::new(resources.clone().into_iter())
    }

    fn initialize(&mut self) -> Result<(), crate::dscerror::DscError> {
        Ok(())
    }
}
