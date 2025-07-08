// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use clap::ValueEnum;

// TODO: ensure lists are complete

// keywords that can be repeated over multiple lines and should be represented as arrays
pub const REPEATABLE_KEYWORDS: [&str; 6] = [
    "hostkey",
    "include",
    "listenaddress",
    "port",
    "setenv",
    "subsystem"
];

#[derive(Clone, Debug, Eq, PartialEq, ValueEnum)]
pub enum RepeatableKeyword {
    HostKey,
    Include,
    ListenAddress,
    Port,
    SetEnv,
    Subsystem,
}

// keywords that can have multiple argments per line and should be represented as arrays
// but cannot be repeated over multiple lines, as subsequent entries are ignored
pub const MULTI_ARG_KEYWORDS: [&str; 7] = [
    "casignaturealgorithms",
    "ciphers",
    "hostbasedacceptedalgorithms",
    "hostkeyalgorithms",
    "kexalgorithms",
    "macs",
    "pubkeyacceptedalgorithms"
];

pub const REGISTRY_PATH: &str = "HKLM\\SOFTWARE\\OpenSSH";
pub const DEFAULT_SHELL: &str = "DefaultShell";
pub const DEFAULT_SHELL_CMD_OPTION: &str = "DefaultShellCommandOption";
pub const DEFAULT_SHELL_ESCAPE_ARGS: &str = "DefaultShellEscapeArguments";
