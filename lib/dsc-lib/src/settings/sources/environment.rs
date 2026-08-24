//! Defines the `DscSettingsEnvironmentData` struct, which represents the DSC settings that can be
//! configured via environment variables.
//! 
//! When defining a new environment variable for a DSC setting, follow these guidelines:
//! 
//! 1. Ensure that the field is defined in the [`fields`] module following that module guidance.
//! 1. If the field is defined as a new type in the [`fields`] module, ensure that the type
//!    implements the [`FromStr`] trait to enable parsing from a string.
//! 1. Determine the appropriate name for the environment variable:
//! 
//!    - Always prefix the name with `DSC_` and use `SCREAMING_SNAKE_CASE`.
//!    - If the field is defined as a command line argument, use the long name of the argument, like
//!      `DSC_TRACE_LEVEL` for the `--trace-level` argument.
//!    - If the field isn't defined as a command line argument, choose a semantically meaningful
//!      name that clearly indicates the purpose of the environment variable. If you're not sure,
//!      default to using the field name for top-level fields. For nested fields, like
//!      `my_new_area.foo.bar`, use the field name with underscores, like `DSC_MY_NEW_AREA_FOO_BAR`.
//! 1. Add a field to the [`DscSettingsEnvironmentData`] struct in this module:
//! 
//!    - Name the field the same as the environment variable, using snake case. For example, the
//!      `DSC_TRACE_LEVEL` environment variable would correspond to a field named `dsc_trace_level`.
//!   - Define the field's type as `Option<T>`, where `T` is the type of the field defined in the
//!     [`fields`] module (or the externally defined type if the field doesn't require a new type).
//! 1. Implement a method to retrieve the value of the environment variable and parse it into the
//!    the appropriate type:
//!    - Name the method `get_env_<field_name>`, like `get_env_trace_level` for the
//!      `dsc_trace_level` field.
//!    - If the parsing for the field is infallible, define the return type as [`Option<T>`] to
//!      handle the case where the environment variable is not set.
//!    - If the parsing for the field is fallible, define the return type as
//!      [`Result<Option<T>, DscSettingsError>`] to surface parsing errors.
//! 1. Update the [`from_env()`] method to call the new `get_env_<field_name>` method and set the
//!    corresponding field in the [`DscSettingsEnvironmentData`] struct.
//! 
//!    If the method is infallible, you can just set the field to the return value of the method.
//!    For example, if the new variable is `DSC_NEW_FIELD`, you would add the following snippet:
//! 
//!    ```ignore
//!    dsc_new_field: Self::get_env_new_field(),
//!    ```
//! 
//!    If the method is fallible, set the field to a match statement that handles the [`Ok`] and
//!    [`Err`] cases, emitting a warning for the invalid value and setting the field to [`None`]
//!    in the [`Err`] case. For example, if the new variable is `DSC_NEW_FIELD`, you would add the
//!    following snippet:
//! 
//!    ```ignore
//!    dsc_new_field: match Self::get_env_new_field() {
//!       Ok(value) => value,
//!      Err(err) => {
//!         warn!("ignoring invalid DSC_NEW_FIELD environment variable: {}", err);
//!         None
//!       }
//!    },
//!    ```
//! 
//! 1. Update the [`try_from_env()`] method to call the new `get_env_<field_name>` method and set
//!    the corresponding field in the [`DscSettingsEnvironmentData`] struct.
//! 
//!    If the method is infallible, you can just set the field to the return value of the method.
//!    For example, if the new variable is `DSC_NEW_FIELD`, you would add the following snippet:
//! 
//!    ```ignore
//!    data.dsc_new_field = Self::get_env_new_field();
//!    ```
//! 
//!    If the method is fallible, use a match statement to handle the [`Ok`] and [`Err`] cases,
//!    pushing any errors to the `errors` vector. For example, if the new variable is
//!    `DSC_NEW_FIELD`, you would add the following snippet:
//! 
//!    ```ignore
//!    match Self::get_env_new_field() {
//!        Ok(value) => data.dsc_new_field = value,
//!        Err(err) => errors.push(err),
//!    };
//!    ```

use std::{path::PathBuf, str::FromStr};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tracing::warn;

use crate::settings::{DscSettingsError, TraceFormatField, TraceLevelField};

#[derive(Default, Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub struct DscSettingsEnvironmentData {
    /// `DSC_TRACE_LEVEL` - Defines the trace level to use.
    /// 
    /// This environment variable can be set to one of the following values (case insensitive):
    /// 
    /// - [`error`] - Only emit error messages.
    /// - [`warn`] - Only emit warning and error messages.
    /// - [`info`] - Only emit informational, warning, and error messages.
    /// - [`debug`] - Emit all messages except for trace messages.
    /// - [`trace`] - Emit all messages, including trace messages.
    /// 
    /// Setting this environment variable to an invalid value will raise an [`InvalidTraceLevel`]
    /// error when initializing the settings. DSC will raise a warning and ignore the environment
    /// variable when loading the environment settings data.
    /// 
    /// [`error`]: crate::settings::TraceLevelField::Error
    /// [`warn`]: crate::settings::TraceLevelField::Warn
    /// [`info`]: crate::settings::TraceLevelField::Info
    /// [`debug`]: crate::settings::TraceLevelField::Debug
    /// [`trace`]: crate::settings::TraceLevelField::Trace
    /// [`InvalidTraceLevel`]: crate::settings::DscSettingsError::InvalidTraceLevel
    pub dsc_trace_level: Option<TraceLevelField>,
    /// `DSC_TRACE_FORMAT` - Defines the trace format to use.
    /// 
    /// This environment variable can be set to one of the following values (case insensitive):
    /// 
    /// - [`default`] - Emit trace messages in the default format, which is a human-readable format
    ///   that includes the timestamp, log level, and message.
    /// - [`json`] - Emit trace messages in JSON format.
    /// - [`plaintext`] - Emit trace messages in plain text format.
    /// 
    /// Setting this environment variable to an invalid value will result in a
    /// [`DscSettingsError::InvalidTraceFormat`] error when initializing the settings. DSC will
    /// raise a warning and ignore the environment variable when loading the environment settings
    /// data.
    /// 
    /// [`default`]: crate::settings::TraceFormatField::Default
    /// [`json`]: crate::settings::TraceFormatField::Json
    /// [`plaintext`]: crate::settings::TraceFormatField::Plaintext
    pub dsc_trace_format: Option<TraceFormatField>,
    /// `DSC_RESOURCE_PATH` - Defines a list of paths to use when searching for DSC resource,
    /// extension, and other manifests.
    /// 
    /// When defined, DSC will search for resources, extensions, and other manifests in the
    /// specified paths. Effectively, this environment maps to the following settings in the
    /// [`DscPreferenceFileData`] struct:
    /// 
    /// ```ignore
    /// directories: dsc_resource_path
    /// ```
    /// 
    /// If either the [`restricted`] or [`append_env_path`] settings are defined in the [`Policy`],
    /// [`Machine`], [`User`], or [`Workspace`] scopes, the behavior of this environment variable
    /// may be affected. See the documentation for those settings for more information.
    /// 
    /// If this environment variable is defined with the [`DSC_RESTRICTED_PATH`] environment
    /// variable, the value of that variable takes precedence.
    /// 
    /// [`restricted`]: crate::settings::ResourcePathFileData::restricted
    /// [`append_env_path`]: crate::settings::ResourcePathFileData::append_env_path
    /// [`Policy`]: crate::settings::DscSettingsScope::Policy
    /// [`Machine`]: crate::settings::DscSettingsScope::Machine
    /// [`User`]: crate::settings::DscSettingsScope::User
    /// [`Workspace`]: crate::settings::DscSettingsScope::Workspace
    /// [`DSC_RESTRICTED_PATH`]: Self::dsc_restricted_path
    /// [`DscPreferenceFileData`]: crate::settings::DscPreferenceFileData
    pub dsc_resource_path: Option<Vec<PathBuf>>,
    /// `DSC_RESTRICTED_PATH` - Defines a list of paths to use when searching for and invoking
    /// resources, extensions, and other executables.
    /// 
    /// When defined, DSC will only search for resources, extensions, and other executables in the
    /// specified paths. DSC will _not_ allow invoking any executables outside of the specified
    /// paths. Effectively, this environment maps to the following settings in the
    /// [`DscPreferenceFileData`] struct:
    /// 
    /// ```ignore
    /// directories: dsc_restricted_path
    /// restricted: true
    /// ```
    /// 
    /// This environment variable should be defined as a list of paths separated by the
    /// platform-specific path separator (`;` on Windows, `:` on Unix-like systems).
    /// 
    /// If this environment variable is defined with the [`DSC_RESOURCE_PATH`] environment
    /// variable, the value of this variable takes precedence.
    /// 
    /// [`DSC_RESOURCE_PATH`]: Self::dsc_resource_path
    /// [`DscPreferenceFileData`]: crate::settings::DscPreferenceFileData
    pub dsc_restricted_path: Option<Vec<PathBuf>>,
    /// `DSC_IGNORE_SETTINGS_FILE` - Indicates whether to ignore settings files.
    /// 
    /// When this environment variable is set to `true` or `1`, DSC will not automatically load
    /// settings files. When resolving settings, DSC will ignore the settings files even if they
    /// were manually loaded. This effectively skips processing the [`Machine`], [`User`], and
    /// [`Workspace`] settings scopes.
    /// 
    /// When this environment variable is set to `false` or `0`, DSC will load and resolve settings
    /// files as normal. This is the default behavior when the environment variable is not set.
    /// 
    /// The [`Policy`] scope is always processed regardless of this setting.
    /// 
    /// This environment variable can be overridden by the [`--ignore-settings-file`] CLI argument,
    /// which has a higher precedence.
    /// 
    /// [`Machine`]: crate::settings::DscSettingsScope::Machine
    /// [`User`]: crate::settings::DscSettingsScope::User
    /// [`Workspace`]: crate::settings::DscSettingsScope::Workspace
    /// [`Policy`]: crate::settings::DscSettingsScope::Policy
    /// [`--ignore-settings-file`]: crate::settings::DscSettingsCliData::ignore_settings_file
    pub dsc_ignore_settings_file: Option<bool>,
}

impl DscSettingsEnvironmentData {
    pub const DSC_TRACE_LEVEL_ENV_VAR: &str = "DSC_TRACE_LEVEL";
    pub const DSC_TRACE_FORMAT_ENV_VAR: &str = "DSC_TRACE_FORMAT";
    pub const DSC_RESOURCE_PATH_ENV_VAR: &str = "DSC_RESOURCE_PATH";
    pub const DSC_RESTRICTED_PATH_ENV_VAR: &str = "DSC_RESTRICTED_PATH";
    pub const DSC_IGNORE_SETTINGS_FILE_ENV_VAR: &str = "DSC_IGNORE_SETTINGS_FILE";
    /// Retrieves the value of the `DSC_TRACE_LEVEL` environment variable and parses it into a [`TraceLevelField`].
    /// 
    /// # Returns
    /// 
    /// - [`Some`] [`TraceLevelField`] if the environment variable is set and valid.
    /// - [`None`] if the environment variable is not set.
    /// 
    /// # Errors
    /// 
    /// If the environment variable is set but contains an invalid value, this function returns a
    /// [`DscSettingsError::InvalidTraceLevel`] error.
    pub fn get_env_trace_level() -> Result<Option<TraceLevelField>, DscSettingsError> {
        let Some(level) = std::env::var(Self::DSC_TRACE_LEVEL_ENV_VAR).ok() else {
            return Ok(None)
        };

        match TraceLevelField::from_str(&level) {
            Ok(trace_level) => Ok(Some(trace_level)),
            Err(source ) => {
                Err(DscSettingsError::LoadEnvironmentError { 
                    env_var: Self::DSC_TRACE_LEVEL_ENV_VAR,
                    source: Box::new(source),
                })
            }
        }
    }
    /// Retrieves the value of the `DSC_TRACE_FORMAT` environment variable and parses it into a [`TraceFormatField`].
    /// 
    /// # Returns
    /// 
    /// - [`Some`] [`TraceFormatField`] if the environment variable is set and valid.
    /// - [`None`] if the environment variable is not set.
    /// 
    /// # Errors
    /// 
    /// If the environment variable is set but contains an invalid value, this function returns a
    /// [`DscSettingsError::InvalidTraceFormat`] error.
    pub fn get_env_trace_format() -> Result<Option<TraceFormatField>, DscSettingsError> {
        let Some(format) = std::env::var(Self::DSC_TRACE_FORMAT_ENV_VAR).ok() else {
            return Ok(None)
        };

        match TraceFormatField::from_str(&format) {
            Ok(trace_format) => Ok(Some(trace_format)),
            Err(source) => Err(DscSettingsError::LoadEnvironmentError {
                env_var: Self::DSC_TRACE_FORMAT_ENV_VAR,
                source: Box::new(source),
            }),
        }
    }
    /// Retrieves the value of the `DSC_RESOURCE_PATH` environment variable and parses it into a
    /// vector of [`PathBuf`].
    /// 
    /// # Returns
    /// 
    /// - [`Some`] [`Vec<PathBuf>`] if the environment variable is set.
    /// - [`None`] if the environment variable is not set.
    pub fn get_env_resource_path() -> Option<Vec<PathBuf>> {
        let Some(path) = std::env::var(Self::DSC_RESOURCE_PATH_ENV_VAR).ok() else {
            return None;
        };

        Some(std::env::split_paths(&path).collect::<Vec<PathBuf>>())
    }
    /// Retrieves the value of the `DSC_RESTRICTED_PATH` environment variable and parses it into a
    /// vector of [`PathBuf`].
    /// 
    /// # Returns
    /// 
    /// - [`Some`] [`Vec<PathBuf>`] if the environment variable is set.
    /// - [`None`] if the environment variable is not set.
    pub fn get_env_restricted_path() -> Option<Vec<PathBuf>> {
        let Some(path) = std::env::var(Self::DSC_RESTRICTED_PATH_ENV_VAR).ok() else {
            return None;
        };

        Some(std::env::split_paths(&path).collect::<Vec<PathBuf>>())
    }

    /// Retrieves the value of the `DSC_IGNORE_SETTINGS_FILE` environment variable and parses it
    /// into a boolean.
    /// 
    /// # Parsing
    /// 
    /// This function interprets the following values case insensitively:
    /// 
    /// - "true" or "1" as `true`
    /// - "false" or "0" as `false`
    /// 
    /// Any other value is invalid.
    /// 
    /// # Returns
    /// 
    /// - [`Some`] `true` if the environment variable is set to "true" or "1".
    /// - [`Some`] `false` if the environment variable is set to "false" or "0".
    /// - [`None`] if the environment variable is not set.
    /// 
    /// # Errors
    /// 
    /// If the environment variable is set but contains an invalid value, this function returns a
    /// [`DscSettingsError::InvalidIgnoreSettingsFileEnvVar`] error.
    pub fn get_env_ignore_settings_file() -> Result<Option<bool>, DscSettingsError> {
        let Some(value) = std::env::var(Self::DSC_IGNORE_SETTINGS_FILE_ENV_VAR).ok() else {
            return Ok(None);
        };

        match Self::parse_boolean_env_var(&value) {
            Ok(boolean_value) => Ok(Some(boolean_value)),
            Err(source) => Err(DscSettingsError::LoadEnvironmentError {
                env_var: Self::DSC_IGNORE_SETTINGS_FILE_ENV_VAR,
                source: Box::new(source),
            }),
        }
    }

    /// Creates a new instance of [`DscSettingsEnvironmentData`] by reading the relevant environment variables.
    /// 
    /// This function reads the following environment variables:
    /// - `DSC_TRACE_LEVEL`: The trace level to use.
    /// - `DSC_TRACE_FORMAT`: The trace format to use.
    /// - `DSC_RESOURCE_PATH`: A list of paths to use when searching for resources, separated by
    ///   the platform-specific path separator (`;` on Windows, `:` on Unix-like systems).
    /// - `DSC_RESTRICTED_PATH`: A list of paths to use when searching for and invoking resources,
    ///   extensions, and other executables, separated by the platform-specific path separator.
    /// - `DSC_IGNORE_SETTINGS_FILE`: Whether to ignore the settings file.
    /// 
    /// When an environment variable is not set or is invalid, the corresponding field will be `None`. This function
    /// emits warnings for any invalid environment variable values, but will not return an error. Use
    /// [`try_from_env()`] if you want to handle errors instead of emitting warnings and setting the field to `None`.
    /// 
    /// [`try_from_env()`]: Self::try_from_env
    pub fn from_env() -> Self {
        Self {
            dsc_trace_level: match Self::get_env_trace_level() {
                Ok(level) => level,
                Err(err) => {
                    warn!("ignoring invalid {} environment variable: {}", Self::DSC_TRACE_LEVEL_ENV_VAR, err);
                    None
                }
            },
            dsc_trace_format: match Self::get_env_trace_format() {
                Ok(format) => format,
                Err(err) => {
                    warn!("ignoring invalid {} environment variable: {}", Self::DSC_TRACE_FORMAT_ENV_VAR, err);
                    None
                }
            },
            dsc_resource_path: Self::get_env_resource_path(),
            dsc_restricted_path: Self::get_env_restricted_path(),
            dsc_ignore_settings_file: match Self::get_env_ignore_settings_file() {
                Ok(val) => val,
                Err(err) => {
                    warn!("ignoring invalid {} environment variable: {}", Self::DSC_IGNORE_SETTINGS_FILE_ENV_VAR, err);
                    None
                }
            },
        }
    }

    /// Creates a new instance of [`DscSettingsEnvironmentData`] by reading the relevant
    /// environment variables.
    /// 
    /// This function reads the same environment variables as [`from_env()`], but returns an error
    /// if any of them are defined with invalid values.
    /// 
    /// [`from_env()`]: Self::from_env
    pub fn try_from_env() -> Result<Self, DscSettingsError> {
        let mut errors = Vec::new();
        let mut data = Self::default();

        match Self::get_env_trace_level() {
            Ok(level) => data.dsc_trace_level = level,
            Err(err) => errors.push(err),
        }
        match Self::get_env_trace_format() {
            Ok(format) => data.dsc_trace_format = format,
            Err(err) => errors.push(err),
        }
        data.dsc_resource_path = Self::get_env_resource_path();
        data.dsc_restricted_path = Self::get_env_restricted_path();
        match Self::get_env_ignore_settings_file() {
            Ok(val) => data.dsc_ignore_settings_file = val,
            Err(err) => errors.push(err),
        }

        if errors.is_empty() {
            Ok(data)
        } else {
            Err(DscSettingsError::LoadEnvironmentMultipleErrors(errors))
        }
    }

    /// Parses a boolean environment variable value.
    /// 
    /// This function interprets the following values case insensitively:
    /// - `"true"` or `"1"` as `true`
    /// - `"false"` or `"0"` as `false`
    /// 
    /// # Errors
    /// 
    /// If the value isn't one of the recognized boolean representations, this function returns a
    /// [`ParseBooleanEnvVarError`].
    /// 
    /// # Examples
    /// 
    /// The following example shows how different string values are parsed into boolean values:
    /// 
    /// ```rust
    /// assert_eq!(DscSettingsEnvironmentData::parse_boolean_env_var("true"), Ok(true));
    /// assert_eq!(DscSettingsEnvironmentData::parse_boolean_env_var("True"), Ok(true));
    /// assert_eq!(DscSettingsEnvironmentData::parse_boolean_env_var("TRUE"), Ok(true));
    /// assert_eq!(DscSettingsEnvironmentData::parse_boolean_env_var("tRuE"), Ok(true));
    /// assert_eq!(DscSettingsEnvironmentData::parse_boolean_env_var("1"), Ok(true));
    /// assert_eq!(DscSettingsEnvironmentData::parse_boolean_env_var("false"), Ok(false));
    /// assert_eq!(DscSettingsEnvironmentData::parse_boolean_env_var("False"), Ok(false));
    /// assert_eq!(DscSettingsEnvironmentData::parse_boolean_env_var("FALSE"), Ok(false));
    /// assert_eq!(DscSettingsEnvironmentData::parse_boolean_env_var("FfAlSe"), Ok(false));
    /// assert_eq!(DscSettingsEnvironmentData::parse_boolean_env_var("0"), Ok(false));
    /// assert!(DscSettingsEnvironmentData::parse_boolean_env_var("invalid").is_err());
    /// ```
    /// 
    /// [`ParseBooleanEnvVarError`]: DscSettingsError::ParseBooleanEnvVarError
    pub fn parse_boolean_env_var(value: &str) -> Result<bool, DscSettingsError> {
        match value.to_lowercase().as_str() {
            "true" | "1" => Ok(true),
            "false" | "0" => Ok(false),
            _ => Err(DscSettingsError::ParseBooleanEnvVarError {
                value: value.to_string(),
            }),
        }
    }
}
