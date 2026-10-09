// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! Defines the default values for DSC settings fields.

use schemars::{JsonSchema, json_schema};
use serde::Serialize;

use crate::schemas::dsc_repo::{DscRepoSchema, schema_i18n};
use crate::settings::fields::*;

#[cfg(doc)]
pub(in crate::settings) mod _implementation_guidance {
    //! The [`DscSettingsCodeDefaults`] struct should mirror the structure of
    //! [`DscSettingsResolved`], except that instead of using [`DscSettingsResolvedField<T>`] for
    //! each field, it should use the underlying value type for each field.
    //!
    //! The [`DSC_SETTINGS_CODE_DEFAULTS`] constant is a static representation of the code defaults
    //! and every field should be initialized with the appropriate constant from the [`fields`]
    //! module.
    //!
    //! [`DscSettingsCodeDefaults`]: super::DscSettingsCodeDefaults
    //! [`DSC_SETTINGS_CODE_DEFAULTS`]: super::DSC_SETTINGS_CODE_DEFAULTS
    //! [`DscSettingsResolved`]: crate::settings::DscSettingsResolved
    //! [`DscSettingsResolvedField<T>`]: crate::settings::DscSettingsResolvedField
    //! [`fields`]: crate::settings::fields
}

/// Defines the default values for DSC settings fields.
///
/// DSC uses a layered approach to resolving settings values. The code
/// defaults, which this struct represents, are the lowest precedence in the
/// settings hierarchy. They are defined in the [`DSC_SETTINGS_CODE_DEFAULTS`]
/// constant.
///
/// These defaults are used when no other sources define a value for a setting.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, DscRepoSchema)]
#[dsc_repo_schema(base_name = "default", folder_path = "settings/sources")]
pub struct DscSettingsCodeDefaults {
    /// Indicates whether to allow users to ignore settings files.
    ///
    /// The code default for this setting is `false`, which means that users
    /// are allowed to ignore settings files. This setting can only be
    /// overridden by the [`Policy`] scope.
    ///
    /// For more information, see [`forbid_ignore_settings_file`] in the policy
    /// file documentation.
    ///
    /// [`Policy`]: crate::settings::DscSettingsScope::Policy
    /// [`forbid_ignore_settings_file`]: crate::settings::sources::PolicyFileData::forbid_ignore_settings_file
    pub forbid_ignore_settings_file: ForbidIgnoreSettingsFileField,
    /// Indicates whether to ignore settings files.
    ///
    /// The code default for this setting is `false`, which means that DSC will
    /// load and resolve settings files. This setting can be overridden by the
    /// [`Environment`] and [`CLI`] scopes.
    ///
    /// If the [`forbid_ignore_settings_file`] setting is defined as `true` in
    /// the [`Policy`] scope, this setting is effectively ignored and DSC will
    /// always load and resolve settings files.
    ///
    /// For more information, see [`DscSettingsResolved::ignore_settings_file`].
    ///
    /// [`Environment`]: crate::settings::DscSettingsScope::Environment
    /// [`CLI`]: crate::settings::DscSettingsScope::CommandLine
    /// [`Policy`]: crate::settings::DscSettingsScope::Policy
    /// [`forbid_ignore_settings_file`]: crate::settings::sources::PolicyFileData::forbid_ignore_settings_file
    /// [`DscSettingsResolved::ignore_settings_file`]: crate::settings::DscSettingsResolved::ignore_settings_file
    pub ignore_settings_file: IgnoreSettingsFileField,
    /// Defines how DSC should emit trace messages for logging and diagnostics.
    ///
    /// The code default for this container field is defined by
    /// [`CODE_DEFAULT_TRACING`].
    ///
    /// You can override these settings in every scope. For more information,
    /// see:
    ///
    /// - [`PolicyFileData::tracing`] to define these settings in the
    ///   [`Policy`] scope.
    /// - [`PreferenceFileData::tracing`] to define these settings as
    ///   preferences in the [`Machine`], [`User`], and [`Workspace`] scopes.
    /// - [`dsc_trace_level`] and [`dsc_trace_format`] in [`EnvironmentData`]
    ///   to define these settings in the [`Environment`] scope.
    /// - [`trace_format`] and [`trace_level`] in [`CommandLineData`] to define
    ///   these settings in the [`CommandLine`] scope.
    ///
    /// [`dsc_trace_level`]: crate::settings::sources::EnvironmentData::dsc_trace_level
    /// [`dsc_trace_format`]: crate::settings::sources::EnvironmentData::dsc_trace_format
    /// [`Environment`]: crate::settings::DscSettingsScope::Environment
    /// [`CommandLine`]: crate::settings::DscSettingsScope::CommandLine
    /// [`trace_format`]: crate::settings::sources::CommandLineData::trace_format
    /// [`trace_level`]: crate::settings::sources::CommandLineData::trace_level
    /// [`CommandLineData`]: crate::settings::sources::CommandLineData
    /// [`EnvironmentData`]: crate::settings::sources::EnvironmentData
    /// [`PolicyFileData::tracing`]: crate::settings::sources::PolicyFileData::tracing
    /// [`PreferenceFileData::tracing`]: crate::settings::sources::PreferenceFileData::tracing
    /// [`Machine`]: crate::settings::DscSettingsScope::Machine
    /// [`User`]: crate::settings::DscSettingsScope::User
    /// [`Workspace`]: crate::settings::DscSettingsScope::Workspace
    /// [`Policy`]: crate::settings::DscSettingsScope::Policy
    pub tracing: TracingCodeDefaults,
    /// Defines the paths to use when searching for and invoking resources,
    /// extensions, and other executables.
    ///
    /// The code default for this container field is defined by
    /// [`CODE_DEFAULT_RESOURCE_PATH`].
    ///
    /// You can override this setting in every scope except for [`CommandLine`].
    /// For more information, see:
    ///
    /// - [`PolicyFileData::resource_path`] to define this setting in the
    ///   [`Policy`] scope.
    /// - [`PreferenceFileData::resource_path`] to define this setting as a
    ///   preference in the [`Machine`], [`User`], and [`Workspace`] scopes.
    /// - [`dsc_resource_path`] and [`dsc_restricted_path`] in
    ///   [`EnvironmentData`] to define this setting in the [`Environment`]
    ///   scope.
    ///
    /// [`CommandLine`]: crate::settings::DscSettingsScope::CommandLine
    /// [`Policy`]: crate::settings::DscSettingsScope::Policy
    /// [`Machine`]: crate::settings::DscSettingsScope::Machine
    /// [`User`]: crate::settings::DscSettingsScope::User
    /// [`Workspace`]: crate::settings::DscSettingsScope::Workspace
    /// [`PolicyFileData::resource_path`]: crate::settings::sources::PolicyFileData::resource_path
    /// [`PreferenceFileData::resource_path`]: crate::settings::sources::PreferenceFileData::resource_path
    /// [`dsc_resource_path`]: crate::settings::sources::EnvironmentData::dsc_resource_path
    /// [`dsc_restricted_path`]: crate::settings::sources::EnvironmentData::dsc_restricted_path
    /// [`Environment`]: crate::settings::DscSettingsScope::Environment
    /// [`EnvironmentData`]: crate::settings::sources::EnvironmentData
    pub resource_path: ResourcePathCodeDefaults,
}

impl JsonSchema for DscSettingsCodeDefaults {
    fn schema_name() -> std::borrow::Cow<'static, str> {
        "DscSettingsCodeDefaults".into()
    }

    fn json_schema(_: &mut schemars::SchemaGenerator) -> schemars::Schema {
        json_schema!({
            "$schema": Self::default_export_meta_schema_uri(),
            "$id": Self::default_export_schema_id_uri(),
            "title": schema_i18n!("title"),
            "description": schema_i18n!("description"),
            "markdownDescription": schema_i18n!("markdownDescription"),
            "const": DSC_SETTINGS_CODE_DEFAULTS,
        })
    }
}

/// Defines the default values for DSC settings fields.
///
/// The following snippet shows the effective code defaults as YAML data:
///
/// ```yaml
/// forbid_ignore_settings_file: false
/// ignore_settings_file: false
/// tracing:
///   level: warn
///   format: default
/// resource_path:
///   append_env_path: true
///   directories: []
///   restricted: false
/// ```
pub const DSC_SETTINGS_CODE_DEFAULTS: DscSettingsCodeDefaults = DscSettingsCodeDefaults {
    forbid_ignore_settings_file: CODE_DEFAULT_FORBID_IGNORE_SETTINGS_FILE,
    ignore_settings_file: CODE_DEFAULT_IGNORE_SETTINGS_FILE,
    tracing: CODE_DEFAULT_TRACING,
    resource_path: CODE_DEFAULT_RESOURCE_PATH,
};

impl Default for DscSettingsCodeDefaults {
    fn default() -> Self {
        DSC_SETTINGS_CODE_DEFAULTS
    }
}
