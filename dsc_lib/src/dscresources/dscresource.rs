// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::{configure::config_doc::ExecutionKind, dscresources::resource_manifest::Kind};
use dscerror::DscError;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

use super::{command_resource, dscerror, invoke_result::{ExportResult, GetResult, ResourceTestResponse, SetResult, TestResult, ValidateResult, WhatIfChanges}, resource_manifest::import_manifest};

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DscResource {
    /// The namespaced name of the resource.
    #[serde(rename="type")]
    pub type_name: String,
    /// The kind of resource.
    pub kind: Kind,
    /// The version of the resource.
    pub version: String,
    /// The capabilities of the resource.
    pub capabilities: Vec<Capability>,
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
    /// The required resource adapter for the resource.
    #[serde(rename="requireAdapter")]
    pub require_adapter: Option<String>,
    /// The manifest of the resource.
    pub manifest: Option<Value>,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize, JsonSchema)]
pub enum Capability {
    /// The resource supports retriving configuration.
    Get,
    /// The resource supports applying configuration.
    Set,
    /// The resource supports the `_exist` property directly.
    SetHandlesExist,
    /// The resource supports validating configuration.
    Test,
    /// The resource supports deleting configuration.
    Delete,
    /// The resource supports exporting configuration.
    Export,
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
            kind: Kind::Resource,
            version: String::new(),
            capabilities: Vec::new(),
            description: None,
            path: String::new(),
            directory: String::new(),
            implemented_as: ImplementedAs::Command,
            author: None,
            properties: Vec::new(),
            require_adapter: None,
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
    fn set(&self, desired: &str, skip_test: bool, execution_type: &ExecutionKind) -> Result<SetResult, DscError>;

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

    /// Invoke the delete operation on the resource.
    ///
    /// # Arguments
    ///
    /// * `filter` - The filter as JSON to apply to the resource.
    ///
    /// # Errors
    ///
    /// This function will return an error if the underlying resource fails.
    fn delete(&self, filter: &str, execution_type: &ExecutionKind) -> Result<Option<SetResult>, DscError>;

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
    /// # Arguments
    ///
    /// * `input` - Input for export operation.
    ///
    /// # Errors
    ///
    /// This function will return an error if the underlying resource fails.
    fn export(&self, input: &str) -> Result<ExportResult, DscError>;
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

    fn set(&self, desired: &str, skip_test: bool, execution_type: &ExecutionKind) -> Result<SetResult, DscError> {
        match &self.implemented_as {
            ImplementedAs::Custom(_custom) => {
                Err(DscError::NotImplemented("set custom resources".to_string()))
            },
            ImplementedAs::Command => {
                let Some(manifest) = &self.manifest else {
                    return Err(DscError::MissingManifest(self.type_name.clone()));
                };
                let resource_manifest = import_manifest(manifest.clone())?;
                command_resource::invoke_set(&resource_manifest, &self.directory, desired, skip_test, execution_type)
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
                    let actual_state = match get_result {
                        GetResult::Group(results) => {
                            let mut result_array: Vec<Value> = Vec::new();
                            for result in results {
                                result_array.push(serde_json::to_value(result)?);
                            }
                            Value::from(result_array)
                        },
                        GetResult::Resource(response) => {
                            response.actual_state
                        }
                    };
                    let diff_properties = get_diff( &desired_state, &actual_state);
                    let test_result = TestResult::Resource(ResourceTestResponse {
                        desired_state: serde_json::from_str(expected)?,
                        actual_state,
                        in_desired_state: diff_properties.is_empty(),
                        diff_properties,
                    });
                    Ok(test_result)
                }
                else {
                    command_resource::invoke_test(&resource_manifest, &self.directory, expected)
                }
            },
        }
    }

    fn delete(&self, filter: &str, execution_type: &ExecutionKind) -> Result<Option<SetResult>, DscError> {
        match &self.implemented_as {
            ImplementedAs::Custom(_custom) => {
                Err(DscError::NotImplemented("set custom resources".to_string()))
            },
            ImplementedAs::Command => {
                let Some(manifest) = &self.manifest else {
                    return Err(DscError::MissingManifest(self.type_name.clone()));
                };
                let resource_manifest = import_manifest(manifest.clone())?;
                command_resource::invoke_delete(&resource_manifest, &self.directory, filter, execution_type)
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

    fn export(&self, input: &str) -> Result<ExportResult, DscError> {
        let Some(manifest) = &self.manifest else {
            return Err(DscError::MissingManifest(self.type_name.clone()));
        };
        let resource_manifest = import_manifest(manifest.clone())?;
        command_resource::invoke_export(&resource_manifest, &self.directory, Some(input))
    }
}

#[must_use]
pub fn get_well_known_properties() -> HashMap<String, Value> {
    HashMap::<String, Value>::from([
        ("_exist".to_string(), Value::Bool(true)),
    ])
}

#[must_use]
pub fn get_diff(expected: &Value, actual: &Value) -> Vec<String> {
    let mut diff_properties: Vec<String> = Vec::new();
    if expected.is_null() {
        return diff_properties;
    }

    let mut expected = expected.clone();
    let mut actual = actual.clone();

    if let Some(map) = expected.as_object_mut() {
        // handle well-known optional properties with default values by adding them
        for (key, value) in get_well_known_properties() {
            if !map.contains_key(&key) {
                map.insert(key.clone(), value.clone());
            }

            if actual.is_object() && actual[&key].is_null() {
                actual[key.clone()] = value.clone();
            }
        }

        for (key, value) in &*map {
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

#[must_use]
pub fn get_diff_what_if(expected: &Value, actual: &Value) -> Vec<WhatIfChanges> {
    let mut diff_properties: Vec<WhatIfChanges> = Vec::new();
    if expected.is_null() {
        return diff_properties;
    }

    let mut expected = expected.clone();
    let mut actual = actual.clone();

    if let Some(map) = expected.as_object_mut() {
        // handle well-known optional properties with default values by adding them
        for (key, value) in get_well_known_properties() {
            if !map.contains_key(&key) {
                map.insert(key.clone(), value.clone());
            }

            if actual.is_object() && actual[&key].is_null() {
                actual[key.clone()] = value.clone();
            }
        }

        for (key, value) in &*map {
            let mut is_diff = false;
            if value.is_object() {
                let sub_diff = get_diff_what_if(value, &actual[key]);
                if !sub_diff.is_empty() {
                    is_diff = true;
                }
            }
            else {
                match actual.as_object() {
                    Some(actual_object) => {
                        if actual_object.contains_key(key) {
                            if value != &actual[key] {
                                is_diff = true;
                            }
                        }
                        else {
                            is_diff = true;
                        }
                    },
                    None => {
                        is_diff = true;
                    },
                }
            }
            if is_diff {
                diff_properties.push(
                    WhatIfChanges {
                        name: key.to_string(),
                        from: actual[key].clone(),
                        to: value.clone(),
                    }
                );
            }
        }
    }

    diff_properties
}
