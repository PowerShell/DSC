// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use chrono::{DateTime, Local};
use rust_i18n::t;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::{collections::HashMap, fmt::Display};

use crate::{schemas::{
    dsc_repo::DscRepoSchema,
    transforms::{idiomaticize_externally_tagged_enum, idiomaticize_string_enum}
}, types::FullyQualifiedTypeName};

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema, DscRepoSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(transform = idiomaticize_string_enum)]
#[dsc_repo_schema(base_name = "securityContext", folder_path = "metadata/Microsoft.DSC")]
pub enum SecurityContextKind {
    Current,
    Elevated,
    Restricted,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema, DscRepoSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(transform = idiomaticize_string_enum)]
#[dsc_repo_schema(base_name = "operation", folder_path = "metadata/Microsoft.DSC")]
pub enum Operation {
    Get,
    Set,
    Test,
    Export,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema, DscRepoSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(transform = idiomaticize_string_enum)]
#[dsc_repo_schema(base_name = "executionType", folder_path = "metadata/Microsoft.DSC")]
pub enum ExecutionKind {
    Actual,
    WhatIf,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Process {
    pub name: String,
    pub id: u32,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema, DscRepoSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(transform = idiomaticize_externally_tagged_enum)]
#[dsc_repo_schema(base_name = "restartRequired", folder_path = "metadata/Microsoft.DSC")]
pub enum RestartRequired {
    System(String),
    Service(String),
    Process(Process),
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema, DscRepoSchema)]
#[serde(rename_all = "camelCase")]
#[dsc_repo_schema(base_name = "resourceDiscovery", folder_path = "metadata/Microsoft.DSC")]
pub enum ResourceDiscoveryMode {
    PreDeployment,
    DuringDeployment,
}

#[derive(Debug, Default, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct MicrosoftDscMetadata {
    /// The duration of the configuration operation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration: Option<String>,
    /// The end time of the configuration operation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_datetime: Option<String>,
    /// The type of execution
    #[serde(skip_serializing_if = "Option::is_none")]
    pub execution_type: Option<ExecutionKind>,
    /// The operation being performed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operation: Option<Operation>,
    /// Specify specific adapter type used for implicit operations
    #[serde(skip_serializing_if = "Option::is_none")]
    pub require_adapter: Option<String>,
    /// Indicates if resources are discovered pre-deployment or during deployment
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource_discovery: Option<ResourceDiscoveryMode>,
    /// Indicates what needs to be restarted after the configuration operation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub restart_required: Option<Vec<RestartRequired>>,
    /// Copy loop context for resources expanded from copy loops
    #[serde(rename = "copyLoops", skip_serializing_if = "Option::is_none")]
    pub copy_loops: Option<Map<String, Value>>,
    /// The security context of the configuration operation, can be specified to be required
    #[serde(skip_serializing_if = "Option::is_none")]
    pub security_context: Option<SecurityContextKind>,
    /// The start time of the configuration operation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_datetime: Option<String>,
    /// Version of DSC
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
}

impl MicrosoftDscMetadata {
    /// Creates a new instance of `MicrosoftDscMetadata` with the duration
    ///
    /// # Arguments
    ///
    /// * `start` - The start time of the configuration operation
    /// * `end` - The end time of the configuration operation
    ///
    /// # Returns
    ///
    /// A new instance of `MicrosoftDscMetadata` with the duration calculated from the start and end times.
    #[must_use]
    pub fn new_with_duration(start: &DateTime<Local>, end: &DateTime<Local>) -> Self {
        Self {
            duration: Some(end.signed_duration_since(*start).to_string()),
            ..Default::default()
        }
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema, DscRepoSchema)]
#[dsc_repo_schema(base_name = "document.metadata", folder_path = "config")]
pub struct Metadata {
    #[serde(rename = "Microsoft.DSC", skip_serializing_if = "Option::is_none")]
    pub microsoft: Option<MicrosoftDscMetadata>,
    #[serde(flatten)]
    pub other: Map<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema, DscRepoSchema)]
#[dsc_repo_schema(base_name = "document.function", folder_path = "config")]
pub struct UserFunction {
    pub namespace: String,
    pub members: HashMap<String, UserFunctionDefinition>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema, DscRepoSchema)]
#[dsc_repo_schema(base_name = "definition", folder_path = "definitions/functions/user")]
pub struct UserFunctionDefinition {
    pub parameters: Option<Vec<UserFunctionParameter>>,
    pub output: UserFunctionOutput,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema, DscRepoSchema)]
#[dsc_repo_schema(base_name = "parameter", folder_path = "definitions/functions/user")]
pub struct UserFunctionParameter {
    pub name: String,
    pub r#type: DataType,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema, DscRepoSchema)]
#[dsc_repo_schema(base_name = "output", folder_path = "definitions/functions/user")]
pub struct UserFunctionOutput {
    pub r#type: DataType,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum ValueOrCopy {
    Value(String),
    Copy(Copy),
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema, DscRepoSchema)]
#[serde(deny_unknown_fields)]
#[dsc_repo_schema(base_name = "document.output", folder_path = "config")]
pub struct Output {
    pub condition: Option<String>,
    pub r#type: DataType,
    #[serde(flatten)]
    pub value_or_copy: ValueOrCopy,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema, DscRepoSchema)]
#[serde(deny_unknown_fields)]
#[dsc_repo_schema(
    base_name = "document",
    folder_path = "config",
    should_bundle = true,
    schema_field(
        name = schema,
        title = t!("configure.config_doc.configurationDocumentSchemaTitle"),
        description = t!("configure.config_doc.configurationDocumentSchemaDescription"),
    )
)]
pub struct Configuration {
    #[serde(rename = "$schema")]
    #[schemars(schema_with = "Configuration::recognized_schema_uris_subschema")]
    pub schema: String,
    #[serde(rename = "contentVersion")]
    pub content_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub functions: Option<Vec<UserFunction>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Metadata>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub outputs: Option<HashMap<String, Output>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters: Option<HashMap<String, Parameter>>,
    pub resources: Vec<Resource>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variables: Option<Map<String, Value>>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema, DscRepoSchema)]
#[serde(deny_unknown_fields)]
#[dsc_repo_schema(base_name = "document.parameter", folder_path = "config")]
pub struct Parameter {
    #[serde(rename = "type")]
    pub parameter_type: DataType,
    #[serde(rename = "defaultValue", skip_serializing_if = "Option::is_none")]
    pub default_value: Option<Value>,
    #[serde(rename = "allowedValues", skip_serializing_if = "Option::is_none")]
    pub allowed_values: Option<Vec<Value>>,
    #[serde(rename = "minValue", skip_serializing_if = "Option::is_none")]
    pub min_value: Option<i64>,
    #[serde(rename = "maxValue", skip_serializing_if = "Option::is_none")]
    pub max_value: Option<i64>,
    #[serde(rename = "minLength", skip_serializing_if = "Option::is_none")]
    pub min_length: Option<i64>,
    #[serde(rename = "maxLength", skip_serializing_if = "Option::is_none")]
    pub max_length: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Map<String, Value>>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema, DscRepoSchema)]
#[schemars(transform = idiomaticize_string_enum)]
#[dsc_repo_schema(base_name = "dataTypes", folder_path = "definitions/parameters")]
pub enum DataType {
    #[serde(rename = "string")]
    String,
    #[serde(rename = "secureString")]
    SecureString,
    #[serde(rename = "int")]
    Int,
    #[serde(rename = "bool")]
    Bool,
    #[serde(rename = "object")]
    Object,
    #[serde(rename = "secureObject")]
    SecureObject,
    #[serde(rename = "array")]
    Array,
}

impl Display for DataType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let type_str = match self {
            DataType::String => "string",
            DataType::SecureString => "secureString",
            DataType::Int => "int",
            DataType::Bool => "bool",
            DataType::Object => "object",
            DataType::SecureObject => "secureObject",
            DataType::Array => "array",
        };
        write!(f, "{type_str}")
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
#[schemars(transform = idiomaticize_string_enum)]
pub enum CopyMode {
    #[serde(rename = "serial")]
    Serial,
    #[serde(rename = "parallel")]
    Parallel,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(untagged)]
pub enum IntOrExpression {
    Int(i64),
    Expression(String),
}

impl Display for IntOrExpression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IntOrExpression::Int(i) => write!(f, "{i}"),
            IntOrExpression::Expression(s) => write!(f, "{s}"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Copy {
    pub name: String,
    pub count: IntOrExpression,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mode: Option<CopyMode>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "batchSize")]
    pub batch_size: Option<IntOrExpression>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Plan {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none", rename = "promotionCode")]
    pub promotion_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub publisher: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub product: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Identity {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "userAssignedIdentities")]
    pub user_assigned_identities: Option<Map<String, Value>>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Sku {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tier: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub family: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub capacity: Option<i32>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema, DscRepoSchema)]
#[serde(deny_unknown_fields)]
#[dsc_repo_schema(base_name = "document.resource", folder_path = "config")]
pub struct Resource {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub condition: Option<String>,
    /// The fully qualified name of the resource type
    #[serde(rename = "type")]
    pub resource_type: FullyQualifiedTypeName,
    #[serde(skip_serializing_if = "Option::is_none", rename = "requireVersion", alias = "apiVersion")]
    pub require_version: Option<String>,
    /// A friendly name for the resource instance
    #[serde(default)]
    pub name: String, // friendly unique instance name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comments: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,
    #[serde(rename = "dependsOn", skip_serializing_if = "Option::is_none")]
    #[schemars(regex(pattern = r"^\[resourceId\(\s*'[a-zA-Z0-9\.]+/[a-zA-Z0-9]+'\s*,\s*'[a-zA-Z0-9 ]+'\s*\)]$"))]
    pub depends_on: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Map<String, Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub identity: Option<Identity>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sku: Option<Sku>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub copy: Option<Copy>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub plan: Option<Plan>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub properties: Option<Map<String, Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resources: Option<Vec<Resource>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Metadata>,
}

impl Default for Configuration {
    fn default() -> Self {
        Self::new()
    }
}

impl Configuration {
    #[must_use]
    pub fn new() -> Self {
        Self {
            schema: Self::default_schema_id_uri(),
            content_version: Some("1.0.0".to_string()),
            metadata: None,
            parameters: None,
            resources: Vec::new(),
            functions: None,
            variables: None,
            outputs: None,
        }
    }
}

impl Resource {
    #[must_use]
    pub fn new() -> Self {
        Self {
            resource_type: FullyQualifiedTypeName::default(),
            name: String::new(),
            depends_on: None,
            kind: None,
            properties: None,
            metadata: None,
            condition: None,
            identity: None,
            sku: None,
            scope: None,
            copy: None,
            plan: None,
            resources: None,
            comments: None,
            location: None,
            tags: None,
            require_version: None,
        }
    }
}

impl Default for Resource {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod test {
    use crate::schemas::dsc_repo::{DscRepoSchema, UnrecognizedSchemaUri};

    use crate::configure::config_doc::Configuration;

    #[test]
    fn test_validate_schema_uri_with_invalid_uri() {
        let invalid_uri = "https://invalid.schema.uri".to_string();

        let manifest = Configuration{
            schema: invalid_uri.clone(),
            ..Default::default()
        };

        let ref result = manifest.validate_schema_uri();

        assert!(result.as_ref().is_err());

        match result.as_ref().unwrap_err() {
            UnrecognizedSchemaUri(actual, recognized) => {
                assert_eq!(actual, &invalid_uri);
                assert_eq!(recognized, &Configuration::recognized_schema_uris())
            },
        }
    }

    #[test]
    fn test_validate_schema_uri_with_valid_uri() {
        let manifest = Configuration{
            schema: Configuration::default_schema_id_uri(),
            ..Default::default()
        };

        let result = manifest.validate_schema_uri();

        assert!(result.is_ok());
    }

    #[test]
    fn test_invalid_resource_field_in_array() {
        let config_json = r#"{
            "resources": [
                {
                    "invalidField": "someValue"
                }
            ]
        }"#;

        let result: Result<Configuration, _> = serde_json::from_str(config_json);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.starts_with("unknown field `invalidField`, expected one of `condition`, `type`,"));
    }

    #[test]
    fn test_invalid_resource_type_in_array() {
        let config_json = r#"{
            "resources": [
                "invalidType"
            ]
        }"#;

        let result: Result<Configuration, _> = serde_json::from_str(config_json);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("expected struct Resource"));
    }

    #[test]
    fn test_resources_as_array() {
        let config_json = r#"{
            "$schema": "https://aka.ms/dsc/schemas/v3/bundled/config/document.json",
            "resources": [
                {
                    "type": "Microsoft.DSC.Debug/Echo",
                    "name": "echoResource",
                    "apiVersion": "1.0.0"
                },
                {
                    "type": "Microsoft/Process",
                    "name": "processResource",
                    "apiVersion": "0.1.0"
                }
            ]
        }"#;

        let config: Configuration = serde_json::from_str(config_json).unwrap();

        assert_eq!(config.resources.len(), 2);
        assert_eq!(config.resources[0].name, "echoResource");
        assert_eq!(config.resources[0].resource_type, "Microsoft.DSC.Debug/Echo");
        assert_eq!(config.resources[0].require_version.as_deref(), Some("1.0.0"));

        assert_eq!(config.resources[1].name, "processResource");
        assert_eq!(config.resources[1].resource_type, "Microsoft/Process");
        assert_eq!(config.resources[1].require_version.as_deref(), Some("0.1.0"));
    }

}
