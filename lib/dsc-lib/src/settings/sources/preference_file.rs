// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! Defines the [`PreferenceFileData`] struct, which represents the data structure of the
//! preference file for DSC settings.
//!
//! The [`Machine`], [`User`], and [`Workspace`] scopes all read their settings from a preference
//! file.
//!
//! [`Machine`]: crate::settings::DscSettingsScope::Machine
//! [`User`]: crate::settings::DscSettingsScope::User
//! [`Workspace`]: crate::settings::DscSettingsScope::Workspace

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

#[cfg(doc)]
pub(in crate::settings) mod _implementation_guidance {
    //! Most fields defined in the [`DscSettingsResolved`] struct should also be defined in the
    //! [`PreferenceFileData`] struct. The only current exceptions are [`IgnoreSettingsFileField`]
    //! and [`ForbidIgnoreSettingsFileField`], which affect the loading and resolution for settings
    //! files.
    //!
    //! When adding a new top-level field to the preference file, follow these guidelines:
    //!
    //! 1. Ensure that the field is defined in the [`fields`] module following that
    //!    [module guidance](crate::settings::fields::_implementation_guidance).
    //! 1. Ensure that the field is defined in the [`DscSettingsResolved`] struct, following the
    //!    [module guidance](crate::settings::resolved::_implementation_guidance).
    //! 1. Add the field to the [`PreferenceFileData`] struct:
    //!
    //!    - If the field is a container field, define the field in the struct as the appropriate
    //!      `*PreferenceFileData` or `*FileData` struct type.
    //!    - If the field is a leaf field, define the field in the struct as an [`Option<T>`] with
    //!      the appropriate type.
    //!
    //! No changes are required for the `from_file()` method, as it deserializes the field from the
    //! preference file if it's defined.
    //!
    //! [`DscSettingsResolved`]: crate::settings::DscSettingsResolved
    //! [`IgnoreSettingsFileField`]: crate::settings::fields::IgnoreSettingsFileField
    //! [`ForbidIgnoreSettingsFileField`]: crate::settings::fields::ForbidIgnoreSettingsFileField
    //! [`fields`]: crate::settings::fields
    //! [`PreferenceFileData`]: super::PreferenceFileData
}

/// Defines the structure of the preference file for DSC settings.
///
/// The preference file allows users to specify their preferred settings for
/// DSC, which can be overridden by higher-priority sources such as the policy
/// file.
#[derive(Default, Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema, DscRepoSchema)]
#[serde(rename_all = "camelCase")]
#[dsc_repo_schema(base_name = "preferenceFile", folder_path = "settings/sources", should_bundle = true)]
#[schemars(
    title = schema_i18n!("title"),
    description = schema_i18n!("description"),
    extend(
        "$id" = PreferenceFileData::default_export_schema_id_uri(),
        "markdownDescription" = schema_i18n!("markdownDescription")
    )
)]
pub struct PreferenceFileData {
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
impl PreferenceFileData {
    /// Creates an instance of [`PreferenceFileData`] from the specified file.
    ///
    /// This method reads the contents of the specified file and attempts to
    /// deserialize it into an instance of [`PreferenceFileData`].
    ///
    /// # Arguments
    ///
    /// - `file_path` - The path to the preference file.
    ///
    /// # Returns
    ///
    /// A [`Result`] containing the instance of [`PreferenceFileData`] if
    /// successful, or a [`DscSettingsError`] if an error occurred.
    ///
    /// # Errors
    ///
    /// This method raises an error under the following conditions:
    ///
    /// 1. The file can't be read.
    /// 1. The file is empty or contains only spacing characters.
    /// 1. The file contains text that can't be parsed as valid JSON into the
    ///    the [`PreferenceFileData`] structure.
    pub fn from_file(file_path: &Path) -> Result<Self, DscSettingsError> {
        // Make a string representation of the file path for error reporting
        // to avoid repeating the method chains throughout.
        let file_path_str = file_path.to_string_lossy().to_string();
        let scope = Self::scope_for_path(file_path);
        debug!("{}", t!(
            "settings.sources.preference_file.debugLoadingFile",
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

        serde_json::from_str::<PreferenceFileData>(&contents)
            .map_err(|err| DscSettingsError::DataFileUnparseable{
                file_path: file_path_str.clone(),
                scope: scope.clone(),
                source: err,
            })
    }
}

/// Private API
impl PreferenceFileData {
    /// Indicates whether any of the fields were populated from file data.
    ///
    /// This method is used when deciding whether to serialize the file data
    /// and for processing to enable skipping when no fields were defined.
    ///
    /// # Returns
    ///
    /// `true` if any of the file data fields are populated, otherwise `false`.
    pub(crate) fn any_defined(&self) -> bool {
        self.resource_path.is_some() ||
        self.tracing.is_some()
    }

    /// Determines the scope of the preference file based on its file path.
    ///
    /// If the file path matches one of the known settings file paths, the
    /// corresponding scope is returned. Otherwise, the scope is returned
    /// as `unknown`.
    ///
    /// This is used for error messaging.
    ///
    /// # Arguments
    ///
    /// - `file_path` - The path to the preference file.
    ///
    /// # Returns
    ///
    /// A string representing the scope of the preference file.
    fn scope_for_path(file_path: &Path) -> String {
        if file_path == crate::settings::MACHINE_SETTINGS_FILE_PATH.as_path() {
            DscSettingsScope::Machine.to_string()
        } else if file_path == crate::settings::USER_SETTINGS_FILE_PATH.as_path() {
            DscSettingsScope::User.to_string()
        } else if file_path == crate::settings::WORKSPACE_SETTINGS_FILE_PATH.as_path() {
            DscSettingsScope::Workspace.to_string()
        } else {
            String::from("unknown")
        }
    }
}
