// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use std::{convert::Infallible, fmt::Display, str::FromStr};

use rust_i18n::t;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{dscerror::DscError, schemas::dsc_repo::DscRepoSchema, types::{ResourceVersion, SemanticVersionReq}};

/// Defines one or more limitations for a [`ResourceVersion`] to enable version pinning.
///
/// DSC supports both semantic versioning and arbitrary versioning for resources. Semantic
/// versioning is the preferred and recommended versioning strategy. DSC only supports arbitrary
/// versioning for compatibility scenarios.
///
/// Because DSC supports arbitrary string versions for compatibility, version requirements must
/// also support arbitrary string versions.
///
/// When a [`ResourceVersionReq`] is semantic, it behaves like a [`SemanticVersionReq`] and only
/// matches resource versions that are semantic _and_ valid for the given requirement. Arbitrary
/// string versions never match a semantic resource version requirement.
///
/// Similarly, when a [`ResourceVersionReq`] is an arbitrary string, it can never match a
/// semantically versioned [`ResourceVersion`]. Instead, it matches an arbitrary `ResourceVersion`
/// when the arbitrary string version is _exactly_ the same as the arbitrary resource version
/// requirement.
///
/// Arbitrary resource versions and resource version requirements are only defined for
/// compatibility scenarios. You should use semantic versions for resources and resource version
/// requirements.
///
/// ## Defining a resource version requirement
///
/// All strings are valid resource version requirements. However, to usefully define a resource
/// version requirement that supports correctly matching semantic versions, you must define the
/// requirement as valid `SemanticVersionReq`. See the [`SemanticVersionReq` documentation][01] for
/// full details on defining semantic version requirements.
///
/// ## Examples
///
/// When you create a new instance of [`ResourceVersionReq`], the variant is `Semantic` when the
/// input string parses as a [`SemanticVersionReq`]. Otherwise, the new instance is `Arbitrary`.
///
/// ```rust
/// use dsc_lib::types::{ResourceVersion, ResourceVersionReq};
///
/// let semantic_req = ResourceVersionReq::new("^1.2, <1.5");
/// let arbitrary_req = ResourceVersionReq::new("foo");
///
/// let v1_2_3 = &ResourceVersion::new("1.2.3");
/// let v1_5_1 = &ResourceVersion::new("1.5.1");
/// let v_arbitrary = &ResourceVersion::new("foo");
///
/// // Semantic requirement uses underlying semantic version requirement logic:
/// assert!(semantic_req.matches(v1_2_3));
/// assert!(!semantic_req.matches(v1_5_1));
/// // Semantic requirements never match arbitrary versions:
/// assert!(!semantic_req.matches(v_arbitrary));
///
/// // Arbitrary requirements only match arbitrary versions _exactly_:
/// assert!(arbitrary_req.matches(v_arbitrary));
/// // Differing casing causes the match to fail:
/// assert!(!arbitrary_req.matches(&ResourceVersion::new("FOO")));
/// ```
///
/// [01]: SemanticVersionReq
#[derive(Debug, Clone, Hash, Eq, Serialize, Deserialize, JsonSchema, DscRepoSchema)]
#[dsc_repo_schema(base_name = "resourceVersionReq", folder_path = "definitions")]
#[serde(untagged)]
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
    /// Defines the required version for the resource as an arbitrary string.
    ///
    /// DSC uses this variant for any requirement that can't be parsed as a semantic version
    /// requirement. This variant remains supported for compatibility purposes but is _not_
    /// recommended for production usage.
    ///
    /// When a resource version requirement is defined as an arbitrary string:
    ///
    /// 1. It can never match a semantically versioned resource.
    /// 1. It only matches a resource with an arbitrary string version
    ///    ([`ResourceVersion::Arbitrary`]) when the resource version and this version requirement
    ///    are exactly the same. The comparison is case-sensitive.
    #[schemars(
        title = t!("schemas.definitions.resourceVersionReq.arbitraryVariant.title"),
        description = t!("schemas.definitions.resourceVersionReq.arbitraryVariant.description"),
        extend(
            "deprecated" = true,
            "deprecationMessage" = t!("schemas.definitions.resourceVersionReq.arbitraryVariant.deprecationMessage"),
            "markdownDescription" = t!("schemas.definitions.resourceVersionReq.arbitraryVariant.markdownDescription"),
            "examples" = [
                "2026-02",
                "1.2.0.0"
            ]
        )
    )]
    Arbitrary(String),
}

impl ResourceVersionReq {
    /// Creates a new instance of [`ResourceVersionReq`].
    ///
    /// If the input string is a valid semantic version requirement, the function returns the
    /// [`Semantic`] variant. Otherwise, the function returns the [`Arbitrary`] variant for
    /// arbitrary version requirement strings.
    ///
    /// [`Semantic`]: ResourceVersionReq::Semantic
    /// [`Arbitrary`]: ResourceVersionReq::Arbitrary
    pub fn new(requirement_string: &str) -> Self {
        match SemanticVersionReq::parse(requirement_string) {
            Ok(req) => Self::Semantic(req),
            Err(_) => Self::Arbitrary(requirement_string.to_string()),
        }
    }

    /// Indicates whether the resource version requirement is semantic.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use dsc_lib::types::ResourceVersionReq;
    ///
    /// let semantic = ResourceVersionReq::new("^1.2, <1.5");
    /// let arbitrary = ResourceVersionReq::new("2026-01");
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

    /// Indicates whether the resource version requirement is an arbitrary string.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use dsc_lib::types::ResourceVersionReq;
    ///
    /// let arbitrary = ResourceVersionReq::new("2026-01");
    /// let semantic = ResourceVersionReq::new("^1.2, <1.5");
    ///
    /// assert_eq!(arbitrary.is_arbitrary(), true);
    /// assert_eq!(semantic.is_arbitrary(), false);
    /// ```
    pub fn is_arbitrary(&self) -> bool {
        match self {
            Self::Arbitrary(_) => true,
            _ => false,
        }
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
    /// let semantic = ResourceVersionReq::new("1.2.3");
    /// let date = ResourceVersionReq::new("2026-01-15");
    /// let arbitrary = ResourceVersionReq::new("arbitrary_version");
    ///
    /// assert_eq!(
    ///     semantic.as_semver_req(),
    ///     Some(&SemanticVersionReq::parse("^1.2.3").unwrap())
    /// );
    /// assert_eq!(
    ///     date.as_semver_req(),
    ///     None
    /// );
    /// assert_eq!(
    ///     arbitrary.as_semver_req(),
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

    /// Compares an instance of [`ResourceVersion`] to the requirement, returning `true` if the
    /// version is valid for the requirement and otherwise `false`.
    ///
    /// The comparison depends on whether the requirement and version are semantic or arbitrary:
    ///
    /// - When both the requirement and version are semantic, this function uses the logic for
    ///   comparing versions and requirements defined by [`SemanticVersionReq`].
    /// - When both the requirement and version are arbitrary, the version is only valid for the
    ///   requirement when it is exactly the same string as the requirement.
    /// - Otherwise, this function returns `false` because an arbitrary version can never match a
    ///   semantic requirement and a semantic version can never match an arbitrary requirement.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use dsc_lib::types::{ResourceVersion, ResourceVersionReq};
    ///
    /// let semantic_req = ResourceVersionReq::new("^1.2.3, <1.5");
    /// assert!(semantic_req.matches(&ResourceVersion::new("1.2.3")));
    /// assert!(semantic_req.matches(&ResourceVersion::new("1.3.0")));
    /// assert!(!semantic_req.matches(&ResourceVersion::new("1.0.0")));
    /// assert!(!semantic_req.matches(&ResourceVersion::new("1.5.0")));
    /// assert!(!semantic_req.matches(&ResourceVersion::new("2026-02")));
    ///
    /// let arbitrary_req = ResourceVersionReq::new("February 2026");
    /// assert!(arbitrary_req.matches(&ResourceVersion::new("February 2026")));
    /// assert!(!arbitrary_req.matches(&ResourceVersion::new("February2026")));
    /// assert!(!arbitrary_req.matches(&ResourceVersion::new("february 2026")));
    /// ```
    pub fn matches(&self, resource_version: &ResourceVersion) -> bool {
        match self {
            Self::Semantic(req) => {
                match resource_version {
                    ResourceVersion::Semantic(version) => req.matches(version),
                    ResourceVersion::Arbitrary(_) => false,
                }
            },
            Self::Arbitrary(req) => {
                match resource_version {
                    ResourceVersion::Semantic(_) => false,
                    ResourceVersion::Arbitrary(version) => req == version,
                }
            }
        }
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
            Self::Arbitrary(s) => write!(f, "{}", s),
        }
    }
}

// Parsing from a string is just calling `Self::new()`
impl FromStr for ResourceVersionReq {
    type Err = Infallible;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::new(s))
    }
}

// Implemented various conversion traits to move between `ResourceVersionReq`, `SemanticVersionReq`,
// `String`, and string slice (`str`).
impl From<String> for ResourceVersionReq {
    fn from(value: String) -> Self {
        match SemanticVersionReq::parse(&value) {
            Ok(req) => ResourceVersionReq::Semantic(req),
            Err(_) => ResourceVersionReq::Arbitrary(value),
        }
    }
}

impl From<ResourceVersionReq> for String {
    fn from(value: ResourceVersionReq) -> Self {
        value.to_string()
    }
}

impl From<&String> for ResourceVersionReq {
    fn from(value: &String) -> Self {
        match SemanticVersionReq::parse(value) {
            Ok(req) => ResourceVersionReq::Semantic(req),
            Err(_) => ResourceVersionReq::Arbitrary(value.clone()),
        }
    }
}
// We can't bidirectionally convert string slices, because we can't return a temporary reference.
// We can still convert _from_ string slices, but not _into_ them.
impl From<&str> for ResourceVersionReq {
    fn from(value: &str) -> Self {
        ResourceVersionReq::from(value.to_string())
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

// Creating an instance of `SemanticVersionReq` from `ResourceVersionReq` is the only fallible
// conversion, since `ResourceVersionReq` can define non-semantic version requirements.
impl TryFrom<ResourceVersionReq> for SemanticVersionReq {
    type Error = DscError;

    fn try_from(value: ResourceVersionReq) -> Result<Self, Self::Error> {
        match value {
            ResourceVersionReq::Semantic(req) => Ok(req),
            ResourceVersionReq::Arbitrary(s) => Err(DscError::ResourceVersionReqToSemverConversion(s))
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
                Self::Arbitrary(_) => false,
            },
            Self::Arbitrary(string) => !other.is_semver() && *string == other.to_string(),
        }
    }
}

impl PartialEq<SemanticVersionReq> for ResourceVersionReq {
    fn eq(&self, other: &SemanticVersionReq) -> bool {
        match self {
            Self::Semantic(req) => req == other,
            Self::Arbitrary(_) => false,
        }
    }
}

impl PartialEq<ResourceVersionReq> for SemanticVersionReq {
    fn eq(&self, other: &ResourceVersionReq) -> bool {
        match other {
            ResourceVersionReq::Semantic(req) => self == req,
            ResourceVersionReq::Arbitrary(_) => false,
        }
    }
}

impl PartialEq<&str> for ResourceVersionReq {
    fn eq(&self, other: &&str) -> bool {
        self == &Self::new(*other)
    }
}

impl PartialEq<ResourceVersionReq> for &str {
    fn eq(&self, other: &ResourceVersionReq) -> bool {
        &ResourceVersionReq::new(self) == other
    }
}

impl PartialEq<str> for ResourceVersionReq {
    fn eq(&self, other: &str) -> bool {
        self == &Self::new(other)
    }
}

impl PartialEq<ResourceVersionReq> for str {
    fn eq(&self, other: &ResourceVersionReq) -> bool {
        &ResourceVersionReq::new(self) == other
    }
}

impl PartialEq<String> for ResourceVersionReq {
    fn eq(&self, other: &String) -> bool {
        self == &Self::new(other)
    }
}

impl PartialEq<ResourceVersionReq> for String {
    fn eq(&self, other: &ResourceVersionReq) -> bool {
        &ResourceVersionReq::new(self) == other
    }
}
