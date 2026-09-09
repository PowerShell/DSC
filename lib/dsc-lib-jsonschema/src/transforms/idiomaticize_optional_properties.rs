// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use schemars::Schema;

use crate::transforms::idiomaticize_option_field;
use crate::schema_utility_extensions::SchemaUtilityExtensions;

/// Transforms all optional properties in the given JSON Schema to use the idiomatic `Option` type.
/// 
/// This function iterates over all properties in the schema and applies the
/// [`idiomaticize_option_field`] transform to those that aren't in the `required` keyword array.
/// 
/// # Example
/// 
/// ```rust
/// use schemars::json_schema;
/// use dsc_lib_jsonschema::transforms::idiomaticize_optional_properties;
/// 
/// let mut schema = json_schema!({
///     "title": "Example struct",
///     "type": "object",
///     "required": ["baz"],
///     "properties": {
///         "foo": {
///             "type": ["string", "null"],
///             "pattern": "^\\w+$",
///         },
///         "bar": {
///             "anyOf": [
///                 { "$ref": "$defs/bar" },
///                 { "type": "null" },
///             ]
///         },
///         "baz": {
///             "type": ["string", "null"]
///         }
///     },
///     "$defs": {
///         "bar": {
///             "type": "boolean"
///         }
///     }
/// });
/// idiomaticize_optional_properties(&mut schema);
/// 
/// let expected = json_schema!({
///     "title": "Example struct",
///     "type": "object",
///     "required": ["baz"],
///     "properties": {
///         "foo": {
///             "type": "string",
///             "pattern": "^\\w+$",
///         },
///         "bar": {
///             "$ref": "$defs/bar"
///         },
///         "baz": {
///             "type": ["string", "null"]
///         }
///     },
///     "$defs": {
///         "bar": {
///             "type": "boolean"
///         }
///     }
/// });
/// 
/// assert_eq!(schema, expected);
/// ```
pub fn idiomaticize_optional_properties(schema: &mut Schema) {
    let lookup_schema = schema.clone();
    let required_properties = lookup_schema.get_required_property_names();
    for property_name in lookup_schema.get_properties_keys() {
        if required_properties.contains(&property_name) { 
            continue;
        }
        if let Some(property_schema) = schema.get_property_subschema_mut(&property_name) {
            idiomaticize_option_field(property_schema);
        }
    }
}
