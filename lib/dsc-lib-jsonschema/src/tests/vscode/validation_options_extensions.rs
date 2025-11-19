// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[cfg(test)] mod with_vscode_keyword {
    use jsonschema::Validator;
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use crate::vscode::{
        VSCodeValidationOptionsExtensions,
        keywords::{AllowCommentsKeyword, VSCodeKeyword, VSCodeKeywordDefinition}
    };

    #[test] fn adds_the_keyword_and_schema() {
        let validator = Validator::options()
            .with_vscode_keyword(VSCodeKeyword::AllowComments)
            .build(&json!({
                "$ref": AllowCommentsKeyword::KEYWORD_ID
            })).unwrap();

        let invalid_instance = &json!("invalid");
        let valid_instance = &json!(true);

        assert_eq!(validator.is_valid(invalid_instance), false);
        assert_eq!(validator.is_valid(valid_instance), true);
    }
}
#[cfg(test)] mod with_vscode_keywords {
    use jsonschema::Validator;
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use crate::vscode::{
        VSCodeValidationOptionsExtensions,
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
            SuggestSortTextKeyword
        }
    };

    #[test] fn adds_the_allow_comments_keyword() {
        let validator = Validator::options()
            .with_vscode_keywords()
            .build(&json!({
                "$ref": AllowCommentsKeyword::KEYWORD_ID
            })).unwrap();

        let invalid_instance = &json!("invalid");
        let valid_instance = &json!(true);

        assert_eq!(validator.is_valid(invalid_instance), false);
        assert_eq!(validator.is_valid(valid_instance), true);
    }
    #[test] fn adds_the_allow_trailing_commas_keyword() {
        let validator = Validator::options()
            .with_vscode_keywords()
            .build(&json!({
                "$ref": AllowTrailingCommasKeyword::KEYWORD_ID
            })).unwrap();

        let invalid_instance = &json!("invalid");
        let valid_instance = &json!(true);

        assert_eq!(validator.is_valid(invalid_instance), false);
        assert_eq!(validator.is_valid(valid_instance), true);
    }
    #[test] fn adds_the_completion_detail_keyword() {
        let validator = Validator::options()
            .with_vscode_keywords()
            .build(&json!({
                "$ref": CompletionDetailKeyword::KEYWORD_ID
            })).unwrap();

        let invalid_instance = &json!(false);
        let valid_instance = &json!("valid");

        assert_eq!(validator.is_valid(invalid_instance), false);
        assert_eq!(validator.is_valid(valid_instance), true);
    }
    #[test] fn adds_the_default_snippets_keyword() {
        let validator = Validator::options()
            .with_vscode_keywords()
            .build(&json!({
                "$ref": DefaultSnippetsKeyword::KEYWORD_ID
            })).unwrap();

        let invalid_instance = &json!(false);
        let valid_instance = &json!([{"label": "valid"}]);
        assert_eq!(validator.is_valid(invalid_instance), false);
        assert_eq!(validator.is_valid(valid_instance), true);
    }
    #[test] fn adds_the_deprecation_message_keyword() {
        let validator = Validator::options()
            .with_vscode_keywords()
            .build(&json!({
                "$ref": DeprecationMessageKeyword::KEYWORD_ID
            })).unwrap();

        let invalid_instance = &json!(false);
        let valid_instance = &json!("valid");

        assert_eq!(validator.is_valid(invalid_instance), false);
        assert_eq!(validator.is_valid(valid_instance), true);
    }
    #[test] fn adds_the_do_not_suggest_keyword() {
        let validator = Validator::options()
            .with_vscode_keywords()
            .build(&json!({
                "$ref": DoNotSuggestKeyword::KEYWORD_ID
            })).unwrap();

        let invalid_instance = &json!("invalid");
        let valid_instance = &json!(true);

        assert_eq!(validator.is_valid(invalid_instance), false);
        assert_eq!(validator.is_valid(valid_instance), true);
    }
    #[test] fn adds_the_enum_descriptions_keyword() {
        let validator = Validator::options()
            .with_vscode_keywords()
            .build(&json!({
                "$ref": EnumDescriptionsKeyword::KEYWORD_ID
            })).unwrap();

        let invalid_instance = &json!("invalid");
        let valid_instance = &json!(["valid"]);

        assert_eq!(validator.is_valid(invalid_instance), false);
        assert_eq!(validator.is_valid(valid_instance), true);
    }
    #[test] fn adds_the_enum_details_keyword() {
        let validator = Validator::options()
            .with_vscode_keywords()
            .build(&json!({
                "$ref": EnumDetailsKeyword::KEYWORD_ID
            })).unwrap();

        let invalid_instance = &json!("invalid");
        let valid_instance = &json!(["valid"]);

        assert_eq!(validator.is_valid(invalid_instance), false);
        assert_eq!(validator.is_valid(valid_instance), true);
    }
    #[test] fn adds_the_enum_sort_texts_keyword() {
        let validator = Validator::options()
            .with_vscode_keywords()
            .build(&json!({
                "$ref": EnumSortTextsKeyword::KEYWORD_ID
            })).unwrap();

        let invalid_instance = &json!("invalid");
        let valid_instance = &json!(["valid"]);

        assert_eq!(validator.is_valid(invalid_instance), false);
        assert_eq!(validator.is_valid(valid_instance), true);
    }
    #[test] fn adds_the_error_message_keyword() {
        let validator = Validator::options()
            .with_vscode_keywords()
            .build(&json!({
                "$ref": ErrorMessageKeyword::KEYWORD_ID
            })).unwrap();

        let invalid_instance = &json!(false);
        let valid_instance = &json!("valid");

        assert_eq!(validator.is_valid(invalid_instance), false);
        assert_eq!(validator.is_valid(valid_instance), true);
    }
    #[test] fn adds_the_markdown_description_keyword() {
        let validator = Validator::options()
            .with_vscode_keywords()
            .build(&json!({
                "$ref": MarkdownDescriptionKeyword::KEYWORD_ID
            })).unwrap();

        let invalid_instance = &json!(false);
        let valid_instance = &json!("valid");

        assert_eq!(validator.is_valid(invalid_instance), false);
        assert_eq!(validator.is_valid(valid_instance), true);
    }
    #[test] fn adds_the_markdown_enum_descriptions_keyword() {
        let validator = Validator::options()
            .with_vscode_keywords()
            .build(&json!({
                "$ref": MarkdownEnumDescriptionsKeyword::KEYWORD_ID
            })).unwrap();

        let invalid_instance = &json!("invalid");
        let valid_instance = &json!(["valid"]);

        assert_eq!(validator.is_valid(invalid_instance), false);
        assert_eq!(validator.is_valid(valid_instance), true);
    }
    #[test] fn adds_the_pattern_error_message_keyword() {
        let validator = Validator::options()
            .with_vscode_keywords()
            .build(&json!({
                "$ref": PatternErrorMessageKeyword::KEYWORD_ID
            })).unwrap();

        let invalid_instance = &json!(false);
        let valid_instance = &json!("valid");

        assert_eq!(validator.is_valid(invalid_instance), false);
        assert_eq!(validator.is_valid(valid_instance), true);
    }
    #[test] fn adds_the_suggest_sort_text_keyword() {
        let validator = Validator::options()
            .with_vscode_keywords()
            .build(&json!({
                "$ref": SuggestSortTextKeyword::KEYWORD_ID
            })).unwrap();

        let invalid_instance = &json!(false);
        let valid_instance = &json!("valid");

        assert_eq!(validator.is_valid(invalid_instance), false);
        assert_eq!(validator.is_valid(valid_instance), true);
    }
}

#[cfg(test)] mod with_vscode_completion_keywords {
    use jsonschema::Validator;
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use crate::vscode::{
        VSCodeValidationOptionsExtensions,
        keywords::{
            CompletionDetailKeyword,
            DefaultSnippetsKeyword,
            DoNotSuggestKeyword,
            EnumDetailsKeyword,
            EnumSortTextsKeyword,
            SuggestSortTextKeyword,
            VSCodeKeywordDefinition
        }
    };

    #[test] fn adds_the_completion_detail_keyword() {
        let validator = Validator::options()
            .with_vscode_completion_keywords()
            .build(&json!({
                "$ref": CompletionDetailKeyword::KEYWORD_ID
            })).unwrap();

        let invalid_instance = &json!(false);
        let valid_instance = &json!("valid");

        assert_eq!(validator.is_valid(invalid_instance), false);
        assert_eq!(validator.is_valid(valid_instance), true);
    }
    #[test] fn adds_the_default_snippets_keyword() {
        let validator = Validator::options()
            .with_vscode_completion_keywords()
            .build(&json!({
                "$ref": DefaultSnippetsKeyword::KEYWORD_ID
            })).unwrap();

        let invalid_instance = &json!(false);
        let valid_instance = &json!([{"label": "valid"}]);

        assert_eq!(validator.is_valid(invalid_instance), false);
        assert_eq!(validator.is_valid(valid_instance), true);
    }
    #[test] fn adds_the_do_not_suggest_keyword() {
        let validator = Validator::options()
            .with_vscode_completion_keywords()
            .build(&json!({
                "$ref": DoNotSuggestKeyword::KEYWORD_ID
            })).unwrap();

        let invalid_instance = &json!("invalid");
        let valid_instance = &json!(true);

        assert_eq!(validator.is_valid(invalid_instance), false);
        assert_eq!(validator.is_valid(valid_instance), true);
    }
    #[test] fn adds_the_enum_details_keyword() {
        let validator = Validator::options()
            .with_vscode_completion_keywords()
            .build(&json!({
                "$ref": EnumDetailsKeyword::KEYWORD_ID
            })).unwrap();

        let invalid_instance = &json!("invalid");
        let valid_instance = &json!(["valid"]);

        assert_eq!(validator.is_valid(invalid_instance), false);
        assert_eq!(validator.is_valid(valid_instance), true);
    }
    #[test] fn adds_the_enum_sort_texts_keyword() {
        let validator = Validator::options()
            .with_vscode_completion_keywords()
            .build(&json!({
                "$ref": EnumSortTextsKeyword::KEYWORD_ID
            })).unwrap();

        let invalid_instance = &json!("invalid");
        let valid_instance = &json!(["valid"]);

        assert_eq!(validator.is_valid(invalid_instance), false);
        assert_eq!(validator.is_valid(valid_instance), true);
    }
    #[test] fn adds_the_suggest_sort_text_keyword() {
        let validator = Validator::options()
            .with_vscode_completion_keywords()
            .build(&json!({
                "$ref": SuggestSortTextKeyword::KEYWORD_ID
            })).unwrap();

        let invalid_instance = &json!(false);
        let valid_instance = &json!("valid");

        assert_eq!(validator.is_valid(invalid_instance), false);
        assert_eq!(validator.is_valid(valid_instance), true);
    }
}

#[cfg(test)] mod with_vscode_documentation_keywords {
    use jsonschema::Validator;
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use crate::vscode::{
        VSCodeValidationOptionsExtensions,
        keywords::{
            DeprecationMessageKeyword,
            EnumDescriptionsKeyword,
            MarkdownDescriptionKeyword,
            MarkdownEnumDescriptionsKeyword,
            VSCodeKeywordDefinition
        }
    };

    #[test] fn adds_the_deprecation_message_keyword() {
        let validator = Validator::options()
            .with_vscode_documentation_keywords()
            .build(&json!({
                "$ref": DeprecationMessageKeyword::KEYWORD_ID
            })).unwrap();

        let invalid_instance = &json!(false);
        let valid_instance = &json!("valid");

        assert_eq!(validator.is_valid(invalid_instance), false);
        assert_eq!(validator.is_valid(valid_instance), true);
    }
    #[test] fn adds_the_enum_descriptions_keyword() {
        let validator = Validator::options()
            .with_vscode_documentation_keywords()
            .build(&json!({
                "$ref": EnumDescriptionsKeyword::KEYWORD_ID
            })).unwrap();

        let invalid_instance = &json!("invalid");
        let valid_instance = &json!(["valid"]);

        assert_eq!(validator.is_valid(invalid_instance), false);
        assert_eq!(validator.is_valid(valid_instance), true);
    }
    #[test] fn adds_the_markdown_description_keyword() {
        let validator = Validator::options()
            .with_vscode_documentation_keywords()
            .build(&json!({
                "$ref": MarkdownDescriptionKeyword::KEYWORD_ID
            })).unwrap();

        let invalid_instance = &json!(false);
        let valid_instance = &json!("valid");

        assert_eq!(validator.is_valid(invalid_instance), false);
        assert_eq!(validator.is_valid(valid_instance), true);
    }
    #[test] fn adds_the_markdown_enum_descriptions_keyword() {
        let validator = Validator::options()
            .with_vscode_documentation_keywords()
            .build(&json!({
                "$ref": MarkdownEnumDescriptionsKeyword::KEYWORD_ID
            })).unwrap();

        let invalid_instance = &json!("invalid");
        let valid_instance = &json!(["valid"]);

        assert_eq!(validator.is_valid(invalid_instance), false);
        assert_eq!(validator.is_valid(valid_instance), true);
    }
}

#[cfg(test)] mod with_vscode_error_keywords {
    use jsonschema::Validator;
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use crate::vscode::{
        VSCodeValidationOptionsExtensions,
        keywords::{
            ErrorMessageKeyword,
            PatternErrorMessageKeyword,
            VSCodeKeywordDefinition
        }
    };

    #[test] fn adds_the_error_message_keyword() {
        let validator = Validator::options()
            .with_vscode_error_keywords()
            .build(&json!({
                "$ref": ErrorMessageKeyword::KEYWORD_ID
            })).unwrap();

        let invalid_instance = &json!(false);
        let valid_instance = &json!("valid");

        assert_eq!(validator.is_valid(invalid_instance), false);
        assert_eq!(validator.is_valid(valid_instance), true);
    }
    #[test] fn adds_the_pattern_error_message_keyword() {
        let validator = Validator::options()
            .with_vscode_error_keywords()
            .build(&json!({
                "$ref": PatternErrorMessageKeyword::KEYWORD_ID
            })).unwrap();

        let invalid_instance = &json!(false);
        let valid_instance = &json!("valid");

        assert_eq!(validator.is_valid(invalid_instance), false);
        assert_eq!(validator.is_valid(valid_instance), true);
    }
}

#[cfg(test)] mod with_vscode_parsing_keywords {
    use jsonschema::Validator;
    use serde_json::json;

    use crate::vscode::{
        VSCodeValidationOptionsExtensions,
        keywords::{
            AllowCommentsKeyword,
            AllowTrailingCommasKeyword,
            VSCodeKeywordDefinition
        }
    };

    #[test] fn adds_the_allow_comments_keyword() {
        let validator = Validator::options()
            .with_vscode_parsing_keywords()
            .build(&json!({
                "$ref": AllowCommentsKeyword::KEYWORD_ID
            })).unwrap();

        let invalid_instance = &json!("invalid");
        let valid_instance = &json!(true);

        assert_eq!(validator.is_valid(invalid_instance), false);
        assert_eq!(validator.is_valid(valid_instance), true);
    }
    #[test] fn adds_the_allow_trailing_commas_keyword() {
        let validator = Validator::options()
            .with_vscode_parsing_keywords()
            .build(&json!({
                "$ref": AllowTrailingCommasKeyword::KEYWORD_ID
            })).unwrap();

        let invalid_instance = &json!("invalid");
        let valid_instance = &json!(true);

        assert_eq!(validator.is_valid(invalid_instance), false);
        assert_eq!(validator.is_valid(valid_instance), true);
    }
}

#[cfg(test)] mod with_vscode_vocabulary {
    use jsonschema::Validator;
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use crate::vscode::{
        VSCodeValidationOptionsExtensions,
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
            SuggestSortTextKeyword
        }
    };

    #[test] fn adds_the_allow_comments_keyword() {
        let validator = Validator::options()
            .with_vscode_vocabulary()
            .build(&json!({
                "$ref": AllowCommentsKeyword::KEYWORD_ID
            })).unwrap();

        let invalid_instance = &json!("invalid");
        let valid_instance = &json!(true);

        assert_eq!(validator.is_valid(invalid_instance), false);
        assert_eq!(validator.is_valid(valid_instance), true);
    }
    #[test] fn adds_the_allow_trailing_commas_keyword() {
        let validator = Validator::options()
            .with_vscode_vocabulary()
            .build(&json!({
                "$ref": AllowTrailingCommasKeyword::KEYWORD_ID
            })).unwrap();

        let invalid_instance = &json!("invalid");
        let valid_instance = &json!(true);

        assert_eq!(validator.is_valid(invalid_instance), false);
        assert_eq!(validator.is_valid(valid_instance), true);
    }
    #[test] fn adds_the_completion_detail_keyword() {
        let validator = Validator::options()
            .with_vscode_vocabulary()
            .build(&json!({
                "$ref": CompletionDetailKeyword::KEYWORD_ID
            })).unwrap();

        let invalid_instance = &json!(false);
        let valid_instance = &json!("valid");

        assert_eq!(validator.is_valid(invalid_instance), false);
        assert_eq!(validator.is_valid(valid_instance), true);
    }
    #[test] fn adds_the_default_snippets_keyword() {
        let validator = Validator::options()
            .with_vscode_vocabulary()
            .build(&json!({
                "$ref": DefaultSnippetsKeyword::KEYWORD_ID
            })).unwrap();

        let invalid_instance = &json!(false);
        let valid_instance = &json!([{"label": "valid"}]);

        assert_eq!(validator.is_valid(invalid_instance), false);
        assert_eq!(validator.is_valid(valid_instance), true);
    }
    #[test] fn adds_the_deprecation_message_keyword() {
        let validator = Validator::options()
            .with_vscode_vocabulary()
            .build(&json!({
                "$ref": DeprecationMessageKeyword::KEYWORD_ID
            })).unwrap();

        let invalid_instance = &json!(false);
        let valid_instance = &json!("valid");

        assert_eq!(validator.is_valid(invalid_instance), false);
        assert_eq!(validator.is_valid(valid_instance), true);
    }
    #[test] fn adds_the_do_not_suggest_keyword() {
        let validator = Validator::options()
            .with_vscode_vocabulary()
            .build(&json!({
                "$ref": DoNotSuggestKeyword::KEYWORD_ID
            })).unwrap();

        let invalid_instance = &json!("invalid");
        let valid_instance = &json!(true);

        assert_eq!(validator.is_valid(invalid_instance), false);
        assert_eq!(validator.is_valid(valid_instance), true);
    }
    #[test] fn adds_the_enum_descriptions_keyword() {
        let validator = Validator::options()
            .with_vscode_vocabulary()
            .build(&json!({
                "$ref": EnumDescriptionsKeyword::KEYWORD_ID
            })).unwrap();

        let invalid_instance = &json!("invalid");
        let valid_instance = &json!(["valid"]);

        assert_eq!(validator.is_valid(invalid_instance), false);
        assert_eq!(validator.is_valid(valid_instance), true);
    }
    #[test] fn adds_the_enum_details_keyword() {
        let validator = Validator::options()
            .with_vscode_vocabulary()
            .build(&json!({
                "$ref": EnumDetailsKeyword::KEYWORD_ID
            })).unwrap();

        let invalid_instance = &json!("invalid");
        let valid_instance = &json!(["valid"]);

        assert_eq!(validator.is_valid(invalid_instance), false);
        assert_eq!(validator.is_valid(valid_instance), true);
    }
    #[test] fn adds_the_enum_sort_texts_keyword() {
        let validator = Validator::options()
            .with_vscode_vocabulary()
            .build(&json!({
                "$ref": EnumSortTextsKeyword::KEYWORD_ID
            })).unwrap();

        let invalid_instance = &json!("invalid");
        let valid_instance = &json!(["valid"]);

        assert_eq!(validator.is_valid(invalid_instance), false);
        assert_eq!(validator.is_valid(valid_instance), true);
    }
    #[test] fn adds_the_error_message_keyword() {
        let validator = Validator::options()
            .with_vscode_vocabulary()
            .build(&json!({
                "$ref": ErrorMessageKeyword::KEYWORD_ID
            })).unwrap();

        let invalid_instance = &json!(false);
        let valid_instance = &json!("valid");

        assert_eq!(validator.is_valid(invalid_instance), false);
        assert_eq!(validator.is_valid(valid_instance), true);
    }
    #[test] fn adds_the_markdown_description_keyword() {
        let validator = Validator::options()
            .with_vscode_vocabulary()
            .build(&json!({
                "$ref": MarkdownDescriptionKeyword::KEYWORD_ID
            })).unwrap();

        let invalid_instance = &json!(false);
        let valid_instance = &json!("valid");

        assert_eq!(validator.is_valid(invalid_instance), false);
        assert_eq!(validator.is_valid(valid_instance), true);
    }
    #[test] fn adds_the_markdown_enum_descriptions_keyword() {
        let validator = Validator::options()
            .with_vscode_vocabulary()
            .build(&json!({
                "$ref": MarkdownEnumDescriptionsKeyword::KEYWORD_ID
            })).unwrap();

        let invalid_instance = &json!("invalid");
        let valid_instance = &json!(["valid"]);

        assert_eq!(validator.is_valid(invalid_instance), false);
        assert_eq!(validator.is_valid(valid_instance), true);
    }
    #[test] fn adds_the_pattern_error_message_keyword() {
        let validator = Validator::options()
            .with_vscode_vocabulary()
            .build(&json!({
                "$ref": PatternErrorMessageKeyword::KEYWORD_ID
            })).unwrap();

        let invalid_instance = &json!(false);
        let valid_instance = &json!("valid");

        assert_eq!(validator.is_valid(invalid_instance), false);
        assert_eq!(validator.is_valid(valid_instance), true);
    }
    #[test] fn adds_the_suggest_sort_text_keyword() {
        let validator = Validator::options()
            .with_vscode_vocabulary()
            .build(&json!({
                "$ref": SuggestSortTextKeyword::KEYWORD_ID
            })).unwrap();

        let invalid_instance = &json!(false);
        let valid_instance = &json!("valid");

        assert_eq!(validator.is_valid(invalid_instance), false);
        assert_eq!(validator.is_valid(valid_instance), true);
    }
}

#[cfg(test)] mod with_vscode_dialect {
    use jsonschema::Validator;
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use crate::vscode::{
        VSCodeValidationOptionsExtensions,
        dialect::VSCodeDialect
    };

    #[test] fn adds_the_allow_comments_keyword() {
        let validator = Validator::options()
            .with_vscode_dialect()
            .build(&json!({
                "$ref": VSCodeDialect::SCHEMA_ID,
            })).unwrap();

        let invalid_instance = &json!({"allowComments": "invalid"});
        let valid_instance = &json!({"allowComments": true});

        assert_eq!(validator.is_valid(invalid_instance), false);
        assert_eq!(validator.is_valid(valid_instance), true);
    }
    #[test] fn adds_the_allow_trailing_commas_keyword() {
        let validator = Validator::options()
            .with_vscode_dialect()
            .build(&json!({
                "$ref": VSCodeDialect::SCHEMA_ID,
            })).unwrap();

        let invalid_instance = &json!({"allowTrailingCommas": "invalid"});
        let valid_instance = &json!({"allowTrailingCommas": true});

        assert_eq!(validator.is_valid(invalid_instance), false);
        assert_eq!(validator.is_valid(valid_instance), true);
    }
    #[test] fn adds_the_completion_detail_keyword() {
        let validator = Validator::options()
            .with_vscode_dialect()
            .build(&json!({
                "$ref": VSCodeDialect::SCHEMA_ID
            })).unwrap();

        let invalid_instance = &json!({"completionDetail": false});
        let valid_instance = &json!({"completionDetail": "valid"});

        assert_eq!(validator.is_valid(invalid_instance), false);
        assert_eq!(validator.is_valid(valid_instance), true);
    }
    #[test] fn adds_the_default_snippets_keyword() {
        let validator = Validator::options()
            .with_vscode_dialect()
            .build(&json!({
                "$ref": VSCodeDialect::SCHEMA_ID
            })).unwrap();

        let invalid_instance = &json!({"defaultSnippets": "invalid"});
        let valid_instance = &json!({"defaultSnippets": [{"label": "valid"}]});
        assert_eq!(validator.is_valid(invalid_instance), false);
        assert_eq!(validator.is_valid(valid_instance), true);
    }
    #[test] fn adds_the_deprecation_message_keyword() {
        let validator = Validator::options()
            .with_vscode_dialect()
            .build(&json!({
                "$ref": VSCodeDialect::SCHEMA_ID
            })).unwrap();

        let invalid_instance = &json!({"deprecationMessage": false});
        let valid_instance = &json!({"deprecationMessage": "valid"});

        assert_eq!(validator.is_valid(invalid_instance), false);
        assert_eq!(validator.is_valid(valid_instance), true);
    }
    #[test] fn adds_the_do_not_suggest_keyword() {
        let validator = Validator::options()
            .with_vscode_dialect()
            .build(&json!({
                "$ref": VSCodeDialect::SCHEMA_ID
            })).unwrap();

        let invalid_instance = &json!({"doNotSuggest": "invalid"});
        let valid_instance = &json!({"doNotSuggest": true});

        assert_eq!(validator.is_valid(invalid_instance), false);
        assert_eq!(validator.is_valid(valid_instance), true);
    }
    #[test] fn adds_the_enum_descriptions_keyword() {
        let validator = Validator::options()
            .with_vscode_dialect()
            .build(&json!({
                "$ref": VSCodeDialect::SCHEMA_ID
            })).unwrap();

        let invalid_instance = &json!({"enumDescriptions": "invalid"});
        let valid_instance = &json!({"enumDescriptions": ["valid"]});

        assert_eq!(validator.is_valid(invalid_instance), false);
        assert_eq!(validator.is_valid(valid_instance), true);
    }
    #[test] fn adds_the_enum_details_keyword() {
        let validator = Validator::options()
            .with_vscode_dialect()
            .build(&json!({
                "$ref": VSCodeDialect::SCHEMA_ID
            })).unwrap();

        let invalid_instance = &json!({"enumDetails": "invalid"});
        let valid_instance = &json!({"enumDetails": ["valid"]});

        assert_eq!(validator.is_valid(invalid_instance), false);
        assert_eq!(validator.is_valid(valid_instance), true);
    }
    #[test] fn adds_the_enum_sort_texts_keyword() {
        let validator = Validator::options()
            .with_vscode_dialect()
            .build(&json!({
                "$ref": VSCodeDialect::SCHEMA_ID
            })).unwrap();

        let invalid_instance = &json!({"enumSortTexts": "invalid"});
        let valid_instance = &json!({"enumSortTexts": ["valid"]});

        assert_eq!(validator.is_valid(invalid_instance), false);
        assert_eq!(validator.is_valid(valid_instance), true);
    }
    #[test] fn adds_the_error_message_keyword() {
        let validator = Validator::options()
            .with_vscode_dialect()
            .build(&json!({
                "$ref": VSCodeDialect::SCHEMA_ID
            })).unwrap();

        let invalid_instance = &json!({"errorMessage": false});
        let valid_instance = &json!({"errorMessage": "valid"});

        assert_eq!(validator.is_valid(invalid_instance), false);
        assert_eq!(validator.is_valid(valid_instance), true);
    }
    #[test] fn adds_the_markdown_description_keyword() {
        let validator = Validator::options()
            .with_vscode_dialect()
            .build(&json!({
                "$ref": VSCodeDialect::SCHEMA_ID
            })).unwrap();

        let invalid_instance = &json!({"markdownDescription": false});
        let valid_instance = &json!({"markdownDescription": "valid"});

        assert_eq!(validator.is_valid(invalid_instance), false);
        assert_eq!(validator.is_valid(valid_instance), true);
    }
    #[test] fn adds_the_markdown_enum_descriptions_keyword() {
        let validator = Validator::options()
            .with_vscode_dialect()
            .build(&json!({
                "$ref": VSCodeDialect::SCHEMA_ID
            })).unwrap();

        let invalid_instance = &json!({"markdownEnumDescriptions": "invalid"});
        let valid_instance = &json!({"markdownEnumDescriptions": ["valid"]});

        assert_eq!(validator.is_valid(invalid_instance), false);
        assert_eq!(validator.is_valid(valid_instance), true);
    }
    #[test] fn adds_the_pattern_error_message_keyword() {
        let validator = Validator::options()
            .with_vscode_dialect()
            .build(&json!({
                "$ref": VSCodeDialect::SCHEMA_ID
            })).unwrap();

        let invalid_instance = &json!({"patternErrorMessage": false});
        let valid_instance = &json!({"patternErrorMessage": "valid"});

        assert_eq!(validator.is_valid(invalid_instance), false);
        assert_eq!(validator.is_valid(valid_instance), true);
    }
    #[test] fn adds_the_suggest_sort_text_keyword() {
        let validator = Validator::options()
            .with_vscode_dialect()
            .build(&json!({
                "$ref": VSCodeDialect::SCHEMA_ID
            })).unwrap();

        let invalid_instance = &json!({"suggestSortText": false});
        let valid_instance = &json!({"suggestSortText": "valid"});

        assert_eq!(validator.is_valid(invalid_instance), false);
        assert_eq!(validator.is_valid(valid_instance), true);
    }
}
