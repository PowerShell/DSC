// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! Integration tests for the [`canonicalize_refs_and_defs`] transform. Validates behavior when
//! called on type with various kinds of references and definitions.

use dsc_lib_jsonschema::transforms::canonicalize_refs_and_defs;
use pretty_assertions::assert_eq;
use schemars::json_schema;

#[test] fn when_schema_has_no_bundled_resources() {
    let schema = &mut json_schema!({
        "$id": "https://contoso.com/schemas/example.json",
        "properties": {
            "foo": { "$ref": "#/$defs/foo" },
            "bar": { "$ref": "/schemas/properties/bar.json" },
            "baz": { "$ref": "https://contoso.com/schemas/properties/baz.json" },
        },
    });
    let expected = serde_json::to_string_pretty(schema).unwrap();
    canonicalize_refs_and_defs(schema);
    let actual = serde_json::to_string_pretty(schema).unwrap();

    pretty_assertions::assert_eq!(actual, expected);
}

#[test] fn when_schema_has_bundled_resources() {
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
    canonicalize_refs_and_defs(schema);
    let actual = serde_json::to_string_pretty(schema).unwrap();
    let expected = serde_json::to_string_pretty(&json_schema!({
        "$id": "https://contoso.com/schemas/example.json",
        "properties": {
            "foo": { "$ref": "https://contoso.com/schemas/properties/foo.json" },
            "bar": { "$ref": "https://contoso.com/schemas/properties/bar.json" },
            "baz": { "$ref": "https://contoso.com/schemas/properties/baz.json" },
        },
        "$defs": {
            "https://contoso.com/schemas/properties/foo.json": {
                "$id": "https://contoso.com/schemas/properties/foo.json",
                "type": "string",
            },
            "https://contoso.com/schemas/properties/bar.json": {
                "$id": "https://contoso.com/schemas/properties/bar.json",
                "type": "string",
            },
            "https://contoso.com/schemas/properties/baz.json": {
                "$id": "https://contoso.com/schemas/properties/baz.json",
                "type": "string",
            },
        },
    })).unwrap();

    assert_eq!(actual, expected);
}