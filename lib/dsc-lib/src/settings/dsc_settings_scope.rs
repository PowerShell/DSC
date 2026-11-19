// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use std::{fmt::Display, str::FromStr};

use crate::{
    schemas::{dsc_repo::{DscRepoSchema, schema_i18n}, transforms::idiomaticize_string_enum},
    settings::DscSettingsError
};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Defines the source scope of a setting value for DSC.
///
/// DSC supports multiple sources for settings. This enum represents the scope
/// of a setting value. The scopes are ordered by precedence, with the highest
/// precedence scope being the one that DSC uses:
///
/// 1. [`Default`]
/// 1. [`Machine`]
/// 1. [`User`]
/// 1. [`Workspace`]
/// 1. [`Environment`]
/// 1. [`CommandLine`]
/// 1. [`Policy`]
///
/// The highest precedence source is [`Policy`], which is defined in the
/// machine policy file. Fields defined as policy cannot be overridden by any
/// other source, including environment variables or command line arguments.
///
/// [`Default`]: Self::Default
/// [`Machine`]: Self::Machine
/// [`User`]: Self::User
/// [`Workspace`]: Self::Workspace
/// [`Environment`]: Self::Environment
/// [`CommandLine`]: Self::CommandLine
/// [`Policy`]: Self::Policy
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema, DscRepoSchema)]
#[serde(rename_all = "camelCase", try_from = "String", into = "String")]
#[schemars(!try_from, !into, transform = idiomaticize_string_enum)]
#[dsc_repo_schema(base_name = "scope", folder_path = "settings")]
#[schemars(
    title = schema_i18n!("title"),
    description = schema_i18n!("description"),
    extend(
        "$id" = DscSettingsScope::default_export_schema_id_uri(),
        "markdownDescription" = schema_i18n!("markdownDescription")
    )

)]
pub enum DscSettingsScope {
    /// The default settings staticalally defined in the DSC codebase.
    ///
    /// For more information about the default settings, see
    /// [`DscSettingsCodeDefaults`] and [`DSC_SETTINGS_CODE_DEFAULTS`].
    ///
    /// [`DscSettingsCodeDefaults`]: crate::settings::sources::DscSettingsCodeDefaults
    /// [`DSC_SETTINGS_CODE_DEFAULTS`]: crate::settings::sources::DSC_SETTINGS_CODE_DEFAULTS
    #[schemars(
        title = schema_i18n!("variants.default.title"),
        description = schema_i18n!("variants.default.description"),
        extend("markdownDescription" = schema_i18n!("variants.default.markdownDescription"))
    )]
    Default,
    /// The settings defined for all users on the machine in a preference
    /// settings file.
    ///
    /// The location for the settings file in this scope depends on the
    /// operating system:
    ///
    /// - On Windows: `{ProgramData}\dsc\dsc.settings.json`
    /// - On macOS: `/Library/Application Support/dsc/dsc.settings.json`
    /// - On Linux and other Unix-like systems: `/etc/dsc/dsc.settings.json`
    ///
    /// For more information on the settings you can define for all users, see
    /// [`PreferenceFileData`].
    ///
    /// [`PreferenceFileData`]: crate::settings::sources::PreferenceFileData
    #[schemars(
        title = schema_i18n!("variants.machine.title"),
        description = schema_i18n!("variants.machine.description"),
        extend("markdownDescription" = schema_i18n!("variants.machine.markdownDescription"))
    )]
    Machine,
    /// The settings defined for the current user in a preference settings file.
    ///
    /// The location for the settings file in this scope depends on the
    /// operating system and whether the `XDG_CONFIG_HOME` environment variable
    /// is set:
    ///
    /// - When `XDG_CONFIG_HOME` is set, DSC looks for the settings file at
    ///   `{XDG_CONFIG_HOME}/dsc/user_settings.json` (using `\` instead of `/`
    ///   on Windows).
    /// - When `XDG_CONFIG_HOME` is not set, DSC looks for the settings file
    ///   depending on the operating system:
    ///
    ///   - On Windows: `{APPDATA}\dsc\dsc.settings.json`
    ///   - On macOS: `{HOME}/Library/Application Support/dsc/dsc.settings.json`
    ///   - On Linux and other Unix-like systems: `{HOME}/.config/dsc/dsc.settings.json`
    ///
    /// For more information on the settings you can define for a user, see
    /// [`PreferenceFileData`].
    ///
    /// [`PreferenceFileData`]: crate::settings::sources::PreferenceFileData
    #[schemars(
        title = schema_i18n!("variants.user.title"),
        description = schema_i18n!("variants.user.description"),
        extend("markdownDescription" = schema_i18n!("variants.user.markdownDescription"))
    )]
    User,
    /// The settings defined for the current workspace in a preference settings
    /// file.
    ///
    /// The location for the settings file in this scope is always
    /// `{CWD}/dsc.settings.json`, where `CWD` is the current working directory.
    ///
    /// For more information on the settings you can define for a workspace,
    /// see [`PreferenceFileData`].
    ///
    /// [`PreferenceFileData`]: crate::settings::sources::PreferenceFileData
    #[schemars(
        title = schema_i18n!("variants.workspace.title"),
        description = schema_i18n!("variants.workspace.description"),
        extend("markdownDescription" = schema_i18n!("variants.workspace.markdownDescription"))
    )]
    Workspace,
    /// Settings defined as environment variables.
    ///
    /// For more information on the settings you can define as environment
    /// variables, see [`EnvironmentData`].
    ///
    /// [`EnvironmentData`]: crate::settings::sources::EnvironmentData
    #[schemars(
        title = schema_i18n!("variants.environment.title"),
        description = schema_i18n!("variants.environment.description"),
        extend("markdownDescription" = schema_i18n!("variants.environment.markdownDescription"))
    )]
    Environment,
    /// Settings defined as command line arguments.
    ///
    /// For more information on the settings you can define as command line
    /// arguments, see [`CommandLineData`].
    ///
    /// [`CommandLineData`]: crate::settings::sources::CommandLineData
    #[schemars(
        title = schema_i18n!("variants.cli.title"),
        description = schema_i18n!("variants.cli.description"),
        extend("markdownDescription" = schema_i18n!("variants.cli.markdownDescription"))
    )]
    #[serde(rename = "cli")]
    CommandLine,
    /// The system policy file. Fields defined as policy cannot be overridden.
    ///
    /// Every setting for DSC can be defined in the system policy file. The
    /// location for the policy file depends on the operating system:
    ///
    /// - On Windows: `{ProgramData}\dsc\dsc.policy.json`
    /// - On macOS: `/Library/Application Support/dsc/dsc.policy.json`
    /// - On Linux and other Unix-like systems: `/etc/dsc/dsc.policy.json`
    ///
    /// For more information on the settings you can define as policy, see
    /// [`PolicyFileData`].
    ///
    /// [`PolicyFileData`]: crate::settings::sources::PolicyFileData
    #[schemars(
        title = schema_i18n!("variants.policy.title"),
        description = schema_i18n!("variants.policy.description"),
        extend("markdownDescription" = schema_i18n!("variants.policy.markdownDescription"))
    )]
    Policy,
}

// Public API
impl DscSettingsScope {
    /// Defines all possible scopes for the `DscSettingsScope` enum for
    /// convenience when iterating.
    pub const ALL: [DscSettingsScope; 7] = [
        DscSettingsScope::Default,
        DscSettingsScope::Machine,
        DscSettingsScope::User,
        DscSettingsScope::Workspace,
        DscSettingsScope::Environment,
        DscSettingsScope::CommandLine,
        DscSettingsScope::Policy,
    ];
    /// Defines the scopes that read from a settings preference file.
    pub const FOR_PREFERENCE_FILES: [DscSettingsScope; 3] = [
        DscSettingsScope::Machine,
        DscSettingsScope::User,
        DscSettingsScope::Workspace,
    ];

    /// Parses a string into a `DscSettingsScope`.
    ///
    /// This method parses the input string case insensitively.
    ///
    /// # Arguments
    ///
    /// * `s` - A string slice that holds the name of the scope.
    ///
    /// # Returns
    ///
    /// Returns the matching variant for [`DscSettingsScope`] if the string is
    /// parseable.
    ///
    /// # Errors
    ///
    /// Returns a [`DscSettingsError::InvalidScope`] if the string doesn't
    /// match any valid scope.
    pub fn parse(s: &str) -> Result<DscSettingsScope, DscSettingsError> {
        match s.to_lowercase().as_ref() {
            "default" => Ok(DscSettingsScope::Default),
            "machine" => Ok(DscSettingsScope::Machine),
            "user" => Ok(DscSettingsScope::User),
            "workspace" => Ok(DscSettingsScope::Workspace),
            "environment" => Ok(DscSettingsScope::Environment),
            "cli" => Ok(DscSettingsScope::CommandLine),
            "policy" => Ok(DscSettingsScope::Policy),
            _ => Err(DscSettingsError::InvalidScope { text: s.to_string() }),
        }
    }

    /// Indicates whether the scope is loaded by reading a settings file.
    ///
    /// # Returns
    ///
    /// If the scope is [`Policy`], [`Machine`], [`User`], or [`Workspace`],
    /// this method returns `true`. Otherwise, it returns `false`.
    ///
    /// [`Policy`]: Self::Policy
    /// [`Machine`]: Self::Machine
    /// [`User`]: Self::User
    /// [`Workspace`]: Self::Workspace
    pub fn is_file_based(&self) -> bool {
        matches!(self,
            DscSettingsScope::Machine
            | DscSettingsScope::User
            | DscSettingsScope::Workspace
            | DscSettingsScope::Policy
        )
    }

    /// Indicates whether the scope is a preference settings file.
    ///
    /// # Returns
    ///
    /// If the scope is [`Machine`], [`User`], or [`Workspace`], this method
    /// returns `true`. Otherwise, it returns `false`.
    ///
    /// [`Machine`]: Self::Machine
    /// [`User`]: Self::User
    /// [`Workspace`]: Self::Workspace
    pub fn is_preference(&self) -> bool {
        matches!(self,
            DscSettingsScope::Machine
            | DscSettingsScope::User
            | DscSettingsScope::Workspace
        )
    }

    /// Indicates whether the scope is the system policy file.
    ///
    /// # Returns
    ///
    /// If the scope is [`Policy`], this method returns `true`. Otherwise, it
    /// returns `false`.
    ///
    /// [`Policy`]: Self::Policy
    pub fn is_policy(&self) -> bool {
        matches!(self, DscSettingsScope::Policy)
    }
}

// Private API
impl DscSettingsScope {
    /// A list of all variant names for the `DscSettingsScope` enum, excluding
    /// the last variant.
    ///
    /// Used in error messaging only.
    pub(super) const VARIANT_LIST: &str = "`default`, `machine`, `user`, `workspace`, `environment`, `cli`";
    /// The last variant name for the `DscSettingsScope` enum.
    ///
    /// Used in error messaging only.
    pub(super) const VARIANT_LAST: &str = "`policy`";
}

// Enables `to_string()`
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

// Enables `"text".parse()`
impl FromStr for DscSettingsScope {
    type Err = DscSettingsError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

impl TryFrom<String> for DscSettingsScope {
    type Error = DscSettingsError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::parse(&value)
    }
}

impl From<DscSettingsScope> for String {
    fn from(scope: DscSettingsScope) -> Self {
        scope.to_string()
    }
}

impl TryFrom<&str> for DscSettingsScope {
    type Error = DscSettingsError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::parse(value)
    }
}

// Remaining traits enable comparison
impl PartialEq<&str> for DscSettingsScope {
    fn eq(&self, other: &&str) -> bool {
        self.to_string().as_str() == other.to_lowercase().as_str()
    }
}

impl PartialEq<DscSettingsScope> for &str {
    fn eq(&self, other: &DscSettingsScope) -> bool {
        self.to_lowercase().as_str() == other.to_string().as_str()
    }
}

impl PartialEq<String> for DscSettingsScope {
    fn eq(&self, other: &String) -> bool {
        self.to_string().as_str() == other.to_lowercase().as_str()
    }
}

impl PartialEq<DscSettingsScope> for String {
    fn eq(&self, other: &DscSettingsScope) -> bool {
        self.to_lowercase().as_str() == other.to_string().as_str()
    }
}
