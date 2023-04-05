// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::discovery::discovery_trait::{ResourceDiscovery};
use crate::dscresources::dscresource::{DscResource};

pub struct PowerShellDiscovery {
    pub resources: Vec<DscResource>,
}

impl PowerShellDiscovery {
    pub fn new() -> PowerShellDiscovery {
        PowerShellDiscovery {
            resources: Vec::new(),
        }
    }
}

impl Default for PowerShellDiscovery {
    fn default() -> Self {
        Self::new()
    }
}

impl ResourceDiscovery for PowerShellDiscovery {
    fn discover(&self) -> Box<dyn Iterator<Item = DscResource>> {
        // use `Get-DscResource` to convert to config resources
        // these are just test resources
        let resources = vec![];

        Box::new(resources.clone().into_iter())
    }

    fn initialize(&mut self) -> Result<(), crate::dscerror::DscError> {
        Ok(())
    }
}
