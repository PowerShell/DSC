use crate::discovery::discovery_trait::{ResourceDiscovery};
use crate::dscresources::dscresource::{DscResource, ImplementedAs};
use crate::dscresources::resource_manifest::ResourceManifest;
use crate::dscerror::DscError;
use std::env;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

pub struct CommandDiscovery {
    pub resources: Vec<DscResource>,
    initialized: bool,
}

impl CommandDiscovery {
    pub fn new() -> CommandDiscovery {
        CommandDiscovery {
            resources: Vec::new(),
            initialized: false,
        }
    }
}

impl Default for CommandDiscovery {
    fn default() -> Self {
        Self::new()
    }
}

impl ResourceDiscovery for CommandDiscovery {
    fn discover(&self) -> Box<dyn Iterator<Item = DscResource>> {
        match self.initialized {
            true => Box::new(self.resources.clone().into_iter()),
            false => Box::new(vec![].into_iter()),
        }
    }

    fn initialize(&mut self) -> Result<(), DscError>{
        if self.initialized {
            return Ok(());
        }

        let path_env = match env::var_os("PATH") {
            Some(path_env) => path_env,
            None => {
                return Err(DscError::Operation("Failed to get PATH environment variable".to_string()));
            }
        };

        for path in env::split_paths(&path_env) {
            if path.exists() && path.is_dir() {
                for entry in path.read_dir().unwrap() {
                    let entry = entry.unwrap();
                    let path = entry.path();
                    if path.is_file() {
                        let file_name = path.file_name().unwrap().to_str().unwrap();
                        if file_name.ends_with(".resource.json") {
                            let resource = import_manifest(&path)?;
                            self.resources.push(resource);
                        }
                    }
                }
            }
        }

        self.initialized = true;
        Ok(())
    }
}

fn import_manifest(path: &Path) -> Result<DscResource, DscError> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let manifest: ResourceManifest = match serde_json::from_reader(reader) {
        Ok(manifest) => manifest,
        Err(err) => {
            return Err(DscError::Manifest(path.to_string_lossy().to_string(), err));
        }
    };
    let resource = DscResource {
        name: manifest.name.clone(),
        implemented_as: ImplementedAs::Command,
        path: path.to_str().unwrap().to_string(),
        parent_path: path.parent().unwrap().to_str().unwrap().to_string(),
        manifest: Some(manifest.clone()),
        ..Default::default()
    };

    Ok(resource)
}
