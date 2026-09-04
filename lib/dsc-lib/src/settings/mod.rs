// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! Defines how to load and resolve settings for DSC.
//! 
//! DSC uses a layered approach to settings, composing a set of resolved settings from multiple
//! sources. The [`DscSettings`] struct represents the complete model of settings, including all
//! sources and the resolved effective settings.
//! 
//! 
//! When DSC starts, it loads settings from various [scopes]. Every scope maps to a specific source.
//! After loading the settings from all sources, DSC resolves the effective settings, starting with
//! the default settings defined in the source code and then ensuring that the resolved settings
//! reflect the precedence scope that defined each setting.
//! 
//! # Scopes and sources
//! 
//! The following table defines the scopes and their corresponding sources, in order of precedence
//! from lowest to highest, where later scopes override settings defined in earlier scopes:
//! 
//! | Scope           | Source                         | Description |
//! |:---------------:|:------------------------------:|-------------|
//! | [`Default`]     | [`DSC_SETTINGS_CODE_DEFAULTS`] | The hardcoded default settings defined in the source code. |
//! | [`User`]        | [`DscPreferenceFileData`]      | A settings file defining settings for the current user. |
//! | [`Machine`]     | [`DscPreferenceFileData`]      | A settings file defining settings for every user on the machine. |
//! | [`Workspace`]   | [`DscPreferenceFileData`]      | The settings loaded from the workspace settings file, if it exists. |
//! | [`Environment`] | [`DscSettingsEnvironmentData`] | The settings loaded from the environment variables, if they are set. |
//! | [`CommandLine`] | [`DscSettingsCliData`]         | The settings loaded from the command line arguments, if they are provided. |
//! | [`Policy`]      | [`DscPolicyFileData`]          | The settings loaded from the policy settings file, if it exists. |
//! 
//! Every available setting for DSC can be defined in the [`Policy`] scope, which has the highest
//! precedence and can't be overridden by any other scope. Users can define a policy file to enforce
//! specific settings for all users on a machine.
//! 
//! The [`Machine`], [`User`], and [`Workspace`] scopes all support users defining a preference file
//! that contains settings for DSC. Preference files use the same data structure as policy files,
//! but not every field in a policy file is supported in a preference file. For example, the
//! [`forbid_ignore_settings_file`] field is only supported in a policy file because it controls
//! whether DSC should load and resolve settings from the preference files. Defining this field in
//! a preference file wouldn't make sense.
//! 
//! The [`Environment`] scope allows users to define settings for DSC as environment variables. Not
//! every setting is supported as an environment variable,but every environment variable setting
//! maps to either a specific field in the policy file or a combination of fields in the policy
//! file. For example, users can define either the [`DSC_RESOURCE_PATH`] or the
//! [`DSC_RESTRICTED_PATH`] environment variable as a collection of directories separated by the
//! platform specific path separator for `PATH`-like environment variables. When DSC processes these
//! environment variables, DSC splits the values using the platform specific path separator and uses
//! the resulting collection to populate the [`resource_path.directories`] field in the resolved
//! settings. Additionally, the [`DSC_RESTRICTED_PATH`] environment variable populates the
//! [`resource_path.restricted`] field in the resolved settings.
//! 
//! The [`CommandLine`] scope allows users to define settings for DSC as global command line
//! arguments. Only a few settings are supported as command line arguments to minimize the clutter
//! and cognitive load when invoking DSC commands.
//! 
//! ## Loading settings
//! 
//! With the exception of the [`Default`] and [`CommandLine`] scopes, DSC automatically attempts to
//! load settings from all other scopes during initialization. The [`Default`] scope is statically
//! defined in the source code and the [`CommandLine`] scope must be passed to the
//! [`new_with_command_line()`] constructor.
//! 
//! The other scopes are loaded automatically by calling either the [`load()`] or [`try_load()`]
//! methods. The [`load()`] method attempts to load all sources and converts any errors into
//! warnings, while the [`try_load()`] method attempts to load all sources and collects any errors
//! into a single error value.
//! 
//! When loading settings for the [`Policy`], [`Machine`], [`User`], and [`Workspace`] scopes, DSC
//! checks whether the corresponding settings file exists. If the file exists, DSC tries to read
//! and parse the file into the appropriate data structure ([`DscPolicyFileData`] for [`Policy`]
//! and [`DscPreferenceFileData`] for the others).
//! 
//! When loading settings for the [`Environment`] scope, DSC checks whether the relevant environment
//! variables are defined and reads their values if they exist, parsing them into the appropriate
//! data type for the backing field in [`DscSettingsEnvironmentData`].
//! 
//! # Resolving settings
//! 
//! DSC represents the resolved effective settings in an instance of [`DscSettingsResolved`]. Every
//! field in this struct is either a _leaf_ field where the type is [`DscSettingsResolvedField`], or
//! a _container_ field where the type is another struct with its own leaf and container fields. The
//! leaf fields contain the final value for that setting and the scope that defined it, so users
//! can understand which settings were defined in which scope with what value.
//! 
//! When resolving settings, DSC supports overriding any _leaf_ field in the settings data structure
//! when that field is defined in a higher-precedence scope. It doesn't replace the entire container
//! with the value from the higher-precedence scope. This ensures that users can define only the
//! specific settings they want to override in a higher-precedence scope without needing to redefine
//! the entire collection of settings in that scope.
//! 
//! For example, if the machine settings file defines both [`tracing.level`] and [`tracing.format`],
//! and the user settings file defines only [`tracing.format`], the final resolved settings will 
//! match the following YAML snippet:
//! 
//! ```yaml
//! tracing:
//!   level:
//!     scope: machine
//!     value: <value from machine settings>
//!   format:
//!     scope: user
//!     value: <value from user settings>
//! ```
//! 
//! ## Resolution steps
//! 
//! After loading settings from all sources, DSC  follows these steps to resolve the effective
//! settings:
//! 
//! 1. Initialize the resolved settings with the default settings defined in the source code.
//! 1. Any settings defined in the [`Policy`] scope override the code defaults. These settings are
//!    applied first because they have the highest precedence and can't be overridden by any other
//!    scope.
//! 1. Process the settings defined in the remaining scopes in precedence order, overriding any
//!    settings from the code defaults or prior scopes unless they were defined in the [`Policy`]
//!    scope. The order of precedence for these scopes is as follows:
//! 
//!   - [`Machine`]
//!   - [`User`]
//!   - [`Workspace`]
//!   - [`Environment`]
//!   - [`CommandLine`]
//! 
//! After the initial resolution, DSC caches the resolved settings in a private field of the
//! [`DscSettings`] instance. Repeated access to the resolved settings with [`resolved()`] will use
//! the cached values, ensuring efficient retrieval without reprocessing every loaded source.
//! 
//! # Available settings
//! 
//! The following sections provide an overview of the available settings, how they affect DSC, and
//! how to define them in the various scopes.
//! 
//! ## Ignoring settings files
//! 
//! By default, DSC automatically laods and resolves settings from the [`Machine`], [`User`], and
//! [`Workspace`] settings files if they exist.
//! 
//! DSC defines two opposing settings that control whether DSC should load and resolve settings
//! from the [`Machine`], [`User`], and [`Workspace`] settings files. By default, DSC loads and
//! resolves settings from these files if they exist.
//! 
//! A user can control whether DSC should ignore the preference settings files by:
//! 
//! 1. Defining the [`ignore_settings_file`] field in the [`Policy`] settings file.
//! 1. Defining the [`DSC_IGNORE_SETTINGS_FILE`] environment variable.
//! 1. Specifying the [`--ignore-settings-file`] global command line argument.
//! 
//! When the resolved value for the field is `true`, DSC will ignore the preference settings files
//! when loading and resolving settings.
//! 
//! Additionally, a user can forbid ignoring the preference settings files by defining the
//! [`forbid_ignore_settings_file`] field in the [`Policy`] settings file. When this field is
//! defined as `true`, DSC will always load and resolve settings from the preference files if
//! they exist.
//! 
//! ## Tracing settings
//! 
//! DSC emits messages to `stderr` to provide information about the execution lifecycle. Every
//! message is emitted with a different severity level. DSC supports emitting trace messages in
//! multiple formats.
//! 
//! ### Trace level
//! 
//! By default, DSC emits only [`Warn`] and [`Error`] messages to stderr.
//! 
//! Users can define what level of messages to emit by:
//! 
//! 1. Defining the [`tracing.level`] field in the [`Policy`], [`Machine`], [`User`], or
//!   [`Workspace`] settings files.
//! 1. Defining the [`DSC_TRACE_LEVEL`] environment variable.
//! 1. Specifying the [`--trace-level`] global command line argument.
//! 
//! When the trace level is set, DSC will emit messages of the specified severity and higher to
//! stderr. For example, if the trace level is set to [`Info`], DSC will emit messages with
//! [`Info`], [`Warn`], and [`Error`] severity levels. DSC won't emit messages with [`Debug`]
//! or [`Trace`] severity levels.
//! 
//! ### Trace format
//! 
//! By default, DSC emits messages to stderr as colorized human-readable text.
//! 
//! Users can override the default trace format by:
//! 
//! 1. Defining the [`tracing.format`] field in the [`Policy`], [`Machine`], [`User`], or
//!   [`Workspace`] settings files.
//! 1. Defining the [`DSC_TRACE_FORMAT`] environment variable.
//! 1. Specifying the [`--trace-format`] global command line argument.
//! 
//! The available trace formats include:
//! 
//! - `default`: DSC emits messages to stderr in the default colorized human-readable text format.
//! - `plaintext`: DSC emits messages to stderr as human readable text without colorization.
//! - `json`: DSC emits messages to stderr as compressed JSON objects.
//! 
//! ## Resource path settings
//! 
//! DSC discovers manifest files by searching a collection of directories for files with known
//! extensions. By default, DSC only searches the `PATH` environment variable and the directory
//! that DSC is installed in.
//! 
//! When invoking commands for a manifest, DSC supports invoking commands that exist in `PATH` or
//! that can be resolved relative to the manifest file.
//! 
//! Unlike the tracing settings, which are independent of each other, the resource path settings
//! define how DSC discovers manifest files and invokes commands:
//! 
//! - When [`directories`] is resolved, DSC always searches the specified directories for manifests
//!   and can invoke commands from those directories.
//! - When [`append_env_path`] is resolved as `true`, DSC appends the `PATH` environment variable
//!   to the resolved value for [`directories`] when searching for manifests.
//! - When [`restricted`] is resolved as `true`, DSC restricts the paths from which it can invoke
//!   binaries to the resolved value for [`directories`]. When this setting is `true` it effectively
//!   ignores the `append_env_path` setting and doesn't append the `PATH` environment variable to
//!   set of directories. DSC will only discover and invoke commands from resolved value of 
//!   [`directories`].
//! 
//! <div class="warning">
//! <details open><summary>Restricted path behavior</summary>
//! 
//! When you set [`restricted`] to `true`, DSC will _only_ discover and invoke commands from the
//! resolved value of [`directories`]. This explicitly _does not_ consider the `PATH` environment
//! variable or the DSC installation directory.
//! 
//! To use built-in resources and extensions, you need to ensure that [`directories`] includes the
//! DSC installation directory.
//! 
//! To use resources and extensions that require invoking commands that aren't adjacent to the
//! manifest file, you need to ensure that [`directories`] includes the directories containing
//! those commands.
//! 
//! </details>
//! </div>
//! 
//! To configure how DSC discovers manifests and invokes commands, users can:
//! 
//! 1. Define the [`resourcePath.directories`], [`resourcePath.appendEnvPath`], and
//!   [`resourcePath.restricted`] fields in the [`Policy`], [`Machine`], [`User`], or [`Workspace`]
//!   settings files.
//! 1. Define the [`DSC_RESOURCE_PATH`] environment variable as a string containing a collection of
//!    directories separated by the platform specific path separator for `PATH`-like environment
//!    variables. This effectively populates the [`directories`] field in the resolved settings.
//! 1. Define the [`DSC_RESTRICTED_PATH`] environment variable as a string containing a collection
//!    of directories separated by the platform specific path separator for `PATH`-like environment
//!    variables. This effectively populates the [`directories`] field in the resolved settings and
//!    sets the [`restricted`] field in the resolved settings to `true`.
//! 
//! ### Examples
//! 
//! The following examples clarify the effective behavior of DSC depending on the resource path
//! settings. Each scenario shows the effective resolved settings for the resource path as a YAML
//! snippet before explaining how DSC behaves when discovering manifests and invoking commands.
//! 
//! 1. ```yaml
//!    resourcePath:
//!      directories: []
//!      append_env_path: true
//!      restricted: false
//!    ```
//! 
//!    DSC searches the directories in the `PATH` environment variable and the directory that DSC
//!    is installed in for manifests. When invoking commands for a manifest, DSC can invoke
//!    commands that exist in `PATH`, the DSC installation directory, and relative to any
//!    discovered manifest file.
//! 1. ```yaml
//!    resourcePath:
//!      directories: ["D:\infra\resources", "D:\infra\tools"]
//!      append_env_path: true
//!      restricted: false
//!    ```
//! 
//!    DSC searches the speciied directories, the directories in the `PATH` environment variable,
//!    and the directory that DSC is installed in for manifest files. When invoking commands for a
//!    manifest, DSC can invoke commands that exist those same directories and relative to any
//!    discovered manifest file.
//! 1. ```yaml
//!    resourcePath:
//!      directories: ["D:\infra\resources", "D:\infra\tools"]
//!      append_env_path: true
//!      restricted: true
//!    ```
//! 
//!    DSC _only_ searches the specified directories for manifest files and does not consider the
//!    `PATH` environment variable or the DSC installation directory. When invoking commands for a
//!    manifest, DSC can only invoke commands that exist in the specified directories. Attempting
//!    to invoke commands outside of the specified directories raises an error.
//! 
//! [scopes]: DscSettingsScope
//! [`Default`]: DscSettingsScope::Default
//! [`Machine`]: DscSettingsScope::Machine
//! [`User`]: DscSettingsScope::User
//! [`Workspace`]: DscSettingsScope::Workspace
//! [`Environment`]: DscSettingsScope::Environment
//! [`CommandLine`]: DscSettingsScope::CommandLine
//! [`Policy`]: DscSettingsScope::Policy
//! [`forbid_ignore_settings_file`]: DscPolicyFileData::forbid_ignore_settings_file
//! [`DSC_RESOURCE_PATH`]: DscSettingsEnvironmentData::dsc_resource_path
//! [`DSC_RESTRICTED_PATH`]: DscSettingsEnvironmentData::dsc_restricted_path
//! [`--ignore-settings-file`]: DscSettingsCliData::ignore_settings_file
//! [`ignore_settings_file`]: DscPolicyFileData::ignore_settings_file
//! [`DSC_IGNORE_SETTINGS_FILE`]: DscSettingsEnvironmentData::dsc_ignore_settings_file
//! [`forbid_ignore_settings_file`]: DscPolicyFileData::forbid_ignore_settings_file
//! [`Error`]: TraceLevelField::Error
//! [`Warn`]: TraceLevelField::Warn
//! [`Info`]: TraceLevelField::Info
//! [`Debug`]: TraceLevelField::Debug
//! [`Trace`]: TraceLevelField::Trace
//! [`tracing.level`]: TracingFileData::level
//! [`tracing.format`]: TracingFileData::format
//! [`DSC_TRACE_LEVEL`]: DscSettingsEnvironmentData::dsc_trace_level
//! [`DSC_TRACE_FORMAT`]: DscSettingsEnvironmentData::dsc_trace_format
//! [`--trace-level`]: DscSettingsCliData::trace_level
//! [`--trace-format`]: DscSettingsCliData::trace_format
//! [`resourcePath.directories`]: ResourcePathFileData::directories
//! [`resourcePath.appendEnvPath`]: ResourcePathFileData::append_env_path
//! [`resourcePath.restricted`]: ResourcePathFileData::restricted
//! [`directories`]: ResourcePathResolvedSettings::directories
//! [`restricted`]: ResourcePathResolvedSettings::restricted
//! [`append_env_path`]: ResourcePathResolvedSettings::append_env_path
//! [`resource_path.directories`]: ResourcePathResolvedSettings::directories
//! [`resource_path.restricted`]: ResourcePathResolvedSettings::restricted
//! [`new_with_command_line()`]: DscSettings::new_with_command_line
//! [`load()`]: DscSettings::load
//! [`try_load()`]: DscSettings::try_load
//! [`resolved()`]: DscSettings::resolved

use tracing::{debug, warn};

mod fields;
pub use fields::*;
mod sources;
pub use sources::*;
mod resolved;
pub use resolved::*;
mod constants_and_statics;
pub use constants_and_statics::*;
mod dsc_settings_scope;
pub use dsc_settings_scope::DscSettingsScope;

mod errors;
pub use errors::DscSettingsError;


/// Represents the complete set of DSC settings, including all sources and the resolved effective
/// settings.
/// 
/// During initialization, DSC loads settings from various sources. The following list defines the
/// sources for settings in order of precedence, from lowest to highest where later sources override
/// earlier sources:
/// 
/// - The hardcoded defaults defined in the source code.
/// - The machine settings file, if it exists.
/// - The user settings file, if it exists.
/// - The workspace settings file, if it exists.
/// - The environment variables, if they are set.
/// - The command line arguments, if they are provided.
/// - The policy settings file, if it exists.
/// 
/// You can use the [`load()`] or [`try_load()`] methods to load settings from all sources except
/// for the command line, which must be manually provided with the [`new_with_command_line()`]
/// constructor.
/// 
/// You can access the resolved effective settings by calling the [`resolved()`] method, which
/// returns an instance of [`DscSettingsResolved`] containing the final values for each setting
/// after considering all sources.
/// 
/// [`load()`]: Self::load
/// [`try_load()`]: Self::try_load
/// [`new_with_command_line()`]: Self::new_with_command_line
/// [`resolved()`]: Self::resolved
#[allow(dead_code)]
pub struct DscSettings {
    /// The hardcoded default settings defined in the source code.
    /// 
    /// These defaults have the lowest precedence and will be overridden by any other settings
    /// source if a value is provided.
    /// 
    /// The following snippet shows the effective code defaults as YAML data:
    /// 
    /// ```yaml
    /// forbid_ignore_settings_file: false
    /// ignore_settings_file: false
    /// tracing:
    ///   level:  warn
    ///   format: default
    /// resource_path:
    ///   directories: []
    ///   append_env_path: true
    ///   restricted: false
    /// ```
    /// 
    default: DscSettingsCodeDefaults,
    /// The settings loaded from the machine settings file, if it exists.
    /// 
    /// This field is set to [`None`] if the machine settings file does not exist or if it couldn't
    /// be loaded.
    /// 
    /// The machine settings file has the lowest precedence for settings after the code defaults.
    /// Any settings defined in a different scope will override the values defined in the machine
    /// settings file.
    /// 
    /// The location for the machine settings file depends on the operating system:
    /// 
    /// - On Windows, it's located at `%PROGRAMDATA%\DSC\settings.json`.
    /// - On macOS, it's located at `/Library/Application Support/DSC/settings.json`.
    /// - On Linux, it's located at `/etc/dsc/settings.json`.
    pub machine: Option<DscPreferenceFileData>,
    /// The settings loaded from the user settings file, if it exists.
    /// 
    /// This field is set to [`None`] if the user settings file does not exist or if it couldn't
    /// be loaded.
    /// 
    /// The user settings file has higher precedence than the machine settings file, but lower
    /// precedence than the workspace settings file, environment variables, and command line
    /// arguments. Any settings defined in those scopes will override the values defined in the
    /// user settings file.
    pub user: Option<DscPreferenceFileData>,
    /// The settings loaded from the workspace settings file, if it exists.
    /// 
    /// This field is set to [`None`] if the workspace settings file does not exist or if it
    /// couldn't be loaded.
    /// 
    /// The workspace settings file has higher precedence than the machine and user settings files
    /// but lower precedence than environment variables and command line arguments. Any settings
    /// defined in those scopes will override the values defined in the workspace settings file.
    pub workspace: Option<DscPreferenceFileData>,
    /// The settings loaded from the environment variables, if they exist.
    /// 
    /// This field is set to [`None`] if none of the relevant environment variables are set. DSC
    /// uses the following environment variables for settings:
    /// 
    /// - `DSC_TRACE_LEVEL`: Defines the trace level to use.
    /// - `DSC_TRACE_FORMAT`: Defines the trace format to use.
    /// - `DSC_RESOURCE_PATH`: Defines the resource directories to use.
    /// - `DSC_RESTRICTED_PATH`: Defines the resource directories to use and restricts all DSC
    ///   invocations to those directories exclusively.
    /// - `DSC_IGNORE_SETTINGS_FILE`: Defines whether to ignore settings files.
    /// 
    /// The environment variables have higher precedence than the machine, user, and workspace
    /// settings files but lower precedence than command line arguments. Any settings defined in
    /// the command line arguments will override the values defined in the environment variables.
    pub environment: Option<DscSettingsEnvironmentData>,
    /// The settings loaded from the command line arguments, if they were specified.
    /// 
    /// This field is set to [`None`] if no command line arguments were provided relating to DSC
    /// settings. DSC uses the following command line arguments for settings:
    /// 
    /// - `--trace-level`: Defines the trace level to use.
    /// - `--trace-format`: Defines the trace format to use.
    /// - `--ignore-settings-file`: Defines whether to ignore settings files.
    /// 
    /// The command line arguments have the highest precedence for settings, overriding any values
    /// defined in the machine, user, and workspace settings files, as well as any values defined
    /// in the environment variables. Only policy settings have higher precedence than command
    /// line arguments, and they cannot be overridden.
    pub command_line: Option<DscSettingsCliData>,
    /// The settings loaded from the policy settings file, if it exists.
    /// 
    /// This field is set to [`None`] if the policy settings file doesn't exist or if it couldn't
    /// be loaded.
    /// 
    /// The policy settings file has the highest precedence for settings, overriding any values
    /// defined in other sources, including command line arguments. Policy settings cannot be
    /// overridden.
    pub policy: Option<DscPolicyFileData>,
    resolved: Option<DscSettingsResolved>,
}


// Public API
impl DscSettings {
    /// Creates a new instance of `DscSettings` with all fields except `default` initialized to `None`.
    ///
    /// The `default` field is initialized with the hardcoded defaults defined in
    /// [`DSC_SETTINGS_CODE_DEFAULTS`].
    pub fn new() -> Self {
        Self {
            default: DSC_SETTINGS_CODE_DEFAULTS,
            machine: None,
            user: None,
            workspace: None,
            environment: None,
            command_line: None,
            policy: None,
            resolved: None,
        }
    }

    /// Creates a new instance of `DscSettings` and loads the provided command line data into it.
    ///
    /// The only fields populated in the returned instance are `default` and `command_line`. All
    /// other fields are defined as [`None`].
    ///
    /// # Arguments
    ///
    /// - `cli_data`: The command line data to load into the new instance.
    pub fn new_with_command_line(cli_data: DscSettingsCliData) -> Self {
        let mut settings = Self::new();
        settings.command_line = Some(cli_data);

        settings
    }

    pub fn resolved(&mut self) -> &DscSettingsResolved {
        if self.resolved.is_none() {
            self.resolve_all();
        }

        self.resolved.as_ref().unwrap()
    }

    /// Indicates whether the policy settings forbid ignoring settings files.
    ///
    /// # Returns
    ///
    /// This method returns `true` if the policy settings forbid ignoring settings files, and
    /// `false` otherwise. If the policy settings haven't been loaded, this method returns `false`.
    pub fn policy_forbids_ignoring_settings_files(&self) -> bool {
        self.policy.as_ref().is_some_and(|p| {
            p.forbid_ignore_settings_file.is_some_and(|v| v == true)
        })
    }

    /// Indicates whether settings files should be ignored based on the current settings sources.
    /// 
    /// # Returns
    /// 
    /// This return value for this method depends on the loaded policy, environment, and command
    /// line settings. The precedence rules are as follows:
    /// 
    /// 1. If `policy.forbid_ignore_settings_file` is set to `true`, this method always returns
    ///    `false`, regardless of the other sources.
    /// 1. If `command_line.ignore_settings_file` is set, this method returns its value.
    /// 1. If `environment.dsc_ignore_settings_file` is set, this method returns its value.
    /// 1. If none of the above conditions are met, this method returns `false`.
    pub fn ignoring_settings_files(&self) -> bool {
        // If the policy forbids ignoring settings files, always return false.
        if self.policy_forbids_ignoring_settings_files() {
            return false;
        }
        if let Some(policy_ignore) = self.policy.as_ref().and_then(|p| p.ignore_settings_file) {
            return policy_ignore;
        }
        if let Some(cli_ignore) = self.command_line.as_ref().and_then(|cli| cli.ignore_settings_file) {
            return cli_ignore;
        }
        if let Some(env_ignore) = self.environment.as_ref().and_then(|env| env.dsc_ignore_settings_file) {
            return env_ignore;
        }

        false
    }

    /// Resolves the effective settings by applying the precedence rules to all loaded sources.
    ///
    /// The resolution steps are:
    ///
    /// 1. Initialize with default settings from source code.
    /// 1. If policy file settings are loaded, apply them next, as they have the highest precedence.
    /// 1. Apply settings file sources in order of precedence ([`Machine`], then [`User`], then
    ///   [`Workspace`]).
    /// 1. If environment settings are loaded, apply them next, overriding any non-policy settings.
    /// 1. If command line settings are loaded, apply them last, overriding any non-policy settings.
    ///
    /// Resolution is performed for every setting, _not_ by container. If the machine settings file
    /// defines both `tracing.level` and `tracing.format`, and the user settings file defines only
    /// `tracing.format`, the final resolved settings will have `tracing.level` from the machine
    /// settings file and `tracing.format` from the user settings file.
    /// 
    /// Settings must be explicitly overridden by a higher-precedence source to be applied. DSC
    /// doesn't support effectively "undefining" a setting in a higher precedence source.
    /// 
    /// [`Machine`]: DscSettingsScope::Machine
    /// [`User`]: DscSettingsScope::User
    /// [`Workspace`]: DscSettingsScope::Workspace
    /// [`Environment`]: DscSettingsScope::Environment
    /// [`Cli`]: DscSettingsScope::CommandLine
    /// [`Policy`]: DscSettingsScope::Policy
    pub fn resolve_all(&mut self) {
        let mut resolving = DscSettingsResolved::default();
        // First, resolve policy settings, as they have the highest precedence.
        self.resolve_policy(&mut resolving);
        // Only resolve file-based settings if ignoring settings files is not forbidden by policy
        // and not specified in the policy, command line, or environment.
        if !self.ignoring_settings_files() {
            // Resolve file-based settings in order of precedence (Machine, User, Workspace).
            for source in DscSettingsScope::FILE_BASED.iter().cloned() {
                self.resolve_file_based_settings(source, &mut resolving);
            }
        }
        // Resolve environment settings, which have higher precedence than file-based settings.
        self.resolve_environment(&mut resolving);
        // Finally, resolve command line settings, which have the highest precedence (except for policy).
        self.resolve_command_line(&mut resolving);

        self.resolved = Some(resolving);
    }

    /// Attempts to load settings from all non-CLI sources and converts loading errors into warnings.
    ///
    /// When loading a source raises an error, this method logs the error as a warning and continues
    /// loading the remaining sources. Only sources that are successfully loaded are stored in the
    /// instance. All other sources remain set to [`None`].
    pub fn load(&mut self) {
        if let Err(e) = self.load_policy() {
            warn!("failed to load policy settings: {}", e);
        }
        if let Err(e) = self.load_machine() {
            warn!("failed to load machine settings: {}", e);
        }
        if let Err(e) = self.load_user() {
            warn!("failed to load user settings: {}", e);
        }
        if let Err(e) = self.load_workspace() {
            warn!("failed to load workspace settings: {}", e);
        }
        self.load_environment();
    }

    /// Attempts to load settings from all non-CLI sources and collects any errors when loading
    /// each source.
    ///
    /// To load settings and ignore sources with errors, use the [`load()`] method instead.
    ///
    /// # Errors
    ///
    /// If loading any source raises an error, this method returns an instance of
    /// [`LoadMultipleErrors`] containing a vector of all errors encountered.
    ///
    /// [`load()`]: Self::load
    /// [`LoadMultipleErrors`]: DscSettingsError::LoadMultipleErrors
    pub fn try_load(&mut self) -> Result<(), DscSettingsError> {
        let mut errors : Vec<DscSettingsError> = Vec::new();
        if let Err(e) = self.load_policy() {
            errors.push(e);
        }
        if let Err(e) = self.load_machine() {
            errors.push(e);
        }
        if let Err(e) = self.load_user() {
            errors.push(e);
        }
        if let Err(e) = self.load_workspace() {
            errors.push(e);
        }
        self.load_environment();

        if errors.is_empty() {
            Ok(())
        } else {
            Err(DscSettingsError::LoadMultipleErrors(errors))
        }
    }
}

// Private API
impl DscSettings {
    /// Loads the environment variables into the instance.
    fn load_environment(&mut self) {
        self.environment = Some(DscSettingsEnvironmentData::from_env());
    }

    /// Attempts to load the machine settings file into the instance
    /// 
    /// If the machine settings file doesn't exist, this method does nothing and returns
    /// `Ok(())`. If the file exists and is validly defined, this method loads the settings into
    /// the instance.
    /// 
    /// # Errors
    /// 
    /// If the file exists with invalid data, this method returns an error indicating the failure
    /// to load the machine settings file.
    fn load_machine(&mut self) -> Result<(), DscSettingsError> {
        let machine_settings_path = MACHINE_SETTINGS_FILE_PATH.as_path();
        if machine_settings_path.exists() {
            let data = DscPreferenceFileData::from_file(&machine_settings_path)?;
            debug!("Loaded machine settings from '{}'", machine_settings_path.to_string_lossy());
            self.machine = Some(data);
        } else {
            debug!("Machine settings file '{}' does not exist, skipping", machine_settings_path.to_string_lossy());
        }

        Ok(())
    }

    fn load_user(&mut self) -> Result<(), DscSettingsError> {
        let user_settings_path = USER_SETTINGS_FILE_PATH.as_path();
        if user_settings_path.exists() {
            let data = DscPreferenceFileData::from_file(&user_settings_path)?;
            debug!("Loaded user settings from '{}'", user_settings_path.to_string_lossy());
            self.user = Some(data);
        } else {
            debug!("User settings file '{}' does not exist, skipping", user_settings_path.to_string_lossy());
        }

        Ok(())
    }
    fn load_workspace(&mut self) -> Result<(), DscSettingsError> {
        let workspace_settings_path = WORKSPACE_SETTINGS_FILE_PATH.as_path();
        if workspace_settings_path.exists() {
            let data = DscPreferenceFileData::from_file(&workspace_settings_path)?;
            debug!("Loaded workspace settings from '{}'", workspace_settings_path.to_string_lossy());
            self.workspace = Some(data);
        } else {
            debug!("Workspace settings file '{}' does not exist, skipping", workspace_settings_path.to_string_lossy());
        }

        Ok(())
    }

    fn load_policy(&mut self) -> Result<(), DscSettingsError> {
        let policy_settings_path = POLICY_SETTINGS_FILE_PATH.as_path();
        if policy_settings_path.exists() {
            let data = DscPolicyFileData::from_file(&policy_settings_path)?;
            debug!("Loaded policy settings from '{}'", policy_settings_path.to_string_lossy());
            self.policy = Some(data);
        } else {
            debug!("Policy settings file '{}' does not exist, skipping", policy_settings_path.to_string_lossy());
        }

        Ok(())
    }

    /// Applies the settings from the [`Cli`] scope to the resolved settings.
    /// 
    /// If the command line settings aren't loaded, this method returns immediately without
    /// modifying the resolved settings. If the command line settings are loaded, this method
    /// applies them to the resolved settings, overriding any values from lower-precedence sources.
    /// 
    /// # Arguments
    /// 
    /// - `resolving` - A mutable reference to the instance of [`DscSettingsResolved`] that
    ///   represents the intermediate state of the resolved settings.
    /// 
    /// [`Cli`]: DscSettingsScope::CommandLine
    fn resolve_command_line(&mut self, resolving: &mut DscSettingsResolved) {
        let Some(cli_data) = self.command_line.as_ref() else {
            return;
        };
        let scope = DscSettingsScope::CommandLine;

        if let Some(level) = cli_data.trace_level.as_ref() {
            if resolving.tracing.level.scope < scope {
                resolving.tracing.level = DscSettingsResolvedField::new(level.clone(), scope);
            }
        }
        if let Some(format) = cli_data.trace_format.as_ref() {
            if resolving.tracing.format.scope < scope {
                resolving.tracing.format = DscSettingsResolvedField::new(format.clone(), scope);
            }
        }

        if let Some(ignore_settings_file) = cli_data.ignore_settings_file.as_ref() {
            // check if this option is forbidden by policy
            if self.policy_forbids_ignoring_settings_files() {
                warn!("Ignoring the --ignore-settings-file option because it's forbidden by policy.");
            } else {
                if resolving.ignore_settings_file.scope < scope {
                    resolving.ignore_settings_file = DscSettingsResolvedField::new(*ignore_settings_file, scope);
                }
            }
        }
    }

    /// Applies the settings from the [`Environment`] scope to the resolved settings.
    /// 
    /// If the environment settings aren't loaded, this method returns immediately without modifying
    /// the resolved settings. If the environment settings are loaded, this method applies them to
    /// the resolved settings, overriding any values from lower-precedence sources.
    /// 
    /// # Arguments
    /// 
    /// - `resolving` - A mutable reference to the instance of [`DscSettingsResolved`] that
    ///   represents the intermediate state of the resolved settings.
    /// 
    /// [`Environment`]: DscSettingsScope::Environment
    fn resolve_environment(&mut self, resolving: &mut DscSettingsResolved) {
        let Some(env_data) = self.environment.as_ref() else {
            return;
        };
        let scope = DscSettingsScope::Environment;


        if let Some(level) = env_data.dsc_trace_level.as_ref() {
            if resolving.tracing.level.scope < scope {
                resolving.tracing.level = DscSettingsResolvedField::new(level.clone(), scope);
            }
        }
        if let Some(format) = env_data.dsc_trace_format.as_ref() {
            if resolving.tracing.format.scope < scope {
                resolving.tracing.format = DscSettingsResolvedField::new(format.clone(), scope);
            }
        }

        if let Some(restricted_path) = env_data.dsc_restricted_path.as_ref() {
            if resolving.resource_path.restricted.scope < scope {
                let directories = restricted_path.clone();
                resolving.resource_path.directories = DscSettingsResolvedField::new(directories, scope);
                resolving.resource_path.restricted = DscSettingsResolvedField::new(true, scope);
            }
        } else if let Some(resource_path) = env_data.dsc_resource_path.as_ref() {
            if resolving.resource_path.directories.scope < scope {
                let directories = resource_path.clone();
                resolving.resource_path.directories = DscSettingsResolvedField::new(directories, scope);
            }
        }

        if let Some(ignore_settings_file) = env_data.dsc_ignore_settings_file.as_ref() {
            // Only override the ignore_settings_file setting if it's not forbidden by policy
            if self.policy_forbids_ignoring_settings_files() {
                warn!("Ignoring the DSC_IGNORE_SETTINGS_FILE environment variable because it's forbidden by policy.");
            } else {
                if resolving.ignore_settings_file.scope < scope {
                    resolving.ignore_settings_file = DscSettingsResolvedField::new(*ignore_settings_file, scope);
                }
            }
        }
    }

    /// Resolves the settings from file-based sources and applies them to the resolved settings.
    /// 
    /// # Arguments
    ///
    /// - `source` - The source scope from which to resolve the settings. This scope must be one of
    ///   the file-based sources: [`Machine`], [`User`], or [`Workspace`]. Specifying any other
    ///   scope returns early from the method without modifying the resolved settings or raising any
    ///   errors.
    /// - `resolving` - A mutable reference to the instance of [`DscSettingsResolved`] that
    ///   represents the intermediate state of the resolved settings.
    fn resolve_file_based_settings(&mut self,  source: DscSettingsScope, resolving: &mut DscSettingsResolved) {
        let file_data = match source {
            DscSettingsScope::Machine => self.machine.as_ref(),
            DscSettingsScope::User => self.user.as_ref(),
            DscSettingsScope::Workspace => self.workspace.as_ref(),
            _ => None,
        };
        let Some(file_data) = file_data else {
            return;
        };

        if let Some(tracing) = &file_data.tracing {
            if let Some(level) = tracing.level.as_ref() {
                if resolving.tracing.level.scope < source {
                    resolving.tracing.level = DscSettingsResolvedField::new(level.clone(), source);
                }
            }
            if let Some(format) = tracing.format.as_ref() {
                if resolving.tracing.format.scope < source {
                    resolving.tracing.format = DscSettingsResolvedField::new(format.clone(), source);
                }
            }
        }

        if let Some(resource_path) = &file_data.resource_path {
            if let Some(append_env_path) = resource_path.append_env_path.as_ref() {
                if resolving.resource_path.append_env_path.scope < source {
                    resolving.resource_path.append_env_path = DscSettingsResolvedField::new(append_env_path.clone(), source);
                }
            }
            if let Some(directories) = resource_path.directories.as_ref() {
                if resolving.resource_path.directories.scope < source {
                    resolving.resource_path.directories = DscSettingsResolvedField::new(directories.clone(), source);
                }
            }
            if let Some(restrict_path) = resource_path.restricted.as_ref() {
                if resolving.resource_path.restricted.scope < source {
                    resolving.resource_path.restricted = DscSettingsResolvedField::new(restrict_path.clone(), source);
                }
            }
        }
    }

    /// Resolves the policy settings and applies them to the resolved settings.
    /// 
    /// If the policy settings aren't loaded, this method returns immediately without modifying the
    /// resolved settings. If the policy settings are loaded, this method applies them to the
    /// resolved settings, overriding any values from lower-precedence sources.
    fn resolve_policy(&mut self, resolving: &mut DscSettingsResolved) {
        let Some(policy) = self.policy.as_ref() else {
            return;
        };
        
        let scope = DscSettingsScope::Policy;

        if let Some(forbid_ignore) = policy.forbid_ignore_settings_file {
            resolving.forbid_ignore_settings_file = DscSettingsResolvedField::new(
                forbid_ignore,
                scope
            );
        }
        if let Some(ignore) = policy.ignore_settings_file {
            if let Some(forbid_ignore) = policy.forbid_ignore_settings_file {
                // If the settings are incompatible, prefer forbidding and don't ignore the
                // settings files.
                if forbid_ignore && ignore{
                    warn!("Ignoring the policy setting 'ignore_settings_file' because 'forbid_ignore_settings_file' is set to true.");
                } else {
                    resolving.ignore_settings_file = DscSettingsResolvedField::new(
                        ignore,
                        scope
                    );
                }
            } else {
                resolving.ignore_settings_file = DscSettingsResolvedField::new(
                    ignore,
                    scope
                );
            }
        }

        if let Some(tracing) = &policy.tracing {
            if let Some(level) = tracing.level.as_ref() {
                resolving.tracing.level = DscSettingsResolvedField::new(
                    level.clone(),
                    scope
                );
            }
            if let Some(format) = tracing.format.as_ref() {
                resolving.tracing.format = DscSettingsResolvedField::new(
                    format.clone(),
                    scope
                );
            }
        }
        if let Some(resource_path) = &policy.resource_path {
            if let Some(append_env_path) = resource_path.append_env_path.as_ref() {
                resolving.resource_path.append_env_path = DscSettingsResolvedField::new(
                    append_env_path.clone(),
                    scope
                );
            }
            if let Some(directories) = resource_path.directories.as_ref() {
                resolving.resource_path.directories = DscSettingsResolvedField::new(
                    directories.clone(),
                    scope
                );
            }
            if let Some(restrict_path) = resource_path.restricted.as_ref() {
                resolving.resource_path.restricted = DscSettingsResolvedField::new(
                    restrict_path.clone(),
                    scope
                );
            }
        }
    }
}
