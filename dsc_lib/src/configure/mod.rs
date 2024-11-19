// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::configure::config_doc::{ExecutionKind, Metadata};
use crate::configure::parameters::Input;
use crate::dscerror::DscError;
use crate::dscresources::{
    {dscresource::{Capability, Invoke, get_diff}, invoke_result::{SetResult, ResourceSetResponse}},
    invoke_result::GetResult,
    resource_manifest::Kind,
};
use crate::DscResource;
use crate::discovery::Discovery;
use crate::parser::Statement;
use self::context::Context;
use self::config_doc::{Configuration, DataType, MicrosoftDscMetadata, Operation, SecurityContextKind};
use self::depends_on::get_resource_invocation_order;
use self::config_result::{ConfigurationExportResult, ConfigurationGetResult, ConfigurationSetResult, ConfigurationTestResult};
use self::contraints::{check_length, check_number_limits, check_allowed_values};
use indicatif::ProgressStyle;
use security_context_lib::{SecurityContext, get_security_context};
use serde_json::{Map, Value};
use std::path::PathBuf;
use std::{collections::HashMap, mem};
use tracing::{debug, info, trace, warn_span, Span};
use tracing_indicatif::span_ext::IndicatifSpanExt;
pub mod context;
pub mod config_doc;
pub mod config_result;
pub mod contraints;
pub mod depends_on;
pub mod parameters;

pub struct Configurator {
    json: String,
    config: Configuration,
    pub context: Context,
    discovery: Discovery,
    statement_parser: Statement,
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
pub fn add_resource_export_results_to_configuration(resource: &DscResource, adapter_resource: Option<&DscResource>, conf: &mut Configuration, input: &str) -> Result<(), DscError> {

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

    Ok(())
}

// for values returned by resources, they may look like expressions, so we make sure to escape them in case
// they are re-used to apply configuration
fn escape_property_values(properties: &Map<String, Value>) -> Result<Option<Map<String, Value>>, DscError> {
    debug!("Escape returned property values");
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
                            return Err(DscError::Parser("Nested arrays not supported".to_string()));
                        },
                        Value::String(_) => {
                            // use as_str() so that the enclosing quotes are not included for strings
                            let Some(statement) = element.as_str() else {
                                return Err(DscError::Parser("Array element could not be transformed as string".to_string()));
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
                    return Err(DscError::Parser(format!("Property value '{value}' could not be transformed as string")));
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

fn get_progress_bar_span(len: u64) -> Result<Span, DscError> {
    // use warn_span since that is the default logging level but progress bars will be suppressed if error trace level is used
    let pb_span = warn_span!("");
    pb_span.pb_set_style(&ProgressStyle::with_template(
        "{spinner:.green} [{elapsed_precise:.cyan}] [{bar:40.cyan/blue}] {pos:>7}/{len:7} {msg:.yellow}"
    )?);
    pb_span.pb_set_length(len);
    Ok(pb_span)
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

fn check_security_context(metadata: &Option<Metadata>) -> Result<(), DscError> {
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
                            return Err(DscError::SecurityContext("Elevated security context required".to_string()));
                        }
                    },
                    SecurityContextKind::Restricted => {
                        if get_security_context() != SecurityContext::User {
                            return Err(DscError::SecurityContext("Restricted security context required".to_string()));
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
    pub fn new(json: &str) -> Result<Configurator, DscError> {
        let discovery = Discovery::new()?;
        let mut config = Configurator {
            json: json.to_owned(),
            config: Configuration::new(),
            context: Context::new(),
            discovery,
            statement_parser: Statement::new()?,
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
        let pb_span = get_progress_bar_span(resources.len() as u64)?;
        let pb_span_enter = pb_span.enter();
        for resource in resources {
            Span::current().pb_inc(1);
            pb_span.pb_set_message(format!("Get '{}'", resource.name).as_str());
            let properties = self.invoke_property_expressions(&resource.properties)?;
            let Some(dsc_resource) = self.discovery.find_resource(&resource.resource_type) else {
                return Err(DscError::ResourceNotFound(resource.resource_type));
            };
            debug!("resource_type {}", &resource.resource_type);
            let filter = add_metadata(&dsc_resource.kind, properties)?;
            trace!("filter: {filter}");
            let start_datetime = chrono::Local::now();
            let get_result = dsc_resource.get(&filter)?;
            let end_datetime = chrono::Local::now();
            self.context.outputs.insert(format!("{}:{}", resource.resource_type, resource.name), serde_json::to_value(&get_result)?);
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
                result: get_result,
            };
            result.results.push(resource_result);
        }

        result.metadata = Some(
            self.get_result_metadata(Operation::Get)
        );
        std::mem::drop(pb_span_enter);
        std::mem::drop(pb_span);
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
    pub fn invoke_set(&mut self, skip_test: bool) -> Result<ConfigurationSetResult, DscError> {
        let mut result = ConfigurationSetResult::new();
        let resources = get_resource_invocation_order(&self.config, &mut self.statement_parser, &self.context)?;
        let pb_span = get_progress_bar_span(resources.len() as u64)?;
        let pb_span_enter = pb_span.enter();
        for resource in resources {
            Span::current().pb_inc(1);
            pb_span.pb_set_message(format!("Set '{}'", resource.name).as_str());
            let properties = self.invoke_property_expressions(&resource.properties)?;
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
            trace!("desired: {desired}");

            let start_datetime;
            let end_datetime;
            let set_result;
            if exist || dsc_resource.capabilities.contains(&Capability::SetHandlesExist) {
                debug!("Resource handles _exist or _exist is true");
                start_datetime = chrono::Local::now();
                set_result = dsc_resource.set(&desired, skip_test, &self.context.execution_type)?;
                end_datetime = chrono::Local::now();
            } else if dsc_resource.capabilities.contains(&Capability::Delete) {
                if self.context.execution_type == ExecutionKind::WhatIf {
                    // TODO: add delete what-if support
                    return Err(DscError::NotSupported("What-if execution not supported for delete".to_string()));
                }
                debug!("Resource implements delete and _exist is false");
                let before_result = dsc_resource.get(&desired)?;
                start_datetime = chrono::Local::now();
                dsc_resource.delete(&desired)?;
                let after_result = dsc_resource.get(&desired)?;
                // convert get result to set result
                set_result = match before_result {
                    GetResult::Resource(before_response) => {
                        let GetResult::Resource(after_result) = after_result else {
                            return Err(DscError::NotSupported("Group resources not supported for delete".to_string()))
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
                        return Err(DscError::NotSupported("Group resources not supported for delete".to_string()));
                    },
                };
                end_datetime = chrono::Local::now();
            } else {
                return Err(DscError::NotImplemented(format!("Resource '{}' does not support `delete` and does not handle `_exist` as false", resource.resource_type)));
            }

            self.context.outputs.insert(format!("{}:{}", resource.resource_type, resource.name), serde_json::to_value(&set_result)?);
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
                result: set_result,
            };
            result.results.push(resource_result);
        }

        result.metadata = Some(
            self.get_result_metadata(Operation::Set)
        );
        mem::drop(pb_span_enter);
        mem::drop(pb_span);
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
        let pb_span = get_progress_bar_span(resources.len() as u64)?;
        let pb_span_enter = pb_span.enter();
        for resource in resources {
            Span::current().pb_inc(1);
            pb_span.pb_set_message(format!("Test '{}'", resource.name).as_str());
            let properties = self.invoke_property_expressions(&resource.properties)?;
            let Some(dsc_resource) = self.discovery.find_resource(&resource.resource_type) else {
                return Err(DscError::ResourceNotFound(resource.resource_type));
            };
            debug!("resource_type {}", &resource.resource_type);
            let expected = add_metadata(&dsc_resource.kind, properties)?;
            trace!("expected: {expected}");
            let start_datetime = chrono::Local::now();
            let test_result = dsc_resource.test(&expected)?;
            let end_datetime = chrono::Local::now();
            self.context.outputs.insert(format!("{}:{}", resource.resource_type, resource.name), serde_json::to_value(&test_result)?);
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
                result: test_result,
            };
            result.results.push(resource_result);
        }

        result.metadata = Some(
            self.get_result_metadata(Operation::Test)
        );
        std::mem::drop(pb_span_enter);
        std::mem::drop(pb_span);
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

        let pb_span = get_progress_bar_span(self.config.resources.len() as u64)?;
        let pb_span_enter = pb_span.enter();
        let resources = self.config.resources.clone();
        for resource in &resources {
            Span::current().pb_inc(1);
            pb_span.pb_set_message(format!("Export '{}'", resource.name).as_str());
            let properties = self.invoke_property_expressions(&resource.properties)?;
            let Some(dsc_resource) = self.discovery.find_resource(&resource.resource_type) else {
                return Err(DscError::ResourceNotFound(resource.resource_type.clone()));
            };
            let input = add_metadata(&dsc_resource.kind, properties)?;
            trace!("input: {input}");
            add_resource_export_results_to_configuration(dsc_resource, Some(dsc_resource), &mut conf, input.as_str())?;
        }

        conf.metadata = Some(self.get_result_metadata(Operation::Export));
        result.result = Some(conf);
        std::mem::drop(pb_span_enter);
        std::mem::drop(pb_span);
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
    pub fn set_context(&mut self, parameters_input: &Option<Value>) -> Result<(), DscError> {
        let config = serde_json::from_str::<Configuration>(self.json.as_str())?;
        self.set_parameters(parameters_input, &config)?;
        self.set_variables(&config)?;
        Ok(())
    }

    fn set_parameters(&mut self, parameters_input: &Option<Value>, config: &Configuration) -> Result<(), DscError> {
        // set default parameters first
        let Some(parameters) = &config.parameters else {
            if parameters_input.is_none() {
                info!("No parameters defined in configuration and no parameters input");
                return Ok(());
            }
            return Err(DscError::Validation("No parameters defined in configuration".to_string()));
        };

        for (name, parameter) in parameters {
            debug!("Processing parameter '{name}'");
            if let Some(default_value) = &parameter.default_value {
                debug!("Set default parameter '{name}'");
                // default values can be expressions
                let value = if default_value.is_string() {
                    if let Some(value) = default_value.as_str() {
                        self.statement_parser.parse_and_execute(value, &self.context)?
                    } else {
                        return Err(DscError::Parser("Default value as string is not defined".to_string()));
                    }
                } else {
                    default_value.clone()
                };
                Configurator::validate_parameter_type(name, &value, &parameter.parameter_type)?;
                self.context.parameters.insert(name.clone(), (value, parameter.parameter_type.clone()));
            }
        }

        let Some(parameters_input) = parameters_input else {
            debug!("No parameters input");
            return Ok(());
        };

        trace!("parameters_input: {parameters_input}");
        let parameters: HashMap<String, Value> = serde_json::from_value::<Input>(parameters_input.clone())?.parameters;
        let Some(parameters_constraints) = &config.parameters else {
            return Err(DscError::Validation("No parameters defined in configuration".to_string()));
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
                    info!("Set secure parameter '{name}'");
                } else {
                    info!("Set parameter '{name}' to '{value}'");
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
                return Err(DscError::Validation(format!("Parameter '{name}' not defined in configuration")));
            }
        }
        Ok(())
    }

    fn set_variables(&mut self, config: &Configuration) -> Result<(), DscError> {
        let Some(variables) = &config.variables else {
            debug!("No variables defined in configuration");
            return Ok(());
        };

        for (name, value) in variables {
            let new_value = if let Some(string) = value.as_str() {
                self.statement_parser.parse_and_execute(string, &self.context)?
            }
            else {
                value.clone()
            };
            info!("Set variable '{name}' to '{new_value}'");
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
                    return Err(DscError::Validation(format!("Parameter '{name}' is not a string")));
                }
            },
            DataType::Int => {
                if !value.is_i64() {
                    return Err(DscError::Validation(format!("Parameter '{name}' is not an integer")));
                }
            },
            DataType::Bool => {
                if !value.is_boolean() {
                    return Err(DscError::Validation(format!("Parameter '{name}' is not a boolean")));
                }
            },
            DataType::Array => {
                if !value.is_array() {
                    return Err(DscError::Validation(format!("Parameter '{name}' is not an array")));
                }
            },
            DataType::Object | DataType::SecureObject => {
                if !value.is_object() {
                    return Err(DscError::Validation(format!("Parameter '{name}' is not an object")));
                }
            },
        }

        Ok(())
    }

    fn validate_config(&mut self) -> Result<(), DscError> {
        let config: Configuration = serde_json::from_str(self.json.as_str())?;
        check_security_context(&config.metadata)?;

        // Perform discovery of resources used in config
        let required_resources = config.resources.iter().map(|p| p.resource_type.clone()).collect::<Vec<String>>();
        self.discovery.find_resources(&required_resources);
        self.config = config;
        Ok(())
    }

    fn invoke_property_expressions(&mut self, properties: &Option<Map<String, Value>>) -> Result<Option<Map<String, Value>>, DscError> {
        debug!("Invoke property expressions");
        if properties.is_none() {
            return Ok(None);
        }

        let mut result: Map<String, Value> = Map::new();
        if let Some(properties) = properties {
            for (name, value) in properties {
                trace!("Invoke property expression for {name}: {value}");
                match value {
                    Value::Object(object) => {
                        let value = self.invoke_property_expressions(&Some(object.clone()))?;
                        result.insert(name.clone(), serde_json::to_value(value)?);
                        continue;
                    },
                    Value::Array(array) => {
                        let mut result_array: Vec<Value> = Vec::new();
                        for element in array {
                            match element {
                                Value::Object(object) => {
                                    let value = self.invoke_property_expressions(&Some(object.clone()))?;
                                    result_array.push(serde_json::to_value(value)?);
                                    continue;
                                },
                                Value::Array(_) => {
                                    return Err(DscError::Parser("Nested arrays not supported".to_string()));
                                },
                                Value::String(_) => {
                                    // use as_str() so that the enclosing quotes are not included for strings
                                    let Some(statement) = element.as_str() else {
                                        return Err(DscError::Parser("Array element could not be transformed as string".to_string()));
                                    };
                                    let statement_result = self.statement_parser.parse_and_execute(statement, &self.context)?;
                                    let Some(string_result) = statement_result.as_str() else {
                                        return Err(DscError::Parser("Array element could not be transformed as string".to_string()));
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
                            return Err(DscError::Parser(format!("Property value '{value}' could not be transformed as string")));
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
