// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use dsc_lib::configure::config_doc::Configuration;
use dsc_lib::util::parse_input_to_json;
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
/// A tuple containing the path to the parameters file specified in the Include input and the content of
/// the file as a JSON string.
///
/// # Errors
///
/// This function will return an error if the Include input is not valid JSON, if the file
/// specified in the Include input cannot be read, or if the content of the file cannot be
/// deserialized as YAML or JSON.
pub fn get_config(input: &str) -> Result<(Option<String>, String), String> {
    debug!("Processing Include input");

    // deserialize the Include input
    let include = match serde_json::from_str::<Include>(input) {
        Ok(include) => include,
        Err(err) => {
            return Err(format!("Error: Failed to deserialize Include input: {err}"));
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
                    return Err(format!("Error: Failed to read file '{include_path:?}': {err}"));
                }
            }
        },
        Err(err) => {
            return Err(format!("Error: Failed to open included file '{include_path:?}': {err}"));
        }
    }
    // convert the buffer to a string
    let include_content = match String::from_utf8(buffer) {
        Ok(input) => input,
        Err(err) => {
            return Err(format!("Error: Invalid UTF-8 sequence in included file '{include_path:?}': {err}"));
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
                    return Err(format!("Error: Failed to read the configuration file '{include_path:?}' as YAML or JSON: {err}"));
                }
            }
        }
    };

    // serialize the Configuration as JSON
    let config_json = match serde_json::to_string(&configuration) {
        Ok(json) => json,
        Err(err) => {
            return Err(format!("Error: JSON Error: {err}"));
        }
    };

    let parameters = if let Some(parameters_file) = include.parameters_file {
        // combine the path with DSC_CONFIG_ROOT
        let parameters_file = normalize_path(Path::new(&parameters_file))?;
        info!("Resolving parameters from file '{parameters_file:?}'");
        match std::fs::read_to_string(&parameters_file) {
            Ok(parameters) => {
                let parameters_json = match parse_input_to_json(&parameters) {
                    Ok(json) => json,
                    Err(err) => {
                        return Err(format!("Failed to parse parameters file '{parameters_file:?}' to JSON: {err}"));
                    }
                };
                Some(parameters_json)
            },
            Err(err) => {
                return Err(format!("Failed to resolve parameters file '{parameters_file:?}': {err}"));
            }
        }
    } else {
        debug!("No parameters file found");
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
            return Err(format!("Error: Include path must not contain '..': {path:?}"));
        }

        // use DSC_CONFIG_ROOT env var as current directory
        let current_directory = match std::env::var(DSC_CONFIG_ROOT) {
            Ok(current_directory) => current_directory,
            Err(_err) => {
                // use current working directory
                match std::env::current_dir() {
                    Ok(current_directory) => current_directory.to_string_lossy().into_owned(),
                    Err(err) => {
                        return Err(format!("Error: Failed to get current directory: {err}"));
                    }
                }
            }
        };

        // combine the current directory with the Include path
        Ok(Path::new(&current_directory).join(path))
    }
}
