// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::discovery::discovery_trait::ResourceDiscovery;
use crate::dscresources::dscresource::{DscResource, ImplementedAs};
use crate::dscresources::resource_manifest::{ResourceManifest, import_manifest};
use crate::dscresources::command_resource::invoke_command;
use crate::dscerror::{DscError, StreamMessage, StreamMessageType};
use std::collections::BTreeMap;
use std::env;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use tracing::{debug, error};

pub struct CommandDiscovery {
}

impl CommandDiscovery {
    pub fn new() -> CommandDiscovery {
        CommandDiscovery {
        }
    }
}

impl Default for CommandDiscovery {
    fn default() -> Self {
        Self::new()
    }
}

impl ResourceDiscovery for CommandDiscovery {

    fn list_available_resources(&mut self) -> Result<BTreeMap<String, DscResource>, DscError> {

        let mut resources: BTreeMap<String, DscResource> = BTreeMap::new();
        let mut provider_resources: Vec<String> = Vec::new();
        // try DSC_RESOURCE_PATH env var first otherwise use PATH
        let path_env = match env::var_os("DSC_RESOURCE_PATH") {
            Some(value) => value,
            None => {
                match env::var_os("PATH") {
                    Some(value) => value,
                    None => {
                        return Err(DscError::Operation("Failed to get PATH environment variable".to_string()));
                    }
                }
            }
        };

        for path in env::split_paths(&path_env) {
            if path.exists() && path.is_dir() {
                for entry in path.read_dir().unwrap() {
                    let entry = entry.unwrap();
                    let path = entry.path();
                    if path.is_file() {
                        let file_name = path.file_name().unwrap().to_str().unwrap();
                        if file_name.to_lowercase().ends_with(".dsc.resource.json") {
                            let resource = load_manifest(&path)?;
                            if resource.manifest.is_some() {
                                let manifest = import_manifest(resource.manifest.clone().unwrap())?;
                                if manifest.provider.is_some() {
                                    provider_resources.push(resource.type_name.clone());
                                }
                            }
                            resources.insert(resource.type_name.clone(), resource);
                        }
                    }
                }
            }
        }

        debug!("Found {} non-provider resources", resources.len());

        // now go through the provider resources and add them to the list of resources
        for provider in provider_resources {
            let provider_resource = resources.get(&provider).unwrap();
            let provider_type_name = provider_resource.type_name.clone();
            let provider_path = provider_resource.path.clone();
            let manifest = import_manifest(provider_resource.manifest.clone().unwrap())?;
            let mut provider_resources_count = 0;
            // invoke the list command
            let list_command = manifest.provider.unwrap().list;
            let (exit_code, stdout, stderr) = match invoke_command(&list_command.executable, list_command.args, None, Some(&provider_resource.directory), None)
            {
                Ok((exit_code, stdout, stderr)) => (exit_code, stdout, stderr),
                Err(e) => {
                    error!("Could not start {}: {}", list_command.executable, e);
                    continue;
                },
            };

            if exit_code != 0 {
                    error!("Provider failed to list resources with exit code {exit_code}: {stderr}");
            }

            for line in stdout.lines() {
                match serde_json::from_str::<DscResource>(line){
                    Result::Ok(resource) => {
                        if resource.requires.is_none() {
                            error!("{}", DscError::MissingRequires(provider.clone(), resource.type_name.clone()).to_string());
                            continue;
                        }
                        resources.insert(resource.type_name.clone(), resource);
                        provider_resources_count += 1;
                    },
                    Result::Err(err) => {
                        error!("Failed to parse resource: {line} -> {err}");
                        continue;
                    }
                };
            }

            debug!("Provider {} listed {} resources", provider_type_name, provider_resources_count);
        }

        Ok(resources)
    }
}

fn load_manifest(path: &Path) -> Result<DscResource, DscError> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let manifest: ResourceManifest = match serde_json::from_reader(reader) {
        Ok(manifest) => manifest,
        Err(err) => {
            return Err(DscError::Manifest(path.to_string_lossy().to_string(), err));
        }
    };
    let resource = DscResource {
        type_name: manifest.resource_type.clone(),
        implemented_as: ImplementedAs::Command,
        description: manifest.description.clone(),
        version: manifest.version.clone(),
        path: path.to_str().unwrap().to_string(),
        directory: path.parent().unwrap().to_str().unwrap().to_string(),
        manifest: Some(serde_json::to_value(manifest)?),
        ..Default::default()
    };

    Ok(resource)
}
