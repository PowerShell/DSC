use std::{fmt::Display, str::FromStr};

use miette::Diagnostic;
use rust_i18n::t;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::types::{FullyQualifiedTypeName, FullyQualifiedTypeNameError, WildcardTypeName, WildcardTypeNameError};

#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub enum TypeNameFilter {
    /// Filter by exact type name.
    Literal(FullyQualifiedTypeName),
    /// Filter by wildcard type name, where `*` can be used as a wildcard character.
    Wildcard(WildcardTypeName),
}

/// Defines errors that can occur when parsing or working with a [`TypeNameFilter`].
/// 
/// This includes errors from parsing both [`FullyQualifiedTypeName`]s and [`WildcardTypeName`]s.
#[derive(Debug, Clone, Error, Diagnostic, PartialEq)]
pub enum TypeNameFilterError {
    /// Indicates a parsing error for a [`Wildcard`] type name filter.
    /// 
    /// [`Wildcard`]: TypeNameFilter::Wildcard
    #[error("{t}", t = t!(
        "types.type_name_filter.invalidWildcardTypeNameFilter",
        "err" => source
    ))]
    InvalidWildcardTypeNameFilter{
        /// The source error that occurred while parsing the wildcard type name filter.
        #[from]
        source: WildcardTypeNameError,
    },

    /// Indicates a parsing error for a [`Literal`] type name filter.
    /// 
    /// [`Literal`]: TypeNameFilter::Literal
    #[error("{t}", t = t!(
        "types.type_name_filter.invalidLiteralTypeNameFilter",
        "err" => source
    ))]
    InvalidLiteralTypeNameFilter{
        /// The source error that occurred while parsing the literal type name filter.
        #[from]
        source: FullyQualifiedTypeNameError,
    },

    /// Indicates that conversion failed for a [`Literal`] instance into a [`WildcardTypeName`].
    /// 
    /// [`Literal`]: TypeNameFilter::Literal
    #[error("{t}", t = t!(
        "types.type_name_filter.invalidConversionToWildcardTypeName",
        "type_name" => type_name
    ))]
    InvalidConversionToWildcardTypeName{
        /// The inner literal type name that failed to convert to a wildcard type name.
        type_name: FullyQualifiedTypeName,
    },

    /// Indicates that conversion failed for a [`Wildcard`] instance into a
    /// [`FullyQualifiedTypeName`].
    /// 
    /// [`Wildcard`]: TypeNameFilter::Wildcard
    #[error("{t}", t = t!(
        "types.type_name_filter.invalidConversionToFullyQualifiedTypeName",
        "type_name" => type_name
    ))]
    InvalidConversionToFullyQualifiedTypeName{
        /// The inner wildcard type name that failed to convert to a fully qualified type name.
        type_name: WildcardTypeName,
    },
}

impl TypeNameFilter {
    /// Parses a string into a `TypeNameFilter`. The string can be either a literal fully qualified
    /// type name or a wildcard type name.
    /// 
    /// If the input string contains a `*`, it will be parsed as a `WildcardTypeName`. Otherwise,
    /// it will be parsed as a `FullyQualifiedTypeName`.
    ///
    /// # Arguments
    ///
    /// - `text` - The input string to parse.
    /// 
    /// # Errors
    /// 
    /// This function returns a [`TypeNameFilterError`] if the input string is not a valid literal
    /// fully qualified type name ([`FullyQualifiedTypeName`]) or a valid wildcard type name
    /// ([`WildcardTypeName`]). The error will indicate which type of parsing failed and include
    /// diagnostics for every validation error encountered during parsing.
    ///
    /// # Returns
    ///
    /// A result containing the parsed [`TypeNameFilter`] or an error if the input is invalid.
    /// 
    /// # Examples
    /// 
    /// The following snippet shows how various valid input strings are parsed:
    /// 
    /// ```rust
    /// use dsc_lib::types::{TypeNameFilter, TypeNameFilterError};
    /// let literal = TypeNameFilter::parse("Contoso.Example/Resource").unwrap();
    /// assert!(matches!(literal, TypeNameFilter::Literal(_)));
    /// let wildcard_name = TypeNameFilter::parse("Contoso.Example/*").unwrap();
    /// assert!(matches!(wildcard_name, TypeNameFilter::Wildcard(_)));
    /// let wildcard_namespace = TypeNameFilter::parse("Contoso.*").unwrap();
    /// assert!(matches!(wildcard_namespace, TypeNameFilter::Wildcard(_)));
    /// let wildcard_owner = TypeNameFilter::parse("*/Resource").unwrap();
    /// assert!(matches!(wildcard_owner, TypeNameFilter::Wildcard(_)));
    /// ```
    /// 
    /// The following snippet shows how invalid input strings result in errors:
    /// 
    /// ```rust
    /// use dsc_lib::types::{TypeNameFilter, TypeNameFilterError};
    /// let invalid = TypeNameFilter::parse("Invalid/Name/With/Too/Many/Segments").unwrap_err();
    /// assert!(matches!(invalid, TypeNameFilterError::InvalidLiteralTypeNameFilter{..}));
    /// let invalid_wildcard = TypeNameFilter::parse("Invalid*Wildcard!").unwrap_err();
    /// assert!(matches!(invalid_wildcard, TypeNameFilterError::InvalidWildcardTypeNameFilter{..}));
    /// ```
    pub fn parse(text: &str) -> Result<Self, TypeNameFilterError> {
        // If the text contains a '*', attempt to parse it as a WildcardTypeName. Otherwise, parse
        // it as a FullyQualifiedTypeName.
        if text.contains('*') {
            let wildcard = WildcardTypeName::parse(text)?;
            Ok(TypeNameFilter::Wildcard(wildcard))
        } else {
            let literal = FullyQualifiedTypeName::parse(text)?;
            Ok(TypeNameFilter::Literal(literal))
        }
    }

    /// Checks if the filter is empty. A filter is considered empty if it does not contain any
    /// valid type name information.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        match self {
            TypeNameFilter::Literal(literal) => literal.is_empty(),
            TypeNameFilter::Wildcard(wildcard) => wildcard.is_empty(),
        }
    }

    /// Checks if a given candidate [`FullyQualifiedTypeName`] matches the filter.
    /// 
    /// For a literal filter, the match is exact. A candidate matches a literal filter only if it's
    /// exactly equal to the literal type name. [`FullyQualifiedTypeName`] comparisons are
    /// case-insensitive, so the candidate `Contoso.Example/Resource` would match a literal filter
    /// defined as `contoso.example/resource`.
    /// 
    /// For a wildcard filter, the match is based on the wildcard pattern. The wildcard filter is
    /// converted to a regular expression where `*` matches any sequence of characters, and the
    /// regex is anchored to match the entire candidate string. For example, a wildcard filter of
    /// `Contoso*` would match candidates like `Contoso.Example/Resource` and `Contoso/Resource`.
    /// 
    /// # Arguments
    /// 
    /// - `candidate` - The fully qualified type name to check against the filter.
    /// 
    /// # Returns
    /// 
    /// `true` if the candidate matches the filter, `false` otherwise.
    /// 
    /// # Examples
    /// 
    /// The following snippet shows how candidates match against a literal filter:
    /// 
    /// ```rust
    /// use dsc_lib::types::{TypeNameFilter, FullyQualifiedTypeName};
    /// let filter = TypeNameFilter::Literal("Contoso.Example/Resource".parse().unwrap());
    /// // The candidate matches the filter exactly, including casing.
    /// assert!(filter.is_match(&"Contoso.Example/Resource".parse().unwrap()));
    /// // The candidate still matches the filter, even with different casing.
    /// assert!(filter.is_match(&"contoso.example/resource".parse().unwrap()));
    /// // The candidate does not match the filter if the text varies beyond casing.
    /// assert!(!filter.is_match(&"Example.Contoso/Resource".parse().unwrap()));
    /// ```
    /// 
    /// The following snippet shows how candidates match against a wildcard filter:
    /// 
    /// ```rust
    /// use dsc_lib::types::{TypeNameFilter, FullyQualifiedTypeName};
    /// let filter = TypeNameFilter::Wildcard("Contoso*".parse().unwrap());
    /// // The candidate matches the filter if it starts with "Contoso".
    /// assert!(filter.is_match(&"Contoso.Example/Resource".parse().unwrap()));
    /// assert!(filter.is_match(&"Contoso/Resource".parse().unwrap()));
    /// // The candidate does not match the filter if it does not start with "Contoso".
    /// assert!(!filter.is_match(&"Example.Contoso/Resource".parse().unwrap()));
    /// ```
    pub fn is_match(&self, candidate: &FullyQualifiedTypeName) -> bool {
        match self {
            TypeNameFilter::Literal(literal) => literal == candidate,
            TypeNameFilter::Wildcard(wildcard) => wildcard.is_match(candidate),
        }
     }
}

impl Default for TypeNameFilter {
    fn default() -> Self {
        TypeNameFilter::Wildcard(WildcardTypeName::default())
    }
}

impl Display for TypeNameFilter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TypeNameFilter::Literal(literal) => literal.fmt(f),
            TypeNameFilter::Wildcard(wildcard) => wildcard.fmt(f),
        }
    }
}

impl FromStr for TypeNameFilter {
    type Err = TypeNameFilterError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

impl TryFrom<String> for TypeNameFilter {
    type Error = TypeNameFilterError;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        Self::parse(&s)
    }
}

impl From<TypeNameFilter> for String {
    fn from(filter: TypeNameFilter) -> Self {
        filter.to_string()
    }
}

impl TryFrom<&str> for TypeNameFilter {
    type Error = TypeNameFilterError;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        Self::parse(s)
    }
}

impl From<WildcardTypeName> for TypeNameFilter {
    fn from(wildcard: WildcardTypeName) -> Self {
        TypeNameFilter::Wildcard(wildcard)
    }
}

impl From<FullyQualifiedTypeName> for TypeNameFilter {
    fn from(literal: FullyQualifiedTypeName) -> Self {
        TypeNameFilter::Literal(literal)
    }
}

impl TryFrom<TypeNameFilter> for FullyQualifiedTypeName {
    type Error = TypeNameFilterError;

    fn try_from(filter: TypeNameFilter) -> Result<Self, Self::Error> {
        match filter {
            TypeNameFilter::Literal(literal) => Ok(literal),
            TypeNameFilter::Wildcard(wildcard) => Err(
                TypeNameFilterError::InvalidConversionToFullyQualifiedTypeName{
                    type_name: wildcard
                }
            ),
        }
    }
}

impl TryFrom<TypeNameFilter> for WildcardTypeName {
    type Error = TypeNameFilterError;

    fn try_from(filter: TypeNameFilter) -> Result<Self, Self::Error> {
        match filter {
            TypeNameFilter::Wildcard(wildcard) => Ok(wildcard),
            TypeNameFilter::Literal(literal) => Err(
                TypeNameFilterError::InvalidConversionToWildcardTypeName{
                    type_name: literal
                }
            ),
        }
    }
}
