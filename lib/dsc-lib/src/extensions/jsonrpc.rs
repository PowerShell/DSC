// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use schemars::JsonSchema;
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;

use crate::dscresources::resource_manifest::JsonRpc;

/// An extension operation uses either its existing command contract or JSON-RPC.
#[derive(Debug, Clone, PartialEq, Serialize, JsonSchema)]
#[serde(untagged, deny_unknown_fields)]
#[schemars(inline)]
pub enum ExtensionMethod<T> {
    Command(T),
    JsonRpc { jsonrpc: JsonRpc },
}

impl<'de, T: serde::de::DeserializeOwned> Deserialize<'de> for ExtensionMethod<T> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = Value::deserialize(deserializer)?;
        if value.get("jsonrpc").is_some() {
            #[derive(Deserialize)]
            #[serde(deny_unknown_fields)]
            struct RpcMethod {
                jsonrpc: JsonRpc,
            }

            let method: RpcMethod = serde_json::from_value(value).map_err(serde::de::Error::custom)?;
            if method.jsonrpc.executable.is_empty()
                || method.jsonrpc.initialize_timeout_ms == Some(0)
                || method.jsonrpc.shutdown_timeout_ms == Some(0)
            {
                return Err(serde::de::Error::custom(
                    rust_i18n::t!("extensions.jsonrpc.invalidTransport")
                ));
            }
            Ok(Self::JsonRpc { jsonrpc: method.jsonrpc })
        } else {
            serde_json::from_value(value).map(Self::Command).map_err(serde::de::Error::custom)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ExtensionMethod;
    use crate::extensions::{discover::DiscoverMethod, extension_manifest::ExtensionManifest};
    use serde_json::{Value, json};
    use test_case::test_case;

    #[test_case(json!({"executable": "discover"}); "command")]
    #[test_case(json!({"executable": "discover", "network": true}); "network command")]
    #[test_case(json!({"jsonrpc": {"executable": "discover", "args": ["rpc"], "requestTimeoutMS": 0}}); "jsonrpc")]
    #[test_case(json!({"jsonrpc": {"executable": "discover"}, "network": true}); "network jsonrpc")]
    fn transport_round_trip(method: Value) {
        let parsed: DiscoverMethod = serde_json::from_value(method.clone()).unwrap();
        let mut expected = method;
        if expected.get("network").is_none() {
            expected["network"] = json!(false);
        }
        if expected.get("jsonrpc").is_none() {
            expected["args"] = Value::Null;
        }
        assert_eq!(serde_json::to_value(parsed).unwrap(), expected);
    }

    #[test_case(json!({"jsonrpc": null}); "null transport")]
    #[test_case(json!({"jsonrpc": {"executable": ""}}); "empty executable")]
    #[test_case(json!({"jsonrpc": {"executable": "discover", "initializeTimeoutMS": 0}}); "zero initialize timeout")]
    #[test_case(json!({"jsonrpc": {"executable": "discover", "shutdownTimeoutMS": 0}}); "zero shutdown timeout")]
    #[test_case(json!({"jsonrpc": {"executable": "discover", "args": [42]}}); "non-string arguments")]
    #[test_case(json!({"jsonrpc": {"executable": "discover", "input": "stdin"}}); "unknown transport field")]
    #[test_case(json!({"jsonrpc": {"executable": "discover"}, "executable": "fallback"}); "mixed transports")]
    #[test_case(json!({"executable": "discover", "jsonRpc": {}}); "misspelled transport")]
    #[test_case(json!({"executable": "discover", "network": "true"}); "non-boolean network")]
    #[test_case(json!({"jsonrpc": {"executable": "discover"}, "network": null}); "null network")]
    fn rejects_invalid_transport(method: Value) {
        assert!(serde_json::from_value::<DiscoverMethod>(method).is_err());
    }

    #[test]
    fn independently_selects_discover_and_secret_transports() {
        let manifest: ExtensionManifest = serde_json::from_value(json!({
            "$schema": "https://aka.ms/dsc/schemas/v3/bundled/extension/manifest.json",
            "type": "Test/MixedExtension",
            "version": "1.0.0",
            "discover": {"jsonrpc": {"executable": "discover"}},
            "secret": {"executable": "secret", "args": [{"nameArg": "--name"}]}
        })).unwrap();
        assert!(matches!(manifest.discover.unwrap().method, ExtensionMethod::JsonRpc { .. }));
        assert!(matches!(manifest.secret, Some(ExtensionMethod::Command(_))));
    }
}
