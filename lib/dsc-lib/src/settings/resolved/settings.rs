use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::settings::{DscSettingsResolvedField, DscSettingsScope, CODE_DEFAULT_FORBID_IGNORE_SETTINGS_FILE, CODE_DEFAULT_IGNORE_SETTINGS_FILE, ResourcePathResolvedSettings, TracingResolvedSettings};

/// Defines the effective settings for DSC after resolving all the settings sources.
///
///
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct DscSettingsResolved {
    /// Indicates whether to allow users to ignore settings files.
    ///
    /// This setting is only available in the [`Policy`] scope. The default setting is `false`.
    ///
    /// When this setting is `false`, users can indicate that DSC should not load and resolve
    /// settings files by specifying the [`--ignore-settings-file`] CLI option or defining the
    /// [`DSC_IGNORE_SETTINGS_FILE`] environment variable. Doing so effectively skips processing
    /// the [`Machine`], [`User`], and [`Workspace`] settings scopes. The [`Policy`] scope is
    /// always processed regardless of this setting.
    ///
    /// When the policy scope defines this setting as `true`, users aren't allowed to ignore
    /// settings files. If a user attempts to ignore settings files, DSC raises a warning and
    /// indicates that the command is processing settings files as normal.
    ///
    /// [`Policy`]: DscSettingsScope::Policy
    /// [`Machine`]: DscSettingsScope::Machine
    /// [`User`]: DscSettingsScope::User
    /// [`Workspace`]: DscSettingsScope::Workspace
    /// [`--ignore-settings-file`]: crate::settings::DscSettingsCliData::ignore_settings_file
    /// [`DSC_IGNORE_SETTINGS_FILE`]: crate::settings::DscSettingsEnvironmentData::dsc_ignore_settings_file
    pub forbid_ignore_settings_file: DscSettingsResolvedField<bool>,
    /// Indicates whether to ignore settings files.
    ///
    /// This setting is available in the [`Environment`] and [`Cli`] scopes. The default setting is
    /// `false`.
    ///
    /// When this setting is `true`, DSC ignores the [`Machine`], [`User`], and [`Workspace`]
    /// settings scopes. DSC won't automatically load those settings files and will ignore them
    /// during settings resolution even if they were manually loaded. The [`Policy`] scope is
    /// always processed regardless of this setting.
    ///
    /// If the [`forbid_ignore_settings_file`] setting is defined as `true` in the [`Policy`] scope,
    /// this setting is effectively ignored and DSC will always load and resolve settings files if
    /// they exist.
    ///
    /// [`Environment`]: DscSettingsScope::Environment
    /// [`Cli`]: DscSettingsScope::CommandLine
    /// [`Machine`]: DscSettingsScope::Machine
    /// [`User`]: DscSettingsScope::User
    /// [`Workspace`]: DscSettingsScope::Workspace
    /// [`Policy`]: DscSettingsScope::Policy
    /// [`forbid_ignore_settings_file`]: Self::forbid_ignore_settings_file
    pub ignore_settings_file: DscSettingsResolvedField<bool>,
    /// Indicates how DSC should emit messages during command execution.
    ///
    /// These settings control which messages DSC emits to stderr and the format it emits them  in.
    ///
    /// The following snippet shows the effective code defaults as YAML data:
    ///
    /// ```yaml
    /// tracing:
    ///   level: info
    ///   format: default
    /// ```
    ///
    /// For more information on defining these settings, see:
    ///
    /// - [`TracingFileData`] for defining them in the [policy settings file] or [preference settings files].
    /// - [`DscSettingsEnvironmentData`] for defining them as environment variables.
    /// - [`DscSettingsCliData`] for defining them as CLI options.
    ///
    /// [`TracingFileData`]: crate::settings::TracingFileData
    /// [`DscSettingsEnvironmentData`]: crate::settings::DscSettingsEnvironmentData
    /// [`DscSettingsCliData`]: crate::settings::DscSettingsCliData
    /// [policy settings file]: crate::settings::TracingFileData
    /// [preference settings files]: crate::settings::TracingFileData
    pub tracing: TracingResolvedSettings,
    /// Indicates how DSC should discover manifests and binaries during command execution.
    ///
    /// These settings control which directories DSC searches for manifests and binaries, whether to include the system
    /// `PATH` environment variable in the search, and whether to restrict the search to only the specified directories.
    ///
    /// The following snippet shows the effective code defaults as YAML data:
    ///
    /// ```yaml
    /// resource_path:
    ///   include_system_path: true
    ///   restrict_to_specified_dirs: false
    /// ```
    ///
    ///
    /// For more information on defining these settings, see:
    ///
    /// - [`ResourcePathFileData`] for defining them in the [policy settings file] or [preference settings files].
    /// - [`DscSettingsEnvironmentData`] for defining them as environment variables.
    /// - [`DscSettingsCliData`] for defining them as CLI options.
    ///
    /// [`ResourcePathFileData`]: crate::settings::ResourcePathFileData
    /// [`DscSettingsEnvironmentData`]: crate::settings::DscSettingsEnvironmentData
    /// [`DscSettingsCliData`]: crate::settings::DscSettingsCliData
    /// [policy settings file]: crate::settings::ResourcePathFileData
    /// [preference settings files]: crate::settings::ResourcePathFileData
    pub resource_path: ResourcePathResolvedSettings,
}

impl Default for DscSettingsResolved {
    fn default() -> Self {
        let scope = DscSettingsScope::Default;
        Self {
            forbid_ignore_settings_file: DscSettingsResolvedField::new(CODE_DEFAULT_FORBID_IGNORE_SETTINGS_FILE, scope),
            ignore_settings_file: DscSettingsResolvedField::new(CODE_DEFAULT_IGNORE_SETTINGS_FILE, scope),
            tracing: TracingResolvedSettings::default(),
            resource_path: ResourcePathResolvedSettings::default(),
        }
    }
}
