// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use std::{fmt::Display, hash::Hash, str::FromStr, sync::OnceLock};

use miette::Diagnostic;
use regex::Regex;
use rust_i18n::t;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    schemas::dsc_repo::DscRepoSchema,
    types::{
        DateVersion,
        DateVersionError,
        SemanticVersion,
        SemanticVersionError,
        SemanticVersionReq,
    },
};

/// Defines the version of a DSC resource.
///
/// DSC supports both semantic versioning and date versioning for resources. Semantic
/// versioning is the preferred and recommended versioning strategy. DSC only supports date
/// versioning for compatibility scenarios.
///
/// When the version is defined as a valid semantic version ([`ResourceVersion::Semantic`]), DSC
/// can correctly compare versions to determine the latest version or match a
/// [`SemanticVersionReq`]. Where possible, resource and extension authors should follow semantic
/// versioning for the best user experience.
///
/// When the version is a date version ([`ResourceVersion::Date`]), DSC compares the dates to see
/// which one is newer. Date versions are only equivalent when they define the same date and
/// optional prerelease segment. Both versions must define or omit the prerelease segment. If
/// the prerelease segment is defined, the segments must be identical - the comparison is case
/// sensitive.
///
/// # Examples
///
/// The following examples show how different instances of [`ResourceVersion`] compare to other
/// instances of `ResourceVersion`, [`SemanticVersion`], [`DateVersion`], [`String`], and [`str`].
///
/// ```rust
/// /// First define a semantic version and a date version to compare against.
/// # use dsc_lib::types::{DateVersion, ResourceVersion, SemanticVersion};
/// let semantic = ResourceVersion::parse("1.2.3").unwrap();
/// let date = ResourceVersion::parse("2026-01-15").unwrap();
/// ```
///
/// You can compare instances of [`ResourceVersion::Semantic`] to strings, string slices, and
/// semantic versions.
/// ```rust
/// # use dsc_lib::types::{DateVersion, ResourceVersion, SemanticVersion};
/// # let semantic = ResourceVersion::parse("1.2.3").unwrap();
/// # let date = ResourceVersion::parse("2026-01-15").unwrap();
/// assert_eq!(semantic, SemanticVersion::parse("1.2.3").unwrap());
/// assert_eq!(semantic, "1.2.3");
/// assert_ne!(semantic, "1.2.*".to_string());
/// ```
///
/// You can compare instances of [`ResourceVersion::Date`] to strings, string slices, and
/// date versions.
///
/// ```rust
/// # use dsc_lib::types::{DateVersion, ResourceVersion, SemanticVersion};
/// # let semantic = ResourceVersion::parse("1.2.3").unwrap();
/// # let date = ResourceVersion::parse("2026-01-15").unwrap();
/// assert_eq!(date, DateVersion::parse("2026-01-15").unwrap());
/// assert_eq!(date, "2026-01-15");
/// assert_ne!(date, "2026-02-03".to_string());
/// ```
///
/// When a semantic version is compared to a date version, the semantic version is always treated as being higher:
///
/// ```rust
/// # use dsc_lib::types::{DateVersion, ResourceVersion, SemanticVersion};
/// # let semantic = ResourceVersion::parse("1.2.3").unwrap();
/// # let date = ResourceVersion::parse("2026-01-15").unwrap();
/// assert!(semantic > date);
/// assert!(date < SemanticVersion::parse("0.1.0").unwrap());
/// ```
///
/// Semantic version comparisons work as expected.
///
/// ```rust
/// # use dsc_lib::types::{DateVersion, ResourceVersion, SemanticVersion};
/// # let semantic = ResourceVersion::parse("1.2.3").unwrap();
/// # let date = ResourceVersion::parse("2026-01-15").unwrap();
/// //
/// assert!(semantic < SemanticVersion::parse("1.2.4").unwrap());
/// assert!(semantic >= SemanticVersion::parse("1.0.0").unwrap());
/// ```
///
/// When comparing a semantic version to a string, the comparison uses semantic version ordering if
/// the string can be parsed as a semantic version. If the string can be parsed as a date version,
/// the semantic version is always greater.
///
/// ```rust
/// # use dsc_lib::types::{DateVersion, ResourceVersion, SemanticVersion};
/// # let semantic = ResourceVersion::parse("1.2.3").unwrap();
/// # let date = ResourceVersion::parse("2026-01-15").unwrap();
/// assert!(semantic < "1.2.4");
/// assert!(semantic > "2026-01-15");
/// ```
///
/// Date version comparisons are deterministic: Comparing _first_ the date. If the date is
/// identical, a stable date version sorts as higher than a prerelease date version. If the dates
/// are identical and both versions are prerelease, the prerelease segments are compared
/// lexicographically:
///
/// ```rust
/// # use dsc_lib::types::{DateVersion, ResourceVersion, SemanticVersion};
/// # let semantic = ResourceVersion::parse("1.2.3").unwrap();
/// # let date = ResourceVersion::parse("2026-01-15").unwrap();
/// assert!(date < "2026-02-01");
/// assert!(date >= "2026-01-15");
/// assert!(date > "2026-01-15-preview");
/// let alpha = ResourceVersion::parse("2026-01-15-alpha").unwrap();
/// let beta = ResourceVersion::parse("2026-01-15-beta").unwrap();
/// assert!(alpha < beta);
/// ```
///
/// Comparing a resource version to a string that doesn't parse as a valid resource version always
/// returns `false`:
///
/// ```rust
/// # use dsc_lib::types::{DateVersion, ResourceVersion, SemanticVersion};
/// # let semantic = ResourceVersion::parse("1.2.3").unwrap();
/// # let date = ResourceVersion::parse("2026-01-15").unwrap();
/// assert_eq!(semantic > "foo", false);
/// assert_eq!(semantic < "foo", false);
/// assert_eq!(date <= "foo", false);
/// assert_eq!(date >= "foo", false);
/// assert_eq!(date == "foo", false);
/// ```
///
/// Finally, you can freely convert between strings and [`ResourceVersion`]:
///
/// ```rust
/// # use dsc_lib::types::ResourceVersion;
/// let semantic: ResourceVersion = "1.2.3".parse().unwrap();
/// let date = ResourceVersion::parse("2026-01-15").unwrap();
///
/// // Define a function that expects a string:
/// fn expects_string(input: &str) {
///     println!("Input was: '{input}'")
/// }
///
/// // You can pass the `ResourceVersion` in a few ways:
/// expects_string(&semantic.to_string());
/// expects_string(date.to_string().as_str());
/// ```
///
/// [01]: https://doc.rust-lang.org/std/cmp/trait.Ord.html#lexicographical-comparison
/// [02]: https://www.iso.org/iso-8601-date-and-time-format.html
#[derive(Debug, Clone, Eq, Serialize, Deserialize, JsonSchema, DscRepoSchema)]
#[dsc_repo_schema(base_name = "resourceVersion", folder_path = "definitions")]
#[serde(untagged, try_from = "String", into = "String")]
#[schemars(!try_from, !into)]
#[schemars(
    title = t!("schemas.definitions.resourceVersion.title"),
    description = t!("schemas.definitions.resourceVersion.description"),
    extend(
        "markdownDescription" = t!("schemas.definitions.resourceVersion.markdownDescription")
    )
)]
pub enum ResourceVersion {
    /// Defines the resource's version as a semantic version, containing an inner [`SemanticVersion`].
    /// This is the preferred and recommended versioning scheme for DSC resources.
    ///
    /// For more information about defining semantic versions, see [`SemanticVersion`]. For more
    /// information about semantic versioning, see [semver.org][01].
    ///
    /// [01]: https://semver.org
    #[schemars(
        title = t!("schemas.definitions.resourceVersion.semanticVariant.title"),
        description = t!("schemas.definitions.resourceVersion.semanticVariant.description"),
        extend(
            "markdownDescription" = t!("schemas.definitions.resourceVersion.semanticVariant.markdownDescription")
        )
    )]
    Semantic(SemanticVersion),
    /// Defines the resource's version as a date version, containing an inner [`DateVersion`]. This
    /// variant remains supported for compatibility purposes but is _not_ recommended for
    /// production usage.
    ///
    /// When a resource defines the version as a date version:
    ///
    /// 1. You can only use exact match version requirements for that resource. You can't define
    ///    a range of valid date versions to support. You must specify the exact date version of
    ///    the resource.
    /// 1. When DSC discovers a multiple manifests for a resource, DSC always treats semantically
    ///    versioned resources as newer than resources with a date version.
    ///
    /// For more information about defining date versions, see [`DateVersion`]. For more information
    /// about defining a version requirement for a resource with a date version, see
    /// [`ResourceVersionReq`].
    ///
    /// [`ResourceVersionReq`]: crate::types::ResourceVersionReq
    /// [01]: https://doc.rust-lang.org/std/cmp/trait.Ord.html#lexicographical-comparison
    #[schemars(
        title = t!("schemas.definitions.resourceVersion.dateVariant.title"),
        description = t!("schemas.definitions.resourceVersion.dateVariant.description"),
        extend(
            "deprecated" = true,
            "deprecationMessage" = t!("schemas.definitions.resourceVersion.dateVariant.deprecationMessage"),
            "markdownDescription" = t!("schemas.definitions.resourceVersion.dateVariant.markdownDescription"),
        )
    )]
    Date(DateVersion),
}

/// Defines errors that can occur when parsing or working with [`ResourceVersion`].
#[derive(Debug, Error, Diagnostic)]
#[non_exhaustive]
pub enum ResourceVersionError {
    /// Indicates that the input string didn't match the approximate shape of either a semantic
    /// version or a date version, so it couldn't be parsed as a resource version at all.
    #[error("{t}", t = t!(
        "types.resource_version.unparseableVersion",
        "text" => text,
    ))]
    UnparseableVersion{
        /// The input string that failed to parse as a resource version.
        text: String
    },

    /// Indicates that the input string was recognized as a [`DateVersion`] but failed to parse.
    #[error("{t}", t = t!(
        "types.resource_version.invalidDateVersion",
        "err" => source
    ))]
    InvalidDateVersion{
        #[from] source: DateVersionError,
    },

    /// Indicates that the input string was recognized as a [`SemanticVersion`] but failed to parse.
    #[error("{t}", t = t!(
        "types.resource_version.invalidSemanticVersion",
        "err" => source
    ))]
    InvalidSemanticVersion{
        #[from] source: SemanticVersionError,
    },

    /// Indicates that the [`ResourceVersion`] couldn't convert into a [`SemanticVersion`] because the
    /// underlying variant was a [`DateVersion`].
    #[error("{t}", t = t!(
        "types.resource_version.invalidConversionToSemanticVersion",
        "version" => version,
    ))]
    InvalidConversionToSemanticVersion{
        /// The inner [`DateVersion`] for a [`ResourceVersion`] that failed to convert into a
        /// [`SemanticVersion`].
        version: DateVersion,
    },

    /// Indicates that the [`ResourceVersion`] couldn't convert into a [`DateVersion`] because the
    /// underlying variant was a [`SemanticVersion`].
    #[error("{t}", t = t!(
        "types.resource_version.invalidConversionToDateVersion",
        "version" => version,
    ))]
    InvalidConversionToDateVersion{
        /// The inner [`SemanticVersion`] for a [`ResourceVersion`] that failed to convert into a
        /// [`DateVersion`].
        version: SemanticVersion,
    },
}

/// Defines a regular expression for approximating whether a string looks like a semantic version.
/// This is used during parsing to determine whether to parse the input string as a semantic
/// version and forward the appropriate parsing errors, if any.
///
/// This regex is kept as a static to enable lazy initialization and prevent needing to recompile
/// the regex pattern every time we parse a resource version.
static APPROXIMATE_SEMVER_REGEX: OnceLock<Regex> = OnceLock::new();

/// Defines a regular expression for approximating whether a string looks like a date version. This
/// is used during parsing to determine whether to parse the input string as a date version and
/// forward the appropriate parsing errors, if any.
///
/// This regex is kept as a static to enable lazy initialization and prevent needing to recompile
/// the regex pattern every time we parse a resource version.
static APPROXIMATE_DATEVER_REGEX: OnceLock<Regex> = OnceLock::new();

impl ResourceVersion {
    /// Parses a string into a new instance of [`ResourceVersion`].
    ///
    /// If the input string can be parsed as a [`SemanticVersion`], this function returns the
    /// [`Semantic`] variant. If the input string can be parsed as a [`DateVersion`], this function
    /// returns the [`Date`] variant. If the input string can't be parsed as either a semantic or
    /// date version, this function raises an error.
    ///
    /// # Examples
    ///
    /// The following snippet shows how you can parse a resource version from input strings:
    ///
    /// ```rust
    /// use chrono::Datelike;
    /// use dsc_lib::types::ResourceVersion;
    ///
    /// let semantic = ResourceVersion::parse("1.2.3").unwrap();
    /// let date = ResourceVersion::parse("2026-02-15").unwrap();
    ///
    /// assert_eq!(semantic.as_semver().unwrap().major, 1);
    /// assert_eq!(date.as_date_version().unwrap().year(), 2026);
    /// ```
    ///
    /// # Error
    ///
    /// When the input string can't be parsed as either a semantic version or a date version, this
    /// function raises a [`ResourceVersionError`]. Parsing can fail for a few reasons:
    ///
    /// - When the input string doesn't match the general shape of either a semantic version or a
    ///   date version, it fails with [`ResourceVersionError::UnparseableVersion`]. For example, the
    ///   string `foo` would fail with this error.
    /// - When the input string matches the general shape of a semantic version but fails to parse
    ///   as a valid semantic version, it raises [`ResourceVersionError::InvalidSemanticVersion`].
    /// - When the input string matches the general shape of a date version but fails to parse as a
    ///   valid date version, it raises [`ResourceVersionError::InvalidDateVersion`].
    ///
    /// [`Date`]: ResourceVersion::Date
    /// [`Semantic`]: ResourceVersion::Semantic
    pub fn parse(text: &str) -> Result<Self, ResourceVersionError> {
        let apparent_semver = APPROXIMATE_SEMVER_REGEX.get_or_init(
            Self::init_approximate_semver_pattern
        );
        let apparent_date = APPROXIMATE_DATEVER_REGEX.get_or_init(
            Self::init_approximate_datever_pattern
        );

        if apparent_semver.is_match(text) {
            match SemanticVersion::parse(text) {
                Ok(v) => Ok(Self::Semantic(v)),
                Err(e) => Err(ResourceVersionError::InvalidSemanticVersion {
                    source: e,
                })
            }
        } else if apparent_date.is_match(text) {
            match DateVersion::parse(text) {
                Ok(v) => Ok(Self::Date(v)),
                Err(e) => Err(ResourceVersionError::InvalidDateVersion {
                    source: e,
                })
            }
        } else {
            Err(ResourceVersionError::UnparseableVersion { text: text.to_string() })
        }
    }

    /// Indicates whether the resource version is semantic.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use dsc_lib::types::ResourceVersion;
    ///
    /// let semantic = ResourceVersion::parse("1.2.3").unwrap();
    /// let date = ResourceVersion::parse("2026-01-02").unwrap();
    ///
    /// assert_eq!(semantic.is_semver(), true);
    /// assert_eq!(date.is_semver(), false);
    /// ```
    pub fn is_semver(&self) -> bool {
        matches!(self, Self::Semantic(_))
    }

    /// Indicates whether the resource version is a date version.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use dsc_lib::types::ResourceVersion;
    ///
    /// let semantic = ResourceVersion::parse("1.2.3").unwrap();
    /// let date = ResourceVersion::parse("2026-01-02").unwrap();
    ///
    /// assert_eq!(semantic.is_date_version(), false);
    /// assert_eq!(date.is_date_version(), true);
    /// ```
    pub fn is_date_version(&self) -> bool {
        matches!(self, Self::Date(_))
    }

    /// Returns the version as a reference to the underlying [`SemanticVersion`] if possible.
    ///
    /// If the underlying version is [`Semantic`], this method returns some semantic version.
    /// Otherwise, it returns [`None`].
    ///
    /// # Examples
    ///
    /// The following examples show how `as_semver()` behaves for different versions.
    ///
    /// ```rust
    /// use dsc_lib::types::{ResourceVersion, SemanticVersion};
    ///
    /// let semantic = ResourceVersion::parse("1.2.3").unwrap();
    /// let date = ResourceVersion::parse("2026-01-15").unwrap();
    ///
    /// assert_eq!(
    ///     semantic.as_semver(),
    ///     Some(&SemanticVersion::new(1, 2, 3))
    /// );
    /// assert_eq!(
    ///     date.as_semver(),
    ///     None
    /// );
    /// ```
    ///
    /// [`Semantic`]: ResourceVersion::Semantic
    pub fn as_semver(&self) -> Option<&SemanticVersion> {
        match self {
            Self::Semantic(v) => Some(v),
            _ => None,
        }
    }

    /// Returns the version as a reference to the underlying [`DateVersion`] if possible.
    ///
    /// If the underlying version is [`Date`], this method returns some date version.
    /// Otherwise, it returns [`None`].
    ///
    /// # Examples
    ///
    /// The following examples show how `as_date_version()` behaves for different versions.
    ///
    /// ```rust
    /// use dsc_lib::types::{ResourceVersion, DateVersion};
    ///
    /// let semantic = ResourceVersion::parse("1.2.3").unwrap();
    /// let date = ResourceVersion::parse("2026-01-15").unwrap();
    ///
    /// assert_eq!(
    ///     semantic.as_date_version(),
    ///     None,
    /// );
    /// assert_eq!(
    ///     date.as_date_version(),
    ///     Some(&DateVersion::parse("2026-01-15").unwrap())
    /// );
    /// ```
    ///
    /// [`Date`]: ResourceVersion::Date
    pub fn as_date_version(&self) -> Option<&DateVersion> {
        match self {
            Self::Date(v) => Some(v),
            _ => None,
        }
    }

    /// Compares an instance of [`ResourceVersion`] with [`SemanticVersionReq`].
    ///
    /// When the instance is [`ResourceVersion::Semantic`], this method applies the canonical
    /// matching logic from [`SemanticVersionReq`] for the version. When the instance is
    /// [`ResourceVersion::Date`], this method always returns `false`.
    ///
    /// For more information about semantic version requirements and syntax, see
    /// [`SemanticVersionReq`].
    ///
    /// # Examples
    ///
    /// The following example shows how comparisons work for different instances of
    /// [`ResourceVersion`].
    ///
    /// ```rust
    /// # use dsc_lib::types::{ResourceVersion, SemanticVersionReq};
    /// let semantic = ResourceVersion::parse("1.2.3").unwrap();
    /// let date = ResourceVersion::parse("2026-01-15").unwrap();
    ///
    /// let ref le_v2_0: SemanticVersionReq = "<=2.0".parse().unwrap();
    /// assert!(semantic.matches_semver_req(le_v2_0));
    /// assert!(!date.matches_semver_req(le_v2_0));
    ///
    /// let ref tilde_v1: SemanticVersionReq = "~1".parse().unwrap();
    /// assert!(semantic.matches_semver_req(tilde_v1));
    /// assert!(!date.matches_semver_req(tilde_v1));
    /// ```
    pub fn matches_semver_req(&self, requirement: &SemanticVersionReq) -> bool {
        match self {
            Self::Semantic(v) => requirement.matches(v),
            _ => false,
        }
    }

    /// Compares an instance of [`ResourceVersion`] with [`DateVersion`].
    ///
    /// When the instance is [`ResourceVersion::Date`], this method checks whether the version is
    /// exactly the same as a given [`DateVersion`]. When the instance is
    /// [`ResourceVersion::Semantic`], this method always returns `false`.
    ///
    /// # Examples
    ///
    /// The following example shows how comparisons work for different instances of
    /// [`ResourceVersion`].
    ///
    /// ```rust
    /// # use dsc_lib::types::{DateVersion, ResourceVersion};
    /// let semantic_version = ResourceVersion::parse("1.2.3").unwrap();
    /// let date_version = ResourceVersion::parse("2026-01-15").unwrap();
    ///
    /// let ref stable_req = DateVersion::parse("2026-01-15").unwrap();
    /// assert!(!semantic_version.matches_date_req(stable_req));
    /// assert!(date_version.matches_date_req(stable_req));
    ///
    /// let ref prerelease_req = DateVersion::parse("2026-01-15-rc").unwrap();
    /// assert!(!semantic_version.matches_date_req(prerelease_req));
    /// assert!(!date_version.matches_date_req(prerelease_req));
    /// ```
    pub fn matches_date_req(&self, requirement: &DateVersion) -> bool {
        match self {
            Self::Date(v) => v == requirement,
            _ => false,
        }
    }

    /// Defines the regular expression pattern that approximates the shape of a semantic version.
    /// This is used to quickly determine whether a string might be a semantic version before
    /// attempting to parse it as one. This allows us to provide better error messages for invalid
    /// versions by distinguishing between strings that look like semantic versions and those that
    /// might be date versions or completely arbitrary strings.
    const APPROXIMATE_SEMVER_PATTERN: &str = const_str::concat!(
        "^",                // Anchor to start of string
        "(?:",              // Open non-capturing group for the alternatives
            r"\d+",         // Match any number of digits alone
            "|",            // or
            r"\d+\.\d+.*",  // Match an apparent major.minor with anything after it
            "|",            // or
            r"\d+[^-]+",    // Match a number followed by anything but a hyphen
        ")$",               // Close non-capturing group and anchor to end of string
    );

    /// Returns the [`Regex`] for [`APPROXIMATE_SEMVER_PATTERN`].
    ///
    /// This private method is used to initialize the [`APPROXIMATE_SEMVER_REGEX`]
    /// private static to reduce the number of times the regular expression is compiled from the
    /// pattern string.
    ///
    /// [`APPROXIMATE_SEMVER_PATTERN`]: ResourceVersion::APPROXIMATE_SEMVER_PATTERN
    fn init_approximate_semver_pattern() -> Regex {
        Regex::new(Self::APPROXIMATE_SEMVER_PATTERN).expect("pattern is valid")
    }

    /// Defines the regular expression pattern that approximates the shape of a date version. This
    /// is used to quickly determine whether a string might be a date version before attempting to
    /// parse it as one. This allows us to provide better error messages for invalid versions by
    /// distinguishing between strings that look like date versions and those that might be
    /// semantic versions or completely arbitrary strings.
    const APPROXIMATE_DATEVER_PATTERN: &str = r"^\d+-\d+";

    /// Returns the [`Regex`] for [`APPROXIMATE_DATEVER_PATTERN`].
    ///
    /// This private method is used to initialize the [`APPROXIMATE_DATEVER_REGEX`]
    /// private static to reduce the number of times the regular expression is compiled from the
    /// pattern string.
    ///
    /// [`APPROXIMATE_DATEVER_PATTERN`]: ResourceVersion::APPROXIMATE_DATEVER_PATTERN
    fn init_approximate_datever_pattern() -> Regex {
        Regex::new(Self::APPROXIMATE_DATEVER_PATTERN).expect("pattern is valid")
    }
}

// Default to semantic version `0.0.0` rather than an empty string.
impl Default for ResourceVersion {
    fn default() -> Self {
        Self::Semantic(SemanticVersion::default())
    }
}

// Enable using `ResourceVersion` in `format!` and similar macros.
impl Display for ResourceVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Semantic(v) => write!(f, "{}", v),
            Self::Date(s) => write!(f, "{}", s),
        }
    }
}

// Parsing from a string is just calling `Self::parse()`
impl FromStr for ResourceVersion {
    type Err = ResourceVersionError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

// Implemented various conversion traits to move between `ResourceVersion`, `SemanticVersion`,
// `DateVersion`, `String`, and string slice (`str`).
impl TryFrom<&String> for ResourceVersion {
    type Error = ResourceVersionError;
    fn try_from(value: &String) -> Result<Self, Self::Error> {
        Self::parse(value.as_str())
    }
}

impl TryFrom<String> for ResourceVersion {
    type Error = ResourceVersionError;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::parse(value.as_str())
    }
}

impl From<ResourceVersion> for String {
    fn from(value: ResourceVersion) -> Self {
        value.to_string()
    }
}

// We can't bidirectionally convert string slices, because we can't return a temporary reference.
// We can still convert _from_ string slices, but not _into_ them.
impl TryFrom<&str> for ResourceVersion {
    type Error = ResourceVersionError;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::parse(value)
    }
}

impl From<SemanticVersion> for ResourceVersion {
    fn from(value: SemanticVersion) -> Self {
        Self::Semantic(value)
    }
}

impl From<&SemanticVersion> for ResourceVersion {
    fn from(value: &SemanticVersion) -> Self {
        Self::Semantic(value.clone())
    }
}

// Creating an instance of `SemanticVersion` from `ResourceVersion` is a fallible conversion,
// since `ResourceVersion` can define non-semantic versions.
impl TryFrom<ResourceVersion> for SemanticVersion {
    type Error = ResourceVersionError;

    fn try_from(value: ResourceVersion) -> Result<Self, Self::Error> {
        match value {
            ResourceVersion::Semantic(v) => Ok(v),
            ResourceVersion::Date(version) => Err(
                ResourceVersionError::InvalidConversionToSemanticVersion { version }
            ),
        }
    }
}

impl From<DateVersion> for ResourceVersion {
    fn from(value: DateVersion) -> Self {
        Self::Date(value)
    }
}

impl From<&DateVersion> for ResourceVersion {
    fn from(value: &DateVersion) -> Self {
        Self::Date(value.clone())
    }
}

// Creating an instance of `DateVersion` from `ResourceVersion` is a fallible conversion,
// since `ResourceVersion` can define non-semantic versions.
impl TryFrom<ResourceVersion> for DateVersion {
    type Error = ResourceVersionError;

    fn try_from(value: ResourceVersion) -> Result<Self, Self::Error> {
        match value {
            ResourceVersion::Date(d) => Ok(d),
            ResourceVersion::Semantic(version) => Err(
                ResourceVersionError::InvalidConversionToDateVersion { version }
            ),
        }
    }
}

impl Hash for ResourceVersion {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            Self::Semantic(v) => v.hash(state),
            Self::Date(v) => v.hash(state),
        }
    }
}

// Implement traits for comparing `ResourceVersion` to strings, semantic versions, and date
// versions bi-directionally.
impl PartialEq for ResourceVersion {
    fn eq(&self, other: &Self) -> bool {
        match self {
            Self::Semantic(version) => match other {
                Self::Semantic(other_version) => version == other_version,
                _ => false,
            },
            Self::Date(date) => match other {
                Self::Date(other_date) => date == other_date,
                _ => false,
            }
        }
    }
}

impl PartialEq<SemanticVersion> for ResourceVersion {
    fn eq(&self, other: &SemanticVersion) -> bool {
        match self {
            Self::Semantic(v) => v == other,
            _ => false,
        }
    }
}

impl PartialEq<ResourceVersion> for SemanticVersion {
    fn eq(&self, other: &ResourceVersion) -> bool {
        match other {
            ResourceVersion::Semantic(v) => self == v,
            _ => false,
        }
    }
}

impl PartialEq<DateVersion> for ResourceVersion {
    fn eq(&self, other: &DateVersion) -> bool {
        match self {
            Self::Date(v) => v == other,
            _ => false,
        }
    }
}

impl PartialEq<ResourceVersion> for DateVersion {
    fn eq(&self, other: &ResourceVersion) -> bool {
        match other {
            ResourceVersion::Date(v) => self == v,
            _ => false,
        }
    }
}

impl PartialEq<&str> for ResourceVersion {
    fn eq(&self, other: &&str) -> bool {
        if let Ok(other_version) = Self::parse(other) {
            self == &other_version
        } else {
            false
        }
    }
}

impl PartialEq<ResourceVersion> for &str {
    fn eq(&self, other: &ResourceVersion) -> bool {
        if let Ok(version) = ResourceVersion::parse(self) {
            &version == other
        } else {
            false
        }
    }
}

impl PartialEq<String> for ResourceVersion {
    fn eq(&self, other: &String) -> bool {
        if let Ok(other_version) = Self::parse(other.as_str()) {
            self == &other_version
        } else {
            false
        }
    }
}

impl PartialEq<ResourceVersion> for String {
    fn eq(&self, other: &ResourceVersion) -> bool {
        if let Ok(version) = ResourceVersion::parse(self.as_str()) {
            &version == other
        } else {
            false
        }
    }
}

impl PartialEq<str> for ResourceVersion {
    fn eq(&self, other: &str) -> bool {
        if let Ok(other_version) = Self::parse(other) {
            self == &other_version
        } else {
            false
        }
    }
}

impl PartialEq<ResourceVersion> for str {
    fn eq(&self, other: &ResourceVersion) -> bool {
        if let Ok(version) = ResourceVersion::parse(self) {
            &version == other
        } else {
            false
        }
    }
}

impl Ord for ResourceVersion {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
         match self {
            Self::Semantic(version) => match other {
                Self::Semantic(other_version) => version.cmp(other_version),
                Self::Date(_) => std::cmp::Ordering::Greater,
            },
            Self::Date(date) => match other {
                Self::Semantic(_) => std::cmp::Ordering::Less,
                Self::Date(other_date) => date.cmp(other_date),
            },
        }
    }
}

impl PartialOrd for ResourceVersion {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialOrd<SemanticVersion> for ResourceVersion {
    fn partial_cmp(&self, other: &SemanticVersion) -> Option<std::cmp::Ordering> {
        match self {
            Self::Semantic(v) => v.partial_cmp(other),
            Self::Date(_) => Some(std::cmp::Ordering::Less),
        }
    }
}

impl PartialOrd<ResourceVersion> for SemanticVersion {
    fn partial_cmp(&self, other: &ResourceVersion) -> Option<std::cmp::Ordering> {
        match other {
            ResourceVersion::Semantic(v) => self.partial_cmp(v),
            ResourceVersion::Date(_) => Some(std::cmp::Ordering::Greater),
        }
    }
}

impl PartialOrd<DateVersion> for ResourceVersion {
    fn partial_cmp(&self, other: &DateVersion) -> Option<std::cmp::Ordering> {
        match self {
            Self::Semantic(_) => Some(std::cmp::Ordering::Greater),
            Self::Date(v) => v.partial_cmp(other),
        }
    }
}

impl PartialOrd<ResourceVersion> for DateVersion {
    fn partial_cmp(&self, other: &ResourceVersion) -> Option<std::cmp::Ordering> {
        match other {
            ResourceVersion::Date(v) => self.partial_cmp(v),
            ResourceVersion::Semantic(_) => Some(std::cmp::Ordering::Less),
        }
    }
}

impl PartialOrd<String> for ResourceVersion {
    fn partial_cmp(&self, other: &String) -> Option<std::cmp::Ordering> {
        match ResourceVersion::parse(other.as_str()) {
            Ok(other_version) => self.partial_cmp(&other_version),
            Err(_) => None,
         }
    }
}

impl PartialOrd<ResourceVersion> for String {
    fn partial_cmp(&self, other: &ResourceVersion) -> Option<std::cmp::Ordering> {
         match ResourceVersion::parse(self.as_str()) {
            Ok(version) => version.partial_cmp(other),
            Err(_) => None,
         }
    }
}

impl PartialOrd<&str> for ResourceVersion {
    fn partial_cmp(&self, other: &&str) -> Option<std::cmp::Ordering> {
        match ResourceVersion::parse(other) {
            Ok(other_version) => self.partial_cmp(&other_version),
            Err(_) => None,
        }
    }
}

impl PartialOrd<str> for ResourceVersion {
    fn partial_cmp(&self, other: &str) -> Option<std::cmp::Ordering> {
        match ResourceVersion::parse(other) {
            Ok(other_version) => self.partial_cmp(&other_version),
            Err(_) => None,
        }
    }
}

impl PartialOrd<ResourceVersion> for &str {
    fn partial_cmp(&self, other: &ResourceVersion) -> Option<std::cmp::Ordering> {
        match ResourceVersion::parse(self) {
            Ok(version) => version.partial_cmp(other),
            Err(_) => None,
         }
    }
}

impl PartialOrd<ResourceVersion> for str {
    fn partial_cmp(&self, other: &ResourceVersion) -> Option<std::cmp::Ordering> {
        match ResourceVersion::parse(self) {
            Ok(version) => version.partial_cmp(other),
            Err(_) => None,
         }
    }
}
