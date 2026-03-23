// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::{configure::{Configurator, config_doc::{Configuration, ExecutionKind, Resource}, context::ProcessMode, parameters::{SECURE_VALUE_REDACTED, is_secure_value}}, dscresources::resource_manifest::{AdapterInputKind, Kind}, types::FullyQualifiedTypeName};
use crate::discovery::discovery_trait::DiscoveryFilter;
use crate::dscresources::invoke_result::{ResourceGetResponse, ResourceSetResponse};
use crate::schemas::transforms::idiomaticize_string_enum;
use dscerror::DscError;
use jsonschema::Validator;
use rust_i18n::t;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::collections::HashMap;
use std::path::PathBuf;
use tracing::{debug, info, trace, warn};

use crate::schemas::dsc_repo::DscRepoSchema;

use super::{
    command_resource,
    dscerror,
    invoke_result::{
        DeleteResultKind, ExportResult, GetResult, ResolveResult, ResourceTestResponse, SetResult, TestResult, ValidateResult
    },
    resource_manifest::ResourceManifest,
};

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema, DscRepoSchema)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
#[dsc_repo_schema(base_name = "list", folder_path = "outputs/resource")]
pub struct DscResource {
    /// The namespaced name of the resource.
    #[serde(rename="type")]
    pub type_name: FullyQualifiedTypeName,
    /// The kind of resource.
    pub kind: Kind,
    /// The version of the resource.
    pub version: String,
    /// The capabilities of the resource.
    pub capabilities: Vec<Capability>,
    /// An optional message indicating the resource is deprecated.  If provided, the message will be shown when the resource is used.
    pub deprecation_message: Option<String>,
    /// The file path to the resource.
    pub path: PathBuf,
    /// The description of the resource.
    pub description: Option<String>,
    // The directory path to the resource.
    pub directory: PathBuf,
    /// The implementation of the resource.
    pub implemented_as: Option<ImplementedAs>,
    /// The author of the resource.
    pub author: Option<String>,
    /// The properties of the resource.
    pub properties: Option<Vec<String>>,
    /// The required resource adapter for the resource.
    pub require_adapter: Option<FullyQualifiedTypeName>,
    /// The JSON Schema of the resource.
    pub schema: Option<Map<String, Value>>,
    /// The target resource for the resource adapter.
    pub target_resource: Option<Box<DscResource>>,
    /// The manifest of the resource.
    pub manifest: Option<ResourceManifest>,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize, JsonSchema, DscRepoSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(transform = idiomaticize_string_enum)]
#[dsc_repo_schema(base_name = "resourceCapabilities", folder_path = "definitions")]
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
            type_name: FullyQualifiedTypeName::default(),
            kind: Kind::Resource,
            version: String::new(),
            capabilities: Vec::new(),
            deprecation_message: None,
            description: None,
            path: PathBuf::new(),
            directory: PathBuf::new(),
            implemented_as: Some(ImplementedAs::Command),
            author: None,
            properties: None,
            require_adapter: None,
            schema: None,
            target_resource: None,
            manifest: None,
        }
    }

    fn create_config_for_adapter(self, adapter: &FullyQualifiedTypeName, input: &str) -> Result<Configurator, DscError> {
        // create new configuration with adapter and use this as the resource
        let mut configuration = Configuration::new();
        let mut property_map = Map::new();
        property_map.insert("name".to_string(), Value::String(self.type_name.to_string()));
        property_map.insert("type".to_string(), Value::String(self.type_name.to_string()));
        if !input.is_empty() {
            let resource_properties: Value = serde_json::from_str(input)?;
            property_map.insert("properties".to_string(), resource_properties);
        }
        let mut resources_map = Map::new();
        resources_map.insert("resources".to_string(), Value::Array(vec![Value::Object(property_map)]));
        let adapter_resource = Resource {
            name: self.type_name.to_string(),
            resource_type: adapter.parse()?,
            properties: Some(resources_map),
            ..Default::default()
        };
        configuration.resources.push(adapter_resource);
        let config_json = serde_json::to_string(&configuration)?;
        let mut configurator = Configurator::new(&config_json, crate::progress::ProgressFormat::None)?;
        // don't process expressions again as they would have already been processed before being passed to the adapter
        configurator.context.process_mode = ProcessMode::NoExpressionEvaluation;
        Ok(configurator)
    }

    fn invoke_get_with_adapter(&self, adapter: &FullyQualifiedTypeName, target_resource: &DscResource, filter: &str) -> Result<GetResult, DscError> {
        let mut configurator = self.clone().create_config_for_adapter(adapter, filter)?;
        let mut adapter = Self::get_adapter_resource(&mut configurator, adapter)?;
        if get_adapter_input_kind(&adapter)? == AdapterInputKind::Single {
            adapter.target_resource = Some(Box::new(target_resource.clone()));
            return adapter.get(filter);
        }

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
        Ok(get_result)
    }

    fn invoke_set_with_adapter(&self, adapter: &FullyQualifiedTypeName, target_resource: &DscResource, desired: &str, skip_test: bool, execution_type: &ExecutionKind) -> Result<SetResult, DscError> {
        let mut configurator = self.clone().create_config_for_adapter(adapter, desired)?;
        let mut adapter = Self::get_adapter_resource(&mut configurator, adapter)?;
        if get_adapter_input_kind(&adapter)? == AdapterInputKind::Single {
            adapter.target_resource = Some(Box::new(target_resource.clone()));
            return adapter.set(desired, skip_test, execution_type);
        }

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
        Ok(set_result)
    }

    fn invoke_test_with_adapter(&self, adapter: &FullyQualifiedTypeName, target_resource: &DscResource, expected: &str) -> Result<TestResult, DscError> {
        let mut configurator = self.clone().create_config_for_adapter(adapter, expected)?;
        let mut adapter = Self::get_adapter_resource(&mut configurator, adapter)?;
        if get_adapter_input_kind(&adapter)? == AdapterInputKind::Single {
            adapter.target_resource = Some(Box::new(target_resource.clone()));
            return adapter.test(expected);
        }

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
        Ok(test_result)
    }

    fn invoke_delete_with_adapter(&self, adapter: &FullyQualifiedTypeName, target_resource: &DscResource, filter: &str, execution_type: &ExecutionKind) -> Result<DeleteResultKind, DscError> {
        let mut configurator = self.clone().create_config_for_adapter(adapter, filter)?;
        let mut adapter = Self::get_adapter_resource(&mut configurator, adapter)?;
        if get_adapter_input_kind(&adapter)? == AdapterInputKind::Single {
            if adapter.capabilities.contains(&Capability::Delete) {
                adapter.target_resource = Some(Box::new(target_resource.clone()));
                return adapter.delete(filter, execution_type);
            }
            return Err(DscError::NotSupported(t!("dscresources.dscresource.adapterDoesNotSupportDelete", adapter = adapter.type_name).to_string()));
        }

        configurator.invoke_set(false)?;
        Ok(DeleteResultKind::ResourceActual)
    }

    fn invoke_export_with_adapter(&self, adapter: &FullyQualifiedTypeName, target_resource: &DscResource,input: &str) -> Result<ExportResult, DscError> {
        let mut configurator = self.clone().create_config_for_adapter(adapter, input)?;
        let mut adapter = Self::get_adapter_resource(&mut configurator, adapter)?;
        if get_adapter_input_kind(&adapter)? == AdapterInputKind::Single {
            adapter.target_resource = Some(Box::new(target_resource.clone()));
            return adapter.export(input);
        }

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
        Ok(export_result)
    }

    fn invoke_schema_with_adapter(&self, adapter: &FullyQualifiedTypeName, target_resource: &DscResource) -> Result<String, DscError> {
        let mut configurator = self.clone().create_config_for_adapter(adapter, "")?;
        let mut adapter = Self::get_adapter_resource(&mut configurator, adapter)?;
        if get_adapter_input_kind(&adapter)? == AdapterInputKind::Single {
            adapter.target_resource = Some(Box::new(target_resource.clone()));
            return adapter.schema();
        }

        return Err(DscError::NotSupported(t!("dscresources.dscresource.invokeSchemaNotSupported", resource = self.type_name).to_string()));
    }

    fn get_adapter_resource(configurator: &mut Configurator, adapter: &FullyQualifiedTypeName) -> Result<DscResource, DscError> {
        if let Some(adapter_resource) = configurator.discovery().find_resource(&DiscoveryFilter::new(adapter, None, None))? {
            return Ok(adapter_resource.clone());
        }
        Err(DscError::Operation(t!("dscresources.dscresource.adapterResourceNotFound", adapter = adapter).to_string()))
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
    fn delete(&self, filter: &str, execution_type: &ExecutionKind) -> Result<DeleteResultKind, DscError>;

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
        if let Some(deprecation_message) = self.deprecation_message.as_ref() {
            warn!("{}", t!("dscresources.dscresource.deprecationMessage", resource = self.type_name, message = deprecation_message));
        }
        if let Some(adapter) = &self.require_adapter {
            return self.invoke_get_with_adapter(adapter, &self, filter);
        }

        match &self.implemented_as {
            Some(ImplementedAs::Command) => {
                command_resource::invoke_get(&self, filter, self.target_resource.as_deref())
            },
            _ => {
                Err(DscError::NotImplemented(t!("dscresources.dscresource.customResourceNotSupported").to_string()))
            },
        }
    }

    fn set(&self, desired: &str, skip_test: bool, execution_type: &ExecutionKind) -> Result<SetResult, DscError> {
        debug!("{}", t!("dscresources.dscresource.invokeSet", resource = self.type_name));
        if let Some(deprecation_message) = self.deprecation_message.as_ref() {
            warn!("{}", t!("dscresources.dscresource.deprecationMessage", resource = self.type_name, message = deprecation_message));
        }
        if let Some(adapter) = &self.require_adapter {
            return self.invoke_set_with_adapter(adapter, &self, desired, skip_test, execution_type);
        }

        match &self.implemented_as {
            Some(ImplementedAs::Command) => {
                command_resource::invoke_set(&self, desired, skip_test, execution_type, self.target_resource.as_deref())
            },
            _ => {
                Err(DscError::NotImplemented(t!("dscresources.dscresource.customResourceNotSupported").to_string()))
            },
        }
    }

    fn test(&self, expected: &str) -> Result<TestResult, DscError> {
        debug!("{}", t!("dscresources.dscresource.invokeTest", resource = self.type_name));
        if let Some(deprecation_message) = self.deprecation_message.as_ref() {
            warn!("{}", t!("dscresources.dscresource.deprecationMessage", resource = self.type_name, message = deprecation_message));
        }
        if let Some(adapter) = &self.require_adapter {
            return self.invoke_test_with_adapter(adapter, &self, expected);
        }

        match &self.implemented_as {
            Some(ImplementedAs::Command) => {
                let Some(manifest) = &self.manifest else {
                    return Err(DscError::MissingManifest(self.type_name.to_string()));
                };

                // if test is not directly implemented, then we need to handle it here
                if manifest.test.is_none() {
                    let get_result = self.get(expected)?;
                    let mut desired_state = serde_json::from_str(expected)?;
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
                    desired_state = redact(&desired_state);
                    let test_result = TestResult::Resource(ResourceTestResponse {
                        desired_state,
                        actual_state,
                        in_desired_state: diff_properties.is_empty(),
                        diff_properties,
                    });
                    Ok(test_result)
                }
                else {
                    command_resource::invoke_test(&self, expected, self.target_resource.as_deref())
                }
            },
            _ => {
                Err(DscError::NotImplemented(t!("dscresources.dscresource.customResourceNotSupported").to_string()))
            },
        }
    }

    fn delete(&self, filter: &str, execution_type: &ExecutionKind) -> Result<DeleteResultKind, DscError> {
        debug!("{}", t!("dscresources.dscresource.invokeDelete", resource = self.type_name));
        if let Some(deprecation_message) = self.deprecation_message.as_ref() {
            warn!("{}", t!("dscresources.dscresource.deprecationMessage", resource = self.type_name, message = deprecation_message));
        }
        if let Some(adapter) = &self.require_adapter {
            return self.invoke_delete_with_adapter(adapter, &self, filter, execution_type);
        }

        match &self.implemented_as {
            Some(ImplementedAs::Command) => {
                command_resource::invoke_delete(&self, filter, self.target_resource.as_deref(), execution_type)
            },
            _ => {
                Err(DscError::NotImplemented(t!("dscresources.dscresource.customResourceNotSupported").to_string()))
            },
        }
    }

    fn validate(&self, config: &str) -> Result<ValidateResult, DscError> {
        debug!("{}", t!("dscresources.dscresource.invokeValidate", resource = self.type_name));
        if let Some(deprecation_message) = self.deprecation_message.as_ref() {
            warn!("{}", t!("dscresources.dscresource.deprecationMessage", resource = self.type_name, message = deprecation_message));
        }
        if self.require_adapter.is_some() {
            return Err(DscError::NotSupported(t!("dscresources.dscresource.invokeValidateNotSupported", resource = self.type_name).to_string()));
        }

        match &self.implemented_as {
            Some(ImplementedAs::Command) => {
                command_resource::invoke_validate(&self, config, self.target_resource.as_deref())
            },
            _ => {
                Err(DscError::NotImplemented(t!("dscresources.dscresource.customResourceNotSupported").to_string()))
            },
        }
    }

    fn schema(&self) -> Result<String, DscError> {
        debug!("{}", t!("dscresources.dscresource.invokeSchema", resource = self.type_name));
        if let Some(deprecation_message) = self.deprecation_message.as_ref() {
            warn!("{}", t!("dscresources.dscresource.deprecationMessage", resource = self.type_name, message = deprecation_message));
        }
        if let Some(schema) = &self.schema {
            return Ok(serde_json::to_string(schema)?);
        }
        if let Some(adapter) = &self.require_adapter {
            return self.invoke_schema_with_adapter(adapter, &self);
        }

        match &self.implemented_as {
            Some(ImplementedAs::Command) => {
                command_resource::get_schema(&self, self.target_resource.as_deref())
            },
            _ => {
                Err(DscError::NotImplemented(t!("dscresources.dscresource.customResourceNotSupported").to_string()))
            },
        }
    }

    fn export(&self, input: &str) -> Result<ExportResult, DscError> {
        debug!("{}", t!("dscresources.dscresource.invokeExport", resource = self.type_name));
        if let Some(deprecation_message) = self.deprecation_message.as_ref() {
            warn!("{}", t!("dscresources.dscresource.deprecationMessage", resource = self.type_name, message = deprecation_message));
        }
        if let Some(adapter) = &self.require_adapter {
            return self.invoke_export_with_adapter(adapter, &self, input);
        }

        command_resource::invoke_export(&self, Some(input), self.target_resource.as_deref())
    }

    fn resolve(&self, input: &str) -> Result<ResolveResult, DscError> {
        debug!("{}", t!("dscresources.dscresource.invokeResolve", resource = self.type_name));
        if let Some(deprecation_message) = self.deprecation_message.as_ref() {
            warn!("{}", t!("dscresources.dscresource.deprecationMessage", resource = self.type_name, message = deprecation_message));
        }
        if self.require_adapter.is_some() {
            return Err(DscError::NotSupported(t!("dscresources.dscresource.invokeResolveNotSupported", resource = self.type_name).to_string()));
        }

        command_resource::invoke_resolve(&self, input)
    }
}

#[must_use]
pub fn get_well_known_properties() -> HashMap<String, Value> {
    HashMap::<String, Value>::from([
        ("_exist".to_string(), Value::Bool(true)),
    ])
}

/// Checks if the JSON value is sensitive and should be redacted
///
/// # Arguments
///
/// * `value` - The JSON value to check
///
/// # Returns
///
/// Original value if not sensitive, otherwise a redacted value
pub fn redact(value: &Value) -> Value {
    if is_secure_value(value) {
        return Value::String(SECURE_VALUE_REDACTED.to_string());
    }

    if let Some(map) = value.as_object() {
        let mut new_map = Map::new();
        for (key, val) in map {
            new_map.insert(key.clone(), redact(val));
        }
        return Value::Object(new_map);
    }

    if let Some(array) = value.as_array() {
        let new_array: Vec<Value> = array.iter().map(redact).collect();
        return Value::Array(new_array);
    }

    value.clone()
}

/// Gets the input kind for an adapter resource
///
/// # Arguments
/// * `adapter` - The adapter resource to get the input kind for
///
/// # Returns
/// * `Result<AdapterInputKind, DscError>` - The input kind of the adapter or an error if not found
///
/// # Errors
/// * `DscError` - The adapter manifest is not found or invalid
pub fn get_adapter_input_kind(adapter: &DscResource) -> Result<AdapterInputKind, DscError> {
    if let Some(manifest) = &adapter.manifest {
        if let Some(adapter_operation) = &manifest.adapter {
            return Ok(adapter_operation.input_kind.clone());
        }
    }
    Err(DscError::Operation(t!("dscresources.dscresource.adapterManifestNotFound", adapter = adapter.type_name).to_string()))
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
            if is_secure_value(value) {
                // skip secure values as they are not comparable
                continue;
            }

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

/// Validates the properties of a resource against its schema.
///
/// # Arguments
///
/// * `properties` - The properties of the resource to validate.
/// * `schema` - The schema to validate against.
///
/// # Returns
///
/// * `Result<(), DscError>` - Ok if valid, Err with message if invalid.
///
/// # Errors
///
/// * `DscError` - Error if the schema is invalid
pub fn validate_properties(resource: &DscResource, properties: &Value) -> Result<(), DscError> {
    // if so, see if it implements validate via the resource manifest
    let type_name = resource.type_name.clone();
    if let Some(schema) = &resource.schema {
        debug!("{}: {type_name} ", t!("dscresources.dscresource.validatingAgainstSchema"));
        let schema = serde_json::to_value(schema)?;
        return validate_json(&resource.type_name, &schema, properties);
    }
    if let Some(manifest) = resource.manifest.clone() {
        if manifest.validate.is_some() {
            debug!("{}: {type_name} ", t!("dscresources.dscresource.resourceImplementsValidate"));
            let resource_config = properties.to_string();
            let result = resource.validate(&resource_config)?;
            if !result.valid {
                let reason = result.reason.unwrap_or(t!("dscresources.dscresource.noReason").to_string());
                return Err(DscError::Validation(format!("{}: {type_name} {reason}", t!("dscresources.dscresource.resourceValidationFailed"))));
            }
            return Ok(())
        }
        // use schema validation
        trace!("{}: {type_name}", t!("dscresources.dscresource.resourceDoesNotImplementValidate"));
        let Ok(schema) = resource.schema() else {
            return Err(DscError::Validation(format!("{}: {type_name}", t!("dscresources.dscresource.noSchemaOrValidate"))));
        };
        let schema = serde_json::from_str(&schema)?;
        return validate_json(&resource.type_name, &schema, properties)
    }
    Err(DscError::Validation(format!("{}: {type_name}", t!("dscresources.dscresource.noManifest"))))
}

/// Validate the JSON against the schema.
///
/// # Arguments
///
/// * `source` - The source of the JSON
/// * `schema` - The schema to validate against
/// * `json` - The JSON to validate
///
/// # Returns
///
/// Nothing on success.
///
/// # Errors
///
/// * `DscError` - The JSON is invalid
pub fn validate_json(source: &str, schema: &Value, json: &Value) -> Result<(), DscError> {
    debug!("{}: {source}", t!("dscresources.dscresource.validatingSchema"));
    trace!("JSON: {json}");
    trace!("Schema: {schema}");
    let compiled_schema = match Validator::new(schema) {
        Ok(compiled_schema) => compiled_schema,
        Err(err) => {
            return Err(DscError::Validation(format!("{}: {err}", t!("dscresources.dscresource.failedToCompileSchema"))));
        }
    };

    if let Err(err) = compiled_schema.validate(json) {
        return Err(DscError::Validation(format!("{}: '{source}' {err}", t!("dscresources.dscresource.validationFailed"))));
    }

    Ok(())
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
