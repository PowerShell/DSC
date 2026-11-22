// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::settings::Authentication;
use rust_i18n::t;
use serde_json::{Value, json};
use std::io::{BufRead, Read, Write};

fn read(input: &mut impl BufRead) -> Result<Value, String> {
    let mut line = String::new();
    input
        .take(8 * 1024 * 1024 + 1)
        .read_line(&mut line)
        .map_err(|error| error.to_string())?;
    if line.len() > 8 * 1024 * 1024 {
        return Err(t!("oci.limit").to_string());
    }
    let value: Value = serde_json::from_str(&line).map_err(|_| t!("rpc.invalid").to_string())?;
    if value["jsonrpc"] != "2.0" || !value.is_object() {
        return Err(t!("rpc.invalid").to_string());
    }
    Ok(value)
}

fn write(output: &mut impl Write, message: &Value) -> Result<(), String> {
    serde_json::to_writer(&mut *output, message).map_err(|error| error.to_string())?;
    writeln!(output)
        .and_then(|()| output.flush())
        .map_err(|error| error.to_string())
}

pub fn secret(
    input: &mut impl BufRead,
    output: &mut impl Write,
    auth: &Authentication,
    id: u64,
) -> Result<String, String> {
    let mut params = json!({"name": auth.secret_name});
    if let Some(vault) = &auth.secret_vault {
        params["vault"] = json!(vault);
    }
    let callback_id = format!("oci-secret-{id}");
    write(
        output,
        &json!({"jsonrpc": "2.0", "id": callback_id, "method": "secret.get", "params": params}),
    )?;
    let response = read(input)?;
    if response["id"] != callback_id || response.get("error").is_some() {
        return Err(t!("rpc.secret").to_string());
    }
    response["result"]
        .as_object()
        .filter(|object| object.len() == 1)
        .and_then(|object| object.get("secureString"))
        .and_then(Value::as_str)
        .filter(|token| !token.trim().is_empty())
        .map(str::to_string)
        .ok_or_else(|| t!("rpc.secret").to_string())
}

pub async fn serve(input: &mut impl BufRead, output: &mut impl Write) -> Result<(), String> {
    let mut initialized = false;
    let mut shutting_down = false;
    let mut secret_service = false;
    let mut callback_id = 0;
    loop {
        let request = read(input)?;
        let method = request["method"]
            .as_str()
            .ok_or_else(|| t!("rpc.invalid").to_string())?;
        if method == "exit" && shutting_down && request.get("id").is_none() {
            return Ok(());
        }
        let id = request
            .get("id")
            .filter(|id| id.is_string() || id.is_i64() || id.is_u64())
            .ok_or_else(|| t!("rpc.invalid").to_string())?;
        let result = match method {
            "initialize" if !initialized => {
                initialized = true;
                secret_service = request["params"]["hostServices"]
                    .as_array()
                    .is_some_and(|services| services.iter().any(|service| service == "secret.get"));
                Ok(json!({"protocolVersion": 1, "operations": ["discover"]}))
            }
            "dsc_invoke"
                if initialized
                    && !shutting_down
                    && request["params"]["operation"] == "discover" =>
            {
                let mut results = Vec::new();
                match crate::settings::effective() {
                    Err(error) => Err(error),
                    Ok(registries) => {
                        let mut failure = None;
                        for (name, registry) in registries {
                            let token = match &registry.authentication {
                                Some(_) if !secret_service => {
                                    Err(t!("rpc.secretUnavailable").to_string())
                                }
                                Some(auth) => {
                                    callback_id += 1;
                                    secret(input, output, auth, callback_id).map(Some)
                                }
                                None => Ok(None),
                            };
                            let discovered = match token {
                                Ok(token) => {
                                    crate::oci::discover(&registry.location, token.as_deref()).await
                                }
                                Err(error) => Err(error),
                            };
                            match discovered {
                                Ok(manifests) => results.extend(manifests.into_iter().map(|manifest|
                                    json!({"manifestContent": manifest, "registry": name}))),
                                Err(error) => {
                                    failure = Some(t!("rpc.operation", registry = name, error = error).to_string());
                                    break;
                                },
                            }
                        }
                        failure.map_or_else(|| Ok(Value::Array(results)), Err)
                    }
                }
            }
            "shutdown" if initialized && !shutting_down => {
                shutting_down = true;
                Ok(Value::Null)
            }
            _ => Err(t!("rpc.invalid").to_string()),
        };
        let response = match result {
            Ok(result) => json!({"jsonrpc": "2.0", "id": id, "result": result}),
            Err(error) => {
                json!({"jsonrpc": "2.0", "id": id, "error": {"code": -32001, "message": error}})
            }
        };
        write(output, &response)?;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn reverse_secret_preserves_name_and_optional_vault_without_echoing_value() {
        let auth = Authentication {
            secret_name: "access".into(),
            secret_vault: Some("vault".into()),
        };
        let mut input = Cursor::new(b"{\"jsonrpc\":\"2.0\",\"id\":\"oci-secret-1\",\"result\":{\"secureString\":\"token\"}}\n");
        let mut output = Vec::new();
        assert_eq!(secret(&mut input, &mut output, &auth, 1).unwrap(), "token");
        let request: Value = serde_json::from_slice(&output).unwrap();
        assert_eq!(request["params"], json!({"name":"access","vault":"vault"}));
        assert!(!String::from_utf8(output).unwrap().contains("token"));
    }

    #[test]
    fn rejects_secret_errors_wrong_ids_and_objects_without_leaking_diagnostics() {
        let auth = Authentication {
            secret_name: "access".into(),
            secret_vault: None,
        };
        for response in [
            json!({"id":"wrong","result":{"secureString":"sensitive"}}),
            json!({"id":"oci-secret-1","result":{"secureObject":{"password":"sensitive"}}}),
            json!({"id":"oci-secret-1","error":{"code":-1,"message":"sensitive"}}),
        ] {
            let mut response = response;
            response["jsonrpc"] = json!("2.0");
            let mut input = Cursor::new(format!("{response}\n"));
            let mut output = Vec::new();
            let error = secret(&mut input, &mut output, &auth, 1).unwrap_err();
            assert!(!error.contains("sensitive"));
            let request: Value = serde_json::from_slice(&output).unwrap();
            assert!(request["params"].get("vault").is_none());
        }
    }
}
