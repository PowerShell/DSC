// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use std::{collections::HashMap, ops::{Deref, DerefMut}, sync::LazyLock};

use rust_i18n::t;
use schemars::{JsonSchema, json_schema};
use serde::{Deserialize, Serialize};

use crate::{schemas::dsc_repo::DscRepoSchema, types::ExitCode};

/// Defines a map of exit codes to their semantic meaning for operation commands.
///
/// DSC resources and extensions may define any number of operation commands like `get` or
/// `secret`. DSC always considers commands that exit with code `0` to be successful operations and
/// commands that exit with any nonzero code to have failed.
///
/// Resource and extension authors can provide more useful information to users by defining an
/// [`ExitCodesMap`]. When a resource or extension defines the `exitCodes` field in its manifest,
/// DSC surfaces the associated string as part of the error message.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, DscRepoSchema)]
#[dsc_repo_schema(base_name = "exitCodes", folder_path = "definitions")]
pub struct ExitCodesMap(HashMap<ExitCode, String>);

/// Defines the default map as a private static to use with the `get_code_or_default` method,
/// minimizing the performance hit compared to reconstructing the default map on every method
/// invocation.
static DEFAULT_MAP: LazyLock<ExitCodesMap> = LazyLock::new(|| ExitCodesMap::default());

impl ExitCodesMap {
    /// Defines the regular expression for validating a string as an exit code.
    ///
    /// The string must consist only of ASCII digits (`[0-9]`) with an optional leading hyphen
    /// (`-`). If the string can't be parsed as an [`i32`], the value is invalid.
    ///
    /// This value is only used in the JSON Schema for validating the property names for the map
    /// of exit codes to their descriptions. For JSON and YAML, DSC expects the keys to always be
    /// strings but they _must_ map to 32-bit integers.
    pub const KEY_VALIDATING_PATTERN: &str = r"^-?[0-9]+$";

    /// Creates a new instance of [`ExitCodesMap`] with the default capacity.
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    /// Creates a new instance of [`ExitCodesMap`] with the given capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self(HashMap::with_capacity(capacity))
    }

    /// Looks up an [`ExitCode`] in the map and returns a reference to its description, if the map
    /// contains the given exit code.
    pub fn get_code(&self, code: i32) -> Option<&String> {
        self.0.get(&ExitCode::new(code))
    }

    /// Looks up an [`ExitCode`] in the map and returns its description, if the map contains the
    /// given exit code, or the default description.
    ///
    /// The default description is retrieved from the `default()` map:
    ///
    /// - Exit code `0` returns the description for `0` in the default map.
    /// - All other exit codes return the description for `1` in the default map.
    pub fn get_code_or_default(&self, code: i32) -> String {
        match self.0.get(&ExitCode::new(code)) {
            Some(description) => description.clone(),
            None => match code {
                0 => (&*DEFAULT_MAP).get_code(0).expect("default always defines exit code 0").clone(),
                _ => (&*DEFAULT_MAP).get_code(1).expect("default always defines exit code 1").clone(),
            }
        }
    }

    /// Indicates whether the [`ExitCodesMap`] is identical to the default map.
    pub fn is_default(&self) -> bool {
        self == &*DEFAULT_MAP
    }

    /// Indicates whether the [`ExitCodesMap`] is empty or identical to the default map.
    ///
    /// Use this method with the `skip_serializing_if` attribute for serde to avoid serializing
    /// empty and default maps.
    pub fn is_empty_or_default(&self) -> bool {
        self.is_empty() || self.is_default()
    }
}

impl Default for ExitCodesMap {
    fn default() -> Self {
        let mut map: HashMap<ExitCode, String> = HashMap::with_capacity(2);
        map.insert(ExitCode::new(0), t!("types.exit_codes_map.successText").into());
        map.insert(ExitCode::new(1), t!("types.exit_codes_map.failureText").into());

        Self(map)
    }
}

impl JsonSchema for ExitCodesMap {
    fn schema_name() -> std::borrow::Cow<'static, str> {
        Self::default_schema_id_uri().into()
    }

    fn json_schema(_: &mut schemars::SchemaGenerator) -> schemars::Schema {
        json_schema!({
            "$schema": "https://json-schema.org/draft/2020-12/schema",
            "title": t!("schemas.definitions.exitCodes.title"),
            "description": t!("schemas.definitions.exitCodes.description"),
            "markdownDescription": t!("schemas.definitions.exitCodes.markdownDescription"),
            "type": "object",
            "minProperties": 1,
            "propertyNames": {
                "pattern": Self::KEY_VALIDATING_PATTERN,
                "patternErrorMessage": t!("schemas.definitions.exitCodes.invalidKeyErrorMessage")
            },
            "patternProperties": {
                Self::KEY_VALIDATING_PATTERN: {
                    "type": "string"
                }
            },
            "unevaluatedProperties": false,
            "default": Self::default(),
            "examples": [{
                "0": "Success",
                "1": "Invalid parameter",
                "2": "Invalid input",
                "3": "Registry error",
                "4": "JSON serialization failed"
            }]
        })
    }
}

impl AsRef<ExitCodesMap> for ExitCodesMap {
    fn as_ref(&self) -> &ExitCodesMap {
        &self
    }
}

impl Deref for ExitCodesMap {
    type Target = HashMap<ExitCode, String>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ExitCodesMap {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
