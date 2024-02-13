// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::args::{ConfigSubCommand, DscType, OutputFormat, ResourceSubCommand};
use crate::resource_command::{get_resource, self};
use crate::tablewriter::Table;
use crate::util::{EXIT_DSC_ERROR, EXIT_INVALID_INPUT, EXIT_JSON_ERROR, EXIT_SUCCESS, EXIT_VALIDATION_FAILED, get_schema, write_output, get_input, set_dscconfigroot};
use tracing::error;

use atty::Stream;
use dsc_lib::{
    configure::{Configurator, ErrorAction},
    DscManager,
    dscresources::dscresource::{ImplementedAs, Invoke},
    dscresources::resource_manifest::{import_manifest, ResourceManifest},
};
use jsonschema::JSONSchema;
use serde_yaml::Value;
use std::process::exit;

pub fn config_get(configurator: &mut Configurator, format: &Option<OutputFormat>)
{
    match configurator.invoke_get(ErrorAction::Continue, || { /* code */ }) {
        Ok(result) => {
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
        },
        Err(err) => {
            error!("Error: {err}");
            exit(EXIT_DSC_ERROR);
        }
    }
}

pub fn config_set(configurator: &mut Configurator, format: &Option<OutputFormat>)
{
    match configurator.invoke_set(false, ErrorAction::Continue, || { /* code */ }) {
        Ok(result) => {
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
        },
        Err(err) => {
            error!("Error: {err}");
            exit(EXIT_DSC_ERROR);
        }
    }
}

pub fn config_test(configurator: &mut Configurator, format: &Option<OutputFormat>)
{
    match configurator.invoke_test(ErrorAction::Continue, || { /* code */ }) {
        Ok(result) => {
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

pub fn config(subcommand: &ConfigSubCommand, parameters: &Option<String>, stdin: &Option<String>) {
    let json_string = match subcommand {
        ConfigSubCommand::Get { document, path, .. } |
        ConfigSubCommand::Set { document, path, .. } |
        ConfigSubCommand::Test { document, path, .. } |
        ConfigSubCommand::Validate { document, path, .. } |
        ConfigSubCommand::Export { document, path, .. } => {
            let config_path = path.clone().unwrap_or_default();
            set_dscconfigroot(&config_path);
            get_input(document, stdin, path)
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
            config_get(&mut configurator, format);
        },
        ConfigSubCommand::Set { format, .. } => {
            config_set(&mut configurator, format);
        },
        ConfigSubCommand::Test { format, .. } => {
            config_test(&mut configurator, format);
        },
        ConfigSubCommand::Validate { .. } => {
            validate_config(&json_string);
        },
        ConfigSubCommand::Export { format, .. } => {
            config_export(&mut configurator, format);
        }
    }
}

/// Validate configuration.
#[allow(clippy::too_many_lines)]
pub fn validate_config(config: &str) {
    // first validate against the config schema
    let schema = match serde_json::to_value(get_schema(DscType::Configuration)) {
        Ok(schema) => schema,
        Err(e) => {
            error!("Error: Failed to convert schema to JSON: {e}");
            exit(EXIT_DSC_ERROR);
        },
    };
    let compiled_schema = match JSONSchema::compile(&schema) {
        Ok(schema) => schema,
        Err(e) => {
            error!("Error: Failed to compile schema: {e}");
            exit(EXIT_DSC_ERROR);
        },
    };
    let config_value = match serde_json::from_str(config) {
        Ok(config) => config,
        Err(e) => {
            error!("Error: Failed to parse configuration: {e}");
            exit(EXIT_INVALID_INPUT);
        },
    };
    if let Err(err) = compiled_schema.validate(&config_value) {
        let mut error = "Configuration failed validation: ".to_string();
        for e in err {
            error.push_str(&format!("\n{e} "));
        }
        error!("{error}");
        exit(EXIT_INVALID_INPUT);
    };

    let dsc = match DscManager::new() {
        Ok(dsc) => dsc,
        Err(err) => {
            error!("Error: {err}");
            exit(EXIT_DSC_ERROR);
        }
    };

    // then validate each resource
    let Some(resources) = config_value["resources"].as_array() else {
        error!("Error: Resources not specified");
        exit(EXIT_INVALID_INPUT);
    };
    for resource_block in resources {
        let type_name = resource_block["type"].as_str().unwrap_or_else(|| {
            error!("Error: Resource type not specified");
            exit(EXIT_INVALID_INPUT);
        });
        // get the actual resource
        let Some(resource) = get_resource(&dsc, type_name) else {
            error!("Error: Resource type not found");
            exit(EXIT_DSC_ERROR);
        };
        // see if the resource is command based
        if resource.implemented_as == ImplementedAs::Command {
            // if so, see if it implements validate via the resource manifest
            if let Some(manifest) = resource.manifest.clone() {
                // convert to resource_manifest
                let manifest: ResourceManifest = match serde_json::from_value(manifest) {
                    Ok(manifest) => manifest,
                    Err(e) => {
                        error!("Error: Failed to parse resource manifest: {e}");
                        exit(EXIT_INVALID_INPUT);
                    },
                };
                if manifest.validate.is_some() {
                    let result = match resource.validate(config) {
                        Ok(result) => result,
                        Err(e) => {
                            error!("Error: Failed to validate resource: {e}");
                            exit(EXIT_VALIDATION_FAILED);
                        },
                    };
                    if !result.valid {
                        let reason = result.reason.unwrap_or("No reason provided".to_string());
                        let type_name = resource.type_name.clone();
                        error!("Resource {type_name} failed validation: {reason}");
                        exit(EXIT_VALIDATION_FAILED);
                    }
                }
                else {
                    // use schema validation
                    let Ok(schema) = resource.schema() else {
                        error!("Error: Resource {type_name} does not have a schema nor supports validation");
                        exit(EXIT_VALIDATION_FAILED);
                    };
                    let schema = match serde_json::to_value(&schema) {
                        Ok(schema) => schema,
                        Err(e) => {
                            error!("Error: Failed to convert schema to JSON: {e}");
                            exit(EXIT_DSC_ERROR);
                        },
                    };
                    let compiled_schema = match JSONSchema::compile(&schema) {
                        Ok(schema) => schema,
                        Err(e) => {
                            error!("Error: Failed to compile schema: {e}");
                            exit(EXIT_DSC_ERROR);
                        },
                    };
                    let properties = resource_block["properties"].clone();
                    let validation = compiled_schema.validate(&properties);
                    if let Err(err) = validation {
                        let mut error = String::new();
                        for e in err {
                            error.push_str(&format!("{e} "));
                        }
                        error!("Error: Resource {type_name} failed validation: {error}");
                        exit(EXIT_VALIDATION_FAILED);
                    };
                }
            }
        }

    }
    exit(EXIT_SUCCESS);
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
            let mut table = Table::new(&["Type", "Version", "Methods", "Requires", "Description"]);
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
