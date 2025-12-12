// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

/// This macro exists to validate that you can pass either a string literal or an expression for a
/// schema field metadata item.
macro_rules! testing_title {
    () => {
        "Example schema"
    };
}

#[cfg(test)] mod for_enum {
    #[cfg(test)] mod without_bundling {
        use pretty_assertions::assert_eq;
        use schemars::JsonSchema;
        use dsc_lib_jsonschema::dsc_repo::{DscRepoSchema, RecognizedSchemaVersion};

        #[allow(dead_code)]
        #[derive(Clone, Debug, JsonSchema, DscRepoSchema)]
        #[dsc_repo_schema(base_name = "valid", folder_path = "example")]
        enum Example {
            String(String),
            Boolean(bool)
        }

        #[test] fn test_default_schema_id_uri() {
            assert_eq!(
                Example::default_schema_id_uri(),
                "https://aka.ms/dsc/schemas/v3/example/valid.json".to_string()
            )
        }

        #[test] fn test_get_canonical_schema_id_uri() {
            assert_eq!(
                Example::get_canonical_schema_id_uri(RecognizedSchemaVersion::V3),
                "https://aka.ms/dsc/schemas/v3/example/valid.json".to_string()
            )
        }

        #[test] fn test_get_bundled_schema_id_uri() {
            assert_eq!(
                Example::get_bundled_schema_id_uri(RecognizedSchemaVersion::V3),
                None
            )
        }
        #[test] fn test_get_enhanced_schema_id_uri() {
            assert_eq!(
                Example::get_enhanced_schema_id_uri(RecognizedSchemaVersion::V3),
                None
            )
        }
    }
}

#[cfg(test)] mod for_struct {
    #[cfg(test)] mod without_bundling {
        #[cfg(test)] mod without_schema_field {
            use pretty_assertions::assert_eq;
            use schemars::JsonSchema;
            use dsc_lib_jsonschema::dsc_repo::{DscRepoSchema, RecognizedSchemaVersion};

            #[allow(dead_code)]
            #[derive(Clone, Debug, JsonSchema, DscRepoSchema)]
            #[dsc_repo_schema(base_name = "valid", folder_path = "example")]
            struct Example {
                pub foo: String,
                pub bar: i32,
                pub baz: bool,
            }

            #[test] fn test_default_schema_id_uri() {
                assert_eq!(
                    Example::default_schema_id_uri(),
                    "https://aka.ms/dsc/schemas/v3/example/valid.json".to_string()
                )
            }

            #[test] fn test_get_canonical_schema_id_uri() {
                assert_eq!(
                    Example::get_canonical_schema_id_uri(RecognizedSchemaVersion::V3),
                    "https://aka.ms/dsc/schemas/v3/example/valid.json".to_string()
                )
            }

            #[test] fn test_get_bundled_schema_id_uri() {
                assert_eq!(
                    Example::get_bundled_schema_id_uri(RecognizedSchemaVersion::V3),
                    None
                )
            }
            #[test] fn test_get_enhanced_schema_id_uri() {
                assert_eq!(
                    Example::get_enhanced_schema_id_uri(RecognizedSchemaVersion::V3),
                    None
                )
            }
        }
        #[cfg(test)] mod with_schema_field {
            use pretty_assertions::assert_eq;
            use schemars::JsonSchema;
            use dsc_lib_jsonschema::{dsc_repo::{DscRepoSchema, RecognizedSchemaVersion, UnrecognizedSchemaUri}, schema_utility_extensions::SchemaUtilityExtensions};

            #[allow(dead_code)]
            #[derive(Clone, Debug, JsonSchema, DscRepoSchema)]
            #[dsc_repo_schema(
                base_name = "valid",
                folder_path = "example",
                schema_field(
                    name = schema_version,
                    title = testing_title!(),
                    description = "An example struct with a schema field.",
                )
            )]
            struct Example {
                pub schema_version: String,
                pub foo: String,
                pub bar: i32,
                pub baz: bool,
            }

            #[test] fn test_default_schema_id_uri() {
                assert_eq!(
                    Example::default_schema_id_uri(),
                    "https://aka.ms/dsc/schemas/v3/example/valid.json".to_string()
                )
            }

            #[test] fn test_get_canonical_schema_id_uri() {
                assert_eq!(
                    Example::get_canonical_schema_id_uri(RecognizedSchemaVersion::V3),
                    "https://aka.ms/dsc/schemas/v3/example/valid.json".to_string()
                )
            }

            #[test] fn test_get_bundled_schema_id_uri() {
                assert_eq!(
                    Example::get_bundled_schema_id_uri(RecognizedSchemaVersion::V3),
                    None
                )
            }
            #[test] fn test_get_enhanced_schema_id_uri() {
                assert_eq!(
                    Example::get_enhanced_schema_id_uri(RecognizedSchemaVersion::V3),
                    None
                )
            }

            #[test] fn test_recognized_schema_uris_subschema() {
                let ref mut generator = schemars::SchemaGenerator::default();
                let subschema = Example::recognized_schema_uris_subschema(generator);

                let enum_subschema = subschema.get_keyword_as_array("enum").unwrap();
                let enum_count = enum_subschema.len();
                let expected_count = RecognizedSchemaVersion::all().len() * 2;
                assert_eq!(
                    enum_count,
                    expected_count
                );

                assert_eq!(
                    subschema.get_keyword_as_str("type"),
                    Some("string")
                );

                assert_eq!(
                    subschema.get_keyword_as_str("format"),
                    Some("uri")
                );

                assert_eq!(
                    subschema.get_keyword_as_str("title"),
                    Some("Example schema")
                );

                assert_eq!(
                    subschema.get_keyword_as_str("description"),
                    Some("An example struct with a schema field.")
                );
                assert_eq!(
                    subschema.get_keyword_as_str("markdownDescription"),
                    None
                );
            }

            #[test] fn test_is_recognized_schema_uri() {
                assert_eq!(
                    Example::is_recognized_schema_uri(&"https://incorrect/uri.json".to_string()),
                    false
                );

                assert_eq!(
                    Example::is_recognized_schema_uri(&Example::default_schema_id_uri()),
                    true
                );
            }

            #[test] fn test_validate_schema_uri() {
                let valid_instance = Example {
                    schema_version: Example::default_schema_id_uri(),
                    foo: String::new(),
                    bar: 0,
                    baz: true
                };

                assert_eq!(
                    valid_instance.validate_schema_uri(),
                    Ok(())
                );

                let invalid_uri = "https://incorrect/uri.json".to_string();
                let invalid_instance = Example {
                    schema_version: invalid_uri.clone(),
                    foo: String::new(),
                    bar: 0,
                    baz: true
                };

                assert_eq!(
                    invalid_instance.validate_schema_uri(),
                    Err(UnrecognizedSchemaUri(invalid_uri, Example::recognized_schema_uris()))
                )
            }
        }
    }

    #[cfg(test)] mod with_bundling {
        #[cfg(test)] mod without_schema_field {
            use pretty_assertions::assert_eq;
            use schemars::JsonSchema;
            use dsc_lib_jsonschema::dsc_repo::{DscRepoSchema, RecognizedSchemaVersion};

            #[allow(dead_code)]
            #[derive(Clone, Debug, JsonSchema, DscRepoSchema)]
            #[dsc_repo_schema(base_name = "valid", folder_path = "example", should_bundle = true)]
            struct Example {
                pub foo: String,
                pub bar: i32,
                pub baz: bool,
            }

            #[test] fn test_default_schema_id_uri() {
                assert_eq!(
                    Example::default_schema_id_uri(),
                    "https://aka.ms/dsc/schemas/v3/bundled/example/valid.json".to_string()
                )
            }

            #[test] fn test_get_canonical_schema_id_uri() {
                assert_eq!(
                    Example::get_canonical_schema_id_uri(RecognizedSchemaVersion::V3),
                    "https://aka.ms/dsc/schemas/v3/example/valid.json".to_string()
                )
            }

            #[test] fn test_get_bundled_schema_id_uri() {
                assert_eq!(
                    Example::get_bundled_schema_id_uri(RecognizedSchemaVersion::V3),
                    Some("https://aka.ms/dsc/schemas/v3/bundled/example/valid.json".to_string())
                )
            }
            #[test] fn test_get_enhanced_schema_id_uri() {
                assert_eq!(
                    Example::get_enhanced_schema_id_uri(RecognizedSchemaVersion::V3),
                    Some("https://aka.ms/dsc/schemas/v3/bundled/example/valid.vscode.json".to_string())
                )
            }
        }
    
        #[cfg(test)] mod with_schema_field {
            use pretty_assertions::assert_eq;
            use schemars::JsonSchema;
            use dsc_lib_jsonschema::{dsc_repo::{DscRepoSchema, RecognizedSchemaVersion, UnrecognizedSchemaUri}, schema_utility_extensions::SchemaUtilityExtensions};

            #[allow(dead_code)]
            #[derive(Clone, Debug, JsonSchema, DscRepoSchema)]
            #[dsc_repo_schema(
                base_name = "valid",
                folder_path = "example",
                should_bundle = true,
                schema_field(
                    name = schema_version,
                    title = testing_title!(),
                    description = "An example struct with a schema field.",
                )
            )]
            struct Example {
                pub schema_version: String,
                pub foo: String,
                pub bar: i32,
                pub baz: bool,
            }

            #[test] fn test_default_schema_id_uri() {
                assert_eq!(
                    Example::default_schema_id_uri(),
                    "https://aka.ms/dsc/schemas/v3/bundled/example/valid.json".to_string()
                )
            }

            #[test] fn test_get_canonical_schema_id_uri() {
                assert_eq!(
                    Example::get_canonical_schema_id_uri(RecognizedSchemaVersion::V3),
                    "https://aka.ms/dsc/schemas/v3/example/valid.json".to_string()
                )
            }

            #[test] fn test_get_bundled_schema_id_uri() {
                assert_eq!(
                    Example::get_bundled_schema_id_uri(RecognizedSchemaVersion::V3),
                    Some("https://aka.ms/dsc/schemas/v3/bundled/example/valid.json".to_string())
                )
            }
            #[test] fn test_get_enhanced_schema_id_uri() {
                assert_eq!(
                    Example::get_enhanced_schema_id_uri(RecognizedSchemaVersion::V3),
                    Some("https://aka.ms/dsc/schemas/v3/bundled/example/valid.vscode.json".to_string())
                )
            }

            #[test] fn test_recognized_schema_uris_subschema() {
                let ref mut generator = schemars::SchemaGenerator::default();
                let subschema = Example::recognized_schema_uris_subschema(generator);

                let enum_subschema = subschema.get_keyword_as_array("enum").unwrap();
                let enum_count = enum_subschema.len();
                let expected_count = RecognizedSchemaVersion::all().len() * 6;
                assert_eq!(
                    enum_count,
                    expected_count
                );

                assert_eq!(
                    subschema.get_keyword_as_str("type"),
                    Some("string")
                );

                assert_eq!(
                    subschema.get_keyword_as_str("format"),
                    Some("uri")
                );

                assert_eq!(
                    subschema.get_keyword_as_str("title"),
                    Some("Example schema")
                );

                assert_eq!(
                    subschema.get_keyword_as_str("description"),
                    Some("An example struct with a schema field.")
                );
                assert_eq!(
                    subschema.get_keyword_as_str("markdownDescription"),
                    None
                );
            }

            #[test] fn test_is_recognized_schema_uri() {
                assert_eq!(
                    Example::is_recognized_schema_uri(&"https://incorrect/uri.json".to_string()),
                    false
                );

                assert_eq!(
                    Example::is_recognized_schema_uri(&Example::default_schema_id_uri()),
                    true
                );
            }

            #[test] fn test_validate_schema_uri() {
                let valid_instance = Example {
                    schema_version: Example::default_schema_id_uri(),
                    foo: String::new(),
                    bar: 0,
                    baz: true
                };

                assert_eq!(
                    valid_instance.validate_schema_uri(),
                    Ok(())
                );

                let invalid_uri = "https://incorrect/uri.json".to_string();
                let invalid_instance = Example {
                    schema_version: invalid_uri.clone(),
                    foo: String::new(),
                    bar: 0,
                    baz: true
                };

                assert_eq!(
                    invalid_instance.validate_schema_uri(),
                    Err(UnrecognizedSchemaUri(invalid_uri, Example::recognized_schema_uris()))
                )
            }
        }
    }
}
