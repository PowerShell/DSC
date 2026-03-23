// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use pretty_assertions::assert_eq;
use rust_i18n::t;
use serde_json::json;

use crate::vscode::keywords::{CompletionDetailKeyword, VSCodeKeywordDefinition};

#[test] fn meta_schema_is_valid() {
    let schema = CompletionDetailKeyword::default_schema();
    let result = jsonschema::meta::validate(
        schema.as_value()
    );
    assert!(result.is_ok(), "Unexpected error: {}", result.unwrap_err());
}

#[test] fn non_string_value_is_invalid() {
    let validation_error = keyword_validator!(CompletionDetailKeyword,  &json!({
        "completionDetail": true
    })).unwrap_err().to_owned();

    assert_eq!(
        validation_error.instance_path().as_str(),
        "/completionDetail"
    );

    assert_eq!(
        format!("{validation_error}"),
        t!("vscode.keywords.completion_detail.factory_error_invalid_type")
    );
}

#[test] fn string_value_is_valid() {
    let validator = keyword_validator!(CompletionDetailKeyword,  &json!({
        "completionDetail": "string value"
    }));

    assert!(validator.is_ok());
}
