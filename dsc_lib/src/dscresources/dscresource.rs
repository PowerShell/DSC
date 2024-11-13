// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::{configure::config_doc::ExecutionKind, dscresources::resource_manifest::Kind};
use dscerror::DscError;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use tracing::{debug, info};

use super::{command_resource, dscerror, invoke_result::{ExportResult, GetResult, ResolveResult, ResourceTestResponse, SetResult, TestResult, ValidateResult}, resource_manifest::import_manifest};

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
    /// The resource supports simulating configuration directly.
    WhatIf,
    /// The resource supports validating configuration.
    Test,
    /// The resource supports deleting configuration.
    Delete,
    /// The resource supports exporting configuration.
    Export,
    /// The resource supports resolving imported configuration.
    Resolve,
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
    fn delete(&self, filter: &str) -> Result<(), DscError>;

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

    /// Invoke the resolve operation on the resource.
    ///
    /// # Arguments
    ///
    /// * `input` - The input to the operation to be resolved.
    ///
    /// # Errors
    ///
    /// This function will return an error if the underlying resource fails.
    fn resolve(&self, input: &str) -> Result<ResolveResult, DscError>;
}

impl Invoke for DscResource {
    fn get(&self, filter: &str) -> Result<GetResult, DscError> {
        debug!("Invoking get for resource: {}", self.type_name);
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
        debug!("Invoking set for resource: {}", self.type_name);
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
        debug!("Invoking test for resource: {}", self.type_name);
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
                        desired_state,
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

    fn delete(&self, filter: &str) -> Result<(), DscError> {
        debug!("Invoking delete for resource: {}", self.type_name);
        match &self.implemented_as {
            ImplementedAs::Custom(_custom) => {
                Err(DscError::NotImplemented("set custom resources".to_string()))
            },
            ImplementedAs::Command => {
                let Some(manifest) = &self.manifest else {
                    return Err(DscError::MissingManifest(self.type_name.clone()));
                };
                let resource_manifest = import_manifest(manifest.clone())?;
                command_resource::invoke_delete(&resource_manifest, &self.directory, filter)
            },
        }
    }

    fn validate(&self, config: &str) -> Result<ValidateResult, DscError> {
        debug!("Invoking validate for resource: {}", self.type_name);
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
        debug!("Invoking schema for resource: {}", self.type_name);
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
        debug!("Invoking export for resource: {}", self.type_name);
        let Some(manifest) = &self.manifest else {
            return Err(DscError::MissingManifest(self.type_name.clone()));
        };
        let resource_manifest = import_manifest(manifest.clone())?;
        command_resource::invoke_export(&resource_manifest, &self.directory, Some(input))
    }

    fn resolve(&self, input: &str) -> Result<ResolveResult, DscError> {
        debug!("Invoking resolve for resource: {}", self.type_name);
        let Some(manifest) = &self.manifest else {
            return Err(DscError::MissingManifest(self.type_name.clone()));
        };
        let resource_manifest = import_manifest(manifest.clone())?;
        command_resource::invoke_resolve(&resource_manifest, &self.directory, input)
    }
}

#[must_use]
pub fn get_well_known_properties() -> HashMap<String, Value> {
    HashMap::<String, Value>::from([
        ("_exist".to_string(), Value::Bool(true)),
    ])
}

#[must_use]
/// Performs a comparison of two JSON Values if the expected is a strict subset of the actual
///
/// # Arguments
///
/// * `expected` - The expected value
/// * `actual` - The actual value
///
/// # Returns
///
/// An array of top level properties that differ, if any
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
                    debug!("diff: sub diff for {key}");
                    diff_properties.push(key.to_string());
                }
            }
            else {
                // skip `$schema` key as that is provided as input, but not output typically
                if key == "$schema" {
                    continue;
                }

                if let Some(actual_object) = actual.as_object() {
                    if actual_object.contains_key(key) {
                        if let Some(value_array) = value.as_array() {
                            if let Some(actual_array) = actual[key].as_array() {
                                if !is_same_array(value_array, actual_array) {
                                    info!("diff: arrays differ for {key}");
                                    diff_properties.push(key.to_string());
                                }
                            } else {
                                info!("diff: {} is not an array", actual[key]);
                                diff_properties.push(key.to_string());
                            }
                        } else if value != &actual[key] {
                            diff_properties.push(key.to_string());
                        }
                    } else {
                        info!("diff: {key} missing");
                        diff_properties.push(key.to_string());
                    }
                } else {
                    info!("diff: {key} not object");
                    diff_properties.push(key.to_string());
                }
            }
        }
    }

    diff_properties
}

/// Compares two arrays independent of order
fn is_same_array(expected: &Vec<Value>, actual: &Vec<Value>) -> bool {
    if expected.len() != actual.len() {
        info!("diff: arrays are different lengths");
        return false;
    }

    for item in expected {
        if !array_contains(actual, item) {
            info!("diff: actual array missing expected element");
            return false;
        }
    }

    true
}

fn array_contains(array: &Vec<Value>, find: &Value) -> bool {
    for item in array {
        if find.is_boolean() && item.is_boolean() && find.as_bool().unwrap() == item.as_bool().unwrap() {
            return true;
        }

        if find.is_f64() && item.is_f64() && (find.as_f64().unwrap() - item.as_f64().unwrap()).abs() < 0.1 {
            return true;
        }

        if find.is_i64() && item.is_i64() && find.as_i64().unwrap() == item.as_i64().unwrap() {
            return true;
        }

        if find.is_null() && item.is_null() {
            return true;
        }

        if find.is_number() && item.is_number() && find.as_number().unwrap() == item.as_number().unwrap() {
            return true;
        }

        if find.is_string() && item.is_string() && find.as_str().unwrap() == item.as_str().unwrap() {
            return true;
        }

        if find.is_u64() && item.is_u64() && find.as_u64().unwrap() == item.as_u64().unwrap() {
            return true;
        }

        if find.is_object() && item.is_object() {
            let obj_diff = get_diff(find, item);
            if obj_diff.is_empty() {
                return true;
            }
        }

        if find.is_array() && item.is_array() && is_same_array(item.as_array().unwrap(), find.as_array().unwrap()) {
            return true;
        }
    }

    false
}

#[test]
fn same_array() {
    use serde_json::json;
    let array_one = vec![json!("a"), json!(1), json!({"a":"b"}), json!(null)];
    let array_two = vec![json!("a"), json!(1), json!({"a":"b"}), json!(null)];
    assert_eq!(is_same_array(&array_one, &array_two), true);
}

#[test]
fn same_array_out_of_order() {
    use serde_json::json;
    let array_one = vec![json!("a"), json!(true), json!({"a":"b"})];
    let array_two = vec![json!({"a":"b"}), json!("a"), json!(true)];
    assert_eq!(is_same_array(&array_one, &array_two), true);
}

#[test]
fn different_array() {
    use serde_json::json;
    let array_one = vec![json!("a"), json!(1), json!({"a":"b"})];
    let array_two = vec![json!({"a":"b"}), json!("a"), json!(2)];
    assert_eq!(is_same_array(&array_one, &array_two), false);
}

#[test]
fn different_array_sizes() {
    use serde_json::json;
    let array_one = vec![json!("a"), json!(1), json!({"a":"b"})];
    let array_two = vec![json!({"a":"b"}), json!("a")];
    assert_eq!(is_same_array(&array_one, &array_two), false);
}

#[test]
fn array_with_multiple_objects_with_actual_superset() {
    use serde_json::json;
    let array_one = vec![json!("a"), json!(1), json!({"a":"b"}), json!({"c":"d"})];
    let array_two = vec![json!("a"), json!(1), json!({"c":"d", "a":"b"}), json!({"c":"d"})];
    assert_eq!(is_same_array(&array_one, &array_two), true);
}

#[test]
fn array_with_multiple_objects_with_expected_superset() {
    use serde_json::json;
    let array_one = vec![json!("a"), json!(1), json!({"a":"b", "c":"d"}), json!({"c":"d"})];
    let array_two = vec![json!("a"), json!(1), json!({"a":"b"}), json!({"c":"d"})];
    assert_eq!(is_same_array(&array_one, &array_two), false);
}

#[test]
fn array_with_duplicates_out_of_order() {
    use serde_json::json;
    let array_one = vec![json!("a"), json!(1), json!({"a":"b"}), json!({"a":"b"})];
    let array_two = vec![json!({"a":"b"}), json!("a"), json!(1), json!({"a":"b"})];
    assert_eq!(is_same_array(&array_one, &array_two), true);
}

#[test]
fn same_array_with_nested_array() {
    use serde_json::json;
    let array_one = vec![json!("a"), json!(1), json!({"a":"b"}), json!(vec![json!("a"), json!(1)])];
    let array_two = vec![json!("a"), json!(1), json!({"a":"b"}), json!(vec![json!("a"), json!(1)])];
    assert_eq!(is_same_array(&array_one, &array_two), true);
}

#[test]
fn different_array_with_nested_array() {
    use serde_json::json;
    let array_one = vec![json!("a"), json!(1), json!({"a":"b"}), json!(vec![json!("a"), json!(1)])];
    let array_two = vec![json!("a"), json!(1), json!({"a":"b"}), json!(vec![json!("a"), json!(2)])];
    assert_eq!(is_same_array(&array_one, &array_two), false);
}
