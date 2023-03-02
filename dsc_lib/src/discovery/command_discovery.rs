use crate::discovery::discovery_trait::{ResourceDiscovery};
use crate::dscresources::dscresource::{DscResource, ImplementedAs};
use std::env;
use std::process::Command;

pub struct CommandDiscovery {
    pub resources: Vec<DscResource>,
    initlialized: bool,
}

impl CommandDiscovery {
    pub fn new() -> CommandDiscovery {
        CommandDiscovery {
            resources: Vec::new(),
            initlialized: false,
        }
    }
}

impl Default for CommandDiscovery {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(target_family = "unix")]
const PATH_SEPARATOR: char = ':';

#[cfg(target_family = "windows")]
const PATH_SEPARATOR: char = ';';

impl CommandDiscovery {
    fn initialize(&mut self) {
        if self.initlialized {
            return;
        }

        self.initlialized = true;

        // find resources via PATH including .ps1 resources so PATH doesn't need to be traversed more than once
        // reuse code from https://github.com/PowerShell/MSH/blob/main/config/src/main.rs
        // these are just test resources
        let path_env = match env::var("PATH") {
            Ok(path_env) => path_env,
            Err(_) => {
                eprintln!("Error: PATH environment variable not found.");
                return;
            }
        };

        let path_parts: Vec<&str> = path_env.split(PATH_SEPARATOR).collect();
        for path_part in path_parts {
            let path = std::path::Path::new(path_part);
            if path.exists() && path.is_dir() {
                for entry in path.read_dir().unwrap() {
                    let entry = entry.unwrap();
                    let path = entry.path();
                    if path.is_file() {
                        let file_name = path.file_name().unwrap().to_str().unwrap();
                        if file_name.ends_with(".ps1") {
                            let resource_name = file_name.replace(".ps1", "");
                            let mut dsc_resource = DscResource::new();
                            dsc_resource.name = resource_name;
                            dsc_resource.implemented_as = ImplementedAs::Command;
                            self.resources.push(dsc_resource);
                        }
                    }
                }
            }
        }
    }
}

impl ResourceDiscovery for CommandDiscovery {
    fn discover(&self) -> Box<dyn Iterator<Item = DscResource>> {
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
