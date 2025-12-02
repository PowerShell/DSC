// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use std::borrow::Cow;

use jsonschema::{Keyword, ValidationError, paths::Location};
use rust_i18n::t;
use schemars::{json_schema, JsonSchema};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::vscode::keywords::VSCodeKeywordDefinition;

/// Defines the structure of a snippet in the `defaultSnippets` keyword for the VS Code vocabulary.
///
/// Every snippet must define either the `body` or `bodyText` property, which VS Code uses
/// to insert the snippet into the data file. If you specify both `body` and `bodyText`, the
/// value for `body` supercedes the value for `bodyText`.
///
/// The `description`, and `markdownDescription` properties provide documentation for the
/// snippet and are displayed in the hover text when a user selects the snippet. If you
/// specify both `description` and `markdownDescription`, the text for
/// `markdownDescription` supercedes the text for `description`.
///
/// The `label` property defines a short name for the snippet. If the snippet doesn't define
/// the `label` property, VS Code shows a stringified representation of the snippet instead.
///
/// Snippets are presented to the user in alphabetical order by the value of their `label`
/// property (or the stringified representation of the snippet if it has no label).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Snippet {
    /// Defines a short name for the snippet instead of using the stringified representation
    /// of the snippet's value. The `label` property also affects the order that VS Code
    /// presents the snippets. VS Code sorts the snippets for completion alphabetically by
    /// label.
    pub label: Option<String>,
    /// Defines plain text documentation for the snippet displayed in the completion dialog.
    ///
    /// When the snippet doesn't define the `description` or `markdownDescription` property,
    /// the snippet provides no additional context to the user aside from the label until
    /// they select the snippet for completion. Use the `description` property to provide
    /// information to the user about the snippet. If you need to provide rich formatting,
    /// like links or text formatting, use the `markdownDescription` property.
    ///
    /// If you define both the `description` and `markdownDescription` property for a
    /// snippet, the `markdownDescription` text overrides the `description` text.
    pub description: Option<String>,
    /// Defines formatted documentation for the snippet displayed in the completion dialog.
    ///
    /// When the snippet doesn't define the `description` or `markdownDescription` property,
    /// the snippet provides no additional context to the user aside from the label until
    /// they select the snippet for completion. Use the `description` property to provide
    /// information to the user about the snippet. If you need to provide rich formatting,
    /// like links or text formatting, use the `markdownDescription` property.
    ///
    /// If you define both the `description` and `markdownDescription` property for a
    /// snippet, the `markdownDescription` text overrides the `description` text.
    pub markdown_description: Option<String>,
    /// Defines the data to insert for the snippet. The data can be any type. When the user
    /// selects the snippet, VS Code inserts the data at the cursor. In string literals for
    /// the `body` you can use [snippet syntax][01] to define tabstops, placeholders, and
    /// variables.
    ///
    /// Alternatively, you can define the `bodyText` property for the snippet, which
    /// specifies the text to insert for the snippet as a string.
    ///
    /// If you define both the `bodyText` and `body` properties for a snippet, the `body`
    /// definition overrides the `bodyText` property.
    ///
    /// [01]: https://code.visualstudio.com/docs/editing/userdefinedsnippets#_snippet-syntax
    pub body: Option<Value>,
    /// Defines the data to insert for the snippet as a string literal. When the user
    /// selects the snippet, VS Code inserts the text _without_ the enclosing quotation
    /// marks at the cursor. You can use [snippet syntax][01] to define tabstops,
    /// placeholders, and variables in the `bodyText`.
    ///
    /// Alternatively, you can define the `body` property for the snippet, which specifies
    /// the text to insert for the snippet as data.
    ///
    /// If you define both the `bodyText` and `body` properties for a snippet, the `body`
    /// definition overrides the `bodyText` property.
    ///
    /// [01]: https://code.visualstudio.com/docs/editing/userdefinedsnippets#_snippet-syntax
    pub body_text: Option<String>,
}

/// Defines the `defaultSnippets` keyword for the VS Code vocabulary.
///
/// This keyword Provides snippets for completion of a schema or subschema value or property.
///
/// By default, VS Code presents a set of completion options for data with an associated JSON
/// Schema like suggesting defined property names or enum values. You can use the
/// `defaultSnippets` keyword to provide an array of snippets with more control over the
/// presentation, default values, and enable users to quickly fill out the snippet.
///
/// The keyword expects an array of objects that each define a snippet. For more information
/// about defining snippets, see [Define snippets in JSON Schemas][01]. For more information
/// about the snippet syntax, see [Snippet syntax][02].
///
/// [01]: https://code.visualstudio.com/Docs/languages/json#_define-snippets-in-json-schemas
/// [02]: https://code.visualstudio.com/docs/editing/userdefinedsnippets#_snippet-syntax
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DefaultSnippetsKeyword(Vec<Snippet>);

impl DefaultSnippetsKeyword {
    /// Creates a new instance of the keyword from a vector of snippets.
    #[must_use]
    pub fn new(snippets: Vec<Snippet>) -> Self {
        Self(snippets)
    }
}

impl VSCodeKeywordDefinition for DefaultSnippetsKeyword {
    const KEYWORD_NAME: &str = "defaultSnippets";
    const KEYWORD_ID: &str = "https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/vscode/v0/keywords/defaultSnippets.json";

    fn keyword_factory<'a>(
        _parent: &'a serde_json::Map<String, Value>,
        value: &'a Value,
        path: Location,
    ) -> Result<Box<dyn Keyword>, ValidationError<'a>> {
        if let Some(v) = value.as_array() {
            if v.iter().any(|item| item.as_object().is_none()) {
                Err(ValidationError::custom(
                    Location::new(),
                    path,
                    value,
                    format!(
                        "{} {}",
                        t!("vscode.keywords.default_snippets.factory_error_non_object_item"),
                        t!("vscode.keywords.default_snippets.factory_error_suffix"),
                    ),
                ))
            } else if let Ok(snippets) = serde_json::from_value::<DefaultSnippetsKeyword>(value.clone()){
                Ok(Box::new(snippets))
            } else {
                Err(ValidationError::custom(
                    Location::new(),
                    path,
                    value,
                    format!(
                        "{} {}",
                        t!("vscode.keywords.default_snippets.factory_error_invalid_item"),
                        t!("vscode.keywords.default_snippets.factory_error_suffix"),
                    )
                ))
            }
        } else {
            Err(ValidationError::custom(
                Location::new(),
                path,
                value,
                format!(
                    "{} {}",
                    t!("vscode.keywords.default_snippets.factory_error_not_array"),
                    t!("vscode.keywords.default_snippets.factory_error_suffix"),
                ),
            ))
        }
    }
}

impl JsonSchema for DefaultSnippetsKeyword {
    fn json_schema(_: &mut schemars::SchemaGenerator) -> schemars::Schema {
        json_schema!({
            "$schema": Self::META_SCHEMA,
            "$id": Self::KEYWORD_ID,
            "title": t!("vscode.keywords.defaultSnippets.title"),
            "description": t!("vscode.keywords.defaultSnippets.description"),
            "markdownDescription": t!("vscode.keywords.defaultSnippets.markdownDescription"),
            "unevaluatedItems": false,
            "type": "array",
            "items": {
                "title": t!("vscode.keywords.defaultSnippets.items.title"),
                "description": t!("vscode.keywords.defaultSnippets.items.description"),
                "markdownDescription": t!("vscode.keywords.defaultSnippets.items.markdownDescription"),
                "type": "object",
                "unevaluatedProperties": false,
                "properties": {
                    "label": {
                        "title": t!("vscode.keywords.defaultSnippets.items.properties.label.title"),
                        "description": t!("vscode.keywords.defaultSnippets.items.properties.label.description"),
                        "markdownDescription": t!("vscode.keywords.defaultSnippets.items.properties.label.markdownDescription"),
                        "type": "string"
                    },
                    "description": {
                        "title": t!("vscode.keywords.defaultSnippets.items.properties.description.title"),
                        "description": t!("vscode.keywords.defaultSnippets.items.properties.description.description"),
                        "markdownDescription": t!("vscode.keywords.defaultSnippets.items.properties.description.markdownDescription"),
                        "type": "string"
                    },
                    "markdownDescription": {
                        "title": t!("vscode.keywords.defaultSnippets.items.properties.markdownDescription.title"),
                        "description": t!("vscode.keywords.defaultSnippets.items.properties.markdownDescription.description"),
                        "markdownDescription": t!("vscode.keywords.defaultSnippets.items.properties.markdownDescription.markdownDescription"),
                        "type": "string"
                    },
                    "body": {
                        "title": t!("vscode.keywords.defaultSnippets.items.properties.body.title"),
                        "description": t!("vscode.keywords.defaultSnippets.items.properties.body.description"),
                        "markdownDescription": t!("vscode.keywords.defaultSnippets.items.properties.body.markdownDescription"),
                    },
                    "bodyText": {
                        "title": t!("vscode.keywords.defaultSnippets.items.properties.bodyText.title"),
                        "description": t!("vscode.keywords.defaultSnippets.items.properties.bodyText.description"),
                        "markdownDescription": t!("vscode.keywords.defaultSnippets.items.properties.bodyText.markdownDescription"),
                        "type": "string"
                    },
                },
            }
        })
    }

    fn schema_name() -> Cow<'static, str> {
        Self::KEYWORD_ID.into()
    }
}

impl Keyword for DefaultSnippetsKeyword {
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

impl From<DefaultSnippetsKeyword> for Vec<Snippet> {
    fn from(value: DefaultSnippetsKeyword) -> Self {
        value.0
    }
}

impl From<Vec<Snippet>> for DefaultSnippetsKeyword {
    fn from(value: Vec<Snippet>) -> Self {
        Self(value)
    }
}