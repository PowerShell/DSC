// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![allow(unused_imports)]

use jsonschema::{Keyword, Resource, ValidationError, ValidationOptions, paths::Location};
use schemars::{JsonSchema, Schema, json_schema};
use serde_json::{Map, Value};

// Define the trait for implementing keywords and re-export it from `vscode::keywords`
mod vscode_keyword_definition;
pub use vscode_keyword_definition::VSCodeKeywordDefinition;
/// Define the enum for available keywords and re-export it from `vscode::keywords`
mod vscode_keyword;
pub use vscode_keyword::VSCodeKeyword;
// Define the keywords in separate modules and re-export them from `vscode::keywords`
mod allow_comments;
pub use allow_comments::AllowCommentsKeyword;
mod allow_trailing_commas;
pub use allow_trailing_commas::AllowTrailingCommasKeyword;
mod completion_detail;
pub use completion_detail::CompletionDetailKeyword;
mod default_snippets;
pub use default_snippets::{DefaultSnippetsKeyword, Snippet};
mod deprecation_message;
pub use deprecation_message::DeprecationMessageKeyword;
mod do_not_suggest;
pub use do_not_suggest::DoNotSuggestKeyword;
mod enum_descriptions;
pub use enum_descriptions::EnumDescriptionsKeyword;
mod enum_details;
pub use enum_details::EnumDetailsKeyword;
mod enum_sort_texts;
pub use enum_sort_texts::EnumSortTextsKeyword;
mod error_message;
pub use error_message::ErrorMessageKeyword;
mod markdown_description;
pub use markdown_description::MarkdownDescriptionKeyword;
mod markdown_enum_descriptions;
pub use markdown_enum_descriptions::MarkdownEnumDescriptionsKeyword;
mod pattern_error_message;
pub use pattern_error_message::PatternErrorMessageKeyword;
mod suggest_sort_text;
pub use suggest_sort_text::SuggestSortTextKeyword;

