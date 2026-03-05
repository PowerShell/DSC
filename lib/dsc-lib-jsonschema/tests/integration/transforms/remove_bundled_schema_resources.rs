// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! Integration tests for the [`remove_bundled_schema_resources`] transform. Validates behavior
//! when called on schemas with and without bundled schema resources defined in the `$defs` keyword.

use dsc_lib_jsonschema::transforms::remove_bundled_schema_resources;
use pretty_assertions::assert_eq;
use schemars::json_schema;

#[test] fn when_defs_not_defined() {
    let schema = &mut json_schema!({
        "$id": "https://contoso.com/schemas/example.json",
        "properties": {
            "foo": { "$ref": "#/$defs/foo" },
            "bar": { "$ref": "/schemas/properties/bar.json" },
            "baz": { "$ref": "https://contoso.com/schemas/properties/baz.json" },
        }
    });
    let expected = serde_json::to_string_pretty(schema).unwrap();
    remove_bundled_schema_resources(schema);
    let actual = serde_json::to_string_pretty(schema).unwrap();

    assert_eq!(actual, expected);
}

#[test] fn when_defs_not_contains_bundled_resources() {
    let schema = &mut json_schema!({
        "$id": "https://contoso.com/schemas/example.json",
        "properties": {
            "foo": { "$ref": "#/$defs/foo" },
            "bar": { "$ref": "/schemas/properties/bar.json" },
            "baz": { "$ref": "https://contoso.com/schemas/properties/baz.json" },
        },
        "$defs": {
            "foo": { "type": "string" },
        },
    });
    let expected = serde_json::to_string_pretty(schema).unwrap();
    remove_bundled_schema_resources(schema);
    let actual = serde_json::to_string_pretty(schema).unwrap();

    assert_eq!(actual, expected);
}

#[test] fn when_defs_contains_some_bundled_resources() {
    let schema = &mut json_schema!({
        "$id": "https://contoso.com/schemas/example.json",
        "properties": {
            "foo": { "$ref": "#/$defs/foo" },
            "bar": { "$ref": "/schemas/properties/bar.json" },
            "baz": { "$ref": "https://contoso.com/schemas/properties/baz.json" },
        },
        "$defs": {
            "foo": { "type": "string" },
            "bar": {
                "$id": "https://contoso.com/schemas/properties/bar.json",
                "type": "string",
            },
            "baz": {
                "$id": "https://contoso.com/schemas/properties/baz.json",
                "type": "string",
            },
        },
    });
    remove_bundled_schema_resources(schema);
    let actual = serde_json::to_string_pretty(schema).unwrap();
    let expected = serde_json::to_string_pretty(&json_schema!({
        "$id": "https://contoso.com/schemas/example.json",
        "properties": {
            "foo": { "$ref": "#/$defs/foo" },
            "bar": { "$ref": "/schemas/properties/bar.json" },
            "baz": { "$ref": "https://contoso.com/schemas/properties/baz.json" },
        },
        "$defs": {
            "foo": { "type": "string" },
        },
    })).unwrap();

    assert_eq!(actual, expected);
}

#[test] fn when_all_defs_are_bundled_resources() {
    let schema = &mut json_schema!({
        "$id": "https://contoso.com/schemas/example.json",
        "properties": {
            "foo": { "$ref": "#/$defs/foo" },
            "bar": { "$ref": "/schemas/properties/bar.json" },
            "baz": { "$ref": "https://contoso.com/schemas/properties/baz.json" },
        },
        "$defs": {
            "foo": {
                "$id": "https://contoso.com/schemas/properties/foo.json",
                "type": "string",
            },
            "bar": {
                "$id": "https://contoso.com/schemas/properties/bar.json",
                "type": "string",
            },
            "baz": {
                "$id": "https://contoso.com/schemas/properties/baz.json",
                "type": "string",
            },
        },
    });
    remove_bundled_schema_resources(schema);
    let actual = serde_json::to_string_pretty(schema).unwrap();
    let expected = serde_json::to_string_pretty(&json_schema!({
        "$id": "https://contoso.com/schemas/example.json",
        "properties": {
            "foo": { "$ref": "#/$defs/foo" },
            "bar": { "$ref": "/schemas/properties/bar.json" },
            "baz": { "$ref": "https://contoso.com/schemas/properties/baz.json" },
        },
    })).unwrap();

    assert_eq!(actual, expected);
}
