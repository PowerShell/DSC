// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use rust_i18n::t;
use serde::Deserialize;
use serde_json::{Map, Value};
use std::{fmt, fmt::Write};
use tracing::warn;

use crate::error::SshdConfigError;
use crate::repeat_keyword::{KeywordInfo, ValueSeparator};

#[derive(Debug, Deserialize)]
struct MatchBlock {
    criteria: Map<String, Value>,
    #[serde(flatten)]
    contents: Map<String, Value>,
}

#[derive(Clone, Debug)]
pub struct SshdConfigValue<'a> {
    keyword_info: KeywordInfo,
    key: &'a str,
    value: &'a Value,
    use_quotes: bool,
}

impl<'a> SshdConfigValue<'a> {
    /// Create a new SSHD config value, returning an error if the value is empty/invalid
    pub fn try_new(key: &'a str, value: &'a Value, override_separator: Option<ValueSeparator>, use_quotes: bool) -> Result<Self, SshdConfigError> {
        // Structured keywords (objects with name/value) are allowed
        let is_structured_value = if let Value::Object(obj) = value {
            obj.contains_key("name") && obj.contains_key("value")
        } else {
            false
        };

        if matches!(value, Value::Null) || (matches!(value, Value::Object(_)) && !is_structured_value) {
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

        let mut keyword_info = KeywordInfo::from_keyword(key);

        // Allow separator override
        if let Some(separator) = override_separator {
            keyword_info.separator = separator;
        }

        Ok(Self {
            keyword_info,
            key,
            value,
            use_quotes,
        })
    }

    pub fn write_to_config(&self, config_text: &mut String) -> Result<(), SshdConfigError> {
        if self.keyword_info.is_repeatable {
            if let Value::Array(arr) = self.value {
                for item in arr {
                    let item = SshdConfigValue::try_new(self.key, item, Some(self.keyword_info.separator), self.use_quotes)?;
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
            Value::Object(obj) => {
                // Handle structured keywords (e.g., subsystem with name/value)
                if let (Some(name), Some(value)) = (obj.get("name"), obj.get("value")) {
                    if let Ok(name_formatted) = SshdConfigValue::try_new(self.key, name, Some(self.keyword_info.separator), false) {
                        write!(f, "{name_formatted} ")?;
                    }
                    if let Ok(value_formatted) = SshdConfigValue::try_new(self.key, value, Some(self.keyword_info.separator), false) {
                        write!(f, "{value_formatted}")?;
                    }
                    Ok(())
                } else {
                    // Shouldn't happen for valid structured values
                    Ok(())
                }
            },
            Value::Array(arr) => {
                if arr.is_empty() {
                    return Ok(());
                }

                let separator = match self.keyword_info.separator {
                    ValueSeparator::Comma => ",",
                    ValueSeparator::Space => " ",
                };

                let mut first = true;
                for item in arr {
                    if let Ok(sshd_config_value) = SshdConfigValue::try_new(self.key, item, Some(self.keyword_info.separator), self.use_quotes) {
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
                if self.use_quotes && s.contains(char::is_whitespace) {
                    write!(f, "\"{s}\"")
                } else {
                    write!(f, "{s}")
                }
            },
            Value::Null => Ok(()),
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
        let sshd_config_value = SshdConfigValue::try_new(key, value, Some(ValueSeparator::Comma), true)?;
        match_parts.push(format!("{key} {sshd_config_value}"));
    }

    // Write the Match line with the formatted criteria(s)
    result.push(match_parts.join(" "));

    // Format other keywords in the match block
    for (key, value) in &match_block.contents {
        let sshd_config_value = SshdConfigValue::try_new(key, value, None, true)?;
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

        let sshd_config_value = SshdConfigValue::try_new(key, value, None, true)?;
        sshd_config_value.write_to_config(&mut config_text)?;
    }

    if let Some(match_map) = match_map {
        if let Value::Array(arr) = match_map {
            for item in arr {
                let formatted = format_match_block(item)?;
                writeln!(&mut config_text, "Match {formatted}")?;
            }
        } else {
            let formatted = format_match_block(match_map)?;
            writeln!(&mut config_text, "Match {formatted}")?;
        }
    }

    Ok(config_text)
}
