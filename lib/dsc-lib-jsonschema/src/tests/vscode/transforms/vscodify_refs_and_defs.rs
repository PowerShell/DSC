// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use schemars::json_schema;

use crate::vscode::transforms::vscodify_refs_and_defs;

#[test] fn when_schema_has_no_bundled_schema_resources() {
    let schema = &mut json_schema!({
        "$id": "https://contoso.com/schemas/example.json",
        "properties": {
            "foo": { "$ref": "#/$defs/foo" },
            "bar": { "$ref": "/schemas/properties/bar.json" },
            "baz": { "$ref": "https://contoso.com/schemas/properties/baz.json" },
        },
        "$defs": {
            "foo": {
                "type": "string",
            },
        }
    });
    let expected = serde_json::to_string_pretty(schema).unwrap();

    vscodify_refs_and_defs(schema);
    let actual = serde_json::to_string_pretty(schema).unwrap();

    pretty_assertions::assert_eq!(actual, expected);
}

#[test] fn when_schema_has_bundled_schema_resources() {
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
            "https://contoso.com/schemas/properties/bar.json": {
                "$id": "https://contoso.com/schemas/properties/bar.json",
                "type": "string",
            },
            "https://contoso.com/schemas/properties/baz.json": {
                "$id": "https://contoso.com/schemas/properties/baz.json",
                "type": "string",
            },
        }
    });
    vscodify_refs_and_defs(schema);
    let actual = serde_json::to_string_pretty(schema).unwrap();
    let expected = serde_json::to_string_pretty(&json_schema!({
        "$id": "https://contoso.com/schemas/example.json",
        "properties": {
            "foo": { "$ref": "#/$defs/foo" },
            "bar": { "$ref": "#/$defs/https%3A%2F%2Fcontoso.com%2Fschemas%2Fproperties%2Fbar.json" },
            "baz": { "$ref": "#/$defs/https%3A%2F%2Fcontoso.com%2Fschemas%2Fproperties%2Fbaz.json" },
        },
        "$defs": {
            "foo": {
                "$id": "https://contoso.com/schemas/properties/foo.json",
                "type": "string",
            },
            "https%3A%2F%2Fcontoso.com%2Fschemas%2Fproperties%2Fbar.json": {
                "$id": "https://contoso.com/schemas/properties/bar.json",
                "type": "string",
            },
            "https%3A%2F%2Fcontoso.com%2Fschemas%2Fproperties%2Fbaz.json": {
                "$id": "https://contoso.com/schemas/properties/baz.json",
                "type": "string",
            },
        }
    })).unwrap();

    pretty_assertions::assert_eq!(actual, expected);
}