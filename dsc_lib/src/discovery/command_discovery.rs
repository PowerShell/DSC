// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::discovery::discovery_trait::ResourceDiscovery;
use crate::discovery::convert_wildcard_to_regex;
use crate::dscresources::dscresource::{Capability, DscResource, ImplementedAs};
use crate::dscresources::resource_manifest::{import_manifest, validate_semver, Kind, ResourceManifest};
use crate::dscresources::command_resource::invoke_command;
use crate::dscresources::command_resource::log_resource_traces;
use crate::dscerror::DscError;
use indicatif::ProgressStyle;
use regex::RegexBuilder;
use semver::Version;
use std::collections::{BTreeMap, HashSet};
use std::env;
use std::ffi::OsStr;
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use tracing::{debug, info, trace, warn, warn_span};
use tracing_indicatif::span_ext::IndicatifSpanExt;

pub struct CommandDiscovery {
    // use BTreeMap so that the results are sorted
    resources: BTreeMap<String, Vec<DscResource>>,
    adapters: BTreeMap<String, Vec<DscResource>>,
    adapted_resources: BTreeMap<String, Vec<DscResource>>,
}

impl CommandDiscovery {
    pub fn new() -> CommandDiscovery {
        CommandDiscovery {
            resources: BTreeMap::new(),
            adapters: BTreeMap::new(),
            adapted_resources: BTreeMap::new(),
        }
    }

    fn get_resource_paths() -> Result<Vec<PathBuf>, DscError>
    {
        let mut using_custom_path = false;

        // try DSC_RESOURCE_PATH env var first otherwise use PATH
        let path_env = if let Some(value) = env::var_os("DSC_RESOURCE_PATH") {
            debug!("Using DSC_RESOURCE_PATH: {:?}", value.to_string_lossy());
            using_custom_path = true;
            value
        } else {
            trace!("DSC_RESOURCE_PATH not set, trying PATH");
            match env::var_os("PATH") {
                Some(value) => {
                    debug!("Using PATH: {:?}", value.to_string_lossy());
                    value
                },
                None => {
                    return Err(DscError::Operation("Failed to get PATH environment variable".to_string()));
                }
            }
        };

        let mut paths = env::split_paths(&path_env).collect::<Vec<_>>();

        // add exe home to start of path
        if !using_custom_path {
            if let Some(exe_home) = env::current_exe()?.parent() {
                debug!("Adding exe home to path: {}", exe_home.to_string_lossy());
                paths.insert(0, exe_home.to_path_buf());
            }
        }

        // remove duplicate entries to improve perf of resource search
        let mut uniques = HashSet::new();
        paths.retain(|e|uniques.insert((*e).clone()));

        Ok(paths)
    }
}

impl Default for CommandDiscovery {
    fn default() -> Self {
        Self::new()
    }
}

impl ResourceDiscovery for CommandDiscovery {

    fn discover_resources(&mut self, filter: &str) -> Result<(), DscError> {
        if !self.resources.is_empty() || !self.adapters.is_empty() {
            return Ok(());
        }

        info!("Discovering resources using filter: {filter}");

        let regex_str = convert_wildcard_to_regex(filter);
        debug!("Using regex {regex_str} as filter for adapter name");
        let mut regex_builder = RegexBuilder::new(&regex_str);
        regex_builder.case_insensitive(true);
        let Ok(regex) = regex_builder.build() else {
            return Err(DscError::Operation("Could not build Regex filter for adapter name".to_string()));
        };

        let pb_span = warn_span!("");
        pb_span.pb_set_style(&ProgressStyle::with_template(
            "{spinner:.green} [{elapsed_precise:.cyan}] [{bar:40.cyan/blue}] {pos:>7}/{len:7} {msg:.yellow}"
        )?);
        pb_span.pb_set_message("Searching for resources");
        let _ = pb_span.enter();

        let mut resources = BTreeMap::<String, Vec<DscResource>>::new();
        let mut adapters = BTreeMap::<String, Vec<DscResource>>::new();

        if let Ok(paths) = CommandDiscovery::get_resource_paths() {
            for path in paths {
                trace!("Searching in {:?}", path);
                if path.exists() && path.is_dir() {
                    for entry in path.read_dir().unwrap() {
                        let entry = entry.unwrap();
                        let path = entry.path();
                        if path.is_file() {
                            let Some(os_file_name) = path.file_name() else {
                                // skip if not a file
                                continue;
                            };
                            let Some(file_name) = os_file_name.to_str() else {
                                // skip if not a valid file name
                                continue;
                            };
                            let file_name_lowercase = file_name.to_lowercase();
                            if file_name_lowercase.ends_with(".dsc.resource.json") ||
                                file_name_lowercase.ends_with(".dsc.resource.yaml") ||
                                file_name_lowercase.ends_with(".dsc.resource.yml") {
                                trace!("Found resource manifest: {path:?}");
                                let resource = match load_manifest(&path)
                                {
                                    Ok(r) => r,
                                    Err(e) => {
                                        // At this point we can't determine whether or not the bad manifest contains
                                        // resource that is requested by resource/config operation
                                        // if it is, then "ResouceNotFound" error will be issued later
                                        // and here we just record the error into debug stream.
                                        debug!("{e}");
                                        continue;
                                    },
                                };

                                if regex.is_match(&resource.type_name) {
                                    if let Some(ref manifest) = resource.manifest {
                                        let manifest = import_manifest(manifest.clone())?;
                                        if manifest.kind == Some(Kind::Adapter) {
                                            trace!("Resource adapter {} found", resource.type_name);
                                            insert_resource(&mut adapters, &resource)?;
                                        } else {
                                            trace!("Resource {} found", resource.type_name);
                                            insert_resource(&mut resources, &resource)?;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        debug!("Found {} matching non-adapter-based resources", resources.len());
        self.resources = resources;
        self.adapters = adapters;
        Ok(())
    }

    fn discover_adapted_resources(&mut self, name_filter: &str, adapter_filter: &str) -> Result<(), DscError> {
        if self.resources.is_empty() && self.adapters.is_empty() {
            self.discover_resources("*")?;
        }

        if self.adapters.is_empty() {
            return Ok(());
        }

        let regex_str = convert_wildcard_to_regex(adapter_filter);
        debug!("Using regex {regex_str} as filter for adapter name");
        let mut regex_builder = RegexBuilder::new(&regex_str);
        regex_builder.case_insensitive(true);
        let Ok(regex) = regex_builder.build() else {
            return Err(DscError::Operation("Could not build Regex filter for adapter name".to_string()));
        };

        let name_regex_str = convert_wildcard_to_regex(name_filter);
        debug!("Using regex {name_regex_str} as filter for resource name");
        let mut name_regex_builder = RegexBuilder::new(&name_regex_str);
        name_regex_builder.case_insensitive(true);
        let Ok(name_regex) = name_regex_builder.build() else {
            return Err(DscError::Operation("Could not build Regex filter for resource name".to_string()));
        };

        let pb_span = warn_span!("");
        pb_span.pb_set_style(&ProgressStyle::with_template(
            "{spinner:.green} [{elapsed_precise:.cyan}] [{bar:40.cyan/blue}] {pos:>7}/{len:7} {msg:.yellow}"
        )?);
        pb_span.pb_set_message("Searching for adapted resources");
        let _ = pb_span.enter();

        let mut adapted_resources = BTreeMap::<String, Vec<DscResource>>::new();

        for (adapter_name, adapters) in &self.adapters {
            for adapter in adapters {
                if !regex.is_match(adapter_name) {
                    continue;
                }

                info!("Enumerating resources for adapter '{}'", adapter_name);
                let pb_adapter_span = warn_span!("");
                pb_adapter_span.pb_set_style(&ProgressStyle::with_template(
                    "{spinner:.green} [{elapsed_precise:.cyan}] {msg:.white}"
                )?);
                pb_adapter_span.pb_set_message(format!("Enumerating resources for adapter '{adapter_name}'").as_str());
                let _ = pb_adapter_span.enter();
                let manifest = if let Some(manifest) = &adapter.manifest {
                    if let Ok(manifest) = import_manifest(manifest.clone()) {
                        manifest
                    } else {
                        return Err(DscError::Operation(format!("Failed to import manifest for '{}'", adapter_name.clone())));
                    }
                } else {
                    return Err(DscError::MissingManifest(adapter_name.clone()));
                };

                let mut adapter_resources_count = 0;
                // invoke the list command
                let list_command = manifest.adapter.unwrap().list;
                let (exit_code, stdout, stderr) = match invoke_command(&list_command.executable, list_command.args, None, Some(&adapter.directory), None)
                {
                    Ok((exit_code, stdout, stderr)) => (exit_code, stdout, stderr),
                    Err(e) => {
                        // In case of error, log and continue
                        warn!("Could not start {}: {}", list_command.executable, e);
                        continue;
                    },
                };
                log_resource_traces(&stderr);

                if exit_code != 0 {
                    // in case of failure, log and continue
                    warn!("Adapter failed to list resources with exit code {exit_code}: {stderr}");
                    continue;
                }

                for line in stdout.lines() {
                    match serde_json::from_str::<DscResource>(line){
                        Result::Ok(resource) => {
                            if resource.require_adapter.is_none() {
                                warn!("{}", DscError::MissingRequires(adapter_name.clone(), resource.type_name.clone()).to_string());
                                continue;
                            }

                            if name_regex.is_match(&resource.type_name) {
                                insert_resource(&mut adapted_resources, &resource)?;
                                adapter_resources_count += 1;
                            }
                        },
                        Result::Err(err) => {
                            warn!("Failed to parse resource: {line} -> {err}");
                            continue;
                        }
                    };
                }

                debug!("Adapter '{}' listed {} resources", adapter_name, adapter_resources_count);
            }
        }

        self.adapted_resources = adapted_resources;
        Ok(())
    }

    fn list_available_resources(&mut self, type_name_filter: &str, adapter_name_filter: &str) -> Result<BTreeMap<String, Vec<DscResource>>, DscError> {

        trace!("Listing resources with type_name_filter/adapter_name_filter: {type_name_filter}/{adapter_name_filter}");

        self.discover_resources(type_name_filter)?;

        if !adapter_name_filter.is_empty() {
            self.discover_adapted_resources(type_name_filter, adapter_name_filter)?;
        }

        let mut resources = BTreeMap::<String, Vec<DscResource>>::new();

        if adapter_name_filter.is_empty() {
            resources.append(&mut self.resources);
            resources.append(&mut self.adapters);
        } else {
            resources.append(&mut self.adapted_resources);
        }

        Ok(resources)
    }

    // TODO: handle version requirements
    fn find_resources(&mut self, required_resource_types: &[String]) -> Result<BTreeMap<String, DscResource>, DscError>
    {
        debug!("Searching for resources: {:?}", required_resource_types);
        self.discover_resources("*")?;

        let mut found_resources = BTreeMap::<String, DscResource>::new();
        let mut remaining_required_resource_types = required_resource_types.to_owned();

        for (resource_name, resources) in &self.resources {
            let Some(resource ) = resources.first() else {
                // skip if no resources
                continue;
            };

            if remaining_required_resource_types.contains(&resource_name.to_lowercase())
            {
                // remove the resource from the list of required resources
                remaining_required_resource_types.retain(|x| *x != resource_name.to_lowercase());
                found_resources.insert(resource_name.to_lowercase(), resource.clone());
                if remaining_required_resource_types.is_empty()
                {
                    return Ok(found_resources);
                }
            }
        }
        debug!("Found {} matching non-adapter-based resources", found_resources.len());

        // now go through the adapter resources and add them to the list of resources
        for (adapted_name, adapted_resource) in &self.adapted_resources {
            let Some(adapted_resource) = adapted_resource.first() else {
                // skip if no resources
                continue;
            };

            if remaining_required_resource_types.contains(&adapted_name.to_lowercase())
            {
                remaining_required_resource_types.retain(|x| *x != adapted_name.to_lowercase());
                found_resources.insert(adapted_name.to_lowercase(), adapted_resource.clone());
                if remaining_required_resource_types.is_empty()
                {
                    return Ok(found_resources);
                }
            }
        }
        Ok(found_resources)
    }
}

// helper to insert a resource into a vector of resources in order of newest to oldest
fn insert_resource(resources: &mut BTreeMap<String, Vec<DscResource>>, resource: &DscResource) -> Result<(), DscError> {
    if resources.contains_key(&resource.type_name) {
        let Some(resource_versions) = resources.get_mut(&resource.type_name) else {
            resources.insert(resource.type_name.clone(), vec![resource.clone()]);
            return Ok(());
        };
        // compare the resource versions and insert newest to oldest using semver
        let mut insert_index = resource_versions.len();
        for (index, resource_instance) in resource_versions.iter().enumerate() {
            let resource_instance_version = Version::parse(&resource_instance.version)?;
            let resource_version = Version::parse(&resource.version)?;
            // if the version already exists, we skip
            if resource_instance_version == resource_version {
                return Ok(());
            }

            if resource_instance_version < resource_version {
                insert_index = index;
                break;
            }
        }
        resource_versions.insert(insert_index, resource.clone());
    } else {
        resources.insert(resource.type_name.clone(), vec![resource.clone()]);
    }
    Ok(())
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

    if let Err(err) = validate_semver(&manifest.version) {
        return Err(DscError::Validation(format!("Invalid manifest {path:?} version value: {err}")));
    }

    let kind = if let Some(kind) = manifest.kind.clone() {
        kind
    } else if manifest.adapter.is_some() {
        Kind::Adapter
    } else {
        Kind::Resource
    };

    // all command based resources are required to support `get`
    let mut capabilities = vec![Capability::Get];
    if let Some(set) = &manifest.set {
        capabilities.push(Capability::Set);
        if set.handles_exist == Some(true) {
            capabilities.push(Capability::SetHandlesExist);
        }
    }
    if manifest.test.is_some() {
        capabilities.push(Capability::Test);
    }
    if manifest.delete.is_some() {
        capabilities.push(Capability::Delete);
    }
    if manifest.export.is_some() {
        capabilities.push(Capability::Export);
    }

    let resource = DscResource {
        type_name: manifest.resource_type.clone(),
        kind,
        implemented_as: ImplementedAs::Command,
        description: manifest.description.clone(),
        version: manifest.version.clone(),
        capabilities,
        path: path.to_str().unwrap().to_string(),
        directory: path.parent().unwrap().to_str().unwrap().to_string(),
        manifest: Some(serde_json::to_value(manifest)?),
        ..Default::default()
    };

    Ok(resource)
}
