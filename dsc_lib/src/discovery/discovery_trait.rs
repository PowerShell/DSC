// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::{dscresources::dscresource::DscResource, dscerror::DscError};
use std::collections::BTreeMap;
use crate::util::OutputFormat;

pub trait ResourceDiscovery {
    fn discover_resources(&mut self, filter: &str) -> Result<(), DscError>;
    fn discover_adapted_resources(&mut self, name_filter: &str, adapter_filter: &str, progress_format: &Option<OutputFormat>) -> Result<(), DscError>;
    fn list_available_resources(&mut self, type_name_filter: &str, adapter_name_filter: &str, progress_format: &Option<OutputFormat>) -> Result<BTreeMap<String, Vec<DscResource>>, DscError>;
    fn find_resources(&mut self, required_resource_types: &[String], progress_format: &Option<OutputFormat>) -> Result<BTreeMap<String, DscResource>, DscError>;
}
