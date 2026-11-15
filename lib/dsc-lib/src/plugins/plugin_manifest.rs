// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use rust_i18n::t;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::ffi::OsString;
use std::path::{Component, Path, PathBuf};

use crate::dscerror::DscError;
use crate::plugins::command_name::PluginCommandName;
use crate::schemas::dsc_repo::{DscRepoSchema, schema_i18n};
use crate::types::{FullyQualifiedTypeName, SemanticVersion, TagList};

/// Defines a command plug-in: the command it adds to a DSC host and how to run it.
///
/// DSC discovers plug-in manifests in files whose name ends with `.dsc.plugin.json`,
/// `.dsc.plugin.yaml`, or `.dsc.plugin.yml`, and in the `plugins` property of a manifest list.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema, DscRepoSchema)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
#[schemars(
    transform = PluginManifest::transform_export_schema_uris,
    transform = PluginManifest::transform_schema_docs
)]
#[dsc_repo_schema(
    base_name = "manifest",
    folder_path = "plugin",
    should_bundle = true,
    schema_field(
        name = schema_version,
        title = t!("plugins.plugin_manifest.pluginManifestSchemaTitle"),
        description = t!("plugins.plugin_manifest.pluginManifestSchemaDescription"),
    )
)]
pub struct PluginManifest {
    /// The version of the plug-in manifest schema.
    #[serde(rename = "$schema")]
    #[schemars(schema_with = "PluginManifest::recognized_schema_uris_subschema")]
    pub schema_version: String,
    /// The namespaced name that identifies the plug-in.
    #[serde(rename = "type")]
    #[schemars(extend(
        "title" = schema_i18n!("fields.type.title"),
        "description" = schema_i18n!("fields.type.description"),
        "markdownDescription" = schema_i18n!("fields.type.markdownDescription"),
    ))]
    pub r#type: FullyQualifiedTypeName,
    /// The version of the plug-in using semantic versioning.
    #[schemars(extend(
        "title" = schema_i18n!("fields.version.title"),
        "description" = schema_i18n!("fields.version.description"),
        "markdownDescription" = schema_i18n!("fields.version.markdownDescription"),
    ))]
    pub version: SemanticVersion,
    /// An optional condition for the plug-in to be available. If the condition evaluates to
    /// false, DSC treats the plug-in as not installed.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schemars(extend(
        "title" = schema_i18n!("fields.condition.title"),
        "description" = schema_i18n!("fields.condition.description"),
        "markdownDescription" = schema_i18n!("fields.condition.markdownDescription"),
    ))]
    pub condition: Option<String>,
    /// An optional message indicating the plug-in is deprecated. If provided, DSC shows the
    /// message as a warning each time the command is invoked.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schemars(extend(
        "title" = schema_i18n!("fields.deprecationMessage.title"),
        "description" = schema_i18n!("fields.deprecationMessage.description"),
        "markdownDescription" = schema_i18n!("fields.deprecationMessage.markdownDescription"),
    ))]
    pub deprecation_message: Option<String>,
    /// A one-line description of the command, shown in help and in the list of plug-ins.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schemars(extend(
        "title" = schema_i18n!("fields.description.title"),
        "description" = schema_i18n!("fields.description.description"),
        "markdownDescription" = schema_i18n!("fields.description.markdownDescription"),
    ))]
    pub description: Option<String>,
    /// Tags for the plug-in.
    #[serde(default, skip_serializing_if = "TagList::is_empty")]
    #[schemars(extend(
        "title" = schema_i18n!("fields.tags.title"),
        "description" = schema_i18n!("fields.tags.description"),
        "markdownDescription" = schema_i18n!("fields.tags.markdownDescription"),
    ))]
    pub tags: TagList,
    /// The command the plug-in adds and how DSC runs it.
    pub command: PluginCommand,
    /// Arbitrary data for the plug-in that DSC doesn't interpret.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schemars(extend(
        "title" = schema_i18n!("fields.metadata.title"),
        "description" = schema_i18n!("fields.metadata.description"),
        "markdownDescription" = schema_i18n!("fields.metadata.markdownDescription"),
    ))]
    pub metadata: Option<Map<String, Value>>,
}

impl PluginManifest {
    /// Validates the rules for a plug-in manifest that deserialization doesn't enforce.
    ///
    /// # Errors
    ///
    /// * [`DscError::UnrecognizedSchemaUri`] when the `$schema` property isn't one of the
    ///   recognized URIs for the plug-in manifest schema.
    /// * [`DscError::InvalidManifest`] when a `manifestRelativePath` argument is empty or isn't a
    ///   relative path.
    pub fn validate(&self) -> Result<(), DscError> {
        self.validate_schema_uri()?;

        for arg in self.command.args.iter().flatten() {
            if let PluginArgKind::ManifestRelativePath { manifest_relative_path } = arg
                && !is_manifest_relative(manifest_relative_path) {
                    return Err(DscError::InvalidManifest(t!(
                        "plugins.plugin_manifest.pathNotRelative",
                        plugin = self.r#type,
                        path = manifest_relative_path.display()
                    ).to_string()));
                }
        }

        Ok(())
    }
}

/// Defines the command a plug-in adds to a DSC host and how to run it.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, JsonSchema, DscRepoSchema)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
#[schemars(
    transform = PluginCommand::transform_export_schema_uris,
    transform = PluginCommand::transform_schema_docs
)]
#[dsc_repo_schema(base_name = "manifest.command", folder_path = "plugin")]
pub struct PluginCommand {
    /// The name of the command, which the user types after the name of the host.
    #[schemars(extend(
        "title" = schema_i18n!("fields.name.title"),
        "description" = schema_i18n!("fields.name.description"),
        "markdownDescription" = schema_i18n!("fields.name.markdownDescription"),
    ))]
    pub name: PluginCommandName,
    /// The executable to run for the command.
    #[schemars(extend(
        "title" = schema_i18n!("fields.executable.title"),
        "description" = schema_i18n!("fields.executable.description"),
        "markdownDescription" = schema_i18n!("fields.executable.markdownDescription"),
    ))]
    pub executable: String,
    /// The arguments to pass to the executable before the arguments the user specified.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schemars(extend(
        "title" = schema_i18n!("fields.args.title"),
        "description" = schema_i18n!("fields.args.description"),
        "markdownDescription" = schema_i18n!("fields.args.markdownDescription"),
    ))]
    pub args: Option<Vec<PluginArgKind>>,
}

impl PluginCommand {
    /// Resolves the argument definitions into the arguments to pass to the executable.
    ///
    /// Static string arguments are passed as written. A `manifestRelativePath` argument is
    /// replaced with the path of the file joined to the manifest directory.
    ///
    /// # Arguments
    ///
    /// * `manifest_directory` - The directory that contains the plug-in manifest.
    #[must_use]
    pub fn to_command_args(&self, manifest_directory: &Path) -> Vec<OsString> {
        self.args.iter().flatten().map(|arg| match arg {
            PluginArgKind::String(text) => OsString::from(text),
            PluginArgKind::ManifestRelativePath { manifest_relative_path } => {
                manifest_directory.join(manifest_relative_path).into_os_string()
            },
        }).collect()
    }
}

/// Defines an argument that DSC passes to the executable of a plug-in.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize, JsonSchema)]
#[serde(untagged, deny_unknown_fields)]
#[schemars(inline)]
pub enum PluginArgKind {
    /// A static string argument to pass to the executable as written, like `-NoProfile`.
    String(String),
    /// A file that ships with the plug-in. DSC passes the absolute path of the file, resolved
    /// against the directory that contains the manifest. A plug-in runs in the caller's working
    /// directory, not in the manifest directory, so this is how a manifest names a script that
    /// sits next to it.
    ManifestRelativePath {
        /// The path of the file relative to the directory that contains the manifest, like
        /// `graph.ps1` or `scripts/graph.py`.
        #[serde(rename = "manifestRelativePath")]
        manifest_relative_path: PathBuf,
    },
}

/// Indicates whether a path can be resolved against the manifest directory.
///
/// A path is relative to the manifest directory when it isn't empty and has neither a root, like
/// `/tools/graph.ps1`, nor a prefix, like `C:graph.ps1`. Joining a path with a root or a prefix to
/// the manifest directory would discard part or all of the manifest directory.
///
/// # Arguments
///
/// * `path` - The path from a `manifestRelativePath` argument.
fn is_manifest_relative(path: &Path) -> bool {
    !path.as_os_str().is_empty()
        && !path.components().any(|component| {
            matches!(component, Component::Prefix(_) | Component::RootDir)
        })
}
