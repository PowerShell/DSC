// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::{
    actions::action_manifest::ActionManifest, dscerror::DscError, dscresources::resource_manifest::ResourceManifest,
};
use rust_i18n::t;

pub mod invoke_result;
pub mod resource_invoke;

pub enum ManifestKind {
    Action(Box<ActionManifest>),
    Resource(Box<ResourceManifest>)
}

impl ManifestKind {
    pub fn as_action_manifest(&self) -> Result<&ActionManifest, DscError> {
        if let ManifestKind::Action(manifest) = self {
            Ok(manifest)
        } else {
            Err(DscError::ResourceManifestNotFound(t!("resources.notAnAction").to_string()))
        }
    }

    pub fn as_resource_manifest(&self) -> Result<&ResourceManifest, DscError> {
        if let ManifestKind::Resource(manifest) = self {
            Ok(manifest)
        } else {
            Err(DscError::ResourceManifestNotFound(t!("resources.notAResource").to_string()))
        }
    }
}
