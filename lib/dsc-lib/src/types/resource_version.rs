// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use std::{convert::Infallible, fmt::Display, str::FromStr};

use rust_i18n::t;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{dscerror::DscError, schemas::dsc_repo::DscRepoSchema, types::{SemanticVersion, SemanticVersionReq}};

/// Defines the version of a DSC resource.
///
/// DSC supports both semantic versioning and arbitrary versioning for resources. Semantic
/// versioning is the preferred and recommended versioning strategy. DSC only supports arbitrary
/// versioning for compatibility scenarios.
///
/// When the version is defined as a valid semantic version ([`ResourceVersion::Semantic`]), DSC
/// can correctly compare versions to determine the latest version or match a
/// [`SemanticVersionReq`]. Where possible, resource and extension authors should follow semantic
/// versioning for the best user experience.
///
/// When the version is an arbitrary string (`ResourceVersion::Arbitrary`]), DSC compares the strings
/// using Rust's default string comparison logic. This means that arbitrary string versions are
/// compared [lexicographically][01]. Arbitrary string versions are only equivalent when they
/// contain exactly the same characters - the comparison is case-sensitive. If you're defining a
/// resource that doesn't follow semantic versioning, consider defining the version as an
/// [ISO 8601 date][02], like `2026-01-15`. When you do, DSC can correctly determine that a later
/// date should be treated as a newer version.
///
/// # Examples
///
/// The following example shows how different instances of [`ResourceVersion`] compare to other
/// instances of `ResourceVersion`, [`SemanticVersion`], [`String`], and [`str`].
///
/// ```rust
/// use dsc_lib::types::{ResourceVersion, SemanticVersion};
///
/// let semantic = ResourceVersion::new("1.2.3");
/// let arbitrary = ResourceVersion::new("Foo");
/// let date = ResourceVersion::new("2026-01-15");
///
/// // You can compare instances of `ResourceVersion::Semantic` to strings, string slices, and
/// // semantic versions.
/// assert_eq!(semantic, SemanticVersion::parse("1.2.3").unwrap());
/// assert_eq!(semantic, "1.2.3");
/// assert_ne!(semantic, "1.2.*".to_string());
///
/// // When comparing arbitrary string versions to strings, you can compare `String` instances and
/// // literal strings. The comparisons are case-sensitive.
/// assert_eq!(arbitrary, "Foo");
/// assert_ne!(arbitrary, "foo".to_string());
///
/// // When a semantic version is compared to an arbitrary string version, the semantic version is
/// // always treated as being higher:
/// assert!(semantic > arbitrary);
/// assert!(semantic > date);
/// assert!(arbitrary < SemanticVersion::parse("0.1.0").unwrap());
///
/// // Semantic version comparisons work as expected.
/// assert!(semantic < SemanticVersion::parse("1.2.4").unwrap());
/// assert!(semantic >= SemanticVersion::parse("1.0.0").unwrap());
///
/// // When comparing a semantic version to a string, the comparison uses semantic version ordering
/// // if the string can be parsed as a semantic version.
/// assert!(semantic < "1.2.4");
/// assert!(semantic > "foo".to_string());
///
/// // Arbitrary string version comparisons are lexicographic. DSC has no way of knowing whether
/// // `Bar` should be treated as a newer version than `Foo`:
/// assert!(arbitrary <= "foo");
/// assert_ne!(arbitrary < "Bar", true);
///
/// // String version comparisons for ISO 8601 dates are deterministic:
/// assert!(date < "2026-02-01");
/// assert!(date >= "2026-01");
/// ```
///
/// You can freely convert between strings and `ResourceVersion`:
///
/// ```rust
/// use dsc_lib::types::ResourceVersion;
///
/// let semantic: ResourceVersion = "1.2.3".parse().unwrap();
/// let arbitrary = ResourceVersion::from("foo");
/// let date = ResourceVersion::new("2026-01-15");
///
/// let stringified_semantic = String::from(semantic.clone());
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
#[serde(untagged)]
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
    /// Defines the resource's version as an arbitrary string.
    ///
    /// DSC uses this variant for the version of any DSC resource that defines its
    /// version as a string that can't be parsed as a semantic version. This variant remains
    /// supported for compatibility purposes but is _not_ recommended for production usage.
    ///
    /// When a resource defines the version as an arbitrary string:
    ///
    /// 1. You can only use exact match version requirements for that resource.
    /// 1. When a resource defines the version as an arbitrary string, DSC uses Rust's
    ///    [lexicographic comparison][01] logic to determine the "latest" version of the resource
    ///    to use as the default version when no version requirement is specified.
    /// 1. When DSC discovers a multiple manifests for a resource, DSC always treats semantically
    ///    versioned resources as newer than resources with an arbitrary string version.
    ///
    /// [01]: https://doc.rust-lang.org/std/cmp/trait.Ord.html#lexicographical-comparison
    #[schemars(
        title = t!("schemas.definitions.resourceVersion.arbitraryVariant.title"),
        description = t!("schemas.definitions.resourceVersion.arbitraryVariant.description"),
        extend(
            "deprecated" = true,
            "deprecationMessage" = t!("schemas.definitions.resourceVersion.arbitraryVariant.deprecationMessage"),
            "markdownDescription" = t!("schemas.definitions.resourceVersion.arbitraryVariant.markdownDescription"),
        )
    )]
    Arbitrary(String),
}

impl ResourceVersion {
    /// Creates a new instance of [`ResourceVersion`].
    ///
    /// If the input string is a valid semantic version, the function returns the [`Semantic`]
    /// variant. Otherwise, the function returns the [`Arbitrary`] variant for arbitrary version
    /// strings.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use dsc_lib::types::ResourceVersion;
    ///
    /// fn print_version_message(version: ResourceVersion) {
    ///     match ResourceVersion::new("1.2.3") {
    ///         ResourceVersion::Semantic(v) => println!("Semantic version: {v}"),
    ///         ResourceVersion::Arbitrary(s) => println!("Arbitrary string version: '{s}'"),
    ///     }
    /// }
    ///
    /// // Print for semantic version
    /// print_version_message(ResourceVersion::new("1.2.3"));
    ///
    /// // Print for arbitrary version
    /// print_version_message(ResourceVersion::new("2026-01"));
    /// ```
    ///
    /// [`Semantic`]: ResourceVersion::Semantic
    /// [`Arbitrary`]: ResourceVersion::Arbitrary
    pub fn new(version_string: &str) -> Self {
        match SemanticVersion::parse(version_string) {
            Ok(v) => Self::Semantic(v),
            Err(_) => Self::Arbitrary(version_string.to_string()),
        }
    }

    /// Indicates whether the resource version is semantic.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use dsc_lib::types::ResourceVersion;
    ///
    /// let semantic = ResourceVersion::new("1.2.3");
    /// let arbitrary = ResourceVersion::new("2026-01");
    ///
    /// assert_eq!(semantic.is_semver(), true);
    /// assert_eq!(arbitrary.is_semver(), false);
    /// ```
    pub fn is_semver(&self) -> bool {
        match self {
            Self::Semantic(_) => true,
            _ => false,
        }
    }

    /// Indicates whether the resource version is an arbitrary string.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use dsc_lib::types::ResourceVersion;
    ///
    /// let semantic = ResourceVersion::new("1.2.3");
    /// let arbitrary = ResourceVersion::new("2026-01");
    ///
    /// assert_eq!(semantic.is_semver(), true);
    /// assert_eq!(arbitrary.is_semver(), false);
    /// ```
    pub fn is_arbitrary(&self) -> bool {
        match self {
            Self::Arbitrary(_) => true,
            _ => false,
        }
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
    /// let semantic = ResourceVersion::new("1.2.3");
    /// let date = ResourceVersion::new("2026-01-15");
    /// let arbitrary = ResourceVersion::new("arbitrary_version");
    ///
    /// assert_eq!(
    ///     semantic.as_semver(),
    ///     Some(&SemanticVersion::new(1, 2, 3))
    /// );
    /// assert_eq!(
    ///     date.as_semver(),
    ///     None
    /// );
    /// assert_eq!(
    ///     arbitrary.as_semver(),
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

    /// Compares an instance of [`ResourceVersion`] with [`SemanticVersionReq`].
    ///
    /// When the instance is [`ResourceVersion::Semantic`], this method applies the canonical matching
    /// logic from [`SemanticVersionReq`] for the version. When the instance is
    /// [`ResourceVersion::Arbitrary`], this method always returns `false`.
    ///
    /// For more information about semantic version requirements and syntax, see
    /// ["Specifying Dependencies" in _The Cargo Book_][semver-req].
    ///
    /// # Examples
    ///
    /// The following example shows how comparisons work for different instances of
    /// [`ResourceVersion`].
    ///
    /// ```rust
    /// use dsc_lib::types::{ResourceVersion, SemanticVersionReq};
    ///
    /// let semantic = ResourceVersion::new("1.2.3");
    /// let date = ResourceVersion::new("2026-01-15");
    ///
    /// let ref le_v2_0: SemanticVersionReq = "<=2.0".parse().unwrap();
    /// assert!(semantic.matches_semver_req(le_v2_0));
    /// assert!(!date.matches_semver_req(le_v2_0));
    /// let ref tilde_v1: SemanticVersionReq = "~1".parse().unwrap();
    /// assert!(semantic.matches_semver_req(tilde_v1));
    /// assert!(!date.matches_semver_req(tilde_v1));
    /// ```
    ///
    /// [semver-req]: https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html#version-requirement-syntax
    pub fn matches_semver_req(&self, requirement: &SemanticVersionReq) -> bool {
        match self {
            Self::Semantic(v) => requirement.matches(v),
            Self::Arbitrary(_) => false,
        }
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
            Self::Arbitrary(s) => write!(f, "{}", s),
        }
    }
}

// Parsing from a string is just calling `Self::new()`
impl FromStr for ResourceVersion {
    type Err = Infallible;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::new(s))
    }
}

// Implemented various conversion traits to move between `ResourceVersion`, `SemanticVersion`,
// `String`, and string slice (`str`).
impl From<&String> for ResourceVersion {
    fn from(value: &String) -> Self {
        match SemanticVersion::parse(value) {
            Ok(v) => ResourceVersion::Semantic(v),
            Err(_) => ResourceVersion::Arbitrary(value.clone()),
        }
    }
}

impl From<String> for ResourceVersion {
    fn from(value: String) -> Self {
        match SemanticVersion::parse(&value) {
            Ok(v) => ResourceVersion::Semantic(v),
            Err(_) => ResourceVersion::Arbitrary(value),
        }
    }
}

impl From<ResourceVersion> for String {
    fn from(value: ResourceVersion) -> Self {
        value.to_string()
    }
}

// We can't bidirectionally convert string slices, because we can't return a temporary reference.
// We can still convert _from_ string slices, but not _into_ them.
impl From<&str> for ResourceVersion {
    fn from(value: &str) -> Self {
        ResourceVersion::from(value.to_string())
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

// Creating an instance of `SemanticVersion` from `ResourceVersion` is the only fallible
// conversion, since `ResourceVersion` can define non-semantic versions.
impl TryFrom<ResourceVersion> for SemanticVersion {
    type Error = DscError;

    fn try_from(value: ResourceVersion) -> Result<Self, Self::Error> {
        match value {
            ResourceVersion::Semantic(v) => Ok(v),
            ResourceVersion::Arbitrary(s) => Err(DscError::ResourceVersionToSemverConversion(s)),
        }
    }
}

// Implement traits for comparing `ResourceVersion` to strings and semantic versions bi-directionally.
impl PartialEq for ResourceVersion {
    fn eq(&self, other: &Self) -> bool {
        match self {
            Self::Semantic(version) => match other {
                Self::Semantic(other_version) => version == other_version,
                Self::Arbitrary(_) => false,
            },
            Self::Arbitrary(string) => !other.is_semver() && *string == other.to_string(),
        }
    }
}

impl PartialEq<SemanticVersion> for ResourceVersion {
    fn eq(&self, other: &SemanticVersion) -> bool {
        match self {
            Self::Semantic(v) => v == other,
            Self::Arbitrary(_) => false,
        }
    }
}

impl PartialEq<ResourceVersion> for SemanticVersion {
    fn eq(&self, other: &ResourceVersion) -> bool {
        match other {
            ResourceVersion::Semantic(v) => self == v,
            ResourceVersion::Arbitrary(_) => false,
        }
    }
}

impl PartialEq<&str> for ResourceVersion {
    fn eq(&self, other: &&str) -> bool {
        self == &Self::new(*other)
    }
}

impl PartialEq<ResourceVersion> for &str {
    fn eq(&self, other: &ResourceVersion) -> bool {
        &ResourceVersion::new(self) == other
    }
}

impl PartialEq<String> for ResourceVersion {
    fn eq(&self, other: &String) -> bool {
        self == &Self::new(other)
    }
}

impl PartialEq<ResourceVersion> for String {
    fn eq(&self, other: &ResourceVersion) -> bool {
        &ResourceVersion::new(self) == other
    }
}

impl PartialEq<str> for ResourceVersion {
    fn eq(&self, other: &str) -> bool {
        self == &Self::new(other)
    }
}

impl PartialEq<ResourceVersion> for str {
    fn eq(&self, other: &ResourceVersion) -> bool {
        &ResourceVersion::new(self) == other
    }
}

impl PartialOrd for ResourceVersion {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match self {
            Self::Semantic(version) => match other {
                Self::Semantic(other_version) => version.partial_cmp(other_version),
                Self::Arbitrary(_) => Some(std::cmp::Ordering::Greater),
            },
            Self::Arbitrary(string) => match other {
                Self::Semantic(_) => Some(std::cmp::Ordering::Less),
                Self::Arbitrary(other_string) => string.partial_cmp(other_string),
            },
        }
    }
}

impl PartialOrd<SemanticVersion> for ResourceVersion {
    fn partial_cmp(&self, other: &SemanticVersion) -> Option<std::cmp::Ordering> {
        match self {
            Self::Semantic(v) => v.partial_cmp(other),
            Self::Arbitrary(_) => Some(std::cmp::Ordering::Less),
        }
    }
}

impl PartialOrd<ResourceVersion> for SemanticVersion {
    fn partial_cmp(&self, other: &ResourceVersion) -> Option<std::cmp::Ordering> {
        match other {
            ResourceVersion::Semantic(v) => self.partial_cmp(v),
            ResourceVersion::Arbitrary(_) => Some(std::cmp::Ordering::Greater),
        }
    }
}

impl PartialOrd<String> for ResourceVersion {
    fn partial_cmp(&self, other: &String) -> Option<std::cmp::Ordering> {
        self.partial_cmp(&ResourceVersion::new(other.as_str()))
    }
}

impl PartialOrd<ResourceVersion> for String {
    fn partial_cmp(&self, other: &ResourceVersion) -> Option<std::cmp::Ordering> {
        ResourceVersion::new(self.as_str()).partial_cmp(other)
    }
}

impl PartialOrd<&str> for ResourceVersion {
    fn partial_cmp(&self, other: &&str) -> Option<std::cmp::Ordering> {
        self.partial_cmp(&Self::new(other))
    }
}

impl PartialOrd<str> for ResourceVersion {
    fn partial_cmp(&self, other: &str) -> Option<std::cmp::Ordering> {
        self.partial_cmp(&Self::new(other))
    }
}

impl PartialOrd<ResourceVersion> for &str {
    fn partial_cmp(&self, other: &ResourceVersion) -> Option<std::cmp::Ordering> {
        ResourceVersion::new(self).partial_cmp(other)
    }
}

impl PartialOrd<ResourceVersion> for str {
    fn partial_cmp(&self, other: &ResourceVersion) -> Option<std::cmp::Ordering> {
        ResourceVersion::new(self).partial_cmp(other)
    }
}

// Manually implement total ordering, because partial and total ordering are different for semantic
// versions. See the implementation on `semver::Version` for details.
impl Ord for ResourceVersion {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self {
            Self::Semantic(version) => match other {
                Self::Semantic(other_version) => version.cmp(other_version),
                Self::Arbitrary(_) => std::cmp::Ordering::Greater,
            },
            Self::Arbitrary(version) => match other {
                Self::Semantic(_) => std::cmp::Ordering::Less,
                Self::Arbitrary(other_version) => version.cmp(other_version),
            }
        }
    }
}
