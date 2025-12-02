// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use std::borrow::Cow;

use jsonschema::{Keyword, ValidationError, paths::Location};
use rust_i18n::t;
use schemars::{Schema, JsonSchema, json_schema};
use serde::{Deserialize, Serialize};

use crate::vscode::keywords::VSCodeKeywordDefinition;

/// Defines the `patternErrorMessage` keyword for the VS Code vocabulary.
/// 
/// This keyword defines a friendly error message to raise when a schema or subschema fails
/// validation for the `pattern` keyword.
/// 
/// By default, when a value fails validation for the `pattern` keyword, VS Code raises an
/// error that informs the user that the value is invalid for the given regular expression,
/// which it displays in the message.
/// 
/// Reading and parsing regular expressions can be difficult even for experienced users. You
/// can define the `patternErrorMessage` keyword to provide better information to the user
/// about the expected pattern for the string value.
#[derive(Serialize, Deserialize)]
pub struct PatternErrorMessageKeyword(String);

impl VSCodeKeywordDefinition for PatternErrorMessageKeyword {
    const KEYWORD_NAME: &str = "patternErrorMessage";
    const KEYWORD_ID: &str = "https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/vscode/v0/keywords/patternErrorMessage.json";

    fn keyword_factory<'a>(
        _parent: &'a serde_json::Map<String, serde_json::Value>,
        value: &'a serde_json::Value,
        path: Location,
    ) -> Result<Box<dyn Keyword>, ValidationError<'a>> {
        if let Some(v) = value.as_str() {
            Ok(Box::new(Self(v.to_string())))
        } else {
            Err(ValidationError::custom(
                Location::new(),
                path,
                value,
                t!("vscode.keywords.pattern_error_message.factory_error_invalid_type"),
            ))
        }
    }
}

impl JsonSchema for PatternErrorMessageKeyword {
    fn json_schema(_: &mut schemars::SchemaGenerator) -> Schema {
        json_schema!({
            "$schema": Self::META_SCHEMA,
            "$id": Self::KEYWORD_ID,
            "title": t!("vscode.keywords.patternErrorMessage.title"),
            "description": t!("vscode.keywords.patternErrorMessage.description"),
            "markdownDescription": t!("vscode.keywords.patternErrorMessage.markdownDescription"),
            "type": "string",
        })
    }

    fn schema_name() -> Cow<'static, str> {
        Self::KEYWORD_ID.into()
    }
}

impl Keyword for PatternErrorMessageKeyword {
    fn validate<'i>(
            &self,
            _: &'i serde_json::Value,
            _: &jsonschema::paths::LazyLocation,
        ) -> Result<(), jsonschema::ValidationError<'i>> {
        Ok(())
    }
    fn is_valid(&self, _: &serde_json::Value) -> bool {
        true
    }
}
