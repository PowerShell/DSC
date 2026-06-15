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
        use std::ops::Index;

        use pretty_assertions::assert_eq;
        use schemars::{JsonSchema, Schema, schema_for};
        use dsc_lib_jsonschema::{
            dsc_repo::{DscRepoSchema, RecognizedSchemaVersion, schema_i18n}, schema_utility_extensions::SchemaUtilityExtensions};

        #[allow(dead_code)]
        #[derive(Clone, Debug, JsonSchema, DscRepoSchema)]
        #[dsc_repo_schema(base_name = "valid.enum", folder_path = "example")]
        #[schemars(
            title = schema_i18n!("title"),
            description = schema_i18n!("description"),
            extend(
                "markdownDescription" = schema_i18n!("markdownDescription"),
            )
        )]
        enum Example {
            #[schemars(
                title = schema_i18n!("stringVariant.title"),
                description = schema_i18n!("stringVariant.description"),
                extend(
                    "markdownDescription" = schema_i18n!("stringVariant.markdownDescription")
                )
            )]
            String(String),

            #[schemars(
                title = schema_i18n!("booleanVariant.title"),
                description = schema_i18n!("booleanVariant.description"),
                extend(
                    "markdownDescription" = schema_i18n!("booleanVariant.markdownDescription")
                )
            )]
            Boolean(bool)
        }

        #[test] fn test_default_schema_id_uri() {
            assert_eq!(
                Example::default_schema_id_uri(),
                "https://aka.ms/dsc/schemas/v3/example/valid.enum.json".to_string()
            )
        }

        #[test] fn test_get_canonical_schema_id_uri() {
            assert_eq!(
                Example::get_canonical_schema_id_uri(RecognizedSchemaVersion::V3),
                "https://aka.ms/dsc/schemas/v3/example/valid.enum.json".to_string()
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

        #[test] fn test_schema_docs() {
            let schema = &schema_for!(Example);
            assert_eq!(
                schema.get_keyword_as_str("title"),
                Some("Example valid enum")
            );
            assert_eq!(
                schema.get_keyword_as_str("description"),
                Some("Defines an enum with the DscRepoSchema trait.")
            );
            assert_eq!(
                schema.get_keyword_as_str("markdownDescription"),
                Some("Defines an enum with the `DscRepoSchema` trait.")
            );

            let ref string_variant: Schema = schema.get_keyword_as_array("oneOf")
                .unwrap()
                .index(0)
                .as_object()
                .unwrap()
                .clone()
                .into();

            assert_eq!(
                string_variant.get_keyword_as_str("title"),
                Some("Example string variant")
            );
            assert_eq!(
                string_variant.get_keyword_as_str("description"),
                Some("Defines a string variant for the enum.")
            );
            assert_eq!(
                string_variant.get_keyword_as_str("markdownDescription"),
                Some("Defines a `string` variant for the enum.")
            );

            let ref boolean_variant: Schema = schema.get_keyword_as_array("oneOf")
                .unwrap()
                .index(1)
                .as_object()
                .unwrap()
                .clone()
                .into();

            assert_eq!(
                boolean_variant.get_keyword_as_str("title"),
                Some("Example boolean variant")
            );
            assert_eq!(
                boolean_variant.get_keyword_as_str("description"),
                Some("Defines a boolean variant for the enum.")
            );
            assert_eq!(
                boolean_variant.get_keyword_as_str("markdownDescription"),
                Some("Defines a `boolean` variant for the enum.")
            );
        }
    }
}

#[cfg(test)] mod for_struct {
    #[cfg(test)] mod without_bundling {
        #[cfg(test)] mod without_schema_field {
            use pretty_assertions::assert_eq;
            use schemars::{JsonSchema, schema_for};
            use dsc_lib_jsonschema::{
                dsc_repo::{
                    DscRepoSchema,
                    RecognizedSchemaVersion,
                    schema_i18n
                },
                schema_utility_extensions::SchemaUtilityExtensions
            };

            #[allow(dead_code)]
            #[derive(Clone, Debug, JsonSchema, DscRepoSchema)]
            #[dsc_repo_schema(base_name = "valid.struct", folder_path = "example")]
            #[schemars(
                title = schema_i18n!("title"),
                description = schema_i18n!("description"),
                extend(
                    "markdownDescription" = schema_i18n!("markdownDescription"),
                )
            )]
            struct Example {
                #[schemars(
                    title = schema_i18n!("foo.title"),
                    description = schema_i18n!("foo.description"),
                    extend(
                        "markdownDescription" = schema_i18n!("foo.markdownDescription"),
                    )
                )]
                pub foo: String,

                #[schemars(
                    title = schema_i18n!("bar.title"),
                    description = schema_i18n!("bar.description"),
                    extend(
                        "markdownDescription" = schema_i18n!("bar.markdownDescription"),
                    )
                )]
                pub bar: i32,

                #[schemars(
                    title = schema_i18n!("baz.title"),
                    description = schema_i18n!("baz.description"),
                    extend(
                        "markdownDescription" = schema_i18n!("baz.markdownDescription"),
                    )
                )]
                pub baz: bool,
            }

            #[test] fn test_default_schema_id_uri() {
                assert_eq!(
                    Example::default_schema_id_uri(),
                    "https://aka.ms/dsc/schemas/v3/example/valid.struct.json".to_string()
                )
            }

            #[test] fn test_get_canonical_schema_id_uri() {
                assert_eq!(
                    Example::get_canonical_schema_id_uri(RecognizedSchemaVersion::V3),
                    "https://aka.ms/dsc/schemas/v3/example/valid.struct.json".to_string()
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

            #[test] fn test_schema_docs() {
                let schema = &schema_for!(Example);
                assert_eq!(
                    schema.get_keyword_as_str("title"),
                    Some("Example valid struct")
                );
                assert_eq!(
                    schema.get_keyword_as_str("description"),
                    Some("Defines a struct with the DscRepoSchema trait.")
                );
                assert_eq!(
                    schema.get_keyword_as_str("markdownDescription"),
                    Some("Defines a struct with the `DscRepoSchema` trait.")
                );

                for property_name in vec!["foo", "bar", "baz"] {
                    let property_schema = schema.get_property_subschema(property_name).unwrap();

                    assert_eq!(
                        property_schema.get_keyword_as_string("title"),
                        Some(format!("{property_name} field"))
                    );
                    assert_eq!(
                        property_schema.get_keyword_as_string("description"),
                        Some(format!("Defines the {property_name} field."))
                    );
                    assert_eq!(
                        property_schema.get_keyword_as_string("markdownDescription"),
                        Some(format!("Defines the `{property_name}` field."))
                    );
                }
            }
        }

        #[cfg(test)] mod with_schema_field {
            use pretty_assertions::assert_eq;
            use schemars::{JsonSchema, schema_for};
            use dsc_lib_jsonschema::{
                dsc_repo::{
                    DscRepoSchema,
                    RecognizedSchemaVersion,
                    UnrecognizedSchemaUri,
                    schema_i18n
                },
                schema_utility_extensions::SchemaUtilityExtensions
            };

            #[allow(dead_code)]
            #[derive(Clone, Debug, JsonSchema, DscRepoSchema)]
            #[dsc_repo_schema(
                base_name = "valid",
                folder_path = "example",
                i18n_root_key = "schemas.example.valid.struct",
                schema_field(
                    name = schema_version,
                    title = testing_title!(),
                    description = "An example struct with a schema field.",
                )
            )]
            #[schemars(
                title = schema_i18n!("title"),
                description = schema_i18n!("description"),
                extend(
                    "markdownDescription" = schema_i18n!("markdownDescription"),
                )
            )]
            struct Example {
                pub schema_version: String,
                
                #[schemars(
                    title = schema_i18n!("foo.title"),
                    description = schema_i18n!("foo.description"),
                    extend(
                        "markdownDescription" = schema_i18n!("foo.markdownDescription"),
                    )
                )]
                pub foo: String,

                #[schemars(
                    title = schema_i18n!("bar.title"),
                    description = schema_i18n!("bar.description"),
                    extend(
                        "markdownDescription" = schema_i18n!("bar.markdownDescription"),
                    )
                )]
                pub bar: i32,

                #[schemars(
                    title = schema_i18n!("baz.title"),
                    description = schema_i18n!("baz.description"),
                    extend(
                        "markdownDescription" = schema_i18n!("baz.markdownDescription"),
                    )
                )]
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

            #[test] fn test_schema_docs() {
                let schema = &schema_for!(Example);
                assert_eq!(
                    schema.get_keyword_as_str("title"),
                    Some("Example valid struct")
                );
                assert_eq!(
                    schema.get_keyword_as_str("description"),
                    Some("Defines a struct with the DscRepoSchema trait.")
                );
                assert_eq!(
                    schema.get_keyword_as_str("markdownDescription"),
                    Some("Defines a struct with the `DscRepoSchema` trait.")
                );

                for property_name in vec!["foo", "bar", "baz"] {
                    let property_schema = schema.get_property_subschema(property_name).unwrap();

                    assert_eq!(
                        property_schema.get_keyword_as_string("title"),
                        Some(format!("{property_name} field"))
                    );
                    assert_eq!(
                        property_schema.get_keyword_as_string("description"),
                        Some(format!("Defines the {property_name} field."))
                    );
                    assert_eq!(
                        property_schema.get_keyword_as_string("markdownDescription"),
                        Some(format!("Defines the `{property_name}` field."))
                    );
                }
            }
        }
    }

    #[cfg(test)] mod with_bundling {
        #[cfg(test)] mod without_schema_field {
            use pretty_assertions::assert_eq;
            use schemars::{JsonSchema, schema_for};
            use dsc_lib_jsonschema::{
                dsc_repo::{
                    DscRepoSchema,
                    RecognizedSchemaVersion,
                    schema_i18n
                },
                schema_utility_extensions::SchemaUtilityExtensions
            };

            #[allow(dead_code)]
            #[derive(Clone, Debug, JsonSchema, DscRepoSchema)]
            #[dsc_repo_schema(
                base_name = "valid.struct",
                folder_path = "example",
                should_bundle = true,
                i18n_root_key = "schemas.example.valid.struct"
            )]
            #[schemars(
                title = schema_i18n!("title"),
                description = schema_i18n!("description"),
                extend(
                    "markdownDescription" = schema_i18n!("markdownDescription"),
                )
            )]
            struct Example {
                #[schemars(
                    title = schema_i18n!("foo.title"),
                    description = schema_i18n!("foo.description"),
                    extend(
                        "markdownDescription" = schema_i18n!("foo.markdownDescription"),
                    )
                )]
                pub foo: String,

                #[schemars(
                    title = schema_i18n!("bar.title"),
                    description = schema_i18n!("bar.description"),
                    extend(
                        "markdownDescription" = schema_i18n!("bar.markdownDescription"),
                    )
                )]
                pub bar: i32,

                #[schemars(
                    title = schema_i18n!("baz.title"),
                    description = schema_i18n!("baz.description"),
                    extend(
                        "markdownDescription" = schema_i18n!("baz.markdownDescription"),
                    )
                )]
                pub baz: bool,
            }

            #[test] fn test_default_schema_id_uri() {
                assert_eq!(
                    Example::default_schema_id_uri(),
                    "https://aka.ms/dsc/schemas/v3/bundled/example/valid.struct.json".to_string()
                )
            }

            #[test] fn test_get_canonical_schema_id_uri() {
                assert_eq!(
                    Example::get_canonical_schema_id_uri(RecognizedSchemaVersion::V3),
                    "https://aka.ms/dsc/schemas/v3/example/valid.struct.json".to_string()
                )
            }

            #[test] fn test_get_bundled_schema_id_uri() {
                assert_eq!(
                    Example::get_bundled_schema_id_uri(RecognizedSchemaVersion::V3),
                    Some("https://aka.ms/dsc/schemas/v3/bundled/example/valid.struct.json".to_string())
                )
            }
            #[test] fn test_get_enhanced_schema_id_uri() {
                assert_eq!(
                    Example::get_enhanced_schema_id_uri(RecognizedSchemaVersion::V3),
                    Some("https://aka.ms/dsc/schemas/v3/bundled/example/valid.struct.vscode.json".to_string())
                )
            }

            #[test] fn test_schema_docs() {
                let schema = &schema_for!(Example);
                assert_eq!(
                    schema.get_keyword_as_str("title"),
                    Some("Example valid struct")
                );
                assert_eq!(
                    schema.get_keyword_as_str("description"),
                    Some("Defines a struct with the DscRepoSchema trait.")
                );
                assert_eq!(
                    schema.get_keyword_as_str("markdownDescription"),
                    Some("Defines a struct with the `DscRepoSchema` trait.")
                );

                for property_name in vec!["foo", "bar", "baz"] {
                    let property_schema = schema.get_property_subschema(property_name).unwrap();

                    assert_eq!(
                        property_schema.get_keyword_as_string("title"),
                        Some(format!("{property_name} field"))
                    );
                    assert_eq!(
                        property_schema.get_keyword_as_string("description"),
                        Some(format!("Defines the {property_name} field."))
                    );
                    assert_eq!(
                        property_schema.get_keyword_as_string("markdownDescription"),
                        Some(format!("Defines the `{property_name}` field."))
                    );
                }
            }
        }
    
        #[cfg(test)] mod with_schema_field {
            use pretty_assertions::assert_eq;
            use schemars::{JsonSchema, schema_for};
            use dsc_lib_jsonschema::{
                dsc_repo::{
                    DscRepoSchema,
                    RecognizedSchemaVersion,
                    UnrecognizedSchemaUri,
                    schema_i18n
                },
                schema_utility_extensions::SchemaUtilityExtensions
            };

            #[allow(dead_code)]
            #[derive(Clone, Debug, JsonSchema, DscRepoSchema)]
            #[dsc_repo_schema(
                base_name = "valid",
                folder_path = "example",
                should_bundle = true,
                i18n_root_key = "schemas.example.valid.struct",
                schema_field(
                    name = schema_version,
                    title = testing_title!(),
                    description = "An example struct with a schema field.",
                )
            )]
            #[schemars(
                title = schema_i18n!("title"),
                description = schema_i18n!("description"),
                extend(
                    "markdownDescription" = schema_i18n!("markdownDescription"),
                )
            )]
            struct Example {
                pub schema_version: String,

                #[schemars(
                    title = schema_i18n!("foo.title"),
                    description = schema_i18n!("foo.description"),
                    extend(
                        "markdownDescription" = schema_i18n!("foo.markdownDescription"),
                    )
                )]
                pub foo: String,
                
                #[schemars(
                    title = schema_i18n!("bar.title"),
                    description = schema_i18n!("bar.description"),
                    extend(
                        "markdownDescription" = schema_i18n!("bar.markdownDescription"),
                    )
                )]
                pub bar: i32,

                #[schemars(
                    title = schema_i18n!("baz.title"),
                    description = schema_i18n!("baz.description"),
                    extend(
                        "markdownDescription" = schema_i18n!("baz.markdownDescription"),
                    )
                )]
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

            #[test] fn test_schema_docs() {
                let schema = &schema_for!(Example);
                assert_eq!(
                    schema.get_keyword_as_str("title"),
                    Some("Example valid struct")
                );
                assert_eq!(
                    schema.get_keyword_as_str("description"),
                    Some("Defines a struct with the DscRepoSchema trait.")
                );
                assert_eq!(
                    schema.get_keyword_as_str("markdownDescription"),
                    Some("Defines a struct with the `DscRepoSchema` trait.")
                );

                for property_name in vec!["foo", "bar", "baz"] {
                    let property_schema = schema.get_property_subschema(property_name).unwrap();

                    assert_eq!(
                        property_schema.get_keyword_as_string("title"),
                        Some(format!("{property_name} field"))
                    );
                    assert_eq!(
                        property_schema.get_keyword_as_string("description"),
                        Some(format!("Defines the {property_name} field."))
                    );
                    assert_eq!(
                        property_schema.get_keyword_as_string("markdownDescription"),
                        Some(format!("Defines the `{property_name}` field."))
                    );
                }
            }
        }
    }
}
