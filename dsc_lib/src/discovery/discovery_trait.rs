// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::{dscresources::dscresource::DscResource, dscerror::DscError};

pub trait ResourceDiscovery {
    fn discover(&self) -> Box<dyn Iterator<Item = DscResource>>;
    fn initialize(&mut self) -> Result<(), DscError>;
}
