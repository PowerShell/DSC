// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use std::borrow::Cow;

use jsonschema::{Keyword, ValidationError, paths::Location};
use rust_i18n::t;
use schemars::{Schema, JsonSchema, json_schema};
use serde::{Deserialize, Serialize};

use crate::vscode::keywords::VSCodeKeywordDefinition;

/// Defines the `allowComments` keyword for the VS Code vocabulary.
/// 
/// This keyword indicates whether VS Code should allow comments in the JSON file, even when the
/// file extension isn't `.jsonc`.
///
/// By default, JSON comments in `.json` files cause parsing errors. If you define a JSON Schema
/// with `allowComments` set to `true`, VS Code doesn't raise validation errors for comments in
/// JSON for that schema.
#[derive(Default, Serialize, Deserialize)]
pub struct AllowCommentsKeyword(bool);

impl VSCodeKeywordDefinition for AllowCommentsKeyword {
    const KEYWORD_NAME: &str = "allowComments";
    const KEYWORD_ID: &str = "https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/vscode/v0/keywords/allowComments.json";
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
                t!("vscode.keywords.allow_comments.factory_error_invalid_type"),
            ))
        }
    }
}

impl JsonSchema for AllowCommentsKeyword {
    fn json_schema(_: &mut schemars::SchemaGenerator) -> Schema {
        json_schema!({
            "$schema": Self::META_SCHEMA,
            "$id": Self::KEYWORD_ID,
            "title": t!("vscode.keywords.allowComments.title"),
            "description": t!("vscode.keywords.allowComments.description"),
            "markdownDescription": t!("vscode.keywords.allowComments.markdownDescription"),
            "type": "boolean",
            "default": false
        })
    }

    fn schema_name() -> Cow<'static, str> {
        Self::KEYWORD_ID.into()
    }
}

impl Keyword for AllowCommentsKeyword {
    fn validate<'i>(
            &self,
            _: &'i serde_json::Value,
            _: &jsonschema::paths::LazyLocation,
        ) -> Result<(), ValidationError<'i>> {
        Ok(())
    }
    fn is_valid(&self, _: &serde_json::Value) -> bool {
        true
    }
}
