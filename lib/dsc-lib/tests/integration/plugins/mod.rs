// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! Defines integration tests for the public types in the `plugins` module of [`dsc-lib`].

#[cfg(test)]
mod command_name;
#[cfg(test)]
mod discovery;
#[cfg(test)]
mod dscplugin;
#[cfg(test)]
mod plugin_manifest;

pub(crate) mod helpers {
    //! Defines helper functions for creating plug-in manifests.

    use dsc_lib::plugins::plugin_manifest::PluginManifest;
    use serde_json::{json, Value};

    /// The default URI for the `$schema` property of a plug-in manifest.
    pub(crate) const MANIFEST_SCHEMA_URI: &str = "https://aka.ms/dsc/schemas/v3/bundled/plugin/manifest.json";

    /// Returns the JSON for a manifest with only the required properties.
    pub(crate) fn manifest_json(type_name: &str, version: &str, command: &str) -> Value {
        json!({
            "$schema": MANIFEST_SCHEMA_URI,
            "type": type_name,
            "version": version,
            "command": {
                "name": command,
                "executable": "plugin-executable"
            }
        })
    }

    /// Returns a manifest with only the required properties.
    pub(crate) fn manifest(type_name: &str, version: &str, command: &str) -> PluginManifest {
        serde_json::from_value(manifest_json(type_name, version, command))
            .expect("manifest should be valid")
    }
}
