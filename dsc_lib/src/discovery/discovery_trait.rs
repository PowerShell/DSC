// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::{dscresources::dscresource::DscResource, dscerror::DscError, dscerror::StreamMessageType};

pub trait ResourceDiscovery {
    fn discover(&self) -> Box<dyn Iterator<Item = DscResource>>;
    fn initialize(&mut self) -> Result<(), DscError>;
    fn print_initialization_messages(&mut self, error_format:StreamMessageType, warning_format:StreamMessageType) -> Result<(), DscError>;
}
