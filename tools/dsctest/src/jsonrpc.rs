// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use std::{fs::OpenOptions, io::{self, BufRead, Write}, thread, time::Duration};
use serde_json::{Value, json};

fn write_message(output: &mut impl Write, value: &Value) -> Result<(), Box<dyn std::error::Error>> {
    serde_json::to_writer(&mut *output, value)?;
    writeln!(output)?;
    output.flush()?;
    Ok(())
}

fn record(event: &str, params: &Value) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(path) = std::env::var_os("DSC_TEST_JSONRPC_LOG") {
        let mut file = OpenOptions::new().create(true).append(true).open(path)?;
        let mut entry = serde_json::to_vec(&json!({"pid": std::process::id(), "event": event, "params": params}))?;
        entry.push(b'\n');
        file.write_all(&entry)?;
    }
    Ok(())
}

fn schema() -> Value {
    json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "type": "object",
        "properties": {
            "value": {"type": "integer"},
            "processId": {"type": "integer", "readOnly": true},
            "requestCount": {"type": "integer", "readOnly": true},
            "transport": {"type": "string", "readOnly": true}
        },
        "additionalProperties": true
    })
}

fn invoke(params: &Value, state: &mut Value, count: u64, transport: &str) -> Value {
    let operation = params["operation"].as_str().unwrap_or_default();
    if operation == "schema" { return schema(); }
    if operation == "validate" {
        return json!({"valid": !(params["properties"]["rejectMetadata"] == true
            && params["properties"].get("_metadata").is_some())});
    }
    if operation == "resolve" {
        return json!({"configuration": params["properties"]["configuration"], "parameters": null});
    }
    let mut actual = params.get("properties").cloned().unwrap_or_else(|| json!({}));
    actual["value"] = state.get("value").cloned().unwrap_or(json!(0));
    actual["processId"] = json!(std::process::id());
    actual["requestCount"] = json!(count);
    actual["transport"] = json!(transport);
    if let Some(name) = actual["environmentName"].as_str() {
        actual["environmentValue"] = json!(std::env::var(name).ok());
    }
    let desired = params["properties"].get("value").cloned().unwrap_or(json!(0));
    let diff = if actual["value"] == desired { json!([]) } else { json!(["value"]) };
    let result = match operation {
        "set" | "whatIf" => {
            actual["value"] = desired;
            if params["executionType"] != "whatIf" { *state = actual.clone(); }
            json!({"actualState": actual, "diff": diff})
        },
        "test" => json!({"actualState": actual, "diff": diff}),
        "delete" if params["executionType"] == "whatIf" => {
            actual["_exist"] = json!(false);
            actual
        },
        "delete" => { *state = json!({}); Value::Null },
        "export" => json!([actual]),
        _ => actual,
    };
    if params["resourceType"] == "Test/JsonRpcSynthetic" && matches!(operation, "set" | "test") {
        Value::Null
    } else {
        result
    }
}

pub fn fallback(operation: &str, input: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
    if operation == "list" {
        let mut output = io::stdout().lock();
        for name in ["One", "Two"] {
            write_message(&mut output, &json!({
                "type": format!("Adapted/JsonRpc{name}"), "kind": "resource", "version": "0.1.0",
                "capabilities": ["get", "set", "test", "export"], "path": format!("jsonrpc/{name}"),
                "directory": "jsonrpc", "implementedAs": "TestAdapted", "properties": ["value"],
                "requireAdapter": "Test/JsonRpcAdapter"
            }))?;
        }
        return Ok(());
    }
    let properties = input.map(serde_json::from_str::<Value>).transpose()?.unwrap_or_else(|| json!({}));
    record("fallback", &json!({"operation": operation}))?;
    let result = invoke(&json!({"operation": operation, "properties": properties}), &mut json!({}), 1, "executable");
    let mut output = io::stdout().lock();
    if operation == "set" || operation == "test" {
        write_message(&mut output, &result["actualState"])?;
        write_message(&mut output, &result["diff"])
    } else {
        write_message(&mut output, &result)
    }
}

pub fn serve() -> Result<(), Box<dyn std::error::Error>> {
    let mut output = io::stdout().lock();
    let mut initialized = false;
    let mut shutting_down = false;
    let mut state = json!({});
    let mut count = 0;
    let lifecycle_fault = std::env::var("DSC_TEST_JSONRPC_FAULT").ok();
    record("start", &Value::Null)?;
    for line in io::stdin().lock().lines() {
        let request: Value = serde_json::from_str(&line?)?;
        let method = request["method"].as_str().ok_or("Missing method")?;
        let id = request.get("id").cloned().unwrap_or(Value::Null);
        let params = &request["params"];
        record(method, params)?;
        match (method, lifecycle_fault.as_deref()) {
            ("initialize", Some("initializeTimeout")) | ("shutdown", Some("shutdownTimeout"))
                | ("exit", Some("exitTimeout")) => thread::sleep(Duration::from_secs(10)),
            ("initialize", Some("initializeError")) | ("shutdown", Some("shutdownError")) => {
                write_message(&mut output, &json!({"jsonrpc": "2.0", "id": id,
                    "error": {"code": -32000, "message": "Requested lifecycle error"}}))?;
                continue;
            },
            ("initialize", Some("initializeVersion")) => {
                write_message(&mut output, &json!({"jsonrpc": "2.0", "id": id, "result": {"protocolVersion": 99}}))?;
                continue;
            },
            ("exit", Some("exitOutput")) => {
                write_message(&mut output, &json!({"jsonrpc": "2.0", "id": 0, "result": null}))?;
                return Ok(());
            },
            _ => {},
        }
        let result = match method {
            "initialize" if !initialized => {
                initialized = true;
                json!({"protocolVersion": 1})
            },
            "dsc_invoke" if initialized && !shutting_down => {
                count += 1;
                if params["operation"] == "schema" {
                    match lifecycle_fault.as_deref() {
                        Some("schemaError") => {
                            write_message(&mut output, &json!({"jsonrpc": "2.0", "id": id,
                                "error": {"code": -32001, "message": "Requested schema error"}}))?;
                            continue;
                        },
                        Some("schemaTimeout") => thread::sleep(Duration::from_secs(10)),
                        _ => {},
                    }
                }
                let properties = &params["properties"];
                if let Some(milliseconds) = properties["delayMS"].as_u64() {
                    thread::sleep(Duration::from_millis(milliseconds));
                }
                match properties["fault"].as_str() {
                    Some("exit") => return Ok(()),
                    Some("malformed") => { writeln!(output, "not json")?; output.flush()?; continue; },
                    Some("wrongId") => {
                        write_message(&mut output, &json!({"jsonrpc": "2.0", "id": 0, "result": {}}))?;
                        continue;
                    },
                    Some("nullId") => {
                        write_message(&mut output, &json!({"jsonrpc": "2.0", "id": null, "result": {}}))?;
                        continue;
                    },
                    Some("missingId") => {
                        write_message(&mut output, &json!({"jsonrpc": "2.0", "result": {}}))?;
                        continue;
                    },
                    Some("both") => {
                        write_message(&mut output, &json!({"jsonrpc": "2.0", "id": id, "result": {},
                            "error": {"code": -32000, "message": "Invalid response"}}))?;
                        continue;
                    },
                    Some("batch") => {
                        write_message(&mut output, &json!([{"jsonrpc": "2.0", "id": id, "result": {}}]))?;
                        continue;
                    },
                    Some("invalidResult") => {
                        write_message(&mut output, &json!({"jsonrpc": "2.0", "id": id, "result": null}))?;
                        continue;
                    },
                    Some("error") => {
                        write_message(&mut output, &json!({"jsonrpc": "2.0", "id": id, "error": {
                            "code": properties["errorCode"].as_i64().unwrap_or(-32001),
                            "message": "Requested test error", "data": {"details": "test diagnostic"}
                        }}))?;
                        continue;
                    },
                    _ => {},
                }
                if let Some(notifications) = properties["progress"].as_array() {
                    for notification in notifications {
                        let mut progress = json!({"requestId": id, "message": "JSON-RPC work"});
                        progress.as_object_mut().ok_or("Invalid progress")?
                            .extend(notification.as_object().ok_or("Progress must be an object")?.clone());
                        write_message(&mut output, &json!({"jsonrpc": "2.0", "method": "dsc_progress", "params": progress}))?;
                    }
                }
                if let Some(milliseconds) = properties["notificationDelayMS"].as_u64() {
                    for _ in 0..20 {
                        write_message(&mut output, &json!({"jsonrpc": "2.0", "method": "dsc_progress",
                            "params": {"requestId": id, "message": "Still working"}}))?;
                        thread::sleep(Duration::from_millis(milliseconds));
                    }
                }
                if properties["notifications"] == true {
                    write_message(&mut output, &json!({"jsonrpc": "2.0", "method": "dsc_trace",
                        "params": {"requestId": id, "level": "info", "message": "JSON-RPC trace"}}))?;
                    for completed in [0.5, 1.0] {
                        write_message(&mut output, &json!({"jsonrpc": "2.0", "method": "dsc_progress",
                            "params": {"requestId": id, "message": "JSON-RPC work", "completed": completed, "total": 1.0}}))?;
                    }
                }
                let result = invoke(params, &mut state, count, "jsonrpc");
                if properties["fault"] == "duplicate" {
                    write_message(&mut output, &json!({"jsonrpc": "2.0", "id": id, "result": result}))?;
                }
                result
            },
            "shutdown" if initialized => { shutting_down = true; Value::Null },
            "exit" if shutting_down => return Ok(()),
            _ => {
                write_message(&mut output, &json!({"jsonrpc": "2.0", "id": id,
                    "error": {"code": -32600, "message": "Invalid lifecycle request"}}))?;
                continue;
            },
        };
        write_message(&mut output, &json!({"jsonrpc": "2.0", "id": id, "result": result}))?;
    }
    Err("Unexpected end of input before exit".into())
}
