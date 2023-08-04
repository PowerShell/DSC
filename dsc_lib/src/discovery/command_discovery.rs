// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::discovery::discovery_trait::{ResourceDiscovery};
use crate::dscresources::dscresource::{DscResource, ImplementedAs};
use crate::dscresources::resource_manifest::ResourceManifest;
use crate::dscresources::command_resource::invoke_command;
use crate::dscerror::{DscError, StreamMessage, StreamMessageType};
use std::collections::BTreeMap;
use std::env;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

pub struct CommandDiscovery {
    pub resources: BTreeMap<String, DscResource>,
    provider_resources: Vec<String>,
    initialization_messages: Vec<StreamMessage>,
    initialized: bool,
}

impl CommandDiscovery {
    pub fn new() -> CommandDiscovery {
        CommandDiscovery {
            resources: BTreeMap::new(),
            provider_resources: Vec::new(),
            initialization_messages: Vec::new(),
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
        if self.initialized {
            Box::new(self.resources.values().cloned().collect::<Vec<DscResource>>().into_iter())
        } else {
            Box::new(vec![].into_iter())
        }
    }

    fn initialize(&mut self) -> Result<(), DscError>{
        if self.initialized {
            return Ok(());
        }

        let Some(path_env) = env::var_os("PATH") else {
            return Err(DscError::Operation("Failed to get PATH environment variable".to_string()));
        };

        for path in env::split_paths(&path_env) {
            if path.exists() && path.is_dir() {
                for entry in path.read_dir().unwrap() {
                    let entry = entry.unwrap();
                    let path = entry.path();
                    if path.is_file() {
                        let file_name = path.file_name().unwrap().to_str().unwrap();
                        if file_name.to_lowercase().ends_with(".dsc.resource.json") {
                            let resource = import_manifest(&path)?;
                            if resource.manifest.is_some() {
                                let manifest = serde_json::from_value::<ResourceManifest>(resource.manifest.clone().unwrap())?;
                                if manifest.provider.is_some() {
                                    self.provider_resources.push(resource.type_name.clone());
                                }
                            }
                            self.resources.insert(resource.type_name.clone(), resource.clone());
                        }
                    }
                }
            }
        }

        // now go through the provider resources and add them to the list of resources
        for provider in &self.provider_resources {
            let provider_resource = self.resources.get(provider).unwrap();
            let provider_type_name = provider_resource.type_name.clone();
            let provider_path = provider_resource.path.clone();
            let manifest = serde_json::from_value::<ResourceManifest>(provider_resource.manifest.clone().unwrap())?;
            // invoke the list command
            let list_command = manifest.provider.unwrap().list;
            let (exit_code, stdout, stderr) = match invoke_command(&list_command.executable, list_command.args, None, Some(&provider_resource.directory))
            {
                Ok((exit_code, stdout, stderr)) => (exit_code, stdout, stderr),
                Err(e) => {
                    self.initialization_messages.push(StreamMessage::new_error(
                        format!("Could not start {}: {}", list_command.executable, e),
                        Some(provider_type_name.clone()),
                        Some(provider_path.clone())));

                    continue;
                },
            };

            if exit_code != 0 {
                self.initialization_messages.push(StreamMessage::new_error(
                    format!("Provider failed to list resources with exit code {exit_code}: {stderr}"),
                    Some(provider_type_name.clone()),
                    Some(provider_path.clone())));
            }
            
            for line in stdout.lines() {
                match serde_json::from_str::<DscResource>(line){
                    Result::Ok(resource) => {
                        if resource.requires.is_none() {
                            self.initialization_messages.push(StreamMessage::new_error(
                                DscError::MissingRequires(provider.clone(), resource.type_name.clone()).to_string(),
                                Some(resource.type_name.clone()),
                                Some(resource.path.clone())));
                            
                            continue;
                        }
                        self.resources.insert(resource.type_name.clone(), resource);
                    },
                    Result::Err(err) => {
                        self.initialization_messages.push(StreamMessage::new_error(
                            format!("Failed to parse resource: {line} -> {err}"),
                            Some(provider_type_name.clone()),
                            Some(provider_path.clone())));
                        
                        continue;
                    }
                };
            }
        }

        self.initialized = true;
        Ok(())
    }

    fn print_initialization_messages(&mut self, error_format:StreamMessageType, warning_format:StreamMessageType) -> Result<(), DscError>{
        for msg in &self.initialization_messages {
            msg.print(&error_format, &warning_format)?;
        }
        
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
