// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use dsc_lib_jsonschema::dsc_repo::{DscRepoSchema, schema_i18n};
use schemars::JsonSchema;
use serde::Serialize;

use crate::settings::{
    fields::*,
    DscSettingsResolvedField
};

#[cfg(doc)]
pub(in crate::settings) mod _implementation_guidance {
    //! This module provides guidance for updating the [`DscSettingsResolved`]struct.
    //!
    //! When adding a new top-level field to [`DscSettingsResolved`], follow these steps:
    //!
    //! 1. Ensure that the field is defined in the [`fields`] module following that
    //!    [module guidance](crate::settings::fields::_implementation_guidance).
    //! 1. Add the new field to the [`DscSettingsResolved`] struct.
    //!
    //!    - If the field is a container field, define the field in the struct as the appropriate
    //!      `*ResolvedSettings` struct type.
    //!    - If the field is a leaf field, define the field in the struct as a
    //!      [`DscSettingsResolvedField<T>`] type with `T` as the appropriate type.
    //!
    //! [`DscSettingsResolved`]: super::DscSettingsResolved
    //! [`fields`]: crate::settings::fields
    //! [`DscSettingsResolvedField<T>`]: super::DscSettingsResolvedField
}

/// Defines the effective settings for DSC after resolving all the settings
/// sources.
///
/// The resolved settings are the effective values that DSC uses during command
/// execution. These settings are derived from the various settings sources,
/// including the policy settings file ([`Policy`] scope), preference settings
/// files ([`Machine`], [`User`], and [`Workspace`] scopes), environment
/// variables ([`Environment`] scope), and command-line options
/// ([`CommandLine`] scope).
///
/// The resolved settings represent the final configuration that DSC uses,
/// taking into account the precedence of the different settings sources.
///
/// Every leaf field in the resolved settings contains both the resolved
/// value and the scope from which the value was resolved.
///
/// [`Policy`]: DscSettingsScope::Policy
/// [`Machine`]: DscSettingsScope::Machine
/// [`User`]: DscSettingsScope::User
/// [`Workspace`]: DscSettingsScope::Workspace
/// [`Environment`]: DscSettingsScope::Environment
/// [`CommandLine`]: DscSettingsScope::CommandLine
#[derive(Clone, Debug, PartialEq, Eq, Serialize, JsonSchema, DscRepoSchema)]
#[dsc_repo_schema(base_name = "settings", folder_path = "settings/resolved")]
#[serde(rename_all = "camelCase")]
#[schemars(
    title = schema_i18n!("title"),
    description = schema_i18n!("description"),
    extend(
        "$schema" = Self::default_export_meta_schema_uri(),
        "$id" = Self::default_export_schema_id_uri(),
        "markdownDescription" = schema_i18n!("markdownDescription")
    )
)]
pub struct DscSettingsResolved {
    /// Indicates whether to allow users to ignore settings files.
    ///
    /// This setting is only available in the [`Policy`] scope. The default
    /// setting is `false`.
    ///
    /// When this setting is `false`, users can indicate that DSC should not
    /// load and resolve settings files by specifying the
    /// [`--ignore-settings-file`] CLI option or defining the
    /// [`DSC_IGNORE_SETTINGS_FILE`] environment variable. Doing so effectively
    /// skips processing the [`Machine`], [`User`], and [`Workspace`] settings
    /// scopes. The [`Policy`] scope is always processed regardless of this
    /// setting.
    ///
    /// When the policy scope defines this setting as `true`, users aren't
    /// allowed to ignore settings files. If a user attempts to ignore settings
    /// files, DSC raises a warning and indicates that the command is
    /// processing settings files as normal.
    ///
    /// [`Policy`]: DscSettingsScope::Policy
    /// [`Machine`]: DscSettingsScope::Machine
    /// [`User`]: DscSettingsScope::User
    /// [`Workspace`]: DscSettingsScope::Workspace
    /// [`--ignore-settings-file`]: crate::settings::sources::CommandLineData::ignore_settings_file
    /// [`DSC_IGNORE_SETTINGS_FILE`]: crate::settings::sources::EnvironmentData::dsc_ignore_settings_file
    #[schemars(
        title = schema_i18n!("forbid_ignore_settings_file.title"),
        description = schema_i18n!("forbid_ignore_settings_file.description"),
        extend(
            "markdownDescription" = schema_i18n!("forbid_ignore_settings_file.markdownDescription")
        )
    )]
    pub forbid_ignore_settings_file: DscSettingsResolvedField<ForbidIgnoreSettingsFileField>,
    /// Indicates whether to ignore settings files.
    ///
    /// This setting is available in the [`Environment`] and [`Cli`] scopes.
    /// The default setting is `false`.
    ///
    /// When this setting is `true`, DSC ignores the [`Machine`], [`User`], and
    /// [`Workspace`] settings scopes. DSC won't automatically load those
    /// settings files and will ignore them during settings resolution even if
    /// they were manually loaded. The [`Policy`] scope is always processed
    /// regardless of this setting.
    ///
    /// If the [`forbid_ignore_settings_file`] setting is defined as `true` in
    /// the [`Policy`] scope, this setting is effectively ignored and DSC will
    /// always load and resolve settings files if they exist.
    ///
    /// [`Environment`]: DscSettingsScope::Environment
    /// [`Cli`]: DscSettingsScope::CommandLine
    /// [`Machine`]: DscSettingsScope::Machine
    /// [`User`]: DscSettingsScope::User
    /// [`Workspace`]: DscSettingsScope::Workspace
    /// [`Policy`]: DscSettingsScope::Policy
    /// [`forbid_ignore_settings_file`]: Self::forbid_ignore_settings_file
    #[schemars(
        title = schema_i18n!("ignore_settings_file.title"),
        description = schema_i18n!("ignore_settings_file.description"),
        extend(
            "markdownDescription" = schema_i18n!("ignore_settings_file.markdownDescription")
        )
    )]
    pub ignore_settings_file: DscSettingsResolvedField<IgnoreSettingsFileField>,
    /// Indicates how DSC should emit messages during command execution.
    ///
    /// These settings control which messages DSC emits to stderr and the
    /// format it emits them  in.
    ///
    /// The following snippet shows the effective code defaults as YAML data:
    ///
    /// ```yaml
    /// tracing:
    ///   level: info
    ///   format: default
    /// ```
    ///
    /// For more information on defining these settings, see:
    ///
    /// - [`TracingFileData`] for defining them in the [policy settings file] or
    ///   [preference settings files].
    /// - [`EnvironmentData`] for defining them as environment variables.
    /// - [`CommandLineData`] for defining them as CLI options.
    ///
    /// [`TracingFileData`]: crate::settings::fields::TracingFileData
    /// [`EnvironmentData`]: crate::settings::sources::EnvironmentData
    /// [`CommandLineData`]: crate::settings::sources::CommandLineData
    /// [policy settings file]: crate::settings::sources::PolicyFileData
    /// [preference settings files]: crate::settings::sources::PreferenceFileData
    #[schemars(
        title = schema_i18n!("tracing.title"),
        description = schema_i18n!("tracing.description"),
        extend(
            "markdownDescription" = schema_i18n!("tracing.markdownDescription")
        )
    )]
    pub tracing: TracingResolvedSettings,
    /// Indicates how DSC should discover manifests and binaries during command
    /// execution.
    ///
    /// These settings control which directories DSC searches for manifests and
    /// binaries, whether to include the system `PATH` environment variable in
    /// the search, and whether to restrict the search to only the specified
    /// directories.
    ///
    /// The following snippet shows the effective code defaults as YAML data:
    ///
    /// ```yaml
    /// resourcePath:
    ///   appendEnvPath: true
    ///   restricted: false
    ///   directories: []
    /// ```
    ///
    /// For more information on defining these settings, see:
    ///
    /// - [`ResourcePathFileData`] for defining them in the
    ///   [policy settings file] or[preference settings files].
    /// - [`EnvironmentData`] for defining them as environment variables.
    /// - [`CommandLineData`] for defining them as CLI options.
    ///
    /// [`ResourcePathFileData`]: crate::settings::fields::ResourcePathFileData
    /// [`EnvironmentData`]: crate::settings::sources::EnvironmentData
    /// [`CommandLineData`]: crate::settings::sources::CommandLineData
    /// [policy settings file]: crate::settings::sources::PolicyFileData
    /// [preference settings files]: crate::settings::sources::PreferenceFileData
    #[schemars(
        title = schema_i18n!("resource_path.title"),
        description = schema_i18n!("resource_path.description"),
        extend(
            "markdownDescription" = schema_i18n!("resource_path.markdownDescription")
        )
    )]
    pub resource_path: ResourcePathResolvedSettings,
}

impl Default for DscSettingsResolved {
    fn default() -> Self {
        Self {
            forbid_ignore_settings_file: DscSettingsResolvedField::for_code_default(CODE_DEFAULT_FORBID_IGNORE_SETTINGS_FILE),
            ignore_settings_file: DscSettingsResolvedField::for_code_default(CODE_DEFAULT_IGNORE_SETTINGS_FILE),
            tracing: TracingResolvedSettings::default(),
            resource_path: ResourcePathResolvedSettings::default(),
        }
    }
}
