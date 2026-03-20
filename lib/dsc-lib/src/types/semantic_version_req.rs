// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use std::{fmt::Display, ops::Deref, str::FromStr, sync::OnceLock};

use miette::Diagnostic;
use regex::Regex;
use rust_i18n::t;
use schemars::{json_schema, JsonSchema};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{schemas::dsc_repo::DscRepoSchema, types::SemanticVersion};

/// Defines one or more limitations for a semantic version to enable version pinning.
///
/// DSC uses the semantic version requirements for Rust, as documented in the
/// ["Specifying dependencies" section of the Cargo Book][01]. DSC adheres closely to the
/// Rust syntax for defining semantic version requirements, with the following exceptions:
///
/// 1. DSC semantic version requirements _forbid_ the inclusion of build metadata.
///
///    Rust allows and ignores them. DSC forbids the inclusion of build metadata to limit confusion
///    and unexpected behavior when specifying a version requirement for a DSC resource or
///    extension.
/// 1. DSC semantic version requirements _must_ define a major version segment. All other segments
///    are optional.
///
///    Rust technically supports specifying a wildcard-only version requirement (`*`). DSC forbids
///    specifying this version requirement as it maps to the default version selection and is
///    discouraged when specifying version requirements for production systems.
/// 1. DSC semantic version requirements _must_ explicitly define an operator for every comparator.
///
///    DSC forbids defining a comparator without an operator, like `1.*` or `1.2.3, <1.5`, to
///    reduce ambiguity and unexpected behavior for version pinning. For example, in all other
///    cases, omitting version segments and specifying them as a wildcard has the same behavior
///    _except_ for the comparators `1.2` and `1.2.*`:
///
///    - `1`, `1.*`, and `1.*.*` all have an effective requirement of `>=1.0.0, <2.0.0`.
///    - `>1.2` and `>1.2.*` both have an effective requirement of `>1.2.0`.
///    - `1.2` has an effective requirement of `>=1.2.0, <2.0.0` but `1.2.*` has an effective
///      requirement of `>=1.2.0, <1.3.0`.
///
///    Similarly, it is not immediately obvious to a user who isn't familiar with Rust semantic
///    version requirements that `1.2.3` will match `1.5.7`. It's more common across version
///    requirements to expect an exactly specified version to be an exact match requirement, not
///    a semantically compatible requirement.
/// 1. DSC semantic version requirements only support the asterisk (`*`) character for wildcards,
///    not `x` or `X`.
///
///    DSC forbids specifying wildcards as non-asterisks to reduce ambiguity and unexpected
///    behavior for prerelease segments, where `1.2.3-X` and `1.2.3-rc.x` are _valid_ requirements
///    but where the characters are _not_ interpreted as wildcards. Forbidding the use of these
///    characters prevents users from accidentally defining a requirement they _believe_ will
///    wildcard match on a prerelease segment but actually won't.
///
/// # Default requirement
///
/// The default requirement matches every possible stable version. It only rejects versions with a
/// prerelease segment, like `1.2.3-rc.1`. For DSC, the default requirement is used only when no
/// explicit requirement is given.
///
/// Effectively, the default requirement is `>=0.0.0`.
///
/// # Comparators
///
/// Every requirement defines one or more comparators. A comparator defines an operator and a
/// version for comparing against an instance of [`SemanticVersion`]. A requirement with multiple
/// comparators must separate each pair of comparators with a comma (`,`), like `^1.2, <=1.4`.
///
/// When a requirement specifies multiple comparators, a given instance of [`SemanticVersion`] only
/// matches the requirement when it matches _every_ comparator. Requirements with multiple
/// comparators effectively apply a logical `AND` for each comparator. If a requirement is defined
/// with incompatible comparators then _no_ version will ever match that requirement. For example,
/// the requirement `<1.2, >=2.3` can never match a version because no version can be less than
/// `1.2.0` _and_ greater than or equal to `2.3.0`.
///
/// There is no way to define a requirement using logical `OR` for multiple comparators. Instead,
/// define multiple requirements and check them independently in your code.
///
/// The following example shows how multiple comparators work in practice.
///
/// ```rust
/// use dsc_lib::types::{SemanticVersion, SemanticVersionReq};
///
/// // The requirement acts as a logical AND, matching versions between 1.2.0 and 1.4.0:
/// let valid_req = SemanticVersionReq::parse(">=1.2, <1.4").unwrap();
/// assert_eq!(valid_req.matches(&SemanticVersion::new(1, 1, 0)), false);
/// assert_eq!(valid_req.matches(&SemanticVersion::new(1, 2, 0)), true);
/// assert_eq!(valid_req.matches(&SemanticVersion::new(1, 3, 0)), true);
/// assert_eq!(valid_req.matches(&SemanticVersion::new(1, 4, 0)), false);
///
/// // The invalid requirement never matches any versions:
/// let invalid_req = SemanticVersionReq::parse("<=1.2, >1.4").unwrap();
/// assert_eq!(invalid_req.matches(&SemanticVersion::new(1, 1, 0)), false);
/// assert_eq!(invalid_req.matches(&SemanticVersion::new(1, 2, 0)), false);
/// assert_eq!(invalid_req.matches(&SemanticVersion::new(1, 3, 0)), false);
/// assert_eq!(invalid_req.matches(&SemanticVersion::new(1, 4, 0)), false);
///
/// // To match two or more incompatible version requirements, use an or statement:
/// let le_req = SemanticVersionReq::parse("<=1.2").unwrap();
/// let gt_req = SemanticVersionReq::parse(">1.4").unwrap();
/// let v1_0_0 = &SemanticVersion::new(1, 0, 0);
/// let v1_3_0 = &SemanticVersion::new(1, 3, 0);
/// let v1_5_0 = &SemanticVersion::new(1, 5, 0);
/// assert_eq!(
///     le_req.matches(v1_0_0) || gt_req.matches(v1_0_0),
///     true
/// );
/// assert_eq!(
///     le_req.matches(v1_3_0) || gt_req.matches(v1_3_0),
///     false
/// );
/// assert_eq!(
///     le_req.matches(v1_5_0) || gt_req.matches(v1_5_0),
///     true
/// );
/// ```
///
/// ## Specifying comparator versions
///
/// Every comparator in a version requirement must define a version. Only the major version segment
/// is required. The minor, patch, and prerelease segments are optional. The build metadata segment
/// is forbidden.
///
/// ### Omitting version segments
///
/// When defining a version for a comparator, you must define the major version segment. You can
/// omit either or both the minor and patch version segments. The following comparators define
/// valid versions:
///
/// - `>=1` - Matches all versions greater than or equal to `1.0.0`.
/// - `>=1.2` - Matches all versions greater than or equal to `1.2.0`.
///
/// ### Wildcard version segments
///
/// You can specify the minor and patch version segments as a wildcard with the asterisk (`*`)
/// character, indicating that it should match any version for that segment. If the minor version
/// segment is a wildcard, the patch version segment must either be a wildcard or omitted.
///
/// For DSC semantic version requirements, specifying the version for a comparator with wildcards
/// is equivalent to omitting those version segments.
///
/// The following table shows a how specifying wildcards for a version segment affects the effective
/// requirement for a comparator:
///
/// | Comparator | Effective requirement |
/// |:----------:|:---------------------:|
/// | `^1`       | `>=1.0.0, <2.0.0`     |
/// | `^1.*`     | `>=1.0.0, <2.0.0`     |
/// | `^1.*.*`   | `>=1.0.0, <2.0.0`     |
/// | `^1.2`     | `>=1.2.0, <2.0.0`     |
/// | `^1.2.*`   | `>=1.2.0, <2.0.0`     |
/// | `=1`       | `>=1.0.0, <2.0.0`     |
/// | `=1.*`     | `>=1.0.0, <2.0.0`     |
/// | `=1.*.*`   | `>=1.0.0, <2.0.0`     |
/// | `=1.2`     | `>=1.2.0, <1.3.0`     |
/// | `=1.2.*`   | `>=1.2.0, <1.3.0`     |
///
/// ### Prerelease version segments
///
/// A comparator only ever matches a version with a prerelease segment when the comparator version
/// also defines a prerelease segment. Prerelease segments are only compared when the comparator
/// version and the target version have identical major, minor, and patch version segments.
/// Prerelease segments are compared as strings for ordering.
///
/// The comparator `^1` can never match `1.2.3-rc.1` or `1.3.0-pre`. To define a prerelease
/// segment, you must define the major, minor, and patch version segments as literals without any
/// wildcards, like `1.2.3-rc`.
///
/// To define a comparator with a version that matches any valid prerelease for that version,
/// specify the prerelease segment as `0`, like `1.2.3-0`.
///
/// To help show how prerelease segments affect version matching, the following table defines
/// a series of comparators and whether different versions match those comparators.
///
/// | Comparator version | Matching versions                                                         | Non-matching versions                                 |
/// |:------------------:|:--------------------------------------------------------------------------|:------------------------------------------------------|
/// | `>=2.0.0`          | `2.0.0`, `2.1.0`, `3.0.0`                                                 | `1.2.3`, `2.0.0-alpha.1`, `2.1.0-beta.2`, `3.0.0-rc1` |
/// | `>=2.0.0-alpha`    | `2.0.0`, `2.1.0`, `3.0.0`, `2.0.0-alpha`, `2.0.0-alpha.1`, `2.0.0-beta.1` | `1.2.3`, `2.0.0-0`                                    |
/// | `>=2.0.0-beta`     | `2.0.0`, `2.1.0`, `3.0.0`, `2.0.0-beta`, `2.0.0-beta.1`                   | `1.2.3`, `2.0.0-alpha`                                |
/// | `>=2.0.0-0`        | `2.0.0`, `2.1.0`, `3.0.0`, `2.0.0-1`, `2.0.0-alpha`, `2.0.0-beta`         | `1.2.3`                                               |
///
/// ### Forbidding build metadata in comparator versions
///
/// DSC forbids the inclusion of build metadata in comparator versions to reduce ambiguity. While
/// the underlying Rust implementation for version requirements ([`semver::VersionReq`]) allows
/// versions with build metadata, like `1.2.3+sha123` or `1.2.3-rc.1+dev.debug.linux`, it ignores
/// those segments entirely when matching a semantic version against the requirement.
///
/// To prevent users from assuming that a version requirement might operate on the build metadata,
/// DSC forbids its inclusion in a version requirement string and raises the
/// [`ComparatorIncludesForbiddenBuildMetadata`] error during parsing if one is specified.
///
/// ### Examples of invalid comparator versions
///
/// The following list enumerates a series of invalid versions for comparators with the reasons why
/// each version is invalid for a comparator:
///
/// - `*.*.*` - The major version segment must be a literal number, not a wildcard, like `1.*.*`.
///   If you want to allow any version, do not specify a version requirement explicitly.
/// - `1.*.3` - When the minor version segment is a wildcard, the patch version segment must either
///   be a wildcard or omitted, like `1.*.*` or `1.*`.
/// - `1.2-rc` - A prerelease segment is only valid when the major, minor, and patch version
///   segments are all defined as literals, like `1.2.0-rc`.
/// - `1.2.*-rc` - A prerelease segment is only valid when the major, minor, and patch version
///   segments are literals without any wildcards, like `1.2.0-rc`.
/// - `1.2.3-*` - Wildcards aren't permitted for prerelease version segments. To effectively
///   specify a prerelease segment that matches any prerelease versions for a given version,
///   define the prerelease segment as `0`.
/// - `1.2.3+sha123` - Build metadata segments aren't permitted for versions in comparators.
///
/// ## Specifying comparator operators
///
/// An operator defines how to compare a given [`SemanticVersion`] against the version component
/// of the comparator. The operator for a comparator is required.
///
/// The following list enumerates the available operators. Each definition includes a table of
/// examples demonstrating how the operator behaves.
///
/// - <a id="operator-caret"></a>Caret (`^`) - Indicates that the [`SemanticVersion`] must be
///   semantically compatible with the version for this comparator. The version must be equal to or
///   greater than the given version in the comparator and less than the next major version.
///
///   | Literal comparator | Effective requirement  | Valid versions                               | Invalid versions                             |
///   |:------------------:|:----------------------:|:---------------------------------------------|:---------------------------------------------|
///   | `^1`               | `>=1.0.0, <2.0.0`      | `1.0.0`, `1.2.0`, `1.3.0`                    | `0.1.0`, `2.0.0`, `1.2.3-rc.1`               |
///   | `^1.*`             | `>=1.0.0, <2.0.0`      | `1.0.0`, `1.2.0`, `1.3.0`                    | `0.1.0`, `2.0.0`, `1.2.3-rc.1`               |
///   | `^1.*.*`           | `>=1.0.0, <2.0.0`      | `1.0.0`, `1.2.0`, `1.3.0`                    | `0.1.0`, `2.0.0`, `1.2.3-rc.1`               |
///   | `^1.2`             | `>=1.2.0, <2.0.0`      | `1.2.0`, `1.2.3`, `1.3.0`                    | `1.0.0`, `2.0.0`, `1.2.3-rc.1`               |
///   | `^1.2.*`           | `>=1.2.0, <2.0.0`      | `1.2.0`, `1.2.3`, `1.3.0`                    | `1.0.0`, `2.0.0`, `1.2.3-rc.1`               |
///   | `^1.2.3`           | `>=1.2.3, <2.0.0`      | `1.2.3`, `1.2.4`, `1.3.0`                    | `1.2.0`, `2.0.0`, `1.2.3-rc.1`               |
///   | `^1.2.3-rc.2`      | `>=1.2.3-rc.2, <2.0.0` | `1.2.3`, `1.3.0`, `1.2.3-rc.2`, `1.2.3-rc.3` | `1.2.0`, `2.0.0`, `1.2.3-rc.1`, `1.3.0-rc.2` |
///
/// - <a id="operator-tilde"></a>Tilde (`~`) - Indicates that the [`SemanticVersion`] must be
///   greater than or equal to the version for this comparator. The upper bound of matching
///   versions depends on how many components the version of the comparator defines:
///
///   - If the comparator defines only the major version segment, like `~1`, the comparator
///     matches any version greater than or equal to the given major version and less than the next
///     major version.
///   - If the comparator defines the major and minor version segments, like `~1.2` or `~1.2.3`,
///     the comparator matches any greater than or equal to the given version and less than the
///     next minor version.
///
///   The patch and prerelease segments of the version for the comparator only affect the minimum
///   version bound for the requirement. They don't affect the upper bound.
///
///   | Literal comparator | Effective requirement  | Valid versions                               | Invalid versions               |
///   |:------------------:|:----------------------:|:---------------------------------------------|:-------------------------------|
///   | `~1`               | `>=1.0.0, <2.0.0`      | `1.0.0`, `1.2.0`, `1.3.0`                    | `0.1.0`, `2.0.0`, `1.2.3-rc.1` |
///   | `~1.*`             | `>=1.0.0, <2.0.0`      | `1.0.0`, `1.2.0`, `1.3.0`                    | `0.1.0`, `2.0.0`, `1.2.3-rc.1` |
///   | `~1.*.*`           | `>=1.0.0, <2.0.0`      | `1.0.0`, `1.2.0`, `1.3.0`                    | `0.1.0`, `2.0.0`, `1.2.3-rc.1` |
///   | `~1.2`             | `>=1.2.0, <1.3.0`      | `1.2.0`, `1.2.3`                             | `1.0.0`, `1.3.0`, `1.2.3-rc.1` |
///   | `~1.2.*`           | `>=1.2.0, <1.3.0`      | `1.2.0`, `1.2.3`                             | `1.0.0`, `1.3.0`, `1.2.3-rc.1` |
///   | `~1.2.3`           | `>=1.2.3, <1.3.0`      | `1.2.3`, `1.2.9`                             | `1.2.0`, `1.3.0`, `1.2.3-rc.1` |
///   | `~1.2.3-rc.2`      | `>=1.2.3-rc.2, <1.3.0` | `1.2.3`, `1.2.9`, `1.2.3-rc.2`, `1.2.3-rc.3` | `1.2.0`, `1.3.0`, `1.2.3-rc.1` |
///
/// - <a id="operator-less-than"></a>Less than (`<`) - Indicates that the [`SemanticVersion`] must
///   be less than the version for this comparator. Versions equal to or greater than the
///   comparator version don't match the comparator.
///
///   | Literal comparator | Effective requirement  | Valid versions                          | Invalid versions                             |
///   |:------------------:|:----------------------:|:----------------------------------------|:---------------------------------------------|
///   | `<1`               | `<1.0.0`               | `0.1.0`                                 | `1.0.0`, `1.2.0`, `1.2.3`, `0.1.0-rc.1`      |
///   | `<1.*`             | `<1.0.0`               | `0.1.0`                                 | `1.0.0`, `1.2.0`, `1.2.3`, `0.1.0-rc.1`      |
///   | `<1.*.*`           | `<1.0.0`               | `0.1.0`                                 | `1.0.0`, `1.2.0`, `1.2.3`, `0.1.0-rc.1`      |
///   | `<1.2`             | `<1.2.0`               | `0.1.0`, `1.0.0`, `1.1.1`               | `1.2.0`, `1.2.3`, `1.3.0`, `1.2.0-rc.1`,     |
///   | `<1.2.*`           | `<1.2.0`               | `0.1.0`, `1.0.0`, `1.1.1`               | `1.2.0`, `1.2.3`, `1.3.0`, `1.2.0-rc.1`,     |
///   | `<1.2.3`           | `<1.2.3`               | `0.1.0`, `1.0.0`, `1.2.0`               | `1.2.3`, `1.3.0`, `1.2.3-rc.1`               |
///   | `<1.2.3-rc.2`      | `<1.2.3-rc.2`          | `0.1.0`, `1.0.0`, `1.2.0`, `1.2.3-rc.1` | `1.2.3`, `1.3.0`, `1.0.0-rc.1`, `1.2.3-rc.2` |
///
/// - <a id="operator-less-than-or-equal-to"></a>Less than or equal to (`<=`) - Indicates that the
///   [`SemanticVersion`] must be any version up to the version for this comparator. Versions
///   greater than the comparator version don't match the comparator.
///
///   | Literal comparator | Effective requirement | Valid versions                      | Invalid versions                             |
///   |:------------------:|:---------------------:|:------------------------------------|:---------------------------------------------|
///   | `<=1`              | `<2.0.0`              | `0.1.0`, `1.0.0`, `1.2.3`           | `2.0.0`, `0.1.0-rc.1`, `1.0.0-rc.1`          |
///   | `<=1.*`            | `<2.0.0`              | `0.1.0`, `1.0.0`, `1.2.3`           | `2.0.0`, `0.1.0-rc.1`, `1.0.0-rc.1`          |
///   | `<=1.*.*`          | `<2.0.0`              | `0.1.0`, `1.0.0`, `1.2.3`           | `2.0.0`, `0.1.0-rc.1`, `1.0.0-rc.1`          |
///   | `<=1.2`            | `<1.3.0`              | `0.1.0`, `1.0.0`, `1.2.0`, `1.2.3`  | `1.3.0`, `1.0.0-rc.1`, `1.2.0-rc.1`          |
///   | `<=1.2.*`          | `<1.3.0`              | `0.1.0`, `1.0.0`, `1.2.0`, `1.2.3`  | `1.3.0`, `1.0.0-rc.1`, `1.2.0-rc.1`          |
///   | `<=1.2.3`          | `<=1.2.3`             | `0.1.0`, `1.0.0`, `1.2.3`           | `1.2.4`, `1.3.0`, `1.2.0-rc.1`, `1.2.3-rc.1` |
///   | `<=1.2.3-rc.2`     | `<=1.2.3-rc.2`        | `0.1.0`, `1.2.3-rc.1`, `1.2.3-rc.2` | `1.2.3`, `1.3.0`, `1.0.0-rc.1`, `1.2.3-rc.3` |
///
/// - <a id="operator-exact"></a>Exact (`=`) - Indicates that the [`SemanticVersion`] must be
///   the same as the given version for this comparator. If the comparator version omits
///   version segments or specifies them as wildcards, then the comparator matches a range of
///   versions. A comparator that defines a literal patch version only matches that exact
///   version. A comparator that defines a prerelease segment only matches that exact patch version
///   and prerelease segment.
///
///   | Literal comparator | Effective requirement | Valid versions   | Invalid versions                |
///   |:------------------:|:---------------------:|:-----------------|:--------------------------------|
///   | `=1`               | `>=1.0.0, <2.0.0`     | `1.0.0`, `1.2.3` | `0.1.0`, `2.0.0`, `1.0.0-rc.2`  |
///   | `=1.*`             | `>=1.0.0, <2.0.0`     | `1.0.0`, `1.2.3` | `0.1.0`, `2.0.0`, `1.0.0-rc.2`  |
///   | `=1.*.*`           | `>=1.0.0, <2.0.0`     | `1.0.0`, `1.2.3` | `0.1.0`, `2.0.0`, `1.0.0-rc.2`  |
///   | `=1.2`             | `>=1.2.0, <1.3.0`     | `1.2.0`, `1.2.3` | `1.0.0`, `1.3.0`, `1.2.3-rc.2`  |
///   | `=1.2.*`           | `>=1.2.0, <1.3.0`     | `1.2.0`, `1.2.3` | `1.0.0`, `1.3.0`, `1.2.3-rc.2`  |
///   | `=1.2.3`           | `=1.2.3`              | `1.2.3`          | `1.2.0`, `1.3.0`, `1.2.3-rc.2` |
///   | `=1.2.3-rc.2`      | `=1.2.3-rc.2`         | `1.2.3-rc.2`     | `1.2.3`, `1.3.0`, `1.2.3-rc.1`  |
///
/// - <a id="operator-greater-than"></a>Greater than (`>`) - Indicates that the
///   [`SemanticVersion`] must be greater than the version for this comparator. Versions equal to
///   or less than the comparator version don't match the comparator.
///
///   | Literal comparator | Effective requirement | Valid versions                | Invalid versions                    |
///   |:------------------:|:---------------------:|:------------------------------|:------------------------------------|
///   | `>1`               | `>=2.0.0`             | `2.0.0`, `2.3.4`              | `1.0.0`, `1.2.3`, `2.0.0-rc.2`      |
///   | `>1.*`             | `>=2.0.0`             | `2.0.0`, `2.3.4`              | `1.0.0`, `1.2.3`, `2.0.0-rc.2`      |
///   | `>1.*.*`           | `>=2.0.0`             | `2.0.0`, `2.3.4`              | `1.0.0`, `1.2.3`, `2.0.0-rc.2`      |
///   | `>1.2`             | `>=1.3.0`             | `1.3.0`, `2.0.0`              | `1.0.0`, `1.2.3`, `2.0.0-rc.2`      |
///   | `>1.2.*`           | `>=1.3.0`             | `1.3.0`, `2.0.0`              | `1.0.0`, `1.2.3`, `2.0.0-rc.2`      |
///   | `>1.2.3`           | `>=1.2.4`             | `1.2.4`, `2.0.0`              | `1.2.3`, `2.0.0-rc.2`               |
///   | `>1.2.3-rc.1`      | `>=1.2.3-rc.2`        | `1.2.3`,`2.0.0`, `1.2.3-rc.3` | `1.2.0`, `1.2.3-rc.1`, `2.0.0-rc.2` |
///
/// - <a id="operator-greater-than-or-equal-to"></a>Greater than or equal to (`>=`) - Indicates that
///   the [`SemanticVersion`] must be the same as the version for this comparator or newer.
///   Versions less than the comparator version don't match the comparator.
///
///   | Literal comparator | Effective requirement | Valid versions                               | Invalid versions                    |
///   |:------------------:|:---------------------:|:---------------------------------------------|:------------------------------------|
///   | `>=1`              | `>=1.0.0`             | `1.0.0`, `1.2.3`                             | `0.1.0`, `1.2.3-rc.2`               |
///   | `>=1.*`            | `>=1.0.0`             | `1.0.0`, `1.2.3`                             | `0.1.0`, `1.2.3-rc.2`               |
///   | `>=1.*.*`          | `>=1.0.0`             | `1.0.0`, `1.2.3`                             | `0.1.0`, `1.2.3-rc.2`               |
///   | `>=1.2`            | `>=1.2.0`             | `1.2.0`, `1.2.3`                             | `1.1.1`, `1.2.3-rc.2`               |
///   | `>=1.2.*`          | `>=1.2.0`             | `1.2.0`, `1.2.3`                             | `1.1.1`, `1.2.3-rc.2`               |
///   | `>=1.2.3`          | `>=1.2.3`             | `1.2.3`, `1.3.0`                             | `1.2.2`, `1.2.3-rc.2`, `2.0.0-rc.2` |
///   | `>=1.2.3-rc.1`     | `>=1.2.3-rc.1`        | `1.2.3`, `2.0.0`, `1.2.3-rc.1`, `1.2.3-rc.2` | `1.2.0`, `1.2.3-rc.0`, `2.0.0-rc.2` |
///
/// # Serialization
///
/// Note that during serialization instances of [`SemanticVersionReq`]:
///
/// 1. If the originally parsed requirement defines a version with any wildcards, it serializes
///    with the wildcard segments omitted. For example, consider the following table showing how
///    different comparators serialize:
///
///    | Originally parsed comparator | Serialized comparator |
///    |:----------------------------:|:---------------------:|
///    | `^1.*`                       | `^1`                  |
///    | `^1.*.*`                     | `^1`                  |
///    | `^1.2.*`                     | `^1.2`                |
///
/// 1. If the originally parsed requirement has any separating spaces between an operator and
///    version, like `>= 1.2` or `>=  1.2`, it serializes without any spaces as `>=1.2`.
/// 1. If the originally parsed requirement defines a pair of comparators, it always serializes the
///    pair separated by a comma followed by a single space. For example, all of the originally
///    parsed requirements in the following list serialize as `>=1.2, <1.5`:
///
///    - `>=1.2,<1.5`
///    - `>=1.2 ,<1.5`
///    - `>=1.2,  <1.5`
///    - `>=1.2  ,  <1.5`
///
/// This can make it difficult to effectively round-trip a requirement when deserializing and
/// reserializing. To define a version requirement that will round-trip without any changes:
///
/// 1. Always omit version segments rather than specifying a wildcard.
/// 1. Never separate operators and versions in a comparator with any spaces.
/// 1. When defining a requirement with multiple comparators, always follow the preceding
///    comparator with a comma followed by a single space before the succeeding comparator.
///
/// The following table shows requirements that won't correctly round-trip with an equivalent
/// requirement that _does_ round trip.
///
/// | Non-round-tripping requirement | Round-tripping requirement |
/// |:------------------------------:|:--------------------------:|
/// | `^1.2.*`                       | `^1.2`                     |
/// | `> 1`                          | `>1`                       |
/// | `>1.2 , <=1.5`                 | `>1.2, <=1.5`              |
///
/// # Best practices for defining version requirements
///
/// When defining a comparator for a version requirement, always:
///
/// 1. Immediately follow the operator with the version, like `>1.2` instead of `> 1.2`.
///
///    This reduces the likelihood of changing the requirement string when round-tripping through
///    serialization and deserialization.
/// 1. Omit version segments instead of using wildcards, like `>1.2` instead of `>1.2.*`.
///
///    This reduces the likelihood of changing the requirement string when round-tripping through
///    serialization and deserialization.
/// 1. Separate subsequent comparators from previous comparators in the requirement with a comma
///    followed by a single space, like `>=1.2, <1.5` instead of `>=1.2,<1.5`, `>=1.2 ,<1.5`,
///    or `>=1.2 , <1.5`.
///
///    This reduces the likelihood of changing the requirement string when round-tripping through
///    serialization and deserialization.
/// 1. Define the requirement without leading or trailing spaces, like `^1.2` instead of
///    `  ^1.2 `.
///
///    This reduces the likelihood of changing the requirement when round-tripping through
///    serialization and deserialization.
///
/// [01]: https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html#version-requirement-syntax
/// [`ComparatorIncludesForbiddenBuildMetadata`]: SemanticVersionReqError::ComparatorIncludesForbiddenBuildMetadata
#[derive(Debug, Clone, Hash, Eq, Serialize, Deserialize, DscRepoSchema)]
#[dsc_repo_schema(base_name = "semverRequirement", folder_path = "definitions")]
pub struct SemanticVersionReq(semver::VersionReq);

/// Defines the parsing errors and diagnostics for invalid string representations of a
/// [`SemanticVersionReq`].
///
/// This error type is surfaced through the [`DscError::SemverReq`] error. This error type primarily
/// distinguishes between two kinds of errors returned by [`SemanticVersionReq::parse()`]:
///
/// 1. When the input string is unparseable as a [`semver::VersionReq`], the function raises the
///    [`UnparseableRequirement`] error, which passes the underlying parsing error through to the
///    user.
/// 1. When the input string is parseable as a [`semver::VersionReq`] but fails the more
///    restrictive syntax validation DSC requires, the function raises the [`InvalidRequirement`]
///    error, which collects the diagnostic errors for every mistake in the input string.
///
///    This enables the function to return the full set of invalid components for a requirement
///    together, instead of requiring a user to iteratively discover their mistakes when each
///    error is raised separately and immediately halts execution.
///
/// The remaining variants serve to collect validation errors for each comparator and to
/// distinguish between the different validation failures for a comparator.
///
/// [`DscError::SemverReq`]: crate::dscerror::DscError::SemverReq
/// [`UnparseableRequirement`]: Self::UnparseableRequirement
/// [`InvalidRequirement`]: Self::InvalidRequirement
#[derive(Debug, Diagnostic, Error)]
#[non_exhaustive]
pub enum SemanticVersionReqError {
    /// Indicates that the input string was unparseable by the underlying [`semver`] crate, which
    /// allows a more relaxed syntax. Any string that fails to parse as a [`semver::VersionReq`]
    /// can't parse as a [`SemanticVersionReq`].
    #[error("{t}", t = t!(
        "types.semantic_version_req.unparseableReq",
        "err" => source,
    ))]
    UnparseableRequirement{
        /// The underlying parsing error from the [`semver`] crate, which provides details about
        /// why the input string couldn't be parsed as a valid semantic version requirement.
        #[from] source: semver::Error,
    },

    /// Indicates that the input string was invalid for the syntax that DSC supports.
    ///
    /// When DSC raises this error, the input string was valid for the Rust syntax that [`semver`]
    /// supports but had one or more errors specific to DSC's more restrictive syntax. The `errors`
    /// field contains a collection of one or more errors that show more fully how the input
    /// string failed validation during parsing.
    #[error("{t}", t = t!(
        "types.semantic_version_req.invalidReq",
        "requirement" => requirement,
        "err" => errors.iter().map(|e| e.to_string()).collect::<Vec<_>>().join(", ")
    ))]
    InvalidRequirement{
        /// The input string for the requirement that failed validation during parsing.
        requirement: String,

        /// Collected errors for every invalid comparator in the requirement.
        #[related]
        errors: Vec<SemanticVersionReqError>
    },

    /// Indicates that a specific comparator in the requirement was invalid for DSC.
    ///
    /// When DSC raises this error, the comparator  was valid for the Rust syntax that [`semver`]
    /// supports but had one or more errors specific to DSC's more restrictive syntax. The `errors`
    /// field contains a collection of one or more errors that show more fully how the specific
    /// comparator failed validation during parsing.
    ///
    /// [`InvalidRequirement`]: Self::InvalidRequirement
    #[error("{t}", t = t!(
        "types.semantic_version_req.invalidComparator",
        "comparator" => comparator,
        "err" => errors.iter().map(|e| e.to_string()).collect::<Vec<_>>().join(", ")
    ))]
    InvalidComparator {
        /// The input string for the comparator that failed validation during parsing.
        comparator: String,

        /// Collected errors for every invalid syntax problem with the comparator.
        #[related]
        errors: Vec<SemanticVersionReqError>,
    },

    /// Indicates that a comparator included the build metadata segment for a version, which DSC
    /// forbids for clarity.
    ///
    /// While [`semver`] allows users to define build metadata for the version of a comparator, it
    /// also ignores that segment entirely for the purposes of matching versions. DSC forbids the
    /// build metadata segment in comparators to reduce the ambiguity and false expectations that
    /// it will be used for version matching.
    #[error("{t}", t = t!(
        "types.semantic_version_req.forbiddenBuildMetadata",
        "comparator" => comparator,
        "build" =>  build_metadata,
    ))]
    ComparatorIncludesForbiddenBuildMetadata{
        /// The input string for the comparator that failed validation during parsing.
        comparator: String,
        /// The text of the forbidden build metadata segment
        build_metadata: String
    },

    /// Indicates that a comparator was defined without an explicit operator, which DSC requires
    /// for clarity and predictability.
    ///
    /// [`semver`] supports defining a comparator without an explicit operator, interpreting the
    /// operator as the semantically compatible operator (`^`) unless the version specifies one
    /// or more wildcards, in which case it interprets the comparator as a wildcard operator.
    ///
    /// DSC _requires_ an explicit operator to limit confusion when specifying version pinning in
    /// a configuration document.
    #[error("{t}", t = t!(
        "types.semantic_version_req.missingOperator",
        "comparator" => comparator
    ))]
    ComparatorMissingOperator{
        /// The input string for the comparator that failed validation during parsing.
        comparator: String
    },

    /// Indicates that a comparator was defined with an invalid wildcard character (`x` or `X`).
    ///
    /// [`semver`] supports defining wildcards as asterisks (`*`), `x`, and `X`. DSC forbids using
    /// letters as wildcards to reduce ambiguity and confusion when specifying wildcards and
    /// prerelease version segments.
    #[error("{t}", t = t!(
        "types.semantic_version_req.invalidWildcards",
        "comparator" => comparator
    ))]
    ComparatorWithInvalidWildcards{
        /// The input string for the comparator that failed validation during parsing.
        comparator: String
    },

    /// Indicates that a comparator was defined with a wildcard for the major version segment,
    /// which DSC forbids.
    ///
    /// [`semver`] supports defining the version for a comparator with the major version segment
    /// as a wildcard. DSC forbids this construction, since it maps to "match any version," which
    /// is the default behavior when no version requirement is defined.
    #[error("{t}", t = t!(
        "types.semantic_version_req.wildcardMajorVersion",
        "comparator" => comparator,
        "wildcard" => wildcard
    ))]
    ComparatorWithWildcardMajorVersion{
        /// The input string for the comparator that failed validation during parsing.
        comparator: String,
        /// The wildcard used for the major version segment.
        wildcard: String,
    },
}

/// This static lazily defines the regex for [`SemanticVersionReq`] that finds instances of the
/// forbidden build metadata segment in the version for any comparator. It enables the [`Regex`]
/// instance to be constructed once, the first time it's used, and then reused on all subsequent
/// validation calls. It's kept private, since the API usage is to invoke the
/// [`SemanticVersionReq::parse()`] method to validate and parse a string into a version
/// requirement.
static COMPARATOR_HAS_BUILD_METADATA_REGEX: OnceLock<Regex> = OnceLock::new();

/// This static lazily defines the regex for [`SemanticVersionReq`] that finds comparators that
/// are defined without the mandatory operator before a version. It enables the [`Regex`] instance
/// to be constructed once, the first time it's used, and then reused on all subsequent validation
/// calls. It's kept private, since the API usage is to invoke the [`SemanticVersionReq::parse()`]
/// method to validate and parse a string into a version requirement.
static COMPARATOR_STARTS_WITH_OPERATOR_REGEX: OnceLock<Regex> = OnceLock::new();

/// This static lazily defines the regex for [`SemanticVersionReq`] that finds invalid wildcards
/// (`x` or `X`) that are defined for any comparator version. It enables the [`Regex`] instance
/// to be constructed once, the first time it's used, and then reused on all subsequent validation
/// calls. It's kept private, since the API usage is to invoke the [`SemanticVersionReq::parse()`]
/// method to validate and parse a string into a version requirement.
static COMPARATOR_HAS_INVALID_WILDCARD_REGEX: OnceLock<Regex> = OnceLock::new();

impl SemanticVersionReq {
    /// Parses a given string into a semantic version requirement.
    ///
    /// # Errors
    ///
    /// The parse function returns an error when the string isn't a valid version requirement.
    /// Common parse failures include:
    ///
    /// - Specifying a literal version segment after a wildcard, like `*.1` or `1.x.3`.
    /// - Specifying a wildcard in the prerelease segment, like `1.2.3-*` or `2.0.0-rc.*`. Note that
    ///   specifying an `x` or `X` as a wildcard for the prerelease segment parses but is treated
    ///   as a literal `x` or `X` in the comparison logic because singular alphabetic characters are
    ///   valid prerelease segments.
    /// - Specifying the build metadata segment, like `1.2.3+dev` or `1.2.3-rc.1+dev`.
    /// - Specifying an invalid comparison operator, like `!3.0.0`.
    /// - Specifying an invalid character for a version segment, like `>a.b`.
    /// - Not specifying an additional comparator after a comma, like `>=1.*,`,
    /// - Not specifying a comma between comparators, like `>=1.2 <1.9`.
    pub fn parse(text: &str) -> Result<Self, SemanticVersionReqError> {
        // First verify whether the input can parse as a semantic version requirement at all;
        // If not, error early.
        let requirement = semver::VersionReq::parse(text)?;
        // Next, collect parse errors to provide full feedback to user for invalid comparator
        // definitions:
        let mut errors: Vec<SemanticVersionReqError> = vec![];
        let comparators: Vec<&str> = text.split(',').map(|c| c.trim()).collect();
        let starts_with_operator = COMPARATOR_STARTS_WITH_OPERATOR_REGEX.get_or_init(
            Self::init_operator_pattern
        );
        let invalid_wildcard = COMPARATOR_HAS_INVALID_WILDCARD_REGEX.get_or_init(
            Self::init_wildcard_pattern
        );
        let has_build_metadata = COMPARATOR_HAS_BUILD_METADATA_REGEX.get_or_init(
            Self::init_build_metadata_pattern
        );

        for comparator in comparators {
            let mut comparator_errors: Vec<SemanticVersionReqError> = vec![];
            // Check for missing operators:
            if !starts_with_operator.is_match(comparator) {
                comparator_errors.push(
                    SemanticVersionReqError::ComparatorMissingOperator{
                        comparator: comparator.into()
                    }
                );
            }
            // Check for invalid wildcards:
            if let Some(captures) = invalid_wildcard.captures(comparator) {
                if let Some(wildcard_major) = captures.name("wildcard_major") {
                    comparator_errors.push(
                        SemanticVersionReqError::ComparatorWithWildcardMajorVersion{
                            comparator: comparator.into(),
                            wildcard: wildcard_major.as_str().to_string()
                        }
                    );
                }
                if captures.name("invalid_minor_wildcard").is_some() || captures.name("invalid_patch_wildcard").is_some() {
                    comparator_errors.push(
                        SemanticVersionReqError::ComparatorWithInvalidWildcards{
                            comparator: comparator.into()
                        }
                    );
                }
            }
            // Check for forbidden build metadata
            if let Some(captures) = has_build_metadata.captures(comparator) {
                let build_metadata = captures
                    .name("buildmetadata")
                    .expect("capture requires this group, should always exist")
                    .as_str()
                    .to_string();

                comparator_errors.push(
                    SemanticVersionReqError::ComparatorIncludesForbiddenBuildMetadata{
                        comparator: comparator.into(),
                        build_metadata,
                    }
                );
            }

            if !comparator_errors.is_empty() {
                errors.push(
                    SemanticVersionReqError::InvalidComparator {
                        comparator: comparator.into(),
                        errors: comparator_errors
                    }
                );
            }
        }

        if errors.is_empty() {
            Ok(Self(requirement))
        } else {
            Err(SemanticVersionReqError::InvalidRequirement {
                requirement: text.to_string(),
                errors
            })
        }
    }

    /// Checks whether a given [`SemanticVersion`] is valid for defined requirement.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use dsc_lib::types::{SemanticVersion, SemanticVersionReq};
    ///
    /// let requirement = SemanticVersionReq::parse("^1.2.3").unwrap();
    ///
    /// // 1.3.0 is compatible with the requirement.
    /// assert!(requirement.matches(&SemanticVersion::new(1, 3, 0)));
    /// // 2.0.0 isn't compatible with the requirement.
    /// assert!(!requirement.matches(&SemanticVersion::new(2, 0, 0)));
    /// ```
    /// 
    /// The following example shows how the `matches` function treats prerelease versions as not
    /// matching a requirement unless the requirement explicitly defines a prerelease segment.
    /// 
    /// ```rust
    /// # use dsc_lib::types::{SemanticVersion, SemanticVersionReq};
    /// let v_stable = &SemanticVersion::parse("1.2.3").unwrap();
    /// let v_rc1 = &SemanticVersion::parse("1.2.3-rc.1").unwrap();
    /// let v_rc2 = &SemanticVersion::parse("1.2.3-rc.2").unwrap();
    /// 
    /// // Only the stable version matches the stable requirement 
    /// let stable_req = SemanticVersionReq::parse("^1.2.3").unwrap();
    /// assert!(!stable_req.matches(v_rc1));
    /// assert!(!stable_req.matches(v_rc2));
    /// assert!(stable_req.matches(v_stable));
    /// 
    /// // All three versions match the requirement that explicitly defines the prerelease segment
    /// let prerelease_req = SemanticVersionReq::parse("^1.2.3-rc.1").unwrap();
    /// assert!(prerelease_req.matches(v_stable));
    /// assert!(prerelease_req.matches(v_rc1));
    /// assert!(prerelease_req.matches(v_rc2));
    /// 
    /// ```
    pub fn matches(&self, version: &SemanticVersion) -> bool {
        self.0.matches(version.as_ref())
    }

    /// Defines the validating regular expression for semantic version requirements.
    ///
    /// This regular expression is used for the `pattern` keyword in the JSON Schema for the
    /// [`SemanticVersionReq`] type.
    ///
    /// The pattern is also used for validating an instance during parsing and deserialization. DSC
    /// uses a stricter subset of valid syntax for a version requirement:
    ///
    /// - DSC forbids the inclusion of build metadata, which the underlying version requirement
    ///   silently ignores.
    /// - DSC forbids the use of `x` and `X` as wildcards for version segments. Only an asterisk
    ///   (`*`) is a valid wildcard.
    pub const VALIDATING_PATTERN: &str = const_str::concat!(
        "^",                                    // Anchor to start of string
        SemanticVersionReq::COMPARATOR_PATTERN, // Capture first comparator
        "(?:",                                  // Open non-capturing group for additional comparators
        r"\s*,\s*",                             // Additional comparators must follow a comma with optional spacing around it
        SemanticVersionReq::COMPARATOR_PATTERN, // Capture the additional comparator
        ")",                                    // Close the non-capturing group for additional comparators
        "*",                                    // Mark additional comparators as allowed any number of times
        "$",                                    // Anchor to end of string
    );

    /// Returns the [`Regex`] for [`FORBIDDING_BUILD_METADATA_PATTERN`].
    ///
    /// This private method is used to initialize the [`COMPARATOR_HAS_BUILD_METADATA_REGEX`]
    /// private static to reduce the number of times the regular expression is compiled from the
    /// pattern string.
    ///
    /// [`FORBIDDING_BUILD_METADATA_PATTERN`]: SemanticVersionReq::FORBIDDING_BUILD_METADATA_PATTERN
    fn init_build_metadata_pattern() -> Regex {
        Regex::new(Self::FORBIDDING_BUILD_METADATA_PATTERN).expect("pattern is valid")
    }

    /// Returns the [`Regex`] for [`REQUIRE_OPERATOR_PATTERN`].
    ///
    /// This private method is used to initialize the [`COMPARATOR_STARTS_WITH_OPERATOR_REGEX`]
    /// private static to reduce the number of times the regular expression is compiled from the
    /// pattern string.
    ///
    /// [`REQUIRE_OPERATOR_PATTERN`]: SemanticVersionReq::REQUIRE_OPERATOR_PATTERN
    fn init_operator_pattern() -> Regex {
        let pattern = SemanticVersionReq::REQUIRE_OPERATOR_PATTERN;
        Regex::new(pattern).expect("pattern is valid")
    }

    /// Returns the [`Regex`] for [`VALIDATING_WILDCARDS_PATTERN`].
    ///
    /// This private method is used to initialize the [`COMPARATOR_HAS_INVALID_WILDCARD_REGEX`]
    /// private static to reduce the number of times the regular expression is compiled from the
    /// pattern string.
    ///
    /// [`VALIDATING_WILDCARDS_PATTERN`]: SemanticVersionReq::VALIDATING_WILDCARDS_PATTERN
    fn init_wildcard_pattern() -> Regex {
        let pattern = SemanticVersionReq::VALIDATING_WILDCARDS_PATTERN;
        Regex::new(pattern).expect("pattern is valid")
    }


    /// Defines the regular expression for matching a literal version with build metadata.
    ///
    /// DSC forbids the inclusion of build metadata in a version requirement. To provide better
    /// error messaging, DSC uses this pattern to discover the inclusion of build metadata during
    /// parsing and report it to the user.
    pub const FORBIDDING_BUILD_METADATA_PATTERN: &str = const_str::concat!(
        SemanticVersionReq::LITERAL_VERSION_PATTERN,        // Match a literal version
        "(?:",                                              // Open non-capturing group for build metadata and prefix
        r"\+",                                              // Build metadata is always preceded by a plus sign
        SemanticVersion::CAPTURING_BUILD_METADATA_PATTERN,  // Capture the build metadata
        ")",                                                // Close non-capturing group for build metadata and prefix
    );

    /// Defines the regular expression for matching an operator at the beginning of any comparator.
    ///
    /// DSC requires every comparator to define an explicit operator instead of allowing the
    /// implicit operator behavior that [`semver::VersionReq`] supports. To provide better error
    /// messaging, DSC uses this pattern to verify each comparator during parsing and reports any
    /// comparators that are missing their required operator to the user.
    pub const REQUIRE_OPERATOR_PATTERN: &str = const_str::concat!(
        "^",                                    // Anchor to the start of the string
        SemanticVersionReq::OPERATOR_PATTERN    // Match any valid operator
    );

    /// Defines the regular expression for validating wildcards in a comparator.
    ///
    /// DSC forbids defining the major version segment as any wildcard character. DSC also forbids
    /// defining any wildcards as `x` or `X` instead of `*`. To provide better error messaging, DSC
    /// uses this pattern to discover the inclusion of invalid wildcards during parsing and report
    /// it to the user.
    pub const VALIDATING_WILDCARDS_PATTERN: &str = const_str::concat!(
            "^",                                    // Anchor to the start of the string
            SemanticVersionReq::OPERATOR_PATTERN,   // Match any valid operator
            "?",                                    // Make the operator optional
            r"\s*",                                 // Allow any whitespace after operator
            "(?:",                                  // Start non-capturing group for version.
            r"(?<wildcard_major>[\*xX])",           // Match any wildcard for major version
            "|",                                    // or
            r"\d+",                                 // Match literal major version
            "(?:",                                  // Start non-capturing group for optional segments
            r"\.",                                  // Require period after major and before minor
            "(?:",                                  // Start non-capture group for minor version
            r"(?<invalid_minor_wildcard>[xX])",     // Capture invalid wildcard
            "|",                                    // or
            r"\d+",                                 // Match literal version
            "|",                                    // or
            r"\*",                                  // Match valid wildcard
            ")",                                    // Close non-capture group for minor version
            "(?:",                                  // Open non-capture group for optional patch segment
            r"\.",                                  // Require period after minor and before patch
            "(?:",                                  // Open non-capture group for patch version
            r"(?<invalid_patch_wildcard>[xX])",     // Capture invalid wildcard
            "|",                                    // or
            r"\d+",                                 // match literal version
            "|",                                    // or
            r"\*",                                  // match valid wildcard
            ")",                                    // close non-capture group for patch version
            ")?",                                   // close non-capture group for optional patch segment
            ")?",                                   // close non-capture group for optional segments
            ")"                                     // Close non-capturing group for version
        );

    /// Defines the regular expression for matching a wildcard instead of a version segment.
    ///
    /// While Rust supports specifying the wildcard as `x`, `X`, or `*`, DSC only supports `*` to
    /// minimize ambiguity.
    pub const WILDCARD_SYMBOL_PATTERN: &str = r"\*";

    /// Defines the regular expression for matching a version requirement operator.
    ///
    /// Rust and DSC both support the following table of operators:
    ///
    /// | Operator |           Name           |
    /// |:--------:|:------------------------:|
    /// |   `^`    |          Caret           |
    /// |   `~`    |          Tilde           |
    /// |   `=`    |          Equals          |
    /// |   `<`    |        Less than         |
    /// |   `<=`   |  Less than or equal to   |
    /// |   `>`    |       Greater than       |
    /// |   `>=`   | Greater than or equal to |
    pub const OPERATOR_PATTERN: &str = const_str::concat!(
        "(?:", // Open non-capturing group
        ">=",  // Requirements can be greater than or equal to
        "|",   // or
        ">",   // greater than
        "|",   // or
        "<",   // less than
        "|",   // or
        "<=",  // less than or equal to
        "|",   // or
        "=",   // exactly equal
        "|",   // or
        r"\^", // semver-compatible (caret, also default when no prefix defined)
        "|",   // or
        "~",   // minimal-version (tilde)
        ")",   // Close the non-capturing group
    );

    /// Defines the regular expression for matching a comparator with a leading operator followed
    /// by a literal or wildcard version.
    pub const COMPARATOR_PATTERN: &str = const_str::concat!(
        SemanticVersionReq::OPERATOR_PATTERN,           // Match the operator, if any
        r"\s*",                                         // allow any number of spaces after operator
        "(?:",                                          // Open non-capturing group for wildcard-literal version selection
        SemanticVersionReq::LITERAL_VERSION_PATTERN,    // Match literal version
        "|",                                            // or
        SemanticVersionReq::WILDCARD_VERSION_PATTERN,   // Match version with wildcard
        ")",                                            // Close non-capturing group for wildcard-literal version selection
    );

    /// Defines the regular expression for matching a literal version.
    ///
    /// Literal versions must define the major version segment. The minor, patch, and prerelease
    /// segments are optional. The build metadata segment is forbidden.
    pub const LITERAL_VERSION_PATTERN: &str = const_str::concat!(
        "(?:",                                      // Open non-capturing group for literal version
        SemanticVersion::VERSION_SEGMENT_PATTERN,   // Must define the major version.
        "(?:",                                      // Open non-capturing group for optional minor and patch segments
        r"\.",                                      // Major version must be followed by a period if minor is specified.
        SemanticVersion::VERSION_SEGMENT_PATTERN,   // Match the minor version.
        "(?:",                                      // Open non-capturing group for optional patch segment
        r"\.",                                      // Minor version must be followed by a period if patch is specified.
        SemanticVersion::VERSION_SEGMENT_PATTERN,   // Match the patch version.
        SemanticVersionReq::PRERELEASE_PATTERN,     // Match prerelease, if any - only valid with patch
        ")?",                                       // Close non-capturing group for optional patch segment
        ")?",                                       // Close non-capturing group for optional minor and patch segments
        ")",                                        // Close non-capturing group for literal version
    );

    /// Defines the regular expression for matching a version with a wildcard segment.
    ///
    /// Wildcard versions must define the major version segment. The minor and patch segments are
    /// optional. The prerelease and build metadata segments are forbidden.
    ///
    /// If the wildcard version defines the minor version segment as a wildcard, it must not define
    /// the patch segment. If the wildcard version defines the minor version segment as a literal
    /// version segment, it may define the patch version segment as a wildcard.
    ///
    /// The following table shows a few example wildcard versions, whether they are valid, and why
    /// an example version is invalid.
    ///
    /// | Wildcard version | Valid | Notes                                                                               |
    /// |:----------------:|:-----:|:------------------------------------------------------------------------------------|
    /// |       `1.*`      |  Yes  | Defines a literal major version segment followed by a wildcard minor version.       |
    /// |      `1.2.*`     |  Yes  | Defines literal major and minor segments followed by a wildcard patch version.      |
    /// |      `1.*.*`     |  Yes  | Equivalent to `1.*` - both wildcards match any minor and patch version.             |
    /// |      `1.*.3`     |   No  | If the version includes any wildcards, it must be the last defined version segment. |
    /// |     `1.2.3-*`    |   No  | Defines the prerelease segment as a wildcard, which is forbidden.                   |
    pub const WILDCARD_VERSION_PATTERN: &str = const_str::concat!(
        SemanticVersion::VERSION_SEGMENT_PATTERN,       // Must match the (literal) major version
        "(?:",                                          // Open non-capturing group for optional minor and patch segments
        r"\.",                                          // Must follow major version with period before minor version
        "(?:",                                          // Open non-capturing group for literal-or-wildcard minor
        "(?:",                                          // Open non-capturing group for literal minor followed by optional patch
        SemanticVersion::VERSION_SEGMENT_PATTERN,       // Match literal minor version
        "(?:",                                          // Open non-capturing group for optional literal-or-wildcard patch
        r"\.",                                          // Must follow minor version with period before patch version
        "(?:",                                          // Open non-capturing group to select between wildcard-or-literal patch
        SemanticVersion::VERSION_SEGMENT_PATTERN,       // Match literal patch version
        "|",                                            // or
        SemanticVersionReq::WILDCARD_SYMBOL_PATTERN,    // Match patch version as wildcard
        ")",                                            // Close non-capturing group to select between wildcard-or-literal patch
        ")?",                                           // Close non-capturing group for optional literal-or-wildcard patch
        ")",                                            // Close non-capturing group for literal minor followed by optional patch
        "|",                                            // or
        SemanticVersionReq::WILDCARD_SYMBOL_PATTERN,    // Match minor version as wildcard, must not have following patch
        ")",                                            // Close non-capturing group for literal-or-wildcard minor
        ")?",                                           // Close non-capturing group for optional minor and patch segments
    );

    /// Defines the regular expression for matching a literal prerelease segment.
    ///
    /// Prerelease segments are only valid after the patch version in a literal version. A version
    /// requirement must not specify a prerelease segment with any wildcards in it or after a
    /// wildcard version.
    pub const PRERELEASE_PATTERN: &str = const_str::concat!(
        "(?:",                                          // Open non-capturing group for optional prerelease
        "-",                                            // Must precede prerelease segment with a hyphen
        SemanticVersion::PRERELEASE_SEGMENT_PATTERN,    // Match literal prerelease segment, wildcards not allowed
        ")?",                                           // Close non-capturing group for optional prerelease
    );
}

impl JsonSchema for SemanticVersionReq {
    fn schema_name() -> std::borrow::Cow<'static, str> {
        Self::default_schema_id_uri().into()
    }
    fn json_schema(_: &mut schemars::SchemaGenerator) -> schemars::Schema {
        json_schema!({
            "title": t!("schemas.definitions.semverReq.title"),
            "description": t!("schemas.definitions.semverReq.description"),
            "markdownDescription": t!("schemas.definitions.semverReq.markdownDescription"),
            "type": "string",
            "pattern": SemanticVersionReq::VALIDATING_PATTERN,
            "patternErrorMessage": t!("schemas.definitions.semverReq.patternErrorMessage"),
            "examples": [
                "=1.2.3",
                ">=1.2.3, <2.0.0",
                "^1.2",
                "~2.3",
            ]
        })
    }
}

impl Default for SemanticVersionReq {
    fn default() -> Self {
        Self(semver::VersionReq::default())
    }
}

impl Display for SemanticVersionReq {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

// Infallible conversions
impl From<semver::VersionReq> for SemanticVersionReq {
    fn from(value: semver::VersionReq) -> Self {
        Self(value)
    }
}

impl From<SemanticVersionReq> for semver::VersionReq {
    fn from(value: SemanticVersionReq) -> Self {
        value.0
    }
}

impl From<&semver::VersionReq> for SemanticVersionReq {
    fn from(value: &semver::VersionReq) -> Self {
        Self(value.clone())
    }
}

impl From<&SemanticVersionReq> for semver::VersionReq {
    fn from(value: &SemanticVersionReq) -> Self {
        value.0.clone()
    }
}

impl From<SemanticVersionReq> for String {
    fn from(value: SemanticVersionReq) -> Self {
        value.to_string()
    }
}

// Fallible conversions
impl FromStr for SemanticVersionReq {
    type Err = SemanticVersionReqError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

impl TryFrom<String> for SemanticVersionReq {
    type Error = SemanticVersionReqError;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::parse(value.as_str())
    }
}

impl TryFrom<&str> for SemanticVersionReq {
    type Error = SemanticVersionReqError;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        SemanticVersionReq::from_str(value)
    }
}

// Referencing and dereferencing
impl AsRef<semver::VersionReq> for SemanticVersionReq {
    fn as_ref(&self) -> &semver::VersionReq {
        &self.0
    }
}

impl Deref for SemanticVersionReq {
    type Target = semver::VersionReq;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

// Comparison traits
impl PartialEq for SemanticVersionReq {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl PartialEq<semver::VersionReq> for SemanticVersionReq {
    fn eq(&self, other: &semver::VersionReq) -> bool {
        &self.0 == other
    }
}

impl PartialEq<SemanticVersionReq> for semver::VersionReq {
    fn eq(&self, other: &SemanticVersionReq) -> bool {
        self == &other.0
    }
}

impl PartialEq<String> for SemanticVersionReq {
    fn eq(&self, other: &String) -> bool {
        match Self::parse(other.as_str()) {
            Ok(o) => self == &o,
            Err(_) => false
        }
    }
}

impl PartialEq<SemanticVersionReq> for String {
    fn eq(&self, other: &SemanticVersionReq) -> bool {
        match SemanticVersionReq::parse(self.as_str()) {
            Ok(s) => &s == other,
            Err(_) => false
        }
    }
}

impl PartialEq<&String> for SemanticVersionReq {
    fn eq(&self, other: &&String) -> bool {
        match Self::parse(other.as_str()) {
            Ok(o) => self == &o,
            Err(_) => false
        }
    }
}

impl PartialEq<SemanticVersionReq> for &String {
    fn eq(&self, other: &SemanticVersionReq) -> bool {
        match SemanticVersionReq::parse(self.as_str()) {
            Ok(s) => &s == other,
            Err(_) => false
        }
    }
}

impl PartialEq<str> for SemanticVersionReq {
    fn eq(&self, other: &str) -> bool {
        match Self::parse(other) {
            Ok(o) => self == &o,
            Err(_) => false
        }
    }
}

impl PartialEq<SemanticVersionReq> for str {
    fn eq(&self, other: &SemanticVersionReq) -> bool {
        match SemanticVersionReq::parse(self) {
            Ok(s) => &s == other,
            Err(_) => false
        }
    }
}

impl PartialEq<&str> for SemanticVersionReq {
    fn eq(&self, other: &&str) -> bool {
        match Self::parse(*other) {
            Ok(o) => self == &o,
            Err(_) => false
        }
    }
}

impl PartialEq<SemanticVersionReq> for &str {
    fn eq(&self, other: &SemanticVersionReq) -> bool {
        match SemanticVersionReq::parse(*self) {
            Ok(s) => &s == other,
            Err(_) => false
        }
    }
}
