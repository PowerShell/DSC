// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

// the multi-arg comma-separated and space-separated lists are mutually exclusive, but the repeatable list can overlap with either of them.
// the multi-arg lists are maintained for formatting arrays into the correct format when writing back to the config file.

// keywords that can have multiple comma-separated arguments per line and should be represented as arrays.
pub const MULTI_ARG_KEYWORDS_COMMA_SEP: [&str; 11] = [
    "authenticationmethods",
    "casignaturealgorithms",
    "ciphers",
    "hostbasedacceptedalgorithms",
    "hostkeyalgorithms",
    "kexalgorithms",
    "macs",
    "permituserenvironment",
    "persourcepenaltyexemptlist",
    "pubkeyacceptedalgorithms",
    "rekeylimit" // first arg is bytes, second arg (optional) is amount of time
];

// keywords that can have multiple space-separated arguments per line and should be represented as arrays.
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

// keywords that require structured name-value format (e.g., subsystem has a name and a command value).
pub const STRUCTURED_KEYWORDS: [&str; 1] = [
    "subsystem"
];

#[derive(Clone, Copy, Debug)]
pub enum ValueSeparator {
    Comma,
    Space,
}

#[derive(Clone, Debug)]
pub struct KeywordInfo {
    pub name: String,
    pub is_repeatable: bool,
    pub is_structured: bool,
    pub separator: ValueSeparator,
}

impl KeywordInfo {
    /// Create a new `KeywordInfo` from a keyword string.
    pub fn from_keyword(keyword: &str) -> Self {
        let lowercase_key = keyword.to_lowercase();
        let is_repeatable = REPEATABLE_KEYWORDS.contains(&lowercase_key.as_str());
        let is_structured = STRUCTURED_KEYWORDS.contains(&lowercase_key.as_str());
        let separator = if MULTI_ARG_KEYWORDS_COMMA_SEP.contains(&lowercase_key.as_str()) {
            ValueSeparator::Comma
        } else {
            ValueSeparator::Space
        };

        Self {
            name: lowercase_key,
            is_repeatable,
            is_structured,
            separator,
        }
    }

    /// Check if this keyword allows operator syntax (+, -, ^).
    pub fn allows_operator(&self) -> bool {
        !self.is_structured
    }

    /// Check if this keyword requires structured name-value format.
    pub fn requires_structured_format(&self) -> bool {
        self.is_structured
    }

    /// Check if this keyword can have multiple arguments.
    pub fn is_multi_arg(&self) -> bool {
        self.is_repeatable ||
        MULTI_ARG_KEYWORDS_COMMA_SEP.contains(&self.name.as_str()) ||
        MULTI_ARG_KEYWORDS_SPACE_SEP.contains(&self.name.as_str())
    }
}

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
