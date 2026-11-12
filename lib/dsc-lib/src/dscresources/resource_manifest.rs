// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use rust_i18n::t;
use schemars::JsonSchema;
use semver::Version;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::{
    configure::config_doc::SecurityContextKind,
    schemas::{dsc_repo::DscRepoSchema, transforms::idiomaticize_string_enum},
    types::{ExitCodesMap, FullyQualifiedTypeName, ResourceVersion, TagList},
};

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema, DscRepoSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(
    transform = idiomaticize_string_enum,
    transform = Kind::transform_export_schema_uris,
    transform = Kind::transform_schema_docs,
)]
#[dsc_repo_schema(base_name = "resourceKind", folder_path = "definitions")]
pub enum Kind {
    Adapter,
    Exporter,
    Group,
    Importer,
    Resource,
}

#[derive(Debug, Default, Clone, PartialEq, Deserialize, Serialize, JsonSchema, DscRepoSchema)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
#[schemars(
    transform = ResourceManifest::transport_constraints,
    transform = ResourceManifest::transform_export_schema_uris,
    transform = ResourceManifest::transform_schema_docs,
)]
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
    /// Persistent JSON-RPC entry point. Operation commands are optional fallbacks.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jsonrpc: Option<JsonRpc>,
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
    /// An optional message indicating the resource is deprecated.  If provided, the message will be shown when the resource is used.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deprecation_message: Option<String>,
    /// The kind of resource.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kind: Option<Kind>,
    /// The version of the resource using semantic versioning.
    pub version: ResourceVersion,
    /// The author of the resource.
    pub author: Option<String>,
    /// The description of the resource.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Tags for the resource.
    #[serde(default, skip_serializing_if = "TagList::is_empty")]
    pub tags: TagList,
    /// Details how to call the Get method of the resource.
    pub get: Option<GetMethod>,
    /// Details how to call the Set method of the resource.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub set: Option<SetMethod>,
    /// Details how to call the `WhatIf` method of the resource.
    #[serde(skip_serializing_if = "Option::is_none")]
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
    #[serde(skip_serializing_if = "ExitCodesMap::is_empty_or_default", default)]
    pub exit_codes: ExitCodesMap,
    /// Details how to get the schema of the resource.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema: Option<SchemaKind>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Map<String, Value>>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema, DscRepoSchema)]
#[serde(untagged)]
#[schemars(
    transform = GetArgKind::transform_export_schema_uris,
    transform = GetArgKind::transform_schema_docs,
)]
#[dsc_repo_schema(base_name = "commandArgs.get", folder_path = "definitions")]
pub enum GetArgKind {
    /// The argument is a string.
    String(String),
    AdaptedContent {
        /// The argument that accepts the JSON content from the manifest.
        #[serde(rename = "adaptedContentArg")]
        adapted_content_arg: String,
    },
    Expression {
        /// The argument that accepts the result of the expression.
        arg: Option<String>,
        /// The expression to be evaluated.  `null` result becomes an empty string.
        expression: String,
        /// Indicates if the argument is mandatory.  Default is false.
        mandatory: Option<bool>,
    },
    Json {
        /// The argument that accepts the JSON input object.
        #[serde(rename = "jsonInputArg")]
        json_input_arg: String,
        /// Indicates if argument is mandatory which will pass an empty string if no JSON input is provided.  Default is false.
        mandatory: Option<bool>,
    },
    #[serde(rename_all = "camelCase")]
    ResourcePath {
        /// The argument that accepts the resource path.
        resource_path_arg: String,
        /// Indicates if the argument should be wrapped in quotes.  Default is false.
        #[serde(default)]
        include_quotes: bool,
    },
    #[serde(rename_all = "camelCase")]
    ResourceType {
        /// The argument that accepts the resource type name.
        resource_type_arg: String,
    },
    #[serde(rename_all = "camelCase")]
    ResourceVersion {
        /// The argument that accepts the resource version.
        resource_version_arg: String,
    },
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema, DscRepoSchema)]
#[serde(untagged)]
#[schemars(
    transform = SetDeleteArgKind::transform_export_schema_uris,
    transform = SetDeleteArgKind::transform_schema_docs,
)]
#[dsc_repo_schema(base_name = "commandArgs.setDelete", folder_path = "definitions")]
pub enum SetDeleteArgKind {
    /// The argument is a string.
    String(String),
    AdaptedContent {
        /// The argument that accepts the JSON content from the manifest.
        #[serde(rename = "adaptedContentArg")]
        adapted_content_arg: String,
    },
    Expression {
        /// The argument that accepts the result of the expression.
        arg: Option<String>,
        /// The expression to be evaluated.  `null` result becomes an empty string.
        expression: String,
        /// Indicates if the argument is mandatory.  Default is false.
        mandatory: Option<bool>,
    },
    Json {
        /// The argument that accepts the JSON input object.
        #[serde(rename = "jsonInputArg")]
        json_input_arg: String,
        /// Indicates if argument is mandatory which will pass an empty string if no JSON input is provided.  Default is false.
        mandatory: Option<bool>,
    },
    #[serde(rename_all = "camelCase")]
    ResourcePath {
        /// The argument that accepts the resource path.
        resource_path_arg: String,
        /// Indicates if the resource path should be passed with quotes.  Default is false.
        #[serde(default)]
        include_quotes: bool,
    },
    #[serde(rename_all = "camelCase")]
    ResourceType {
        /// The argument that accepts the resource type name.
        resource_type_arg: String,
    },
    #[serde(rename_all = "camelCase")]
    ResourceVersion {
        /// The argument that accepts the resource version.
        resource_version_arg: String,
    },
    /// The argument is passed when the resource is invoked in what-if mode.
    #[serde(rename_all = "camelCase")]
    WhatIf {
        /// The argument to pass when in what-if mode.
        what_if_arg: String,
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema, DscRepoSchema)]
#[serde(untagged)]
#[schemars(
    transform = SchemaArgKind::transform_export_schema_uris,
    transform = SchemaArgKind::transform_schema_docs,
)]
#[dsc_repo_schema(base_name = "commandArgs.schema", folder_path = "definitions")]
pub enum SchemaArgKind {
    /// The argument is a string.
    String(String),
    #[serde(rename_all = "camelCase")]
    ResourceType {
        /// The argument that accepts the resource type name.
        resource_type_arg: String,
    },
    #[serde(rename_all = "camelCase")]
    ResourceVersion {
        /// The argument that accepts the resource version.
        resource_version_arg: String,
    },
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema, DscRepoSchema)]
#[schemars(
    transform = idiomaticize_string_enum,
    transform = InputKind::transform_schema_docs,
    transform = InputKind::transform_export_schema_uris,
)]
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
#[schemars(
    transform = SchemaKind::transform_export_schema_uris,
    transform = SchemaKind::transform_schema_docs,
)]
#[dsc_repo_schema(base_name = "manifest.schema", folder_path = "resource")]
pub enum SchemaKind {
    /// The schema is returned by running a command.
    #[serde(rename = "command")]
    Command(SchemaCommand),
    /// The schema is embedded in the manifest.
    #[serde(rename = "embedded")]
    Embedded(Value),
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema, DscRepoSchema)]
#[dsc_repo_schema(base_name = "manifest.exportSchema", folder_path = "definitions")]
#[serde(rename_all = "camelCase")]
#[schemars(
    transform = ExportSchemaKind::transform_export_schema_uris,
    transform = ExportSchemaKind::transform_schema_docs,
)]
pub enum ExportSchemaKind {
    /// The export schema is returned by running a command.
    Command(SchemaCommand),
    /// The export schema is embedded in the manifest.
    Embedded(Value),
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
#[schemars(inline)]
pub struct SchemaCommand {
    /// The command to run to get the schema.
    #[serde(default, deserialize_with = "deserialize_executable", skip_serializing_if = "String::is_empty")]
    pub executable: String,
    /// The arguments to pass to the command.
    pub args: Option<Vec<SchemaArgKind>>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema, DscRepoSchema)]
#[schemars(
    transform = idiomaticize_string_enum,
    transform = ReturnKind::transform_schema_docs,
    transform = ReturnKind::transform_export_schema_uris,
)]
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
#[schemars(
    transform = GetMethod::transform_export_schema_uris,
    transform = GetMethod::transform_schema_docs,
)]
pub struct GetMethod {
    /// The command to run to get the state of the resource.
    #[serde(default, deserialize_with = "deserialize_executable", skip_serializing_if = "String::is_empty")]
    pub executable: String,
    /// The arguments to pass to the command to perform a Get.
    pub args: Option<Vec<GetArgKind>>,
    /// How to pass optional input for a Get.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input: Option<InputKind>,
    /// The security context required to run the Get method.  Default if not specified is `current`.
    #[serde(rename = "requireSecurityContext", skip_serializing_if = "Option::is_none")]
    pub require_security_context: Option<SecurityContextKind>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema, DscRepoSchema)]
#[dsc_repo_schema(base_name = "manifest.set", folder_path = "resource")]
#[schemars(
    transform = SetMethod::transform_export_schema_uris,
    transform = SetMethod::transform_schema_docs,
)]
pub struct SetMethod {
    /// Whether JSON-RPC set supports native what-if.
    #[serde(rename = "supportsWhatIf", skip_serializing_if = "Option::is_none")]
    pub supports_what_if: Option<bool>,
    /// The command to run to set the state of the resource.
    #[serde(default, deserialize_with = "deserialize_executable", skip_serializing_if = "String::is_empty")]
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
    /// The security context required to run the Set method.  Default if not specified is `current`.
    #[serde(rename = "requireSecurityContext", skip_serializing_if = "Option::is_none")]
    pub require_security_context: Option<SecurityContextKind>,
    /// The type of return value expected from the Set method when running in what-if mode. When specified, this overrides the `return` property during what-if execution.
    #[serde(rename = "whatIfReturns", skip_serializing_if = "Option::is_none")]
    pub what_if_returns: Option<ReturnKind>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema, DscRepoSchema)]
#[dsc_repo_schema(base_name = "manifest.test", folder_path = "resource")]
#[schemars(
    transform = TestMethod::transform_export_schema_uris,
    transform = TestMethod::transform_schema_docs,
)]
pub struct TestMethod {
    /// The command to run to test the state of the resource.
    #[serde(default, deserialize_with = "deserialize_executable", skip_serializing_if = "String::is_empty")]
    pub executable: String,
    /// The arguments to pass to the command to perform a Test.
    pub args: Option<Vec<GetArgKind>>,
    /// How to pass required input for a Test.
    pub input: Option<InputKind>,
    /// The type of return value expected from the Test method.
    #[serde(rename = "return", skip_serializing_if = "Option::is_none")]
    pub returns: Option<ReturnKind>,
    /// The security context required to run the Test method.  Default if not specified is `current`.
    #[serde(rename = "requireSecurityContext", skip_serializing_if = "Option::is_none")]
    pub require_security_context: Option<SecurityContextKind>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema, DscRepoSchema)]
#[dsc_repo_schema(base_name = "manifest.delete", folder_path = "resource")]
#[schemars(
    transform = DeleteMethod::transform_export_schema_uris,
    transform = DeleteMethod::transform_schema_docs,
)]
pub struct DeleteMethod {
    /// Whether JSON-RPC delete supports native what-if.
    #[serde(rename = "supportsWhatIf", skip_serializing_if = "Option::is_none")]
    pub supports_what_if: Option<bool>,
    /// The command to run to delete the state of the resource.
    #[serde(default, deserialize_with = "deserialize_executable", skip_serializing_if = "String::is_empty")]
    pub executable: String,
    /// The arguments to pass to the command to perform a Delete.
    pub args: Option<Vec<SetDeleteArgKind>>,
    /// How to pass required input for a Delete.
    pub input: Option<InputKind>,
    /// The security context required to run the Delete method.  Default if not specified is `current`.
    #[serde(rename = "requireSecurityContext", skip_serializing_if = "Option::is_none")]
    pub require_security_context: Option<SecurityContextKind>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema, DscRepoSchema)]
#[dsc_repo_schema(base_name = "manifest.validate", folder_path = "resource")]
#[schemars(
    transform = ValidateMethod::transform_export_schema_uris,
    transform = ValidateMethod::transform_schema_docs,
)]
pub struct ValidateMethod { // TODO: enable validation via schema or command
    /// The command to run to validate the state of the resource.
    #[serde(default, deserialize_with = "deserialize_executable", skip_serializing_if = "String::is_empty")]
    pub executable: String,
    /// The arguments to pass to the command to perform a Validate.
    pub args: Option<Vec<GetArgKind>>,
    /// How to pass required input for a Validate.
    pub input: Option<InputKind>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(inline)]
pub enum ExportSchemaOrFiltering {
    Schema(ExportSchemaKind),
    SupportsFiltering(bool),
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema, DscRepoSchema)]
#[dsc_repo_schema(base_name = "manifest.export", folder_path = "resource")]
#[schemars(
    transform = ExportMethod::transform_export_schema_uris,
    transform = ExportMethod::transform_schema_docs,
)]
pub struct ExportMethod {
    /// The command to run to enumerate instances of the resource.
    #[serde(default, deserialize_with = "deserialize_executable", skip_serializing_if = "String::is_empty")]
    pub executable: String,
    /// The arguments to pass to the command to perform a Export.
    pub args: Option<Vec<GetArgKind>>,
    /// How to pass input for a Export.
    pub input: Option<InputKind>,
    /// The security context required to run the Export method.  Default if not specified is `current`.
    #[serde(rename = "requireSecurityContext", skip_serializing_if = "Option::is_none")]
    pub require_security_context: Option<SecurityContextKind>,
    #[serde(flatten, deserialize_with = "deserialize_export_schema_or_filtering", skip_serializing_if = "Option::is_none")]
    pub schema_or_filtering: Option<ExportSchemaOrFiltering>,
}

fn deserialize_export_schema_or_filtering<'de, D: serde::Deserializer<'de>>(deserializer: D)
    -> Result<Option<ExportSchemaOrFiltering>, D::Error>
{
    let mut fields = Map::<String, Value>::deserialize(deserializer)?;
    fields.retain(|name, value| matches!(name.as_str(), "schema" | "supportsFiltering") && !value.is_null());
    if fields.is_empty() { return Ok(None); }
    serde_json::from_value(Value::Object(fields)).map(Some).map_err(serde::de::Error::custom)
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema, DscRepoSchema)]
#[dsc_repo_schema(base_name = "manifest.resolve", folder_path = "resource")]
#[schemars(
    transform = ResolveMethod::transform_export_schema_uris,
    transform = ResolveMethod::transform_schema_docs,
)]
pub struct ResolveMethod {
    /// The command to run to enumerate instances of the resource.
    #[serde(default, deserialize_with = "deserialize_executable", skip_serializing_if = "String::is_empty")]
    pub executable: String,
    /// The arguments to pass to the command to perform a Export.
    pub args: Option<Vec<GetArgKind>>,
    /// How to pass input for a Export.
    pub input: Option<InputKind>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema, DscRepoSchema)]
#[dsc_repo_schema(base_name = "manifest.adapter", folder_path = "resource")]
#[schemars(
    transform = Adapter::transform_export_schema_uris,
    transform = Adapter::transform_schema_docs,
)]
pub struct Adapter {
    /// The way to list adapter supported resources.
    pub list: Option<ListMethod>,
    /// Defines how the adapter supports accepting configuration.
    #[serde(alias = "config", rename = "inputKind")]
    pub input_kind: AdapterInputKind,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
#[schemars(inline, transform = idiomaticize_string_enum)]
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
#[schemars(inline)]
pub struct ListMethod {
    /// The command to run to list resources supported by a group resource.
    pub executable: String,
    /// The arguments to pass to the command to perform a List.
    pub args: Option<Vec<String>>,
}

fn deserialize_executable<'de, D: serde::Deserializer<'de>>(deserializer: D) -> Result<String, D::Error> {
    let executable = String::deserialize(deserializer)?;
    if executable.is_empty() {
        return Err(serde::de::Error::custom(t!("dscresources.jsonrpc.emptyExecutable")));
    }
    Ok(executable)
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
#[schemars(inline)]
pub struct JsonRpc {
    #[schemars(length(min = 1))]
    pub executable: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub args: Option<Vec<String>>,
    #[serde(rename = "initializeTimeoutMS", skip_serializing_if = "Option::is_none")]
    #[schemars(range(min = 1))]
    pub initialize_timeout_ms: Option<u64>,
    #[serde(rename = "requestTimeoutMS", skip_serializing_if = "Option::is_none")]
    pub request_timeout_ms: Option<u64>,
    #[serde(rename = "shutdownTimeoutMS", skip_serializing_if = "Option::is_none")]
    #[schemars(range(min = 1))]
    pub shutdown_timeout_ms: Option<u64>,
}

impl ResourceManifest {
    fn transport_constraints(schema: &mut schemars::Schema) {
        let fallback = serde_json::json!({"anyOf": [{"type": "null"}, {"required": ["executable"]}]});
        let mut properties = Map::new();
        let mut fallbacks = Map::new();
        let mut constraints = Vec::new();
        for operation in ["get", "set", "whatIf", "test", "delete", "export", "resolve", "validate"] {
            properties.insert(operation.to_string(), serde_json::json!({
                "properties": {"executable": {"minLength": 1}},
                "if": {"not": {"required": ["executable"]}},
                "then": {"properties": {"args": {"type": "null"}, "input": {"type": "null"}}}
            }));
            fallbacks.insert(operation.to_string(), fallback.clone());
        }
        fallbacks.insert("schema".to_string(), serde_json::json!({"properties": {"command": fallback}}));
        fallbacks.get_mut("export").unwrap()["properties"] =
            serde_json::json!({"schema": {"properties": {"command": fallback}}});
        constraints.push(serde_json::json!({
            "if": {"not": {"required": ["jsonrpc"], "properties": {"jsonrpc": {"type": "object"}}}},
            "then": {"properties": fallbacks}
        }));
        properties.insert("schema".to_string(), serde_json::json!({"properties": {"command": {
            "properties": {"executable": {"minLength": 1}},
            "if": {"not": {"required": ["executable"]}},
            "then": {"properties": {"args": {"type": "null"}}}
        }}}));
        properties.get_mut("export").unwrap()["properties"] = serde_json::json!({"schema": {"properties": {"command": {
            "properties": {"executable": {"minLength": 1}},
            "if": {"not": {"required": ["executable"]}},
            "then": {"properties": {"args": {"type": "null"}}}
        }}}});
        constraints.push(serde_json::json!({"properties": properties}));
        constraints.push(serde_json::json!({
            "if": {"required": ["jsonrpc"], "properties": {"jsonrpc": {"type": "object"}}},
            "then": {"properties": {"adapter": {"properties": {"inputKind": {"const": "single"}}}}}
        }));
        for operation in ["set", "delete"] {
            constraints.push(serde_json::json!({
                "if": {
                    "required": [operation],
                    "properties": {(operation): {
                        "type": "object", "required": ["supportsWhatIf"],
                        "properties": {"supportsWhatIf": {"const": true}}
                    }}
                },
                "then": {
                    "required": ["jsonrpc"],
                    "properties": {
                        "jsonrpc": {"type": "object"},
                        (operation): {
                            "if": {"required": ["executable"]},
                            "then": {"required": ["args"], "properties": {"args": {"type": "array", "contains": {
                                "type": "object", "required": ["whatIfArg"]
                            }}}}
                        }
                    }
                }
            }));
        }
        schema.insert("allOf".to_string(), Value::Array(constraints));
    }

    pub(crate) fn validate_transport(&self) -> Result<(), crate::DscError> {
        let value = serde_json::to_value(self)?;
        let invalid = || crate::DscError::InvalidManifest(
            t!("dscresources.jsonrpc.invalidManifest", resource = self.resource_type).to_string()
        );
        if let Some(rpc) = &self.jsonrpc
            && (rpc.executable.is_empty()
                || rpc.initialize_timeout_ms == Some(0)
                || rpc.shutdown_timeout_ms == Some(0)
                || self.adapter.as_ref().is_some_and(|adapter| adapter.input_kind != AdapterInputKind::Single))
        {
                return Err(invalid());
        }
        for operation in ["get", "set", "whatIf", "test", "delete", "export", "resolve", "validate"] {
            let Some(method) = value.get(operation).filter(|v| !v.is_null()) else { continue };
            let executable = method.get("executable").and_then(Value::as_str);
            if executable.is_none_or(str::is_empty) {
                if self.jsonrpc.is_none() || method.get("args").is_some_and(|v| !v.is_null())
                    || method.get("input").is_some_and(|v| !v.is_null())
                {
                    return Err(invalid());
                }
            } else if method.get("supportsWhatIf") == Some(&Value::Bool(true))
                && !method.get("args").and_then(Value::as_array).is_some_and(|args|
                    args.iter().any(|arg| arg.get("whatIfArg").is_some()))
            {
                return Err(invalid());
            }
            if method.get("supportsWhatIf") == Some(&Value::Bool(true)) && self.jsonrpc.is_none() {
                return Err(invalid());
            }
        }
        let resource_schema = match &self.schema {
            Some(SchemaKind::Command(command)) => Some(command),
            _ => None,
        };
        let export_schema = match self.export.as_ref().and_then(|export| export.schema_or_filtering.as_ref()) {
            Some(ExportSchemaOrFiltering::Schema(ExportSchemaKind::Command(command))) => Some(command),
            _ => None,
        };
        for command in [resource_schema, export_schema].into_iter().flatten() {
            if command.executable.is_empty() && (self.jsonrpc.is_none() || command.args.is_some()) {
                return Err(invalid());
            }
        }
        Ok(())
    }
}

/// Validate a semantic version string.
///
/// # Arguments
///
/// * `version` - The semantic version string to validate.
///
/// # Returns
///
/// * `Result<(), Error>` - The result of the validation.
///
/// # Errors
///
/// * `Error` - The version string is not a valid semantic version.
pub fn validate_semver(version: &str) -> Result<(), semver::Error> {
    Version::parse(version)?;
    Ok(())
}

#[cfg(test)]
mod test {
    use crate::schemas::dsc_repo::{DscRepoSchema, UnrecognizedSchemaUriError};

    use crate::dscresources::resource_manifest::ResourceManifest;

    #[test]
    fn jsonrpc_manifest_constraints() {
        use serde_json::json;
        let schema = crate::dsc_repo_schema_for!(ResourceManifest);
        let validator = jsonschema::Validator::new(schema.as_value()).unwrap();
        let base = json!({
            "$schema": ResourceManifest::default_schema_id_uri(),
            "type": "Test/JsonRpc", "version": "0.1.0", "get": {},
            "jsonrpc": {"executable": "dsctest"}
        });
        for update in [
            json!({}),
            json!({"set": {"return": "stateAndDiff", "supportsWhatIf": true}}),
            json!({"schema": {"command": {}}}),
            json!({"export": {"schema": {"command": {}}}}),
            json!({"adapter": {"inputKind": "single"}}),
        ] {
            let mut manifest = base.clone();
            manifest.as_object_mut().unwrap().extend(update.as_object().unwrap().clone());
            assert!(serde_json::from_value::<ResourceManifest>(manifest.clone()).unwrap().validate_transport().is_ok(), "{manifest}");
            assert!(validator.is_valid(&manifest), "{manifest}: {:?}", validator.iter_errors(&manifest).collect::<Vec<_>>());
        }
        for update in [
            json!({"get": {"executable": ""}}),
            json!({"schema": {"command": {"executable": ""}}}),
            json!({"export": {"schema": {"command": {"executable": ""}}}}),
        ] {
            let mut manifest = base.clone();
            manifest.as_object_mut().unwrap().extend(update.as_object().unwrap().clone());
            assert!(serde_json::from_value::<ResourceManifest>(manifest.clone()).is_err(), "{manifest}");
            assert!(!validator.is_valid(&manifest), "{manifest}");
        }
        for update in [
            json!({"jsonrpc": null}),
            json!({"get": {"args": ["ignored"]}}),
            json!({"get": {"input": "stdin"}}),
            json!({"schema": {"command": {"args": []}}}),
            json!({"export": {"schema": {"command": {"args": []}}}}),
            json!({"adapter": {"inputKind": "full"}}),
            json!({"adapter": {"inputKind": "sequence"}}),
            json!({"jsonrpc": {"executable": "dsctest", "initializeTimeoutMS": 0}}),
            json!({"jsonrpc": {"executable": "dsctest", "shutdownTimeoutMS": 0}}),
            json!({"set": {"executable": "dsctest", "supportsWhatIf": true}}),
        ] {
            let mut manifest = base.clone();
            manifest.as_object_mut().unwrap().extend(update.as_object().unwrap().clone());
            assert!(serde_json::from_value::<ResourceManifest>(manifest.clone()).unwrap().validate_transport().is_err(), "{manifest}");
            assert!(!validator.is_valid(&manifest), "{manifest}");
        }
    }

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
            UnrecognizedSchemaUriError(actual, recognized) => {
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
