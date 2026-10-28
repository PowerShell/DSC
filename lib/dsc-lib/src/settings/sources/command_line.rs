// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! This module defines the [`CommandLineData`] struct, which represents the command line
//! arguments related to DSC settings.

use serde::Serialize;
use schemars::JsonSchema;

use crate::schemas::dsc_repo::{DscRepoSchema, schema_i18n};
use crate::settings::fields::*;

#[cfg(doc)]
pub(in crate::settings) mod _implementation_guidance {
    //! This documentation provides guidance for defining new command line arguments related to DSC
    //! settings. When adding a new command line argument, follow this guidance:
    //!
    //! 1. Ensure that the field is defined in the [`fields`] module following that
    //!    [module guidance](crate::settings::fields::_implementation_guidance).
    //! 1. Ensure that the field is defined in the [`DscSettingsResolved`] struct, following the
    //!    [module guidance](crate::settings::resolved::_implementation_guidance).
    //! 1. Add the field to the [`CommandLineData`] struct in this module:
    //!
    //!    - Name the field the same as the command line argument's long name, using snake case.
    //!      For example, the `--trace-level` command line argument would correspond to a field
    //!      named `trace_level`.
    //!    - Define the field's type as [`Option<T>`], where `T` is the type of the field defined
    //!      in the [`fields`] module (or the externally defined type if the field doesn't require
    //!      a new type).
    //!
    //! 1. Update the [`DscSettings::resolve_command_line()`] method to appropriately resolve the
    //!    field.
    //!
    //!    For example, when defining a setting for a top-level field named `new_area`, you would
    //!    add the following snippet to the `resolve_command_line()` method:
    //!
    //!    ```ignore
    //!    if let Some(value) = cli_data.new_area.as_ref() {
    //!        if resolving.new_area.scope < DscSettingsScope::CommandLine {
    //!            resolving.new_area = DscSettingsResolvedField::new(
    //!                value.clone(),
    //!                DscSettingsScope::CommandLine
    //!            );
    //!        }
    //!    }
    //!    ```
    //!
    //!    When defining a setting for a nested leaf field, you would add a similar snippet, but
    //!    with the appropriate dot notation to access the nested field. For example, if the field
    //!    is `new_area.foo.bar`, you would add the following snippet:
    //!
    //!    ```ignore
    //!    if let Some(value) = cli_data.new_area.foo.bar.as_ref() {
    //!        if resolving.new_area.foo.bar.scope < DscSettingsScope::CommandLine {
    //!            resolving.new_area.foo.bar = DscSettingsResolvedField::new(
    //!                value.clone(),
    //!                DscSettingsScope::CommandLine
    //!            );
    //!        }
    //!    }
    //!    ```
    //! 1. Ensure that the argument in DSC is defined in the CLI argument parser.
    //!
    //!    - If the argument is a boolean flag, ensure that the Clap attribute defines the
    //!      following fields:
    //!
    //!      - `num_args=0..=1` - Makes the argument accept zero or one value. This allows the
    //!        argument to be specified as a flag without an explicit value, or with an explicit
    //!        value of `true` or `false`.
    //!      - `default_missing_value="true"` - Ensures that if the argument is specified without a
    //!        value, it will be treated as `true`.
    //!      - `require_equals = true` - Ensures that if the argument is specified with a value, it
    //!        must be specified using an equals sign, like `--ignore-settings-file=true`.
    //!
    //!      This is necessary to distinguish between the argument not being specified and being
    //!      specified with a value of `false`. Otherwise, the CLI argument will _always_ supercede
    //!      lower precedence sources. For example, consider the `--ignore-settings-file` argument:
    //!
    //!      ```sh
    //!      DSC_IGNORE_SETTINGS_FILE=true dsc config get -f ./example.dsc.config.yaml
    //!      ```
    //!
    //!      In this case, even though the user specified the environment variable to ignore
    //!      settings files, the argument parser interprets the `--ignore-settings-file` argument
    //!      as `false` and DSC will load settings files during resolution.
    //!
    //!      When the argument is defined with the above attributes, the parser can indicate that
    //!      the argument wasn't specified, and DSC will correctly resolve the setting to `true`
    //!      based on the environment variable.
    //!
    //!      This also enables the user to effectively override the environment variable with the
    //!      `--ignore-settings-file=false` argument.
    //!    - If the argument is for a defined type, ensure that _either_:
    //!
    //!      1. The CLI code defines the `From` trait to convert between the CLI argument type and
    //!         the type defined in the [`fields`] module, or
    //!      1. The CLI code uses the type defined in the [`fields`] module for the argument.
    //!
    //! 1. Ensure that the CLI call to initialize the settings includes the new argument in the
    //!    [`CommandLineData`] struct.
    //!
    //! [`fields`]: crate::settings::fields
    //! [`CommandLineData`]: super::CommandLineData
    //! [`DscSettings::resolve_command_line()`]: crate::settings::DscSettings::resolve_command_line
    //! [`DscSettingsResolved`]: crate::settings::DscSettingsResolved
}

/// Represents the command line arguments related to DSC settings.
///
/// DSC defines several global command line arguments that can be used to
/// override settings defined in preference files or environment variables.
/// This struct captures the values of those command line arguments.
///
/// See the field documentation for more information about each of the settings
/// you can control with environment variables.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, JsonSchema, DscRepoSchema)]
#[serde(rename_all = "camelCase")]
#[dsc_repo_schema(base_name = "cli", folder_path = "settings/sources")]
#[schemars(
    title = schema_i18n!("title"),
    description = schema_i18n!("description"),
    extend(
        "$id" = CommandLineData::default_export_schema_id_uri(),
        "markdownDescription" = schema_i18n!("markdownDescription")
    )
)]
pub struct CommandLineData {
    /// Defines the trace level to use.
    ///
    /// Retrieved from the global `--trace-level` command line argument, this
    /// value maps to the [`tracing.level`] setting.
    ///
    /// It controls the minimum level of messages that DSC emits.
    ///
    /// [`tracing.level`]: crate::settings::fields::TracingLevelField
    #[serde(rename = "--trace-level", skip_serializing_if = "Option::is_none")]
    #[schemars(
        title = schema_i18n!("trace_level.title"),
        description = schema_i18n!("trace_level.description"),
        extend(
            "markdownDescription" = schema_i18n!("trace_level.markdownDescription")
        )
    )]
    pub trace_level: Option<TracingLevelField>,

    /// Defines the trace format to use.
    ///
    /// Retrieved from the global `--trace-format` command line argument, this
    /// value maps to the [`tracing.format`] setting.
    ///
    /// It controls the format that DSC uses when emitting messages.
    ///
    /// [`tracing.format`]: crate::settings::fields::TracingFormatField
    #[serde(rename = "--trace-format", skip_serializing_if = "Option::is_none")]
    #[schemars(
        title = schema_i18n!("trace_format.title"),
        description = schema_i18n!("trace_format.description"),
        extend(
            "markdownDescription" = schema_i18n!("trace_format.markdownDescription")
        )
    )]
    pub trace_format: Option<TracingFormatField>,

    /// Whether to ignore settings files.
    ///
    /// Retrieved from the global `--ignore-settings-file` command line
    /// argument, this value maps to the [`ignore_settings_file`] setting.
    ///
    /// When this is set to `true`, DSC will ignore all settings files,
    /// including machine, user, and workspace settings files. When resolving
    /// settings, DSC will ignore the settings files and only consider the
    /// following sources, in order of precedence:
    ///
    /// 1. The system policy file, if it exists.
    /// 2. The command line arguments, if they're provided.
    /// 3. The environment variables, if they're set.
    /// 4. The code defaults, which are the built-in default values for each
    ///    setting.
    ///
    /// Defining this setting as `true` will raise a warning if the policy
    /// file defines [`forbid_ignore_settings_file`] as `true`.
    ///
    /// [`ignore_settings_file`]: crate::settings::fields::IgnoreSettingsFileField
    /// [`forbid_ignore_settings_file`]: crate::settings::fields::ForbidIgnoreSettingsFileField
    #[serde(rename = "--ignore-settings-file", skip_serializing_if = "Option::is_none")]
    #[schemars(
        title = schema_i18n!("ignore_settings_file.title"),
        description = schema_i18n!("ignore_settings_file.description"),
        extend(
            "markdownDescription" = schema_i18n!("ignore_settings_file.markdownDescription")
        )
    )]
    pub ignore_settings_file: Option<IgnoreSettingsFileField>,
}

/// Private API
impl CommandLineData {
    /// Indicates whether any of the fields were populated from command line
    /// arguments.
    ///
    /// This method is used when deciding whether to serialize the command line
    /// data and for processing to enable skipping when no fields were defined.
    ///
    /// # Returns
    ///
    /// `true` if any of the command line data fields are populated, otherwise
    /// `false`.
    pub(crate) fn any_defined(&self) -> bool {
        self.trace_level.is_some() ||
        self.trace_format.is_some() ||
        self.ignore_settings_file.is_some()
    }
}