// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::dscresources::{
    resource_manifest::{Kind, ResourceManifest},
    dscresource::Capability,
};
use crate::{
    schemas::dsc_repo::DscRepoSchema,
    types::FullyQualifiedTypeName,
};
use rust_i18n::t;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::path::PathBuf;

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema, DscRepoSchema)]
#[serde(deny_unknown_fields)]
#[dsc_repo_schema(
    base_name = "manifest",
    folder_path = "resource",
    should_bundle = true,
    schema_field(
        name = schema_version,
        title = t!("dscresources.resource_manifest.adaptedResourceManifestSchemaTitle"),
        description = t!("dscresources.resource_manifest.adaptedResourceManifestSchemaDescription"),
    )
)]
pub struct AdaptedDscResourceManifest {
    /// The version of the resource manifest schema.
    #[serde(rename = "$schema")]
    #[schemars(schema_with = "ResourceManifest::recognized_schema_uris_subschema")]
    pub schema_version: String,
    /// The namespaced name of the resource.
    #[serde(rename="type")]
    pub type_name: FullyQualifiedTypeName,
    /// The kind of resource.
    pub kind: Kind,
    /// The version of the resource.
    pub version: String,
    /// The capabilities of the resource.
    pub capabilities: Vec<Capability>,
    /// An optional condition for the resource to be active.
    pub condition: Option<String>,
    /// The file path to the resource.
    pub path: PathBuf,
    /// The description of the resource.
    pub description: Option<String>,
    /// The author of the resource.
    pub author: Option<String>,
    /// The required resource adapter for the resource.
    #[serde(rename="requireAdapter")]
    pub require_adapter: FullyQualifiedTypeName,
    /// The JSON Schema of the resource.
    pub schema: Map<String, Value>,
}
