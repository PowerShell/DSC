// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::{dscresources::{
    dscresource::Capability, resource_manifest::{Kind, ResourceManifest}
}, types::ResourceVersion};
use crate::{
    schemas::dsc_repo::DscRepoSchema,
    types::FullyQualifiedTypeName,
};
use rust_i18n::t;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::path::PathBuf;

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(inline)]
pub enum AdaptedPathOrContent {
    Path(PathBuf),
    Content(Map<String, Value>),
}

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema, DscRepoSchema)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
#[dsc_repo_schema(
    base_name = "manifest",
    folder_path = "resource/adapted",
    should_bundle = true,
    schema_field(
        name = schema_version,
        title = t!("dscresources.resource_manifest.adaptedResourceManifestSchemaTitle"),
        description = t!("dscresources.resource_manifest.adaptedResourceManifestSchemaDescription"),
    )
)]
#[schemars(
    transform = AdaptedDscResourceManifest::transform_export_schema_uris,
    transform = AdaptedDscResourceManifest::transform_schema_docs
)]
pub struct AdaptedDscResourceManifest {
    /// The version of the resource manifest schema.
    #[serde(rename = "$schema")]
    #[schemars(schema_with = "AdaptedDscResourceManifest::recognized_schema_uris_with_deprecated_subschema")]
    pub schema_version: String,
    /// The namespaced name of the resource.
    #[serde(rename="type")]
    pub type_name: FullyQualifiedTypeName,
    /// The kind of resource.
    pub kind: Kind,
    /// The version of the resource.
    pub version: ResourceVersion,
    /// The capabilities of the resource.
    pub capabilities: Vec<Capability>,
    /// An optional condition for the resource to be active.
    pub condition: Option<String>,
    /// An optional message indicating the resource is deprecated.  If provided, the message will be shown when the resource is used.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deprecation_message: Option<String>,
    /// The file path to the resource or the content of the resource itself.
    #[serde(flatten)]
    pub path_or_content: AdaptedPathOrContent,
    /// The description of the resource.
    pub description: Option<String>,
    /// The author of the resource.
    pub author: Option<String>,
    /// The required resource adapter for the resource.
    pub require_adapter: FullyQualifiedTypeName,
    /// The JSON Schema of the resource.
    pub schema: Map<String, Value>,
}

impl AdaptedDscResourceManifest {
    pub const LEGACY_SHIPPED_SCHEMA_URI: &'static str = "https://aka.ms/dsc/schemas/v3/bundled/adaptedresource/manifest.json";

    #[must_use]
    pub fn is_deprecated_schema_uri(uri: &String) -> bool {
        uri.as_str() == Self::LEGACY_SHIPPED_SCHEMA_URI || ResourceManifest::is_recognized_schema_uri(uri)
    }

    fn recognized_schema_uris_with_deprecated_subschema(generator: &mut schemars::SchemaGenerator) -> schemars::Schema {
        let mut subschema = <Self as DscRepoSchema>::recognized_schema_uris_subschema(generator);
        subschema.remove("enum");
        let recognized_uris: Vec<Value> = Self::recognized_schema_uris()
            .into_iter()
            .map(Value::String)
            .collect();
        let deprecated_uris: Vec<Value> = ResourceManifest::recognized_schema_uris()
            .into_iter()
            .chain([Self::LEGACY_SHIPPED_SCHEMA_URI.to_string()])
            .map(Value::String)
            .collect();
        subschema.insert("oneOf".to_string(), serde_json::json!([
            {
                "enum": recognized_uris,
            },
            {
                "enum": deprecated_uris,
                "deprecated": true,
                "deprecationMessage": t!(
                    "dscresources.resource_manifest.adaptedResourceManifestDeprecatedSchemaUri",
                    uri = Self::default_schema_id_uri()
                ),
            },
        ]));
        subschema
    }
}
