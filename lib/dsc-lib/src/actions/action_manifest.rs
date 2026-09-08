// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::{
    configure::config_doc::SecurityContextKind, dscresources::resource_manifest::SchemaCommand, schemas::dsc_repo::DscRepoSchema, types::{ExitCodesMap, FullyQualifiedTypeName, SemanticVersion, TagList},
};
use rust_i18n::t;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use schemars::JsonSchema;

#[derive(Debug, Default, Clone, PartialEq, Deserialize, JsonSchema, DscRepoSchema)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
#[dsc_repo_schema(
    base_name = "manifest",
    folder_path = "action",
    should_bundle = true,
    schema_field(
        name = schema_version,
        title = t!("actions.action_manifest.actionManifestSchemaTitle"),
        description = t!("actions.action_manifest.actionManifestSchemaDescription"),
    )
)]
pub(crate) struct ActionManifest {
    /// The version of the action manifest schema.
    #[serde(rename = "$schema")]
    #[schemars(schema_with = "ActionManifest::recognized_schema_uris_subschema")]
    pub schema_version: String,
    /// The namespaced name of the action.
    #[serde(rename = "type")]
    pub type_name: FullyQualifiedTypeName,
    /// The version of the action using semantic versioning.
    pub version: SemanticVersion,
    /// The author of the action.
    pub author: Option<String>,
    /// An optional condition for the action to be active. If the condition evaluates to false, the action is not available.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub condition: Option<String>,
    /// An optional message indicating the action is deprecated. If provided, the message will be shown when the action is used.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deprecation_message: Option<String>,
    /// The description of the action.
    pub description: Option<String>,
    /// Tags for the action.
    #[serde(skip_serializing_if = "TagList::is_empty")]
    pub tags: TagList,
    /// Details how to invoke this action.
    pub invoke: InvokeMethod,
    /// Mapping of exit codes to descriptions.  Zero is always success and non-zero is always failure.
    #[serde(skip_serializing_if = "ExitCodesMap::is_empty_or_default", default)]
    pub exit_codes: ExitCodesMap,
}

#[derive(Debug, Default, Clone, PartialEq, Deserialize, Serialize, JsonSchema, DscRepoSchema)]
#[dsc_repo_schema(base_name = "manifest.invoke", folder_path = "action")]
pub(crate) struct InvokeMethod {
    /// The executable to run on action invocation.
    pub executable: String,
    /// The arguments passed to the executable.
    pub args: Option<Vec<ArgKind>>,
    /// The input schema for the action.  Required if the action expects input.
    pub input_schema: Option<SchemaKind>,
    /// The output schema for the action.  If not defined, then the output is validated against the input schema.
    pub output_schema: Option<SchemaKind>,
    /// The security context required to invoke the action.  If not defined, then no special security context is required.
    pub require_security_context: Option<SecurityContextKind>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema, DscRepoSchema)]
#[serde(untagged)]
#[dsc_repo_schema(base_name = "commandArgs.action", folder_path = "definitions")]
pub enum ArgKind {
    String(String),
    Json {
        /// The argument that accepts the JSON input object.
        #[serde(rename = "jsonInputArg")]
        json_input_arg: String,
        /// Indicates if argument is mandatory which will pass an empty string if no JSON input is provided.  Default is false.
        mandatory: Option<bool>,
    },
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema, DscRepoSchema)]
#[dsc_repo_schema(base_name = "manifest.schema", folder_path = "action")]
pub enum SchemaKind {
    /// The schema is returned by running a command.
    #[serde(rename = "command")]
    Command(SchemaCommand),
    /// The schema is embedded in the manifest.
    #[serde(rename = "embedded")]
    Embedded(Value),
}
