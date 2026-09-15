//! Defines the default values for DSC settings fields.
//! 
//! The [`DscSettingsCodeDefaults`] struct should mirror the structure of [`DscSettingsResolved`],
//! except that instead of using [`DscSettingsResolvedField<T>`] for each field, it should use the
//! underlying value type for each field.
//! 
//! The [`DSC_SETTINGS_CODE_DEFAULTS`] constant is a static representation of the code defaults and
//! every field should be initialized with the appropriate constant from the [`fields`] module.
//! 
//! [`DscSettingsResolved`]: crate::settings::DscSettingsResolved
//! [`DscSettingsResolvedField<T>`]: crate::settings::DscSettingsResolvedField
//! [`fields`]: crate::settings::fields
use schemars::JsonSchema;
use serde::Serialize;

use crate::settings::{
    CODE_DEFAULT_FORBID_IGNORE_SETTINGS_FILE,
    CODE_DEFAULT_RESOURCE_PATH,
    ResourcePathCodeDefaults,
    CODE_DEFAULT_TRACING,
    TracingCodeDefaults
};

/// Defines the default values for DSC settings fields.
/// 
/// DSC uses a layered approach to resolving settings values. The code defaults, which this struct
/// represents, are the lowest precedence in the settings hierarchy. They are defined in the
/// [`DSC_SETTINGS_CODE_DEFAULTS`] constant.
/// 
/// These defaults are used when no other sources define a value for a setting.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, JsonSchema)]
pub struct DscSettingsCodeDefaults {
    /// Indicates whether to allow users to ignore settings files.
    /// 
    /// The code default for this setting is `false`, which means that users are allowed to ignore
    /// settings files. This setting can only be overridden by the [`Policy`] scope.
    /// 
    /// For more information, see [`forbid_ignore_settings_file`] in the policy file documentation.
    /// 
    /// [`Policy`]: crate::settings::DscSettingsScope::Policy
    /// [`forbid_ignore_settings_file`]: crate::settings::DscPolicyFileData::forbid_ignore_settings_file
    pub forbid_ignore_settings_file: bool,
    /// Indicates whether to ignore settings files.
    /// 
    /// The code default for this setting is `false`, which means that DSC will load and resolve
    /// settings files. This setting can be overridden by the [`Environment`] and [`CLI`] scopes.
    /// 
    /// If the [`forbid_ignore_settings_file`] setting is defined as `true` in the [`Policy`] scope,
    /// this setting is effectively ignored and DSC will always load and resolve settings files.
    /// 
    /// For more information, see [`DscSettingsResolved::ignore_settings_file`].
    /// 
    /// [`Environment`]: crate::settings::DscSettingsScope::Environment
    /// [`CLI`]: crate::settings::DscSettingsScope::CommandLine
    /// [`Policy`]: crate::settings::DscSettingsScope::Policy
    /// [`forbid_ignore_settings_file`]: crate::settings::DscPolicyFileData::forbid_ignore_settings_file
    /// [`DscSettingsResolved::ignore_settings_file`]: crate::settings::DscSettingsResolved::ignore_settings_file
    pub ignore_settings_file: bool,
    /// Defines how DSC should emit trace messages for logging and diagnostics.
    /// 
    /// 
    pub tracing: TracingCodeDefaults,
    /// Defines the paths to use when searching for and invoking resources, extensions, and other
    /// executables.
    pub resource_path: ResourcePathCodeDefaults,
}

/// Defines the default values for DSC settings fields.
/// 
/// The following snippet shows the effective code defaults as YAML data:
/// 
/// ```yaml
/// forbid_ignore_settings_file: false
/// ignore_settings_file: false
/// tracing:
///   level: warn
///   format: default
/// resource_path:
///   append_env_path: true
///   directories: []
///   restricted: false
/// ```
pub const DSC_SETTINGS_CODE_DEFAULTS: DscSettingsCodeDefaults = DscSettingsCodeDefaults {
    forbid_ignore_settings_file: CODE_DEFAULT_FORBID_IGNORE_SETTINGS_FILE,
    ignore_settings_file: false,
    tracing: CODE_DEFAULT_TRACING,
    resource_path: CODE_DEFAULT_RESOURCE_PATH,
};

impl Default for DscSettingsCodeDefaults {
    fn default() -> Self {
        DSC_SETTINGS_CODE_DEFAULTS
    }
}
