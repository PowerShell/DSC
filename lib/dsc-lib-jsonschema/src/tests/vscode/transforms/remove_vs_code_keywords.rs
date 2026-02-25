// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use schemars::json_schema;

use crate::vscode::transforms::remove_vs_code_keywords;

#[test] fn when_schema_has_no_vs_code_keywords() {
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
        }
    });
    let expected = serde_json::to_string_pretty(schema).unwrap();
    
    remove_vs_code_keywords(schema);
    let actual = serde_json::to_string_pretty(schema).unwrap();

    pretty_assertions::assert_eq!(actual, expected);
}

#[test] fn when_schema_has_vs_code_keywords() {
    let schema = &mut json_schema!({
        "$id": "https://contoso.com/schemas/example.json",
        "markdownDescription": "VS Code only text.",
        "properties": {
            "foo": { "$ref": "#/$defs/foo" },
            "bar": { "$ref": "/schemas/properties/bar.json" },
            "baz": { "$ref": "https://contoso.com/schemas/properties/baz.json" },
        },
        "$defs": {
            "foo": {
                "$id": "https://contoso.com/schemas/properties/foo.json",
                "type": "string",
                "markdownDescription": "VS Code text for `foo` property.",
            },
            "bar": {
                "$id": "https://contoso.com/schemas/properties/bar.json",
                "type": "string",
                "markdownDescription": "VS Code text for `bar` property.",
            },
            "baz": {
                "$id": "https://contoso.com/schemas/properties/baz.json",
                "type": "string",
                "markdownDescription": "VS Code text for `baz` property.",
            },
        }
    });
    remove_vs_code_keywords(schema);
    let actual = serde_json::to_string_pretty(schema).unwrap();

    let expected = serde_json::to_string_pretty(&json_schema!({
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
        }
    })).unwrap();

    pretty_assertions::assert_eq!(actual, expected);
}
