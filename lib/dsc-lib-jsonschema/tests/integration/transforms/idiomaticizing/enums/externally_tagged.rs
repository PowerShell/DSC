// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! Integration tests for the [`idiomaticize_externally_tagged_enum`]
//! transform. Validates behavior when called on an externally tagged enum with various
//! levels and methods of documentation.
//!
//! [`idiomaticize_externally_tagged_enum`]: crate::transforms::idiomaticize_externally_tagged_enum

use pretty_assertions::assert_eq as assert_pretty_eq;
use schemars::{schema_for, JsonSchema, json_schema};

use dsc_lib_jsonschema::transforms::idiomaticize_externally_tagged_enum;

/// Defines an externally tagged enum where each variant maps to a different type. This
/// enum includes every supported documentation keyword for the enum and each variant.
#[allow(dead_code)]
#[derive(JsonSchema)]
#[serde(rename_all="camelCase")]
#[schemars(
    title = "enum-title",
    description = "enum-description",
    extend("markdownDescription" = "enum-markdown")
)]
enum ExternallyTaggedEnum {
    /// String variant
    #[schemars(
        title = "string-variant-title",
        description = "string-variant-description",
        extend("markdownDescription" = "string-variant-markdown")
    )]
    String(String),
    /// Integer variant
    #[schemars(
        title = "integer-variant-title",
        description = "integer-variant-description",
        extend("markdownDescription" = "integer-variant-markdown")
    )]
    Integer(i64),
    /// Boolean variant
    #[schemars(
        title = "boolean-variant-title",
        description = "boolean-variant-description",
        extend("markdownDescription" = "boolean-variant-markdown")
    )]
    Boolean(bool),
}

/// Checks the expected structure of an externally tagged enum's schema _without_ the
/// idiomaticizing transform. This helps ensure we can catch any cases where
/// [`schemars`] updates the default schema generated for externally tagged enums.
#[test] fn externally_tagged_enum_without_tranform() {
    let ref schema = schema_for!(ExternallyTaggedEnum);
    let ref expected = json_schema!({
        "oneOf": [
            {
                "type": "object",
                "required": ["string"],
                "additionalProperties": false,
                "properties": {
                    "string": {
                        "type": "string",
                    }
                },
                "title": "string-variant-title",
                "description": "string-variant-description",
                "markdownDescription": "string-variant-markdown"
            },
            {
                "required": ["integer"],
                "additionalProperties": false,
                "properties": {
                    "integer": {
                        "type": "integer",
                        "format": "int64",
                    }
                },
                "type": "object",
                "title": "integer-variant-title",
                "description": "integer-variant-description",
                "markdownDescription": "integer-variant-markdown"
            },
            {
                "required": ["boolean"],
                "additionalProperties": false,
                "properties": {
                    "boolean": {
                        "type": "boolean",
                    }
                },
                "type": "object",
                "title": "boolean-variant-title",
                "description": "boolean-variant-description",
                "markdownDescription": "boolean-variant-markdown"
            },
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

/// Checks the expected structure after using [`idiomaticize_externally_tagged_enum`]
/// to convert the structure of the generated schema to an idiomatic representation.
#[test] fn externally_tagged_enum_idiomaticized() {
    let ref mut schema = schema_for!(ExternallyTaggedEnum);
    idiomaticize_externally_tagged_enum(schema);
    let ref expected = json_schema!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "title": "enum-title",
        "description": "enum-description",
        "markdownDescription": "enum-markdown",
        "type": "object",
        "minProperties": 1,
        "maxProperties": 1,
        "additionalProperties": false,
        "properties": {
            "string": {
                "type": "string",
                "title": "string-variant-title",
                "description": "string-variant-description",
                "markdownDescription": "string-variant-markdown"
            },
            "integer": {
                "type": "integer",
                "format": "int64",
                "title": "integer-variant-title",
                "description": "integer-variant-description",
                "markdownDescription": "integer-variant-markdown"
            },
            "boolean": {
                "type": "boolean",
                "title": "boolean-variant-title",
                "description": "boolean-variant-description",
                "markdownDescription": "boolean-variant-markdown"
            }
        }
    });
    assert_pretty_eq!(
        serde_json::to_string_pretty(schema).unwrap(),
        serde_json::to_string_pretty(expected).unwrap()
    );
}

/// Checks the behavior for [`idiomaticize_externally_tagged_enum`] when the defined
/// `enum` doesn't use any documentation annotation keywords.
#[test] fn externally_tagged_enum_without_any_docs_idiomaticized() {
    #[allow(dead_code)]
    #[derive(JsonSchema)]
    #[serde(rename_all="camelCase")]
    enum TestingEnum {
        String(String),
        Integer(i64),
        Boolean(bool),
    }

    let ref mut schema = schema_for!(TestingEnum);
    idiomaticize_externally_tagged_enum(schema);
    let ref expected = json_schema!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "title": "TestingEnum",
        "type": "object",
        "minProperties": 1,
        "maxProperties": 1,
        "additionalProperties": false,
        "properties": {
            "string": { "type": "string" },
            "integer": { "type": "integer", "format": "int64" },
            "boolean": { "type": "boolean" }
        }
    });
    assert_pretty_eq!(
        serde_json::to_string_pretty(schema).unwrap(),
        serde_json::to_string_pretty(expected).unwrap()
    );
}

/// Checks the behavior for [`idiomaticize_externally_tagged_enum`] when the defined
/// `enum` uses Rust documentation strings to document each variant.
#[test] fn externally_tagged_enum_with_rust_docs_idiomaticized() {
    /// # testing-enum-title
    ///
    /// testing-enum-description
    #[allow(dead_code)]
    #[derive(JsonSchema)]
    #[serde(rename_all="camelCase")]
    enum TestingEnum {
        /// # string-variant-title
        ///
        /// string-variant-description
        String(String),
        /// # integer-variant-title
        ///
        /// integer-variant-description
        Integer(i64),
        /// # boolean-variant-title
        ///
        /// boolean-variant-description
        Boolean(bool),
    }

    let ref mut schema = schema_for!(TestingEnum);
    idiomaticize_externally_tagged_enum(schema);
    let ref expected = json_schema!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "title": "testing-enum-title",
        "description": "testing-enum-description",
        "type": "object",
        "minProperties": 1,
        "maxProperties": 1,
        "additionalProperties": false,
        "properties": {
            "string": {
                "title": "string-variant-title",
                "description": "string-variant-description",
                "type": "string"
            },
            "integer": {
                "title": "integer-variant-title",
                "description": "integer-variant-description",
                "type": "integer", "format": "int64"
            },
            "boolean": {
                "title": "boolean-variant-title",
                "description": "boolean-variant-description",
                "type": "boolean"
            }
        }
    });
    assert_pretty_eq!(
        serde_json::to_string_pretty(schema).unwrap(),
        serde_json::to_string_pretty(expected).unwrap()
    );
}

/// Checks the behavior for [`idiomaticize_externally_tagged_enum`] when the defined
/// `enum` uses Rust documentation strings _and_ [`schemars`] attributes to provide
/// documentation annotations.
#[test] fn externally_tagged_enum_with_varied_docs_idiomaticized() {
    /// # testing-enum-title
    ///
    /// testing-enum-description
    #[allow(dead_code)]
    #[derive(JsonSchema)]
    #[serde(rename_all="camelCase")]
    #[schemars(extend("markdownDescription" = "testing-enum-markdown"))]
    enum TestingEnum {
        /// string-variant-description
        #[schemars(
            title = "string-variant-title",
            extend("markdownDescription"="string-variant-markdown")
        )]
        String(String),
        /// # integer-variant-title
        #[schemars(
            description = "integer-variant-description",
            extend("markdownDescription" = "integer-variant-markdown")
        )]
        Integer(i64),
        #[schemars(title = "boolean-variant-title")]
        Boolean(bool),
    }

    let ref mut schema = schema_for!(TestingEnum);
    idiomaticize_externally_tagged_enum(schema);
    let ref expected = json_schema!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "title": "testing-enum-title",
        "description": "testing-enum-description",
        "markdownDescription": "testing-enum-markdown",
        "type": "object",
        "minProperties": 1,
        "maxProperties": 1,
        "additionalProperties": false,
        "properties": {
            "string": {
                "title": "string-variant-title",
                "description": "string-variant-description",
                "markdownDescription": "string-variant-markdown",
                "type": "string"
            },
            "integer": {
                "title": "integer-variant-title",
                "description": "integer-variant-description",
                "markdownDescription": "integer-variant-markdown",
                "type": "integer", "format": "int64"
            },
            "boolean": {
                "title": "boolean-variant-title",
                "type": "boolean"
            }
        }
    });
    assert_pretty_eq!(
        serde_json::to_string_pretty(schema).unwrap(),
        serde_json::to_string_pretty(expected).unwrap()
    );
}

/// Checks the behavior for [`idiomaticize_externally_tagged_enum`] when the defined
/// `enum` uses Rust documentation strings to document only a subset of variants.
#[test] fn externally_tagged_enum_with_some_missing_docs_idiomaticized() {
    /// # testing-enum-title
    ///
    /// testing-enum-description
    #[allow(dead_code)]
    #[derive(JsonSchema)]
    #[serde(rename_all="camelCase")]
    enum TestingEnum {
        /// string-variant-description
        String(String),
        #[schemars(title = "integer-variant-title")]
        Integer(i64),
        Boolean(bool),
    }

    let ref mut schema = schema_for!(TestingEnum);
    idiomaticize_externally_tagged_enum(schema);
    let ref expected = json_schema!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "title": "testing-enum-title",
        "description": "testing-enum-description",
        "type": "object",
        "minProperties": 1,
        "maxProperties": 1,
        "additionalProperties": false,
        "properties": {
            "string": {
                "description": "string-variant-description",
                "type": "string"
            },
            "integer": {
                "title": "integer-variant-title",
                "type": "integer", "format": "int64"
            },
            "boolean": {
                "type": "boolean"
            }
        }
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

    idiomaticize_externally_tagged_enum(transforming_schema);
}

#[test] #[should_panic] fn panics_when_schema_has_non_array_one_of_keyword() {
    let ref mut transforming_schema = json_schema!({
        "type": "object",
        "oneOf": {
            "type": "object"
        }
    });

    idiomaticize_externally_tagged_enum(transforming_schema);
}

#[test] #[should_panic] fn panics_when_schema_has_one_of_item_as_non_object() {
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
            "non-object"
        ]
    });

    idiomaticize_externally_tagged_enum(transforming_schema);
}

#[test] #[should_panic] fn panics_when_schema_has_one_of_item_without_type_keyword() {
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
                "required": ["foo"],
                "properties": {
                    "foo": {
                        "type": "string"
                    }
                }
            },
        ]
    });

    idiomaticize_externally_tagged_enum(transforming_schema);
}

#[test] #[should_panic] fn panics_when_schema_has_one_of_item_with_non_string_type_keyword() {
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
                "type": ["object", "null"],
                "required": ["bar"],
                "properties": {
                    "bar": {
                        "type": "string"
                    }
                }
            },
        ]
    });

    idiomaticize_externally_tagged_enum(transforming_schema);
}

#[test] #[should_panic] fn panics_when_schema_has_one_of_item_with_non_object_type() {
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

    idiomaticize_externally_tagged_enum(transforming_schema);
}

#[test] #[should_panic] fn panics_when_schema_has_one_of_item_without_properties_keyword() {
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
                "type": "object",
                "required": ["bar"],
            },
        ]
    });

    idiomaticize_externally_tagged_enum(transforming_schema);
}

#[test] #[should_panic] fn panics_when_schema_has_one_of_item_with_property_as_non_object() {
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
                "type": "object",
                "required": ["bar"],
                "properties": {
                    "bar": "invalid"
                }
            },
        ]
    });

    idiomaticize_externally_tagged_enum(transforming_schema);
}
