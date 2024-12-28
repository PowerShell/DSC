// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::args::{ConfigSubCommand, DscType, OutputFormat, ResourceSubCommand};
use crate::resolve::{get_contents, Include};
use crate::resource_command::{get_resource, self};
use crate::tablewriter::Table;
use crate::util::{DSC_CONFIG_ROOT, EXIT_DSC_ERROR, EXIT_INVALID_ARGS, EXIT_INVALID_INPUT, EXIT_JSON_ERROR, get_schema, write_output, get_input, set_dscconfigroot, validate_json};
use dsc_lib::{
    configure::{
        config_doc::{
            Configuration,
            ExecutionKind,
            Resource,
        },
        config_result::ResourceGetResult,
        Configurator,
    },
    dscerror::DscError,
    DscManager,
    dscresources::invoke_result::{
        ResolveResult,
        TestResult,
        ValidateResult,
    },
    dscresources::dscresource::{Capability, ImplementedAs, Invoke},
    dscresources::resource_manifest::{import_manifest, ResourceManifest},
};
use rust_i18n::t;
use std::{
    collections::HashMap,
    io::{self, IsTerminal},
    path::Path,
    process::exit
};
use tracing::{debug, error, trace};

pub fn config_get(configurator: &mut Configurator, format: Option<&OutputFormat>, as_group: &bool)
{
    match configurator.invoke_get() {
        Ok(result) => {
            if *as_group {
                let json = match serde_json::to_string(&(result.results)) {
                    Ok(json) => json,
                    Err(err) => {
                        error!("JSON Error: {err}");
                        exit(EXIT_JSON_ERROR);
                    }
                };
                write_output(&json, format);
            }
            else {
                let json = match serde_json::to_string(&result) {
                    Ok(json) => json,
                    Err(err) => {
                        error!("JSON Error: {err}");
                        exit(EXIT_JSON_ERROR);
                    }
                };
                write_output(&json, format);
                if result.had_errors {
                    exit(EXIT_DSC_ERROR);
                }
            }
        },
        Err(err) => {
            error!("{err}");
            exit(EXIT_DSC_ERROR);
        }
    }
}

pub fn config_set(configurator: &mut Configurator, format: Option<&OutputFormat>, as_group: &bool)
{
    match configurator.invoke_set(false) {
        Ok(result) => {
            if *as_group {
                let json = match serde_json::to_string(&(result.results)) {
                    Ok(json) => json,
                    Err(err) => {
                        error!("JSON Error: {err}");
                        exit(EXIT_JSON_ERROR);
                    }
                };
                write_output(&json, format);
            }
            else {
                let json = match serde_json::to_string(&result) {
                    Ok(json) => json,
                    Err(err) => {
                        error!("JSON Error: {err}");
                        exit(EXIT_JSON_ERROR);
                    }
                };
                write_output(&json, format);
                if result.had_errors {
                    exit(EXIT_DSC_ERROR);
                }
            }
        },
        Err(err) => {
            error!("Error: {err}");
            exit(EXIT_DSC_ERROR);
        }
    }
}

pub fn config_test(configurator: &mut Configurator, format: Option<&OutputFormat>, as_group: &bool, as_get: &bool, as_config: &bool)
{
    match configurator.invoke_test() {
        Ok(result) => {
            if *as_group {
                let json = if *as_config {
                    let mut result_configuration = Configuration::new();
                    result_configuration.resources = Vec::new();
                    for test_result in result.results {
                        let properties = match test_result.result {
                            TestResult::Resource(test_response) => {
                                if test_response.actual_state.is_object() {
                                    test_response.actual_state.as_object().cloned()
                                } else {
                                    debug!("{}", t!("subcommand.actualStateNotObject"));
                                    None
                                }
                            },
                            TestResult::Group(_) => {
                                // not expected
                                debug!("{}", t!("subcommand.unexpectedTestResult"));
                                None
                            }
                        };
                        let resource = Resource {
                            name: test_result.name,
                            resource_type: test_result.resource_type,
                            properties,
                            depends_on: None,
                            metadata: None,
                        };
                        result_configuration.resources.push(resource);
                    }
                    match serde_json::to_string(&result_configuration) {
                        Ok(json) => json,
                        Err(err) => {
                            error!("JSON: {err}");
                            exit(EXIT_JSON_ERROR);
                        }
                    }
                }
                else if *as_get {
                    let mut group_result = Vec::<ResourceGetResult>::new();
                    for test_result in result.results {
                        group_result.push(test_result.into());
                    }
                    match serde_json::to_string(&group_result) {
                        Ok(json) => json,
                        Err(err) => {
                            error!("JSON: {err}");
                            exit(EXIT_JSON_ERROR);
                        }
                    }
                }
                else {
                    match serde_json::to_string(&(result.results)) {
                        Ok(json) => json,
                        Err(err) => {
                            error!("JSON: {err}");
                            exit(EXIT_JSON_ERROR);
                        }
                    }
                };
                write_output(&json, format);
            }
            else {
                let json = match serde_json::to_string(&result) {
                    Ok(json) => json,
                    Err(err) => {
                        error!("JSON: {err}");
                        exit(EXIT_JSON_ERROR);
                    }
                };
                write_output(&json, format);
                if result.had_errors {
                    exit(EXIT_DSC_ERROR);
                }
            }
        },
        Err(err) => {
            error!("{err}");
            exit(EXIT_DSC_ERROR);
        }
    }
}

pub fn config_export(configurator: &mut Configurator, format: Option<&OutputFormat>)
{
    match configurator.invoke_export() {
        Ok(result) => {
            let json = match serde_json::to_string(&result.result) {
                Ok(json) => json,
                Err(err) => {
                    error!("JSON: {err}");
                    exit(EXIT_JSON_ERROR);
                }
            };
            write_output(&json, format);
            if result.had_errors {

                for msg in result.messages
                {
                    error!("{:?} {} {}", msg.level, t!("subcommand.message"), msg.message);
                };

                exit(EXIT_DSC_ERROR);
            }
        },
        Err(err) => {
            error!("{err}");
            exit(EXIT_DSC_ERROR);
        }
    }
}

fn initialize_config_root(path: Option<&String>) -> Option<String> {
    // code that calls this pass in either None, Some("-"), or Some(path)
    // in the case of `-` we treat it as None, but need to pass it back as subsequent processing needs to handle it
    let use_stdin = if let Some(specified_path) = path {
        if specified_path != "-" {
            return Some(set_dscconfigroot(specified_path));
        }

        true
    } else {
        false
    };

    if std::env::var(DSC_CONFIG_ROOT).is_ok() {
        let config_root = std::env::var(DSC_CONFIG_ROOT).unwrap_or_default();
        debug!("DSC_CONFIG_ROOT = {config_root}");
    } else {
        let current_directory = std::env::current_dir().unwrap_or_default();
        debug!("DSC_CONFIG_ROOT = {} '{current_directory:?}'", t!("subcommand.currentDirectory"));
        set_dscconfigroot(&current_directory.to_string_lossy());
    }

    // if the path is "-", we need to return it so later processing can handle it correctly
    if use_stdin {
        return Some("-".to_string());
    }

    None
}

#[allow(clippy::too_many_lines)]
pub fn config(subcommand: &ConfigSubCommand, parameters: &Option<String>, mounted_path: Option<&String>, as_group: &bool, as_include: &bool) {
    let (new_parameters, json_string) = match subcommand {
        ConfigSubCommand::Get { input, file, .. } |
        ConfigSubCommand::Set { input, file, .. } |
        ConfigSubCommand::Test { input, file, .. } |
        ConfigSubCommand::Validate { input, file, .. } |
        ConfigSubCommand::Export { input, file, .. } => {
            let new_path = initialize_config_root(file.as_ref());
            let document = get_input(input.as_ref(), new_path.as_ref());
            if *as_include {
                let (new_parameters, config_json) = match get_contents(&document) {
                    Ok((parameters, config_json)) => (parameters, config_json),
                    Err(err) => {
                        error!("{err}");
                        exit(EXIT_DSC_ERROR);
                    }
                };
                (new_parameters, config_json)
            } else {
                (None, document)
            }
        },
        ConfigSubCommand::Resolve { input, file, .. } => {
            let new_path = initialize_config_root(file.as_ref());
            let document = get_input(input.as_ref(), new_path.as_ref());
            let (new_parameters, config_json) = match get_contents(&document) {
                Ok((parameters, config_json)) => (parameters, config_json),
                Err(err) => {
                    error!("{err}");
                    exit(EXIT_DSC_ERROR);
                }
            };
            (new_parameters, config_json)
        }
    };

    let mut configurator = match Configurator::new(&json_string) {
        Ok(configurator) => configurator,
        Err(err) => {
            error!("Error: {err}");
            exit(EXIT_DSC_ERROR);
        }
    };

    if let ConfigSubCommand::Set { what_if , .. } = subcommand {
        if *what_if {
            configurator.context.execution_type = ExecutionKind::WhatIf;
        }
    };

    let parameters: Option<serde_json::Value> = match if new_parameters.is_some() {
        &new_parameters
    } else {
        parameters
    } {
        None => {
            debug!("{}", t!("subcommand.noParameters"));
            None
        },
        Some(parameters) => {
            debug!("{}", t!("subcommand.parameters"));
            match serde_json::from_str(parameters) {
                Ok(json) => Some(json),
                Err(_) => {
                    match serde_yaml::from_str::<serde_yaml::Value>(parameters) {
                        Ok(yaml) => {
                            match serde_json::to_value(yaml) {
                                Ok(json) => Some(json),
                                Err(err) => {
                                    error!("{}: {err}", t!("subcommand.failedConvertJson"));
                                    exit(EXIT_DSC_ERROR);
                                }
                            }
                        },
                        Err(err) => {
                            error!("{}: {err}", t!("subcommand.invalidParamters"));
                            exit(EXIT_INVALID_INPUT);
                        }
                    }
                }
            }
        }
    };

    if let Some(path) = mounted_path {
        if !Path::new(&path).exists() {
            error!("{}: '{path}'", t!("subcommand.invalidPath"));
            exit(EXIT_INVALID_ARGS);
        }

        // make sure path has a trailing separator if it's a drive letter
        if path.len() == 2 && path.chars().nth(1).unwrap_or(' ') == ':' {
            configurator.set_system_root(&format!("{path}\\"));
        } else {
            configurator.set_system_root(path);
        }
    }

    if let Err(err) = configurator.set_context(parameters.as_ref()) {
        error!("{}: {err}", t!("subcommand.failedSetParameters"));
        exit(EXIT_INVALID_INPUT);
    }

    match subcommand {
        ConfigSubCommand::Get { output_format, .. } => {
            config_get(&mut configurator, output_format.as_ref(), as_group);
        },
        ConfigSubCommand::Set { output_format, .. } => {
            config_set(&mut configurator, output_format.as_ref(), as_group);
        },
        ConfigSubCommand::Test { output_format, as_get, as_config, .. } => {
            config_test(&mut configurator, output_format.as_ref(), as_group, as_get, as_config);
        },
        ConfigSubCommand::Validate { input, file, output_format} => {
            let mut result = ValidateResult {
                valid: true,
                reason: None,
            };
            if *as_include {
                let new_path = initialize_config_root(file.as_ref());
                let input = get_input(input.as_ref(), new_path.as_ref());
                match serde_json::from_str::<Include>(&input) {
                    Ok(_) => {
                        // valid, so do nothing
                    },
                    Err(err) => {
                        error!("{}: {err}", t!("subcommand.invalidInclude"));
                        result.valid = false;
                    }
                }
            } else {
                match validate_config(configurator.get_config()) {
                    Ok(()) => {
                        // valid, so do nothing
                    },
                    Err(err) => {
                        error!("{err}");
                        result.valid = false;
                    }
                };
            }

            let Ok(json) = serde_json::to_string(&result) else {
                error!("{}", t!("subcommand.failedSerialize"));
                exit(EXIT_JSON_ERROR);
            };

            write_output(&json, output_format.as_ref());
        },
        ConfigSubCommand::Export { output_format, .. } => {
            config_export(&mut configurator, output_format.as_ref());
        },
        ConfigSubCommand::Resolve { output_format, .. } => {
            let configuration = match serde_json::from_str(&json_string) {
                Ok(json) => json,
                Err(err) => {
                    error!("{}: {err}", t!("subcommand.invalidConfiguration"));
                    exit(EXIT_DSC_ERROR);
                }
            };
            // get the parameters out of the configurator
            let parameters_hashmap = if configurator.context.parameters.is_empty() {
                None
            } else {
                let mut parameters: HashMap<String, serde_json::Value> = HashMap::new();
                for (key, value) in &configurator.context.parameters {
                    parameters.insert(key.clone(), value.0.clone());
                }
                Some(parameters)
            };
            let resolve_result = ResolveResult {
                configuration,
                parameters: parameters_hashmap,
            };
            let json_string = match serde_json::to_string(&resolve_result) {
                Ok(json) => json,
                Err(err) => {
                    error!("{}: {err}", t!("subcommand.failedSerializeResolve"));
                    exit(EXIT_JSON_ERROR);
                }
            };
            write_output(&json_string, output_format.as_ref());
        },
    }
}

/// Validate configuration.
///
/// # Arguments
///
/// * `config` - The configuration to validate.
///
/// # Returns
///
/// Nothing on success.
///
/// # Errors
///
/// * `DscError` - The error that occurred.
pub fn validate_config(config: &Configuration) -> Result<(), DscError> {
    // first validate against the config schema
    debug!("{}", t!("subcommand.validatingConfiguration"));
    let schema = serde_json::to_value(get_schema(DscType::Configuration))?;
    let config_value = serde_json::to_value(config)?;
    validate_json("Configuration", &schema, &config_value)?;
    let mut dsc = DscManager::new()?;

    // then validate each resource
    let Some(resources) = config_value["resources"].as_array() else {
        return Err(DscError::Validation(t!("subcommand.noResources").to_string()));
    };

    // discover the resources
    let mut resource_types = Vec::new();
    for resource_block in resources {
        let Some(type_name) = resource_block["type"].as_str() else {
            return Err(DscError::Validation(t!("subcommand.resourceTypeNotSpecified").to_string()));
        };

        if resource_types.contains(&type_name.to_lowercase()) {
            continue;
        }

        resource_types.push(type_name.to_lowercase().to_string());
    }
    dsc.find_resources(&resource_types);

    for resource_block in resources {
        let Some(type_name) = resource_block["type"].as_str() else {
            return Err(DscError::Validation(t!("subcommand.resourceTypeNotSpecified").to_string()));
        };

        trace!("{} '{}'", t!("subcommand.validatingResource"), resource_block["name"].as_str().unwrap_or_default());

        // get the actual resource
        let Some(resource) = get_resource(&dsc, type_name) else {
            return Err(DscError::Validation(format!("{}: '{type_name}'", t!("subcommand.resourceNotFound"))));
        };

        // see if the resource is command based
        if resource.implemented_as == ImplementedAs::Command {
            // if so, see if it implements validate via the resource manifest
            if let Some(manifest) = resource.manifest.clone() {
                // convert to resource_manifest
                let manifest: ResourceManifest = serde_json::from_value(manifest)?;
                if manifest.validate.is_some() {
                    debug!("{}: {type_name} ", t!("subcommand.resourceImplementsValidate"));
                    // get the resource's part of the config
                    let resource_config = resource_block["properties"].to_string();
                    let result = resource.validate(&resource_config)?;
                    if !result.valid {
                        let reason = result.reason.unwrap_or(t!("subcommand.noReason").to_string());
                        let type_name = resource.type_name.clone();
                        return Err(DscError::Validation(format!("{}: {type_name} {reason}", t!("subcommand.resourceValidationFailed"))));
                    }
                }
                else {
                    // use schema validation
                    trace!("{}: {type_name}", t!("subcommand.resourceDoesNotImplementValidate"));
                    let Ok(schema) = resource.schema() else {
                        return Err(DscError::Validation(format!("{}: {type_name}", t!("subcommand.noSchemaOrValidate"))));
                    };
                    let schema = serde_json::from_str(&schema)?;

                    validate_json(&resource.type_name, &schema, &resource_block["properties"])?;
                }
            } else {
                return Err(DscError::Validation(format!("{}: {type_name}", t!("subcommand.noManifest"))));
            }
        }
    }

    Ok(())
}

#[allow(clippy::too_many_lines)]
pub fn resource(subcommand: &ResourceSubCommand) {
    let mut dsc = match DscManager::new() {
        Ok(dsc) => dsc,
        Err(err) => {
            error!("Error: {err}");
            exit(EXIT_DSC_ERROR);
        }
    };

    match subcommand {
        ResourceSubCommand::List { resource_name, adapter_name, description, tags, output_format } => {
            list_resources(&mut dsc, resource_name.as_ref(), adapter_name.as_ref(), description.as_ref(), tags.as_ref(), output_format.as_ref());
        },
        ResourceSubCommand::Schema { resource , output_format } => {
            dsc.find_resources(&[resource.to_string()]);
            resource_command::schema(&dsc, resource, output_format.as_ref());
        },
        ResourceSubCommand::Export { resource, output_format } => {
            dsc.find_resources(&[resource.to_string()]);
            resource_command::export(&mut dsc, resource, output_format.as_ref());
        },
        ResourceSubCommand::Get { resource, input, file: path, all, output_format } => {
            dsc.find_resources(&[resource.to_string()]);
            if *all { resource_command::get_all(&dsc, resource, output_format.as_ref()); }
            else {
                let parsed_input = get_input(input.as_ref(), path.as_ref());
                resource_command::get(&dsc, resource, parsed_input, output_format.as_ref());
            }
        },
        ResourceSubCommand::Set { resource, input, file: path, output_format } => {
            dsc.find_resources(&[resource.to_string()]);
            let parsed_input = get_input(input.as_ref(), path.as_ref());
            resource_command::set(&dsc, resource, parsed_input, output_format.as_ref());
        },
        ResourceSubCommand::Test { resource, input, file: path, output_format } => {
            dsc.find_resources(&[resource.to_string()]);
            let parsed_input = get_input(input.as_ref(), path.as_ref());
            resource_command::test(&dsc, resource, parsed_input, output_format.as_ref());
        },
        ResourceSubCommand::Delete { resource, input, file: path } => {
            dsc.find_resources(&[resource.to_string()]);
            let parsed_input = get_input(input.as_ref(), path.as_ref());
            resource_command::delete(&dsc, resource, parsed_input);
        },
    }
}

fn list_resources(dsc: &mut DscManager, resource_name: Option<&String>, adapter_name: Option<&String>, description: Option<&String>, tags: Option<&Vec<String>>, format: Option<&OutputFormat>) {
    let mut write_table = false;
    let mut table = Table::new(&[
        t!("subcommand.tableHeader_type").to_string().as_ref(),
        t!("subcommand.tableheader_kind").to_string().as_ref(),
        t!("subcommand.tableheader_version").to_string().as_ref(),
        t!("subcommand.tableheader_capabilities").to_string().as_ref(),
        t!("subcommand.tableheader_adapter").to_string().as_ref(),
        t!("subcommand.tableheader_description").to_string().as_ref(),
    ]);
    if format.is_none() && io::stdout().is_terminal() {
        // write as table if format is not specified and interactive
        write_table = true;
    }
    for resource in dsc.list_available_resources(resource_name.unwrap_or(&String::from("*")), adapter_name.unwrap_or(&String::new())) {
        let mut capabilities = "--------".to_string();
        let capability_types = [
            (Capability::Get, "g"),
            (Capability::Set, "s"),
            (Capability::SetHandlesExist, "x"),
            (Capability::WhatIf, "w"),
            (Capability::Test, "t"),
            (Capability::Delete, "d"),
            (Capability::Export, "e"),
            (Capability::Resolve, "r"),
        ];

        for (i, (capability, letter)) in capability_types.iter().enumerate() {
            if resource.capabilities.contains(capability) {
                capabilities.replace_range(i..=i, letter);
            }
        }

        // if description, tags, or write_table is specified, pull resource manifest if it exists
        if let Some(ref resource_manifest) = resource.manifest {
            let manifest = match import_manifest(resource_manifest.clone()) {
                Ok(resource_manifest) => resource_manifest,
                Err(err) => {
                    error!("{} {}: {err}", t!("subcommand.invalidManifest"), resource.type_name);
                    continue;
                }
            };

            // if description is specified, skip if resource description does not contain it
            if description.is_some() &&
                (manifest.description.is_none() | !manifest.description.unwrap_or_default().to_lowercase().contains(&description.unwrap_or(&String::new()).to_lowercase())) {
                continue;
            }

            // if tags is specified, skip if resource tags do not contain the tags
            if let Some(tags) = tags {
                let Some(manifest_tags) = manifest.tags else { continue; };

                let mut found = false;
                for tag_to_find in tags {
                    for tag in &manifest_tags {
                        if tag.to_lowercase() == tag_to_find.to_lowercase() {
                            found = true;
                            break;
                        }
                    }
                }
                if !found { continue; }
            }
        } else {
            // resource does not have a manifest but filtering on description or tags was requested - skip such resource
            if description.is_some() || tags.is_some() {
                continue;
            }
        }

        if write_table {
            table.add_row(vec![
                resource.type_name,
                format!("{:?}", resource.kind),
                resource.version,
                capabilities,
                resource.require_adapter.unwrap_or_default(),
                resource.description.unwrap_or_default()
            ]);
        }
        else {
            // convert to json
            let json = match serde_json::to_string(&resource) {
                Ok(json) => json,
                Err(err) => {
                    error!("JSON: {err}");
                    exit(EXIT_JSON_ERROR);
                }
            };
            write_output(&json, format);
            // insert newline separating instances if writing to console
            if io::stdout().is_terminal() { println!(); }
        }
    }

    if write_table {
        table.print();
    }
}
