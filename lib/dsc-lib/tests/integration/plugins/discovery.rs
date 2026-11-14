// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

mod helpers {
    //! Defines helper functions for writing plug-in manifests to temporary directories.

    use std::fs;
    use std::path::{Path, PathBuf};

    use dsc_lib::plugins::discovery::discover_plugins_in;
    use dsc_lib::plugins::dscplugin::DscPlugin;
    use serde_json::{json, Value};
    use tempfile::TempDir;

    pub(super) use super::super::helpers::manifest_json;

    /// The commands the tests treat as built into the host.
    pub(super) const RESERVED: [&str; 3] = ["config", "resource", "help"];

    /// Creates a temporary directory to search for plug-ins.
    pub(super) fn directory() -> TempDir {
        tempfile::tempdir().expect("temporary directory should be created")
    }

    /// Writes a JSON value to a file in the directory and returns the path of the file.
    pub(super) fn write_json(directory: &Path, file_name: &str, value: &Value) -> PathBuf {
        let path = directory.join(file_name);
        fs::write(&path, serde_json::to_string_pretty(value).expect("value should serialize"))
            .expect("file should be written");
        path
    }

    /// Writes text to a file in the directory and returns the path of the file.
    pub(super) fn write_text(directory: &Path, file_name: &str, text: &str) -> PathBuf {
        let path = directory.join(file_name);
        fs::write(&path, text).expect("file should be written");
        path
    }

    /// Writes a manifest with only the required properties as `<command>.dsc.plugin.json`.
    pub(super) fn write_manifest(directory: &Path, type_name: &str, version: &str, command: &str) -> PathBuf {
        let file_name = format!("{}.{command}.dsc.plugin.json", type_name.replace('/', "."));
        write_json(directory, &file_name, &manifest_json(type_name, version, command))
    }

    /// Returns a manifest with a condition.
    pub(super) fn manifest_with_condition(type_name: &str, command: &str, condition: &str) -> Value {
        let mut manifest = manifest_json(type_name, "0.1.0", command);
        manifest["condition"] = json!(condition);
        manifest
    }

    /// Discovers the plug-ins in the directories with the reserved commands of the tests.
    pub(super) fn discover(directories: &[&Path]) -> Vec<DscPlugin> {
        let directories: Vec<PathBuf> = directories.iter().map(|directory| directory.to_path_buf()).collect();
        discover_plugins_in(&directories, &RESERVED)
    }

    /// Returns the type, version, command, and shadowed state of each plug-in, for comparisons.
    pub(super) fn summarize(plugins: &[DscPlugin]) -> Vec<(String, String, String, bool)> {
        plugins.iter().map(|plugin| (
            plugin.type_name.to_string(),
            plugin.version.to_string(),
            plugin.command.to_string(),
            plugin.shadowed,
        )).collect()
    }

    /// Returns the expected summary entry for a plug-in.
    pub(super) fn entry(type_name: &str, version: &str, command: &str, shadowed: bool) -> (String, String, String, bool) {
        (type_name.to_string(), version.to_string(), command.to_string(), shadowed)
    }
}

#[cfg(test)]
mod discover_plugins_in {
    use std::fs;

    use serde_json::json;
    use test_case::test_case;

    use super::helpers::*;

    #[test]
    fn empty_directory_has_no_plugins() {
        let directory = directory();

        assert!(discover(&[directory.path()]).is_empty());
    }

    #[test]
    fn missing_directory_has_no_plugins() {
        let directory = directory();
        let missing = directory.path().join("missing");

        assert!(discover(&[&missing]).is_empty());
    }

    #[test_case("graph.dsc.plugin.json"; "json manifest")]
    #[test_case("Graph.DSC.Plugin.JSON"; "file name suffix in another casing")]
    fn finds_json_manifest(file_name: &str) {
        let directory = directory();
        let path = write_json(directory.path(), file_name, &manifest_json("Test/Graph", "0.1.0", "graph"));

        let plugins = discover(&[directory.path()]);

        pretty_assertions::assert_eq!(summarize(&plugins), vec![entry("Test/Graph", "0.1.0", "graph", false)]);
        pretty_assertions::assert_eq!(plugins[0].path, path);
        pretty_assertions::assert_eq!(plugins[0].directory, directory.path());
    }

    #[test_case("graph.dsc.plugin.yaml"; "yaml extension")]
    #[test_case("graph.dsc.plugin.yml"; "yml extension")]
    fn finds_yaml_manifest(file_name: &str) {
        let directory = directory();
        write_text(directory.path(), file_name, concat!(
            "$schema: https://aka.ms/dsc/schemas/v3/bundled/plugin/manifest.json\n",
            "type: Test/Graph\n",
            "version: 0.1.0\n",
            "command:\n",
            "  name: graph\n",
            "  executable: pwsh\n",
            "  args:\n",
            "    - -File\n",
            "    - manifestRelativePath: graph.ps1\n",
        ));

        let plugins = discover(&[directory.path()]);

        pretty_assertions::assert_eq!(summarize(&plugins), vec![entry("Test/Graph", "0.1.0", "graph", false)]);
    }

    #[test]
    fn ignores_other_files_and_subdirectories() {
        let directory = directory();
        write_json(directory.path(), "graph.json", &manifest_json("Test/Json", "0.1.0", "json"));
        write_json(directory.path(), "graph.dsc.extension.json", &manifest_json("Test/Extension", "0.1.0", "extension"));
        write_json(directory.path(), "graph.dsc.plugin.json.bak", &manifest_json("Test/Backup", "0.1.0", "backup"));
        let subdirectory = directory.path().join("nested");
        fs::create_dir(&subdirectory).expect("subdirectory should be created");
        write_manifest(&subdirectory, "Test/Nested", "0.1.0", "nested");

        assert!(discover(&[directory.path()]).is_empty());
    }

    #[test]
    fn finds_plugins_in_manifest_list() {
        let directory = directory();
        let path = write_json(directory.path(), "tools.dsc.manifests.json", &json!({
            "plugins": [
                manifest_json("Test/Graph", "0.1.0", "graph"),
                manifest_json("Test/Explain", "0.2.0", "explain"),
            ]
        }));

        let plugins = discover(&[directory.path()]);

        pretty_assertions::assert_eq!(
            summarize(&plugins),
            vec![
                entry("Test/Explain", "0.2.0", "explain", false),
                entry("Test/Graph", "0.1.0", "graph", false),
            ]
        );
        pretty_assertions::assert_eq!(plugins[0].path, path);
    }

    #[test]
    fn finds_plugins_in_yaml_manifest_list() {
        let directory = directory();
        write_text(directory.path(), "tools.dsc.manifests.yaml", concat!(
            "plugins:\n",
            "  - $schema: https://aka.ms/dsc/schemas/v3/bundled/plugin/manifest.json\n",
            "    type: Test/Graph\n",
            "    version: 0.1.0\n",
            "    command:\n",
            "      name: graph\n",
            "      executable: pwsh\n",
        ));

        pretty_assertions::assert_eq!(
            summarize(&discover(&[directory.path()])),
            vec![entry("Test/Graph", "0.1.0", "graph", false)]
        );
    }

    #[test]
    fn manifest_list_with_invalid_resource_still_yields_plugins() {
        let directory = directory();
        write_json(directory.path(), "tools.dsc.manifests.json", &json!({
            "resources": [{ "type": "not a resource manifest" }],
            "plugins": [manifest_json("Test/Graph", "0.1.0", "graph")]
        }));

        pretty_assertions::assert_eq!(
            summarize(&discover(&[directory.path()])),
            vec![entry("Test/Graph", "0.1.0", "graph", false)]
        );
    }

    #[test_case(json!({ "resources": [] }); "list without plugins")]
    #[test_case(json!({ "plugins": null }); "plugins defined as null")]
    #[test_case(json!({ "plugins": {} }); "plugins defined as an object")]
    #[test_case(json!({ "plugins": [] }); "plugins defined as an empty array")]
    fn manifest_list_without_plugins_has_no_plugins(list: serde_json::Value) {
        let directory = directory();
        write_json(directory.path(), "tools.dsc.manifests.json", &list);

        assert!(discover(&[directory.path()]).is_empty());
    }

    #[test]
    fn skips_invalid_entry_in_manifest_list() {
        let directory = directory();
        write_json(directory.path(), "tools.dsc.manifests.json", &json!({
            "plugins": [
                { "type": "Test/Broken" },
                manifest_json("Test/Graph", "0.1.0", "graph"),
            ]
        }));

        pretty_assertions::assert_eq!(
            summarize(&discover(&[directory.path()])),
            vec![entry("Test/Graph", "0.1.0", "graph", false)]
        );
    }

    #[test]
    fn skips_manifest_that_is_not_valid_json() {
        let directory = directory();
        write_text(directory.path(), "broken.dsc.plugin.json", "{ not json");
        write_manifest(directory.path(), "Test/Graph", "0.1.0", "graph");

        pretty_assertions::assert_eq!(
            summarize(&discover(&[directory.path()])),
            vec![entry("Test/Graph", "0.1.0", "graph", false)]
        );
    }

    #[test]
    fn skips_manifest_with_invalid_command_name() {
        let directory = directory();
        write_json(directory.path(), "broken.dsc.plugin.json", &manifest_json("Test/Broken", "0.1.0", "Not-Valid"));

        assert!(discover(&[directory.path()]).is_empty());
    }

    #[test]
    fn skips_manifest_with_unknown_property() {
        let directory = directory();
        let mut manifest = manifest_json("Test/Broken", "0.1.0", "broken");
        manifest["discover"] = json!({ "executable": "pwsh" });
        write_json(directory.path(), "broken.dsc.plugin.json", &manifest);

        assert!(discover(&[directory.path()]).is_empty());
    }

    #[test]
    fn skips_manifest_with_unrecognized_schema_uri() {
        let directory = directory();
        let mut manifest = manifest_json("Test/Broken", "0.1.0", "broken");
        manifest["$schema"] = json!("https://example.com/not-a-dsc-schema.json");
        write_json(directory.path(), "broken.dsc.plugin.json", &manifest);

        assert!(discover(&[directory.path()]).is_empty());
    }

    #[test]
    fn skips_manifest_with_path_that_is_not_relative() {
        let directory = directory();
        let mut manifest = manifest_json("Test/Broken", "0.1.0", "broken");
        manifest["command"]["args"] = json!([{ "manifestRelativePath": "/tools/broken.ps1" }]);
        write_json(directory.path(), "broken.dsc.plugin.json", &manifest);

        assert!(discover(&[directory.path()]).is_empty());
    }

    #[test]
    fn keeps_manifest_whose_condition_is_true() {
        let directory = directory();
        write_json(
            directory.path(),
            "graph.dsc.plugin.json",
            &manifest_with_condition("Test/Graph", "graph", "[equals(1, 1)]")
        );

        pretty_assertions::assert_eq!(
            summarize(&discover(&[directory.path()])),
            vec![entry("Test/Graph", "0.1.0", "graph", false)]
        );
    }

    #[test]
    fn skips_manifest_whose_condition_is_false() {
        let directory = directory();
        write_json(
            directory.path(),
            "graph.dsc.plugin.json",
            &manifest_with_condition("Test/Graph", "graph", "[equals(1, 2)]")
        );

        assert!(discover(&[directory.path()]).is_empty());
    }

    #[test]
    fn skips_manifest_whose_condition_is_not_a_boolean() {
        let directory = directory();
        write_json(
            directory.path(),
            "graph.dsc.plugin.json",
            &manifest_with_condition("Test/Graph", "graph", "[concat('a', 'b')]")
        );

        assert!(discover(&[directory.path()]).is_empty());
    }

    #[test_case("config"; "name of a built-in command")]
    #[test_case("help"; "help command")]
    fn skips_manifest_with_reserved_command(command: &str) {
        let directory = directory();
        write_manifest(directory.path(), "Test/Reserved", "0.1.0", command);
        write_manifest(directory.path(), "Test/Graph", "0.1.0", "graph");

        pretty_assertions::assert_eq!(
            summarize(&discover(&[directory.path()])),
            vec![entry("Test/Graph", "0.1.0", "graph", false)]
        );
    }

    #[test]
    fn reserved_command_is_compared_case_sensitively() {
        let directory = directory();
        write_manifest(directory.path(), "Test/Graph", "0.1.0", "graph");

        let plugins = dsc_lib::plugins::discovery::discover_plugins_in(
            &[directory.path().to_path_buf()],
            &["GRAPH"]
        );

        pretty_assertions::assert_eq!(summarize(&plugins), vec![entry("Test/Graph", "0.1.0", "graph", false)]);
    }

    #[test]
    fn keeps_highest_version_of_a_type_found_in_later_directory() {
        let first = directory();
        let second = directory();
        write_manifest(first.path(), "Test/Graph", "1.0.0", "graph");
        let newer = write_manifest(second.path(), "Test/Graph", "1.1.0", "graph");

        let plugins = discover(&[first.path(), second.path()]);

        pretty_assertions::assert_eq!(summarize(&plugins), vec![entry("Test/Graph", "1.1.0", "graph", false)]);
        pretty_assertions::assert_eq!(plugins[0].path, newer);
    }

    #[test]
    fn keeps_highest_version_of_a_type_found_in_earlier_directory() {
        let first = directory();
        let second = directory();
        let newer = write_manifest(first.path(), "Test/Graph", "1.1.0", "graph");
        write_manifest(second.path(), "Test/Graph", "1.0.0", "graph");

        let plugins = discover(&[first.path(), second.path()]);

        pretty_assertions::assert_eq!(summarize(&plugins), vec![entry("Test/Graph", "1.1.0", "graph", false)]);
        pretty_assertions::assert_eq!(plugins[0].path, newer);
    }

    #[test]
    fn keeps_highest_version_of_a_type_within_one_directory() {
        let directory = directory();
        write_json(directory.path(), "a.dsc.plugin.json", &manifest_json("Test/Graph", "1.0.0", "graph"));
        write_json(directory.path(), "b.dsc.plugin.json", &manifest_json("Test/Graph", "2.0.0", "graph"));
        write_json(directory.path(), "c.dsc.plugin.json", &manifest_json("Test/Graph", "1.5.0", "graph"));

        pretty_assertions::assert_eq!(
            summarize(&discover(&[directory.path()])),
            vec![entry("Test/Graph", "2.0.0", "graph", false)]
        );
    }

    #[test]
    fn type_names_are_compared_case_insensitively() {
        let first = directory();
        let second = directory();
        write_manifest(first.path(), "Test/Graph", "1.0.0", "graph");
        write_manifest(second.path(), "test/graph", "1.1.0", "graph");

        let plugins = discover(&[first.path(), second.path()]);

        pretty_assertions::assert_eq!(plugins.len(), 1);
        pretty_assertions::assert_eq!(plugins[0].version.to_string(), "1.1.0");
    }

    #[test]
    fn first_directory_provides_a_command_claimed_by_different_types() {
        let first = directory();
        let second = directory();
        write_manifest(first.path(), "Zeta/Graph", "0.1.0", "graph");
        write_manifest(second.path(), "Alpha/Graph", "9.0.0", "graph");

        pretty_assertions::assert_eq!(
            summarize(&discover(&[first.path(), second.path()])),
            vec![
                entry("Zeta/Graph", "0.1.0", "graph", false),
                entry("Alpha/Graph", "9.0.0", "graph", true),
            ]
        );
    }

    #[test]
    fn type_that_sorts_first_provides_a_command_within_one_directory() {
        let directory = directory();
        write_json(directory.path(), "a.dsc.plugin.json", &manifest_json("Zeta/Graph", "0.1.0", "graph"));
        write_json(directory.path(), "b.dsc.plugin.json", &manifest_json("Alpha/Graph", "0.1.0", "graph"));

        pretty_assertions::assert_eq!(
            summarize(&discover(&[directory.path()])),
            vec![
                entry("Alpha/Graph", "0.1.0", "graph", false),
                entry("Zeta/Graph", "0.1.0", "graph", true),
            ]
        );
    }

    #[test]
    fn returns_plugins_in_search_order() {
        let first = directory();
        let second = directory();
        write_manifest(first.path(), "Test/Lint", "0.1.0", "lint");
        write_manifest(first.path(), "Test/Graph", "0.1.0", "graph");
        write_manifest(second.path(), "Test/Explain", "0.1.0", "explain");

        pretty_assertions::assert_eq!(
            summarize(&discover(&[first.path(), second.path()])),
            vec![
                entry("Test/Graph", "0.1.0", "graph", false),
                entry("Test/Lint", "0.1.0", "lint", false),
                entry("Test/Explain", "0.1.0", "explain", false),
            ]
        );
    }
}

#[cfg(test)]
mod find_plugin_by_type {
    use dsc_lib::plugins::discovery::find_plugin_by_type;
    use test_case::test_case;

    use super::helpers::*;

    #[test_case("Alpha/Graph"; "type name as defined")]
    #[test_case("alpha/GRAPH"; "type name in another casing")]
    fn returns_a_shadowed_plugin(type_name: &str) {
        let first = directory();
        let second = directory();
        write_manifest(first.path(), "Zeta/Graph", "0.1.0", "graph");
        write_manifest(second.path(), "Alpha/Graph", "0.1.0", "graph");
        let plugins = discover(&[first.path(), second.path()]);

        let plugin = find_plugin_by_type(&plugins, type_name).expect("plug-in should be found");

        pretty_assertions::assert_eq!(plugin.type_name.to_string(), "Alpha/Graph");
        assert!(plugin.shadowed);
    }

    #[test]
    fn returns_the_plugin_that_provides_its_command() {
        let directory = directory();
        write_manifest(directory.path(), "Test/Graph", "0.1.0", "graph");
        let plugins = discover(&[directory.path()]);

        let plugin = find_plugin_by_type(&plugins, "Test/Graph").expect("plug-in should be found");

        assert!(!plugin.shadowed);
    }

    #[test_case("Test/Lint"; "type that is not installed")]
    #[test_case("graph"; "command name")]
    fn returns_none_for_unknown_type(type_name: &str) {
        let directory = directory();
        write_manifest(directory.path(), "Test/Graph", "0.1.0", "graph");
        let plugins = discover(&[directory.path()]);

        assert!(find_plugin_by_type(&plugins, type_name).is_none());
    }
}

#[cfg(test)]
mod find_plugin {
    use dsc_lib::plugins::discovery::find_plugin;

    use super::helpers::*;

    #[test]
    fn returns_the_plugin_that_provides_the_command() {
        let directory = directory();
        write_manifest(directory.path(), "Test/Graph", "0.1.0", "graph");
        write_manifest(directory.path(), "Test/Lint", "0.1.0", "lint");
        let plugins = discover(&[directory.path()]);

        let plugin = find_plugin(&plugins, "lint").expect("plug-in should be found");

        pretty_assertions::assert_eq!(plugin.type_name.to_string(), "Test/Lint");
    }

    #[test]
    fn never_returns_a_shadowed_plugin() {
        let first = directory();
        let second = directory();
        write_manifest(first.path(), "Zeta/Graph", "0.1.0", "graph");
        write_manifest(second.path(), "Alpha/Graph", "0.1.0", "graph");
        let plugins = discover(&[first.path(), second.path()]);

        let plugin = find_plugin(&plugins, "graph").expect("plug-in should be found");

        pretty_assertions::assert_eq!(plugin.type_name.to_string(), "Zeta/Graph");
        assert!(!plugin.shadowed);
    }

    #[test]
    fn returns_none_for_unknown_command() {
        let directory = directory();
        write_manifest(directory.path(), "Test/Graph", "0.1.0", "graph");
        let plugins = discover(&[directory.path()]);

        assert!(find_plugin(&plugins, "lint").is_none());
    }

    #[test]
    fn compares_the_command_case_sensitively() {
        let directory = directory();
        write_manifest(directory.path(), "Test/Graph", "0.1.0", "graph");
        let plugins = discover(&[directory.path()]);

        assert!(find_plugin(&plugins, "Graph").is_none());
    }
}
