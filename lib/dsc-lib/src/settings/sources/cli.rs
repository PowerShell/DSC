
//! This module defines the `DscSettingsCliData` struct, which represents the command line
//! arguments related to DSC settings.
//! 
//! This documentation provides guidance for defining new command line arguments related to DSC
//! settings. When adding a new command line argument, follow this guidance:
//! 
//! 1. Ensure that the field is defined in the [`fields`] module following that module guidance.
//! 1. Add the field to the [`DscSettingsCliData`] struct in this module:
//! 
//!    - Name the field the same as the command line argument's long name, using snake case. For
//!      example, the `--trace-level` command line argument would correspond to a field named
//!      `trace_level`.
//!    - Define the field's type as `Option<T>`, where `T` is the type of the field defined in the
//!     [`fields`] module (or the externally defined type if the field doesn't require a new type).
//! 1. Update the [`DscSettings::resolve_cli_data`] method to appropriately resolve the field.
//! 
//!    For example, when defining a setting for a top-level field named `new_area`, you would add
//!    the following snippet to the `resolve_cli_data` method:
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
//!    When defining a setting for a nested leaf field, you would add a similar snippet, but with
//!    the appropriate dot notation to access the nested field. For example, if the field is
//!    `new_area.foo.bar`, you would add the following snippet:
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
//!    - If the argument is a boolean flag, ensure that the Clap attribute defines the following
//!      fields:
//! 
//!      - `num_args=0..=1` - Makes the argument accept zero or one value. This allows the argument
//!        to be specified as a flag without an explicit value, or with an explicit value of `true`
//!        or `false`.
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
//!      In this case, even though the user specified the environment variable to ignore settings
//!      files, the argument parser interprets the `--ignore-settings-file` argument as `false` and
//!      DSC will load settings files during resolution.
//! 
//!      When the argument is defined with the above attributes, the parser can indicate that the
//!      argument wasn't specified, and DSC will correctly resolve the setting to `true` based on
//!      the environment variable.
//! 
//!      This also enables the user to effectively override the environment variable with the
//!      `--ignore-settings-file=false` argument.
//!    - If the argument is for a defined type, ensure that _either_:
//! 
//!      1. The CLI code defines the `From` trait to convert between the CLI argument type and the
//!         type defined in the [`fields`] module, or
//!      1. The CLI code uses the type defined in the [`fields`] module for the argument.
//! 1. Ensure that the CLI call to initialize the settings includes the new argument in the `DscSettingsCliData`
//!    struct.

use serde::{Deserialize, Serialize};
use schemars::JsonSchema;

use crate::settings::{TraceFormatField, TraceLevelField};

/// Represents the command line arguments related to DSC settings.
/// 
/// DSC defines several global command line arguments that can be used to override settings
/// defined in preference files or environment variables. This struct captures the values of those
/// command line arguments.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct DscSettingsCliData {
    /// Defines the trace level to use.
    /// 
    /// Retrieved from the global `--trace-level` command line argument.
    pub trace_level: Option<TraceLevelField>,
    /// Defines the trace format to use.
    /// 
    /// Retrieved from the global `--trace-format` command line argument.
    pub trace_format: Option<TraceFormatField>,
    /// Whether to ignore settings files.
    /// 
    /// Retrieved from the global `--ignore-settings-file` command line argument.
    /// 
    /// When this is set to `true`, DSC will ignore all settings files, including machine, user,
    /// and workspace settings files. When resolving settings, DSC will ignore the settings files
    /// and only consider the following sources, in order of precedence:
    /// 
    /// 1. The system policy file, if it exists.
    /// 2. The command line arguments, if they're provided.
    /// 3. The environment variables, if they're set.
    /// 4. The code defaults, which are the built-in default values for each setting.
    pub ignore_settings_file: Option<bool>,
}
