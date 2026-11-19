// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::schemas::dsc_repo::DscRepoSchema;

super::macros::define_boolean_field! {
    base_name: "forbidIgnoreSettingsFile",
    folder_path: "settings/fields",
    field_definition: {
        /// Indicates whether to allow users to ignore settings files.
        ///
        /// This setting field is only available in the [`Policy`] scope. The\
        /// default setting is `false`.
        ///
        /// When this setting is `false`, users can define the
        /// [`--ignore-settings-file`] CLI option or the [`DSC_IGNORE_SETTINGS_FILE`]
        /// environment variable to indicate that DSC shouldn't load and
        /// resolve preference settings files. For more information on ignoring
        /// settings files, see [`IgnoreSettingsFileField`].
        ///
        /// The [`Policy`] scope is always processed regardless of this
        /// setting. For more information on defining this setting in the
        /// policy file, see [`forbid_ignore_settings_file`] in the policy file
        /// documentation.
        ///
        /// [`Policy`]: crate::settings::DscSettingsScope::Policy
        /// [`--ignore-settings-file`]: crate::settings::sources::CommandLineData::ignore_settings_file
        /// [`DSC_IGNORE_SETTINGS_FILE`]: crate::settings::sources::EnvironmentData::dsc_ignore_settings_file
        /// [`IgnoreSettingsFileField`]: crate::settings::fields::IgnoreSettingsFileField
        /// [`forbid_ignore_settings_file`]: crate::settings::sources::PolicyFileData::forbid_ignore_settings_file
        struct ForbidIgnoreSettingsFileField;
    },
    code_default_const: {
        /// Defines the default value for the `forbid_ignore_settings_file`
        /// field in DSC settings.
        CODE_DEFAULT_FORBID_IGNORE_SETTINGS_FILE = false;
    },
}
