// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use schemars::generate::SchemaSettings;
use schemars::transform::transform_subschemas;
use schemars::{Schema, SchemaGenerator};
use regex::Regex;


/// Returns the canonical URI for the resource instance schema.
pub fn get_schema_uri(vscode: bool) -> String {
    const URI_PREFIX: &str = "https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/resources";
    const URI_BASE: &str = "Microsoft.Windows/Registry";
    const URI_VERSION: &str = env!("CARGO_PKG_VERSION");
    let uri_suffix = if vscode { "schema.vscode.json" } else { "schema.json" };

    format!("{}/{}/v{}/{}", URI_PREFIX, URI_BASE, URI_VERSION, uri_suffix)
}

/// Returns a regular expression to validate the `keyPath` resource instance property.
pub fn get_registry_key_path_pattern() -> Regex {
    let prefixes: Vec<String> = vec![
        "HKCC".to_string(), "HKEY_CURRENT_CONFIG".to_string(),
        "HKCU".to_string(), "HKEY_CURRENT_USER".to_string(),
        "HKCR".to_string(), "HKEY_CLASSES_ROOT".to_string(),
        "HKLM".to_string(), "HKEY_LOCAL_MACHINE".to_string(),
        "HKU".to_string(), "HKEY_USERS".to_string()
    ];

    Regex::new(
        format!(r"^({})\\[a-zA-Z0-9-_\\]+?[^\\]$", prefixes.join("|")).as_str()
    ).unwrap()
}

/// Munges the JSON Schema to use the correct URI for the schema ID and removes VS Code keywords.
pub fn canonicalize_schema(schema: &mut Schema) {
    // Update `$schema` to appropriate JSON file for 2020-12, without VS Code keywords
    schema.insert("$id".to_string(), serde_json::Value::String(get_schema_uri(false)));
    remove_vscode_keywords(schema);
}

// This should more properly move to DSC for reuse
/// Recursively removes keywords from the JSON Schema that are only used and understood by VS Code
/// to provide an enhanced authoring and editing experience. The VS Code keywords are annotations
/// that most JSON Schema tools and implementations _should_ ignore, but may still cause issues.
pub fn remove_vscode_keywords(schema: &mut Schema) {
    let keywords: Vec<String> = vec![
        "defaultSnippets".to_string(),
        "errorMessage".to_string(),
        "patternErrorMessage".to_string(),
        "deprecationMessage".to_string(),
        "enumDescriptions".to_string(),
        "markdownEnumDescriptions".to_string(),
        "markdownDescription".to_string(),
        "doNotSuggest".to_string(),
        "suggestSortText".to_string(),
        "allowComments".to_string(),
        "allowTrailingCommas".to_string(),
    ];
    for keyword in keywords {
        schema.remove(&keyword);
    }

    transform_subschemas(&mut remove_vscode_keywords, schema)
}

// This should more properly move to DSC for reuse
/// Returns a `SchemaGenerator` for deriving JSON Schemas from structs. If the function is
/// called to return a generator for JSON Schemas meant for use in VS Code, it uses the
/// deprecated `definitions` keyword instead of `$defs` for references, which VS Code
/// doesn't understand yet.
pub fn get_schema_generator(for_vscode_schema: bool) -> SchemaGenerator {
    let settings: SchemaSettings = if for_vscode_schema {
        SchemaSettings::draft2020_12().with(|s| {
            // More properly, the data should not be included in the object, instead of allowing
            // or expecting null values
            s.option_add_null_type = false;
            // Need to use definitions instead of $defs, which VS Code doesn't understand.
            s.definitions_path = "/definitions".to_string();
        })
    } else {
        SchemaSettings::draft2020_12().with(|s| {
            s.option_add_null_type = false;
        })
    };
    settings.into_generator()
}
