// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::{discovery::{discovery_trait::{DiscoveryFilter, DiscoveryKind, ResourceDiscovery}, matches_adapter_requirement}, dscresources::adapted_resource_manifest::AdaptedDscResourceManifest, parser::Statement};
use crate::{locked_clear, locked_is_empty, locked_extend, locked_clone, locked_get};
use crate::configure::{config_doc::ResourceDiscoveryMode, context::Context};
use crate::dscresources::dscresource::{Capability, DscResource, ImplementedAs};
use crate::dscresources::resource_manifest::{validate_semver, Kind, ResourceManifest, SchemaKind};
use crate::dscresources::command_resource::invoke_command;
use crate::dscerror::DscError;
use crate::extensions::dscextension::{self, DscExtension, Capability as ExtensionCapability};
use crate::extensions::extension_manifest::ExtensionManifest;
use crate::progress::{ProgressBar, ProgressFormat};
use crate::util::convert_wildcard_to_regex;
use crate::schemas::transforms::idiomaticize_externally_tagged_enum;
use regex::RegexBuilder;
use rust_i18n::t;
use semver::{Version, VersionReq};
use schemars::JsonSchema;
use serde::Deserialize;
use std::{collections::{BTreeMap, HashMap, HashSet}, sync::{LazyLock, RwLock}};
use std::env;
use std::ffi::OsStr;
use std::fs::{create_dir_all, read, read_to_string, write};
use std::path::{Path, PathBuf};
use std::str::FromStr;
use tracing::{debug, info, trace, warn};

use crate::util::get_setting;
use crate::util::{canonicalize_which, get_exe_path};

const DSC_ADAPTED_RESOURCE_EXTENSIONS: [&str; 3] = [".dsc.adaptedresource.json", ".dsc.adaptedresource.yaml", ".dsc.adaptedresource.yml"];
const DSC_EXTENSION_EXTENSIONS: [&str; 3] = [".dsc.extension.json", ".dsc.extension.yaml", ".dsc.extension.yml"];
const DSC_MANIFEST_LIST_EXTENSIONS: [&str; 3] = [".dsc.manifests.json", ".dsc.manifests.yaml", ".dsc.manifests.yml"];
const DSC_RESOURCE_EXTENSIONS: [&str; 3] = [".dsc.resource.json", ".dsc.resource.yaml", ".dsc.resource.yml"];

// use BTreeMap so that the results are sorted by the typename, the Vec is sorted by version
static ADAPTERS: LazyLock<RwLock<BTreeMap<String, Vec<DscResource>>>> = LazyLock::new(|| RwLock::new(BTreeMap::new()));
static RESOURCES: LazyLock<RwLock<BTreeMap<String, Vec<DscResource>>>> = LazyLock::new(|| RwLock::new(BTreeMap::new()));
static EXTENSIONS: LazyLock<RwLock<BTreeMap<String, DscExtension>>> = LazyLock::new(|| RwLock::new(BTreeMap::new()));
static ADAPTED_RESOURCES: LazyLock<RwLock<BTreeMap<String, Vec<DscResource>>>> = LazyLock::new(|| RwLock::new(BTreeMap::new()));

#[derive(Deserialize, JsonSchema)]
pub struct ManifestList {
    #[serde(rename = "adaptedResources")]
    pub adapted_resources: Option<Vec<AdaptedDscResourceManifest>>,
    pub resources: Option<Vec<ResourceManifest>>,
    pub extensions: Option<Vec<ExtensionManifest>>,
}

#[derive(Clone, Deserialize, JsonSchema)]
#[schemars(transform = idiomaticize_externally_tagged_enum)]
pub enum ImportedManifest {
    Resource(DscResource),
    Extension(DscExtension),
}


#[derive(Clone)]
pub struct CommandDiscovery {
    progress_format: ProgressFormat,
    discovery_mode: ResourceDiscoveryMode,
}

#[derive(Deserialize)]
pub struct ResourcePathSetting {
    /// whether to allow overriding with the `DSC_RESOURCE_PATH` environment variable
    #[serde(rename = "allowEnvOverride")]
    allow_env_override: bool,
    /// whether to append the PATH environment variable to the list of resource directories
    #[serde(rename = "appendEnvPath")]
    append_env_path: bool,
    /// array of directories that DSC should search for non-built-in resources
    directories: Vec<String>
}

impl Default for ResourcePathSetting {
    fn default() -> ResourcePathSetting {
        ResourcePathSetting {
            allow_env_override: true,
            append_env_path: true,
            directories: vec![],
        }
    }
}

impl CommandDiscovery {
    #[must_use]
    pub fn new(progress_format: ProgressFormat) -> CommandDiscovery {
        CommandDiscovery {
            progress_format,
            discovery_mode: ResourceDiscoveryMode::PreDeployment,
        }
    }

    #[must_use]
    pub fn get_extensions(&self) -> BTreeMap<String, DscExtension> { locked_clone!(EXTENSIONS) }

    fn get_resource_path_setting() -> Result<ResourcePathSetting, DscError>
    {
        if let Ok(v) = get_setting("resourcePath") {
            // if there is a policy value defined - use it; otherwise use setting value
            if v.policy != serde_json::Value::Null {
                match serde_json::from_value::<ResourcePathSetting>(v.policy) {
                    Ok(v) => {
                        return Ok(v);
                    },
                    Err(e) => { return Err(DscError::Setting(format!("{e}"))); }
                }
            } else if v.setting != serde_json::Value::Null {
                match serde_json::from_value::<ResourcePathSetting>(v.setting) {
                    Ok(v) => {
                        return Ok(v);
                    },
                    Err(e) => { return Err(DscError::Setting(format!("{e}"))); }
                }
            }
        }

        Err(DscError::Setting(t!("discovery.commandDiscovery.couldNotReadSetting").to_string()))
    }

    fn get_resource_paths() -> Result<Vec<PathBuf>, DscError>
    {
        let mut resource_path_setting = ResourcePathSetting::default();

        match Self::get_resource_path_setting() {
            Ok(v) => {
                resource_path_setting = v;
            },
            Err(e) => {
                debug!("{e}");
            }
        }

        let mut using_custom_path = false;
        let mut paths: Vec<PathBuf> = vec![];

        let dsc_resource_path = env::var_os("DSC_RESOURCE_PATH");
        if resource_path_setting.allow_env_override && dsc_resource_path.is_some() {
            if let Some(value) = dsc_resource_path {
                debug!("DSC_RESOURCE_PATH: {:?}", value.to_string_lossy());
                using_custom_path = true;
                paths.append(&mut env::split_paths(&value).collect::<Vec<_>>());
            }
        } else {
            for p in resource_path_setting.directories {
                let v = PathBuf::from_str(&p);
                paths.push(v.unwrap_or_default());
            }

            if resource_path_setting.append_env_path {
                debug!("{}", t!("discovery.commandDiscovery.appendingEnvPath"));
                match env::var_os("PATH") {
                    Some(value) => {
                        trace!("{}", t!("discovery.commandDiscovery.originalPath", path = value.to_string_lossy()));
                        paths.append(&mut env::split_paths(&value).collect::<Vec<_>>());
                    },
                    None => {
                        return Err(DscError::Operation(t!("discovery.commandDiscovery.failedGetEnvPath").to_string()));
                    }
                }
            }
        }

        // remove duplicate entries
        let mut uniques: HashSet<PathBuf> = HashSet::new();
        paths.retain(|e|uniques.insert((*e).clone()));

        if using_custom_path {
            // when using custom path, intent is to isolate the search of manifests and executables to the custom path
            // so we replace the PATH with the custom path
            if let Ok(new_path) = env::join_paths(paths.clone()) {
                env::set_var("PATH", new_path);
            } else {
                return Err(DscError::Operation(t!("discovery.commandDiscovery.failedJoinEnvPath").to_string()));
            }
        } else {
            // if exe home is not already in PATH env var then add it to env var and list of searched paths
            if let Some(exe_home) = get_exe_path()?.parent() {
                let exe_home_pb = exe_home.to_path_buf();
                if paths.contains(&exe_home_pb) {
                    trace!("{}", t!("discovery.commandDiscovery.exeHomeAlreadyInPath", path = exe_home.to_string_lossy()));
                } else {
                    trace!("{}", t!("discovery.commandDiscovery.addExeHomeToPath", path = exe_home.to_string_lossy()));
                    paths.push(exe_home_pb);

                    if let Ok(new_path) = env::join_paths(paths.clone()) {
                        env::set_var("PATH", new_path);
                    }
                }
            }
        }

        if let Ok(final_resource_path) = env::join_paths(paths.clone()) {
            debug!("{}", t!("discovery.commandDiscovery.usingResourcePath", path = final_resource_path.to_string_lossy()));
        }

        Ok(paths)
    }
}

impl Default for CommandDiscovery {
    fn default() -> Self {
        Self::new(ProgressFormat::Default)
    }
}

impl ResourceDiscovery for CommandDiscovery {

    #[allow(clippy::too_many_lines)]
    fn discover(&mut self, kind: &DiscoveryKind, filter: &str) -> Result<(), DscError> {
        if self.discovery_mode == ResourceDiscoveryMode::PreDeployment && !locked_is_empty!(RESOURCES) {
            return Ok(());
        } else if self.discovery_mode == ResourceDiscoveryMode::DuringDeployment {
            locked_clear!(RESOURCES);
            locked_clear!(ADAPTERS);
        }

        // if kind is DscResource, we need to discover extensions first
        if *kind == DiscoveryKind::Resource && (self.discovery_mode == ResourceDiscoveryMode::DuringDeployment || locked_is_empty!(EXTENSIONS)){
            self.discover(&DiscoveryKind::Extension, "*")?;
        }

        info!("{}", t!("discovery.commandDiscovery.discoverResources", kind = kind : {:?}, filter = filter));

        let regex_str = convert_wildcard_to_regex(filter);
        debug!("Using regex {regex_str} as filter for adapter name");
        let mut regex_builder = RegexBuilder::new(&regex_str);
        regex_builder.case_insensitive(true);
        let Ok(regex) = regex_builder.build() else {
            return Err(DscError::Operation(t!("discovery.commandDiscovery.invalidAdapterFilter").to_string()));
        };

        let mut progress = ProgressBar::new(1, self.progress_format)?;
        match kind {
            DiscoveryKind::Resource => {
                progress.write_activity(t!("discovery.commandDiscovery.progressSearching").to_string().as_str());
            },
            DiscoveryKind::Extension => {
                progress.write_activity(t!("discovery.commandDiscovery.extensionSearching").to_string().as_str());
            }
        }

        let mut adapters = BTreeMap::<String, Vec<DscResource>>::new();
        let mut resources = BTreeMap::<String, Vec<DscResource>>::new();
        let mut extensions = BTreeMap::<String, DscExtension>::new();

        if let Ok(paths) = CommandDiscovery::get_resource_paths() {
            for path in paths {
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
                            if DSC_MANIFEST_LIST_EXTENSIONS.iter().any(|ext| file_name_lowercase.ends_with(ext)) ||
                                (kind == &DiscoveryKind::Resource && (DSC_RESOURCE_EXTENSIONS.iter().any(|ext| file_name_lowercase.ends_with(ext))) || DSC_ADAPTED_RESOURCE_EXTENSIONS.iter().any(|ext| file_name_lowercase.ends_with(ext))) ||
                                (kind == &DiscoveryKind::Extension && DSC_EXTENSION_EXTENSIONS.iter().any(|ext| file_name_lowercase.ends_with(ext))) {
                                trace!("{}", t!("discovery.commandDiscovery.foundManifest", path = path.to_string_lossy()));
                                let imported_manifests = match load_manifest(&path)
                                {
                                    Ok(r) => r,
                                    Err(e) => {
                                        // At this point we can't determine whether or not the bad manifest contains
                                        // resource that is requested by resource/config operation
                                        // if it is, then "ResouceNotFound" error will be issued later
                                        // and here we just write as warning
                                        warn!("{e}");
                                        continue;
                                    },
                                };

                                for imported_manifest in imported_manifests {
                                    match imported_manifest {
                                        ImportedManifest::Extension(extension) => {
                                            if regex.is_match(&extension.type_name) {
                                                trace!("{}", t!("discovery.commandDiscovery.extensionFound", extension = extension.type_name, version = extension.version));
                                                // we only keep newest version of the extension so compare the version and only keep the newest
                                                if let Some(existing_extension) = extensions.get_mut(extension.type_name.as_ref()) {
                                                    if extension.version > existing_extension.version {
                                                        extensions.insert(extension.type_name.to_string(), extension.clone());
                                                    }
                                                } else {
                                                    extensions.insert(extension.type_name.to_string(), extension.clone());
                                                }
                                            }
                                        },
                                        ImportedManifest::Resource(resource) => {
                                            if regex.is_match(&resource.type_name) {
                                                if let Some(ref manifest) = &resource.manifest {
                                                    if manifest.kind == Some(Kind::Adapter) {
                                                        trace!("{}", t!("discovery.commandDiscovery.adapterFound", adapter = resource.type_name, version = resource.version));
                                                        insert_resource(&mut adapters, &resource);
                                                    }
                                                    // also make sure to add adapters as a resource as well
                                                    trace!("{}", t!("discovery.commandDiscovery.resourceFound", resource = resource.type_name, version = resource.version));
                                                    insert_resource(&mut resources, &resource);
                                                }
                                                if let Some(_adapter) = &resource.require_adapter {
                                                    trace!("{}", t!("discovery.commandDiscovery.adaptedResourceFound", resource = resource.type_name, version = resource.version));
                                                    insert_resource(&mut resources, &resource);
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        progress.write_increment(1);

        match kind {
            DiscoveryKind::Resource => {
                // Now we need to call discover extensions and add those resource to the list of resources
                for extension in locked_clone!(EXTENSIONS).values() {
                    if extension.capabilities.contains(&ExtensionCapability::Discover) {
                        debug!("{}", t!("discovery.commandDiscovery.callingExtension", extension = extension.type_name));
                        let discovered_resources = match extension.discover() {
                            Ok(res) => res,
                            Err(e) => {
                                warn!("{}", t!("discovery.commandDiscovery.extensionDiscoverFailed", extension = extension.type_name, error = e));
                                continue;
                            }
                        };
                        debug!("{}", t!("discovery.commandDiscovery.extensionFoundResources", extension = extension.type_name, count = discovered_resources.len()));
                        for resource in discovered_resources {
                            if regex.is_match(&resource.type_name) {
                                trace!("{}", t!("discovery.commandDiscovery.extensionResourceFound", resource = resource.type_name));
                                insert_resource(&mut resources, &resource);
                            }
                        }
                    }
                }
                locked_extend!(ADAPTERS, adapters);
                locked_extend!(RESOURCES, resources);
            },
            DiscoveryKind::Extension => {
                locked_extend!(EXTENSIONS, extensions);
            }
        }

        Ok(())
    }

    fn discover_adapted_resources(&mut self, name_filter: &str, adapter_filter: &str) -> Result<(), DscError> {
        if self.discovery_mode == ResourceDiscoveryMode::DuringDeployment || (locked_is_empty!(RESOURCES) && locked_is_empty!(ADAPTERS)) {
            self.discover(&DiscoveryKind::Resource, "*")?;
        }

        if locked_is_empty!(ADAPTERS) {
            return Ok(());
        }

        let adapters = locked_clone!(ADAPTERS);
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

        let mut progress = ProgressBar::new(adapters.len() as u64, self.progress_format)?;
        progress.write_activity("Searching for adapted resources");

        let mut adapted_resources = BTreeMap::<String, Vec<DscResource>>::new();

        let mut found_adapter: bool = false;
        for (adapter_name, adapters) in &adapters {
            for adapter in adapters {
                progress.write_increment(1);

                if !regex.is_match(adapter_name) {
                    continue;
                }

                found_adapter = true;
                let mut adapter_progress = ProgressBar::new(1, self.progress_format)?;
                adapter_progress.write_activity(format!("Enumerating resources for adapter '{adapter_name}'").as_str());
                let Some(manifest) = &adapter.manifest else {
                    return Err(DscError::MissingManifest(adapter_name.clone()));
                };

                let mut adapter_resources_count = 0;
                // invoke the list command
                let list_command = &manifest.adapter.clone().unwrap().list;
                let (exit_code, stdout, stderr) = match invoke_command(&list_command.executable, list_command.args.clone(), None, Some(&adapter.directory), None, manifest.exit_codes.as_ref())
                {
                    Ok((exit_code, stdout, stderr)) => (exit_code, stdout, stderr),
                    Err(e) => {
                        // In case of error, log and continue
                        warn!("{e}");
                        continue;
                    },
                };

                if exit_code != 0 {
                    // in case of failure, log and continue
                    warn!("Adapter failed to list resources with exit code {exit_code}: {stderr}");
                    continue;
                }

                for line in stdout.lines() {
                    match serde_json::from_str::<DscResource>(line){
                        Result::Ok(resource) => {
                            if resource.require_adapter.is_none() {
                                warn!("{}", DscError::MissingRequires(adapter_name.clone(), resource.type_name.to_string()).to_string());
                                continue;
                            }

                            if name_regex.is_match(&resource.type_name) {
                                insert_resource(&mut adapted_resources, &resource);
                                adapter_resources_count += 1;
                            }
                        },
                        Result::Err(err) => {
                            warn!("Failed to parse resource: {line} -> {err}");
                        }
                    }
                }

                adapter_progress.write_increment(1);
                debug!("Adapter '{}' listed {} resources", adapter_name, adapter_resources_count);
            }
        }

        if !found_adapter {
            return Err(DscError::AdapterNotFound(adapter_filter.to_string()));
        }

        locked_extend!(ADAPTED_RESOURCES, adapted_resources);

        Ok(())
    }

    fn list_available(&mut self, kind: &DiscoveryKind, type_name_filter: &str, adapter_name_filter: &str) -> Result<BTreeMap<String, Vec<ImportedManifest>>, DscError> {
        let mut resources = BTreeMap::<String, Vec<ImportedManifest>>::new();
        if *kind == DiscoveryKind::Resource {
            if adapter_name_filter.is_empty() {
                self.discover(kind, type_name_filter)?;
                for (resource_name, resources_vec) in &locked_clone!(RESOURCES) {
                    resources.insert(resource_name.clone(), resources_vec.iter().map(|r| ImportedManifest::Resource(r.clone())).collect());
                }
                for (adapter_name, adapter_vec) in &locked_clone!(ADAPTERS) {
                    resources.insert(adapter_name.clone(), adapter_vec.iter().map(|r| ImportedManifest::Resource(r.clone())).collect());
                }
            } else {
                self.discover(kind, "*")?;
                self.discover_adapted_resources(type_name_filter, adapter_name_filter)?;

                // add/update found adapted resources to the lookup_table
                let adapted_resources = locked_clone!(ADAPTED_RESOURCES);
                add_resources_to_lookup_table(&adapted_resources);

                for (adapted_name, adapted_vec) in &adapted_resources {
                    resources.insert(adapted_name.clone(), adapted_vec.iter().map(|r| ImportedManifest::Resource(r.clone())).collect());
                }
            }
        } else {
            self.discover(kind, type_name_filter)?;
            for (extension_name, extension) in &locked_clone!(EXTENSIONS) {
                resources.insert(extension_name.clone(), vec![ImportedManifest::Extension(extension.clone())]);
            }
        }

        Ok(resources)
    }

    fn find_resources(&mut self, required_resource_types: &[DiscoveryFilter]) -> Result<BTreeMap<String, Vec<DscResource>>, DscError> {
        debug!("{}", t!("discovery.commandDiscovery.searchingForResources", resources = required_resource_types : {:?}));
        if self.discovery_mode == ResourceDiscoveryMode::DuringDeployment || locked_is_empty!(RESOURCES) {
            self.discover(&DiscoveryKind::Resource, "*")?;
        }
        let mut found_resources = BTreeMap::<String, Vec<DscResource>>::new();
        let mut required_resources = HashMap::<DiscoveryFilter, bool>::new();
        for filter in required_resource_types {
            required_resources.insert(filter.clone(), false);
        }

        for filter in required_resource_types {
            if let Some(resources) = locked_get!(RESOURCES, filter.resource_type()) {
                filter_resources(&mut found_resources, &mut required_resources, &resources, filter);
            }
            if required_resources.values().all(|&v| v) {
                break;
            }
        }
        debug!("{}", t!("discovery.commandDiscovery.foundNonAdapterResources", count = found_resources.len()));

        if required_resources.values().all(|&v| v) {
            return Ok(found_resources);
        }

        // store the keys of the ADAPTERS into a vec
        let mut adapters: Vec<String> = locked_clone!(ADAPTERS).keys().cloned().collect();
        // sort the adapters by ones specified in the required resources first

        for filter in required_resource_types {
            if let Some(required_adapter) = filter.require_adapter() {
                if !adapters.contains(&required_adapter.to_string()) {
                    return Err(DscError::AdapterNotFound(required_adapter.to_string()));
                }
                // otherwise insert at the front of the list
                adapters.retain(|a| a != required_adapter);
                adapters.insert(0, required_adapter.to_string());
            }
        }

        for adapter_name in &adapters {
            self.discover_adapted_resources("*", adapter_name)?;
            add_resources_to_lookup_table(&locked_clone!(ADAPTED_RESOURCES));
            for filter in required_resource_types {
                if let Some(adapted_resources) = locked_get!(ADAPTED_RESOURCES, filter.resource_type()) {
                    filter_resources(&mut found_resources, &mut required_resources, &adapted_resources, filter);
                }
                if required_resources.values().all(|&v| v) {
                    break;
                }
            }
            if required_resources.values().all(|&v| v) {
                break;
            }
        }

        Ok(found_resources)
    }

    fn get_extensions(&mut self) -> Result<BTreeMap<String, DscExtension>, DscError> {
        if locked_is_empty!(EXTENSIONS) {
            self.discover(&DiscoveryKind::Extension, "*")?;
        }
        Ok(locked_clone!(EXTENSIONS))
    }

    fn set_discovery_mode(&mut self, mode: &ResourceDiscoveryMode) {
        self.discovery_mode = mode.clone();
    }
}

fn filter_resources(found_resources: &mut BTreeMap<String, Vec<DscResource>>, required_resources: &mut HashMap<DiscoveryFilter, bool>, resources: &[DscResource], filter: &DiscoveryFilter) {
    for resource in resources {
        if let Some(required_version) = filter.version() {
            if let Ok(resource_version) = Version::parse(&resource.version) {
                if let Ok(version_req) = VersionReq::parse(required_version) {
                    if version_req.matches(&resource_version) && matches_adapter_requirement(resource, filter) {
                        found_resources.entry(filter.resource_type().to_string()).or_default().push(resource.clone());
                        required_resources.insert(filter.clone(), true);
                        debug!("{}", t!("discovery.commandDiscovery.foundResourceWithVersion", resource = resource.type_name, version = resource.version));
                        break;
                    }
                }
            } else {
                // if not semver, we do a string comparison
                if resource.version == *required_version && matches_adapter_requirement(resource, filter) {
                    found_resources.entry(filter.resource_type().to_string()).or_default().push(resource.clone());
                    required_resources.insert(filter.clone(), true);
                    debug!("{}", t!("discovery.commandDiscovery.foundResourceWithVersion", resource = resource.type_name, version = resource.version));
                    break;
                }
            }
        } else {
            if matches_adapter_requirement(resource, filter) {
                found_resources.entry(filter.resource_type().to_string()).or_default().push(resource.clone());
                required_resources.insert(filter.clone(), true);
                break;
            }
        }
        if required_resources.values().all(|&v| v) {
            return;
        }
    }
}

/// Inserts a resource into tree adding to vector if already exists
fn insert_resource(resources: &mut BTreeMap<String, Vec<DscResource>>, resource: &DscResource) {
    if let Some(resource_versions) = resources.get_mut(&resource.type_name.to_lowercase()) {
        // compare the resource versions and insert newest to oldest using semver
        let mut insert_index = resource_versions.len();
        for (index, resource_instance) in resource_versions.iter().enumerate() {
            let resource_instance_version = match Version::parse(&resource_instance.version) {
                Ok(v) => v,
                Err(_err) => {
                    continue;
                },
            };
            let resource_version = match Version::parse(&resource.version) {
                Ok(v) => v,
                Err(_err) => {
                    continue;
                },
            };

            if resource_instance_version < resource_version {
                insert_index = index;
                break;
            }
        }
        resource_versions.insert(insert_index, resource.clone());
    } else {
        resources.insert(resource.type_name.to_lowercase(), vec![resource.clone()]);
    }
}

fn evaluate_condition(condition: Option<&str>) -> Result<bool, DscError> {
    if let Some(cond) = condition {
        let mut statement = Statement::new()?;
        let result = statement.parse_and_execute(cond, &Context::new())?;
        if let Some(bool_result) = result.as_bool() {
            return Ok(bool_result);
        }
        return Err(DscError::Validation(t!("discovery.commandDiscovery.conditionNotBoolean", condition = cond).to_string()));
    }
    Ok(true)
}

/// Loads a manifest from the given path and returns a vector of `ImportedManifest`.
///
/// # Arguments
///
/// * `path` - The path to the manifest file.
///
/// # Returns
///
/// * `Vec<ImportedManifest>` if the manifest was loaded successfully.
///
/// # Errors
///
/// * Returns a `DscError` if the manifest could not be loaded or parsed.
pub fn load_manifest(path: &Path) -> Result<Vec<ImportedManifest>, DscError> {
    let contents = read_to_string(path)?;
    let Some(file_name_lowercase) = path.file_name().and_then(OsStr::to_str).map(|s| s.to_lowercase()) else {
        return Err(DscError::InvalidManifest(t!("discovery.commandDiscovery.invalidManifestFileName", path = path.to_string_lossy()).to_string()));
    };
    let extension_is_json = path.extension().is_some_and(|ext| ext.eq_ignore_ascii_case("json"));
    if DSC_ADAPTED_RESOURCE_EXTENSIONS.iter().any(|ext| file_name_lowercase.ends_with(ext)) {
        let resource = if extension_is_json {
            match serde_json::from_str::<AdaptedDscResourceManifest>(&contents) {
                Ok(resource) => resource,
                Err(err) => {
                    return Err(DscError::InvalidManifest(t!("discovery.commandDiscovery.invalidAdaptedResourceManifest", resource = path.to_string_lossy(), err = err).to_string()));
                }
            }
        } else {
            match serde_yaml::from_str::<AdaptedDscResourceManifest>(&contents) {
                Ok(resource) => resource,
                Err(err) => {
                    return Err(DscError::InvalidManifest(t!("discovery.commandDiscovery.invalidAdaptedResourceManifest", resource = path.to_string_lossy(), err = err).to_string()));
                }
            }
        };
        if !evaluate_condition(resource.condition.as_deref())? {
            debug!("{}", t!("discovery.commandDiscovery.conditionNotMet", path = path.to_string_lossy(), condition = resource.condition.unwrap_or_default(), resource = resource.type_name));
            return Ok(vec![]);
        }
        let resource = load_adapted_resource_manifest(&path, &resource)?;
        return Ok(vec![ImportedManifest::Resource(resource)]);
    }
    if DSC_RESOURCE_EXTENSIONS.iter().any(|ext| file_name_lowercase.ends_with(ext)) {
        let manifest = if extension_is_json {
            match serde_json::from_str::<ResourceManifest>(&contents) {
                Ok(manifest) => manifest,
                Err(err) => {
                    return Err(DscError::InvalidManifest(t!("discovery.commandDiscovery.invalidResourceManifest", resource = path.to_string_lossy(), err = err).to_string()));
                }
            }
        } else {
            match serde_yaml::from_str::<ResourceManifest>(&contents) {
                Ok(manifest) => manifest,
                Err(err) => {
                    return Err(DscError::InvalidManifest(t!("discovery.commandDiscovery.invalidResourceManifest", resource = path.to_string_lossy(), err = err).to_string()));
                }
            }
        };
        if !evaluate_condition(manifest.condition.as_deref())? {
            debug!("{}", t!("discovery.commandDiscovery.conditionNotMet", path = path.to_string_lossy(), condition = manifest.condition.unwrap_or_default(), resource = manifest.resource_type));
            return Ok(vec![]);
        }
        let resource = load_resource_manifest(path, &manifest)?;
        return Ok(vec![ImportedManifest::Resource(resource)]);
    }
    if DSC_EXTENSION_EXTENSIONS.iter().any(|ext| file_name_lowercase.ends_with(ext)) {
        let manifest = if extension_is_json {
            match serde_json::from_str::<ExtensionManifest>(&contents) {
                Ok(manifest) => manifest,
                Err(err) => {
                    return Err(DscError::InvalidManifest(t!("discovery.commandDiscovery.invalidExtensionManifest", resource = path.to_string_lossy(), err = err).to_string()));
                }
            }
        } else {
            match serde_yaml::from_str::<ExtensionManifest>(&contents) {
                Ok(manifest) => manifest,
                Err(err) => {
                    return Err(DscError::InvalidManifest(t!("discovery.commandDiscovery.invalidExtensionManifest", resource = path.to_string_lossy(), err = err).to_string()));
                }
            }
        };
        if !evaluate_condition(manifest.condition.as_deref())? {
                debug!("{}", t!("discovery.commandDiscovery.conditionNotMet", path = path.to_string_lossy(), condition = manifest.condition.unwrap_or_default(), resource = manifest.r#type));
                return Ok(vec![]);
        }
        let extension = load_extension_manifest(path, &manifest)?;
        return Ok(vec![ImportedManifest::Extension(extension)]);
    }
    if DSC_MANIFEST_LIST_EXTENSIONS.iter().any(|ext| file_name_lowercase.ends_with(ext)) {
        let mut resources: Vec<ImportedManifest> = vec![];
        let manifest_list = if extension_is_json {
            match serde_json::from_str::<ManifestList>(&contents) {
                Ok(manifest) => manifest,
                Err(err) => {
                    return Err(DscError::InvalidManifest(t!("discovery.commandDiscovery.invalidManifestList", resource = path.to_string_lossy(), err = err).to_string()));
                }
            }
        } else {
            match serde_yaml::from_str::<ManifestList>(&contents) {
                Ok(manifest) => manifest,
                Err(err) => {
                    return Err(DscError::InvalidManifest(t!("discovery.commandDiscovery.invalidManifestList", resource = path.to_string_lossy(), err = err).to_string()));
                }
            }
        };
        if let Some(adapted_resources) = &manifest_list.adapted_resources {
            for resource in adapted_resources {
                if !evaluate_condition(resource.condition.as_deref())? {
                    debug!("{}", t!("discovery.commandDiscovery.conditionNotMet", path = path.to_string_lossy(), condition = resource.condition.as_ref() : {:?}, resource = resource.type_name));
                    continue;
                }
                let resource = load_adapted_resource_manifest(&path, resource)?;
                resources.push(ImportedManifest::Resource(resource));
            }
        }
        if let Some(resource_manifests) = &manifest_list.resources {
            for res_manifest in resource_manifests {
                if !evaluate_condition(res_manifest.condition.as_deref())? {
                    debug!("{}", t!("discovery.commandDiscovery.conditionNotMet", path = path.to_string_lossy(), condition = res_manifest.condition.as_ref() : {:?}, resource = res_manifest.resource_type));
                    continue;
                }
                let resource = load_resource_manifest(path, res_manifest)?;
                resources.push(ImportedManifest::Resource(resource));
            }
        }
        if let Some(extension_manifests) = &manifest_list.extensions {
            for ext_manifest in extension_manifests {
                if !evaluate_condition(ext_manifest.condition.as_deref())? {
                    debug!("{}", t!("discovery.commandDiscovery.conditionNotMet", path = path.to_string_lossy(), condition = ext_manifest.condition.as_ref() : {:?}, resource = ext_manifest.r#type));
                    continue;
                }
                let extension = load_extension_manifest(path, ext_manifest)?;
                resources.push(ImportedManifest::Extension(extension));
            }
        }
        return Ok(resources);
    }
    Err(DscError::InvalidManifest(t!("discovery.commandDiscovery.invalidManifestFile", resource = path.to_string_lossy()).to_string()))
}

fn load_adapted_resource_manifest(path: &Path, manifest: &AdaptedDscResourceManifest) -> Result<DscResource, DscError> {
    if let Err(err) = validate_semver(&manifest.version) {
        warn!("{}", t!("discovery.commandDiscovery.invalidManifestVersion", path = path.to_string_lossy(), err = err).to_string());
    }

    let directory = path.parent().unwrap();
    let resource_path = directory.join(&manifest.path);
    if !resource_path.exists() {
        return Err(DscError::InvalidManifest(t!("discovery.commandDiscovery.adaptedResourcePathNotFound", path = resource_path.to_string_lossy(), resource = manifest.type_name).to_string()));
    }

    let resource = DscResource {
        type_name: manifest.type_name.clone(),
        kind: Kind::Resource,
        implemented_as: None,
        description: manifest.description.clone(),
        version: manifest.version.clone(),
        capabilities: manifest.capabilities.clone(),
        require_adapter: Some(manifest.require_adapter.clone()),
        path: resource_path,
        directory: directory.to_path_buf(),
        manifest: None,
        schema: Some(manifest.schema.clone()),
        ..Default::default()
    };

    Ok(resource)
}

fn load_resource_manifest(path: &Path, manifest: &ResourceManifest) -> Result<DscResource, DscError> {
    if let Err(err) = validate_semver(&manifest.version) {
        warn!("{}", t!("discovery.commandDiscovery.invalidManifestVersion", path = path.to_string_lossy(), err = err).to_string());
    }

    let kind = if let Some(kind) = manifest.kind.clone() {
        kind
    } else if manifest.adapter.is_some() {
        Kind::Adapter
    } else {
        Kind::Resource
    };

    let mut capabilities: Vec<Capability> = vec![];
    if let Some(get) = &manifest.get {
        verify_executable(&manifest.resource_type, "get", &get.executable, path.parent().unwrap());
        capabilities.push(Capability::Get);
    }
    if let Some(set) = &manifest.set {
        verify_executable(&manifest.resource_type, "set", &set.executable, path.parent().unwrap());
        capabilities.push(Capability::Set);
        if set.handles_exist == Some(true) {
            capabilities.push(Capability::SetHandlesExist);
        }
    }
    if let Some(test) = &manifest.test {
        verify_executable(&manifest.resource_type, "test", &test.executable, path.parent().unwrap());
        capabilities.push(Capability::Test);
    }
    if let Some(delete) = &manifest.delete {
        verify_executable(&manifest.resource_type, "delete", &delete.executable, path.parent().unwrap());
        capabilities.push(Capability::Delete);
    }
    if let Some(export) = &manifest.export {
        verify_executable(&manifest.resource_type, "export", &export.executable, path.parent().unwrap());
        capabilities.push(Capability::Export);
    }
    if let Some(resolve) = &manifest.resolve {
        verify_executable(&manifest.resource_type, "resolve", &resolve.executable, path.parent().unwrap());
        capabilities.push(Capability::Resolve);
    }
    if let Some(SchemaKind::Command(command)) = &manifest.schema {
        verify_executable(&manifest.resource_type, "schema", &command.executable, path.parent().unwrap());
    }

    let resource = DscResource {
        type_name: manifest.resource_type.clone(),
        kind,
        implemented_as: Some(ImplementedAs::Command),
        description: manifest.description.clone(),
        version: manifest.version.clone(),
        capabilities,
        path: path.to_path_buf(),
        directory: path.parent().unwrap().to_path_buf(),
        manifest: Some(manifest.clone()),
        ..Default::default()
    };

    Ok(resource)
}

fn load_extension_manifest(path: &Path, manifest: &ExtensionManifest) -> Result<DscExtension, DscError> {
    let mut capabilities: Vec<dscextension::Capability> = vec![];
    if let Some(discover) = &manifest.discover {
        verify_executable(&manifest.r#type, "discover", &discover.executable, path.parent().unwrap());
        capabilities.push(dscextension::Capability::Discover);
    }
    if let Some(secret) = &manifest.secret {
        verify_executable(&manifest.r#type, "secret", &secret.executable, path.parent().unwrap());
        capabilities.push(dscextension::Capability::Secret);
    }
    let import = if let Some(import) = &manifest.import {
        verify_executable(&manifest.r#type, "import", &import.executable, path.parent().unwrap());
        capabilities.push(dscextension::Capability::Import);
        if import.file_extensions.is_empty() {
            warn!("{}", t!("discovery.commandDiscovery.importExtensionsEmpty", extension = manifest.r#type));
            None
        } else {
            Some(import.clone())
        }
    } else {
        None
    };

    let extension = DscExtension {
        type_name: manifest.r#type.clone(),
        description: manifest.description.clone(),
        version: manifest.version.clone(),
        capabilities,
        import,
        path: path.to_path_buf(),
        directory: path.parent().unwrap().to_path_buf(),
        manifest: serde_json::to_value(manifest)?,
        ..Default::default()
    };

    Ok(extension)
}

fn verify_executable(resource: &str, operation: &str, executable: &str, directory: &Path) {
    if canonicalize_which(executable, Some(directory)).is_err() {
        info!("{}", t!("discovery.commandDiscovery.executableNotFound", resource = resource, operation = operation, executable = executable));
    }
}

fn add_resources_to_lookup_table(adapted_resources: &BTreeMap<String, Vec<DscResource>>)
{
    let mut lookup_table = load_adapted_resources_lookup_table();

    let mut lookup_table_changed = false;
    for (resource_name, res_vec) in adapted_resources {
        if let Some(adapter_name) = &res_vec[0].require_adapter {
            let new_value = adapter_name.to_string();
            let oldvalue = lookup_table.insert(resource_name.to_string().to_lowercase(), new_value.clone());
            if !lookup_table_changed && (oldvalue.is_none() || oldvalue.is_some_and(|val| val != new_value)) {
                lookup_table_changed = true;
            }
        } else {
            debug!("{}", t!("discovery.commandDiscovery.resourceMissingRequireAdapter", resource = resource_name));
        }
    }
    if lookup_table_changed {
        save_adapted_resources_lookup_table(&lookup_table);
    }
}

fn save_adapted_resources_lookup_table(lookup_table: &HashMap<String, String>)
{
    if let Ok(lookup_table_json) = serde_json::to_string(&lookup_table) {
        let file_path = get_lookup_table_file_path();
        debug!("Saving lookup table with {} items to {:?}", lookup_table.len(), file_path);

        let path = std::path::Path::new(&file_path);
        if let Some(prefix) = path.parent() {
            if create_dir_all(prefix).is_ok()  {
                if write(file_path.clone(), lookup_table_json).is_err() {
                    info!("Unable to write lookup_table file {file_path:?}");
                }
            } else {
                info!("Unable to create parent directories of the lookup_table file {file_path:?}");
            }
        } else {
            info!("Unable to get directory of the lookup_table file {file_path:?}");
        }
    } else {
        info!("Unable to serialize lookup_table to json");
    }
}

fn load_adapted_resources_lookup_table() -> HashMap<String, String>
{
    let file_path = get_lookup_table_file_path();

    let lookup_table: HashMap<String, String> = match read(file_path.clone()){
        Ok(data) => { serde_json::from_slice(&data).unwrap_or_default() },
        Err(_) => { HashMap::new() }
    };

    debug!("Read {} items into lookup table from {:?}", lookup_table.len(), file_path);
    lookup_table
}

#[cfg(target_os = "windows")]
fn get_lookup_table_file_path() -> String
{
    // $env:LocalAppData+"dsc\AdaptedResourcesLookupTable.json"
    let Ok(local_app_data_path) = std::env::var("LocalAppData") else { return String::new(); };

    Path::new(&local_app_data_path).join("dsc").join("AdaptedResourcesLookupTable.json").display().to_string()
}

#[cfg(not(target_os = "windows"))]
fn get_lookup_table_file_path() -> String
{
    // $env:HOME+".dsc/AdaptedResourcesLookupTable.json"
    let Ok(home_path) = std::env::var("HOME") else { return String::new(); };
    Path::new(&home_path).join(".dsc").join("AdaptedResourcesLookupTable.json").display().to_string()
}
