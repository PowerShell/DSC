// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! Defines sources for DSC settings, including code defaults, policy and preference files,
//! environment variables, and command line arguments.
//!
//! DSC settings are resolved for multiple scopes, each of which has a specific source. The
//! following table shows how every scope maps to a source:
//!
//! |      Scope      |           Source            |
//! |:---------------:|:---------------------------:|
//! |   [`Default`]   | [`DscSettingsCodeDefaults`] |
//! |   [`Machine`]   |   [`PreferenceFileData`]    |
//! |    [`User`]     |   [`PreferenceFileData`]    |
//! |  [`Workspace`]  |   [`PreferenceFileData`]    |
//! | [`Environment`] |     [`EnvironmentData`]     |
//! | [`CommandLine`] |     [`CommandLineData`]     |
//! |   [`Policy`]    |     [`PolicyFileData`]      |
//!
//! The sources defined in this module depend on the types defined in the [`fields`] module. Every
//! top-level field is defined a struct or enum in the [`fields`] module.
//!
//! Every setting field is included in the [`PolicyFileData`] struct to enable systems
//! administrators to fully control how DSC behaves. Most settings are defined in the
//! [`PreferenceFileData`] struct to enable users to define their own preferences. A smaller set
//! of settings are defined in the [`EnvironmentData`] struct to enable users to override settings
//! on a per-command basis. An even smaller set of settings are defined in the [`CommandLineData`]
//! struct to support overriding a setting for a single command invocation.
//!
//! [`Default`]: super::DscSettingsScope::Default
//! [`Machine`]: super::DscSettingsScope::Machine
//! [`User`]: super::DscSettingsScope::User
//! [`Workspace`]: super::DscSettingsScope::Workspace
//! [`Environment`]: super::DscSettingsScope::Environment
//! [`CommandLine`]: super::DscSettingsScope::CommandLine
//! [`Policy`]: super::DscSettingsScope::Policy
//! [`fields`]: super::fields

#[cfg(doc)]
pub(in crate::settings) mod _implementation_guidance {
    //! Every setting field _must_ be defined in the [`PolicyFileData`] struct to enable systems
    //! administrators to fully control how DSC behaves in production environments. Fields may be
    //! defined in other sources using the following guidelines:
    //!
    //! 1. Define the field in the [`PreferenceFileData`] struct unless the field is strictly
    //!    applicable as policy. For example, the `forbid_ignore_settings_file` only makes sense in
    //!    the policy file.
    //! 1. If the field controls behavior that a user may want to override on a per-command *or
    //!    per-terminal-session basis, define the field in the [`EnvironmentData`] struct to enable
    //!    users to override the field using an environment variable.
    //!
    //!    Non-Windows platforms allow users to prepend environment variables to a command to
    //!    affect behavior for that invocation only. We want to support this idiomatic behavior.
    //!    For example:
    //!
    //!    ```sh
    //!    # Uses trace level from defaults/files
    //!    dsc config --parameter-file ./example.dsc.params.yaml get -f ./example.dsc.config.yaml
    //!    # Overrides trace level for this invocation only
    //!    DSC_TRACE_LEVEL=debug dsc config get -f ./example.dsc.config.yaml
    //!    ```
    //!
    //!    For guidance on how to define the environment variable for a field, see the
    //!    [`environment`] module [guidance](super::environment::_implementation_guidance).
    //! 1. Only define the field as a command line argument if it improves the user experience
    //!    _substantially_ and helps with discoverability. Any field represented in the CLI must be
    //!    defined for the root command. The more options available on the root command, the higher
    //!    the cognitive load for users.
    //!
    //!    Only surface critical settings and extremely common settings in the CLI.
    //!
    //!    For guidance on how to define the command line argument for a field, see the
    //!    [`command_line`] module.
    //!
    //! [`PreferenceFileData`]: super::PreferenceFileData
    //! [`PolicyFileData`]: super::PolicyFileData
    //! [`EnvironmentData`]: super::EnvironmentData
    //! [`environment`]: super::environment
    //! [`command_line`]: super::command_line
}

mod command_line;
pub use command_line::*;
mod code_defaults;
pub use code_defaults::*;
mod environment;
pub use environment::*;
mod preference_file;
pub use preference_file::*;
mod policy_file;
pub use policy_file::*;
