// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use std::str::FromStr;

use jsonschema::ValidationOptions;

use crate::vscode::keywords::{
    AllowCommentsKeyword,
    AllowTrailingCommasKeyword,
    CompletionDetailKeyword,
    DefaultSnippetsKeyword,
    DeprecationMessageKeyword,
    DoNotSuggestKeyword,
    EnumDescriptionsKeyword,
    EnumDetailsKeyword,
    EnumSortTextsKeyword,
    ErrorMessageKeyword,
    MarkdownDescriptionKeyword,
    MarkdownEnumDescriptionsKeyword,
    PatternErrorMessageKeyword,
    SuggestSortTextKeyword,
    VSCodeKeywordDefinition
};

/// Defines the available keywords for VS Code's extended vocabulary.
///
/// These keywords are annotation keywords that don't change the validation processing, so any
/// consumer of a schema using these keywords can safely ignore them if it doesn't understand
/// the keywords.
///
/// The transformers and generators in this library strip the VS Code keywords from canonical
/// schemas, as they are primarily for improving the development experience in a code editor,
/// not machine processing. Removing them from canonical schemas makes canonical schemas smaller
/// and more compatible, as some JSON Schema implementations may error on unrecognized keywords
/// instead of ignoring them.
pub enum VSCodeKeyword {
    AllowComments,
    AllowTrailingCommas,
    CompletionDetail,
    DefaultSnippets,
    DeprecationMessage,
    DoNotSuggest,
    EnumDescriptions,
    EnumDetails,
    EnumSortTexts,
    ErrorMessage,
    MarkdownDescription,
    MarkdownEnumDescriptions,
    PatternErrorMessage,
    SuggestSortText
}

impl VSCodeKeyword {
    /// Contains the name of every keyword in the VS Code vocabulary.
    pub const ALL: [&str; 14] = [
        AllowCommentsKeyword::KEYWORD_NAME,
        AllowTrailingCommasKeyword::KEYWORD_NAME,
        CompletionDetailKeyword::KEYWORD_NAME,
        DefaultSnippetsKeyword::KEYWORD_NAME,
        DeprecationMessageKeyword::KEYWORD_NAME,
        DoNotSuggestKeyword::KEYWORD_NAME,
        EnumDescriptionsKeyword::KEYWORD_NAME,
        EnumDetailsKeyword::KEYWORD_NAME,
        EnumSortTextsKeyword::KEYWORD_NAME,
        ErrorMessageKeyword::KEYWORD_NAME,
        MarkdownDescriptionKeyword::KEYWORD_NAME,
        MarkdownEnumDescriptionsKeyword::KEYWORD_NAME,
        PatternErrorMessageKeyword::KEYWORD_NAME,
        SuggestSortTextKeyword::KEYWORD_NAME,
    ];
    /// Contains the name of every keyword in the VS Code vocabulary that affects whether and how
    /// a schema or subschema is presented for completion with IntelliSense in VS Code.
    pub const COMPLETION: [&str; 6] = [
        CompletionDetailKeyword::KEYWORD_NAME,
        DefaultSnippetsKeyword::KEYWORD_NAME,
        DoNotSuggestKeyword::KEYWORD_NAME,
        EnumDetailsKeyword::KEYWORD_NAME,
        EnumSortTextsKeyword::KEYWORD_NAME,
        SuggestSortTextKeyword::KEYWORD_NAME,
    ];
    /// Contains the name of every keyword in the VS Code vocabulary that provides enhanced
    /// documentation for a schema or subschema.
    pub const DOCUMENTATION: [&str; 4] = [
        DeprecationMessageKeyword::KEYWORD_NAME,
        EnumDescriptionsKeyword::KEYWORD_NAME,
        MarkdownDescriptionKeyword::KEYWORD_NAME,
        MarkdownEnumDescriptionsKeyword::KEYWORD_NAME,
    ];
    /// Contains the name of every keyword in the VS Code vocabulary that provides enhanced error
    /// messaging for invalid instances.
    pub const ERROR: [&str; 2] = [
        ErrorMessageKeyword::KEYWORD_NAME,
        PatternErrorMessageKeyword::KEYWORD_NAME,
    ];
    /// Contains the name of every keyword in the VS Code vocabulary that affects how VS Code
    /// validates the JSON from a parsing perspective as opposed to validating the _data_.
    pub const PARSING: [&str; 2] = [
        AllowCommentsKeyword::KEYWORD_NAME,
        AllowTrailingCommasKeyword::KEYWORD_NAME,
    ];

    /// Returns the name of a keyword for use in a JSON Schema, like `allowComments` for
    /// [`VSCodeKeyword::AllowComments`].
    #[must_use]
    pub const fn name(&self) -> &str {
        match self {
            Self::AllowComments => AllowCommentsKeyword::KEYWORD_NAME,
            Self::AllowTrailingCommas => AllowTrailingCommasKeyword::KEYWORD_NAME,
            Self::CompletionDetail => CompletionDetailKeyword::KEYWORD_NAME,
            Self::DefaultSnippets => DefaultSnippetsKeyword::KEYWORD_NAME,
            Self::DeprecationMessage => DeprecationMessageKeyword::KEYWORD_NAME,
            Self::DoNotSuggest => DoNotSuggestKeyword::KEYWORD_NAME,
            Self::EnumDescriptions => EnumDescriptionsKeyword::KEYWORD_NAME,
            Self::EnumDetails => EnumDetailsKeyword::KEYWORD_NAME,
            Self::EnumSortTexts => EnumSortTextsKeyword::KEYWORD_NAME,
            Self::ErrorMessage => ErrorMessageKeyword::KEYWORD_NAME,
            Self::MarkdownDescription => MarkdownDescriptionKeyword::KEYWORD_NAME,
            Self::MarkdownEnumDescriptions => MarkdownEnumDescriptionsKeyword::KEYWORD_NAME,
            Self::PatternErrorMessage => PatternErrorMessageKeyword::KEYWORD_NAME,
            Self::SuggestSortText => SuggestSortTextKeyword::KEYWORD_NAME,
        }
    }
    /// Registers the keyword with an instance of [`ValidationOptions`] from the `jsonschema`
    /// crate.
    /// 
    /// This convenience method enables you to quickly add keywords to the validator. However,
    /// it doesn't follow the builder pattern typically used with [`ValidationOptions`].
    /// 
    /// For a more ergonomic way to register keywords, see [`VSCodeValidationOptionsExtensions`].
    /// 
    /// [`VSCodeValidationOptionsExtensions`]: crate::vscode::VSCodeValidationOptionsExtensions
    pub fn register(self, options: ValidationOptions) -> ValidationOptions {
        match self {
            Self::AllowComments => options.with_keyword(
                AllowCommentsKeyword::KEYWORD_NAME,
                AllowCommentsKeyword::keyword_factory
            ).with_resource(
                AllowCommentsKeyword::KEYWORD_ID,
                AllowCommentsKeyword::default_schema_resource()
            ),

            Self::AllowTrailingCommas => options.with_keyword(
                AllowTrailingCommasKeyword::KEYWORD_NAME,
                AllowTrailingCommasKeyword::keyword_factory
            ).with_resource(
                AllowTrailingCommasKeyword::KEYWORD_ID,
                AllowTrailingCommasKeyword::default_schema_resource()
            ),

            Self::CompletionDetail => options.with_keyword(
                CompletionDetailKeyword::KEYWORD_NAME,
                CompletionDetailKeyword::keyword_factory
            ).with_resource(
                CompletionDetailKeyword::KEYWORD_ID,
                CompletionDetailKeyword::default_schema_resource()
            ),
            Self::DefaultSnippets => options.with_keyword(
                DefaultSnippetsKeyword::KEYWORD_NAME,
                DefaultSnippetsKeyword::keyword_factory
            ).with_resource(
                DefaultSnippetsKeyword::KEYWORD_ID,
                DefaultSnippetsKeyword::default_schema_resource()
            ),
            Self::DeprecationMessage => options.with_keyword(
                DeprecationMessageKeyword::KEYWORD_NAME,
                DeprecationMessageKeyword::keyword_factory
            ).with_resource(
                DeprecationMessageKeyword::KEYWORD_ID,
                DeprecationMessageKeyword::default_schema_resource()
            ),
            Self::DoNotSuggest => options.with_keyword(
                DoNotSuggestKeyword::KEYWORD_NAME,
                DoNotSuggestKeyword::keyword_factory
            ).with_resource(
                DoNotSuggestKeyword::KEYWORD_ID,
                DoNotSuggestKeyword::default_schema_resource()
            ),
            Self::EnumDescriptions => options.with_keyword(
                EnumDescriptionsKeyword::KEYWORD_NAME,
                EnumDescriptionsKeyword::keyword_factory
            ).with_resource(
                EnumDescriptionsKeyword::KEYWORD_ID,
                EnumDescriptionsKeyword::default_schema_resource()
            ),
            Self::EnumDetails => options.with_keyword(
                EnumDetailsKeyword::KEYWORD_NAME,
                EnumDetailsKeyword::keyword_factory
            ).with_resource(
                EnumDetailsKeyword::KEYWORD_ID,
                EnumDetailsKeyword::default_schema_resource()
            ),
            Self::EnumSortTexts => options.with_keyword(
                EnumSortTextsKeyword::KEYWORD_NAME,
                EnumSortTextsKeyword::keyword_factory
            ).with_resource(
                EnumSortTextsKeyword::KEYWORD_ID,
                EnumSortTextsKeyword::default_schema_resource()
            ),
            Self::ErrorMessage => options.with_keyword(
                ErrorMessageKeyword::KEYWORD_NAME,
                ErrorMessageKeyword::keyword_factory
            ).with_resource(
                ErrorMessageKeyword::KEYWORD_ID,
                ErrorMessageKeyword::default_schema_resource()
            ),
            Self::MarkdownDescription => options.with_keyword(
                MarkdownDescriptionKeyword::KEYWORD_NAME,
                MarkdownDescriptionKeyword::keyword_factory
            ).with_resource(
                MarkdownDescriptionKeyword::KEYWORD_ID,
                MarkdownDescriptionKeyword::default_schema_resource()
            ),
            Self::MarkdownEnumDescriptions => options.with_keyword(
                MarkdownEnumDescriptionsKeyword::KEYWORD_NAME,
                MarkdownEnumDescriptionsKeyword::keyword_factory
            ).with_resource(
                MarkdownEnumDescriptionsKeyword::KEYWORD_ID,
                MarkdownEnumDescriptionsKeyword::default_schema_resource()
            ),
            Self::PatternErrorMessage => options.with_keyword(
                PatternErrorMessageKeyword::KEYWORD_NAME,
                PatternErrorMessageKeyword::keyword_factory
            ).with_resource(
                PatternErrorMessageKeyword::KEYWORD_ID,
                PatternErrorMessageKeyword::default_schema_resource()
            ),
            Self::SuggestSortText => options.with_keyword(
                SuggestSortTextKeyword::KEYWORD_NAME,
                SuggestSortTextKeyword::keyword_factory
            ).with_resource(
                SuggestSortTextKeyword::KEYWORD_ID,
                SuggestSortTextKeyword::default_schema_resource()
            ),
        }
    }
}

impl From<VSCodeKeyword> for String {
    fn from(value: VSCodeKeyword) -> Self {
        value.name().to_string()
    }
}
