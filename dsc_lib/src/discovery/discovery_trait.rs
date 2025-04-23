// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::{dscerror::DscError, dscresources::dscresource::DscResource};
use std::collections::BTreeMap;

use super::command_discovery::ManifestResource;

#[derive(PartialEq)]
pub enum DiscoveryKind {
    Resource,
    Extension,
}

pub trait ResourceDiscovery {
    fn discover(&mut self, kind: &DiscoveryKind, filter: &str) -> Result<(), DscError>;
    fn discover_adapted_resources(&mut self, name_filter: &str, adapter_filter: &str) -> Result<(), DscError>;
    fn list_available(&mut self, kind: &DiscoveryKind, type_name_filter: &str, adapter_name_filter: &str) -> Result<BTreeMap<String, Vec<ManifestResource>>, DscError>;
    fn find_resources(&mut self, required_resource_types: &[String]) -> Result<BTreeMap<String, DscResource>, DscError>;
}
