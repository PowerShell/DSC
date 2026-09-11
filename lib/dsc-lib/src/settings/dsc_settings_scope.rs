use std::fmt::Display;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Defines the source of a setting value.
/// 
/// DSC supports multiple sources for settings. This enum represents the source of a setting value.
/// The sources are ordered by precedence, with the highest precedence source being the one that
/// DSC uses.
/// 
/// The highest precedence source is [`Policy`], which is defined in the machine policy file. Fields
/// defined as policy cannot be overridden by any other source, including environment variables or
/// command line arguments.
/// 
/// [`Policy`]: Self::Policy
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum DscSettingsScope {
    /// The default settings staticalally defined in the DSC codebase.
    Default,
    /// The settings defined for all users on the machine in a [preference settings file].
    /// 
    /// The location for the settings file in this scope depends on the operating system:
    /// 
    /// - On Windows, it is typically located at `{PROGRAM_DATA}\DSC\machine_settings.json`.
    /// - On Unix-like systems, it is typically located at `/etc/dsc/machine_settings.json`.
    /// 
    /// [preference settings file]: crate::settings::DscPreferenceFileData
    Machine,
    /// The settings defined for the current user in a [preference settings file].
    /// 
    /// [preference settings file]: crate::settings::DscPreferenceFileData
    User,
    /// The settings defined for the current workspace in a [preference settings file].
    /// 
    /// [preference settings file]: crate::settings::DscPreferenceFileData
    Workspace,
    /// Settings defined as environment variables.
    Environment,
    /// Settings defined as command line arguments.
    #[serde(rename = "cli")]
    CommandLine,
    /// The system policy file. Fields defined as policy cannot be overridden.
    Policy,
}

impl DscSettingsScope {
    pub const ALL: [DscSettingsScope; 7] = [
        DscSettingsScope::Default,
        DscSettingsScope::Machine,
        DscSettingsScope::User,
        DscSettingsScope::Workspace,
        DscSettingsScope::Environment,
        DscSettingsScope::CommandLine,
        DscSettingsScope::Policy,
    ];
    pub const FILE_BASED: [DscSettingsScope; 3] = [
        DscSettingsScope::Machine,
        DscSettingsScope::User,
        DscSettingsScope::Workspace,
    ];
}

impl Display for DscSettingsScope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let source_str = match self {
            DscSettingsScope::Default => "default",
            DscSettingsScope::Machine => "machine",
            DscSettingsScope::User => "user",
            DscSettingsScope::Workspace => "workspace",
            DscSettingsScope::Environment => "environment",
            DscSettingsScope::CommandLine => "cli",
            DscSettingsScope::Policy => "policy",
        };
        write!(f, "{}", source_str)
    }
}
