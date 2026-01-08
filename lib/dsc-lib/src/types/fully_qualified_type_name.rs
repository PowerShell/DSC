// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use std::fmt::{Display, Formatter};
use std::ops::Deref;
use std::str::FromStr;
use std::sync::OnceLock;

use regex::Regex;
use rust_i18n::t;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::dscerror::DscError;
use crate::schemas::dsc_repo::DscRepoSchema;

/// Defines the fully qualified type name for a DSC resource or extension. The fully qualified name
/// uniquely identifies each resource and extension.
#[derive(
    Clone,
    Debug,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Serialize,
    Deserialize,
    JsonSchema,
    DscRepoSchema,
)]
#[serde(try_from = "String")]
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

/// This static lazily defines the validating regex for [`FullyQualifiedTypeName`]. It enables the
/// [`Regex`] instance to be constructed once, the first time it's used, and then reused on all
/// subsequent validation calls. It's kept private, since the API usage is to invoke the
/// [`FullyQualifiedTypeName::validate()`] method for direct validation or to leverage this static
/// from within the constructor for [`FullyQualifiedTypeName`].
static VALIDATING_REGEX: OnceLock<Regex> = OnceLock::new();

impl FullyQualifiedTypeName {
    /// Defines the regular expression for validating a string as a fully qualified type name.
    ///
    /// The string must begin with one or more alphanumeric characters and underscores that define
    /// the `owner` for the type. Following the `owner` segment, the string may include any number
    /// of `namespace` segments, which must be separated from the previous segment by a single
    /// period (`.`). Finally, the string must include a forward slash (`/`) followed by one or
    /// more alphanumeric characters and underscores to define the `name` segment.
    pub const VALIDATING_PATTERN: &str = r"^\w+(\.\w+)*\/\w+$";

    /// Returns the [`Regex`] for [`Self::VALIDATING_PATTERN`].
    ///
    /// This private method is used to initialize the [`VALIDATING_REGEX`] private static to reduce
    /// the number of times the regular expression is compiled from the pattern string.
    fn init_pattern() -> Regex {
        Regex::new(Self::VALIDATING_PATTERN).expect("pattern is valid")
    }

    /// Validates a given string as a fully qualified name.
    ///
    /// A string is valid if it matches the [`VALIDATING_PATTERN`]. If the string is invalid, DSC
    /// raises the [`DscError::InvalidTypeName`] error.
    ///
    /// [`VALIDATING_PATTERN`]: Self::VALIDATING_PATTERN
    pub fn validate(name: &str) -> Result<(), DscError> {
        let pattern = VALIDATING_REGEX.get_or_init(Self::init_pattern);
        match pattern.is_match(name) {
            true => Ok(()),
            false => Err(DscError::InvalidTypeName(
                name.to_string(),
                pattern.to_string(),
            )),
        }
    }

    /// Creates a new instance of [`FullyQualifiedName`] from a string if the input is valid for the
    /// [`VALIDATING_PATTERN`]. If the string is invalid, the method raises the
    /// [`DscError::InvalidTypeName`] error.
    pub fn new(name: &str) -> Result<Self, DscError> {
        Self::validate(name)?;
        Ok(Self(name.to_string()))
    }
}

impl Default for FullyQualifiedTypeName {
    fn default() -> Self {
        Self(String::new())
    }
}

impl FromStr for FullyQualifiedTypeName {
    type Err = DscError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s)
    }
}

impl TryFrom<String> for FullyQualifiedTypeName {
    type Error = DscError;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value.as_str())
    }
}

impl Display for FullyQualifiedTypeName {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AsRef<str> for FullyQualifiedTypeName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl Deref for FullyQualifiedTypeName {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
