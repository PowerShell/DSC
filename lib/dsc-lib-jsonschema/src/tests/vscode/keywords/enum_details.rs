// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use pretty_assertions::assert_eq;
use rust_i18n::t;
use serde_json::json;

use crate::vscode::keywords::{EnumDetailsKeyword, VSCodeKeywordDefinition};

#[test] fn meta_schema_is_valid() {
    let schema = EnumDetailsKeyword::default_schema();
    let result = jsonschema::meta::validate(
        schema.as_value()
    );
    assert!(result.is_ok(), "Unexpected error: {}", result.unwrap_err());
}

#[test] fn non_array_value_is_invalid() {
    let validation_error = keyword_validator!(EnumDetailsKeyword,  &json!({
        "enumDetails": true
    })).unwrap_err().to_owned();

    assert_eq!(
        validation_error.instance_path().as_str(),
        "/enumDetails"
    );

    assert_eq!(
        format!("{validation_error}"),
        format!(
            "{} {}",
            t!("vscode.keywords.enum_details.factory_error_not_array"),
            t!("vscode.keywords.enum_details.factory_error_suffix")
        )
    );
}
#[test] fn non_string_item_in_array_value_is_invalid() {
    let validation_error = keyword_validator!(EnumDetailsKeyword,  &json!({
        "enumDetails": [
            "a",
            1,
            "c"
        ]
    })).unwrap_err().to_owned();

    assert_eq!(
        validation_error.instance_path().as_str(),
        "/enumDetails"
    );

    assert_eq!(
        format!("{validation_error}"),
        format!(
            "{} {}",
            t!("vscode.keywords.enum_details.factory_error_non_string_item"),
            t!("vscode.keywords.enum_details.factory_error_suffix")
        )
    );
}

#[test] fn string_array_value_is_valid() {
    let validator = keyword_validator!(EnumDetailsKeyword,  &json!({
        "enumDetails": [
            "a",
            "b",
            "c"
        ]
    }));

    assert!(validator.is_ok());
}
