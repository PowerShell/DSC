
use dsc_lib::settings::*;

#[cfg(test)] mod builders;
use builders::*;

#[cfg(test)] mod dsc_settings {
    use std::{path::PathBuf, sync::LazyLock};
    use test_case::test_case;
    use super::*;

    /// Defines policy data with the following settings:
    /// 
    /// - `forbid_ignore_settings_file`: `true`
    /// - `resource_path.append_env_path`: `false`
    /// - `resource_path.directories`: `["/etc/dsc"]`
    /// - `resource_path.restrict_path`: `true`
    /// 
    /// It doesn't define any values for `tracing`.
    static POLICY_DATA: LazyLock<DscPolicyFileData> = LazyLock::new(|| {
        PolicyDataBuilder::new()
            .with_forbid_ignore_settings_file(true)
            .with_resource_path(
                ResourcePathDataBuilder::new()
                .with_append_env_path(false)
                .with_directories(vec!["/etc/dsc".to_string()])
                .with_restrict_path(true)
                .build()
            )
            .build()
    });

    /// Defines machine data with the following settings:
    /// 
    /// - `tracing.level`: `warn`
    /// - `resource_path.append_env_path`: `true`
    /// - `resource_path.directories`: `["/usr/local/bin"]`
    static MACHINE_DATA: LazyLock<DscPreferenceFileData> = LazyLock::new(|| {
        PreferenceDataBuilder::new()
            .with_tracing(
                TracingDataBuilder::new()
                .with_level(TraceLevelField::Warn)
                .build()
            )
            .with_resource_path(
                ResourcePathDataBuilder::new()
                .with_append_env_path(true)
                .with_directories(vec!["/usr/local/bin".to_string()])
                .build()
            )
            .build()
    });

    /// Defines workspace data with the following settings:
    /// 
    /// - `tracing.level`: `warn`
    /// - `resource_path.directories`: `["~/infra/dsc/resources", "~/infra/dsc/extensions"]`
    static WORKSPACE_DATA: LazyLock<DscPreferenceFileData> = LazyLock::new(|| {
        PreferenceDataBuilder::new()
            .with_tracing(
                TracingDataBuilder::new()
                .with_level(TraceLevelField::Warn)
                .build()
            )
            .with_resource_path(
                ResourcePathDataBuilder::new()
                .with_directories(vec![
                    "~/infra/dsc/resources".to_string(),
                    "~/infra/dsc/extensions".to_string()
                ])
                .build()
            )
            .build()
    });

    /// Defines user data with the following settings:
    /// 
    /// - `tracing.level`: `debug`
    static USER_DATA: LazyLock<DscPreferenceFileData> = LazyLock::new(|| {
        PreferenceDataBuilder::new()
            .with_tracing(
                TracingDataBuilder::new()
                .with_level(TraceLevelField::Debug)
                .build()
            )
            .build()
    });

    /// Defines environment data with the following settings:
    /// 
    /// - `dsc_resource_path`: `["/usr/bin"]`
    /// - `dsc_ignore_settings_file`: `true`
    /// - `dsc_trace_level`: `info`
    static ENV_DATA: LazyLock<DscSettingsEnvironmentData> = LazyLock::new(|| {
        EnvironmentDataBuilder::new()
            .with_resource_path(vec![PathBuf::from("/usr/bin")])
            .with_ignore_settings_file(true)
            .with_trace_level(TraceLevelField::Info)
            .build()
    });

    static CLI_DATA: LazyLock<DscSettingsCliData> = LazyLock::new(|| {
        CliDataBuilder::new()
            .with_ignore_settings_file(false)
            .with_trace_level(TraceLevelField::Debug)
            .build()
    });

    fn assert_pretty_resolved_eq(expected: DscSettingsResolved) -> impl Fn(DscSettingsResolved) {
        move |actual: DscSettingsResolved| { pretty_assertions::assert_eq!(actual, expected) }
    }

    #[test_case(
        &mut SettingsBuilder::new().build() => 
        using assert_pretty_resolved_eq(
            ResolvedSettingsBuilder::new().build()
        );
        "with_code_defaults_only"
    )]
    #[test_case(
        &mut SettingsBuilder::new()
            .with_machine(MACHINE_DATA.clone())
            .build() =>
        using assert_pretty_resolved_eq(
            ResolvedSettingsBuilder::new()
                .with_resource_path(
                    ResourcePathResolvedSettingsBuilder::new()
                        .with_append_env_path(true, DscSettingsScope::Machine)
                        .with_directories(vec!["/usr/local/bin".to_string()], DscSettingsScope::Machine)
                        .build()
                )
                .with_tracing(
                    TracingResolvedSettingsBuilder::new()
                        .with_level(TraceLevelField::Warn, DscSettingsScope::Machine)
                        .build()
                )
                .build()
        );
        "machine_overrides_code_defaults"
    )]
    #[test_case(
        &mut SettingsBuilder::new()
            .with_machine(MACHINE_DATA.clone())
            .with_user(USER_DATA.clone())
            .build() =>
        using assert_pretty_resolved_eq(
            ResolvedSettingsBuilder::new()
                .with_resource_path(
                    ResourcePathResolvedSettingsBuilder::new()
                        .with_append_env_path(true, DscSettingsScope::Machine)
                        .with_directories(vec!["/usr/local/bin".to_string()], DscSettingsScope::Machine)
                        .build()
                )
                .with_tracing(
                    TracingResolvedSettingsBuilder::new()
                        .with_level(TraceLevelField::Debug, DscSettingsScope::User)
                        .build()
                )
                .build()
        );
        "user_overrides_machine"
    )]
    #[test_case(
        &mut SettingsBuilder::new()
            .with_machine(MACHINE_DATA.clone())
            .with_user(USER_DATA.clone())
            .with_workspace(WORKSPACE_DATA.clone())
            .build() =>
        using assert_pretty_resolved_eq(
            ResolvedSettingsBuilder::new()
                .with_resource_path(
                    ResourcePathResolvedSettingsBuilder::new()
                        .with_append_env_path(true, DscSettingsScope::Machine)
                        .with_directories(vec![
                            "~/infra/dsc/resources".to_string(),
                            "~/infra/dsc/extensions".to_string()
                        ], DscSettingsScope::Workspace)
                        .build()
                )
                .with_tracing(
                    TracingResolvedSettingsBuilder::new()
                        .with_level(TraceLevelField::Warn, DscSettingsScope::Workspace)
                        .build()
                )
                .build()
        );
        "workspace_overrides_user"
    )]
    #[test_case(
        &mut SettingsBuilder::new()
            .with_machine(MACHINE_DATA.clone())
            .with_user(USER_DATA.clone())
            .with_workspace(WORKSPACE_DATA.clone())
            .with_environment(ENV_DATA.clone())
            .build() =>
        using assert_pretty_resolved_eq(
            ResolvedSettingsBuilder::new()
                .with_ignore_settings_file(true, DscSettingsScope::Environment)
                .with_resource_path(
                    ResourcePathResolvedSettingsBuilder::new()
                        .with_directories(
                            vec!["/usr/bin".to_string()],
                            DscSettingsScope::Environment
                        )
                        .build()
                )
                .with_tracing(
                    TracingResolvedSettingsBuilder::new()
                        .with_level(TraceLevelField::Info, DscSettingsScope::Environment)
                        .build()
                )
                .build()
        );
        "environment_overrides_workspace"
    )]
    #[test_case(
        &mut SettingsBuilder::new()
            .with_machine(MACHINE_DATA.clone())
            .with_user(USER_DATA.clone())
            .with_workspace(WORKSPACE_DATA.clone())
            .with_environment(ENV_DATA.clone())
            .with_command_line(CLI_DATA.clone())
            .build() =>
        using assert_pretty_resolved_eq(
            ResolvedSettingsBuilder::new()
                .with_ignore_settings_file(false, DscSettingsScope::CommandLine)
                .with_resource_path(
                    ResourcePathResolvedSettingsBuilder::new()
                        .with_append_env_path(true, DscSettingsScope::Machine)
                        .with_directories(
                            vec!["/usr/bin".to_string()],
                            DscSettingsScope::Environment
                        )
                        .build()
                )
                .with_tracing(
                    TracingResolvedSettingsBuilder::new()
                        .with_level(TraceLevelField::Debug, DscSettingsScope::CommandLine)
                        .build()
                )
                .build()
        );
        "command_line_overrides_environment"
    )]
    #[test_case(
        &mut SettingsBuilder::new()
            .with_policy(POLICY_DATA.clone())
            .with_machine(MACHINE_DATA.clone())
            .with_user(USER_DATA.clone())
            .with_workspace(WORKSPACE_DATA.clone())
            .with_environment(ENV_DATA.clone())
            .with_command_line(CLI_DATA.clone())
            .build() =>
        using assert_pretty_resolved_eq(ResolvedSettingsBuilder::new()
            .with_forbid_ignore_settings_file(true, DscSettingsScope::Policy)
            .with_ignore_settings_file(false, DscSettingsScope::Default)
            .with_resource_path(
                ResourcePathResolvedSettingsBuilder::new()
                    .with_append_env_path(false, DscSettingsScope::Policy)
                    .with_directories(vec!["/etc/dsc".to_string()], DscSettingsScope::Policy)
                    .with_restrict_path(true, DscSettingsScope::Policy)
                    .build()
            )
            .with_tracing(
                TracingResolvedSettingsBuilder::new()
                    .with_level(TraceLevelField::Debug, DscSettingsScope::CommandLine)
                    .with_format(TraceFormatField::Default, DscSettingsScope::Default)
                    .build()
            )
            .build()
        );
        "policy_overrides_all"
    )]
    fn resolved(settings: &mut DscSettings) -> DscSettingsResolved {
        settings.resolved().clone()
    }

    #[test_case(
        &mut SettingsBuilder::new().build() =>
        false;
        "without_policy_returns_false"
    )]
    #[test_case(
        &mut SettingsBuilder::new()
            .with_policy(
                PolicyDataBuilder::new()
                    .with_resource_path(
                        ResourcePathDataBuilder::new()
                        .with_append_env_path(false)
                        .with_directories(vec!["/etc/dsc".to_string()])
                        .with_restrict_path(true)
                        .build()
                    )
                    .build()
            )
            .build() =>
        false;
        "with_policy_field_undefined_returns_false"
    )]
    #[test_case(
        &mut SettingsBuilder::new()
            .with_policy(
                PolicyDataBuilder::new()
                    .with_forbid_ignore_settings_file(false)
                    .build()
            )
            .build() =>
        false;
        "with_policy_field_false_returns_false"
    )]
    #[test_case(
        &mut SettingsBuilder::new()
            .with_policy(
                PolicyDataBuilder::new()
                    .with_forbid_ignore_settings_file(true)
                    .build()
            )
            .build() =>
        true;
        "with_policy_field_true_returns_true"
    )]
    fn policy_forbids_ignoring_settings_files(settings: &mut DscSettings) -> bool {
        settings.policy_forbids_ignoring_settings_files()
    }

    #[test_case(
        &mut SettingsBuilder::new().build() =>
        false;
        "code_defaults_only_returns_false"
    )]
    #[test_case(
        &mut SettingsBuilder::new()
            .with_environment(
                EnvironmentDataBuilder::new()
                    .with_ignore_settings_file(true)
                    .build()
            )
            .build() =>
        true;
        "with_env_var_true_without_policy_or_cli_returns_true"
    )]
    #[test_case(
        &mut SettingsBuilder::new()
            .with_environment(
                EnvironmentDataBuilder::new()
                    .with_ignore_settings_file(false)
                    .build()
            )
            .build() =>
        false;
        "with_env_var_false_without_policy_or_cli_returns_false"
    )]
    #[test_case(
        &mut SettingsBuilder::new()
            .with_command_line(
                CliDataBuilder::new()
                    .with_ignore_settings_file(true)
                    .build()
            )
            .build() =>
        true;
        "with_cli_arg_true_without_policy_or_env_var_returns_true"
    )]
    #[test_case(
        &mut SettingsBuilder::new()
            .with_policy(
                PolicyDataBuilder::new()
                    .with_forbid_ignore_settings_file(true)
                    .build()
            )
            .with_environment(
                EnvironmentDataBuilder::new()
                    .with_ignore_settings_file(true)
                    .build()
            )
            .build() =>
        false;
        "with_policy_forbidding_and_env_var_true_returns_false"
    )]
    #[test_case(
        &mut SettingsBuilder::new()
            .with_policy(
                PolicyDataBuilder::new()
                    .with_forbid_ignore_settings_file(true)
                    .build()
            )
            .build() =>
        true;
        "with_policy_field_true_returns_true"
    )]
    fn ignoring_settings_files(settings: &mut DscSettings) -> bool {
        settings.ignoring_settings_files()
    }
}