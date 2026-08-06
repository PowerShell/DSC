// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[allow(non_snake_case)]
#[cfg(test)] mod VSCODE_DIALECT_SCHEMA_BUNDLED {
    use crate::vscode::dialect::VSCODE_DIALECT_SCHEMA_BUNDLED;

    #[test] fn meta_schema_is_valid() {
        let schema = VSCODE_DIALECT_SCHEMA_BUNDLED.clone();
        let result = jsonschema::meta::validate(
            schema.as_value()
        );
        assert!(result.is_ok(), "Unexpected error: {}", result.unwrap_err());
    }
}

#[allow(non_snake_case)]
#[cfg(test)] mod VSCODE_DIALECT_SCHEMA_CANONICAL {
    use crate::vscode::dialect::VSCODE_DIALECT_SCHEMA_CANONICAL;

    #[test] fn meta_schema_is_valid() {
        let schema = VSCODE_DIALECT_SCHEMA_CANONICAL.clone();
        let result = jsonschema::meta::validate(
            schema.as_value()
        );
        assert!(result.is_ok(), "Unexpected error: {}", result.unwrap_err());
    }
}

#[allow(non_snake_case)]
#[cfg(test)] mod VSCodeDialect {
    #[cfg(test)] mod json_schema_bundled {
        use schemars::{SchemaGenerator, generate::SchemaSettings};

        use crate::vscode::{dialect::VSCodeDialect, vocabulary::VSCodeVocabulary};

        #[test] fn returns_schema_with_bundled_definitions() {
            let schema = VSCodeDialect::json_schema_bundled(
                &mut SchemaGenerator::new(SchemaSettings::draft2020_12())
            );
            let has_defs = schema.get("$defs")
                .and_then(serde_json::Value::as_object)
                .is_some_and(|defs| defs.keys().len() > 0);

            assert!(has_defs);
        }
        #[test] fn includes_vscode_vocabulary() {
            let schema = VSCodeDialect::json_schema_bundled(
                &mut SchemaGenerator::new(SchemaSettings::draft2020_12())
            );
            let vocab = schema.get("$vocabulary")
                .and_then(serde_json::Value::as_object).unwrap();

            assert_eq!(vocab.get(VSCodeVocabulary::SPEC_URI).unwrap(), false)
        }
    }
    #[cfg(test)] mod json_schema_canonical {
        use pretty_assertions::assert_eq;
        use schemars::{SchemaGenerator, generate::SchemaSettings};

        use crate::vscode::{dialect::VSCodeDialect, vocabulary::VSCodeVocabulary};

        #[test] fn returns_schema_without_bundled_definitions() {
            let schema = VSCodeDialect::json_schema_canonical(
                &mut SchemaGenerator::new(SchemaSettings::draft2020_12())
            );

            assert_eq!(schema.get("$defs"), None);
        }
        #[test] fn includes_vscode_vocabulary() {
            let schema = VSCodeDialect::json_schema_canonical(
                &mut SchemaGenerator::new(SchemaSettings::draft2020_12())
            );
            let vocab = schema.get("$vocabulary")
                .and_then(serde_json::Value::as_object).unwrap();

            assert_eq!(vocab.get(VSCodeVocabulary::SPEC_URI).unwrap(), false)
        }
    }
    #[cfg(test)] mod schema_resource_bundled {
        use schemars::{SchemaGenerator, generate::SchemaSettings};

        use crate::vscode::dialect::VSCodeDialect;

        #[test] fn does_not_panic() {
            VSCodeDialect::schema_resource_bundled(
                &mut SchemaGenerator::new(SchemaSettings::draft2020_12())
            );
        }
    }
    #[cfg(test)] mod schema_resource_canonical {
        use schemars::{SchemaGenerator, generate::SchemaSettings};

        use crate::vscode::dialect::VSCodeDialect;

        #[test] fn does_not_panic() {
            VSCodeDialect::schema_resource_canonical(
                &mut SchemaGenerator::new(SchemaSettings::draft2020_12())
            );
        }
    }
    #[cfg(test)] mod default_schema_resource {
        use crate::vscode::dialect::VSCodeDialect;

        #[test] fn does_not_panic() {
            let _ = VSCodeDialect::default_schema_resource();
        }

        #[test] fn returns_resource_with_correct_id() {
            let resource = VSCodeDialect::default_schema_resource();
            let contents = resource.contents();
            let id = contents.get("$id").and_then(serde_json::Value::as_str);

            assert_eq!(id, Some(VSCodeDialect::SCHEMA_ID));
        }

        #[test] fn returns_canonical_form_without_bundled_defs() {
            let resource = VSCodeDialect::default_schema_resource();
            let contents = resource.contents();
            let has_defs = contents.get("$defs")
                .and_then(serde_json::Value::as_object)
                .is_some_and(|defs| !defs.is_empty());

            assert!(!has_defs, "default_schema_resource should return canonical form without bundled $defs");
        }
    }
}
