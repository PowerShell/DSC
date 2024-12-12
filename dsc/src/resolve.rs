// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use dsc_lib::configure::config_doc::Configuration;
use dsc_lib::util::parse_input_to_json;
use rust_i18n::t;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::io::Read;
use std::fs::File;
use std::path::{Path, PathBuf};
use tracing::{debug, info};

use crate::util::DSC_CONFIG_ROOT;

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
pub struct Include {
    /// The path to the file to include.  Path is relative to the file containing the include
    /// and not allowed to reference parent directories.  If a configuration document is used
    /// instead of a file, then the path is relative to the current working directory.
    #[serde(rename = "configurationFile")]
    pub configuration_file: String,
    #[serde(rename = "parametersFile")]
    pub parameters_file: Option<String>,
}

/// Read the file specified in the Include input and return the content as a JSON string.
///
/// # Arguments
///
/// * `input` - The Include input as a JSON string.
///
/// # Returns
///
/// A tuple containing the contents of the parameters file as JSON and the configuration content
/// as a JSON string.
///
/// # Errors
///
/// This function will return an error if the Include input is not valid JSON, if the file
/// specified in the Include input cannot be read, or if the content of the file cannot be
/// deserialized as YAML or JSON.
pub fn get_contents(input: &str) -> Result<(Option<String>, String), String> {
    debug!("{}", t!("resolve.processingInclude"));

    // deserialize the Include input
    let include = match serde_json::from_str::<Include>(input) {
        Ok(include) => include,
        Err(err) => {
            return Err(format!("{}: {err}", t!("resolve.invalidInclude")));
        }
    };

    let include_path = normalize_path(Path::new(&include.configuration_file))?;

    // read the file specified in the Include input
    let mut buffer: Vec<u8> = Vec::new();
    match File::open(&include_path) {
        Ok(mut file) => {
            match file.read_to_end(&mut buffer) {
                Ok(_) => (),
                Err(err) => {
                    return Err(format!("{} '{include_path:?}': {err}", t!("resolve.failedToReadFile")));
                }
            }
        },
        Err(err) => {
            return Err(format!("{} '{include_path:?}': {err}", t!("resolve.failedToOpenFile")));
        }
    }
    // convert the buffer to a string
    let include_content = match String::from_utf8(buffer) {
        Ok(input) => input,
        Err(err) => {
            return Err(format!("{} '{include_path:?}': {err}", t!("resolve.invalidFileContent")));
        }
    };

    // try to deserialize the Include content as YAML first
    let configuration: Configuration = match serde_yaml::from_str(&include_content) {
        Ok(configuration) => configuration,
        Err(_err) => {
            // if that fails, try to deserialize it as JSON
            match serde_json::from_str(&include_content) {
                Ok(configuration) => configuration,
                Err(err) => {
                    return Err(format!("{} '{include_path:?}': {err}", t!("resolve.invalidFile")));
                }
            }
        }
    };

    // serialize the Configuration as JSON
    let config_json = match serde_json::to_string(&configuration) {
        Ok(json) => json,
        Err(err) => {
            return Err(format!("JSON: {err}"));
        }
    };

    let parameters = if let Some(parameters_file) = include.parameters_file {
        // combine the path with DSC_CONFIG_ROOT
        let parameters_file = normalize_path(Path::new(&parameters_file))?;
        info!("{} '{parameters_file:?}'", t!("resolve.resolvingParameters"));
        match std::fs::read_to_string(&parameters_file) {
            Ok(parameters) => {
                let parameters_json = match parse_input_to_json(&parameters) {
                    Ok(json) => json,
                    Err(err) => {
                        return Err(format!("{} '{parameters_file:?}': {err}", t!("resolve.failedParseParametersFile")));
                    }
                };
                Some(parameters_json)
            },
            Err(err) => {
                return Err(format!("{} '{parameters_file:?}': {err}", t!("resolve.failedResolveParametersFile")));
            }
        }
    } else {
        debug!("{}", t!("resolve.noParametersFile"));
        None
    };

    Ok((parameters, config_json))
}

fn normalize_path(path: &Path) -> Result<PathBuf, String> {
    if path.is_absolute() {
        Ok(path.to_path_buf())
    } else {
        // check that no components of the path are '..'
        if path.components().any(|c| c == std::path::Component::ParentDir) {
            return Err(format!("{}: {path:?}", t!("resolve.invalidPath")));
        }

        // use DSC_CONFIG_ROOT env var as current directory
        let current_directory = match std::env::var(DSC_CONFIG_ROOT) {
            Ok(current_directory) => current_directory,
            Err(_err) => {
                // use current working directory
                match std::env::current_dir() {
                    Ok(current_directory) => current_directory.to_string_lossy().into_owned(),
                    Err(err) => {
                        return Err(format!("{}: {err}", t!("resolve.failedGetCurrentDirectory")));
                    }
                }
            }
        };

        // combine the current directory with the Include path
        Ok(Path::new(&current_directory).join(path))
    }
}
