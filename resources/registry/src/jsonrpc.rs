// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use std::io::{self, BufRead, Write};

use dsc_lib_registry::{RegistryHelper, config::Registry};
use rust_i18n::t;
use schemars::schema_for;
use serde::Deserialize;
use serde_json::{Value, json};

use crate::types::RegistryList;

#[derive(Default, PartialEq)]
enum State {
    #[default]
    Uninitialized,
    Running,
    Shutdown,
}

struct RpcError {
    code: i32,
    message: String,
}

impl RpcError {
    fn new(code: i32, message: impl ToString) -> Self {
        Self { code, message: message.to_string() }
    }

    fn invalid_params(message: impl ToString) -> Self {
        Self::new(-32602, message)
    }
}

#[derive(Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
enum ExecutionType {
    Actual,
    WhatIf,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct InvokeParams {
    operation: String,
    execution_type: ExecutionType,
    resource_type: String,
    properties: Option<Value>,
}

fn invoke(params: Value) -> Result<Value, RpcError> {
    let params: InvokeParams = serde_json::from_value(params).map_err(RpcError::invalid_params)?;
    let list = match params.resource_type.as_str() {
        "Microsoft.Windows/Registry" => false,
        "Microsoft.Windows/RegistryList" => true,
        _ => return Err(RpcError::invalid_params(t!("jsonrpc.unsupportedResource", resource = params.resource_type))),
    };
    if params.operation == "schema" {
        let schema = if list { schema_for!(RegistryList) } else { schema_for!(Registry) };
        return serde_json::to_value(schema).map_err(|err| RpcError::new(-32603, err));
    }
    if !matches!(params.operation.as_str(), "get" | "set" | "test" | "delete")
        || (list && params.operation == "delete")
    {
        return Err(RpcError::invalid_params(t!("jsonrpc.unsupportedOperation", operation = params.operation)));
    }
    let properties = params.properties.filter(Value::is_object)
        .ok_or_else(|| RpcError::invalid_params(t!("jsonrpc.invalidParams")))?;
    if list {
        let mut registry_list: RegistryList = serde_json::from_value(properties).map_err(RpcError::invalid_params)?;
        registry_list.inherit_registry_file_path();
        let mut entries = Vec::new();
        for registry in registry_list.registry_entries {
            let result = invoke_registry(&params.operation, &params.execution_type, &registry)?;
            if matches!(params.operation.as_str(), "get" | "test") || params.execution_type == ExecutionType::WhatIf {
                entries.push(result);
            }
        }
        return if params.operation == "set" && params.execution_type == ExecutionType::Actual {
            Ok(Value::Null)
        } else if params.operation == "test" {
            let in_desired_state = entries.iter().all(|entry| entry["_inDesiredState"] == true);
            Ok(json!({"registryEntries": entries, "_inDesiredState": in_desired_state}))
        } else {
            Ok(json!({"registryEntries": entries}))
        };
    }
    let registry: Registry = serde_json::from_value(properties).map_err(RpcError::invalid_params)?;
    invoke_registry(&params.operation, &params.execution_type, &registry)
}

fn invoke_registry(operation: &str, execution_type: &ExecutionType, registry: &Registry) -> Result<Value, RpcError> {
    let mut helper = RegistryHelper::new_from_registry(registry).map_err(RpcError::invalid_params)?;
    if *execution_type == ExecutionType::WhatIf {
        helper.enable_what_if();
    }
    let result = match operation {
        "get" => helper.get().map(Some),
        "test" => helper.test().map(Some),
        "delete" => helper.remove(),
        // Match the CLI's _exist handling, including synthetic deletion during set.
        _ if registry.exist == Some(false) => helper.remove(),
        _ => helper.set(),
    }.map_err(|err| RpcError::new(-32000, err))?;
    if matches!(operation, "set" | "delete") && *execution_type == ExecutionType::Actual {
        return Ok(Value::Null);
    }
    // An offline delete of a missing key has no preview; return its unchanged state.
    let result = match result {
        Some(registry) => registry,
        None => helper.get().map_err(|err| RpcError::new(-32000, err))?,
    };
    serde_json::to_value(result).map_err(|err| RpcError::new(-32603, err))
}

fn dispatch(method: &str, params: Value, state: &mut State) -> Result<Value, RpcError> {
    match method {
        "initialize" if *state == State::Uninitialized => {
            let versions = params.get("protocolVersions").and_then(Value::as_array)
                .ok_or_else(|| RpcError::invalid_params(t!("jsonrpc.invalidParams")))?;
            if !versions.contains(&json!(1)) {
                return Err(RpcError::invalid_params(t!("jsonrpc.unsupportedVersion")));
            }
            *state = State::Running;
            Ok(json!({"protocolVersion": 1}))
        },
        "dsc_invoke" if *state == State::Running => invoke(params),
        "shutdown" if *state == State::Running => {
            *state = State::Shutdown;
            Ok(Value::Null)
        },
        "initialize" | "dsc_invoke" | "shutdown" => {
            Err(RpcError::new(-32600, t!("jsonrpc.invalidLifecycle")))
        },
        _ => Err(RpcError::new(-32601, t!("jsonrpc.methodNotFound", method = method))),
    }
}

fn write_response(output: &mut impl Write, id: &Value, result: Result<Value, RpcError>) -> io::Result<()> {
    let response = match result {
        Ok(result) => json!({"jsonrpc": "2.0", "id": id, "result": result}),
        Err(err) => json!({"jsonrpc": "2.0", "id": id, "error": {"code": err.code, "message": err.message}}),
    };
    serde_json::to_writer(&mut *output, &response)?;
    writeln!(output)?;
    output.flush()
}

pub fn serve(input: impl BufRead, output: &mut impl Write) -> io::Result<()> {
    let mut state = State::default();
    for line in input.lines() {
        let request: Value = match serde_json::from_str(&line?) {
            Ok(request) => request,
            Err(err) => {
                write_response(output, &Value::Null, Err(RpcError::new(-32700, err)))?;
                continue;
            },
        };
        let id = request.get("id").cloned().unwrap_or(Value::Null);
        let method = request.get("method").and_then(Value::as_str);
        if !request.is_object() || request["jsonrpc"] != "2.0" || method.is_none()
            || (request.get("id").is_some() && !(id.is_i64() || id.is_u64() || id.is_string()))
        {
            write_response(output, &Value::Null, Err(RpcError::new(-32600, t!("jsonrpc.invalidRequest"))))?;
            continue;
        }
        let method = method.unwrap();
        if request.get("id").is_none() {
            if method == "exit" {
                return if state == State::Shutdown {
                    Ok(())
                } else {
                    Err(io::Error::other(t!("jsonrpc.unexpectedExit").to_string()))
                };
            }
            continue;
        }
        let result = dispatch(method, request.get("params").cloned().unwrap_or(Value::Null), &mut state);
        write_response(output, &id, result)?;
    }
    Err(io::Error::new(io::ErrorKind::UnexpectedEof, t!("jsonrpc.unexpectedEnd").to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(requests: &[Value]) -> Vec<Value> {
        let input = requests.iter().map(Value::to_string).collect::<Vec<_>>().join("\n");
        let mut output = Vec::new();
        serve(io::Cursor::new(input), &mut output).unwrap();
        String::from_utf8(output).unwrap().lines()
            .map(|line| serde_json::from_str(line).unwrap()).collect()
    }

    fn initialize() -> Value {
        json!({"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {"protocolVersions": [1]}})
    }

    fn shutdown() -> Value {
        json!({"jsonrpc": "2.0", "id": 100, "method": "shutdown"})
    }

    fn exit() -> Value {
        json!({"jsonrpc": "2.0", "method": "exit"})
    }

    fn request(id: i64, params: Value) -> Value {
        json!({"jsonrpc": "2.0", "id": id, "method": "dsc_invoke", "params": params})
    }

    #[test]
    fn lifecycle_and_repeated_schema_requests() {
        let params = json!({"operation": "schema", "executionType": "actual", "resourceType": "Microsoft.Windows/Registry"});
        let responses = run(&[initialize(), request(2, params.clone()), request(3, params), shutdown(), exit()]);
        assert_eq!(responses.len(), 4);
        assert_eq!(responses[0]["result"]["protocolVersion"], 1);
        assert_eq!(responses[1]["id"], 2);
        assert_eq!(responses[2]["id"], 3);
        assert_eq!(responses[1]["result"], serde_json::to_value(schema_for!(Registry)).unwrap());
        assert_eq!(responses[1]["result"], responses[2]["result"]);
        assert_eq!(responses[3]["result"], Value::Null);
    }

    #[test]
    fn selects_schema_for_each_resource_type() {
        let mut params = json!({"operation": "schema", "executionType": "actual",
            "resourceType": "Microsoft.Windows/Registry"});
        let single = request(2, params.clone());
        params["resourceType"] = json!("Microsoft.Windows/RegistryList");
        let responses = run(&[initialize(), single, request(3, params), shutdown(), exit()]);
        assert_eq!(responses[1]["result"], serde_json::to_value(schema_for!(Registry)).unwrap());
        assert_eq!(responses[2]["result"], serde_json::to_value(schema_for!(RegistryList)).unwrap());
    }

    #[test]
    fn empty_list_preserves_operation_return_contracts() {
        let params = json!({"operation": "get", "executionType": "actual",
            "resourceType": "Microsoft.Windows/RegistryList", "properties": {"registryEntries": []}});
        for (operation, execution, expected) in [
            ("get", "actual", json!({"registryEntries": []})),
            ("test", "actual", json!({"registryEntries": [], "_inDesiredState": true})),
            ("set", "actual", Value::Null),
            ("set", "whatIf", json!({"registryEntries": []})),
        ] {
            let mut params = params.clone();
            params["operation"] = json!(operation);
            params["executionType"] = json!(execution);
            assert_eq!(invoke(params).ok(), Some(expected));
        }
    }

    #[test]
    fn rejects_invalid_list_input_and_undeclared_delete() {
        for (operation, properties) in [
            ("delete", json!({"registryEntries": []})),
            ("get", json!({})),
            ("set", json!({"registryEntries": {}})),
            ("set", json!({"registryEntries": [{"keyPath": "HKCU"}, {}]})),
        ] {
            let result = invoke(json!({"operation": operation, "executionType": "actual",
                "resourceType": "Microsoft.Windows/RegistryList", "properties": properties}));
            assert_eq!(result.err().map(|err| err.code), Some(-32602));
        }
    }

    #[test]
    fn enforces_lifecycle_order() {
        let responses = run(&[
            request(0, json!({})), initialize(), initialize(), shutdown(),
            request(2, json!({})), exit(),
        ]);
        for index in [0, 2, 4] {
            assert_eq!(responses[index]["error"]["code"], -32600);
        }
    }

    #[test]
    fn rejects_unsupported_version_without_initializing() {
        let responses = run(&[
            json!({"jsonrpc": "2.0", "id": 0, "method": "initialize", "params": {"protocolVersions": [2]}}),
            initialize(), shutdown(), exit(),
        ]);
        assert_eq!(responses[0]["error"]["code"], -32602);
        assert_eq!(responses[1]["result"]["protocolVersion"], 1);
    }

    #[test]
    fn rejects_invalid_invocations_and_continues() {
        let valid = json!({"operation": "get", "executionType": "actual",
            "resourceType": "Microsoft.Windows/Registry", "properties": {"keyPath": "HKCU"}});
        let mut requests = vec![initialize()];
        for (field, value) in [
            ("operation", json!("unsupported")), ("executionType", json!("invalid")),
            ("resourceType", json!("Other/Registry")), ("properties", json!([])),
            ("properties", json!({})), ("properties", json!({"keyPath": "INVALID\\Key"})),
            ("properties", json!({"keyPath": "HKCU", "unknown": true})),
        ] {
            let mut params = valid.clone();
            params[field] = value;
            requests.push(request(2, params));
        }
        requests.extend([request(3, Value::Null), shutdown(), exit()]);
        let responses = run(&requests);
        for response in &responses[1..9] {
            assert_eq!(response["error"]["code"], -32602);
            assert!(response.get("result").is_none());
            assert!(response["error"]["message"].is_string());
        }
    }

    #[test]
    fn handles_malformed_messages_and_unknown_methods() {
        let mut input = String::from("not json\n");
        for request in [
            json!([]), json!({"jsonrpc": "1.0", "id": 2, "method": "initialize"}),
            initialize(), json!({"jsonrpc": "2.0", "id": "unknown", "method": "unknown"}),
            json!({"jsonrpc": "2.0", "method": "unknown"}), shutdown(), exit(),
        ] {
            input.push_str(&format!("{request}\n"));
        }
        let mut output = Vec::new();
        serve(io::Cursor::new(input), &mut output).unwrap();
        let responses: Vec<Value> = String::from_utf8(output).unwrap().lines()
            .map(|line| serde_json::from_str(line).unwrap()).collect();
        assert_eq!(responses.len(), 6);
        assert_eq!(responses[0]["error"]["code"], -32700);
        assert_eq!(responses[0]["id"], Value::Null);
        assert_eq!(responses[1]["error"]["code"], -32600);
        assert_eq!(responses[2]["error"]["code"], -32600);
        assert_eq!(responses[4]["error"]["code"], -32601);
        assert_eq!(responses[4]["id"], "unknown");
    }

    #[test]
    fn requires_shutdown_and_exit() {
        assert!(serve(io::Cursor::new(format!("{}\n", exit())), &mut Vec::new()).is_err());
        assert!(serve(io::Cursor::new(format!("{}\n", initialize())), &mut Vec::new()).is_err());
    }
}
