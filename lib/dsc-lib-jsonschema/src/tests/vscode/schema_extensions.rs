// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[cfg(test)] mod get_completion_detail {
    use pretty_assertions::assert_eq;
    use schemars::json_schema;

    use crate::vscode::VSCodeSchemaExtensions;

    #[test] fn returns_none_when_keyword_missing() {
        assert_eq!(
            json_schema!({
                "$id": "https://contoso.com/schemas/test/example.json"
            }).get_completion_detail(),
            None
        )
    }
    #[test] fn returns_none_when_keyword_is_invalid() {
        assert_eq!(
            json_schema!({
                "$id": "https://contoso.com/schemas/test/example.json",
                "completionDetail": false
            }).get_completion_detail(),
            None
        )
    }
    #[test] fn returns_string_when_keyword_is_valid() {
        assert_eq!(
            json_schema!({
                "$id": "https://contoso.com/schemas/test/example.json",
                "completionDetail": "valid"
            }).get_completion_detail(),
            Some("valid")
        )
    }
}
#[cfg(test)] mod get_default_snippets {
    use pretty_assertions::assert_eq;
    use schemars::json_schema;

    use crate::vscode::{VSCodeSchemaExtensions, keywords::Snippet};

    #[test] fn returns_none_when_keyword_is_missing() {
        assert_eq!(
            json_schema!({
                "$id": "https://contoso.com/schemas/test/example.json"
            }).get_default_snippets(),
            None
        )
    }
    #[test] fn returns_none_when_keyword_is_invalid() {
        assert_eq!(
            json_schema!({
                "$id": "https://contoso.com/schemas/test/example.json",
                "defaultSnippets": false
            }).get_default_snippets(),
            None
        )
    }
    #[test] fn returns_vec_of_snippets_when_keyword_is_valid() {
        let snippets = vec![Snippet{
            label: Some("Example".to_string()),
            ..Default::default()
        }];
        assert_eq!(
            json_schema!({
                "$id": "https://contoso.com/schemas/test/example.json",
                "defaultSnippets": snippets
            }).get_default_snippets(),
            Some(snippets)
        )
    }
}
#[cfg(test)] mod get_deprecation_message {
    use pretty_assertions::assert_eq;
    use schemars::json_schema;

    use crate::vscode::VSCodeSchemaExtensions;

    #[test] fn returns_none_when_keyword_is_missing() {
        assert_eq!(
            json_schema!({
                "$id": "https://contoso.com/schemas/test/example.json"
            }).get_deprecation_message(),
            None
        )
    }
    #[test] fn returns_none_when_keyword_is_invalid() {
        assert_eq!(
            json_schema!({
                "$id": "https://contoso.com/schemas/test/example.json",
                "deprecationMessage": false
            }).get_deprecation_message(),
            None
        )
    }
    #[test] fn returns_string_when_keyword_is_valid() {
        assert_eq!(
            json_schema!({
                "$id": "https://contoso.com/schemas/test/example.json",
                "deprecationMessage": "valid"
            }).get_deprecation_message(),
            Some("valid")
        )
    }
}
#[cfg(test)] mod get_enum_descriptions {
    use pretty_assertions::assert_eq;
    use schemars::json_schema;

    use crate::vscode::VSCodeSchemaExtensions;

    #[test] fn returns_none_when_keyword_is_missing() {
        assert_eq!(
            json_schema!({
                "$id": "https://contoso.com/schemas/test/example.json"
            }).get_enum_descriptions(),
            None
        )
    }
    #[test] fn returns_none_when_keyword_is_invalid() {
        assert_eq!(
            json_schema!({
                "$id": "https://contoso.com/schemas/test/example.json",
                "enumDescriptions": false
            }).get_enum_descriptions(),
            None
        )
    }
    #[test] fn returns_vec_of_strings_when_keyword_is_valid() {
        assert_eq!(
            json_schema!({
                "$id": "https://contoso.com/schemas/test/example.json",
                "enumDescriptions": ["valid"]
            }).get_enum_descriptions(),
            Some(vec!["valid"])
        )
    }
}
#[cfg(test)] mod get_enum_details {
    use pretty_assertions::assert_eq;
    use schemars::json_schema;

    use crate::vscode::VSCodeSchemaExtensions;

    #[test] fn returns_none_when_keyword_is_missing() {
        assert_eq!(
            json_schema!({
                "$id": "https://contoso.com/schemas/test/example.json"
            }).get_enum_details(),
            None
        )
    }
    #[test] fn returns_none_when_keyword_is_invalid() {
        assert_eq!(
            json_schema!({
                "$id": "https://contoso.com/schemas/test/example.json",
                "enumDetails": false
            }).get_enum_details(),
            None
        )
    }
    #[test] fn returns_vec_of_strings_when_keyword_is_valid() {
        assert_eq!(
            json_schema!({
                "$id": "https://contoso.com/schemas/test/example.json",
                "enumDetails": ["valid"]
            }).get_enum_details(),
            Some(vec!["valid"])
        )
    }
}
#[cfg(test)] mod get_enum_sort_texts {
    use pretty_assertions::assert_eq;
    use schemars::json_schema;

    use crate::vscode::VSCodeSchemaExtensions;

    #[test] fn returns_none_when_keyword_is_missing() {
        assert_eq!(
            json_schema!({
                "$id": "https://contoso.com/schemas/test/example.json"
            }).get_enum_sort_texts(),
            None
        )
    }
    #[test] fn returns_none_when_keyword_is_invalid() {
        assert_eq!(
            json_schema!({
                "$id": "https://contoso.com/schemas/test/example.json",
                "enumSortTexts": false
            }).get_enum_sort_texts(),
            None
        )
    }
    #[test] fn returns_vec_of_strings_when_keyword_is_valid() {
        assert_eq!(
            json_schema!({
                "$id": "https://contoso.com/schemas/test/example.json",
                "enumSortTexts": ["valid"]
            }).get_enum_sort_texts(),
            Some(vec!["valid"])
        )
    }
}
#[cfg(test)] mod get_error_message {
    use pretty_assertions::assert_eq;
    use schemars::json_schema;

    use crate::vscode::VSCodeSchemaExtensions;

    #[test] fn returns_none_when_keyword_is_missing() {
        assert_eq!(
            json_schema!({
                "$id": "https://contoso.com/schemas/test/example.json"
            }).get_error_message(),
            None,
        )
    }
    #[test] fn returns_none_when_keyword_is_invalid() {
        assert_eq!(
            json_schema!({
                "$id": "https://contoso.com/schemas/test/example.json",
                "errorMessage": false
            }).get_error_message(),
            None
        )
    }
    #[test] fn returns_string_when_keyword_is_valid() {
        assert_eq!(
            json_schema!({
                "$id": "https://contoso.com/schemas/test/example.json",
                "errorMessage": "valid"
            }).get_error_message(),
            Some("valid")
        )
    }
}
#[cfg(test)] mod get_markdown_description {
    use pretty_assertions::assert_eq;
    use schemars::json_schema;

    use crate::vscode::VSCodeSchemaExtensions;

    #[test] fn returns_none_when_keyword_is_missing() {
        assert_eq!(
            json_schema!({
                "$id": "https://contoso.com/schemas/test/example.json"
            }).get_markdown_description(),
            None
        )
    }
    #[test] fn returns_none_when_keyword_is_invalid() {
        assert_eq!(
            json_schema!({
                "$id": "https://contoso.com/schemas/test/example.json",
                "markdownDescription": false
            }).get_markdown_description(),
            None
        )
    }
    #[test] fn returns_string_when_keyword_is_valid() {
        assert_eq!(
            json_schema!({
                "$id": "https://contoso.com/schemas/test/example.json",
                "markdownDescription": "valid"
            }).get_markdown_description(),
            Some("valid")
        )
    }
}
#[cfg(test)] mod get_markdown_enum_descriptions {
    use pretty_assertions::assert_eq;
    use schemars::json_schema;

    use crate::vscode::VSCodeSchemaExtensions;

    #[test] fn returns_none_when_keyword_is_missing() {
        assert_eq!(
            json_schema!({
                "$id": "https://contoso.com/schemas/test/example.json"
            }).get_markdown_enum_descriptions(),
            None
        )
    }
    #[test] fn returns_none_when_keyword_is_invalid() {
        assert_eq!(
            json_schema!({
                "$id": "https://contoso.com/schemas/test/example.json",
                "markdownEnumDescriptions": false
            }).get_markdown_enum_descriptions(),
            None
        )
    }
    #[test] fn returns_vec_of_strings_when_keyword_is_valid() {
        assert_eq!(
            json_schema!({
                "$id": "https://contoso.com/schemas/test/example.json",
                "markdownEnumDescriptions": ["valid"]
            }).get_markdown_enum_descriptions(),
            Some(vec!["valid"])
        )
    }
}
#[cfg(test)] mod get_pattern_error_message {
    use pretty_assertions::assert_eq;
    use schemars::json_schema;

    use crate::vscode::VSCodeSchemaExtensions;

    #[test] fn returns_none_when_keyword_is_missing() {
        assert_eq!(
            json_schema!({
                "$id": "https://contoso.com/schemas/test/example.json"
            }).get_pattern_error_message(),
            None
        )
    }
    #[test] fn returns_none_when_keyword_is_invalid() {
        assert_eq!(
            json_schema!({
                "$id": "https://contoso.com/schemas/test/example.json",
                "patternErrorMessage": false
            }).get_pattern_error_message(),
            None
        )
    }
    #[test] fn returns_string_when_keyword_is_valid() {
        assert_eq!(
            json_schema!({
                "$id": "https://contoso.com/schemas/test/example.json",
                "patternErrorMessage": "valid"
            }).get_pattern_error_message(),
            Some("valid")
        )
    }
}
#[cfg(test)] mod get_suggest_sort_text {
    use pretty_assertions::assert_eq;
    use schemars::json_schema;

    use crate::vscode::VSCodeSchemaExtensions;

    #[test] fn returns_none_when_keyword_is_missing() {
        assert_eq!(
            json_schema!({
                "$id": "https://contoso.com/schemas/test/example.json"
            }).get_suggest_sort_text(),
            None
        )
    }
    #[test] fn returns_none_when_keyword_is_invalid() {
        assert_eq!(
            json_schema!({
                "$id": "https://contoso.com/schemas/test/example.json",
                "suggestSortText": false
            }).get_suggest_sort_text(),
            None
        )
    }
    #[test] fn returns_string_when_keyword_is_valid() {
        assert_eq!(
            json_schema!({
                "$id": "https://contoso.com/schemas/test/example.json",
                "suggestSortText": "valid"
            }).get_suggest_sort_text(),
            Some("valid")
        )
    }
}
#[cfg(test)] mod should_allow_comments {
    use pretty_assertions::assert_eq;
    use schemars::json_schema;

    use crate::vscode::VSCodeSchemaExtensions;

    #[test] fn returns_false_when_keyword_is_missing() {
        assert_eq!(
            json_schema!({
                "$id": "https://contoso.com/schemas/test/example.json"
            }).should_allow_comments(),
            false
        )
    }
    #[test] fn returns_false_when_keyword_is_invalid() {
        assert_eq!(
            json_schema!({
                "$id": "https://contoso.com/schemas/test/example.json",
                "allowComments": "invalid"
            }).should_allow_comments(),
            false
        )
    }
    #[test] fn returns_false_when_keyword_is_false() {
        assert_eq!(
            json_schema!({
                "$id": "https://contoso.com/schemas/test/example.json",
                "allowComments": false
            }).should_allow_comments(),
            false
        )
    }
    #[test] fn returns_true_when_keyword_is_true() {
        assert_eq!(
            json_schema!({
                "$id": "https://contoso.com/schemas/test/example.json",
                "allowComments": true
            }).should_allow_comments(),
            true
        )
    }
}
#[cfg(test)] mod should_allow_trailing_commas {
    use pretty_assertions::assert_eq;
    use schemars::json_schema;

    use crate::vscode::VSCodeSchemaExtensions;

    #[test] fn returns_false_when_keyword_is_missing() {
        assert_eq!(
            json_schema!({
                "$id": "https://contoso.com/schemas/test/example.json"
            }).should_allow_trailing_commas(),
            false
        )
    }
    #[test] fn returns_false_when_keyword_is_invalid() {
        assert_eq!(
            json_schema!({
                "$id": "https://contoso.com/schemas/test/example.json",
                "allowTrailingCommas": "invalid"
            }).should_allow_trailing_commas(),
            false
        )
    }
    #[test] fn returns_false_when_keyword_is_false() {
        assert_eq!(
            json_schema!({
                "$id": "https://contoso.com/schemas/test/example.json",
                "allowTrailingCommas": false
            }).should_allow_trailing_commas(),
            false
        )
    }
    #[test] fn returns_true_when_keyword_is_true() {
        assert_eq!(
            json_schema!({
                "$id": "https://contoso.com/schemas/test/example.json",
                "allowTrailingCommas": true
            }).should_allow_trailing_commas(),
            true
        )
    }
}
#[cfg(test)] mod should_not_suggest {
    use pretty_assertions::assert_eq;
    use schemars::json_schema;

    use crate::vscode::VSCodeSchemaExtensions;

    #[test] fn returns_false_when_keyword_is_missing() {
        assert_eq!(
            json_schema!({
                "$id": "https://contoso.com/schemas/test/example.json"
            }).should_not_suggest(),
            false
        )
    }
    #[test] fn returns_false_when_keyword_is_invalid() {
        assert_eq!(
            json_schema!({
                "$id": "https://contoso.com/schemas/test/example.json",
                "doNotSuggest": "invalid"
            }).should_not_suggest(),
            false
        )
    }
    #[test] fn returns_false_when_keyword_is_false() {
        assert_eq!(
            json_schema!({
                "$id": "https://contoso.com/schemas/test/example.json",
                "doNotSuggest": false
            }).should_not_suggest(),
            false
        )
    }
    #[test] fn returns_true_when_keyword_is_true() {
        assert_eq!(
            json_schema!({
                "$id": "https://contoso.com/schemas/test/example.json",
                "doNotSuggest": true
            }).should_not_suggest(),
            true
        )
    }
}
#[cfg(test)] mod uses_vscode_dialect {
    use pretty_assertions::assert_eq;
    use schemars::json_schema;

    use crate::vscode::{VSCodeSchemaExtensions, dialect::VSCodeDialect};

    #[test] fn returns_false_when_schema_has_no_explicit_dialect() {
        assert_eq!(
            json_schema!({
                "$id": "https://contoso.com/schemas/test/example.json"
            }).uses_vscode_dialect(),
            false
        )
    }
    #[test] fn returns_false_when_schema_is_not_vscode_dialect() {
        assert_eq!(
            json_schema!({
                "$schema": "https://json-schema.org/draft/2020-12/schema",
                "$id": "https://contoso.com/schemas/test/example.json"
            }).uses_vscode_dialect(),
            false
        )
    }
    #[test] fn returns_true_when_schema_is_vscode_dialect() {
        assert_eq!(
            json_schema!({
                "$schema": VSCodeDialect::SCHEMA_ID,
                "$id": "https://contoso.com/schemas/test/example.json"
            }).uses_vscode_dialect(),
            true
        )
    }
}
