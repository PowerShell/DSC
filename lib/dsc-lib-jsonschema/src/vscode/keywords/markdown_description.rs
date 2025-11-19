// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use std::borrow::Cow;

use jsonschema::{Keyword, ValidationError, paths::Location};
use rust_i18n::t;
use schemars::{Schema, JsonSchema, json_schema};
use serde::{Deserialize, Serialize};

use crate::vscode::keywords::VSCodeKeywordDefinition;

/// Defines the `markdownDescription` keyword for the VS Code vocabulary.
///
/// This keyword Defines documentation for the schema or subschema displayed as hover text in VS Code.
///
/// By default, VS Code displays the text defined in the `description` keyword in the hover
/// text for properties and values. VS Code interprets the `description` keyword literally,
/// without converting any apparent markup.
///
/// You can define the `markdownDescription` keyword to provide descriptive text as markdown,
/// including links and code blocks. When a schema or subschema defines the
/// `markdownDescription` keyword, that value supercedes any defined text in the `description`
/// keyword.
///
/// You can also use the `markdownEnumDescriptions` keyword to document the values defined
/// for the `enum` keyword.
///
/// For more information, see [Use rich formatting in hovers][01].
///
/// [01]: https://code.visualstudio.com/Docs/languages/json#_use-rich-formatting-in-hovers
#[derive(Serialize, Deserialize)]
pub struct MarkdownDescriptionKeyword(pub String);

impl VSCodeKeywordDefinition for MarkdownDescriptionKeyword {
    const KEYWORD_NAME: &str = "markdownDescription";
    const KEYWORD_ID: &str = "https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/vscode/v0/keywords/markdownDescription.json";

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
                t!("vscode.keywords.markdown_description.factory_error_invalid_type"),
            ))
        }
    }
}

impl JsonSchema for MarkdownDescriptionKeyword {
    fn json_schema(_: &mut schemars::SchemaGenerator) -> Schema {
        json_schema!({
            "$schema": Self::META_SCHEMA,
            "$id": Self::KEYWORD_ID,
            "title": t!("vscode.keywords.markdownDescription.title"),
            "description": t!("vscode.keywords.markdownDescription.description"),
            "markdownDescription": t!("vscode.keywords.markdownDescription.markdownDescription"),
            "type": "string",
        })
    }

    fn schema_name() -> Cow<'static, str> {
        Self::KEYWORD_ID.into()
    }
}

impl Keyword for MarkdownDescriptionKeyword {
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
