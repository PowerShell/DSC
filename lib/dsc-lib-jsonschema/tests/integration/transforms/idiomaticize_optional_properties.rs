// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use dsc_lib_jsonschema::schema_utility_extensions::SchemaUtilityExtensions;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[schemars(
    title = "StructField.definition",
    extend(
        "pattern" = "^\\w+$"
    )
)]
pub struct StructField(String);

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[schemars(
    inline,
    title = "StructFieldInlined.definition",
    extend(
        "pattern" = "^\\w+$"
    )
)]
pub struct StructFieldInlined(String);

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all="camelCase")]
#[schemars(
    title = "EnumField.definition",
)]
pub enum EnumField {
    Foo,
    Bar,
    Baz
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all="camelCase")]
#[schemars(
    inline,
    title = "EnumFieldInlined.definition",
)]
pub enum EnumFieldInlined {
    Foo,
    Bar,
    Baz
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct Example {
    #[schemars(title = "struct_field.field")]
    pub struct_field: Option<StructField>,
    #[schemars(title = "struct_field_inlined.field")]
    pub struct_field_inlined: Option<StructFieldInlined>,
    #[schemars(title = "enum_field.field")]
    pub enum_field: Option<EnumField>,
    #[schemars(title = "enum_field_inlined.field")]
    pub enum_field_inlined: Option<EnumFieldInlined>,
    #[schemars(title = "primitive_field.field")]
    pub primitive_field: Option<String>,
}

#[cfg(test)] mod without_transform {
    use schemars::json_schema;

    use super::*;

    fn test_field(name: &str, expected: &schemars::Schema) {
        let parent_schema = schemars::schema_for!(Example);
        let field_schema = parent_schema.get_property_subschema(name)
            .expect(&format!("schema should define 'properties.{name}' as subschema"));

        pretty_assertions::assert_eq!(
            serde_json::to_string_pretty(field_schema).unwrap(),
            serde_json::to_string_pretty(expected).unwrap()
        );
    }

    #[test] fn field_defined_as_option_wrapping_struct() {
        test_field("struct_field", &json_schema!({
            "title": "struct_field.field",
            "anyOf": [
                { "$ref": "#/$defs/StructField" },
                { "type": "null" },
            ]
        }));
    }

    #[test] fn field_defined_as_option_wrapping_struct_inlined() {
        test_field("struct_field_inlined", &json_schema!({
            "title": "struct_field_inlined.field",
            "type": ["string", "null"],
            "pattern": "^\\w+$"
        }));
    }

    #[test] fn field_defined_as_option_wrapping_enum() {
        test_field("enum_field", &json_schema!({
            "title": "enum_field.field",
            "anyOf": [
                { "$ref": "#/$defs/EnumField" },
                { "type": "null" },
            ]
        }));
    }

    #[test] fn field_defined_as_option_wrapping_enum_inlined() {
        test_field("enum_field_inlined", &json_schema!({
            "title": "enum_field_inlined.field",
            "type": ["string", "null"],
            "enum": ["foo", "bar", "baz", null]
        }));
    }

    #[test] fn field_defined_as_option_wrapping_primitive() {
        test_field("primitive_field", &json_schema!({
            "title": "primitive_field.field",
            "type": ["string", "null"],
        }));
    }
}

#[cfg(test)] mod with_transform {
    use dsc_lib_jsonschema::transforms::idiomaticize_optional_properties;
    use schemars::json_schema;

    use super::*;

    fn test_field(name: &str, expected: &schemars::Schema) {
        let mut parent_schema = schemars::schema_for!(Example);
        idiomaticize_optional_properties(&mut parent_schema);

        let field_schema = parent_schema.get_property_subschema(name)
            .expect(&format!("schema should define 'properties.{name}' as subschema"));

        pretty_assertions::assert_eq!(
            serde_json::to_string_pretty(field_schema).unwrap(),
            serde_json::to_string_pretty(expected).unwrap()
        );
    }

    #[test] fn field_defined_as_option_wrapping_struct() {
        test_field("struct_field", &json_schema!({
            "title": "struct_field.field",
            "$ref": "#/$defs/StructField"
        }));
    }

    #[test] fn field_defined_as_option_wrapping_struct_inlined() {
        test_field("struct_field_inlined", &json_schema!({
            "title": "struct_field_inlined.field",
            "type": "string",
            "pattern": "^\\w+$"
        }));
    }

    #[test] fn field_defined_as_option_wrapping_enum() {
        test_field("enum_field", &json_schema!({
            "title": "enum_field.field",
            "$ref": "#/$defs/EnumField"
        }));
    }

    #[test] fn field_defined_as_option_wrapping_enum_inlined() {
        test_field("enum_field_inlined", &json_schema!({
            "title": "enum_field_inlined.field",
            "type": "string",
            "enum": ["foo", "bar", "baz"]
        }));
    }

    #[test] fn field_defined_as_option_wrapping_primitive() {
        test_field("primitive_field", &json_schema!({
            "title": "primitive_field.field",
            "type": "string",
        }));
    }
}