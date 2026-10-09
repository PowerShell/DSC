// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::schemas::dsc_repo::DscRepoSchema;
use super::macros::*;

define_container_field!{
    base_name: "resourcePath",
    folder_path: "settings/fields",
    field_definition: {
        /// Defines the available settings for the resource path configuration
        /// in DSC.
        ///
        /// This struct is used to represent the settings as they are defined
        /// in both [`PolicyFileData`] and [`PreferenceFileData`].
        ///
        /// DSC discovers manifest files by searching a collection of
        /// directories for files with known extensions. By default, DSC only
        /// searches the `PATH` environment variable and the directory that DSC
        /// is installed in.
        ///
        /// When invoking commands for a manifest, DSC supports invoking
        /// commands that exist in `PATH` or that can be resolved relative to
        /// the manifest file.
        ///
        /// The resource path settings define how DSC discovers manifest files
        /// and invokes commands:
        ///
        /// - When [`directories`] is resolved, DSC always searches the
        ///   specified directories for manifests and can invoke commands from
        ///   those directories.
        /// - When [`append_env_path`] is resolved as `true`, DSC appends the
        ///   `PATH` environment variable to the resolved value for
        ///   [`directories`] when searching for manifests.
        /// - When [`restricted`] is resolved as `true`, DSC restricts the
        ///   paths from which it can invoke binaries to the resolved value for
        ///   [`directories`]. When this setting is `true` it effectively
        ///   ignores the `append_env_path` setting and doesn't append the
        ///   `PATH` environment variable to set of directories. DSC will only
        ///   discover and invoke commands from resolved value of
        ///   [`directories`].
        ///
        /// <div class="warning">
        /// <details open><summary>Restricted path behavior</summary>
        ///
        /// When you set [`restricted`] to `true`, DSC will _only_ discover and
        /// invoke commands from the resolved value of [`directories`]. This
        /// explicitly _does not_ consider the `PATH` environment variable or
        /// the DSC installation directory.
        ///
        /// To use built-in resources and extensions, you need to ensure that
        /// [`directories`] includes the DSC installation directory.
        ///
        /// To use resources and extensions that require invoking commands that
        /// aren't adjacent to the manifest file, you need to ensure that
        /// [`directories`] includes the directories containing those commands.
        ///
        /// </details>
        /// </div>
        ///
        /// To configure how DSC discovers manifests and invokes commands,
        /// users can:
        ///
        /// 1. Define the `resourcePath.directories`,
        ///   `resourcePath.appendEnvPath`, and `resourcePath.restricted`
        ///   fields in the [`Policy`], [`Machine`], [`User`], or [`Workspace`]
        ///   settings files.
        /// 1. Define the [`DSC_RESOURCE_PATH`] environment variable as a
        ///    string containing a collection of directories separated by the
        ///    platform specific path separator for `PATH`-like environment
        ///    variables. This effectively populates the [`directories`] field
        ///    in the resolved settings.
        /// 1. Define the [`DSC_RESTRICTED_PATH`] environment variable as a
        ///    string containing a collection of directories separated by the
        ///    platform specific path separator for `PATH`-like environment
        ///    variables. This effectively populates the [`directories`] field
        ///    in the resolved settings and sets the [`restricted`] field in
        ///    the resolved settings to `true`.
        ///
        /// # Examples
        ///
        /// The following examples clarify the effective behavior of DSC
        /// depending on the resource path settings. Each scenario shows the
        /// effective resolved settings for the resource path as a YAML snippet
        /// before explaining how DSC behaves when discovering manifests and
        /// invoking commands.
        ///
        /// 1. ```yaml
        ///    resourcePath:
        ///      directories: []
        ///      append_env_path: true
        ///      restricted: false
        ///    ```
        ///
        ///    DSC searches the directories in the `PATH` environment variable
        ///    and the directory that DSC is installed in for manifests. When
        ///    invoking commands for a manifest, DSC can invoke commands that
        ///    exist in `PATH`, the DSC installation directory, and relative to
        ///    any discovered manifest file.
        ///
        /// 1. ```yaml
        ///    resourcePath:
        ///      directories: ["D:\infra\resources", "D:\infra\tools"]
        ///      append_env_path: true
        ///      restricted: false
        ///    ```
        ///
        ///    DSC searches the speciied directories, the directories in the
        ///    `PATH` environment variable, and the directory that DSC is
        ///    installed in for manifest files. When invoking commands for a
        ///    manifest, DSC can invoke commands that exist those same
        ///    directories and relative to any discovered manifest file.
        ///
        /// 1. ```yaml
        ///    resourcePath:
        ///      directories: ["D:\infra\resources", "D:\infra\tools"]
        ///      append_env_path: true
        ///      restricted: true
        ///    ```
        ///
        ///    DSC _only_ searches the specified directories for manifest files
        ///    and does not consider the `PATH` environment variable or the DSC
        ///    installation directory. When invoking commands for a manifest,
        ///    DSC can only invoke commands that exist in the specified
        ///    directories. Attempting to invoke commands outside of the
        ///    specified directories raises an error.
        ///
        /// [`PolicyFileData`]: crate::settings::sources::PolicyFileData
        /// [`PreferenceFileData`]: crate::settings::sources::PreferenceFileData
        /// [`directories`]: Self::directories
        /// [`append_env_path`]: Self::append_env_path
        /// [`restricted`]: Self::restricted
        /// [`Policy`]: crate::settings::DscSettingsScope::Policy
        /// [`Machine`]: crate::settings::DscSettingsScope::Machine
        /// [`User`]: crate::settings::DscSettingsScope::User
        /// [`Workspace`]: crate::settings::DscSettingsScope::Workspace
        /// [`DSC_RESOURCE_PATH`]: crate::settings::sources::EnvironmentData::dsc_resource_path
        /// [`DSC_RESTRICTED_PATH`]: crate::settings::sources::EnvironmentData::dsc_restricted_path
        #[schemars(extend(
            "unevaluatedProperties" = false,
            "allOf" = [
                {   // if restricted is true, appendEnvPath must be false or not defined
                    "if": {
                        "required": ["restricted"],
                        "properties": { "restricted": { "const": true } }
                    },
                    "then": {
                        "oneOf": [
                            { "not": { "required": ["appendEnvPath"] } },
                            { "required": ["appendEnvPath"], "properties": { "appendEnvPath": { "const": false } } }
                        ]
                    },
                },
                {   // if appendEnvPath is true, restricted must be false or not defined
                    "if": {
                        "required": ["appendEnvPath"],
                        "properties": { "appendEnvPath": { "const": true } }
                    },
                    "then": {
                        "oneOf": [
                            { "not": { "required": ["restricted"] } },
                            { "required": ["restricted"], "properties": { "restricted": { "const": false } } }
                        ]
                    },
                }
            ]
        ))]
        struct ResourcePathFileData {
            /// Directories that DSC should search for executables and
            /// manifests.
            ///
            /// This field specifies the list of directories that DSC will
            /// search for executables and manifests.
            ///
            /// If [`restricted`] is set to `true`, DSC will only invoke
            /// executables in the specified directories.
            ///
            /// [`restricted`]: Self::restricted
            directories: Vec<std::path::PathBuf> = Vec::new(),
            /// Whether to append the `PATH` environment variable.
            ///
            /// If set to `false`, DSC will _not_ append the `PATH` environment
            /// variable to the list of directories.
            ///
            /// If [`restricted`] is set to `true`, DSC ignores this setting.
            /// DSC will only search the paths specified in the [`directories`]
            /// field. Attempts to invoke any executables outside of those
            /// directories will raise an error.
            ///
            /// [`restricted`]: Self::restricted
            /// [`directories`]: Self::directories
            append_env_path: bool = true,
            /// Whether to restrict execution to the specified directories.
            ///
            /// If set to `true`, DSC will restrict execution to the paths
            /// specified in the [`directories`] field and ignore the
            /// [`append_env_path`] setting.
            ///
            /// [`directories`]: Self::directories
            /// [`append_env_path`]: Self::append_env_path
            restricted: bool = false,
        }
    },
    resolved_container_type: {
        /// Defines the resolved settings for the resource path settings
        /// in DSC.
        ///
        /// For more information about the available settings, see
        /// [`ResourcePathFileData`].
        ResourcePathResolvedSettings
    },
    code_default_type: {
        /// Defines the structure of the code defaults for the resource path
        /// settings in DSC.
        ///
        /// For more information about the available settings, see
        /// [`ResourcePathFileData`]. For more information about the code
        /// defaults, see [`CODE_DEFAULT_RESOURCE_PATH`].
        ResourcePathCodeDefaults
    },
    code_default_const: {
        /// Defines the default values for the resource path configuration in
        /// DSC.
        ///
        /// The following snippet shows the effective code defaults as YAML
        /// data:
        ///
        /// ```yaml
        /// resource_path:
        ///   directories: []
        ///   append_env_path: true
        ///   restricted: false
        /// ```
        ///
        /// With these code defaults, DSC will append the `PATH` environment
        /// to the empty set of directories and won't restrict execution to
        /// those directories.
        ///
        /// For more information about defining the settings, see
        /// [`ResourcePathFileData`].
        CODE_DEFAULT_RESOURCE_PATH
    }
}
