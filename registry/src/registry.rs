// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use registry::Registry;
use crate::config::RegistryConfig;

pub struct RegistryHelper {
    config: RegistryConfig,
}

impl RegistryHelper {
    pub fn new(config: &str) -> Result<Self, RegistryError> {
        Self {
            config: serde_json::from_str(config)?,
        }
    }

    pub fn get(&self) -> Result<RegistryConfig, RegistryError> {
        
    }

    pub fn set(&self) -> Result<(), RegistryError> {
        unimplemented!()
    }

    pub fn remove(&self) -> Result<(), RegistryError> {
        unimplemented!()
    }
}
