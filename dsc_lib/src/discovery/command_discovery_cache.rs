// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::dscresources::dscresource::DscResource;
use crate::extensions::dscextension::DscExtension;
use std::collections::BTreeMap;
use std::sync::{LazyLock, Mutex};

// use BTreeMap so that the results are sorted by the typename, the Vec is sorted by version
static ADAPTERS: LazyLock<Mutex<BTreeMap<String, Vec<DscResource>>>> = LazyLock::new(|| Mutex::new(BTreeMap::new()));
static RESOURCES: LazyLock<Mutex<BTreeMap<String, Vec<DscResource>>>> = LazyLock::new(|| Mutex::new(BTreeMap::new()));
static EXTENSIONS: LazyLock<Mutex<BTreeMap<String, DscExtension>>> = LazyLock::new(|| Mutex::new(BTreeMap::new()));
static ADAPTED_RESOURCES: LazyLock<Mutex<BTreeMap<String, Vec<DscResource>>>> = LazyLock::new(|| Mutex::new(BTreeMap::new()));

// Adapter functions

pub fn adapters_is_empty() -> bool {
    ADAPTERS.lock().unwrap().is_empty()
}

pub fn extend_adapters(new_adapters: BTreeMap<String, Vec<DscResource>>) {
    ADAPTERS.lock().unwrap().extend(new_adapters);
}

pub fn get_adapters() -> BTreeMap<String, Vec<DscResource>> {
    ADAPTERS.lock().unwrap().clone()
}

// Adapted Resource functions

pub fn extend_adapted_resources(new_adapted_resources: BTreeMap<String, Vec<DscResource>>) {
    ADAPTED_RESOURCES.lock().unwrap().extend(new_adapted_resources);
}

pub fn get_adapted_resource(type_name: &str) -> Option<Vec<DscResource>> {
    if let Some(resources) = ADAPTED_RESOURCES.lock().unwrap().get(type_name) {
        return Some(resources.clone());
    }
    None
}

pub fn get_adapted_resources() -> BTreeMap<String, Vec<DscResource>> {
    ADAPTED_RESOURCES.lock().unwrap().clone()
}

// Extension functions

pub fn extend_extensions(new_extensions: BTreeMap<String, DscExtension>) {
    EXTENSIONS.lock().unwrap().extend(new_extensions);
}

pub fn extensions_is_empty() -> bool {
    EXTENSIONS.lock().unwrap().is_empty()
}

pub fn get_extensions() -> BTreeMap<String, DscExtension> {
    EXTENSIONS.lock().unwrap().clone()
}

// Resource functions

pub fn extend_resources(new_resources: BTreeMap<String, Vec<DscResource>>) {
    RESOURCES.lock().unwrap().extend(new_resources);
}

pub fn get_resource(type_name: &str) -> Option<Vec<DscResource>> {
    if let Some(resources) = RESOURCES.lock().unwrap().get(type_name) {
        return Some(resources.clone());
    }
    None
}

pub fn get_resources() -> BTreeMap<String, Vec<DscResource>> {
    RESOURCES.lock().unwrap().clone()
}

pub fn resources_is_empty() -> bool {
    RESOURCES.lock().unwrap().is_empty()
}
