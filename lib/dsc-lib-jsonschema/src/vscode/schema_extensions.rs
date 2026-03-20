// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use core::{clone::Clone, iter::Iterator, option::Option::None};

use schemars::Schema;

use crate::{
    schema_utility_extensions::SchemaUtilityExtensions,
    vscode::{
        dialect::VSCodeDialect,
        keywords::{
        AllowCommentsKeyword, AllowTrailingCommasKeyword, CompletionDetailKeyword, DefaultSnippetsKeyword, DeprecationMessageKeyword, DoNotSuggestKeyword, EnumDescriptionsKeyword, EnumDetailsKeyword, EnumSortTextsKeyword, ErrorMessageKeyword, MarkdownDescriptionKeyword, MarkdownEnumDescriptionsKeyword, PatternErrorMessageKeyword, Snippet, SuggestSortTextKeyword, VSCodeKeywordDefinition
    }}
};

/// Provides helper functions for working with VS Code's extended JSON Schema keywords with
/// [`schemars::Schema`] instances.
///
/// The `get_*` functions simplify retrieving the value of a VS Code keyword for a given type. If
/// the schema defines the keyword with the expected type, those functions return a reference to
/// that value as the correct type. If the keyword doesn't exist or has the wrong value type, the
/// functions return [`None`].
/// 
/// The `should_*` function simplify retrieving the effective boolean value for the following
/// keywords, returning the defined value if it exists and otherwise `false`:
/// 
/// - [`allowComments`]
/// - [`allowTrailingCommas`]
/// - [`doNotSuggest`]
/// 
/// [`allowComments`]: AllowCommentsKeyword
/// [`allowTrailingCommas`]: AllowTrailingCommasKeyword
/// [`doNotSuggest`]: DoNotSuggestKeyword
pub trait VSCodeSchemaExtensions {
    /// Retrieves the value for the [`CompletionDetailKeyword`] (`completionDetail`) if it's
    /// defined in the schema.
    /// 
    /// If the schema doesn't define the keyword, or defines the keyword with an invalid value,
    /// this method returns [`None`]. Otherwise, this method returns the completion detail string.
    fn get_completion_detail(&self) -> Option<&str>;
    /// Retrieves the value for the [`DefaultSnippetsKeyword`] (`defaultSnippets`) if it's
    /// defined in the schema.
    /// 
    /// If the schema doesn't define the keyword, or defines the keyword with an invalid value,
    /// this method returns [`None`]. Otherwise, this method returns a vector of snippets.
    fn get_default_snippets(&self) -> Option<Vec<Snippet>>;
    /// Retrieves the value for the [`DeprecationMessageKeyword`] (`deprecationMessage`) if it's
    /// defined in the schema.
    /// 
    /// If the schema doesn't define the keyword, or defines the keyword with an invalid value,
    /// this method returns [`None`]. Otherwise, this method returns the deprecation message string.
    fn get_deprecation_message(&self) -> Option<&str>;
    /// Retrieves the value for the [`EnumDescriptionsKeyword`] (`enumDescriptions`) if it's
    /// defined in the schema.
    /// 
    /// If the schema doesn't define the keyword, or defines the keyword with an invalid value,
    /// this method returns [`None`]. Otherwise, this method returns a vector of strings, where
    /// each string describes a value for the `enum` keyword.
    /// 
    /// Each item in the vector this method returns corresponds to an item in the vector for the
    /// `enum` keyword by index.
    fn get_enum_descriptions(&self) -> Option<Vec<&str>>;
    /// Retrieves the value for the [`EnumDetailsKeyword`] (`enumDetails`) if it's
    /// defined in the schema.
    /// 
    /// If the schema doesn't define the keyword, or defines the keyword with an invalid value,
    /// this method returns [`None`]. Otherwise, this method returns a vector of strings, where
    /// each string provides completion details for a value for the `enum` keyword.
    /// 
    /// Each item in the vector this method returns corresponds to an item in the vector for the
    /// `enum` keyword by index.
    fn get_enum_details(&self) -> Option<Vec<&str>>;
    /// Retrieves the value for the [`EnumSortTextsKeyword`] (`enumSortTexts`) if it's
    /// defined in the schema.
    /// 
    /// If the schema doesn't define the keyword, or defines the keyword with an invalid value,
    /// this method returns [`None`]. Otherwise, this method returns a vector of strings, where
    /// each string provides an alternate string to sort the values when using IntelliSense.
    /// 
    /// Each item in the vector this method returns corresponds to an item in the vector for the
    /// `enum` keyword by index.
    fn get_enum_sort_texts(&self) -> Option<Vec<&str>>;
    /// Retrieves the value for the [`ErrorMessageKeyword`] (`errorMessage`) if it's
    /// defined in the schema.
    /// 
    /// If the schema doesn't define the keyword, or defines the keyword with an invalid value,
    /// this method returns [`None`]. Otherwise, this method returns the error message string.
    fn get_error_message(&self) -> Option<&str>;
    /// Retrieves the value for the [`MarkdownDescriptionKeyword`] (`markdownDescription`) if it's
    /// defined in the schema.
    /// 
    /// If the schema doesn't define the keyword, or defines the keyword with an invalid value,
    /// this method returns [`None`]. Otherwise, this method returns the description string.
    fn get_markdown_description(&self) -> Option<&str>;
    /// Retrieves the value for the [`MarkdownEnumDescriptionsKeyword`] (`markdownEnumDescriptions`)
    /// if it's defined in the schema.
    /// 
    /// If the schema doesn't define the keyword, or defines the keyword with an invalid value,
    /// this method returns [`None`]. Otherwise, this method returns a vector of strings, where
    /// each string describes a value for the `enum` keyword.
    /// 
    /// Each item in the vector this method returns corresponds to an item in the vector for the
    /// `enum` keyword by index.
    fn get_markdown_enum_descriptions(&self) -> Option<Vec<&str>>;
    /// Retrieves the value for the [`PatternErrorMessageKeyword`] (`patternErrorMessage`) if it's
    /// defined in the schema.
    /// 
    /// If the schema doesn't define the keyword, or defines the keyword with an invalid value,
    /// this method returns [`None`]. Otherwise, this method returns the error message string.
    fn get_pattern_error_message(&self) -> Option<&str>;
    /// Retrieves the value for the [`SuggestSortTextKeyword`] (`suggestSortText`) if it's
    /// defined in the schema.
    /// 
    /// If the schema doesn't define the keyword, or defines the keyword with an invalid value,
    /// this method returns [`None`]. Otherwise, this method returns the alternate string to sort
    /// the instance with when using IntelliSense
    fn get_suggest_sort_text(&self) -> Option<&str>;
    /// Indicates whether the schema defines the [`AllowCommentsKeyword`] (`allowComments`) as
    /// `true`.
    /// 
    /// This method returns `true` only when the schema defines the keyword as `true`. If the
    /// schema doesn't define the keyword, defines it incorrectly, or defines it as `false`, this
    /// method returns `false`. 
    fn should_allow_comments(&self) -> bool;
    /// Indicates whether the schema defines the [`AllowTrailingCommasKeyword`]
    /// (`allowTrailingCommas`) as `true`.
    /// 
    /// This method returns `true` only when the schema defines the keyword as `true`. If the
    /// schema doesn't define the keyword, defines it incorrectly, or defines it as `false`, this
    /// method returns `false`.
    fn should_allow_trailing_commas(&self) -> bool;
    /// Indicates whether the schema defines the [`DoNotSuggestKeyword`] (`doNotSuggest`) as
    /// `true`.
    /// 
    /// This method returns `true` only when the schema defines the keyword as `true`. If the
    /// schema doesn't define the keyword, defines it incorrectly, or defines it as `false`, this
    /// method returns `false`.
    fn should_not_suggest(&self) -> bool;
    /// Indicates whether the schema uses the [`VSCodeDialect`] by checking the value of the
    /// `$schema` keyword.
    /// 
    /// If the schema doesn't define its dialect or defines a different dialect, this method
    /// returns `false`. If the schema defines its dialect as the VS Code dialect, this method
    /// returns `true`.
    /// 
    /// Note that any schema may use keywords from the VS Code vocabulary even if the schema uses
    /// a different dialect. The keywords are annotation keywords and don't affect validation for
    /// data instances, so validators may safely ignore them.
    /// 
    /// [`VSCodeVocabulary`]: crate::vscode::vocabulary::VSCodeVocabulary
    fn uses_vscode_dialect(&self) -> bool;
}

impl VSCodeSchemaExtensions for Schema {
    fn get_markdown_description(&self) -> Option<&str> {
        self.get_keyword_as_str(MarkdownDescriptionKeyword::KEYWORD_NAME)
    }
    fn get_markdown_enum_descriptions(&self) -> Option<Vec<&str>> {
        match self.get_keyword_as_array(MarkdownEnumDescriptionsKeyword::KEYWORD_NAME) {
            None => None,
            Some(list) => list.iter().map(|v| v.as_str()).collect(),
        }
    }
    fn should_allow_comments(&self) -> bool {
        self.get_keyword_as_bool(AllowCommentsKeyword::KEYWORD_NAME).unwrap_or_default()
    }
    fn should_allow_trailing_commas(&self) -> bool {
        self.get_keyword_as_bool(AllowTrailingCommasKeyword::KEYWORD_NAME).unwrap_or_default()
    }
    fn should_not_suggest(&self) -> bool {
        self.get_keyword_as_bool(DoNotSuggestKeyword::KEYWORD_NAME).unwrap_or_default()
    }
    fn get_completion_detail(&self) -> Option<&str> {
        self.get_keyword_as_str(CompletionDetailKeyword::KEYWORD_NAME)
    }
    fn get_deprecation_message(&self) -> Option<&str> {
        self.get_keyword_as_str(DeprecationMessageKeyword::KEYWORD_NAME)
    }
    fn get_enum_descriptions(&self) -> Option<Vec<&str>> {
        match self.get_keyword_as_array(EnumDescriptionsKeyword::KEYWORD_NAME) {
            None => None,
            Some(list) => list.iter().map(|v| v.as_str()).collect(),
        }
    }
    fn get_enum_details(&self) -> Option<Vec<&str>> {
        match self.get_keyword_as_array(EnumDetailsKeyword::KEYWORD_NAME) {
            None => None,
            Some(list) => list.iter().map(|v| v.as_str()).collect(),
        }
    }
    fn get_enum_sort_texts(&self) -> Option<Vec<&str>> {
        match self.get_keyword_as_array(EnumSortTextsKeyword::KEYWORD_NAME) {
            None => None,
            Some(list) => list.iter().map(|v| v.as_str()).collect(),
        }
    }
    fn get_error_message(&self) -> Option<&str> {
        self.get_keyword_as_str(ErrorMessageKeyword::KEYWORD_NAME)
    }
    fn get_pattern_error_message(&self) -> Option<&str> {
        self.get_keyword_as_str(PatternErrorMessageKeyword::KEYWORD_NAME)
    }
    fn get_suggest_sort_text(&self) -> Option<&str> {
        self.get_keyword_as_str(SuggestSortTextKeyword::KEYWORD_NAME)
    }
    fn get_default_snippets(&self) -> Option<Vec<Snippet>> {
        match self.get(DefaultSnippetsKeyword::KEYWORD_NAME) {
            Some(snippets_json) => serde_json::from_value(snippets_json.clone()).ok(),
            None => None,
        }
    }
    fn uses_vscode_dialect(&self) -> bool {
        if let Some(dialect) = self.get("$schema").and_then(|d| d.as_str()) {
            dialect == VSCodeDialect::SCHEMA_ID
        } else {
            false
        }
    }
}