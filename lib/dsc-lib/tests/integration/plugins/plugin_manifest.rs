// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[cfg(test)]
mod methods {
    #[cfg(test)]
    mod validate {
        use dsc_lib::dscerror::DscError;
        use dsc_lib::plugins::plugin_manifest::PluginManifest;
        use dsc_lib::schemas::dsc_repo::DscRepoSchema;
        use serde_json::json;
        use test_case::test_case;

        use super::super::super::helpers::{manifest_json, MANIFEST_SCHEMA_URI};

        /// Returns a manifest whose command passes a single `manifestRelativePath` argument.
        fn manifest_with_path(path: &str) -> PluginManifest {
            let mut manifest = manifest_json("Test/Graph", "0.1.0", "graph");
            manifest["command"]["args"] = json!(["-File", { "manifestRelativePath": path }]);
            serde_json::from_value(manifest).expect("manifest should deserialize")
        }

        #[test]
        fn default_schema_uri_is_the_documented_uri() {
            pretty_assertions::assert_eq!(PluginManifest::default_schema_id_uri(), MANIFEST_SCHEMA_URI);
        }

        #[test]
        fn manifest_with_required_properties_is_valid() {
            let manifest: PluginManifest = serde_json::from_value(manifest_json("Test/Graph", "0.1.0", "graph"))
                .expect("manifest should deserialize");

            assert!(manifest.validate().is_ok());
        }

        #[test]
        fn unrecognized_schema_uri_is_invalid() {
            let mut manifest = manifest_json("Test/Graph", "0.1.0", "graph");
            manifest["$schema"] = json!("https://example.com/not-a-dsc-schema.json");
            let manifest: PluginManifest = serde_json::from_value(manifest).expect("manifest should deserialize");

            assert!(matches!(manifest.validate(), Err(DscError::UnrecognizedSchemaUri(_))));
        }

        #[test_case("graph.ps1"; "file next to the manifest")]
        #[test_case("scripts/graph.py"; "file in a subdirectory")]
        #[test_case("../shared/graph.ps1"; "file in a sibling directory")]
        fn relative_path_is_valid(path: &str) {
            assert!(manifest_with_path(path).validate().is_ok());
        }

        #[test_case(""; "empty path")]
        #[test_case("/tools/graph.ps1"; "path with a root")]
        #[cfg_attr(windows, test_case(r"C:\tools\graph.ps1"; "path with a drive and a root"))]
        #[cfg_attr(windows, test_case("C:graph.ps1"; "path with a drive and no root"))]
        #[cfg_attr(windows, test_case(r"\\server\share\graph.ps1"; "UNC path"))]
        fn path_that_is_not_relative_is_invalid(path: &str) {
            let err = manifest_with_path(path).validate().unwrap_err();

            assert!(matches!(err, DscError::InvalidManifest(_)), "unexpected error: {err:?}");
            assert!(err.to_string().contains("Test/Graph"), "error should name the plug-in: {err}");
        }
    }

    #[cfg(test)]
    mod to_command_args {
        use std::ffi::OsString;
        use std::path::Path;

        use dsc_lib::plugins::plugin_manifest::PluginManifest;
        use serde_json::json;

        use super::super::super::helpers::manifest_json;

        #[test]
        fn without_args_returns_no_arguments() {
            let manifest: PluginManifest = serde_json::from_value(manifest_json("Test/Graph", "0.1.0", "graph"))
                .expect("manifest should deserialize");

            assert!(manifest.command.to_command_args(Path::new("plugins")).is_empty());
        }

        #[test]
        fn passes_strings_as_written_and_resolves_manifest_relative_paths() {
            let mut manifest = manifest_json("Test/Graph", "0.1.0", "graph");
            manifest["command"]["args"] = json!([
                "-NoProfile",
                "-File",
                { "manifestRelativePath": "graph.ps1" },
                "./not-resolved.txt"
            ]);
            let manifest: PluginManifest = serde_json::from_value(manifest).expect("manifest should deserialize");
            let directory = Path::new("plugins").join("graph");

            pretty_assertions::assert_eq!(
                manifest.command.to_command_args(&directory),
                vec![
                    OsString::from("-NoProfile"),
                    OsString::from("-File"),
                    directory.join("graph.ps1").into_os_string(),
                    OsString::from("./not-resolved.txt"),
                ]
            );
        }
    }
}

#[cfg(test)]
mod serde {
    use std::path::PathBuf;

    use dsc_lib::plugins::plugin_manifest::{PluginArgKind, PluginManifest};
    use serde_json::{json, Value};
    use test_case::test_case;

    use super::super::helpers::manifest_json;

    /// Returns the minimal manifest with one property replaced or removed.
    fn manifest_with(property: &str, value: Option<Value>) -> Value {
        let mut manifest = manifest_json("Test/Graph", "0.1.0", "graph");
        let object = manifest.as_object_mut().expect("manifest is an object");
        match value {
            Some(value) => { object.insert(property.to_string(), value); },
            None => { object.remove(property); },
        }
        manifest
    }

    #[test]
    fn deserializing_minimal_manifest() {
        let manifest: PluginManifest = serde_json::from_value(manifest_json("Test/Graph", "0.1.0", "graph"))
            .expect("manifest should deserialize");

        pretty_assertions::assert_eq!(manifest.r#type.to_string(), "Test/Graph");
        pretty_assertions::assert_eq!(manifest.version.to_string(), "0.1.0");
        pretty_assertions::assert_eq!(manifest.command.name.to_string(), "graph");
        pretty_assertions::assert_eq!(manifest.command.executable, "plugin-executable");
        assert!(manifest.command.args.is_none());
        assert!(manifest.condition.is_none());
        assert!(manifest.deprecation_message.is_none());
        assert!(manifest.description.is_none());
        assert!(manifest.tags.is_empty());
        assert!(manifest.metadata.is_none());
    }

    #[test]
    fn deserializing_manifest_with_every_property() {
        let mut manifest = manifest_json("Test/Graph", "0.1.0", "graph");
        manifest["condition"] = json!("[equals(1, 1)]");
        manifest["deprecationMessage"] = json!("Use 'dsc diagram' instead.");
        manifest["description"] = json!("Show the dependency graph of a configuration document");
        manifest["tags"] = json!(["graph", "authoring"]);
        manifest["metadata"] = json!({ "owner": "tests" });
        manifest["command"]["args"] = json!(["-File", { "manifestRelativePath": "graph.ps1" }]);

        let manifest: PluginManifest = serde_json::from_value(manifest).expect("manifest should deserialize");

        pretty_assertions::assert_eq!(manifest.condition.as_deref(), Some("[equals(1, 1)]"));
        pretty_assertions::assert_eq!(manifest.deprecation_message.as_deref(), Some("Use 'dsc diagram' instead."));
        pretty_assertions::assert_eq!(
            manifest.description.as_deref(),
            Some("Show the dependency graph of a configuration document")
        );
        pretty_assertions::assert_eq!(manifest.tags.len(), 2);
        pretty_assertions::assert_eq!(manifest.metadata.expect("metadata is defined")["owner"], json!("tests"));
        pretty_assertions::assert_eq!(
            manifest.command.args,
            Some(vec![
                PluginArgKind::String("-File".to_string()),
                PluginArgKind::ManifestRelativePath { manifest_relative_path: PathBuf::from("graph.ps1") },
            ])
        );
    }

    #[test]
    fn deserializing_yaml_manifest() {
        let yaml = r#"
            $schema: https://aka.ms/dsc/schemas/v3/bundled/plugin/manifest.json
            type: Test/Graph
            version: 0.1.0
            command:
              name: graph
              executable: pwsh
              args:
                - -File
                - manifestRelativePath: graph.ps1
        "#;

        let manifest: PluginManifest = serde_yaml::from_str(yaml).expect("manifest should deserialize");

        pretty_assertions::assert_eq!(manifest.command.name.to_string(), "graph");
        pretty_assertions::assert_eq!(manifest.command.args.map(|args| args.len()), Some(2));
    }

    #[test_case("$schema"; "schema")]
    #[test_case("type"; "type")]
    #[test_case("version"; "version")]
    #[test_case("command"; "command")]
    fn deserializing_without_required_property_fails(property: &str) {
        let manifest = manifest_with(property, None);

        assert!(serde_json::from_value::<PluginManifest>(manifest).is_err());
    }

    #[test_case("type", json!("NotATypeName"); "type that is not a fully qualified type name")]
    #[test_case("version", json!("1.0"); "version that is not a semantic version")]
    #[test_case("command", json!("graph"); "command that is not an object")]
    #[test_case("command", json!({ "name": "graph" }); "command without an executable")]
    #[test_case("command", json!({ "executable": "pwsh" }); "command without a name")]
    #[test_case("command", json!({ "name": "Graph", "executable": "pwsh" }); "command with an invalid name")]
    #[test_case(
        "command",
        json!({ "name": "graph", "executable": "pwsh", "input": "stdin" });
        "command with an unknown property"
    )]
    #[test_case(
        "command",
        json!({ "name": "graph", "executable": "pwsh", "args": [1] });
        "argument that is a number"
    )]
    #[test_case(
        "command",
        json!({ "name": "graph", "executable": "pwsh", "args": [{ "jsonInputArg": "--input" }] });
        "argument kind of another manifest"
    )]
    #[test_case(
        "command",
        json!({ "name": "graph", "executable": "pwsh", "args": [{ "manifestRelativePath": "a", "other": "b" }] });
        "argument object with an extra property"
    )]
    #[test_case("discover", json!({ "executable": "pwsh" }); "property of an extension manifest")]
    fn deserializing_invalid_property_fails(property: &str, value: Value) {
        let manifest = manifest_with(property, Some(value));

        assert!(serde_json::from_value::<PluginManifest>(manifest).is_err());
    }

    #[test]
    fn serializing_omits_undefined_optional_properties() {
        let json = manifest_json("Test/Graph", "0.1.0", "graph");
        let manifest: PluginManifest = serde_json::from_value(json.clone()).expect("manifest should deserialize");

        pretty_assertions::assert_eq!(
            serde_json::to_value(&manifest).expect("serialization should never fail"),
            json
        );
    }

    #[test]
    fn serializing_round_trips_arguments() {
        let mut json = manifest_json("Test/Graph", "0.1.0", "graph");
        json["command"]["args"] = json!(["-File", { "manifestRelativePath": "graph.ps1" }]);
        let manifest: PluginManifest = serde_json::from_value(json.clone()).expect("manifest should deserialize");

        pretty_assertions::assert_eq!(
            serde_json::to_value(&manifest).expect("serialization should never fail"),
            json
        );
    }
}

#[cfg(test)]
mod manifest {
    crate::macros::test_dsc_repo_schema! {
        define_statics: {
            dsc_lib::plugins::plugin_manifest::PluginManifest,
            properties: {
                TYPE_SCHEMA => "type",
                VERSION_SCHEMA => "version",
                CONDITION_SCHEMA => "condition",
                DEPRECATION_MESSAGE_SCHEMA => "deprecationMessage",
                DESCRIPTION_SCHEMA => "description",
                TAGS_SCHEMA => "tags",
                METADATA_SCHEMA => "metadata",
            }
        },
        test_meta_schema: {},
        test_docs: {
            title => "title",
            description => "description",
            markdown_description => "markdownDescription",
            type_title => (TYPE_SCHEMA, "title"),
            type_description => (TYPE_SCHEMA, "description"),
            type_markdown_description => (TYPE_SCHEMA, "markdownDescription"),
            version_title => (VERSION_SCHEMA, "title"),
            version_description => (VERSION_SCHEMA, "description"),
            version_markdown_description => (VERSION_SCHEMA, "markdownDescription"),
            condition_title => (CONDITION_SCHEMA, "title"),
            condition_description => (CONDITION_SCHEMA, "description"),
            condition_markdown_description => (CONDITION_SCHEMA, "markdownDescription"),
            deprecation_message_title => (DEPRECATION_MESSAGE_SCHEMA, "title"),
            deprecation_message_description => (DEPRECATION_MESSAGE_SCHEMA, "description"),
            deprecation_message_markdown_description => (DEPRECATION_MESSAGE_SCHEMA, "markdownDescription"),
            description_title => (DESCRIPTION_SCHEMA, "title"),
            description_description => (DESCRIPTION_SCHEMA, "description"),
            description_markdown_description => (DESCRIPTION_SCHEMA, "markdownDescription"),
            tags_title => (TAGS_SCHEMA, "title"),
            tags_description => (TAGS_SCHEMA, "description"),
            tags_markdown_description => (TAGS_SCHEMA, "markdownDescription"),
            metadata_title => (METADATA_SCHEMA, "title"),
            metadata_description => (METADATA_SCHEMA, "description"),
            metadata_markdown_description => (METADATA_SCHEMA, "markdownDescription")
        },
        test_validation: {
            manifest_with_required_properties_is_valid => {
                input_json: json!({
                    "$schema": "https://aka.ms/dsc/schemas/v3/bundled/plugin/manifest.json",
                    "type": "Test/Graph",
                    "version": "0.1.0",
                    "command": { "name": "graph", "executable": "pwsh" }
                }),
                expected_valid: true,
            },
            manifest_with_every_property_is_valid => {
                input_json: json!({
                    "$schema": "https://aka.ms/dsc/schemas/v3/bundled/plugin/manifest.json",
                    "type": "Test/Graph",
                    "version": "0.1.0",
                    "condition": "[not(equals(tryWhich('pwsh'), null()))]",
                    "deprecationMessage": "Use 'dsc diagram' instead.",
                    "description": "Show the dependency graph of a configuration document",
                    "tags": ["graph", "authoring"],
                    "metadata": { "owner": "tests" },
                    "command": {
                        "name": "graph",
                        "executable": "pwsh",
                        "args": ["-NoProfile", "-File", { "manifestRelativePath": "graph.ps1" }]
                    }
                }),
                expected_valid: true,
            },
            manifest_without_schema_is_invalid => {
                input_json: json!({
                    "type": "Test/Graph",
                    "version": "0.1.0",
                    "command": { "name": "graph", "executable": "pwsh" }
                }),
                expected_valid: false,
            },
            manifest_with_unrecognized_schema_is_invalid => {
                input_json: json!({
                    "$schema": "https://example.com/not-a-dsc-schema.json",
                    "type": "Test/Graph",
                    "version": "0.1.0",
                    "command": { "name": "graph", "executable": "pwsh" }
                }),
                expected_valid: false,
            },
            manifest_without_command_is_invalid => {
                input_json: json!({
                    "$schema": "https://aka.ms/dsc/schemas/v3/bundled/plugin/manifest.json",
                    "type": "Test/Graph",
                    "version": "0.1.0"
                }),
                expected_valid: false,
            },
            manifest_with_invalid_command_name_is_invalid => {
                input_json: json!({
                    "$schema": "https://aka.ms/dsc/schemas/v3/bundled/plugin/manifest.json",
                    "type": "Test/Graph",
                    "version": "0.1.0",
                    "command": { "name": "Graph", "executable": "pwsh" }
                }),
                expected_valid: false,
            },
            manifest_with_unknown_property_is_invalid => {
                input_json: json!({
                    "$schema": "https://aka.ms/dsc/schemas/v3/bundled/plugin/manifest.json",
                    "type": "Test/Graph",
                    "version": "0.1.0",
                    "command": { "name": "graph", "executable": "pwsh" },
                    "discover": { "executable": "pwsh" }
                }),
                expected_valid: false,
            },
            manifest_with_unknown_argument_kind_is_invalid => {
                input_json: json!({
                    "$schema": "https://aka.ms/dsc/schemas/v3/bundled/plugin/manifest.json",
                    "type": "Test/Graph",
                    "version": "0.1.0",
                    "command": {
                        "name": "graph",
                        "executable": "pwsh",
                        "args": [{ "jsonInputArg": "--input" }]
                    }
                }),
                expected_valid: false,
            },
            string_is_invalid => {
                input_json: json!("Test/Graph"),
                expected_valid: false,
            }
        }
    }
}

#[cfg(test)]
mod command {
    crate::macros::test_dsc_repo_schema! {
        define_statics: {
            dsc_lib::plugins::plugin_manifest::PluginCommand,
            properties: {
                NAME_SCHEMA => "name",
                EXECUTABLE_SCHEMA => "executable",
                ARGS_SCHEMA => "args",
            }
        },
        test_meta_schema: {},
        test_docs: {
            title => "title",
            description => "description",
            markdown_description => "markdownDescription",
            name_title => (NAME_SCHEMA, "title"),
            name_description => (NAME_SCHEMA, "description"),
            name_markdown_description => (NAME_SCHEMA, "markdownDescription"),
            executable_title => (EXECUTABLE_SCHEMA, "title"),
            executable_description => (EXECUTABLE_SCHEMA, "description"),
            executable_markdown_description => (EXECUTABLE_SCHEMA, "markdownDescription"),
            args_title => (ARGS_SCHEMA, "title"),
            args_description => (ARGS_SCHEMA, "description"),
            args_markdown_description => (ARGS_SCHEMA, "markdownDescription")
        },
        test_validation: {
            command_with_name_and_executable_is_valid => {
                input_json: json!({ "name": "graph", "executable": "pwsh" }),
                expected_valid: true,
            },
            command_with_string_arguments_is_valid => {
                input_json: json!({ "name": "graph", "executable": "pwsh", "args": ["-NoProfile", "-File"] }),
                expected_valid: true,
            },
            command_with_manifest_relative_path_argument_is_valid => {
                input_json: json!({
                    "name": "graph",
                    "executable": "pwsh",
                    "args": ["-File", { "manifestRelativePath": "graph.ps1" }]
                }),
                expected_valid: true,
            },
            command_with_empty_arguments_is_valid => {
                input_json: json!({ "name": "graph", "executable": "pwsh", "args": [] }),
                expected_valid: true,
            },
            command_without_name_is_invalid => {
                input_json: json!({ "executable": "pwsh" }),
                expected_valid: false,
            },
            command_without_executable_is_invalid => {
                input_json: json!({ "name": "graph" }),
                expected_valid: false,
            },
            command_with_numeric_argument_is_invalid => {
                input_json: json!({ "name": "graph", "executable": "pwsh", "args": [1] }),
                expected_valid: false,
            },
            command_with_unknown_property_is_invalid => {
                input_json: json!({ "name": "graph", "executable": "pwsh", "input": "stdin" }),
                expected_valid: false,
            },
            empty_object_is_invalid => {
                input_json: json!({}),
                expected_valid: false,
            }
        }
    }
}
