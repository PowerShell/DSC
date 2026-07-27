// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use std::sync::LazyLock;

use jsonschema::ValidationOptions;
use referencing::Registry;

use crate::vscode::{
    dialect::VSCodeDialect,
    vocabulary::VSCodeVocabulary,
    keywords::{
        VSCodeKeyword,
        VSCodeKeywordDefinition,
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
    },
};

/// A static registry containing all VS Code keyword schema resources, the vocabulary schema,
/// and the dialect meta schema.
///
/// This registry is lazily initialized on first use and lives for the duration of the program.
/// It is used by the [`VSCodeValidationOptionsExtensions`] trait methods to register schema
/// resources with `with_registry()`.
pub static VSCODE_DIALECT_REGISTRY: LazyLock<Registry<'static>> = LazyLock::new(|| {
    Registry::new()
        .add(AllowCommentsKeyword::KEYWORD_ID, AllowCommentsKeyword::default_schema_resource())
        .expect("valid URI")
        .add(AllowTrailingCommasKeyword::KEYWORD_ID, AllowTrailingCommasKeyword::default_schema_resource())
        .expect("valid URI")
        .add(CompletionDetailKeyword::KEYWORD_ID, CompletionDetailKeyword::default_schema_resource())
        .expect("valid URI")
        .add(DefaultSnippetsKeyword::KEYWORD_ID, DefaultSnippetsKeyword::default_schema_resource())
        .expect("valid URI")
        .add(DeprecationMessageKeyword::KEYWORD_ID, DeprecationMessageKeyword::default_schema_resource())
        .expect("valid URI")
        .add(DoNotSuggestKeyword::KEYWORD_ID, DoNotSuggestKeyword::default_schema_resource())
        .expect("valid URI")
        .add(EnumDescriptionsKeyword::KEYWORD_ID, EnumDescriptionsKeyword::default_schema_resource())
        .expect("valid URI")
        .add(EnumDetailsKeyword::KEYWORD_ID, EnumDetailsKeyword::default_schema_resource())
        .expect("valid URI")
        .add(EnumSortTextsKeyword::KEYWORD_ID, EnumSortTextsKeyword::default_schema_resource())
        .expect("valid URI")
        .add(ErrorMessageKeyword::KEYWORD_ID, ErrorMessageKeyword::default_schema_resource())
        .expect("valid URI")
        .add(MarkdownDescriptionKeyword::KEYWORD_ID, MarkdownDescriptionKeyword::default_schema_resource())
        .expect("valid URI")
        .add(MarkdownEnumDescriptionsKeyword::KEYWORD_ID, MarkdownEnumDescriptionsKeyword::default_schema_resource())
        .expect("valid URI")
        .add(PatternErrorMessageKeyword::KEYWORD_ID, PatternErrorMessageKeyword::default_schema_resource())
        .expect("valid URI")
        .add(SuggestSortTextKeyword::KEYWORD_ID, SuggestSortTextKeyword::default_schema_resource())
        .expect("valid URI")
        .add(VSCodeVocabulary::SCHEMA_ID, VSCodeVocabulary::default_schema_resource())
        .expect("valid URI")
        .add(VSCodeDialect::SCHEMA_ID, VSCodeDialect::default_schema_resource())
        .expect("valid URI")
        .prepare()
        .expect("valid registry")
});

/// Defines extension methods to the [`jsonschema::ValidationOptions`] to simplify registering the
/// VS Code [keywords], [vocabulary], and [dialect meta schema].
/// 
/// [keywords]: VSCodeKeyword
/// [vocabulary]: VSCodeVocabulary
/// [dialect meta schema]: VSCodeDialect
pub trait VSCodeValidationOptionsExtensions<'i> {
    /// Registers a single VS Code keyword for use with a [`jsonschema::Validator`].
    /// 
    /// This function registers a specific VS Code keyword with the [`with_keyword()`] builder
    /// method and adds the [`VSCODE_DIALECT_REGISTRY`] containing all schema resources.
    /// 
    /// [`with_keyword()`]: ValidationOptions::with_keyword
    /// [`with_vscode_dialect()`]: VSCodeValidationOptionsExtensions::with_vscode_dialect
    /// [`with_vscode_vocabulary()`]: VSCodeValidationOptionsExtensions::with_vscode_vocabulary
    fn with_vscode_keyword(self, keyword: VSCodeKeyword) -> ValidationOptions<'i>;
    /// Registers every VS Code keyword for use with a [`jsonschema::Validator`].
    /// 
    /// This function registers each of the VS Code keywords with the [`with_keyword()`] builder
    /// method and adds the [`VSCODE_DIALECT_REGISTRY`] containing all schema resources.
    /// 
    /// Use this function when you only want to register the VS Code vocabulary keywords.
    /// If you are using the VS Code vocabulary in your own meta schema dialect, use the
    /// [`with_vscode_vocabulary()`] method instead. If you are using the VS Code meta schema
    /// dialect directly, use the [`with_vscode_dialect()`] method instead.
    /// 
    /// [`with_keyword()`]: ValidationOptions::with_keyword
    /// [`with_vscode_dialect()`]: VSCodeValidationOptionsExtensions::with_vscode_dialect
    /// [`with_vscode_vocabulary()`]: VSCodeValidationOptionsExtensions::with_vscode_vocabulary
    fn with_vscode_keywords(self) -> ValidationOptions<'i>;
    /// Registers the VS Code completion keywords for use with a [`jsonschema::Validator`].
    /// 
    /// This function registers the following VS Code keywords with the [`with_keyword()`] builder
    /// method and adds the [`VSCODE_DIALECT_REGISTRY`] containing all schema resources:
    /// 
    /// - [`completionDetail`]
    /// - [`defaultSnippets`]
    /// - [`doNotSuggest`]
    /// - [`enumDetails`]
    /// - [`enumSortTexts`]
    /// - [`suggestSortText`]
    /// 
    /// [`completionDetail`]: super::keywords::CompletionDetailKeyword
    /// [`defaultSnippets`]: super::keywords::DefaultSnippetsKeyword
    /// [`doNotSuggest`]: super::keywords::DoNotSuggestKeyword
    /// [`enumDetails`]: super::keywords::EnumDetailsKeyword
    /// [`enumSortTexts`]: super::keywords::EnumSortTextsKeyword
    /// [`suggestSortText`]: super::keywords::SuggestSortTextKeyword
    /// [`with_keyword()`]: ValidationOptions::with_keyword
    /// [`with_vscode_dialect()`]: VSCodeValidationOptionsExtensions::with_vscode_dialect
    /// [`with_vscode_vocabulary()`]: VSCodeValidationOptionsExtensions::with_vscode_vocabulary
    fn with_vscode_completion_keywords(self) -> ValidationOptions<'i>;
    /// Registers the VS Code documentation keywords for use with a [`jsonschema::Validator`].
    /// 
    /// This function registers the following VS Code keywords with the [`with_keyword()`] builder
    /// method and adds the [`VSCODE_DIALECT_REGISTRY`] containing all schema resources:
    /// 
    /// - [`deprecationMessageKeyword`]
    /// - [`enumDescriptionsKeyword`]
    /// - [`markdownDescriptionKeyword`]
    /// - [`markdownEnumDescriptionsKeyword`]
    /// 
    /// [`deprecationMessageKeyword`]: super::keywords::DeprecationMessageKeyword
    /// [`enumDescriptionsKeyword`]: super::keywords::EnumDescriptionsKeyword
    /// [`markdownDescriptionKeyword`]: super::keywords::MarkdownDescriptionKeyword
    /// [`markdownEnumDescriptionsKeyword`]: super::keywords::MarkdownEnumDescriptionsKeyword
    /// [`with_keyword()`]: ValidationOptions::with_keyword
    /// [`with_vscode_dialect()`]: VSCodeValidationOptionsExtensions::with_vscode_dialect
    /// [`with_vscode_vocabulary()`]: VSCodeValidationOptionsExtensions::with_vscode_vocabulary
    fn with_vscode_documentation_keywords(self) -> ValidationOptions<'i>;
    /// Registers the VS Code error messaging keywords for use with a [`jsonschema::Validator`].
    /// 
    /// This function registers the following VS Code keywords with the [`with_keyword()`] builder
    /// method and adds the [`VSCODE_DIALECT_REGISTRY`] containing all schema resources:
    /// 
    /// - [`errorMessageKeyword`]
    /// - [`patternErrorMessageKeyword`]
    /// 
    /// [`errorMessageKeyword`]: super::keywords::ErrorMessageKeyword
    /// [`patternErrorMessageKeyword`]: super::keywords::PatternErrorMessageKeyword
    /// [`with_keyword()`]: ValidationOptions::with_keyword
    /// [`with_vscode_dialect()`]: VSCodeValidationOptionsExtensions::with_vscode_dialect
    /// [`with_vscode_vocabulary()`]: VSCodeValidationOptionsExtensions::with_vscode_vocabulary
    fn with_vscode_error_keywords(self) -> ValidationOptions<'i>;
    /// Registers the VS Code parsing rules keywords for use with a [`jsonschema::Validator`].
    /// 
    /// This function registers the following VS Code keywords with the [`with_keyword()`] builder
    /// method and adds the [`VSCODE_DIALECT_REGISTRY`] containing all schema resources:
    /// 
    /// - [`allowCommentsKeyword`]
    /// - [`allowTrailingCommasKeyword`]
    /// 
    /// [`allowCommentsKeyword`]: super::keywords::AllowCommentsKeyword
    /// [`allowTrailingCommasKeyword`]: super::keywords::AllowTrailingCommasKeyword
    /// [`with_keyword()`]: ValidationOptions::with_keyword
    /// [`with_vscode_dialect()`]: VSCodeValidationOptionsExtensions::with_vscode_dialect
    /// [`with_vscode_vocabulary()`]: VSCodeValidationOptionsExtensions::with_vscode_vocabulary
    fn with_vscode_parsing_keywords(self) -> ValidationOptions<'i>;
    /// Registers the VS Code vocabulary and keywords for use with a [`jsonschema::Validator`].
    /// 
    /// This function registers each of the VS Code keywords with the [`with_keyword()`] builder
    /// method and adds the [`VSCODE_DIALECT_REGISTRY`] containing all schema resources including
    /// the vocabulary schema.
    /// 
    /// This is a convenience method for registering the vocabulary and keywords. You don't need to
    /// separately add the keywords or schema resources. Use this convenience method when you are
    /// defining your own meta schema dialect that uses the VS Code vocabulary.
    /// 
    /// If you are using the VS Code meta schema directly without extending the dialect for your
    /// own purposes, use the [`with_vscode_dialect()`] method instead.
    /// 
    /// [`with_keyword()`]: ValidationOptions::with_keyword
    /// [vocabulary schema]: super::vocabulary::VSCODE_VOCABULARY_SCHEMA_CANONICAL
    /// [`with_vscode_dialect()`]: VSCodeValidationOptionsExtensions::with_vscode_dialect
    fn with_vscode_vocabulary(self) -> ValidationOptions<'i>;
    /// Registers the VS Code dialect meta schema, vocabulary, and keywords for use with a
    /// [`jsonschema::Validator`].
    /// 
    /// This function registers each of the VS Code keywords with the [`with_keyword()`] builder
    /// method and adds the [`VSCODE_DIALECT_REGISTRY`] containing all schema resources including
    /// the vocabulary schema and dialect meta schema.
    /// 
    /// This is a convenience method for registering the meta schema, vocabulary, and keywords
    /// together. You don't need to separately add the keywords or schema resources. Use this
    /// convenience method when you are using the VS Code meta schema dialect and vocabulary.
    /// 
    /// If you're using your own dialect that includes the VS Code vocabulary, use the
    /// [`with_vscode_vocabulary()`] method instead.
    /// 
    /// [`with_keyword()`]: ValidationOptions::with_keyword
    /// [vocabulary schema]: super::vocabulary::VSCODE_VOCABULARY_SCHEMA_CANONICAL
    /// [dialect meta schema]: super::dialect::VSCODE_DIALECT_SCHEMA_CANONICAL
    /// [`with_vscode_vocabulary()`]: VSCodeValidationOptionsExtensions::with_vscode_vocabulary
    fn with_vscode_dialect(self) -> ValidationOptions<'i>;
}

impl<'i> VSCodeValidationOptionsExtensions<'i> for ValidationOptions<'i> {
    fn with_vscode_keyword(self, keyword: VSCodeKeyword) -> ValidationOptions<'i> {
        keyword.register(self)
            .with_registry(&VSCODE_DIALECT_REGISTRY)
    }
    fn with_vscode_keywords(self) -> ValidationOptions<'i> {
        self
            .with_registry(&VSCODE_DIALECT_REGISTRY)
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
    fn with_vscode_completion_keywords(self) -> ValidationOptions<'i> {
        self
            .with_registry(&VSCODE_DIALECT_REGISTRY)
            .with_vscode_keyword(VSCodeKeyword::CompletionDetail)
            .with_vscode_keyword(VSCodeKeyword::DefaultSnippets)
            .with_vscode_keyword(VSCodeKeyword::DoNotSuggest)
            .with_vscode_keyword(VSCodeKeyword::EnumDetails)
            .with_vscode_keyword(VSCodeKeyword::EnumSortTexts)
            .with_vscode_keyword(VSCodeKeyword::SuggestSortText)
    }
    fn with_vscode_documentation_keywords(self) -> ValidationOptions<'i> {
        self
            .with_registry(&VSCODE_DIALECT_REGISTRY)
            .with_vscode_keyword(VSCodeKeyword::DeprecationMessage)
            .with_vscode_keyword(VSCodeKeyword::EnumDescriptions)
            .with_vscode_keyword(VSCodeKeyword::MarkdownDescription)
            .with_vscode_keyword(VSCodeKeyword::MarkdownEnumDescriptions)
    }
    fn with_vscode_error_keywords(self) -> ValidationOptions<'i> {
        self
            .with_registry(&VSCODE_DIALECT_REGISTRY)
            .with_vscode_keyword(VSCodeKeyword::ErrorMessage)
            .with_vscode_keyword(VSCodeKeyword::PatternErrorMessage)
    }
    fn with_vscode_parsing_keywords(self) -> ValidationOptions<'i> {
        self
            .with_registry(&VSCODE_DIALECT_REGISTRY)
            .with_vscode_keyword(VSCodeKeyword::AllowComments)
            .with_vscode_keyword(VSCodeKeyword::AllowTrailingCommas)
    }
    fn with_vscode_vocabulary(self) -> ValidationOptions<'i> {
        self.with_vscode_keywords()
    }
    fn with_vscode_dialect(self) -> ValidationOptions<'i> {
        self.with_vscode_keywords()
    }
}
