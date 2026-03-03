// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! This build script generates code for the `RecognizedSchemaVersion` enum. It uses PowerShell to
//! query for the latest tags and generates data in the `.versions.json` file. The build script uses
//! that data to create the enum variants that indicate which versions of DSC a schema is recognized
//! for.
//!
//! Generating this code enables us to use the enum without having to manually update the definition
//! for every release.

use std::env;
use std::fs;
use std::fs::read_to_string;
use std::path::Path;
use std::process::Command;
use std::process::Output;

use serde::Deserialize;

/// Representation of the data in `.versions.json` file.
#[derive(Debug, Clone, Hash, Eq, PartialEq, PartialOrd, Deserialize)]
#[serde(rename_all="camelCase")]
struct VersionInfo {
    pub all: Vec<String>,
    pub latest_major: String,
    pub latest_minor: String,
    pub latest_patch: String,
}

/// Constructs the enum type definition. It emits a string like:
///
/// ```rust
/// #[derive(Debug, Default, Clone, Copy, Hash, Eq, PartialEq)]
/// pub enum RecognizedSchemaVersion {
///     /// Represents the `vNext` schema folder.
///     VNext,
///     /// Represents the `v3` schema folder
///     #[default]
///     V3,
///     /// Represents the `v3.1` schema folder
///     V3_1,
///     /// Represents the `v3.1.2` schema folder
///     V3_1_2,
///     /// Represents the `v3.1.1` schema folder
///     V3_1_1,
///     /// Represents the `v3.1.0` schema folder
///     V3_1_0,
///     /// Represents the `v3.0` schema folder
///     V3_0,
///     /// Represents the `v3.0.2` schema folder
///     V3_0_2,
///     /// Represents the `v3.0.1` schema folder
///     V3_0_1,
///     /// Represents the `v3.0.0` schema folder
///     V3_0_0,
/// }
/// ```
fn format_type_definition(version_info: &VersionInfo) -> String {
    let mut lines = vec![
        "#[derive(Debug, Default, Clone, Copy, Hash, Eq, PartialEq)]".to_string(),
        "pub enum RecognizedSchemaVersion {".to_string(),
        "    /// Represents the `vNext` schema folder.".to_string(),
        "    VNext,".to_string(),
    ];

    for (index, version) in version_info.all.iter().enumerate() {
        let comment = format!(
            "    /// Represents the `{}` schema folder",
            version.replace('_', ".").to_lowercase()
        );
        lines.push(comment);

        if index == 0 {
            lines.push("    #[default]".to_string());
        }

        lines.push(format!("    {version},"));
    }

    lines.push("}".to_string());

    lines.join("\n")
}

/// Constructs the implementation for the [`std::fmt::Display`] trait. It emits a string like:
///
/// ```rust
/// impl std::fmt::Display for RecognizedSchemaVersion {
///     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
///         match self {
///             Self::VNext => write!(f, "vNext"),
///             Self::V3 => write!(f, "v3"),
///             Self::V3_1 => write!(f, "v3.1"),
///             Self::V3_1_2 => write!(f, "v3.1.2"),
///             Self::V3_1_1 => write!(f, "v3.1.1"),
///             Self::V3_1_0 => write!(f, "v3.1.0"),
///             Self::V3_0 => write!(f, "v3.0"),
///             Self::V3_0_2 => write!(f, "v3.0.2"),
///             Self::V3_0_1 => write!(f, "v3.0.1"),
///             Self::V3_0_0 => write!(f, "v3.0.0"),
///         }
///     }
/// }
/// ```
fn format_display_trait_impl(version_info: &VersionInfo) -> String {
    let mut lines = vec![
        "impl std::fmt::Display for RecognizedSchemaVersion {".to_string(),
        "    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {".to_string(),
        "        match self {".to_string(),
        "            Self::VNext => write!(f, \"vNext\"),".to_string(),
    ];

    for version in version_info.all.clone() {
        lines.push(format!(
            "            Self::{version} => write!(f, \"{}\"),",
            version.replace('_', ".").to_lowercase()
        ));
    }
    lines.push("        }".to_string());
    lines.push("    }".to_string());
    lines.push("}".to_string());

    lines.join("\n")
}

/// Emits the definition for the `all()` method. Called by the [`format_method_impl`] formatter.
///
/// Emits a string (indented 4 spaces, because it's placed in an `impl` block) like:
///
/// ```rust
///     /// Returns every recognized schema version for convenient iteration.
///     #[must_use]
///     pub fn all() -> Vec<RecognizedSchemaVersion> {
///         vec![
///             Self::VNext,
///             Self::V3,
///             Self::V3_1,
///             Self::V3_1_2,
///             Self::V3_1_1,
///             Self::V3_1_0,
///             Self::V3_0,
///             Self::V3_0_2,
///             Self::V3_0_1,
///             Self::V3_0_0,
///         ]
///     }
/// ```
fn format_method_all(version_info: &VersionInfo) -> String {
    let mut lines = vec![
        "    /// Returns every recognized schema version for convenient iteration.".to_string(),
        "    #[must_use]".to_string(),
        "    pub fn all() -> Vec<RecognizedSchemaVersion> {".to_string(),
        "        vec![".to_string(),
        "            Self::VNext,".to_string(),
    ];

    for version in version_info.all.clone() {
        lines.push(format!("            Self::{version},"));
    }
    lines.push("        ]".to_string());
    lines.push("    }".to_string());

    lines.join("\n")
}

/// Emits the definition for the `all_major()` method. Called by the [`format_method_impl`] formatter.
///
/// Emits a string (indented 4 spaces, because it's placed in an `impl` block) like:
///
/// ```rust
///     /// Returns every recognized major version, like `v3`.
///     #[must_use]
///     pub fn all_major() -> Vec<RecognizedSchemaVersion> {
///         vec![
///             Self::V3,
///         ]
///     }
/// ```
fn format_method_all_major(version_info: &VersionInfo) -> String {
    let mut lines = vec![
        "    /// Returns every recognized major version, like `v3`.".to_string(),
        "    #[must_use]".to_string(),
        "    pub fn all_major() -> Vec<RecognizedSchemaVersion> {".to_string(),
        "        vec![".to_string(),
    ];

    for version in version_info.all.clone() {
        if version.split('_').collect::<Vec<_>>().len() == 1 {
            lines.push(format!("            Self::{version},"));
        }
    }

    lines.push("        ]".to_string());
    lines.push("    }".to_string());

    lines.join("\n")
}

/// Emits the definition for the `all_minor()` method. Called by the [`format_method_impl`] formatter.
///
/// Emits a string (indented 4 spaces, because it's placed in an `impl` block) like:
///
/// ```rust
///     /// Returns every recognized minor version, like `v3.1`.
///     #[must_use]
///     pub fn all_minor() -> Vec<RecognizedSchemaVersion> {
///         vec![
///             Self::V3_1,
///             Self::V3_0,
///         ]
///     }
/// ```
fn format_method_all_minor(version_info: &VersionInfo) -> String {
    let mut lines = vec![
        "    /// Returns every recognized minor version, like `v3.1`.".to_string(),
        "    #[must_use]".to_string(),
        "    pub fn all_minor() -> Vec<RecognizedSchemaVersion> {".to_string(),
        "        vec![".to_string(),
    ];

    for version in version_info.all.clone() {
        if version.split('_').collect::<Vec<_>>().len() == 2 {
            lines.push(format!("            Self::{version},"));
        }
    }

    lines.push("        ]".to_string());
    lines.push("    }".to_string());

    lines.join("\n")
}

/// Emits the definition for the `all_patch()` method. Called by the [`format_method_impl`] formatter.
///
/// Emits a string (indented 4 spaces, because it's placed in an `impl` block) like:
///
/// ```rust
///     /// Returns every recognized patch version, like `v3.1.1`.
///     #[must_use]
///     pub fn all_patch() -> Vec<RecognizedSchemaVersion> {
///         vec![
///             Self::V3_1_2,
///             Self::V3_1_1,
///             Self::V3_1_0,
///             Self::V3_0_2,
///             Self::V3_0_1,
///             Self::V3_0_0,
///         ]
///     }
/// ```
fn format_method_all_patch(version_info: &VersionInfo) -> String {
    let mut lines = vec![
        "    /// Returns every recognized patch version, like `v3.1.1`.".to_string(),
        "    #[must_use]".to_string(),
        "    pub fn all_patch() -> Vec<RecognizedSchemaVersion> {".to_string(),
        "        vec![".to_string(),
    ];

    for version in version_info.all.clone() {
        if version.split('_').collect::<Vec<_>>().len() == 3 {
            lines.push(format!("            Self::{version},"));
        }
    }

    lines.push("        ]".to_string());
    lines.push("    }".to_string());

    lines.join("\n")
}

/// Emits the definition for the `latest()` method. Called by the [`format_method_impl`] formatter.
///
/// Emits a string (indented 4 spaces, because it's placed in an `impl` block) like:
///
/// ```rust
///     /// Returns the latest version with major, minor, and patch segments, like `3.0.0`.
///     #[must_use]
///     pub fn latest() -> RecognizedSchemaVersion {
///         Self::V3_1_2
///     }
/// ```
fn format_method_latest(version_info: &VersionInfo) -> String {
    [
        "    /// Returns the latest version with major, minor, and patch segments, like `3.0.0`.".to_string(),
        "    #[must_use]".to_string(),
        "    pub fn latest() -> RecognizedSchemaVersion {".to_string(),
        format!("        Self::{}", version_info.latest_patch),
        "    }".to_string(),
    ].join("\n")
}

/// Emits the definition for the `latest_minor()` method. Called by the [`format_method_impl`] formatter.
///
/// Emits a string (indented 4 spaces, because it's placed in an `impl` block) like:
///
/// ```rust
///     /// Returns the latest minor version for the latest major version, like `3.0`.
///     #[must_use]
///     pub fn latest_minor() -> RecognizedSchemaVersion {
///         Self::V3_1
///     }
/// ```
fn format_method_latest_minor(version_info: &VersionInfo) -> String {
    [
        "    /// Returns the latest minor version for the latest major version, like `3.0`.".to_string(),
        "    #[must_use]".to_string(),
        "    pub fn latest_minor() -> RecognizedSchemaVersion {".to_string(),
        format!("        Self::{}", version_info.latest_minor),
        "    }".to_string(),
    ].join("\n")
}

/// Emits the definitions for the `latest_major()` method. Called by the [`format_method_impl`] formatter.
///
/// Emits a string (indented 4 spaces, because it's placed in an `impl` block) like:
///
/// ```rust
///     /// Returns the latest major version, like `3`
///     #[must_use]
///     pub fn latest_major() -> RecognizedSchemaVersion {
///         Self::V3
///     }
/// ```
fn format_method_latest_major(version_info: &VersionInfo) -> String {
    [
        "    /// Returns the latest major version, like `3`".to_string(),
        "    #[must_use]".to_string(),
        "    pub fn latest_major() -> RecognizedSchemaVersion {".to_string(),
        format!("        Self::{}", version_info.latest_major),
        "    }".to_string(),
    ].join("\n")
}

/// Emits the method implementation block for the enum type. Emits a string like:
///
/// ```rust
/// impl RecognizedSchemaVersion {
///     /// Returns every recognized schema version for convenient iteration.
///     #[must_use]
///     pub fn all() -> Vec<RecognizedSchemaVersion> {
///         vec![
///             Self::VNext,
///             Self::V3,
///             Self::V3_1,
///             Self::V3_1_2,
///             Self::V3_1_1,
///             Self::V3_1_0,
///             Self::V3_0,
///             Self::V3_0_2,
///             Self::V3_0_1,
///             Self::V3_0_0,
///         ]
///     }
///
///     /// Returns every recognized major version, like `v3`.
///     #[must_use]
///     pub fn all_major() -> Vec<RecognizedSchemaVersion> {
///         vec![
///             Self::V3,
///         ]
///     }
///
///     /// Returns every recognized minor version, like `v3.1`.
///     #[must_use]
///     pub fn all_minor() -> Vec<RecognizedSchemaVersion> {
///         vec![
///             Self::V3_1,
///             Self::V3_0,
///         ]
///     }
///
///     /// Returns every recognized patch version, like `v3.1.1`.
///     #[must_use]
///     pub fn all_patch() -> Vec<RecognizedSchemaVersion> {
///         vec![
///             Self::V3_1_2,
///             Self::V3_1_1,
///             Self::V3_1_0,
///             Self::V3_0_2,
///             Self::V3_0_1,
///             Self::V3_0_0,
///         ]
///     }
///
///     /// Returns the latest version with major, minor, and patch segments, like `3.0.0`.
///     #[must_use]
///     pub fn latest() -> RecognizedSchemaVersion {
///         Self::V3_1_2
///     }
///
///     /// Returns the latest minor version for the latest major version, like `3.0`.
///     #[must_use]
///     pub fn latest_minor() -> RecognizedSchemaVersion {
///         Self::V3_1
///     }
///
///     /// Returns the latest major version, like `3`
///     #[must_use]
///     pub fn latest_major() -> RecognizedSchemaVersion {
///         Self::V3
///     }
/// }
/// ```
fn format_method_impl(version_info: &VersionInfo) -> String {
    [
        "impl RecognizedSchemaVersion {".to_string(),
        format_method_all(version_info),
        String::new(),
        format_method_all_major(version_info),
        String::new(),
        format_method_all_minor(version_info),
        String::new(),
        format_method_all_patch(version_info),
        String::new(),
        format_method_latest(version_info),
        String::new(),
        format_method_latest_minor(version_info),
        String::new(),
        format_method_latest_major(version_info),
        "}".to_string(),
    ].join("\n")
}

/// Emits the documentation string for the enum type.
fn format_type_docs() -> String {
    [
        "/// Defines the versions of DSC recognized for schema validation and handling.",
        "///",
        "/// The DSC schemas are published into three folders:",
        "///",
        "/// - `v<major>.<minor>.<patch>` always includes the exact JSON Schema that shipped in that release",
        "///   of DSC.",
        "/// - `v<major>.<minor>` always includes the latest JSON Schema compatible with that minor version",
        "///   of DSC.",
        "/// - `v<major>` always includes the latest JSON Schema compatible with that major version of DSC.",
        "///",
        "/// Pinning to `v<major>` requires the least-frequent updating of the `$schema` in configuration",
        "/// documents and resource manifests, but also introduces changes that affect those schemas",
        "/// (without breaking changes) regularly. Some of the added features may not be effective in the",
        "/// version of DSC a user has installed.",
        "///",
        "/// Pinning to `v<major>.<minor>` ensures that users always have the latest schemas for the version",
        "/// of DSC they're using without schema changes that they may not be able to take advantage of.",
        "/// However, it requires updating the resource manifests and configuration documents with each",
        "/// minor release of DSC.",
        "///",
        "/// Pinning to `v<major>.<minor>.<patch>` is the most specific option, but requires the most",
        "/// frequent updating on the part of resource and configuration authors.",
        "///",
        "/// Additionally, we define the `vNext` folder, which always contains the latest schemas for DSC",
        "/// types, even when they haven't been released yet. You can use the `vNext` folder when working with",
        "/// prerelease versions and building from source between releases."
    ].join("\n")
}

/// Composes the contents for the type definition file using the other formatter functions.
fn format_file_content(version_info: &VersionInfo) -> String {
    [
        format_type_docs(),
        format_type_definition(version_info),
        String::new(),
        format_display_trait_impl(version_info),
        String::new(),
        format_method_impl(version_info),
        String::new(),
    ].join("\n")
}

/// Invokes the `.versions.ps1` PowerShell script to query git tags and update the `.versions.json`
/// file if needed.
fn update_versions_data(script_path: &Path) -> Output {
    Command::new("pwsh")
        .args([
            "-NoLogo",
            "-NonInteractive",
            "-NoProfile",
            "-ExecutionPolicy",
            "Bypass",
            "-File",
             &script_path.to_string_lossy(),
        ])
        .output()
        .unwrap_or_else(|e| {
            panic!(
                "failed to execute PowerShell script '{}': {}",
                script_path.display(),
                e
            )
        })
}

/// Constructs the `recognized_schema_version.rs` file in the output directory, which the library
/// uses as generated code for the `RecognizedSchemaVersion` enum.
fn main() {
    let project_dir = env::var_os("CARGO_MANIFEST_DIR")
        .expect("env var 'CARGO_MANIFEST_DIR' not defined");
    let data_path = Path::new(&project_dir).join(".versions.json");
    let script_path = Path::new(&project_dir).join(".versions.ps1");
    let out_dir = env::var_os("OUT_DIR")
        .expect("env var 'OUT_DIR' not defined");
    let dest_path = Path::new(&out_dir).join("recognized_schema_version.rs");

    let profile = env::var_os("PROFILE")
        .expect("env var 'PROFILE' not defined");

    // Update the versions data if needed, only on debug builds.
    if profile == "debug" {
        let output = update_versions_data(&script_path);
        assert!(
            output.status.success(),
            "Failed to update versions data via PowerShell script.\nExit code: {:?}\nStdout: {}\nStderr: {}",
            output.status.code(),
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let version_data = read_to_string(data_path)
        .expect("Failed to read .versions.json file");
    let version_info: VersionInfo = serde_json::from_str(&version_data)
        .expect(format!(
            "Failed to parse version data from .versions.json:\n---FILE TEXT START---\n{version_data}\n---FILE TEXT END---\n"
        ).as_str());
    let contents = format_file_content(&version_info);

    fs::write(
        &dest_path,
        contents
    ).expect("Failed to write recognized_schema_version.rs");

    println!("cargo::rerun-if-changed=build.rs");
    println!("cargo::rerun-if-changed=.versions.json");
    println!("cargo::rerun-if-changed=.versions.ps1");
}
