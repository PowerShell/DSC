// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use std::borrow::Cow;

use jsonschema::{Keyword, ValidationError, paths::Location};
use rust_i18n::t;
use schemars::{Schema, JsonSchema, json_schema};
use serde::{Deserialize, Serialize};

use crate::vscode::keywords::VSCodeKeywordDefinition;

/// Defines the `enumDetails` keyword for the VS Code vocabulary.
///
/// This keyword defines additional information for IntelliSense when completing a proposed enum
/// value, shown before the description.
///
/// By default, when VS Code suggests a completion for an item defined in the `enum` keyword,
/// VS Code displays hover text with a description. If the schema defined the `description`,
/// `enumDescriptions`, or `markdownEnumDescriptions` keywords, VS Code displays that text.
/// The `markdownEnumDescriptions` keyword overrides the `enumDescriptions` keyword, which
/// overrides the `description` keyword.
///
/// When you define the `enumDetails` keyword, VS Code displays the string for that enum
/// value as monospace code-formatted text. The keyword expects an array of strings. VS Code
/// correlates the items in the `enumDetails` keyword to the items in the `enum` keyword by
/// their index. The first item in `enumDetails` maps to the first item in `enum` and so on.
#[derive(Serialize, Deserialize)]
pub struct EnumDetailsKeyword(Vec<String>);

impl VSCodeKeywordDefinition for EnumDetailsKeyword {
    const KEYWORD_NAME: &str = "enumDetails";
    const KEYWORD_ID: &str = "https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/vscode/v0/keywords/enumDetails.json";

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
                        t!("vscode.keywords.enum_details.factory_error_non_string_item"),
                        t!("vscode.keywords.enum_details.factory_error_suffix"),
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
                    t!("vscode.keywords.enum_details.factory_error_not_array"),
                    t!("vscode.keywords.enum_details.factory_error_suffix"),
                ),
            ))
        }
    }
}

impl JsonSchema for EnumDetailsKeyword {
    fn json_schema(_: &mut schemars::SchemaGenerator) -> Schema {
        json_schema!({
            "$schema": Self::META_SCHEMA,
            "$id": Self::KEYWORD_ID,
            "title": t!("vscode.keywords.enumDetails.title"),
            "description": t!("vscode.keywords.enumDetails.description"),
            "markdownDescription": t!("vscode.keywords.enumDetails.markdownDescription"),
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

impl Keyword for EnumDetailsKeyword {
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
