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
    types::{DateVersion, DateVersionError, ResourceVersion, SemanticVersionReq, SemanticVersionReqError}
};

/// Defines one or more limitations for a [`ResourceVersion`] to enable version pinning.
///
/// DSC supports both semantic versioning and date versioning for resources. Semantic versioning is
/// the preferred and recommended versioning strategy. DSC only supports date versioning for
/// compatibility scenarios.
///
/// Because DSC supports date versions for compatibility, version requirements must also support
/// date versions.
///
/// When a [`ResourceVersionReq`] is semantic, it behaves like a [`SemanticVersionReq`] and only
/// matches resource versions that are semantic _and_ valid for the given requirement. Date
/// versions never match a semantic resource version requirement.
///
/// Similarly, when a [`ResourceVersionReq`] is a date version, it can never match a semantically
/// versioned [`ResourceVersion`]. Instead, it matches an `ResourceVersion` when the date version
/// is _exactly_ the same as the resource version requirement.
///
/// Date resource versions and resource version requirements are only defined for compatibility
/// scenarios. You should use semantic versions for resources and resource version requirements.
///
/// ## Defining a resource version requirement
///
/// Not every string is a valid resource version requirement. To usefully define a resource
/// version requirement that supports correctly matching semantic versions, you must define the
/// requirement as valid `SemanticVersionReq`. See the [`SemanticVersionReq` documentation][01] for
/// full details on defining semantic version requirements.
///
/// If the string can't be parsed as a semantic version requirement, it _must_ parse as a valid
/// date version. For more information on defining date versions, see [`DateVersion`].
///
/// If the string doesn't parse as either a semantic version requirement or a date version, the
/// value is invalid.
///
/// ## Examples
///
/// When you create a new instance of [`ResourceVersionReq`], the variant is `Semantic` when the
/// input string parses as a [`SemanticVersionReq`]. If it parses as a [`DateVersion`], the variant
/// is `Date`. Parsing fails if the input string doesn't parse as either a [`SemanticVersionReq`]
/// or [`DateVersion`].
///
/// ```rust
/// use dsc_lib::types::{ResourceVersion, ResourceVersionReq};
///
/// let semantic_req = ResourceVersionReq::parse("^1.2, <1.5").unwrap();
/// let date_req = ResourceVersionReq::parse("2026-01-15-rc").unwrap();
///
/// let v1_2_3 = &ResourceVersion::parse("1.2.3").unwrap();
/// let v1_5_1 = &ResourceVersion::parse("1.5.1").unwrap();
/// let v_date = &ResourceVersion::parse("2026-01-15-rc").unwrap();
///
/// // Semantic requirement uses underlying semantic version requirement logic:
/// assert!(semantic_req.matches(v1_2_3));
/// assert!(!semantic_req.matches(v1_5_1));
/// // Semantic requirements never match date versions:
/// assert!(!semantic_req.matches(v_date));
///
/// // Date requirements only match date versions _exactly_:
/// assert!(date_req.matches(v_date));
/// // Differing casing causes the match to fail:
/// assert!(!date_req.matches(&ResourceVersion::parse("2026-01-15-RC").unwrap()));
/// ```
///
/// [01]: SemanticVersionReq
#[derive(Debug, Clone, Eq, Serialize, Deserialize, JsonSchema, DscRepoSchema)]
#[dsc_repo_schema(base_name = "resourceVersionReq", folder_path = "definitions")]
#[serde(untagged, try_from = "String", into = "String")]
#[schemars(!try_from, !into)]
#[schemars(
    title = t!("schemas.definitions.resourceVersionReq.title"),
    description = t!("schemas.definitions.resourceVersionReq.description"),
    extend(
        "markdownDescription" = t!("schemas.definitions.resourceVersionReq.markdownDescription")
    )
)]
pub enum ResourceVersionReq {
    /// Defines the version requirement for the resource as a semantic version requirement,
    /// containing an inner [`SemanticVersionReq`]. This is the preferred and recommended way to pin
    /// versions for DSC resources.
    ///
    /// For more information about defining semantic version requirements, see
    /// [`SemanticVersionReq`]. For more information about semantic versioning, see
    /// [semver.org][01].
    ///
    /// [01]: https://semver.org
    #[schemars(
        title = t!("schemas.definitions.resourceVersionReq.semanticVariant.title"),
        description = t!("schemas.definitions.resourceVersionReq.semanticVariant.description"),
        extend(
            "markdownDescription" = t!("schemas.definitions.resourceVersionReq.semanticVariant.markdownDescription")
        )
    )]
    Semantic(SemanticVersionReq),
    /// Defines the required version for the resource as a specific [`DateVersion`].
    ///
    /// DSC uses this variant for pinning resources that use date versioning. This variant remains
    /// supported for compatibility purposes but is _not_ recommended for production usage.
    ///
    /// When a resource version requirement is defined as a date version:
    ///
    /// 1. It can never match a semantically versioned resource.
    /// 1. It only matches a resource with a date version ([`ResourceVersion::Date`]) when the
    ///    resource version and this version requirement are exactly the same. The comparison is
    ///    case-sensitive for the prerelease segment, if present.
    #[schemars(
        title = t!("schemas.definitions.resourceVersionReq.dateVariant.title"),
        description = t!("schemas.definitions.resourceVersionReq.dateVariant.description"),
        extend(
            "deprecated" = true,
            "deprecationMessage" = t!("schemas.definitions.resourceVersionReq.dateVariant.deprecationMessage"),
            "markdownDescription" = t!("schemas.definitions.resourceVersionReq.dateVariant.markdownDescription"),
            "examples" = [
                "2026-02-03",
                "2026-11-27-preview"
            ]
        )
    )]
    Date(DateVersion),
}

/// Defines errors that can occur when parsing a string into a [`ResourceVersionReq`] or converting
/// between a [`ResourceVersionReq`] and another type (e.g. [`SemanticVersionReq`] or
/// [`DateVersion`]).
#[derive(Debug, Error, Diagnostic)]
#[non_exhaustive]
pub enum ResourceVersionReqError {
    /// Indicates that a string couldn't be parsed as a valid resource version requirement because
    /// it didn't match the expected format for either a semantic version requirement or a date
    /// version.
    #[error("{t}", t = t!(
        "types.resource_version_req.unparseableRequirement",
        "text" => text,
    ))]
    UnparseableRequirement{
        /// The input string that couldn't be parsed as a resource version requirement.
        text: String,
    },

    /// Indicates that a string matched the general shape of a semantic version requirement but
    /// failed to parse as a valid semantic version requirement.
    #[error("{t}", t = t!(
        "types.resource_version_req.invalidSemanticVersionRequirement",
        "err" => source
    ))]
    InvalidSemanticVersionRequirement{
        /// The error raised when attempting to parse a semantic version requirement from a string
        /// that matched the general shape of a semantic version requirement but failed to parse as
        /// a valid semantic version requirement.
        #[from] source: SemanticVersionReqError,
    },

    /// Indicates that a string matched the general shape of a date version but failed to parse as a
    /// valid date version.
    #[error("{t}", t = t!(
        "types.resource_version_req.invalidDateVersionRequirement",
        "err" => source
    ))]
    InvalidDateVersionRequirement{
        /// The error raised when attempting to parse a date version requirement from a string that
        /// matched the general shape of a date version but failed to parse as a valid date version.
        #[from] source: DateVersionError,
    },

    /// Indicates that an attempt was made to convert a [`ResourceVersionReq`] to a
    /// [`SemanticVersionReq`] but the conversion was invalid.
    ///
    /// This can only occur when the resource version requirement is a date version requirement.
    #[error("{t}", t = t!(
        "types.resource_version_req.invalidConversionToSemanticVersionReq",
        "req" => req
    ))]
    InvalidConversionToSemanticVersionReq{
        /// The inner date version for a resource version requirement that failed to convert to a
        /// semantic version requirement.
        req: DateVersion
    },

    /// Indicates that an attempt was made to convert a [`ResourceVersionReq`] to a [`DateVersion`]
    /// but the conversion was invalid.
    ///
    /// This can only occur when the resource version requirement is a semantic version requirement.
    #[error("{t}", t = t!(
        "types.resource_version_req.invalidConversionToDateVersion",
        "req" => req
    ))]
    InvalidConversionToDateVersion{
        /// The inner semantic version requirement for a resource version requirement that failed
        /// to convert to a date version.
        req: SemanticVersionReq
    },
}

/// Defines a regular expression for approximating whether a string looks like a semantic version
/// requirement. This is used during parsing to determine whether to parse the input string as a
/// semantic version requirement and forward the appropriate parsing errors, if any.
///
/// This regex is kept as a static to enable lazy initialization and prevent needing to recompile
/// the regex pattern every time we parse a resource version requirement.
static APPROXIMATE_SEMVER_REGEX: OnceLock<Regex> = OnceLock::new();

/// Defines a regular expression for approximating whether a string looks like a date version. This
/// is used during parsing to determine whether to parse the input string as a date version and
/// forward the appropriate parsing errors, if any.
///
/// This regex is kept as a static to enable lazy initialization and prevent needing to recompile
/// the regex pattern every time we parse a resource version requirement.
static APPROXIMATE_DATEVER_REGEX: OnceLock<Regex> = OnceLock::new();

impl ResourceVersionReq {
    /// Parses a string into a new instance of [`ResourceVersionReq`].
    ///
    /// If the input string can be parsed as a [`SemanticVersionReq`], this function returns the
    /// [`Semantic`] variant. If the input string can be parsed as a [`DateVersion`], this function
    /// returns the [`Date`] variant. If the input string can't be parsed as either a semantic
    /// version requirement or a date version, this function raises an error.
    ///
    /// # Examples
    ///
    /// The following snippet shows how you can parse a resource version requirement from input
    /// strings:
    ///
    /// ```rust
    /// # use chrono::Datelike;
    /// # use dsc_lib::types::ResourceVersionReq;
    /// let semantic = ResourceVersionReq::parse("^1.2, <1.5").unwrap();
    /// assert!(semantic.matches(
    ///     &dsc_lib::types::ResourceVersion::parse("1.3.0").unwrap()
    /// ));
    ///
    /// let date = ResourceVersionReq::parse("2026-02-15").unwrap();
    /// assert!(date.matches(
    ///     &dsc_lib::types::ResourceVersion::parse("2026-02-15").unwrap()
    /// ));
    /// ```
    ///
    /// # Error
    ///
    /// This function raises a [`ResourceVersionReqError`] when the input string can't be parsed as
    /// either a [`SemanticVersionReq`] or [`DateVersion`].
    ///
    /// [`Semantic`]: ResourceVersionReq::Semantic
    /// [`Date`]: ResourceVersionReq::Date
    pub fn parse(text: &str) -> Result<Self, ResourceVersionReqError> {
        let apparent_semver = APPROXIMATE_SEMVER_REGEX.get_or_init(
            Self::init_approximate_semver_pattern
        );
        let apparent_date = APPROXIMATE_DATEVER_REGEX.get_or_init(
            Self::init_approximate_datever_pattern
        );

        if apparent_date.is_match(text) {
            match DateVersion::parse(text) {
                Ok(date) => Ok(Self::Date(date)),
                Err(e) => Err(
                    ResourceVersionReqError::InvalidDateVersionRequirement { source: e }
                ),
            }
        } else if apparent_semver.is_match(text) {
            match SemanticVersionReq::parse(text) {
                Ok(req) => Ok(Self::Semantic(req)),
                Err(e) => Err(
                    ResourceVersionReqError::InvalidSemanticVersionRequirement { source: e }
                ),
            }
        } else {
            Err(ResourceVersionReqError::UnparseableRequirement { text: text.to_string() })
        }
    }

    /// Indicates whether the resource version requirement is semantic.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use dsc_lib::types::ResourceVersionReq;
    ///
    /// let semantic = ResourceVersionReq::parse("^1.2, <1.5").unwrap();
    /// let date = ResourceVersionReq::parse("2026-01-15").unwrap();
    ///
    /// assert_eq!(semantic.is_semver(), true);
    /// assert_eq!(date.is_semver(), false);
    /// ```
    pub fn is_semver(&self) -> bool {
        matches!(self, Self::Semantic(_))
    }

    /// Indicates whether the resource version requirement is for a specific [`DateVersion`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// use dsc_lib::types::ResourceVersionReq;
    ///
    /// let date = ResourceVersionReq::parse("2026-01-15").unwrap();
    /// let semantic = ResourceVersionReq::parse("^1.2, <1.5").unwrap();
    ///
    /// assert_eq!(date.is_date_version(), true);
    /// assert_eq!(semantic.is_date_version(), false);
    /// ```
    pub fn is_date_version(&self) -> bool {
        matches!(self, Self::Date(_))
    }

    /// Returns the requirement as a reference to the underlying [`SemanticVersionReq`] if possible.
    ///
    /// If the underlying requirement is [`Semantic`], this method returns some semantic version
    /// requirement. Otherwise, it returns [`None`].
    ///
    /// # Examples
    ///
    /// The following examples show how `as_semver_req()` behaves for different requirements.
    ///
    /// ```rust
    /// use dsc_lib::types::{ResourceVersionReq, SemanticVersionReq};
    ///
    /// let semantic = ResourceVersionReq::parse("^1.2.3").unwrap();
    /// let date = ResourceVersionReq::parse("2026-01-15").unwrap();
    ///
    /// assert_eq!(
    ///     semantic.as_semver_req(),
    ///     Some(&SemanticVersionReq::parse("^1.2.3").unwrap())
    /// );
    /// assert_eq!(
    ///     date.as_semver_req(),
    ///     None
    /// );
    /// ```
    ///
    /// [`Semantic`]: ResourceVersionReq::Semantic
    pub fn as_semver_req(&self) -> Option<&SemanticVersionReq> {
        match self {
            Self::Semantic(req) => Some(req),
            _ => None,
        }
    }

    /// Returns the requirement as a reference to the underlying [`DateVersion`] if possible.
    ///
    /// If the underlying requirement is [`Date`], this method returns some date version.
    /// Otherwise, it returns [`None`].
    ///
    /// # Examples
    ///
    /// The following examples show how `as_date_version()` behaves for different requirements.
    ///
    /// ```rust
    /// use dsc_lib::types::{ResourceVersionReq, DateVersion};
    ///
    /// let semantic = ResourceVersionReq::parse("^1.2.3").unwrap();
    /// let date = ResourceVersionReq::parse("2026-01-15").unwrap();
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
    /// [`Date`]: ResourceVersionReq::Date
    pub fn as_date_version(&self) -> Option<&DateVersion> {
        match self {
            Self::Date(req) => Some(req),
            _ => None,
        }
    }

    /// Compares an instance of [`ResourceVersion`] to the requirement, returning `true` if the
    /// version is valid for the requirement and otherwise `false`.
    ///
    /// The comparison depends on whether the requirement and version are semantic or arbitrary:
    ///
    /// - When both the requirement and version are semantic, this function uses the logic for
    ///   comparing versions and requirements defined by [`SemanticVersionReq`].
    /// - When both the requirement and version are date versions, the version is only valid for the
    ///   requirement when it is exactly the same date version as the requirement.
    /// - Otherwise, this function returns `false` because a date version can never match a
    ///   semantic requirement and a semantic version can never match a date version requirement.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use dsc_lib::types::{ResourceVersion, ResourceVersionReq};
    ///
    /// let semantic_req = ResourceVersionReq::parse("^1.2.3, <1.5").unwrap();
    /// assert!(semantic_req.matches(&ResourceVersion::parse("1.2.3").unwrap()));
    /// assert!(semantic_req.matches(&ResourceVersion::parse("1.3.0").unwrap()));
    /// assert!(!semantic_req.matches(&ResourceVersion::parse("1.0.0").unwrap()));
    /// assert!(!semantic_req.matches(&ResourceVersion::parse("1.5.0").unwrap()));
    /// assert!(!semantic_req.matches(&ResourceVersion::parse("2026-02-15").unwrap()));
    ///
    /// let date_req = ResourceVersionReq::parse("2026-02-15").unwrap();
    /// assert!(date_req.matches(&ResourceVersion::parse("2026-02-15").unwrap()));
    /// assert!(!date_req.matches(&ResourceVersion::parse("2026-02-15-preview").unwrap()));
    /// assert!(!date_req.matches(&ResourceVersion::parse("2026-02-01").unwrap()));
    /// ```
    pub fn matches(&self, resource_version: &ResourceVersion) -> bool {
        match self {
            Self::Semantic(req) => {
                match resource_version {
                    ResourceVersion::Semantic(version) => req.matches(version),
                    ResourceVersion::Date(_) => false,
                }
            },
            Self::Date(req) => {
                match resource_version {
                    ResourceVersion::Semantic(_) => false,
                    ResourceVersion::Date(version) => req == version,
                }
            }
        }
    }

    /// Defines the regex pattern that the [`ResourceVersionReq::parse()`] method uses to check
    /// whether to parse an input string as a semantic version requirement. If the input string
    /// defines a comparator operator or contains a string that _looks like_ a version, this
    /// pattern matches.
    const APPROXIMATE_SEMVER_PATTERN: &str = const_str::concat!(
        SemanticVersionReq::OPERATOR_PATTERN,   // Any operator
        "|",                                    // Or
        r"(?:\d+|[\*xX])(?:\.(?:\d+|[\*xX]))?"  // Shaped like a version
    );

    /// Returns the [`Regex`] for [`APPROXIMATE_SEMVER_PATTERN`].
    ///
    /// This private method is used to initialize the [`APPROXIMATE_SEMVER_REGEX`]
    /// private static to reduce the number of times the regular expression is compiled from the
    /// pattern string.
    ///
    /// [`APPROXIMATE_SEMVER_PATTERN`]: ResourceVersionReq::APPROXIMATE_SEMVER_PATTERN
    fn init_approximate_semver_pattern() -> Regex {
        Regex::new(Self::APPROXIMATE_SEMVER_PATTERN).expect("pattern is valid")
    }

    /// Defines the regex pattern that the [`ResourceVersionReq::parse()`] method uses to check
    /// whether to parse an input string as a date version requirement. If the input string
    /// starts with numbers followed by a hyphen and more numbers, this pattern matches.
    const APPROXIMATE_DATEVER_PATTERN: &str = r"^\d+-\d+";

    /// Returns the [`Regex`] for [`APPROXIMATE_DATEVER_PATTERN`].
    ///
    /// This private method is used to initialize the [`APPROXIMATE_DATEVER_REGEX`]
    /// private static to reduce the number of times the regular expression is compiled from the
    /// pattern string.
    ///
    /// [`APPROXIMATE_DATEVER_PATTERN`]: ResourceVersionReq::APPROXIMATE_DATEVER_PATTERN
    fn init_approximate_datever_pattern() -> Regex {
        Regex::new(Self::APPROXIMATE_DATEVER_PATTERN).expect("pattern is valid")
    }
}

// Default to matching any stable semantic version rather than an empty string.
impl Default for ResourceVersionReq {
    fn default() -> Self {
        Self::Semantic(SemanticVersionReq::default())
    }
}

// Enable using `ResourceVersionReq` in `format!` and similar macros.
impl Display for ResourceVersionReq {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Semantic(req) => write!(f, "{}", req),
            Self::Date(v) => write!(f, "{}", v),
        }
    }
}

// Parsing from a string delegates t0 `Self::parse()`
impl FromStr for ResourceVersionReq {
    type Err = ResourceVersionReqError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

// Implemented various conversion traits to move between `ResourceVersionReq`, `SemanticVersionReq`,
// `String`, and string slice (`str`).
impl TryFrom<String> for ResourceVersionReq {
    type Error = ResourceVersionReqError;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::parse(value.as_str())
    }
}

impl From<ResourceVersionReq> for String {
    fn from(value: ResourceVersionReq) -> Self {
        value.to_string()
    }
}

impl TryFrom<&String> for ResourceVersionReq {
    type Error = ResourceVersionReqError;
    fn try_from(value: &String) -> Result<Self, Self::Error> {
        Self::parse(value.as_str())
    }
}
// We can't bidirectionally convert string slices, because we can't return a temporary reference.
// We can still convert _from_ string slices, but not _into_ them.
impl TryFrom<&str> for ResourceVersionReq {
    type Error = ResourceVersionReqError;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::parse(value)
    }
}

impl From<SemanticVersionReq> for ResourceVersionReq {
    fn from(value: SemanticVersionReq) -> Self {
        Self::Semantic(value)
    }
}

impl From<&SemanticVersionReq> for ResourceVersionReq {
    fn from(value: &SemanticVersionReq) -> Self {
        Self::Semantic(value.clone())
    }
}

// Creating an instance of `SemanticVersionReq` from `ResourceVersionReq` is a fallible conversion,
// since `ResourceVersionReq` can define non-semantic version requirements.
impl TryFrom<ResourceVersionReq> for SemanticVersionReq {
    type Error = ResourceVersionReqError;

    fn try_from(value: ResourceVersionReq) -> Result<Self, Self::Error> {
        match value {
            ResourceVersionReq::Semantic(req) => Ok(req),
            ResourceVersionReq::Date(req) => Err(
                ResourceVersionReqError::InvalidConversionToSemanticVersionReq { req }
            ),
        }
    }
}


impl From<DateVersion> for ResourceVersionReq {
    fn from(value: DateVersion) -> Self {
        Self::Date(value)
    }
}

impl From<&DateVersion> for ResourceVersionReq {
    fn from(value: &DateVersion) -> Self {
        Self::Date(value.clone())
    }
}

// Creating an instance of `DateVersion` from `ResourceVersionReq` is a fallible conversion,
// since `ResourceVersionReq` can define semantic version requirements.
impl TryFrom<ResourceVersionReq> for DateVersion {
    type Error = ResourceVersionReqError;

    fn try_from(value: ResourceVersionReq) -> Result<Self, Self::Error> {
        match value {
            ResourceVersionReq::Date(d) => Ok(d),
            ResourceVersionReq::Semantic(req) => Err(
                ResourceVersionReqError::InvalidConversionToDateVersion { req }
            ),
        }
    }
}

impl Hash for ResourceVersionReq {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            Self::Semantic(req) => req.hash(state),
            Self::Date(req) => req.hash(state),
        }
    }
}

// Implement traits for comparing `ResourceVersionReq` to strings and semantic version requirements
// bi-directionally.
impl PartialEq for ResourceVersionReq {
    fn eq(&self, other: &Self) -> bool {
        match self {
            Self::Semantic(req) => match other {
                Self::Semantic(other_req) => req == other_req,
                _ => false
            },
            Self::Date(date) => match other {
                Self::Date(other_date) => date == other_date,
                _ => false,
            }
        }
    }
}

impl PartialEq<SemanticVersionReq> for ResourceVersionReq {
    fn eq(&self, other: &SemanticVersionReq) -> bool {
        match self {
            Self::Semantic(req) => req == other,
            _ => false,
        }
    }
}

impl PartialEq<ResourceVersionReq> for SemanticVersionReq {
    fn eq(&self, other: &ResourceVersionReq) -> bool {
        match other {
            ResourceVersionReq::Semantic(req) => self == req,
            _ => false,
        }
    }
}

impl PartialEq<DateVersion> for ResourceVersionReq {
    fn eq(&self, other: &DateVersion) -> bool {
        match self {
            Self::Date(req) => req == other,
            _ => false,
        }
    }
}

impl PartialEq<ResourceVersionReq> for DateVersion {
    fn eq(&self, other: &ResourceVersionReq) -> bool {
        match other {
            ResourceVersionReq::Date(req) => self == req,
            _ => false,
        }
    }
}

impl PartialEq<&str> for ResourceVersionReq {
    fn eq(&self, other: &&str) -> bool {
        match Self::parse(other) {
            Ok(other_req) => self == &other_req,
            Err(_) => false,
        }
    }
}

impl PartialEq<ResourceVersionReq> for &str {
    fn eq(&self, other: &ResourceVersionReq) -> bool {
        match ResourceVersionReq::parse(self) {
            Ok(req) => &req == other,
            Err(_) => false,
        }
    }
}

impl PartialEq<str> for ResourceVersionReq {
    fn eq(&self, other: &str) -> bool {
        match Self::parse(other) {
            Ok(other_req) => self == &other_req,
            Err(_) => false,
        }
    }
}

impl PartialEq<ResourceVersionReq> for str {
    fn eq(&self, other: &ResourceVersionReq) -> bool {
        match ResourceVersionReq::parse(self) {
            Ok(req) => &req == other,
            Err(_) => false,
        }
    }
}

impl PartialEq<String> for ResourceVersionReq {
    fn eq(&self, other: &String) -> bool {
        match Self::parse(other.as_str()) {
            Ok(other_req) => self == &other_req,
            Err(_) => false,
        }
    }
}

impl PartialEq<ResourceVersionReq> for String {
    fn eq(&self, other: &ResourceVersionReq) -> bool {
        match ResourceVersionReq::parse(self.as_str()) {
            Ok(req) => &req == other,
            Err(_) => false,
        }
    }
}
