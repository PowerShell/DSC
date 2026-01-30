// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use std::{convert::Infallible, fmt::Display, str::FromStr};

use crate::{dscerror::DscError, schemas::dsc_repo::DscRepoSchema};
use rust_i18n::t;
use schemars::{json_schema, JsonSchema};
use serde::{Deserialize, Serialize};

/// Defines the version of a DSC resource or extension.
///
/// DSC supports both semantic versioning and arbitrary versioning for types. When the version is
/// defined as a valid semantic version ([`TypeVersion::Semantic`]), DSC can correctly compare
/// versions to determine the latest version or match a semantic version requirement. Where
/// possible, resource and extension authors should consider following semantic versioning for the
/// best user experience.
///
/// When the version is an arbitrary string, DSC compares the strings after lower-casing them. If
/// a type defines the current version as `Foo` and the next version as `Bar`, DSC's comparison
/// logic will treat `Foo` as newer than `Bar`. If you're defining a type that doesn't follow
/// semantic versioning, consider defining the version as an [ISO 8601 date], like `2026-01-15`.
/// When you do, DSC can correctly determine that a later date should be treated as a newer version.
///
/// # Examples
///
/// The following example shows how different instances of [`TypeVersion`] compare to other
/// instances of `TypeVersion`, [`String`], and [`semver::Version`].
///
/// ```rust
/// use dsc_lib::TypeVersion;
/// use semver::Version;
///
/// let semantic = TypeVersion::new("1.2.3");
/// let arbitrary = TypeVersion::new("Foo");
/// let date = TypeVersion::new("2026-01-15");
///
/// // You can compare instances of `TypeVersion::Semantic` to strings and semantic versions.
/// assert_eq!(semantic, semver::Version::parse("1.2.3").unwrap());
/// assert_eq!(semantic, "1.2.3");
///
/// // When comparing to strings, you can compare `String` instances and literal strings. Casing
/// // is ignored for these comparisons.
/// assert_eq!(arbitrary, "Foo");
/// assert_eq!(arbitrary, "foo".to_string());
///
/// // When a semantic version is compared to an arbitrary string version, the semantic version is
/// // always treated as being higher:
/// assert!(semantic > arbitrary);
/// assert!(semantic > date);
/// assert!(arbitrary < semver::Version::parse("0.1.0").unwrap());
///
/// // Semantic version comparisons work as expected.
/// assert!(semantic < semver::Version::parse("1.2.4").unwrap());
/// assert!(semantic >= semver::Version::parse("1.0.0").unwrap());
///
/// // String version comparisons are case-insensitive but rely on Rust's underlying string
/// // comparison logic. DSC has no way of knowing whether `Bar` should be treated as a newer
/// // version than `Foo`:
/// assert!(arbitrary >= "foo");
/// assert_ne!(arbitrary < "Bar", true);
///
/// // String version comparisons for ISO 8601 dates are deterministic:
/// assert!(date < "2026-02-01");
/// assert!(date >= "2026-01");
/// ```
///
/// You can freely convert between strings and `TypeVersion`:
///
/// ```rust
/// use dsc_lib::TypeVersion;
///
/// let semantic: TypeVersion = "1.2.3".parse().unwrap();
/// let arbitrary = TypeVersion::from("foo");
/// let date = TypeVersion::new("2026-01-15");
///
/// let stringified_semantic = String::from(semantic.clone());
///
/// // Define a function that expects a string:
/// fn expects_string(input: &str) {
///     println!("Input was: '{input}'")
/// }
///
/// // You can pass the `TypeVersion` in a few ways:
/// expects_string(&semantic.to_string());
/// expects_string(date.to_string().as_str());
/// ```
///
/// [ISO 8601 date]: https://www.iso.org/iso-8601-date-and-time-format.html
#[derive(Debug, Clone, Eq, Ord, Serialize, Deserialize, JsonSchema, DscRepoSchema)]
#[dsc_repo_schema(base_name = "version", folder_path = "definitions")]
#[serde(untagged)]
#[schemars(
    title = t!("schemas.definitions.version.title"),
    description = t!("schemas.definitions.version.description"),
    extend(
        "markdownDescription" = t!("schemas.definitions.version.markdownDescription")
    )
)]
pub enum TypeVersion {
    /// Defines the type's version as a semantic version, containing an inner [`semver::Version`].
    /// This is the preferred and recommended versioning scheme for DSC resources and extensions.
    ///
    /// For more information about defining semantic versions, see [semver.org].
    ///
    /// [semver.org]: https://semver.org
    #[schemars(schema_with = "TypeVersion::semver_schema")]
    Semantic(semver::Version),
    /// Defines the type's version as an arbitrary string.
    ///
    /// DSC uses this variant for the version of any DSC resource or extension that defines its
    /// version as a string that can't be parsed as a semantic version.
    ///
    /// If you're defining a version for a resource or extension that doesn't use semantic
    /// versioning, consider specifying the version as an [ISO-8601 date], like `2026-01-01`. When
    /// you do, DSC can still correctly discover the latest version for that type by string
    /// comparisons.
    ///
    /// [ISO 8601 date]: https://www.iso.org/iso-8601-date-and-time-format.html
    #[schemars(
        title = t!("schemas.definitions.version.stringVariant.title"),
        description = t!("schemas.definitions.version.stringVariant.description"),
        extend(
            "markdownDescription" = t!("schemas.definitions.version.stringVariant.markdownDescription")
        )
    )]
    String(String),
}

impl TypeVersion {
    /// Defines the validating regular expression for semantic versions.
    ///
    /// This regular expression was retrieved from [semver.org] and is used for the `pattern`
    /// keyword in the JSON Schema for the semantic version variant ([`TypeVersion::Semantic`]).
    ///
    /// The pattern isn't used for validating an instance during or after deserialization. Instead,
    /// it provides author-time feedback to manifest maintainers so they can avoid runtime failures.
    ///
    /// During deserialization, the library first tries to parse the string as a semantic version.
    /// If the value parses successfully, it's deserialized as a [`TypeVersion::Semantic`] instance.
    /// Otherwise, it's deserialized as a [`TypeVersion::String`] instance.
    ///
    /// [semver.org]: https://semver.org/#is-there-a-suggested-regular-expression-regex-to-check-a-semver-string
    const SEMVER_PATTERN: &str = r"^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)(?:-((?:0|[1-9]\d*|\d*[a-zA-Z-][0-9a-zA-Z-]*)(?:\.(?:0|[1-9]\d*|\d*[a-zA-Z-][0-9a-zA-Z-]*))*))?(?:\+([0-9a-zA-Z-]+(?:\.[0-9a-zA-Z-]+)*))?$";

    /// Creates a new instance of [`TypeVersion`].
    ///
    /// If the input string is a valid semantic version, the function returns the [`Semantic`]
    /// variant. Otherwise, the function returns the [`String`] variant for arbitrary version
    /// strings.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use dsc_lib::TypeVersion;
    ///
    /// fn print_version_message(version: TypeVersion) {
    ///     match TypeVersion::new("1.2.3") {
    ///         TypeVersion::Semantic(v) => println!("Semantic version: {v}"),
    ///         TypeVersion::String(s) => println!("Arbitrary string version: '{s}'"),
    ///     }
    /// }
    ///
    /// // Print for semantic version
    /// print_version_message(TypeVersion::new("1.2.3"));
    ///
    /// // Print for arbitrary version
    /// print_version_message(TypeVersion::new("2026-01"));
    /// ```
    ///
    /// [`Semantic`]: TypeVersion::Semantic
    /// [`String`]: TypeVersion::String
    pub fn new(version_string: &str) -> Self {
        Self::from_str(version_string).unwrap()
    }

    /// Indicates whether the version is semantic or an arbitrary string.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use dsc_lib::TypeVersion;
    ///
    /// let semantic = TypeVersion::new("1.2.3");
    /// let arbitrary = TypeVersion::new("2026-01");
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

    /// Returns the version as a reference to the underlying [`semver::Version`] if possible.
    ///
    /// If the underlying version is [`Semantic`], this method returns some semantic version.
    /// Otherwise, it returns [`None`].
    /// 
    /// # Examples
    /// 
    /// The following examples show how `as_semver()` behaves for different versions.
    /// 
    /// ```rust
    /// use dsc_lib::TypeVersion;
    /// 
    /// let semantic = TypeVersion::new("1.2.3");
    /// let date = TypeVersion::new("2026-01-15");
    /// let arbitrary = TypeVersion::new("arbitrary_version");
    /// 
    /// assert_eq!(
    ///     semantic.as_semver(),
    ///     Some(&semver::Version::parse("1.2.3").unwrap())
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
    /// [`Semantic`]: TypeVersion::Semantic
    pub fn as_semver(&self) -> Option<&semver::Version> {
        match self {
            Self::Semantic(v) => Some(v),
            _ => None,
        }
    }

    /// Returns the JSON schema for semantic version strings.
    pub fn semver_schema(_: &mut schemars::SchemaGenerator) -> schemars::Schema {
        json_schema!({
            "title": t!("schemas.definitions.semver.title"),
            "description": t!("schemas.definitions.semver.description"),
            "markdownDescription": t!("schemas.definitions.semver.markdownDescription"),
            "type": "string",
            "pattern": TypeVersion::SEMVER_PATTERN,
            "patternErrorMessage": t!("schemas.definitions.semver.patternErrorMessage"),
        })
    }

    /// Compares an instance of [`TypeVersion`] with [`semver::VersionReq`].
    ///
    /// When the instance is [`TypeVersion::Semantic`], this method applies the canonical matching
    /// logic from [`semver`] for the version. When the instance is [`TypeVersion::String`], this
    /// method always returns `false`.
    ///
    /// For more information about semantic version requirements and syntax, see
    /// ["Specifying Dependencies" in _The Cargo Book_][semver-req].
    ///
    /// # Examples
    ///
    /// The following example shows how comparisons work for different instances of [`TypeVersion`].
    ///
    /// ```rust
    /// use dsc_lib::TypeVersion;
    /// use semver::VersionReq;
    ///
    /// let semantic = TypeVersion::new("1.2.3");
    /// let date = TypeVersion::new("2026-01-15");
    ///
    /// let ref le_v2_0: VersionReq = "<=2.0".parse().unwrap();
    /// assert!(semantic.matches_semver_req(le_v2_0));
    /// assert!(!date.matches_semver_req(le_v2_0));
    /// let ref tilde_v1: VersionReq = "~1".parse().unwrap();
    /// assert!(semantic.matches_semver_req(tilde_v1));
    /// assert!(!date.matches_semver_req(tilde_v1));
    /// ```
    ///
    /// [semver-req]: https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html#version-requirement-syntax
    pub fn matches_semver_req(&self, requirement: &semver::VersionReq) -> bool {
        match self {
            Self::Semantic(v) => requirement.matches(v),
            Self::String(_) => false,
        }
    }
}

// Default to semantic version `0.0.0` rather than an empty string.
impl Default for TypeVersion {
    fn default() -> Self {
        Self::Semantic(semver::Version {
            major: 0,
            minor: 0,
            patch: 0,
            pre: semver::Prerelease::EMPTY,
            build: semver::BuildMetadata::EMPTY,
        })
    }
}

// Enable using `TypeVersion` in `format!` and similar macros.
impl Display for TypeVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Semantic(v) => write!(f, "{}", v),
            Self::String(s) => write!(f, "{}", s),
        }
    }
}

// Parse a `TypeVersion` from a string literal
impl FromStr for TypeVersion {
    type Err = Infallible;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match semver::Version::parse(s) {
            Ok(v) => Ok(TypeVersion::Semantic(v)),
            Err(_) => Ok(TypeVersion::String(s.to_string())),
        }
    }
}

// Implemented various conversion traits to move between `TypeVersion`, `String`, and
// `semver::Version`.
impl From<&String> for TypeVersion {
    fn from(value: &String) -> Self {
        match semver::Version::parse(value) {
            Ok(v) => TypeVersion::Semantic(v),
            Err(_) => TypeVersion::String(value.clone()),
        }
    }
}

impl From<String> for TypeVersion {
    fn from(value: String) -> Self {
        match semver::Version::parse(&value) {
            Ok(v) => TypeVersion::Semantic(v),
            Err(_) => TypeVersion::String(value),
        }
    }
}

impl From<TypeVersion> for String {
    fn from(value: TypeVersion) -> Self {
        value.to_string()
    }
}

// We can't bidirectionally convert string slices, because we can't return a temporary reference.
// We can still convert _from_ string slices, but _into_ them.
impl From<&str> for TypeVersion {
    fn from(value: &str) -> Self {
        TypeVersion::from(value.to_string())
    }
}

impl From<semver::Version> for TypeVersion {
    fn from(value: semver::Version) -> Self {
        Self::Semantic(value)
    }
}
impl From<&semver::Version> for TypeVersion {
    fn from(value: &semver::Version) -> Self {
        Self::Semantic(value.clone())
    }
}

// Creating an instance of `semver::Version` from `TypeVersion` is the only fallible conversion,
// since `TypeVersion` can define non-semantic versions.
impl TryFrom<TypeVersion> for semver::Version {
    type Error = DscError;

    fn try_from(value: TypeVersion) -> Result<Self, Self::Error> {
        match value {
            TypeVersion::Semantic(v) => Ok(v),
            TypeVersion::String(s) => Err(DscError::TypeVersionToSemverConversion(s)),
        }
    }
}

// Implement traits for comparing `TypeVersion` to strings and semantic versions bi-directionally.
impl PartialEq for TypeVersion {
    fn eq(&self, other: &Self) -> bool {
        match self {
            Self::Semantic(version) => match other {
                Self::Semantic(other_version) => version == other_version,
                Self::String(_) => false,
            },
            Self::String(string) => {
                !other.is_semver() && *string.to_lowercase() == other.to_string().to_lowercase()
            }
        }
    }
}

impl PartialEq<semver::Version> for TypeVersion {
    fn eq(&self, other: &semver::Version) -> bool {
        match self {
            Self::Semantic(v) => v == other,
            Self::String(_) => false,
        }
    }
}

impl PartialEq<TypeVersion> for semver::Version {
    fn eq(&self, other: &TypeVersion) -> bool {
        match other {
            TypeVersion::Semantic(v) => self == v,
            TypeVersion::String(_) => false,
        }
    }
}

impl PartialEq<&str> for TypeVersion {
    fn eq(&self, other: &&str) -> bool {
        self.to_string().to_lowercase() == *other.to_lowercase()
    }
}

impl PartialEq<TypeVersion> for &str {
    fn eq(&self, other: &TypeVersion) -> bool {
        self.to_lowercase() == other.to_string().to_lowercase()
    }
}

impl PartialEq<String> for TypeVersion {
    fn eq(&self, other: &String) -> bool {
        self.to_string().to_lowercase() == other.to_lowercase()
    }
}

impl PartialEq<TypeVersion> for String {
    fn eq(&self, other: &TypeVersion) -> bool {
        self.to_lowercase() == other.to_string().to_lowercase()
    }
}

impl PartialEq<str> for TypeVersion {
    fn eq(&self, other: &str) -> bool {
        self.eq(&other.to_string())
    }
}

impl PartialEq<TypeVersion> for str {
    fn eq(&self, other: &TypeVersion) -> bool {
        self.to_string().eq(other)
    }
}

impl PartialOrd for TypeVersion {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match self {
            Self::Semantic(version) => match other {
                Self::Semantic(other_version) => version.partial_cmp(other_version),
                Self::String(_) => Some(std::cmp::Ordering::Greater),
            },
            Self::String(string) => match other {
                Self::Semantic(_) => Some(std::cmp::Ordering::Less),
                Self::String(other_string) => string
                    .to_lowercase()
                    .partial_cmp(&other_string.to_lowercase()),
            },
        }
    }
}

impl PartialOrd<semver::Version> for TypeVersion {
    fn partial_cmp(&self, other: &semver::Version) -> Option<std::cmp::Ordering> {
        match self {
            Self::Semantic(v) => v.partial_cmp(other),
            Self::String(_) => Some(std::cmp::Ordering::Less),
        }
    }
}

impl PartialOrd<TypeVersion> for semver::Version {
    fn partial_cmp(&self, other: &TypeVersion) -> Option<std::cmp::Ordering> {
        match other {
            TypeVersion::Semantic(v) => self.partial_cmp(v),
            TypeVersion::String(_) => Some(std::cmp::Ordering::Greater),
        }
    }
}

impl PartialOrd<String> for TypeVersion {
    fn partial_cmp(&self, other: &String) -> Option<std::cmp::Ordering> {
        self.partial_cmp(&TypeVersion::new(other.as_str()))
    }
}

impl PartialOrd<TypeVersion> for String {
    fn partial_cmp(&self, other: &TypeVersion) -> Option<std::cmp::Ordering> {
        TypeVersion::new(self.as_str()).partial_cmp(other)
    }
}

impl PartialOrd<&str> for TypeVersion {
    fn partial_cmp(&self, other: &&str) -> Option<std::cmp::Ordering> {
        self.partial_cmp(&other.to_string())
    }
}

impl PartialOrd<str> for TypeVersion {
    fn partial_cmp(&self, other: &str) -> Option<std::cmp::Ordering> {
        self.partial_cmp(&other.to_string())
    }
}

impl PartialOrd<TypeVersion> for &str {
    fn partial_cmp(&self, other: &TypeVersion) -> Option<std::cmp::Ordering> {
        self.to_string().partial_cmp(other)
    }
}

impl PartialOrd<TypeVersion> for str {
    fn partial_cmp(&self, other: &TypeVersion) -> Option<std::cmp::Ordering> {
        self.to_string().partial_cmp(other)
    }
}
