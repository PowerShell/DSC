// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use pretty_assertions::assert_eq;
use rust_i18n::t;
use serde_json::json;

use crate::vscode::keywords::{MarkdownEnumDescriptionsKeyword, VSCodeKeywordDefinition};

#[test] fn meta_schema_is_valid() {
    let schema = MarkdownEnumDescriptionsKeyword::default_schema();
    let result = jsonschema::meta::validate(
        schema.as_value()
    );
    assert!(result.is_ok(), "Unexpected error: {}", result.unwrap_err());
}

#[test] fn non_array_value_is_invalid() {
    let validation_error = keyword_validator!(MarkdownEnumDescriptionsKeyword,  &json!({
        "markdownEnumDescriptions": true
    })).unwrap_err().to_owned();

    assert_eq!(
        validation_error.instance_path().as_str(),
        "/markdownEnumDescriptions"
    );

    assert_eq!(
        format!("{validation_error}"),
        format!(
            "{} {}",
            t!("vscode.keywords.markdown_enum_descriptions.factory_error_not_array"),
            t!("vscode.keywords.markdown_enum_descriptions.factory_error_suffix")
        )
    );
}
#[test] fn non_string_item_in_array_value_is_invalid() {
    let validation_error = keyword_validator!(MarkdownEnumDescriptionsKeyword,  &json!({
        "markdownEnumDescriptions": [
            "a",
            1,
            "c"
        ]
    })).unwrap_err().to_owned();

    assert_eq!(
        validation_error.instance_path().as_str(),
        "/markdownEnumDescriptions"
    );

    assert_eq!(
        format!("{validation_error}"),
        format!(
            "{} {}",
            t!("vscode.keywords.markdown_enum_descriptions.factory_error_non_string_item"),
            t!("vscode.keywords.markdown_enum_descriptions.factory_error_suffix")
        )
    );
}

#[test] fn string_array_value_is_valid() {
    let validator = keyword_validator!(MarkdownEnumDescriptionsKeyword,  &json!({
        "markdownEnumDescriptions": [
            "a",
            "b",
            "c"
        ]
    }));

    assert!(validator.is_ok());
}
