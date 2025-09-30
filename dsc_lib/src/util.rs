// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::dscerror::DscError;
use rust_i18n::t;
use serde_json::Value;
use std::fs;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use std::path::Path;
use std::env;
use tracing::debug;

pub struct DscSettingValue {
    pub setting:  Value,
    pub policy: Value,
}

impl Default for DscSettingValue {
    fn default() -> DscSettingValue {
        DscSettingValue {
            setting: Value::Null,
            policy: Value::Null,
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

/// Converts a wildcard string to a regex pattern.
///
/// # Arguments
///
/// * `wildcard` - A string slice that holds the wildcard pattern.
///
/// # Returns
/// A string that holds the regex pattern.
#[must_use]
pub fn convert_wildcard_to_regex(wildcard: &str) -> String {
    let mut regex = wildcard.to_string().replace('.', "\\.").replace('?', ".").replace('*', ".*?");
    regex.insert(0, '^');
    regex.push('$');
    regex
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_wildcard_to_regex() {
        let wildcard = "*";
        let regex = convert_wildcard_to_regex(wildcard);
        assert_eq!(regex, "^.*?$");

        let wildcard = "File";
        let regex = convert_wildcard_to_regex(wildcard);
        assert_eq!(regex, "^File$");

        let wildcard = "r*";
        let regex = convert_wildcard_to_regex(wildcard);
        assert_eq!(regex, "^r.*?$");
    }
}

/// Will search setting files for the specified setting.
/// Performance implication: Use this function economically as every call opens/reads several config files.
/// TODO: cache the config
///
/// # Arguments
///
/// * `value_name` - The name of the setting.
///
/// # Errors
///
/// Will return `Err` if could not find requested setting.
pub fn get_setting(value_name: &str) -> Result<DscSettingValue, DscError> {

    const SETTINGS_FILE_NAME: &str = "dsc.settings.json";
    // Note that default settings file has root nodes as settings schema version that is specific to this version of dsc
    const DEFAULT_SETTINGS_FILE_NAME: &str = "dsc_default.settings.json";
    const DEFAULT_SETTINGS_SCHEMA_VERSION: &str = "1";

    let mut result: DscSettingValue = DscSettingValue::default();
    let mut settings_file_path : PathBuf;

    if let Some(exe_home) = get_exe_path()?.parent() {
        // First, get setting from the default settings file
        settings_file_path = exe_home.join(DEFAULT_SETTINGS_FILE_NAME);
        if let Ok(v) = load_value_from_json(&settings_file_path, DEFAULT_SETTINGS_SCHEMA_VERSION) {
            if let Some(n) = v.get(value_name) {
                result.setting = n.clone();
                debug!("{}", t!("util.foundSetting", name = value_name, path = settings_file_path.to_string_lossy()));
            }
        } else {
            debug!("{}", t!("util.notFoundSetting", name = value_name, path = settings_file_path.to_string_lossy()));
        }

        // Second, get setting from the active settings file overwriting previous value
        settings_file_path = exe_home.join(SETTINGS_FILE_NAME);
        if let Ok(v) = load_value_from_json(&settings_file_path, value_name) {
            result.setting = v;
            debug!("{}", t!("util.foundSetting", name = value_name, path = settings_file_path.to_string_lossy()));
        } else {
            debug!("{}", t!("util.notFoundSetting", name = value_name, path = settings_file_path.to_string_lossy()));
        }
    } else {
        debug!("{}", t!("util.failedToGetExePath"));
    }

    // Third, get setting from the policy
    settings_file_path = PathBuf::from(get_settings_policy_file_path());
    if let Ok(v) = load_value_from_json(&settings_file_path, value_name) {
        result.policy = v;
        debug!("{}", t!("util.foundSetting", name = value_name, path = settings_file_path.to_string_lossy()));
    } else {
        debug!("{}", t!("util.notFoundSetting", name = value_name, path = settings_file_path.to_string_lossy()));
    }

    if (result.setting == serde_json::Value::Null) && (result.policy == serde_json::Value::Null) {
        return Err(DscError::NotSupported(t!("util.settingNotFound", name = value_name).to_string()));
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

/// Gets path to the current dsc process.
/// If dsc is started using a symlink, this functon returns target of the symlink.
///
/// # Errors
///
/// Will return `Err` if path to the current exe can't be retrived.
pub fn get_exe_path() -> Result<PathBuf, DscError> {
    if let Ok(exe) = env::current_exe() {
        if let Ok(target_path) = fs::read_link(exe.clone()) {
            return Ok(target_path);
        }

        return Ok(exe);
    }

    Err(DscError::NotSupported(t!("util.failedToGetExePath").to_string()))
}

#[cfg(target_os = "windows")]
fn get_settings_policy_file_path() -> String
{
    // $env:ProgramData+"\dsc\dsc.settings.json"
    // This location is writable only by admins, but readable by all users
    let Ok(local_program_data_path) = std::env::var("ProgramData") else { return String::new(); };
    Path::new(&local_program_data_path).join("dsc").join("dsc.settings.json").display().to_string()
}

#[cfg(not(target_os = "windows"))]
fn get_settings_policy_file_path() -> String
{
    // "/etc/dsc/dsc.settings.json"
    // This location is writable only by admins, but readable by all users
    Path::new("/etc").join("dsc").join("dsc.settings.json").display().to_string()
}

#[macro_export]
macro_rules! locked_is_empty {
    ($lockable:expr) => {{
        $lockable.lock().unwrap().is_empty()
    }};
}

#[macro_export]
macro_rules! locked_extend {
    ($lockable:expr, $items:expr) => {{
        $lockable.lock().unwrap().extend($items);
    }};
}

#[macro_export]
macro_rules! locked_clone {
    ($lockable:expr) => {{
        $lockable.lock().unwrap().clone()
    }};
}

#[macro_export]
macro_rules! locked_get {
    ($lockable:expr, $key:expr) => {{
        if let Some(v) = $lockable.lock().unwrap().get($key) {
            Some(v.clone())
        } else {
            None
        }
    }};
}
