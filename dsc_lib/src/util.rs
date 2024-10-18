// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::dscerror::DscError;
use serde_json::Value;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use std::path::Path;
use std::env;
use tracing::debug;

pub struct DscSettingValue {
    pub setting:  serde_json::Value,
    pub policy: serde_json::Value,
}

impl Default for DscSettingValue {
    fn default() -> DscSettingValue {
        DscSettingValue {
            setting: serde_json::Value::Null,
            policy: serde_json::Value::Null,
        }
    }
}

/// Return JSON string whether the input is JSON or YAML
///
/// # Arguments
///
/// * `value` - A string slice that holds the input value
///
/// # Returns
///
/// A string that holds the JSON value
///
/// # Errors
///
/// This function will return an error if the input value is not valid JSON or YAML
pub fn parse_input_to_json(value: &str) -> Result<String, DscError> {
    match serde_json::from_str(value) {
        Ok(json) => Ok(json),
        Err(_) => {
            match serde_yaml::from_str::<Value>(value) {
                Ok(yaml) => {
                    match serde_json::to_value(yaml) {
                        Ok(json) => Ok(json.to_string()),
                        Err(err) => {
                            Err(DscError::Json(err))
                        }
                    }
                },
                Err(err) => {
                    Err(DscError::Yaml(err))
                }
            }
        }
    }
}

/// Will search setting files for the specified setting.
///
/// # Arguments
///
/// * `value_name` - The name of the setting.
///
/// # Errors
///
/// Will return `Err` if could not find requested setting.
pub fn get_setting(value_name: &str) -> Result<DscSettingValue, DscError> {

    const SETTINGS_FILE_NAME: &str = "settings.dsc.json";
    // Note that default settings file name has a version that is specific to this version of dsc
    const DEFAULT_SETTINGS_FILE_NAME: &str = "default_settings.v1.dsc.json";

    let mut result: DscSettingValue = DscSettingValue::default();
    let mut settings_file_path : PathBuf;

    if let Some(exe_home) = env::current_exe()?.parent() {
        // First, get setting from the default settings file
        settings_file_path = exe_home.join(DEFAULT_SETTINGS_FILE_NAME);
        if let Ok(v) = load_value_from_json(&settings_file_path, value_name) {
            result.setting = v;
            debug!("Found setting '{}' in {}", &value_name, settings_file_path.to_string_lossy());
        } else {
            debug!("Did not find setting '{}' in {}", &value_name, settings_file_path.to_string_lossy());
        }

        // Second, get setting from the active settings file overwriting previous value
        settings_file_path = exe_home.join(SETTINGS_FILE_NAME);
        if let Ok(v) = load_value_from_json(&settings_file_path, value_name) {
            result.setting = v;
            debug!("Found setting '{}' in {}", &value_name, settings_file_path.to_string_lossy());
        } else {
            debug!("Did not find setting '{}' in {}", &value_name, settings_file_path.to_string_lossy());
        }
    } else {
        debug!("Can't get dsc executable path");
    }

    // Third, get setting from the policy
    settings_file_path = PathBuf::from(get_settings_policy_file_path());
    if let Ok(v) = load_value_from_json(&settings_file_path, value_name) {
        result.policy = v;
        debug!("Found setting '{}' in {}", &value_name, settings_file_path.to_string_lossy());
    } else {
        debug!("Did not find setting '{}' in {}", &value_name, settings_file_path.to_string_lossy());
    }

    if (result.setting == serde_json::Value::Null) &&
       (result.policy == serde_json::Value::Null) {
        return Err(DscError::NotSupported(format!("Could not find '{value_name}' in settings").to_string()));
    }

    Ok(result)
}

fn load_value_from_json(path: &PathBuf, value_name: &str) -> Result<serde_json::Value, DscError> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let root: serde_json::Value = match serde_json::from_reader(reader) {
        Ok(j) => j,
        Err(err) => {
            return Err(DscError::Json(err));
        }
    };

    if let Some(r) = root.as_object() {
        for (key, value) in r {
            if *key == value_name {
                return Ok(value.clone())
            }
        }
    }

    Err(DscError::NotSupported(value_name.to_string()))
}

#[cfg(target_os = "windows")]
fn get_settings_policy_file_path() -> String
{
    // $env:ProgramData+"\dsc\settings.dsc.json"
    // This location is writable only by admins, but readable by all users
    let Ok(local_program_data_path) = std::env::var("ProgramData") else { return String::new(); };
    Path::new(&local_program_data_path).join("dsc").join("settings.dsc.json").display().to_string()
}

#[cfg(not(target_os = "windows"))]
fn get_settings_policy_file_path() -> String
{
    // "/etc/.dsc/settings.dsc.json"
    // This location is writable only by admins, but readable by all users
    Path::new("/etc").join(".dsc").join("settings.dsc.json").display().to_string()
}
