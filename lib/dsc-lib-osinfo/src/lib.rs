// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::string::ToString;
use version_compare::Cmp;

/// Returns information about the operating system.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct OsInfo {
    family: Family,
    /// Defines the version of the operating system as a string.
    version: String,
    /// Defines the Windows operating system edition, like `Windows 11` or `Windows Server 2016`.
    #[serde(skip_serializing_if = "Option::is_none")]
    edition: Option<String>,
    /// Defines the codename for the operating system as returned from `lsb_release --codename`.
    #[serde(skip_serializing_if = "Option::is_none")]
    codename: Option<String>,
    bitness: Option<i32>,
    /// Defines the processor architecture as reported by `uname -m` on the operating system.
    #[serde(skip_serializing_if = "Option::is_none")]
    architecture: Option<String>,
    /// Support returning generated name for the OSInfo instance for export.
    #[serde(rename = "_name", skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    /// Indicates whether the resource is in the desired state. Only emitted by the test operation.
    #[serde(rename = "_inDesiredState", skip_serializing_if = "Option::is_none")]
    in_desired_state: Option<bool>,
}

/// Desired state for the test operation. All fields are optional.
/// The `version` field may include a comparison operator prefix: `>`, `<`, `=`, `>=`, or `<=`.
#[derive(Debug, Default, Deserialize)]
pub struct OsTestInput {
    pub family: Option<Family>,
    pub version: Option<String>,
    pub edition: Option<String>,
    pub codename: Option<String>,
    pub bitness: Option<i32>,
    pub architecture: Option<String>,
    #[serde(rename = "_name")]
    pub name: Option<String>,
}

/// Defines whether the operating system is Linux, macOS, or Windows.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Family {
    Linux,
    #[serde(rename = "macOS")]
    MacOS,
    Windows,
}

impl Display for Family {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Family::Linux => write!(f, "Linux"),
            Family::MacOS => write!(f, "macOS"),
            Family::Windows => write!(f, "Windows"),
        }
    }
}

impl OsInfo {
    pub fn new(include_name: bool) -> Self {
        let os_info = os_info::get();
        let edition = os_info.edition().map(ToString::to_string);
        let codename = os_info.codename().map(ToString::to_string);
        let architecture = os_info.architecture().map(ToString::to_string);
        let family = match os_info.os_type() {
            os_info::Type::Macos => Family::MacOS,
            os_info::Type::Windows => Family::Windows,
            _ => Family::Linux,
        };
        let bits = match os_info.bitness() {
            os_info::Bitness::X32 => Some(32),
            os_info::Bitness::X64 => Some(64),
            _ => None,
        };
        let version = os_info.version().to_string();
        let name = if include_name {
            Some(
                match &architecture {
                    Some(arch) => format!("{family} {version} {arch}"),
                    None => format!("{family:?} {version}"),
                }
            )
        } else {
            None
        };
        Self {
            family,
            version,
            edition,
            codename,
            bitness: bits,
            architecture,
            name,
            in_desired_state: None,
        }
    }
}

/// Parse the optional comparison operator prefix from a version constraint string.
///
/// Returns a tuple of `(operator, version_str)` where `operator` is one of
/// `">"`, `"<"`, `"="`, `">="`, `"<="`, and `version_str` is the version
/// string with whitespace trimmed.  When no operator prefix is present, the
/// operator defaults to `"="` (exact match).
///
/// An operator is only recognised when the remainder after stripping it starts
/// with an ASCII digit.  Strings like `">> 1.0"` (double operator) are
/// therefore treated as literal exact-match strings rather than producing a
/// misleading parsed operator.
fn parse_version_constraint(constraint: &str) -> (&str, &str) {
    let constraint = constraint.trim();
    // Check two-character operators before single-character ones.
    if let Some(rest) = constraint.strip_prefix(">=") {
        let rest = rest.trim();
        if rest.starts_with(|c: char| c.is_ascii_digit()) {
            return (">=", rest);
        }
    } else if let Some(rest) = constraint.strip_prefix("<=") {
        let rest = rest.trim();
        if rest.starts_with(|c: char| c.is_ascii_digit()) {
            return ("<=", rest);
        }
    } else if let Some(rest) = constraint.strip_prefix('>') {
        let rest = rest.trim();
        if rest.starts_with(|c: char| c.is_ascii_digit()) {
            return (">", rest);
        }
    } else if let Some(rest) = constraint.strip_prefix('<') {
        let rest = rest.trim();
        if rest.starts_with(|c: char| c.is_ascii_digit()) {
            return ("<", rest);
        }
    } else if let Some(rest) = constraint.strip_prefix('=') {
        let rest = rest.trim();
        if rest.starts_with(|c: char| c.is_ascii_digit()) {
            return ("=", rest);
        }
    }
    // No recognised operator, or the remainder does not look like a version.
    // Treat the entire string as a literal exact-match value.
    ("=", constraint)
}

/// Returns `true` when `actual` satisfies the `constraint`.
///
/// `constraint` may be a plain version string (exact match) or a version
/// string prefixed with one of the comparison operators `>`, `<`, `=`, `>=`,
/// or `<=`.  Operator and version string may be separated by optional
/// whitespace.  Comparison is performed by the `version_compare` crate, which
/// handles version strings that are not strict semver (e.g. `"22.04"`,
/// `"10.15.7"`, `"11"`).  Returns `false` when either version string cannot
/// be parsed.
fn version_matches(constraint: &str, actual: &str) -> bool {
    let (operator, desired_ver) = parse_version_constraint(constraint);

    if operator == "=" {
        return desired_ver == actual;
    }

    match version_compare::compare(actual, desired_ver) {
        Ok(cmp) => match operator {
            ">" => cmp == Cmp::Gt,
            "<" => cmp == Cmp::Lt,
            ">=" => matches!(cmp, Cmp::Gt | Cmp::Eq),
            "<=" => matches!(cmp, Cmp::Lt | Cmp::Eq),
            _ => false,
        },
        Err(()) => false,
    }
}

/// Perform the test operation against the current OS state.
///
/// Parses `input_json` as the desired state (`OsTestInput`), retrieves the
/// actual OS information, and evaluates each specified field.  For `version`,
/// an optional comparison operator prefix is supported (see
/// `version_matches`).  Returns the actual `OsInfo` with `_inDesiredState`
/// set to indicate whether all specified fields are satisfied.
///
/// # Errors
///
/// Returns an error string when `input_json` cannot be parsed as valid JSON.
pub fn perform_test(input_json: &str) -> Result<OsInfo, String> {
    let desired: OsTestInput = serde_json::from_str(input_json)
        .map_err(|e| format!("Failed to parse test input as JSON: {e}"))?;

    // name is ignored for test since it's only generated for export and not a property of the actual OS state.
    let actual = OsInfo::new(false);

    let mut in_desired_state = true;

    if let Some(desired_family) = desired.family
        && desired_family != actual.family {
            in_desired_state = false;
        }

    if let Some(ref desired_version) = desired.version
        && !version_matches(desired_version, &actual.version) {
            in_desired_state = false;
        }

    if let Some(ref desired_edition) = desired.edition
        && actual.edition.as_deref() != Some(desired_edition.as_str()) {
            in_desired_state = false;
        }

    if let Some(ref desired_codename) = desired.codename
        && actual.codename.as_deref() != Some(desired_codename.as_str()) {
            in_desired_state = false;
        }

    if let Some(desired_bitness) = desired.bitness
        && actual.bitness != Some(desired_bitness) {
            in_desired_state = false;
        }

    if let Some(ref desired_architecture) = desired.architecture
        && actual.architecture.as_deref() != Some(desired_architecture.as_str()) {
            in_desired_state = false;
        }

    Ok(OsInfo { in_desired_state: Some(in_desired_state), ..actual })
}
