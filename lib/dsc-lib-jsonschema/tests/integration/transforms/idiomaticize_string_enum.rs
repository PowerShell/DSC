// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! Integration tests for the [`idiomaticize_string_enum`]
//! transform. Validates behavior when called on an externally tagged enum with various
//! levels and methods of documentation.
//!
//! [`idiomaticize_string_enum`]: crate::transforms::idiomaticize_string_enum

use pretty_assertions::assert_eq as assert_pretty_eq;
use schemars::{schema_for, JsonSchema, json_schema};

use dsc_lib_jsonschema::transforms::idiomaticize_string_enum;

/// Defines an enum where each variant maps to a string value. This enum includes every
/// supported documentation keyword for the enum and each variant.
#[allow(dead_code)]
#[derive(JsonSchema)]
#[schemars(
    title="enum-title",
    description="enum-description",
    extend("markdownDescription" = "enum-markdown")
)]
#[serde(rename_all="camelCase")]
enum StringEnum {
    #[schemars(
        title="foo-title",
        description="foo-description",
        extend("markdownDescription"="foo-markdown")
    )]
    Foo,
    #[schemars(
        title="bar-title",
        description="bar-description",
        extend("markdownDescription"="bar-markdown")
    )]
    Bar,
    #[schemars(
        title="baz-title",
        description="baz-description",
        extend("markdownDescription"="baz-markdown")
    )]
    Baz
}

/// Checks the expected structure of a string enum's schema _without_ the idiomaticizing
/// transform. This helps ensure we can catch any cases where [`schemars`] updates the default
/// schema generated for string enums.
#[test] fn string_enum_without_tranform() {
    let ref schema = schema_for!(StringEnum);
    let ref expected = json_schema!({
        "oneOf": [
            {
                "type": "string",
                "const": "foo",
                "title": "foo-title",
                "description": "foo-description",
                "markdownDescription": "foo-markdown"
            },
            {
                "type": "string",
                "const": "bar",
                "title": "bar-title",
                "description": "bar-description",
                "markdownDescription": "bar-markdown"
            },
            {
                "type": "string",
                "const": "baz",
                "title": "baz-title",
                "description": "baz-description",
                "markdownDescription": "baz-markdown"
            }
        ],
        "title": "enum-title",
        "description": "enum-description",
        "markdownDescription": "enum-markdown",
        "$schema": "https://json-schema.org/draft/2020-12/schema"
    });
    assert_pretty_eq!(
        serde_json::to_string_pretty(schema).unwrap(),
        serde_json::to_string_pretty(expected).unwrap()
    );
}
/// Checks the expected structure after using the [`idiomaticize_string_enum`] function to
/// convert the structure of the generated schema to an idiomatic representation.
#[test] fn string_enum_idiomaticized() {
    let ref mut schema = schema_for!(StringEnum);
    idiomaticize_string_enum(schema);
    let ref expected = json_schema!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "title": "enum-title",
        "description": "enum-description",
        "type": "string",
        "markdownDescription": "enum-markdown",
        "enum": ["foo", "bar", "baz"],
        "enumDescriptions": ["foo-description", "bar-description", "baz-description"],
        "enumMarkdownDescriptions": ["foo-markdown", "bar-markdown", "baz-markdown"]
    });
    assert_pretty_eq!(
        serde_json::to_string_pretty(schema).unwrap(),
        serde_json::to_string_pretty(expected).unwrap()
    );
}

/// Checks the behavior for [`idiomaticize_string_enum`] when the defined `enum` doesn't use
/// any documentation annotation keywords.
#[test] fn string_enum_without_any_docs_idiomaticized() {
    #[allow(dead_code)]
    #[derive(JsonSchema)]
    #[serde(rename_all="camelCase")]
    enum TestingEnum {
        Foo,
        Bar,
        Baz
    }

    let ref mut schema = schema_for!(TestingEnum);
    idiomaticize_string_enum(schema);
    let ref expected = json_schema!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "title": "TestingEnum",
        "type": "string",
        "enum": ["foo", "bar", "baz"]
    });
    assert_pretty_eq!(
        serde_json::to_string_pretty(schema).unwrap(),
        serde_json::to_string_pretty(expected).unwrap()
    );
}

/// Checks the behavior for [`idiomaticize_string_enum`] when the defined `enum` uses Rust
/// documentation strings to document each variant.
#[test] fn string_enum_with_rust_docs_idiomaticized() {
    #[allow(dead_code)]
    #[derive(JsonSchema)]
    #[serde(rename_all="camelCase")]
    enum TestingEnum {
        /// Foo-description
        Foo,
        /// Bar-description
        Bar,
        /// Baz-description
        Baz
    }

    let ref mut schema = schema_for!(TestingEnum);
    idiomaticize_string_enum(schema);
    let ref expected = json_schema!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "title": "TestingEnum",
        "type": "string",
        "enum": ["foo", "bar", "baz"],
        "enumDescriptions": ["Foo-description", "Bar-description", "Baz-description"],
        "enumMarkdownDescriptions": ["Foo-description", "Bar-description", "Baz-description"],
    });
    assert_pretty_eq!(
        serde_json::to_string_pretty(schema).unwrap(),
        serde_json::to_string_pretty(expected).unwrap()
    );
}

/// Checks the behavior for [`idiomaticize_string_enum`] when the defined `enum` uses Rust
/// documentation strings _and_ [`schemars`] attributes to provide documentation annotations.
#[test] fn string_enum_with_varied_docs_idiomaticized() {
    #[allow(dead_code)]
    #[derive(JsonSchema)]
    #[serde(rename_all="camelCase")]
    enum TestingEnum {
        /// Foo-description
        #[schemars(title="Foo-title", extend("markdownDescription"="Foo-markdown"))]
        Foo,
        #[schemars(
            title="Bar-title", extend("markdownDescription"="Bar-markdown"))]
        Bar,
        #[schemars(title="Baz-title")]
        Baz
    }

    let ref mut schema = schema_for!(TestingEnum);
    idiomaticize_string_enum(schema);
    let ref expected = json_schema!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "title": "TestingEnum",
        "type": "string",
        "enum": ["foo", "bar", "baz"],
        "enumDescriptions": ["Foo-description", "Bar-title", "Baz-title"],
        "enumMarkdownDescriptions": ["Foo-markdown", "Bar-markdown", "Baz-title"],
    });
    assert_pretty_eq!(
        serde_json::to_string_pretty(schema).unwrap(),
        serde_json::to_string_pretty(expected).unwrap()
    );
}

/// Checks the behavior for [`idiomaticize_string_enum`] when the defined `enum` uses Rust
/// documentation strings to document only a subset of variants.
#[test] fn string_enum_with_some_missing_docs_idiomaticized() {
    #[allow(dead_code)]
    #[derive(JsonSchema)]
    #[serde(rename_all="camelCase")]
    enum TestingEnum {
        /// Foo-description
        Foo,
        /// Bar-description
        Bar,
        Baz
    }

    let ref mut schema = schema_for!(TestingEnum);
    idiomaticize_string_enum(schema);
    // Note that for some reason, non-documented items go before documented
    // ones when generating the schema.
    let ref expected = json_schema!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "title": "TestingEnum",
        "type": "string",
        "enum": ["baz", "foo", "bar"],
        "enumDescriptions": ["", "Foo-description", "Bar-description"],
        "enumMarkdownDescriptions": ["", "Foo-description", "Bar-description"],
    });
    assert_pretty_eq!(
        serde_json::to_string_pretty(schema).unwrap(),
        serde_json::to_string_pretty(expected).unwrap()
    );
}

#[test] #[should_panic] fn panics_when_schema_missing_oneof_keyword() {
    let ref mut transforming_schema = json_schema!({
        "type": "object",
        "properties": {
            "foo": {
                "type": "string"
            }
        }
    });

    idiomaticize_string_enum(transforming_schema);
}

#[test] #[should_panic] fn panics_when_schema_has_non_array_one_of_keyword() {
    let ref mut transforming_schema = json_schema!({
        "type": "object",
        "oneOf": {
            "type": "object"
        }
    });

    idiomaticize_string_enum(transforming_schema);
}

#[test] #[should_panic] fn panics_when_schema_has_one_of_item_as_non_object() {
    let ref mut transforming_schema = json_schema!({
        "type": "object",
        "oneOf": [
            "non-object"
        ]
    });

    idiomaticize_string_enum(transforming_schema);
}

#[test] #[should_panic] fn panics_when_schema_has_one_of_item_without_type_keyword() {
    let ref mut transforming_schema = json_schema!({
        "type": "object",
        "oneOf": [
            {
                "const": "foo"
            }
        ]
    });

    idiomaticize_string_enum(transforming_schema);
}

#[test] #[should_panic] fn panics_when_schema_has_one_of_item_with_non_string_type_keyword() {
    let ref mut transforming_schema = json_schema!({
        "type": "object",
        "oneOf": [
            {
                "type": ["string", "null"],
                "const": "foo"
            }
        ]
    });

    idiomaticize_string_enum(transforming_schema);
}

#[test] #[should_panic] fn panics_when_schema_has_one_of_item_with_non_string_type() {
    let ref mut transforming_schema = json_schema!({
        "type": "object",
        "oneOf": [
            {
                "type": "object",
                "required": ["foo"],
                "properties": {
                    "foo": {
                        "type": "string"
                    }
                }
            },
            {
                "type": "string",
                "const": "bar"
            },
        ]
    });

    idiomaticize_string_enum(transforming_schema);
}

#[test] #[should_panic] fn panics_when_schema_has_one_of_item_with_non_array_enum_keyword() {
    let ref mut transforming_schema = json_schema!({
        "type": "object",
        "oneOf": [
            {
                "type": "string",
                "enum": "foo"
            },
            {
                "type": "string",
                "const": "bar"
            },
        ]
    });

    idiomaticize_string_enum(transforming_schema);
}

#[test] #[should_panic] fn panics_when_schema_has_one_of_item_with_non_string_enum_item() {
    let ref mut transforming_schema = json_schema!({
        "type": "object",
        "oneOf": [
            {
                "type": "string",
                "enum": [false]
            },
            {
                "type": "string",
                "const": "bar"
            },
        ]
    });

    idiomaticize_string_enum(transforming_schema);
}

#[test] #[should_panic] fn panics_when_schema_has_one_of_item_without_enum_or_const() {
    let ref mut transforming_schema = json_schema!({
        "type": "object",
        "oneOf": [
            {
                "type": "string",
            },
        ]
    });

    idiomaticize_string_enum(transforming_schema);
}

#[test] #[should_panic] fn panics_when_schema_has_one_of_item_with_non_string_const() {
    let ref mut transforming_schema = json_schema!({
        "type": "object",
        "oneOf": [
            {
                "type": "string",
                "const": false
            },
        ]
    });

    idiomaticize_string_enum(transforming_schema);
}
