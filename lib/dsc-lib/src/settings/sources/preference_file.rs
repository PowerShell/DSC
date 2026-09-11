use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::settings::{DscSettingsError, ResourcePathFileData, TracingFileData};

#[derive(Default, Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct DscPreferenceFileData {
    pub tracing: Option<TracingFileData>,
    pub resource_path: Option<ResourcePathFileData>,
}

impl DscPreferenceFileData {
    pub fn from_file(file_path: &std::path::Path) -> Result<Self, DscSettingsError> {
        let contents = std::fs::read_to_string(file_path)
            .map_err(|err| DscSettingsError::FileReadError {
                file_path: file_path.to_string_lossy().to_string(),
                source: err,
            })?;
        serde_json::from_str::<DscPreferenceFileData>(&contents)
            .map_err(|err| DscSettingsError::ParseDataFileError{
                file_path: file_path.to_string_lossy().to_string(),
                source: err,
            })
    }
}
