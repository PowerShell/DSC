// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use rust_i18n::t;
use serde_json::Value;
use std::collections::HashSet;
use std::ffi::OsStr;
use std::fs::read_to_string;
use std::path::{Path, PathBuf};
use tracing::{debug, trace, warn};

use crate::discovery::command_discovery::{
    CommandDiscovery,
    DSC_MANIFEST_LIST_EXTENSIONS,
    evaluate_condition,
};
use crate::dscerror::DscError;
use crate::plugins::command_name::PluginCommandName;
use crate::plugins::dscplugin::DscPlugin;
use crate::plugins::plugin_manifest::PluginManifest;
use crate::settings::global as global_settings;

/// The file name suffixes that identify a plug-in manifest.
pub const DSC_PLUGIN_EXTENSIONS: [&str; 3] = [".dsc.plugin.json", ".dsc.plugin.yaml", ".dsc.plugin.yml"];

/// Discovers the command plug-ins that are available to a DSC host.
///
/// DSC searches the same directories it searches for resource and extension manifests, in the
/// same order, and applies the rules that [`discover_plugins_in()`] describes.
///
/// When the `plugins.enabled` setting is `false`, the function doesn't search and returns an empty
/// list.
///
/// # Arguments
///
/// * `reserved_commands` - The names and aliases of the commands that are built into the host.
///
/// # Returns
///
/// The discovered plug-ins in the order DSC found them, including shadowed plug-ins.
///
/// # Errors
///
/// Returns a [`DscError`] when DSC can't resolve the directories to search, for example when the
/// `resourcePath.restricted` setting is `true` and `resourcePath.directories` is empty.
pub fn discover_plugins<S: AsRef<str>>(reserved_commands: &[S]) -> Result<Vec<DscPlugin>, DscError> {
    if !global_settings::get_or_init().resolved().plugins.enabled.value {
        debug!("{}", t!("plugins.discovery.disabled", setting = "plugins.enabled"));
        return Ok(Vec::new());
    }

    let directories = CommandDiscovery::get_resource_paths()?;
    Ok(discover_plugins_in(&directories, reserved_commands))
}

/// Discovers the command plug-ins defined in a list of directories.
///
/// DSC reads the files whose name ends with one of the [`DSC_PLUGIN_EXTENSIONS`] and the `plugins`
/// property of manifest lists. Subdirectories aren't searched. Discovery never runs a plug-in and
/// never asks a discovery extension for manifests.
///
/// The function applies these rules to what it finds:
///
/// 1. A manifest that fails to load or validate is skipped with a warning.
/// 1. A manifest whose `condition` evaluates to `false` is skipped.
/// 1. A manifest whose command name is one of the `reserved_commands` is skipped with a warning,
///    because the host matches its own commands first and the plug-in could never run.
/// 1. When the same type is found more than once, the highest version is kept.
/// 1. When plug-ins of different types use the same command name, the one found first provides
///    the command and the others are marked as [`shadowed`]. Within one directory, the type name
///    that sorts first is found first.
///
/// Unlike [`discover_plugins()`], this function doesn't consult the DSC settings.
///
/// # Arguments
///
/// * `directories` - The directories to search, in the order to search them.
/// * `reserved_commands` - The names and aliases of the commands that are built into the host.
///
/// # Returns
///
/// The discovered plug-ins in the order DSC found them, including shadowed plug-ins.
///
/// [`shadowed`]: DscPlugin::shadowed
pub fn discover_plugins_in<S: AsRef<str>>(directories: &[PathBuf], reserved_commands: &[S]) -> Vec<DscPlugin> {
    let mut plugins: Vec<DscPlugin> = Vec::new();
    for directory in directories {
        let mut found = load_plugins_in_directory(directory);
        found.sort_by(|a, b| a.type_name.cmp(&b.type_name));

        for plugin in found {
            if reserved_commands.iter().any(|reserved| &*plugin.command == reserved.as_ref()) {
                warn!("{}", t!(
                    "plugins.discovery.reservedCommand",
                    plugin = plugin.type_name,
                    path = plugin.path.display(),
                    command = plugin.command
                ));
                continue;
            }

            if let Some(existing) = plugins.iter_mut().find(|p| p.type_name == plugin.type_name) {
                if plugin.version > existing.version {
                    trace!("{}", t!(
                        "plugins.discovery.newerVersionFound",
                        plugin = plugin.type_name,
                        version = plugin.version,
                        path = plugin.path.display()
                    ));
                    *existing = plugin;
                }
                continue;
            }

            trace!("{}", t!(
                "plugins.discovery.pluginFound",
                plugin = plugin.type_name,
                version = plugin.version,
                command = plugin.command
            ));
            plugins.push(plugin);
        }
    }

    let mut provided: HashSet<PluginCommandName> = HashSet::new();
    for plugin in &mut plugins {
        plugin.shadowed = !provided.insert(plugin.command.clone());
        if plugin.shadowed {
            debug!("{}", t!(
                "plugins.discovery.pluginShadowed",
                plugin = plugin.type_name,
                command = plugin.command
            ));
        }
    }

    plugins
}

/// Finds the plug-in that provides a command.
///
/// # Arguments
///
/// * `plugins` - The plug-ins that [`discover_plugins()`] returned.
/// * `command` - The command name to look up. The comparison is case-sensitive.
///
/// # Returns
///
/// The plug-in that provides the command, or [`None`] when no plug-in does. A shadowed plug-in is
/// never returned.
#[must_use]
pub fn find_plugin<'a>(plugins: &'a [DscPlugin], command: &str) -> Option<&'a DscPlugin> {
    plugins.iter().find(|plugin| !plugin.shadowed && &*plugin.command == command)
}

/// Finds a plug-in by its fully qualified type name.
///
/// Unlike [`find_plugin()`], this function also returns a shadowed plug-in, which is how a user
/// runs a plug-in that doesn't provide its command.
///
/// # Arguments
///
/// * `plugins` - The plug-ins that [`discover_plugins()`] returned.
/// * `type_name` - The type name to look up, like `OpenDSC/Lint`. The comparison is
///   case-insensitive.
///
/// # Returns
///
/// The plug-in with the type name, or [`None`] when no plug-in has it.
#[must_use]
pub fn find_plugin_by_type<'a>(plugins: &'a [DscPlugin], type_name: &str) -> Option<&'a DscPlugin> {
    plugins.iter().find(|plugin| plugin.type_name == type_name)
}

/// Loads every plug-in defined in a directory.
///
/// Manifests that can't be loaded, don't validate, or whose condition isn't met are skipped. A
/// directory that can't be read yields no plug-ins.
///
/// # Arguments
///
/// * `directory` - The directory to search. Subdirectories aren't searched.
fn load_plugins_in_directory(directory: &Path) -> Vec<DscPlugin> {
    let Ok(entries) = directory.read_dir() else {
        trace!("{}", t!("plugins.discovery.directoryNotReadable", path = directory.display()));
        return Vec::new();
    };

    // Sort the candidates so the result doesn't depend on the order the file system lists them.
    let mut candidates: Vec<(PathBuf, bool)> = entries
        .filter_map(Result::ok)
        .filter_map(|entry| {
            let path = entry.path();
            let file_name = path.file_name().and_then(OsStr::to_str)?.to_lowercase();
            if DSC_PLUGIN_EXTENSIONS.iter().any(|ext| file_name.ends_with(ext)) {
                Some((path, false))
            } else if DSC_MANIFEST_LIST_EXTENSIONS.iter().any(|ext| file_name.ends_with(ext)) {
                Some((path, true))
            } else {
                None
            }
        })
        .collect();
    candidates.sort();

    let mut plugins = Vec::new();
    for (path, is_manifest_list) in candidates {
        let manifests = if is_manifest_list {
            load_manifests_from_list(&path)
        } else {
            match load_manifest_file(&path) {
                Ok(manifest) => vec![manifest],
                Err(err) => {
                    warn!("{}", t!("plugins.discovery.failedLoadManifest", path = path.display(), err = err));
                    continue;
                }
            }
        };

        for manifest in manifests {
            match load_plugin(&path, manifest) {
                Ok(Some(plugin)) => plugins.push(plugin),
                Ok(None) => {},
                Err(err) => {
                    warn!("{}", t!("plugins.discovery.failedLoadManifest", path = path.display(), err = err));
                }
            }
        }
    }

    plugins
}

/// Reads a plug-in manifest from a file.
///
/// # Arguments
///
/// * `path` - The path of a file whose name ends with one of the [`DSC_PLUGIN_EXTENSIONS`].
///
/// # Errors
///
/// Returns a [`DscError`] when the file can't be read or its content isn't a valid plug-in
/// manifest.
fn load_manifest_file(path: &Path) -> Result<PluginManifest, DscError> {
    let contents = read_to_string(path)?;
    if is_json_file(path) {
        Ok(serde_json::from_str::<PluginManifest>(&contents)?)
    } else {
        Ok(serde_yaml::from_str::<PluginManifest>(&contents)?)
    }
}

/// Reads the plug-in manifests from the `plugins` property of a manifest list.
///
/// Only the `plugins` property is interpreted, so an invalid resource or extension in the same
/// list doesn't hide the plug-ins. Resource discovery reports a list that can't be parsed, so this
/// function only logs that at the debug level. An invalid entry in `plugins` is skipped with a
/// warning.
///
/// # Arguments
///
/// * `path` - The path of a manifest list file.
fn load_manifests_from_list(path: &Path) -> Vec<PluginManifest> {
    let list = read_to_string(path).map_err(DscError::from).and_then(|contents| {
        if is_json_file(path) {
            Ok(serde_json::from_str::<Value>(&contents)?)
        } else {
            Ok(serde_yaml::from_str::<Value>(&contents)?)
        }
    });
    let list = match list {
        Ok(list) => list,
        Err(err) => {
            debug!("{}", t!("plugins.discovery.failedLoadManifestList", path = path.display(), err = err));
            return Vec::new();
        }
    };

    let Some(entries) = list.get("plugins").filter(|plugins| !plugins.is_null()) else {
        return Vec::new();
    };
    let Some(entries) = entries.as_array() else {
        warn!("{}", t!("plugins.discovery.pluginsNotArray", path = path.display()));
        return Vec::new();
    };

    let mut manifests = Vec::with_capacity(entries.len());
    for entry in entries {
        match serde_json::from_value::<PluginManifest>(entry.clone()) {
            Ok(manifest) => manifests.push(manifest),
            Err(err) => {
                warn!("{}", t!("plugins.discovery.failedLoadManifest", path = path.display(), err = err));
            }
        }
    }

    manifests
}

/// Validates a manifest and evaluates its condition.
///
/// # Arguments
///
/// * `path` - The path of the file that defines the manifest.
/// * `manifest` - The manifest to turn into a plug-in.
///
/// # Returns
///
/// The plug-in, or [`None`] when the condition of the manifest evaluates to `false`.
///
/// # Errors
///
/// Returns a [`DscError`] when the manifest doesn't validate or its condition can't be evaluated
/// as a boolean.
fn load_plugin(path: &Path, manifest: PluginManifest) -> Result<Option<DscPlugin>, DscError> {
    manifest.validate()?;

    if !evaluate_condition(manifest.condition.as_deref())? {
        debug!("{}", t!(
            "plugins.discovery.conditionNotMet",
            path = path.display(),
            condition = manifest.condition.as_deref().unwrap_or_default(),
            plugin = manifest.r#type
        ));
        return Ok(None);
    }

    Ok(Some(DscPlugin::from_manifest(path, manifest)))
}

/// Indicates whether a manifest file is JSON, as opposed to YAML, based on its file extension.
///
/// # Arguments
///
/// * `path` - The path of the manifest file.
fn is_json_file(path: &Path) -> bool {
    path.extension().is_some_and(|ext| ext.eq_ignore_ascii_case("json"))
}
