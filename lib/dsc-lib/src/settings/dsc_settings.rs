// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use rust_i18n::t;
use schemars::JsonSchema;
use serde::Serialize;
use tracing::{debug, trace,warn};

#[cfg(not(test))]
use std::path::Path;

#[cfg(test)]
use crate::tests::stubs::path::Path;

use crate::{
    schemas::dsc_repo::{DscRepoSchema, schema_i18n},
    settings::{
        DscSettingsError, DscSettingsResolved, DscSettingsResolvedField, DscSettingsScope,
        MACHINE_SETTINGS_FILE_PATH, POLICY_SETTINGS_FILE_PATH, USER_SETTINGS_FILE_PATH, WORKSPACE_SETTINGS_FILE_PATH,
        sources::*
    }
};

#[cfg(doc)]
pub(in crate::settings) mod _implementation_guidance {
    //! The public API for [`DscSettings`] should rarely require updates, even when defining new
    //! fields. Unless the newly defined field impacts settings resolution and loading _generally_,
    //! it only needs to be addressed in the various `resolve_*` private methods.
    //!
    //! When a new field is defined, ensure that:
    //!
    //! 1. You update the [`resolve_policy()`] method to resolve the field using the `resolve_field`
    //!    macro defined in the method:
    //!
    //!    - If the new field is a top-level leaf field, add a line like:
    //!
    //!      ```rust, ignore
    //!      resolve_field!(new_field_name);
    //!      ```
    //!
    //!    - If the new field is a top-level container field, add a line like:
    //!
    //!      ```rust, ignore
    //!      resolve_field!(container_name [first_leaf_field, second_leaf_field]);
    //!      ```
    //!
    //!    - If the new field is a leaf in an existing container, find the line for the container
    //!      and add the new leaf field to the list of fields to resolve.
    //!
    //! 1. If the field is represented in preference file data, update the [`resolve_preference()`]
    //!    method following the same instructions as for [`resolve_policy()`].
    //! 1. If the field is represented in environment variables, update the
    //!    [`resolve_environment()`] method to resolve the field. Follow the established pattern in
    //!    the method.
    //! 1. If the field is represented in command line arguments, update the
    //!    [`resolve_command_line()`] method to resolve the field. Follow the established pattern
    //!    in the method.
    //!
    //! [`DscSettings`]: super::DscSettings
    //! [`resolve_policy()`]: super::DscSettings::resolve_policy
    //! [`resolve_preference()`]: super::DscSettings::resolve_preference
    //! [`resolve_environment()`]: super::DscSettings::resolve_environment
    //! [`resolve_command_line()`]: super::DscSettings::resolve_command_line
}

/// Represents the complete set of DSC settings, including all sources and the
/// resolved effective settings.
///
/// During initialization, DSC loads settings from various sources. The
/// following list defines the sources for settings in order of precedence,
/// from lowest to highest where later sources override earlier sources:
///
/// - The hardcoded defaults defined in the source code.
/// - The machine preference settings file, if it exists.
/// - The user preference settings file, if it exists.
/// - The workspace preference settings file, if it exists.
/// - The environment variables, if they are set.
/// - The command line arguments, if they are provided.
/// - The policy settings file, if it exists.
///
/// You can use the [`load()`] or [`try_load()`] methods to load settings from
/// all sources except for the command line, which must be manually provided
/// with the [`new_with_command_line()`] constructor.
///
/// You can access the resolved effective settings by calling the [`resolved()`]
/// method, which returns an instance of [`DscSettingsResolved`] containing the
/// final values for each setting after considering all sources.
///
/// For more information about DSC settings, refer to the
/// [module documentation](crate::settings).
///
/// [`load()`]: Self::load
/// [`try_load()`]: Self::try_load
/// [`new_with_command_line()`]: Self::new_with_command_line
/// [`resolved()`]: Self::resolved
#[allow(dead_code)]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, JsonSchema, DscRepoSchema)]
#[dsc_repo_schema(base_name = "all", folder_path = "settings")]
#[serde(rename_all = "camelCase")]
#[schemars(
    title = schema_i18n!("title"),
    description = schema_i18n!("description"),
    extend(
        "$id" = DscSettings::default_export_schema_id_uri(),
        "markdownDescription" = schema_i18n!("markdownDescription")
    )
)]
pub struct DscSettings {
    /// The hardcoded default settings defined in the source code.
    ///
    /// These defaults have the lowest precedence and will be overridden by any
    /// other settings source if a value is provided.
    ///
    /// The following snippet shows the effective code defaults as YAML data:
    ///
    /// ```yaml
    /// forbid_ignore_settings_file: false
    /// ignore_settings_file: false
    /// tracing:
    ///   level:  warn
    ///   format: default
    /// resource_path:
    ///   directories: []
    ///   append_env_path: true
    ///   restricted: false
    /// ```
    #[schemars(
        title = schema_i18n!("fields.default.title"),
        description = schema_i18n!("fields.default.description"),
        extend("markdownDescription" = schema_i18n!("fields.default.markdownDescription"))
    )]
    default: DscSettingsCodeDefaults,

    /// The settings loaded from the machine settings file, if it exists.
    ///
    /// This field is set to [`None`] if the machine settings file does not
    /// exist or if it couldn't be loaded.
    ///
    /// The machine settings file has the lowest precedence for settings after
    /// the code defaults. Any settings defined in a different scope will
    /// override the values defined in the machine settings file.
    ///
    /// The location for the machine settings file depends on the operating
    /// system:
    ///
    /// - On Windows: `%PROGRAMDATA%\dsc\dsc.settings.json`.
    /// - On macOS: `/Library/Application Support/dsc/dsc.settings.json`.
    /// - On Linux: `/etc/dsc/dsc.settings.json`.
    ///
    /// For more information about defining settings in a machine preference
    /// file, see [`PreferenceFileData`].
    #[schemars(
        title = schema_i18n!("fields.machine.title"),
        description = schema_i18n!("fields.machine.description"),
        extend("markdownDescription" = schema_i18n!("fields.machine.markdownDescription"))
    )]
    pub machine: Option<PreferenceFileData>,

    /// The settings loaded from the user settings file, if it exists.
    ///
    /// This field is set to [`None`] if the user settings file does not exist
    /// or if it couldn't be loaded.
    ///
    /// The user settings file has higher precedence than the machine settings
    /// file, but lower precedence than the workspace settings file,
    /// environment variables, and command line arguments. Any settings defined
    /// in those scopes will override the values defined in the user settings
    /// file.
    ///
    /// The location for the user settings file depends on the operating system
    /// and whether the `XDG_CONFIG_HOME` environment variable is set.
    ///
    /// - When the `XDG_CONFIG_HOME` environment variable is set, the user
    ///   settings file is located at `$XDG_CONFIG_HOME/dsc/dsc.settings.json`.
    /// - When the `XDG_CONFIG_HOME` environment variable is not set, the
    ///   location depends on the operating system:
    ///
    ///   - On Windows: `%APPDATA%\dsc\dsc.settings.json`.
    ///   - On macOS: `~/Library/Application Support/dsc/dsc.settings.json`.
    ///   - On Linux: `~/.config/dsc/dsc.settings.json`.
    ///
    /// For more information about defining settings in a user preference file,
    /// see [`PreferenceFileData`].
    #[schemars(
        title = schema_i18n!("fields.user.title"),
        description = schema_i18n!("fields.user.description"),
        extend("markdownDescription" = schema_i18n!("fields.user.markdownDescription"))
    )]
    pub user: Option<PreferenceFileData>,

    /// The settings loaded from the workspace settings file, if it exists.
    ///
    /// This field is set to [`None`] if the workspace settings file does not
    /// exist or if it couldn't be loaded.
    ///
    /// The workspace settings file has higher precedence than the machine and
    /// user settings files but lower precedence than environment variables and
    /// command line arguments. Any settings defined in those scopes will
    /// override the values defined in the workspace settings file.
    ///
    /// The location for the workspace settings file is always relative to the
    /// current working directory: `{CWD}/dsc.settings.json`.
    ///
    /// For more information about defining settings in a workspace preference
    /// file, see [`PreferenceFileData`].
    #[schemars(
        title = schema_i18n!("fields.workspace.title"),
        description = schema_i18n!("fields.workspace.description"),
        extend("markdownDescription" = schema_i18n!("fields.workspace.markdownDescription"))
    )]
    pub workspace: Option<PreferenceFileData>,

    /// The settings loaded from the environment variables, if they exist.
    ///
    /// This field is set to [`None`] if none of the relevant environment
    /// variables are set. DSC uses the following environment variables for
    /// settings:
    ///
    /// - `DSC_TRACE_LEVEL`: Defines the trace level to use.
    /// - `DSC_TRACE_FORMAT`: Defines the trace format to use.
    /// - `DSC_RESOURCE_PATH`: Defines the resource directories to use.
    /// - `DSC_RESTRICTED_PATH`: Defines the resource directories to use and
    ///   restricts all DSC invocations to those directories exclusively.
    /// - `DSC_IGNORE_SETTINGS_FILE`: Defines whether to ignore settings files.
    ///
    /// The environment variables have higher precedence than the machine,
    /// user, and workspace settings files but lower precedence than command
    /// line arguments. Any settings defined in the command line arguments will
    /// override the values defined in the environment variables.
    ///
    /// For more information about how environment variables affect DSC
    /// settings, see [`EnvironmentData`].
    #[schemars(
        title = schema_i18n!("fields.environment.title"),
        description = schema_i18n!("fields.environment.description"),
        extend("markdownDescription" = schema_i18n!("fields.environment.markdownDescription"))
    )]
    pub environment: Option<EnvironmentData>,

    /// The settings loaded from the command line arguments, if they were
    /// specified.
    ///
    /// This field is set to [`None`] if no command line arguments were
    /// provided relating to DSC settings. DSC uses the following command line
    /// arguments for settings:
    ///
    /// - `--trace-level`: Defines the trace level to use.
    /// - `--trace-format`: Defines the trace format to use.
    /// - `--ignore-settings-file`: Defines whether to ignore settings files.
    ///
    /// The command line arguments have the highest precedence for settings,
    /// overriding any values defined in the machine, user, and workspace
    /// settings files, as well as any values defined in the environment
    /// variables. Only policy settings have higher precedence than command
    /// line arguments, and they cannot be overridden.
    ///
    /// For more information about how command line arguments affect DSC
    /// settings, see [`CommandLineData`].
    #[serde(rename = "cli")]
    #[schemars(
        title = schema_i18n!("fields.command_line.title"),
        description = schema_i18n!("fields.command_line.description"),
        extend("markdownDescription" = schema_i18n!("fields.command_line.markdownDescription"))
    )]
    pub command_line: Option<CommandLineData>,

    /// The settings loaded from the policy settings file, if it exists.
    ///
    /// This field is set to [`None`] if the policy settings file doesn't exist
    /// or if it couldn't be loaded.
    ///
    /// The policy settings file has the highest precedence for settings,
    /// overriding any values defined in other sources, including command line
    /// arguments. Policy settings cannot be overridden.
    ///
    /// For more information about defining policy settings, see
    /// [`PolicyFileData`].
    #[schemars(
        title = schema_i18n!("fields.policy.title"),
        description = schema_i18n!("fields.policy.description"),
        extend("markdownDescription" = schema_i18n!("fields.policy.markdownDescription"))
    )]
    pub policy: Option<PolicyFileData>,

    /// The resolved settings after considering all sources and their precedence.
    ///
    /// This field is set to [`None`] if the resolved settings have not been
    /// computed yet. It represents the final effective settings that DSC will
    /// use during its operation.
    #[schemars(
        title = schema_i18n!("fields.resolved.title"),
        description = schema_i18n!("fields.resolved.description"),
        extend("markdownDescription" = schema_i18n!("fields.resolved.markdownDescription"))
    )]
    resolved: Option<DscSettingsResolved>,
}

/// Public API for interacting with [`DscSettings`]
impl DscSettings {
    /// Creates a new instance of `DscSettings` with all fields except `default`
    /// initialized to `None`.
    ///
    /// The `default` field is initialized with the hardcoded defaults defined
    /// in [`DSC_SETTINGS_CODE_DEFAULTS`].
    ///
    /// # Returns
    ///
    /// A new instance of `DscSettings` with only the default settings loaded.
    pub fn new() -> Self {
        Self {
            default: DSC_SETTINGS_CODE_DEFAULTS,
            machine: None,
            user: None,
            workspace: None,
            environment: None,
            command_line: None,
            policy: None,
            resolved: None,
        }
    }

    /// Creates a new instance of `DscSettings` and loads the provided command
    /// line data into it.
    ///
    /// The only fields populated in the returned instance are the default
    /// settings and [`command_line`]. All other fields are defined as `None`.
    ///
    /// # Arguments
    ///
    /// - `cli_data`: The command line data to load into the new instance.
    ///
    /// [`command_line`]: Self::command_line
    pub fn new_with_command_line(cli_data: CommandLineData) -> Self {
        let mut settings = Self::new();
        settings.command_line = Some(cli_data);

        settings
    }

    /// Returns the resolved settings from all loaded sources.
    ///
    /// This method acts as a getter for the resolved settings, lazily
    /// computing the resolution if it hasn't been computed yet.
    ///
    /// This method _only_ works from currently loaded settings. You must use
    /// the [`load()`] or [`try_load()`] methods to populate the settings from
    /// external sources before calling this method.
    ///
    /// # Returns
    ///
    /// Returns a reference to the resolved settings.
    ///
    /// # Example
    ///
    /// ```rust, ignore
    /// let mut settings = DscSettings::new();
    /// settings.load();
    /// let resolved = settings.resolved();
    /// ```
    ///
    /// [`load()`]: Self::load
    /// [`try_load()`]: Self::try_load
    pub fn resolved(&mut self) -> &DscSettingsResolved {
        if self.resolved.is_none() {
            self.resolve_all();
        }

        self.resolved.as_ref().unwrap()
    }

    /// Indicates whether the policy settings forbid ignoring settings files.
    ///
    /// # Returns
    ///
    /// This method returns `true` if the policy settings forbid ignoring
    /// settings files, and `false` otherwise. If the policy settings haven't
    /// been loaded, this method returns `false`.
    pub fn policy_forbids_ignoring_settings_files(&self) -> bool {
        self.policy.as_ref().is_some_and(|p| {
            p.forbid_ignore_settings_file.is_some_and(|v| v == true)
        })
    }

    /// Indicates whether settings files should be ignored based on the current
    /// settings sources.
    ///
    /// # Returns
    ///
    /// This return value for this method depends on the loaded policy,
    /// environment, and command line settings. The precedence rules are as
    /// follows:
    ///
    /// 1. If `policy.forbid_ignore_settings_file` is set to `true`, this
    ///    method always returns `false`, regardless of the other sources.
    /// 1. If `command_line.ignore_settings_file` is set, this method returns
    ///    its value.
    /// 1. If `environment.dsc_ignore_settings_file` is set, this method
    ///    returns its value.
    /// 1. If none of the above conditions are met, this method returns `false`.
    pub fn ignoring_settings_files(&self) -> bool {
        // If the policy forbids ignoring settings files, always return false.
        if self.policy_forbids_ignoring_settings_files() {
            return false;
        }
        if let Some(policy_ignore) = self.policy.as_ref()
            .and_then(|p| p.ignore_settings_file) {
                return *policy_ignore;
            }
        if let Some(cli_ignore) = self.command_line.as_ref()
            .and_then(|cli| cli.ignore_settings_file) {
                return *cli_ignore;
            }
        if let Some(env_ignore) = self.environment.as_ref()
            .and_then(|env| env.dsc_ignore_settings_file) {
                return *env_ignore;
            }

        false
    }

    /// Checks whether the policy folder has secure permissions.
    /// 
    /// DSC requires that the policy folder has secure permissions to prevent
    /// unauthorized access or modifications. If the folder doesn't have secure
    /// permissions, DSC won't load the policy file.
    /// 
    /// This function checks the permissions for the folder depending on the
    /// platform:
    /// 
    /// - On Windows:
    /// 
    ///   1. The folder must be owned by the Administrators group or the Local
    ///      System account.
    ///   1. The write permissions for the folder must be limited to the owner.
    ///   1. No inheritedd ACEs (Access Control Entries) should grant write
    ///      permissions to other users.
    /// 
    /// - On Linux and macOS:
    ///
    ///   1. The folder must be owned by the root user.
    ///   1. The folder must not have group or other write permissions.
    /// 
    /// # Returns
    /// 
    /// If the policy folder has secure permissions, this function returns
    /// `Ok(())`. If the folder doesn't have secure permissions, the function
    /// returns an `Err` with [`DscSettingsError::PolicyFolderNotSecure`].
    pub fn validate_policy_folder_security() -> Result<(), DscSettingsError> {
        let folder_path = POLICY_SETTINGS_FILE_PATH.as_path().parent()
            .expect("policy settings file always has a parent folder");
        
        // The folder not existing is fine, since we won't try to load the
        // policy file.
        if !folder_path.exists() {
            return Ok(());
        }
        
        if !Self::policy_folder_has_secure_permissions(folder_path) {
            #[cfg(target_os = "windows")]
            let required = t!(
                "settings.dsc_settings.policyFolderNotSecureWindows",
                path = folder_path.display()
            );
            #[cfg(not(target_os = "windows"))]
            let required = t!(
                "settings.dsc_settings.policyFolderNotSecureLinux",
                path = folder_path.display()
            );

            return Err(DscSettingsError::PolicyFolderNotSecure {
                path: folder_path.display().to_string(),
                required: required.to_string(),
            });
        }

        Ok(())
    }

    /// Resolves the effective settings by applying the precedence rules to all
    /// loaded sources.
    ///
    /// The resolution steps are:
    ///
    /// 1. Initialize with default settings from source code.
    /// 1. If policy file settings are loaded, apply them next, as they have
    ///    the highest precedence.
    /// 1. Apply settings file sources in order of precedence ([`Machine`],
    ///    then [`User`], then [`Workspace`]).
    /// 1. If environment settings are loaded, apply them next, overriding any
    ///    non-policy settings.
    /// 1. If command line settings are loaded, apply them last, overriding any
    ///    non-policy settings.
    ///
    /// Resolution is performed for every setting, _not_ by container. If the
    /// machine settings file defines both `tracing.level` and `tracing.format`,
    /// and the user settings file defines only `tracing.format`, the final
    /// resolved settings will have `tracing.level` from the machine settings
    /// file and `tracing.format` from the user settings file.
    ///
    /// Settings must be explicitly overridden by a higher-precedence source to
    /// be applied. DSC doesn't support effectively "undefining" a setting in
    /// a higher precedence source.
    ///
    /// [`Machine`]: DscSettingsScope::Machine
    /// [`User`]: DscSettingsScope::User
    /// [`Workspace`]: DscSettingsScope::Workspace
    /// [`Environment`]: DscSettingsScope::Environment
    /// [`Cli`]: DscSettingsScope::CommandLine
    /// [`Policy`]: DscSettingsScope::Policy
    pub fn resolve_all(&mut self) {
        debug!("{}", t!("settings.dsc_settings.debugResolving"));
        let mut resolving = DscSettingsResolved::default();

        macro_rules! emit_resolving_msg {
            ($scope:expr) => {
                debug!("{}", t!("settings.dsc_settings.debugResolvingScope", scope = $scope));
            };
        }
        macro_rules! emit_skipping_msg {
            ($scope:expr) => {
                debug!("{}", t!("settings.dsc_settings.debugSkippingScope", scope = $scope));
            };
        }
        macro_rules! emit_resolved_msgs {
            ($scope:expr) => {
                debug!("{}", t!("settings.dsc_settings.debugResolvedScope", scope = $scope));
                trace!("{}", t!(
                    "settings.dsc_settings.traceResolvedScope",
                    scope = $scope,
                    effective = serde_json::to_string(&resolving).unwrap()
                ));
            };
        }

        // First, resolve policy settings, as they have the highest precedence.
        if self.scope_is_loaded(DscSettingsScope::Policy) {
            emit_resolving_msg!("policy");
            self.resolve_policy(&mut resolving);
            emit_resolved_msgs!("policy");
        } else {
            emit_skipping_msg!("policy");
        }

        // Only resolve file-based settings if ignoring settings files is not forbidden by policy
        // and not specified in the policy, command line, or environment.
        if !self.ignoring_settings_files() {
            // Resolve file-based settings in order of precedence (Machine, User, Workspace).
            for source in DscSettingsScope::FOR_PREFERENCE_FILES.iter().cloned() {
                if self.scope_is_loaded(source) {
                    emit_resolving_msg!(source.to_string());
                    self.resolve_preference(source, &mut resolving);
                    emit_resolved_msgs!(source.to_string());
                } else {
                    emit_skipping_msg!(source.to_string());
                }
            }
        }
        // Resolve environment settings, which have higher precedence than file-based settings.
        if self.scope_is_loaded(DscSettingsScope::Environment) {
            emit_resolving_msg!("environment");
            self.resolve_environment(&mut resolving);
            emit_resolved_msgs!("environment");
        } else {
            emit_skipping_msg!("environment");
        }
        // Finally, resolve command line settings, which have the highest precedence (except for policy).
        if self.scope_is_loaded(DscSettingsScope::CommandLine) {
            emit_resolving_msg!("cli");
            self.resolve_command_line(&mut resolving);
            emit_resolved_msgs!("cli");
        } else {
            emit_skipping_msg!("cli");
        }

        debug!("{}", t!("settings.dsc_settings.debugResolved"));
        trace!("{}", t!(
            "settings.dsc_settings.traceResolved",
            effective = serde_json::to_string(&resolving).unwrap()
        ));
        self.resolved = Some(resolving);
    }

    /// Attempts to load settings from all non-CLI sources and converts loading
    /// errors into warnings.
    ///
    /// When loading a source raises an error, this method logs the error as a
    /// warning and continues loading the remaining sources. Only sources that
    /// are successfully loaded are stored in the instance. All other sources
    /// remain set to [`None`].
    pub fn load(&mut self) {
        // Always load policy first.
        if let Err(e) = self.load_policy() {
            warn!("{}", t!(
                "settings.dsc_settings.warnLoadScopeFailed",
                scope = "policy",
                error = e
            ));
        }
        // Load environment so we can check whether to load preferences.
        self.load_environment();

        // If ignoring settings files, emit debug and return early
        if self.ignoring_settings_files() {
            self.emit_debug_load_ignoring_settings_files();
            // Clear all loaded preference file data
            self.machine = None;
            self.user = None;
            self.workspace = None;
            return;
        }

        if let Err(e) = self.load_machine() {
            warn!("{}", t!(
                "settings.dsc_settings.warnLoadScopeFailed",
                scope = "machine",
                error = e
            ));
        }
        if let Err(e) = self.load_user() {
            warn!("{}", t!(
                "settings.dsc_settings.warnLoadScopeFailed",
                scope = "user",
                error = e
            ));
        }
        if let Err(e) = self.load_workspace() {
            warn!("{}", t!(
                "settings.dsc_settings.warnLoadScopeFailed",
                scope = "workspace",
                error = e
            ));
        }
    }

    /// Attempts to load settings from all non-CLI sources and collects any
    /// errors when loading each source.
    ///
    /// To load settings and ignore sources with errors, use the [`load()`]
    /// method instead.
    ///
    /// # Errors
    ///
    /// If loading any source raises an error, this method returns an instance
    /// of [`LoadMultipleErrors`] containing a vector of all errors encountered.
    ///
    /// [`load()`]: Self::load
    /// [`LoadMultipleErrors`]: DscSettingsError::LoadMultipleErrors
    pub fn try_load(&mut self) -> Result<(), DscSettingsError> {
        let mut errors : Vec<DscSettingsError> = Vec::new();
        if let Err(e) = self.load_policy() {
            errors.push(e);
        }

        // Load environment first to determine whether to load preferences
        if let Err(e) = self.try_load_environment() {
            errors.push(e);
        }

        if self.ignoring_settings_files() {
            // If ignoring settings files, we only emit debug information and
            // don't attempt to load preference files.
            self.emit_debug_load_ignoring_settings_files();
            // Clear all loaded preference file data
            self.machine = None;
            self.user = None;
            self.workspace = None;
        } else {
            if let Err(e) = self.load_machine() {
                errors.push(e);
            }
            if let Err(e) = self.load_user() {
                errors.push(e);
            }
            if let Err(e) = self.load_workspace() {
                errors.push(e);
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(DscSettingsError::LoadMultipleErrors{errors})
        }
    }
}

#[macro_use]
pub(super) mod macros {
    //! Defines convenience macros for resolving settings.

    /// Helper to define the `resolve_field` macro for preference file data
    /// with resolved hygeine.
    ///
    /// This macro defines the `resolve_field` macro with the given resolving
    /// context and file data, ensuring that the macro works correctly with
    /// macro hygiene.
    ///
    /// Without this helper, using the `resolve_field` macro would require
    /// passing the `resolving` and `policy` variables to every invocation.
    macro_rules! define_preference_resolver {
        ($resolving:ident, $file_data:ident, $source:ident, $dollar:tt) => {
            /// Helper for resolving fields for preference file data.
            ///
            /// This macro is a convenience for performing the following
            /// operations:
            ///
            /// 1. Checking whether a given container field is defined in the
            ///    data.
            /// 1. Checking each nested field within the container field to see
            ///    if it is defined.
            /// 1. If the nested field is defined, compare the scope of the
            ///    previously resolved field with the current source.
            /// 1. If the current source has higher precedence than the
            ///    previously resolved field, update the resolved field with
            ///    the new value and scope.
            ///
            /// # Syntax
            ///
            /// ## Leaf field
            ///
            /// ```rust, ignore
            /// resolve_field!(<leaf_field>);
            /// ```
            ///
            /// - `<leaf_field>` must be the name of a top-level field in the
            ///   file data.
            ///
            /// ## Container field
            ///
            /// ```rust, ignore
            /// resolve_field!(<container_field> => [<nested_field>, ...]);
            /// ```
            ///
            /// - `container_field` must be the name of a field in the file
            ///   data that contains the subfields listed in the brackets.
            /// - `nested_field` must be the name of a subfield within the
            ///   container field. Define every nested field that you want to
            ///   resolve.
            ///
            /// # Example
            ///
            /// The following macro invocation demonstrates how to resolve the
            /// `tracing` container field with its nested fields `level` and
            /// `format`:
            ///
            /// ```rust, ignore
            /// resolve_field!(tracing => [level, format]);
            /// ```
            ///
            /// It expands to the following code:
            ///
            /// ```rust, ignore
            /// if let Some(tracing) = &file_data.tracing {
            ///     if let Some(level) = tracing.level.as_ref() {
            ///         if resolving.tracing.level.scope < source {
            ///             resolving.tracing.level = DscSettingsResolvedField::new(
            ///                 level.clone(),
            ///                 source
            ///             );
            ///         }
            ///     }
            ///     if let Some(format) = tracing.format.as_ref() {
            ///         if resolving.tracing.format.scope < source {
            ///             resolving.tracing.format = DscSettingsResolvedField::new(
            ///                 format.clone(),
            ///                 source
            ///             );
            ///         }
            ///     }
            /// }
            /// ```
            ///
            /// # Note
            ///
            /// This macro doesn't currently support deeply resolving deeply
            /// nested fields.
            macro_rules! resolve_field {
                ($field:ident) => {
                    if let Some($field) = $file_data.$field.as_ref()
                        && $resolving.$field.scope < $source {
                            $resolving.$field = DscSettingsResolvedField::new(
                                $field.clone(),
                                $source
                            );
                        }
                };
                ($container_field:ident => [$dollar($field:ident),*]) => {
                    if let Some($container_field) = &$file_data.$container_field {
                        $dollar(
                            if let Some($field) = $container_field.$field.as_ref()
                                && $resolving.$container_field.$field.scope < $source {
                                    $resolving.$container_field.$field = DscSettingsResolvedField::new(
                                        $field.clone(),
                                        $source
                                    );
                                }
                        )*
                    }
                };
            }
        };
    }
    pub(super) use define_preference_resolver;

    /// Helper to define the `resolve_field` macro for policy file data with
    /// resolved hygiene.
    ///
    /// This macro defines the `resolve_field` macro with the given resolving
    /// context and file data, ensuring that the macro works correctly with
    /// macro hygiene.
    ///
    /// Without this helper, using the `resolve_field` macro would require
    /// passing the `resolving` and `policy` variables to every invocation.
    macro_rules! define_policy_resolver {
        ($resolving:ident, $file_data:ident, $dollar:tt) => {
            /// Helper for resolving policy fields.
            ///
            /// This macro is a convenience for performing the following
            /// operations:
            ///
            /// 1. Checking whether a given field is defined in the data.
            /// 1. If the field is a leaf field, override the previously
            ///    resolved value with the new value from policy.
            /// 1. If the field is a container field, check each nested field
            ///    within the container field to see if it is defined.
            /// 1. If the nested field is defined, update the resolved field
            ///    with the new value from policy.
            ///
            /// # Syntax
            ///
            /// ## Leaf field
            ///
            /// ```rust, ignore
            /// resolve_field!(<leaf_field>);
            /// ```
            ///
            /// - `<leaf_field>` must be the name of a top-level field in the
            ///   file data.
            ///
            /// ## Container field
            ///
            /// ```rust, ignore
            /// resolve_field!(<container_field> => [<nested_field>, ...]);
            /// ```
            ///
            /// - `container_field` must be the name of a field in the file
            ///   data that contains the subfields listed in the brackets.
            /// - `nested_field` must be the name of a subfield within the
            ///   container field. Define every nested field that you want to
            ///   resolve.
            ///
            /// # Example
            ///
            /// The following macro invocation demonstrates how to resolve the
            /// `tracing` container field with its nested fields `level` and
            /// `format`:
            ///
            /// ```rust, ignore
            /// resolve_field!(tracing => [level, format]);
            /// ```
            ///
            /// It expands to the following code:
            ///
            /// ```rust, ignore
            /// if let Some(tracing) = &file_data.tracing {
            ///     if let Some(level) = tracing.level.as_ref() {
            ///         resolving.tracing.level = DscSettingsResolvedField::new(
            ///             level.clone(),
            ///             source
            ///         );
            ///     }
            ///     if let Some(format) = tracing.format.as_ref() {
            ///         resolving.tracing.format = DscSettingsResolvedField::new(
            ///             format.clone(),
            ///             source
            ///         );
            ///     }
            /// }
            /// ```
            ///
            /// # Note
            ///
            /// This macro doesn't currently support deeply resolving deeply
            /// nested fields.
            macro_rules! resolve_field {
                ($field:ident) => {
                    if let Some($field) = $file_data.$field.as_ref() {
                        $resolving.$field = DscSettingsResolvedField::for_policy($field.clone());
                    }
                };
                ($container_field:ident => [$dollar($field:ident),+]) => {
                    if let Some($container_field) = &$file_data.$container_field {
                        $dollar(
                            if let Some($field) = $container_field.$field.as_ref() {
                                $resolving.$container_field.$field = DscSettingsResolvedField::for_policy(
                                    $field.clone()
                                );
                            }
                        )+
                    }
                };
            }
        };
    }
    pub(super) use define_policy_resolver;
}

/// Private API for [`DscSettings`]
impl DscSettings {
    /// Indicates whether a setting scope has been loaded with any data.
    ///
    /// After loading settings from all sources, it may be the case that one
    /// or more scopes loaded without errors but no fields in the data for that
    /// scope were defined.
    ///
    /// This method provides a short way to check if any data has been loaded
    /// for a given scope before invoking the resolution logic.
    ///
    /// # Arguments
    ///
    /// * `scope` - The setting scope to check for loaded data.
    ///
    /// # Returns
    ///
    /// `true` if the specified setting scope has been loaded with any data,
    /// otherwise `false`.
    pub(crate) fn scope_is_loaded(&self, scope: DscSettingsScope) -> bool {
        match scope {
            DscSettingsScope::Default => true,
            DscSettingsScope::Machine => self.machine.as_ref().is_some_and(|data| data.any_defined()),
            DscSettingsScope::User => self.user.as_ref().is_some_and(|data| data.any_defined()),
            DscSettingsScope::Workspace => self.workspace.as_ref().is_some_and(|data| data.any_defined()),
            DscSettingsScope::Environment => self.environment.as_ref().is_some_and(|data| data.any_defined()),
            DscSettingsScope::CommandLine => self.command_line.as_ref().is_some_and(|data| data.any_defined()),
            DscSettingsScope::Policy => self.policy.as_ref().is_some_and(|data| data.any_defined()),
        }
    }

    /// Emits a debug message when skipping loading preference file settings.
    ///
    /// This convenience method is only intended for use in the [`load()`] and
    /// [`try_load()`] methods.
    /// 
    /// [`load()`]: Self::load
    /// [`try_load()`]: Self::try_load
    fn emit_debug_load_ignoring_settings_files(&self) {
        macro_rules! check_ignore {
            ($source:ident, $field:ident) => {
                self.$source.as_ref()
                    .is_some_and(|d| d.$field.is_some_and(|v| v == true))
            };
        }

        let scope = if check_ignore!(policy, ignore_settings_file) {
            DscSettingsScope::Policy
        } else if check_ignore!(environment, dsc_ignore_settings_file) {
            DscSettingsScope::Environment
        } else {
            DscSettingsScope::CommandLine
        };
        let setting = match scope {
            DscSettingsScope::Policy => "`ignoreSettingsFile`",
            DscSettingsScope::Environment => "`DSC_IGNORE_SETTINGS_FILE`",
            DscSettingsScope::CommandLine => "`--ignore-settings-file`",
            _ => unreachable!(),
        };
        debug!("{}", t!(
            "settings.dsc_settings.debugLoadIgnoringSettingsFiles",
            skipping_scopes = "`machine`, `user`, `workspace`",
            scope = scope,
            setting = setting
        ));
    }

    /// Loads the environment variables into the instance, ignoring invalidly
    /// defined variables.
    ///
    /// This method loads the environment variables into the instance if any
    /// are validly defined. When trying to load an environment variable with
    /// an invalid value, this method emits warning messages.
    ///
    /// To capture errors when loading the environment instead of ignoring them,
    /// use the [`try_load_environment()`] method instead.
    ///
    /// [`try_load_environment()`]: Self::try_load_environment
    fn load_environment(&mut self) {
        debug!("{}", t!("settings.dsc_settings.debugLoadEnvStart"));
        let data = EnvironmentData::from_env();
        if data.any_defined() {
            debug!("{}", t!("settings.dsc_settings.debugLoadEnvSuccess"));
            self.environment = Some(data);
        } else {
            debug!("{}", t!("settings.dsc_settings.debugLoadEnvNoneDefined"));
        }
    }

    /// Attempts to load the environment settings data into the instance.
    ///
    /// This method loads the environment variables into the instance if every
    /// defined variable has a valid value. This method collects the errors
    /// for invalidly defined variables and returns them together for a more
    /// comprehensive error report.
    ///
    /// To raise warnings for invalid variables when loading the environment
    /// instead of raising an error, use the [`load_environment()`] method
    /// instead.
    ///
    /// # Errors
    ///
    /// If the environment settings cannot be loaded from the environment
    /// variables, this method raises the [`LoadEnvironmentMultipleErrors`]
    /// error, which contains every error for invalidly defined environment
    /// variables.
    ///
    /// Undefined environment variables don't raise an error.
    ///
    /// [`LoadEnvironmentMultipleErrors`]: DscSettingsError::LoadEnvironmentMultipleErrors
    /// [`load_environment()`]: Self::load_environment
    fn try_load_environment(&mut self) -> Result<(), DscSettingsError> {
        debug!("{}", t!("settings.dsc_settings.debugLoadEnvStart"));
        let data = EnvironmentData::try_from_env()?;
        if data.any_defined() {
            debug!("{}", t!("settings.dsc_settings.debugLoadEnvSuccess"));
            self.environment = Some(data);
        } else {
            debug!("{}", t!("settings.dsc_settings.debugLoadEnvNoneDefined"));
        }
        Ok(())
    }

    /// Attempts to load the machine settings file into the instance
    ///
    /// If the machine settings file doesn't exist, this method does nothing
    /// and returns `Ok(())`. If the file exists and is validly defined, this
    /// method loads the settings into the instance.
    ///
    /// # Errors
    ///
    /// If the file exists with invalid data, this method returns an error
    /// indicating the failure to load the machine settings file.
    fn load_machine(&mut self) -> Result<(), DscSettingsError> {
        let machine_settings_path = MACHINE_SETTINGS_FILE_PATH.as_path();
        debug!("{}", t!(
            "settings.dsc_settings.debugLoadPreferenceStart",
            scope = "machine",
            path = machine_settings_path.to_string_lossy()
        ));
        if machine_settings_path.exists() {
            let data = PreferenceFileData::from_file(machine_settings_path)?;
            debug!("{}", t!(
                "settings.dsc_settings.debugLoadPreferenceSuccess",
                scope = "machine",
                path = machine_settings_path.to_string_lossy()
            ));
            self.machine = Some(data);
        } else {
            debug!("{}", t!(
                "settings.dsc_settings.debugLoadPreferenceFileNotFound",
                scope = "machine",
                path = machine_settings_path.to_string_lossy()
            ));
        }

        Ok(())
    }

    /// Attempts to load the user settings file into the instance
    ///
    /// If the user settings file doesn't exist, this method does nothing and
    /// returns `Ok(())`. If the file exists and is validly defined, this
    /// method loads the settings into the instance.
    ///
    /// # Errors
    ///
    /// If the file exists with invalid data, this method returns an error
    /// indicating the failure to load the user settings file.
    fn load_user(&mut self) -> Result<(), DscSettingsError> {
        let user_settings_path = match USER_SETTINGS_FILE_PATH.as_ref() {
            Some(path) => path.as_path(),
            None => {
                debug!("{}", t!("settings.dsc_settings.debugLoadUserForNonUserAccount"));
                return Ok(());
            }
        };
        debug!("{}", t!(
            "settings.dsc_settings.debugLoadPreferenceStart",
            scope = "user",
            path = user_settings_path.to_string_lossy()
        ));
        if user_settings_path.exists() {
            let data = PreferenceFileData::from_file(user_settings_path)?;
            debug!("{}", t!(
                "settings.dsc_settings.debugLoadPreferenceSuccess",
                scope = "user",
                path = user_settings_path.to_string_lossy()
            ));
            self.user = Some(data);
        } else {
            debug!("{}", t!(
                "settings.dsc_settings.debugLoadPreferenceFileNotFound",
                scope = "user",
                path = user_settings_path.to_string_lossy()
            ));
        }

        Ok(())
    }

    /// Attempts to load the workspace settings file into the instance
    ///
    /// If the workspace settings file doesn't exist, this method does nothing
    /// and returns `Ok(())`. If the file exists and is validly defined, this
    /// method loads the settings into the instance.
    ///
    /// # Errors
    ///
    /// If the file exists with invalid data, this method returns an error
    /// indicating the failure to load the workspace settings file.
    fn load_workspace(&mut self) -> Result<(), DscSettingsError> {
        let workspace_settings_path = WORKSPACE_SETTINGS_FILE_PATH.as_path();
        debug!("{}", t!(
            "settings.dsc_settings.debugLoadPreferenceStart",
            scope = "workspace",
            path = workspace_settings_path.to_string_lossy()
        ));
        if workspace_settings_path.exists() {
            let data = PreferenceFileData::from_file(workspace_settings_path)?;
            debug!("{}", t!(
                "settings.dsc_settings.debugLoadPreferenceSuccess",
                scope = "workspace",
                path = workspace_settings_path.to_string_lossy()
            ));
            self.workspace = Some(data);
        } else {
            debug!("{}", t!(
                "settings.dsc_settings.debugLoadPreferenceFileNotFound",
                scope = "workspace",
                path = workspace_settings_path.to_string_lossy()
            ));
        }

        Ok(())
    }

    /// Attempts to load the policy settings file into the instance
    ///
    /// If the policy settings file doesn't exist, this method does nothing and
    /// returns `Ok(())`. If the file exists and is validly defined, this
    /// method loads the settings into the instance.
    ///
    /// # Errors
    ///
    /// If the file exists with invalid data, this method returns an error
    /// indicating the failure to load the policy settings file.
    fn load_policy(&mut self) -> Result<(), DscSettingsError> {
        let policy_settings_path = POLICY_SETTINGS_FILE_PATH.as_path();

        debug!("{}", t!(
            "settings.dsc_settings.debugLoadPolicyStart",
            path = policy_settings_path.to_string_lossy()
        ));
        if policy_settings_path.exists() {
            Self::validate_policy_folder_security()?;

            let data = PolicyFileData::from_file(policy_settings_path)?;
            debug!("{}", t!(
                "settings.dsc_settings.debugLoadPolicySuccess",
                path = policy_settings_path.to_string_lossy()
            ));
            self.policy = Some(data);
        } else {
            debug!("{}", t!(
                "settings.dsc_settings.debugLoadPolicyFileNotFound",
                path = policy_settings_path.to_string_lossy()
            ));
        }

        Ok(())
    }

    /// Applies the settings from the [`CommandLine`] scope to the resolved
    /// settings.
    ///
    /// If the command line settings aren't loaded, this method returns
    /// immediately without modifying the resolved settings. If the command
    /// line settings are loaded, this method applies them to the resolved
    /// settings, overriding any values from lower-precedence sources.
    ///
    /// # Arguments
    ///
    /// - `resolving` - A mutable reference to the instance of
    ///   [`DscSettingsResolved`] that represents the intermediate state of the
    ///   resolved settings.
    ///
    /// [`CommandLine`]: DscSettingsScope::CommandLine
    fn resolve_command_line(&mut self, resolving: &mut DscSettingsResolved) {
        let Some(cli_data) = self.command_line.as_ref() else {
            return;
        };
        let scope = DscSettingsScope::CommandLine;

        if let Some(level) = cli_data.trace_level.as_ref()
            && resolving.tracing.level.scope < scope {
                resolving.tracing.level = DscSettingsResolvedField::for_command_line(level.clone());
            }

        if let Some(format) = cli_data.trace_format.as_ref()
            && resolving.tracing.format.scope < scope {
                resolving.tracing.format = DscSettingsResolvedField::for_command_line(format.clone());
            }

        if let Some(ignore_settings_file) = cli_data.ignore_settings_file.as_ref() {
            // check if this option is forbidden by policy
            if self.policy_forbids_ignoring_settings_files() && **ignore_settings_file {
                warn!("{}", t!(
                    "settings.dsc_settings.warnCliIgnoreSetting",
                    cli_arg = "--ignore-settings-file",
                    policy_field = "forbid_ignore_settings_file"
                ));
            } else {
                if resolving.ignore_settings_file.scope < scope {
                    resolving.ignore_settings_file = DscSettingsResolvedField::for_command_line(*ignore_settings_file);
                }
            }
        }
    }

    /// Applies the settings from the [`Environment`] scope to the resolved
    /// settings.
    ///
    /// If the environment settings aren't loaded, this method returns
    /// immediately without modifying the resolved settings. If the environment
    /// settings are loaded, this method applies them to the resolved settings,
    /// overriding any values from lower-precedence sources.
    ///
    /// # Arguments
    ///
    /// - `resolving` - A mutable reference to the instance of
    ///   [`DscSettingsResolved`] that represents the intermediate state of the
    ///   resolved settings.
    ///
    /// [`Environment`]: DscSettingsScope::Environment
    fn resolve_environment(&mut self, resolving: &mut DscSettingsResolved) {
        let Some(env_data) = self.environment.as_ref() else {
            return;
        };
        let scope = DscSettingsScope::Environment;

        if let Some(level) = env_data.dsc_trace_level.as_ref()
            && resolving.tracing.level.scope < scope {
                resolving.tracing.level = DscSettingsResolvedField::for_environment(level.clone());
            }

        if let Some(format) = env_data.dsc_trace_format.as_ref()
            && resolving.tracing.format.scope < scope {
                resolving.tracing.format = DscSettingsResolvedField::for_environment(format.clone());
            }


        if let Some(restricted_path) = env_data.dsc_restricted_path.as_ref() {
            if resolving.resource_path.restricted.scope < scope && resolving.resource_path.directories.scope < scope {
                let directories = restricted_path.clone();
                resolving.resource_path.directories = DscSettingsResolvedField::for_environment(directories);
                resolving.resource_path.restricted = DscSettingsResolvedField::for_environment(true);
            }
        } else if let Some(resource_path) = env_data.dsc_resource_path.as_ref()
            && resolving.resource_path.directories.scope < scope {
                let directories = resource_path.clone();
                resolving.resource_path.directories = DscSettingsResolvedField::for_environment(directories);
        }

        if let Some(ignore_settings_file) = env_data.dsc_ignore_settings_file.as_ref() {
            // Only override the ignore_settings_file setting if it's not forbidden by policy
            if self.policy_forbids_ignoring_settings_files() && **ignore_settings_file {
                warn!("{}", t!(
                    "settings.dsc_settings.warnEnvIgnoreSetting",
                    env_var = "DSC_IGNORE_SETTINGS_FILE",
                    policy_field = "forbid_ignore_settings_file"
                ));
            } else {
                if resolving.ignore_settings_file.scope < scope {
                    resolving.ignore_settings_file = DscSettingsResolvedField::for_environment(*ignore_settings_file);
                }
            }
        }
    }

    /// Resolves the settings from preference file sources and applies them to
    /// the resolved settings.
    ///
    /// # Arguments
    ///
    /// - `source` - The source scope from which to resolve the settings. This
    ///   scope must be one of the file-based sources: [`Machine`], [`User`],
    ///   or [`Workspace`]. Specifying any other scope returns early from the
    ///   method without modifying the resolved settings or raising any errors.
    /// - `resolving` - A mutable reference to the instance of
    ///   [`DscSettingsResolved`] that represents the intermediate state of the
    ///   resolved settings.
    ///
    /// [`Machine`]: DscSettingsScope::Machine
    /// [`User`]: DscSettingsScope::User
    /// [`Workspace`]: DscSettingsScope::Workspace
    fn resolve_preference(&mut self,  source: DscSettingsScope, resolving: &mut DscSettingsResolved) {
        let file_data = match source {
            DscSettingsScope::Machine => self.machine.as_ref(),
            DscSettingsScope::User => self.user.as_ref(),
            DscSettingsScope::Workspace => self.workspace.as_ref(),
            _ => None,
        };
        let Some(file_data) = file_data else {
            return;
        };

        // Define the `resolve_field!` macro for the current preference file
        // and resolving context. This is required to work around macro hygiene
        // issues.
        macros::define_preference_resolver!(resolving, file_data, source, $);

        resolve_field!(tracing => [level, format]);
        resolve_field!(resource_path => [append_env_path, directories, restricted]);
    }

    /// Resolves the policy settings and applies them to the resolved settings.
    ///
    /// If the [`Policy`] settings aren't loaded, this method returns
    /// immediately without modifying the resolved settings. If the policy
    /// settings are loaded, this method applies them to the resolved settings,
    /// overriding any values from lower-precedence sources.
    ///
    /// # Arguments
    ///
    /// - `resolving` - A mutable reference to the instance of
    ///   [`DscSettingsResolved`] that represents the intermediate state of the
    ///   resolved settings.
    ///
    /// [`Policy`]: DscSettingsScope::Policy
    pub(in crate::settings) fn resolve_policy(&mut self, resolving: &mut DscSettingsResolved) {
        let Some(policy) = self.policy.as_ref() else {
            return;
        };
        // Define the policy resolver macro for the current policy and resolving
        // context. This is required to work around macro hygiene issues.
        macros::define_policy_resolver!(resolving, policy, $);

        // Resolve the `forbid_ignore_settings_file` setting first, since it
        // affects resolution of `ignore_settings_file`.
        resolve_field!(forbid_ignore_settings_file);

        // Need to handle `ignore_settings_file` manually because it has a
        // dependency on `forbid_ignore_settings_file` and some complex
        // interactions that can't be generically modeled with the macro.
        if let Some(ignore) = policy.ignore_settings_file {
            if let Some(forbid_ignore) = policy.forbid_ignore_settings_file {
                // If the settings are incompatible, prefer forbidding and
                // don't ignore the settings files.
                if *forbid_ignore && *ignore{
                    warn!("{}", t!(
                        "settings.dsc_settings.warnPolicyIgnoreSetting",
                        ignore_settings_file = "ignore_settings_file",
                        forbid_ignore_settings_file = "forbid_ignore_settings_file"
                    ));
                } else {
                    resolving.ignore_settings_file = DscSettingsResolvedField::for_policy(ignore);
                }
            } else {
                resolving.ignore_settings_file = DscSettingsResolvedField::for_policy(ignore);
            }
        }

        // Resolve remaining fields as normal.
        resolve_field!(tracing => [level, format]);
        resolve_field!(resource_path => [append_env_path, directories, restricted]);
    }

    /// Indicates whether the folder containing the DSC policy settings file has
    /// secure permissions.
    /// 
    /// This function verifies the following ACL rules:
    /// 
    /// 1. The folder is owned by the Administrators group or the Local System
    ///    account.
    /// 1. Only the owner has write permissions.
    /// 1. No inherited ACEs grant write permissions to other users.
    /// 
    /// # Arguments
    /// 
    /// - `folder_path`: The path to the folder containing the DSC policy
    ///   settings file.
    ///
    /// # Returns
    ///
    /// `true` if the folder has secure permissions according to the rules and
    /// `false` otherwise.
    #[cfg(all(target_os = "windows", not(test)))]
    fn policy_folder_has_secure_permissions(folder_path: &Path) -> bool {
        use windows::Win32::Security::{
            Authorization::{GetNamedSecurityInfoW, SE_FILE_OBJECT},
            IsWellKnownSid, GetAce,
            WinBuiltinAdministratorsSid, WinLocalSystemSid,
            ACL, ACCESS_ALLOWED_ACE, PSECURITY_DESCRIPTOR,
        };
        use windows::Win32::Foundation::{LocalFree, HLOCAL};
        use windows::core::PCWSTR;
        use std::ptr;

        const FILE_WRITE_DATA: u32 = 0x0002;
        const FILE_APPEND_DATA: u32 = 0x0004;
        const WRITE_DAC: u32 = 0x0004_0000;
        const WRITE_OWNER: u32 = 0x0008_0000;
        const GENERIC_WRITE: u32 = 0x4000_0000;
        const GENERIC_ALL: u32 = 0x1000_0000;
        const WRITE_MASK: u32 = FILE_WRITE_DATA | FILE_APPEND_DATA | WRITE_DAC | WRITE_OWNER | GENERIC_WRITE | GENERIC_ALL;

        const INHERIT_ONLY_ACE: u8 = 0x08;

        let folder_wide: Vec<u16> = folder_path
            .to_string_lossy()
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect();

        let mut p_sd: PSECURITY_DESCRIPTOR = PSECURITY_DESCRIPTOR(ptr::null_mut());
        let mut p_dacl: *mut ACL = ptr::null_mut();

        let result = unsafe {
            GetNamedSecurityInfoW(
                PCWSTR(folder_wide.as_ptr()),
                SE_FILE_OBJECT,
                windows::Win32::Security::DACL_SECURITY_INFORMATION,
                None,
                None,
                Some(&mut p_dacl),
                None,
                &mut p_sd,
            )
        };

        // Fail closed: if we can't read the security descriptor, treat as insecure
        if result.is_err() {
            return false;
        }

        // A NULL DACL means full access to everyone
        if p_dacl.is_null() {
            unsafe { let _ = LocalFree(Some(HLOCAL(p_sd.0.cast()))); }
            return false;
        }

        let ace_count = unsafe { (*p_dacl).AceCount };

        for i in 0..ace_count {
            let mut ace_ptr: *mut core::ffi::c_void = ptr::null_mut();
            let ok = unsafe { GetAce(p_dacl, i as u32, &mut ace_ptr) };
            if ok.is_err() {
                // Fail closed: if we can't read an ACE, treat as insecure
                unsafe { let _ = LocalFree(Some(HLOCAL(p_sd.0.cast()))); }
                return false;
            }

            let header = unsafe { &*(ace_ptr as *const windows::Win32::Security::ACE_HEADER) };

            // Skip inherit-only ACEs as they don't apply to this folder
            if (header.AceFlags & INHERIT_ONLY_ACE) != 0 {
                continue;
            }

            // Skip deny ACEs (types 1, 6, 10) - they restrict access, not grant it
            if header.AceType == 1 || header.AceType == 6 || header.AceType == 10 {
                continue;
            }

            // For ACCESS_ALLOWED_ACE_TYPE (0), check the SID
            if header.AceType == 0 {
                let ace = unsafe { &*(ace_ptr as *const ACCESS_ALLOWED_ACE) };
                if (ace.Mask & WRITE_MASK) == 0 {
                    continue;
                }

                let sid_ptr = &ace.SidStart as *const u32 as *const core::ffi::c_void;
                let psid = windows::Win32::Security::PSID(sid_ptr as *mut core::ffi::c_void);

                let is_system = unsafe { IsWellKnownSid(psid, WinLocalSystemSid).as_bool() };
                let is_admin = unsafe { IsWellKnownSid(psid, WinBuiltinAdministratorsSid).as_bool() };

                if !is_system && !is_admin {
                    unsafe { let _ = LocalFree(Some(HLOCAL(p_sd.0.cast()))); }
                    return false;
                }
                continue;
            }

            // For any other allow-type ACE (callback, object, etc.) we cannot
            // reliably extract the SID, so fail closed to be safe
            let ace = unsafe { &*(ace_ptr as *const ACCESS_ALLOWED_ACE) };
            if (ace.Mask & WRITE_MASK) != 0 {
                unsafe { let _ = LocalFree(Some(HLOCAL(p_sd.0.cast()))); }
                return false;
            }
        }

        unsafe { let _ = LocalFree(Some(HLOCAL(p_sd.0.cast()))); }
        true
    }

    /// Indicates whether the folder containing the DSC policy settings file has
    /// secure permissions.
    /// 
    /// This function verifies the following permissions per Unix conventions:
    /// 
    /// - The folder must be owned by the root user.
    /// - The folder must not have group or other write permissions.
    /// 
    /// # Arguments
    ///
    /// - `folder_path`: The path to the folder containing the DSC policy
    ///   settings file.
    /// 
    /// # Returns
    ///
    /// `true` if the folder has secure permissions according to the rules and
    /// `false` otherwise.
    #[cfg(all(not(target_os = "windows"), not(test)))]
    fn policy_folder_has_secure_permissions(folder_path: &Path) -> bool {
        use std::os::unix::fs::MetadataExt;

        // Fail closed: if we can't read metadata, treat as insecure
        let Ok(metadata) = std::fs::metadata(folder_path) else {
            return false;
        };

        let mode = metadata.mode();
        let uid = metadata.uid();

        // Verify owner is root and no group or other write bits are set
        uid == 0 && (mode & 0o020) == 0 && (mode & 0o002) == 0
    }

    /// Indicates whether the folder containing the DSC policy settings file has
    /// secure permissions.
    #[cfg(test)]
    fn policy_folder_has_secure_permissions(_folder_path: &Path) -> bool {
        // In tests, we can assume the folder has secure permissions
        match TEST_POLICY_FOLDER_IS_SECURE.get() {
                Some(mutex) => {
                    *mutex.lock().unwrap()
                },
                None => {
                    panic!("TEST_POLICY_FOLDER_IS_SECURE has not been initialized");
                }
            }
    }
}

/// Helper static for stubbed unit tests to mock the security status of the
/// policy folder.
#[cfg(test)]
pub(crate) static TEST_POLICY_FOLDER_IS_SECURE: std::sync::OnceLock<std::sync::Mutex<bool>> = std::sync::OnceLock::new();

/// Helper macro for stubbed unit tests to mock the security status of the
/// policy folder.
/// 
/// This macro sets the result for the [`policy_folder_has_secure_permissions`]
/// function.
/// 
/// # Arguments
/// 
/// - `$value` (optional): The boolean value indicating whether the policy
///   folder has secure permissions. If omitted, it defaults to `true`.
/// 
/// # Examples
/// 
/// Invoking the macro without any arguments sets the result to true:
/// 
/// ```rust, ignore
/// set_test_policy_folder_is_secure!();
/// assert!(DscSettings::validate_policy_folder_security().is_ok());
/// ```
/// 
/// You can also provide a boolean value to set the security status explicitly:
/// 
/// ```rust, ignore
/// set_test_policy_folder_is_secure!(false);
/// assert!(DscSettings::validate_policy_folder_security().is_err());
/// set_test_policy_folder_is_secure!(true);
/// assert!(DscSettings::validate_policy_folder_security().is_ok());
/// ```
#[cfg(test)]
macro_rules! set_test_policy_folder_is_secure {
    () => {
        $crate::settings::dsc_settings::set_test_policy_folder_is_secure!(true);
    };
    ($value:literal) => {
        {
            match $crate::settings::TEST_POLICY_FOLDER_IS_SECURE.get() {
                Some(mutex) => {
                    let mut guard = mutex.lock().unwrap();
                    *guard = $value;
                }
                None => {
                    $crate::settings::TEST_POLICY_FOLDER_IS_SECURE.get_or_init(|| std::sync::Mutex::new($value));
                }
            }
        }
    };
}
#[cfg(test)]
pub(crate) use set_test_policy_folder_is_secure;

impl Default for DscSettings {
    fn default() -> Self {
        Self::new()
    }
}