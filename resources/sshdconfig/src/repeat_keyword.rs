// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use rust_i18n::t;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::error::SshdConfigError;
use crate::inputs::Metadata;
// the multi-arg comma-separated and space-separated lists are mutually exclusive, but the repeatable list can overlap with either of them.
// the multi-arg lists are maintained for formatting arrays into the correct format when writing back to the config file.

// keywords that can have multiple comma-separated arguments per line and should be represented as arrays.
pub const MULTI_ARG_KEYWORDS_COMMA_SEP: [&str; 11] = [
    "authenticationmethods",
    "casignaturealgorithms",
    "ciphers",
    "hostbasedacceptedalgorithms",
    "hostkeyalgorithms",
    "kexalgorithms",
    "macs",
    "permituserenvironment",
    "persourcepenaltyexemptlist",
    "pubkeyacceptedalgorithms",
    "rekeylimit" // first arg is bytes, second arg (optional) is amount of time
];

// keywords that can have multiple space-separated arguments per line and should be represented as arrays.
pub const MULTI_ARG_KEYWORDS_SPACE_SEP: [&str; 11] = [
    "acceptenv",
    "allowgroups",
    "allowusers",
    "authorizedkeysfile",
    "channeltimeout",
    "denygroups",
    "denyusers",
    "ipqos",
    "permitlisten",
    "permitopen",
    "persourcepenalties",
];

// keywords that can be repeated over multiple lines and should be represented as arrays.
pub const REPEATABLE_KEYWORDS: [&str; 12] = [
    "acceptenv",
    "allowgroups",
    "allowusers",
    "denygroups",
    "denyusers",
    "hostkey",
    "include",
    "listenaddress",
    "match",
    "port",
    "setenv",
    "subsystem"
];

// keywords that require structured name-value format (e.g., subsystem has a name and a command value).
pub const STRUCTURED_KEYWORDS: [&str; 1] = [
    "subsystem"
];

#[derive(Clone, Copy, Debug)]
pub enum ValueSeparator {
    Comma,
    Space,
}

/// A name-value entry for structured keywords like subsystem.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NameValueEntry {
    /// The entry name (e.g., subsystem name like "sftp")
    pub name: String,
    /// The entry value (e.g., subsystem command path and args)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
}

/// Input for name-value keyword single-entry operations (e.g., subsystem).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct RepeatInput {
    /// Whether the entry should exist (true) or be removed (false)
    #[serde(rename = "_exist", default)]
    pub exist: bool,
    /// Metadata for the operation
    #[serde(rename = "_metadata", skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Metadata>,
    /// The keyword and its entry (e.g., "subsystem": {"name": "sftp", "value": "/usr/bin/sftp"})
    #[serde(flatten)]
    pub additional_properties: Map<String, Value>,
}

/// Input for name-value keyword list operations with purge support (e.g., subsystem list).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct RepeatListInput {
    /// Whether to remove entries not in the input list
    #[serde(rename = "_purge", default)]
    pub purge: bool,
    /// Metadata for the operation
    #[serde(rename = "_metadata", skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Metadata>,
    /// The keyword and its array of entries (e.g., "subsystem": [{"name": "sftp", "value": "..."}])
    #[serde(flatten)]
    pub additional_properties: Map<String, Value>,
}

#[derive(Clone, Debug)]
pub struct KeywordInfo {
    pub name: String,
    pub is_repeatable: bool,
    pub is_structured: bool,
    pub separator: ValueSeparator,
}

impl KeywordInfo {
    /// Create a new `KeywordInfo` from a keyword string.
    pub fn from_keyword(keyword: &str) -> Self {
        let lowercase_key = keyword.to_lowercase();
        let is_repeatable = REPEATABLE_KEYWORDS.contains(&lowercase_key.as_str());
        let is_structured = STRUCTURED_KEYWORDS.contains(&lowercase_key.as_str());
        let separator = if MULTI_ARG_KEYWORDS_COMMA_SEP.contains(&lowercase_key.as_str()) {
            ValueSeparator::Comma
        } else {
            ValueSeparator::Space
        };

        Self {
            name: lowercase_key,
            is_repeatable,
            is_structured,
            separator,
        }
    }

    /// Check if this keyword allows operator syntax (+, -, ^).
    pub fn allows_operator(&self) -> bool {
        !self.is_structured
    }

    /// Check if this keyword requires structured name-value format.
    pub fn requires_structured_format(&self) -> bool {
        self.is_structured
    }

    /// Check if this keyword can have multiple arguments.
    pub fn is_multi_arg(&self) -> bool {
        self.is_repeatable ||
        MULTI_ARG_KEYWORDS_COMMA_SEP.contains(&self.name.as_str()) ||
        MULTI_ARG_KEYWORDS_SPACE_SEP.contains(&self.name.as_str())
    }
}

/// Extract and validate a single keyword from `additional_properties`.
///
/// This function ensures that exactly one keyword is present in the input map.
///
/// # Parameters
///
/// * `additional_properties` - A map containing keyword-value pairs from the input
///
/// # Returns
///
/// Returns `Ok((String, Value))` containing the keyword name and its value if exactly one keyword is found.
///
/// # Errors
///
/// * Returns `SshdConfigError::InvalidInput` if no keywords are found in the input or if more than one keyword is found in the input
pub fn extract_single_keyword(additional_properties: Map<String, Value>) -> Result<(String, Value), SshdConfigError> {
    let mut keywords: Vec<(String, Value)> = additional_properties.into_iter().collect();

    if keywords.is_empty() {
        return Err(SshdConfigError::InvalidInput(t!("repeat_keyword.noKeywordFoundInInput").to_string()));
    }

    if keywords.len() > 1 {
        return Err(SshdConfigError::InvalidInput(
            t!("repeat_keyword.multipleKeywordsNotAllowed", count = keywords.len()).to_string()
        ));
    }

    Ok(keywords.remove(0))
}

/// Parse and validate an array of name-value entries.
///
/// This function converts JSON values into `NameValueEntry` structs and validates that
/// each entry has both a non-empty name and a non-empty value field.
///
/// # Parameters
///
/// * `entries_array` - A slice of JSON values representing name-value entries
///
/// # Returns
///
/// Returns `Ok(Vec<NameValueEntry>)` containing the parsed and validated entries.
///
/// # Errors
///
/// * Returns `SshdConfigError::InvalidInput` if an entry cannot be parsed from JSON
/// * Returns `SshdConfigError::InvalidInput` if an entry has an empty name field
/// * Returns `SshdConfigError::InvalidInput` if an entry has a missing or empty value field
pub fn parse_and_validate_entries(entries_array: &[Value]) -> Result<Vec<NameValueEntry>, SshdConfigError> {
    let mut entries: Vec<NameValueEntry> = Vec::new();

    for entry_value in entries_array {
        let entry: NameValueEntry = serde_json::from_value(entry_value.clone())
            .map_err(|e| SshdConfigError::InvalidInput(t!("repeat_keyword.failedToParse", input = e.to_string()).to_string()))?;

        // Validate required name field
        if entry.name.is_empty() {
            return Err(SshdConfigError::InvalidInput(
                t!("repeat_keyword.entryNameRequired").to_string()
            ));
        }

        // Validate value field is present
        if entry.value.is_none() || entry.value.as_ref().unwrap().is_empty() {
            return Err(SshdConfigError::InvalidInput(
                t!("repeat_keyword.entryValueRequired", name = entry.name).to_string()
            ));
        }

        entries.push(entry);
    }

    Ok(entries)
}

/// Find the index of a name-value entry in a keyword array by matching the name field (case-sensitive).
///
/// This function searches for an entry with a matching name field, and optionally a matching value field.
///
/// # Parameters
///
/// * `keyword_array` - A slice of JSON values representing keyword entries
/// * `entry_name` - The name to search for (case-sensitive comparison)
/// * `match_value` - Optional value to match; if provided, both name and value must match
///
/// # Returns
///
/// Returns `Some(usize)` containing the index of the matching entry if found,
/// or `None` if no matching entry exists.
pub fn find_name_value_entry_index(keyword_array: &[Value], entry_name: &str, match_value: Option<&str>) -> Option<usize> {
    keyword_array.iter().position(|item| {
        if let Value::Object(obj) = item {
            if let Some(Value::String(name)) = obj.get("name") {
                if name != entry_name {
                    return false;
                }

                // If match_value is specified, also check the value field
                if let Some(expected_value) = match_value {
                    if let Some(Value::String(actual_value)) = obj.get("value") {
                        return actual_value == expected_value;
                    }
                    return false;
                }

                return true;
            }
        }
        false
    })
}

/// Add or update a name-value entry in the config map.
///
/// This function either updates an existing entry with the same name, or appends a new entry
/// to the keyword array. If the keyword doesn't exist in the config, it creates a new array.
///
/// # Parameters
///
/// * `config` - A mutable reference to the configuration map
/// * `keyword` - The keyword name (e.g., "subsystem")
/// * `entry` - The name-value entry to add or update
///
/// # Returns
///
/// Returns `Ok(())` if the entry is successfully added or updated.
///
/// # Errors
///
/// * Returns `SshdConfigError::InvalidInput` if the entry has a `None` value
/// * Returns `SshdConfigError` if JSON serialization of the entry fails
pub fn add_or_update_entry(config: &mut Map<String, Value>, keyword: &str, entry: &NameValueEntry) -> Result<(), SshdConfigError> {
    if entry.value.is_none() {
        return Err(SshdConfigError::InvalidInput(
            t!("repeat_keyword.nameValueEntryRequiresValue").to_string()
        ));
    }

    let entry_value = serde_json::to_value(entry)?;

    if let Some(existing) = config.get_mut(keyword) {
        if let Value::Array(arr) = existing {
            if let Some(index) = find_name_value_entry_index(arr, &entry.name, None) {
                // Entry exists, update it
                arr[index] = entry_value;
            } else {
                // Entry doesn't exist, append it
                arr.push(entry_value);
            }
        } else {
            *existing = Value::Array(vec![entry_value]);
        }
    } else {
        let new_array = Value::Array(vec![entry_value]);
        config.insert(keyword.to_string(), new_array);
    }
    Ok(())
}

/// Remove a keyword entry based on the keyword's name field.
///
/// This function searches for and removes an entry with the specified name from the
/// keyword's array in the configuration map. If the keyword or entry doesn't exist,
/// the function has no effect.
///
/// # Parameters
///
/// * `config` - A mutable reference to the configuration map
/// * `keyword` - The keyword name (e.g., "subsystem")
/// * `entry_name` - The name of the entry to remove
pub fn remove_entry(config: &mut Map<String, Value>, keyword: &str, entry_name: &str) {
    if let Some(Value::Array(arr)) = config.get_mut(keyword) {
        if let Some(index) = find_name_value_entry_index(arr, entry_name, None) {
            arr.remove(index);
        }
    }
}
