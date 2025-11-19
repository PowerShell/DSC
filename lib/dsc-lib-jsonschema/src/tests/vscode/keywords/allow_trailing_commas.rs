// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use pretty_assertions::assert_eq;
use rust_i18n::t;
use serde_json::json;

use crate::vscode::keywords::{AllowTrailingCommasKeyword, VSCodeKeywordDefinition};

#[test] fn meta_schema_is_valid() {
    let schema = AllowTrailingCommasKeyword::default_schema();
    let result = jsonschema::meta::validate(
        schema.as_value()
    );
    assert!(result.is_ok(), "Unexpected error: {}", result.unwrap_err());
}

#[test] fn non_boolean_value_is_invalid() {
    let validation_error = keyword_validator!(AllowTrailingCommasKeyword,  &json!({
        "allowTrailingCommas": "yes"
    })).unwrap_err().to_owned();

    assert_eq!(
        validation_error.instance_path().as_str(),
        "/allowTrailingCommas"
    );

    assert_eq!(
        format!("{validation_error}"),
        t!("vscode.keywords.allow_trailing_commas.factory_error_invalid_type")
    );
}

#[test] fn boolean_value_is_valid() {
    let validator = keyword_validator!(AllowTrailingCommasKeyword,  &json!({
        "allowTrailingCommas": true
    }));

    assert!(validator.is_ok());
}
