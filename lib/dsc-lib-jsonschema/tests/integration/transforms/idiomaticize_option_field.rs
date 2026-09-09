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

fn test_field<ContainerType: JsonSchema>(name: &str, expected: &schemars::Schema) {
    let parent_schema = schemars::schema_for!(ContainerType);
    let field_schema = parent_schema.get_property_subschema(name)
        .expect(&format!("schema should define 'properties.{name}' as subschema"));

    pretty_assertions::assert_eq!(
        serde_json::to_string_pretty(field_schema).unwrap(),
        serde_json::to_string_pretty(expected).unwrap()
    );
}

#[cfg(test)] mod without_transform {
    use schemars::{json_schema, schema_for};
    use serde_json::json;

    use super::*;

    #[derive(Debug, Serialize, Deserialize, JsonSchema)]
    pub struct Example {
        pub struct_field: Option<StructField>,
        pub struct_field_inlined: Option<StructFieldInlined>,
        pub enum_field: Option<EnumField>,
        pub enum_field_inlined: Option<EnumFieldInlined>,
        pub primitive_field: Option<String>,
    }

    #[test] fn field_defined_as_option_wrapping_struct() {
        test_field::<Example>("struct_field", &json_schema!({
            "anyOf": [
                { "$ref": "#/$defs/StructField"},
                { "type": "null" }
            ]
        }));
    }
    #[test] fn field_defined_as_option_wrapping_struct_inlined() {
        let ref mut expected = schema_for!(StructFieldInlined);
        // Schemars inserts the null type
        expected.insert("type".to_string(), json!(["string", "null"]));
        // Inlined schemas drop the `$schema` keyword
        expected.remove("$schema");
        test_field::<Example>("struct_field_inlined", expected);
    }
    #[test] fn field_defined_as_option_wrapping_enum() {
        test_field::<Example>("enum_field", &json_schema!({
            "anyOf": [
                { "$ref": "#/$defs/EnumField"},
                { "type": "null" }
            ]
        }));
    }
    #[test] fn field_defined_as_option_wrapping_enum_inlined() {
        let ref mut expected = schema_for!(EnumFieldInlined);
        // Schemars adds null type automatically
        expected.insert("type".to_string(), json!(["string", "null"]));
        // Schemars adds `null` as valid enum value
        expected.get_keyword_as_array_mut("enum").map(|v| v.push(json!(null)));
        // Inlined schemas drop the `$schema` keyword
        expected.remove("$schema");

        test_field::<Example>("enum_field_inlined", expected);
    }
    #[test] fn field_defined_as_option_wrapping_primitive() {
        test_field::<Example>("primitive_field", &json_schema!({
            "type": ["string", "null"]
        }));
    }
}

#[cfg(test)] mod with_transform {
    use super::*;
    use dsc_lib_jsonschema::transforms::idiomaticize_option_field;

    #[cfg(test)] mod without_field_keywords {
        use schemars::{json_schema, schema_for};

        use super::*;

        #[derive(Debug, Serialize, Deserialize, JsonSchema)]
        pub struct Example {
            #[schemars(transform = idiomaticize_option_field)]
            pub struct_field: Option<StructField>,
            #[schemars(transform = idiomaticize_option_field)]
            pub struct_field_inlined: Option<StructFieldInlined>,
            #[schemars(transform = idiomaticize_option_field)]
            pub enum_field: Option<EnumField>,
            #[schemars(transform = idiomaticize_option_field)]
            pub enum_field_inlined: Option<EnumFieldInlined>,
            #[schemars(transform = idiomaticize_option_field)]
            pub primitive_field: Option<String>,
        }

        #[test] fn field_defined_as_option_wrapping_struct() {
            test_field::<Example>("struct_field", &json_schema!({
                "$ref": "#/$defs/StructField"
            }));
        }
        #[test] fn field_defined_as_option_wrapping_struct_inlined() {
            let ref mut expected = schema_for!(StructFieldInlined);
            // Inlined schemas drop the `$schema` keyword
            expected.remove("$schema");
            test_field::<Example>("struct_field_inlined", expected);
        }
        #[test] fn field_defined_as_option_wrapping_enum() {
            test_field::<Example>("enum_field", &json_schema!({
                "$ref": "#/$defs/EnumField"
            }));
        }
        #[test] fn field_defined_as_option_wrapping_enum_inlined() {
            let ref mut expected = schema_for!(EnumFieldInlined);
            // Inlined schemas drop the `$schema` keyword
            expected.remove("$schema");

            test_field::<Example>("enum_field_inlined", expected);
        }
        #[test] fn field_defined_as_option_wrapping_primitive() {
            test_field::<Example>("primitive_field", &json_schema!({
                "type": "string"
            }));
        }
    }

    #[cfg(test)] mod with_field_keywords {
        use schemars::{json_schema, schema_for};
        use serde_json::json;

        use super::*;

        #[derive(Debug, Serialize, Deserialize, JsonSchema)]
        pub struct Example {
            #[schemars(
                title = "struct_field.field",
                transform = idiomaticize_option_field
            )]
            pub struct_field: Option<StructField>,
            #[schemars(
                title = "struct_field_inlined.field",
                transform = idiomaticize_option_field
            )]
            pub struct_field_inlined: Option<StructFieldInlined>,
            #[schemars(
                title = "enum_field.field",
                transform = idiomaticize_option_field
            )]
            pub enum_field: Option<EnumField>,
            #[schemars(
                title = "enum_field_inlined.field",
                transform = idiomaticize_option_field
            )]
            pub enum_field_inlined: Option<EnumFieldInlined>,
            #[schemars(
                title = "primitive_field.field",
                transform = idiomaticize_option_field
            )]
            pub primitive_field: Option<String>,
        }

        #[test] fn field_defined_as_option_wrapping_struct() {
            test_field::<Example>("struct_field", &json_schema!({
                "$ref": "#/$defs/StructField",
                "title": "struct_field.field"
            }));
        }
        #[test] fn field_defined_as_option_wrapping_struct_inlined() {
            let ref mut expected = schema_for!(StructFieldInlined);
            // Inlined schemas drop the `$schema` keyword
            expected.remove("$schema");
            // Should have the attributed `title` keyword
            expected.insert("title".to_string(), json!("struct_field_inlined.field"));
            test_field::<Example>("struct_field_inlined", expected);
        }
        #[test] fn field_defined_as_option_wrapping_enum() {
            test_field::<Example>("enum_field", &json_schema!({
                "$ref": "#/$defs/EnumField",
                "title": "enum_field.field"
            }));
        }
        #[test] fn field_defined_as_option_wrapping_enum_inlined() {
            let ref mut expected = schema_for!(EnumFieldInlined);
            // Inlined schemas drop the `$schema` keyword
            expected.remove("$schema");
            // Should have the attributed `title` keyword
            expected.insert("title".to_string(), json!("enum_field_inlined.field"));

            test_field::<Example>("enum_field_inlined", expected);
        }
        #[test] fn field_defined_as_option_wrapping_primitive() {
            test_field::<Example>("primitive_field", &json_schema!({
                "type": "string",
                "title": "primitive_field.field"
            }));
        }
    }

}