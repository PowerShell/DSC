use crate::config::{EnsureKind, RegistryConfig, RegistryValueData};
use ntreg::{registry_key::RegistryKey, registry_value::RegistryValueData as NtRegistryValueData, registry_value::RegistryValue};
use ntstatuserror::{NtStatusError, NtStatusErrorKind};
use std::fmt::{Display, Formatter};

#[derive(Debug, PartialEq, Eq)]
pub enum RegistryError {
    NtStatus(NtStatusError),
    Json(String),
    Input(String),
}

impl From<NtStatusError> for RegistryError {
    fn from(err: NtStatusError) -> Self {
        RegistryError::NtStatus(err)
    }
}

impl Display for RegistryError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            RegistryError::NtStatus(err) => write!(f, "{}", err),
            RegistryError::Json(err) => write!(f, "{}", err),
            RegistryError::Input(err) => write!(f, "{}", err),
        }
    }
}

pub fn config_get(config: &RegistryConfig) -> Result<String, RegistryError> {
    let mut reg_result = RegistryConfig::default();

    let reg_key = match RegistryKey::new(config.key_path.as_str()) {
        Ok(reg_key) => reg_key,
        Err(NtStatusError { status: NtStatusErrorKind::ObjectNameNotFound, ..}) => {
            match serde_json::to_string(&reg_result) {
                Ok(reg_json) => {
                    return Ok(reg_json);
                },
                Err(err) => {
                    return Err(RegistryError::Json(err.to_string()));
                }
            }
        }
        Err(err) => {
            return Err(RegistryError::NtStatus(err));
        }
    };

    reg_result.key_path = config.key_path.clone();

    if config.value_name.is_some() {
        let reg_value = match reg_key.get_value(config.value_name.as_ref().unwrap().as_str()) {
            Ok(reg_value) => reg_value,
            Err(err) => {
                return Err(RegistryError::NtStatus(err));
            }
        };

        reg_result.value_name = Some(reg_value.name);
        reg_result.value_data = Some(convert_ntreg_data(&reg_value.data)?);
    }

    match serde_json::to_string(&reg_result) {
        Ok(reg_json) => {
            Ok(reg_json)
        },
        Err(err) => {
            Err(RegistryError::Json(err.to_string()))
        }
    }
}

pub fn config_set(config: &RegistryConfig) -> Result<(String, bool), RegistryError> {
    let mut reg_result: RegistryConfig = RegistryConfig::default();
    let in_desired_state = true;

    let reg_key: RegistryKey;
    match &config.value_name {
        None => {
            match config.ensure.as_ref().unwrap() {
                EnsureKind::Present => {
                    open_or_create_key(&config.key_path)?;
                    reg_result.key_path = config.key_path.clone();
                },
                EnsureKind::Absent => {
                    remove_key(&config.key_path)?;
                },
            }
        },
        Some(value_name) => {
            reg_result.key_path = config.key_path.clone();
            reg_result.value_name = Some(value_name.clone());
            match &config.ensure {
                Some(EnsureKind::Present) | None => {
                    reg_key = open_or_create_key(&config.key_path)?;
                    match config.value_data.as_ref() {
                        Some(value_data) => {
                            reg_result.value_data = Some(value_data.clone());
                            reg_key.set_value(value_name, &convert_configreg_data(value_data))?;
                        },
                        None => {
                            // just verify that the value exists
                            match reg_key.get_value(value_name) {
                                Ok(_) => {},
                                Err(NtStatusError { status: NtStatusErrorKind::ObjectNameNotFound, .. }) => {
                                    reg_key.set_value(value_name, &NtRegistryValueData::None)?;
                                },
                                Err(err) => {
                                    return Err(RegistryError::NtStatus(err));
                                }
                            }
                        }
                    }
                },
                Some(EnsureKind::Absent) => {
                    reg_key = open_or_create_key(&config.key_path)?;
                    match reg_key.delete_value(value_name) {
                        Ok(_) | Err(NtStatusError { status: NtStatusErrorKind::ObjectNameNotFound, ..}) => {},
                        Err(err) => {
                            return Err(RegistryError::NtStatus(err));
                        }
                    }
                },
            }
        }
    }

    let reg_json = match serde_json::to_string(&reg_result) {
        Ok(reg_json) => reg_json,
        Err(err) => {
            return Err(RegistryError::Json(err.to_string()));
        }
    };

    Ok((reg_json, in_desired_state))
}

fn get_parent_key_path(key_path: &str) -> Result<&str, RegistryError> {
    match key_path.rfind('\\') {
        Some(index) => Ok(&key_path[..index]),
        None => {
            Err(RegistryError::Input(format!("Invalid key path: {}", key_path)))
        }
    }
}

fn remove_key(key_path: &str) -> Result<(), RegistryError> {
    match RegistryKey::new(key_path) {
        Ok(key) => {
            key.delete(true)?;
            Ok(())
        },
        Err(NtStatusError { status: NtStatusErrorKind::ObjectNameNotFound, ..}) => {
            Ok(()) // key doesn't exist, so we're good
        }
        Err(err) => {
            Err(RegistryError::NtStatus(err))
        }
    }
}

fn open_or_create_key(key_path: &str) -> Result<RegistryKey, RegistryError> {
    let reg_key: RegistryKey;
    match RegistryKey::new(key_path) {
        Ok(key) => {
            reg_key = key;
        },
        Err(NtStatusError { status: NtStatusErrorKind::ObjectNameNotFound, ..}) => {
            // need to handle case like `HKLM\1\2\3` where neither `1` nor `2` exist
            // so we need to find the top most parent that currently exists and then create the necessary subkeys in order
            let (parent_key, subkeys) = get_valid_parent_key_and_subkeys(key_path)?;
            let mut current_key = parent_key;
            for subkey in subkeys {
                current_key = current_key.create_key(subkey)?;
            }
            reg_key = current_key;
        },
        Err(err) => {
            return Err(RegistryError::NtStatus(err));
        }
    }

    Ok(reg_key)
}

fn get_valid_parent_key_and_subkeys(key_path: &str) -> Result<(RegistryKey, Vec<&str>), RegistryError> {
    let parent_key: RegistryKey;
    let mut subkeys: Vec<&str> = Vec::new();
    let parent_key_path = get_parent_key_path(key_path)?;
    let subkey_name = &key_path[parent_key_path.len() + 1..];
    subkeys.push(subkey_name);
    let mut current_key_path = parent_key_path;

    loop {
        match RegistryKey::new(current_key_path) {
            Ok(key) => {
                parent_key = key;
                break;
            },
            Err(NtStatusError { status: NtStatusErrorKind::ObjectNameNotFound, ..}) => {
                let parent_key_path = get_parent_key_path(current_key_path)?;
                let subkey_name = &current_key_path[parent_key_path.len() + 1..];
                subkeys.insert(0, subkey_name);
                current_key_path = parent_key_path;
            },
            Err(err) => {
                return Err(RegistryError::NtStatus(err));
            }
        }
    }

    Ok((parent_key, subkeys))
}

pub fn validate_config(config: &RegistryConfig) -> Result<(), RegistryError>{
    if config.value_data.is_some() && config.value_name.is_none() {
        return Err(RegistryError::Input("value_name is required when value_data is specified.".to_string()));
    }

    Ok(())
}

pub fn config_test(config: &RegistryConfig) -> Result<String, RegistryError> {
    if config.value_name.is_none() {
        Ok(test_key(config)?)
    }
    else {
        Ok(test_value(config)?)
    }
}

fn test_value(config: &RegistryConfig) -> Result<String, RegistryError> {
    let mut reg_result: RegistryConfig = Default::default();
    let mut in_desired_state = true;

    let reg_key = match RegistryKey::new(config.key_path.as_str()) {
        Ok(key) => {
            key
        },
        Err(NtStatusError { status: NtStatusErrorKind::ObjectNameNotFound, ..}) => {
            reg_result.key_path = String::new();
            reg_result.in_desired_state = Some(false);
            return Ok(reg_result.to_json());
        },
        Err(err) => {
            return Err(RegistryError::NtStatus(err));
        }
    };

    reg_result.key_path = config.key_path.clone();

    let value_name = config.value_name.as_ref().unwrap();
    let mut value_exists = false;
    let reg_value: RegistryValue = match reg_key.get_value(value_name) {
        Ok(value) => {
            value_exists = true;
            reg_result.value_name = Some(value.name.clone());
            reg_result.value_data = Some(convert_ntreg_data(&value.data)?);
            value
        },
        Err(NtStatusError { status: NtStatusErrorKind::ObjectNameNotFound, ..}) => {
            RegistryValue {
                key_path: config.key_path.clone(),
                name : String::new(),
                data : NtRegistryValueData::None,
            }
        },
        Err(err) => {
            return Err(RegistryError::NtStatus(err));
        }
    };

    match &config.ensure.as_ref().unwrap() {
        EnsureKind::Present => {
            if value_exists {
                in_desired_state = reg_values_are_eq(config, &reg_value)?;
            }
            else {
                in_desired_state = false;
            }
        },
        EnsureKind::Absent => {
            if value_exists {
                in_desired_state = false;
            }
        }
    }

    reg_result.in_desired_state = Some(in_desired_state);
    Ok(reg_result.to_json())
}

fn reg_values_are_eq(config: &RegistryConfig, reg_value: &RegistryValue) -> Result<bool, RegistryError> {
    let mut in_desired_state = true;

    if !reg_value.name.eq(config.value_name.as_ref().unwrap().as_str()) {
        in_desired_state = false;
    }

    if config.value_data.is_some() && reg_value.data == NtRegistryValueData::None {
        in_desired_state = false;
    }
    else if config.value_data.is_none() {
        in_desired_state = true;
    }
    else {
        let reg_value_data = convert_ntreg_data(&reg_value.data)?;
        if reg_value_data != config.value_data.to_owned().unwrap() {
            in_desired_state = false;
        }
    }

    Ok(in_desired_state)
}

fn test_key(config: &RegistryConfig) -> Result<String, RegistryError> {
    let mut reg_result: RegistryConfig = Default::default();

    let key_exists = match RegistryKey::new(config.key_path.as_str()) {
        Ok( _ ) => {
            true
        },
        Err(NtStatusError { status: NtStatusErrorKind::ObjectNameNotFound, ..}) => {
            false
        },
        Err(err) => {
            return Err(RegistryError::NtStatus(err));
        }
    };

    let mut in_desired_state = true;
    match &config.ensure.as_ref().unwrap() {
        EnsureKind::Present => {
            if !key_exists {
                reg_result.key_path = String::new();
                in_desired_state = false;
            }
        },
        EnsureKind::Absent => {
            if key_exists {
                reg_result.key_path = config.key_path.clone();
                in_desired_state = false;
            }
        }
    }
        
    reg_result.in_desired_state = Some(in_desired_state);
    Ok(reg_result.to_json())
}

fn convert_ntreg_data(reg_data: &NtRegistryValueData) -> Result<RegistryValueData, RegistryError> {
    match reg_data {
        NtRegistryValueData::String(data) => Ok(RegistryValueData::String(data.clone())),
        NtRegistryValueData::MultiString(data) => Ok(RegistryValueData::MultiString(data.clone())),
        NtRegistryValueData::Binary(data) => Ok(RegistryValueData::Binary(data.clone())),
        NtRegistryValueData::DWord(data) => Ok(RegistryValueData::DWord(*data)),
        NtRegistryValueData::QWord(data) => Ok(RegistryValueData::QWord(*data)),
        NtRegistryValueData::ExpandString(data) => Ok(RegistryValueData::ExpandString(data.clone())),
        _ => {
            Err(RegistryError::Input("Unsupported registry value type".to_string()))
        }
    }
}

fn convert_configreg_data(reg_data: &RegistryValueData) -> NtRegistryValueData {
    match reg_data {
        RegistryValueData::String(data) => NtRegistryValueData::String(data.clone()),
        RegistryValueData::MultiString(data) => NtRegistryValueData::MultiString(data.clone()),
        RegistryValueData::Binary(data) => NtRegistryValueData::Binary(data.clone()),
        RegistryValueData::DWord(data) => NtRegistryValueData::DWord(*data),
        RegistryValueData::QWord(data) => NtRegistryValueData::QWord(*data),
        RegistryValueData::ExpandString(data) => NtRegistryValueData::ExpandString(data.clone()),
    }
}

#[test]
fn test_registry_value_present() {
    let input_json: &str = r#"
    {
        "keyPath": "HKLM\\Software\\Microsoft\\Windows\\CurrentVersion",
        "valueName": "ProgramFilesPath",
        "_ensure": "Present"
    }
    "#;

    let config: RegistryConfig = serde_json::from_str(input_json).unwrap();
    let json = config_test(&config).unwrap();
    assert_eq!(json, r#"{"$id":"https://developer.microsoft.com/json-schemas/windows/registry/20230303/Microsoft.Windows.Registry.schema.json","keyPath":"HKLM\\Software\\Microsoft\\Windows\\CurrentVersion","valueName":"ProgramFilesPath","valueData":{"ExpandString":"%ProgramFiles%"},"_inDesiredState":true}"#);
}

#[test]
fn test_registry_value_absent() {
    let input_json: &str = r#"
    {
        "keyPath": "HKLM\\Software\\Microsoft\\Windows\\CurrentVersion",
        "valueName": "DoesNotExist",
        "_ensure": "Absent"
    }
    "#;

    let config: RegistryConfig = serde_json::from_str(input_json).unwrap();
    let json = config_test(&config).unwrap();
    assert_eq!(json, r#"{"$id":"https://developer.microsoft.com/json-schemas/windows/registry/20230303/Microsoft.Windows.Registry.schema.json","keyPath":"HKLM\\Software\\Microsoft\\Windows\\CurrentVersion","_inDesiredState":true}"#);
}
