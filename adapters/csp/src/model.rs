// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use rust_i18n::t;
use serde_json::{Map, Value, json};
use std::fmt;

pub const ADAPTER_TYPE: &str = "Microsoft.Windows.Adapter/ConfigurationServiceProvider";
pub const RESOURCE_PREFIX: &str = "Microsoft.Windows.CSP/";

#[derive(Debug)]
pub enum AdapterError {
    Input(String),
    Resource(String),
}

impl fmt::Display for AdapterError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Input(message) | Self::Resource(message) => formatter.write_str(message),
        }
    }
}

impl std::error::Error for AdapterError {}

impl From<String> for AdapterError {
    fn from(message: String) -> Self {
        Self::Resource(message)
    }
}

#[derive(Clone, Debug)]
pub struct Property {
    pub name: String,
    pub cim_type: i32,
    pub key: bool,
    pub readable: bool,
    pub writable: bool,
}

impl Property {
    pub fn schema(&self) -> Result<Value, AdapterError> {
        let scalar = match self.cim_type & 0xff {
            2 => json!({"type": "integer", "minimum": i16::MIN, "maximum": i16::MAX}),
            3 => json!({"type": "integer", "minimum": i32::MIN, "maximum": i32::MAX}),
            4 => json!({"type": "number", "minimum": -f32::MAX, "maximum": f32::MAX}),
            5 => json!({"type": "number"}),
            8 | 101 | 102 => json!({"type": "string"}),
            11 => json!({"type": "boolean"}),
            16 => json!({"type": "integer", "minimum": i8::MIN, "maximum": i8::MAX}),
            17 => json!({"type": "integer", "minimum": u8::MIN, "maximum": u8::MAX}),
            18 => json!({"type": "integer", "minimum": u16::MIN, "maximum": u16::MAX}),
            19 => json!({"type": "integer", "minimum": u32::MIN, "maximum": u32::MAX}),
            20 => json!({"type": "integer", "minimum": i64::MIN, "maximum": i64::MAX}),
            21 => json!({"type": "integer", "minimum": u64::MIN, "maximum": u64::MAX}),
            103 => json!({"type": "string", "minLength": 1, "maxLength": 1}),
            _ => {
                return Err(AdapterError::Resource(
                    t!(
                        "adapter.unsupportedType",
                        name = self.name,
                        cimType = self.cim_type
                    )
                    .to_string(),
                ));
            }
        };
        let mut schema = if self.cim_type & 0x2000 != 0 {
            json!({"anyOf": [{"type": "array", "items": scalar}, {"type": "null"}]})
        } else if self.key {
            scalar
        } else {
            json!({"anyOf": [scalar, {"type": "null"}]})
        };
        if !self.writable && !self.key {
            schema["readOnly"] = json!(true);
        }
        if !self.readable {
            schema["writeOnly"] = json!(true);
        }
        Ok(schema)
    }

    pub fn accepts(&self, value: &Value) -> bool {
        if value.is_null() {
            return !self.key;
        }
        if self.cim_type & 0x2000 != 0 {
            let scalar = Self {
                cim_type: self.cim_type & 0xff,
                key: true,
                ..self.clone()
            };
            return value
                .as_array()
                .is_some_and(|items| items.iter().all(|v| scalar.accepts(v)));
        }
        match self.cim_type & 0xff {
            2 => value.as_i64().is_some_and(|v| i16::try_from(v).is_ok()),
            3 => value.as_i64().is_some_and(|v| i32::try_from(v).is_ok()),
            4 => value
                .as_f64()
                .is_some_and(|v| v.abs() <= f64::from(f32::MAX)),
            5 => value.as_f64().is_some_and(f64::is_finite),
            8 | 101 | 102 => value.is_string(),
            11 => value.is_boolean(),
            16 => value.as_i64().is_some_and(|v| i8::try_from(v).is_ok()),
            17 => value.as_u64().is_some_and(|v| u8::try_from(v).is_ok()),
            18 => value.as_u64().is_some_and(|v| u16::try_from(v).is_ok()),
            19 => value.as_u64().is_some_and(|v| u32::try_from(v).is_ok()),
            20 => value.as_i64().is_some(),
            21 => value.as_u64().is_some(),
            103 => value
                .as_str()
                .is_some_and(|v| v.encode_utf16().count() == 1),
            _ => false,
        }
    }
}

#[derive(Debug)]
pub struct Class {
    pub name: String,
    pub partitions: Vec<String>,
    pub properties: Vec<Property>,
}

impl Class {
    pub fn writable(&self) -> bool {
        self.properties.iter().any(|p| !p.key && p.writable)
    }

    pub fn schema(&self) -> Result<Value, AdapterError> {
        let mut properties = Map::new();
        for property in &self.properties {
            properties.insert(property.name.clone(), property.schema()?);
        }
        properties.insert("_exist".to_string(), json!({"type": "boolean"}));
        properties.insert(
            "_inDesiredState".to_string(),
            json!({"type": "boolean", "readOnly": true}),
        );
        Ok(json!({
            "$schema": "https://json-schema.org/draft/2020-12/schema",
            "title": format!("{RESOURCE_PREFIX}{}", self.name),
            "type": "object",
            "additionalProperties": false,
            "properties": properties
        }))
    }

    pub fn parse_input(
        &self,
        input: &str,
        export: bool,
    ) -> Result<Map<String, Value>, AdapterError> {
        let input: Map<String, Value> = serde_json::from_str(input).map_err(|error| {
            AdapterError::Input(t!("adapter.invalidInput", error = error).to_string())
        })?;
        for (name, value) in &input {
            if name == "_exist" && value.is_boolean() && !export {
                continue;
            }
            let property = self
                .properties
                .iter()
                .find(|p| p.name == *name && (!export || p.key))
                .ok_or_else(|| {
                    AdapterError::Input(t!("adapter.unknownProperty", name = name).to_string())
                })?;
            if !property.accepts(value) {
                return Err(AdapterError::Input(
                    t!(
                        "adapter.invalidValue",
                        name = name,
                        cimType = property.cim_type
                    )
                    .to_string(),
                ));
            }
        }
        Ok(input)
    }

    pub fn require_keys(&self, input: &Map<String, Value>) -> Result<(), AdapterError> {
        for property in self.properties.iter().filter(|p| p.key) {
            if input.get(&property.name).is_none_or(Value::is_null) {
                return Err(AdapterError::Input(
                    t!("adapter.missingKey", name = property.name).to_string(),
                ));
            }
        }
        Ok(())
    }

    pub fn validate_set(&self, input: &Map<String, Value>) -> Result<(), AdapterError> {
        if !self.writable() {
            return Err(AdapterError::Input(
                t!("adapter.readOnlyClass", name = self.name).to_string(),
            ));
        }
        for name in input.keys() {
            if let Some(property) = self.properties.iter().find(|p| p.name == *name)
                && !property.key
                && !property.writable
            {
                return Err(AdapterError::Input(
                    t!("adapter.readOnly", name = name).to_string(),
                ));
            }
            if input[name].as_array().is_some_and(Vec::is_empty) {
                return Err(AdapterError::Input(
                    t!("adapter.emptyArray", name = name).to_string(),
                ));
            }
        }
        Ok(())
    }

    pub fn absent(&self, input: &Map<String, Value>) -> Map<String, Value> {
        let mut state: Map<String, Value> = self
            .properties
            .iter()
            .filter(|p| p.key)
            .filter_map(|p| input.get(&p.name).map(|v| (p.name.clone(), v.clone())))
            .collect();
        state.insert("_exist".to_string(), json!(false));
        state
    }

    pub fn query(&self, input: &Map<String, Value>) -> String {
        let filters: Vec<String> = self
            .properties
            .iter()
            .filter(|p| p.key)
            .filter_map(|p| {
                input.get(&p.name).map(|value| {
                    let literal = value.as_str().map_or_else(
                        || value.to_string(),
                        |s| format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\"")),
                    );
                    format!("{} = {literal}", p.name)
                })
            })
            .collect();
        if filters.is_empty() {
            format!("SELECT * FROM {}", self.name)
        } else {
            format!(
                "SELECT * FROM {} WHERE {}",
                self.name,
                filters.join(" AND ")
            )
        }
    }
}

pub fn class_name(resource_type: &str) -> Result<&str, AdapterError> {
    let name = resource_type.strip_prefix(RESOURCE_PREFIX).filter(|name| {
        name.starts_with("MDM_")
            && name.len() > 4
            && name.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'_')
    });
    name.ok_or_else(|| {
        AdapterError::Input(t!("adapter.invalidType", name = resource_type).to_string())
    })
}

pub fn diff(desired: &Map<String, Value>, actual: &Map<String, Value>) -> Vec<String> {
    if desired.get("_exist") == Some(&json!(false)) {
        return if actual.get("_exist") == Some(&json!(false)) {
            Vec::new()
        } else {
            vec!["_exist".to_string()]
        };
    }
    let mut diff: Vec<String> = desired
        .iter()
        .filter(|(name, value)| actual.get(*name) != Some(*value))
        .map(|(name, _)| name.clone())
        .collect();
    if actual.get("_exist") == Some(&json!(false)) && !diff.iter().any(|n| n == "_exist") {
        diff.push("_exist".to_string());
    }
    diff
}

#[cfg(test)]
mod tests {
    use super::*;

    fn class() -> Class {
        Class {
            name: "MDM_Test".to_string(),
            partitions: vec!["local-system".to_string()],
            properties: vec![
                Property {
                    name: "ParentID".to_string(),
                    cim_type: 8,
                    key: true,
                    readable: true,
                    writable: false,
                },
                Property {
                    name: "InstanceID".to_string(),
                    cim_type: 8,
                    key: true,
                    readable: true,
                    writable: false,
                },
                Property {
                    name: "Setting".to_string(),
                    cim_type: 3,
                    key: false,
                    readable: true,
                    writable: true,
                },
                Property {
                    name: "Result".to_string(),
                    cim_type: 11,
                    key: false,
                    readable: true,
                    writable: false,
                },
            ],
        }
    }

    #[test]
    fn validates_resource_type_before_using_it_in_wql() {
        assert_eq!(
            class_name("Microsoft.Windows.CSP/MDM_Test").unwrap(),
            "MDM_Test"
        );
        for name in [
            "MDM_Test",
            "Microsoft.Windows.CSP/Win32_Process",
            "Microsoft.Windows.CSP/MDM_",
            "Microsoft.Windows.CSP/MDM_Test WHERE 1=1",
        ] {
            assert!(class_name(name).is_err());
        }
    }

    #[test]
    fn validates_keys_unknown_properties_and_ranges() {
        let class = class();
        let input = class
            .parse_input(
                r#"{"ParentID":"./Vendor/MSFT","InstanceID":"Test","Setting":2147483647}"#,
                false,
            )
            .unwrap();
        assert!(class.require_keys(&input).is_ok());
        for input in [
            r#"{"Setting":2147483648}"#,
            r#"{"Setting":-2147483649}"#,
            r#"{"Setting":1.5}"#,
            r#"{"Setting":"1"}"#,
            r#"{"unknown":0}"#,
            r#"{"ParentID":null}"#,
            "[]",
        ] {
            assert!(class.parse_input(input, false).is_err(), "{input}");
        }
        assert!(class.require_keys(&Map::new()).is_err());
        assert!(class.parse_input(r#"{"Setting":null}"#, false).is_ok());
        assert!(class.parse_input(r#"{"Setting":1}"#, true).is_err());
        assert!(class.parse_input("{}", true).is_ok());
    }

    #[test]
    fn set_rejects_read_only_properties_before_mutation() {
        let class = class();
        assert!(
            class
                .validate_set(json!({"Result":true}).as_object().unwrap())
                .is_err()
        );
        assert!(
            class
                .validate_set(json!({"Setting":1}).as_object().unwrap())
                .is_ok()
        );
        let mut read_only = class;
        read_only
            .properties
            .iter_mut()
            .for_each(|p| p.writable = false);
        assert!(!read_only.writable());
        assert!(read_only.validate_set(&Map::new()).is_err());
    }

    #[test]
    fn schema_preserves_wmi_names_and_read_only_metadata() {
        let schema = class().schema().unwrap();
        assert_eq!(schema["additionalProperties"], false);
        assert_eq!(schema["properties"]["Result"]["readOnly"], true);
        assert_eq!(
            schema["properties"]["Setting"]["anyOf"][0]["maximum"],
            i32::MAX
        );
        assert!(schema["properties"].get("ParentID").is_some());
    }

    #[test]
    fn queries_escape_keys_and_export_can_filter_on_partial_keys() {
        let class = class();
        let input = json!({"InstanceID": "a\"b\\c'd"});
        assert_eq!(
            class.query(input.as_object().unwrap()),
            r#"SELECT * FROM MDM_Test WHERE InstanceID = "a\"b\\c'd""#
        );
        assert_eq!(class.query(&Map::new()), "SELECT * FROM MDM_Test");
    }

    #[test]
    fn test_compares_only_requested_properties_and_handles_absence() {
        let desired = json!({"ParentID":"p","InstanceID":"i","Setting":1});
        let actual =
            json!({"ParentID":"p","InstanceID":"i","Setting":1,"Result":false,"_exist":true});
        assert!(diff(desired.as_object().unwrap(), actual.as_object().unwrap()).is_empty());
        let absent = class().absent(desired.as_object().unwrap());
        assert_eq!(
            diff(desired.as_object().unwrap(), &absent),
            ["Setting", "_exist"]
        );
        let deleted = json!({"_exist":false,"Setting":1});
        assert!(diff(deleted.as_object().unwrap(), &absent).is_empty());
        assert_eq!(
            diff(deleted.as_object().unwrap(), actual.as_object().unwrap()),
            ["_exist"]
        );
    }

    #[test]
    fn validates_arrays_and_integer_widths_without_truncation() {
        for (cim_type, accepted, rejected) in [
            (17, json!(255), json!(256)),
            (18, json!(65535), json!(-1)),
            (19, json!(u32::MAX), json!(u64::from(u32::MAX) + 1)),
            (20, json!(i64::MIN), json!(u64::MAX)),
            (21, json!(u64::MAX), json!(-1)),
            (3 | 0x2000, json!([1, 2]), json!([1, "2"])),
            (103, json!("a"), json!("ab")),
        ] {
            let property = Property {
                cim_type,
                ..class().properties[2].clone()
            };
            assert!(property.accepts(&accepted));
            assert!(!property.accepts(&rejected));
            assert!(property.schema().is_ok());
        }
    }
}
