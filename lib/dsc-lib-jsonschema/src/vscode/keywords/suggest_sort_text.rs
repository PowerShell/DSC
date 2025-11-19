// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use std::borrow::Cow;

use jsonschema::{Keyword, ValidationError, paths::Location};
use rust_i18n::t;
use schemars::{Schema, JsonSchema, json_schema};
use serde::{Deserialize, Serialize};

use crate::vscode::keywords::VSCodeKeywordDefinition;

/// Defines the `suggestSortText` keyword for the VS Code vocabulary.
/// 
/// This keyword defines an alternate string to use when sorting a suggestion.
/// 
/// By default, suggestions are displayed in alphabetical order. You can define the
/// `suggestSortText` keyword to change how the suggestions are sorted. For example, in
/// the following schema, VS Code will suggest the `baz`, then `bar`, then `foo` properties:
/// 
/// ```json
/// {
///     "type": "object",
///     "properties": {
///         "foo": {
///             "suggestSortText": "c",
///         }
///         "bar": {
///             "suggestSortText": "b",
///         }
///         "baz": {
///             "suggestSortText": "a",
///         }
///     }
/// }
/// ```
#[derive(Serialize, Deserialize)]
pub struct SuggestSortTextKeyword(String);

impl VSCodeKeywordDefinition for SuggestSortTextKeyword {
    const KEYWORD_NAME: &str = "suggestSortText";
    const KEYWORD_ID: &str = "https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/vscode/v0/keywords/suggestSortText.json";

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
                t!("vscode.keywords.suggest_sort_text.factory_error_invalid_type"),
            ))
        }
    }
}

impl JsonSchema for SuggestSortTextKeyword {
    fn json_schema(_: &mut schemars::SchemaGenerator) -> Schema {
        json_schema!({
            "$schema": Self::META_SCHEMA,
            "$id": Self::KEYWORD_ID,
            "title": t!("vscode.keywords.suggestSortText.title"),
            "description": t!("vscode.keywords.suggestSortText.description"),
            "markdownDescription": t!("vscode.keywords.suggestSortText.markdownDescription"),
            "type": "string",
        })
    }

    fn schema_name() -> Cow<'static, str> {
        Self::KEYWORD_ID.into()
    }
}

impl Keyword for SuggestSortTextKeyword {
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
