// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use registry::{Hive, Security, value::Data};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use utfx::UCString;

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[allow(clippy::struct_field_names)]
pub struct RefreshEnv {
    #[serde(rename="_metadata", skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Map<String, Value>>,
    pub name: String,
    pub value: String,
}

impl RefreshEnv {
    pub fn set(&self) {
        #[cfg(windows)]
        {
            // Set the environment variable in the registry for current user
            let hkcu = Hive::CurrentUser.open("Environment", Security::Write).unwrap();
            let ucstring = UCString::<u16>::from_str(&self.value).unwrap();
            let data = Data::String(ucstring);
            hkcu.set_value(&self.name, &data).unwrap();
        }
        #[cfg(not(windows))]
        {
            // Do nothing on non-Windows
        }
    }

    pub fn get(&self) -> RefreshEnv {
        #[cfg(windows)]
        {
            // Get the environment variable from the registry for current user
            let hkcu = Hive::CurrentUser.open("Environment", Security::Read).unwrap();
            if let Ok(data) = hkcu.value(&self.name) {
                if let Data::String(value) = data {
                    return RefreshEnv {
                        metadata: None,
                        name: self.name.clone(),
                        value: value.to_string_lossy(),
                    };
                }
            }
            
            RefreshEnv {
                metadata: None,
                name: self.name.clone(),
                value: String::new(),
            }
        }
        #[cfg(not(windows))]
        {
            // Return the input value on non-Windows since we can't read from the registry
            RefreshEnv {
                metadata: None,
                name: self.name.clone(),
                value: self.value.clone(),
            }
        }
    }
}
