// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use std::borrow::Cow;

use jsonschema::{Keyword, ValidationError, paths::Location};
use rust_i18n::t;
use schemars::{Schema, JsonSchema, json_schema};
use serde::{Deserialize, Serialize};

use crate::vscode::keywords::VSCodeKeywordDefinition;

/// Defines the `deprecationMessage` keyword for the VS Code vocabulary.
/// 
/// This keyword defines a message to surface as a warning to users when they specify a deprecated
/// property in their data.
/// 
/// This keyword only has an affect when defined in a schema or subschema that also defines
/// the `deprecated` keyword as `true`. When you define the `deprecationMessage` keyword for
/// a deprecated schema or subschema, VS Code displays the provided message instead of the
/// default warning about deprecation.
#[derive(Serialize, Deserialize)]
pub struct DeprecationMessageKeyword(String);

impl VSCodeKeywordDefinition for DeprecationMessageKeyword {
    const KEYWORD_NAME: &str = "deprecationMessage";
    const KEYWORD_ID: &str = "https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/vscode/v0/keywords/deprecationMessage.json";

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
                t!("vscode.keywords.deprecation_message.factory_error_invalid_type"),
            ))
        }
    }
}

impl JsonSchema for DeprecationMessageKeyword {
    fn json_schema(_: &mut schemars::SchemaGenerator) -> Schema {
        json_schema!({
            "$schema": Self::META_SCHEMA,
            "$id": Self::KEYWORD_ID,
            "title": t!("vscode.keywords.deprecationMessage.title"),
            "description": t!("vscode.keywords.deprecationMessage.description"),
            "markdownDescription": t!("vscode.keywords.deprecationMessage.markdownDescription"),
            "type": "string",
        })
    }

    fn schema_name() -> Cow<'static, str> {
        Self::KEYWORD_ID.into()
    }
}

impl Keyword for DeprecationMessageKeyword {
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
