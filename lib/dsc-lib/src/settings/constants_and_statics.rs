// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! Defines static lazy-initialized paths to the various settings files used by DSC.
//!
//! These paths are determined at runtime based on the operating system and environment variables,
//! and they provide a consistent way to access the settings files across different platforms.

use std::sync::LazyLock;

#[cfg(not(test))]
use std::path::{PathBuf, Path};

#[cfg(test)]
use crate::tests::stubs::path::{PathBuf, Path};

/// Name of the settings file used for the machine, user, and workspace scopes.
pub const SETTINGS_PREFERENCE_FILE_NAME: &str = "dsc.settings.json";
/// Name of the policy file used for the policy scope.
pub const SETTINGS_POLICY_FILE_NAME: &str = "dsc.policy.json";

/// Defines the full path to the policy settings file, which is located in a
/// platform-specific folder.
///
/// The pseudo-path for this file depends on the platform:
///
/// - On Windows: `{ProgramData}\dsc\dsc.policy.json`
/// - On macOS: `/Library/Application Support/dsc/dsc.policy.json`
/// - On Linux and other Unix-like systems: `/etc/dsc/dsc.policy.json`
pub static POLICY_SETTINGS_FILE_PATH: LazyLock<PathBuf> = LazyLock::new(|| {
    machine_folder_path().join(SETTINGS_POLICY_FILE_NAME)
});

/// Defines the full path to the machine settings file, which is located in a
/// platform-specific folder.
///
/// The pseudo-path for this file depends on the platform:
///
/// - On Windows: `{ProgramData}\dsc\dsc.settings.json`
/// - On macOS: `/Library/Application Support/dsc/dsc.settings.json`
/// - On Linux and other Unix-like systems: `/etc/dsc/dsc.settings.json`
pub static MACHINE_SETTINGS_FILE_PATH: LazyLock<PathBuf> = LazyLock::new(|| {
    machine_folder_path().join(SETTINGS_PREFERENCE_FILE_NAME)
});

/// Defines the full path to the user settings file, which is located in a
/// platform-specific folder based on the user's home directory or environment
/// variables.
///
/// The pseudo-path for this file depends on the platform and whether the
/// `XDG_CONFIG_HOME` environment variable is set. When `XDG_CONFIG_HOME` is
/// set, the path is always `{XDG_CONFIG_HOME}/dsc/dsc.settings.json` (using
/// `\` instead of `/` on Windows).
///
/// Otherwise, the path varies by platform:
///
/// - On Windows: `{APPDATA}\dsc\dsc.settings.json`
/// - On macOS: `{HOME}/Library/Application Support/dsc/dsc.settings.json`
/// - On Linux and other Unix-like systems: `{HOME}/.config/dsc/dsc.settings.json`
pub static USER_SETTINGS_FILE_PATH: LazyLock<PathBuf> = LazyLock::new(|| {
    if let Some(xdg_config_home) = std::env::var_os("XDG_CONFIG_HOME") {
        return Path::new(&xdg_config_home)
            .join("dsc")
            .join(SETTINGS_PREFERENCE_FILE_NAME);
    }

    #[cfg(target_os = "windows")]
    {
        let app_data = std::env::var_os("APPDATA")
            .expect("Couldn't retrieve the APPDATA environment variable");
        Path::new(&app_data)
            .join("dsc")
            .join(SETTINGS_PREFERENCE_FILE_NAME)
    }
    #[cfg(target_os = "macos")]
    {
        let home = std::env::var_os("HOME")
            .expect("Couldn't retrieve the HOME environment variable");
        Path::new(&home)
            .join("Library")
            .join("Application Support")
            .join("dsc")
            .join(SETTINGS_PREFERENCE_FILE_NAME)
    }
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        let home = std::env::var_os("HOME")
            .expect("Couldn't retrieve the HOME environment variable");
        Path::new(&home)
            .join(".config")
            .join("dsc")
            .join(SETTINGS_PREFERENCE_FILE_NAME)
    }
});

/// Defines the full path to the workspace settings file, which is located in
/// the current working directory.
///
/// The pseudo-path for this file is `{CWD}/dsc.settings.json`.
pub static WORKSPACE_SETTINGS_FILE_PATH: LazyLock<PathBuf> = LazyLock::new(|| {
    Path::new(
        &std::env::current_dir()
            .expect("Couldn't retrieve the current working directory"),
    )
    .join(SETTINGS_PREFERENCE_FILE_NAME)
});

/// Helper function for retrieving the platform-specific folder path for
/// machine-level settings files.
///
/// Both the policy and machine scope settings files are located in the same
/// folder. This helper just reduces the duplication for determining the folder
/// path.
///
/// # Returns
///
/// A [`PathBuf`] representing the platform-specific folder path for
/// machine-level settings files.
fn machine_folder_path() -> PathBuf {
    #[cfg(target_os = "windows")]
    {
        let program_data = std::env::var_os("ProgramData")
            .expect("Couldn't retrieve the ProgramData environment variable");
        Path::new(&program_data).join("dsc")
    }
    #[cfg(target_os = "macos")]
    {
        Path::new("/Library").join("Application Support").join("dsc")
    }
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        Path::new("/etc").join("dsc")
    }
}
