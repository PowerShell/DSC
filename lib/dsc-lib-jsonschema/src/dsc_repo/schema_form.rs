// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

/// Defines the different forms of JSON Schema that DSC publishes.
#[derive(Debug, Default, Clone, Copy, Hash, Eq, PartialEq)]
pub enum SchemaForm {
    /// Indicates that the schema is bundled using the 2020-12 schema bundling contract.
    ///
    /// These schemas include all of their references in the `$defs` keyword where the key for
    /// each reference is the `$id` of that subschema and the value is the subschema.
    ///
    /// The bundled schemas are preferred for offline usage or where network latency is a concern.
    #[default]
    Bundled,
    /// Indicates that the schema is enhanced for interactively viewing, authoring, and editing
    /// the data in VS Code.
    ///
    /// These schemas include keywords not recognized by JSON Schema libraries and clients outside
    /// of VS Code, like `markdownDescription` and `defaultSnippets`. The schema references and
    /// definitions do not follow the canonical bundling for schema 2020-12, as the VS Code
    /// JSON language server doesn't correctly resolve canonically bundled schemas.
    VSCode,
    /// Indicates that the schema is canonical but not bundled. It may contain references to other
    /// JSON Schemas that require resolution by retrieving those schemas over the network. All
    /// DSC schemas are published in this form for easier review, reuse, and retrieval.
    Canonical,
}

impl SchemaForm {
    /// Returns the file extension for a given form of schema.
    ///
    /// The extension for [`Bundled`] and [`Canonical`] schemas is `.json`
    ///
    /// The extension for [`VSCode`] schemas is `.vscode.json`
    ///
    /// [`Bundled`]: SchemaForm::Bundled
    /// [`Canonical`]: SchemaForm::Canonical
    /// [`VSCode`]: SchemaForm::VSCode
    #[must_use]
    pub fn to_extension(&self) -> String {
        match self {
            Self::Bundled | Self::Canonical => ".json".to_string(),
            Self::VSCode => ".vscode.json".to_string(),
        }
    }

    /// Return the prefix for a schema's folder path.
    ///
    /// The [`Bundled`] and [`VSCode`] schemas are always published in the `bundled` folder
    /// immediately beneath the version folder. The [`Canonical`] schemas use the folder path
    /// as defined for that schema.
    ///
    /// [`Bundled`]: SchemaForm::Bundled
    /// [`Canonical`]: SchemaForm::Canonical
    /// [`VSCode`]: SchemaForm::VSCode
    #[must_use]
    pub fn to_folder_prefix(&self) -> String {
        match self {
            Self::Bundled | Self::VSCode => "bundled/".to_string(),
            Self::Canonical => String::new(),
        }
    }

    /// Returns every schema form for convenient iteration.
    #[must_use]
    pub fn all() -> Vec<SchemaForm> {
        vec![
            Self::Bundled,
            Self::VSCode,
            Self::Canonical,
        ]
    }
}
