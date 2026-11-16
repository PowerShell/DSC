// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::ffi::OsString;
use std::path::{Path, PathBuf};

use crate::dscerror::DscError;
use crate::dscresources::command_resource::verify_executable_is_invokable;
use crate::plugins::command_name::PluginCommandName;
use crate::plugins::plugin_manifest::PluginManifest;
use crate::schemas::dsc_repo::{DscRepoSchema, schema_i18n};
use crate::types::{FullyQualifiedTypeName, SemanticVersion};
use crate::util::canonicalize_which;

/// Describes a discovered command plug-in.
///
/// This is the data a host reports when it lists the available plug-ins, and what it needs to run
/// the plug-in's command.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize, JsonSchema, DscRepoSchema)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
#[dsc_repo_schema(base_name = "list", folder_path = "outputs/plugin")]
#[schemars(
    transform = DscPlugin::transform_export_schema_uris,
    transform = DscPlugin::transform_schema_docs
)]
pub struct DscPlugin {
    /// The namespaced name that identifies the plug-in.
    #[serde(rename = "type")]
    #[schemars(extend(
        "title" = schema_i18n!("fields.type.title"),
        "description" = schema_i18n!("fields.type.description"),
        "markdownDescription" = schema_i18n!("fields.type.markdownDescription"),
    ))]
    pub type_name: FullyQualifiedTypeName,
    /// The version of the plug-in.
    #[schemars(extend(
        "title" = schema_i18n!("fields.version.title"),
        "description" = schema_i18n!("fields.version.description"),
        "markdownDescription" = schema_i18n!("fields.version.markdownDescription"),
    ))]
    pub version: SemanticVersion,
    /// The name of the command the plug-in adds.
    #[schemars(extend(
        "title" = schema_i18n!("fields.command.title"),
        "description" = schema_i18n!("fields.command.description"),
        "markdownDescription" = schema_i18n!("fields.command.markdownDescription"),
    ))]
    pub command: PluginCommandName,
    /// The description of the command.
    #[schemars(extend(
        "title" = schema_i18n!("fields.description.title"),
        "description" = schema_i18n!("fields.description.description"),
        "markdownDescription" = schema_i18n!("fields.description.markdownDescription"),
    ))]
    pub description: Option<String>,
    /// An optional message indicating the plug-in is deprecated.
    #[schemars(extend(
        "title" = schema_i18n!("fields.deprecationMessage.title"),
        "description" = schema_i18n!("fields.deprecationMessage.description"),
        "markdownDescription" = schema_i18n!("fields.deprecationMessage.markdownDescription"),
    ))]
    pub deprecation_message: Option<String>,
    /// The path of the file that defines the plug-in manifest.
    #[schemars(extend(
        "title" = schema_i18n!("fields.path.title"),
        "description" = schema_i18n!("fields.path.description"),
        "markdownDescription" = schema_i18n!("fields.path.markdownDescription"),
    ))]
    pub path: PathBuf,
    /// The directory that contains the plug-in manifest.
    #[schemars(extend(
        "title" = schema_i18n!("fields.directory.title"),
        "description" = schema_i18n!("fields.directory.description"),
        "markdownDescription" = schema_i18n!("fields.directory.markdownDescription"),
    ))]
    pub directory: PathBuf,
    /// Indicates whether another plug-in that DSC found earlier provides the same command. A
    /// shadowed plug-in is never run for its command.
    #[schemars(extend(
        "title" = schema_i18n!("fields.shadowed.title"),
        "description" = schema_i18n!("fields.shadowed.description"),
        "markdownDescription" = schema_i18n!("fields.shadowed.markdownDescription"),
    ))]
    pub shadowed: bool,
    /// The manifest of the plug-in.
    pub manifest: PluginManifest,
}

/// Defines the executable and the leading arguments to run for a plug-in command.
///
/// The host appends the arguments the user specified after [`args`] before it starts
/// [`executable`].
///
/// [`args`]: Self::args
/// [`executable`]: Self::executable
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PluginInvocation {
    /// The resolved path of the executable to run.
    pub executable: PathBuf,
    /// The arguments from the manifest, to pass before the arguments the user specified.
    pub args: Vec<OsString>,
}

impl DscPlugin {
    /// Creates the description of a plug-in from its manifest.
    ///
    /// The plug-in isn't marked as shadowed. Discovery decides that after it has found every
    /// plug-in.
    ///
    /// # Arguments
    ///
    /// * `manifest_path` - The path of the file that defines the manifest.
    /// * `manifest` - The manifest of the plug-in.
    #[must_use]
    pub fn from_manifest(manifest_path: &Path, manifest: PluginManifest) -> Self {
        Self {
            type_name: manifest.r#type.clone(),
            version: manifest.version.clone(),
            command: manifest.command.name.clone(),
            description: manifest.description.clone(),
            deprecation_message: manifest.deprecation_message.clone(),
            path: manifest_path.to_path_buf(),
            directory: manifest_path.parent().map(Path::to_path_buf).unwrap_or_default(),
            shadowed: false,
            manifest,
        }
    }

    /// Resolves the executable and the leading arguments to run for the plug-in's command.
    ///
    /// DSC resolves the executable the way it does for resource and extension manifests: first in
    /// the `PATH` environment variable, then relative to the directory that contains the
    /// manifest.
    ///
    /// # Errors
    ///
    /// * [`DscError::CommandOperation`] when the executable can't be found.
    /// * [`DscError::CommandNotInRestrictedDirs`] when the `resourcePath.restricted` setting is
    ///   `true` and the executable isn't in one of the `resourcePath.directories`.
    pub fn resolve_invocation(&self) -> Result<PluginInvocation, DscError> {
        let executable = PathBuf::from(
            canonicalize_which(&self.manifest.command.executable, Some(&self.directory))?
        );
        verify_executable_is_invokable(&executable)?;

        Ok(PluginInvocation {
            executable,
            args: self.manifest.command.to_command_args(&self.directory),
        })
    }
}
