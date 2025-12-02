// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

// note that it is possible for a keyword to be in one, neither, or both of the multi-arg and repeatable lists below.

// keywords that can have multiple comma-separated arguments per line and should be represented as arrays.
pub const MULTI_ARG_KEYWORDS: [&str; 22] = [
    "acceptenv",
    "allowgroups",
    "allowusers",
    "authenticationmethods",
    "authorizedkeysfile",
    "casignaturealgorithms",
    "channeltimeout",
    "ciphers",
    "denygroups",
    "denyusers",
    "hostbasedacceptedalgorithms",
    "hostkeyalgorithms",
    "ipqos",
    "kexalgorithms",
    "macs",
    "permitlisten",
    "permitopen",
    "permituserenvironment",
    "persourcepenalties",
    "persourcepenaltyexemptlist",
    "pubkeyacceptedalgorithms",
    "rekeylimit" // first arg is bytes, second arg (optional) is amount of time
];

// keywords that can be repeated over multiple lines and should be represented as arrays.
pub const REPEATABLE_KEYWORDS: [&str; 12] = [
    "acceptenv",
    "allowgroups",
    "allowusers",
    "denygroups",
    "denyusers",
    "hostkey",
    "include",
    "listenaddress",
    "match",
    "port",
    "setenv",
    "subsystem"
];

#[cfg(windows)]
pub mod windows {
    pub const REGISTRY_PATH: &str = "HKLM\\SOFTWARE\\OpenSSH";
    pub const DEFAULT_SHELL: &str = "DefaultShell";
    pub const DEFAULT_SHELL_CMD_OPTION: &str = "DefaultShellCommandOption";
    pub const DEFAULT_SHELL_ESCAPE_ARGS: &str = "DefaultShellEscapeArguments";
}
