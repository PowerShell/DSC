// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use std::{fmt::Display, ops::Deref, str::FromStr};

use crate::{dscerror::DscError, schemas::dsc_repo::DscRepoSchema};
use rust_i18n::t;
use schemars::{json_schema, JsonSchema};
use serde::{Deserialize, Serialize};

/// Defines a semantic version for use with DSC.
///
/// This type is a wrapper around the [`semver::Version`] type that enables DSC to provide a more
/// complete JSON Schema for semantic versions.
///
/// A semantic version adheres to the specification defined at [semver.org][01].
///
/// ## Syntax
///
/// A semantic version is composed of the mandatory major, minor, and patch version segments.
/// Optionally, a semantic version may define the prerelease and build metadata segments. The
/// string defining a semantic version must not include any spacing characters.
///
/// The major, minor, and patch version segments must parse as zero or a positive integer ([`u64`]).
/// These segments must not contain leading zeroes. The string `01.020.0034` isn't a valid semantic
/// version and should be written as `1.20.34` instead.
///
/// The prerelease segment must be prefixed with a single hyphen (`-`) and define one or more
/// identifiers. The build metadata segment must be prefixed with a single plus sign (`+`) and
/// define one or more identifiers.
///
/// An identifier for prerelease and build metadata segments must be a string consisting of only
/// ASCII alphanumeric characters underscores (regex `\w`). Identifiers in prerelease and build
/// metadata segments must be separated by a single period (`.`), like `rc.1` or
/// `dev.mac_os.sha123`.
///
/// ### Syntax parsing examples
///
/// Stable versions are defined as `<major>.<minor>.<patch>`:
///
/// ```rust
/// # use dsc_lib::types::SemanticVersion;
/// let v1_2_3 = SemanticVersion::parse("1.2.3").unwrap();
/// assert!(v1_2_3.major == 1);
/// assert!(v1_2_3.minor == 2);
/// assert!(v1_2_3.patch == 3);
/// ```
/// 
/// Omitting a version segment, specifying a non-digit for a version segment, and specifying a
/// leading zero for a version segment all cause parsing errors:
///
/// ```rust
/// # use dsc_lib::types::SemanticVersion;
/// assert!(SemanticVersion::parse("1.2").is_err());
/// assert!(SemanticVersion::parse("1.x.3").is_err());
/// assert!(SemanticVersion::parse("1.2.03").is_err());
/// ```
///
/// Prerelease segments immediately follow the patch version segment and are prefixed with a hyphen
/// (`-`):
///
/// ```rust
/// # use dsc_lib::types::SemanticVersion;
/// let v1_2_3_rc = SemanticVersion::parse("1.2.3-rc").unwrap();
/// assert!(v1_2_3_rc.pre.as_str() == "rc");
/// ```
///
/// Build metadata immediately follows either the patch version segment or prerelease segment and
/// are prefixed with a plus sign (`+`):
///
/// ```rust
/// # use dsc_lib::types::SemanticVersion;
/// let v1_2_3_ci = SemanticVersion::parse("1.2.3+ci").unwrap();
/// let v1_2_3_rc_ci = SemanticVersion::parse("1.2.3-rc+ci").unwrap();
/// assert!(v1_2_3_ci.build.as_str() == "ci");
/// assert!(v1_2_3_rc_ci.build.as_str() == "ci");
/// ```
///
/// Putting build metadata before prerelease causes the intended prerelease segment to parse as
/// part of the build metadata:
///
/// ```rust
/// # use dsc_lib::types::SemanticVersion;
/// let build_first = SemanticVersion::parse("1.2.3+ci-rc").unwrap();
/// assert!(build_first.build.as_str() == "ci-rc");
/// assert!(build_first.pre.as_str() == "");
///
/// let pre_first = SemanticVersion::parse("1.2.3-rc+ci").unwrap();
/// assert!(pre_first.build.as_str() == "ci");
/// assert!(pre_first.pre.as_str() == "rc");
/// ```
///
/// Build metadata and prerelease segments may contain multiple components with a separating
/// period:
///
/// ```rust
/// # use dsc_lib::types::SemanticVersion;
/// assert!(SemanticVersion::parse("1.2.3-rc.1").is_ok());
/// ```
///
/// Build metadata and prerelease segments and subsegments may start with a hyphen:
///
/// ```rust
/// # use dsc_lib::types::SemanticVersion;
/// assert!(SemanticVersion::parse("1.2.3--rc.-1").is_ok());
/// assert!(SemanticVersion::parse("1.2.3+-ci.-1").is_ok());
/// ```
///
/// Digit-only identifiers for prerelease segments must not have leading zeroes but may consist of
/// a single zero:
///
/// ```rust
/// # use dsc_lib::types::SemanticVersion;
/// assert!(SemanticVersion::parse("1.2.3-rc.0").is_ok());
/// assert!(SemanticVersion::parse("1.2.3-rc.01").is_err());
/// assert!(SemanticVersion::parse("1.2.3-0").is_ok());
/// assert!(SemanticVersion::parse("1.2.3-01").is_err());
/// assert!(SemanticVersion::parse("1.2.3-rc.0").is_ok());
/// assert!(SemanticVersion::parse("1.2.3-rc.000").is_err());
/// ```
/// 
/// Digit-only identifiers for build metadata segments can have leading zeroes and consist of
/// multiple zeroes:
/// 
/// ```rust
/// # use dsc_lib::types::SemanticVersion;
/// assert!(SemanticVersion::parse("1.2.3+ci.0").is_ok());
/// assert!(SemanticVersion::parse("1.2.3+ci.01").is_ok());
/// assert!(SemanticVersion::parse("1.2.3+0").is_ok());
/// assert!(SemanticVersion::parse("1.2.3+01").is_ok());
/// assert!(SemanticVersion::parse("1.2.3+ci.0").is_ok());
/// assert!(SemanticVersion::parse("1.2.3+ci.000").is_ok());
/// ```
///
/// Specifying any character other than a hyphen (`-`), digit (`0-9`), ASCII alphabetic (`a-z` and
/// `A-Z`), or period (`.`) for a prerelease or build metadata segment causes a parsing error:
///
/// ```rust
/// # use dsc_lib::types::SemanticVersion;
/// assert!(SemanticVersion::parse("1.2.3-rc@4").is_err());
/// assert!(SemanticVersion::parse("1.2.3+ci!4").is_err());
/// ```
///
/// ## Semantic version ordering
///
/// The comparison for semantic versions is performed segment by segment where the segment for the
/// left hand side of the comparison may be equal to, greater than, or less than the segment for
/// the right hand side of the comparison. The comparison logic follows these steps:
///
/// 1. If the major version segments of the semantic versions are unequal, the version with a
///    higher major version segment is greater. If the major version segments are equal, compare
///    the minor version segments.
/// 1. If the minor version segments of the semantic versions are unequal, the version with a
///    higher minor version segment is greater. If the minor version segments are equal, compare
///    the patch version segments.
/// 1. If the patch version segments of the semantic versions are unequal, the version with a
///    higher patch version segment is greater. If the patch version segments are equal, compare
///    the prerelease segments.
/// 1. If only one version defines a prerelease segment then the stable version, which doesn't
///    define a prerelease segment, is greater. If both versions define a prerelease segment,
///    compare the identifiers for each prerelease segment in their defined order:
///
///    - If both identifiers contain only digits then the identifiers are compared numerically.
///    - If both identifiers contain non-digit characters then the identifiers are compared
///      in ASCII sort order (hyphen < digits < uppercase letters < lowercase letters).
///    - If the identifiers are identical for both versions, continue to the next identifier, if
///      any.
///    - If only one version defines the next identifier, that version is greater than the other
///      version. For example, `1.2.3-rc.1.a` is greater than `1.2.3-rc.1`.
///
///    If the prerelease segments are equal, compare the build metadata segments.
/// 1. If only one version defines a build metadata segment then the version with build metadata
///    is greater. If both versions define a build metadata segment, compare the identifiers for
///    each segment in their defined order. The comparison logic for build metadata identifiers is
///    is the same as for prerelease identifiers.
/// 1. If all segments are equal then the versions are equal.
///
/// Note that build metadata is _always_ ignored when matching against a [`SemanticVersionReq`].
/// This comparison is _only_ for distinguishing precedence between versions to find the latest
/// version. This behavior can be surprising for end-users, since versions with build metadata
/// are typically seen with development builds, not release builds. Prefer omitting build metadata
/// when defining semantic versions for DSC to make a more consistent experience for users.
///
/// ### Ordering examples
///
/// ```rust
/// use dsc_lib::types::SemanticVersion;
///
/// let v1_0_0: SemanticVersion = "1.0.0".parse().unwrap();
/// let v2_0_0: SemanticVersion = "2.0.0".parse().unwrap();
/// let v1_2_3: SemanticVersion = "1.2.3".parse().unwrap();
/// let v1_2_3_pre: SemanticVersion = "1.2.3-rc.1".parse().unwrap();
/// let v1_2_3_build: SemanticVersion = "1.2.3+ci.1".parse().unwrap();
/// let v1_2_3_pre_build: SemanticVersion = "1.2.3-rc.1+ci.1".parse().unwrap();
///
/// // Comparisons of stable versions work as expected
/// assert!(v1_0_0 < v1_2_3);
/// assert!(v2_0_0 > v1_2_3);
/// // Stable versions is always greater than prerelease for same version
/// assert!(v1_0_0 < v1_2_3_pre);
/// assert!(v1_2_3 > v1_2_3_pre);
/// // Version with build metadata is greater than same version
/// assert!(v1_2_3_build > v1_2_3);
/// assert!(v1_2_3_pre_build > v1_2_3_pre);
/// // Build metadata is ignored when versions aren't the same
/// assert!(v2_0_0 > v1_2_3_build);
///
/// let rc:   SemanticVersion = "1.2.3-rc".parse().unwrap();
/// let rc_1: SemanticVersion = "1.2.3-rc.1".parse().unwrap();
/// let rc_1_2: SemanticVersion = "1.2.3-rc.1.2".parse().unwrap();
/// // When the first identifier is identical, the version with an extra
/// // identifier is greater
/// assert!(rc < rc_1);
/// assert!(rc_1 < rc_1_2);
///
/// // To correct sort prerelease and build versions, make sure to separate
/// // the alpha segment like `rc` or `ci`from the numeric. Otherwise, the
/// // ordering may be unexpected, like `rc11` < `rc2`
/// let rc11:  SemanticVersion = "1.2.3-rc11".parse().unwrap();
/// let rc2:   SemanticVersion = "1.2.3-rc2".parse().unwrap();
/// let rc_11: SemanticVersion = "1.2.3-rc.11".parse().unwrap();
/// let rc_2:  SemanticVersion = "1.2.3-rc.2".parse().unwrap();
/// assert!(rc2 > rc11);
/// assert!(rc_11 > rc_2);
///
/// // For identifiers, hyphen < digit < uppercase alpha < lowercase alpha
/// // Showing for build but ordering applies to prerelease identifiers too
/// let middle_hyphen: SemanticVersion = "1.2.3+a-a".parse().unwrap();
/// let middle_digit:  SemanticVersion = "1.2.3+a0a".parse().unwrap();
/// let middle_upper:  SemanticVersion = "1.2.3+aAa".parse().unwrap();
/// let middle_lower:  SemanticVersion = "1.2.3+aaa".parse().unwrap();
/// assert!(middle_hyphen < middle_digit);
/// assert!(middle_digit < middle_upper);
/// assert!(middle_upper < middle_lower);
/// ```
/// 
/// # Determining latest version
/// 
/// DSC uses the default ordering for semantic versions where:
/// 
/// - A higher version supercedes a lower version, regardless of prerelease and build metadata.
/// - A stable version supercedes the same version with a prerelease segment.
/// - A stable version with build metadata supercedes the same version without build metadata.
/// - Prerelease and build metadata segments are compared lexicographically.
/// 
/// Consider the following example:
/// 
/// ```rust
/// # use dsc_lib::types::SemanticVersion;
/// let v1_0_0 = SemanticVersion::parse("1.0.0").unwrap();
/// let v1_2_3 = SemanticVersion::parse("1.2.3").unwrap();
/// let v2_0_0 = SemanticVersion::parse("2.0.0").unwrap();
/// let v1_2_3_rc_1 = SemanticVersion::parse("1.2.3-rc.1").unwrap();
/// let v1_2_3_ci_1 = SemanticVersion::parse("1.2.3+ci.1").unwrap();
///
/// let mut versions = vec![
///     v1_0_0.clone(),
///     v1_2_3.clone(),
///     v2_0_0.clone(),
///     v1_2_3_rc_1.clone(),
///     v1_2_3_ci_1.clone()
/// ];
/// versions.sort();
///
/// assert_eq!(
///     versions,
///     vec![
///         v1_0_0,
///         v1_2_3_rc_1,
///         v1_2_3,
///         v1_2_3_ci_1,
///         v2_0_0
///     ]
/// );
/// ```
/// 
/// When the versions are sorted, the latest version is `2.0.0`. When considering the versions
/// `1.2.3`, `1.2.3-rc.1`, and `1.2.3+ci.1`, they sort as `1.2.3+ci.1 > 1.2.3 > 1.2.3-rc.1`.
/// 
/// Versions sorting with build metadata as later than typical stable versions may be surprising to
/// end users. When publishing DSC resources and extensions, authors should always set the version
/// in a manifest to _exclude_ build metadata.
/// 
/// # Matching version requirements
/// 
/// DSC uses the [`SemanticVersionReq`] type for defining version ranges. This enables users to
/// pin to specific versions or a range of supported versions. For more information, see
/// [`SemanticVersionReq`].
/// 
/// [01]: https://semver.org
/// [`SemanticVersionReq`]: crate::types::SemanticVersionReq
#[derive(Debug, Clone, Hash, Eq, Serialize, Deserialize, DscRepoSchema)]
#[dsc_repo_schema(base_name = "semver", folder_path = "definitions")]
pub struct SemanticVersion(semver::Version);

impl SemanticVersion {
    /// Create an instance of [`SemanticVersion`] with empty prerelease and build segments.
    ///
    /// # Examples
    ///
    /// ```rust
    /// pub use dsc_lib::types::SemanticVersion;
    ///
    /// let version = &SemanticVersion::new(1, 2, 3);
    ///
    /// assert!(
    ///     semver::VersionReq::parse(">1.0").unwrap().matches(version)
    /// );
    /// ```
    pub fn new(major: u64, minor: u64, patch: u64) -> Self {
        Self(semver::Version::new(major, minor, patch))
    }

    /// Create an instance of [`SemanticVersion`] from a string representation.
    ///
    /// # Errors
    ///
    /// Parsing fails when the input string isn't a valid semantic version. Common parse errors
    /// include:
    ///
    /// - Not specifying major, minor, and patch segments, like `1` or `1.0` instead of `1.0.0`.
    /// - Specifying a leading zero before a non-zero digit in a version segment, like `01.02.03`
    ///   instead of `1.2.3`.
    /// - Specifying a non-digit character in a major, minor, or patch version segment, like
    ///   `1a.2.3`, `1.2b.3`, or `1.2.c3`.
    /// - Specifying a hyphen after the version without a prelease segment, like `1.2.3-`.
    /// - Specifying a plus sign after the version without a build metadata segment, like `1.2.3+`.
    /// - Invalid characters in prerelease or build metadata segments, which only allow the
    ///   characters `a-z`, `a-Z`, `0-9`, `-`, and `.`, such as `1.2.3-rc_1` or `1.2.3+ci@sha`.
    pub fn parse(value: &str) -> Result<Self, DscError> {
        match semver::Version::parse(value) {
            Ok(v) => Ok(Self(v)),
            Err(e) => Err(DscError::SemVer(e)),
        }
    }

    /// Defines the validating regular expression for semantic versions.
    ///
    /// This regular expression is adapted from the officially recommended pattern on [semver.org]
    /// and is used for the `pattern` keyword in the JSON Schema for the [`SemanticVersion`] type.
    ///
    /// The pattern isn't used for validating an instance during or after deserialization. Instead,
    /// it provides author-time feedback to manifest maintainers so they can avoid runtime failures.
    ///
    /// During deserialization, the library first tries to parse the string as a semantic version.
    /// If the value parses successfully, it's deserialized as a [`SemanticVersion`] instance.
    /// Otherwise, deserialization fails.
    ///
    /// [semver.org]: https://semver.org/#is-there-a-suggested-regular-expression-regex-to-check-a-semver-string
    pub const VALIDATING_PATTERN: &str = const_str::concat!(
        "^",                                               // Anchor pattern to start of string.
        SemanticVersion::CAPTURING_VERSION_MAJOR_PATTERN,  // Must include major version segment,
        r"\.",                                             // then a period (`.`),
        SemanticVersion::CAPTURING_VERSION_MINOR_PATTERN,  // then the minor version segment,
        r"\.",                                             // then a period (`.`),
        SemanticVersion::CAPTURING_VERSION_PATCH_PATTERN,  // then the patch version segment.
        "(?:",                                             // Open non-capturing group for prerelease segment.
        "-",                                               // Prerelease follows patch version with a hyphen
        SemanticVersion::CAPTURING_PRERELEASE_PATTERN,     // then the actual segment.
        ")",                                               // Close the non-capturing group for prerelease.
        "?",                                               // Mark the prerelease segment as optional.
        "(?:",                                             // Open non-capturing group for build metadata segment.
        r"\+",                                             // Build follows patch or prerelease with a plus
        SemanticVersion::CAPTURING_BUILD_METADATA_PATTERN, // then the actual segment.
        ")",                                               // Close the non-capturing group for build metadata.
        "?",                                               // Mark the build metadata segment as optional.
        "$"                                                // Anchor pattern to end of string.
    );

    pub(crate) const VERSION_SEGMENT_PATTERN: &str = const_str::concat!(
        "(?:",          // Open non-capturing group for version segment
        "0",            // segments can be either zero
        "|",            // or
        r"[1-9]\d*",    // any integer greater than zero
        ")",            // Close non-capturing group for version segment
    );
    pub(crate) const CAPTURING_VERSION_MAJOR_PATTERN: &str = const_str::concat!(
        "(?<major>",                              // Open the named capture group
        SemanticVersion::VERSION_SEGMENT_PATTERN, // Capture the version segment
        ")"                                       // Close the named capture group
    );
    pub(crate) const CAPTURING_VERSION_MINOR_PATTERN: &str = const_str::concat!(
        "(?<minor>",                              // Open the named capture group
        SemanticVersion::VERSION_SEGMENT_PATTERN, // Capture the version segment
        ")"                                       // Close the named capture group
    );
    pub(crate) const CAPTURING_VERSION_PATCH_PATTERN: &str = const_str::concat!(
        "(?<patch>",                              // Open the named capture group
        SemanticVersion::VERSION_SEGMENT_PATTERN, // Capture the version segment
        ")"                                       // Close the named capture group
    );
    pub(crate) const PRERELEASE_SUBSEGMENT_PATTERN: &str = const_str::concat!(
        "(?:",                                    // Open non-capturing group to avoid cluttering results.
        SemanticVersion::VERSION_SEGMENT_PATTERN, // Subsegment can either be a version segment
        "|",                                      // or
        r"\d*[a-zA-Z-]",                          // any number of digits followed by a letter or hyphen, then
        "[0-9a-zA-Z-]*",                          // any number of digits, letters, or hyphens.
        ")"                                       // Close the non-capturing group.
    );
    pub(crate) const PRERELEASE_SEGMENT_PATTERN: &str = const_str::concat!(
        SemanticVersion::PRERELEASE_SUBSEGMENT_PATTERN, // Start with a valid prerelease subsegment
        "(?:",                                          // Open a non-capturing group to avoid cluttering.
        r"\.",                                          // First character after prior subsegment must be a `.`,
        SemanticVersion::PRERELEASE_SUBSEGMENT_PATTERN, // followed by another valid prerelease segment.
        ")",                                            // Close the non-capturing group for extra subsegments.
        "*"                                             // Match additional subsegments zero or more times.
    );
    pub(crate) const CAPTURING_PRERELEASE_PATTERN: &str = const_str::concat!(
        "(?<prerelease>",                            // Open named capture group.
        SemanticVersion::PRERELEASE_SEGMENT_PATTERN, // Capture the segment.
        ")"                                          // Close the named capture group.
    );
    /// A subsegment of build metadata consists of one or more digits, letters, and hyphens.
    pub(crate) const BUILD_METADATA_SUBSEGMENT_PATTERN: &str = "[0-9a-zA-Z-]+";
    pub(crate) const BUILD_METADATA_SEGMENT_PATTERN: &str = const_str::concat!(
        SemanticVersion::BUILD_METADATA_SUBSEGMENT_PATTERN, // Start with a valid build metadata subsegment
        "(?:",                                              // Open a non-capturing group to avoid cluttering.
        r"\.",                                              // First character after prior subsegment must be a `.`,
        SemanticVersion::BUILD_METADATA_SUBSEGMENT_PATTERN, // followed by another valid subsegment.
        ")",                                                // Close the non-capturing group for extra subsegments.
        "*"                                                 // Match additional subsegments zero or more times.
    );
    pub(crate) const CAPTURING_BUILD_METADATA_PATTERN: &str = const_str::concat!(
        "(?<buildmetadata>",                             // Open named capture group.
        SemanticVersion::BUILD_METADATA_SEGMENT_PATTERN, // Capture the segment.
        ")"                                              // Close the named capture group.
    );
}

impl JsonSchema for SemanticVersion {
    fn schema_name() -> std::borrow::Cow<'static, str> {
        Self::default_schema_id_uri().into()
    }
    fn json_schema(_: &mut schemars::SchemaGenerator) -> schemars::Schema {
        json_schema!({
            "title": t!("schemas.definitions.semver.title"),
            "description": t!("schemas.definitions.semver.description"),
            "markdownDescription": t!("schemas.definitions.semver.markdownDescription"),
            "type": "string",
            "pattern": Self::VALIDATING_PATTERN,
            "patternErrorMessage": t!("schemas.definitions.semver.patternErrorMessage"),
        })
    }
}

impl Default for SemanticVersion {
    fn default() -> Self {
        Self(semver::Version::new(0, 0, 0))
    }
}

impl Display for SemanticVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

// Infallible conversions
impl From<semver::Version> for SemanticVersion {
    fn from(value: semver::Version) -> Self {
        Self(value)
    }
}

impl From<SemanticVersion> for semver::Version {
    fn from(value: SemanticVersion) -> Self {
        value.0
    }
}

impl From<&semver::Version> for SemanticVersion {
    fn from(value: &semver::Version) -> Self {
        Self(value.clone())
    }
}

impl From<&SemanticVersion> for semver::Version {
    fn from(value: &SemanticVersion) -> Self {
        value.0.clone()
    }
}

impl From<SemanticVersion> for String {
    fn from(value: SemanticVersion) -> Self {
        value.to_string()
    }
}

// Fallible conversions
impl FromStr for SemanticVersion {
    type Err = DscError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        SemanticVersion::parse(s)
    }
}

impl TryFrom<String> for SemanticVersion {
    type Error = DscError;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        match semver::Version::parse(value.as_str()) {
            Ok(v) => Ok(Self(v)),
            Err(e) => Err(DscError::SemVer(e)),
        }
    }
}

impl TryFrom<&str> for SemanticVersion {
    type Error = DscError;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        SemanticVersion::from_str(value)
    }
}

// Referencing and dereferencing
impl AsRef<semver::Version> for SemanticVersion {
    fn as_ref(&self) -> &semver::Version {
        &self.0
    }
}

impl Deref for SemanticVersion {
    type Target = semver::Version;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

// Comparison traits
impl PartialEq for SemanticVersion {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl PartialEq<semver::Version> for SemanticVersion {
    fn eq(&self, other: &semver::Version) -> bool {
        &self.0 == other
    }
}

impl PartialEq<SemanticVersion> for semver::Version {
    fn eq(&self, other: &SemanticVersion) -> bool {
        self == &other.0
    }
}

impl PartialEq<String> for SemanticVersion {
    fn eq(&self, other: &String) -> bool {
        match Self::parse(other.as_str()) {
            Ok(other_version) => self.eq(&other_version),
            Err(_) => false,
        }
    }
}

impl PartialEq<SemanticVersion> for String {
    fn eq(&self, other: &SemanticVersion) -> bool {
        match SemanticVersion::parse(self.as_str()) {
            Ok(version) => version.eq(other),
            Err(_) => false,
        }
    }
}

impl PartialEq<str> for SemanticVersion {
    fn eq(&self, other: &str) -> bool {
        match Self::parse(other) {
            Ok(other_version) => self.eq(&other_version),
            Err(_) => false,
        }
    }
}

impl PartialEq<SemanticVersion> for str {
    fn eq(&self, other: &SemanticVersion) -> bool {
        match SemanticVersion::parse(self) {
            Ok(version) => version.eq(other),
            Err(_) => false,
        }
    }
}

impl PartialEq<&str> for SemanticVersion {
    fn eq(&self, other: &&str) -> bool {
        match Self::parse(*other) {
            Ok(other_version) => self.eq(&other_version),
            Err(_) => false,
        }
    }
}

impl PartialEq<SemanticVersion> for &str {
    fn eq(&self, other: &SemanticVersion) -> bool {
        match SemanticVersion::parse(*self) {
            Ok(version) => version.eq(other),
            Err(_) => false,
        }
    }
}

impl PartialOrd for SemanticVersion {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl Ord for SemanticVersion {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.cmp(&other.0)
    }
}

impl PartialOrd<semver::Version> for SemanticVersion {
    fn partial_cmp(&self, other: &semver::Version) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(other)
    }
}

impl PartialOrd<SemanticVersion> for semver::Version {
    fn partial_cmp(&self, other: &SemanticVersion) -> Option<std::cmp::Ordering> {
        self.partial_cmp(&other.0)
    }
}

impl PartialOrd<String> for SemanticVersion {
    fn partial_cmp(&self, other: &String) -> Option<std::cmp::Ordering> {
        match Self::parse(other.as_str()) {
            Ok(other_version) => self.partial_cmp(&other_version),
            Err(_) => None,
        }
    }
}

impl PartialOrd<SemanticVersion> for String {
    fn partial_cmp(&self, other: &SemanticVersion) -> Option<std::cmp::Ordering> {
        match SemanticVersion::parse(self.as_str()) {
            Ok(version) => version.partial_cmp(other),
            Err(_) => None,
        }
    }
}

impl PartialOrd<str> for SemanticVersion {
    fn partial_cmp(&self, other: &str) -> Option<std::cmp::Ordering> {
        match Self::parse(other) {
            Ok(other_version) => self.partial_cmp(&other_version),
            Err(_) => None,
        }
    }
}

impl PartialOrd<SemanticVersion> for str {
    fn partial_cmp(&self, other: &SemanticVersion) -> Option<std::cmp::Ordering> {
        match SemanticVersion::parse(self) {
            Ok(version) => version.partial_cmp(other),
            Err(_) => None,
        }
    }
}
