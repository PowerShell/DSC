// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use std::borrow::Cow;

use jsonschema::{Keyword, ValidationError, paths::Location};
use rust_i18n::t;
use schemars::{Schema, JsonSchema, json_schema};
use serde::{Deserialize, Serialize};

use crate::vscode::keywords::VSCodeKeywordDefinition;

/// Defines the `doNotSuggest` keyword for the VS Code vocabulary.
/// 
/// This keyword indicates whether VS Code should avoid suggesting the property for IntelliSense.
///
/// By default, VS Code will show any defined property in the `properties` keyword as a
/// completion option with IntelliSense. You can define the `doNotSuggest` keyword in a
/// property subschema as `true` to indicate that VS Code should not show that property for
/// IntelliSense.
#[derive(Serialize, Deserialize)]
pub struct DoNotSuggestKeyword(bool);

impl VSCodeKeywordDefinition for DoNotSuggestKeyword {
    const KEYWORD_NAME: &str = "doNotSuggest";
    const KEYWORD_ID: &str = "https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/vscode/v0/keywords/doNotSuggest.json";

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
                t!("vscode.keywords.do_not_suggest.factory_error_invalid_type"),
            ))
        }
    }
}

impl JsonSchema for DoNotSuggestKeyword {
    fn json_schema(_: &mut schemars::SchemaGenerator) -> Schema {
        json_schema!({
            "$schema": Self::META_SCHEMA,
            "$id": Self::KEYWORD_ID,
            "title": t!("vscode.keywords.doNotSuggest.title"),
            "description": t!("vscode.keywords.doNotSuggest.description"),
            "markdownDescription": t!("vscode.keywords.doNotSuggest.markdownDescription"),
            "type": "boolean",
            "default": false
        })
    }

    fn schema_name() -> Cow<'static, str> {
        Self::KEYWORD_ID.into()
    }
}

impl Keyword for DoNotSuggestKeyword {
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
