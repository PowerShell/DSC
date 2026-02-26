// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use rust_i18n::t;
use serde_json::{Map, Value};
use std::fmt;

use crate::error::SshdConfigError;

/// Canonical properties that have special meaning in the DSC resource.
/// These properties are prefixed with underscore and are not SSH configuration keywords.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CanonicalProperty {
    /// Boolean flag indicating if an entry should exist or be removed
    Exist,
    /// Boolean flag to include default values in output
    IncludeDefaults,
    /// Contains SSH default values inherited from system
    InheritedDefaults,
    /// Metadata object containing filepath and other configuration info
    Metadata,
    /// Boolean flag indicating if non-specified entries should be removed
    Purge,
}

impl CanonicalProperty {
    /// Returns the string key for this canonical property.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Exist => "_exist",
            Self::IncludeDefaults => "_includeDefaults",
            Self::InheritedDefaults => "_inheritedDefaults",
            Self::Metadata => "_metadata",
            Self::Purge => "_purge",
        }
    }

    /// Parse a string into a canonical property.
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "_exist" => Some(Self::Exist),
            "_includeDefaults" => Some(Self::IncludeDefaults),
            "_inheritedDefaults" => Some(Self::InheritedDefaults),
            "_metadata" => Some(Self::Metadata),
            "_purge" => Some(Self::Purge),
            _ => None,
        }
    }

    /// Returns all canonical properties as a slice.
    pub const fn all() -> &'static [CanonicalProperty] {
        &[
            Self::Exist,
            Self::IncludeDefaults,
            Self::InheritedDefaults,
            Self::Metadata,
            Self::Purge,
        ]
    }
}

impl fmt::Display for CanonicalProperty {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Helper struct for working with canonical properties.
pub struct CanonicalProperties;

impl CanonicalProperties {
    /// Check if a string key is a canonical property.
    pub fn is_canonical(key: &str) -> bool {
        CanonicalProperty::from_str(key).is_some()
    }

    /// Remove all canonical properties from a map.
    pub fn remove_all(map: &mut Map<String, Value>) {
        for prop in CanonicalProperty::all() {
            map.remove(prop.as_str());
        }
    }

    /// Extract and validate a boolean canonical property from a map.
    ///
    /// # Arguments
    ///
    /// * `map` - The map to extract the value from
    /// * `prop` - The canonical property to extract
    /// * `default` - The default value to return if the property is not found
    ///
    /// # Errors
    ///
    /// Returns an error if the value exists but is not a boolean.
    pub fn extract_bool(
        map: &mut Map<String, Value>,
        prop: CanonicalProperty,
        default: bool,
    ) -> Result<bool, SshdConfigError> {
        if let Some(value) = map.remove(prop.as_str()) {
            if let Value::Bool(b) = value {
                Ok(b)
            } else {
                Err(SshdConfigError::InvalidInput(
                    t!("canonical_properties.inputMustBeBoolean", input = prop.as_str()).to_string()
                ))
            }
        } else {
            Ok(default)
        }
    }
}
