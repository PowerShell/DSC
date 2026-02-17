// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::configure::config_doc::{ConfigDirective, ExecutionInformation, ResourceDirective};
use crate::configure::context::{Context, ProcessMode};
use crate::configure::parameters::import_parameters;
use crate::configure::{config_doc::{ExecutionKind, IntOrExpression, Metadata, Parameter, Resource, ResourceDiscoveryMode, RestartRequired, ValueOrCopy}};
use crate::discovery::discovery_trait::DiscoveryFilter;
use crate::dscerror::DscError;
use crate::dscresources::{
    {dscresource::{Capability, Invoke, get_diff, validate_properties, get_adapter_input_kind},
    invoke_result::{DeleteResult, DeleteResultKind, GetResult, SetResult, TestResult, ExportResult, ResourceSetResponse}},
    resource_manifest::{AdapterInputKind, Kind},
};
use crate::DscResource;
use crate::discovery::Discovery;
use crate::parser::Statement;
use crate::progress::{Failure, ProgressBar, ProgressFormat};
use crate::types::{SemanticVersion, SemanticVersionReq};
use crate::util::resource_id;
use self::config_doc::{Configuration, DataType, MicrosoftDscMetadata, Operation, SecurityContextKind};
use self::depends_on::get_resource_invocation_order;
use self::config_result::{ConfigurationExportResult, ConfigurationGetResult, ConfigurationSetResult, ConfigurationTestResult};
use self::constraints::{check_length, check_number_limits, check_allowed_values};
use rust_i18n::t;
use dsc_lib_security_context::{SecurityContext, get_security_context};
use serde_json::{Map, Value};
use std::path::PathBuf;
use std::collections::HashMap;
use tracing::{debug, info, trace, warn};
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
pub fn add_resource_export_results_to_configuration(resource: &DscResource, conf: &mut Configuration, input: &str) -> Result<ExportResult, DscError> {

    let export_result = resource.export(input)?;

    if resource.kind == Kind::Exporter {
        for instance in &export_result.actual_state {
            let resource = serde_json::from_value::<Resource>(instance.clone())?;
            conf.resources.push(resource);
        }
    } else {
        for (i, instance) in export_result.actual_state.iter().enumerate() {
            let mut r: Resource = config_doc::Resource::new();
            r.resource_type.clone_from(&resource.type_name);
            let mut props: Map<String, Value> = serde_json::from_value(instance.clone())?;
            if let Some(kind) = props.remove("_kind") {
                if !kind.is_string() {
                    return Err(DscError::Parser(t!("configure.mod.propertyNotString", name = "_kind", value = kind).to_string()));
                }
                r.kind = kind.as_str().map(std::string::ToString::to_string);
            }
            r.name = if let Some(name) = props.remove("_name") {
                name.as_str()
                    .map(std::string::ToString::to_string)
                    .ok_or_else(|| DscError::Parser(t!("configure.mod.propertyNotString", name = "_name", value = name).to_string()))?
            } else {
                let resource_type_short = if let Some(pos) = resource.type_name.find('/') {
                    &resource.type_name[pos + 1..]
                } else {
                    &resource.type_name
                };
                format!("{resource_type_short}-{i}")
            };
            let mut metadata = Metadata {
                microsoft: None,
                other: Map::new(),
            };
            if let Some(security_context) = props.remove("_securityContext") {
                let security_context: SecurityContextKind = serde_json::from_value(security_context)?;
                metadata.microsoft = Some(
                        MicrosoftDscMetadata {
                            security_context: Some(security_context),
                            ..Default::default()
                        }
                );
            }
            r.properties = escape_property_values(&props)?;
            let mut properties = serde_json::to_value(&r.properties)?;
            let mut execution_information = ExecutionInformation::new();
            get_metadata_from_result(None, &mut properties, &mut metadata, &mut execution_information)?;
            r.properties = Some(properties.as_object().cloned().unwrap_or_default());
            r.metadata = if metadata.microsoft.is_some() || !metadata.other.is_empty() {
                Some(metadata)
            } else {
                None
            };
            r.execution_information = Some(execution_information);

            conf.resources.push(r);
        }
    }

    Ok(export_result)
}

// for values returned by resources, they may look like expressions, so we make sure to escape them in case
// they are re-used to apply configuration
fn escape_property_values(properties: &Map<String, Value>) -> Result<Option<Map<String, Value>>, DscError> {
    let mut result: Map<String, Value> = Map::new();
    for (name, value) in properties {
        match value {
            Value::Object(object) => {
                let value = escape_property_values(&object.clone())?;
                result.insert(name.clone(), serde_json::to_value(value)?);
            },
            Value::Array(array) => {
                let mut result_array: Vec<Value> = Vec::new();
                for element in array {
                    match element {
                        Value::Object(object) => {
                            let value = escape_property_values(&object.clone())?;
                            result_array.push(serde_json::to_value(value)?);
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

fn add_metadata(dsc_resource: &DscResource, mut properties: Option<Map<String, Value>>, resource_metadata: Option<Metadata> ) -> Result<String, DscError> {
    if dsc_resource.kind == Kind::Adapter && get_adapter_input_kind(dsc_resource)? == AdapterInputKind::Full {
        // add metadata to the properties so the adapter knows this is a config
        let mut metadata: Map<String, Value> = Map::new();
        if let Some(resource_metadata) = resource_metadata {
            if !resource_metadata.other.is_empty() {
                metadata.extend(resource_metadata.other);
            }
        }
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

    if let Some(resource_metadata) = resource_metadata {
        let other_metadata = resource_metadata.other;
        let mut props = if let Some(props) = properties {
            props
        } else {
            Map::new()
        };
        props.insert("_metadata".to_string(), Value::Object(other_metadata));
        let modified_props = Value::from(props.clone());
        if let Ok(()) = validate_properties(dsc_resource, &modified_props) {} else {
            info!("{}", t!("configure.mod.schemaExcludesMetadata"));
            props.remove("_metadata");
        }
        return Ok(serde_json::to_string(&props)?);
    }

    match properties {
        Some(properties) => {
            Ok(serde_json::to_string(&properties)?)
        },
        _ => {
            Ok(String::new())
        }
    }
}

fn get_require_adapter_from_directive(resource_directives: &Option<ResourceDirective>) -> Option<String> {
    if let Some(directives) = resource_directives {
        if let Some(require_adapter) = &directives.require_adapter {
            return Some(require_adapter.clone());
        }
    }
    None
}

fn check_security_context(metadata: Option<&Metadata>, directives: Option<&ConfigDirective>) -> Result<(), DscError> {
    if metadata.is_none() && directives.is_none() {
        return Ok(());
    }

    let mut security_context_required: Option<&SecurityContextKind> = None;
    if let Some(metadata) = &metadata {
        if let Some(microsoft_dsc) = &metadata.microsoft {
            if let Some(required_security_context) = &microsoft_dsc.security_context {
                warn!("{}", t!("configure.mod.securityContextInMetadataDeprecated"));
                security_context_required = Some(required_security_context);
            }
        }
    }

    if let Some(directives) = directives {
        if let Some(directive_security_context) = &directives.security_context {
            if security_context_required.is_some() && security_context_required != Some(directive_security_context) {
                return Err(DscError::SecurityContext(t!("configure.mod.conflictingSecurityContext", metadata = security_context_required.unwrap(), directive = directive_security_context).to_string()));
            }
            security_context_required = Some(directive_security_context);
        }
    }

    match security_context_required {
        Some(SecurityContextKind::Elevated) => {
            if get_security_context() != SecurityContext::Admin {
                return Err(DscError::SecurityContext(t!("configure.mod.elevationRequired").to_string()));
            }
        },
        Some(SecurityContextKind::Restricted) => {
            if get_security_context() != SecurityContext::User {
                return Err(DscError::SecurityContext(t!("configure.mod.restrictedRequired").to_string()));
            }
        },
        None | Some(SecurityContextKind::Current) => {
            // no check needed
        },
    }

    Ok(())
}

fn get_metadata_from_result(mut context: Option<&mut Context>, result: &mut Value, metadata: &mut Metadata, execution_information: &mut ExecutionInformation) -> Result<(), DscError> {
    if let Some(metadata_value) = result.get("_metadata") {
        if let Some(metadata_map) = metadata_value.as_object() {
            for (key, value) in metadata_map {
                if key.starts_with("Microsoft.DSC") {
                    warn!("{}", t!("configure.mod.metadataMicrosoftDscIgnored", key = key));
                    continue;
                }
                if let Some(ref mut context) = context {
                    if key == "_restartRequired" {
                        if let Ok(restart_required) = serde_json::from_value::<Vec<RestartRequired>>(value.clone()) {
                            execution_information.restart_required = Some(restart_required.clone());
                            context.restart_required.get_or_insert_with(Vec::new).extend(restart_required);
                        } else {
                            warn!("{}", t!("configure.mod.metadataRestartRequiredInvalid", value = value));
                            continue;
                        }
                    }
                }
                metadata.other.insert(key.clone(), value.clone());
            }
        } else {
            return Err(DscError::Parser(t!("configure.mod.metadataNotObject", value = metadata_value).to_string()));
        }
        if let Some(value_map) = result.as_object_mut() {
            value_map.remove("_metadata");
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
        let discovery = Discovery::new();
        let mut config = Configurator {
            json: json.to_owned(),
            config: Configuration::new(),
            context: Context::new(),
            discovery: discovery.clone(),
            statement_parser: Statement::new()?,
            progress_format,
        };
        config.validate_config()?;
        for extension in discovery.extensions.values() {
            config.context.extensions.push(extension.clone());
        }
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

    /// Get the discovery.
    ///
    /// # Returns
    ///
    /// * `&Discovery` - The discovery.
    pub fn discovery(&mut self) -> &mut Discovery {
        &mut self.discovery
    }

    fn get_properties(&mut self, resource: &Resource, resource_kind: &Kind) -> Result<Option<Map<String, Value>>, DscError> {
        // Restore copy loop context from resource metadata under Microsoft.DSC/copyLoops if present
        if let Some(metadata) = &resource.metadata {
            if let Some(microsoft) = &metadata.microsoft {
                if let Some(copy_loops) = &microsoft.copy_loops {
                    for (loop_name, value) in copy_loops {
                        if let Some(index) = value.as_i64() {
                            self.context.copy.insert(loop_name.to_string(), index);
                            self.context.copy_current_loop_name.clone_from(loop_name);
                        }
                    }
                }
            }
        }

        let result = match resource_kind {
            Kind::Group => {
                // if Group resource, we leave it to the resource to handle expressions
                Ok(resource.properties.clone())
            },
            _ => {
                Ok(invoke_property_expressions(&mut self.statement_parser, &self.context, resource.properties.as_ref())?)
            },
        };

        // Clear copy loop context after processing resource
        self.context.copy.clear();
        self.context.copy_current_loop_name.clear();

        result
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
        let resources = get_resource_invocation_order(&self.config, &mut self.statement_parser, &mut self.context)?;
        let mut progress = ProgressBar::new(resources.len() as u64, self.progress_format)?;
        let discovery = &mut self.discovery.clone();
        for resource in resources {
            let evaluated_name = self.evaluate_resource_name(&resource.name)?;

            progress.set_resource(&evaluated_name, &resource.resource_type);
            progress.write_activity(format!("Get '{evaluated_name}'").as_str());
            if self.skip_resource(&resource)? {
                progress.write_increment(1);
                continue;
            }
            let adapter = get_require_adapter_from_directive(&resource.directives);
            let Some(dsc_resource) = discovery.find_resource(&DiscoveryFilter::new(&resource.resource_type, resource.require_version.as_deref(), adapter.as_deref()))? else {
                return Err(DscError::ResourceNotFound(resource.resource_type.to_string(), resource.require_version.as_deref().unwrap_or("").to_string()));
            };
            let properties = self.get_properties(&resource, &dsc_resource.kind)?;
            let filter = add_metadata(&dsc_resource, properties, resource.metadata.clone())?;
            let start_datetime = chrono::Local::now();
            let mut get_result = match dsc_resource.get(&filter) {
                Ok(result) => result,
                Err(e) => {
                    progress.set_failure(get_failure_from_error(&e));
                    progress.write_increment(1);
                    return Err(e);
                },
            };
            let end_datetime = chrono::Local::now();
            let mut execution_information = ExecutionInformation::new_with_duration(&start_datetime, &end_datetime);
            let mut metadata = Metadata {
                microsoft: Some(
                    MicrosoftDscMetadata::new_with_duration(&start_datetime, &end_datetime)
                ),
                other: Map::new(),
            };

            match &mut get_result {
                GetResult::Resource(ref mut resource_result) => {
                    self.context.references.insert(resource_id(&resource.resource_type, &evaluated_name), serde_json::to_value(&resource_result.actual_state)?);
                    get_metadata_from_result(Some(&mut self.context), &mut resource_result.actual_state, &mut metadata, &mut execution_information)?;
                },
                GetResult::Group(group) => {
                    let mut results = Vec::<Value>::new();
                    for result in group {
                        results.push(serde_json::to_value(&result.result)?);
                    }
                    self.context.references.insert(resource_id(&resource.resource_type, &evaluated_name), Value::Array(results.clone()));
                },
            }
            let resource_result = config_result::ResourceGetResult {
                execution_information: Some(execution_information),
                metadata: Some(metadata),
                name: evaluated_name,
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
        let mut execution_information = ExecutionInformation::new();
        self.get_execution_information(Operation::Get, &mut execution_information);
        result.execution_information = Some(execution_information);
        self.process_output()?;
        if !self.context.outputs.is_empty() {
            result.outputs = Some(self.context.outputs.clone());
        }
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
        let resources = get_resource_invocation_order(&self.config, &mut self.statement_parser, &mut self.context)?;
        let mut progress = ProgressBar::new(resources.len() as u64, self.progress_format)?;
        let discovery = &mut self.discovery.clone();
        for resource in resources {
            let evaluated_name = self.evaluate_resource_name(&resource.name)?;

            progress.set_resource(&evaluated_name, &resource.resource_type);
            progress.write_activity(format!("Set '{evaluated_name}'").as_str());
            if self.skip_resource(&resource)? {
                progress.write_increment(1);
                continue;
            }
            let adapter = get_require_adapter_from_directive(&resource.directives);
            let Some(dsc_resource) = discovery.find_resource(&DiscoveryFilter::new(&resource.resource_type, resource.require_version.as_deref(), adapter.as_deref()))? else {
                return Err(DscError::ResourceNotFound(resource.resource_type.to_string(), resource.require_version.as_deref().unwrap_or("").to_string()));
            };
            let properties = self.get_properties(&resource, &dsc_resource.kind)?;
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

            let desired = add_metadata(&dsc_resource, properties, resource.metadata.clone())?;
            trace!("{}", t!("configure.mod.desired", state = desired));

            let start_datetime;
            let end_datetime;
            let mut set_result;
            let mut delete_what_if_metadata: Option<DeleteResult> = None;
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
                let delete_result = match dsc_resource.delete(&desired, &self.context.execution_type) {
                    Ok(result) => result,
                    Err(e) => {
                        progress.set_failure(get_failure_from_error(&e));
                        progress.write_increment(1);
                        return Err(e);
                    },
                };

                match delete_result {
                    DeleteResultKind::SyntheticWhatIf(test_result) => {
                        end_datetime = chrono::Local::now();
                        set_result = test_result.into();
                    },
                    _ => {
                        if let DeleteResultKind::ResourceWhatIf(delete_res) = delete_result {
                            delete_what_if_metadata = Some(delete_res);
                        }

                        let after_result = match dsc_resource.get(&desired) {
                            Ok(result) => result,
                            Err(e) => {
                                progress.set_failure(get_failure_from_error(&e));
                                progress.write_increment(1);
                                return Err(e);
                            },
                        };
                        end_datetime = chrono::Local::now();

                        set_result = match before_result {
                            GetResult::Resource(before_response) => {
                                let GetResult::Resource(after_result) = after_result else {
                                    return Err(DscError::NotSupported(t!("configure.mod.groupNotSupportedForDelete").to_string()))
                                };
                                let diff = get_diff(&before_response.actual_state, &after_result.actual_state);
                                let mut before: Map<String, Value> = serde_json::from_value(before_response.actual_state)?;
                                if before.contains_key("result") && !before.contains_key("resources") {
                                    before.insert("resources".to_string(), before["result"].clone());
                                    before.remove("result");
                                }
                                let before_value = serde_json::to_value(&before)?;
                                SetResult::Resource(ResourceSetResponse {
                                    before_state: before_value.clone(),
                                    after_state: after_result.actual_state,
                                    changed_properties: Some(diff),
                                })
                            },
                            GetResult::Group(_) => {
                                return Err(DscError::NotSupported(t!("configure.mod.groupNotSupportedForDelete").to_string()))
                            },
                        };
                    },
                }
            } else {
                return Err(DscError::NotImplemented(t!("configure.mod.deleteNotSupported", resource = resource.resource_type).to_string()));
            }

            // Process metadata - only add whatIf if we have ResourceWhatIf variant
            let mut execution_information = ExecutionInformation::new_with_duration(&start_datetime, &end_datetime);
            let mut other_metadata = Map::new();
            if self.context.execution_type == ExecutionKind::WhatIf {
                if let Some(delete_res) = delete_what_if_metadata {
                    if let Some(metadata) = delete_res.metadata {
                        if let Some(what_if) = metadata.what_if {
                            execution_information.what_if = Some(what_if.clone());
                            other_metadata.insert("whatIf".to_string(), what_if);
                        }
                    }
                }
            }

            let mut metadata = Metadata {
                microsoft: Some(
                    MicrosoftDscMetadata::new_with_duration(&start_datetime, &end_datetime)
                ),
                other: other_metadata,
            };
            match &mut set_result {
                SetResult::Resource(resource_result) => {
                    self.context.references.insert(resource_id(&resource.resource_type, &evaluated_name), serde_json::to_value(&resource_result.after_state)?);
                    get_metadata_from_result(Some(&mut self.context), &mut resource_result.after_state, &mut metadata, &mut execution_information)?;
                },
                SetResult::Group(group) => {
                    let mut results = Vec::<Value>::new();
                    for result in group {
                        results.push(serde_json::to_value(&result.result)?);
                    }
                    self.context.references.insert(resource_id(&resource.resource_type, &evaluated_name), Value::Array(results.clone()));
                },
            }
            let resource_result = config_result::ResourceSetResult {
                execution_information: Some(execution_information),
                metadata: Some(metadata),
                name: evaluated_name,
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
        let mut execution_information = ExecutionInformation::new();
        self.get_execution_information(Operation::Set, &mut execution_information);
        result.execution_information = Some(execution_information);
        self.process_output()?;
        if !self.context.outputs.is_empty() {
            result.outputs = Some(self.context.outputs.clone());
        }
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
        let resources = get_resource_invocation_order(&self.config, &mut self.statement_parser, &mut self.context)?;
        let mut progress = ProgressBar::new(resources.len() as u64, self.progress_format)?;
        let discovery = &mut self.discovery.clone();
        for resource in resources {
            let evaluated_name = self.evaluate_resource_name(&resource.name)?;

            progress.set_resource(&evaluated_name, &resource.resource_type);
            progress.write_activity(format!("Test '{evaluated_name}'").as_str());
            if self.skip_resource(&resource)? {
                progress.write_increment(1);
                continue;
            }
            let adapter = get_require_adapter_from_directive(&resource.directives);
            let Some(dsc_resource) = discovery.find_resource(&DiscoveryFilter::new(&resource.resource_type, resource.require_version.as_deref(), adapter.as_deref()))? else {
                return Err(DscError::ResourceNotFound(resource.resource_type.to_string(), resource.require_version.as_deref().unwrap_or("").to_string()));
            };
            let properties = self.get_properties(&resource, &dsc_resource.kind)?;
            debug!("resource_type {}", &resource.resource_type);
            let expected = add_metadata(&dsc_resource, properties, resource.metadata.clone())?;
            trace!("{}", t!("configure.mod.expectedState", state = expected));
            let start_datetime = chrono::Local::now();
            let mut test_result = match dsc_resource.test(&expected) {
                Ok(result) => result,
                Err(e) => {
                    progress.set_failure(get_failure_from_error(&e));
                    progress.write_increment(1);
                    return Err(e);
                },
            };
            let end_datetime = chrono::Local::now();
            let mut execution_information = ExecutionInformation::new_with_duration(&start_datetime, &end_datetime);
            let mut metadata = Metadata {
                microsoft: Some(
                    MicrosoftDscMetadata::new_with_duration(&start_datetime, &end_datetime)
                ),
                other: Map::new(),
            };
            match &mut test_result {
                TestResult::Resource(resource_test_result) => {
                    self.context.references.insert(resource_id(&resource.resource_type, &evaluated_name), serde_json::to_value(&resource_test_result.actual_state)?);
                    get_metadata_from_result(Some(&mut self.context), &mut resource_test_result.actual_state, &mut metadata, &mut execution_information)?;
                },
                TestResult::Group(group) => {
                    let mut results = Vec::<Value>::new();
                    for result in group {
                        results.push(serde_json::to_value(&result.result)?);
                    }
                    self.context.references.insert(resource_id(&resource.resource_type, &evaluated_name), Value::Array(results.clone()));
                },
            }
            let resource_result = config_result::ResourceTestResult {
                execution_information: Some(execution_information),
                metadata: Some(metadata),
                name: evaluated_name,
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
        let mut execution_information = ExecutionInformation::new();
        self.get_execution_information(Operation::Test, &mut execution_information);
        result.execution_information = Some(execution_information);
        self.process_output()?;
        if !self.context.outputs.is_empty() {
            result.outputs = Some(self.context.outputs.clone());
        }
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
        conf.metadata.clone_from(&self.config.metadata);

        let mut progress = ProgressBar::new(self.config.resources.len() as u64, self.progress_format)?;
        let resources = self.config.resources.clone();
        let discovery = &mut self.discovery.clone();
        for resource in &resources {
            let evaluated_name = self.evaluate_resource_name(&resource.name)?;

            progress.set_resource(&evaluated_name, &resource.resource_type);
            progress.write_activity(format!("Export '{evaluated_name}'").as_str());
            if self.skip_resource(resource)? {
                progress.write_increment(1);
                continue;
            }
            let adapter = get_require_adapter_from_directive(&resource.directives);
            let Some(dsc_resource) = discovery.find_resource(&DiscoveryFilter::new(&resource.resource_type, resource.require_version.as_deref(), adapter.as_deref()))? else {
                return Err(DscError::ResourceNotFound(resource.resource_type.to_string(), resource.require_version.as_deref().unwrap_or("").to_string()));
            };
            let properties = self.get_properties(resource, &dsc_resource.kind)?;
            debug!("resource_type {}", &resource.resource_type);
            let input = add_metadata(&dsc_resource, properties, resource.metadata.clone())?;
            trace!("{}", t!("configure.mod.exportInput", input = input));
            let export_result = match add_resource_export_results_to_configuration(&dsc_resource, &mut conf, input.as_str()) {
                Ok(result) => result,
                Err(e) => {
                    progress.set_failure(get_failure_from_error(&e));
                    progress.write_increment(1);
                    return Err(e);
                },
            };
            self.context.references.insert(resource_id(&resource.resource_type, &evaluated_name), serde_json::to_value(&export_result.actual_state)?);
            progress.set_result(&serde_json::to_value(export_result)?);
            progress.write_increment(1);
        }

        let export_metadata = self.get_result_metadata(Operation::Export);
        match conf.metadata {
            Some(mut metadata) => {
                metadata.microsoft = export_metadata.microsoft;
                conf.metadata = Some(metadata);
            },
            _ => {
                conf.metadata = Some(export_metadata);
            },
        }

        result.result = Some(conf);
        self.process_output()?;
        if !self.context.outputs.is_empty() {
            result.outputs = Some(self.context.outputs.clone());
        }
        Ok(result)
    }

    fn skip_resource(&mut self, resource: &Resource) -> Result<bool, DscError> {
        if let Some(condition) = &resource.condition {
            let condition_result = self.statement_parser.parse_and_execute(condition, &self.context)?;
            if condition_result != Value::Bool(true) {
                info!("{}", t!("configure.config_doc.skippingResource", name = resource.name, condition = condition, result = condition_result));
                return Ok(true);
            }
        }
        Ok(false)
    }

    /// Process the outputs defined in the configuration.
    ///
    /// # Errors
    ///
    /// This function will return an error if the output processing fails.
    pub fn process_output(&mut self) -> Result<(), DscError> {
        if self.config.outputs.is_none() || self.context.execution_type == ExecutionKind::WhatIf {
            return Ok(());
        }
        if let Some(outputs) = &self.config.outputs {
            for (name, output) in outputs {
                if let Some(condition) = &output.condition {
                    let condition_result = self.statement_parser.parse_and_execute(condition, &self.context)?;
                    if condition_result != Value::Bool(true) {
                        info!("{}", t!("configure.mod.skippingOutput", name = name));
                        continue;
                    }
                }

                if let ValueOrCopy::Value(value) = &output.value_or_copy {
                    let value_result = self.statement_parser.parse_and_execute(value, &self.context)?;
                    if output.r#type == DataType::SecureString || output.r#type == DataType::SecureObject {
                        warn!("{}", t!("configure.mod.secureOutputSkipped", name = name));
                        continue;
                    }
                    // TODO: handle nullable when supported
                    if value_result.is_string() && output.r#type != DataType::String ||
                        value_result.is_i64() && output.r#type != DataType::Int ||
                        value_result.is_boolean() && output.r#type != DataType::Bool ||
                        value_result.is_array() && output.r#type != DataType::Array ||
                        value_result.is_object() && output.r#type != DataType::Object {
                            return Err(DscError::Validation(t!("configure.mod.outputTypeNotMatch", name = name, expected_type = output.r#type).to_string()));
                    }
                    self.context.outputs.insert(name.clone(), value_result);
                } else {
                    warn!("{}", t!("configure.mod.copyNotSupported", name = name));
                }
            }
        }
        Ok(())
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

        self.context.extensions = self.discovery.extensions.values().cloned().collect();
        self.set_parameters(parameters_input, &config)?;
        self.set_variables(&config)?;
        self.set_user_functions(&config)?;
        Ok(())
    }

    fn set_parameters(&mut self, parameters_input: Option<&Value>, config: &Configuration) -> Result<(), DscError> {
        let Some(parameters) = &config.parameters else {
            if parameters_input.is_none() {
                info!("{}", t!("configure.mod.noParameters"));
                return Ok(());
            }
            return Err(DscError::Validation(t!("configure.mod.noParametersDefined").to_string()));
        };

        // process input parameters first
        if let Some(parameters_input) = parameters_input {
            let input_parameters: HashMap<String, Value> = import_parameters(parameters_input)?;
            for (name, value) in input_parameters {
                if let Some(constraint) = parameters.get(&name) {
                    debug!("Validating parameter '{name}'");
                    check_length(&name, &value, constraint)?;
                    check_allowed_values(&name, &value, constraint)?;
                    check_number_limits(&name, &value, constraint)?;
                    // TODO: additional array constraints
                    // TODO: object constraints

                    validate_parameter_type(&name, &value, &constraint.parameter_type)?;
                    if constraint.parameter_type == DataType::SecureString || constraint.parameter_type == DataType::SecureObject {
                        info!("{}", t!("configure.mod.setSecureParameter", name = name));
                    } else {
                        info!("{}", t!("configure.mod.setParameter", name = name, value = value));
                    }

                    self.context.parameters.insert(name.clone(), (value.clone(), constraint.parameter_type.clone()));
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
        }

        // Now process default values for parameters that weren't provided in input
        let mut unresolved_parameters: HashMap<String, &Parameter> = parameters
            .iter()
            .filter(|(name, _)| !self.context.parameters.contains_key(*name))
            .map(|(k, v)| (k.clone(), v))
            .collect();

        while !unresolved_parameters.is_empty() {
            let mut resolved_in_this_pass = Vec::new();

            for (name, parameter) in &unresolved_parameters {
                debug!("{}", t!("configure.mod.processingParameter", name = name));
                if let Some(default_value) = &parameter.default_value {
                    debug!("{}", t!("configure.mod.setDefaultParameter", name = name));
                    let value_result = if default_value.is_string() {
                        if let Some(value) = default_value.as_str() {
                            self.context.process_mode = ProcessMode::ParametersDefault;
                            let result = self.statement_parser.parse_and_execute(value, &self.context);
                            self.context.process_mode = ProcessMode::Normal;
                            result
                        } else {
                            return Err(DscError::Parser(t!("configure.mod.defaultStringNotDefined").to_string()));
                        }
                    } else {
                        Ok(default_value.clone())
                    };

                    if let Ok(value) = value_result {
                        check_length(name, &value, parameter)?;
                        check_allowed_values(name, &value, parameter)?;
                        check_number_limits(name, &value, parameter)?;
                        validate_parameter_type(name, &value, &parameter.parameter_type)?;
                        self.context.parameters.insert(name.to_string(), (value, parameter.parameter_type.clone()));
                        resolved_in_this_pass.push(name.clone());
                    }
                } else {
                    resolved_in_this_pass.push(name.clone());
                }
            }

            if resolved_in_this_pass.is_empty() {
                let unresolved_names: Vec<_> = unresolved_parameters.keys().map(std::string::String::as_str).collect();
                return Err(DscError::Validation(t!("configure.mod.circularDependency", parameters = unresolved_names.join(", ")).to_string()));
            }

            for name in &resolved_in_this_pass {
                unresolved_parameters.remove(name);
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

    fn set_user_functions(&mut self, config: &Configuration) -> Result<(), DscError> {
        let Some(functions) = &config.functions else {
            return Ok(());
        };

        for user_function in functions {
            for (function_name, function_definition) in &user_function.members {
                if self.context.user_functions.contains_key(&format!("{}.{}", user_function.namespace, function_name)) {
                    return Err(DscError::Validation(t!("configure.mod.userFunctionAlreadyDefined", name = function_name, namespace = user_function.namespace).to_string()));
                }
                debug!("{}", t!("configure.mod.addingUserFunction", name = format!("{}.{}", user_function.namespace, function_name)));
                self.context.user_functions.insert(format!("{}.{}", user_function.namespace, function_name), function_definition.clone());
            }
        }
        Ok(())
    }

    fn get_result_metadata(&self, operation: Operation) -> Metadata {
        let end_datetime = chrono::Local::now();
        let version = self
            .context
            .dsc_version
            .clone()
            .unwrap_or_else(|| env!("CARGO_PKG_VERSION").to_string());
        Metadata {
            microsoft: Some(
                MicrosoftDscMetadata {
                    duration: Some(end_datetime.signed_duration_since(self.context.start_datetime).to_string()),
                    end_datetime: Some(end_datetime.to_rfc3339()),
                    execution_type: Some(self.context.execution_type.clone()),
                    operation: Some(operation),
                    restart_required: self.context.restart_required.clone(),
                    security_context: Some(self.context.security_context.clone()),
                    start_datetime: Some(self.context.start_datetime.to_rfc3339()),
                    version: Some(version),
                    copy_loops: None,
                }
            ),
            other: Map::new(),
        }
    }

    fn get_execution_information(&self, operation: Operation, execution_information: &mut ExecutionInformation) {
        let end_datetime = chrono::Local::now();
        execution_information.duration = Some(end_datetime.signed_duration_since(self.context.start_datetime).to_string());
        execution_information.end_datetime = Some(end_datetime.to_rfc3339());
        execution_information.start_datetime = Some(self.context.start_datetime.to_rfc3339());
        execution_information.version = self.context.dsc_version.clone();
        execution_information.execution_type = Some(self.context.execution_type.clone());
        execution_information.operation = Some(operation);
        execution_information.restart_required = self.context.restart_required.clone();
    }

    fn validate_config(&mut self) -> Result<(), DscError> {
        let config: Configuration = serde_json::from_str(self.json.as_str())?;
        check_security_context(config.metadata.as_ref(), config.directives.as_ref())?;

        if let Some(directives) = &config.directives {
            if let Some(version) = &directives.version {
                let dsc_version = SemanticVersion::parse(env!("CARGO_PKG_VERSION"))?;
                let version_req = SemanticVersionReq::parse(&version)?;
                if !version_req.matches(&dsc_version) {
                    return Err(DscError::Validation(t!("configure.mod.versionNotSatisfied", required_version = version, current_version = env!("CARGO_PKG_VERSION")).to_string()));
                }
            }
        }

        let mut resource_discovery_mode = ResourceDiscoveryMode::PreDeployment;
        if let Some(directives) = &config.directives {
            if let Some(resource_discovery_directive) = &directives.resource_discovery {
                resource_discovery_mode = resource_discovery_directive.clone();
            }
        }

        if resource_discovery_mode == ResourceDiscoveryMode::DuringDeployment {
            debug!("{}", t!("configure.mod.skippingResourceDiscovery"));
            self.discovery.refresh_cache = true;
        } else {
            // Perform discovery of resources used in config
            // create an array of DiscoveryFilter using the resource types and requireVersion from the config
            let mut discovery_filter: Vec<DiscoveryFilter> = Vec::new();
            let config_copy = config.clone();
            for resource in config_copy.resources {
                let adapter = get_require_adapter_from_directive(&resource.directives);
                let filter = DiscoveryFilter::new(&resource.resource_type, resource.require_version.as_deref(), adapter.as_deref());
                if !discovery_filter.contains(&filter) {
                    discovery_filter.push(filter);
                }
                // defer actual unrolling until parameters are available
                if let Some(copy) = &resource.copy {
                    debug!("{}", t!("configure.mod.validateCopy", name = &copy.name, count = copy.count));
                    if copy.mode.is_some() {
                        return Err(DscError::Validation(t!("configure.mod.copyModeNotSupported").to_string()));
                    }
                    if copy.batch_size.is_some() {
                        return Err(DscError::Validation(t!("configure.mod.copyBatchSizeNotSupported").to_string()));
                    }
                }
            }
            self.discovery.find_resources(&discovery_filter, self.progress_format)?;

            // now check that each resource in the config was found
            for resource in config.resources.iter() {
                let adapter = get_require_adapter_from_directive(&resource.directives);
                let Some(_dsc_resource) = self.discovery.find_resource(&DiscoveryFilter::new(&resource.resource_type, resource.require_version.as_deref(), adapter.as_deref()))? else {
                    return Err(DscError::ResourceNotFound(resource.resource_type.to_string(), resource.require_version.as_deref().unwrap_or("").to_string()));
                };
            }
        }

        self.config = config;
        Ok(())
    }

    /// Evaluate resource name expression and return the resolved name.
    ///
    /// This method evaluates DSC expressions in a resource name, handling both
    /// expressions and literals appropriately.
    ///
    /// # Arguments
    /// * `name` - The resource name that should be evaluated
    ///
    /// # Returns
    /// * `String` - The evaluated resource name
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - Resource name expression evaluation fails
    /// - Expression does not result in a string value
    /// - Statement parser encounters invalid syntax
    fn evaluate_resource_name(&mut self, name: &str) -> Result<String, DscError> {
        if self.context.process_mode == ProcessMode::Copy {
            return Ok(name.to_string());
        }

        // evaluate the resource name (handles both expressions and literals)
        let Value::String(evaluated_name) = self.statement_parser.parse_and_execute(name, &self.context)? else {
            return Err(DscError::Parser(t!("configure.mod.nameResultNotString").to_string()))
        };

        Ok(evaluated_name)
    }
}

pub fn invoke_property_expressions(parser: &mut Statement, context: &Context, properties: Option<&Map<String, Value>>) -> Result<Option<Map<String, Value>>, DscError> {
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
                    let value = invoke_property_expressions(parser, context, Some(object))?;
                    result.insert(name.clone(), serde_json::to_value(value)?);
                },
                Value::Array(array) => {
                    let mut result_array: Vec<Value> = Vec::new();
                    for element in array {
                        match element {
                            Value::Object(object) => {
                                let value = invoke_property_expressions(parser, context, Some(object))?;
                                result_array.push(serde_json::to_value(value)?);
                            },
                            Value::Array(_) => {
                                return Err(DscError::Parser(t!("configure.mod.nestedArraysNotSupported").to_string()));
                            },
                            Value::String(_) => {
                                // use as_str() so that the enclosing quotes are not included for strings
                                let Some(statement) = element.as_str() else {
                                    return Err(DscError::Parser(t!("configure.mod.arrayElementCouldNotTransformAsString").to_string()));
                                };
                                let statement_result = parser.parse_and_execute(statement, context)?;
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
                    let statement_result = parser.parse_and_execute(statement, context)?;
                    if let Some(string_result) = statement_result.as_str() {
                        result.insert(name.clone(), Value::String(string_result.to_string()));
                    } else {
                        result.insert(name.clone(), statement_result);
                    }
                },
                _ => {
                    result.insert(name.clone(), value.clone());
                },
            }
        }
    }
    Ok(Some(result))
}


/// Validate that a parameter value matches the expected type.
///
/// # Arguments
/// * `name` - The name of the parameter.
/// * `value` - The value of the parameter.
/// * `parameter_type` - The expected type of the parameter.
///
/// # Returns
/// * `Result<(), DscError>` - Ok if the value matches the expected type, Err otherwise.
///
/// # Errors
/// This function will return an error if the value does not match the expected type.
///
pub fn validate_parameter_type(name: &str, value: &Value, parameter_type: &DataType) -> Result<(), DscError> {
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
