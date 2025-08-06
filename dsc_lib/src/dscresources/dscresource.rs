// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::{configure::{config_doc::{Configuration, ExecutionKind, Resource}, Configurator}, dscresources::resource_manifest::Kind};
use crate::dscresources::invoke_result::{ResourceGetResponse, ResourceSetResponse};
use dscerror::DscError;
use rust_i18n::t;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
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
#[serde(rename_all = "camelCase")]
pub enum Capability {
    /// The resource supports retrieving configuration.
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

    fn create_config_for_adapter(self, adapter: &str, input: &str) -> Result<Configurator, DscError> {
        // create new configuration with adapter and use this as the resource
        let mut configuration = Configuration::new();
        let mut property_map = Map::new();
        property_map.insert("name".to_string(), Value::String(self.type_name.clone()));
        property_map.insert("type".to_string(), Value::String(self.type_name.clone()));
        if !input.is_empty() {
            let resource_properties: Value = serde_json::from_str(input)?;
            property_map.insert("properties".to_string(), resource_properties);
        }
        let mut resources_map = Map::new();
        resources_map.insert("resources".to_string(), Value::Array(vec![Value::Object(property_map)]));
        let adapter_resource = Resource {
            name: self.type_name.clone(),
            resource_type: adapter.to_string(),
            properties: Some(resources_map),
            ..Default::default()
        };
        configuration.resources.push(adapter_resource);
        let config_json = serde_json::to_string(&configuration)?;
        let mut configurator = Configurator::new(&config_json, crate::progress::ProgressFormat::None)?;
        // don't process expressions again as they would have already been processed before being passed to the adapter
        configurator.process_expressions = false;
        Ok(configurator)
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
        debug!("{}", t!("dscresources.dscresource.invokeGet", resource = self.type_name));
        if let Some(adapter) = &self.require_adapter {
            let mut configurator = self.clone().create_config_for_adapter(adapter, filter)?;
            let result = configurator.invoke_get()?;
            let GetResult::Resource(ref resource_result) = result.results[0].result else {
                return Err(DscError::Operation(t!("dscresources.dscresource.invokeReturnedWrongResult", operation = "get", resource = self.type_name).to_string()));
            };
            let properties = resource_result.actual_state
                .as_object().ok_or(DscError::Operation(t!("dscresources.dscresource.propertyIncorrectType", property = "actualState", property_type = "object").to_string()))?
                .get("result").ok_or(DscError::Operation(t!("dscresources.dscresource.propertyNotFound", property = "result").to_string()))?
                .as_array().ok_or(DscError::Operation(t!("dscresources.dscresource.propertyIncorrectType", property = "result", property_type = "array").to_string()))?[0]
                .as_object().ok_or(DscError::Operation(t!("dscresources.dscresource.propertyIncorrectType", property = "result", property_type = "object").to_string()))?
                .get("properties").ok_or(DscError::Operation(t!("dscresources.dscresource.propertyNotFound", property = "properties").to_string()))?.clone();
            let get_result = GetResult::Resource(ResourceGetResponse {
                actual_state: properties.clone(),
            });
            return Ok(get_result);
        }

        match &self.implemented_as {
            ImplementedAs::Custom(_custom) => {
                Err(DscError::NotImplemented(t!("dscresources.dscresource.customResourceNotSupported").to_string()))
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
        debug!("{}", t!("dscresources.dscresource.invokeSet", resource = self.type_name));
        if let Some(adapter) = &self.require_adapter {
            let mut configurator = self.clone().create_config_for_adapter(adapter, desired)?;
            let result = configurator.invoke_set(false)?;
            let SetResult::Resource(ref resource_result) = result.results[0].result else {
                return Err(DscError::Operation(t!("dscresources.dscresource.invokeReturnedWrongResult", operation = "set", resource = self.type_name).to_string()));
            };
            let before_state = resource_result.before_state
                .as_object().ok_or(DscError::Operation(t!("dscresources.dscresource.propertyIncorrectType", property = "beforeState", property_type = "object").to_string()))?
                .get("resources").ok_or(DscError::Operation(t!("dscresources.dscresource.propertyNotFound", property = "resources").to_string()))?
                .as_array().ok_or(DscError::Operation(t!("dscresources.dscresource.propertyIncorrectType", property = "result", property_type = "array").to_string()))?[0]
                .as_object().ok_or(DscError::Operation(t!("dscresources.dscresource.propertyIncorrectType", property = "result", property_type = "object").to_string()))?
                .get("properties").ok_or(DscError::Operation(t!("dscresources.dscresource.propertyNotFound", property = "properties").to_string()))?.clone();
            let after_state = resource_result.after_state
                .as_object().ok_or(DscError::Operation(t!("dscresources.dscresource.propertyIncorrectType", property = "afterState", property_type = "object").to_string()))?
                .get("result").ok_or(DscError::Operation(t!("dscresources.dscresource.propertyNotFound", property = "result").to_string()))?
                .as_array().ok_or(DscError::Operation(t!("dscresources.dscresource.propertyIncorrectType", property = "result", property_type = "array").to_string()))?[0]
                .as_object().ok_or(DscError::Operation(t!("dscresources.dscresource.propertyIncorrectType", property = "result", property_type = "object").to_string()))?
                .get("properties").ok_or(DscError::Operation(t!("dscresources.dscresource.propertyNotFound", property = "properties").to_string()))?.clone();
            let diff = get_diff(&before_state, &after_state);
            let set_result = SetResult::Resource(ResourceSetResponse {
                before_state: before_state.clone(),
                after_state: after_state.clone(),
                changed_properties: if diff.is_empty() { None } else { Some(diff) },
            });
            return Ok(set_result);
        }

        match &self.implemented_as {
            ImplementedAs::Custom(_custom) => {
                Err(DscError::NotImplemented(t!("dscresources.dscresource.customResourceNotSupported").to_string()))
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
        debug!("{}", t!("dscresources.dscresource.invokeTest", resource = self.type_name));
        if let Some(adapter) = &self.require_adapter {
            let mut configurator = self.clone().create_config_for_adapter(adapter, expected)?;
            let result = configurator.invoke_test()?;
            let TestResult::Resource(ref resource_result) = result.results[0].result else {
                return Err(DscError::Operation(t!("dscresources.dscresource.invokeReturnedWrongResult", operation = "test", resource = self.type_name).to_string()));
            };
            let desired_state = resource_result.desired_state
                .as_object().ok_or(DscError::Operation(t!("dscresources.dscresource.propertyIncorrectType", property = "desiredState", property_type = "object").to_string()))?
                .get("resources").ok_or(DscError::Operation(t!("dscresources.dscresource.propertyNotFound", property = "resources").to_string()))?
                .as_array().ok_or(DscError::Operation(t!("dscresources.dscresource.propertyIncorrectType", property = "resources", property_type = "array").to_string()))?[0]
                .as_object().ok_or(DscError::Operation(t!("dscresources.dscresource.propertyIncorrectType", property = "resources", property_type = "object").to_string()))?
                .get("properties").ok_or(DscError::Operation(t!("dscresources.dscresource.propertyNotFound", property = "properties").to_string()))?.clone();
            let actual_state = resource_result.actual_state
                .as_object().ok_or(DscError::Operation(t!("dscresources.dscresource.propertyIncorrectType", property = "actualState", property_type = "object").to_string()))?
                .get("result").ok_or(DscError::Operation(t!("dscresources.dscresource.propertyNotFound", property = "result").to_string()))?
                .as_array().ok_or(DscError::Operation(t!("dscresources.dscresource.propertyIncorrectType", property = "result", property_type = "array").to_string()))?[0]
                .as_object().ok_or(DscError::Operation(t!("dscresources.dscresource.propertyIncorrectType", property = "result", property_type = "object").to_string()))?
                .get("properties").ok_or(DscError::Operation(t!("dscresources.dscresource.propertyNotFound", property = "properties").to_string()))?.clone();
            let diff_properties = get_diff(&desired_state, &actual_state);
            let test_result = TestResult::Resource(ResourceTestResponse {
                desired_state,
                actual_state,
                in_desired_state: resource_result.in_desired_state,
                diff_properties,
            });
            return Ok(test_result);
        }

        match &self.implemented_as {
            ImplementedAs::Custom(_custom) => {
                Err(DscError::NotImplemented(t!("dscresources.dscresource.customResourceNotSupported").to_string()))
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
        debug!("{}", t!("dscresources.dscresource.invokeDelete", resource = self.type_name));
        if let Some(adapter) = &self.require_adapter {
            let mut configurator = self.clone().create_config_for_adapter(adapter, filter)?;
            configurator.invoke_set(false)?;
            return Ok(());
        }

        match &self.implemented_as {
            ImplementedAs::Custom(_custom) => {
                Err(DscError::NotImplemented(t!("dscresources.dscresource.customResourceNotSupported").to_string()))
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
        debug!("{}", t!("dscresources.dscresource.invokeValidate", resource = self.type_name));
        if self.require_adapter.is_some() {
            return Err(DscError::NotSupported(t!("dscresources.dscresource.invokeValidateNotSupported", resource = self.type_name).to_string()));
        }

        match &self.implemented_as {
            ImplementedAs::Custom(_custom) => {
                Err(DscError::NotImplemented(t!("dscresources.dscresource.customResourceNotSupported").to_string()))
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
        debug!("{}", t!("dscresources.dscresource.invokeSchema", resource = self.type_name));
        if self.require_adapter.is_some() {
            return Err(DscError::NotSupported(t!("dscresources.dscresource.invokeSchemaNotSupported", resource = self.type_name).to_string()));
        }

        match &self.implemented_as {
            ImplementedAs::Custom(_custom) => {
                Err(DscError::NotImplemented(t!("dscresources.dscresource.customResourceNotSupported").to_string()))
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
        debug!("{}", t!("dscresources.dscresource.invokeExport", resource = self.type_name));
        if let Some(adapter) = &self.require_adapter {
            let mut configurator = self.clone().create_config_for_adapter(adapter, input)?;
            let result = configurator.invoke_export()?;
            let Some(configuration) = result.result else {
                return Err(DscError::Operation(t!("dscresources.dscresource.invokeExportReturnedNoResult", resource = self.type_name).to_string()));
            };
            let mut export_result = ExportResult {
                actual_state: Vec::new(),
            };
            debug!("Export result: {}", serde_json::to_string(&configuration)?);
            for resource in configuration.resources {
                let Some(properties) = resource.properties else {
                    return Err(DscError::Operation(t!("dscresources.dscresource.invokeExportReturnedNoResult", resource = self.type_name).to_string()));
                };
                let results = properties
                    .get("result").ok_or(DscError::Operation(t!("dscresources.dscresource.propertyNotFound", property = "result").to_string()))?
                    .as_array().ok_or(DscError::Operation(t!("dscresources.dscresource.propertyIncorrectType", property = "result", property_type = "array").to_string()))?;
                for result in results {
                    export_result.actual_state.push(serde_json::to_value(result.clone())?);
                }
            }
            return Ok(export_result);
        }

        let Some(manifest) = &self.manifest else {
            return Err(DscError::MissingManifest(self.type_name.clone()));
        };
        let resource_manifest = import_manifest(manifest.clone())?;
        command_resource::invoke_export(&resource_manifest, &self.directory, Some(input))
    }

    fn resolve(&self, input: &str) -> Result<ResolveResult, DscError> {
        debug!("{}", t!("dscresources.dscresource.invokeResolve", resource = self.type_name));
        if self.require_adapter.is_some() {
            return Err(DscError::NotSupported(t!("dscresources.dscresource.invokeResolveNotSupported", resource = self.type_name).to_string()));
        }

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
                    debug!("{}", t!("dscresources.dscresource.subDiff", key = key));
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
                                    info!("{}", t!("dscresources.dscresource.diffArray", key = key));
                                    diff_properties.push(key.to_string());
                                }
                            } else {
                                info!("{}", t!("dscresources.dscresource.diffNotArray", key = actual[key]));
                                diff_properties.push(key.to_string());
                            }
                        } else if value != &actual[key] {
                            diff_properties.push(key.to_string());
                        }
                    } else {
                        info!("{}", t!("dscresources.dscresource.diffKeyMissing", key = key));
                        diff_properties.push(key.to_string());
                    }
                } else {
                    info!("{}", t!("dscresources.dscresource.diffKeyNotObject", key = key));
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
        info!("{}", t!("dscresources.dscresource.diffArraySize"));
        return false;
    }

    for item in expected {
        if !array_contains(actual, item) {
            info!("{}", t!("dscresources.dscresource.diffMissingItem"));
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
