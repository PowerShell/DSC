// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use rust_i18n::t;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::collections::HashMap;

use crate::{dscerror::DscError, schemas::DscRepoSchema};

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum ContextKind {
    Configuration,
    Resource,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum SecurityContextKind {
    Current,
    Elevated,
    Restricted,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum Operation {
    Get,
    Set,
    Test,
    Export,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum ExecutionKind {
    Actual,
    WhatIf,
}

#[derive(Debug, Default, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
pub struct MicrosoftDscMetadata {
    /// Version of DSC
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    /// The operation being performed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operation: Option<Operation>,
    /// The type of execution
    #[serde(rename = "executionType", skip_serializing_if = "Option::is_none")]
    pub execution_type: Option<ExecutionKind>,
    /// The start time of the configuration operation
    #[serde(rename = "startDatetime", skip_serializing_if = "Option::is_none")]
    pub start_datetime: Option<String>,
    /// The end time of the configuration operation
    #[serde(rename = "endDatetime", skip_serializing_if = "Option::is_none")]
    pub end_datetime: Option<String>,
    /// The duration of the configuration operation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration: Option<String>,
    /// The security context of the configuration operation, can be specified to be required
    #[serde(rename = "securityContext", skip_serializing_if = "Option::is_none")]
    pub security_context: Option<SecurityContextKind>,
    /// Identifies if the operation is part of a configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<ContextKind>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
pub struct Metadata {
    #[serde(rename = "Microsoft.DSC", skip_serializing_if = "Option::is_none")]
    pub microsoft: Option<MicrosoftDscMetadata>,
    #[serde(flatten)]
    pub other: Map<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Configuration {
    #[serde(rename = "$schema")]
    #[schemars(schema_with = "Configuration::recognized_schema_uris_subschema")]
    pub schema: String,
    #[serde(rename = "contentVersion")]
    pub content_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters: Option<HashMap<String, Parameter>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variables: Option<Map<String, Value>>,
    pub resources: Vec<Resource>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Metadata>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
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

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
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

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
pub struct Resource {
    /// The fully qualified name of the resource type
    #[serde(rename = "type")]
    pub resource_type: String,
    /// A friendly name for the resource instance
    pub name: String, // friendly unique instance name
    #[serde(rename = "dependsOn", skip_serializing_if = "Option::is_none")]
    #[schemars(regex(pattern = r"^\[resourceId\(\s*'[a-zA-Z0-9\.]+/[a-zA-Z0-9]+'\s*,\s*'[a-zA-Z0-9 ]+'\s*\)]$"))]
    pub depends_on: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub properties: Option<Map<String, Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Map<String, Value>>,
}

impl Default for Configuration {
    fn default() -> Self {
        Self::new()
    }
}

impl DscRepoSchema for Configuration {
    const SCHEMA_FILE_BASE_NAME: &'static str = "document";
    const SCHEMA_FOLDER_PATH: &'static str = "config";
    const SCHEMA_SHOULD_BUNDLE: bool = true;

    fn schema_metadata() -> schemars::schema::Metadata {
        schemars::schema::Metadata {
            title: Some(t!("configure.config_doc.configurationDocumentSchemaTitle").into()),
            description: Some(t!("configure.config_doc.configurationDocumentSchemaDescription").into()),
            ..Default::default()
        }
    }

    fn validate_schema_uri(&self) -> Result<(), DscError> {
        if Self::is_recognized_schema_uri(&self.schema) {
            Ok(())
        } else {
            Err(DscError::UnrecognizedSchemaUri(
                self.schema.clone(),
                Self::recognized_schema_uris(),
            ))
        }
    }
}

impl Configuration {
    #[must_use]
    pub fn new() -> Self {
        Self {
            schema: Self::default_schema_id_uri(),
            content_version: Some("1.0.0".to_string()),
            parameters: None,
            variables: None,
            resources: Vec::new(),
            metadata: None,
        }
    }
}

impl Resource {
    #[must_use]
    pub fn new() -> Self {
        Self {
            resource_type: String::new(),
            name: String::new(),
            depends_on: None,
            properties: None,
            metadata: None,
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
    use crate::{
        configure::config_doc::Configuration,
        dscerror::DscError,
        schemas::DscRepoSchema
    };

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
            DscError::UnrecognizedSchemaUri(actual, recognized) => {
                assert_eq!(actual, &invalid_uri);
                assert_eq!(recognized, &Configuration::recognized_schema_uris())
            },
            _ => {
                panic!("Expected validate_schema_uri() to error on unrecognized schema uri, but was {:?}", result.as_ref().unwrap_err())
            }
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
}
