// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use std::borrow::Cow;

use jsonschema::{Keyword, ValidationError, paths::Location};
use rust_i18n::t;
use schemars::{Schema, JsonSchema, json_schema};
use serde::{Deserialize, Serialize};

use crate::vscode::keywords::VSCodeKeywordDefinition;

/// Defines the `allowTrailingCommas` keyword for the VS Code vocabulary.
/// 
/// This keyword indicates whether VS Code should allow trailing commas in the JSON file.
/// 
/// By default, a comma after the last item in an array or last key-value pair in an object
/// causes a parsing error. If you define a JSON Schema with `allowTrailingCommas` set to
/// `true`, VS Code doesn't raise validation errors for commas after the last item in arrays
/// or last key-value pair in objects for that Schema.
#[derive(Default, Serialize, Deserialize)]
pub struct AllowTrailingCommasKeyword(bool);

impl VSCodeKeywordDefinition for AllowTrailingCommasKeyword {
    const KEYWORD_NAME: &str = "allowTrailingCommas";
    const KEYWORD_ID: &str = "https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/vscode/v0/keywords/allowTrailingCommas.json";

    fn keyword_factory<'a>(
        _parent: &'a serde_json::Map<String, serde_json::Value>,
        value: &'a serde_json::Value,
        path: Location,
    ) -> Result<Box<dyn Keyword>, ValidationError<'a>> {
        if let Some(v) = value.as_bool() {
            Ok(Box::new(Self(v)))
        } else {
            Err(ValidationError::custom(
                Location::new(),
                path,
                value,
                t!("vscode.keywords.allow_trailing_commas.factory_error_invalid_type"),
            ))
        }
    }
}

impl JsonSchema for AllowTrailingCommasKeyword {
    fn json_schema(_: &mut schemars::SchemaGenerator) -> Schema {
        json_schema!({
            "$schema": Self::META_SCHEMA,
            "$id": Self::KEYWORD_ID,
            "title": t!("vscode.keywords.allowTrailingCommas.title"),
            "description": t!("vscode.keywords.allowTrailingCommas.description"),
            "markdownDescription": t!("vscode.keywords.allowTrailingCommas.markdownDescription"),
            "type": "boolean",
            "default": false
        })
    }

    fn schema_name() -> Cow<'static, str> {
        Self::KEYWORD_ID.into()
    }
}

impl Keyword for AllowTrailingCommasKeyword {
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
