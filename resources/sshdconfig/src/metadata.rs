// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

// keywords that can have multiple comma-separated argments per line but cannot be repeated over multiple lines,
// as subsequent entries are ignored, should be represented as arrays
pub const MULTI_ARG_KEYWORDS_COMMA_SEP: [&str; 10] = [
    "authenticationmethods",
    "casignaturealgorithms",
    "ciphers",
    "hostbasedacceptedalgorithms",
    "hostkeyalgorithms",
    "kexalgorithms",
    "macs",
    "permituserenvironment",
    "persourcepenaltyexemptlist",
    "pubkeyacceptedalgorithms"
];

// keywords that can have multiple space-separated argments per line but cannot be repeated over multiple lines,
// as subsequent entries are ignored, should be represented as arrays
pub const MULTI_ARG_KEYWORDS_SPACE_SEP: [&str; 11] = [
    "acceptenv",
    "allowgroups",
    "allowusers",
    "authorizedkeysfile",
    "channeltimeout",
    "denygroups",
    "denyusers",
    "ipqos",
    "permitlisten",
    "permitopen",
    "persourcepenalties",
];

// keywords that can be repeated over multiple lines and should be represented as arrays.
// note that some keywords can be both multi-arg space-separated and repeatable
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
