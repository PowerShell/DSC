// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

pub const SSHD_CONFIG_HEADER: &str = "# This file is managed by the Microsoft.OpenSSH.SSHD/sshd_config DSC Resource";
pub const SSHD_CONFIG_HEADER_VERSION: &str = concat!("# The Microsoft.OpenSSH.SSHD/sshd_config DSC Resource version is ", env!("CARGO_PKG_VERSION"));
pub const SSHD_CONFIG_HEADER_WARNING: &str = "# Please do not modify manually, as any changes may be overwritten";
pub const SSHD_CONFIG_DEFAULT_PATH_UNIX: &str = "/etc/ssh/sshd_config";
// For Windows, full path is constructed at runtime using ProgramData environment variable
pub const SSHD_CONFIG_DEFAULT_PATH_WINDOWS: &str = "\\ssh\\sshd_config";

#[cfg(windows)]
pub mod windows {
    pub const REGISTRY_PATH: &str = "HKLM\\SOFTWARE\\OpenSSH";
    pub const DEFAULT_SHELL: &str = "DefaultShell";
    pub const DEFAULT_SHELL_CMD_OPTION: &str = "DefaultShellCommandOption";
    pub const DEFAULT_SHELL_ESCAPE_ARGS: &str = "DefaultShellEscapeArguments";
}
