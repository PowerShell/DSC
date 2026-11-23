// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::settings::{Authentication, Registry};
use rust_i18n::t;
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    io::{BufRead, Read, Write},
};

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
    serve_with_settings(
        input,
        output,
        &mut std::io::stderr(),
        crate::settings::effective,
    )
    .await
}

async fn serve_with_settings(
    input: &mut impl BufRead,
    output: &mut impl Write,
    diagnostics: &mut impl Write,
    read_settings: impl Fn() -> Result<BTreeMap<String, Registry>, String>,
) -> Result<(), String> {
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
                match read_settings() {
                    Err(error) => Err(error),
                    Ok(registries) => {
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
                                    writeln!(diagnostics, "{}", json!({
                                        "warn": t!("rpc.operation", registry = name, error = error).to_string()
                                    })).map_err(|error| error.to_string())?;
                                },
                            }
                        }
                        Ok(Value::Array(results))
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
    use crate::oci::tests::{RegistryServer, routes};
    use std::io::Cursor;

    #[tokio::test]
    async fn retains_healthy_registries_before_and_after_catalog_and_secret_failures() {
        for secret_service in [false, true] {
            let healthy = RegistryServer::new(routes(false), None);
            let unavailable = RegistryServer::new(routes(false), Some("sensitive"));
            let anonymous = Registry {
                location: healthy.url.clone(),
                authentication: None,
            };
            let registries = BTreeMap::from([
                ("a-healthy".into(), anonymous.clone()),
                (
                    "b-unavailable".into(),
                    Registry {
                        location: unavailable.url.clone(),
                        ..anonymous.clone()
                    },
                ),
                (
                    "c-secret".into(),
                    Registry {
                        authentication: Some(Authentication {
                            secret_name: "access".into(),
                            secret_vault: None,
                        }),
                        ..anonymous.clone()
                    },
                ),
                ("d-healthy".into(), anonymous),
            ]);
            let services = if secret_service {
                vec!["secret.get"]
            } else {
                Vec::new()
            };
            let mut requests = vec![
                json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{"hostServices":services}}),
                json!({"jsonrpc":"2.0","id":2,"method":"dsc_invoke","params":{"operation":"discover"}}),
            ];
            if secret_service {
                requests.push(json!({
                    "jsonrpc":"2.0","id":"oci-secret-1",
                    "error":{"code":-1,"message":"sensitive"}
                }));
            }
            requests.extend([
                json!({"jsonrpc":"2.0","id":3,"method":"shutdown"}),
                json!({"jsonrpc":"2.0","method":"exit"}),
            ]);
            let mut input = Cursor::new(
                requests
                    .into_iter()
                    .map(|request| format!("{request}\n"))
                    .collect::<String>(),
            );
            let mut output = Vec::new();
            let mut diagnostics = Vec::new();
            serve_with_settings(&mut input, &mut output, &mut diagnostics, || {
                Ok(registries.clone())
            })
            .await
            .unwrap();

            let output = String::from_utf8(output).unwrap();
            let responses: Vec<Value> = output
                .lines()
                .map(|line| serde_json::from_str(line).unwrap())
                .collect();
            let discovered = responses
                .iter()
                .find(|response| response["id"] == 2)
                .unwrap()["result"]
                .as_array()
                .unwrap();
            let names: Vec<_> = discovered
                .iter()
                .map(|result| result["registry"].as_str().unwrap())
                .collect();
            assert_eq!(names, ["a-healthy", "d-healthy"]);
            assert_eq!(
                responses
                    .iter()
                    .filter(|response| response["method"] == "secret.get")
                    .count(),
                usize::from(secret_service)
            );
            let diagnostics = String::from_utf8(diagnostics).unwrap();
            let warnings: Vec<Value> = diagnostics
                .lines()
                .map(|line| serde_json::from_str(line).unwrap())
                .collect();
            assert_eq!(warnings.len(), 2);
            assert!(
                warnings[0]["warn"]
                    .as_str()
                    .unwrap()
                    .contains("b-unavailable")
            );
            assert!(warnings[1]["warn"].as_str().unwrap().contains("c-secret"));
            assert!(!output.contains("sensitive"));
            assert!(!diagnostics.contains("sensitive"));
        }
    }

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
