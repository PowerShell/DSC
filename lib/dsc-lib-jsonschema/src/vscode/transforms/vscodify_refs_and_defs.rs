// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use schemars::Schema;

use crate::schema_utility_extensions::SchemaUtilityExtensions;

/// Replaces the keys in the `$defs` keyword with their urlencoded equivalent and updates all
/// references to the subschema to use the `#/$defs/...` pointer to the renamed definition key.
///
/// The VS Code extension for JSON Schema does not correctly discover canonically bundled schema
/// resources. This transformer:
///
/// 1. Finds all bundled schema resources.
/// 1. Compares the key for that bundled schema resource in `$defs` to the same key after URL
///    encoding.
/// 1. If the current key and url encoded key are the same, the transformer doesn't modify the
///    schema for that definition.
/// 1. If the current key and URL encoded key are different, the transformer renames the definition
///    to the new key and replaces _all_ references to the definition with the `#/$defs/<new_key>`
///    pointer. This modifies references to the site-relative URI, absolute URI, and prior pointer
///    value.
///
/// # Examples
///
/// This example shows how the transformer modifies a canonically bundled schema to enable VS Code
/// to resolve references to bundled schema resources.
///
/// ```rust
/// use schemars::json_schema;
/// use pretty_assertions::assert_eq;
/// use dsc_lib_jsonschema::vscode::transforms::vscodify_refs_and_defs;
///
/// let schema = &mut json_schema!({
///     "$id": "https://contoso.com/schemas/example.json",
///     "$schema": "https://json-schema.org/draft/2020-12/schema",
///     "properties": {
///         "foo": { "$ref": "https://contoso.com/schemas/definitions/foo.json" },
///         "bar": { "$ref": "/schemas/definitions/bar.json" },
///         "baz": { "$ref": "https://tstoys.com/schemas/baz.json" },
///     },
///     "$defs": {
///         "https://contoso.com/schemas/definitions/foo.json": {
///             "$id": "https://contoso.com/schemas/definitions/foo.json",
///             "$schema": "https://json-schema.org/draft/2020-12/schema",
///             "type": "string",
///         },
///         "https://contoso.com/schemas/definitions/bar.json": {
///             "$id": "https://contoso.com/schemas/definitions/bar.json",
///             "$schema": "https://json-schema.org/draft/2020-12/schema",
///             "type": "string",
///         },
///         "https://tstoys.com/schemas/baz.json": {
///             "$id": "https://tstoys.com/schemas/baz.json",
///             "$schema": "https://json-schema.org/draft/2020-12/schema",
///             "type": "string",
///         },
///     },
/// });
///
/// vscodify_refs_and_defs(schema);
/// let actual = serde_json::to_string_pretty(schema).unwrap();
///
/// let expected =serde_json::to_string_pretty(&json_schema!({
///     "$id": "https://contoso.com/schemas/example.json",
///     "$schema": "https://json-schema.org/draft/2020-12/schema",
///     "properties": {
///         "foo": { "$ref": "#/$defs/https%3A%2F%2Fcontoso.com%2Fschemas%2Fdefinitions%2Ffoo.json" },
///         "bar": { "$ref": "#/$defs/https%3A%2F%2Fcontoso.com%2Fschemas%2Fdefinitions%2Fbar.json" },
///         "baz": { "$ref": "#/$defs/https%3A%2F%2Ftstoys.com%2Fschemas%2Fbaz.json" },
///     },
///     "$defs": {
///         "https%3A%2F%2Fcontoso.com%2Fschemas%2Fdefinitions%2Ffoo.json": {
///             "$id": "https://contoso.com/schemas/definitions/foo.json",
///             "$schema": "https://json-schema.org/draft/2020-12/schema",
///             "type": "string",
///         },
///         "https%3A%2F%2Fcontoso.com%2Fschemas%2Fdefinitions%2Fbar.json": {
///             "$id": "https://contoso.com/schemas/definitions/bar.json",
///             "$schema": "https://json-schema.org/draft/2020-12/schema",
///             "type": "string",
///         },
///         "https%3A%2F%2Ftstoys.com%2Fschemas%2Fbaz.json": {
///             "$id": "https://tstoys.com/schemas/baz.json",
///             "$schema": "https://json-schema.org/draft/2020-12/schema",
///             "type": "string",
///         },
///     },
/// })).unwrap();
///
/// assert_eq!(actual, expected);
/// ```
pub fn vscodify_refs_and_defs(schema: &mut Schema) {
    let lookup_schema = schema.clone();
    for bundled_resource_id in lookup_schema.get_bundled_schema_resource_ids(true) {
        let Some(def_key) = lookup_schema
            .get_bundled_schema_resource_defs_key(&bundled_resource_id.to_string()) else {
            continue;
        };

        let encoded_key = &urlencoding::encode(def_key.as_str()).to_string();

        if def_key != encoded_key {
            schema.rename_defs_subschema(def_key.as_ref(), encoded_key.as_ref());
            let new_reference = &format!("#/$defs/{encoded_key}");
            for reference in lookup_schema.get_references_to_bundled_schema_resource(bundled_resource_id) {
                schema.replace_references(reference, new_reference.as_ref());
            }
        }
    }
}