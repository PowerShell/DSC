// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::args::OutputFormat;
use crate::util::{EXIT_DSC_ERROR, EXIT_INVALID_ARGS, EXIT_JSON_ERROR, add_type_name_to_json, write_output};
use dsc_lib::configure::config_doc::Configuration;
use dsc_lib::configure::add_resource_export_results_to_configuration;
use dsc_lib::dscresources::invoke_result::{GetResult, ResourceGetResponse};
use dsc_lib::dscerror::DscError;
use tracing::{error, debug};

use dsc_lib::{
    dscresources::dscresource::{Invoke, DscResource},
    DscManager
};
use std::process::exit;

pub fn get(dsc: &DscManager, resource_type: &str, mut input: String, format: &Option<OutputFormat>) {
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

pub fn get_all(dsc: &DscManager, resource_type: &str, format: &Option<OutputFormat>) {
    let mut input = String::new();
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
            error!("Provider '{}' not found", requires);
            return;
        };
    }

    let export_result = match resource.export(&input) {
        Ok(export) => { export }
        Err(err) => {
            error!("Error: {err}");
            exit(EXIT_DSC_ERROR);
        }
    };

    for instance in export_result.actual_state
    {
        let get_result = GetResult::Resource(ResourceGetResponse {
            actual_state: instance.clone(),
        });

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
pub fn set(dsc: &DscManager, resource_type: &str, mut input: String, format: &Option<OutputFormat>) {
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
pub fn test(dsc: &DscManager, resource_type: &str, mut input: String, format: &Option<OutputFormat>) {
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
    let mut input = String::new();
    let Some(dsc_resource) = get_resource(dsc, resource_type) else {
        error!("{}", DscError::ResourceNotFound(resource_type.to_string()).to_string());
        return
    };

    let mut provider_resource: Option<&DscResource> = None;
    if let Some(requires) = &dsc_resource.requires {
        input = add_type_name_to_json(input, dsc_resource.type_name.clone());
        if let Some(pr) = get_resource(dsc, requires) {
            provider_resource = Some(pr);
        } else {
            error!("Provider '{}' not found", requires);
            return;
        };
    }

    let mut conf = Configuration::new();

    if let Err(err) = add_resource_export_results_to_configuration(dsc_resource, provider_resource, &mut conf, &input) {
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
