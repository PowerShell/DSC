// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! Unit tests for [`dsc-lib-jsonschema::schema_utility_extensions`].

use core::{clone::Clone, convert::Into};
use std::sync::LazyLock;

// use pretty_assertions::assert_eq;
use schemars::{json_schema, Schema};
use serde_json::{json, Map, Value};

use crate::schema_utility_extensions::SchemaUtilityExtensions;

// Setup static data for the `get_keyword_as*` test macros
static ARRAY_VALUE: LazyLock<Vec<Value>> = LazyLock::new(|| Vec::from([
    Value::String("a".to_string()),
    Value::String("b".to_string()),
    Value::String("c".to_string()),
]));
static BOOLEAN_VALUE: bool = true;
static INTEGER_VALUE: i64  = 5;
static FLOAT_VALUE:   f64  = 1.2;
static OBJECT_VALUE: LazyLock<Map<String, Value>> = LazyLock::new(|| json!({
    "foo": "bar"
}).as_object().unwrap().clone());
static NULL_VALUE:    ()   = ();
static STRING_VALUE: &str  = "value";
static SUBSCHEMA_VALUE: LazyLock<Schema> = LazyLock::new(|| json_schema!({
    "$id": "https://schema.contoso.com/test/get_keyword_as/subschema.json"
}));
static TEST_SCHEMA: LazyLock<Schema> = LazyLock::new(|| json_schema!({
    "$id": "https://schema.contoso.com/test/get_keyword_as.json",
    "array": *ARRAY_VALUE,
    "boolean": BOOLEAN_VALUE,
    "integer": INTEGER_VALUE,
    "float": FLOAT_VALUE,
    "object": *OBJECT_VALUE,
    "null": null,
    "string": *STRING_VALUE,
    "subschema": *SUBSCHEMA_VALUE,
}));

/// Defines test cases for a given `get_keyword_as` function (non-mutable).
///
/// Each test case verifies behavior when:
///
/// - The given keyword doesn't exist (return [`None`])
/// - The given keyword has the wrong data type (return [`None`])
/// - The given keyword has the correct data type (return [`Some`] with the data).
///
/// # Arguments
///
/// The first argument must be the identifier for the function to test. The second argument is
/// name of a keyword to retrieve with invalid data. The third argument is the name of a keyword
/// to retrieve with valid data. The last argument  is the expected value for the valid lookup.
macro_rules! test_cases_for_get_keyword_as {
    ($(
        $test_function:ident: $invalid_lookup:expr, $valid_lookup:expr, $expected_valid:expr,
    )*) => {
    $(
        #[cfg(test)]
        mod $test_function {
            #![allow(unused_imports)]
            use super::*;
            // use super::super::*;
            use pretty_assertions::assert_eq;
            use schemars::{json_schema, Schema};
            use serde_json::{json, Map, Value, Number};

            #[test] fn when_keyword_missing() {
                let schema = TEST_SCHEMA.clone();
                assert_eq!(schema.$test_function("not_exist"), None);
            }
            #[test] fn when_keyword_has_invalid_type() {
                let schema = TEST_SCHEMA.clone();
                assert_eq!(schema.$test_function($invalid_lookup), None);
            }
            #[test] fn when_keyword_has_valid_type() {
                let schema = TEST_SCHEMA.clone();
                assert_eq!(schema.$test_function($valid_lookup), $expected_valid);
            }
        }
    )*
    };
}

/// Defines test cases for a given `get_keyword_as` function (mutable).
///
/// Each test case verifies behavior when:
///
/// - The given keyword doesn't exist (return [`None`])
/// - The given keyword has the wrong data type (return [`None`])
/// - The given keyword has the correct data type (return [`Some`] with the data).
///
/// # Arguments
///
/// The first argument must be the identifier for the function to test. The second argument is
/// name of a keyword to retrieve with invalid data. The third argument is the name of a keyword
/// to retrieve with valid data. The last argument  is the expected value for the valid lookup.
macro_rules! test_cases_for_get_keyword_as_mut {
    ($(
        $test_function:ident: $invalid_lookup:expr, $valid_lookup:expr, $expected_valid:expr,
    )*) => {
    $(
        #[cfg(test)]
        mod $test_function {
            #![allow(unused_imports)]
            use super::*;
            // use super::super::*;
            use pretty_assertions::assert_eq;
            use schemars::{json_schema, Schema};
            use serde_json::{json, Map, Value, Number};

            #[test] fn when_keyword_missing() {
                let ref mut schema = TEST_SCHEMA.clone();
                assert_eq!(schema.$test_function("not_exist"), None);
            }
            #[test] fn when_keyword_has_invalid_type() {
                let ref mut schema = TEST_SCHEMA.clone();
                assert_eq!(schema.$test_function($invalid_lookup), None);
            }
            #[test] fn when_keyword_has_valid_type() {
                let ref mut schema = TEST_SCHEMA.clone();
                assert_eq!(schema.$test_function($valid_lookup), $expected_valid);
            }
        }
    )*
    };
}

test_cases_for_get_keyword_as!(
    get_keyword_as_array: "boolean", "array", Some(&*ARRAY_VALUE),
    get_keyword_as_bool: "array", "boolean", Some(BOOLEAN_VALUE),
    get_keyword_as_f64: "array", "float", Some(FLOAT_VALUE),
    get_keyword_as_i64: "array", "integer", Some(INTEGER_VALUE),
    get_keyword_as_null: "array", "null", Some(NULL_VALUE),
    get_keyword_as_object: "array", "object", Some(&*OBJECT_VALUE),
    get_keyword_as_number: "array", "integer", Some(&(INTEGER_VALUE.into())),
    get_keyword_as_str: "array", "string", Some(STRING_VALUE),
    get_keyword_as_string: "array", "string", Some(STRING_VALUE.to_string()),
    get_keyword_as_subschema: "array", "subschema", Some(&*SUBSCHEMA_VALUE),
);


test_cases_for_get_keyword_as_mut!(
    get_keyword_as_array_mut: "boolean", "array", Some(&mut (*ARRAY_VALUE).clone()),
    get_keyword_as_object_mut: "array", "object", Some(&mut (*OBJECT_VALUE).clone()),
    get_keyword_as_subschema_mut: "array", "subschema", Some(&mut (*SUBSCHEMA_VALUE).clone()),
);

#[cfg(test)] mod get_id {
    use pretty_assertions::assert_eq;
    use schemars::json_schema;

    use crate::schema_utility_extensions::SchemaUtilityExtensions;

    #[test] fn when_id_keyword_missing() {
        let ref schema = json_schema!({
            "title": "Missing ID"
        });
        assert_eq!(schema.get_id(), None);
    }
    #[test] fn when_id_keyword_is_not_string() {
        let ref schema = json_schema!({
            "$id": 5,
        });
        assert_eq!(schema.get_id(), None);
    }
    #[test] fn when_id_keyword_is_string() {
        let id = "https://schemas.contoso.com/test/valid_id.json";
        let ref schema = json_schema!({
            "$id": id
        });
        assert_eq!(schema.get_id(), Some(id));
    }
}
#[cfg(test)] mod get_id_as_url {
    use pretty_assertions::assert_eq;
    use schemars::json_schema;
    use url::Url;

    use crate::schema_utility_extensions::SchemaUtilityExtensions;

    #[test] fn when_id_keyword_missing() {
        let ref schema = json_schema!({
            "title": "Missing ID"
        });
        assert_eq!(schema.get_id_as_url(), None);
    }
    #[test] fn when_id_keyword_is_not_string() {
        let ref schema = json_schema!({
            "$id": 5,
        });
        assert_eq!(schema.get_id_as_url(), None);
    }
    #[test] fn when_id_keyword_is_string_but_not_valid_url() {
        let ref schema = json_schema!({
            "$id": "invalid",
        });
        assert_eq!(schema.get_id_as_url(), None);
    }
    #[test] fn when_id_keyword_is_relative_url() {
        let ref schema = json_schema!({
            "$id": "/test/valid_id.json",
        });
        assert_eq!(schema.get_id_as_url(), None);
    }
    #[test] fn when_id_keyword_is_absolute_url() {
        let id_str = "https://schemas.contoso.com/test/valid_id.json";
        let id_url = Url::parse(id_str).unwrap();
        let ref schema = json_schema!({
            "$id": id_str
        });
        assert_eq!(schema.get_id_as_url(), Some(id_url));
    }
}

#[cfg(test)] mod has_id_keyword {
    use pretty_assertions::assert_eq;
    use schemars::json_schema;

    use crate::schema_utility_extensions::SchemaUtilityExtensions;

    #[test] fn when_keyword_exists() {
        let ref schema = json_schema!({
            "$id": "https://schemas.contoso.com/test/valid_id.json"
        });
        assert_eq!(schema.has_id_keyword(), true);
    }
    #[test] fn when_keyword_not_exists() {
        let ref schema = json_schema!({
            "title": "Missing '$id' keyword"
        });
        assert_eq!(schema.has_id_keyword(), false);
    }
}

#[cfg(test)] mod set_id {
    use pretty_assertions::assert_eq;
    use schemars::json_schema;

    use crate::schema_utility_extensions::SchemaUtilityExtensions;

    #[test] fn when_id_already_defined() {
        let id = "https://schemas.contoso.com/test/valid_id.json";
        let ref mut schema = json_schema!({
            "$id": id
        });
        assert_eq!(
            schema.set_id("https://schemas.contoso.com/test/new_id.json"),
            Some(id.to_string())
        );
    }
    #[test] fn when_id_not_already_defined() {
        let id_uri = "https://schemas.contoso.com/test/valid_id.json";
        let ref mut schema = json_schema!({
            "title": "Without initial '$id' keyword"
        });
        assert_eq!(schema.set_id(id_uri), None);
    }
}

#[cfg(test)] mod get_defs {
    use pretty_assertions::assert_eq;
    use schemars::json_schema;
    use serde_json::json;

    use crate::schema_utility_extensions::SchemaUtilityExtensions;

    #[test] fn when_defs_keyword_missing() {
        let ref schema = json_schema!({
            "title": "Schema without '$defs' keyword"
        });
        assert_eq!(schema.get_defs(), None);
    }
    #[test] fn when_defs_keyword_is_not_object() {
        let ref schema = json_schema!({
            "title": "Schema with non-object '$defs' keyword",
            "$defs": "invalid"
        });
        assert_eq!(schema.get_defs(), None);
    }
    #[test] fn when_defs_keyword_is_object() {
        let defs_json= json!({
            "first": {
                "title": "first definition subschema"
            },
            "second": {
                "title": "second definition subschema"
            },
        });
        let defs_object = defs_json.as_object().unwrap();
        let ref schema = json_schema!({
            "title": "schema with '$defs' as object",
            "$defs": defs_object,
        });
        assert_eq!(schema.get_defs(), Some(defs_object));
    }
}

#[cfg(test)] mod get_defs_mut {
    use pretty_assertions::assert_eq;
    use schemars::json_schema;
    use serde_json::json;

    use crate::schema_utility_extensions::SchemaUtilityExtensions;

    #[test] fn when_defs_keyword_missing() {
        let ref mut schema = json_schema!({
            "title": "Schema without '$defs' keyword"
        });
        assert_eq!(schema.get_defs_mut(), None);
    }
    #[test] fn when_defs_keyword_is_not_object() {
        let ref mut schema = json_schema!({
            "title": "Schema with non-object '$defs' keyword",
            "$defs": "invalid"
        });
        assert_eq!(schema.get_defs_mut(), None);
    }
    #[test] fn when_defs_keyword_is_object() {
        let defs_json= json!({
            "first": {
                "title": "first definition subschema"
            },
            "second": {
                "title": "second definition subschema"
            },
        });
        let ref mut defs_object = defs_json.as_object().unwrap().clone();
        let ref mut schema = json_schema!({
            "title": "schema with '$defs' as object",
            "$defs": defs_object.clone(),
        });
        assert_eq!(schema.get_defs_mut(), Some(defs_object));
    }
}
#[cfg(test)] mod get_defs_subschema_from_id {
    use core::option::Option::None;

    use pretty_assertions::assert_eq;
    use schemars::json_schema;

    use crate::schema_utility_extensions::SchemaUtilityExtensions;

    #[test] fn when_defs_keyword_missing() {
        let ref schema = json_schema!({
            "$id": "https://contoso.com/schemas/test.json"
        });
        assert_eq!(
            schema.get_defs_subschema_from_id("https://contoso.com/schemas/foo.json"),
            None
        );
    }
    #[test] fn when_defs_keyword_is_not_object() {
        let ref schema = json_schema!({
            "$id": "https://contoso.com/schemas/test.json",
            "$defs": "invalid"
        });
        assert_eq!(
            schema.get_defs_subschema_from_id("https://contoso.com/schemas/foo.json"),
            None
        );
    }
    #[test] fn when_defs_keyword_is_object_and_entry_missing_id() {
        let ref schema = json_schema!({
            "$id": "https://contoso.com/schemas/test.json",
            "$defs": {
                "foo": {
                    "title": "Foo"
                }
            }
        });
        assert_eq!(
            schema.get_defs_subschema_from_id("https://contoso.com/schemas/foo.json"),
            None
        );
    }
    #[test] fn when_defs_keyword_is_object_and_entry_has_matching_id_keyword() {
        let ref schema = json_schema!({
            "$id": "https://contoso.com/schemas/test.json",
            "$defs": {
                "foo": {
                    "$id": "https://contoso.com/schemas/foo.json",
                    "title": "Foo"
                }
            }
        });
        let ref expected = json_schema!({
            "$id": "https://contoso.com/schemas/foo.json",
            "title": "Foo"
        });
        assert_eq!(
            schema.get_defs_subschema_from_id("https://contoso.com/schemas/foo.json"),
            Some(expected)
        );
    }
}
#[cfg(test)] mod get_defs_subschema_from_id_mut {
    use core::option::Option::None;

    use pretty_assertions::assert_eq;
    use schemars::json_schema;

    use crate::schema_utility_extensions::SchemaUtilityExtensions;

    #[test] fn when_defs_keyword_missing() {
        let ref mut schema = json_schema!({
            "$id": "https://contoso.com/schemas/test.json"
        });
        assert_eq!(
            schema.get_defs_subschema_from_id_mut("https://contoso.com/schemas/foo.json"),
            None
        );
    }
    #[test] fn when_defs_keyword_is_not_object() {
        let ref mut schema = json_schema!({
            "$id": "https://contoso.com/schemas/test.json",
            "$defs": "invalid"
        });
        assert_eq!(
            schema.get_defs_subschema_from_id_mut("https://contoso.com/schemas/foo.json"),
            None
        );
    }
    #[test] fn when_defs_keyword_is_object_and_entry_missing_id() {
        let ref mut schema = json_schema!({
            "$id": "https://contoso.com/schemas/test.json",
            "$defs": {
                "foo": {
                    "title": "Foo"
                }
            }
        });
        assert_eq!(
            schema.get_defs_subschema_from_id_mut("https://contoso.com/schemas/foo.json"),
            None
        );
    }
    #[test] fn when_defs_keyword_is_object_and_entry_has_matching_id_keyword() {
        let ref mut schema = json_schema!({
            "$id": "https://contoso.com/schemas/test.json",
            "$defs": {
                "foo": {
                    "$id": "https://contoso.com/schemas/foo.json",
                    "title": "Foo"
                }
            }
        });
        let ref mut expected = json_schema!({
            "$id": "https://contoso.com/schemas/foo.json",
            "title": "Foo"
        });
        assert_eq!(
            schema.get_defs_subschema_from_id_mut("https://contoso.com/schemas/foo.json"),
            Some(expected)
        );
    }
}
#[cfg(test)] mod get_defs_subschema_from_reference {
    use pretty_assertions::assert_eq;
    use schemars::json_schema;

    use crate::schema_utility_extensions::SchemaUtilityExtensions;

    #[test] fn when_defs_keyword_missing() {
        let schema = json_schema!({
            "title": "missing defs"
        });
        assert_eq!(
            schema.get_defs_subschema_from_reference("#/$defs/first"),
            None
        );
    }
    #[test] fn when_defs_keyword_is_not_object() {
        let schema = json_schema!({
            "title": "missing defs",
            "$defs": "invalid"
        });
        assert_eq!(
            schema.get_defs_subschema_from_reference("#/$defs/first"),
            None
        );
    }
    #[test] fn when_defs_keyword_is_object_and_entry_missing() {
        let schema = json_schema!({
            "title": "missing defs",
            "$defs": {
                "second": {
                    "title": "second value"
                }
            }
        });
        assert_eq!(
            schema.get_defs_subschema_from_reference("#/$defs/first"),
            None
        );
    }
    #[test] fn when_defs_keyword_is_object_and_entry_is_not_object() {
        let schema = json_schema!({
            "title": "missing defs",
            "$defs": {
                "first": "invalid"
            }
        });
        assert_eq!(
            schema.get_defs_subschema_from_reference("#/$defs/first"),
            None
        );
    }
    #[test] fn with_defs_pointer_reference() {
        let schema = json_schema!({
            "$schema": "https://json-schema.org/draft/2020-12/schema",
            "$id": "https://contoso.com/schemas/object.json",
            "type": "object",
            "$defs": {
                "foo": {
                    "$id": "https://contoso.com/schemas/foo.json",
                    "title": "Foo"
                }
            }
        });
        let ref expected = json_schema!({
            "$id": "https://contoso.com/schemas/foo.json",
            "title": "Foo"
        });
        assert_eq!(
            schema.get_defs_subschema_from_reference("#/$defs/foo").unwrap(),
            expected
        );
    }
    #[test] fn with_absolute_id_uri_reference() {
        let schema = json_schema!({
            "$schema": "https://json-schema.org/draft/2020-12/schema",
            "$id": "https://contoso.com/schemas/object.json",
            "type": "object",
            "$defs": {
                "foo": {
                    "$id": "https://contoso.com/schemas/foo.json",
                    "title": "Foo"
                }
            }
        });
        let ref expected = json_schema!({
            "$id": "https://contoso.com/schemas/foo.json",
            "title": "Foo"
        });
        assert_eq!(
            schema.get_defs_subschema_from_reference("/schemas/foo.json").unwrap(),
            expected
        );
    }
    #[test] fn with_relative_id_uri_reference() {
        let schema = json_schema!({
            "$schema": "https://json-schema.org/draft/2020-12/schema",
            "$id": "https://contoso.com/schemas/object.json",
            "type": "object",
            "$defs": {
                "foo": {
                    "$id": "https://contoso.com/schemas/foo.json",
                    "title": "Foo"
                }
            }
        });
        let ref expected = json_schema!({
            "$id": "https://contoso.com/schemas/foo.json",
            "title": "Foo"
        });
        assert_eq!(
            schema.get_defs_subschema_from_reference("https://contoso.com/schemas/foo.json").unwrap(),
            expected
        );
    }
}

#[cfg(test)] mod get_defs_subschema_from_reference_mut {
    use schemars::json_schema;

    use crate::schema_utility_extensions::SchemaUtilityExtensions;

    #[test] fn when_defs_keyword_missing() {
        let ref mut schema = json_schema!({
            "title": "missing defs"
        });
        assert_eq!(
            schema.get_defs_subschema_from_reference_mut("#/$defs/first"),
            None
        );
    }
    #[test] fn when_defs_keyword_is_not_object() {
        let ref mut schema = json_schema!({
            "title": "missing defs",
            "$defs": "invalid"
        });
        assert_eq!(
            schema.get_defs_subschema_from_reference_mut("#/$defs/first"),
            None
        );
    }
    #[test] fn when_defs_keyword_is_object_and_entry_missing() {
        let ref mut schema = json_schema!({
            "title": "missing defs",
            "$defs": {
                "second": {
                    "title": "second value"
                }
            }
        });
        assert_eq!(
            schema.get_defs_subschema_from_reference_mut("#/$defs/first"),
            None
        );
    }
    #[test] fn when_defs_keyword_is_object_and_entry_is_not_object() {
        let ref mut schema = json_schema!({
            "title": "missing defs",
            "$defs": {
                "first": "invalid"
            }
        });
        assert_eq!(
            schema.get_defs_subschema_from_reference_mut("#/$defs/first"),
            None
        );
    }
    #[test] fn with_defs_pointer_reference() {
        let ref mut schema = json_schema!({
            "$schema": "https://json-schema.org/draft/2020-12/schema",
            "$id": "https://contoso.com/schemas/object.json",
            "type": "object",
            "$defs": {
                "foo": {
                    "$id": "https://contoso.com/schemas/foo.json",
                    "title": "Foo"
                }
            }
        });
        let ref mut expected = json_schema!({
            "$id": "https://contoso.com/schemas/foo.json",
            "title": "Foo"
        });

        assert_eq!(
            schema.get_defs_subschema_from_reference_mut("#/$defs/foo"),
            Some(expected)
        );
    }
    #[test] fn with_absolute_id_uri_reference() {
        let ref mut schema = json_schema!({
            "$schema": "https://json-schema.org/draft/2020-12/schema",
            "$id": "https://contoso.com/schemas/object.json",
            "type": "object",
            "$defs": {
                "foo": {
                    "$id": "https://contoso.com/schemas/foo.json",
                    "title": "Foo"
                }
            }
        });
        let ref mut expected = json_schema!({
            "$id": "https://contoso.com/schemas/foo.json",
            "title": "Foo"
        });
        assert_eq!(
            schema.get_defs_subschema_from_reference_mut("/schemas/foo.json").unwrap(),
            expected
        );
    }
    #[test] fn with_relative_id_uri_reference() {
        let ref mut schema = json_schema!({
            "$schema": "https://json-schema.org/draft/2020-12/schema",
            "$id": "https://contoso.com/schemas/object.json",
            "type": "object",
            "$defs": {
                "foo": {
                    "$id": "https://contoso.com/schemas/foo.json",
                    "title": "Foo"
                }
            }
        });
        let ref mut expected = json_schema!({
            "$id": "https://contoso.com/schemas/foo.json",
            "title": "Foo"
        });
        assert_eq!(
            schema.get_defs_subschema_from_reference_mut("https://contoso.com/schemas/foo.json").unwrap(),
            expected
        );
    }
}

#[cfg(test)] mod get_bundled_schema_resource_defs_key {
    use schemars::json_schema;

    use crate::schema_utility_extensions::SchemaUtilityExtensions;

    #[test] fn when_defs_not_defined() {
        let schema = &json_schema!({
            "$id": "https://contoso.com/schemas/example.json",
            "properties": {
                "foo": { "$ref": "#/$defs/foo" },
                "bar": { "$ref": "/schemas/properties/bar.json" },
                "baz": { "$ref": "https://contoso.com/schemas/properties/baz.json" },
            }
        });
        pretty_assertions::assert_eq!(
            schema.get_bundled_schema_resource_defs_key(
                &"https://contoso.com/schemas/properties/bar.json".to_string()
            ),
            None
        );
    }

    #[test] fn when_defs_not_contains_bundled_resource() {
        let schema = &json_schema!({
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
            },
        });
        pretty_assertions::assert_eq!(
            schema.get_bundled_schema_resource_defs_key(
                &"https://contoso.com/schemas/properties/bar.json".to_string()
            ),
            None
        );
    }

    #[test] fn when_defs_contains_bundled_resource() {
        let schema = &json_schema!({
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
            },
        });
        pretty_assertions::assert_eq!(
            schema.get_bundled_schema_resource_defs_key(
                &"https://contoso.com/schemas/properties/foo.json".to_string()
            ),
            Some(&"foo".to_string())
        );
    }
}

#[cfg(test)] mod insert_defs_subschema {
    #[test] fn when_defs_keyword_missing() {}
    #[test] fn when_defs_keyword_is_not_object() {}
    #[test] fn when_defs_keyword_is_object_and_entry_not_exist() {}
    #[test] fn when_defs_keyword_is_object_and_entry_exists() {}
}

#[cfg(test)] mod remove_bundled_schema_resources {
    use schemars::json_schema;

    use crate::schema_utility_extensions::SchemaUtilityExtensions;

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
        schema.remove_bundled_schema_resources();
        let actual = serde_json::to_string_pretty(schema).unwrap();

        pretty_assertions::assert_eq!(actual, expected);
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
        schema.remove_bundled_schema_resources();
        let actual = serde_json::to_string_pretty(schema).unwrap();

        pretty_assertions::assert_eq!(actual, expected);
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
        schema.remove_bundled_schema_resources();
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

        pretty_assertions::assert_eq!(actual, expected);
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
        schema.remove_bundled_schema_resources();
        let actual = serde_json::to_string_pretty(schema).unwrap();
        let expected = serde_json::to_string_pretty(&json_schema!({
            "$id": "https://contoso.com/schemas/example.json",
            "properties": {
                "foo": { "$ref": "#/$defs/foo" },
                "bar": { "$ref": "/schemas/properties/bar.json" },
                "baz": { "$ref": "https://contoso.com/schemas/properties/baz.json" },
            },
        })).unwrap();

        pretty_assertions::assert_eq!(actual, expected);
    }
}

#[cfg(test)] mod rename_defs_subschema_for_reference {
    use pretty_assertions::assert_eq;
    use schemars::json_schema;

    use crate::schema_utility_extensions::SchemaUtilityExtensions;

    #[test] fn when_defs_not_defined() {
        let ref mut schema = json_schema!({
            "$id": "https://contoso.com/schemas/test.json"
        });
        let expected = json_schema!({
            "$id": "https://contoso.com/schemas/test.json"
        });
        schema.rename_defs_subschema_for_reference("#/$defs/not_exist", "not_exist");
        assert_eq!(schema.clone(), expected);
    }
    #[test] fn when_defs_subschema_not_defined() {
        let ref mut schema = json_schema!({
            "$id": "https://contoso.com/schemas/test.json",
            "$defs": {
                "foo": {
                    "$id": "https://contoso.com/schemas/foo.json"
                }
            }

        });
        let expected = json_schema!({
            "$id": "https://contoso.com/schemas/test.json",
            "$defs": {
                "foo": {
                    "$id": "https://contoso.com/schemas/foo.json"
                }
            }
        });
        schema.rename_defs_subschema_for_reference("#/$defs/not_exist", "not_exist");
        assert_eq!(schema.clone(), expected);
    }

    #[test] fn rename_by_defs_pointer_reference() {
        let ref mut schema = json_schema!({
            "$id": "https://contoso.com/schemas/test.json",
            "$defs": {
                "foo": {
                    "$id": "https://contoso.com/schemas/foo.json"
                }
            }

        });
        let expected = json_schema!({
            "$id": "https://contoso.com/schemas/test.json",
            "$defs": {
                "https://contoso.com/schemas/foo.json": {
                    "$id": "https://contoso.com/schemas/foo.json"
                }
            }
        });
        schema.rename_defs_subschema_for_reference(
            "#/$defs/foo",
            "https://contoso.com/schemas/foo.json"
        );
        assert_eq!(schema.clone(), expected);
    }

    #[test] fn rename_by_absolute_id_uri() {
        let ref mut schema = json_schema!({
            "$id": "https://contoso.com/schemas/test.json",
            "$defs": {
                "foo": {
                    "$id": "https://contoso.com/schemas/foo.json"
                }
            }

        });
        let expected = json_schema!({
            "$id": "https://contoso.com/schemas/test.json",
            "$defs": {
                "https://contoso.com/schemas/foo.json": {
                    "$id": "https://contoso.com/schemas/foo.json"
                }
            }
        });
        schema.rename_defs_subschema_for_reference(
            "https://contoso.com/schemas/foo.json",
            "https://contoso.com/schemas/foo.json"
        );
        assert_eq!(schema.clone(), expected);
    }
    #[test] fn rename_by_relative_id_uri() {
        let ref mut schema = json_schema!({
            "$id": "https://contoso.com/schemas/test.json",
            "$defs": {
                "foo": {
                    "$id": "https://contoso.com/schemas/foo.json"
                }
            }

        });
        let expected = json_schema!({
            "$id": "https://contoso.com/schemas/test.json",
            "$defs": {
                "https://contoso.com/schemas/foo.json": {
                    "$id": "https://contoso.com/schemas/foo.json"
                }
            }
        });
        schema.rename_defs_subschema_for_reference(
            "/schemas/foo.json",
            "https://contoso.com/schemas/foo.json"
        );
        assert_eq!(schema.clone(), expected);
    }
}

#[cfg(test)] mod get_properties {
    use pretty_assertions::assert_eq;
    use schemars::json_schema;
    use serde_json::json;

    use crate::schema_utility_extensions::SchemaUtilityExtensions;

    #[test] fn when_properties_keyword_missing() {
        let ref schema = json_schema!({
            "title": "Missing properties"
        });
        assert_eq!(schema.get_properties(), None);
    }
    #[test] fn when_properties_keyword_is_not_object() {
        let ref schema = json_schema!({
            "properties": "invalid"
        });
        assert_eq!(schema.get_properties(), None);
    }
    #[test] fn when_properties_keyword_is_object() {
        let ref properties = json!({
            "foo": {
                "title": "Foo property"
            }
        });
        let ref schema = json_schema!({
            "properties": properties
        });
        assert_eq!(
            schema.get_properties().unwrap(),
            properties.as_object().unwrap()
        );
    }
}

#[cfg(test)] mod get_properties_mut {
    use pretty_assertions::assert_eq;
    use schemars::json_schema;
    use serde_json::json;

    use crate::schema_utility_extensions::SchemaUtilityExtensions;

    #[test] fn when_properties_keyword_missing() {
        let ref mut schema = json_schema!({
            "title": "Missing properties"
        });
        assert_eq!(schema.get_properties_mut(), None);
    }
    #[test] fn when_properties_keyword_is_not_object() {
        let ref mut schema = json_schema!({
            "properties": "invalid"
        });
        assert_eq!(schema.get_properties_mut(), None);
    }
    #[test] fn when_properties_keyword_is_object() {
        let ref mut properties = json!({
            "foo": {
                "title": "Foo property"
            }
        });
        let ref mut schema = json_schema!({
            "properties": properties
        });
        assert_eq!(
            schema.get_properties_mut().unwrap(),
            properties.as_object_mut().unwrap()
        );
    }
}

#[cfg(test)] mod get_property_subschema {
    use core::option::Option::None;

    use pretty_assertions::assert_eq;
    use schemars::json_schema;

    use crate::schema_utility_extensions::SchemaUtilityExtensions;

    #[test] fn when_properties_keyword_missing() {
        let ref schema = json_schema!({
            "title": "Missing properties"
        });
        assert_eq!(schema.get_property_subschema("foo"), None)
    }
    #[test] fn when_properties_keyword_is_not_object() {
        let ref schema = json_schema!({
            "properties": "Invalid"
        });
        assert_eq!(schema.get_property_subschema("foo"), None)
    }
    #[test] fn when_given_property_missing() {
        let ref schema = json_schema!({
            "properties": {
                "bar": { "title": "Bar property" }
            }
        });
        assert_eq!(schema.get_property_subschema("foo"), None)
    }
    #[test] fn when_given_property_is_not_object() {
        let ref schema = json_schema!({
            "properties": {
                "foo": "invalid"
            }
        });
        assert_eq!(schema.get_property_subschema("foo"), None)
    }
    #[test] fn when_given_property_is_object() {
        let ref property = json_schema!({
            "title": "Foo property"
        });
        let ref schema = json_schema!({
            "properties": {
                "foo": property
            }
        });
        assert_eq!(
            schema.get_property_subschema("foo").unwrap(),
            property
        )
    }
}

#[cfg(test)] mod get_property_subschema_mut {
    use core::option::Option::None;

    use pretty_assertions::assert_eq;
    use schemars::json_schema;

    use crate::schema_utility_extensions::SchemaUtilityExtensions;

    #[test] fn when_properties_keyword_missing() {
        let ref mut schema = json_schema!({
            "title": "Missing properties"
        });
        assert_eq!(schema.get_property_subschema_mut("foo"), None)
    }
    #[test] fn when_properties_keyword_is_not_object() {
        let ref mut schema = json_schema!({
            "properties": "Invalid"
        });
        assert_eq!(schema.get_property_subschema_mut("foo"), None)
    }
    #[test] fn when_given_property_missing() {
        let ref mut schema = json_schema!({
            "properties": {
                "bar": { "title": "Bar property" }
            }
        });
        assert_eq!(schema.get_property_subschema_mut("foo"), None)
    }
    #[test] fn when_given_property_is_not_object() {
        let ref mut schema = json_schema!({
            "properties": {
                "foo": "invalid"
            }
        });
        assert_eq!(schema.get_property_subschema_mut("foo"), None)
    }
    #[test] fn when_given_property_is_object() {
        let ref mut property = json_schema!({
            "title": "Foo property"
        });
        let ref mut schema = json_schema!({
            "properties": {
                "foo": property
            }
        });
        assert_eq!(
            schema.get_property_subschema_mut("foo").unwrap(),
            property
        )
    }
}

#[cfg(test)] mod get_references_to_bundled_schema_resource {
    use schemars::json_schema;

    use crate::schema_utility_extensions::SchemaUtilityExtensions;

    #[test] fn when_defs_not_defined() {
        let schema = &json_schema!({
            "$id": "https://contoso.com/schemas/example.json",
            "properties": {
                "fragment_uri": { "$ref": "#/$defs/foo" },
                "absolute_uri": { "$ref": "https://contoso.com/schemas/properties/foo.json" },
                "relative_uri": { "$ref": "/schemas/properties/foo.json" },
            },
        });

        assert!(
            schema.get_references_to_bundled_schema_resource(
                "https://contoso.com/schemas/properties/foo.json"
            ).is_empty()
        );
    }

    #[test] fn when_bundled_resource_defined() {
        let schema = &json_schema!({
            "$id": "https://contoso.com/schemas/example.json",
            "properties": {
                "fragment_uri": { "$ref": "#/$defs/foo" },
                "absolute_uri": { "$ref": "https://contoso.com/schemas/properties/foo.json" },
                "relative_uri": { "$ref": "/schemas/properties/foo.json" },
            },
            "$defs": {
                "foo": {
                    "$id": "https://contoso.com/schemas/properties/foo.json",
                    "type": "string",
                },
            },
        });

        let expected: std::collections::HashSet<&str> = vec![
            "#/$defs/foo",
            "https://contoso.com/schemas/properties/foo.json",
            "/schemas/properties/foo.json",
        ].iter().cloned().collect();

        pretty_assertions::assert_eq!(
            schema.get_references_to_bundled_schema_resource(
                "https://contoso.com/schemas/properties/foo.json"
            ),
            expected
        )
    }
}

#[test] fn to_value_with_stable_order() {
    let schema = &schemars::json_schema!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "$id": "https://contoso.com/schemas/example.json",
        "type": "object",
        "properties": {
            "foo": { "$ref": "#/$defs/foo" },
            "bar": { "type": "boolean" },
        },
        "$defs": {
            "foo": { "type": "string" },
        },
    });
    let actual = &schema.to_value_with_stable_order();
    let expected = &serde_json::json!({
        "$defs": {
            "foo": { "type": "string" },
        },
        "$id": "https://contoso.com/schemas/example.json",
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "properties": {
            "bar": { "type": "boolean" },
            "foo": { "$ref": "#/$defs/foo" },
        },
        "type": "object",
    });

    pretty_assertions::assert_eq!(
        serde_json::to_string_pretty(actual).unwrap(),
        serde_json::to_string_pretty(expected).unwrap()
    )
}

#[cfg(test)] mod canonicalize_refs_and_defs_for_bundled_resources {
    use schemars::json_schema;

    use crate::schema_utility_extensions::SchemaUtilityExtensions;

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
        schema.canonicalize_refs_and_defs_for_bundled_resources();
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
        schema.canonicalize_refs_and_defs_for_bundled_resources();
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

        pretty_assertions::assert_eq!(actual, expected);
    }
}