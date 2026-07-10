// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::dscerror::DscError;
use rust_i18n::t;
use serde_json::Value;
use std::{
    fs,
    fs::canonicalize,
    path::{Path, PathBuf},
    env,
};
use which::which;

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

/// Generates a resource ID from the specified type and name.
///
/// # Arguments
/// * `type_name` - The resource type in the format "namespace/type".
/// * `name` - The resource name.
///
/// # Returns
/// A string that holds the resource ID in the format "namespace/type:name".
#[must_use]
pub fn resource_id(type_name: &str, name: &str) -> String {
    let mut result = String::new();
    result.push_str(type_name);
    result.push(':');
    let encoded = urlencoding::encode(name);
    result.push_str(&encoded);
    result
}

pub fn canonicalize_which(executable: &str, cwd: Option<&Path>) -> Result<String, DscError> {
    // Use PathBuf to handle path separators robustly
    let mut executable_path = PathBuf::from(executable);
    if cfg!(target_os = "windows") && executable_path.extension().is_none() {
        executable_path.set_extension("exe");
    }
    if which(executable).is_err() {
        if let Some(cwd_path) = cwd {
            if let Ok(canonical_path) = canonicalize(cwd_path.join(&executable_path)) {
                return Ok(canonical_path.to_string_lossy().to_string());
            }
            return Err(DscError::CommandOperation(t!("util.executableNotFoundInWorkingDirectory", executable = &executable, cwd = cwd_path.display()).to_string(), executable_path.to_string_lossy().to_string()));
        }
        return Err(DscError::CommandOperation(t!("util.executableNotFound", executable = &executable).to_string(), executable.to_string()));
    }
    Ok(executable.to_string())
}

#[macro_export]
macro_rules! locked_clear {
    ($lockable:expr) => {{
        $lockable.write().unwrap().clear();
    }};
}

#[macro_export]
macro_rules! locked_is_empty {
    ($lockable:expr) => {{
        $lockable.read().unwrap().is_empty()
    }};
}

#[macro_export]
macro_rules! locked_extend {
    ($lockable:expr, $items:expr) => {{
        $lockable.write().unwrap().extend($items);
    }};
}

#[macro_export]
macro_rules! locked_clone {
    ($lockable:expr) => {{
        $lockable.read().unwrap().clone()
    }};
}

#[macro_export]
macro_rules! locked_get {
    ($lockable:expr, $key:expr) => {{
        if let Some(v) = $lockable.read().unwrap().get($key) {
            Some(v.clone())
        } else {
            None
        }
    }};
}
