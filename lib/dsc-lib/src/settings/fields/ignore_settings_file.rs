// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::schemas::dsc_repo::DscRepoSchema;

super::macros::define_boolean_field!(
    base_name: "ignoreSettingsFile",
    folder_path: "settings/fields",
    field_definition: {
        /// Indicates whether DSC should ignore settings files.
        ///
        /// This setting is available in the [`Policy`], [`Environment`], and
        /// [`Cli`] scopes. The default setting is `false`.
        ///
        /// When this setting is `false`, DSC loads and resolves the preference
        /// settings files for the [`Machine`], [`User`], and [`Workspace`]
        /// settings scopes if they exist.
        ///
        /// When this setting is `true`, DSC ignores the [`Machine`], [`User`],
        /// and [`Workspace`] settings scopes. DSC won't automatically load
        /// those settings files and will ignore them during settings
        /// resolution even if they were manually loaded.
        ///
        /// The [`Policy`] scope is always processed regardless of this setting.
        ///
        /// You can override this setting by:
        ///
        /// - Defining the [`ignore_settings_file`] field in the policy
        ///   settings file.
        /// - Defining the [`DSC_IGNORE_SETTINGS_FILE`] environment variable.
        /// - Specifying the [`--ignore-settings-file`] CLI option.
        ///
        /// If the policy settings file defines [`forbid_ignore_settings_file`]
        /// as `true`, DSC effectively ignores this setting and will always
        /// load and resolve the preference settings files if they exist.
        ///
        /// [`Policy`]: crate::settings::DscSettingsScope::Policy
        /// [`Environment`]: crate::settings::DscSettingsScope::Environment
        /// [`Cli`]: crate::settings::DscSettingsScope::CommandLine
        /// [`Machine`]: crate::settings::DscSettingsScope::Machine
        /// [`User`]: crate::settings::DscSettingsScope::User
        /// [`Workspace`]: crate::settings::DscSettingsScope::Workspace
        /// [`ignore_settings_file`]: crate::settings::sources::PolicyFileData::ignore_settings_file
        /// [`DSC_IGNORE_SETTINGS_FILE`]: crate::settings::sources::EnvironmentData::dsc_ignore_settings_file
        /// [`--ignore-settings-file`]: crate::settings::sources::CommandLineData::ignore_settings_file
        /// [`forbid_ignore_settings_file`]: crate::settings::sources::PolicyFileData::forbid_ignore_settings_file
        struct IgnoreSettingsFileField;
    },
    code_default_const: {
        /// Defines the code default value for the `ignore_settings_file` field.
        CODE_DEFAULT_IGNORE_SETTINGS_FILE = false;
    },
);

