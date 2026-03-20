// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use schemars::{JsonSchema, Schema, json_schema};
use serde::{Deserialize, Serialize};

use crate::dsc_repo::{
    DscRepoSchema,
    RecognizedSchemaVersion,
    SchemaForm,
    SchemaUriPrefix,
    get_default_schema_uri,
    get_recognized_schema_uri
};

#[test]
fn test_get_recognized_schema_uri() {
    let expected = "https://aka.ms/dsc/schemas/v3/bundled/config/document.json".to_string();
    let actual = get_recognized_schema_uri(
        "document",
        "config",
        RecognizedSchemaVersion::V3,
        SchemaForm::Bundled,
        SchemaUriPrefix::AkaDotMs
    );
    assert_eq!(expected, actual)
}

#[test]
fn test_get_default_schema_uri() {
    let expected_bundled = "https://aka.ms/dsc/schemas/v3/bundled/config/document.json".to_string();
    let expected_canonical = "https://aka.ms/dsc/schemas/v3/config/document.json".to_string();

    let schema_file_base_name = "document";
    let schema_folder_path = "config";

    assert_eq!(
        expected_bundled,
        get_default_schema_uri(schema_file_base_name, schema_folder_path, true)
    );
    assert_eq!(
        expected_canonical,
        get_default_schema_uri(schema_file_base_name, schema_folder_path, false)
    );
}

#[test]
fn test_dsc_repo_schema_bundled() {
    #[derive(Debug, Default, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
    struct ExampleBundledSchema {
        pub schema_version: String,
    }

    impl DscRepoSchema for ExampleBundledSchema {
        const SCHEMA_FILE_BASE_NAME: &'static str = "schema";
        const SCHEMA_FOLDER_PATH: &'static str = "example";
        const SCHEMA_SHOULD_BUNDLE: bool = true;

        fn schema_property_metadata() -> Schema {
            json_schema!({
                "description": "An example schema for testing.",
            })
        }
    }

    let bundled_uri = "https://aka.ms/dsc/schemas/v3/bundled/example/schema.json".to_string();
    let vscode_uri = "https://aka.ms/dsc/schemas/v3/bundled/example/schema.vscode.json".to_string();
    let canonical_uri = "https://aka.ms/dsc/schemas/v3/example/schema.json".to_string();
    let schema_version = RecognizedSchemaVersion::V3;

    assert_eq!(
        bundled_uri,
        ExampleBundledSchema::default_schema_id_uri()
    );

    assert_eq!(
        Some(bundled_uri),
        ExampleBundledSchema::get_bundled_schema_id_uri(schema_version)
    );

    assert_eq!(
        Some(vscode_uri),
        ExampleBundledSchema::get_enhanced_schema_id_uri(schema_version)
    );

    assert_eq!(
        canonical_uri,
        ExampleBundledSchema::get_canonical_schema_id_uri(schema_version)
    )
}

#[test]
fn test_dsc_repo_schema_not_bundled() {
    #[derive(Debug, Default, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
    struct ExampleNotBundledSchema {
        pub schema_version: String,
    }

    impl DscRepoSchema for ExampleNotBundledSchema {
        const SCHEMA_FILE_BASE_NAME: &'static str = "schema";
        const SCHEMA_FOLDER_PATH: &'static str = "example";
        const SCHEMA_SHOULD_BUNDLE: bool = false;

        fn schema_property_metadata() -> Schema {
            json_schema!({
                "description": "An example schema for testing.",
            })
        }
    }

    let canonical_uri = "https://aka.ms/dsc/schemas/v3/example/schema.json".to_string();
    let schema_version = RecognizedSchemaVersion::V3;
    assert_eq!(
        canonical_uri,
        ExampleNotBundledSchema::default_schema_id_uri()
    );

    assert_eq!(
        None,
        ExampleNotBundledSchema::get_bundled_schema_id_uri(schema_version)
    );

    assert_eq!(
        None,
        ExampleNotBundledSchema::get_enhanced_schema_id_uri(schema_version)
    );

    assert_eq!(
        canonical_uri,
        ExampleNotBundledSchema::get_canonical_schema_id_uri(schema_version)
    )
}
