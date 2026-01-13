// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use rust_i18n::t;
use serde::Deserialize;
use serde_json::{Map, Value};
use std::{fmt, fmt::Write};
use tracing::warn;

use crate::error::SshdConfigError;
use crate::metadata::{MULTI_ARG_KEYWORDS_COMMA_SEP, REPEATABLE_KEYWORDS};

#[derive(Debug, Deserialize)]
struct MatchBlock {
    criteria: Map<String, Value>,
    #[serde(flatten)]
    contents: Map<String, Value>,
}

#[derive(Clone, Debug)]
pub struct SshdConfigValue<'a> {
    is_repeatable: bool,
    key: &'a str,
    separator: ValueSeparator,
    value: &'a Value,

}

#[derive(Clone, Copy, Debug)]
pub enum ValueSeparator {
    Comma,
    Space,
}

impl<'a> SshdConfigValue<'a> {
    /// Create a new SSHD config value, returning an error if the value is empty/invalid
    pub fn try_new(key: &'a str, value: &'a Value, override_separator: Option<ValueSeparator>) -> Result<Self, SshdConfigError> {
        if matches!(value, Value::Null | Value::Object(_)) {
            return Err(SshdConfigError::ParserError(
                t!("formatter.invalidValue", key = key).to_string()
            ));
        }

        if let Value::Array(arr) = value {
            if arr.is_empty() {
                return Err(SshdConfigError::ParserError(
                    t!("formatter.invalidValue", key = key).to_string()
                ));
            }
        }

        let separator = match override_separator {
            Some(separator) => separator,
            None => {
                 if MULTI_ARG_KEYWORDS_COMMA_SEP.contains(&key) {
                    ValueSeparator::Comma
                } else {
                    ValueSeparator::Space
                }
            }
        };

        let is_repeatable = REPEATABLE_KEYWORDS.contains(&key);

        Ok(Self {
            is_repeatable,
            key,
            separator,
            value,
        })
    }

    pub fn write_to_config(&self, config_text: &mut String) -> Result<(), SshdConfigError> {
        if self.is_repeatable {
            if let Value::Array(arr) = self.value {
                for item in arr {
                    let item = SshdConfigValue::try_new(self.key, item, Some(self.separator))?;
                    writeln!(config_text, "{} {item}", self.key)?;
                }
            } else {
                writeln!(config_text, "{} {self}", self.key)?;
            }
        } else {
            writeln!(config_text, "{} {self}", self.key)?;
        }
        Ok(())
    }
}

impl fmt::Display for SshdConfigValue<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.value {
            Value::Array(arr) => {
                if arr.is_empty() {
                    return Ok(());
                }

                let separator = match self.separator {
                    ValueSeparator::Comma => ",",
                    ValueSeparator::Space => " ",
                };

                let mut first = true;
                for item in arr {
                    if let Ok(sshd_config_value) = SshdConfigValue::try_new(self.key, item, Some(self.separator)) {
                        let formatted = sshd_config_value.to_string();
                        if !formatted.is_empty() {
                            if !first {
                                write!(f, "{separator}")?;
                            }
                            write!(f, "{formatted}")?;
                            first = false;
                        }
                    } else {
                        warn!("{}", t!("formatter.invalidArrayItem", key = self.key, item = item).to_string());
                    }
                }
                Ok(())
            },
            Value::Bool(b) => write!(f, "{}", if *b { "yes" } else { "no" }),
            Value::Number(n) => write!(f, "{n}"),
            Value::String(s) => {
                if s.contains(char::is_whitespace) {
                    write!(f, "\"{s}\"")
                } else {
                    write!(f, "{s}")
                }
            },
            Value::Null | Value::Object(_) => Ok(()),
        }
    }
}

fn format_match_block(match_obj: &Value) -> Result<String, SshdConfigError> {
    let match_block = match serde_json::from_value::<MatchBlock>(match_obj.clone()) {
        Ok(result) => {
            result
        }
        Err(e) => {
            return Err(SshdConfigError::ParserError(t!("formatter.deserializeFailed", error = e).to_string()));
        }
    };

    if match_block.criteria.is_empty() {
        return Err(SshdConfigError::InvalidInput(t!("formatter.matchBlockMissingCriteria").to_string()));
    }

    let mut match_parts = vec![];
    let mut result = vec![];

    for (key, value) in &match_block.criteria {
        // all match criteria values are comma-separated
        let sshd_config_value = SshdConfigValue::try_new(key, value, Some(ValueSeparator::Comma))?;
        match_parts.push(format!("{key} {sshd_config_value}"));
    }

    // Write the Match line with the formatted criteria(s)
    result.push(match_parts.join(" "));

    // Format other keywords in the match block
    for (key, value) in &match_block.contents {
        let sshd_config_value = SshdConfigValue::try_new(key, value, None)?;
        result.push(format!("\t{key} {sshd_config_value}"));
    }

    Ok(result.join("\n"))
}

/// Write configuration map to config text string
///
/// # Errors
///
/// This function will return an error if formatting fails.
pub fn write_config_map_to_text(global_map: &Map<String, Value>) -> Result<String, SshdConfigError> {
    let match_map = global_map.get("match");
    let mut config_text = String::new();

    for (key, value) in global_map {
        let key_lower = key.to_lowercase();

        if key_lower == "match" {
            continue; // match blocks are handled after global settings
        }

        let sshd_config_value = SshdConfigValue::try_new(key, value, None)?;
        sshd_config_value.write_to_config(&mut config_text)?;
    }

    if let Some(match_map) = match_map {
        if let Value::Array(arr) = match_map {
            for item in arr {
                let formatted = format_match_block(item)?;
                writeln!(&mut config_text, "match {formatted}")?;
            }
        } else {
            let formatted = format_match_block(match_map)?;
            writeln!(&mut config_text, "match {formatted}")?;
        }
    }

    Ok(config_text)
}
