// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

pub(super) mod fixtures {
    //! Defines test fixtures for the integration tests.

    use dsc_lib::settings::sources::*;
    use dsc_lib::settings::fields::*;
    use std::path::PathBuf;
    use std::sync::LazyLock;

    use super::super::builders::*;

    /// Defines policy data with the following settings:
    ///
    /// - `forbid_ignore_settings_file`: `true`
    /// - `resource_path.append_env_path`: `false`
    /// - `resource_path.directories`: `["/etc/dsc"]`
    /// - `resource_path.restrict_path`: `true`
    ///
    /// It doesn't define any values for `tracing`.
    pub(super) static POLICY_DATA: LazyLock<PolicyFileData> = LazyLock::new(|| {
        PolicyFileDataBuilder::new()
            .with_forbid_ignore_settings_file(true)
            .with_resource_path(
                ResourcePathFileDataBuilder::new()
                .with_append_env_path(false)
                .with_directories(vec!["/etc/dsc"])
                .with_restricted(true)
                .build()
            )
            .build()
    });

    /// Defines machine data with the following settings:
    ///
    /// - `tracing.level`: `warn`
    /// - `resource_path.append_env_path`: `true`
    /// - `resource_path.directories`: `["/usr/local/bin"]`
    pub(super) static MACHINE_DATA: LazyLock<PreferenceFileData> = LazyLock::new(|| {
        PreferenceFileDataBuilder::new()
            .with_tracing(
                TracingFileDataBuilder::new()
                .with_level(TracingLevelField::Warn)
                .build()
            )
            .with_resource_path(
                ResourcePathFileDataBuilder::new()
                .with_append_env_path(true)
                .with_directories(vec!["/usr/local/bin"])
                .build()
            )
            .build()
    });

    /// Defines workspace data with the following settings:
    ///
    /// - `tracing.level`: `warn`
    /// - `resource_path.directories`: `["~/infra/dsc/resources", "~/infra/dsc/extensions"]`
    pub(super) static WORKSPACE_DATA: LazyLock<PreferenceFileData> = LazyLock::new(|| {
        PreferenceFileDataBuilder::new()
            .with_tracing(
                TracingFileDataBuilder::new()
                .with_level(TracingLevelField::Warn)
                .build()
            )
            .with_resource_path(
                ResourcePathFileDataBuilder::new()
                .with_directories(vec![
                    "~/infra/dsc/resources",
                    "~/infra/dsc/extensions"
                ])
                .build()
            )
            .build()
    });

    /// Defines user data with the following settings:
    ///
    /// - `tracing.level`: `debug`
    pub(super) static USER_DATA: LazyLock<PreferenceFileData> = LazyLock::new(|| {
        PreferenceFileDataBuilder::new()
            .with_tracing(
                TracingFileDataBuilder::new()
                .with_level(TracingLevelField::Debug)
                .build()
            )
            .build()
    });

    /// Defines environment data with the following settings:
    ///
    /// - `dsc_resource_path`: `["/usr/bin"]`
    /// - `dsc_ignore_settings_file`: `true`
    /// - `dsc_trace_level`: `info`
    pub(super) static ENVIRONMENT_DATA: LazyLock<EnvironmentData> = LazyLock::new(|| {
        EnvironmentDataBuilder::new()
            .with_resource_path(vec![PathBuf::from("/usr/bin")])
            .with_ignore_settings_file(true)
            .with_trace_level(TracingLevelField::Info)
            .build()
    });

    pub(super) static COMMAND_LINE_DATA: LazyLock<CommandLineData> = LazyLock::new(|| {
        CommandLineDataBuilder::new()
            .with_ignore_settings_file(false)
            .with_trace_level(TracingLevelField::Debug)
            .build()
    });
}

mod methods {
    use super::fixtures::*;
    use super::super::builders::*;
    use dsc_lib::settings::{DscSettings, DscSettingsScope, fields::*};

    #[test] fn new() {
        let actual = DscSettings::new();
        pretty_assertions::assert_eq!(actual.policy, None);
        pretty_assertions::assert_eq!(actual.machine, None);
        pretty_assertions::assert_eq!(actual.user, None);
        pretty_assertions::assert_eq!(actual.environment, None);
        pretty_assertions::assert_eq!(actual.command_line, None);
    }

    #[test] fn new_with_command_line() {
        let cli_data = COMMAND_LINE_DATA.clone();
        let actual = DscSettingsBuilder::new()
            .with_command_line(cli_data)
            .build();

        pretty_assertions::assert_eq!(actual.machine, None);
        pretty_assertions::assert_eq!(actual.user, None);
        pretty_assertions::assert_eq!(actual.environment, None);
        pretty_assertions::assert_eq!(actual.command_line, Some(COMMAND_LINE_DATA.clone()));
        pretty_assertions::assert_eq!(actual.policy, None);
    }
    mod resolved {
        use super::*;
        #[test] fn with_code_defaults_only() {
            let settings = &mut DscSettingsBuilder::new().build();
            let expected = DscSettingsResolvedBuilder::new().build();
            let actual = settings.resolved().clone();
            pretty_assertions::assert_eq!(actual, expected)
        }

        #[test] fn machine_overrides_code_defaults() {
            let settings = &mut DscSettingsBuilder::new()
                .with_machine(MACHINE_DATA.clone())
                .build();
            let expected = DscSettingsResolvedBuilder::new()
                .with_resource_path(
                    ResourcePathResolvedSettingsBuilder::new()
                        .with_append_env_path(true, DscSettingsScope::Machine)
                        .with_directories(vec!["/usr/local/bin"], DscSettingsScope::Machine)
                        .build()
                )
                .with_tracing(
                    TracingResolvedSettingsBuilder::new()
                        .with_level(TracingLevelField::Warn, DscSettingsScope::Machine)
                        .build()
                )
                .build();
            let actual = settings.resolved().clone();
            pretty_assertions::assert_eq!(actual, expected)
        }

        #[test] fn user_overrides_machine() {
            let settings = &mut DscSettingsBuilder::new()
                .with_machine(MACHINE_DATA.clone())
                .with_user(USER_DATA.clone())
                .build();
            let expected = DscSettingsResolvedBuilder::new()
                .with_resource_path(
                    ResourcePathResolvedSettingsBuilder::new()
                        .with_append_env_path(true, DscSettingsScope::Machine)
                        .with_directories(vec!["/usr/local/bin"], DscSettingsScope::Machine)
                        .build()
                )
                .with_tracing(
                    TracingResolvedSettingsBuilder::new()
                        .with_level(TracingLevelField::Debug, DscSettingsScope::User)
                        .build()
                )
                .build();
            let actual = settings.resolved().clone();
            pretty_assertions::assert_eq!(actual, expected)
        }

        #[test] fn workspace_overrides_user() {
            let settings = &mut DscSettingsBuilder::new()
                .with_machine(MACHINE_DATA.clone())
                .with_user(USER_DATA.clone())
                .with_workspace(WORKSPACE_DATA.clone())
                .build();
            let expected = DscSettingsResolvedBuilder::new()
                .with_resource_path(
                    ResourcePathResolvedSettingsBuilder::new()
                        .with_append_env_path(true, DscSettingsScope::Machine)
                        .with_directories(
                            vec!["~/infra/dsc/resources", "~/infra/dsc/extensions"],
                            DscSettingsScope::Workspace
                        )
                        .build()
                )
                .with_tracing(
                    TracingResolvedSettingsBuilder::new()
                        .with_level(TracingLevelField::Warn, DscSettingsScope::Workspace)
                        .build()
                )
                .build();
            let actual = settings.resolved().clone();
            pretty_assertions::assert_eq!(actual, expected)
        }

        #[test] fn environment_overrides_workspace() {
            let settings = &mut DscSettingsBuilder::new()
                .with_machine(MACHINE_DATA.clone())
                .with_user(USER_DATA.clone())
                .with_workspace(WORKSPACE_DATA.clone())
                .with_environment(ENVIRONMENT_DATA.clone())
                .build();
            let expected = DscSettingsResolvedBuilder::new()
                .with_ignore_settings_file(true, DscSettingsScope::Environment)
                .with_resource_path(
                    ResourcePathResolvedSettingsBuilder::new()
                        .with_directories(
                            vec!["/usr/bin"],
                            DscSettingsScope::Environment
                        )
                        .build()
                )
                .with_tracing(
                    TracingResolvedSettingsBuilder::new()
                        .with_level(TracingLevelField::Info, DscSettingsScope::Environment)
                        .build()
                )
                .build();
            let actual = settings.resolved().clone();
            pretty_assertions::assert_eq!(actual, expected)
        }

        #[test] fn command_line_overrides_environment() {
            let settings = &mut DscSettingsBuilder::new()
                .with_machine(MACHINE_DATA.clone())
                .with_user(USER_DATA.clone())
                .with_workspace(WORKSPACE_DATA.clone())
                .with_environment(ENVIRONMENT_DATA.clone())
                .with_command_line(COMMAND_LINE_DATA.clone())
                .build();
            let expected = DscSettingsResolvedBuilder::new()
                .with_ignore_settings_file(false, DscSettingsScope::CommandLine)
                .with_resource_path(
                    ResourcePathResolvedSettingsBuilder::new()
                        .with_append_env_path(true, DscSettingsScope::Machine)
                        .with_directories(
                            vec!["/usr/bin"],
                            DscSettingsScope::Environment
                        )
                        .build()
                )
                .with_tracing(
                    TracingResolvedSettingsBuilder::new()
                        .with_level(TracingLevelField::Debug, DscSettingsScope::CommandLine)
                        .build()
                )
                .build();
            let actual = settings.resolved().clone();
            pretty_assertions::assert_eq!(actual, expected)
        }

        #[test] fn policy_overrides_all() {
            let settings = &mut DscSettingsBuilder::new()
                .with_machine(MACHINE_DATA.clone())
                .with_user(USER_DATA.clone())
                .with_workspace(WORKSPACE_DATA.clone())
                .with_environment(ENVIRONMENT_DATA.clone())
                .with_command_line(COMMAND_LINE_DATA.clone())
                .with_policy(POLICY_DATA.clone())
                .build();
            let expected = DscSettingsResolvedBuilder::new()
                .with_forbid_ignore_settings_file(true, DscSettingsScope::Policy)
                .with_ignore_settings_file(false, DscSettingsScope::Default)
                .with_resource_path(
                    ResourcePathResolvedSettingsBuilder::new()
                        .with_append_env_path(false, DscSettingsScope::Policy)
                        .with_directories(vec!["/etc/dsc"], DscSettingsScope::Policy)
                        .with_restricted(true, DscSettingsScope::Policy)
                        .build()
                )
                .with_tracing(
                    TracingResolvedSettingsBuilder::new()
                        .with_level(TracingLevelField::Debug, DscSettingsScope::CommandLine)
                        .with_format(TracingFormatField::Default, DscSettingsScope::Default)
                        .build()
                )
                .build();
            let actual = settings.resolved().clone();
            pretty_assertions::assert_eq!(actual, expected)
        }
    }

    mod policy_forbids_ignoring_settings_files {
        use super::*;

        #[test] fn without_policy_returns_false() {
            let settings = &mut DscSettingsBuilder::new().build();
            let expected = false;
            let actual = settings.policy_forbids_ignoring_settings_files();
            pretty_assertions::assert_eq!(actual, expected);
        }
        #[test] fn with_policy_field_undefined_returns_false() {
            let settings = DscSettingsBuilder::new()
                .with_policy(
                    PolicyFileDataBuilder::new()
                        .with_resource_path(
                            ResourcePathFileDataBuilder::new()
                            .with_append_env_path(false)
                            .with_directories(vec!["/etc/dsc"])
                            .with_restricted(true)
                            .build()
                        )
                        .build()
                )
                .build();
            let expected = false;
            let actual = settings.policy_forbids_ignoring_settings_files();
            pretty_assertions::assert_eq!(actual, expected);
        }
        #[test] fn with_policy_field_false_returns_false() {
            let settings = DscSettingsBuilder::new()
                .with_policy(
                    PolicyFileDataBuilder::new()
                        .with_forbid_ignore_settings_file(false)
                        .build()
                )
                .build();
            let expected = false;
            let actual = settings.policy_forbids_ignoring_settings_files();
            pretty_assertions::assert_eq!(actual, expected);
        }

        #[test] fn with_policy_field_true_returns_true() {
            let settings = DscSettingsBuilder::new()
                .with_policy(
                    PolicyFileDataBuilder::new()
                        .with_forbid_ignore_settings_file(true)
                        .build()
                )
                .build();
            let expected = true;
            let actual = settings.policy_forbids_ignoring_settings_files();
            pretty_assertions::assert_eq!(actual, expected);
        }
    }

    mod ignoring_settings_files {
        use super::*;

        #[test] fn code_defaults_only_returns_false() {
            let settings = DscSettingsBuilder::new().build();
            let expected = false;
            let actual = settings.ignoring_settings_files();
            pretty_assertions::assert_eq!(actual, expected);
        }

        #[test] fn with_env_var_true_without_policy_or_cli_returns_true() {
            let settings = DscSettingsBuilder::new()
                .with_environment(
                    EnvironmentDataBuilder::new()
                        .with_ignore_settings_file(true)
                        .build()
                )
                .build();
            let expected = true;
            let actual = settings.ignoring_settings_files();
            pretty_assertions::assert_eq!(actual, expected);
        }

        #[test] fn with_env_var_false_without_policy_or_cli_returns_false() {
            let settings = DscSettingsBuilder::new()
                .with_environment(
                    EnvironmentDataBuilder::new()
                        .with_ignore_settings_file(false)
                        .build()
                )
                .build();
            let expected = false;
            let actual = settings.ignoring_settings_files();
            pretty_assertions::assert_eq!(actual, expected);
        }

        #[test] fn with_cli_arg_true_without_policy_or_env_var_returns_true() {
            let settings = DscSettingsBuilder::new()
                .with_command_line(
                    CommandLineDataBuilder::new()
                        .with_ignore_settings_file(true)
                        .build()
                )
                .build();
            let expected = true;
            let actual = settings.ignoring_settings_files();
            pretty_assertions::assert_eq!(actual, expected);
        }

        #[test] fn with_policy_forbidding_and_env_var_true_returns_false() {
            let settings = DscSettingsBuilder::new()
                .with_policy(
                    PolicyFileDataBuilder::new()
                        .with_forbid_ignore_settings_file(true)
                        .build()
                )
                .with_environment(
                    EnvironmentDataBuilder::new()
                        .with_ignore_settings_file(true)
                        .build()
                )
                .build();
            let expected = false;
            let actual = settings.ignoring_settings_files();
            pretty_assertions::assert_eq!(actual, expected);
        }

        #[test] fn with_policy_forbidding_and_cli_arg_true_returns_false() {
            let settings = DscSettingsBuilder::new()
                .with_policy(
                    PolicyFileDataBuilder::new()
                        .with_forbid_ignore_settings_file(true)
                        .build()
                )
                .with_command_line(
                    CommandLineDataBuilder::new()
                        .with_ignore_settings_file(true)
                        .build()
                )
                .build();
            let expected = false;
            let actual = settings.ignoring_settings_files();
            pretty_assertions::assert_eq!(actual, expected);
        }

        #[test] fn with_policy_field_true_returns_true() {
            let settings = DscSettingsBuilder::new()
                .with_policy(
                    PolicyFileDataBuilder::new()
                        .with_ignore_settings_file(true)
                        .build()
                )
                .build();
            let expected = true;
            let actual = settings.ignoring_settings_files();
            pretty_assertions::assert_eq!(actual, expected);
        }
    }

    #[test] fn resolve_all() {
        let settings = &mut DscSettingsBuilder::new()
            .with_machine(MACHINE_DATA.clone())
            .with_user(USER_DATA.clone())
            .with_environment(ENVIRONMENT_DATA.clone())
            .with_command_line(COMMAND_LINE_DATA.clone())
            .with_policy(POLICY_DATA.clone())
            .build();

        settings.resolve_all();
    }

    // No testing for load() or try_load() - those are handled in the unit tests
    // at `src/tests/settings/dsc_settings.rs` to use the stubs for reading files
    // and environment variables.
}

crate::macros::test_dsc_repo_schema! {
    define_statics: {
        dsc_lib::settings::DscSettings,
        properties: {
            DEFAULT_SCHEMA => "default",
            MACHINE_SCHEMA => "machine",
            USER_SCHEMA => "user",
            WORKSPACE_SCHEMA => "workspace",
            ENVIRONMENT_SCHEMA => "environment",
            COMMAND_LINE_SCHEMA => "cli",
            POLICY_SCHEMA => "policy",
        }
    },
    test_meta_schema: {},
    test_docs: {
        title => "title",
        description => "description",
        markdown_description => "markdownDescription",
        default_title => (DEFAULT_SCHEMA, "title"),
        default_description => (DEFAULT_SCHEMA, "description"),
        default_markdown_description => (DEFAULT_SCHEMA, "markdownDescription"),
        machine_title => (MACHINE_SCHEMA, "title"),
        machine_description => (MACHINE_SCHEMA, "description"),
        machine_markdown_description => (MACHINE_SCHEMA, "markdownDescription"),
        user_title => (USER_SCHEMA, "title"),
        user_description => (USER_SCHEMA, "description"),
        user_markdown_description => (USER_SCHEMA, "markdownDescription"),
        workspace_title => (WORKSPACE_SCHEMA, "title"),
        workspace_description => (WORKSPACE_SCHEMA, "description"),
        workspace_markdown_description => (WORKSPACE_SCHEMA, "markdownDescription"),
        environment_title => (ENVIRONMENT_SCHEMA, "title"),
        environment_description => (ENVIRONMENT_SCHEMA, "description"),
        environment_markdown_description => (ENVIRONMENT_SCHEMA, "markdownDescription"),
        command_line_title => (COMMAND_LINE_SCHEMA, "title"),
        command_line_description => (COMMAND_LINE_SCHEMA, "description"),
        command_line_markdown_description => (COMMAND_LINE_SCHEMA, "markdownDescription"),
        policy_title => (POLICY_SCHEMA, "title"),
        policy_description => (POLICY_SCHEMA, "description"),
        policy_markdown_description => (POLICY_SCHEMA, "markdownDescription"),
    },
    test_validation: {
        with_only_code_defaults_is_valid => {
            input_json: json!({
                "default": dsc_lib::settings::sources::DSC_SETTINGS_CODE_DEFAULTS,
            }),
            expected_valid: true,
        }
    }
}

mod serde {
    use super::super::builders::*;
    use super::fixtures::*;
    #[test] fn serializing() {
        let settings = &mut DscSettingsBuilder::new()
            .with_machine(MACHINE_DATA.clone())
            .with_user(USER_DATA.clone())
            .with_environment(ENVIRONMENT_DATA.clone())
            .with_command_line(COMMAND_LINE_DATA.clone())
            .with_policy(POLICY_DATA.clone())
            .build();

        settings.resolve_all();

        serde_json::to_value(settings)
            .expect("serialization should never fail");
    }
}
