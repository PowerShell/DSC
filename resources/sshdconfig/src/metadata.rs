// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

// keywords that can have multiple arguments per line but cannot be repeated over multiple lines,
// as subsequent entries are ignored, should be represented as arrays
pub const MULTI_ARG_KEYWORDS: [&str; 17] = [
    "authenticationmethods",
    "authorizedkeysfile",
    "casignaturealgorithms",
    "channeltimeout",
    "ciphers",
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
// note that some keywords can be both multi-arg and repeatable, in which case they only need to be listed here
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
