// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use std::borrow::Cow;

use jsonschema::{Keyword, ValidationError, paths::Location};
use rust_i18n::t;
use schemars::{Schema, JsonSchema, json_schema};
use serde::{Deserialize, Serialize};

use crate::vscode::keywords::VSCodeKeywordDefinition;

/// Defines the `markdownEnumDescriptions` keyword for the VS Code vocabulary.
/// 
/// This keyword defines documentation for enum values displayed as hover text in VS Code.
/// 
/// By default, when a user hovers on or selects completion for a value that is validated by
/// the `enum` keyword, VS Code displays the text from the `description` or
/// `markdownDescription` keywords for the schema or subschema. You can use the
/// `markdownEnumDescriptions` keyword to define documentation for each enum value.
/// 
/// When a schema or subschema defines the `markdownEnumDescriptions` keyword, that value
/// supercedes any defined text in the `description`, `markdownDescription`, or
/// `enumDescriptions` keywords.
/// 
/// The keyword expects an array of strings. VS Code correlates the items in the
/// `markdownEnumDescriptions` keyword to the items in the `enum` keyword by their index. The
/// first item in `markdownEnumDescriptions` maps to the first item in `enum` and so on.
#[derive(Serialize, Deserialize)]
pub struct MarkdownEnumDescriptionsKeyword(Vec<String>);

impl VSCodeKeywordDefinition for MarkdownEnumDescriptionsKeyword {
    const KEYWORD_NAME: &str = "markdownEnumDescriptions";
    const KEYWORD_ID: &str = "https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/vscode/v0/keywords/markdownEnumDescriptions.json";

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
                        t!("vscode.keywords.markdown_enum_descriptions.factory_error_non_string_item"),
                        t!("vscode.keywords.markdown_enum_descriptions.factory_error_suffix"),
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
                    t!("vscode.keywords.markdown_enum_descriptions.factory_error_not_array"),
                    t!("vscode.keywords.markdown_enum_descriptions.factory_error_suffix"),
                ),
            ))
        }
    }
}

impl JsonSchema for MarkdownEnumDescriptionsKeyword {
    fn json_schema(_: &mut schemars::SchemaGenerator) -> Schema {
        json_schema!({
            "$schema": Self::META_SCHEMA,
            "$id": Self::KEYWORD_ID,
            "title": t!("vscode.keywords.markdownEnumDescriptions.title"),
            "description": t!("vscode.keywords.markdownEnumDescriptions.description"),
            "markdownDescription": t!("vscode.keywords.markdownEnumDescriptions.markdownDescription"),
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

impl Keyword for MarkdownEnumDescriptionsKeyword {
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