// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use std::borrow::Cow;

use jsonschema::{Keyword, ValidationError, paths::Location};
use rust_i18n::t;
use schemars::{Schema, JsonSchema, json_schema};
use serde::{Deserialize, Serialize};

use crate::vscode::keywords::VSCodeKeywordDefinition;

/// Defines the `enumSortTexts` keyword for the VS Code vocabulary.
///
/// This keyword defines a alternate strings to use when sorting a suggestion for enum values.
///
/// By default, suggestions are sorted alphabetically, not in the order that you define
/// items in the `enum` keyword array. You can use the `enumSortText` keyword to override
/// the order the values are displayed, providing a different string for each value.
///
/// The keyword expects an array of strings. VS Code correlates the items in the
/// `enumSortText` keyword to the items in the `enum` keyword by their index. The first item
/// in `enumSortText` maps to the first item in `enum` and so on.
///
/// For example, in the following schema, VS Code will suggest the `baz`, then `bar`, then
/// `foo` values:
///
/// ```json
/// {
///     "type": "string",
///     "enum": ["foo", "bar", "baz"],
///     "enumSortText": ["c", "b", "a"]
/// }
/// ```
#[derive(Serialize, Deserialize)]
pub struct EnumSortTextsKeyword(Vec<String>);

impl VSCodeKeywordDefinition for EnumSortTextsKeyword {
    const KEYWORD_NAME: &str = "enumSortTexts";
    const KEYWORD_ID: &str = "https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/vscode/v0/keywords/enumSortTexts.json";

    fn keyword_factory<'a>(
        _parent: &'a serde_json::Map<String, serde_json::Value>,
        value: &'a serde_json::Value,
        path: Location,
    ) -> Result<Box<dyn Keyword>, ValidationError<'a>> {
        if let Some(v) = value.as_array() {
            if v.iter().all(|item| item.as_str().is_some()) {
                Ok(Box::new(Self(
                    v.iter().map(|item| item.as_str().unwrap().to_string()).collect()
                )))
            } else {
                Err(ValidationError::custom(
                    Location::new(),
                    path,
                    value,
                    format!(
                        "{} {}",
                        t!("vscode.keywords.enum_sort_texts.factory_error_non_string_item"),
                        t!("vscode.keywords.enum_sort_texts.factory_error_suffix"),
                    ),
                ))
            }
        } else {
            Err(ValidationError::custom(
                Location::new(),
                path,
                value,
                format!(
                    "{} {}",
                    t!("vscode.keywords.enum_sort_texts.factory_error_not_array"),
                    t!("vscode.keywords.enum_sort_texts.factory_error_suffix"),
                ),
            ))
        }
    }
}

impl JsonSchema for EnumSortTextsKeyword {
    fn json_schema(_: &mut schemars::SchemaGenerator) -> Schema {
        json_schema!({
            "$schema": Self::META_SCHEMA,
            "$id": Self::KEYWORD_ID,
            "title": t!("vscode.keywords.enumSortTexts.title"),
            "description": t!("vscode.keywords.enumSortTexts.description"),
            "markdownDescription": t!("vscode.keywords.enumSortTexts.markdownDescription"),
            "type": "array",
            "items": {
                "type": "string"
            }
        })
    }

    fn schema_name() -> Cow<'static, str> {
        Self::KEYWORD_ID.into()
    }
}

impl Keyword for EnumSortTextsKeyword {
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
