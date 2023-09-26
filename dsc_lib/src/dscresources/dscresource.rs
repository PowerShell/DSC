// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use dscerror::DscError;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use super::{command_resource, dscerror, resource_manifest::import_manifest, invoke_result::{GetResult, SetResult, TestResult, ValidateResult, ExportResult}};

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
    /// The description of the resource.
    pub description: Option<String>,
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
            description: None,
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
    /// * `skip_test` - Whether to skip the test operation.
    ///
    /// # Errors
    ///
    /// This function will return an error if the underlying resource fails.
    fn set(&self, desired: &str, skip_test: bool) -> Result<SetResult, DscError>;

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

    /// Invoke the export operation on the resource.
    ///
    /// # Errors
    ///
    /// This function will return an error if the underlying resource fails.
    fn export(&self) -> Result<ExportResult, DscError>;
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
                let resource_manifest = import_manifest(manifest.clone())?;
                command_resource::invoke_get(&resource_manifest, &self.directory, filter)
            },
        }
    }

    fn set(&self, desired: &str, skip_test: bool) -> Result<SetResult, DscError> {
        match &self.implemented_as {
            ImplementedAs::Custom(_custom) => {
                Err(DscError::NotImplemented("set custom resources".to_string()))
            },
            ImplementedAs::Command => {
                let Some(manifest) = &self.manifest else {
                    return Err(DscError::MissingManifest(self.type_name.clone()));
                };
                let resource_manifest = import_manifest(manifest.clone())?;
                command_resource::invoke_set(&resource_manifest, &self.directory, desired, skip_test)
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
                let resource_manifest = import_manifest(manifest.clone())?;
                if resource_manifest.test.is_none() {
                    let get_result = self.get(expected)?;
                    let desired_state = serde_json::from_str(expected)?;
                    let diff_properties = get_diff(&desired_state, &get_result.actual_state);
                    let test_result = TestResult {
                        desired_state: serde_json::from_str(expected)?,
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
                let resource_manifest = import_manifest(manifest.clone())?;
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
                let resource_manifest = import_manifest(manifest.clone())?;
                command_resource::get_schema(&resource_manifest, &self.directory)
            },
        }
    }

    fn export(&self) -> Result<ExportResult, DscError> {
        match &self.implemented_as {
            ImplementedAs::Custom(_custom) => {
                Err(DscError::NotImplemented("export custom resources".to_string()))
            },
            ImplementedAs::Command => {
                let Some(manifest) = &self.manifest else {
                    return Err(DscError::MissingManifest(self.type_name.clone()));
                };
                let resource_manifest = import_manifest(manifest.clone())?;
                command_resource::invoke_export(&resource_manifest, &self.directory)
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
        let mut handled_exist = false;
        for (key, value) in map {
            if value.is_object() {
                let sub_diff = get_diff(value, &actual[key]);
                if !sub_diff.is_empty() {
                    diff_properties.push(key.to_string());
                }
            }
            else {
                match actual.as_object() {
                    Some(actual_object) => {
                        // handle `_exist` which defaults to `true` if not specified
                        if key.eq("_exist") {
                            handled_exist = true;
                            // if actual object doesn't have `_exist`, it's assumed to be `true`
                            if !actual_object.contains_key(key) {
                                if value.as_bool() == Some(false) {
                                    diff_properties.push(key.to_string());
                                }
                            }
                            else if value != &actual[key] {
                                diff_properties.push(key.to_string());
                            }
                        }
                        else if actual_object.contains_key(key) {
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

        // handle the case where the actual object has `_exist` but wasn't specified in the expected object
        if !handled_exist {
            if let Some(actual_object) = actual.as_object() {
                if actual_object.contains_key("_exist") {
                    // if expected didn't have `_exist`, it is assumed to be `true` so we only handle `false` case
                    if actual["_exist"].as_bool() != Some(true) {
                        diff_properties.push("_exist".to_string());
                    }
                }
            }
        }
    }

    diff_properties
}
