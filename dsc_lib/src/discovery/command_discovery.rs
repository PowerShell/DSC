// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::discovery::discovery_trait::ResourceDiscovery;
use crate::discovery::convert_wildcard_to_regex;
use crate::dscresources::dscresource::{Capability, DscResource, ImplementedAs};
use crate::dscresources::resource_manifest::{import_manifest, Kind, ResourceManifest};
use crate::dscresources::command_resource::invoke_command;
use crate::dscresources::command_resource::log_resource_traces;
use crate::dscerror::DscError;
use indicatif::ProgressStyle;
use regex::RegexBuilder;
use std::collections::{BTreeMap, HashSet};
use std::env;
use std::ffi::OsStr;
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use tracing::{debug, error, trace, warn, warn_span, Span};
use tracing_indicatif::span_ext::IndicatifSpanExt;

pub struct CommandDiscovery {
}

impl CommandDiscovery {
    pub fn new() -> CommandDiscovery {
        CommandDiscovery {
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

    #[allow(clippy::too_many_lines)]
    fn list_available_resources(&mut self, type_name_filter: &str, adapter_name_filter: &str) -> Result<BTreeMap<String, DscResource>, DscError> {

        debug!("Listing resources with type_name_filter/adapter_name_filter: {type_name_filter}/{adapter_name_filter}");

        let pb_span = warn_span!("");
        pb_span.pb_set_style(&ProgressStyle::with_template(
            "{spinner:.green} [{elapsed_precise:.cyan}] {msg:.yellow}"
        )?);
        pb_span.pb_set_message("Searching for resources");
        let _ = pb_span.enter();

        let mut resources: BTreeMap<String, DscResource> = BTreeMap::new();
        let mut adapter_resources: BTreeMap<String, DscResource> = BTreeMap::new();

        let regex_str = convert_wildcard_to_regex(type_name_filter);
        debug!("Using regex {regex_str} as filter for resource type");
        let mut regex_builder = RegexBuilder::new(&regex_str);
        regex_builder.case_insensitive(true);
        let Ok(type_regex) = regex_builder.build() else {
            let err_str = "Could not build Regex filter for resource type";
            error!(err_str);
            return Err(DscError::Operation(err_str.to_string()));
        };

        if let Ok(paths) = CommandDiscovery::get_resource_paths() {
            for path in paths {
                debug!("Searching in {:?}", path);
                if path.exists() && path.is_dir() {
                    for entry in path.read_dir().unwrap() {
                        let entry = entry.unwrap();
                        let path = entry.path();
                        if path.is_file() {
                            let file_name = path.file_name().unwrap().to_str().unwrap();
                            let file_name_lowercase = file_name.to_lowercase();
                            if file_name_lowercase.ends_with(".dsc.resource.json") ||
                            file_name_lowercase.ends_with(".dsc.resource.yaml") ||
                            file_name_lowercase.ends_with(".dsc.resource.yml") {
                                let resource = match load_manifest(&path)
                                {
                                    Ok(r) => r,
                                    Err(e) => {
                                        // In case of "resource list" operation - print all failures to read manifests as warnings
                                        warn!("{}", e);
                                        continue;
                                    },
                                };

                                if let Some(ref manifest) = resource.manifest {
                                    let manifest = import_manifest(manifest.clone())?;
                                    if manifest.kind == Some(Kind::Adapter) {
                                        adapter_resources.insert(resource.type_name.to_lowercase(),resource.clone());
                                    }
                                }

                                if adapter_name_filter.is_empty() && type_regex.is_match(&resource.type_name) {
                                    resources.insert(resource.type_name.to_lowercase(), resource);
                                }
                            }
                        }
                    }
                }
            }
        }
        debug!("Found {} matching non-adapter-based resources", resources.len());

        if !adapter_name_filter.is_empty() {
            let regex_str = convert_wildcard_to_regex(adapter_name_filter);
            debug!("Using regex {regex_str} as filter for adapter name");
            let mut regex_builder = RegexBuilder::new(&regex_str);
            regex_builder.case_insensitive(true);
            let Ok(adapter_regex) = regex_builder.build() else {
                let err_str = "Could not build Regex filter for adapter name";
                error!(err_str);
                return Err(DscError::Operation(err_str.to_string()));
            };

            // now go through the adapter resources and add them to the list of resources
            for adapter in adapter_resources {
                if adapter_regex.is_match(&adapter.1.type_name) {
                    debug!("Enumerating resources for adapter '{}'", adapter.1.type_name);
                    let pb_adapter_span = warn_span!("");
                    pb_adapter_span.pb_set_style(&ProgressStyle::with_template(
                        "{spinner:.green} [{elapsed_precise:.cyan}] {msg:.white}"
                    )?);
                    pb_adapter_span.pb_set_message(format!("Enumerating resources for adapter '{}'", adapter.1.type_name).as_str());
                    let _ = pb_adapter_span.enter();
                    let adapter_resource = adapter.1;
                    let adapter_type_name = adapter_resource.type_name.clone();
                    let manifest = if let Some(manifest) = adapter_resource.manifest {
                        if let Ok(manifest) = import_manifest(manifest) {
                            manifest
                        } else {
                            return Err(DscError::Operation(format!("Failed to import manifest for '{}'", adapter_resource.type_name.clone())));
                        }
                    } else {
                        return Err(DscError::MissingManifest(adapter_resource.type_name.clone()));
                    };
                    let mut adapter_resources_count = 0;
                    // invoke the list command
                    let list_command = manifest.adapter.unwrap().list;
                    let (exit_code, stdout, stderr) = match invoke_command(&list_command.executable, list_command.args, None, Some(&adapter_resource.directory), None)
                    {
                        Ok((exit_code, stdout, stderr)) => (exit_code, stdout, stderr),
                        Err(e) => {
                            // In case of "resource list" operation - print failure from adapter as warning
                            warn!("Could not start {}: {}", list_command.executable, e);
                            continue;
                        },
                    };
                    log_resource_traces(&stderr);

                    if exit_code != 0 {
                        // In case of "resource list" operation - print failure from adapter as warning
                        warn!("Adapter failed to list resources with exit code {exit_code}: {stderr}");
                    }

                    for line in stdout.lines() {
                        match serde_json::from_str::<DscResource>(line){
                            Result::Ok(resource) => {
                                if resource.require_adapter.is_none() {
                                    warn!("{}", DscError::MissingRequires(adapter.0.clone(), resource.type_name.clone()).to_string());
                                    continue;
                                }
                                if type_regex.is_match(&resource.type_name) {
                                    resources.insert(resource.type_name.to_lowercase(), resource);
                                    adapter_resources_count += 1;
                                }
                            },
                            Result::Err(err) => {
                                warn!("Failed to parse resource: {line} -> {err}");
                                continue;
                            }
                        };
                    }

                    debug!("Adapter '{}' listed {} matching resources", adapter_type_name, adapter_resources_count);
                }
            }
        }
        Ok(resources)
    }

    #[allow(clippy::too_many_lines)]
    fn discover_resources(&mut self, required_resource_types: &[String]) -> Result<BTreeMap<String, DscResource>, DscError>
    {
        debug!("Searching for resources: {:?}", required_resource_types);

        let pb_span = warn_span!("");
        pb_span.pb_set_style(&ProgressStyle::with_template(
            "{spinner:.green} [{elapsed_precise:.cyan}] [{bar:40.cyan/blue}] {pos:>7}/{len:7} {msg:.yellow}"
        )?);
        pb_span.pb_set_message("Searching for resources");
        let _ = pb_span.enter();

        let mut resources: BTreeMap<String, DscResource> = BTreeMap::new();
        let mut adapter_resources: BTreeMap<String, DscResource> = BTreeMap::new();
        let mut remaining_required_resource_types = required_resource_types.to_owned();

        if let Ok(paths) = CommandDiscovery::get_resource_paths() {
            for path in paths {
                debug!("Searching in {:?}", path);
                if path.exists() && path.is_dir() {
                    for entry in path.read_dir().unwrap() {
                        let entry = entry.unwrap();
                        let path = entry.path();
                        if path.is_file() {
                            let file_name = path.file_name().unwrap().to_str().unwrap();
                            let file_name_lowercase = file_name.to_lowercase();
                            if file_name_lowercase.ends_with(".dsc.resource.json") ||
                            file_name_lowercase.ends_with(".dsc.resource.yaml") ||
                            file_name_lowercase.ends_with(".dsc.resource.yml") {
                                let resource = match load_manifest(&path)
                                {
                                    Ok(r) => r,
                                    Err(e) => {
                                        /*  In case of non-list resource/config operations:
                                            At this point we can't determine whether or not the bad manifest contains resource that is requested by resource/config operation
                                            if it is, then "ResouceNotFound" error will be issued later
                                            and here we just record the error into debug stream.*/
                                        debug!("{}", e);
                                        continue;
                                    },
                                };

                                if let Some(ref manifest) = resource.manifest {
                                    let manifest = import_manifest(manifest.clone())?;
                                    if manifest.kind == Some(Kind::Adapter) {
                                        adapter_resources.insert(resource.type_name.to_lowercase(), resource.clone());
                                        resources.insert(resource.type_name.to_lowercase(), resource.clone());
                                    }
                                }
                                if remaining_required_resource_types.contains(&resource.type_name.to_lowercase())
                                {
                                    remaining_required_resource_types.retain(|x| *x != resource.type_name.to_lowercase());
                                    debug!("Found {} in {}", &resource.type_name, path.display());
                                    Span::current().pb_inc(1);
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
        }
        debug!("Found {} matching non-adapter-based resources", resources.len());

        // now go through the adapter resources and add them to the list of resources
        for adapter in adapter_resources {
            debug!("Enumerating resources for adapter '{}'", adapter.1.type_name);
            let pb_adapter_span = warn_span!("");
            pb_adapter_span.pb_set_style(&ProgressStyle::with_template(
                "{spinner:.green} [{elapsed_precise:.cyan}] {msg:.white}"
            )?);
            pb_adapter_span.pb_set_message(format!("Enumerating resources for adapter '{}'", adapter.1.type_name).as_str());
            let _ = pb_adapter_span.enter();
            let adapter_resource = adapter.1;
            let adapter_type_name = adapter_resource.type_name.clone();
            let manifest = if let Some(manifest) = adapter_resource.manifest {
                if let Ok(manifest) = import_manifest(manifest) {
                    manifest
                } else {
                    return Err(DscError::Operation(format!("Failed to import manifest for '{}'", adapter_resource.type_name.clone())));
                }
            } else {
                return Err(DscError::MissingManifest(adapter_resource.type_name.clone()));
            };
            let mut adapter_resources_count = 0;
            // invoke the list command
            let list_command = manifest.adapter.unwrap().list;
            let (exit_code, stdout, stderr) = match invoke_command(&list_command.executable, list_command.args, None, Some(&adapter_resource.directory), None)
            {
                Ok((exit_code, stdout, stderr)) => (exit_code, stdout, stderr),
                Err(e) => {
                    /* In case of non-list resource/config operations:
                       print failure from adapter as error because this adapter was specifically requested by current resource/config operation*/
                    error!("Could not start {}: {}", list_command.executable, e);
                    continue;
                },
            };
            log_resource_traces(&stderr);

            if exit_code != 0 {
                 /* In case of non-list resource/config operations:
                    print failure from adapter as error because this adapter was specifically requested by current resource/config operation*/
                error!("Adapter failed to list resources with exit code {exit_code}: {stderr}");
            }

            for line in stdout.lines() {
                match serde_json::from_str::<DscResource>(line){
                    Result::Ok(resource) => {
                        if resource.require_adapter.is_none() {
                            error!("{}", DscError::MissingRequires(adapter.0.clone(), resource.type_name.clone()).to_string());
                            continue;
                        }
                        if remaining_required_resource_types.contains(&resource.type_name.to_lowercase())
                        {
                            remaining_required_resource_types.retain(|x| *x != resource.type_name.to_lowercase());
                            debug!("Found {} in {}", &resource.type_name, &resource.path);
                            resources.insert(resource.type_name.to_lowercase(), resource);
                            adapter_resources_count += 1;
                            if remaining_required_resource_types.is_empty()
                            {
                                return Ok(resources);
                            }
                        }
                    },
                    Result::Err(err) => {
                        error!("Failed to parse resource: {line} -> {err}");
                        continue;
                    }
                };
            }

            debug!("Adapter '{}' listed {} matching resources", adapter_type_name, adapter_resources_count);
        }
        Ok(resources)
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

    let kind = if let Some(kind) = manifest.kind.clone() {
        kind
    } else if manifest.adapter.is_some() {
        Kind::Adapter
    } else {
        Kind::Resource
    };

    // all command based resources are required to support `get`
    let mut capabilities = vec![Capability::Get];
    if manifest.set.is_some() {
        capabilities.push(Capability::Set);
    }
    if manifest.test.is_some() {
        capabilities.push(Capability::Test);
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
