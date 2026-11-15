// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use std::{fmt::{Display, Formatter}, ops::Deref, str::FromStr};

use miette::Diagnostic;
use rust_i18n::t;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::schemas::dsc_repo::{DscRepoSchema, schema_i18n};

/// Defines the name of the command a plug-in adds to a DSC host.
///
/// A command name is what the user types after the name of the host, like `lint` in `dsc lint`.
/// A valid command name:
///
/// - Starts with a lowercase ASCII letter.
/// - Contains only lowercase ASCII letters, ASCII digits, and hyphens.
/// - Doesn't end with a hyphen or contain consecutive hyphens.
///
/// These rules are the same as the [`VALIDATING_PATTERN`]. DSC compares command names
/// case-sensitively, and because a valid name never contains an uppercase letter, two names are
/// equal only when their text is identical.
///
/// Every constructor validates these rules, and deserializing a name that breaks them fails, so an
/// instance is always valid.
///
/// # Examples
///
/// ```rust
/// use dsc_lib::plugins::command_name::PluginCommandName;
///
/// let name = PluginCommandName::parse("lint").unwrap();
/// assert_eq!(name, "lint");
///
/// assert!(PluginCommandName::parse("Lint").is_err());
/// assert!(PluginCommandName::parse("resource-kit").is_ok());
/// assert!(PluginCommandName::parse("resource--kit").is_err());
/// ```
///
/// [`VALIDATING_PATTERN`]: PluginCommandName::VALIDATING_PATTERN
#[derive(
    Clone,
    Debug,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    Serialize,
    Deserialize,
    JsonSchema,
    DscRepoSchema
)]
#[dsc_repo_schema(base_name = "pluginCommandName", folder_path = "definitions")]
#[serde(try_from = "String", into = "String")]
#[schemars(
    transform = PluginCommandName::transform_export_schema_uris,
    transform = PluginCommandName::transform_schema_docs_strict,
    extend(
        "pattern" = PluginCommandName::VALIDATING_PATTERN,
        "patternErrorMessage" = schema_i18n!("patternErrorMessage"),
    )
)]
pub struct PluginCommandName(String);

/// Defines the errors that can occur when parsing a [`PluginCommandName`].
#[derive(Debug, Clone, PartialEq, Eq, Error, Diagnostic)]
pub enum PluginCommandNameError {
    /// Indicates that the text isn't a valid command name for one or more reasons.
    ///
    /// The `errors` field lists every rule the text breaks, so the author of a plug-in manifest
    /// can review all of them at once instead of discovering them one at a time.
    #[error("{t}", t = t!(
        "plugins.command_name.invalidName",
        name = text,
        err = errors.iter().map(ToString::to_string).collect::<Vec<String>>().join(", ")
    ))]
    InvalidName {
        /// The text that failed to parse as a command name.
        text: String,
        /// The specific rules the text breaks.
        #[related]
        errors: Vec<PluginCommandNameError>,
    },

    /// Indicates that the text is an empty string.
    ///
    /// Validation raises this error in the `errors` field of the main [`InvalidName`] error.
    ///
    /// [`InvalidName`]: Self::InvalidName
    #[error("{t}", t = t!("plugins.command_name.emptyName"))]
    EmptyName,

    /// Indicates that the text doesn't start with a lowercase ASCII letter.
    ///
    /// Validation raises this error in the `errors` field of the main [`InvalidName`] error.
    ///
    /// [`InvalidName`]: Self::InvalidName
    #[error("{t}", t = t!("plugins.command_name.invalidFirstCharacter", character = character))]
    InvalidFirstCharacter {
        /// The first character of the text.
        character: char,
    },

    /// Indicates that the text contains characters other than lowercase ASCII letters, ASCII
    /// digits, and hyphens.
    ///
    /// Validation raises this error in the `errors` field of the main [`InvalidName`] error.
    ///
    /// [`InvalidName`]: Self::InvalidName
    #[error("{t}", t = t!("plugins.command_name.invalidCharacters", characters = characters))]
    InvalidCharacters {
        /// The distinct invalid characters, in the order they first appear in the text.
        characters: String,
    },

    /// Indicates that the text ends with a hyphen or contains consecutive hyphens.
    ///
    /// Validation raises this error in the `errors` field of the main [`InvalidName`] error.
    ///
    /// [`InvalidName`]: Self::InvalidName
    #[error("{t}", t = t!("plugins.command_name.misplacedHyphen"))]
    MisplacedHyphen,
}

impl PluginCommandName {
    /// Defines the regular expression that a valid command name matches.
    ///
    /// The name must start with a lowercase ASCII letter, followed by any number of lowercase
    /// ASCII letters and ASCII digits. A hyphen may separate groups of letters and digits, but the
    /// name can't end with a hyphen or contain consecutive hyphens.
    ///
    /// [`parse()`] checks the same rules without the regular expression so it can report every
    /// rule a name breaks. The pattern is published in the JSON Schema for the type.
    ///
    /// [`parse()`]: Self::parse
    pub const VALIDATING_PATTERN: &str = r"^[a-z][a-z0-9]*(-[a-z0-9]+)*$";

    /// Parses a string as a command name.
    ///
    /// # Arguments
    ///
    /// * `text` - The text to parse as a command name.
    ///
    /// # Errors
    ///
    /// Returns [`PluginCommandNameError::InvalidName`] when the text breaks any of the rules for
    /// a command name. The error lists every rule the text breaks.
    pub fn parse(text: &str) -> Result<Self, PluginCommandNameError> {
        Self::validate(text)?;
        Ok(Self(text.to_string()))
    }

    /// Validates a string as a command name without creating an instance.
    ///
    /// # Arguments
    ///
    /// * `text` - The text to validate as a command name.
    ///
    /// # Errors
    ///
    /// Returns [`PluginCommandNameError::InvalidName`] when the text breaks any of the rules for
    /// a command name. The error lists every rule the text breaks:
    ///
    /// * [`PluginCommandNameError::EmptyName`] when the text is an empty string. No other rule is
    ///   checked for an empty string.
    /// * [`PluginCommandNameError::InvalidFirstCharacter`] when the text doesn't start with a
    ///   lowercase ASCII letter.
    /// * [`PluginCommandNameError::InvalidCharacters`] when the text contains characters other
    ///   than lowercase ASCII letters, ASCII digits, and hyphens.
    /// * [`PluginCommandNameError::MisplacedHyphen`] when the text ends with a hyphen or contains
    ///   consecutive hyphens.
    pub fn validate(text: &str) -> Result<(), PluginCommandNameError> {
        let mut errors = Vec::new();

        match text.chars().next() {
            None => errors.push(PluginCommandNameError::EmptyName),
            Some(first) => {
                if !first.is_ascii_lowercase() {
                    errors.push(PluginCommandNameError::InvalidFirstCharacter { character: first });
                }

                let mut invalid_characters = String::new();
                for character in text.chars() {
                    let is_valid = character.is_ascii_lowercase()
                        || character.is_ascii_digit()
                        || character == '-';
                    if !is_valid && !invalid_characters.contains(character) {
                        invalid_characters.push(character);
                    }
                }
                if !invalid_characters.is_empty() {
                    errors.push(PluginCommandNameError::InvalidCharacters {
                        characters: invalid_characters,
                    });
                }

                if text.ends_with('-') || text.contains("--") {
                    errors.push(PluginCommandNameError::MisplacedHyphen);
                }
            },
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(PluginCommandNameError::InvalidName { text: text.to_string(), errors })
        }
    }
}

// Enables using command names in `format!()` and similar macros.
impl Display for PluginCommandName {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

// Enables passing a command name as `&str`.
impl AsRef<str> for PluginCommandName {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

// Enables directly accessing string methods on a command name, like `starts_with()`.
impl Deref for PluginCommandName {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}

impl FromStr for PluginCommandName {
    type Err = PluginCommandNameError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

// Enables converting from a `String` and raising the appropriate error for an invalid name. Serde
// uses this implementation to validate a name while deserializing it.
impl TryFrom<String> for PluginCommandName {
    type Error = PluginCommandNameError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::validate(&value)?;
        Ok(Self(value))
    }
}

// Enables converting from a string slice and raising the appropriate error for an invalid name.
impl TryFrom<&str> for PluginCommandName {
    type Error = PluginCommandNameError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::parse(value)
    }
}

// Enables infallibly converting a command name into the underlying string.
impl From<PluginCommandName> for String {
    fn from(value: PluginCommandName) -> Self {
        value.0
    }
}

// Enable comparing the newtype with strings in either direction.
impl PartialEq<String> for PluginCommandName {
    fn eq(&self, other: &String) -> bool {
        &self.0 == other
    }
}

impl PartialEq<PluginCommandName> for String {
    fn eq(&self, other: &PluginCommandName) -> bool {
        self == &other.0
    }
}

impl PartialEq<str> for PluginCommandName {
    fn eq(&self, other: &str) -> bool {
        self.0 == other
    }
}

impl PartialEq<PluginCommandName> for str {
    fn eq(&self, other: &PluginCommandName) -> bool {
        self == other.0
    }
}

impl PartialEq<&str> for PluginCommandName {
    fn eq(&self, other: &&str) -> bool {
        self.0 == *other
    }
}

impl PartialEq<PluginCommandName> for &str {
    fn eq(&self, other: &PluginCommandName) -> bool {
        *self == other.0
    }
}
