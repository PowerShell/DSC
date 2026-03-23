// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use std::borrow::Cow;

use jsonschema::{Keyword, ValidationError, paths::Location};
use rust_i18n::t;
use schemars::{Schema, JsonSchema, json_schema};
use serde::{Deserialize, Serialize};

use crate::vscode::keywords::VSCodeKeywordDefinition;

/// Defines the `completionDetail` keyword for the VS Code vocabulary.
/// 
/// This keyword Defines additional information for IntelliSense when completing a proposed item,
/// replacing the `title` keyword as code-formatted text.
///
/// By default, when a user completes a value for a schema or subschema, VS Code displays
/// additional information in hover text. If the schema defines the `title` keyword, the
/// hover text includes the title string as the first line of the hover text.
///
/// If you define the `completionDetail` keyword, VS Code displays the string as monospace
/// code-formatted text instead of the `title` keyword's value.
///
/// If the schema defines the `description` or `markdownDescription` keywords, that text is
/// displayed in the hover text after the value from the `completionDetail` or `title`
/// keyword.
#[derive(Serialize, Deserialize)]
pub struct CompletionDetailKeyword(String);

impl VSCodeKeywordDefinition for CompletionDetailKeyword {
    const KEYWORD_NAME: &str = "completionDetail";
    const KEYWORD_ID: &str = "https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/vscode/v0/keywords/completionDetail.json";

    fn keyword_factory<'a>(
        _parent: &'a serde_json::Map<String, serde_json::Value>,
        value: &'a serde_json::Value,
        path: jsonschema::paths::Location,
    ) -> Result<Box<dyn Keyword>, jsonschema::ValidationError<'a>> {
        if let Some(v) = value.as_str() {
            Ok(Box::new(Self(v.to_string())))
        } else {
            Err(ValidationError::custom(
                Location::new(),
                path,
                value,
                t!("vscode.keywords.completion_detail.factory_error_invalid_type"),
            ))
        }
    }
}

impl JsonSchema for CompletionDetailKeyword {
    fn json_schema(_: &mut schemars::SchemaGenerator) -> Schema {
        json_schema!({
            "$schema": Self::META_SCHEMA,
            "$id": Self::KEYWORD_ID,
            "title": t!("vscode.keywords.completionDetail.title"),
            "description": t!("vscode.keywords.completionDetail.description"),
            "markdownDescription": t!("vscode.keywords.completionDetail.markdownDescription"),
            "type": "string",
        })
    }

    fn schema_name() -> Cow<'static, str> {
        Self::KEYWORD_ID.into()
    }
}

impl Keyword for CompletionDetailKeyword {
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
