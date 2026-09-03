// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use schemars::Schema;
use serde_json::json;

use crate::schema_utility_extensions::SchemaUtilityExtensions;

/// Transforms the default generated schema for optional fields into a more idiomatic representation.
///
/// This transform is intended to be applied to the schema for a single `Option<T>` field.
/// It removes the `null` branch that schemars adds (via `type: ["…", "null"]`, `enum: […, null]`, or `anyOf`).
/// The field’s optionality should instead be expressed by omitting the property name from `required`.
/// 
/// # Panics
/// 
/// This transform panics if any apparently optional field doesn't define either:
/// 
/// - `type` as an array where one value is `"null"`
/// - `anyOf` with exactly two subschemas, one of which is just `{ "type": "null" }`
/// 
/// # Example
///
/// ```rust
/// use schemars::json_schema;
/// use dsc_lib_jsonschema::transforms::idiomaticize_option_field;
///
/// let mut schema = json_schema!({
///     "title": "Example",
///     "description": "Optional string",
///     "anyOf": [
///         { "type": "null" },
///         {
///             "type": "string",
///             "pattern": "^\\w+$",
///             "title": "Foo"
///         }
///     ]
/// });
///
/// idiomaticize_option_field(&mut schema);
///
/// let expected = json_schema!({
///     "title": "Example",
///     "description": "Optional string",
///     "type": "string",
///     "pattern": "^\\w+$"
/// });
///
/// assert_eq!(schema, expected);
/// ```
///
/// ```
/// use schemars::json_schema;
/// use dsc_lib_jsonschema::transforms::idiomaticize_option_field;
///
/// let mut schema = json_schema!({
///     "title": "Example",
///     "description": "Optional string",
///     "type": ["null", "string"],
///     "pattern": "^\\w+$"
/// });
///
/// idiomaticize_option_field(&mut schema);
///
/// let expected = json_schema!({
///     "title": "Example",
///     "description": "Optional string",
///     "type": "string",
///     "pattern": "^\\w+$"
/// });
///
/// assert_eq!(schema, expected);
/// ```
pub fn idiomaticize_option_field(schema: &mut Schema) {
    // Workaround for inability to borrow both mutably and immutably.
    let lookup_schema = schema.clone();
    let mut munged_schema = false;

    // First, handle the case where the schema defines `type` with two values, one of which is
    // `"null"`. This is emitted by schemars for `Option<T>` fields where `T` is a type that
    // schemars implemented `JsonSchema` for, like `String` or `i32`.
    if let Some(types) = lookup_schema.get_keyword_as_array("type")
        && types.len() == 2 && types.contains(&json!("null")) {
            let actual_type = types.iter().find(|t| t != &&serde_json::json!("null"));
            schema.insert("type".to_string(), actual_type.unwrap().clone());

            munged_schema = true;
    }
    
    // Handle `null` in `enum` keyword - remove if needed.
    if let Some(enum_values) = lookup_schema.get_keyword_as_array("enum") 
        && enum_values.contains(&json!(null)) {
            let mut new_enum_values = enum_values.clone();
            new_enum_values.retain(|v| v != &json!(null));
            schema.insert("enum".to_string(), json!(new_enum_values));

            munged_schema = true;
    }

    // If we munged the schema for type/enum, return early. The remaining code handles cases where
    // schemars inserted an `anyOf` keyword for referencing the underlying type schema.
    if munged_schema {
        return;
    }

    // Next, handle the case where the schema uses `anyOf` to represent an optional field.
    // This is emitted by schemars for `Option<T>` fields where `T` is a type that implements
    // `JsonSchema`. In this case, `anyOf` defines exactly two subschemas, one of which only
    // specifies `type` as `"null"`. Usually, the other subschema only includes a reference to
    // the underlying type schema (`$ref` keyword) unless that schema is inlined.
    let any_ofs = lookup_schema.get("anyOf")
        .unwrap_or_else(|| panic_t!(
            "transforms.idiomaticize_option_field.applies_to",
            transforming_schema = serde_json::to_string_pretty(schema).unwrap()
        ))
        .as_array()
        .unwrap_or_else(|| panic_t!(
            "transforms.idiomaticize_option_field.anyOf_array",
            transforming_schema = serde_json::to_string_pretty(schema).unwrap()
        ));

    if any_ofs.len() != 2 {
        panic_t!(
            "transforms.idiomaticize_option_field.anyOf_length_mismatch",
            actual_length = any_ofs.len(),
            transforming_schema = serde_json::to_string_pretty(schema).unwrap()
        );
    }

    let null_schema = any_ofs
        .iter()
        .find(|s| s.get("type").is_some_and(|t| t == "null"));
    if null_schema.is_none() {
        panic_t!(
            "transforms.idiomaticize_option_field.null_schema_missing",
            transforming_schema = serde_json::to_string_pretty(schema).unwrap()
        );
    }
    let actual_schema = any_ofs
        .iter()
        .find(|s| s.get("type").is_none_or(|t| t != "null"));
    if actual_schema.is_none() {
        panic_t!(
            "transforms.idiomaticize_option_field.actual_schema_missing",
            transforming_schema = serde_json::to_string_pretty(schema).unwrap()
        );
    }

    // At this point, we've verified that the target schema supports this transform.
    let actual_schema: &Schema = actual_schema.unwrap().try_into().unwrap();
    let munging_schema_keys = schema.get_defined_keywords();
    let actual_schema_keys = actual_schema.get_defined_keywords();

    for key in actual_schema_keys {
        if !munging_schema_keys.contains(&key) {
            schema.insert(key.clone(), actual_schema.get(&key).unwrap().clone());
        }
    }

    schema.remove("anyOf");
}
