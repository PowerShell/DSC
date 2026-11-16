// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::schemas::dsc_repo::DscRepoSchema;
use super::macros::*;

define_container_field!{
    base_name: "plugins",
    folder_path: "settings/fields",
    field_definition: {
        /// Defines the available settings for command plug-ins in DSC.
        ///
        /// This struct is used to represent the settings as they are defined
        /// in both [`PolicyFileData`] and [`PreferenceFileData`].
        ///
        /// A command plug-in adds a top-level command to the `dsc` CLI
        /// without being compiled into it. When the first argument to `dsc`
        /// isn't a built-in command, DSC looks for a plug-in manifest that
        /// provides the command and runs the executable that the manifest
        /// names.
        ///
        /// The plug-in settings define whether DSC uses plug-ins at all:
        ///
        /// - When [`enabled`] is resolved as `true`, DSC discovers plug-in
        ///   manifests, lists plug-in commands in the help, and dispatches
        ///   unknown commands to the plug-in that provides them.
        /// - When [`enabled`] is resolved as `false`, DSC doesn't discover
        ///   plug-in manifests. A command that isn't built in fails as an
        ///   unrecognized subcommand, the help doesn't list plug-in commands,
        ///   and listing the plug-ins returns nothing.
        ///
        /// To configure whether DSC uses plug-ins, users can define the
        /// `plugins.enabled` field in the [`Policy`], [`Machine`], [`User`],
        /// or [`Workspace`] settings files. The setting can't be defined as an
        /// environment variable or a command line argument.
        ///
        /// Defining the field in the [`Policy`] settings file gives an
        /// administrator one switch to keep a machine to the built-in
        /// commands, because no other scope can override a policy setting.
        ///
        /// The directories that DSC searches for plug-in manifests are the
        /// same directories it searches for every other manifest. To limit
        /// where plug-ins can be discovered instead of disabling them, see
        /// [`ResourcePathFileData`].
        ///
        /// [`PolicyFileData`]: crate::settings::sources::PolicyFileData
        /// [`PreferenceFileData`]: crate::settings::sources::PreferenceFileData
        /// [`ResourcePathFileData`]: crate::settings::fields::ResourcePathFileData
        /// [`enabled`]: Self::enabled
        /// [`Policy`]: crate::settings::DscSettingsScope::Policy
        /// [`Machine`]: crate::settings::DscSettingsScope::Machine
        /// [`User`]: crate::settings::DscSettingsScope::User
        /// [`Workspace`]: crate::settings::DscSettingsScope::Workspace
        #[schemars(extend("unevaluatedProperties" = false))]
        struct PluginsFileData {
            /// Whether DSC discovers and runs command plug-ins.
            ///
            /// If set to `false`, DSC doesn't discover plug-in manifests and
            /// only the built-in commands are available. The default value is
            /// `true`.
            enabled: bool = true,
        }
    },
    resolved_container_type: {
        /// Defines the resolved settings for command plug-ins in DSC.
        ///
        /// For more information about the available settings, see
        /// [`PluginsFileData`].
        PluginsResolvedSettings
    },
    code_default_type: {
        /// Defines the structure of the code defaults for the plug-in settings
        /// in DSC.
        ///
        /// For more information about the available settings, see
        /// [`PluginsFileData`]. For more information about the code defaults,
        /// see [`CODE_DEFAULT_PLUGINS`].
        PluginsCodeDefaults
    },
    code_default_const: {
        /// Defines the default values for the plug-in settings in DSC.
        ///
        /// The following snippet shows the effective code defaults as YAML
        /// data:
        ///
        /// ```yaml
        /// plugins:
        ///   enabled: true
        /// ```
        ///
        /// With these code defaults, DSC discovers plug-in manifests and
        /// dispatches commands that aren't built in to the plug-in that
        /// provides them.
        ///
        /// For more information about defining the settings, see
        /// [`PluginsFileData`].
        CODE_DEFAULT_PLUGINS
    }
}
