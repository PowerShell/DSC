// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use std::borrow::Cow;

use jsonschema::{Keyword, ValidationError, paths::Location};
use rust_i18n::t;
use schemars::{Schema, JsonSchema, json_schema};
use serde::{Deserialize, Serialize};

use crate::vscode::keywords::VSCodeKeywordDefinition;

/// Defines the `enumDescriptions` keyword for the VS Code vocabulary.
///
/// This keyword Defines per-value descriptions for schemas that use the `enum` keyword.
///
/// The builtin keywords for JSON Schema includes the `description` keyword, which you can use
/// to document a given schema or subschema. However, for schemas that use the `enum` keyword
/// to define an array of valid values, JSON Schema provides no keyword for documenting each
/// value.
///
/// With the `enumDescriptions` keyword from the VS Code vocabulary, you can document each
/// item in the `enum` keyword array. VS Code interprets each item in `enumDescriptions` as
/// documenting the item at the same index in the `enum` keyword.
///
/// This documentation is surfaced in VS Code on hover for an enum value and for IntelliSense
/// when completing an enum value.
///
/// If you want to use Markdown syntax for the annotation, specify the
/// [`markdownEnumDescriptions` keyword] instead.
/// 
/// [`markdownEnumDescriptions` keyword]: crate::vscode::keywords::MarkdownEnumDescriptionsKeyword
#[derive(Serialize, Deserialize)]
pub struct EnumDescriptionsKeyword(Vec<String>);

impl VSCodeKeywordDefinition for EnumDescriptionsKeyword {
    const KEYWORD_NAME: &str = "enumDescriptions";
    const KEYWORD_ID: &str = "https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/vscode/v0/keywords/enumDescriptions.json";

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
                        t!("vscode.keywords.enum_descriptions.factory_error_non_string_item"),
                        t!("vscode.keywords.enum_descriptions.factory_error_suffix"),
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
                    t!("vscode.keywords.enum_descriptions.factory_error_not_array"),
                    t!("vscode.keywords.enum_descriptions.factory_error_suffix"),
                ),
            ))
        }
    }
}

impl JsonSchema for EnumDescriptionsKeyword {
    fn json_schema(_: &mut schemars::SchemaGenerator) -> Schema {
        json_schema!({
            "$schema": Self::META_SCHEMA,
            "$id": Self::KEYWORD_ID,
            "title": t!("vscode.keywords.enumDescriptions.title"),
            "description": t!("vscode.keywords.enumDescriptions.description"),
            "markdownDescription": t!("vscode.keywords.enumDescriptions.markdownDescription"),
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

impl Keyword for EnumDescriptionsKeyword {
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
