// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use std::collections::HashMap;

use crate::error::RegistryResourceError;
use dsc_lib_registry::{RegistryHelper, config::RegistryValueData};
use rust_i18n::t;
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::{Map, Value};
use tracing::{debug, trace, warn};

#[derive(Deserialize)]
struct AdaptedRegistryResource {
    #[serde(flatten)]
    properties: Map<String, Value>,
}

#[derive(Debug, Deserialize, JsonSchema)]
enum RegistryDataType {
    #[serde(rename = "REG_BINARY")]
    Binary,
    #[serde(rename = "REG_DWORD")]
    Dword,
    #[serde(rename = "REG_EXPAND_SZ")]
    ExpandString,
    #[serde(rename = "REG_MULTI_SZ")]
    MultiString,
    #[serde(rename = "REG_SZ")]
    String,
    #[serde(rename = "REG_QWORD")]
    Qword,
}

#[derive(Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
enum JsonType {
    Boolean,
    BooleanArray,
    Number,
    NumberArray,
    String,
    StringArray,
}

#[derive(Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AdaptedRegistryValue {
    key_path: String,
    value_name: String,
    value_type: RegistryDataType,
    json_type: JsonType,
    map_json_to_registry: Value,
    default_value_if_not_found: Value,
}

fn build_resource_map(adapted_resource: &str) -> Result<HashMap<String, AdaptedRegistryValue>, RegistryResourceError> {
    let adapted: AdaptedRegistryResource = serde_json::from_str(adapted_resource)
        .map_err(|e| RegistryResourceError::AdaptedResource(e.to_string()))?;
    adapted.properties.into_iter()
        .map(|(k, v)| {
            let arv: AdaptedRegistryValue = serde_json::from_value(v)
                .map_err(|e| RegistryResourceError::AdaptedResource(e.to_string()))?;
            Ok((k, arv))
        })
        .collect()
}

pub fn adapter_get(input: &str, adapted_resource: &str) -> Result<String, RegistryResourceError> {
    let resource_map = build_resource_map(adapted_resource)?;
    let mut result = Map::new();

    let input_map: Map<String, Value> = serde_json::from_str(input)
        .map_err(|e| RegistryResourceError::AdaptedResource(e.to_string()))?;
    for (key, value) in input_map.iter() {
        if let Some(adapted_registry_value) = resource_map.get(key) {
            debug!("{}", t!("adapter.getProcessingKey", key = key));
            if let Some(json_map) = adapted_registry_value.map_json_to_registry.as_object() {
                let registry_data = get_registry_value_data(&adapted_registry_value.value_name, value, json_map, &adapted_registry_value.value_type)?;
                let registry_helper = RegistryHelper::new(&adapted_registry_value.key_path, Some(adapted_registry_value.value_name.clone()), Some(registry_data))?;
                match registry_helper.get() {
                    Ok(registry) => {
                        if let Some(exist) = registry.exist && !exist {
                            let default_registry_data = convert_default_value_to_registry_data(&adapted_registry_value.default_value_if_not_found, &adapted_registry_value.value_type)?;
                            let json_value = convert_registry_value_data_to_mapped_json(&default_registry_data, &adapted_registry_value.json_type, json_map)?;
                            result.insert(key.clone(), json_value);
                            continue;
                        }
                        if let Some(registry_value) = registry.value_data {
                            let json_value = convert_registry_value_data_to_mapped_json(&registry_value, &adapted_registry_value.json_type, json_map)?;
                            result.insert(key.clone(), json_value);
                        } else {
                            return Err(RegistryResourceError::AdaptedResource(t!("adapter.registryValueNotFound", key_path = adapted_registry_value.key_path, value_name = adapted_registry_value.value_name).to_string()));
                        }
                    },
                    Err(e) => {
                        return Err(RegistryResourceError::RegistryError(e));
                    }
                }
            } else {
                warn!("No mapping found for key {}", key);
            }   
        } else {
            debug!("{}", t!("adapter.getNoAdaptedRegistryValueFound", key = key));
        }
    }

    serde_json::to_string(&result).map_err(|e| RegistryResourceError::AdaptedResource(e.to_string()))
}

pub fn adapter_set(input: &str, adapted_resource: &str) -> Result<(), RegistryResourceError> {
    let resource_map = build_resource_map(adapted_resource)?;

    let input_map: Map<String, Value> = serde_json::from_str(input)
        .map_err(|e| RegistryResourceError::AdaptedResource(e.to_string()))?;
    for (key, value) in input_map.iter() {
        if let Some(adapted_registry_value) = resource_map.get(key) {
            debug!("{}", t!("adapter.setProcessingKey", key = key));
            if let Some(json_map) = adapted_registry_value.map_json_to_registry.as_object() {
                let registry_data = get_registry_value_data(&adapted_registry_value.value_name, value, json_map, &adapted_registry_value.value_type)?;
                let registry_helper = RegistryHelper::new(&adapted_registry_value.key_path, Some(adapted_registry_value.value_name.clone()), Some(registry_data))?;
                if let Err(e) = registry_helper.set() {
                    return Err(RegistryResourceError::RegistryError(e));
                }
            } else {
                warn!("No mapping found for key {}", key);
            }
        } else {
            debug!("{}", t!("adapter.setNoAdaptedRegistryValueFound", key = key));
        }
    }

    Ok(())
}

pub fn adapter_export(input: &str, adapted_resource: &str) -> Result<String, RegistryResourceError> {
    trace!("Adapter Export with input: {input}");

    // if input is provided, use that to perform a `get` and return that result
    // if no input is provided, then create an input that contains all keys in the adapted resource with first values and perform a `get` to return the default values for all keys
    let input: String = if input.is_empty() {
        let resource_map = build_resource_map(adapted_resource)?;
        let mut map = Map::new();
        for (key, adapted_registry_value) in &resource_map {
            if let Some(json_map) = adapted_registry_value.map_json_to_registry.as_object() {
                let first_key = json_map.keys().next().cloned().unwrap_or_default();
                map.insert(key.clone(), Value::String(first_key));
            } else  {
                return Err(RegistryResourceError::AdaptedResource(t!("adapter.mapJsonToRegistryNotFound", key = key).to_string()));
            }
        }
        serde_json::to_string(&map).map_err(|e| RegistryResourceError::AdaptedResource(e.to_string()))?
    } else {
        input.to_string()
    };
    adapter_get(&input, adapted_resource)
}

fn convert_default_value_to_registry_data(default_value: &Value, reg_type: &RegistryDataType) -> Result<RegistryValueData, RegistryResourceError> {
    match (default_value, reg_type) {
        (Value::Bool(b), RegistryDataType::Dword) => Ok(RegistryValueData::DWord(if *b { 1 } else { 0 })),
        (Value::Number(n), RegistryDataType::Dword) => {
            if let Some(u) = n.as_u64() {
                Ok(RegistryValueData::DWord(u as u32))
            } else {
                Err(RegistryResourceError::AdaptedResource(t!("adapter.couldNotConvertDefaultValue", default_value = default_value.to_string(), reg_type = reg_type : {:?}).to_string()))
            }
        },
        (Value::String(s), RegistryDataType::String) => Ok(RegistryValueData::String(s.clone())),
        (Value::String(s), RegistryDataType::ExpandString) => Ok(RegistryValueData::ExpandString(s.clone())),
        (Value::Array(a), RegistryDataType::MultiString) => {
            let mut result = Vec::new();
            for v in a {
                if let Value::String(s) = v {
                    result.push(s.clone());
                } else {
                    return Err(RegistryResourceError::AdaptedResource(t!("adapter.nonStringInMultiStringDefault", value = v.to_string()).to_string()));
                }
            }
            Ok(RegistryValueData::MultiString(result))
        },
        (Value::Array(a), RegistryDataType::Binary) => {
            let mut result = Vec::new();
            for v in a {
                if let Value::Number(s) = v {
                    if let Some(u) = s.as_u64() {
                        result.push(u as u8);
                    } else {
                        return Err(RegistryResourceError::AdaptedResource(t!("adapter.couldNotConvertDefaultValue", default_value = default_value.to_string(), reg_type = reg_type : {:?}).to_string()));
                    }
                } else {
                    return Err(RegistryResourceError::AdaptedResource(t!("adapter.couldNotConvertDefaultValue", default_value = default_value.to_string(), reg_type = reg_type : {:?}).to_string()));
                }
            }
            Ok(RegistryValueData::Binary(result))
        },
        _ => Err(RegistryResourceError::AdaptedResource(t!("adapter.couldNotConvertDefaultValue", default_value = default_value.to_string(), reg_type = reg_type : {:?}).to_string())),
    }
}

fn convert_registry_value_data_to_mapped_json(value_data: &RegistryValueData, json_type: &JsonType, map: &Map<String, Value>) -> Result<Value, RegistryResourceError> {
    // use the `map` to reverse convert the `RegistryValueData` back to the original JSON value based on the registry data type
    // convert the registry value to the appropriate JSON type based on `json_type`
    // for a boolean, a 0 is false and a 1 is true
    // a reg_binary for json_type that is an array will be null delimited
    let json_value = match (value_data, json_type) {
        (RegistryValueData::Binary(byte_vec), JsonType::StringArray) => {
            // Pre-build reverse lookup: deserialized bytes -> key name
            let reverse_map: Vec<(String, Vec<u8>)> = map.iter().filter_map(|(k, v)| {
                match serde_json::from_value::<Vec<u8>>(v.clone()) {
                    Ok(bytes) => Some((k.clone(), bytes)),
                    Err(_) => {
                        warn!("Failed to convert value to Vec<u8>: {:?}", v);
                        None
                    }
                }
            }).collect();
            if reverse_map.is_empty() {
                return Err(RegistryResourceError::AdaptedResource(t!("adapter.emptyReverseMap").to_string()));
            }
            let first_value_length = reverse_map.first().map(|(_, b)| b.len()).unwrap_or(0);
            let mut result = Vec::new();
            for slice in byte_vec.chunks(first_value_length) {
                let matched_key = reverse_map.iter().find_map(|(k, bytes)| {
                    if bytes == slice {
                        Some(k.clone())
                    } else {
                        None
                    }
                });
                if let Some(key) = matched_key {
                    result.push(Value::String(key));
                } else {
                    let hex_string = slice.iter().map(|b| format!("{:02X}", b)).collect::<Vec<String>>().join(" ");
                    return Err(RegistryResourceError::AdaptedResource(t!("adapter.unmappedByteSlice", hex_string = hex_string).to_string()));
                }
            }
            Value::Array(result)
        },
        (RegistryValueData::DWord(dword), JsonType::Boolean) => {
            match dword {
                0 => Value::Bool(false),
                1 => Value::Bool(true),
                _ => return Err(RegistryResourceError::AdaptedResource(t!("adapter.valueMappingNotFound", value = dword.to_string(), reg_type = "RegDword").to_string())),
            }
        },
        (RegistryValueData::DWord(dword), JsonType::String) => {
            let mapped_value = map.iter().find_map(|(k, v)| {
                if v.as_u64() == Some(*dword as u64) {
                    Some(k.clone())
                } else {
                    None
                }
            }).ok_or_else(|| RegistryResourceError::AdaptedResource(t!("adapter.valueMappingNotFound", value = dword.to_string(), reg_type = "RegDword").to_string()))?;
            Value::String(mapped_value)
        },
        (RegistryValueData::String(s), JsonType::String) => {
            let mapped_key = map.iter().find_map(|(k, v)| {
                if v.as_str().is_some_and(|v_str| v_str == s) {
                    Some(k.clone())
                } else {
                    None
                }
            }).ok_or_else(|| RegistryResourceError::AdaptedResource(t!("adapter.valueMappingNotFound", value = s, reg_type = "RegString").to_string()))?;
            Value::String(mapped_key.to_string())
        },
        (RegistryValueData::String(s), JsonType::Boolean) => {
            let mapped_value = map.iter().find_map(|(k, v)| {
                if v.as_str().is_some_and(|v_str| v_str == s) {
                    Some(k.clone())
                } else {
                    None
                }
            }).ok_or_else(|| RegistryResourceError::AdaptedResource(t!("adapter.valueMappingNotFound", value = s, reg_type = "RegString").to_string()))?;
            match mapped_value.as_str() {
                "true" => Value::Bool(true),
                "false" => Value::Bool(false),
                _ => return Err(RegistryResourceError::AdaptedResource(t!("adapter.valueMappingNotFound", value = s, reg_type = "RegString").to_string())),
            }
        },
        _ => return Err(RegistryResourceError::AdaptedResource(t!("adapter.unsupportedConversionToJsonType", registry_value_data = format!("{:?}", value_data), json_type = format!("{:?}", json_type)).to_string())),
    };
    Ok(json_value)
}

fn get_registry_value_data(value_name: &str, value: &Value, map: &Map<String, Value>, data_type: &RegistryDataType) -> Result<RegistryValueData, RegistryResourceError> {
    let registry_value_data = if let Some(value_array) = value.as_array() {
        match data_type {
            RegistryDataType::Binary => {
                let mut byte_vec = Vec::new();
                for item in value_array.iter() {
                    if let Some(s) = item.as_str() {
                        let mapped_value = map.get(s).ok_or_else(|| RegistryResourceError::AdaptedResource(t!("adapter.valueMappingNotFound", value = s, reg_type = "RegBinary").to_string()))?;
                        let byte_vec_item = serde_json::from_value::<Vec<u8>>(mapped_value.clone()).map_err(|e| RegistryResourceError::AdaptedResource(e.to_string()))?;
                        byte_vec.extend(byte_vec_item);
                    } else {
                        return Err(RegistryResourceError::AdaptedResource(t!("adapter.unsupportedValueType", value_name = value_name, value = format!("{:?}", item)).to_string()));
                    }
                }
                RegistryValueData::Binary(byte_vec)
            },
            RegistryDataType::MultiString => {
                let mut string_vec = Vec::new();
                for item in value_array.iter() {
                    if let Some(s) = item.as_str() {
                        let mapped_value = map.get(s).ok_or_else(|| RegistryResourceError::AdaptedResource(t!("adapter.valueMappingNotFound", value = s, reg_type = "RegMultiString").to_string()))?;
                        let string_vec_item = mapped_value.as_str().ok_or_else(|| RegistryResourceError::AdaptedResource(t!("adapter.unsupportedValueType", value_name = value_name, value = format!("{:?}", mapped_value)).to_string()))?.to_string();
                        string_vec.push(string_vec_item);
                    } else {
                        return Err(RegistryResourceError::AdaptedResource(t!("adapter.unsupportedValueType", value_name = value_name, value = format!("{:?}", item)).to_string()));
                    }
                }                    
                RegistryValueData::MultiString(string_vec)
            },
            _ => return Err(RegistryResourceError::AdaptedResource(t!("adapter.unsupportedValueType", value_name = value_name, value = format!("{:?}", value)).to_string())),
        }
    } else {
        let value_str = if let Some(s) = value.as_str() {
            s.to_string()
        } else if value.is_number() || value.is_boolean() {
            value.to_string()
        } else {
            return Err(RegistryResourceError::AdaptedResource(t!("adapter.unsupportedValueType", value_name = value_name, value = format!("{:?}", value)).to_string()));
        };
        match data_type {
            RegistryDataType::Binary => {
                let mapped_value = map.get(&value_str).ok_or_else(|| RegistryResourceError::AdaptedResource(t!("adapter.valueMappingNotFound", value = value_str, reg_type = "RegBinary").to_string()))?;
                let byte_vec = serde_json::from_value::<Vec<u8>>(mapped_value.clone()).map_err(|e| RegistryResourceError::AdaptedResource(e.to_string()))?;
                RegistryValueData::Binary(byte_vec)
            },
            RegistryDataType::Dword => {
                let mapped_value = map.get(&value_str).ok_or_else(|| RegistryResourceError::AdaptedResource(t!("adapter.valueMappingNotFound", value = value_str, reg_type = "RegDword").to_string()))?;
                let dword = mapped_value.as_u64().ok_or_else(|| RegistryResourceError::AdaptedResource(t!("adapter.mappedValueNotU64", value = value_str).to_string()))? as u32;
                RegistryValueData::DWord(dword)
            },
            RegistryDataType::ExpandString => {
                let mapped_value = map.get(&value_str).ok_or_else(|| RegistryResourceError::AdaptedResource(t!("adapter.valueMappingNotFound", value = value_str, reg_type = "RegExpandString").to_string()))?;
                let expand_string = mapped_value.as_str().ok_or_else(|| RegistryResourceError::AdaptedResource(t!("adapter.mappedValueNotString", value = value_str).to_string()))?.to_string();
                RegistryValueData::ExpandString(expand_string)
            },
            RegistryDataType::MultiString => {
                let mapped_value = map.get(&value_str).ok_or_else(|| RegistryResourceError::AdaptedResource(t!("adapter.valueMappingNotFound", value = value_str, reg_type = "RegMultiString").to_string()))?;
                let multi_string = serde_json::from_value::<Vec<String>>(mapped_value.clone()).map_err(|e| RegistryResourceError::AdaptedResource(e.to_string()))?;
                RegistryValueData::MultiString(multi_string)
            },
            RegistryDataType::Qword => {
                let mapped_value = map.get(&value_str).ok_or_else(|| RegistryResourceError::AdaptedResource(t!("adapter.valueMappingNotFound", value = value_str, reg_type = "RegQword").to_string()))?;
                let qword = mapped_value.as_u64().ok_or_else(|| RegistryResourceError::AdaptedResource(t!("adapter.mappedValueNotU64", value = value_str).to_string()))?;
                RegistryValueData::QWord(qword)
            },
            RegistryDataType::String => {
                let mapped_value = map.get(&value_str).ok_or_else(|| RegistryResourceError::AdaptedResource(t!("adapter.valueMappingNotFound", value = value_str, reg_type = "RegString").to_string()))?;
                let string = mapped_value.as_str().ok_or_else(|| RegistryResourceError::AdaptedResource(t!("adapter.mappedValueNotString", value = value_str).to_string()))?.to_string();
                RegistryValueData::String(string)
            },
        }
    };
    Ok(registry_value_data)
}
