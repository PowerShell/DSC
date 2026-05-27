// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use std::collections::HashMap;

use crate::error::RegistryResourceError;
use dsc_lib_registry::{RegistryHelper, config::RegistryValueData};
use rust_i18n::t;
use serde::Deserialize;
use serde_json::{Map, Value};
use tracing::{debug, trace, warn};

#[derive(Deserialize)]
struct AdaptedRegistryResource {
    #[serde(flatten)]
    properties: Map<String, Value>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum RegistryDataType {
    RegBinary,
    RegDword,
    #[serde(rename = "REG_SZ")]
    RegString,
    #[serde(rename = "REG_EXPAND_SZ")]
    RegExpandString,
    #[serde(rename = "REG_MULTI_SZ")]
    RegMultiString,
    RegQword,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
enum JsonType {
    Boolean,
    BooleanArray,
    Number,
    NumberArray,
    String,
    StringArray,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct AdaptedRegistryValue {
    key_path: String,
    value_name: String,
    value_type: RegistryDataType,
    json_type: JsonType,
    map_json_to_registry: Value,
    default_value_if_not_found: Value,
}

pub fn adapter_get(input: &str, adapted_resource: &str) -> Result<String, RegistryResourceError> {
    let adapted_resource: AdaptedRegistryResource = serde_json::from_str(adapted_resource)
        .map_err(|e| RegistryResourceError::AdaptedResource(e.to_string()))?;
    let mut result = Map::new();
    let mut resource_map = HashMap::new();

    for (key, value) in adapted_resource.properties.iter() {
        let adapted_registry_value: AdaptedRegistryValue = serde_json::from_value(value.clone())
            .map_err(|e| RegistryResourceError::AdaptedResource(e.to_string()))?;
        resource_map.insert(key.clone(), adapted_registry_value);
    }

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
                        if let Some(exist) = registry.exist {
                            if !exist {
                                let default_registry_data = convert_default_value_to_registry_data(&adapted_registry_value.default_value_if_not_found, &adapted_registry_value.value_type)?;
                                let json_value = convert_registry_value_data_to_mapped_json(&default_registry_data, &adapted_registry_value.json_type, json_map)?;
                                result.insert(key.clone(), json_value);
                                continue;
                            }
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

    Ok(serde_json::to_string(&result).map_err(|e| RegistryResourceError::AdaptedResource(e.to_string()))?)
}

fn convert_default_value_to_registry_data(default_value: &Value, reg_type: &RegistryDataType) -> Result<RegistryValueData, RegistryResourceError> {
    match (default_value, reg_type) {
        (Value::Bool(b), RegistryDataType::RegDword) => Ok(RegistryValueData::DWord(if *b { 1 } else { 0 })),
        (Value::String(s), RegistryDataType::RegString) => Ok(RegistryValueData::String(s.clone())),
        (Value::String(s), RegistryDataType::RegExpandString) => Ok(RegistryValueData::ExpandString(s.clone())),
        (Value::Array(a), RegistryDataType::RegMultiString) => {
            let mut result = Vec::new();
            for v in a {
                if let Value::String(s) = v {
                    result.push(s.clone());
                }
            }
            Ok(RegistryValueData::MultiString(result))
        },
        (Value::Array(a), RegistryDataType::RegBinary) => {
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
            // use first value in map to get length of bytes to compare
            let first_value = map.values().next();
            let first_value_length = first_value.and_then(|v| v.as_array()).map(|a| a.len()).unwrap_or(0);
            let mut result = Vec::new();
            for slice in byte_vec.chunks(first_value_length) {
                let matched_key = map.iter().find_map(|(k, v)| {
                    if let Ok(mapped_bytes) = serde_json::from_value::<Vec<u8>>(v.clone()) {
                        if mapped_bytes == slice {
                            return Some(k.clone());
                        }
                    } else {
                        warn!("Failed to convert value to Vec<u8>: {:?}", v);
                    }
                    None
                });
                if let Some(key) = matched_key {
                    result.push(Value::String(key));
                } else {
                    // convert slice to string as hex bytes
                    let hex_string = slice.iter().map(|b| format!("{:02X}", b)).collect::<Vec<String>>().join(" ");
                    warn!("No mapping found for byte slice {hex_string}, skipping");
                }
            }
            Value::Array(result)
        },
        (RegistryValueData::DWord(dword), JsonType::Boolean) => {
            match dword {
                0 => Value::Bool(false),
                1 => Value::Bool(true),
                _ => return Err(RegistryResourceError::AdaptedResource(t!("adapter.couldNotConvertRegistryValue", key_path = "unknown", value_name = "unknown", json_type = "boolean").to_string())),
            }
        },
        (RegistryValueData::DWord(dword), JsonType::String) => {
            let mapped_value = map.iter().find_map(|(k, v)| {
                if v.as_u64().map_or(false, |num| num == *dword as u64) {
                    Some(k.clone())
                } else {
                    None
                }
            }).ok_or_else(|| RegistryResourceError::AdaptedResource(t!("adapter.mappingNotFound", key_path = "unknown", value_name = "unknown").to_string()))?;
            Value::String(mapped_value)
        },
        (RegistryValueData::String(s), JsonType::String) => {
            let mapped_key = map.iter().find_map(|(k, v)| {
                if v.as_str().map_or(false, |v_str| v_str == s) {
                    Some(k.clone())
                } else {
                    None
                }
            }).ok_or_else(|| RegistryResourceError::AdaptedResource(t!("adapter.mappingNotFound", key_path = "unknown", value_name = "unknown").to_string()))?;
            Value::String(mapped_key.to_string())
        },
        _ => return Err(RegistryResourceError::AdaptedResource(t!("adapter.unsupportedConversionToJsonType", registry_value_data = format!("{:?}", value_data), json_type = format!("{:?}", json_type)).to_string())),
    };
    Ok(json_value)
}

fn get_registry_value_data(value_name: &str, value: &Value, map: &Map<String, Value>, data_type: &RegistryDataType) -> Result<RegistryValueData, RegistryResourceError> {
    let registry_value_data = if value.is_array() {
        let value_array = value.as_array().unwrap();
        match data_type {
            RegistryDataType::RegBinary => {
                let mut byte_vec = Vec::new();
                for item in value_array.iter() {
                    if let Some(s) = item.as_str() {
                        let mapped_value = map.get(s).ok_or_else(|| RegistryResourceError::AdaptedResource(format!("No mapping found for value {} in binary type", s)))?;
                        let byte_vec_item = serde_json::from_value::<Vec<u8>>(mapped_value.clone()).map_err(|e| RegistryResourceError::AdaptedResource(e.to_string()))?;
                        byte_vec.extend(byte_vec_item);
                    } else {
                        return Err(RegistryResourceError::AdaptedResource(t!("adapter.unsupportedValueType", value_name = value_name, value = format!("{:?}", item)).to_string()));
                    }
                }
                RegistryValueData::Binary(byte_vec)
            },
            RegistryDataType::RegMultiString => {
                let mut string_vec = Vec::new();
                for item in value_array.iter() {
                    if let Some(s) = item.as_str() {
                        let mapped_value = map.get(s).ok_or_else(|| RegistryResourceError::AdaptedResource(format!("No mapping found for value {} in multi string type", s)))?;
                        let string_vec_item = mapped_value.as_str().ok_or_else(|| RegistryResourceError::AdaptedResource(format!("Mapped value for {} is not a string", s)))?.to_string();
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
        let value_str = if value.is_string() {
            value.as_str().unwrap().to_string()
        } else if value.is_number() || value.is_boolean() {
            value.to_string()
        } else {
            return Err(RegistryResourceError::AdaptedResource(t!("adapter.unsupportedValueType", value_name = value_name, value = format!("{:?}", value)).to_string()));
        };
        match data_type {
            RegistryDataType::RegBinary => {
                let mapped_value = map.get(&value_str).ok_or_else(|| RegistryResourceError::AdaptedResource(format!("No mapping found for value {} in binary type", value_str)))?;
                let byte_vec = serde_json::from_value::<Vec<u8>>(mapped_value.clone()).map_err(|e| RegistryResourceError::AdaptedResource(e.to_string()))?;
                RegistryValueData::Binary(byte_vec)
            },
            RegistryDataType::RegDword => {
                let mapped_value = map.get(&value_str).ok_or_else(|| RegistryResourceError::AdaptedResource(format!("No mapping found for value {} in dword type", value_str)))?;
                let dword = mapped_value.as_u64().ok_or_else(|| RegistryResourceError::AdaptedResource(format!("Mapped value for {} is not a u64", value_str)))? as u32;
                RegistryValueData::DWord(dword)
            },
            RegistryDataType::RegExpandString => {
                let mapped_value = map.get(&value_str).ok_or_else(|| RegistryResourceError::AdaptedResource(format!("No mapping found for value {} in expand string type", value_str)))?;
                let expand_string = mapped_value.as_str().ok_or_else(|| RegistryResourceError::AdaptedResource(format!("Mapped value for {} is not a string", value_str)))?.to_string();
                RegistryValueData::ExpandString(expand_string)
            },
            RegistryDataType::RegMultiString => {
                let mapped_value = map.get(&value_str).ok_or_else(|| RegistryResourceError::AdaptedResource(format!("No mapping found for value {} in multi string type", value_str)))?;
                let multi_string = serde_json::from_value::<Vec<String>>(mapped_value.clone()).map_err(|e| RegistryResourceError::AdaptedResource(e.to_string()))?;
                RegistryValueData::MultiString(multi_string)
            },
            RegistryDataType::RegQword => {
                let mapped_value = map.get(&value_str).ok_or_else(|| RegistryResourceError::AdaptedResource(format!("No mapping found for value {} in qword type", value_str)))?;
                let qword = mapped_value.as_u64().ok_or_else(|| RegistryResourceError::AdaptedResource(format!("Mapped value for {} is not a u64", value_str)))?;
                RegistryValueData::QWord(qword)
            },
            RegistryDataType::RegString => {
                let mapped_value = map.get(&value_str).ok_or_else(|| RegistryResourceError::AdaptedResource(format!("No mapping found for value {} in string type", value_str)))?;
                let string = mapped_value.as_str().ok_or_else(|| RegistryResourceError::AdaptedResource(format!("Mapped value for {} is not a string", value_str)))?.to_string();
                RegistryValueData::String(string)
            },
        }
    };
    Ok(registry_value_data)
}

pub fn adapter_set(input: &str, adapted_resource: &str) -> Result<String, RegistryResourceError> {
    trace!("Adapter Set with input: {input}");
    let adapted_resource: AdaptedRegistryResource = serde_json::from_str(adapted_resource)
        .map_err(|e| RegistryResourceError::AdaptedResource(e.to_string()))?;
    
    for (key, value) in adapted_resource.properties.iter() {
        trace!("Property: {key} = {value}");
    }
    Ok("{}".to_string())
}

pub fn adapter_export(input: &str, adapted_resource: &str) -> Result<String, RegistryResourceError> {
    trace!("Adapter Export with input: {input}");

    // if input is provided, use that to perform a `get` and return that result
    // if no input is provided, then create an input that contains all keys in the adapted resource with empty values and perform a `get` to return the default values for all keys
    let input: String = if input.is_empty() {
        let adapted_resource_map: AdaptedRegistryResource = serde_json::from_str(adapted_resource)
            .map_err(|e| RegistryResourceError::AdaptedResource(e.to_string()))?;
        let mut map = Map::new();
        for key in adapted_resource_map.properties.keys() {
            map.insert(key.clone(), Value::String(String::new()));
        }
        serde_json::to_string(&map).map_err(|e| RegistryResourceError::AdaptedResource(e.to_string()))?
    } else {
        input.to_string()
    };
    adapter_get(&input, adapted_resource)
}
