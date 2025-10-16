// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! Defines various functions that implement the [`Transform`] trait for [`schemars`], enabling you
//! modify generated JSON Schemas.
//!
//! [`Transform`]: schemars::transform

use core::{assert, cmp::PartialEq};
use std::{ops::Index};
use schemars::Schema;
use serde_json::{self, json, Map, Value};

use crate::vscode::VSCODE_KEYWORDS;

/// Munges the generated schema for externally tagged enums into an idiomatic object schema.
///
/// Schemars generates the schema for externally tagged enums as a schema with the `oneOf`
/// keyword where every tag is a different item in the array. Each item defines a type with a
/// single property, requires that property, and disallows specifying any other properties.
///
/// This transformer returns the schema as a single object schema with each of the tags defined
/// as properties. It sets both the `minProperties` and `maxProperties` keywords to `1`. This
/// is more idiomatic, shorter to read and parse, easier to reason about, and matches the
/// underlying data semantics more accurately.
///
/// This transformer should _only_ be used on externally tagged enums. You must specify it with the
/// [schemars `transform()` attribute][`transform`].
///
/// # Examples
///
/// The following struct derives [`JsonSchema`] without specifying the [`transform`] attribute
/// with [`idiomaticize_externally_tagged_enum`]:
///
/// ```
/// use pretty_assertions::assert_eq;
/// use serde_json;
/// use schemars::{schema_for, JsonSchema, json_schema};
/// #[derive(JsonSchema)]
/// pub enum ExternallyTaggedEnum {
///     Name(String),
///     Count(f32),
/// }
///
/// let generated_schema = schema_for!(ExternallyTaggedEnum);
/// let expected_schema  = json_schema!({
///     "$schema": "https://json-schema.org/draft/2020-12/schema",
///     "title": "ExternallyTaggedEnum",
///     "oneOf": [
///         {
///             "type": "object",
///             "properties": {
///                 "Name": {
///                     "type": "string"
///                 }
///             },
///             "additionalProperties": false,
///             "required": ["Name"]
///         },
///         {
///             "type": "object",
///             "properties": {
///                 "Count": {
///                     "type": "number",
///                     "format": "float"
///                 }
///             },
///             "additionalProperties": false,
///             "required": ["Count"]
///         }
///     ]
/// });
/// assert_eq!(generated_schema, expected_schema);
/// ```
///
/// While the derived schema _does_ effectively validate the enum, it's difficult to understand
/// without deep familiarity with JSON Schema. Compare it to the same enum with the
/// [`idiomaticize_externally_tagged_enum`] transform applied:
///
/// ```
/// use pretty_assertions::assert_eq;
/// use serde_json;
/// use schemars::{schema_for, JsonSchema, json_schema};
/// use dsc_lib_jsonschema::transforms::idiomaticize_externally_tagged_enum;
///
/// #[derive(JsonSchema)]
/// #[schemars(transform = idiomaticize_externally_tagged_enum)]
/// pub enum ExternallyTaggedEnum {
///     Name(String),
///     Count(f32),
/// }
///
/// let generated_schema = schema_for!(ExternallyTaggedEnum);
/// let expected_schema  = json_schema!({
///     "$schema": "https://json-schema.org/draft/2020-12/schema",
///     "title": "ExternallyTaggedEnum",
///     "type": "object",
///     "properties": {
///         "Name": {
///             "type": "string"
///         },
///         "Count": {
///             "type": "number",
///             "format": "float"
///         }
///     },
///     "minProperties": 1,
///     "maxProperties": 1,
///     "additionalProperties": false
/// });
/// assert_eq!(generated_schema, expected_schema);
/// ```
///
/// The transformed schema is shorter, clearer, and idiomatic for JSON Schema draft 2019-09 and
/// later. It validates values as effectively as the default output for an externally tagged
/// enum, but is easier for your users and integrating developers to understand and work
/// with.
///
/// # Panics
///
/// This transform panics when called against a generated schema that doesn't define the `oneOf`
/// keyword. Schemars uses the `oneOf` keyword when generating subschemas for externally tagged
/// enums. This transform panics on an invalid application of the transform to prevent unexpected
/// behavior for the schema transformation. This ensures invalid applications are caught during
/// development and CI instead of shipping broken schemas.
///
/// [`JsonSchema`]: schemars::JsonSchema
/// [`transform`]: derive@schemars::JsonSchema
pub fn idiomaticize_externally_tagged_enum(schema: &mut Schema) {
    // First, retrieve the oneOf keyword entries. If this transformer was called against an invalid
    // schema or subschema, it should fail fast.
    let one_ofs = schema.get("oneOf")
        .unwrap_or_else(|| panic_t!(
            "transforms.idiomaticize_externally_tagged_enum.applies_to",
            transforming_schema = serde_json::to_string_pretty(schema).unwrap()
        ))
        .as_array()
        .unwrap_or_else(|| panic_t!(
            "transforms.idiomaticize_externally_tagged_enum.oneOf_array",
            transforming_schema = serde_json::to_string_pretty(schema).unwrap()
        ));
    // Initialize the map of properties to fill in when introspecting on the items in the oneOf array.
    let mut properties_map = Map::new();

    for item in one_ofs {
        let item_data: Map<String, Value> = item.as_object()
            .unwrap_or_else(|| panic_t!(
                "transforms.idiomaticize_externally_tagged_enum.oneOf_item_as_object",
                transforming_schema = serde_json::to_string_pretty(schema).unwrap(),
                invalid_item = serde_json::to_string_pretty(item).unwrap()
            ))
            .clone();
        // If we're accidentally operating on an invalid schema, short-circuit.
        let item_data_type = item_data.get("type")
            .unwrap_or_else(|| panic_t!(
                "transforms.idiomaticize_externally_tagged_enum.oneOf_item_define_type",
                transforming_schema = serde_json::to_string_pretty(schema).unwrap(),
                invalid_item = serde_json::to_string_pretty(&item_data).unwrap()
            ))
            .as_str()
            .unwrap_or_else(|| panic_t!(
                "transforms.idiomaticize_externally_tagged_enum.oneOf_item_type_string",
                transforming_schema = serde_json::to_string_pretty(schema).unwrap(),
                invalid_item = serde_json::to_string_pretty(&item_data).unwrap()
            ));
        assert_t!(
            !item_data_type.ne("object"),
            "transforms.idiomaticize_externally_tagged_enum.oneOf_item_not_object_type",
            transforming_schema = serde_json::to_string_pretty(schema).unwrap(),
            invalid_item = serde_json::to_string_pretty(&item_data).unwrap(),
            invalid_type = item_data_type
        );
        // Retrieve the title and description from the top-level of the item, if any. Depending on
        // the implementation, these values might be set on the item, in the property, or both.
        let item_title = item_data.get("title");
        let item_desc = item_data.get("description");
        // Retrieve the property definitions. There should never be more than one property per item,
        // but this implementation doesn't guard against that edge case..
        let properties_data = item_data.get("properties")
            .unwrap_or_else(|| panic_t!(
                "transforms.idiomaticize_externally_tagged_enum.oneOf_item_properties_missing",
                transforming_schema = serde_json::to_string_pretty(schema).unwrap(),
                invalid_item = serde_json::to_string_pretty(&item_data).unwrap(),
            ))
            .as_object()
            .unwrap_or_else(|| panic_t!(
                "transforms.idiomaticize_externally_tagged_enum.oneOf_item_properties_not_object",
                transforming_schema = serde_json::to_string_pretty(schema).unwrap(),
                invalid_item = serde_json::to_string_pretty(&item_data).unwrap(),
            ))
            .clone();
        for property_name in properties_data.keys() {
            // Retrieve the property definition to munge as needed.
            let mut property_data = properties_data.get(property_name)
                .unwrap() // can't fail because we're iterating on keys in the map
                .as_object()
                .unwrap_or_else(|| panic_t!(
                    "transforms.idiomaticize_externally_tagged_enum.oneOf_item_properties_entry_not_object",
                    transforming_schema = serde_json::to_string_pretty(schema).unwrap(),
                    invalid_item = serde_json::to_string_pretty(&item_data).unwrap(),
                    name = property_name
                ))
                .clone();
            // Process the annotation keywords. If they are defined on the item but not the property,
            // insert the item-defined keywords into the property data.
            if let Some(t) = item_title && property_data.get("title").is_none() {
                    property_data.insert("title".into(), t.clone());
            }
            if let Some(d) = item_desc && property_data.get("description").is_none() {
                property_data.insert("description".into(), d.clone());
            }
            for keyword in VSCODE_KEYWORDS {
                if let Some(keyword_value) = item_data.get(keyword) && property_data.get(keyword).is_none() {
                    property_data.insert(keyword.to_string(), keyword_value.clone());
                }
            }
            // Insert the processed property into the top-level properties definition.
            properties_map.insert(property_name.into(), serde_json::Value::Object(property_data));
        }
    }
    // Replace the oneOf array with an idiomatic object schema definition
    schema.remove("oneOf");
    schema.insert("type".to_string(), json!("object"));
    schema.insert("minProperties".to_string(), json!(1));
    schema.insert("maxProperties".to_string(), json!(1));
    schema.insert("additionalProperties".to_string(), json!(false));
    schema.insert("properties".to_string(), properties_map.into());
}

/// Munges the generated schema for enums that only define string variants into an idiomatic string
/// schema.
///
/// When an enum defines string variants without documenting any of the variants, Schemars correctly
/// generates the schema as a `string` subschema with the `enum` keyword. However, if you define any
/// documentation keywords for any variants, Schemars generates the schema with the `oneOf` keyword
/// where every variant is a different item in the array. Each item defines a type with a constant
/// string value, and all annotation keywords for that variant.
///
/// This transformer returns the schema as a single string schema with each of the variants defined
/// as an item in the `enum` keyword. It hoists the per-variant documentation to the extended
/// keywords recognized by VS Code: `enumDescriptions` and `enumMarkdownDescriptions`. This is more
/// idiomatic, shorter to read and parse, easier to reason about, and matches the underlying data
/// semantics more accurately.
///
/// # Examples
///
/// The following struct derives [`JsonSchema`] without specifying the [`transform`] attribute
/// with [`idiomaticize_string_enum`]:
///
/// ```rust
/// use pretty_assertions::assert_eq;
/// use serde_json;
/// use schemars::{schema_for, JsonSchema, json_schema};
///
/// #[derive(JsonSchema)]
/// #[serde(rename_all="camelCase")]
/// enum StringEnum {
///     /// # foo-title
///     ///
///     ///foo-description
///     Foo,
///     /// # bar-title
///     ///
///     /// bar-description
///     Bar,
///     /// # baz-title
///     ///
///     /// baz-description
///     Baz
/// }
///
/// let generated_schema = schema_for!(StringEnum);
/// let expected_schema = json_schema!({
///     "$schema": "https://json-schema.org/draft/2020-12/schema",
///     "title": "StringEnum",
///     "oneOf": [
///         {
///             "type": "string",
///             "const": "foo",
///             "title": "foo-title",
///             "description": "foo-description"
///         },
///         {
///             "type": "string",
///             "const": "bar",
///             "title": "bar-title",
///             "description": "bar-description",
///         },
///         {
///             "type": "string",
///             "const": "baz",
///             "title": "baz-title",
///             "description": "baz-description",
///         }
///     ],
/// });
/// assert_eq!(generated_schema, expected_schema);
/// ```
///
/// While the derived schema _does_ effectively validate the enum, it's difficult to understand
/// without deep familiarity with JSON Schema. Compare it to the same enum with the
/// [`idiomaticize_string_enum`] transform applied:
///
/// ```rust
/// use pretty_assertions::assert_eq;
/// use serde_json;
/// use schemars::{schema_for, JsonSchema, json_schema};
/// use dsc_lib_jsonschema::transforms::idiomaticize_string_enum;
///
/// #[derive(JsonSchema)]
/// #[serde(rename_all="camelCase")]
/// #[schemars(transform = idiomaticize_string_enum)]
/// enum StringEnum {
///     /// # foo-title
///     ///
///     ///foo-description
///     Foo,
///     /// # bar-title
///     ///
///     /// bar-description
///     Bar,
///     /// # baz-title
///     ///
///     /// baz-description
///     Baz
/// }
///
/// let generated_schema = schema_for!(StringEnum);
/// let expected_schema = json_schema!({
///     "type": "string",
///     "enum": [
///         "foo",
///         "bar",
///         "baz"
///     ],
///     "enumDescriptions": [
///         "foo-description",
///         "bar-description",
///         "baz-description",
///     ],
///     "enumMarkdownDescriptions": [
///         "foo-description",
///         "bar-description",
///         "baz-description",
///     ],
///     "title": "StringEnum",
///     "$schema": "https://json-schema.org/draft/2020-12/schema",
/// });
/// assert_eq!(generated_schema, expected_schema);
/// ```
///
/// # Panics
///
/// If this transform is applied to a schema that defines the `enum` keyword, it immediately
/// returns without modifying the schema. Otherwise, it checks whether the schema defines the
/// `oneOf` keyword. If the generated schema doesn't define the `oneOf` keyword, this transform
/// panics.
///
/// Schemars uses the `oneOf` keyword when generating subschemas for string enums with annotation
/// keywords. This transform panics on an invalid application of the transform to prevent
/// unexpectedbehavior for the schema transformation. This ensures invalid applications are caught
/// during development and CI instead of shipping broken schemas.
///
/// [`JsonSchema`]: schemars::JsonSchema
/// [`transform`]: derive@schemars::JsonSchema#transform
pub fn idiomaticize_string_enum(schema: &mut Schema) {
    #![allow(clippy::too_many_lines)]
    // If this transform is called against a schema defining `enums`, there's nothing to do.
    if schema.get("enum").is_some() {
        return;
    }
    // First, retrieve the oneOf keyword entries. If this transformer was called against an invalid
    // schema or subschema, it should fail fast.
    let one_ofs = schema.get("oneOf")
        .unwrap_or_else(|| panic_t!(
            "transforms.idiomaticize_string_enum.applies_to",
            transforming_schema = serde_json::to_string_pretty(schema).unwrap()
        ))
        .as_array()
        .unwrap_or_else(|| panic_t!(
            "transforms.idiomaticize_string_enum.oneOf_array",
            transforming_schema = serde_json::to_string_pretty(schema).unwrap()
        ));
    // Initialize the vectors for enums, their descriptions, and their markdown descriptions.
    let mut enums: Vec<String> = Vec::with_capacity(one_ofs.len());
    let mut enum_descriptions: Vec<String> = Vec::with_capacity(one_ofs.len());
    let mut enum_markdown_descriptions: Vec<String> = Vec::with_capacity(one_ofs.len());

    // Iterate over the enums to add to the holding vectors.
    for (index, item) in one_ofs.iter().enumerate() {
        let item_data = item.as_object()
            .unwrap_or_else(|| panic_t!(
                "transforms.idiomaticize_string_enum.oneOf_item_as_object",
                transforming_schema = serde_json::to_string_pretty(schema).unwrap(),
                invalid_item = serde_json::to_string_pretty(item).unwrap()
            ))
            .clone();
        // If we're accidentally operating on an invalid schema, short-circuit.
        let item_data_type = item_data.get("type")
            .unwrap_or_else(|| panic_t!(
                "transforms.idiomaticize_string_enum.oneOf_item_define_type",
                transforming_schema = serde_json::to_string_pretty(schema).unwrap(),
                invalid_item = serde_json::to_string_pretty(&item_data).unwrap()
            ))
            .as_str()
            .unwrap_or_else(|| panic_t!(
                "transforms.idiomaticize_string_enum.oneOf_item_type_string",
                transforming_schema = serde_json::to_string_pretty(schema).unwrap(),
                invalid_item = serde_json::to_string_pretty(&item_data).unwrap()
            ));
        assert_t!(
            !item_data_type.ne("string"),
            "transforms.idiomaticize_string_enum.oneOf_item_not_string_type",
            transforming_schema = serde_json::to_string_pretty(schema).unwrap(),
            invalid_item = serde_json::to_string_pretty(&item_data).unwrap(),
            invalid_type = item_data_type
        );
        // Retrieve the title, description, and markdownDescription from the item, if any.
        let item_title = item_data.get("title").and_then(|v| v.as_str());
        let item_desc = item_data.get("description").and_then(|v| v.as_str());
        let item_md_desc = item_data.get("markdownDescription").and_then(|v| v.as_str());
        // Retrieve the value for the enum - schemars emits as a `const` for each item that has
        // docs, and an enum with a single value for non-documented enums.
        let item_enum: &str;
        if let Some(item_enum_value) = item_data.get("enum") {
            item_enum = item_enum_value.as_array()
                        .unwrap_or_else(|| panic_t!(
                            "transforms.idiomaticize_string_enum.oneOf_item_enum_not_array",
                            transforming_schema = serde_json::to_string_pretty(schema).unwrap(),
                            invalid_item = serde_json::to_string_pretty(&item_data).unwrap()
                        ))
                        .index(0)
                        .as_str()
                        .unwrap_or_else(|| panic_t!(
                            "transforms.idiomaticize_string_enum.oneOf_item_enum_item_not_string",
                            transforming_schema = serde_json::to_string_pretty(schema).unwrap(),
                            invalid_item = serde_json::to_string_pretty(&item_data).unwrap()
                        ));
        } else {
            item_enum = item_data.get("const")
                        .unwrap_or_else(|| panic_t!(
                            "transforms.idiomaticize_string_enum.oneOf_item_const_missing",
                            transforming_schema = serde_json::to_string_pretty(schema).unwrap(),
                            invalid_item = serde_json::to_string_pretty(&item_data).unwrap()
                        ))
                        .as_str()
                        .unwrap_or_else(|| panic_t!(
                            "transforms.idiomaticize_string_enum.oneOf_item_const_not_string",
                            transforming_schema = serde_json::to_string_pretty(schema).unwrap(),
                            invalid_item = serde_json::to_string_pretty(&item_data).unwrap()
                        ));
        }

        enums.insert(index, item_enum.to_string());

        // Define the enumDescription entry as description with title as fallback. If neither
        // keyword is defined, add as an empty string.
        let desc = match item_desc {
            Some(d) => d,
            None => item_title.unwrap_or_default(),
        };
        enum_descriptions.insert(index, desc.to_string());
        // Define the enumMarkdownDescription entry as markdownDescription with description
        // then title as fallback. If none of the keywords are defined, add as an empty string.
        let md_desc = match item_md_desc {
            Some(d) => d,
            None => desc,
        };
        enum_markdown_descriptions.insert(index, md_desc.to_string());
    }
    // Replace the oneOf array with an idiomatic object schema definition
    schema.remove("oneOf");
    schema.insert("type".to_string(), json!("string"));
    schema.insert("enum".to_string(), serde_json::to_value(enums).unwrap());
    if enum_descriptions.iter().any(|e| !e.is_empty()) {
        schema.insert(
            "enumDescriptions".to_string(),
            serde_json::to_value(enum_descriptions).unwrap()
        );
    }
    if enum_markdown_descriptions.iter().any(|e| !e.is_empty()) {
        schema.insert(
            "enumMarkdownDescriptions".to_string(),
            serde_json::to_value(enum_markdown_descriptions).unwrap()
        );
    }
}
