// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::args::{ConfigSubCommand, SchemaType, ExtensionSubCommand, FunctionSubCommand, GetOutputFormat, ListOutputFormat, OutputFormat, ResourceSubCommand};
use crate::resolve::{get_contents, Include};
use crate::resource_command::{get_resource, self};
use crate::tablewriter::Table;
use crate::util::{get_input, get_schema, in_desired_state, set_dscconfigroot, write_object, DSC_CONFIG_ROOT, EXIT_DSC_ASSERTION_FAILED, EXIT_DSC_ERROR, EXIT_INVALID_ARGS, EXIT_INVALID_INPUT, EXIT_JSON_ERROR};
use dsc_lib::functions::FunctionArgKind;
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
    discovery::discovery_trait::{DiscoveryFilter, DiscoveryKind},
    discovery::command_discovery::ImportedManifest,
    dscerror::DscError,
    DscManager,
    dscresources::invoke_result::{
        ResolveResult,
        TestResult,
        ValidateResult,
    },
    dscresources::dscresource::{Capability, ImplementedAs, validate_json, validate_properties},
    extensions::dscextension::Capability as ExtensionCapability,
    functions::FunctionDispatcher,
    progress::ProgressFormat,
    util::convert_wildcard_to_regex,
};
use regex::RegexBuilder;
use rust_i18n::t;
use core::convert::AsRef;
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
                write_object(&json, format, false);
            }
            else {
                let json = match serde_json::to_string(&result) {
                    Ok(json) => json,
                    Err(err) => {
                        error!("JSON Error: {err}");
                        exit(EXIT_JSON_ERROR);
                    }
                };
                write_object(&json, format, false);
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
                write_object(&json, format, false);
            }
            else {
                let json = match serde_json::to_string(&result) {
                    Ok(json) => json,
                    Err(err) => {
                        error!("JSON Error: {err}");
                        exit(EXIT_JSON_ERROR);
                    }
                };
                write_object(&json, format, false);
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

pub fn config_test(configurator: &mut Configurator, format: Option<&OutputFormat>, as_group: &bool, as_get: &bool, as_config: &bool, as_assert: &bool)
{
    match configurator.invoke_test() {
        Ok(result) => {
            if *as_group {
                let json = if *as_config {
                    let mut result_configuration = Configuration::new();
                    result_configuration.resources = Vec::new();
                    for test_result in result.results {
                        if *as_assert && !in_desired_state(&test_result) {
                            error!("{}", t!("subcommand.assertionFailed", resource_type = test_result.resource_type));
                            exit(EXIT_DSC_ASSERTION_FAILED);
                        }
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
                            ..Default::default()
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
                        if *as_assert && !in_desired_state(&test_result) {
                            error!("{}", t!("subcommand.assertionFailed", resource_type = test_result.resource_type));
                            exit(EXIT_DSC_ASSERTION_FAILED);
                        }
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
                    if *as_assert {
                        for test_result in &result.results {
                            if !in_desired_state(test_result) {
                                error!("{}", t!("subcommand.assertionFailed", resource_type = test_result.resource_type));
                                exit(EXIT_DSC_ASSERTION_FAILED);
                            }
                        }
                    }
                    match serde_json::to_string(&(result.results)) {
                        Ok(json) => json,
                        Err(err) => {
                            error!("JSON: {err}");
                            exit(EXIT_JSON_ERROR);
                        }
                    }
                };
                write_object(&json, format, false);
            }
            else {
                let json = match serde_json::to_string(&result) {
                    Ok(json) => json,
                    Err(err) => {
                        error!("JSON: {err}");
                        exit(EXIT_JSON_ERROR);
                    }
                };
                write_object(&json, format, false);
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
            write_object(&json, format, false);
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
        set_dscconfigroot(current_directory.to_str().unwrap_or_default());
    }

    // if the path is "-", we need to return it so later processing can handle it correctly
    if use_stdin {
        return Some("-".to_string());
    }

    None
}

#[allow(clippy::too_many_lines)]
#[allow(clippy::too_many_arguments)]
pub fn config(subcommand: &ConfigSubCommand, parameters: &Option<String>, mounted_path: Option<&String>, as_group: &bool, as_assert: &bool, as_include: &bool, progress_format: ProgressFormat) {
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

    let mut configurator = match Configurator::new(&json_string, progress_format) {
        Ok(configurator) => configurator,
        Err(err) => {
            error!("Error: {err}");
            exit(EXIT_DSC_ERROR);
        }
    };

    configurator.context.dsc_version = Some(env!("CARGO_PKG_VERSION").to_string());

    if let ConfigSubCommand::Set { what_if , .. } = subcommand {
        if *what_if {
            configurator.context.execution_type = ExecutionKind::WhatIf;
        }
    }

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
                            error!("{}: {err}", t!("subcommand.invalidParameters"));
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
            config_test(&mut configurator, output_format.as_ref(), as_group, as_get, as_config, as_assert);
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
                match validate_config(configurator.get_config(), progress_format) {
                    Ok(()) => {
                        // valid, so do nothing
                    },
                    Err(err) => {
                        error!("{err}");
                        result.valid = false;
                    }
                }
            }

            let Ok(json) = serde_json::to_string(&result) else {
                error!("{}", t!("subcommand.failedSerialize"));
                exit(EXIT_JSON_ERROR);
            };

            write_object(&json, output_format.as_ref(), false);
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
            write_object(&json_string, output_format.as_ref(), false);
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
pub fn validate_config(config: &Configuration, progress_format: ProgressFormat) -> Result<(), DscError> {
    // first validate against the config schema
    debug!("{}", t!("subcommand.validatingConfiguration"));
    let schema = serde_json::to_value(get_schema(SchemaType::Configuration))?;
    let config_value = serde_json::to_value(config)?;
    validate_json("Configuration", &schema, &config_value)?;
    let mut dsc = DscManager::new();

    // then validate each resource
    let Some(resources) = config_value["resources"].as_array() else {
        return Err(DscError::Validation(t!("subcommand.noResources").to_string()));
    };

    // discover the resources
    let mut resource_types = Vec::<DiscoveryFilter>::new();
    for resource_block in resources {
        let Some(type_name) = resource_block["type"].as_str() else {
            return Err(DscError::Validation(t!("subcommand.resourceTypeNotSpecified").to_string()));
        };
        resource_types.push(DiscoveryFilter::new(type_name, resource_block["requireVersion"].as_str(), None));
    }
    dsc.find_resources(&resource_types, progress_format)?;

    for resource_block in resources {
        let Some(type_name) = resource_block["type"].as_str() else {
            return Err(DscError::Validation(t!("subcommand.resourceTypeNotSpecified").to_string()));
        };

        trace!("{} '{}'", t!("subcommand.validatingResource"), resource_block["name"].as_str().unwrap_or_default());

        // get the actual resource
        let Some(resource) = get_resource(&mut dsc, type_name, resource_block["requireVersion"].as_str()) else {
            return Err(DscError::Validation(format!("{}: '{type_name}'", t!("subcommand.resourceNotFound"))));
        };

        // see if the resource is command based
        if resource.implemented_as == Some(ImplementedAs::Command) {
            validate_properties(resource, &resource_block["properties"])?;
        }
    }

    Ok(())
}

pub fn extension(subcommand: &ExtensionSubCommand, progress_format: ProgressFormat) {
    let mut dsc = DscManager::new();

    match subcommand {
        ExtensionSubCommand::List{extension_name, output_format} => {
            list_extensions(&mut dsc, extension_name.as_ref(), output_format.as_ref(), progress_format);
        },
    }
}

pub fn function(subcommand: &FunctionSubCommand) {
    let functions = FunctionDispatcher::new();
    match subcommand {
        FunctionSubCommand::List { function_name, output_format } => {
            list_functions(&functions, function_name.as_ref(), output_format.as_ref());
        },
    }
}

#[allow(clippy::too_many_lines)]
pub fn resource(subcommand: &ResourceSubCommand, progress_format: ProgressFormat) {
    let mut dsc = DscManager::new();

    match subcommand {
        ResourceSubCommand::List { resource_name, adapter_name, description, tags, output_format } => {
            list_resources(&mut dsc, resource_name.as_ref(), adapter_name.as_ref(), description.as_ref(), tags.as_ref(), output_format.as_ref(), progress_format);
        },
        ResourceSubCommand::Schema { resource , version, output_format } => {
            if let Err(err) = dsc.find_resources(&[DiscoveryFilter::new(resource, version.as_deref(), None)], progress_format) {
                error!("{}: {err}", t!("subcommand.failedDiscoverResource"));
                exit(EXIT_DSC_ERROR);
            }
            resource_command::schema(&mut dsc, resource, version.as_deref(), output_format.as_ref());
        },
        ResourceSubCommand::Export { resource, version, input, file, output_format } => {
            if let Err(err) = dsc.find_resources(&[DiscoveryFilter::new(resource, version.as_deref(), None)], progress_format) {
                error!("{}: {err}", t!("subcommand.failedDiscoverResource"));
                exit(EXIT_DSC_ERROR);
            }
            let parsed_input = get_input(input.as_ref(), file.as_ref());
            resource_command::export(&mut dsc, resource, version.as_deref(), &parsed_input, output_format.as_ref());
        },
        ResourceSubCommand::Get { resource, version, input, file: path, all, output_format } => {
            if let Err(err) = dsc.find_resources(&[DiscoveryFilter::new(resource, version.as_deref(), None)], progress_format) {
                error!("{}: {err}", t!("subcommand.failedDiscoverResource"));
                exit(EXIT_DSC_ERROR);
            }
            if *all {
                resource_command::get_all(&mut dsc, resource, version.as_deref(), output_format.as_ref());
            }
            else {
                if *output_format == Some(GetOutputFormat::JsonArray) {
                    error!("{}", t!("subcommand.jsonArrayNotSupported"));
                    exit(EXIT_INVALID_ARGS);
                }
                let parsed_input = get_input(input.as_ref(), path.as_ref());
                resource_command::get(&mut dsc, resource, version.as_deref(), &parsed_input, output_format.as_ref());
            }
        },
        ResourceSubCommand::Set { resource, version, input, file: path, output_format, what_if } => {
            if let Err(err) = dsc.find_resources(&[DiscoveryFilter::new(resource, version.as_deref(), None)], progress_format) {
                error!("{}: {err}", t!("subcommand.failedDiscoverResource"));
                exit(EXIT_DSC_ERROR);
            }
            let parsed_input = get_input(input.as_ref(), path.as_ref());
            resource_command::set(&mut dsc, resource, version.as_deref(), &parsed_input, output_format.as_ref(), *what_if);
        },
        ResourceSubCommand::Test { resource, version, input, file: path, output_format } => {
            if let Err(err) = dsc.find_resources(&[DiscoveryFilter::new(resource, version.as_deref(), None)], progress_format) {
                error!("{}: {err}", t!("subcommand.failedDiscoverResource"));
                exit(EXIT_DSC_ERROR);
            }
            let parsed_input = get_input(input.as_ref(), path.as_ref());
            resource_command::test(&mut dsc, resource, version.as_deref(), &parsed_input, output_format.as_ref());
        },
        ResourceSubCommand::Delete { resource, version, input, file: path } => {
            if let Err(err) = dsc.find_resources(&[DiscoveryFilter::new(resource, version.as_deref(), None)], progress_format) {
                error!("{}: {err}", t!("subcommand.failedDiscoverResource"));
                exit(EXIT_DSC_ERROR);
            }
            let parsed_input = get_input(input.as_ref(), path.as_ref());
            resource_command::delete(&mut dsc, resource, version.as_deref(), &parsed_input);
        },
    }
}

fn list_extensions(dsc: &mut DscManager, extension_name: Option<&String>, format: Option<&ListOutputFormat>, progress_format: ProgressFormat) {
    let mut write_table = false;
    let mut table = Table::new(&[
        t!("subcommand.tableHeader_type").to_string().as_ref(),
        t!("subcommand.tableHeader_version").to_string().as_ref(),
        t!("subcommand.tableHeader_capabilities").to_string().as_ref(),
        t!("subcommand.tableHeader_description").to_string().as_ref(),
    ]);
    if format.is_none() && io::stdout().is_terminal() {
        // write as table if format is not specified and interactive
        write_table = true;
    }
    let mut include_separator = false;
    for manifest_resource in dsc.list_available(&DiscoveryKind::Extension, extension_name.unwrap_or(&String::from("*")), "", progress_format) {
        if let ImportedManifest::Extension(extension) = manifest_resource {
            let capability_types = [
                (ExtensionCapability::Discover, "d"),
                (ExtensionCapability::Secret, "s"),
                (ExtensionCapability::Import, "i"),
            ];
            let mut capabilities = "-".repeat(capability_types.len());

            for (i, (capability, letter)) in capability_types.iter().enumerate() {
                if extension.capabilities.contains(capability) {
                    capabilities.replace_range(i..=i, letter);
                }
            }

            if write_table {
                table.add_row(vec![
                    extension.type_name.to_string(),
                    extension.version,
                    capabilities,
                    extension.description.unwrap_or_default()
                ]);
            }
            else {
                // convert to json
                let json = match serde_json::to_string(&extension) {
                    Ok(json) => json,
                    Err(err) => {
                        error!("JSON: {err}");
                        exit(EXIT_JSON_ERROR);
                    }
                };
                let format = match format {
                    Some(ListOutputFormat::Json) => Some(OutputFormat::Json),
                    Some(ListOutputFormat::PrettyJson) => Some(OutputFormat::PrettyJson),
                    Some(ListOutputFormat::Yaml) => Some(OutputFormat::Yaml),
                    _ => None,
                };
                write_object(&json, format.as_ref(), include_separator);
                include_separator = true;
                // insert newline separating instances if writing to console
                if io::stdout().is_terminal() { println!(); }
            }
        }
    }

    if write_table {
        let truncate = format != Some(&ListOutputFormat::TableNoTruncate);
        table.print(truncate);
    }
}

fn list_functions(functions: &FunctionDispatcher, function_name: Option<&String>, output_format: Option<&ListOutputFormat>) {
    let mut write_table = false;
    let mut table = Table::new(&[
        t!("subcommand.tableHeader_functionCategory").to_string().as_ref(),
        t!("subcommand.tableHeader_functionName").to_string().as_ref(),
        t!("subcommand.tableHeader_minArgs").to_string().as_ref(),
        t!("subcommand.tableHeader_maxArgs").to_string().as_ref(),
        t!("subcommand.tableHeader_argTypes").to_string().as_ref(),
        t!("subcommand.tableHeader_description").to_string().as_ref(),
    ]);
    if output_format.is_none() && io::stdout().is_terminal() {
        // write as table if format is not specified and interactive
        write_table = true;
    }
    let mut include_separator = false;
    let returned_types= [
        (FunctionArgKind::Array, "a"),
        (FunctionArgKind::Boolean, "b"),
        (FunctionArgKind::Number, "n"),
        (FunctionArgKind::String, "s"),
        (FunctionArgKind::Object, "o"),
    ];

    let asterisks = String::from("*");
    let name = function_name.unwrap_or(&asterisks);
    let regex_str = convert_wildcard_to_regex(name);
    let mut regex_builder = RegexBuilder::new(&regex_str);
    regex_builder.case_insensitive(true);
    let Ok(regex) = regex_builder.build() else {
        error!("{}: {}", t!("subcommand.invalidFunctionFilter"), regex_str);
        exit(EXIT_INVALID_ARGS);
    };

    let mut functions_list = functions.list();
    functions_list.sort();
    for function in functions_list {
        if !regex.is_match(&function.name) {
            continue;
        }

        if write_table {
            // construct arg_types from '-' times number of accepted_arg_types
            let mut arg_types = "-".repeat(returned_types.len());
            for (i, (arg_type, letter)) in returned_types.iter().enumerate() {
                if function.return_types.contains(arg_type) {
                    arg_types.replace_range(i..=i, letter);
                }
            }

            let max_args = if function.max_args == usize::MAX {
                t!("subcommand.maxInt").to_string()
            } else {
                function.max_args.to_string()
            };

            table.add_row(vec![
                function.category.iter().map(std::string::ToString::to_string).collect::<Vec<String>>().join(", "),
                function.name,
                function.min_args.to_string(),
                max_args,
                arg_types,
                function.description
            ]);
        }
        else {
            let json = match serde_json::to_string(&function) {
                Ok(json) => json,
                Err(err) => {
                    error!("JSON: {err}");
                    exit(EXIT_JSON_ERROR);
                }
            };
            let format = match output_format {
                Some(ListOutputFormat::Json) => Some(OutputFormat::Json),
                Some(ListOutputFormat::PrettyJson) => Some(OutputFormat::PrettyJson),
                Some(ListOutputFormat::Yaml) => Some(OutputFormat::Yaml),
                _ => None,
            };
            write_object(&json, format.as_ref(), include_separator);
            include_separator = true;
            // insert newline separating instances if writing to console
            if io::stdout().is_terminal() { println!(); }
        }
    }

    if write_table {
        let truncate = output_format != Some(&ListOutputFormat::TableNoTruncate);
        table.print(truncate);
    }
}

pub fn list_resources(dsc: &mut DscManager, resource_name: Option<&String>, adapter_name: Option<&String>, description: Option<&String>, tags: Option<&Vec<String>>, format: Option<&ListOutputFormat>, progress_format: ProgressFormat) {
    let mut write_table = false;
    let mut table = Table::new(&[
        t!("subcommand.tableHeader_type").to_string().as_ref(),
        t!("subcommand.tableHeader_kind").to_string().as_ref(),
        t!("subcommand.tableHeader_version").to_string().as_ref(),
        t!("subcommand.tableHeader_capabilities").to_string().as_ref(),
        t!("subcommand.tableHeader_adapter").to_string().as_ref(),
        t!("subcommand.tableHeader_description").to_string().as_ref(),
    ]);
    if format == Some(&ListOutputFormat::TableNoTruncate) || (format.is_none() && io::stdout().is_terminal()) {
        // write as table if format is not specified and interactive
        write_table = true;
    }
    let mut include_separator = false;
    for manifest_resource in dsc.list_available(&DiscoveryKind::Resource, resource_name.unwrap_or(&String::from("*")), adapter_name.unwrap_or(&String::new()), progress_format) {
        if let ImportedManifest::Resource(resource) = manifest_resource {
            let capability_types = [
                (Capability::Get, "g"),
                (Capability::Set, "s"),
                (Capability::SetHandlesExist, "x"),
                (Capability::Test, "t"),
                (Capability::Delete, "d"),
                (Capability::Export, "e"),
                (Capability::Resolve, "r"),
            ];
            let mut capabilities = "-".repeat(capability_types.len());

            for (i, (capability, letter)) in capability_types.iter().enumerate() {
                if resource.capabilities.contains(capability) {
                    capabilities.replace_range(i..=i, letter);
                }
            }

            // if description, tags, or write_table is specified, pull resource manifest if it exists
            if let Some(ref manifest) = resource.manifest {
                // if description is specified, skip if resource description does not contain it
                if description.is_some() && (manifest.description.is_none() | !manifest.description.clone().unwrap_or_default().to_lowercase().contains(&description.unwrap_or(&String::new()).to_lowercase())) {
                    continue;
                }

                // if tags is specified, skip if resource tags do not contain the tags
                if let Some(tags) = tags {
                    if manifest.tags.is_empty() { continue; }

                    let mut found = false;
                    for tag_to_find in tags {
                        for tag in manifest.tags.as_ref() {
                            if tag == tag_to_find {
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
                    resource.type_name.to_string(),
                    format!("{:?}", resource.kind),
                    resource.version,
                    capabilities,
                    resource.require_adapter.unwrap_or_default().to_string(),
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
                let format = match format {
                    Some(ListOutputFormat::Json) => Some(OutputFormat::Json),
                    Some(ListOutputFormat::PrettyJson) => Some(OutputFormat::PrettyJson),
                    Some(ListOutputFormat::Yaml) => Some(OutputFormat::Yaml),
                    _ => None,
                };
                write_object(&json, format.as_ref(), include_separator);
                include_separator = true;
                // insert newline separating instances if writing to console
                if io::stdout().is_terminal() { println!(); }
            }
        }
    }

    if write_table {
        let truncate = format != Some(&ListOutputFormat::TableNoTruncate);
        table.print(truncate);
    }
}
