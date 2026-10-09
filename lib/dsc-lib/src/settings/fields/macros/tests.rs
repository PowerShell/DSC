// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![allow(dead_code)]

use crate::schemas::schema_utility_extensions::SchemaUtilityExtensions;
use crate::schemas::dsc_repo::DscRepoSchema;

use super::*;

#[cfg(test)] mod define_container_field {
    use super::*;
    use pretty_assertions::assert_eq;

    define_container_field!{
        base_name: "container",
        folder_path: "settings/test",
        field_definition: {
            /// Manages `example` data.
            #[schemars(extend("x-extra-keyword" = true))]
            struct ExampleFileData {
                /// The `foo` field
                foo: bool = false,
                /// The `bar` field
                bar: String = String::new(),
                /// The `baz` field
                baz: Vec<String> = Vec::new(),
            }
        },
        resolved_container_type: {
            ExampleResolvedSettings
        },
        code_default_type: {
            /// The structure of the default values for the `example` field.
            ExampleCodeDefaults
        },
        code_default_const: {
            /// The default settings for the `example` field.
            CODE_DEFAULT_EXAMPLE
        }
    }

    #[test]
    fn check_schema() {
        let schema = schemars::schema_for!(ExampleFileData);
        assert_eq!(
            schema.get("x-extra-keyword"),
            Some(&serde_json::json!(true))
        );
        assert_eq!(
            schema.get_keyword_as_string("$schema"),
            Some(ExampleFileData::default_export_meta_schema_uri())
        );
        assert_eq!(
            schema.get_keyword_as_string("$id"),
            Some(ExampleFileData::default_export_schema_id_uri())
        );
        for property_name in ["foo", "bar", "baz"] {
            let property_schema = schema
                .get_property_subschema(property_name)
                .unwrap_or_else(|| panic!("Property schema for '{property_name}' not found"));

            assert_eq!(
                property_schema.get_keyword_as_string("title"),
                Some(format!("Container '{}' field title", property_name))
            );
            assert_eq!(
                property_schema.get_keyword_as_string("description"),
                Some(format!("Container '{}' field description", property_name))
            );
            assert_eq!(
                property_schema.get_keyword_as_string("markdownDescription"),
                Some(format!("Container '{}' field markdown description", property_name))
            );
        }
    }

    #[test] fn check_emitted() {
        let default = CODE_DEFAULT_EXAMPLE.clone();
        assert_eq!(default.foo, false);
        assert_eq!(default.bar, String::new());
        assert_eq!(default.baz, Vec::<String>::new());

        let resolved = ExampleResolvedSettings::default();
        assert_eq!(resolved.foo.value, false);
        assert_eq!(resolved.bar.value, String::new());
        assert_eq!(resolved.baz.value, Vec::<String>::new());
        assert_eq!(resolved.foo.scope, crate::settings::DscSettingsScope::Default);
        assert_eq!(resolved.bar.scope, crate::settings::DscSettingsScope::Default);
        assert_eq!(resolved.baz.scope, crate::settings::DscSettingsScope::Default);
    }
}

#[cfg(test)] mod define_boolean_field {
    use super::*;
    use pretty_assertions::assert_eq;

    define_boolean_field! {
        base_name: "boolean",
        folder_path: "settings/test",
        field_definition: {
            /// The boolean field
            #[schemars(extend("x-extra-keyword" = true))]
            struct ExampleField;
        },
        code_default_const: {
            /// The default settings for the `boolean` field.
            CODE_DEFAULT_BOOLEAN = true;
        }
    }

    #[test]
    fn check_schema() {
        let schema = schemars::schema_for!(ExampleField);
        assert_eq!(
            schema.get("x-extra-keyword"),
            Some(&serde_json::json!(true))
        );
        assert_eq!(
            schema.get_keyword_as_string("$schema"),
            Some(ExampleField::default_export_meta_schema_uri())
        );
        assert_eq!(
            schema.get_keyword_as_string("$id"),
            Some(ExampleField::default_export_schema_id_uri())
        );
        assert_eq!(
            schema.get_keyword_as_str("title"),
            Some("Boolean title")
        );
        assert_eq!(
            schema.get_keyword_as_str("description"),
            Some("Boolean description")
        );
        assert_eq!(
            schema.get_keyword_as_str("markdownDescription"),
            Some("Boolean markdown description")
        );
        assert_eq!(
            schema.get_keyword_as_array("enum"),
            Some(&vec![serde_json::json!(true), serde_json::json!(false)])
        );
        assert_eq!(
            schema.get_keyword_as_array("markdownEnumDescriptions"),
            Some(&vec![
                serde_json::json!("Boolean markdown enum description for true"),
                serde_json::json!("Boolean markdown enum description for false")
            ])
        );
    }

    #[test] fn check_emitted() {
        let default = CODE_DEFAULT_BOOLEAN;
        assert_eq!(default, ExampleField::new(true));
        // From impls
        let from_bool: ExampleField = true.into();
        let from_field: bool = ExampleField::new(true).into();
        // Equality checks
        assert!(from_bool == true);
        assert!(false != from_bool);
        assert!(from_field == ExampleField::new(true));
        assert!(ExampleField::new(false) != from_field);
        // deref
        let field = ExampleField::new(true);
        assert!(*field);
        // as_ref
        let field_ref= field.as_ref();
        assert_eq!(field_ref, &true);
        // display
        assert_eq!(&format!("{}", field), "true");
    }
}

#[cfg(test)] mod define_string_field {
    use crate::settings::DscSettingsError;

use super::*;
    use pretty_assertions::assert_eq;

    define_string_enum_field!{
        base_name: "string",
        folder_path: "settings/test",
        parse_error_variant: InvalidTracingFormat, // Reuse real error to avoid cluttering the enum
        field_definition: {
            /// The string enum field
            #[schemars(extend("x-extra-keyword" = true))]
            enum ExampleField {
                Foo { display: "foo", lowercase: "foo" },
                Bar { display: "bar", lowercase: "bar" },
                Baz { display: "baz", lowercase: "baz" },
            }
        }
    }

    #[test]
    fn check_schema() {
        let schema = schemars::schema_for!(ExampleField);
        assert_eq!(
            schema.get("x-extra-keyword"),
            Some(&serde_json::json!(true))
        );
        assert_eq!(
            schema.get_keyword_as_string("$schema"),
            Some(ExampleField::default_export_meta_schema_uri())
        );
        assert_eq!(
            schema.get_keyword_as_string("$id"),
            Some(ExampleField::default_export_schema_id_uri())
        );
        assert_eq!(
            schema.get_keyword_as_str("title"),
            Some("String title")
        );
        assert_eq!(
            schema.get_keyword_as_str("description"),
            Some("String description")
        );
        assert_eq!(
            schema.get_keyword_as_str("markdownDescription"),
            Some("String markdown description")
        );
        assert_eq!(
            schema.get_keyword_as_array("enum"),
            Some(&vec![
                serde_json::json!("foo"),
                serde_json::json!("bar"),
                serde_json::json!("baz"),
            ])
        );
        assert_eq!(
            schema.get_keyword_as_array("markdownEnumDescriptions"),
            Some(&vec![
                serde_json::json!("String 'foo' variant markdown description"),
                serde_json::json!("String 'bar' variant markdown description"),
                serde_json::json!("String 'baz' variant markdown description"),
            ])
        );
    }

    #[test]
    fn check_emitted() {
        assert_eq!(
            ExampleField::ALL,
            [ExampleField::Foo, ExampleField::Bar, ExampleField::Baz]
        );
        // Verify parser
        let parse_cases = [
            (ExampleField::Foo, ["foo", "FOO", "FoO"]),
            (ExampleField::Bar, ["bar", "BAR", "BaR"]),
            (ExampleField::Baz, ["baz", "BAZ", "BaZ"]),
        ];
        for (variant, texts) in parse_cases {
            for text in texts {
                assert_eq!(
                    ExampleField::parse(text).unwrap_or_else(|_| panic!("{text} should parse as {variant:?}")),
                    variant
                );
                // Ensure FromStr is implemented to use parser
                assert_eq!(
                    text.parse::<ExampleField>().unwrap_or_else(|_| panic!("{text} should parse as {variant:?}")),
                    variant
                );
                // Ensure TryFrom<String> reuses parser
                assert_eq!(
                    ExampleField::try_from(String::from(text)).unwrap_or_else(|_| panic!("{text} should convert as {variant:?}")),
                    variant
                );
                // Ensure TryFrom<&str> reuses parser
                assert_eq!(
                    ExampleField::try_from(text).unwrap_or_else(|_| panic!("{text} should convert as {variant:?}")),
                    variant
                );
            }
        }
        assert!(
            matches!(
                ExampleField::parse("invalid").expect_err("invalid should not parse"),
                DscSettingsError::InvalidTracingFormat{text} if text == String::from("invalid")
            )
        );
        // Verify display and to string
        assert_eq!(ExampleField::Foo.to_string(), "foo");
        assert_eq!(&format!("Variant {}", ExampleField::Foo), "Variant foo");
        assert_eq!(&format!("Variant {}", ExampleField::Bar), "Variant bar");
        assert_eq!(&format!("Variant {}", ExampleField::Baz), "Variant baz");
        assert_eq!(ExampleField::Bar.to_string(), "bar");
        assert_eq!(ExampleField::Baz.to_string(), "baz");
        // Verify variant constants for error messages
        assert_eq!(
            ExampleField::VARIANT_LIST,
            "`foo`, `bar`"
        );
        assert_eq!(
            ExampleField::VARIANT_LAST,
            "`baz`"
        );
    }
}