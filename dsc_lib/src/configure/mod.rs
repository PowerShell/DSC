// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::configure::config_doc::{ExecutionKind, Metadata};
use crate::configure::parameters::Input;
use crate::dscerror::DscError;
use crate::dscresources::invoke_result::ExportResult;
use crate::dscresources::{
    {dscresource::{Capability, Invoke, get_diff},
    invoke_result::{GetResult, SetResult, TestResult,  ResourceSetResponse}},
    resource_manifest::Kind,
};
use crate::DscResource;
use crate::discovery::Discovery;
use crate::parser::Statement;
use crate::progress::{Failure, ProgressBar, ProgressFormat};
use self::context::Context;
use self::config_doc::{Configuration, DataType, MicrosoftDscMetadata, Operation, SecurityContextKind};
use self::depends_on::get_resource_invocation_order;
use self::config_result::{ConfigurationExportResult, ConfigurationGetResult, ConfigurationSetResult, ConfigurationTestResult};
use self::constraints::{check_length, check_number_limits, check_allowed_values};
use rust_i18n::t;
use security_context_lib::{SecurityContext, get_security_context};
use serde_json::{Map, Value};
use std::path::PathBuf;
use std::collections::HashMap;
use tracing::{debug, info, trace};
pub mod context;
pub mod config_doc;
pub mod config_result;
pub mod constraints;
pub mod depends_on;
pub mod parameters;

pub struct Configurator {
    json: String,
    config: Configuration,
    pub context: Context,
    discovery: Discovery,
    statement_parser: Statement,
    progress_format: ProgressFormat,
}

/// Add the results of an export operation to a configuration.
///
/// # Arguments
///
/// * `resource` - The resource to export.
/// * `conf` - The configuration to add the results to.
///
/// # Panics
///
/// Doesn't panic because there is a match/Some check before `unwrap()`; false positive.
///
/// # Errors
///
/// This function will return an error if the underlying resource fails.
pub fn add_resource_export_results_to_configuration(resource: &DscResource, adapter_resource: Option<&DscResource>, conf: &mut Configuration, input: &str) -> Result<ExportResult, DscError> {

    let export_result = match adapter_resource {
        Some(_) => adapter_resource.unwrap().export(input)?,
        _ => resource.export(input)?
    };

    for (i, instance) in export_result.actual_state.iter().enumerate() {
        let mut r = config_doc::Resource::new();
        r.resource_type.clone_from(&resource.type_name);
        r.name = format!("{}-{i}", r.resource_type);
        let props: Map<String, Value> = serde_json::from_value(instance.clone())?;
        r.properties = escape_property_values(&props)?;

        conf.resources.push(r);
    }

    Ok(export_result)
}

// for values returned by resources, they may look like expressions, so we make sure to escape them in case
// they are re-used to apply configuration
fn escape_property_values(properties: &Map<String, Value>) -> Result<Option<Map<String, Value>>, DscError> {
    debug!("{}", t!("configure.mod.escapePropertyValues"));
    let mut result: Map<String, Value> = Map::new();
    for (name, value) in properties {
        match value {
            Value::Object(object) => {
                let value = escape_property_values(&object.clone())?;
                result.insert(name.clone(), serde_json::to_value(value)?);
                continue;
            },
            Value::Array(array) => {
                let mut result_array: Vec<Value> = Vec::new();
                for element in array {
                    match element {
                        Value::Object(object) => {
                            let value = escape_property_values(&object.clone())?;
                            result_array.push(serde_json::to_value(value)?);
                            continue;
                        },
                        Value::Array(_) => {
                            return Err(DscError::Parser(t!("configure.mod.nestedArraysNotSupported").to_string()));
                        },
                        Value::String(_) => {
                            // use as_str() so that the enclosing quotes are not included for strings
                            let Some(statement) = element.as_str() else {
                                return Err(DscError::Parser(t!("configure.mod.arrayElementCouldNotTransformAsString").to_string()));
                            };
                            if statement.starts_with('[') && statement.ends_with(']') {
                                result_array.push(Value::String(format!("[{statement}")));
                            } else {
                                result_array.push(element.clone());
                            }
                        }
                        _ => {
                            result_array.push(element.clone());
                        }
                    }
                }
                result.insert(name.clone(), serde_json::to_value(result_array)?);
            },
            Value::String(_) => {
                // use as_str() so that the enclosing quotes are not included for strings
                let Some(statement) = value.as_str() else {
                    return Err(DscError::Parser(t!("configure.mod.valueCouldNotBeTransformedAsString", value = value).to_string()));
                };
                if statement.starts_with('[') && statement.ends_with(']') {
                    result.insert(name.clone(), Value::String(format!("[{statement}")));
                } else {
                    result.insert(name.clone(), value.clone());
                }
            },
            _ => {
                result.insert(name.clone(), value.clone());
            },
        }
    }
    Ok(Some(result))
}

fn add_metadata(kind: &Kind, mut properties: Option<Map<String, Value>> ) -> Result<String, DscError> {
    if *kind == Kind::Adapter {
        // add metadata to the properties so the adapter knows this is a config
        let mut metadata = Map::new();
        let mut dsc_value = Map::new();
        dsc_value.insert("context".to_string(), Value::String("configuration".to_string()));
        metadata.insert("Microsoft.DSC".to_string(), Value::Object(dsc_value));
        if let Some(mut properties) = properties {
            properties.insert("metadata".to_string(), Value::Object(metadata));
            return Ok(serde_json::to_string(&properties)?);
        }
        properties = Some(metadata);
        return Ok(serde_json::to_string(&properties)?);
    }

    Ok(serde_json::to_string(&properties)?)
}

fn check_security_context(metadata: Option<&Metadata>) -> Result<(), DscError> {
    if metadata.is_none() {
        return Ok(());
    }

    if let Some(metadata) = &metadata {
        if let Some(microsoft_dsc) = &metadata.microsoft {
            if let Some(required_security_context) = &microsoft_dsc.security_context {
                match required_security_context {
                    SecurityContextKind::Current => {
                        // no check needed
                    },
                    SecurityContextKind::Elevated => {
                        if get_security_context() != SecurityContext::Admin {
                            return Err(DscError::SecurityContext(t!("configure.mod.elevationRequired").to_string()));
                        }
                    },
                    SecurityContextKind::Restricted => {
                        if get_security_context() != SecurityContext::User {
                            return Err(DscError::SecurityContext(t!("configure.mod.restrictedRequired").to_string()));
                        }
                    },
                }
            }
        }
    }

    Ok(())
}

impl Configurator {
    /// Create a new `Configurator` instance.
    ///
    /// # Arguments
    ///
    /// * `config` - The configuration to use in JSON.
    ///
    /// # Errors
    ///
    /// This function will return an error if the configuration is invalid or the underlying discovery fails.
    pub fn new(json: &str, progress_format: ProgressFormat) -> Result<Configurator, DscError> {
        let discovery = Discovery::new()?;
        let mut config = Configurator {
            json: json.to_owned(),
            config: Configuration::new(),
            context: Context::new(),
            discovery,
            statement_parser: Statement::new()?,
            progress_format,
        };
        config.validate_config()?;
        Ok(config)
    }

    /// Get the configuration.
    ///
    /// # Returns
    ///
    /// * `&Configuration` - The configuration.
    #[must_use]
    pub fn get_config(&self) -> &Configuration {
        &self.config
    }

    /// Invoke the get operation on a resource.
    ///
    /// # Returns
    ///
    /// * `ConfigurationGetResult` - The result of the get operation.
    ///
    /// # Errors
    ///
    /// This function will return an error if the underlying resource fails.
    pub fn invoke_get(&mut self) -> Result<ConfigurationGetResult, DscError> {
        let mut result = ConfigurationGetResult::new();
        let resources = get_resource_invocation_order(&self.config, &mut self.statement_parser, &self.context)?;
        let mut progress = ProgressBar::new(resources.len() as u64, self.progress_format)?;
        for resource in resources {
            progress.set_resource(&resource.name, &resource.resource_type);
            progress.write_activity(format!("Get '{}'", resource.name).as_str());
            let properties = self.invoke_property_expressions(resource.properties.as_ref())?;
            let Some(dsc_resource) = self.discovery.find_resource(&resource.resource_type) else {
                return Err(DscError::ResourceNotFound(resource.resource_type));
            };
            debug!("resource_type {}", &resource.resource_type);
            let filter = add_metadata(&dsc_resource.kind, properties)?;
            trace!("filter: {filter}");
            let start_datetime = chrono::Local::now();
            let get_result = match dsc_resource.get(&filter) {
                Ok(result) => result,
                Err(e) => {
                    progress.set_failure(get_failure_from_error(&e));
                    progress.write_increment(1);
                    return Err(e);
                },
            };
            let end_datetime = chrono::Local::now();
            match &get_result {
                GetResult::Resource(resource_result) => {
                    self.context.references.insert(format!("{}:{}", resource.resource_type, resource.name), serde_json::to_value(&resource_result.actual_state)?);
                },
                GetResult::Group(group) => {
                    let mut results = Vec::<Value>::new();
                    for result in group {
                        results.push(serde_json::to_value(&result.result)?);
                    }
                    self.context.references.insert(format!("{}:{}", resource.resource_type, resource.name), Value::Array(results.clone()));
                },
            }
            let resource_result = config_result::ResourceGetResult {
                metadata: Some(
                    Metadata {
                        microsoft: Some(
                            MicrosoftDscMetadata {
                                duration: Some(end_datetime.signed_duration_since(start_datetime).to_string()),
                                ..Default::default()
                            }
                        )
                    }
                ),
                name: resource.name.clone(),
                resource_type: resource.resource_type.clone(),
                result: get_result.clone(),
            };
            result.results.push(resource_result);
            progress.set_result(&serde_json::to_value(get_result)?);
            progress.write_increment(1);
        }

        result.metadata = Some(
            self.get_result_metadata(Operation::Get)
        );
        Ok(result)
    }

    /// Invoke the set operation on a resource.
    ///
    /// # Arguments
    ///
    /// * `skip_test` - Whether to skip the test operation.
    ///
    /// # Returns
    ///
    /// * `ConfigurationSetResult` - The result of the set operation.
    ///
    /// # Errors
    ///
    /// This function will return an error if the underlying resource fails.
    #[allow(clippy::too_many_lines)]
    pub fn invoke_set(&mut self, skip_test: bool) -> Result<ConfigurationSetResult, DscError> {
        let mut result = ConfigurationSetResult::new();
        let resources = get_resource_invocation_order(&self.config, &mut self.statement_parser, &self.context)?;
        let mut progress = ProgressBar::new(resources.len() as u64, self.progress_format)?;
        for resource in resources {
            progress.set_resource(&resource.name, &resource.resource_type);
            progress.write_activity(format!("Set '{}'", resource.name).as_str());
            let properties = self.invoke_property_expressions(resource.properties.as_ref())?;
            let Some(dsc_resource) = self.discovery.find_resource(&resource.resource_type) else {
                return Err(DscError::ResourceNotFound(resource.resource_type));
            };
            debug!("resource_type {}", &resource.resource_type);

            // see if the properties contains `_exist` and is false
            let exist = match &properties {
                Some(property_map) => {
                    if let Some(exist) = property_map.get("_exist") {
                        !matches!(exist, Value::Bool(false))
                    } else {
                        true
                    }
                },
                _ => {
                    true
                }
            };

            let desired = add_metadata(&dsc_resource.kind, properties)?;
            trace!("{}", t!("configure.mod.desired", state = desired));

            let start_datetime;
            let end_datetime;
            let set_result;
            if exist || dsc_resource.capabilities.contains(&Capability::SetHandlesExist) {
                debug!("{}", t!("configure.mod.handlesExist"));
                start_datetime = chrono::Local::now();
                set_result = match dsc_resource.set(&desired, skip_test, &self.context.execution_type) {
                    Ok(result) => result,
                    Err(e) => {
                        progress.set_failure(get_failure_from_error(&e));
                        progress.write_increment(1);
                        return Err(e);
                    },
                };
                end_datetime = chrono::Local::now();
            } else if dsc_resource.capabilities.contains(&Capability::Delete) {
                if self.context.execution_type == ExecutionKind::WhatIf {
                    // TODO: add delete what-if support
                    return Err(DscError::NotSupported(t!("configure.mod.whatIfNotSupportedForDelete").to_string()));
                }
                debug!("{}", t!("configure.mod.implementsDelete"));
                let before_result = match dsc_resource.get(&desired) {
                    Ok(result) => result,
                    Err(e) => {
                        progress.set_failure(get_failure_from_error(&e));
                        progress.write_increment(1);
                        return Err(e);
                    },
                };
                start_datetime = chrono::Local::now();
                if let Err(e) = dsc_resource.delete(&desired) {
                    progress.set_failure(get_failure_from_error(&e));
                    progress.write_increment(1);
                    return Err(e);
                }
                let after_result = match dsc_resource.get(&desired) {
                    Ok(result) => result,
                    Err(e) => {
                        progress.set_failure(get_failure_from_error(&e));
                        progress.write_increment(1);
                        return Err(e);
                    },
                };
                // convert get result to set result
                set_result = match before_result {
                    GetResult::Resource(before_response) => {
                        let GetResult::Resource(after_result) = after_result else {
                            return Err(DscError::NotSupported(t!("configure.mod.groupNotSupportedForDelete").to_string()))
                        };
                        let before_value = serde_json::to_value(&before_response.actual_state)?;
                        let after_value = serde_json::to_value(&after_result.actual_state)?;
                        SetResult::Resource(ResourceSetResponse {
                            before_state: before_response.actual_state,
                            after_state: after_result.actual_state,
                            changed_properties: Some(get_diff(&before_value, &after_value)),
                        })
                    },
                    GetResult::Group(_) => {
                        return Err(DscError::NotSupported(t!("configure.mod.groupNotSupportedForDelete").to_string()))
                    },
                };
                end_datetime = chrono::Local::now();
            } else {
                return Err(DscError::NotImplemented(t!("configure.mod.deleteNotSupported", resource = resource.resource_type).to_string()));
            }

            match &set_result {
                SetResult::Resource(resource_result) => {
                    self.context.references.insert(format!("{}:{}", resource.resource_type, resource.name), serde_json::to_value(&resource_result.after_state)?);
                },
                SetResult::Group(group) => {
                    let mut results = Vec::<Value>::new();
                    for result in group {
                        results.push(serde_json::to_value(&result.result)?);
                    }
                    self.context.references.insert(format!("{}:{}", resource.resource_type, resource.name), Value::Array(results.clone()));
                },
            }
            let resource_result = config_result::ResourceSetResult {
                metadata: Some(
                    Metadata {
                        microsoft: Some(
                            MicrosoftDscMetadata {
                                duration: Some(end_datetime.signed_duration_since(start_datetime).to_string()),
                                ..Default::default()
                            }
                        )
                    }
                ),
                name: resource.name.clone(),
                resource_type: resource.resource_type.clone(),
                result: set_result.clone(),
            };
            result.results.push(resource_result);
            progress.set_result(&serde_json::to_value(set_result)?);
            progress.write_increment(1);
        }

        result.metadata = Some(
            self.get_result_metadata(Operation::Set)
        );
        Ok(result)
    }

    /// Invoke the test operation on a resource.
    ///
    /// # Returns
    ///
    /// * `ConfigurationTestResult` - The result of the test operation.
    ///
    /// # Errors
    ///
    /// This function will return an error if the underlying resource fails.
    pub fn invoke_test(&mut self) -> Result<ConfigurationTestResult, DscError> {
        let mut result = ConfigurationTestResult::new();
        let resources = get_resource_invocation_order(&self.config, &mut self.statement_parser, &self.context)?;
        let mut progress = ProgressBar::new(resources.len() as u64, self.progress_format)?;
        for resource in resources {
            progress.set_resource(&resource.name, &resource.resource_type);
            progress.write_activity(format!("Test '{}'", resource.name).as_str());
            let properties = self.invoke_property_expressions(resource.properties.as_ref())?;
            let Some(dsc_resource) = self.discovery.find_resource(&resource.resource_type) else {
                return Err(DscError::ResourceNotFound(resource.resource_type));
            };
            debug!("resource_type {}", &resource.resource_type);
            let expected = add_metadata(&dsc_resource.kind, properties)?;
            trace!("{}", t!("configure.mod.expectedState", state = expected));
            let start_datetime = chrono::Local::now();
            let test_result = match dsc_resource.test(&expected) {
                Ok(result) => result,
                Err(e) => {
                    progress.set_failure(get_failure_from_error(&e));
                    progress.write_increment(1);
                    return Err(e);
                },
            };
            let end_datetime = chrono::Local::now();
            match &test_result {
                TestResult::Resource(resource_test_result) => {
                    self.context.references.insert(format!("{}:{}", resource.resource_type, resource.name), serde_json::to_value(&resource_test_result.actual_state)?);
                },
                TestResult::Group(group) => {
                    let mut results = Vec::<Value>::new();
                    for result in group {
                        results.push(serde_json::to_value(&result.result)?);
                    }
                    self.context.references.insert(format!("{}:{}", resource.resource_type, resource.name), Value::Array(results.clone()));
                },
            }
            let resource_result = config_result::ResourceTestResult {
                metadata: Some(
                    Metadata {
                        microsoft: Some(
                            MicrosoftDscMetadata {
                                duration: Some(end_datetime.signed_duration_since(start_datetime).to_string()),
                                ..Default::default()
                            }
                        )
                    }
                ),
                name: resource.name.clone(),
                resource_type: resource.resource_type.clone(),
                result: test_result.clone(),
            };
            result.results.push(resource_result);
            progress.set_result( &serde_json::to_value(test_result)?);
            progress.write_increment(1);
        }

        result.metadata = Some(
            self.get_result_metadata(Operation::Test)
        );
        Ok(result)
    }

    /// Invoke the export operation on a configuration.
    ///
    /// # Returns
    ///
    /// * `ConfigurationExportResult` - The result of the export operation.
    ///
    /// # Errors
    ///
    /// This function will return an error if the underlying resource fails.
    pub fn invoke_export(&mut self) -> Result<ConfigurationExportResult, DscError> {
        let mut result = ConfigurationExportResult::new();
        let mut conf = config_doc::Configuration::new();

        let mut progress = ProgressBar::new(self.config.resources.len() as u64, self.progress_format)?;
        let resources = self.config.resources.clone();
        for resource in &resources {
            progress.set_resource(&resource.name, &resource.resource_type);
            progress.write_activity(format!("Export '{}'", resource.name).as_str());
            let properties = self.invoke_property_expressions(resource.properties.as_ref())?;
            let Some(dsc_resource) = self.discovery.find_resource(&resource.resource_type) else {
                return Err(DscError::ResourceNotFound(resource.resource_type.clone()));
            };
            let input = add_metadata(&dsc_resource.kind, properties)?;
            trace!("{}", t!("configure.mod.exportInput", input = input));
            let export_result = match add_resource_export_results_to_configuration(dsc_resource, Some(dsc_resource), &mut conf, input.as_str()) {
                Ok(result) => result,
                Err(e) => {
                    progress.set_failure(get_failure_from_error(&e));
                    progress.write_increment(1);
                    return Err(e);
                },
            };
            self.context.references.insert(format!("{}:{}", resource.resource_type, resource.name), serde_json::to_value(&export_result.actual_state)?);
            progress.set_result(&serde_json::to_value(export_result)?);
            progress.write_increment(1);
        }

        conf.metadata = Some(self.get_result_metadata(Operation::Export));
        result.result = Some(conf);
        Ok(result)
    }

    /// Set the mounted path for the configuration.
    ///
    /// # Arguments
    ///
    /// * `system_root` - The system root to set.
    pub fn set_system_root(&mut self, system_root: &str) {
        self.context.system_root = PathBuf::from(system_root);
    }

    /// Set the parameters and variables context for the configuration.
    ///
    /// # Arguments
    ///
    /// * `parameters_input` - The parameters to set.
    ///
    /// # Errors
    ///
    /// This function will return an error if the parameters are invalid.
    pub fn set_context(&mut self, parameters_input: Option<&Value>) -> Result<(), DscError> {
        let config = serde_json::from_str::<Configuration>(self.json.as_str())?;
        self.set_parameters(parameters_input, &config)?;
        self.set_variables(&config)?;
        Ok(())
    }

    fn set_parameters(&mut self, parameters_input: Option<&Value>, config: &Configuration) -> Result<(), DscError> {
        // set default parameters first
        let Some(parameters) = &config.parameters else {
            if parameters_input.is_none() {
                info!("{}", t!("configure.mod.noParameters"));
                return Ok(());
            }
            return Err(DscError::Validation(t!("configure.mod.noParametersDefined").to_string()));
        };

        for (name, parameter) in parameters {
            debug!("{}", t!("configure.mod.processingParameter", name = name));
            if let Some(default_value) = &parameter.default_value {
                debug!("{}", t!("configure.mod.setDefaultParameter", name = name));
                // default values can be expressions
                let value = if default_value.is_string() {
                    if let Some(value) = default_value.as_str() {
                        self.statement_parser.parse_and_execute(value, &self.context)?
                    } else {
                        return Err(DscError::Parser(t!("configure.mod.defaultStringNotDefined").to_string()));
                    }
                } else {
                    default_value.clone()
                };
                Configurator::validate_parameter_type(name, &value, &parameter.parameter_type)?;
                self.context.parameters.insert(name.clone(), (value, parameter.parameter_type.clone()));
            }
        }

        let Some(parameters_input) = parameters_input else {
            debug!("{}", t!("configure.mod.noParametersInput"));
            return Ok(());
        };

        trace!("parameters_input: {parameters_input}");
        let parameters: HashMap<String, Value> = serde_json::from_value::<Input>(parameters_input.clone())?.parameters;
        let Some(parameters_constraints) = &config.parameters else {
            return Err(DscError::Validation(t!("configure.mod.noParametersDefined").to_string()));
        };
        for (name, value) in parameters {
            if let Some(constraint) = parameters_constraints.get(&name) {
                debug!("Validating parameter '{name}'");
                check_length(&name, &value, constraint)?;
                check_allowed_values(&name, &value, constraint)?;
                check_number_limits(&name, &value, constraint)?;
                // TODO: additional array constraints
                // TODO: object constraints

                Configurator::validate_parameter_type(&name, &value, &constraint.parameter_type)?;
                if constraint.parameter_type == DataType::SecureString || constraint.parameter_type == DataType::SecureObject {
                    info!("{}", t!("configure.mod.setSecureParameter", name = name));
                } else {
                    info!("{}", t!("configure.mod.setParameter", name = name, value = value));
                }

                self.context.parameters.insert(name.clone(), (value.clone(), constraint.parameter_type.clone()));
                // also update the configuration with the parameter value
                if let Some(parameters) = &mut self.config.parameters {
                    if let Some(parameter) = parameters.get_mut(&name) {
                        parameter.default_value = Some(value);
                    }
                }
            }
            else {
                return Err(DscError::Validation(t!("configure.mod.parameterNotDefined", name = name).to_string()));
            }
        }
        Ok(())
    }

    fn set_variables(&mut self, config: &Configuration) -> Result<(), DscError> {
        let Some(variables) = &config.variables else {
            debug!("{}", t!("configure.mod.noVariables"));
            return Ok(());
        };

        for (name, value) in variables {
            let new_value = if let Some(string) = value.as_str() {
                self.statement_parser.parse_and_execute(string, &self.context)?
            }
            else {
                value.clone()
            };
            info!("{}", t!("configure.mod.setVariable", name = name, value = new_value));
            self.context.variables.insert(name.to_string(), new_value);
        }
        Ok(())
    }

    fn get_result_metadata(&self, operation: Operation) -> Metadata {
        let end_datetime = chrono::Local::now();
        Metadata {
            microsoft: Some(
                MicrosoftDscMetadata {
                    context: None,
                    version: Some(env!("CARGO_PKG_VERSION").to_string()),
                    operation: Some(operation),
                    execution_type: Some(self.context.execution_type.clone()),
                    start_datetime: Some(self.context.start_datetime.to_rfc3339()),
                    end_datetime: Some(end_datetime.to_rfc3339()),
                    duration: Some(end_datetime.signed_duration_since(self.context.start_datetime).to_string()),
                    security_context: Some(self.context.security_context.clone()),
                }
            )
        }
    }

    fn validate_parameter_type(name: &str, value: &Value, parameter_type: &DataType) -> Result<(), DscError> {
        match parameter_type {
            DataType::String | DataType::SecureString => {
                if !value.is_string() {
                    return Err(DscError::Validation(t!("configure.mod.parameterNotString", name = name).to_string()));
                }
            },
            DataType::Int => {
                if !value.is_i64() {
                    return Err(DscError::Validation(t!("configure.mod.parameterNotInteger", name = name).to_string()));
                }
            },
            DataType::Bool => {
                if !value.is_boolean() {
                    return Err(DscError::Validation(t!("configure.mod.parameterNotBoolean", name = name).to_string()));
                }
            },
            DataType::Array => {
                if !value.is_array() {
                    return Err(DscError::Validation(t!("configure.mod.parameterNotArray", name = name).to_string()));
                }
            },
            DataType::Object | DataType::SecureObject => {
                if !value.is_object() {
                    return Err(DscError::Validation(t!("configure.mod.parameterNotObject", name = name).to_string()));
                }
            },
        }

        Ok(())
    }

    fn validate_config(&mut self) -> Result<(), DscError> {
        let config: Configuration = serde_json::from_str(self.json.as_str())?;
        check_security_context(config.metadata.as_ref())?;

        // Perform discovery of resources used in config
        let required_resources = config.resources.iter().map(|p| p.resource_type.clone()).collect::<Vec<String>>();
        self.discovery.find_resources(&required_resources, self.progress_format);
        self.config = config;
        Ok(())
    }

    fn invoke_property_expressions(&mut self, properties: Option<&Map<String, Value>>) -> Result<Option<Map<String, Value>>, DscError> {
        debug!("{}", t!("configure.mod.invokePropertyExpressions"));
        if properties.is_none() {
            return Ok(None);
        }

        let mut result: Map<String, Value> = Map::new();
        if let Some(properties) = properties {
            for (name, value) in properties {
                trace!("{}", t!("configure.mod.invokeExpression", name = name, value = value));
                match value {
                    Value::Object(object) => {
                        let value = self.invoke_property_expressions(Some(object))?;
                        result.insert(name.clone(), serde_json::to_value(value)?);
                        continue;
                    },
                    Value::Array(array) => {
                        let mut result_array: Vec<Value> = Vec::new();
                        for element in array {
                            match element {
                                Value::Object(object) => {
                                    let value = self.invoke_property_expressions(Some(object))?;
                                    result_array.push(serde_json::to_value(value)?);
                                    continue;
                                },
                                Value::Array(_) => {
                                    return Err(DscError::Parser(t!("configure.mod.nestedArraysNotSupported").to_string()));
                                },
                                Value::String(_) => {
                                    // use as_str() so that the enclosing quotes are not included for strings
                                    let Some(statement) = element.as_str() else {
                                        return Err(DscError::Parser(t!("configure.mod.arrayElementCouldNotTransformAsString").to_string()));
                                    };
                                    let statement_result = self.statement_parser.parse_and_execute(statement, &self.context)?;
                                    let Some(string_result) = statement_result.as_str() else {
                                        return Err(DscError::Parser(t!("configure.mod.arrayElementCouldNotTransformAsString").to_string()));
                                    };
                                    result_array.push(Value::String(string_result.to_string()));
                                }
                                _ => {
                                    result_array.push(element.clone());
                                }
                            }
                        }
                        result.insert(name.clone(), serde_json::to_value(result_array)?);
                    },
                    Value::String(_) => {
                        // use as_str() so that the enclosing quotes are not included for strings
                        let Some(statement) = value.as_str() else {
                            return Err(DscError::Parser(t!("configure.mod.valueCouldNotBeTransformedAsString", value = value).to_string()));
                        };
                        let statement_result = self.statement_parser.parse_and_execute(statement, &self.context)?;
                        if let Some(string_result) = statement_result.as_str() {
                            result.insert(name.clone(), Value::String(string_result.to_string()));
                        } else {
                            result.insert(name.clone(), statement_result);
                        };
                    },
                    _ => {
                        result.insert(name.clone(), value.clone());
                    },
                }
            }
        }
        Ok(Some(result))
    }
}

fn get_failure_from_error(err: &DscError) -> Option<Failure> {
    match err {
        DscError::CommandExit(_resource, exit_code, reason) => {
            Some(Failure {
                message: reason.to_string(),
                exit_code: *exit_code,
            })
        },
        DscError::CommandExitFromManifest(_resource, exit_code, reason) => {
            Some(Failure {
                message: reason.to_string(),
                exit_code: *exit_code,
            })
        },
        _ => None,
    }
}
