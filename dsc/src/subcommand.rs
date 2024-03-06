// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::args::{ConfigSubCommand, DscType, OutputFormat, ResourceSubCommand};
use crate::resource_command::{get_resource, self};
use crate::Stream;
use crate::tablewriter::Table;
use crate::util::{EXIT_DSC_ERROR, EXIT_INVALID_INPUT, EXIT_JSON_ERROR, EXIT_VALIDATION_FAILED, get_schema, write_output, get_input, set_dscconfigroot, validate_json};
use dsc_lib::configure::{Configurator, ErrorAction, config_result::ResourceGetResult};
use dsc_lib::dscerror::DscError;
use dsc_lib::dscresources::invoke_result::{
    GroupResourceSetResponse, GroupResourceTestResponse, TestResult
};
use dsc_lib::{
    DscManager,
    dscresources::invoke_result::ValidateResult,
    dscresources::dscresource::{ImplementedAs, Invoke},
    dscresources::resource_manifest::{import_manifest, ResourceManifest},
};
use serde_yaml::Value;
use std::process::exit;
use tracing::{debug, error, trace};

pub fn config_get(configurator: &mut Configurator, format: &Option<OutputFormat>, as_group: &bool)
{
    match configurator.invoke_get(ErrorAction::Continue, || { /* code */ }) {
        Ok(result) => {
            if *as_group {
                let mut group_result = Vec::<ResourceGetResult>::new();
                for result in result.results {
                    group_result.push(result);
                };
                let json = match serde_json::to_string(&group_result) {
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

pub fn config_set(configurator: &mut Configurator, format: &Option<OutputFormat>, as_group: &bool)
{
    match configurator.invoke_set(false, ErrorAction::Continue, || { /* code */ }) {
        Ok(result) => {
            if *as_group {
                let group_result = GroupResourceSetResponse {
                    results: result.results
                };
                let json = match serde_json::to_string(&group_result) {
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

pub fn config_test(configurator: &mut Configurator, format: &Option<OutputFormat>, as_group: &bool, as_get: &bool)
{
    match configurator.invoke_test(ErrorAction::Continue, || { /* code */ }) {
        Ok(result) => {
            if *as_group {
                let mut in_desired_state = true;
                for test_result in &result.results {
                    match &test_result.result {
                        TestResult::Resource(resource_test_result) => {
                            if !resource_test_result.in_desired_state {
                                in_desired_state = false;
                                break;
                            }
                        },
                        TestResult::Group(group_resource_test_result) => {
                            if !group_resource_test_result.in_desired_state {
                                in_desired_state = false;
                                break;
                            }
                        }
                    }
                }
                let json = if *as_get {
                    let mut group_result = Vec::<ResourceGetResult>::new();
                    for test_result in result.results {
                        group_result.push(test_result.into());
                    }
                    match serde_json::to_string(&group_result) {
                        Ok(json) => json,
                        Err(err) => {
                            error!("JSON Error: {err}");
                            exit(EXIT_JSON_ERROR);
                        }
                    }
                }
                else {
                    let group_result = GroupResourceTestResponse {
                        results: result.results,
                        in_desired_state
                    };
                    match serde_json::to_string(&group_result) {
                        Ok(json) => json,
                        Err(err) => {
                            error!("JSON Error: {err}");
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

pub fn config_export(configurator: &mut Configurator, format: &Option<OutputFormat>)
{
    match configurator.invoke_export(ErrorAction::Continue, || { /* code */ }) {
        Ok(result) => {
            let json = match serde_json::to_string(&result.result) {
                Ok(json) => json,
                Err(err) => {
                    error!("JSON Error: {err}");
                    exit(EXIT_JSON_ERROR);
                }
            };
            write_output(&json, format);
            if result.had_errors {

                for msg in result.messages
                {
                    error!("{:?} message {}", msg.level, msg.message);
                };

                exit(EXIT_DSC_ERROR);
            }
        },
        Err(err) => {
            error!("Error: {err}");
            exit(EXIT_DSC_ERROR);
        }
    }
}

pub fn config(subcommand: &ConfigSubCommand, parameters: &Option<String>, stdin: &Option<String>, as_group: &bool) {
    let json_string = match subcommand {
        ConfigSubCommand::Get { document, path, .. } |
        ConfigSubCommand::Set { document, path, .. } |
        ConfigSubCommand::Test { document, path, .. } |
        ConfigSubCommand::Validate { document, path, .. } |
        ConfigSubCommand::Export { document, path, .. } => {
            let mut new_path = path;
            let opt_new_path;
            if path.is_some()
            {
                let config_path = path.clone().unwrap_or_default();
                opt_new_path = Some(set_dscconfigroot(&config_path));
                new_path = &opt_new_path;
            }
            get_input(document, stdin, new_path)
        }
    };

    let mut configurator = match Configurator::new(&json_string) {
        Ok(configurator) => configurator,
        Err(err) => {
            error!("Error: {err}");
            exit(EXIT_DSC_ERROR);
        }
    };

    let parameters: Option<serde_json::Value> = match parameters {
        None => None,
        Some(parameters) => {
            match serde_json::from_str(parameters) {
                Ok(json) => Some(json),
                Err(_) => {
                    match serde_yaml::from_str::<Value>(parameters) {
                        Ok(yaml) => {
                            match serde_json::to_value(yaml) {
                                Ok(json) => Some(json),
                                Err(err) => {
                                    error!("Error: Failed to convert YAML to JSON: {err}");
                                    exit(EXIT_DSC_ERROR);
                                }
                            }
                        },
                        Err(err) => {
                            error!("Error: Parameters are not valid JSON or YAML: {err}");
                            exit(EXIT_INVALID_INPUT);
                        }
                    }
                }
            }
        }
    };

    if let Err(err) = configurator.set_parameters(&parameters) {
        error!("Error: Parameter input failure: {err}");
        exit(EXIT_INVALID_INPUT);
    }

    match subcommand {
        ConfigSubCommand::Get { format, .. } => {
            config_get(&mut configurator, format, as_group);
        },
        ConfigSubCommand::Set { format, .. } => {
            config_set(&mut configurator, format, as_group);
        },
        ConfigSubCommand::Test { format, as_get, .. } => {
            config_test(&mut configurator, format, as_group, as_get);
        },
        ConfigSubCommand::Validate { format, .. } => {
            let mut result = ValidateResult {
                valid: true,
                reason: None,
            };
            let valid = match validate_config(&json_string) {
                Ok(()) => {
                    true
                },
                Err(err) => {
                    error!("{err}");
                    result.valid = false;
                    false
                }
            };

            let Ok(json) = serde_json::to_string(&result) else {
                error!("Failed to convert validation result to JSON");
                exit(EXIT_JSON_ERROR);
            };

            write_output(&json, format);
            if !valid {
                exit(EXIT_VALIDATION_FAILED);
            }
        },
        ConfigSubCommand::Export { format, .. } => {
            config_export(&mut configurator, format);
        }
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
pub fn validate_config(config: &str) -> Result<(), DscError> {
    // first validate against the config schema
    debug!("Validating configuration against schema");
    let schema = serde_json::to_value(get_schema(DscType::Configuration))?;
    let config_value = serde_json::from_str(config)?;
    validate_json("Configuration", &schema, &config_value)?;
    let mut dsc = DscManager::new()?;

    // then validate each resource
    let Some(resources) = config_value["resources"].as_array() else {
        return Err(DscError::Validation("Error: Resources not specified".to_string()));
    };

    // discover the resources
    let mut resource_types = Vec::new();
    for resource_block in resources {
        let Some(type_name) = resource_block["type"].as_str() else {
            return Err(DscError::Validation("Error: Resource type not specified".to_string()));
        };

        if resource_types.contains(&type_name.to_lowercase()) {
            continue;
        }

        resource_types.push(type_name.to_lowercase().to_string());
    }
    dsc.discover_resources(&resource_types);

    for resource_block in resources {
        let Some(type_name) = resource_block["type"].as_str() else {
            return Err(DscError::Validation("Error: Resource type not specified".to_string()));
        };

        trace!("Validating resource named '{}'", resource_block["name"].as_str().unwrap_or_default());

        // get the actual resource
        let Some(resource) = get_resource(&dsc, type_name) else {
            return Err(DscError::Validation(format!("Error: Resource type '{type_name}' not found")));
        };

        // see if the resource is command based
        if resource.implemented_as == ImplementedAs::Command {
            // if so, see if it implements validate via the resource manifest
            if let Some(manifest) = resource.manifest.clone() {
                // convert to resource_manifest
                let manifest: ResourceManifest = serde_json::from_value(manifest)?;
                if manifest.validate.is_some() {
                    debug!("Resource {type_name} implements validation");
                    // get the resource's part of the config
                    let resource_config = resource_block["properties"].to_string();
                    let result = resource.validate(&resource_config)?;
                    if !result.valid {
                        let reason = result.reason.unwrap_or("No reason provided".to_string());
                        let type_name = resource.type_name.clone();
                        return Err(DscError::Validation(format!("Resource {type_name} failed validation: {reason}")));
                    }
                }
                else {
                    // use schema validation
                    trace!("Resource {type_name} does not implement validation, using schema");
                    let Ok(schema) = resource.schema() else {
                        return Err(DscError::Validation(format!("Error: Resource {type_name} does not have a schema nor supports validation")));
                    };
                    let schema = serde_json::from_str(&schema)?;

                    validate_json(&resource.type_name, &schema, &resource_block["properties"])?;
                }
            } else {
                return Err(DscError::Validation(format!("Error: Resource {type_name} does not have a manifest")));
            }
        }
    }

    Ok(())
}

pub fn resource(subcommand: &ResourceSubCommand, stdin: &Option<String>) {
    let mut dsc = match DscManager::new() {
        Ok(dsc) => dsc,
        Err(err) => {
            error!("Error: {err}");
            exit(EXIT_DSC_ERROR);
        }
    };

    match subcommand {
        ResourceSubCommand::List { resource_name, description, tags, format } => {

            let mut write_table = false;
            let mut methods: Vec<String> = Vec::new();
            let mut table = Table::new(&["Type", "Kind", "Version", "Methods", "Requires", "Description"]);
            if format.is_none() && atty::is(Stream::Stdout) {
                // write as table if format is not specified and interactive
                write_table = true;
            }
            for resource in dsc.list_available_resources(&resource_name.clone().unwrap_or_default()) {
                // if description, tags, or write_table is specified, pull resource manifest if it exists
                if description.is_some() || tags.is_some() || write_table {
                    let Some(ref resource_manifest) = resource.manifest else {
                        continue;
                    };
                    let manifest = match import_manifest(resource_manifest.clone()) {
                        Ok(resource_manifest) => resource_manifest,
                        Err(err) => {
                            error!("Error in manifest for {0}: {err}", resource.type_name);
                            continue;
                        }
                    };

                    // if description is specified, skip if resource description does not contain it
                    if description.is_some() &&
                        (manifest.description.is_none() | !manifest.description.unwrap_or_default().to_lowercase().contains(&description.as_ref().unwrap_or(&String::new()).to_lowercase())) {
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

                    methods = vec!["get".to_string()];
                    if manifest.set.is_some() { methods.push("set".to_string()); }
                    if manifest.test.is_some() { methods.push("test".to_string()); }
                    if manifest.export.is_some() { methods.push("export".to_string()); }
                }

                if write_table {
                    table.add_row(vec![
                        resource.type_name,
                        format!("{:?}", resource.kind),
                        resource.version,
                        methods.join(", "),
                        resource.requires.unwrap_or_default(),
                        resource.description.unwrap_or_default()
                    ]);
                }
                else {
                    // convert to json
                    let json = match serde_json::to_string(&resource) {
                        Ok(json) => json,
                        Err(err) => {
                            error!("JSON Error: {err}");
                            exit(EXIT_JSON_ERROR);
                        }
                    };
                    write_output(&json, format);
                    // insert newline separating instances if writing to console
                    if atty::is(Stream::Stdout) { println!(); }
                }
            }

            if write_table { table.print(); }
        },
        ResourceSubCommand::Schema { resource , format } => {
            dsc.discover_resources(&[resource.to_lowercase().to_string()]);
            resource_command::schema(&dsc, resource, format);
        },
        ResourceSubCommand::Export { resource, format } => {
            dsc.discover_resources(&[resource.to_lowercase().to_string()]);
            resource_command::export(&mut dsc, resource, format);
        },
        ResourceSubCommand::Get { resource, input, path, all, format } => {
            dsc.discover_resources(&[resource.to_lowercase().to_string()]);
            if *all { resource_command::get_all(&dsc, resource, format); }
            else {
                let parsed_input = get_input(input, stdin, path);
                resource_command::get(&dsc, resource, parsed_input, format);
            }
        },
        ResourceSubCommand::Set { resource, input, path, format } => {
            dsc.discover_resources(&[resource.to_lowercase().to_string()]);
            let parsed_input = get_input(input, stdin, path);
            resource_command::set(&dsc, resource, parsed_input, format);
        },
        ResourceSubCommand::Test { resource, input, path, format } => {
            dsc.discover_resources(&[resource.to_lowercase().to_string()]);
            let parsed_input = get_input(input, stdin, path);
            resource_command::test(&dsc, resource, parsed_input, format);
        },
    }
}
