// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::args::{GetOutputFormat, OutputFormat};
use crate::util::{write_object, EXIT_DSC_ERROR, EXIT_DSC_RESOURCE_NOT_FOUND, EXIT_INVALID_ARGS, EXIT_JSON_ERROR};
use dsc_lib::configure::add_resource_export_results_to_configuration;
use dsc_lib::configure::config_doc::{Configuration, ExecutionKind};
use dsc_lib::dscerror::DscError;
use dsc_lib::dscresources::{
    invoke_result::{GetResult, ResourceGetResponse},
    resource_manifest::Kind,
};
use rust_i18n::t;
use tracing::{debug, error};

use dsc_lib::{
    dscresources::dscresource::{DscResource, Invoke},
    DscManager,
};
use std::process::exit;

pub fn get(
    dsc: &mut DscManager,
    resource_type: &str,
    version: Option<&str>,
    input: &str,
    format: Option<&GetOutputFormat>,
) {
    let Some(resource) = get_resource(dsc, resource_type, version) else {
        error!(
            "{}",
            DscError::ResourceNotFound(resource_type.to_string(), version.unwrap_or("").to_string()).to_string()
        );
        exit(EXIT_DSC_RESOURCE_NOT_FOUND);
    };

    debug!(
        "{} {} {:?}",
        resource.type_name,
        t!("resource_command.implementedAs"),
        resource.implemented_as
    );
    if resource.kind == Kind::Adapter {
        error!(
            "{}: {}",
            t!("resource_command.invalidOperationOnAdapter"),
            resource.type_name
        );
        exit(EXIT_DSC_ERROR);
    }

    match resource.get(input) {
        Ok(result) => {
            if let GetResult::Resource(response) = &result {
                if format == Some(&GetOutputFormat::PassThrough) {
                    let json = match serde_json::to_string(&response.actual_state) {
                        Ok(json) => json,
                        Err(err) => {
                            error!("{}", t!("resource_command.jsonError", err = err));
                            exit(EXIT_JSON_ERROR);
                        }
                    };
                    write_object(&json, Some(&OutputFormat::Json), false);
                    return;
                }
            }

            // convert to json
            let json = match serde_json::to_string(&result) {
                Ok(json) => json,
                Err(err) => {
                    error!("{}", t!("resource_command.jsonError", err = err));
                    exit(EXIT_JSON_ERROR);
                }
            };
            let format = match format {
                Some(&GetOutputFormat::PrettyJson) => Some(&OutputFormat::PrettyJson),
                Some(&GetOutputFormat::Yaml) => Some(&OutputFormat::Yaml),
                None => None,
                _ => Some(&OutputFormat::Json),
            };
            write_object(&json, format, false);
        }
        Err(err) => {
            error!("{err}");
            exit(EXIT_DSC_ERROR);
        }
    }
}

pub fn get_all(dsc: &mut DscManager, resource_type: &str, version: Option<&str>, format: Option<&GetOutputFormat>) {
    let input = String::new();
    let Some(resource) = get_resource(dsc, resource_type, version) else {
        error!(
            "{}",
            DscError::ResourceNotFound(resource_type.to_string(), version.unwrap_or("").to_string()).to_string()
        );
        exit(EXIT_DSC_RESOURCE_NOT_FOUND);
    };

    debug!(
        "{} {} {:?}",
        resource.type_name,
        t!("resource_command.implementedAs"),
        resource.implemented_as
    );
    if resource.kind == Kind::Adapter {
        error!(
            "{}: {}",
            t!("resource_command.invalidOperationOnAdapter"),
            resource.type_name
        );
        exit(EXIT_DSC_ERROR);
    }

    let export_result = match resource.export(&input) {
        Ok(export) => export,
        Err(err) => {
            error!("{err}");
            exit(EXIT_DSC_ERROR);
        }
    };

    if format == Some(&GetOutputFormat::JsonArray) {
        let json = match serde_json::to_string(&export_result.actual_state) {
            Ok(json) => json,
            Err(err) => {
                error!("{}", t!("resource_command.jsonError", err = err));
                exit(EXIT_JSON_ERROR);
            }
        };
        write_object(&json, Some(&OutputFormat::Json), false);
        return;
    }

    let mut include_separator = false;
    for instance in export_result.actual_state {
        let get_result = GetResult::Resource(ResourceGetResponse {
            actual_state: instance.clone(),
        });

        let json = match serde_json::to_string(&get_result) {
            Ok(json) => json,
            Err(err) => {
                error!("{}", t!("resource_command.jsonError", err = err));
                exit(EXIT_JSON_ERROR);
            }
        };
        let format = match format {
            Some(&GetOutputFormat::PrettyJson) => Some(&OutputFormat::PrettyJson),
            Some(&GetOutputFormat::Yaml) => Some(&OutputFormat::Yaml),
            None => None,
            _ => Some(&OutputFormat::Json),
        };
        write_object(&json, format, include_separator);
        include_separator = true;
    }
}

pub fn set(
    dsc: &mut DscManager,
    resource_type: &str,
    version: Option<&str>,
    input: &str,
    format: Option<&OutputFormat>,
) {
    if input.is_empty() {
        error!("{}", t!("resource_command.setInputEmpty"));
        exit(EXIT_INVALID_ARGS);
    }

    let Some(resource) = get_resource(dsc, resource_type, version) else {
        error!(
            "{}",
            DscError::ResourceNotFound(resource_type.to_string(), version.unwrap_or("").to_string()).to_string()
        );
        exit(EXIT_DSC_RESOURCE_NOT_FOUND);
    };

    debug!(
        "{} {} {:?}",
        resource.type_name,
        t!("resource_command.implementedAs"),
        resource.implemented_as
    );
    if resource.kind == Kind::Adapter {
        error!(
            "{}: {}",
            t!("resource_command.invalidOperationOnAdapter"),
            resource.type_name
        );
        exit(EXIT_DSC_ERROR);
    }

    match resource.set(input, true, &ExecutionKind::Actual) {
        Ok(result) => {
            // convert to json
            let json = match serde_json::to_string(&result) {
                Ok(json) => json,
                Err(err) => {
                    error!("{}", t!("resource_command.jsonError", err = err));
                    exit(EXIT_JSON_ERROR);
                }
            };
            write_object(&json, format, false);
        }
        Err(err) => {
            error!("{err}");
            exit(EXIT_DSC_ERROR);
        }
    }
}

pub fn test(
    dsc: &mut DscManager,
    resource_type: &str,
    version: Option<&str>,
    input: &str,
    format: Option<&OutputFormat>,
) {
    if input.is_empty() {
        error!("{}", t!("resource_command.testInputEmpty"));
        exit(EXIT_INVALID_ARGS);
    }

    let Some(resource) = get_resource(dsc, resource_type, version) else {
        error!(
            "{}",
            DscError::ResourceNotFound(resource_type.to_string(), version.unwrap_or("").to_string()).to_string()
        );
        exit(EXIT_DSC_RESOURCE_NOT_FOUND);
    };

    debug!(
        "{} {} {:?}",
        resource.type_name,
        t!("resource_command.implementedAs"),
        resource.implemented_as
    );
    if resource.kind == Kind::Adapter {
        error!(
            "{}: {}",
            t!("resource_command.invalidOperationOnAdapter"),
            resource.type_name
        );
        exit(EXIT_DSC_ERROR);
    }

    match resource.test(input) {
        Ok(result) => {
            // convert to json
            let json = match serde_json::to_string(&result) {
                Ok(json) => json,
                Err(err) => {
                    error!("JSON: {err}");
                    exit(EXIT_JSON_ERROR);
                }
            };
            write_object(&json, format, false);
        }
        Err(err) => {
            error!("{err}");
            exit(EXIT_DSC_ERROR);
        }
    }
}

pub fn delete(dsc: &mut DscManager, resource_type: &str, version: Option<&str>, input: &str) {
    let Some(resource) = get_resource(dsc, resource_type, version) else {
        error!(
            "{}",
            DscError::ResourceNotFound(resource_type.to_string(), version.unwrap_or("").to_string()).to_string()
        );
        exit(EXIT_DSC_RESOURCE_NOT_FOUND);
    };

    debug!(
        "{} {} {:?}",
        resource.type_name,
        t!("resource_command.implementedAs"),
        resource.implemented_as
    );
    if resource.kind == Kind::Adapter {
        error!(
            "{}: {}",
            t!("resource_command.invalidOperationOnAdapter"),
            resource.type_name
        );
        exit(EXIT_DSC_ERROR);
    }

    match resource.delete(input) {
        Ok(()) => {}
        Err(err) => {
            error!("{err}");
            exit(EXIT_DSC_ERROR);
        }
    }
}

pub fn schema(dsc: &mut DscManager, resource_type: &str, version: Option<&str>, format: Option<&OutputFormat>) {
    let Some(resource) = get_resource(dsc, resource_type, version) else {
        error!(
            "{}",
            DscError::ResourceNotFound(resource_type.to_string(), version.unwrap_or("").to_string()).to_string()
        );
        exit(EXIT_DSC_RESOURCE_NOT_FOUND);
    };
    if resource.kind == Kind::Adapter {
        error!(
            "{}: {}",
            t!("resource_command.invalidOperationOnAdapter"),
            resource.type_name
        );
        exit(EXIT_DSC_ERROR);
    }

    match resource.schema() {
        Ok(json) => {
            // verify is json
            match serde_json::from_str::<serde_json::Value>(json.as_str()) {
                Ok(_) => (),
                Err(err) => {
                    error!("{err}");
                    exit(EXIT_JSON_ERROR);
                }
            }
            write_object(&json, format, false);
        }
        Err(err) => {
            error!("{err}");
            exit(EXIT_DSC_ERROR);
        }
    }
}

pub fn export(
    dsc: &mut DscManager,
    resource_type: &str,
    version: Option<&str>,
    input: &str,
    format: Option<&OutputFormat>,
) {
    let Some(dsc_resource) = get_resource(dsc, resource_type, version) else {
        error!(
            "{}",
            DscError::ResourceNotFound(resource_type.to_string(), version.unwrap_or("").to_string()).to_string()
        );
        exit(EXIT_DSC_RESOURCE_NOT_FOUND);
    };

    if dsc_resource.kind == Kind::Adapter {
        error!(
            "{}: {}",
            t!("resource_command.invalidOperationOnAdapter"),
            dsc_resource.type_name
        );
        exit(EXIT_DSC_ERROR);
    }

    let mut conf = Configuration::new();
    if let Err(err) = add_resource_export_results_to_configuration(dsc_resource, &mut conf, input) {
        error!("{err}");
        exit(EXIT_DSC_ERROR);
    }

    let json = match serde_json::to_string(&conf) {
        Ok(json) => json,
        Err(err) => {
            error!("JSON: {err}");
            exit(EXIT_JSON_ERROR);
        }
    };
    write_object(&json, format, false);
}

#[must_use]
pub fn get_resource<'a>(dsc: &'a mut DscManager, resource: &str, version: Option<&str>) -> Option<&'a DscResource> {
    //TODO: add dynamically generated resource to dsc
    dsc.find_resource(resource, version)
}
