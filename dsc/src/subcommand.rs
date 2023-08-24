// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::args::{ConfigSubCommand, DscType, OutputFormat, ResourceSubCommand};
use crate::resource_command::{get_resource, self};
use crate::tablewriter::Table;
use crate::util::{EXIT_DSC_ERROR, EXIT_INVALID_ARGS, EXIT_INVALID_INPUT, EXIT_JSON_ERROR, EXIT_SUCCESS, EXIT_VALIDATION_FAILED, get_schema, serde_json_value_to_string, write_output};

use atty::Stream;
use dsc_lib::{
    configure::{Configurator, ErrorAction},
    DscManager,
    dscresources::dscresource::{ImplementedAs, Invoke},
    dscresources::resource_manifest::ResourceManifest,
};
use jsonschema::{JSONSchema, ValidationError};
use serde_yaml::Value;
use std::process::exit;

pub fn config_get(configurator: Configurator, format: &Option<OutputFormat>)
{
    match configurator.invoke_get(ErrorAction::Continue, || { /* code */ }) {
        Ok(result) => {
            let json = match serde_json::to_string(&result) {
                Ok(json) => json,
                Err(err) => {
                    eprintln!("JSON Error: {err}");
                    exit(EXIT_JSON_ERROR);
                }
            };
            write_output(&json, format);
            if result.had_errors {
                exit(EXIT_DSC_ERROR);
            }
        },
        Err(err) => {
            eprintln!("Error: {err}");
            exit(EXIT_DSC_ERROR);
        }
    }
}

pub fn config_set(configurator: Configurator, format: &Option<OutputFormat>)
{
    match configurator.invoke_set(ErrorAction::Continue, || { /* code */ }) {
        Ok(result) => {
            let json = match serde_json::to_string(&result) {
                Ok(json) => json,
                Err(err) => {
                    eprintln!("JSON Error: {err}");
                    exit(EXIT_JSON_ERROR);
                }
            };
            write_output(&json, format);
            if result.had_errors {
                exit(EXIT_DSC_ERROR);
            }
        },
        Err(err) => {
            eprintln!("Error: {err}");
            exit(EXIT_DSC_ERROR);
        }
    }
}

pub fn config_test(configurator: Configurator, format: &Option<OutputFormat>)
{
    match configurator.invoke_test(ErrorAction::Continue, || { /* code */ }) {
        Ok(result) => {
            let json = match serde_json::to_string(&result) {
                Ok(json) => json,
                Err(err) => {
                    eprintln!("JSON Error: {err}");
                    exit(EXIT_JSON_ERROR);
                }
            };
            write_output(&json, format);
            if result.had_errors {
                exit(EXIT_DSC_ERROR);
            }
        },
        Err(err) => {
            eprintln!("Error: {err}");
            exit(EXIT_DSC_ERROR);
        }
    }
}

pub fn config(subcommand: &ConfigSubCommand, format: &Option<OutputFormat>, stdin: &Option<String>) {
    if stdin.is_none() {
        eprintln!("Configuration must be piped to STDIN");
        exit(EXIT_INVALID_ARGS);
    }

    let json: serde_json::Value = match serde_json::from_str(stdin.as_ref().unwrap()) {
        Ok(json) => json,
        Err(_) => {
            match serde_yaml::from_str::<Value>(stdin.as_ref().unwrap()) {
                Ok(yaml) => {
                    match serde_json::to_value(yaml) {
                        Ok(json) => json,
                        Err(err) => {
                            eprintln!("Error: Failed to convert YAML to JSON: {err}");
                            exit(EXIT_DSC_ERROR);
                        }
                    }
                },
                Err(err) => {
                    eprintln!("Error: Input is not valid JSON or YAML: {err}");
                    exit(EXIT_INVALID_INPUT);
                }
            }
        }
    };

    let json_string = serde_json_value_to_string(&json);
    let configurator = match Configurator::new(&json_string) {
        Ok(configurator) => configurator,
        Err(err) => {
            eprintln!("Error: {err}");
            exit(EXIT_DSC_ERROR);
        }
    };

    match subcommand {
        ConfigSubCommand::Get => {
            config_get(configurator, format);
        },
        ConfigSubCommand::Set => {
            config_set(configurator, format);
        },
        ConfigSubCommand::Test => {
            config_test(configurator, format);
        },
        ConfigSubCommand::Validate => {
            validate_config(&json_string);
        }
    }
}

pub fn validate_config(config: &str) {
    // first validate against the config schema
    let schema = match serde_json::to_value(get_schema(DscType::Configuration)) {
        Ok(schema) => schema,
        Err(e) => {
            eprintln!("Error: Failed to convert schema to JSON: {e}");
            exit(EXIT_DSC_ERROR);
        },
    };
    let compiled_schema = match JSONSchema::compile(&schema) {
        Ok(schema) => schema,
        Err(e) => {
            eprintln!("Error: Failed to compile schema: {e}");
            exit(EXIT_DSC_ERROR);
        },
    };
    let config_value = match serde_json::from_str(config) {
        Ok(config) => config,
        Err(e) => {
            eprintln!("Error: Failed to parse configuration: {e}");
            exit(EXIT_INVALID_INPUT);
        },
    };
    if let Err(err) = compiled_schema.validate(&config_value) {
        let mut error = "Configuration failed validation: ".to_string();
        for e in err {
            error.push_str(&format!("\n{e} "));
        }
        eprintln!("{}", error);
        exit(EXIT_INVALID_INPUT);
    };

    let mut dsc = match DscManager::new() {
        Ok(dsc) => dsc,
        Err(err) => {
            eprintln!("Error: {err}");
            exit(EXIT_DSC_ERROR);
        }
    };

    // then validate each resource
    for resource_block in config_value["resources"].as_array().unwrap().iter() {
        let type_name = resource_block["type"].as_str().unwrap_or_else(|| {
            eprintln!("Error: Resource type not specified");
            exit(EXIT_INVALID_INPUT);
        });
        // get the actual resource
        let resource = get_resource(&mut dsc, type_name);
        // see if the resource is command based
        if resource.implemented_as == ImplementedAs::Command {
            // if so, see if it implements validate via the resource manifest
            if let Some(manifest) = resource.manifest.clone() {
                // convert to resource_manifest
                let manifest: ResourceManifest = match serde_json::from_value(manifest) {
                    Ok(manifest) => manifest,
                    Err(e) => {
                        eprintln!("Error: Failed to parse resource manifest: {e}");
                        exit(EXIT_INVALID_INPUT);
                    },
                };
                if manifest.validate.is_some() {
                    let result = match resource.validate(config) {
                        Ok(result) => result,
                        Err(e) => {
                            eprintln!("Error: Failed to validate resource: {e}");
                            exit(EXIT_VALIDATION_FAILED);
                        },
                    };
                    if !result.valid {
                        let reason = result.reason.unwrap_or("No reason provided".to_string());
                        let type_name = resource.type_name;
                        eprintln!("Resource {type_name} failed validation: {reason}");
                        exit(EXIT_VALIDATION_FAILED);
                    }
                }
                else {
                    // use schema validation
                    let Ok(schema) = resource.schema() else {
                        eprintln!("Error: Resource {type_name} does not have a schema nor supports validation");
                        exit(EXIT_VALIDATION_FAILED);
                    };
                    let schema = match serde_json::to_value(&schema) {
                        Ok(schema) => schema,
                        Err(e) => {
                            eprintln!("Error: Failed to convert schema to JSON: {e}");
                            exit(EXIT_DSC_ERROR);
                        },
                    };
                    let compiled_schema = match JSONSchema::compile(&schema) {
                        Ok(schema) => schema,
                        Err(e) => {
                            eprintln!("Error: Failed to compile schema: {e}");
                            exit(EXIT_DSC_ERROR);
                        },
                    };
                    let properties = resource_block["properties"].clone();
                    let _result: Result<(), ValidationError> = match compiled_schema.validate(&properties) {
                        Ok(_) => Ok(()),
                        Err(err) => {
                            let mut error = String::new();
                            for e in err {
                                error.push_str(&format!("{e} "));
                            }

                            eprintln!("Error: Resource {type_name} failed validation: {error}");
                            exit(EXIT_VALIDATION_FAILED);
                        },
                    };
                }
            }
        }

    }
    exit(EXIT_SUCCESS);
}

pub fn resource(subcommand: &ResourceSubCommand, format: &Option<OutputFormat>, stdin: &Option<String>) {
    let mut dsc = match DscManager::new() {
        Ok(dsc) => dsc,
        Err(err) => {
            eprintln!("Error: {err}");
            exit(EXIT_DSC_ERROR);
        }
    };

    match subcommand {
        ResourceSubCommand::List { resource_name, description, tags } => {
            match dsc.initialize_discovery() {
                Ok(_) => (),
                Err(err) => {
                    eprintln!("Error: {err}");
                    exit(EXIT_DSC_ERROR);
                }
            };
            let mut write_table = false;
            let mut table = Table::new(vec!["Type", "Version", "Requires", "Description"]);
            if format.is_none() && atty::is(Stream::Stdout) {
                // write as table if fornat is not specified and interactive
                write_table = true;
            }
            for resource in dsc.find_resource(&resource_name.clone().unwrap_or_default()) {
                // if description is specified, skip if resource description does not contain it
                if description.is_some() || tags.is_some() {
                    if resource.manifest.is_none() {
                        continue;
                    }

                    let resource_manifest = match serde_json::from_value::<ResourceManifest>(resource.clone().manifest.unwrap().clone()) {
                        Ok(resource_manifest) => resource_manifest,
                        Err(err) => {
                            eprintln!("Error in manifest for {0}: {err}", resource.type_name);
                            continue;
                        }
                    };

                    if description.is_some() {
                        if resource_manifest.description.is_none() {
                            continue;
                        }

                        if !resource_manifest.description.unwrap().to_lowercase().contains(&description.as_ref().unwrap().to_lowercase()) {
                            continue;
                        }
                    }

                    // if tags is specified, skip if resource tags do not contain the tags
                    if tags.is_some() {
                        if resource_manifest.tags.is_none() {
                            continue;
                        }

                        let mut found = false;
                        for tag_to_find in tags.clone().unwrap() {
                            for tag in resource_manifest.tags.clone().unwrap() {
                                if tag.to_lowercase() == tag_to_find.to_lowercase() {
                                    found = true;
                                    break;
                                }
                            }
                        }
                        if !found {
                            continue;
                        }
                    }
                }

                if write_table {
                    table.add_row(vec![
                        resource.type_name,
                        resource.version,
                        resource.requires.unwrap_or_default(),
                        resource.description.unwrap_or_default()
                    ]);
                }
                else {
                    // convert to json
                    let json = match serde_json::to_string(&resource) {
                        Ok(json) => json,
                        Err(err) => {
                            eprintln!("JSON Error: {err}");
                            exit(EXIT_JSON_ERROR);
                        }
                    };
                    write_output(&json, format);
                    // insert newline separating instances if writing to console
                    if atty::is(Stream::Stdout) {
                        println!();
                    }
                }
            }

            if write_table {
                table.print();
            }
        },
       ResourceSubCommand::Get { resource, input } => {
            resource_command::get(&mut dsc, resource, input, stdin, format);
        },
        ResourceSubCommand::Set { resource, input } => {
            resource_command::set(&mut dsc, resource, input, stdin, format);
        },
        ResourceSubCommand::Test { resource, input } => {
            resource_command::test(&mut dsc, resource, input, stdin, format);
        },
        ResourceSubCommand::Schema { resource } => {
            resource_command::schema(&mut dsc, resource, format);
        },
        ResourceSubCommand::Export { resource} => {
            resource_command::export(&mut dsc, resource, format);
        },
    }
}
