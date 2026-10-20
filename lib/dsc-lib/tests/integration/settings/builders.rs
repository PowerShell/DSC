// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! Defines builders for constructing settings data structures for testing purposes.
//!
//! The implementation for DSC retrieves settings from both the file system and the environment. However, for testing
//! purposes, it's useful to construct settings data structures directly in memory without relying on external files or
//! environment variables.
//!
//! This makes it easier to validate behavior for settings resolution, precedence, and policy enforcement without
//! needing to set up specific files or environment states.
//!
//! Acceptance tests written in Pester are more suited to testing the file system and environment interactions, while
//! these builders are intended for integration tests that focus on the correctness of the public API.

#![allow(dead_code)]

use std::path::PathBuf;
use dsc_lib::settings::fields::*;
use dsc_lib::settings::sources::*;
use dsc_lib::settings::{DscSettings, DscSettingsResolved, DscSettingsResolvedField, DscSettingsScope};

/// Builder for [`TracingFileData`]
pub struct TracingFileDataBuilder {
    level: Option<TracingLevelField>,
    format: Option<TracingFormatField>,
}

impl TracingFileDataBuilder {
    /// Creates a new instance of the builder with all fields undefined.
    ///
    /// # Returns
    ///
    /// An instance of [`TracingFileDataBuilder`].
    pub fn new() -> Self {
        Self {
            level: None,
            format: None,
        }
    }

    /// Adds the specified [`TracingLevelField`] to the builder.
    ///
    /// # Arguments
    ///
    /// - `level` - The tracing level to be set in the builder.
    ///
    /// # Returns
    ///
    /// The updated instance of [`TracingFileDataBuilder`].
    pub fn with_level(mut self, level: TracingLevelField) -> Self {
        self.level = Some(level);
        self
    }

    /// Adds the specified [`TracingFormatField`] to the builder.
    ///
    /// # Arguments
    ///
    /// - `format` - The tracing format to be set in the builder.
    ///
    /// # Returns
    ///
    /// The updated instance of [`TracingFileDataBuilder`].
    pub fn with_format(mut self, format: TracingFormatField) -> Self {
        self.format = Some(format);
        self
    }

    /// Builds the [`TracingFileData`] instance from the specified fields.
    ///
    /// # Returns
    ///
    /// An instance of [`TracingFileData`].
    pub fn build(self) -> TracingFileData {
        TracingFileData {
            level: self.level,
            format: self.format,
        }
    }
}

/// Builder for [`ResourcePathFileData`]
pub struct ResourcePathFileDataBuilder {
    append_env_path: Option<bool>,
    directories: Option<Vec<PathBuf>>,
    restricted: Option<bool>,
}

impl ResourcePathFileDataBuilder {
    /// Creates a new instance of the builder with all fields undefined.
    ///
    /// # Returns
    ///
    /// An instance of [`ResourcePathFileDataBuilder`].
    pub fn new() -> Self {
        Self {
            append_env_path: None,
            directories: None,
            restricted: None,
        }
    }

    /// Adds the specified append environment path flag to the builder.
    ///
    /// # Arguments
    ///
    /// - `append` - A boolean indicating whether to append the environment path.
    ///
    /// # Returns
    ///
    /// The updated instance of [`ResourcePathFileDataBuilder`].
    pub fn with_append_env_path(mut self, append: bool) -> Self {
        self.append_env_path = Some(append);
        self
    }

    /// Adds the specified directories to the builder.
    ///
    /// # Arguments
    ///
    /// - `dirs` - A vector of directory paths as strings.
    ///
    /// # Returns
    ///
    /// The updated instance of [`ResourcePathFileDataBuilder`].
    pub fn with_directories(mut self, dirs: Vec<&str>) -> Self {
        self.directories = Some(dirs.into_iter().map(PathBuf::from).collect());
        self
    }

    /// Adds the specified restricted flag to the builder.
    ///
    /// # Arguments
    ///
    /// - `restrict` - A boolean indicating whether the resource path is restricted.
    ///
    /// # Returns
    ///
    /// The updated instance of [`ResourcePathFileDataBuilder`].
    pub fn with_restricted(mut self, restrict: bool) -> Self {
        self.restricted = Some(restrict);
        self
    }

    /// Builds the [`ResourcePathFileData`] instance from the specified fields.
    ///
    /// # Returns
    ///
    /// An instance of [`ResourcePathFileData`].
    pub fn build(self) -> ResourcePathFileData {
        ResourcePathFileData {
            append_env_path: self.append_env_path,
            directories: self.directories,
            restricted: self.restricted,
        }
    }
}

/// Builder for [`PolicyFileData`]
pub struct PolicyFileDataBuilder {
    pub forbid_ignore_settings_file: Option<ForbidIgnoreSettingsFileField>,
    pub ignore_settings_file: Option<IgnoreSettingsFileField>,
    pub tracing: Option<TracingFileData>,
    pub resource_path: Option<ResourcePathFileData>,
}

impl PolicyFileDataBuilder {
    /// Creates a new instance of the builder with all fields undefined.
    ///
    /// # Returns
    ///
    /// An instance of [`PolicyFileDataBuilder`].
    pub fn new() -> Self {
        Self {
            forbid_ignore_settings_file: None,
            ignore_settings_file: None,
            tracing: None,
            resource_path: None,
        }
    }
    /// Adds the specified forbid ignore settings file flag to the builder.
    ///
    /// # Arguments
    ///
    /// - `forbid` - A boolean indicating whether to forbid ignoring the settings file.
    ///
    /// # Returns
    ///
    /// The updated instance of [`PolicyFileDataBuilder`].
    pub fn with_forbid_ignore_settings_file(mut self, forbid: bool) -> Self {
        self.forbid_ignore_settings_file = Some(forbid.into());
        self
    }
    /// Adds the specified ignore settings file flag to the builder.
    ///
    /// # Arguments
    ///
    /// - `ignore` - A boolean indicating whether to ignore the settings file.
    ///
    /// # Returns
    ///
    /// The updated instance of [`PolicyFileDataBuilder`].
    pub fn with_ignore_settings_file(mut self, ignore: bool) -> Self {
        self.ignore_settings_file = Some(ignore.into());
        self
    }
    /// Adds the specified tracing configuration to the builder.
    ///
    /// # Arguments
    ///
    /// - `tracing` - An instance of [`TracingFileData`].
    ///
    /// # Returns
    ///
    /// The updated instance of [`PolicyFileDataBuilder`].
    pub fn with_tracing(mut self, tracing: TracingFileData) -> Self {
        self.tracing = Some(tracing);
        self
    }
    /// Adds the specified resource path configuration to the builder.
    ///
    /// # Arguments
    ///
    /// - `resource_path` - An instance of [`ResourcePathFileData`].
    ///
    /// # Returns
    ///
    /// The updated instance of [`PolicyFileDataBuilder`].
    pub fn with_resource_path(mut self, resource_path: ResourcePathFileData) -> Self {
        self.resource_path = Some(resource_path);
        self
    }
    /// Builds the [`PolicyFileData`] instance from the specified fields.
    ///
    /// # Returns
    ///
    /// An instance of [`PolicyFileData`].
    pub fn build(self) -> PolicyFileData {
        PolicyFileData {
            forbid_ignore_settings_file: self.forbid_ignore_settings_file,
            ignore_settings_file: self.ignore_settings_file,
            tracing: self.tracing,
            resource_path: self.resource_path,
        }
    }
}

/// Builder for [`PreferenceFileData`]
pub struct PreferenceFileDataBuilder {
    pub tracing: Option<TracingFileData>,
    pub resource_path: Option<ResourcePathFileData>,
}
impl PreferenceFileDataBuilder {
    /// Creates a new instance of the [`PreferenceFileDataBuilder`] with default values.
    ///
    /// # Returns
    ///
    /// An instance of [`PreferenceFileDataBuilder`].
    pub fn new() -> Self {
        Self {
            tracing: None,
            resource_path: None,
        }
    }

    /// Adds the specified tracing configuration to the builder.
    ///
    /// # Arguments
    ///
    /// - `tracing` - An instance of [`TracingFileData`].
    ///
    /// # Returns
    ///
    /// The updated instance of [`PreferenceFileDataBuilder`].
    pub fn with_tracing(mut self, tracing: TracingFileData) -> Self {
        self.tracing = Some(tracing);
        self
    }

    /// Adds the specified resource path configuration to the builder.
    ///
    /// # Arguments
    ///
    /// - `resource_path` - An instance of [`ResourcePathFileData`].
    ///
    /// # Returns
    ///
    /// The updated instance of [`PreferenceFileDataBuilder`].
    pub fn with_resource_path(mut self, resource_path: ResourcePathFileData) -> Self {
        self.resource_path = Some(resource_path);
        self
    }

    /// Builds the [`PreferenceFileData`] instance from the specified fields.
    ///
    /// # Returns
    ///
    /// An instance of [`PreferenceFileData`].
    pub fn build(self) -> PreferenceFileData {
        PreferenceFileData {
            tracing: self.tracing,
            resource_path: self.resource_path,
        }
    }
}

/// Builder for [`CommandLineData`]
pub struct CommandLineDataBuilder {
    pub trace_level: Option<TracingLevelField>,
    pub trace_format: Option<TracingFormatField>,
    pub ignore_settings_file: Option<IgnoreSettingsFileField>,
}

impl CommandLineDataBuilder {
    /// Creates a new instance of the [`CommandLineDataBuilder`] with default values.
    ///
    /// # Returns
    ///
    /// An instance of [`CommandLineDataBuilder`].
    pub fn new() -> Self {
        Self {
            trace_level: None,
            trace_format: None,
            ignore_settings_file: None,
        }
    }

    /// Adds the specified trace level configuration to the builder.
    ///
    /// # Arguments
    ///
    /// - `level` - An instance of [`TracingLevelField`].
    ///
    /// # Returns
    ///
    /// The updated instance of [`CommandLineDataBuilder`].
    pub fn with_trace_level(mut self, level: TracingLevelField) -> Self {
        self.trace_level = Some(level);
        self
    }

    /// Adds the specified trace format configuration to the builder.
    ///
    /// # Arguments
    ///
    /// - `format` - An instance of [`TracingFormatField`].
    ///
    /// # Returns
    ///
    /// The updated instance of [`CommandLineDataBuilder`].
    pub fn with_trace_format(mut self, format: TracingFormatField) -> Self {
        self.trace_format = Some(format);
        self
    }

    /// Adds the specified ignore settings file configuration to the builder.
    ///
    /// # Arguments
    ///
    /// - `ignore` - A boolean indicating whether to ignore the settings file.
    ///
    /// # Returns
    ///
    /// The updated instance of [`CommandLineDataBuilder`].
    pub fn with_ignore_settings_file(mut self, ignore: bool) -> Self {
        self.ignore_settings_file = Some(ignore.into());
        self
    }

    /// Builds the [`CommandLineData`] instance from the specified fields.
    ///
    /// # Returns
    ///
    /// An instance of [`CommandLineData`].
    pub fn build(self) -> CommandLineData {
        CommandLineData {
            trace_level: self.trace_level,
            trace_format: self.trace_format,
            ignore_settings_file: self.ignore_settings_file,
        }
    }
}

/// Builder for [`EnvironmentData`]
pub struct EnvironmentDataBuilder {
    dsc_trace_level: Option<TracingLevelField>,
    dsc_trace_format: Option<TracingFormatField>,
    dsc_resource_path: Option<Vec<PathBuf>>,
    dsc_restricted_path: Option<Vec<PathBuf>>,
    dsc_ignore_settings_file: Option<IgnoreSettingsFileField>,
}

impl EnvironmentDataBuilder {
    /// Creates a new instance of the [`EnvironmentDataBuilder`] with default values.
    ///
    /// # Returns
    ///
    /// An instance of [`EnvironmentDataBuilder`].
    pub fn new() -> Self {
        Self {
            dsc_trace_level: None,
            dsc_trace_format: None,
            dsc_resource_path: None,
            dsc_restricted_path: None,
            dsc_ignore_settings_file: None,
        }
    }
    /// Adds the specified trace level configuration to the builder.
    ///
    /// # Arguments
    ///
    /// - `level` - An instance of [`TracingLevelField`].
    ///
    /// # Returns
    ///
    /// The updated instance of [`EnvironmentDataBuilder`].
    pub fn with_trace_level(mut self, level: TracingLevelField) -> Self {
        self.dsc_trace_level = Some(level);
        self
    }

    /// Adds the specified trace format configuration to the builder.
    ///
    /// # Arguments
    ///
    /// - `format` - An instance of [`TracingFormatField`].
    ///
    /// # Returns
    ///
    /// The updated instance of [`EnvironmentDataBuilder`].
    pub fn with_trace_format(mut self, format: TracingFormatField) -> Self {
        self.dsc_trace_format = Some(format);
        self
    }

    /// Adds the specified resource path configuration to the builder.
    ///
    /// # Arguments
    ///
    /// - `path` - A vector of [`PathBuf`] representing the resource paths.
    ///
    /// # Returns
    ///
    /// The updated instance of [`EnvironmentDataBuilder`].
    pub fn with_resource_path(mut self, path: Vec<PathBuf>) -> Self {
        self.dsc_resource_path = Some(path);
        self
    }

    /// Adds the specified restricted path configuration to the builder.
    ///
    /// # Arguments
    ///
    /// - `path` - A vector of [`PathBuf`] representing the restricted paths.
    ///
    /// # Returns
    ///
    /// The updated instance of [`EnvironmentDataBuilder`].
    pub fn with_restricted_path(mut self, path: Vec<PathBuf>) -> Self {
        self.dsc_restricted_path = Some(path);
        self
    }

    /// Adds the specified ignore settings file configuration to the builder.
    ///
    /// # Arguments
    ///
    /// - `ignore` - A boolean indicating whether to ignore the settings file.
    ///
    /// # Returns
    ///
    /// The updated instance of [`EnvironmentDataBuilder`].
    pub fn with_ignore_settings_file(mut self, ignore: bool) -> Self {
        self.dsc_ignore_settings_file = Some(ignore.into());
        self
    }
    /// Builds the [`EnvironmentData`] instance from the specified fields.
    ///
    /// # Returns
    ///
    /// An instance of [`EnvironmentData`].
    pub fn build(self) -> EnvironmentData {
        EnvironmentData {
            dsc_trace_level: self.dsc_trace_level,
            dsc_trace_format: self.dsc_trace_format,
            dsc_resource_path: self.dsc_resource_path,
            dsc_restricted_path: self.dsc_restricted_path,
            dsc_ignore_settings_file: self.dsc_ignore_settings_file,
        }
    }
}

/// Builder for creating instances of [`DscSettings`].
pub struct DscSettingsBuilder {
    machine: Option<PreferenceFileData>,
    user: Option<PreferenceFileData>,
    workspace: Option<PreferenceFileData>,
    environment: Option<EnvironmentData>,
    command_line: Option<CommandLineData>,
    policy: Option<PolicyFileData>,
}

impl DscSettingsBuilder {
    /// Creates a new instance of [`DscSettingsBuilder`] with all fields set to `None`.
    ///
    /// # Returns
    ///
    /// A new instance of [`DscSettingsBuilder`].
    pub fn new() -> Self {
        Self {
            machine: None,
            user: None,
            workspace: None,
            environment: None,
            command_line: None,
            policy: None,
        }
    }

    /// Adds the specified machine scope preferences to the builder.
    ///
    /// # Arguments
    ///
    /// - `machine_data` - The machine scope preferences to add.
    ///
    /// # Returns
    ///
    /// The updated instance of [`DscSettingsBuilder`].
    pub fn with_machine(mut self, machine_data: PreferenceFileData) -> Self {
        self.machine = Some(machine_data);
        self
    }

    /// Adds the specified user scope preferences to the builder.
    ///
    /// # Arguments
    ///
    /// - `user_data` - The user scope preferences to add.
    ///
    /// # Returns
    ///
    /// The updated instance of [`DscSettingsBuilder`].
    pub fn with_user(mut self, user_data: PreferenceFileData) -> Self {
        self.user = Some(user_data);
        self
    }

    /// Adds the specified workspace scope preferences to the builder.
    ///
    /// # Arguments
    ///
    /// - `workspace_data` - The workspace scope preferences to add.
    ///
    /// # Returns
    ///
    /// The updated instance of [`DscSettingsBuilder`].
    pub fn with_workspace(mut self, workspace_data: PreferenceFileData) -> Self {
        self.workspace = Some(workspace_data);
        self
    }
    /// Adds the specified environment scope preferences to the builder.
    ///
    /// # Arguments
    ///
    /// - `environment_data` - The environment scope preferences to add.
    ///
    /// # Returns
    ///
    /// The updated instance of [`DscSettingsBuilder`].
    pub fn with_environment(mut self, environment_data: EnvironmentData) -> Self {
        self.environment = Some(environment_data);
        self
    }
    /// Adds the specified command line scope preferences to the builder.
    ///
    /// # Arguments
    ///
    /// - `command_line_data` - The command line scope preferences to add.
    ///
    /// # Returns
    ///
    /// The updated instance of [`DscSettingsBuilder`].
    pub fn with_command_line(mut self, command_line_data: CommandLineData) -> Self {
        self.command_line = Some(command_line_data);
        self
    }
    /// Adds the specified policy settings to the builder.
    ///
    /// # Arguments
    ///
    /// - `policy_data` - The policy scope preferences to add.
    ///
    /// # Returns
    ///
    /// The updated instance of [`DscSettingsBuilder`].
    pub fn with_policy(mut self, policy_data: PolicyFileData) -> Self {
        self.policy = Some(policy_data);
        self
    }
    /// Builds the final instance of [`DscSettings`] from the builder.
    ///
    /// # Returns
    ///
    /// The constructed instance of [`DscSettings`].
    pub fn build(self) -> DscSettings {
        let mut settings = DscSettings::new();

        settings.machine = self.machine;
        settings.user = self.user;
        settings.workspace = self.workspace;
        settings.environment = self.environment;
        settings.command_line = self.command_line;
        settings.policy = self.policy;

        settings
    }
}

/// Builder for constructing instances of [`ResourcePathResolvedSettings`].
pub struct ResourcePathResolvedSettingsBuilder {
    pub append_env_path: Option<DscSettingsResolvedField<bool>>,
    pub directories: Option<DscSettingsResolvedField<Vec<PathBuf>>>,
    pub restricted: Option<DscSettingsResolvedField<bool>>,
}

impl ResourcePathResolvedSettingsBuilder {
    /// Creates a new instance of the builder with default values.
    ///
    /// # Returns
    ///
    /// The newly created instance of [`ResourcePathResolvedSettingsBuilder`].
    pub fn new() -> Self {
        Self {
            append_env_path: None,
            directories: None,
            restricted: None,
        }
    }

    /// Adds the specified append environment path setting to the builder.
    ///
    /// # Arguments
    ///
    /// - `value` - The value to set for the append environment path setting.
    /// - `scope` - The scope of the setting.
    ///
    /// # Returns
    ///
    /// The updated instance of [`ResourcePathResolvedSettingsBuilder`].
    pub fn with_append_env_path(mut self, value: bool, scope: DscSettingsScope) -> Self {
        self.append_env_path = Some(DscSettingsResolvedField::new(value, scope));
        self
    }

    /// Adds the specified directories setting to the builder.
    ///
    /// # Arguments
    ///
    /// - `value` - The value to set for the directories setting.
    /// - `scope` - The scope of the setting.
    ///
    /// # Returns
    ///
    /// The updated instance of [`ResourcePathResolvedSettingsBuilder`].
    pub fn with_directories(mut self, value: Vec<&str>, scope: DscSettingsScope) -> Self {
        self.directories = Some(DscSettingsResolvedField::new(
            value.into_iter().map(PathBuf::from).collect(),
            scope,
        ));
        self
    }

    /// Adds the specified restricted setting to the builder.
    ///
    /// # Arguments
    ///
    /// - `value` - The value to set for the restricted setting.
    /// - `scope` - The scope of the setting.
    ///
    /// # Returns
    ///
    /// The updated instance of [`ResourcePathResolvedSettingsBuilder`].
    pub fn with_restricted(mut self, value: bool, scope: DscSettingsScope) -> Self {
        self.restricted = Some(DscSettingsResolvedField::new(value, scope));
        self
    }

    /// Builds the final instance of [`ResourcePathResolvedSettings`] from the builder.
    ///
    /// # Returns
    ///
    /// The constructed instance of [`ResourcePathResolvedSettings`].
    pub fn build(self) -> ResourcePathResolvedSettings {
        let mut resolved = ResourcePathResolvedSettings::default();
        if let Some(append_env_path) = self.append_env_path {
            resolved.append_env_path = append_env_path;
        }
        if let Some(directories) = self.directories {
            resolved.directories = directories;
        }
        if let Some(restricted) = self.restricted {
            resolved.restricted = restricted;
        }

        resolved
    }
}

/// Builder for constructing instances of [`TracingResolvedSettings`].
pub struct TracingResolvedSettingsBuilder {
    pub level: Option<DscSettingsResolvedField<TracingLevelField>>,
    pub format: Option<DscSettingsResolvedField<TracingFormatField>>,
}

impl TracingResolvedSettingsBuilder {
    /// Creates a new instance of the builder with default values.
    ///
    /// # Returns
    ///
    /// The newly created instance of [`TracingResolvedSettingsBuilder`].
    pub fn new() -> Self {
        Self {
            level: None,
            format: None,
        }
    }

    /// Adds the specified tracing level setting to the builder.
    ///
    /// # Arguments
    ///
    /// - `value` - The value to set for the tracing level setting.
    /// - `scope` - The scope of the setting.
    ///
    /// # Returns
    ///
    /// The updated instance of [`TracingResolvedSettingsBuilder`].
    pub fn with_level(mut self, value: TracingLevelField, scope: DscSettingsScope) -> Self {
        self.level = Some(DscSettingsResolvedField::new(value, scope));
        self
    }

    /// Adds the specified tracing format setting to the builder.
    ///
    /// # Arguments
    ///
    /// - `value` - The value to set for the tracing format setting.
    /// - `scope` - The scope of the setting.
    ///
    /// # Returns
    ///
    /// The updated instance of [`TracingResolvedSettingsBuilder`].
    pub fn with_format(mut self, value: TracingFormatField, scope: DscSettingsScope) -> Self {
        self.format = Some(DscSettingsResolvedField::new(value, scope));
        self
    }
    /// Builds the final instance of [`TracingResolvedSettings`] from the builder.
    ///
    /// # Returns
    ///
    /// The constructed instance of [`TracingResolvedSettings`].
    pub fn build(self) -> TracingResolvedSettings {
        let mut resolved = TracingResolvedSettings::default();
        if let Some(level) = self.level {
            resolved.level = level;
        }
        if let Some(format) = self.format {
            resolved.format = format;
        }

        resolved
    }
}

/// Builder for constructing instances of [`DscSettingsResolved`].
pub struct DscSettingsResolvedBuilder {
    pub forbid_ignore_settings_file: Option<DscSettingsResolvedField<ForbidIgnoreSettingsFileField>>,
    pub ignore_settings_file: Option<DscSettingsResolvedField<IgnoreSettingsFileField>>,
    pub tracing: Option<TracingResolvedSettings>,
    pub resource_path: Option<ResourcePathResolvedSettings>,
}

impl DscSettingsResolvedBuilder {
    /// Creates a new instance of the builder with default values.
    ///
    /// # Returns
    ///
    /// The newly created instance of [`DscSettingsResolvedBuilder`].
    pub fn new() -> Self {
        Self {
            forbid_ignore_settings_file: None,
            ignore_settings_file: None,
            tracing: None,
            resource_path: None,
        }
    }

    /// Adds the specified forbid ignore settings file setting to the builder.
    ///
    /// # Arguments
    ///
    /// - `value` - The value to set for the forbid ignore settings file setting.
    /// - `scope` - The scope of the setting.
    ///
    /// # Returns
    ///
    /// The updated instance of [`DscSettingsResolvedBuilder`].
    pub fn with_forbid_ignore_settings_file(mut self, value: bool, scope: DscSettingsScope) -> Self {
        self.forbid_ignore_settings_file = Some(DscSettingsResolvedField::new(value.into(), scope));
        self
    }

    /// Adds the specified ignore settings file setting to the builder.
    ///
    /// # Arguments
    ///
    /// - `value` - The value to set for the ignore settings file setting.
    /// - `scope` - The scope of the setting.
    ///
    /// # Returns
    ///
    /// The updated instance of [`DscSettingsResolvedBuilder`].
    pub fn with_ignore_settings_file(mut self, value: bool, scope: DscSettingsScope) -> Self {
        self.ignore_settings_file = Some(DscSettingsResolvedField::new(value.into(), scope));
        self
    }

    /// Adds the specified tracing settings to the builder.
    ///
    /// # Arguments
    ///
    /// - `tracing` - The tracing settings to set.
    ///
    /// # Returns
    ///
    /// The updated instance of [`DscSettingsResolvedBuilder`].
    pub fn with_tracing(mut self, tracing: TracingResolvedSettings) -> Self {
        self.tracing = Some(tracing);
        self
    }

    /// Adds the specified resource path settings to the builder.
    ///
    /// # Arguments
    ///
    /// - `resource_path` - The resource path settings to set.
    ///
    /// # Returns
    ///
    /// The updated instance of [`DscSettingsResolvedBuilder`].
    pub fn with_resource_path(mut self, resource_path: ResourcePathResolvedSettings) -> Self {
        self.resource_path = Some(resource_path);
        self
    }
    /// Builds the final instance of [`DscSettingsResolved`] using the settings provided to the builder.
    ///
    /// # Returns
    ///
    /// The constructed instance of [`DscSettingsResolved`].
    pub fn build(self) -> DscSettingsResolved {
        let mut settings = DscSettingsResolved::default();
        if let Some(forbid_ignore_settings_file) = self.forbid_ignore_settings_file {
            settings.forbid_ignore_settings_file = forbid_ignore_settings_file;
        }
        if let Some(ignore_settings_file) = self.ignore_settings_file {
            settings.ignore_settings_file = ignore_settings_file;
        }
        if let Some(tracing) = self.tracing {
            settings.tracing = tracing;
        }
        if let Some(resource_path) = self.resource_path {
            settings.resource_path = resource_path;
        }

        settings
    }
}
