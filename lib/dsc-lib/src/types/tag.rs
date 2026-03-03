// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use std::{fmt::{Display, Formatter}, hash::Hash, ops::Deref, str::FromStr, sync::OnceLock};

use regex::Regex;
use rust_i18n::t;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::dscerror::DscError;

/// Defines a short term that applies to a DSC resource or extension and can be used for filtering.
///
/// DSC enables filtering for various types by their defined tags. A valid tag must be defined as
/// a string containing _only_ unicode word characters. The following list contains the valid
/// character sets for a tag:
///
/// - `\p{alpha}` - All alphabetic characters, regardless of casing, use of marks, or script.
/// - `\p{gc=Mark}` - Characters that modify other characters
/// - `\p{digit}` - Characters `[0-9]`
/// - `\p{gc=Connector_Punctuation}` - Underscore (`_`) and similar characters
/// - `\p{Join_Control}` - characters that function for cursive joining and ligatures
///
/// Empty strings and strings containing invalid characters are not valid tags.
///
/// For more information on unicode word characters, see [Annex C: Compatibility property][01] in
/// [Unicode Technical Standard #18][02].
///
/// [01]: https://www.unicode.org/reports/tr18/#Compatibility_Properties
/// [02]: https://www.unicode.org/reports/tr18
#[derive(
    Clone,
    Debug,
    Serialize,
    Deserialize,
    JsonSchema
)]
#[serde(try_from = "String")]
#[schemars(
    title = t!("schemas.definitions.tag.title"),
    description = t!("schemas.definitions.tag.description"),
    extend(
        "pattern" = Tag::VALIDATING_PATTERN,
        "patternErrorMessage" = t!("schemas.definitions.tag.patternErrorMessage"),
        "markdownDescription" = t!("schemas.definitions.tag.markdownDescription"),
    ),
    inline
)]
pub struct Tag(String);

/// This static lazily defines the validating regex for [`Tag`]. It enables the [`Regex`] instance
/// to be constructed once, the first time it's used, and then reused on all subsequent validation
/// calls. It's kept private, since the API usage is to invoke the [`Tag::validate()`] method for
/// direct validation or to leverage this static from within the constructor for [`Tag`].
static VALIDATING_REGEX: OnceLock<Regex> = OnceLock::new();

impl Tag {
    /// Creates a new instance of [`Tag`] from a string if the input is valid for the
    /// [`VALIDATING_PATTERN`]. If the string is invalid, the method raises the
    /// [`DscError::InvalidTag`] error.
    ///
    /// [`VALIDATING_PATTERN`]: Tag::VALIDATING_PATTERN
    pub fn new(name: &str) -> Result<Self, DscError> {
        Self::validate(name)?;
        Ok(Self(name.to_string()))
    }

    /// Validates a given string as a custom tag.
    ///
    /// A string is valid if it matches the [`VALIDATING_PATTERN`]. If the string is invalid, DSC
    /// raises the [`DscError::InvalidTag`] error.
    ///
    /// [`VALIDATING_PATTERN`]: Tag::VALIDATING_PATTERN
    pub fn validate(name: &str) -> Result<(), DscError> {
        let pattern = VALIDATING_REGEX.get_or_init(Self::init_pattern);
        match pattern.is_match(name) {
            true => Ok(()),
            false => Err(DscError::InvalidTag(
                name.to_string(),
                pattern.to_string(),
            )),
        }
    }

    /// Defines the regular expression for validating a string as a tag.
    ///
    /// The string must consist _only_ of unicode word characters. The following list contains the
    /// valid character sets for a tag:
    ///
    /// - `\p{alpha}` - All alphabetic characters, regardless of casing, use of marks, or script.
    /// - `\p{gc=Mark}` - Characters that modify other characters
    /// - `\p{digit}` - Characters `[0-9]`
    /// - `\p{gc=Connector_Punctuation}` - Underscore (`_`) and similar characters
    /// - `\p{Join_Control}` - characters that function for cursive joining and ligatures
    ///
    /// For more information, see [Annex C: Compatibility property][01] in
    /// [Unicode Technical Standard #18][02].
    ///
    /// [01]: https://www.unicode.org/reports/tr18/#Compatibility_Properties
    /// [02]: https://www.unicode.org/reports/tr18
    pub const VALIDATING_PATTERN: &str = r"^\w+$";

    /// Returns the [`Regex`] for [`Self::VALIDATING_PATTERN`].
    ///
    /// This private method is used to initialize the [`VALIDATING_REGEX`] private static to reduce
    /// the number of times the regular expression is compiled from the pattern string.
    fn init_pattern() -> Regex {
        Regex::new(Self::VALIDATING_PATTERN).expect("pattern is valid")
    }
}

// Enables using tags in `format!()` and similar macros.
impl Display for Tag {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Hash for Tag {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        // Hash on the lowercased characters instead of lowercasing the string for performance
        for ch in self.0.chars() {
            for lower in ch.to_lowercase() {
                lower.hash(state);
            }
        }
    }
}

// Enables passing a tag as `&str`
impl AsRef<str> for Tag {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

// Enables directly accessing string methods on a tag, like `.to_lowercase()` or `starts_with()`.
impl Deref for Tag {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}

impl FromStr for Tag {
    type Err = DscError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s)
    }
}

// Enables converting from a `String` and raising the appropriate error message for an invalid tag.
impl TryFrom<String> for Tag {
    type Error = DscError;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::validate(&value)?;
        Ok(Self(value))
    }
}

// Enables converting a tag into a string.
impl From<Tag> for String {
    fn from(value: Tag) -> Self {
        value.0
    }
}

// Enables converting from a string slice and raising the appropriate error message for an invalid
// tag.
impl TryFrom<&str> for Tag {
    type Error = DscError;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

// We implement `PartialEq` by hand for various types because tags should be compared case
// insensitively. This obviates the need to `.to_string().to_lowercase()` for comparisons.
impl PartialEq for Tag {
    fn eq(&self, other: &Self) -> bool {
        self.to_lowercase() == other.to_lowercase()
    }
}

impl Eq for Tag {}

impl PartialEq<String> for Tag {
    fn eq(&self, other: &String) -> bool {
        self.to_lowercase() == other.to_lowercase()
    }
}

impl PartialEq<Tag> for String {
    fn eq(&self, other: &Tag) -> bool {
        self.to_lowercase() == other.to_lowercase()
    }
}

impl PartialEq<&String> for Tag {
    fn eq(&self, other: &&String) -> bool {
        self.to_lowercase() == other.to_lowercase()
    }
}

impl PartialEq<Tag> for &String {
    fn eq(&self, other: &Tag) -> bool {
        self.to_lowercase() == other.to_lowercase()
    }
}

impl PartialEq<str> for Tag {
    fn eq(&self, other: &str) -> bool {
        self.to_lowercase() == other.to_lowercase()
    }
}

impl PartialEq<Tag> for str {
    fn eq(&self, other: &Tag) -> bool {
        self.to_lowercase() == other.to_lowercase()
    }
}

impl PartialEq<&str> for Tag {
    fn eq(&self, other: &&str) -> bool {
        self.to_lowercase() == other.to_lowercase()
    }
}

impl PartialEq<Tag> for &str {
    fn eq(&self, other: &Tag) -> bool {
        self.to_lowercase() == other.to_lowercase()
    }
}

// Implement ordering case insensitively
impl Ord for Tag {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.to_lowercase().cmp(&other.to_lowercase())
    }
}

impl PartialOrd for Tag {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialOrd<String> for Tag {
    fn partial_cmp(&self, other: &String) -> Option<std::cmp::Ordering> {
        self.to_lowercase().partial_cmp(&other.to_lowercase())
    }
}

impl PartialOrd<Tag> for String {
    fn partial_cmp(&self, other: &Tag) -> Option<std::cmp::Ordering> {
        self.to_lowercase().partial_cmp(&other.to_lowercase())
    }
}

impl PartialOrd<str> for Tag {
    fn partial_cmp(&self, other: &str) -> Option<std::cmp::Ordering> {
        self.to_lowercase().partial_cmp(&other.to_lowercase())
    }
}

impl PartialOrd<Tag> for str {
    fn partial_cmp(&self, other: &Tag) -> Option<std::cmp::Ordering> {
        self.to_lowercase().partial_cmp(&other.to_lowercase())
    }
}

impl PartialOrd<&str> for Tag {
    fn partial_cmp(&self, other: &&str) -> Option<std::cmp::Ordering> {
        self.to_lowercase().partial_cmp(&other.to_lowercase())
    }
}

impl PartialOrd<Tag> for &str {
    fn partial_cmp(&self, other: &Tag) -> Option<std::cmp::Ordering> {
        self.to_lowercase().partial_cmp(&other.to_lowercase())
    }
}
