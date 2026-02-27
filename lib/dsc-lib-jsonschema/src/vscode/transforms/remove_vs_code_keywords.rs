// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use schemars::{Schema, transform::transform_subschemas};

use crate::vscode::keywords::VSCodeKeyword;

/// Recursively removes all VS Code keywords from the schema.
///
/// This transformer recurses through every level of the schema to find every defined
/// [`VSCodeKeyword`] and removes them. While the VS Code keywords are annotation keywords that
/// don't affect the validation for a schema, some validation libraries may error on the inclusion
/// of unknown keywords. Removing them from the canonical and bundled forms for a schema removes
/// that error path.
///
/// Further, removing the VS Code keywords makes for a much smaller schema, since many of the VS
/// Code keywords provide extended documentation and VS Code specific functionality, like snippets.
/// This can reduce the time required to retrieve and parse the schemas, in addition to minimizing
/// network costs.
///
/// # Examples
///
/// The following example shows how you can use the transformer to remove all VS Code keywords from
/// a given schema.
///
/// ```rust
/// use pretty_assertions::assert_eq;
/// use schemars::json_schema;
/// use dsc_lib_jsonschema::vscode::transforms::remove_vs_code_keywords;
///
/// let schema = &mut json_schema!({
///     "$id": "https://contoso.com/schemas/example.json",
///     "$schema": "https://json-schema.org/draft/2020-12/schema",
///     "title": "Example schema",
///     "description": "This example schema describes an object.",
///     "markdownDescription": "This _markdown_ text is for VS Code only.",
///     "properties": {
///         "name": { "$ref": "#/$defs/name" },
///         "state": {
///             "title": "Feature state",
///             "description": "Defines whether the feature should be enabled, disabled, or toggled.",
///             "markdownDescription": concat!(
///                 "Defines whether feature should be enabled, disabled, or toggled. ",
///                 "The `toggle` option should only be used in testing and development. ",
///                 "Setting a feature to `toggle` will cause it to change on every `set` operation."
///             ),
///             "type": "string",
///             "enum": ["on", "off", "toggle"],
///             "markdownEnumDescriptions": [
///                 "Sets the named feature to `on`, enabling it.",
///                 "Sets the named feature to `off`, disabling it.",
///                 "Toggles the named feature, enabling it if disabled and disabling it if enabled.",
///             ],
///         },
///     },
///     "$defs": {
///         "name": {
///             "title": "Feature name",
///             "description": "Defines the feature to manage.",
///             "markdownDescription": concat!(
///                 "Defines the feature to manage by its name. ",
///                 "For a full list of available features and their names, see ",
///                 "[Feature list](https://contoso.com/example/features)."
///             ),
///             "type": "string",
///         },
///     },
/// });
///
/// remove_vs_code_keywords(schema);
///
/// let actual = serde_json::to_string_pretty(schema).unwrap();
/// let expected = serde_json::to_string_pretty(&json_schema!({
///     "$id": "https://contoso.com/schemas/example.json",
///     "$schema": "https://json-schema.org/draft/2020-12/schema",
///     "title": "Example schema",
///     "description": "This example schema describes an object.",
///     "properties": {
///         "name": { "$ref": "#/$defs/name" },
///         "state": {
///             "title": "Feature state",
///             "description": "Defines whether the feature should be enabled, disabled, or toggled.",
///             "type": "string",
///             "enum": ["on", "off", "toggle"],
///         },
///     },
///     "$defs": {
///         "name": {
///             "title": "Feature name",
///             "description": "Defines the feature to manage.",
///             "type": "string",
///         },
///     },
/// })).unwrap();
///
/// assert_eq!(actual, expected);
/// ```
pub fn remove_vs_code_keywords(schema: &mut Schema) {
    for keyword in VSCodeKeyword::ALL {
        schema.remove(keyword);
    }

    transform_subschemas(&mut remove_vs_code_keywords, schema);
}
