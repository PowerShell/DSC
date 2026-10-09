// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::settings::{DscSettings, DscSettingsError};
use crate::settings::fields::*;

#[macro_use]
pub(super) mod macros {
    /// Defines a macro to panic with the current settings and a custom message.
    ///
    /// This helps provide more context to a test failure to reduce the need
    /// for debugging to understand the state of the settings at the time of
    /// the panic.
    ///
    /// This macro is only intended to be used in tests for [`DscSettings`].
    ///
    /// # Examples
    ///
    /// ## For specific variable
    ///
    /// This example shows how to panic with the current settings and a
    /// custom message.
    ///
    /// ```rust, ignore
    /// let foo_settings = DscSettings::default();
    /// let msg = "example info";
    /// panic_with_settings!(foo_settings, "Custom panic message: {}", msg);
    /// ```
    ///
    /// The first arugment to the macro is the variable containing the settings
    /// instance. The second argument is a format string. All remaining
    /// arguments are used to format the custom panic message.
    ///
    /// ## For generic variable
    ///
    /// This example shows how to panic with the current settings and a
    /// custom message.
    ///
    /// ```rust, ignore
    /// let settings = DscSettings::default();
    /// let msg = "example info";
    /// panic_with_settings!("Custom panic message: {}", msg);
    /// ```
    ///
    /// In this example, because the variable is already defined as the
    /// default identity `settings`, the invocation omits the variable name
    /// for the first argument and passes only the format string and arguments.
    macro_rules! panic_with_settings {
        ($settings:ident, $fmt:expr $(, $arg:tt)*) => {
            std::panic::panic_any(
                $crate::tests::settings::dsc_settings::helpers::SettingsPanic{
                    settings: $settings.clone(),
                    message: format!($fmt $(, $arg)*),
                }
            )
        };
        ($fmt:expr $(, $arg:tt)*) => {
            std::panic::panic_any(
                $crate::tests::settings::dsc_settings::helpers::SettingsPanic{
                    settings: settings.clone(),
                    message: format!($fmt $(, $arg)*),
                }
            );
        };
    }

    /// Defines the testing context for the current test, only intended for
    /// testing [`DscSettings`].
    ///
    /// It provides a quick way to set up the stubs for environment variables
    /// and settings files, which [`DscSettings`] reads for the load methods.
    ///
    /// This macro should be invoked at the beginning of each test to ensure a
    /// clean and consistent testing environment.
    ///
    /// <div class="warning">
    ///
    /// When using this macro, make sure to annotate the test functions with
    /// `#[serial]` to ensure the tests aren't run in parallel. Parallel
    /// execution can cause race conditions and poison the mutexes for the
    /// stubs.
    ///
    /// </div>
    ///
    /// ## Syntax
    ///
    /// This macro accepts zero or more key-value pairs. Each key represents a
    /// source scope:
    ///
    /// - `policy`: Repesents the policy settings file.
    ///
    ///   Define the value as a JSON object to stub the file as existing with
    ///   the given data.
    ///
    ///   Define the value as `read_error("custom error message")` to simulate
    ///   a read error.
    /// - `machine`: Represents the machine preference settings file.
    ///
    ///   Define the value as a JSON object to stub the file as existing with
    ///   the given data.
    ///
    ///   Define the value as `read_error("custom error message")` to simulate
    ///   a read error.
    /// - `user`: Represents the user preference settings file.
    ///
    ///   Define the value as a JSON object to stub the file as existing with
    ///   the given data.
    ///
    ///   Define the value as `read_error("custom error message")` to simulate
    ///   a read error.
    /// - `workspace`: Represents the workspace preference settings file.
    ///
    ///   Define the value as a JSON object to stub the file as existing with
    ///   the given data.
    ///
    ///   Define the value as `read_error("custom error message")` to simulate
    ///   a read error.
    /// - `environment`: Represents the environment variables.
    ///
    ///   Define the value as a key-value map to stub the environment variables
    ///   as existing with the given data. Omit any variables you want to treat
    ///   as undefined.
    ///
    /// # Examples
    ///
    /// ## Resetting the context
    ///
    /// For some tests, you may not want _any_ settings files or environment variables defined.
    /// In such cases, you can simply invoke `define_context!{}` at the beginning of your test:
    ///
    /// ```rust, ignore
    /// #[serial]
    /// #[test]
    /// fn without_any_defined() {
    ///     define_context!{}
    ///
    ///     let mut settings = DscSettings::default();
    ///     settings.load();
    ///
    ///     pretty_assertions::assert_eq!(
    ///         settings,
    ///         DscSettings::default()
    ///     );
    /// }
    /// ```
    ///
    /// ## Defining multiple sources
    ///
    /// To define multiple sources, specify each source as a separate key-value pair within the
    /// macro. The order doesn't matter.
    ///
    /// ```rust, ignore
    /// define_context!(
    ///     policy: { "forbidIgnoreSettingsFile": true },
    ///     machine: { "tracing": { "level": "info" } },
    ///     environment: {
    ///         DSC_TRACE_FORMAT => "json"
    ///     }
    /// )
    /// ```
    macro_rules! define_context {
        () => {
            let _old_hook = helpers::install_custom_hook();
            $crate::tests::stubs::env::StubbedEnvContext::reset();
            $crate::tests::stubs::path::StubbedPathContextMap::reset();
        };
        ($($all:tt)+) => {
            define_context!();
            add_context!($($all)+);
        };
    }

    /// Semi-private macro used internally by `define_context!`. Do not define
    /// directly.
    macro_rules! add_context {
        () => {};
        (policy: {$($json:tt)*} $(, $($rest:tt)*)?) => {
            define_policy_file!(json: {$($json)*});
            $(add_context!($($rest)*);)?
        };
        (policy: read_error($($policy_err:tt)+) $(, $($rest:tt)*)?) => {
            define_policy_file!(read_err: $($policy_err)+);
            $(add_context!($($rest)*);)?
        };
        (machine: {$($json:tt)*} $(, $($rest:tt)*)?) => {
            define_preference_file!(machine, json: {$($json)*});
            $(add_context!($($rest)*);)?
        };
        (machine: read_err($($machine_err:tt)+) $(, $($rest:tt)*)?) => {
            define_preference_file!(machine, read_err: $($machine_err)+);
            $(add_context!($($rest)*);)?
        };
        (user: {$($json:tt)*} $(, $($rest:tt)*)?) => {
            define_preference_file!(user, json: {$($json)*});
            $(add_context!($($rest)*);)?
        };
        (user: read_err($($user_err:tt)+) $(, $($rest:tt)*)?) => {
            define_preference_file!(user, read_err: $($user_err)+);
            $(add_context!($($rest)*);)?
        };
        (workspace: {$($json:tt)*} $(, $($rest:tt)*)?) => {
            define_preference_file!(workspace, json: {$($json)*});
            $(add_context!($($rest)*);)?
        };
        (workspace: read_err($($workspace_err:tt)+) $(, $($rest:tt)*)?) => {
            define_preference_file!(workspace, read_err: $($workspace_err)+);
            $(add_context!($($rest)*);)?
        };
        (
            environment: {
                $($env_var:ident: $env_var_value:literal),*
                $(,)?
            }
            $(, $($rest:tt)*)?
        ) => {
            $(
                define_env_var!(name: stringify!($env_var), value: $env_var_value.to_string());
            )*
            $(add_context!($($rest)*);)?
        };
    }

    /// Defines an environment variable for testing purposes.
    ///
    /// Use this macro in a test function when you want to update the environment after initial
    /// setup. For example, to reload settings with new environment variables.
    ///
    /// Prefer using the [`define_context!`] macro for initial setup in a test function over
    /// composing the environment with multiple invocations of this macro.
    ///
    /// # Examples
    ///
    /// ## Variable with value
    ///
    /// This example shows how you can define an environment variable with a
    /// specific value.
    ///
    /// ```rust, ignore
    /// define_env_var!(name: "MY_ENV_VAR", value: "my_value");
    /// ```
    ///
    /// ## Variable with error
    ///
    /// This example shows how you can define an environment variable that
    /// results in a [`std::env::VarError::NotPresent`] error when accessed.
    ///
    /// ```rust, ignore
    /// define_env_var!(name: "MY_ENV_VAR", err: NotPresent);
    /// ```
    macro_rules! define_env_var {
        (name: $name:expr, value: $value:expr) => {
            $crate::tests::stubs::env::StubbedEnvContext::add($name, Ok($value));
        };
        (name: $name:expr, err: $err_variant:ident) => {
            $crate::tests::stubs::env::StubbedEnvContext::add($name, Err(std::env::VarError::$err_variant));
        };
        (name: $name:expr, err: $err:expr) => {
            $crate::tests::stubs::env::StubbedEnvContext::add($name, Err($err));
        };
    }

    /// Defines a stub for the machine policy file in the test environment.
    ///
    /// # Examples
    ///
    /// ## With JSON data
    ///
    /// In this example, the macro defines an entry for the policy file in the
    /// stubbed context map that:
    ///
    /// - indicates that the policy file should exist.
    /// - specifies that the policy file is a regular file.
    /// - provides the JSON content to be returned when the file is read.
    ///
    /// ```rust, ignore
    /// define_policy_file!(json: { "key": "value" });
    /// ```
    ///
    /// ## With a not-found read error
    ///
    /// In this example, the macro defines an entry for the policy file in the
    /// stubbed context map that:
    ///
    /// - indicates that the policy file should exist.
    /// - specifies that the policy file is a regular file.
    /// - provides a not-found error with the specified message attempting to
    ///   read the file.
    ///
    /// ```rust, ignore
    /// // String literal
    /// define_policy_file!(read_err: "custom message");
    /// // Expression
    /// define_policy_file!(read_err: format!("custom message: {}", 42));
    /// ```
    ///
    /// ## With a custom read error and generic message
    ///
    /// In this example, the macro defines an entry for the policy file in the
    /// stubbed context map that:
    ///
    /// - indicates that the policy file should exist.
    /// - specifies that the policy file is a regular file.
    /// - provides a custom read error with the specified kind and message.
    ///
    /// ```rust, ignore
    /// define_policy_file!(read_err: PermissionDenied});
    /// ```
    ///
    /// ## With a custom read error and message
    ///
    /// In this example, the macro defines an entry for the policy file in the
    /// stubbed context map that:
    ///
    /// - indicates that the policy file should exist.
    /// - specifies that the policy file is a regular file.
    /// - provides a custom read error with the specified kind and message.
    ///
    /// ```rust, ignore
    /// define_policy_file!(read_err: {
    ///     kind: PermissionDenied,
    ///     msg: "custom message"
    /// });
    /// ```
    macro_rules! define_policy_file {
        (json: {$($json:tt)*}) => {
            $crate::tests::stubs::path::StubbedPathContextMap::add(
                $crate::settings::POLICY_SETTINGS_FILE_PATH.to_string(),
                $crate::tests::stubs::path::StubbedPathContext {
                    should_exist: true,
                    is_file: true,
                    read_result: Ok(
                        serde_json::to_string_pretty(
                            &serde_json::json!({$($json)*})
                        ).unwrap()
                    ),
                    ..Default::default()
                }
            );
        };
        (read_err: $err_kind:ident) => {
            define_policy_file!{
                read_err: {
                    kind: $err_kind,
                    msg: format!("error reading '{}'", $crate::settings::POLICY_SETTINGS_FILE_PATH.to_string_lossy()).as_str()
                }
            }
        };
        (read_err: $err_msg:literal) => {
            define_policy_file! {
                read_err: {
                    kind: NotFound,
                    msg: $err_msg
                }
            }
        };
        (read_err: {kind: $err_kind:ident, msg: $err_msg:expr}) => {
            $crate::tests::stubs::path::StubbedPathContextMap::add(
                $crate::settings::POLICY_SETTINGS_FILE_PATH.to_string(),
                $crate::tests::stubs::path::StubbedPathContext {
                    should_exist: true,
                    is_file: true,
                    read_result: Err(std::io::Error::new(std::io::ErrorKind::$err_kind, $err_msg)),
                    ..Default::default()
                }
            );
        };
        (read_err: $err_msg:expr) => {
            define_policy_file! {
                read_err: {
                    kind: NotFound,
                    msg: $err_msg
                }
            }
        };
    }

    /// Defines a preference file for testing purposes in the stubbed context.
    ///
    /// # Syntax
    ///
    /// The macro uses the following patterns:
    ///
    /// 1. Define with JSON data:
    ///
    ///    ```rust, ignore
    ///    define_preference_file!(<scope>, json: { "key": "value" });
    ///    ```
    ///
    /// 1. Define with a read error:
    ///
    ///    ```rust, ignore
    ///    define_preference_file!(<scope>, read_err: <err>);
    ///    ```
    ///
    ///    Where `<err>` is either a variant of [`std::io::ErrorKind`] or a
    ///    custom error message as a string.
    ///
    /// For both patterns, the first argument, `<scope>`, indicates which
    /// preference file to stub:
    ///
    /// - `machine`
    /// - `user`
    /// - `workspace`
    ///
    /// # Examples
    ///
    /// ## Machine preference file with JSON data
    ///
    /// Define a machine preference file with JSON data:
    ///
    /// ```rust, ignore
    /// define_preference_file!(machine, json: { "key": "value" });
    /// ```
    ///
    /// ## User with error kind
    ///
    /// Define a user preference file with a read error:
    ///
    /// ```rust, ignore
    /// define_preference_file!(user, read_err: NotFound);
    /// ```
    ///
    /// ## Workspace with custom error message
    ///
    /// Define a workspace preference file with a read error using a custom
    /// error message:
    ///
    /// ```rust, ignore
    /// define_preference_file!(workspace, read_err: "custom error message");
    /// ```
    macro_rules! define_preference_file {
        (machine, json: {$($json:tt)*}) => {
            define_preference_file!{
                path: $crate::settings::MACHINE_SETTINGS_FILE_PATH,
                json: {$($json)*}
            }
        };
        (machine, read_err: $($err:tt)+) => {
            define_preference_file!{
                path: $crate::settings::MACHINE_SETTINGS_FILE_PATH,
                read_err: $($err)+
            }
        };
        (user, json: {$($json:tt)*}) => {
            define_preference_file!{
                path: $crate::settings::USER_SETTINGS_FILE_PATH,
                json: {$($json)*}
            }
        };
        (user, read_err: $($err:tt)+) => {
            define_preference_file!{
                path: $crate::settings::USER_SETTINGS_FILE_PATH,
                read_err: $($err)+
            }
        };
        (workspace, json: {$($json:tt)*}) => {
            define_preference_file!{
                path: $crate::settings::WORKSPACE_SETTINGS_FILE_PATH,
                json: {$($json)*}
            }
        };
        (workspace, read_err: $($err:tt)+) => {
            define_preference_file!{
                path: $crate::settings::WORKSPACE_SETTINGS_FILE_PATH,
                read_err: $($err)+
            }
        };
        (path: $path:expr, json: {$($json:tt)*}) => {
            $crate::tests::stubs::path::StubbedPathContextMap::add(
                $path.to_string(),
                $crate::tests::stubs::path::StubbedPathContext {
                    should_exist: true,
                    is_file: true,
                    read_result: Ok(
                        serde_json::to_string_pretty(
                            &serde_json::json!({$($json)*})
                        ).unwrap()
                    ),
                    ..Default::default()
                }
            );
        };
        (path: $path:expr, read_err: $err_kind:ident) => {
            define_preference_file!{
                path: $path,
                read_err: {
                    kind: $err_kind,
                    msg: format!("error reading '{}'", $path.to_string()).as_str()
                }
            }
        };
        (path: $path:expr, read_err: $err_msg:literal) => {
            define_preference_file! {
                path: $path,
                read_err: {
                    kind: NotFound,
                    msg: $err_msg
                }
            }
        };
        (path: $path:expr, read_err: {kind: $err_kind:ident, msg: $err_msg:expr}) => {
            $crate::tests::stubs::path::StubbedPathContextMap::add(
                $path.to_string_lossy().to_string(),
                $crate::tests::stubs::path::StubbedPathContext {
                    should_exist: true,
                    is_file: true,
                    read_result: Err(std::io::Error::new(std::io::ErrorKind::$err_kind, $err_msg)),
                    ..Default::default()
                }
            );
        };
        (path: $path:expr, read_err: $err_msg:expr) => {
            define_preference_file! {
                path: $path,
                read_err: {
                    kind: NotFound,
                    msg: $err_msg
                }
            }
        };
    }
}

pub(super) mod helpers {
    use std::panic::{self, PanicHookInfo};

    /// Defines information for the custom panic hook used in these tests.
    pub struct SettingsPanic {
        /// The panic message associated with this panic.
        pub message: String,
        /// The settings associated with this panic.
        pub settings: crate::settings::DscSettings,
    }

    /// Defines a custom panic hook for testing the DSC settings.
    ///
    /// This provides an alternate failure view for the tests in this file that
    /// captures additional context about the panic, such as the current
    /// settings and stubbed environment.
    ///
    /// This makes it easier to understand why a test failed. Without this
    /// hook, you need to debug the test and inspect the stubbed context and
    /// settings carefully before the panic.
    ///
    /// # Arguments
    ///
    /// - `info` - A reference to the [`PanicHookInfo`] containing information
    ///   about the panic.
    fn custom_panic_hook(info: &PanicHookInfo) {
        let mut context = serde_json::Map::new();
        let location = info
            .location()
            .map(|l| format!("{}:{}:{}", l.file(), l.line(), l.column()))
            .unwrap_or_else(|| "unknown location".to_string());

        let message = if let Some(sp) = info.payload().downcast_ref::<SettingsPanic>() {
            context.insert(String::from("settings"), serde_json::to_value(&sp.settings).unwrap());
            &sp.message
        } else if let Some(s) = info.payload_as_str() {
            s
        } else if let Some(s) = info.payload().downcast_ref::<Box<dyn std::fmt::Display>>() {
            &s.to_string()
        } else {
            "unknown panic payload"
        };

        if let Ok(file_context) = crate::tests::stubs::path::StubbedPathContextMap::get().lock() {
            context.insert(String::from("stubbed_files"), file_context.as_json());
        }
        if let Ok(env_context) = crate::tests::stubs::env::StubbedEnvContext::get().lock() {
            context.insert(String::from("stubbed_env"), env_context.as_json());
        }

        let context = serde_json::to_string_pretty(&context).unwrap();
        let thread = std::thread::current().name().unwrap_or("main").to_string();
        let color_start = "\x1b[31;1m";
        let color_end = "\x1b[0m";
        eprintln!("{color_start}thread '{thread}' panicked{color_end}");
        eprintln!("{color_start}at:{color_end}      {location}");
        eprintln!("{color_start}message:{color_end} {message}");
        eprintln!("{color_start}context:{color_end} {context}");
        // Emit backtrace if needed
        if let Ok(bt) = std::env::var("RUST_BACKTRACE") {
            if bt != "0" {
                eprintln!("Backtrace:\n{}", std::backtrace::Backtrace::force_capture());
            }
        }
    }
    /// Install the custom hook and return the previous one so it can be restored later
    pub(super) fn install_custom_hook() -> Box<dyn Fn(&PanicHookInfo) + Sync + Send + 'static> {
        let old_hook = panic::take_hook();
        panic::set_hook(Box::new(custom_panic_hook));
        old_hook
    }
}

/// Tests for [`DscSettings::load()`] with stubbed environment and filesystem.
mod load {
    use super::*;
    use pretty_assertions::assert_eq;
    use serial_test::serial;

    #[serial]
    #[test]
    fn when_no_settings_defined() {
        define_context!{}

        let mut actual = DscSettings::new();
        actual.load();

        assert_eq!(actual.policy, None);
        assert_eq!(actual.machine, None);
        assert_eq!(actual.user, None);
        assert_eq!(actual.workspace, None);
        assert_eq!(actual.environment, None);
        assert_eq!(actual.command_line, None);
    }

    #[serial]
    #[test]
    fn when_all_defined_valid() {
        define_context! {
            policy: {
                "forbidIgnoreSettingsFile": false,
                "ignoreSettingsFile": false,
            },
            machine: {
                "tracing": {
                    "level": "info",
                },
            },
            user: {
                "resourcePath": {
                    "directories": [
                        "/usr/resources",
                    ],
                },
            },
            workspace: {
                "tracing": {
                    "format": "json",
                },
            },
            environment: {
                DSC_TRACE_LEVEL: "debug",
                DSC_IGNORE_SETTINGS_FILE: "true",
            }
        }

        let mut settings = DscSettings::new();
        settings.load();

        if let Some(policy) = settings.policy.as_ref() {
            assert_eq!(policy.forbid_ignore_settings_file.expect("policy should define forbidIgnoreSettingsFile"), false);
            assert_eq!(policy.ignore_settings_file.expect("policy should define ignoreSettingsFile"), false);
            assert_eq!(policy.resource_path, None);
            assert_eq!(policy.tracing, None)
        } else {
            panic_with_settings!(settings, "expected policy settings to be defined")
        }

        if let Some(machine) = settings.machine.as_ref() {
            assert_eq!(machine.resource_path, None);
            let tracing = machine.tracing.as_ref().expect("machine should define tracing");
            assert_eq!(
                tracing.level.as_ref().expect("machine should define tracing.level"),
                &TracingLevelField::Info
            );
            assert_eq!(tracing.format, None);
        } else {
            panic_with_settings!(settings, "expected machine settings to be defined")
        }

        if let Some(user) = settings.user.as_ref() {
            assert_eq!(user.tracing, None);
            let resource_path = user.resource_path.as_ref().expect("user should define 'resourcePath'");
            assert_eq!(
                resource_path.directories.as_ref().expect("user should define resourcePath.directories"),
                &vec![std::path::PathBuf::from("/usr/resources")]
            );
            assert_eq!(resource_path.append_env_path, None);
            assert_eq!(resource_path.restricted, None);
        } else {
            panic_with_settings!(settings, "expected user settings to be defined")
        }

        if let Some(workspace) = settings.workspace.as_ref() {
            assert_eq!(workspace.resource_path, None);
            let tracing = workspace.tracing.as_ref().expect("workspace should define tracing");
            assert_eq!(
                tracing.format.as_ref().expect("workspace should define tracing.format"),
                &TracingFormatField::Json
            );
            assert_eq!(tracing.level, None);
        } else {
            panic_with_settings!(settings, "expected workspace settings to be defined")
        }

        if let Some(environment) = settings.environment.as_ref() {
            assert_eq!(environment.dsc_ignore_settings_file, Some(true.into()));
            assert_eq!(environment.dsc_trace_level, Some(TracingLevelField::Debug));
            assert_eq!(environment.dsc_trace_format, None);
            assert_eq!(environment.dsc_resource_path, None);
            assert_eq!(environment.dsc_restricted_path, None);
        } else {
            panic_with_settings!(settings, "expected environment settings to be defined");
        }
    }

    #[test]
    #[serial]
    fn when_all_defined_with_invalid_fields() {
        define_context! {
            policy: {
                "forbidIgnoreSettingsFile": "invalid",
            },
            machine: {
                "tracing": {
                    "level": "invalid",
                },
            },
            user: {
                "resourcePath": {
                    "directories": "invalid"
                },
            },
            workspace: {
                "tracing": {
                    "format": "invalid",
                },
            },
            environment: {
                DSC_TRACE_LEVEL: "invalid",
                DSC_IGNORE_SETTINGS_FILE: "invalid",
            }
        }

        let mut settings = DscSettings::new();
        settings.load();

        assert_eq!(settings.policy, None);
        assert_eq!(settings.machine, None);
        assert_eq!(settings.user, None);
        assert_eq!(settings.workspace, None);
        assert_eq!(settings.environment, None);
        assert_eq!(settings.command_line, None);
    }
}

/// Tests for [`DscSettings::try_load()`] with stubbed environment and
/// filesystem.
mod try_load {
    use std::ops::Index;

    use super::*;
    use pretty_assertions::assert_eq;
    use serial_test::serial;

    #[serial]
    #[test]
    fn when_no_settings_defined() {
        define_context!{}

        let mut actual = DscSettings::new();
        actual.try_load().expect("settings should load without error");

        assert_eq!(actual.policy, None);
        assert_eq!(actual.machine, None);
        assert_eq!(actual.user, None);
        assert_eq!(actual.workspace, None);
        assert_eq!(actual.environment, None);
        assert_eq!(actual.command_line, None);
    }

    #[serial]
    #[test]
    fn when_all_defined_valid() {
        define_context! {
            policy: {
                "forbidIgnoreSettingsFile": false,
                "ignoreSettingsFile": false,
            },
            machine: {
                "tracing": {
                    "level": "info",
                },
            },
            user: {
                "resourcePath": {
                    "directories": [
                        "/usr/resources",
                    ],
                },
            },
            workspace: {
                "tracing": {
                    "format": "json",
                },
            },
            environment: {
                DSC_TRACE_LEVEL: "debug",
                DSC_IGNORE_SETTINGS_FILE: "true",
            }
        }

        let mut settings = DscSettings::new();
        settings.try_load().expect("settings should load without error");

        if let Some(policy) = settings.policy.as_ref() {
            assert_eq!(policy.forbid_ignore_settings_file.expect("policy should define forbidIgnoreSettingsFile"), false);
            assert_eq!(policy.ignore_settings_file.expect("policy should define ignoreSettingsFile"), false);
            assert_eq!(policy.resource_path, None);
            assert_eq!(policy.tracing, None)
        } else {
            panic_with_settings!(settings, "expected policy settings to be defined")
        }

        if let Some(machine) = settings.machine.as_ref() {
            assert_eq!(machine.resource_path, None);
            let tracing = machine.tracing.as_ref().expect("machine should define tracing");
            assert_eq!(
                tracing.level.as_ref().expect("machine should define tracing.level"),
                &TracingLevelField::Info
            );
            assert_eq!(tracing.format, None);
        } else {
            panic_with_settings!(settings, "expected machine settings to be defined")
        }

        if let Some(user) = settings.user.as_ref() {
            assert_eq!(user.tracing, None);
            let resource_path = user.resource_path.as_ref().expect("user should define 'resourcePath'");
            assert_eq!(
                resource_path.directories.as_ref().expect("user should define resourcePath.directories"),
                &vec![std::path::PathBuf::from("/usr/resources")]
            );
            assert_eq!(resource_path.append_env_path, None);
            assert_eq!(resource_path.restricted, None);
        } else {
            panic_with_settings!(settings, "expected user settings to be defined")
        }

        if let Some(workspace) = settings.workspace.as_ref() {
            assert_eq!(workspace.resource_path, None);
            let tracing = workspace.tracing.as_ref().expect("workspace should define tracing");
            assert_eq!(
                tracing.format.as_ref().expect("workspace should define tracing.format"),
                &TracingFormatField::Json
            );
            assert_eq!(tracing.level, None);
        } else {
            panic_with_settings!(settings, "expected workspace settings to be defined")
        }

        if let Some(environment) = settings.environment.as_ref() {
            assert_eq!(environment.dsc_ignore_settings_file, Some(true.into()));
            assert_eq!(environment.dsc_trace_level, Some(TracingLevelField::Debug));
            assert_eq!(environment.dsc_trace_format, None);
            assert_eq!(environment.dsc_resource_path, None);
            assert_eq!(environment.dsc_restricted_path, None);
        } else {
            panic_with_settings!(settings, "expected environment settings to be defined");
        }
    }

    #[test]
    #[serial]
    fn when_all_defined_with_invalid_fields() {
        define_context! {
            policy: {
                "forbidIgnoreSettingsFile": "invalid",
            },
            machine: {
                "tracing": {
                    "level": "invalid",
                },
            },
            user: {
                "resourcePath": {
                    "directories": "invalid"
                },
            },
            workspace: {
                "tracing": {
                    "format": "invalid",
                },
            },
            environment: {
                DSC_TRACE_LEVEL: "invalid",
                DSC_IGNORE_SETTINGS_FILE: "invalid",
            }
        }

        let mut settings = DscSettings::new();

        match settings.try_load().expect_err("expected an error due to invalid fields") {
            DscSettingsError::LoadMultipleErrors{ errors } => {
                assert_eq!(errors.len(), 5);
                assert!(matches!(errors.index(0), DscSettingsError::DataFileUnparseable { .. }));
                assert!(matches!(errors.index(1), DscSettingsError::DataFileUnparseable { .. }));
                assert!(matches!(errors.index(2), DscSettingsError::DataFileUnparseable { .. }));
                assert!(matches!(errors.index(3), DscSettingsError::DataFileUnparseable { .. }));
                assert!(matches!(errors.index(4), DscSettingsError::LoadEnvironmentMultipleErrors { .. }));
            },
            e => panic!("expected DscSettingsError::LoadMultipleErrors but got {:#?}", e),
        }

        assert_eq!(settings.policy, None);
        assert_eq!(settings.machine, None);
        assert_eq!(settings.user, None);
        assert_eq!(settings.workspace, None);
        assert_eq!(settings.environment, None);
        assert_eq!(settings.command_line, None);
    }
}
