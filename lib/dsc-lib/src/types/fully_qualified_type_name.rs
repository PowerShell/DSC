// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use std::fmt::{Display, Formatter};
use std::hash::Hash;
use std::ops::Deref;
use std::str::FromStr;
use std::sync::OnceLock;

use miette::Diagnostic;
use regex::Regex;
use rust_i18n::t;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::schemas::dsc_repo::DscRepoSchema;

/// Defines the fully qualified type name for a DSC resource or extension. The fully qualified name
/// uniquely identifies each resource and extension.
///
/// # Syntax
///
/// Fully qualified type names use the following syntax:
///
/// ```yaml
/// <owner>[.<namespace>...]/<name>
/// ```
///
/// Where:
///
/// 1. The `owner` segment is mandatory and indicates the party responsible for publishing and
///    maintaining the type.
/// 1. The type may define any number of `namespace` segments for organizing the type. Namespaces
///    must be separated from the `owner` segment and other `namespace` segments by a single dot
///    (`.`).
/// 1. The `name` segment is mandatory and indicates the specific name of the type. It must be
///    separated from the preceding segment by a forward slash (`/`).
/// 1. Every segment must consist only of unicode alphanumeric characters and underscores.
///
/// Conventionally, the first character of each segment is capitalized. When a segment
/// contains a brand or proper name, use the correct casing for that word, like
/// `TailspinToys/Settings`, not `Tailspintoys/Settings`.
///
/// Example fully qualified type names include:
///
/// - `Microsoft/OSInfo`
/// - `Microsoft.SqlServer/Database`
/// - `Microsoft.Windows.IIS/WebApp`
///
/// # Comparisons
///
/// For equivalency, Fully qualified types are compared case-insensitively as strings. Instances
/// [`FullyQualifiedTypeName`] are equal if their string representations are equal when lowercased.
///
/// ```rust
/// # use dsc_lib::types::FullyQualifiedTypeName;
/// # use pretty_assertions::assert_eq;
/// assert_eq!(
///     FullyQualifiedTypeName::parse("Microsoft/OSInfo").unwrap(),
///     FullyQualifiedTypeName::parse("microsoft/osinfo").unwrap()
/// );
/// ```
///
/// For ordering, fully qualified type names are ordered by actual string representations _without_
/// lowercasing. The ordering is [lexicographic][01], reusing the underlying Rust ordering for
/// strings.
///
/// ```rust
/// # use dsc_lib::types::FullyQualifiedTypeName;
/// # use pretty_assertions::assert_eq;
///
/// let mut names = vec![
///     FullyQualifiedTypeName::parse("Microsoft/OSInfo").unwrap(),
///     FullyQualifiedTypeName::parse("Contoso/Resource").unwrap(),
///     FullyQualifiedTypeName::parse("TailspinToys/Settings").unwrap(),
/// ];
/// names.sort();
///
/// assert_eq!(
///     names,
///     vec![
///         FullyQualifiedTypeName::parse("Contoso/Resource").unwrap(),
///         FullyQualifiedTypeName::parse("Microsoft/OSInfo").unwrap(),
///         FullyQualifiedTypeName::parse("TailspinToys/Settings").unwrap(),
///     ]
/// );
/// ```
///
/// # JSON Schema Validation
///
/// For JSON schema validation, the fully qualified type name must be defined as a string and is
/// validated against the following regular expression pattern:
///
/// ```regex
/// "^\w+(\.\w+)*\/\w+$"
/// ```
///
/// This pattern enforces the following rules:
///
/// 1. The `owner` segment must be defined and consist of one or more unicode word characters
///    (alphanumeric or underscores).
/// 1. The `namespace` segments are optional, but if defined, each must consist of one or more
///    unicode word characters and be separated by a single dot (`.`). Consecutive dots or dots at
///    the beginning or end of the owner and namespace portion are forbidden.
/// 1. The `name` segment must be defined and consist of one or more unicode word characters.
///
/// Note that validation for the JSON Schema is necessarily less expressive than the parsing errors
/// from the [`parse()`] method, because the input either validates against the regular expression
/// or it fails entirely without specific diagnostics.
///
/// [01]: https://doc.rust-lang.org/std/cmp/trait.Ord.html#lexicographical-comparison
/// [`parse()`]: Self::parse()
#[derive(
    Clone,
    Debug,
    Eq,
    Serialize,
    Deserialize,
    JsonSchema,
    DscRepoSchema,
)]
#[serde(try_from = "String", into = "String")]
#[schemars(
    title = t!("schemas.definitions.resourceType.title"),
    description = t!("schemas.definitions.resourceType.description"),
    extend(
        "pattern" = FullyQualifiedTypeName::VALIDATING_PATTERN,
        "patternErrorMessage" = t!("schemas.definitions.resourceType.patternErrorMessage"),
        "markdownDescription" = t!("schemas.definitions.resourceType.markdownDescription"),
    )
)]
#[dsc_repo_schema(base_name = "resourceType", folder_path = "definitions")]
pub struct FullyQualifiedTypeName(String);

/// Defines the various errors that can occur when parsing and working with instances of
/// [`FullyQualifiedTypeName`].
#[derive(Debug, Clone, PartialEq, Error, Diagnostic)]
pub enum FullyQualifiedTypeNameError {
    /// Indicates that the provided fully qualified type name is invalid for
    /// one or more reasons.
    ///
    /// This error can occur for multiple reasons, such as invalid characters in the owner,
    /// namespace, or name segments, or if the input string is empty or missing required
    /// segments. The `errors` field contains a list of specific validation errors that were
    /// encountered while parsing the fully qualified type name to enable reviewing all issues
    /// rather than just the first one encountered.
    #[error("{t}", t = t!(
        "types.fully_qualified_type_name.invalidTypeName",
        name = text,
        err = errors.iter().map(|e| e.to_string()).collect::<Vec<String>>().join(", ")
    ))]
    InvalidTypeName {
        /// The input text that failed to parse as a fully qualified type name.
        text: String,
        /// A list of specific validation errors that were encountered while parsing the fully
        /// qualified type name.
        ///
        /// Each error in this list indicates a specific issue with the input text, such as invalid
        /// characters in a segment or missing required segments.
        #[related]
        errors: Vec<FullyQualifiedTypeNameError>,
    },

    /// Indicates that the provided fully qualified type name is an empty string.
    #[error("{t}", t = t!(
        "types.fully_qualified_type_name.emptyTypeName"
    ))]
    EmptyTypeName,

    /// Indicates that the provided fully qualified type name is invalid because it contains
    /// invalid characters in the owner segment (the first segment before any dots or slashes).
    ///
    /// The owner segment must contain only unicode alphanumeric characters and underscores. If it
    /// contains any other characters, validation raises this error in the `errors` field of the
    /// main [`InvalidTypeName`] error.
    ///
    /// [`InvalidTypeName`]: Self::InvalidTypeName
    #[error("{t}", t = t!(
        "types.fully_qualified_type_name.invalidOwnerSegment",
        "owner" => segment_text
    ))]
    InvalidOwnerSegment {
        /// The owner segment text that contains invalid characters.
        segment_text: String,
    },

    /// Indicates that the provided fully qualified type name is invalid because it defines a
    /// namespace segment without defining a non-dot character before it, like
    /// `.Contoso.Example/Resource`.
    ///
    /// The owner segment must be defined before any namespace segments. It must contain only
    /// unicode alphanumeric characters and underscores. If the input string contains a namespace
    /// segment that is not preceded by a valid owner segment, validation raises this error in the
    /// `errors` field of the main [`InvalidTypeName`] error.
    ///
    /// [`InvalidTypeName`]: Self::InvalidTypeName
    #[error("{t}", t = t!(
        "types.fully_qualified_type_name.emptyOwnerSegment"
    ))]
    EmptyOwnerSegment,

    /// Indicates that the provided fully qualified  type name is invalid because it contains
    /// invalid characters in a namespace segment (any segments between the owner and the name,
    /// separated by dots (`.`)).
    ///
    /// Each namespace segment must contain only unicode alphanumeric characters and underscores.
    /// If it contains any other characters, validation raises this error in the `errors` field of
    /// the main [`InvalidTypeName`] error.
    ///
    /// [`InvalidTypeName`]: Self::InvalidTypeName
    #[error("{t}", t = t!(
        "types.fully_qualified_type_name.invalidNamespaceSegment",
        "namespace" => segment_text
    ))]
    InvalidNamespaceSegment {
        /// The namespace segment text that contains invalid characters.
        segment_text: String,
    },

    /// Indicates that the provided fully qualified type name is invalid because it contains
    /// an empty namespace segment (i.e., two consecutive dots with no characters in between).
    ///
    /// If the fully qualified type name contains any empty namespace segments, validation raises
    /// this error in the `errors` field of the main [`InvalidTypeName`] error.
    /// 
    /// [`InvalidTypeName`]: Self::InvalidTypeName
    #[error("{t}", t = t!(
        "types.fully_qualified_type_name.emptyNamespaceSegment",
        "index" => index
    ))]
    EmptyNamespaceSegment {
        /// The 1-based index of the empty namespace segment in the fully qualified type name.
        index: usize,
    },

    /// Indicates that the provided fully qualified type name is invalid because it contains
    /// invalid characters in the name segment (the last segment after the forward slash (`/`)).
    ///
    /// The name segment must contain only unicode alphanumeric characters and underscores. If it
    /// contains any other characters, validation raises this error in the `errors` field of the
    /// main [`InvalidTypeName`] error.
    ///
    /// [`InvalidTypeName`]: Self::InvalidTypeName
    #[error("{t}", t = t!(
        "types.fully_qualified_type_name.invalidNameSegment",
        "name" => segment_text
    ))]
    InvalidNameSegment {
        segment_text: String,
    },

    /// Indicates that the provided fully qualified type name is invalid because it is missing the
    /// required name segment (i.e., the segment after the forward slash (`/`)).
    ///
    /// A fully qualified type name must include a name segment. If the input string is missing the
    /// forward slash or if there are no characters after the forward slash, validation raises this
    /// error in the `errors` field of the main [`InvalidTypeName`] error.
    ///
    /// [`InvalidTypeName`]: Self::InvalidTypeName
    #[error("{t}", t = t!(
        "types.fully_qualified_type_name.missingNameSegment"
    ))]
    MissingNameSegment,
}

/// This static lazily defines the validating regex for [`FullyQualifiedTypeName`]. It enables the
/// [`Regex`] instance to be constructed once, the first time it's used, and then reused on all
/// subsequent validation calls. It's kept private, since the API usage is to invoke the
/// [`FullyQualifiedTypeName::validate()`] method for direct validation or to leverage this static
/// from within the constructor for [`FullyQualifiedTypeName`].
static VALIDATING_SEGMENT_REGEX: OnceLock<Regex> = OnceLock::new();

impl FullyQualifiedTypeName {
    /// Parses a given string into a [`FullyQualifiedTypeName`] instance.
    ///
    /// # Arguments
    ///
    /// - `text` - The input string to parse as a fully qualified type name.
    ///
    /// # Errors
    ///
    /// This function returns a [`FullyQualifiedTypeNameError`] if the input string is not a valid
    /// fully qualified type name. The error will indicate which validation rules were violated and
    /// include diagnostics for every validation error encountered during parsing.
    ///
    /// # Returns
    ///
    /// A result containing the parsed [`FullyQualifiedTypeName`] or a
    /// [`FullyQualifiedTypeNameError`] if the input string is invalid.
    ///
    /// # Examples
    ///
    /// The following snippets shows how various valid input strings are parsed:
    ///
    /// - A minimal valid fully qualified type name defines only the owner and name segments.
    ///   ```rust
    ///   # use dsc_lib::types::FullyQualifiedTypeName;
    ///   let _ = FullyQualifiedTypeName::parse("Contoso/Resource").unwrap();
    ///   ```
    ///
    /// - A fully qualified type name can include namespaces between the owner and name segments,
    ///   separated by a dot (`.`).
    ///
    ///   ```rust
    ///   # use dsc_lib::types::FullyQualifiedTypeName;
    ///   let _ = FullyQualifiedTypeName::parse("Contoso.Example/Resource").unwrap();
    ///   ```
    ///
    /// - A fully qualified type name can include multiple namespaces between the owner and name
    ///   segments.
    ///
    ///   ```rust
    ///   # use dsc_lib::types::FullyQualifiedTypeName;
    ///   let _ = FullyQualifiedTypeName::parse("Contoso.Example.SubExample/Resource").unwrap();
    ///   ```
    ///
    /// The following snippets shows how invalid input strings result in errors:
    ///
    /// - An empty string is not a valid fully qualified type name.
    ///
    ///   ```rust
    ///   # use dsc_lib::types::{FullyQualifiedTypeName, FullyQualifiedTypeNameError};
    ///   # use pretty_assertions::assert_eq;
    ///   assert_eq!(
    ///       FullyQualifiedTypeName::parse("").unwrap_err(),
    ///       FullyQualifiedTypeNameError::EmptyTypeName
    ///   );
    ///   ```
    ///
    /// - A missing name segment is not valid, regardless of whether the `/` is included.
    ///
    ///   ```rust
    ///   # use dsc_lib::types::{FullyQualifiedTypeName, FullyQualifiedTypeNameError};
    ///   # use pretty_assertions::assert_eq;
    ///   for input in ["Contoso", "Contoso/", "Contoso.Example", "Contoso.Example/"] {
    ///       assert_eq!(
    ///           FullyQualifiedTypeName::parse(input).unwrap_err(),
    ///           FullyQualifiedTypeNameError::InvalidTypeName {
    ///               text: input.to_string(),
    ///               errors: vec![FullyQualifiedTypeNameError::MissingNameSegment]
    ///           }
    ///       );
    ///   }
    ///   ```
    /// - An empty owner segment is not valid.
    ///
    ///   ```rust
    ///   # use dsc_lib::types::{FullyQualifiedTypeName, FullyQualifiedTypeNameError};
    ///   # use pretty_assertions::assert_eq;
    ///   for input in [".Contoso.Example/Resource", "/Resource"] {
    ///       assert_eq!(
    ///           FullyQualifiedTypeName::parse(input).unwrap_err(),
    ///           FullyQualifiedTypeNameError::InvalidTypeName {
    ///               text: input.to_string(),
    ///               errors: vec![FullyQualifiedTypeNameError::EmptyOwnerSegment]
    ///           }
    ///       );
    ///   }
    ///   ```
    ///
    /// - Characters other than unicode alphanumeric characters and underscores in any segment are
    ///   not valid.
    ///
    ///   ```rust
    ///   # use dsc_lib::types::{FullyQualifiedTypeName, FullyQualifiedTypeNameError};
    ///   # use pretty_assertions::assert_eq;
    ///   let input = "Contoso&Invalid.Example!Invalid/Resource-Invalid";
    ///   assert_eq!(
    ///      FullyQualifiedTypeName::parse(input).unwrap_err(),
    ///         FullyQualifiedTypeNameError::InvalidTypeName {
    ///          text: input.to_string(),
    ///          errors: vec![
    ///              FullyQualifiedTypeNameError::InvalidOwnerSegment {
    ///                  segment_text: "Contoso&Invalid".to_string(),
    ///             },
    ///             FullyQualifiedTypeNameError::InvalidNamespaceSegment {
    ///                  segment_text: "Example!Invalid".to_string(),
    ///             },
    ///             FullyQualifiedTypeNameError::InvalidNameSegment {
    ///                  segment_text: "Resource-Invalid".to_string(),
    ///             },
    ///          ],
    ///      }
    ///   );
    ///   ```
    /// 
    /// - An empty namespace segment is not valid.
    ///
    ///   ```rust
    ///   # use dsc_lib::types::{FullyQualifiedTypeName, FullyQualifiedTypeNameError};
    ///   # use pretty_assertions::assert_eq;
    ///   let input = "Contoso.Example.With.Empty..Namespace/Resource";
    ///   assert_eq!(
    ///       FullyQualifiedTypeName::parse(input).unwrap_err(),
    ///       FullyQualifiedTypeNameError::InvalidTypeName {
    ///           text: input.to_string(),
    ///           errors: vec![FullyQualifiedTypeNameError::EmptyNamespaceSegment {
    ///               index: 4
    ///           }],
    ///       }
    ///   );
    ///   ```
    pub fn parse(text: &str) -> Result<Self, FullyQualifiedTypeNameError> {
        // If the input text is empty, return an error immediately to avoid unnecessary processing.
        if text.is_empty() {
            return Err(FullyQualifiedTypeNameError::EmptyTypeName);
        }

        let errors = &mut Vec::<FullyQualifiedTypeNameError>::new();
        let owner: String;
        let namespaces: Vec<String>;
        let name: String;
        let validating_segment_regex = Self::init_validating_segment_regex();

        if let Some((owner_and_namespaces, name_segment)) = text.rsplit_once('/') {
            name = name_segment.to_string();
            let mut segments = owner_and_namespaces
                .split('.')
                .map(|s| s.to_string())
                .collect::<Vec<String>>();
            owner = segments.remove(0);
            namespaces = segments;
        } else if text.contains('.') {
            let mut segments = text
                .split('.')
                .map(|s| s.to_string())
                .collect::<Vec<String>>();
            owner = segments.remove(0);
            namespaces = segments;
            name = String::new();
        } else {
            owner = text.to_string();
            namespaces = Vec::new();
            name = String::new();
        }

        if owner.is_empty() {
            errors.push(FullyQualifiedTypeNameError::EmptyOwnerSegment);
        } else if !validating_segment_regex.is_match(&owner) {
            errors.push(FullyQualifiedTypeNameError::InvalidOwnerSegment {
                segment_text: owner.clone(),
            });
        }

        for (index, namespace) in namespaces.into_iter().enumerate() {
            if namespace.is_empty() {
                errors.push(FullyQualifiedTypeNameError::EmptyNamespaceSegment {
                    // Insert the index as 1-based for more user-friendly error messages
                    index: index + 1
                });
            } else if !validating_segment_regex.is_match(&namespace) {
                 errors.push(FullyQualifiedTypeNameError::InvalidNamespaceSegment {
                    segment_text: namespace.clone(),
                });
            }
        }

        if name.is_empty() {
            errors.push(FullyQualifiedTypeNameError::MissingNameSegment);
        } else if !validating_segment_regex.is_match(&name) {
            errors.push(FullyQualifiedTypeNameError::InvalidNameSegment {
                segment_text: name.clone()
            });
        }

        if errors.is_empty() {
            Ok(Self(text.to_string()))
        } else {
            Err(FullyQualifiedTypeNameError::InvalidTypeName {
                text: text.to_string(),
                errors: errors.clone(),
            })
        }
    }

    /// Defines the regular expression for validating a string as a fully qualified type name.
    ///
    /// This pattern is only used for the JSON Schema validation of the entire type name string.
    /// When parsing and validating a type name string, the implementation slices the string into
    /// its segments (owner, namespaces, and name) and validates each segment individually against
    /// the [`VALIDATING_SEGMENT_PATTERN`] for more specific error messages indicating which
    /// segment(s) are invalid and how.
    ///
    /// The string must begin with one or more unicode alphanumeric characters and underscores that
    /// define the `owner` for the type. Following the `owner` segment, the string may include any
    /// number of `namespace` segments, which must be separated from the previous segment by a
    /// single period (`.`). Finally, the string must include a forward slash (`/`) followed by one
    /// or more unicode alphanumeric characters and underscores to define the `name` segment.
    /// 
    /// [`VALIDATING_SEGMENT_PATTERN`]: Self::VALIDATING_SEGMENT_PATTERN
    pub const VALIDATING_PATTERN: &str = r"^\w+(\.\w+)*\/\w+$";

    /// Defines the regular expression for validating a segment in a fully qualified type name.
    ///
    /// Each segment must contain only unicode alphanumeric characters and underscores. This
    /// regular expression is applied to each individual segment of the type name (owner,
    /// namespaces, and name) rather than the entire type name string to provide more specific
    /// error messages indicating which segment is invalid when a type name fails validation.
    /// For example, if the type name is `Owner.Namespace/Name` and the `Namespace` segment
    /// contains an invalid character, the error message can specifically indicate that the
    /// `Namespace` segment is invalid rather than just indicating that the entire type name
    /// is invalid.
    ///
    /// This also obviates needing to check for the namespace separator (`.`) and type/name
    /// separator (`/`) in the regex pattern, since the segments are validated individually after
    /// slicing the input string based on those separators.
    pub const VALIDATING_SEGMENT_PATTERN: &'static str = r"^\w+$";

    /// Initializes and returns the validating regex for type name segments and returns a reference
    /// to the compiled regex.
    ///
    /// This helps avoid recompiling the same regex on every validation call by constructing it
    /// once and then reusing it for subsequent validations.
    fn init_validating_segment_regex() -> &'static Regex {
        VALIDATING_SEGMENT_REGEX.get_or_init(|| Regex::new(Self::VALIDATING_SEGMENT_PATTERN).unwrap())
    }
}

// While it's technically never valid for a _defined_ FQTN to be empty, we need the default
// implementation for creating empty instances of various structs to then populate/modify.
impl Default for FullyQualifiedTypeName {
    fn default() -> Self {
        Self(String::new())
    }
}

// We implement `PartialEq` by hand for various types because FQTNs should be compared
// case insensitively. This obviates the need to `.to_string().to_lowercase()` for comparisons.
impl PartialEq for FullyQualifiedTypeName {
    fn eq(&self, other: &Self) -> bool {
        self.0.to_lowercase() == other.0.to_lowercase()
    }
}

impl PartialEq<String> for FullyQualifiedTypeName {
    fn eq(&self, other: &String) -> bool {
        self.0.to_lowercase() == other.to_lowercase()
    }
}

impl PartialEq<FullyQualifiedTypeName> for String {
    fn eq(&self, other: &FullyQualifiedTypeName) -> bool {
        self.to_lowercase() == other.0.to_lowercase()
    }
}

impl PartialEq<str> for FullyQualifiedTypeName {
    fn eq(&self, other: &str) -> bool {
        self.0.to_lowercase() == other.to_lowercase()
    }
}

impl PartialEq<FullyQualifiedTypeName> for str {
    fn eq(&self, other: &FullyQualifiedTypeName) -> bool {
        self.to_lowercase() == other.0.to_lowercase()
    }
}

impl PartialEq<&str> for FullyQualifiedTypeName {
    fn eq(&self, other: &&str) -> bool {
        self.0.to_lowercase() == other.to_lowercase()
    }
}

impl PartialEq<FullyQualifiedTypeName> for &str {
    fn eq(&self, other: &FullyQualifiedTypeName) -> bool {
        self.to_lowercase() == other.0.to_lowercase()
    }
}

// Implement `Ord` and `PartialOrd` by hand to ignore case sensitivity.
impl Ord for FullyQualifiedTypeName {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.to_lowercase().cmp(&other.0.to_lowercase())
    }
}

impl PartialOrd for FullyQualifiedTypeName {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

// Enables using the construct `"Owner/Name".parse()` to convert a literal string into an FQTN.
impl FromStr for FullyQualifiedTypeName {
    type Err = FullyQualifiedTypeNameError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

// Enables converting from a `String` and raising the appropriate error message for an invalid
// FQTN.
impl TryFrom<String> for FullyQualifiedTypeName {
    type Error = FullyQualifiedTypeNameError;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::parse(value.as_str())
    }
}

impl TryFrom<&str> for FullyQualifiedTypeName {
    type Error = FullyQualifiedTypeNameError;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::parse(value)
    }
}

// Enables converting an FQTN into a string.
impl From<FullyQualifiedTypeName> for String {
    fn from(value: FullyQualifiedTypeName) -> Self {
        value.0
    }
}

// Enables using FQTNs in `format!()` and similar macros.
impl Display for FullyQualifiedTypeName {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

// Enables passing an FQTN as `&str`
impl AsRef<str> for FullyQualifiedTypeName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

// Enables directly accessing string methods on an FQTN, like `.to_lowercase()` or `starts_with()`.
impl Deref for FullyQualifiedTypeName {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Hash for FullyQualifiedTypeName {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.to_lowercase().hash(state);
    }
}
