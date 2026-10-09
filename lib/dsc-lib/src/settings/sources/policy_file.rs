// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! Defines the [`PolicyFileData`] struct, which represents the data structure of the policy
//! file for DSC settings.

#[cfg(doc)]
pub(in crate::settings) mod _implementation_guidance {
    //! Every field defined in [`DscSettingsResolved`] struct should also be defined in the
    //! [`PolicyFileData`] struct.
    //!
    //! When adding a new top-level field to the policy file, follow these guidelines:
    //!
    //! 1. Ensure that the field is defined in the [`fields`] module following that
    //!    [module guidance](crate::settings::fields::_implementation_guidance).
    //! 1. Ensure that the field is defined in the [`DscSettingsResolved`] struct, following the
    //!    [module guidance](crate::settings::resolved::_implementation_guidance).
    //! 1. Add the field to the [`PolicyFileData`] struct:
    //!
    //!    - If the field is a container field, define the field in the struct as the appropriate
    //!      `*PolicyFileData` or `*FileData` struct type.
    //!    - If the field is a leaf field, define the field in the struct as an
    //!      [`Option<T>`] with the appropriate type.
    //!
    //! No changes are required for the `from_file()` method, as it deserializes the field from the
    //! policy file if it's defined.
    //!
    //! [`DscSettingsResolved`]: crate::settings::DscSettingsResolved
    //! [`PolicyFileData`]: super::PolicyFileData
    //! [`fields`]: crate::settings::fields
}

#[cfg(not(test))]
use std::{fs::read_to_string, path::Path};

#[cfg(test)]
use crate::tests::stubs::{fs::read_to_string, path::Path};

use rust_i18n::t;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tracing::debug;

use crate::settings::{DscSettingsError, DscSettingsScope, fields::*};
use crate::schemas::dsc_repo::{DscRepoSchema, schema_i18n};

/// Defines all settings for DSC as policy, which cannot be overridden.
///
/// Any settings defined in the system policy settings file take precedence
/// over all other settings. Every setting that controls how DSC behaves is
/// represented in the policy file data.
#[derive(Default, Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema, DscRepoSchema)]
#[serde(rename_all = "camelCase")]
#[dsc_repo_schema(base_name = "policyFile", folder_path = "settings/sources", should_bundle = true)]
#[schemars(
    title = schema_i18n!("title"),
    description = schema_i18n!("description"),
    extend(
        "$id" = Self::default_export_schema_id_uri(),
        "markdownDescription" = schema_i18n!("markdownDescription")
    )
)]
pub struct PolicyFileData {
    /// Indicates whether DSC should allow ignoring preference settings files
    /// when loading and resolving settings.
    ///
    /// When this field is set to `true`, DSC will raise a warning if the user
    /// defines the [`DSC_IGNORE_SETTINGS_FILE`] environment variable as `true`
    /// or passes the [`--ignore-settings-file`] command-line argument. DSC
    /// will load and resolve the preference files as if those options weren't
    /// specified.
    ///
    /// This setting supercedes [`ignore_settings_file`]. If
    /// `forbid_ignore_settings_file` is set to `true`, DSC will ignore the
    /// `ignore_settings_file` policy setting.
    ///
    /// [`DSC_IGNORE_SETTINGS_FILE`]: crate::settings::sources::EnvironmentData::dsc_ignore_settings_file
    /// [`--ignore-settings-file`]: crate::settings::sources::CommandLineData::ignore_settings_file
    /// [`ignore_settings_file`]: Self::ignore_settings_file
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schemars(
        title = schema_i18n!("forbid_ignore_settings_file.title"),
        description = schema_i18n!("forbid_ignore_settings_file.description"),
        extend(
            "markdownDescription" = schema_i18n!("forbid_ignore_settings_file.markdownDescription")
        ),
    )]
    pub forbid_ignore_settings_file: Option<ForbidIgnoreSettingsFileField>,
    /// Indicates whether DSC should ignore preference settings files when
    /// loading and resolving settings.
    ///
    /// When this field is set to `true`, DSC will ignore preference settings
    /// files. It won't automatically load them and will ignore them when
    /// resolving settings.
    ///
    /// If you define the `forbid_ignore_settings_file` field as `true`, DSC
    /// ignores this setting.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schemars(
        title = schema_i18n!("ignore_settings_file.title"),
        description = schema_i18n!("ignore_settings_file.description"),
        extend(
            "markdownDescription" = schema_i18n!("ignore_settings_file.markdownDescription")
        ),
    )]
    pub ignore_settings_file: Option<IgnoreSettingsFileField>,
    /// Defines settings that affect how DSC emits trace messages.
    ///
    /// You can control the level of the messages DSC emits and the format that
    /// DSC emits the messages as.
    ///
    /// For more information, see [`TracingFileData`].
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schemars(
        title = schema_i18n!("tracing.title"),
        description = schema_i18n!("tracing.description"),
        extend(
            "markdownDescription" = schema_i18n!("tracing.markdownDescription")
        ),
    )]
    pub tracing: Option<TracingFileData>,
    /// Defines settings that affect how DSC discovers and invokes resources
    /// and other manifests.
    ///
    /// You can control the directories that DSC searches for executables and
    /// manifests, whether to append the `PATH` environment variable, and
    /// whether to restrict execution to the specified directories.
    ///
    /// For more information, see [`ResourcePathFileData`].
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schemars(
        title = schema_i18n!("resource_path.title"),
        description = schema_i18n!("resource_path.description"),
        extend(
            "markdownDescription" = schema_i18n!("resource_path.markdownDescription")
        ),
    )]
    pub resource_path: Option<ResourcePathFileData>,
}

/// Public API
impl PolicyFileData {
    /// Creates an instance of [`PolicyFileData`] from the specified file.
    ///
    /// This method reads the contents of the specified file and attempts to
    /// deserialize it into an instance of [`PolicyFileData`].
    ///
    /// # Arguments
    ///
    /// - `file_path` - The path to the preference file.
    ///
    /// # Returns
    ///
    /// A [`Result`] containing the instance of [`PolicyFileData`] if
    /// successful, or a [`DscSettingsError`] if an error occurred.
    ///
    /// # Errors
    ///
    /// This method raises an error under the following conditions:
    ///
    /// 1. The file can't be read.
    /// 1. The file is empty or contains only spacing characters.
    /// 1. The file contains text that can't be parsed as valid JSON into the
    ///    the [`PolicyFileData`] structure.
    pub fn from_file(file_path: &Path) -> Result<Self, DscSettingsError> {
        // Make a string representation of the file path and scope for error
        // reporting to avoid repeating the method chains throughout.
        let file_path_str = file_path.to_string_lossy().to_string();
        let scope = DscSettingsScope::Policy.to_string();
        debug!("{}", t!(
            "settings.sources.policy_file.debugLoadingFile",
            file_path = file_path_str.clone(),
            scope = scope.clone(),
        ));
        // Retrieve the file contents as a string or error
        let contents = read_to_string(file_path)
            .map_err(|err| DscSettingsError::DataFileReadError {
                file_path: file_path_str.clone(),
                scope: scope.clone(),
                source: err,
            })?;
        // Error early if file is empty or contains only spacing characters
        if contents.trim().is_empty() {
            return Err(DscSettingsError::DataFileEmpty {
                file_path: file_path_str.clone(),
                scope: scope.clone(),
            });
        }

        serde_json::from_str::<PolicyFileData>(&contents)
            .map_err(|err| DscSettingsError::DataFileUnparseable{
                file_path: file_path_str.clone(),
                scope: scope.clone(),
                source: err,
            })
    }
}

/// Private API
impl PolicyFileData {
    /// Indicates whether any of the fields were populated from file data.
    ///
    /// This method is used when deciding whether to serialize the file data
    /// and for processing to enable skipping when no fields were defined.
    ///
    /// # Returns
    ///
    /// `true` if any of the file data fields are populated, otherwise `false`.
    pub(crate) fn any_defined(&self) -> bool {
        self.forbid_ignore_settings_file.is_some() ||
        self.ignore_settings_file.is_some() ||
        self.resource_path.is_some() ||
        self.tracing.is_some()
    }
}