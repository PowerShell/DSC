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

use dsc_lib::settings::{DscPolicyFileData, DscSettingsResolvedField, DscSettings, DscSettingsCliData, DscSettingsEnvironmentData, DscPreferenceFileData, DscSettingsResolved, DscSettingsScope, ResourcePathFileData, ResourcePathResolvedSettings, TraceFormatField, TraceLevelField, TracingFileData, TracingResolvedSettings};

pub struct TracingDataBuilder {
    level: Option<TraceLevelField>,
    format: Option<TraceFormatField>,
}

impl TracingDataBuilder {
    pub fn new() -> Self {
        Self {
            level: None,
            format: None,
        }
    }

    pub fn with_level(mut self, level: TraceLevelField) -> Self {
        self.level = Some(level);
        self
    }

    pub fn with_format(mut self, format: TraceFormatField) -> Self {
        self.format = Some(format);
        self
    }

    pub fn build(self) -> TracingFileData {
        TracingFileData {
            level: self.level,
            format: self.format,
        }
    }
}

pub struct ResourcePathDataBuilder {
    append_env_path: Option<bool>,
    directories: Option<Vec<String>>,
    restrict_path: Option<bool>,
}

impl ResourcePathDataBuilder {
    pub fn new() -> Self {
        Self {
            append_env_path: None,
            directories: None,
            restrict_path: None,
        }
    }

    pub fn with_append_env_path(mut self, append: bool) -> Self {
        self.append_env_path = Some(append);
        self
    }

    pub fn with_directories(mut self, dirs: Vec<String>) -> Self {
        self.directories = Some(dirs);
        self
    }

    pub fn with_restrict_path(mut self, restrict: bool) -> Self {
        self.restrict_path = Some(restrict);
        self
    }

    pub fn build(self) -> ResourcePathFileData {
        ResourcePathFileData {
            append_env_path: self.append_env_path,
            directories: self.directories,
            restricted: self.restrict_path,
        }
    }
}

pub struct PolicyDataBuilder {
    pub forbid_ignore_settings_file: Option<bool>,
    pub tracing: Option<TracingFileData>,
    pub resource_path: Option<ResourcePathFileData>,
}

impl PolicyDataBuilder {
    pub fn new() -> Self {
        Self {
            forbid_ignore_settings_file: None,
            tracing: None,
            resource_path: None,
        }
    }

    pub fn with_forbid_ignore_settings_file(mut self, forbid: bool) -> Self {
        self.forbid_ignore_settings_file = Some(forbid);
        self
    }

    pub fn with_tracing(mut self, tracing: TracingFileData) -> Self {
        self.tracing = Some(tracing);
        self
    }

    pub fn with_resource_path(mut self, resource_path: ResourcePathFileData) -> Self {
        self.resource_path = Some(resource_path);
        self
    }

    pub fn build(self) -> DscPolicyFileData {
        DscPolicyFileData {
            forbid_ignore_settings_file: self.forbid_ignore_settings_file,
            tracing: self.tracing,
            resource_path: self.resource_path,
        }
    }
}

pub struct PreferenceDataBuilder {
    pub tracing: Option<TracingFileData>,
    pub resource_path: Option<ResourcePathFileData>,
}
impl PreferenceDataBuilder {
    pub fn new() -> Self {
        Self {
            tracing: None,
            resource_path: None,
        }
    }

    pub fn with_tracing(mut self, tracing: TracingFileData) -> Self {
        self.tracing = Some(tracing);
        self
    }

    pub fn with_resource_path(mut self, resource_path: ResourcePathFileData) -> Self {
        self.resource_path = Some(resource_path);
        self
    }

    pub fn build(self) -> DscPreferenceFileData {
        DscPreferenceFileData {
            tracing: self.tracing,
            resource_path: self.resource_path,
        }
    }
}

pub struct CommandLineDataBuilder {
    pub trace_level: Option<TraceLevelField>,
    pub trace_format: Option<TraceFormatField>,
    pub ignore_settings_file: Option<bool>,
}

impl CommandLineDataBuilder {
    pub fn new() -> Self {
        Self {
            trace_level: None,
            trace_format: None,
            ignore_settings_file: None,
        }
    }

    pub fn with_trace_level(mut self, level: TraceLevelField) -> Self {
        self.trace_level = Some(level);
        self
    }

    pub fn with_trace_format(mut self, format: TraceFormatField) -> Self {
        self.trace_format = Some(format);
        self
    }

    pub fn with_ignore_settings_file(mut self, ignore: bool) -> Self {
        self.ignore_settings_file = Some(ignore);
        self
    }

    pub fn build(self) -> DscSettingsCliData {
        DscSettingsCliData {
            trace_level: self.trace_level,
            trace_format: self.trace_format,
            ignore_settings_file: self.ignore_settings_file,
        }
    }
}

pub struct EnvironmentDataBuilder {
    dsc_trace_level: Option<TraceLevelField>,
    dsc_trace_format: Option<TraceFormatField>,
    dsc_resource_path: Option<Vec<PathBuf>>,
    dsc_restricted_path: Option<Vec<PathBuf>>,
    dsc_ignore_settings_file: Option<bool>,
}

impl EnvironmentDataBuilder {
    pub fn new() -> Self {
        Self {
            dsc_trace_level: None,
            dsc_trace_format: None,
            dsc_resource_path: None,
            dsc_restricted_path: None,
            dsc_ignore_settings_file: None,
        }
    }
    pub fn with_trace_level(mut self, level: TraceLevelField) -> Self {
        self.dsc_trace_level = Some(level);
        self
    }

    pub fn with_trace_format(mut self, format: TraceFormatField) -> Self {
        self.dsc_trace_format = Some(format);
        self
    }

    pub fn with_resource_path(mut self, path: Vec<PathBuf>) -> Self {
        self.dsc_resource_path = Some(path);
        self
    }

    pub fn with_restricted_path(mut self, path: Vec<PathBuf>) -> Self {
        self.dsc_restricted_path = Some(path);
        self
    }

    pub fn with_ignore_settings_file(mut self, ignore: bool) -> Self {
        self.dsc_ignore_settings_file = Some(ignore);
        self
    }
    pub fn build(self) -> DscSettingsEnvironmentData {
        DscSettingsEnvironmentData {
            dsc_trace_level: self.dsc_trace_level,
            dsc_trace_format: self.dsc_trace_format,
            dsc_resource_path: self.dsc_resource_path,
            dsc_restricted_path: self.dsc_restricted_path,
            dsc_ignore_settings_file: self.dsc_ignore_settings_file,
        }
    }
}

pub struct CliDataBuilder {
    pub trace_level: Option<TraceLevelField>,
    pub trace_format: Option<TraceFormatField>,
    pub ignore_settings_file: Option<bool>,
}

impl CliDataBuilder {
    pub fn new() -> Self {
        Self {
            trace_level: None,
            trace_format: None,
            ignore_settings_file: None,
        }
    }

    pub fn with_trace_level(mut self, level: TraceLevelField) -> Self {
        self.trace_level = Some(level);
        self
    }

    pub fn with_trace_format(mut self, format: TraceFormatField) -> Self {
        self.trace_format = Some(format);
        self
    }

    pub fn with_ignore_settings_file(mut self, ignore: bool) -> Self {
        self.ignore_settings_file = Some(ignore);
        self
    }

    pub fn build(self) -> DscSettingsCliData {
        DscSettingsCliData {
            trace_level: self.trace_level,
            trace_format: self.trace_format,
            ignore_settings_file: self.ignore_settings_file,
        }
    }
}

pub struct SettingsBuilder {
    machine: Option<DscPreferenceFileData>,
    user: Option<DscPreferenceFileData>,
    workspace: Option<DscPreferenceFileData>,
    environment: Option<DscSettingsEnvironmentData>,
    command_line: Option<DscSettingsCliData>,
    policy: Option<DscPolicyFileData>,
}

impl SettingsBuilder {
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

    pub fn with_machine(mut self, machine_data: DscPreferenceFileData) -> Self {
        self.machine = Some(machine_data);
        self
    }

    pub fn with_user(mut self, user_data: DscPreferenceFileData) -> Self {
        self.user = Some(user_data);
        self
    }

    pub fn with_workspace(mut self, workspace_data: DscPreferenceFileData) -> Self {
        self.workspace = Some(workspace_data);
        self
    }
    pub fn with_environment(mut self, environment_data: DscSettingsEnvironmentData) -> Self {
        self.environment = Some(environment_data);
        self
    }
    pub fn with_command_line(mut self, command_line_data: DscSettingsCliData) -> Self {
        self.command_line = Some(command_line_data);
        self
    }
    pub fn with_policy(mut self, policy_data: DscPolicyFileData) -> Self {
        self.policy = Some(policy_data);
        self
    }
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

pub struct ResourcePathResolvedSettingsBuilder {
    pub append_env_path: Option<DscSettingsResolvedField<bool>>,
    pub directories: Option<DscSettingsResolvedField<Vec<String>>>,
    pub restrict_path: Option<DscSettingsResolvedField<bool>>,
}

impl ResourcePathResolvedSettingsBuilder {
    pub fn new() -> Self {
        Self {
            append_env_path: None,
            directories: None,
            restrict_path: None,
        }
    }

    pub fn with_append_env_path(mut self, value: bool, scope: DscSettingsScope) -> Self {
        self.append_env_path = Some(DscSettingsResolvedField::new(value, scope));
        self
    }

    pub fn with_directories(mut self, value: Vec<String>, scope: DscSettingsScope) -> Self {
        self.directories = Some(DscSettingsResolvedField::new(value, scope));
        self
    }

    pub fn with_restrict_path(mut self, value: bool, scope: DscSettingsScope) -> Self {
        self.restrict_path = Some(DscSettingsResolvedField::new(value, scope));
        self
    }

    pub fn build(self) -> ResourcePathResolvedSettings {
        let mut resolved = ResourcePathResolvedSettings::code_defaults();
        if let Some(append_env_path) = self.append_env_path {
            resolved.append_env_path = append_env_path;
        }
        if let Some(directories) = self.directories {
            resolved.directories = directories;
        }
        if let Some(restrict_path) = self.restrict_path {
            resolved.restrict_path = restrict_path;
        }

        resolved
    }
}

pub struct TracingResolvedSettingsBuilder {
    pub level: Option<DscSettingsResolvedField<TraceLevelField>>,
    pub format: Option<DscSettingsResolvedField<TraceFormatField>>,
}

impl TracingResolvedSettingsBuilder {
    pub fn new() -> Self {
        Self {
            level: None,
            format: None,
        }
    }

    pub fn with_level(mut self, value: TraceLevelField, scope: DscSettingsScope) -> Self {
        self.level = Some(DscSettingsResolvedField::new(value, scope));
        self
    }

    pub fn with_format(mut self, value: TraceFormatField, scope: DscSettingsScope) -> Self {
        self.format = Some(DscSettingsResolvedField::new(value, scope));
        self
    }

    pub fn build(self) -> TracingResolvedSettings {
        let mut resolved = TracingResolvedSettings::code_defaults();
        if let Some(level) = self.level {
            resolved.level = level;
        }
        if let Some(format) = self.format {
            resolved.format = format;
        }

        resolved
    }
}

pub struct ResolvedSettingsBuilder {
    pub forbid_ignore_settings_file: Option<DscSettingsResolvedField<bool>>,
    pub ignore_settings_file: Option<DscSettingsResolvedField<bool>>,
    pub tracing: Option<TracingResolvedSettings>,
    pub resource_path: Option<ResourcePathResolvedSettings>,
}

impl ResolvedSettingsBuilder {
    pub fn new() -> Self {
        Self {
            forbid_ignore_settings_file: None,
            ignore_settings_file: None,
            tracing: None,
            resource_path: None,
        }
    }

    pub fn with_forbid_ignore_settings_file(mut self, value: bool, scope: DscSettingsScope) -> Self {
        self.forbid_ignore_settings_file = Some(DscSettingsResolvedField::new(value, scope));
        self
    }

    pub fn with_ignore_settings_file(mut self, value: bool, scope: DscSettingsScope) -> Self {
        self.ignore_settings_file = Some(DscSettingsResolvedField::new(value, scope));
        self
    }

    pub fn with_tracing(mut self, tracing: TracingResolvedSettings) -> Self {
        self.tracing = Some(tracing);
        self
    }

    pub fn with_resource_path(mut self, resource_path: ResourcePathResolvedSettings) -> Self {
        self.resource_path = Some(resource_path);
        self
    }
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
