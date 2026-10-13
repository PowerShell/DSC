// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use schemars::Schema;

use crate::schema_utility_extensions::SchemaUtilityExtensions;

/// Canonicalizes the references to and definitions for bundled schema resources.
///
/// Bundled schema resources are any definition in the `$defs` keyword that specifies the `$id`
/// keyword.
///
/// This transformer:
///
/// 1. Standardizes the key for bundled schema resources to the ID URI for that resource. When
///    a JSON Schema client resolves bundled schema resources.
/// 1. Replaces _all_ references to the bundled schema resource with the ID for that resource.
///    This converts all fragment pointer references, like `#/$defs/foo`, to the absolute URI
///    for the schema resource. Similarly, any relative URIs to the bundled resource, like
///    `/schemas/foo.json`, are also updated to the absolute URI.
///
/// This standardizes the structure and references for bundled schema resources to enable more
/// consistent operations on them.
///
/// # Examples
///
/// The following snippet shows how this method transforms the schema.
///
/// ```rust
/// use dsc_lib_jsonschema::transforms::canonicalize_refs_and_defs;
/// use schemars::json_schema;
/// # use pretty_assertions::assert_eq;
///
/// let schema = &mut json_schema!({
///     "$id": "https://contoso.com/schemas/example.json",
///     "$schema": "https://json-schema.org/draft/2020-12/schema",
///     "properties": {
///         "foo": { "$ref": "#/$defs/foo" },
///         "bar": { "$ref": "/schemas/definitions/bar.json" },
///         "baz": { "$ref": "https://tstoys.com/schemas/baz.json" },
///     },
///     "$defs": {
///         "foo": {
///             "$id": "https://contoso.com/schemas/definitions/foo.json",
///             "$schema": "https://json-schema.org/draft/2020-12/schema",
///             "type": "string",
///         },
///         "bar": {
///             "$id": "https://contoso.com/schemas/definitions/bar.json",
///             "$schema": "https://json-schema.org/draft/2020-12/schema",
///             "type": "string",
///         },
///         "baz": {
///             "$id": "https://tstoys.com/schemas/baz.json",
///             "$schema": "https://json-schema.org/draft/2020-12/schema",
///             "type": "string",
///         },
///     },
/// });
/// canonicalize_refs_and_defs(schema);
/// let actual = serde_json::to_string_pretty(schema).unwrap();
///
/// let expected = serde_json::to_string_pretty(&json_schema!({
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
/// })).unwrap();
///
/// assert_eq!(actual, expected);
/// ```
pub fn canonicalize_refs_and_defs(schema: &mut Schema) {
    schema.canonicalize_refs_and_defs_for_bundled_resources();
}
