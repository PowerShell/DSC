use crate::dscerror::DscError;
use crate::discovery::discovery_trait::{ResourceDiscovery};
use crate::dscresources::dscresource::{DscResource, ImplementedAs};

pub struct CommandDiscovery {
    pub resources: Vec<DscResource>,
}

impl CommandDiscovery {
    pub fn new() -> CommandDiscovery {
        CommandDiscovery {
            resources: Vec::new(),
        }
    }
}

impl ResourceDiscovery for CommandDiscovery {
    fn discover(&self) -> Box<dyn Iterator<Item = DscResource>> {
        // find resources via PATH including .ps1 resources so PATH doesn't need to be traversed more than once
        // reuse code from https://github.com/PowerShell/MSH/blob/main/config/src/main.rs
        // these are just test resources
        let mut sshd_resource = DscResource::new();
        sshd_resource.name = "SSHDConfig".to_string();
        sshd_resource.implemented_as = ImplementedAs::Command;
        let mut registry_resource = DscResource::new();
        registry_resource.name = "Registry".to_string();
        registry_resource.implemented_as = ImplementedAs::Command;

        let resources = vec![
            sshd_resource,
            registry_resource,
        ];

        Box::new(resources.into_iter())
    }
}
