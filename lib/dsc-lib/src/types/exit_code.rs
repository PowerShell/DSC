// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use std::{borrow::Borrow, fmt::Display, ops::Deref, str::FromStr};

use serde::{Deserialize, Serialize};

use crate::dscerror::DscError;

/// Defines a program exit code as a 32-bit integer ([`i32`]).
///
/// DSC uses exit codes to determine whether invoked commands, including resource and extension
/// operations, are successful. DSC treats exit code `0` as successful and all other exit codes
/// as indicating a failure.
#[derive(Debug, Copy, Clone, Hash, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct ExitCode(i32);

impl ExitCode {
    /// Creates an instance of [`ExitCode`] from an [`i32`].
    pub fn new(code: i32) -> Self {
        Self(code)
    }

    /// Parses a string into an [`ExitCode`].
    ///
    /// If the string can be parsed as an [`i32`] and doesn't have a leading plus sign (`+`), the
    /// function returns an [`ExitCode`].
    ///
    /// DSC forbids a leading plus sign for parsing integers into exit codes to support fully
    /// round-tripping the data from string representation to wrapped-integer and back. While Rust
    /// supports parsing a string like `+123` as the [`i32`] value `123`, it serializes that value
    /// to a string as `123`, not `+123`.
    ///
    /// # Errors
    ///
    /// The function raises an error when:
    ///
    /// - The input text has a leading plus sign ([`DscError::InvalidExitCodePlusPrefix`])
    /// - The input text can't be parsed as an [`i32`] ([`DscError::InvalidExitCode`])
    pub fn parse(text: &str) -> Result<ExitCode, DscError> {
        match i32::from_str(text) {
            Ok(code) => {
                // If text parsed as an exit code but has a leading plus sign, reject it. This
                // only affects parsing directly, since DSC only deserializes after validating
                // the data against the JSON Schema, which forbids a leading plus-sign for the
                // exit codes map.
                if text.starts_with("+") {
                    Err(DscError::InvalidExitCodePlusPrefix(text.to_string()))
                } else {
                    Ok(Self(code))
                }
            },
            Err(err) => Err(DscError::InvalidExitCode(text.to_string(), err)),
        }
    }
}

impl AsRef<i32> for ExitCode {
    fn as_ref(&self) -> &i32 {
        &self.0
    }
}

impl Deref for ExitCode {
    type Target = i32;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Borrow<i32> for ExitCode {
    fn borrow(&self) -> &i32 {
        &self.0
    }
}

impl Display for ExitCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl FromStr for ExitCode {
    type Err = DscError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

impl TryFrom<String> for ExitCode {
    type Error = DscError;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::parse(value.as_str())
    }
}

impl TryFrom<&str> for ExitCode {
    type Error = DscError;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::parse(value)
    }
}

impl From<ExitCode> for String {
    fn from(value: ExitCode) -> Self {
        value.to_string()
    }
}

impl From<i32> for ExitCode {
    fn from(value: i32) -> Self {
        Self(value)
    }
}

impl From<ExitCode> for i32 {
    fn from(value: ExitCode) -> Self {
        value.0
    }
}

impl PartialEq for ExitCode {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }
}

impl PartialEq<i32> for ExitCode {
    fn eq(&self, other: &i32) -> bool {
        self.0.eq(other)
    }
}

impl PartialEq<ExitCode> for i32 {
    fn eq(&self, other: &ExitCode) -> bool {
        self.eq(&other.0)
    }
}
