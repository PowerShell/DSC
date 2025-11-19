// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use std::borrow::Cow;

use jsonschema::{Keyword, ValidationError, paths::Location};
use rust_i18n::t;
use schemars::{Schema, JsonSchema, json_schema};
use serde::{Deserialize, Serialize};

use crate::vscode::keywords::VSCodeKeywordDefinition;

/// Defines the `errorMessage` keyword for the VS Code vocabulary.
/// 
/// This keyword Defines a friendly error message to raise when a schema or subschema fails validation.
/// 
/// By default, VS Code surfaces a default error message for data that fails schema
/// validation, like specifying an invalid type. You can use the `errorMessage` keyword to
/// define a custom message to raise in the editor when the data fails validation for the
/// following cases:
/// 
/// - When the data is an invalid type as validated by the `type` keyword.
/// - When the subschema defined for the `not` keyword is valid.
/// - When the data is invalid for the defined values in the `enum` keyword.
/// - When the data is invalid for the defined value in the `const` keyword.
/// - When a string doesn't match the regular expression defined in the `pattern` keyword.
///   This message is overridden by the `patternErrorMessage` keyword if it's defined.
/// - When a string value doesn't match a required format.
/// - When the data is for an array that is validated by the `minContains` or `maxContains`
///   keywords and fails those validations.
/// - When the data includes a property that was defined in the `properties` keyword as
///   `false`, forbidding the property.
/// - When the data includes a property that was defined in the `patternProperties` keyword as
///   `false`, forbidding matching property names.
/// - When the data includes a property that wasn't defined in the `properties` or
///   `patternProperties` keyword and the schema defines `additionalProperties` as `false`.
/// - When the data includes a property that isn't evaluated by any keywords and the schema
///   defines `unevaluatedProperties` as `false`.
/// 
/// The value for the `errorMessage` keyword supercedes all default messages for the schema
/// or subschema where you define the keyword. You can provide per-validation failure
/// messages by defining the validating keywords in separate entries of the `allOf` keyword
/// and defining the `errorMessage` keyword for each entry.
#[derive(Serialize, Deserialize)]
pub struct ErrorMessageKeyword(String);

impl VSCodeKeywordDefinition for ErrorMessageKeyword {
    const KEYWORD_NAME: &str = "errorMessage";
    const KEYWORD_ID: &str = "https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/vscode/v0/keywords/errorMessage.json";

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
                t!("vscode.keywords.error_message.factory_error_invalid_type"),
            ))
        }
    }
}

impl JsonSchema for ErrorMessageKeyword {
    fn json_schema(_: &mut schemars::SchemaGenerator) -> Schema {
        json_schema!({
            "$schema": Self::META_SCHEMA,
            "$id": Self::KEYWORD_ID,
            "title": t!("vscode.keywords.errorMessage.title"),
            "description": t!("vscode.keywords.errorMessage.description"),
            "markdownDescription": t!("vscode.keywords.errorMessage.markdownDescription"),
            "type": "string",
        })
    }

    fn schema_name() -> Cow<'static, str> {
        Self::KEYWORD_ID.into()
    }
}

impl Keyword for ErrorMessageKeyword {
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
