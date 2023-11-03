// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use jsonschema::JSONSchema;

use crate::dscerror::DscError;
use crate::dscresources::dscresource::Invoke;
use crate::DscResource;
use crate::discovery::Discovery;
use crate::parser::Statement;
use self::config_doc::Configuration;
use self::depends_on::get_resource_invocation_order;
use self::config_result::{ConfigurationGetResult, ConfigurationSetResult, ConfigurationTestResult, ConfigurationExportResult, ResourceMessage, MessageLevel};
use serde_json::{Map, Value};
use std::collections::{HashMap, HashSet};
use tracing::debug;

pub mod config_doc;
pub mod config_result;
pub mod depends_on;

pub struct Configurator {
    config: String,
    discovery: Discovery,
    statement_parser: Statement,
}

#[derive(Debug, Clone, Copy)]
pub enum ErrorAction {
    Continue,
    Stop,
}

/// Add the results of an export operation to a configuration.
///
/// # Arguments
///
/// * `resource` - The resource to export.
/// * `conf` - The configuration to add the results to.
///
/// # Errors
///
/// This function will return an error if the underlying resource fails.
pub fn add_resource_export_results_to_configuration(resource: &DscResource, conf: &mut Configuration) -> Result<(), DscError> {
    let export_result = resource.export()?;

    for (i, instance) in export_result.actual_state.iter().enumerate()
    {
        let mut r = config_doc::Resource::new();
        r.resource_type = resource.type_name.clone();
        r.name = format!("{}-{i}", r.resource_type);
        let props: Map<String, Value> = serde_json::from_value(instance.clone())?;
        r.properties = Some(props);

        conf.resources.push(r);
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
    pub fn new(config: &str) -> Result<Configurator, DscError> {
        let discovery = Discovery::new()?;
        Ok(Configurator {
            config: config.to_owned(),
            discovery,
            statement_parser: Statement::new()?,
        })
    }

    /// Invoke the get operation on a resource.
    ///
    /// # Arguments
    ///
    /// * `error_action` - The error action to use.
    /// * `progress_callback` - A callback to call when progress is made.
    ///
    /// # Errors
    ///
    /// This function will return an error if the underlying resource fails.
    pub fn invoke_get(&mut self, _error_action: ErrorAction, _progress_callback: impl Fn() + 'static) -> Result<ConfigurationGetResult, DscError> {
        let (config, messages, had_errors) = self.validate_config()?;
        let mut result = ConfigurationGetResult::new();
        result.messages = messages;
        result.had_errors = had_errors;
        if had_errors {
            return Ok(result);
        }
        for resource in get_resource_invocation_order(&config, &mut self.statement_parser)? {
            let properties = self.invoke_property_expressions(&resource.properties)?;
            let Some(dsc_resource) = self.discovery.find_resource(&resource.resource_type.to_lowercase()) else {
                return Err(DscError::ResourceNotFound(resource.resource_type));
            };
            debug!("resource_type {}", &resource.resource_type);
            let filter = serde_json::to_string(&properties)?;
            let get_result = dsc_resource.get(&filter)?;
            let resource_result = config_result::ResourceGetResult {
                name: resource.name.clone(),
                resource_type: resource.resource_type.clone(),
                result: get_result,
            };
            result.results.push(resource_result);
        }

        Ok(result)
    }

    /// Invoke the set operation on a resource.
    ///
    /// # Arguments
    ///
    /// * `error_action` - The error action to use.
    /// * `progress_callback` - A callback to call when progress is made.
    ///
    /// # Errors
    ///
    /// This function will return an error if the underlying resource fails.
    pub fn invoke_set(&mut self, skip_test: bool, _error_action: ErrorAction, _progress_callback: impl Fn() + 'static) -> Result<ConfigurationSetResult, DscError> {
        let (config, messages, had_errors) = self.validate_config()?;
        let mut result = ConfigurationSetResult::new();
        result.messages = messages;
        result.had_errors = had_errors;
        if had_errors {
            return Ok(result);
        }
        for resource in get_resource_invocation_order(&config, &mut self.statement_parser)? {
            let properties = self.invoke_property_expressions(&resource.properties)?;
            let Some(dsc_resource) = self.discovery.find_resource(&resource.resource_type.to_lowercase()) else {
                return Err(DscError::ResourceNotFound(resource.resource_type));
            };
            debug!("resource_type {}", &resource.resource_type);
            let desired = serde_json::to_string(&properties)?;
            let set_result = dsc_resource.set(&desired, skip_test)?;
            let resource_result = config_result::ResourceSetResult {
                name: resource.name.clone(),
                resource_type: resource.resource_type.clone(),
                result: set_result,
            };
            result.results.push(resource_result);
        }

        Ok(result)
    }

    /// Invoke the test operation on a resource.
    ///
    /// # Arguments
    ///
    /// * `error_action` - The error action to use.
    /// * `progress_callback` - A callback to call when progress is made.
    ///
    /// # Errors
    ///
    /// This function will return an error if the underlying resource fails.
    pub fn invoke_test(&mut self, _error_action: ErrorAction, _progress_callback: impl Fn() + 'static) -> Result<ConfigurationTestResult, DscError> {
        let (config, messages, had_errors) = self.validate_config()?;
        let mut result = ConfigurationTestResult::new();
        result.messages = messages;
        result.had_errors = had_errors;
        if had_errors {
            return Ok(result);
        }
        for resource in get_resource_invocation_order(&config, &mut self.statement_parser)? {
            let properties = self.invoke_property_expressions(&resource.properties)?;
            let Some(dsc_resource) = self.discovery.find_resource(&resource.resource_type.to_lowercase()) else {
                return Err(DscError::ResourceNotFound(resource.resource_type));
            };
            debug!("resource_type {}", &resource.resource_type);
            let expected = serde_json::to_string(&properties)?;
            let test_result = dsc_resource.test(&expected)?;
            let resource_result = config_result::ResourceTestResult {
                name: resource.name.clone(),
                resource_type: resource.resource_type.clone(),
                result: test_result,
            };
            result.results.push(resource_result);
        }

        Ok(result)
    }

    fn find_duplicate_resource_types(config: &Configuration) -> Vec<String>
    {
        let mut map: HashMap<&String, i32> = HashMap::new();
        let mut result: HashSet<String> = HashSet::new();
        let resource_list = &config.resources;
        if resource_list.is_empty() {
            return Vec::new();
        }

        for r in resource_list
        {
            let v = map.entry(&r.resource_type).or_insert(0);
            *v += 1;
            if *v > 1 {
                result.insert(r.resource_type.clone());
            }
        }

        result.into_iter().collect()
    }

    /// Invoke the export operation on a configuration.
    ///
    /// # Arguments
    ///
    /// * `error_action` - The error action to use.
    /// * `progress_callback` - A callback to call when progress is made.
    ///
    /// # Returns
    ///
    /// * `ConfigurationExportResult` - The result of the export operation.
    ///
    /// # Errors
    ///
    /// This function will return an error if the underlying resource fails.
    pub fn invoke_export(&mut self, _error_action: ErrorAction, _progress_callback: impl Fn() + 'static) -> Result<ConfigurationExportResult, DscError> {
        let (config, messages, had_errors) = self.validate_config()?;

        let duplicates = Self::find_duplicate_resource_types(&config);
        if !duplicates.is_empty()
        {
            let duplicates_string = &duplicates.join(",");
            return Err(DscError::Validation(format!("Resource(s) {duplicates_string} specified multiple times")));
        }

        let mut result = ConfigurationExportResult {
            result: None,
            messages,
            had_errors
        };

        if had_errors {
            return Ok(result);
        };
        let mut conf = config_doc::Configuration::new();

        for resource in &config.resources {
            let Some(dsc_resource) = self.discovery.find_resource(&resource.resource_type.to_lowercase()) else {
                return Err(DscError::ResourceNotFound(resource.resource_type.clone()));
            };
            add_resource_export_results_to_configuration(dsc_resource, &mut conf)?;
        }

        result.result = Some(conf);

        Ok(result)
    }

    fn validate_config(&mut self) -> Result<(Configuration, Vec<ResourceMessage>, bool), DscError> {
        let config: Configuration = serde_json::from_str(self.config.as_str())?;
        let mut messages: Vec<ResourceMessage> = Vec::new();
        let mut has_errors = false;

        // Perform discovery of resources used in config
        let mut required_resources = config.resources.iter().map(|p| p.resource_type.to_lowercase()).collect::<Vec<String>>();
        required_resources.sort_unstable();
        required_resources.dedup();
        self.discovery.discover_resources(&required_resources);

        // Now perform the validation
        for resource in &config.resources {
            let Some(dsc_resource) = self.discovery.find_resource(&resource.resource_type.to_lowercase()) else {
                return Err(DscError::ResourceNotFound(resource.resource_type.clone()));
            };

            debug!("resource_type {}", &resource.resource_type);
            //TODO: remove this after schema validation for classic PS resources is implemented
            if resource.resource_type == "DSC/PowerShellGroup" {continue;}

            let input = serde_json::to_string(&resource.properties)?;
            let schema = match dsc_resource.schema() {
                Ok(schema) => schema,
                Err(DscError::SchemaNotAvailable(_) ) => {
                    messages.push(ResourceMessage {
                        name: resource.name.clone(),
                        resource_type: resource.resource_type.clone(),
                        message: "Schema not available".to_string(),
                        level: MessageLevel::Warning,
                    });
                    continue;
                },
                Err(e) => {
                    return Err(e);
                },
            };
            let schema = serde_json::from_str(&schema)?;
            let compiled_schema = match JSONSchema::compile(&schema) {
                Ok(schema) => schema,
                Err(e) => {
                    messages.push(ResourceMessage {
                        name: resource.name.clone(),
                        resource_type: resource.resource_type.clone(),
                        message: format!("Failed to compile schema: {e}"),
                        level: MessageLevel::Error,
                    });
                    has_errors = true;
                    continue;
                },
            };
            let input = serde_json::from_str(&input)?;
            if let Err(err) = compiled_schema.validate(&input) {
                let mut error = format!("Resource '{}' failed validation: ", resource.name);
                for e in err {
                    error.push_str(&format!("\n{e} "));
                }
                messages.push(ResourceMessage {
                    name: resource.name.clone(),
                    resource_type: resource.resource_type.clone(),
                    message: error,
                    level: MessageLevel::Error,
                });
                has_errors = true;
                continue;
            };
        }

        Ok((config, messages, has_errors))
    }

    fn invoke_property_expressions(&mut self, properties: &Option<Map<String, Value>>) -> Result<Option<Map<String, Value>>, DscError> {
        if properties.is_none() {
            return Ok(None);
        }

        let mut result: Map<String, Value> = Map::new();
        if let Some(properties) = properties {
            for (name, value) in properties {
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
                                _ => {
                                    let value = self.statement_parser.parse_and_execute(&element.to_string())?;
                                    result_array.push(serde_json::from_str(&value)?);
                                }
                            }
                        }
                        result.insert(name.clone(), serde_json::to_value(result_array)?);
                    },
                    _ => {
                        let value = self.statement_parser.parse_and_execute(&value.to_string())?;
                        result.insert(name.clone(), serde_json::from_str(&value)?);
                    },
                }
            }
        }
        Ok(Some(result))
    }
}
