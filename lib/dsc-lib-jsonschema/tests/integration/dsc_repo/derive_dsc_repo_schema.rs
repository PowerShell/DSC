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
                    UnrecognizedSchemaUriError,
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
                    Err(UnrecognizedSchemaUriError(invalid_uri, Example::recognized_schema_uris()))
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
                    UnrecognizedSchemaUriError,
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
                    Err(UnrecognizedSchemaUriError(invalid_uri, Example::recognized_schema_uris()))
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


#[cfg(test)] mod for_root_document {
    #![allow(dead_code)]

    use std::sync::LazyLock;

    use pretty_assertions::assert_eq;
    use schemars::{JsonSchema, Schema, schema_for};
    use dsc_lib_jsonschema::{
        dsc_repo::{
            DscRepoSchema, RecognizedSchemaVersion, SchemaForm, UnrecognizedSchemaUriError, schema_i18n
        }, schema_utility_extensions::SchemaUtilityExtensions
    };

    #[derive(Clone, Debug, Default, JsonSchema, DscRepoSchema)]
    #[dsc_repo_schema(base_name = "string_enum", folder_path = "example/root/fields")]
    #[schemars(
        title = schema_i18n!("title"),
        description = schema_i18n!("description"),
        extend(
            "$id" = Self::default_export_schema_id_uri(),
            "markdownDescription" = schema_i18n!("markdownDescription"),
        ),
        rename_all = "camelCase"
    )]
    enum ExampleStringEnum {
        #[schemars(
            title = schema_i18n!("variants.foo.title"),
            description = schema_i18n!("variants.foo.description"),
            extend(
                "markdownDescription" = schema_i18n!("variants.foo.markdownDescription"),
            ),
        )]
        #[default]
        Foo,
        #[schemars(
            title = schema_i18n!("variants.bar.title"),
            description = schema_i18n!("variants.bar.description"),
            extend(
                "markdownDescription" = schema_i18n!("variants.bar.markdownDescription"),
            ),
        )]
        Bar,
        #[schemars(
            title = schema_i18n!("variants.baz.title"),
            description = schema_i18n!("variants.baz.description"),
            extend(
                "markdownDescription" = schema_i18n!("variants.baz.markdownDescription"),
            ),
        )]
        Baz,
    }

    #[derive(Clone, Debug, JsonSchema, DscRepoSchema)]
    #[dsc_repo_schema(base_name = "tagged_variant_enum", folder_path = "example/root/fields")]
    #[schemars(
        title = schema_i18n!("title"),
        description = schema_i18n!("description"),
        extend(
            "$id" = Self::default_export_schema_id_uri(),
            "markdownDescription" = schema_i18n!("markdownDescription"),
        ),
        rename_all = "camelCase",
    )]
    enum ExampleTaggedVariantEnum {
        #[schemars(
            title = schema_i18n!("variants.string.title"),
            description = schema_i18n!("variants.string.description"),
            extend(
                "markdownDescription" = schema_i18n!("variants.string.markdownDescription"),
            ),
        )]
        String(String),
        #[schemars(
            title = schema_i18n!("variants.boolean.title"),
            description = schema_i18n!("variants.boolean.description"),
            extend(
                "markdownDescription" = schema_i18n!("variants.boolean.markdownDescription"),
            ),
        )]
        Boolean(bool),
    }
    impl Default for ExampleTaggedVariantEnum {
        fn default() -> Self {
            Self::String(String::default())
        }
    }

    #[derive(Clone, Debug, JsonSchema, DscRepoSchema)]
    #[dsc_repo_schema(base_name = "untagged_variant_enum", folder_path = "example/root/fields")]
    #[schemars(
        title = schema_i18n!("title"),
        description = schema_i18n!("description"),
        extend(
            "$id" = Self::default_export_schema_id_uri(),
            "markdownDescription" = schema_i18n!("markdownDescription"),
        ),
        rename_all = "camelCase",
        untagged
    )]
    enum ExampleUntaggedVariantEnum {
        #[schemars(
            title = schema_i18n!("variants.string.title"),
            description = schema_i18n!("variants.string.description"),
            extend(
                "markdownDescription" = schema_i18n!("variants.string.markdownDescription"),
            ),
        )]
        String(String),
        #[schemars(
            title = schema_i18n!("variants.boolean.title"),
            description = schema_i18n!("variants.boolean.description"),
            extend(
                "markdownDescription" = schema_i18n!("variants.boolean.markdownDescription"),
            ),
        )]
        Boolean(bool),
    }
    impl Default for ExampleUntaggedVariantEnum {
        fn default() -> Self {
            Self::String(String::default())
        }
    }

    #[derive(Clone, Debug, Default, JsonSchema, DscRepoSchema)]
    #[dsc_repo_schema(base_name = "boolean_struct", folder_path = "example/root/fields")]
    #[schemars(
        title = schema_i18n!("title"),
        description = schema_i18n!("description"),
        extend(
            "$id" = Self::default_export_schema_id_uri(),
            "markdownDescription" = schema_i18n!("markdownDescription"),
        ),
        rename_all = "camelCase"
    )]
    struct ExampleBooleanStruct(bool);
    #[derive(Clone, Debug, Default, JsonSchema, DscRepoSchema)]
    #[dsc_repo_schema(base_name = "string_struct", folder_path = "example/root/fields")]
    #[schemars(
        title = schema_i18n!("title"),
        description = schema_i18n!("description"),
        extend(
            "$id" = Self::default_export_schema_id_uri(),
            "markdownDescription" = schema_i18n!("markdownDescription"),
        ),
        rename_all = "camelCase"
    )]
    struct ExampleStringStruct(String);

    #[derive(Clone, Debug, Default, JsonSchema, DscRepoSchema)]
    #[dsc_repo_schema(base_name = "object_struct", folder_path = "example/root/fields")]
    #[schemars(
        title = schema_i18n!("title"),
        description = schema_i18n!("description"),
        extend(
            "$id" = Self::default_export_schema_id_uri(),
            "markdownDescription" = schema_i18n!("markdownDescription"),
        ),
        rename_all = "camelCase"
    )]
    struct ExampleObjectStruct {
        #[schemars(
            title = schema_i18n!("fields.nested_string_enum.title"),
            description = schema_i18n!("fields.nested_string_enum.description"),
            extend(
                "markdownDescription" = schema_i18n!("fields.nested_string_enum.markdownDescription"),
            ),
        )]
        nested_string_enum: ExampleStringEnum,
        #[schemars(
            title = schema_i18n!("fields.nested_boolean_struct.title"),
            description = schema_i18n!("fields.nested_boolean_struct.description"),
            extend(
                "markdownDescription" = schema_i18n!("fields.nested_boolean_struct.markdownDescription"),
            ),
        )]
        nested_boolean_struct: ExampleBooleanStruct,
    }

    #[derive(Clone, Debug, JsonSchema, DscRepoSchema)]
    #[dsc_repo_schema(
        base_name = "document",
        folder_path = "example/root",
        should_bundle = true,
        schema_field(
            name = schema_version,
            title = schema_i18n!("fields.schema_version.title"),
            description = schema_i18n!("fields.schema_version.description"),
            markdown_description = schema_i18n!("fields.schema_version.markdownDescription")
        )
    )]
    #[schemars(
        title = schema_i18n!("title"),
        description = schema_i18n!("description"),
        extend(
            "$id" = Self::default_export_schema_id_uri(),
            "markdownDescription" = schema_i18n!("markdownDescription"),
        ),
        rename_all = "camelCase"
    )]
    struct ExampleRootDocument {
        pub schema_version: String,
        #[schemars(
            title = schema_i18n!("fields.string_enum.title"),
            description = schema_i18n!("fields.string_enum.description"),
            extend(
                "markdownDescription" = schema_i18n!("fields.string_enum.markdownDescription"),
            ),
        )]
        string_enum: ExampleStringEnum,
        #[schemars(
            title = schema_i18n!("fields.tagged_variant_enum.title"),
            description = schema_i18n!("fields.tagged_variant_enum.description"),
            extend(
                "markdownDescription" = schema_i18n!("fields.tagged_variant_enum.markdownDescription"),
            ),
        )]
        tagged_variant_enum: ExampleTaggedVariantEnum,
        #[schemars(
            title = schema_i18n!("fields.untagged_variant_enum.title"),
            description = schema_i18n!("fields.untagged_variant_enum.description"),
            extend(
                "markdownDescription" = schema_i18n!("fields.untagged_variant_enum.markdownDescription"),
            ),
        )]
        untagged_variant_enum: ExampleUntaggedVariantEnum,
        #[schemars(
            title = schema_i18n!("fields.boolean_struct.title"),
            description = schema_i18n!("fields.boolean_struct.description"),
            extend(
                "markdownDescription" = schema_i18n!("fields.boolean_struct.markdownDescription"),
            ),
        )]
        boolean_struct: ExampleBooleanStruct,
        #[schemars(
            title = schema_i18n!("fields.string_struct.title"),
            description = schema_i18n!("fields.string_struct.description"),
            extend(
                "markdownDescription" = schema_i18n!("fields.string_struct.markdownDescription"),
            ),
        )]
        string_struct: ExampleStringStruct,
        #[schemars(
            title = schema_i18n!("fields.object_struct.title"),
            description = schema_i18n!("fields.object_struct.description"),
            extend(
                "markdownDescription" = schema_i18n!("fields.object_struct.markdownDescription"),
            ),
        )]
        object_struct: ExampleObjectStruct,
    }
    impl Default for ExampleRootDocument {
        fn default() -> Self {
            Self {
                schema_version: Self::default_schema_id_uri(),
                string_enum: ExampleStringEnum::default(),
                tagged_variant_enum: ExampleTaggedVariantEnum::default(),
                untagged_variant_enum: ExampleUntaggedVariantEnum::default(),
                boolean_struct: ExampleBooleanStruct::default(),
                string_struct: ExampleStringStruct::default(),
                object_struct: ExampleObjectStruct::default(),
            }
        }
    }

    static ROOT_SCHEMA_CANONICAL: LazyLock<Schema> = LazyLock::new(|| {
        ExampleRootDocument::generate_exportable_schema(
            RecognizedSchemaVersion::VNext,
            SchemaForm::Canonical
        )
    });

    #[test] fn test_default_schema_id_uri() {
        assert_eq!(
            ExampleRootDocument::default_schema_id_uri(),
            "https://aka.ms/dsc/schemas/v3/bundled/example/root/document.json".to_string()
        )
    }

    #[test] fn test_get_canonical_schema_id_uri() {
        assert_eq!(
            ExampleRootDocument::get_canonical_schema_id_uri(RecognizedSchemaVersion::V3),
            "https://aka.ms/dsc/schemas/v3/example/root/document.json".to_string()
        )
    }

    #[test] fn test_get_bundled_schema_id_uri() {
        assert_eq!(
            ExampleRootDocument::get_bundled_schema_id_uri(RecognizedSchemaVersion::V3),
            Some("https://aka.ms/dsc/schemas/v3/bundled/example/root/document.json".to_string())
        )
    }
    
    #[test] fn test_get_enhanced_schema_id_uri() {
        assert_eq!(
            ExampleRootDocument::get_enhanced_schema_id_uri(RecognizedSchemaVersion::V3),
            Some("https://aka.ms/dsc/schemas/v3/bundled/example/root/document.vscode.json".to_string())
        )
    }

    #[test] fn test_recognized_schema_uris_subschema() {
        let ref mut generator = schemars::SchemaGenerator::default();
        let subschema = ExampleRootDocument::recognized_schema_uris_subschema(generator);

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
            Some("Schema version")
        );

        assert_eq!(
            subschema.get_keyword_as_str("description"),
            Some("Defines the schema version with the DscRepoSchema trait.")
        );
        assert_eq!(
            subschema.get_keyword_as_str("markdownDescription"),
            Some("Defines the schema version with the `DscRepoSchema` trait.")
        );
    }

    #[test] fn test_is_recognized_schema_uri() {
        assert_eq!(
            ExampleRootDocument::is_recognized_schema_uri(&"https://incorrect/uri.json".to_string()),
            false
        );

        assert_eq!(
            ExampleRootDocument::is_recognized_schema_uri(&ExampleRootDocument::default_schema_id_uri()),
            true
        );
    }

    #[test] fn test_validate_schema_uri() {
        let valid_instance = ExampleRootDocument::default();

        assert_eq!(
            valid_instance.validate_schema_uri(),
            Ok(())
        );

        let invalid_uri = "https://incorrect/uri.json".to_string();
        let invalid_instance = ExampleRootDocument {
            schema_version: invalid_uri.clone(),
            ..Default::default()
        };

        assert_eq!(
            invalid_instance.validate_schema_uri(),
            Err(UnrecognizedSchemaUriError(invalid_uri, ExampleRootDocument::recognized_schema_uris()))
        )
    }

    #[test] fn test_schema_docs() {
        let schema = &schema_for!(ExampleRootDocument);
        assert_eq!(
            schema.get_keyword_as_str("title"),
            Some("Example document title")
        );
        assert_eq!(
            schema.get_keyword_as_str("description"),
            Some("Example document description")
        );
        assert_eq!(
            schema.get_keyword_as_str("markdownDescription"),
            Some("Example document markdown description.")
        );

        let fields = vec![
            "string_enum",
            "tagged_variant_enum",
            "untagged_variant_enum",
            "boolean_struct",
            "string_struct",
            "object_struct"
        ];
        fn camel_case_field(s: &str) -> String {
            let mut result = String::with_capacity(s.len());
            let mut uppercase_next = false;

            for character in s.chars() {
                if character == '_' {
                    uppercase_next = !result.is_empty();
                } else if result.is_empty() {
                    result.extend(character.to_lowercase());
                    uppercase_next = false;
                } else if uppercase_next {
                    result.extend(character.to_uppercase());
                    uppercase_next = false;
                } else {
                    result.push(character);
                }
            }

            result
        }
        
        for field in fields {
            let field_text = field.replace("_", " ");
            let property_name = camel_case_field(field);
            let property_schema = schema.get_property_subschema(&property_name).unwrap();

            assert_eq!(
                property_schema.get_keyword_as_string("title"),
                Some(format!("Example document {field_text} field"))
            );
            assert_eq!(
                property_schema.get_keyword_as_string("description"),
                Some(format!("Example document {field_text} description."))
            );
            assert_eq!(
                property_schema.get_keyword_as_string("markdownDescription"),
                Some(format!("Example document {field_text} markdown description."))
            );
        }
    }

}