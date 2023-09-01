// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::args::OutputFormat;
use crate::util::{EXIT_DSC_ERROR, EXIT_INVALID_ARGS, EXIT_JSON_ERROR, add_type_name_to_json, write_output};
use dsc_lib::configure::config_doc::Resource;
use dsc_lib::configure::config_doc::Configuration;
use std::collections::HashMap;

use dsc_lib::{
    dscresources::dscresource::{Invoke, DscResource},
    DscManager
};
use std::process::exit;

pub fn get(dsc: &mut DscManager, resource: &str, input: &Option<String>, stdin: &Option<String>, format: &Option<OutputFormat>) {
    // TODO: support streaming stdin which includes resource and input
    let mut input = get_input(input, stdin);
    let mut resource = get_resource(dsc, resource);
    //TODO: add to debug stream: println!("handle_resource_get - {} implemented_as - {:?}", resource.type_name, resource.implemented_as);
    if resource.requires.is_some()
    {
        input = add_type_name_to_json(input, resource.type_name);
        resource = get_resource(dsc, &resource.requires.clone().unwrap());
    }

    //TODO: add to debug stream: println!("handle_resource_get - input - {}", input);

    match resource.get(input.as_str()) {
        Ok(result) => {
            // convert to json
            let json = match serde_json::to_string(&result) {
                Ok(json) => json,
                Err(err) => {
                    eprintln!("JSON Error: {err}");
                    exit(EXIT_JSON_ERROR);
                }
            };
            write_output(&json, format);
        }
        Err(err) => {
            eprintln!("Error: {err}");
            exit(EXIT_DSC_ERROR);
        }
    }
}

pub fn set(dsc: &mut DscManager, resource: &str, input: &Option<String>, stdin: &Option<String>, format: &Option<OutputFormat>) {
    let mut input = get_input(input, stdin);
    let mut resource = get_resource(dsc, resource);

    //TODO: add to debug stream: println!("handle_resource_set - {} implemented_as - {:?}", resource.type_name, resource.implemented_as);

    if resource.requires.is_some()
    {
        input = add_type_name_to_json(input, resource.type_name);
        resource = get_resource(dsc, &resource.requires.clone().unwrap());
    }

    //TODO: add to debug stream: println!("handle_resource_get - input - {}", input);

    match resource.set(input.as_str()) {
        Ok(result) => {
            // convert to json
            let json = match serde_json::to_string(&result) {
                Ok(json) => json,
                Err(err) => {
                    eprintln!("JSON Error: {err}");
                    exit(EXIT_JSON_ERROR);
                }
            };
            write_output(&json, format);
        }
        Err(err) => {
            eprintln!("Error: {err}");
            exit(EXIT_DSC_ERROR);
        }
    }
}

pub fn test(dsc: &mut DscManager, resource: &str, input: &Option<String>, stdin: &Option<String>, format: &Option<OutputFormat>) {
    let mut input = get_input(input, stdin);
    let mut resource = get_resource(dsc, resource);

    //TODO: add to debug stream: println!("handle_resource_test - {} implemented_as - {:?}", resource.type_name, resource.implemented_as);

    if resource.requires.is_some()
    {
        input = add_type_name_to_json(input, resource.type_name);
        resource = get_resource(dsc, &resource.requires.clone().unwrap());
    }

    //TODO: add to debug stream: println!("handle_resource_test - input - {}", input);

    match resource.test(input.as_str()) {
        Ok(result) => {
            // convert to json
            let json = match serde_json::to_string(&result) {
                Ok(json) => json,
                Err(err) => {
                    eprintln!("JSON Error: {err}");
                    exit(EXIT_JSON_ERROR);
                }
            };
            write_output(&json, format);
        }
        Err(err) => {
            eprintln!("Error: {err}");
            exit(EXIT_DSC_ERROR);
        }
    }
}

pub fn schema(dsc: &mut DscManager, resource: &str, format: &Option<OutputFormat>) {
    let resource = get_resource(dsc, resource);
    match resource.schema() {
        Ok(json) => {
            // verify is json
            match serde_json::from_str::<serde_json::Value>(json.as_str()) {
                Ok(_) => (),
                Err(err) => {
                    eprintln!("Error: {err}");
                    exit(EXIT_JSON_ERROR);
                }
            };
            write_output(&json, format);
        }
        Err(err) => {
            eprintln!("Error: {err}");
            exit(EXIT_DSC_ERROR);
        }
    }
}

pub fn export(dsc: &mut DscManager, resource: &str, format: &Option<OutputFormat>) {
    let dsc_resource = get_resource(dsc, resource);

    let mut conf = Configuration::new();

    let export_result = match dsc_resource.export() {
        Ok(export) => { export }
        Err(err) => {
            eprintln!("Error: {err}");
            exit(EXIT_DSC_ERROR);
        }
    };

    for (i, instance) in export_result.actual_state.iter().enumerate()
    {
        let mut r = Resource::new();
        r.resource_type = dsc_resource.type_name.clone();
        r.name = format!("{}-{i}", r.resource_type);
        let props: HashMap<String, serde_json::Value> = serde_json::from_value(instance.clone()).unwrap();
        r.properties = Some(props);

        conf.resources.push(r);
    }

    let json = match serde_json::to_string(&conf) {
        Ok(json) => json,
        Err(err) => {
            eprintln!("JSON Error: {err}");
            exit(EXIT_JSON_ERROR);
        }
    };
    write_output(&json, format);
}

pub fn get_resource(dsc: &mut DscManager, resource: &str) -> DscResource {
    // check if resource is JSON or just a name
    match serde_json::from_str(resource) {
        Ok(resource) => resource,
        Err(err) => {
            if resource.contains('{') {
                eprintln!("Not valid resource JSON: {err}\nInput was: {resource}");
                exit(EXIT_INVALID_ARGS);
            }

            match dsc.initialize_discovery() {
                Ok(_) => (),
                Err(err) => {
                    eprintln!("Error: {err}");
                    exit(EXIT_DSC_ERROR);
                }
            };
            let resources: Vec<DscResource> = dsc.find_resource(resource).collect();
            match resources.len() {
                0 => {
                    eprintln!("Error: Resource not found: '{resource}'");
                    exit(EXIT_INVALID_ARGS);
                }
                1 => resources[0].clone(),
                _ => {
                    eprintln!("Error: Multiple resources found");
                    exit(EXIT_INVALID_ARGS);
                }
            }
        }
    }
}

fn get_input(input: &Option<String>, stdin: &Option<String>) -> String {
    let input = match (input, stdin) {
        (Some(_input), Some(_stdin)) => {
            eprintln!("Error: Cannot specify both --input and stdin");
            exit(EXIT_INVALID_ARGS);
        }
        (Some(input), None) => input.clone(),
        (None, Some(stdin)) => stdin.clone(),
        (None, None) => {
            return String::new();
        },
    };

    if input.is_empty() {
        return String::new();
    }

    match serde_json::from_str::<serde_json::Value>(&input) {
        Ok(_) => input,
        Err(json_err) => {
            match serde_yaml::from_str::<serde_yaml::Value>(&input) {
                Ok(yaml) => {
                    match serde_json::to_string(&yaml) {
                        Ok(json) => json,
                        Err(err) => {
                            eprintln!("Error: Cannot convert YAML to JSON: {err}");
                            exit(EXIT_INVALID_ARGS);
                        }
                    }
                },
                Err(err) => {
                    if input.contains('{') {
                        eprintln!("Error: Input is not valid JSON: {json_err}");
                    }
                    else {
                        eprintln!("Error: Input is not valid YAML: {err}");
                    }
                    exit(EXIT_INVALID_ARGS);
                }
            }
        }
    }
}
