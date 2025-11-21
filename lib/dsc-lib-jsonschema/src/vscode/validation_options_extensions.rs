// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use jsonschema::ValidationOptions;

use crate::vscode::{
    dialect::{VSCodeDialect, VSCODE_DIALECT_SCHEMA_RESOURCE_CANONICAL},
    vocabulary::{VSCodeVocabulary, VSCODE_VOCABULARY_SCHEMA_RESOURCE_CANONICAL},
    keywords::VSCodeKeyword,
};

/// Defines extension methods to the [`jsonschema::ValidationOptions`] to simplify registering the
/// VS Code [keywords], [vocabulary], and [dialect meta schema].
/// 
/// [keywords]: VSCodeKeyword
/// [vocabulary]: VSCodeVocabulary
/// [dialect meta schema]: VSCodeDialect
pub trait VSCodeValidationOptionsExtensions {
    /// Registers a single VS Code keyword for use with a [`jsonschema::Validator`].
    /// 
    /// This function registers a specific VS Code keyword and the schema resource that defines it
    /// with the [`with_keyword()`] and [`with_resource()`] builder methods.
    /// 
    /// [`with_keyword()`]: ValidationOptions::with_keyword
    /// [`with_resource()`]: ValidationOptions::with_resource
    /// [`with_vscode_dialect()`]: VSCodeValidationOptionsExtensions::with_vscode_dialect
    /// [`with_vscode_vocabulary()`]: VSCodeValidationOptionsExtensions::with_vscode_vocabulary
    fn with_vscode_keyword(self, keyword: VSCodeKeyword) -> ValidationOptions;
    /// Registers every VS Code keyword for use with a [`jsonschema::Validator`].
    /// 
    /// This function registers each of the VS Code keywords and the schema resources that define
    /// them with the [`with_keyword()`] and [`with_resource()`] builder methods.
    /// 
    /// Use this function when you only want to register the VS Code vocabulary keywords.
    /// If you are using the VS Code vocabulary in your own meta schema dialect, use the
    /// [`with_vscode_vocabulary()`] method instead. If you are using the VS Code meta schema
    /// dialect directly, use the [`with_vscode_dialect()`] method instead.
    /// 
    /// [`with_keyword()`]: ValidationOptions::with_keyword
    /// [`with_resource()`]: ValidationOptions::with_resource
    /// [`with_vscode_dialect()`]: VSCodeValidationOptionsExtensions::with_vscode_dialect
    /// [`with_vscode_vocabulary()`]: VSCodeValidationOptionsExtensions::with_vscode_vocabulary
    fn with_vscode_keywords(self) -> ValidationOptions;
    /// Registers the VS Code completion keywords for use with a [`jsonschema::Validator`].
    /// 
    /// This function registers the following VS Code Keywords and the schema resources that
    /// define them with the [`with_keyword()`] and [`with_resource()`] builder methods:
    /// 
    /// - [`completionDetail`]
    /// - [`defaultSnippets`]
    /// - [`doNotSuggest`]
    /// - [`enumDetails`]
    /// - [`enumSortTexts`]
    /// - [`suggestSortText`]
    /// 
    /// Use this function when you only want to register a subset of VS Code vocabulary keywords.
    /// If you are using the VS Code vocabulary in your own meta schema dialect, use the
    /// [`with_vscode_vocabulary()`] method instead. If you are using the VS Code meta schema
    /// dialect directly, use the [`with_vscode_dialect()`] method instead.
    /// 
    /// [`completionDetail`]: super::keywords::CompletionDetailKeyword
    /// [`defaultSnippets`]: super::keywords::DefaultSnippetsKeyword
    /// [`doNotSuggest`]: super::keywords::DoNotSuggestKeyword
    /// [`enumDetails`]: super::keywords::EnumDetailsKeyword
    /// [`enumSortTexts`]: super::keywords::EnumSortTextsKeyword
    /// [`suggestSortText`]: super::keywords::SuggestSortTextKeyword
    /// [`with_keyword()`]: ValidationOptions::with_keyword
    /// [`with_resource()`]: ValidationOptions::with_resource
    /// [`with_vscode_dialect()`]: VSCodeValidationOptionsExtensions::with_vscode_dialect
    /// [`with_vscode_vocabulary()`]: VSCodeValidationOptionsExtensions::with_vscode_vocabulary
    fn with_vscode_completion_keywords(self) -> ValidationOptions;
    /// Registers the VS Code documentation keywords for use with a [`jsonschema::Validator`].
    /// 
    /// This function registers the following VS Code Keywords and the schema resources that
    /// define them with the [`with_keyword()`] and [`with_resource()`] builder methods:
    /// 
    /// - [`deprecationMessageKeyword`]
    /// - [`enumDescriptionsKeyword`]
    /// - [`markdownDescriptionKeyword`]
    /// - [`markdownEnumDescriptionsKeyword`]
    /// 
    /// Use this function when you only want to register a subset of VS Code vocabulary keywords.
    /// If you are using the VS Code vocabulary in your own meta schema dialect, use the
    /// [`with_vscode_vocabulary()`] method instead. If you are using the VS Code meta schema
    /// dialect directly, use the [`with_vscode_dialect()`] method instead.
    /// 
    /// [`deprecationMessageKeyword`]: super::keywords::DeprecationMessageKeyword
    /// [`enumDescriptionsKeyword`]: super::keywords::EnumDescriptionsKeyword
    /// [`markdownDescriptionKeyword`]: super::keywords::MarkdownDescriptionKeyword
    /// [`markdownEnumDescriptionsKeyword`]: super::keywords::MarkdownEnumDescriptionsKeyword
    /// [`enumSortTexts`]: super::keywords::EnumSortTextsKeyword
    /// [`suggestSortText`]: super::keywords::SuggestSortTextKeyword
    /// [`with_keyword()`]: ValidationOptions::with_keyword
    /// [`with_resource()`]: ValidationOptions::with_resource
    /// [`with_vscode_dialect()`]: VSCodeValidationOptionsExtensions::with_vscode_dialect
    /// [`with_vscode_vocabulary()`]: VSCodeValidationOptionsExtensions::with_vscode_vocabulary
    fn with_vscode_documentation_keywords(self) -> ValidationOptions;
    /// Registers the VS Code error messaging keywords for use with a [`jsonschema::Validator`].
    /// 
    /// This function registers the following VS Code Keywords and the schema resources that
    /// define them with the [`with_keyword()`] and [`with_resource()`] builder methods:
    /// 
    /// - [`errorMessageKeyword`]
    /// - [`patternErrorMessageKeyword`]
    /// 
    /// Use this function when you only want to register a subset of VS Code vocabulary keywords.
    /// If you are using the VS Code vocabulary in your own meta schema dialect, use the
    /// [`with_vscode_vocabulary()`] method instead. If you are using the VS Code meta schema
    /// dialect directly, use the [`with_vscode_dialect()`] method instead.
    /// 
    /// [`errorMessageKeyword`]: super::keywords::ErrorMessageKeyword
    /// [`patternErrorMessageKeyword`]: super::keywords::PatternErrorMessageKeyword
    /// [`with_keyword()`]: ValidationOptions::with_keyword
    /// [`with_resource()`]: ValidationOptions::with_resource
    /// [`with_vscode_dialect()`]: VSCodeValidationOptionsExtensions::with_vscode_dialect
    /// [`with_vscode_vocabulary()`]: VSCodeValidationOptionsExtensions::with_vscode_vocabulary
    fn with_vscode_error_keywords(self) -> ValidationOptions;
    /// Registers the VS Code parsing rules keywords for use with a [`jsonschema::Validator`].
    /// 
    /// This function registers the following VS Code Keywords and the schema resources that
    /// define them with the [`with_keyword()`] and [`with_resource()`] builder methods:
    /// 
    /// - [`allowCommentsKeyword`]
    /// - [`allowTrailingCommasKeyword`]
    /// 
    /// Use this function when you only want to register a subset of VS Code vocabulary keywords.
    /// If you are using the VS Code vocabulary in your own meta schema dialect, use the
    /// [`with_vscode_vocabulary()`] method instead. If you are using the VS Code meta schema
    /// dialect directly, use the [`with_vscode_dialect()`] method instead.
    /// 
    /// [`allowCommentsKeyword`]: super::keywords::AllowCommentsKeyword
    /// [`allowTrailingCommasKeyword`]: super::keywords::AllowTrailingCommasKeyword
    /// [`with_keyword()`]: ValidationOptions::with_keyword
    /// [`with_resource()`]: ValidationOptions::with_resource
    /// [`with_vscode_dialect()`]: VSCodeValidationOptionsExtensions::with_vscode_dialect
    /// [`with_vscode_vocabulary()`]: VSCodeValidationOptionsExtensions::with_vscode_vocabulary
    fn with_vscode_parsing_keywords(self) -> ValidationOptions;
    /// Registers the VS Code vocabulary and keywords for use with a [`jsonschema::Validator`].
    /// 
    /// This function registers each of the VS Code keywords and the schema resources that define
    /// them with the [`with_keyword()`] and [`with_resource()`] builder methods. It also registers
    /// the canonical form of the [vocabulary schema] as a schema resource.
    /// 
    /// This is a convenience method for registering the vocabulary and keywords. You don't need to
    /// separately add the keywords or schema resources. Use this convenience method when you are
    /// defining your own meta schema dialect that uses the VS Code vocabulary.
    /// 
    /// If you are using the VS Code meta schema directly without extending the dialect for your
    /// own purposes, use the [`with_vscode_dialect()`] method instead.
    /// 
    /// [`with_keyword()`]: ValidationOptions::with_keyword
    /// [`with_resource()`]: ValidationOptions::with_resource
    /// [vocabulary schema]: super::vocabulary::VSCODE_VOCABULARY_SCHEMA_CANONICAL
    /// [`with_vscode_dialect()`]: VSCodeValidationOptionsExtensions::with_vscode_dialect
    fn with_vscode_vocabulary(self) -> ValidationOptions;
    /// Registers the VS Code dialect meta schema, vocabulary, and keywords for use with a
    /// [`jsonschema::Validator`].
    /// 
    /// This function registers each of the VS Code keywords and the schema resources that define
    /// them with the [`with_keyword()`] and [`with_resource()`] builder methods. It also registers
    /// the canonical form of the [vocabulary schema] and [dialect meta schema] as schema resources.
    /// 
    /// This is a convenience method for registering the meta schema, vocabulary, and keywords
    /// together. You don't need to separately add the keywords or schema resources. Use this
    /// convenience method when you are using the VS Code meta schema dialect and vocabulary.
    /// 
    /// If you're using your own dialect that includes the VS Code vocabulary, use the
    /// [`with_vscode_vocabulary()`] method instead.
    /// 
    /// [`with_keyword()`]: ValidationOptions::with_keyword
    /// [`with_resource()`]: ValidationOptions::with_resource
    /// [vocabulary schema]: super::vocabulary::VSCODE_VOCABULARY_SCHEMA_CANONICAL
    /// [dialect meta schema]: super::dialect::VSCODE_DIALECT_SCHEMA_CANONICAL
    /// [`with_vscode_vocabulary()`]: VSCodeValidationOptionsExtensions::with_vscode_vocabulary
    fn with_vscode_dialect(self) -> ValidationOptions;
}

impl VSCodeValidationOptionsExtensions for ValidationOptions {
    fn with_vscode_keyword(self, keyword: VSCodeKeyword) -> Self {
        keyword.register(self)
    }
    fn with_vscode_keywords(self) -> Self {
        self
            .with_vscode_keyword(VSCodeKeyword::AllowComments)
            .with_vscode_keyword(VSCodeKeyword::AllowTrailingCommas)
            .with_vscode_keyword(VSCodeKeyword::CompletionDetail)
            .with_vscode_keyword(VSCodeKeyword::DefaultSnippets)
            .with_vscode_keyword(VSCodeKeyword::DeprecationMessage)
            .with_vscode_keyword(VSCodeKeyword::DoNotSuggest)
            .with_vscode_keyword(VSCodeKeyword::EnumDescriptions)
            .with_vscode_keyword(VSCodeKeyword::EnumDetails)
            .with_vscode_keyword(VSCodeKeyword::EnumSortTexts)
            .with_vscode_keyword(VSCodeKeyword::ErrorMessage)
            .with_vscode_keyword(VSCodeKeyword::MarkdownDescription)
            .with_vscode_keyword(VSCodeKeyword::MarkdownEnumDescriptions)
            .with_vscode_keyword(VSCodeKeyword::PatternErrorMessage)
            .with_vscode_keyword(VSCodeKeyword::SuggestSortText)
    }
    fn with_vscode_completion_keywords(self) -> Self {
        self
            .with_vscode_keyword(VSCodeKeyword::CompletionDetail)
            .with_vscode_keyword(VSCodeKeyword::DefaultSnippets)
            .with_vscode_keyword(VSCodeKeyword::DoNotSuggest)
            .with_vscode_keyword(VSCodeKeyword::EnumDetails)
            .with_vscode_keyword(VSCodeKeyword::EnumSortTexts)
            .with_vscode_keyword(VSCodeKeyword::SuggestSortText)
    }
    fn with_vscode_documentation_keywords(self) -> ValidationOptions {
        self
            .with_vscode_keyword(VSCodeKeyword::DeprecationMessage)
            .with_vscode_keyword(VSCodeKeyword::EnumDescriptions)
            .with_vscode_keyword(VSCodeKeyword::MarkdownDescription)
            .with_vscode_keyword(VSCodeKeyword::MarkdownEnumDescriptions)
    }
    fn with_vscode_error_keywords(self) -> ValidationOptions {
        self
            .with_vscode_keyword(VSCodeKeyword::ErrorMessage)
            .with_vscode_keyword(VSCodeKeyword::PatternErrorMessage)
    }
    fn with_vscode_parsing_keywords(self) -> ValidationOptions {
        self
            .with_vscode_keyword(VSCodeKeyword::AllowComments)
            .with_vscode_keyword(VSCodeKeyword::AllowTrailingCommas)
    }
    fn with_vscode_vocabulary(self) -> ValidationOptions {
        self
            .with_vscode_keywords()
            .with_resource(
                VSCodeVocabulary::SCHEMA_ID,
                (**VSCODE_VOCABULARY_SCHEMA_RESOURCE_CANONICAL).clone()
            )
    }
    fn with_vscode_dialect(self) -> ValidationOptions {
        self
            .with_vscode_vocabulary()
            .with_resource(
                VSCodeDialect::SCHEMA_ID,
                (**VSCODE_DIALECT_SCHEMA_RESOURCE_CANONICAL).clone()
            )
    }
}
