// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use schemars::Schema;

use crate::schema_utility_extensions::SchemaUtilityExtensions;

/// Removes every entry in the `$defs` keyword that contains a bundled schema resource.
///
/// Bundled schema resources are any definition in the `$defs` keyword that specifies the `$id`
/// keyword.
///
/// This transform doesn't update any references to the bundled schema resources. If the
/// reference to the bundled resource uses the URI fragment pointer to the `$defs` keyword, those
/// references will be broken. If the references point to the bundled schema resource by absolute
/// or relative URI, those references are still valid.
///
/// After removing bundled schema resources from the `$defs` keyword, the transform removes the
/// `$defs` keyword if it is empty.
///
/// # Examples
///
/// The following snippet shows how this transform removes bundled schema resources.
///
/// ```rust
/// use schemars::json_schema;
/// use dsc_lib_jsonschema::transforms::remove_bundled_schema_resources;
/// # use pretty_assertions::assert_eq;
///
/// let schema = &mut json_schema!({
///     "$id": "https://contoso.com/schemas/example.json",
///     "$schema": "https://json-schema.org/draft/2020-12/schema",
///     "properties": {
///         "foo": { "$ref": "https://contoso.com/schemas/definitions/foo.json" },
///         "bar": { "$ref": "https://contoso.com/schemas/definitions/bar.json" },
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
/// remove_bundled_schema_resources(schema);
/// let actual = serde_json::to_string_pretty(schema).unwrap();
///
/// let expected = serde_json::to_string_pretty(&json_schema!({
///     "$id": "https://contoso.com/schemas/example.json",
///     "$schema": "https://json-schema.org/draft/2020-12/schema",
///     "properties": {
///         "foo": { "$ref": "https://contoso.com/schemas/definitions/foo.json" },
///         "bar": { "$ref": "https://contoso.com/schemas/definitions/bar.json" },
///         "baz": { "$ref": "https://tstoys.com/schemas/baz.json" },
///     }
/// })).unwrap();
///
/// assert_eq!(actual, expected);
/// ```
pub fn remove_bundled_schema_resources(schema: &mut Schema) {
    schema.remove_bundled_schema_resources();
}
