// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use dscerror::DscError;
use resource_manifest::ResourceManifest;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use super::{command_resource, dscerror, resource_manifest, invoke_result::{GetResult, SetResult, TestResult, ValidateResult}};

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DscResource {
    /// The namespaced name of the resource.
    #[serde(rename="type")]
    pub type_name: String,
    /// The version of the resource.
    pub version: String,
    /// The file path to the resource.
    pub path: String,
    // The directory path to the resource.
    pub directory: String,
    /// The implementation of the resource.
    #[serde(rename="implementedAs")]
    pub implemented_as: ImplementedAs,
    /// The author of the resource.
    pub author: Option<String>,
    /// The properties of the resource.
    pub properties: Vec<String>,
    /// The required resource provider for the resource.
    pub requires: Option<String>,
    /// The manifest of the resource.
    pub manifest: Option<Value>,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(untagged)]
pub enum ImplementedAs {
    /// A command line executable
    Command,
    /// A custom resource
    Custom(String),
}

impl DscResource {
    #[must_use]
    pub fn new() -> Self {
        Self {
            type_name: String::new(),
            version: String::new(),
            path: String::new(),
            directory: String::new(),
            implemented_as: ImplementedAs::Command,
            author: None,
            properties: Vec::new(),
            requires: None,
            manifest: None,
        }
    }
}

impl Default for DscResource {
    fn default() -> Self {
        DscResource::new()
    }
}

/// The interface for a DSC resource.
pub trait Invoke {
    /// Invoke the get operation on the resource.
    ///
    /// # Arguments
    ///
    /// * `filter` - The filter as JSON to apply to the resource.
    ///
    /// # Errors
    ///
    /// This function will return an error if the underlying resource fails.
    fn get(&self, filter: &str) -> Result<GetResult, DscError>;

    /// Invoke the set operation on the resource.
    ///
    /// # Arguments
    ///
    /// * `desired` - The desired state as JSON to apply to the resource.
    ///
    /// # Errors
    ///
    /// This function will return an error if the underlying resource fails.
    fn set(&self, desired: &str) -> Result<SetResult, DscError>;

    /// Invoke the test operation on the resource.
    ///
    /// # Arguments
    ///
    /// * `expected` - The expected state as JSON to apply to the resource.
    ///
    /// # Errors
    ///
    /// This function will return an error if the underlying resource fails.
    fn test(&self, expected: &str) -> Result<TestResult, DscError>;

    /// Invoke the validate operation on the resource.
    ///
    /// # Arguments
    ///
    /// * `config` - The configuration as JSON to have the resource validate.
    ///
    /// # Errors
    ///
    /// This function will return an error if the underlying resource or validation fails.
    fn validate(&self, config: &str) -> Result<ValidateResult, DscError>;

    /// Get the schema for the resource.
    ///
    /// # Errors
    ///
    /// This function will return an error if the underlying resource fails.
    fn schema(&self) -> Result<String, DscError>;
}

impl Invoke for DscResource {
    fn get(&self, filter: &str) -> Result<GetResult, DscError> {
        match &self.implemented_as {
            ImplementedAs::Custom(_custom) => {
                Err(DscError::NotImplemented("get custom resources".to_string()))
            },
            ImplementedAs::Command => {
                let Some(manifest) = &self.manifest else {
                    return Err(DscError::MissingManifest(self.type_name.clone()));
                };
                let resource_manifest = serde_json::from_value::<ResourceManifest>(manifest.clone())?;
                command_resource::invoke_get(&resource_manifest, &self.directory, filter)
            },
        }
    }

    fn set(&self, desired: &str) -> Result<SetResult, DscError> {
        match &self.implemented_as {
            ImplementedAs::Custom(_custom) => {
                Err(DscError::NotImplemented("set custom resources".to_string()))
            },
            ImplementedAs::Command => {
                let Some(manifest) = &self.manifest else {
                    return Err(DscError::MissingManifest(self.type_name.clone()));
                };
                let resource_manifest = serde_json::from_value::<ResourceManifest>(manifest.clone())?;
                command_resource::invoke_set(&resource_manifest, &self.directory, desired)
            },
        }
    }

    fn test(&self, expected: &str) -> Result<TestResult, DscError> {
        match &self.implemented_as {
            ImplementedAs::Custom(_custom) => {
                Err(DscError::NotImplemented("test custom resources".to_string()))
            },
            ImplementedAs::Command => {
                let Some(manifest) = &self.manifest else {
                    return Err(DscError::MissingManifest(self.type_name.clone()));
                };

                // if test is not directly implemented, then we need to handle it here
                let resource_manifest = serde_json::from_value::<ResourceManifest>(manifest.clone())?;
                if resource_manifest.test.is_none() {
                    let get_result = self.get(expected)?;
                    let expected_state = serde_json::from_str(expected)?;
                    let diff_properties = get_diff(&expected_state, &get_result.actual_state);
                    let test_result = TestResult {
                        expected_state: serde_json::from_str(expected)?,
                        actual_state: get_result.actual_state,
                        in_desired_state: diff_properties.is_empty(),
                        diff_properties,
                    };
                    Ok(test_result)
                }
                else {
                    command_resource::invoke_test(&resource_manifest, &self.directory, expected)
                }
            },
        }
    }

    fn validate(&self, config: &str) -> Result<ValidateResult, DscError> {
        match &self.implemented_as {
            ImplementedAs::Custom(_custom) => {
                Err(DscError::NotImplemented("validate custom resources".to_string()))
            },
            ImplementedAs::Command => {
                let Some(manifest) = &self.manifest else {
                    return Err(DscError::MissingManifest(self.type_name.clone()));
                };
                let resource_manifest = serde_json::from_value::<ResourceManifest>(manifest.clone())?;
                command_resource::invoke_validate(&resource_manifest, &self.directory, config)
            },
        }
    }

    fn schema(&self) -> Result<String, DscError> {
        match &self.implemented_as {
            ImplementedAs::Custom(_custom) => {
                Err(DscError::NotImplemented("schema custom resources".to_string()))
            },
            ImplementedAs::Command => {
                let Some(manifest) = &self.manifest else {
                    return Err(DscError::MissingManifest(self.type_name.clone()));
                };
                let resource_manifest = serde_json::from_value::<ResourceManifest>(manifest.clone())?;
                command_resource::get_schema(&resource_manifest, &self.directory)
            },
        }
    }
}

#[must_use]
pub fn get_diff(expected: &Value, actual: &Value) -> Vec<String> {
    let mut diff_properties: Vec<String> = Vec::new();
    if expected.is_null() {
        return diff_properties;
    }

    if let Some(map) = expected.as_object() {
        for (key, value) in map {
            // skip meta properties
            if key.starts_with('_') || key.starts_with('$') {
                continue;
            }

            if value.is_object() {
                let sub_diff = get_diff(value, &actual[key]);
                if !sub_diff.is_empty() {
                    diff_properties.push(key.to_string());
                }
            }
            else {
                match actual.as_object() {
                    Some(actual_object) => {
                        if actual_object.contains_key(key) {
                            if value != &actual[key] {
                                diff_properties.push(key.to_string());
                            }
                        }
                        else {
                            diff_properties.push(key.to_string());
                        }
                    },
                    None => {
                        diff_properties.push(key.to_string());
                    },
                }
            }
        }
    }

    diff_properties
}
