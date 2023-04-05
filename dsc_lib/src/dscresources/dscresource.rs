// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use dscerror::DscError;
use resource_manifest::ResourceManifest;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use super::{*, invoke_result::{GetResult, SetResult, TestResult}};

// TODO: this should be redesigned to match our new ARM based syntax
// example is `name` should now be `type`
#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DscResource {
    #[serde(rename="Type")]
    pub type_name: String,
    #[serde(rename="FriendlyName")]
    pub friendly_name: Option<String>,
    #[serde(rename="Module")]
    pub module: Option<String>,
    #[serde(rename="ModuleName")]
    pub module_name: Option<String>,
    #[serde(rename="Version")]
    pub version: String,
    #[serde(rename="Path")]
    pub path: String,
    #[serde(rename="ParentPath")]
    pub parent_path: String,
    #[serde(rename="ImplementedAs")]
    pub implemented_as: ImplementedAs,
    #[serde(rename="CompanyName")]
    pub company_name: Option<String>,
    #[serde(rename="Properties")]
    pub properties: Vec<String>,
    #[serde(rename="Manifest")]
    pub manifest: Option<ResourceManifest>,
}

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
pub enum ImplementedAs {
    /// A PowerShell script function or class
    PowerShell,
    /// A PowerShell .ps1 script file
    PowerShellScript,
    /// A command line executable
    Command,
}

impl DscResource {
    pub fn new() -> Self {
        Self {
            type_name: String::new(),
            friendly_name: None,
            module: None,
            module_name: None,
            version: String::new(),
            path: String::new(),
            parent_path: String::new(),
            implemented_as: ImplementedAs::PowerShell,
            company_name: None,
            properties: Vec::new(),
            manifest: None,
        }
    }
}

impl Default for DscResource {
    fn default() -> Self {
        DscResource::new()
    }
}

pub trait Invoke {
    // the strings are expected to be json
    fn get(&self, filter: &str) -> Result<GetResult, DscError>;
    fn set(&self, desired: &str) -> Result<SetResult, DscError>;
    fn test(&self, expected: &str) -> Result<TestResult, DscError>;
    fn schema(&self) -> Result<String, DscError>;
}

impl Invoke for DscResource {
    fn get(&self, filter: &str) -> Result<GetResult, DscError> {
        match self.implemented_as {
            ImplementedAs::PowerShell => {
                let resource_name = &self.name;
                powershell_resource::invoke_get(resource_name, filter)
            },
            ImplementedAs::PowerShellScript => {
                Err(DscError::NotImplemented("get PowerShellScript resources".to_string()))
            },
            ImplementedAs::Command => {
                let manifest = match &self.manifest {
                    None => {
                        return Err(DscError::MissingManifest(self.type_name.clone()));
                    },
                    Some(manifest) => manifest,
                };
                command_resource::invoke_get(manifest, filter)
            },
        }
    }

    fn set(&self, desired: &str) -> Result<SetResult, DscError> {
        match self.implemented_as {
            ImplementedAs::PowerShell => {
                let resource_name = &self.name;
                powershell_resource::invoke_set(resource_name, desired)
            },
            ImplementedAs::PowerShellScript => {
                Err(DscError::NotImplemented("set PowerShellScript resources".to_string()))
            },
            ImplementedAs::Command => {
                let manifest = match &self.manifest {
                    None => {
                        return Err(DscError::MissingManifest(self.type_name.clone()));
                    },
                    Some(manifest) => manifest,
                };
                command_resource::invoke_set(manifest, desired)
            },
        }
    }

    fn test(&self, expected: &str) -> Result<TestResult, DscError> {
        match self.implemented_as {
            ImplementedAs::PowerShell => {
                let resource_name = &self.name;
                powershell_resource::invoke_test(resource_name, expected)
            },
            ImplementedAs::PowerShellScript => {
                Err(DscError::NotImplemented("test PowerShellScript resources".to_string()))
            },
            ImplementedAs::Command => {
                let manifest = match &self.manifest {
                    None => {
                        return Err(DscError::MissingManifest(self.type_name.clone()));
                    },
                    Some(manifest) => manifest,
                };

                // if test is not directly implemented, then we need to handle it here
                if manifest.test.is_none() {
                    let get_result = self.get(expected)?;
                    let expected_state = serde_json::from_str(expected)?;
                    let diff_properties = get_diff(&expected_state, &get_result.actual_state);
                    let test_result = TestResult {
                        expected_state: serde_json::from_str(expected)?,
                        actual_state: get_result.actual_state,
                        diff_properties: Some(diff_properties),
                    };
                    return Ok(test_result);
                }
                else {
                    command_resource::invoke_test(manifest, expected)
                }
            },
        }
    }

    fn schema(&self) -> Result<String, DscError> {
        match self.implemented_as {
            ImplementedAs::PowerShell => {
                Err(DscError::NotImplemented("schema PowerShell resources".to_string()))
            },
            ImplementedAs::PowerShellScript => {
                Err(DscError::NotImplemented("schema PowerShellScript resources".to_string()))
            },
            ImplementedAs::Command => {
                let manifest = match &self.manifest {
                    None => {
                        return Err(DscError::MissingManifest(self.type_name.clone()));
                    },
                    Some(manifest) => manifest,
                };
                command_resource::get_schema(manifest)
            },
        }
    }
}

pub fn get_diff(expected: &Value, actual: &Value) -> Vec<String> {
    let mut diff_properties: Vec<String> = Vec::new();
    if expected.is_null() {
        return diff_properties;
    }

    for (key, value) in expected.as_object().unwrap() {
        // skip meta properties
        if key.starts_with("_") || key.starts_with("$") {
            continue;
        }

        if value.is_object() {
            let sub_diff = get_diff(value, &actual[key]);
            if sub_diff.len() > 0 {
                diff_properties.push(key.to_string());
            }
        }
        else {
            match actual.as_object() {
                Some(actual_object) => {
                    if !actual_object.contains_key(key) {
                        diff_properties.push(key.to_string());
                    }
                    else {
                        if value != &actual[key] {
                            diff_properties.push(key.to_string());
                        }
                    }
                },
                None => {
                    diff_properties.push(key.to_string());
                },
            }
        }
    }
    diff_properties
}
