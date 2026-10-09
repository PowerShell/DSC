// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! Defines how to load and resolve settings for DSC.
//!
//! DSC uses a layered approach to settings, composing a set of resolved settings from multiple
//! sources. The [`DscSettings`] struct represents the complete model of settings, including all
//! sources and the resolved effective settings.
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
//! | [`Default`]     | [`DSC_SETTINGS_CODE_DEFAULTS`] | The hardcoded default settings defined in the source code.          |
//! | [`User`]        | [`PreferenceFileData`]         | A settings file defining preferences for the current user.          |
//! | [`Machine`]     | [`PreferenceFileData`]         | A settings file defining preferences for every user on the machine. |
//! | [`Workspace`]   | [`PreferenceFileData`]         | A settings file defining preferences for the current workspace.     |
//! | [`Environment`] | [`EnvironmentData`]            | Settings loaded from the environment variables                      |
//! | [`CommandLine`] | [`CommandLineData`]            | Settings loaded from the command line arguments.                    |
//! | [`Policy`]      | [`PolicyFileData`]             | A settings file defining policy for every user.                     |
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
//! and parse the file into the appropriate data structure ([`PolicyFileData`] for [`Policy`]
//! and [`PreferenceFileData`] for the others).
//!
//! When loading settings for the [`Environment`] scope, DSC checks whether the relevant environment
//! variables are defined and reads their values if they exist, parsing them into the appropriate
//! data type for the backing field in [`EnvironmentData`].
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
//!    [`Workspace`] settings files.
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
//!    [`Workspace`] settings files.
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
//!    [`resourcePath.restricted`] fields in the [`Policy`], [`Machine`], [`User`], or [`Workspace`]
//!    settings files.
//! 1. Define the [`DSC_RESOURCE_PATH`] environment variable as a string containing a collection of
//!    directories separated by the platform specific path separator for `PATH`-like environment
//!    variables. This effectively populates the [`directories`] field in the resolved settings.
//! 1. Define the [`DSC_RESTRICTED_PATH`] environment variable as a string containing a collection
//!    of directories separated by the platform specific path separator for `PATH`-like environment
//!    variables. This effectively populates the [`directories`] field in the resolved settings and
//!    sets the [`restricted`] field in the resolved settings to `true`.
//!
//! ### Resource path xamples
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
//! [`DSC_RESOURCE_PATH`]: sources::EnvironmentData::dsc_resource_path
//! [`DSC_RESTRICTED_PATH`]: sources::EnvironmentData::dsc_restricted_path
//! [`--ignore-settings-file`]: sources::CommandLineData::ignore_settings_file
//! [`ignore_settings_file`]: sources::PolicyFileData::ignore_settings_file
//! [`DSC_IGNORE_SETTINGS_FILE`]: sources::EnvironmentData::dsc_ignore_settings_file
//! [`Error`]: fields::TracingLevelField::Error
//! [`Warn`]: fields::TracingLevelField::Warn
//! [`Info`]: fields::TracingLevelField::Info
//! [`Debug`]: fields::TracingLevelField::Debug
//! [`Trace`]: fields::TracingLevelField::Trace
//! [`tracing.level`]: fields::TracingFileData::level
//! [`tracing.format`]: fields::TracingFileData::format
//! [`DSC_TRACE_LEVEL`]: sources::EnvironmentData::dsc_trace_level
//! [`DSC_TRACE_FORMAT`]: sources::EnvironmentData::dsc_trace_format
//! [`--trace-level`]: sources::CommandLineData::trace_level
//! [`--trace-format`]: sources::CommandLineData::trace_format
//! [`resourcePath.directories`]: fields::ResourcePathFileData::directories
//! [`resourcePath.appendEnvPath`]: fields::ResourcePathFileData::append_env_path
//! [`resourcePath.restricted`]: fields::ResourcePathFileData::restricted
//! [`directories`]: fields::ResourcePathResolvedSettings::directories
//! [`restricted`]: fields::ResourcePathResolvedSettings::restricted
//! [`append_env_path`]: fields::ResourcePathResolvedSettings::append_env_path
//! [`resource_path.directories`]: fields::ResourcePathResolvedSettings::directories
//! [`resource_path.restricted`]: fields::ResourcePathResolvedSettings::restricted
//! [`new_with_command_line()`]: DscSettings::new_with_command_line
//! [`load()`]: DscSettings::load
//! [`try_load()`]: DscSettings::try_load
//! [`resolved()`]: DscSettings::resolved
//! [`DSC_SETTINGS_CODE_DEFAULTS`]: sources::DSC_SETTINGS_CODE_DEFAULTS
//! [`PreferenceFileData`]: sources::PreferenceFileData
//! [`EnvironmentData`]: sources::EnvironmentData
//! [`CommandLineData`]: sources::CommandLineData
//! [`PolicyFileData`]: sources::PolicyFileData
//! [`forbid_ignore_settings_file`]: sources::PolicyFileData::forbid_ignore_settings_file

pub mod fields;
pub mod sources;
mod constants_and_statics;
pub use constants_and_statics::*;
mod dsc_settings_scope;
pub use dsc_settings_scope::DscSettingsScope;
mod dsc_settings;
pub use dsc_settings::DscSettings;
mod resolved;
pub use resolved::*;
mod errors;
pub use errors::DscSettingsError;
