//! Defines the [`DscPolicyFileData`] struct, which represents the data structure of the policy
//! file for DSC settings.
//! 
//! Every field defined in [`DscSettingsResolved`] struct should also be defined in the
//! [`DscPolicyFileData`] struct. The only exception is the `ignore_settings_file` field, which is
//! superceded by the `forbid_ignore_settings_file` field in the policy file. The
//! `ignore_settings_file` field is only definable in the environment and CLI sources.
//! 
//! When adding a new top-level field to the policy file, follow these guidelines:
//! 
//! 1. Ensure that the field is defined in the [`fields`] module following that module guidance.
//! 1. Add the field to the [`DscSettingsResolved`] struct:
//! 
//!    - If the field is a container field, define the field in the struct as the appropriate
//!      `*PolicyFileData` or `*FileData` struct type.
//!    - If the field is a top-level leaf field, define the field in the struct as an
//!      [`Option<T>`] with the appropriate type.
//! 1. If the field is a top-level leaf field, define the field in the [`DscPolicyFileData`] struct
//!    as an [`Option<T>`] with the appropriate type.
//! 
//! No changes are required for the `from_file()` method, as it deserializes the field from the
//! policy file if it's defined.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::settings::{DscSettingsError, ResourcePathFileData, TracingFileData};

#[derive(Default, Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct DscPolicyFileData {
    pub forbid_ignore_settings_file: Option<bool>,
    pub ignore_settings_file: Option<bool>,
    pub tracing: Option<TracingFileData>,
    pub resource_path: Option<ResourcePathFileData>,
}

impl DscPolicyFileData {
    pub fn from_file(file_path: &std::path::Path) -> Result<Self, DscSettingsError> {
        let contents = std::fs::read_to_string(file_path)
            .map_err(|err| DscSettingsError::FileReadError{
                file_path: file_path.to_string_lossy().to_string(),
                source: err,
            })?;
        serde_json::from_str::<DscPolicyFileData>(&contents)
            .map_err(|err| DscSettingsError::ParseDataFileError{
                file_path: file_path.to_string_lossy().to_string(),
                source: err,
            })
    }
}
