// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::discovery::discovery_trait::ResourceDiscovery;
use crate::dscresources::dscresource::{DscResource, ImplementedAs};
use crate::dscresources::resource_manifest::{ResourceManifest, import_manifest};
use crate::dscresources::command_resource::invoke_command;
use crate::dscerror::DscError;
use std::collections::BTreeMap;
use std::env;
use std::ffi::OsStr;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use tracing::{debug, error, warn};

pub struct CommandDiscovery {
}

impl CommandDiscovery {
    pub fn new() -> CommandDiscovery {
        CommandDiscovery {
        }
    }

    #[allow(clippy::too_many_lines)]
    fn search_for_resources(required_resource_types: &[String]) -> Result<BTreeMap<String, DscResource>, DscError>
    {
        let return_all_resources = required_resource_types.len() == 1 && required_resource_types[0] == "*";

        let mut resources: BTreeMap<String, DscResource> = BTreeMap::new();
        let mut provider_resources: Vec<String> = Vec::new();
        let mut remaining_required_resource_types = required_resource_types.to_owned();
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
                        if file_name.to_lowercase().ends_with(".dsc.resource.json") |
                        file_name.to_lowercase().ends_with(".dsc.resource.yaml") |
                        file_name.to_lowercase().ends_with(".dsc.resource.yml") {
                            let resource = match load_manifest(&path)
                            {
                                Ok(r) => r,
                                Err(e) => {
                                    if return_all_resources {
                                        // In case of "resource list" operation - print all failures to read manifests as warnings
                                        warn!("{}", e);
                                    } else {
                                        /* In case of other resource/config operations:
                                           At this point we can't determine whether or not the bad manifest contains resource that is requested by resource/config operation
                                           if it is, then "ResouceNotFound" error will be issued later
                                           and here we just record the error into debug stream.*/
                                        debug!("{}", e);
                                    }
                                    continue;
                                },
                            };

                            if resource.manifest.is_some() {
                                let manifest = import_manifest(resource.manifest.clone().unwrap())?;
                                if manifest.provider.is_some() {
                                    provider_resources.push(resource.type_name.to_lowercase());
                                    resources.insert(resource.type_name.to_lowercase(), resource.clone());
                                }
                            }
                            if return_all_resources
                            {
                                resources.insert(resource.type_name.to_lowercase(), resource);
                            }
                            else if remaining_required_resource_types.contains(&resource.type_name.to_lowercase())
                            {
                                remaining_required_resource_types.retain(|x| *x != resource.type_name.to_lowercase());
                                debug!("Found {} in {}", &resource.type_name, path.display());
                                resources.insert(resource.type_name.to_lowercase(), resource);
                                if remaining_required_resource_types.is_empty()
                                {
                                    return Ok(resources);
                                }
                            }
                        }
                    }
                }
            }
        }

        debug!("Found {} matching non-provider resources", resources.len() - provider_resources.len());

        // now go through the provider resources and add them to the list of resources
        for provider in provider_resources {
            debug!("Enumerating resources for provider {}", provider);
            let provider_resource = resources.get(&provider).unwrap();
            let provider_type_name = provider_resource.type_name.clone();
            let manifest = import_manifest(provider_resource.manifest.clone().unwrap())?;
            let mut provider_resources_count = 0;
            // invoke the list command
            let list_command = manifest.provider.unwrap().list;
            let (exit_code, stdout, stderr) = match invoke_command(&list_command.executable, list_command.args, None, Some(&provider_resource.directory), None)
            {
                Ok((exit_code, stdout, stderr)) => (exit_code, stdout, stderr),
                Err(e) => {
                    /* In case of "resource list" operation - print failure from provider as warning
                       In case of other resource/config operations:
                       print failure from provider as error because this provider was specifically requested by current resource/config operation*/
                    if return_all_resources {
                        warn!("Could not start {}: {}", list_command.executable, e);
                    } else {
                        error!("Could not start {}: {}", list_command.executable, e);
                    }
                    continue;
                },
            };

            if exit_code != 0 {
                /* In case of "resource list" operation - print failure from provider as warning
                    In case of other resource/config operations:
                    print failure from provider as error because this provider was specifically requested by current resource/config operation*/
                if return_all_resources {
                    warn!("Provider failed to list resources with exit code {exit_code}: {stderr}");
                } else {
                    error!("Provider failed to list resources with exit code {exit_code}: {stderr}");
                }
            }

            for line in stdout.lines() {
                match serde_json::from_str::<DscResource>(line){
                    Result::Ok(resource) => {
                        if resource.requires.is_none() {
                            if return_all_resources {
                                warn!("{}", DscError::MissingRequires(provider.clone(), resource.type_name.clone()).to_string());
                            } else {
                                error!("{}", DscError::MissingRequires(provider.clone(), resource.type_name.clone()).to_string());
                            }
                            continue;
                        }
                        if return_all_resources
                        {
                            resources.insert(resource.type_name.to_lowercase(), resource);
                            provider_resources_count += 1;
                        }
                        else if remaining_required_resource_types.contains(&resource.type_name.to_lowercase())
                        {
                            remaining_required_resource_types.retain(|x| *x != resource.type_name.to_lowercase());
                            debug!("Found {} in {}", &resource.type_name, &resource.path);
                            resources.insert(resource.type_name.to_lowercase(), resource);
                            if remaining_required_resource_types.is_empty()
                            {
                                return Ok(resources);
                            }
                        }
                    },
                    Result::Err(err) => {
                        if return_all_resources {
                            warn!("Failed to parse resource: {line} -> {err}");
                        } else {
                            error!("Failed to parse resource: {line} -> {err}");
                        }
                        continue;
                    }
                };
            }

            debug!("Provider {} listed {} matching resources", provider_type_name, provider_resources_count);
        }

        Ok(resources)
    }
}

impl Default for CommandDiscovery {
    fn default() -> Self {
        Self::new()
    }
}

impl ResourceDiscovery for CommandDiscovery {

    fn list_available_resources(&mut self) -> Result<BTreeMap<String, DscResource>, DscError> {

        let required_resource_types = vec!["*".to_string()];
        CommandDiscovery::search_for_resources(&required_resource_types)
    }


    fn discover_resources(&mut self, required_resource_types: &[String]) -> Result<BTreeMap<String, DscResource>, DscError>
    {
        CommandDiscovery::search_for_resources(required_resource_types)
    }
}

fn load_manifest(path: &Path) -> Result<DscResource, DscError> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let manifest: ResourceManifest = if path.extension() == Some(OsStr::new("json")) {
        match serde_json::from_reader(reader) {
            Ok(manifest) => manifest,
            Err(err) => {
                return Err(DscError::Manifest(path.to_string_lossy().to_string(), err));
            }
        }
    }
    else {
        match serde_yaml::from_reader(reader) {
            Ok(manifest) => manifest,
            Err(err) => {
                return Err(DscError::ManifestYaml(path.to_string_lossy().to_string(), err));
            }
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
