// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use pretty_assertions::assert_eq;
use rust_i18n::t;
use serde_json::json;

use crate::vscode::keywords::{PatternErrorMessageKeyword, VSCodeKeywordDefinition};

#[test] fn meta_schema_is_valid() {
    let schema = PatternErrorMessageKeyword::default_schema();
    let result = jsonschema::meta::validate(
        schema.as_value()
    );
    assert!(result.is_ok(), "Unexpected error: {}", result.unwrap_err());
}

#[test] fn non_string_value_is_invalid() {
    let validation_error = keyword_validator!(PatternErrorMessageKeyword,  &json!({
        "patternErrorMessage": true
    })).unwrap_err().to_owned();

    assert_eq!(
        validation_error.instance_path().as_str(),
        "/patternErrorMessage"
    );

    assert_eq!(
        format!("{validation_error}"),
        t!("vscode.keywords.pattern_error_message.factory_error_invalid_type")
    );
}

#[test] fn string_value_is_valid() {
    let validator = keyword_validator!(PatternErrorMessageKeyword,  &json!({
        "patternErrorMessage": "string value"
    }));

    assert!(validator.is_ok());
}
