// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use rust_i18n::t;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::collections::HashMap;

use crate::{
    schemas::{dsc_repo::DscRepoSchema, transforms::idiomaticize_string_enum},
    types::{FullyQualifiedTypeName, ResourceVersion},
};

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema, DscRepoSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(transform = idiomaticize_string_enum)]
#[dsc_repo_schema(base_name = "resourceKind", folder_path = "definitions")]
pub enum Kind {
    Adapter,
    Exporter,
    Group,
    Importer,
    Resource,
}

#[derive(Debug, Default, Clone, PartialEq, Deserialize, Serialize, JsonSchema, DscRepoSchema)]
#[serde(deny_unknown_fields)]
#[dsc_repo_schema(
    base_name = "manifest",
    folder_path = "resource",
    should_bundle = true,
    schema_field(
        name = schema_version,
        title = t!("dscresources.resource_manifest.resourceManifestSchemaTitle"),
        description = t!("dscresources.resource_manifest.resourceManifestSchemaDescription"),
    )
)]
pub struct ResourceManifest {
    /// The version of the resource manifest schema.
    #[serde(rename = "$schema")]
    #[schemars(schema_with = "ResourceManifest::recognized_schema_uris_subschema")]
    pub schema_version: String,
    /// The namespaced name of the resource.
    #[serde(rename = "type")]
    pub resource_type: FullyQualifiedTypeName,
    /// An optional condition for the resource to be active.  If the condition evaluates to false, the resource is skipped.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub condition: Option<String>,
    /// The kind of resource.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kind: Option<Kind>,
    /// The version of the resource using semantic versioning.
    pub version: ResourceVersion,
    /// The description of the resource.
    pub description: Option<String>,
    /// Tags for the resource.
    pub tags: Option<Vec<String>>,
    /// Details how to call the Get method of the resource.
    pub get: Option<GetMethod>,
    /// Details how to call the Set method of the resource.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub set: Option<SetMethod>,
    /// Details how to call the `WhatIf` method of the resource.
    #[serde(rename = "whatIf", skip_serializing_if = "Option::is_none")]
    pub what_if: Option<SetMethod>,
    /// Details how to call the Test method of the resource.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub test: Option<TestMethod>,
    /// Details how to call the Delete method of the resource.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delete: Option<DeleteMethod>,
    /// Details how to call the Export method of the resource.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub export: Option<ExportMethod>,
    /// Details how to call the Resolve method of the resource.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolve: Option<ResolveMethod>,
    /// Details how to call the Validate method of the resource.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub validate: Option<ValidateMethod>,
    /// Indicates the resource is a adapter of other resources.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub adapter: Option<Adapter>,
    /// Mapping of exit codes to descriptions.  Zero is always success and non-zero is always failure.
    #[serde(rename = "exitCodes", skip_serializing_if = "Option::is_none")]
    pub exit_codes: Option<HashMap<String, String>>, // we have to make this a string key instead of i32 due to https://github.com/serde-rs/json/issues/560
    /// Details how to get the schema of the resource.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema: Option<SchemaKind>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Map<String, Value>>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema, DscRepoSchema)]
#[serde(untagged)]
#[dsc_repo_schema(base_name = "commandArgs.get", folder_path = "definitions")]
pub enum GetArgKind {
    /// The argument is a string.
    String(String),
    /// The argument accepts the JSON input object.
    Json {
        /// The argument that accepts the JSON input object.
        #[serde(rename = "jsonInputArg")]
        json_input_arg: String,
        /// Indicates if argument is mandatory which will pass an empty string if no JSON input is provided.  Default is false.
        mandatory: Option<bool>,
    },
    ResourcePath {
        /// The argument that accepts the resource path.
        #[serde(rename = "resourcePathArg")]
        resource_path_arg: String,
    },
    ResourceType {
        /// The argument that accepts the resource type name.
        #[serde(rename = "resourceTypeArg")]
        resource_type_arg: String,
    },
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema, DscRepoSchema)]
#[serde(untagged)]
#[dsc_repo_schema(base_name = "commandArgs.setDelete", folder_path = "definitions")]
pub enum SetDeleteArgKind {
    /// The argument is a string.
    String(String),
    /// The argument accepts the JSON input object.
    Json {
        /// The argument that accepts the JSON input object.
        #[serde(rename = "jsonInputArg")]
        json_input_arg: String,
        /// Indicates if argument is mandatory which will pass an empty string if no JSON input is provided.  Default is false.
        mandatory: Option<bool>,
    },
    ResourcePath {
        /// The argument that accepts the resource path.
        #[serde(rename = "resourcePathArg")]
        resource_path_arg: String,
    },
    ResourceType {
        /// The argument that accepts the resource type name.
        #[serde(rename = "resourceTypeArg")]
        resource_type_arg: String,
    },
    /// The argument is passed when the resource is invoked in what-if mode.
    WhatIf {
        /// The argument to pass when in what-if mode.
        #[serde(rename = "whatIfArg")]
        what_if_arg: String,
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema, DscRepoSchema)]
#[schemars(transform = idiomaticize_string_enum)]
#[dsc_repo_schema(base_name = "inputKind", folder_path = "definitions")]
pub enum InputKind {
    /// The input is accepted as environmental variables.
    #[serde(rename = "env")]
    Env,
    /// The input is accepted as a JSON object via STDIN.
    #[serde(rename = "stdin")]
    Stdin,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema, DscRepoSchema)]
#[dsc_repo_schema(base_name = "manifest.schema", folder_path = "resource")]
pub enum SchemaKind {
    /// The schema is returned by running a command.
    #[serde(rename = "command")]
    Command(SchemaCommand),
    /// The schema is embedded in the manifest.
    #[serde(rename = "embedded")]
    Embedded(Value),
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
pub struct SchemaCommand {
    /// The command to run to get the schema.
    pub executable: String,
    /// The arguments to pass to the command.
    pub args: Option<Vec<String>>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema, DscRepoSchema)]
#[schemars(transform = idiomaticize_string_enum)]
#[dsc_repo_schema(base_name = "returnKind", folder_path = "definitions")]
pub enum ReturnKind {
    /// The return JSON is the state of the resource.
    #[serde(rename = "state")]
    State,
    /// The return JSON is the state of the resource and the diff.
    #[serde(rename = "stateAndDiff")]
    StateAndDiff,
}

#[derive(Debug, Default, Clone, PartialEq, Deserialize, Serialize, JsonSchema, DscRepoSchema)]
#[dsc_repo_schema(base_name = "manifest.get", folder_path = "resource")]
pub struct GetMethod {
    /// The command to run to get the state of the resource.
    pub executable: String,
    /// The arguments to pass to the command to perform a Get.
    pub args: Option<Vec<GetArgKind>>,
    /// How to pass optional input for a Get.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input: Option<InputKind>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema, DscRepoSchema)]
#[dsc_repo_schema(base_name = "manifest.set", folder_path = "resource")]
pub struct SetMethod {
    /// The command to run to set the state of the resource.
    pub executable: String,
    /// The arguments to pass to the command to perform a Set.
    pub args: Option<Vec<SetDeleteArgKind>>,
    /// How to pass required input for a Set.
    pub input: Option<InputKind>,
    /// Whether to run the Test method before the Set method.  True means the resource will perform its own test before running the Set method.
    #[serde(rename = "implementsPretest", skip_serializing_if = "Option::is_none")]
    pub pre_test: Option<bool>,
    /// Indicates that the resource directly handles `_exist` as a property.
    #[serde(rename = "handlesExist", skip_serializing_if = "Option::is_none")]
    pub handles_exist: Option<bool>,
    /// The type of return value expected from the Set method.
    #[serde(rename = "return", skip_serializing_if = "Option::is_none")]
    pub returns: Option<ReturnKind>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema, DscRepoSchema)]
#[dsc_repo_schema(base_name = "manifest.test", folder_path = "resource")]
pub struct TestMethod {
    /// The command to run to test the state of the resource.
    pub executable: String,
    /// The arguments to pass to the command to perform a Test.
    pub args: Option<Vec<GetArgKind>>,
    /// How to pass required input for a Test.
    pub input: Option<InputKind>,
    /// The type of return value expected from the Test method.
    #[serde(rename = "return", skip_serializing_if = "Option::is_none")]
    pub returns: Option<ReturnKind>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema, DscRepoSchema)]
#[dsc_repo_schema(base_name = "manifest.delete", folder_path = "resource")]
pub struct DeleteMethod {
    /// The command to run to delete the state of the resource.
    pub executable: String,
    /// The arguments to pass to the command to perform a Delete.
    pub args: Option<Vec<SetDeleteArgKind>>,
    /// How to pass required input for a Delete.
    pub input: Option<InputKind>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema, DscRepoSchema)]
#[dsc_repo_schema(base_name = "manifest.validate", folder_path = "resource")]
pub struct ValidateMethod { // TODO: enable validation via schema or command
    /// The command to run to validate the state of the resource.
    pub executable: String,
    /// The arguments to pass to the command to perform a Validate.
    pub args: Option<Vec<GetArgKind>>,
    /// How to pass required input for a Validate.
    pub input: Option<InputKind>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema, DscRepoSchema)]
#[dsc_repo_schema(base_name = "manifest.export", folder_path = "resource")]
pub struct ExportMethod {
    /// The command to run to enumerate instances of the resource.
    pub executable: String,
    /// The arguments to pass to the command to perform a Export.
    pub args: Option<Vec<GetArgKind>>,
    /// How to pass input for a Export.
    pub input: Option<InputKind>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema, DscRepoSchema)]
#[dsc_repo_schema(base_name = "manifest.resolve", folder_path = "resource")]
pub struct ResolveMethod {
    /// The command to run to enumerate instances of the resource.
    pub executable: String,
    /// The arguments to pass to the command to perform a Export.
    pub args: Option<Vec<GetArgKind>>,
    /// How to pass input for a Export.
    pub input: Option<InputKind>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema, DscRepoSchema)]
#[dsc_repo_schema(base_name = "manifest.adapter", folder_path = "resource")]
pub struct Adapter {
    /// The way to list adapter supported resources.
    pub list: ListMethod,
    /// Defines how the adapter supports accepting configuration.
    #[serde(alias = "config", rename = "inputKind")]
    pub input_kind: AdapterInputKind,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
#[schemars(transform = idiomaticize_string_enum)]
pub enum AdapterInputKind {
    /// The adapter accepts full unprocessed configuration.
    #[serde(rename = "full")]
    Full,
    /// The adapter accepts configuration as a sequence.
    #[serde(rename = "sequence")]
    Sequence,
    /// The adapter accepts a single resource input.
    #[serde(rename = "single")]
    Single,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
pub struct ListMethod {
    /// The command to run to list resources supported by a group resource.
    pub executable: String,
    /// The arguments to pass to the command to perform a List.
    pub args: Option<Vec<String>>,
}

#[cfg(test)]
mod test {
    use crate::schemas::dsc_repo::{DscRepoSchema, UnrecognizedSchemaUri};

    use crate::dscresources::resource_manifest::ResourceManifest;

    #[test]
    fn test_validate_schema_uri_with_invalid_uri() {
        let invalid_uri = "https://invalid.schema.uri".to_string();

        let manifest = ResourceManifest{
            schema_version: invalid_uri.clone(),
            resource_type: "Microsoft.Dsc.Test/InvalidSchemaUri".parse().unwrap(),
            version: "0.1.0".parse().unwrap(),
            ..Default::default()
        };

        let ref result = manifest.validate_schema_uri();

        assert!(result.as_ref().is_err());

        match result.as_ref().unwrap_err() {
            UnrecognizedSchemaUri(actual, recognized) => {
                assert_eq!(actual, &invalid_uri);
                assert_eq!(recognized, &ResourceManifest::recognized_schema_uris())
            },
        }
    }

    #[test]
    fn test_validate_schema_uri_with_valid_uri() {
        let manifest = ResourceManifest{
            schema_version: ResourceManifest::default_schema_id_uri(),
            resource_type: "Microsoft.Dsc.Test/ValidSchemaUri".parse().unwrap(),
            version: "0.1.0".parse().unwrap(),
            ..Default::default()
        };

        let result = manifest.validate_schema_uri();

        assert!(result.is_ok());
    }
}
