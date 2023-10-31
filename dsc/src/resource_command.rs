// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::args::OutputFormat;
use crate::util::{EXIT_DSC_ERROR, EXIT_INVALID_ARGS, EXIT_JSON_ERROR, add_type_name_to_json, write_output};
use dsc_lib::configure::config_doc::Configuration;
use dsc_lib::configure::add_resource_export_results_to_configuration;
use dsc_lib::dscresources::invoke_result::GetResult;
use dsc_lib::dscerror::DscError;
use tracing::{error, debug};

use dsc_lib::{
    dscresources::dscresource::{Invoke, DscResource},
    DscManager
};
use std::process::exit;

pub fn get(dsc: &DscManager, resource_type: &str, input: &Option<String>, stdin: &Option<String>, format: &Option<OutputFormat>) {
    // TODO: support streaming stdin which includes resource and input
    let mut input = get_input(input, stdin);

    let Some(mut resource) = get_resource(dsc, resource_type) else {
        error!("{}", DscError::ResourceNotFound(resource_type.to_string()).to_string());
        return
    };

    debug!("resource.type_name - {} implemented_as - {:?}", resource.type_name, resource.implemented_as);
    if let Some(requires) = &resource.requires {
        input = add_type_name_to_json(input, resource.type_name.clone());
        if let Some(pr) = get_resource(dsc, requires) {
            resource = pr;
        } else {
            error!("Provider {} not found", requires);
            return;
        };
    }

    match resource.get(input.as_str()) {
        Ok(result) => {
            // convert to json
            let json = match serde_json::to_string(&result) {
                Ok(json) => json,
                Err(err) => {
                    error!("JSON Error: {err}");
                    exit(EXIT_JSON_ERROR);
                }
            };
            write_output(&json, format);
        }
        Err(err) => {
            error!("Error: {err}");
            exit(EXIT_DSC_ERROR);
        }
    }
}

pub fn get_all(dsc: &DscManager, resource_type: &str, _input: &Option<String>, _stdin: &Option<String>, format: &Option<OutputFormat>) {
    let Some(resource) = get_resource(dsc, resource_type) else {
        error!("{}", DscError::ResourceNotFound(resource_type.to_string()).to_string());
        return
    };
    debug!("resource.type_name - {} implemented_as - {:?}", resource.type_name, resource.implemented_as);
    let export_result = match resource.export() {
        Ok(export) => { export }
        Err(err) => {
            error!("Error: {err}");
            exit(EXIT_DSC_ERROR);
        }
    };

    for instance in export_result.actual_state
    {
        let get_result = GetResult {
            actual_state: instance.clone(),
        };

        let json = match serde_json::to_string(&get_result) {
            Ok(json) => json,
            Err(err) => {
                error!("JSON Error: {err}");
                exit(EXIT_JSON_ERROR);
            }
        };
        write_output(&json, format);
    }
}

/// Set operation.
///
/// # Panics
///
/// Will panic if provider-based resource is not found.
///
pub fn set(dsc: &DscManager, resource_type: &str, input: &Option<String>, stdin: &Option<String>, format: &Option<OutputFormat>) {
    let mut input = get_input(input, stdin);
    if input.is_empty() {
        error!("Error: Input is empty");
        exit(EXIT_INVALID_ARGS);
    }

    let Some(mut resource) = get_resource(dsc, resource_type) else {
        error!("{}", DscError::ResourceNotFound(resource_type.to_string()).to_string());
        return
    };

    debug!("resource.type_name - {} implemented_as - {:?}", resource.type_name, resource.implemented_as);

    if let Some(requires) = &resource.requires {
        input = add_type_name_to_json(input, resource.type_name.clone());
        if let Some(pr) = get_resource(dsc, requires) {
            resource = pr;
        } else {
            error!("Provider {} not found", requires);
            return;
        };
    }

    match resource.set(input.as_str(), true) {
        Ok(result) => {
            // convert to json
            let json = match serde_json::to_string(&result) {
                Ok(json) => json,
                Err(err) => {
                    error!("JSON Error: {err}");
                    exit(EXIT_JSON_ERROR);
                }
            };
            write_output(&json, format);
        }
        Err(err) => {
            error!("Error: {err}");
            exit(EXIT_DSC_ERROR);
        }
    }
}

/// Test operation.
///
/// # Panics
///
/// Will panic if provider-based resource is not found.
///
pub fn test(dsc: &DscManager, resource_type: &str, input: &Option<String>, stdin: &Option<String>, format: &Option<OutputFormat>) {
    let mut input = get_input(input, stdin);
    let Some(mut resource) = get_resource(dsc, resource_type) else {
        error!("{}", DscError::ResourceNotFound(resource_type.to_string()).to_string());
        return
    };

    debug!("resource.type_name - {} implemented_as - {:?}", resource.type_name, resource.implemented_as);

    if let Some(requires) = &resource.requires {
        input = add_type_name_to_json(input, resource.type_name.clone());
        if let Some(pr) = get_resource(dsc, requires) {
            resource = pr;
        } else {
            error!("Provider {} not found", requires);
            return;
        };
    }

    match resource.test(input.as_str()) {
        Ok(result) => {
            // convert to json
            let json = match serde_json::to_string(&result) {
                Ok(json) => json,
                Err(err) => {
                    error!("JSON Error: {err}");
                    exit(EXIT_JSON_ERROR);
                }
            };
            write_output(&json, format);
        }
        Err(err) => {
            error!("Error: {err}");
            exit(EXIT_DSC_ERROR);
        }
    }
}

pub fn schema(dsc: &DscManager, resource_type: &str, format: &Option<OutputFormat>) {
    let Some(resource) = get_resource(dsc, resource_type) else {
        error!("{}", DscError::ResourceNotFound(resource_type.to_string()).to_string());
        return
    };
    match resource.schema() {
        Ok(json) => {
            // verify is json
            match serde_json::from_str::<serde_json::Value>(json.as_str()) {
                Ok(_) => (),
                Err(err) => {
                    error!("Error: {err}");
                    exit(EXIT_JSON_ERROR);
                }
            };
            write_output(&json, format);
        }
        Err(err) => {
            error!("Error: {err}");
            exit(EXIT_DSC_ERROR);
        }
    }
}

pub fn export(dsc: &mut DscManager, resource_type: &str, format: &Option<OutputFormat>) {
    let Some(dsc_resource) = get_resource(dsc, resource_type) else {
        error!("{}", DscError::ResourceNotFound(resource_type.to_string()).to_string());
        return
    };

    let mut conf = Configuration::new();

    if let Err(err) = add_resource_export_results_to_configuration(dsc_resource, &mut conf) {
        error!("Error: {err}");
        exit(EXIT_DSC_ERROR);
    }

    let json = match serde_json::to_string(&conf) {
        Ok(json) => json,
        Err(err) => {
            error!("JSON Error: {err}");
            exit(EXIT_JSON_ERROR);
        }
    };
    write_output(&json, format);
}

#[must_use]
pub fn get_resource<'a>(dsc: &'a DscManager, resource: &str) -> Option<&'a DscResource> {
    //TODO: add dinamically generated resource to dsc
    dsc.find_resource(String::from(resource).to_lowercase().as_str())
}

fn get_input(input: &Option<String>, stdin: &Option<String>) -> String {
    let input = match (input, stdin) {
        (Some(_input), Some(_stdin)) => {
            error!("Error: Cannot specify both --input and stdin");
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
                            error!("Error: Cannot convert YAML to JSON: {err}");
                            exit(EXIT_INVALID_ARGS);
                        }
                    }
                },
                Err(err) => {
                    if input.contains('{') {
                        error!("Error: Input is not valid JSON: {json_err}");
                    }
                    else {
                        error!("Error: Input is not valid YAML: {err}");
                    }
                    exit(EXIT_INVALID_ARGS);
                }
            }
        }
    }
}
