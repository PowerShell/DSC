// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use pretty_assertions::assert_eq;
use rust_i18n::t;
use serde_json::json;

use crate::vscode::keywords::{DefaultSnippetsKeyword, VSCodeKeywordDefinition};

#[test] fn meta_schema_is_valid() {
    let schema = DefaultSnippetsKeyword::default_schema();
    let result = jsonschema::meta::validate(
        schema.as_value()
    );
    assert!(result.is_ok(), "Unexpected error: {}", result.unwrap_err());
}

#[test] fn non_array_value_is_invalid() {
    let validation_error = keyword_validator!(DefaultSnippetsKeyword,  &json!({
        "defaultSnippets": true
    })).unwrap_err().to_owned();

    assert_eq!(
        validation_error.instance_path().as_str(),
        "/defaultSnippets"
    );

    assert_eq!(
        format!("{validation_error}"),
        format!(
            "{} {}",
            t!("vscode.keywords.default_snippets.factory_error_not_array"),
            t!("vscode.keywords.default_snippets.factory_error_suffix"),
        ),
    );
}

#[test] fn array_with_non_object_is_invalid() {
    let validation_error = keyword_validator!(DefaultSnippetsKeyword,  &json!({
        "defaultSnippets": [{"label": "valid"}, "invalid"]
    })).unwrap_err().to_owned();

    assert_eq!(
        validation_error.instance_path().as_str(),
        "/defaultSnippets"
    );

    assert_eq!(
        format!("{validation_error}"),
        format!(
            "{} {}",
            t!("vscode.keywords.default_snippets.factory_error_non_object_item"),
            t!("vscode.keywords.default_snippets.factory_error_suffix"),
        ),
    );
}
#[test] fn array_with_non_snippet_object_is_invalid() {
    let validation_error = keyword_validator!(DefaultSnippetsKeyword,  &json!({
        "defaultSnippets": [{"label": "valid"}, {"invalid": true}]
    })).unwrap_err().to_owned();

    assert_eq!(
        validation_error.instance_path().as_str(),
        "/defaultSnippets"
    );

    assert_eq!(
        format!("{validation_error}"),
        format!(
            "{} {}",
            t!("vscode.keywords.default_snippets.factory_error_invalid_item"),
            t!("vscode.keywords.default_snippets.factory_error_suffix"),
        ),
    );
}
#[test] fn array_with_invalid_snippet_object_is_invalid() {
    let validation_error = keyword_validator!(DefaultSnippetsKeyword,  &json!({
        "defaultSnippets": [{"label": "valid"}, {"label": false}]
    })).unwrap_err().to_owned();

    assert_eq!(
        validation_error.instance_path().as_str(),
        "/defaultSnippets"
    );

    assert_eq!(
        format!("{validation_error}"),
        format!(
            "{} {}",
            t!("vscode.keywords.default_snippets.factory_error_invalid_item"),
            t!("vscode.keywords.default_snippets.factory_error_suffix"),
        ),
    );
}

#[test] fn array_of_valid_snippets_is_valid() {
    let validator = keyword_validator!(DefaultSnippetsKeyword,  &json!({
        "defaultSnippets": [{"label": "valid"}]
    }));

    assert!(validator.is_ok());
}
