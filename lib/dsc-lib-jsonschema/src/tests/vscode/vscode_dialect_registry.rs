// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[allow(non_snake_case)]
#[cfg(test)] mod VSCODE_DIALECT_REGISTRY {
    use crate::vscode::{
        VSCODE_DIALECT_REGISTRY,
        dialect::VSCodeDialect,
        vocabulary::VSCodeVocabulary,
        keywords::{
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

    #[test] fn initializes_without_panic() {
        let _ = &*VSCODE_DIALECT_REGISTRY;
    }

    #[test] fn contains_dialect_schema() {
        assert!(
            VSCODE_DIALECT_REGISTRY.contains_resource(VSCodeDialect::SCHEMA_ID),
            "Registry should contain the dialect meta schema"
        );
    }

    #[test] fn contains_vocabulary_schema() {
        assert!(
            VSCODE_DIALECT_REGISTRY.contains_resource(VSCodeVocabulary::SCHEMA_ID),
            "Registry should contain the vocabulary schema"
        );
    }

    #[test] fn contains_allow_comments_keyword_schema() {
        assert!(VSCODE_DIALECT_REGISTRY.contains_resource(AllowCommentsKeyword::KEYWORD_ID));
    }

    #[test] fn contains_allow_trailing_commas_keyword_schema() {
        assert!(VSCODE_DIALECT_REGISTRY.contains_resource(AllowTrailingCommasKeyword::KEYWORD_ID));
    }

    #[test] fn contains_completion_detail_keyword_schema() {
        assert!(VSCODE_DIALECT_REGISTRY.contains_resource(CompletionDetailKeyword::KEYWORD_ID));
    }

    #[test] fn contains_default_snippets_keyword_schema() {
        assert!(VSCODE_DIALECT_REGISTRY.contains_resource(DefaultSnippetsKeyword::KEYWORD_ID));
    }

    #[test] fn contains_deprecation_message_keyword_schema() {
        assert!(VSCODE_DIALECT_REGISTRY.contains_resource(DeprecationMessageKeyword::KEYWORD_ID));
    }

    #[test] fn contains_do_not_suggest_keyword_schema() {
        assert!(VSCODE_DIALECT_REGISTRY.contains_resource(DoNotSuggestKeyword::KEYWORD_ID));
    }

    #[test] fn contains_enum_descriptions_keyword_schema() {
        assert!(VSCODE_DIALECT_REGISTRY.contains_resource(EnumDescriptionsKeyword::KEYWORD_ID));
    }

    #[test] fn contains_enum_details_keyword_schema() {
        assert!(VSCODE_DIALECT_REGISTRY.contains_resource(EnumDetailsKeyword::KEYWORD_ID));
    }

    #[test] fn contains_enum_sort_texts_keyword_schema() {
        assert!(VSCODE_DIALECT_REGISTRY.contains_resource(EnumSortTextsKeyword::KEYWORD_ID));
    }

    #[test] fn contains_error_message_keyword_schema() {
        assert!(VSCODE_DIALECT_REGISTRY.contains_resource(ErrorMessageKeyword::KEYWORD_ID));
    }

    #[test] fn contains_markdown_description_keyword_schema() {
        assert!(VSCODE_DIALECT_REGISTRY.contains_resource(MarkdownDescriptionKeyword::KEYWORD_ID));
    }

    #[test] fn contains_markdown_enum_descriptions_keyword_schema() {
        assert!(VSCODE_DIALECT_REGISTRY.contains_resource(MarkdownEnumDescriptionsKeyword::KEYWORD_ID));
    }

    #[test] fn contains_pattern_error_message_keyword_schema() {
        assert!(VSCODE_DIALECT_REGISTRY.contains_resource(PatternErrorMessageKeyword::KEYWORD_ID));
    }

    #[test] fn contains_suggest_sort_text_keyword_schema() {
        assert!(VSCODE_DIALECT_REGISTRY.contains_resource(SuggestSortTextKeyword::KEYWORD_ID));
    }
}

#[allow(non_snake_case)]
#[cfg(test)] mod VSCodeKeyword_register {
    use jsonschema::Validator;
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use crate::vscode::{
        VSCODE_DIALECT_REGISTRY,
        keywords::{VSCodeKeyword, VSCodeKeywordDefinition, AllowCommentsKeyword},
    };

    #[test] fn registers_keyword_factory_without_registry() {
        // register() alone adds the keyword factory but not the registry,
        // so a schema using the keyword name directly (not $ref) should work
        let result = Validator::options()
            .with_keyword(
                AllowCommentsKeyword::KEYWORD_NAME,
                AllowCommentsKeyword::keyword_factory
            )
            .build(&json!({
                "allowComments": true
            }));

        assert!(result.is_ok());
    }

    #[test] fn keyword_validates_with_register_and_registry() {
        // register() + with_registry() together should allow $ref-based schemas
        let validator = VSCodeKeyword::AllowComments.register(
            Validator::options()
        )
        .with_registry(&VSCODE_DIALECT_REGISTRY)
        .build(&json!({
            "$ref": AllowCommentsKeyword::KEYWORD_ID
        })).unwrap();

        assert_eq!(validator.is_valid(&json!("invalid")), false);
        assert_eq!(validator.is_valid(&json!(true)), true);
    }
}
