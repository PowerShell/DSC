// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use pretty_assertions::assert_eq;
use rust_i18n::t;
use serde_json::json;

use crate::vscode::keywords::{ErrorMessageKeyword, VSCodeKeywordDefinition};

#[test] fn meta_schema_is_valid() {
    let schema = ErrorMessageKeyword::default_schema();
    let result = jsonschema::meta::validate(
        schema.as_value()
    );
    assert!(result.is_ok(), "Unexpected error: {}", result.unwrap_err());
}

#[test] fn non_string_value_is_invalid() {
    let validation_error = keyword_validator!(ErrorMessageKeyword,  &json!({
        "errorMessage": true
    })).unwrap_err().to_owned();

    assert_eq!(
        validation_error.instance_path().as_str(),
        "/errorMessage"
    );

    assert_eq!(
        format!("{validation_error}"),
        t!("vscode.keywords.error_message.factory_error_invalid_type")
    );
}

#[test] fn string_value_is_valid() {
    let validator = keyword_validator!(ErrorMessageKeyword,  &json!({
        "errorMessage": "string value"
    }));

    assert!(validator.is_ok());
}
