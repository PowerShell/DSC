// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[cfg(test)]
mod methods {
    #[cfg(test)]
    mod from_manifest {
        use std::path::Path;

        use dsc_lib::plugins::dscplugin::DscPlugin;
        use dsc_lib::plugins::plugin_manifest::PluginManifest;
        use serde_json::json;

        use super::super::super::helpers::{manifest, manifest_json};

        #[test]
        fn copies_the_identity_of_the_manifest() {
            let manifest = manifest("Test/Graph", "1.2.3", "graph");
            let path = Path::new("plugins").join("graph.dsc.plugin.json");

            let plugin = DscPlugin::from_manifest(&path, manifest.clone());

            pretty_assertions::assert_eq!(plugin.type_name.to_string(), "Test/Graph");
            pretty_assertions::assert_eq!(plugin.version.to_string(), "1.2.3");
            pretty_assertions::assert_eq!(plugin.command.to_string(), "graph");
            pretty_assertions::assert_eq!(plugin.description, None);
            pretty_assertions::assert_eq!(plugin.deprecation_message, None);
            pretty_assertions::assert_eq!(plugin.path, path);
            pretty_assertions::assert_eq!(plugin.directory, Path::new("plugins"));
            pretty_assertions::assert_eq!(plugin.manifest, manifest);
        }

        #[test]
        fn copies_the_description_and_deprecation_message() {
            let mut json = manifest_json("Test/Graph", "1.2.3", "graph");
            json["description"] = json!("Show the dependency graph of a configuration document");
            json["deprecationMessage"] = json!("Use 'dsc diagram' instead.");
            let manifest: PluginManifest = serde_json::from_value(json).expect("manifest should deserialize");

            let plugin = DscPlugin::from_manifest(Path::new("graph.dsc.plugin.json"), manifest);

            pretty_assertions::assert_eq!(
                plugin.description.as_deref(),
                Some("Show the dependency graph of a configuration document")
            );
            pretty_assertions::assert_eq!(plugin.deprecation_message.as_deref(), Some("Use 'dsc diagram' instead."));
        }

        #[test]
        fn is_not_shadowed() {
            let plugin = DscPlugin::from_manifest(
                Path::new("graph.dsc.plugin.json"),
                manifest("Test/Graph", "1.2.3", "graph")
            );

            assert!(!plugin.shadowed);
        }
    }
}

#[cfg(test)]
mod serde {
    use std::path::Path;

    use dsc_lib::plugins::dscplugin::DscPlugin;
    use serde_json::json;

    use super::super::helpers::{manifest, manifest_json};

    #[test]
    fn serializing_uses_the_documented_property_names() {
        let path = Path::new("plugins").join("graph.dsc.plugin.json");
        let plugin = DscPlugin::from_manifest(&path, manifest("Test/Graph", "1.2.3", "graph"));

        pretty_assertions::assert_eq!(
            serde_json::to_value(&plugin).expect("serialization should never fail"),
            json!({
                "type": "Test/Graph",
                "version": "1.2.3",
                "command": "graph",
                "description": null,
                "deprecationMessage": null,
                "path": path,
                "directory": Path::new("plugins"),
                "shadowed": false,
                "manifest": manifest_json("Test/Graph", "1.2.3", "graph")
            })
        );
    }

    #[test]
    fn round_trips() {
        let plugin = DscPlugin::from_manifest(
            &Path::new("plugins").join("graph.dsc.plugin.json"),
            manifest("Test/Graph", "1.2.3", "graph")
        );

        let json = serde_json::to_string(&plugin).expect("serialization should never fail");
        let actual: DscPlugin = serde_json::from_str(&json).expect("plug-in should deserialize");

        pretty_assertions::assert_eq!(actual, plugin);
    }
}

crate::macros::test_dsc_repo_schema! {
    define_statics: {
        dsc_lib::plugins::dscplugin::DscPlugin,
        properties: {
            TYPE_SCHEMA => "type",
            VERSION_SCHEMA => "version",
            COMMAND_SCHEMA => "command",
            DESCRIPTION_SCHEMA => "description",
            DEPRECATION_MESSAGE_SCHEMA => "deprecationMessage",
            PATH_SCHEMA => "path",
            DIRECTORY_SCHEMA => "directory",
            SHADOWED_SCHEMA => "shadowed",
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
        command_title => (COMMAND_SCHEMA, "title"),
        command_description => (COMMAND_SCHEMA, "description"),
        command_markdown_description => (COMMAND_SCHEMA, "markdownDescription"),
        description_title => (DESCRIPTION_SCHEMA, "title"),
        description_description => (DESCRIPTION_SCHEMA, "description"),
        description_markdown_description => (DESCRIPTION_SCHEMA, "markdownDescription"),
        deprecation_message_title => (DEPRECATION_MESSAGE_SCHEMA, "title"),
        deprecation_message_description => (DEPRECATION_MESSAGE_SCHEMA, "description"),
        deprecation_message_markdown_description => (DEPRECATION_MESSAGE_SCHEMA, "markdownDescription"),
        path_title => (PATH_SCHEMA, "title"),
        path_description => (PATH_SCHEMA, "description"),
        path_markdown_description => (PATH_SCHEMA, "markdownDescription"),
        directory_title => (DIRECTORY_SCHEMA, "title"),
        directory_description => (DIRECTORY_SCHEMA, "description"),
        directory_markdown_description => (DIRECTORY_SCHEMA, "markdownDescription"),
        shadowed_title => (SHADOWED_SCHEMA, "title"),
        shadowed_description => (SHADOWED_SCHEMA, "description"),
        shadowed_markdown_description => (SHADOWED_SCHEMA, "markdownDescription")
    },
    test_validation: {
        plugin_with_every_property_is_valid => {
            input_json: json!({
                "type": "Test/Graph",
                "version": "1.2.3",
                "command": "graph",
                "description": "Show the dependency graph of a configuration document",
                "deprecationMessage": null,
                "path": "/opt/plugins/graph.dsc.plugin.json",
                "directory": "/opt/plugins",
                "shadowed": false,
                "manifest": {
                    "$schema": "https://aka.ms/dsc/schemas/v3/bundled/plugin/manifest.json",
                    "type": "Test/Graph",
                    "version": "1.2.3",
                    "command": { "name": "graph", "executable": "pwsh" }
                }
            }),
            expected_valid: true,
        },
        plugin_without_manifest_is_invalid => {
            input_json: json!({
                "type": "Test/Graph",
                "version": "1.2.3",
                "command": "graph",
                "description": null,
                "deprecationMessage": null,
                "path": "/opt/plugins/graph.dsc.plugin.json",
                "directory": "/opt/plugins",
                "shadowed": false
            }),
            expected_valid: false,
        },
        plugin_with_invalid_command_is_invalid => {
            input_json: json!({
                "type": "Test/Graph",
                "version": "1.2.3",
                "command": "Graph",
                "description": null,
                "deprecationMessage": null,
                "path": "/opt/plugins/graph.dsc.plugin.json",
                "directory": "/opt/plugins",
                "shadowed": false,
                "manifest": {
                    "$schema": "https://aka.ms/dsc/schemas/v3/bundled/plugin/manifest.json",
                    "type": "Test/Graph",
                    "version": "1.2.3",
                    "command": { "name": "graph", "executable": "pwsh" }
                }
            }),
            expected_valid: false,
        },
        empty_object_is_invalid => {
            input_json: json!({}),
            expected_valid: false,
        }
    }
}
