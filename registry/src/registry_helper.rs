// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use registry::{Data, Hive, RegKey, Security, key::Error};
use utfx::{U16CString, UCString};
use crate::config::{RegistryConfig, RegistryValueData};
use crate::error::RegistryError;

pub struct RegistryHelper {
    config: RegistryConfig,
}

impl RegistryHelper {
    pub fn new(config: &str) -> Result<Self, RegistryError> {
        Ok(
            Self {
            config: serde_json::from_str(config)?,
            }
        )
    }

    pub fn get(&self) -> Result<RegistryConfig, RegistryError> {
        let reg_key = self.open(Security::Read)?;
        if let Some(value_name) = &self.config.value_name {
            let value = reg_key.value(value_name)?;
            Ok(RegistryConfig {
                key_path: self.config.key_path.clone(),
                value_name: Some(value_name.clone()),
                value_data: Some(convert_reg_value(&value)?),
            })
        } else {
            Ok(RegistryConfig {
                key_path: self.config.key_path.clone(),
                value_name: None,
                value_data: None,
            })
        }
    }

    pub fn set(&self) -> Result<(), RegistryError> {
        let reg_key = match self.open(Security::Write) {
            Ok(reg_key) => reg_key,
            // handle NotFound error
            Err(RegistryError::RegistryKeyNotFound(_)) => {
                // create the key if it doesn't exist
                let (hive, subkey) = self.get_hive()?;

                // do this recursively per segment of the subkey
                let mut current_subkey = "".to_string();
                for segment in subkey.split('\\') {
                    current_subkey = format!("{}\\{}", current_subkey.clone(), segment);
                    hive.create(current_subkey, Security::Write)?;
                }

                hive.open(subkey, Security::Write)?
            },
            Err(e) => return Err(e),
        };

        if let Some(value_data) = &self.config.value_data {
            let Ok(value_name) = U16CString::from_str(self.config.value_name.as_ref().unwrap()) else {
                return Err(RegistryError::Utf16Conversion("valueName".to_string()));
            };

            match value_data {
                RegistryValueData::String(s) => {
                    let Ok(utf16) = U16CString::from_str(s) else {
                        return Err(RegistryError::Utf16Conversion("valueData".to_string()));
                    };
                    reg_key.set_value(&value_name, &Data::String(utf16))?;
                },
                RegistryValueData::ExpandString(s) => {
                    let Ok(utf16) = U16CString::from_str(s) else {
                        return Err(RegistryError::Utf16Conversion("valueData".to_string()));
                    };
                    reg_key.set_value(&value_name, &Data::ExpandString(utf16))?;  
                },
                RegistryValueData::Binary(b) => {
                    reg_key.set_value(&value_name, &Data::Binary(b.clone()))?;
                },
                RegistryValueData::DWord(d) => {
                    reg_key.set_value(&value_name, &Data::U32(*d))?;
                },
                RegistryValueData::MultiString(m) => {
                    let mut m16: Vec<UCString<u16>> = Vec::<UCString<u16>>::new();
                    for s in m {
                        let Ok(utf16) = U16CString::from_str(s) else {
                            return Err(RegistryError::Utf16Conversion("valueData".to_string()));
                        };
                        m16.push(utf16);
                    }
                    reg_key.set_value(&value_name, &Data::MultiString(m16))?
                },
                RegistryValueData::QWord(q) => {
                    reg_key.set_value(&value_name, &Data::U64(*q))?
                },
            }
        }

        Ok(())
    }

    pub fn remove(&self) -> Result<(), RegistryError> {
        let reg_key = self.open(Security::Write)?;
        if let Some(value_name) = &self.config.value_name {
            reg_key.delete_value(value_name)?;
        } else {
            reg_key.delete_self(true)?;
        }
        Ok(())
    }

    fn get_hive(&self) -> Result<(Hive, &str), RegistryError> {
        // split the key path to hive and subkey otherwise it's just a hive
        let (hive, subkey)= match self.config.key_path.find('\\') {
            Some(index) => {
                // split at index, but don't include the character at index
                let (hive, subkey) = self.config.key_path.split_at(index);
                (hive, &subkey[1..])
            },
            None => (self.config.key_path.as_str(), "".as_ref()),
        };

        match hive {
            "HKEY_LOCAL_MACHINE" | "HKLM" => Ok((Hive::LocalMachine, subkey)),
            "HKEY_CURRENT_USER" | "HKCU" => Ok((Hive::CurrentUser, subkey)),
            "HKEY_CLASSES_ROOT" | "HKCR" => Ok((Hive::ClassesRoot, subkey)),
            "HKEY_USERS" | "HKU" => Ok((Hive::Users, subkey)),
            "HKEY_CURRENT_CONFIG" | "HKCC" => Ok((Hive::CurrentConfig, subkey)),
            _ => Err(RegistryError::InvalidHive(hive.to_string()))
        }
    }

    fn open(&self, permission: Security) -> Result<RegKey, RegistryError> {
        let (hive, subkey) = self.get_hive()?;
        match hive.open(subkey, permission) {
            Ok(regkey) => Ok(regkey),
            // handle NotFound error
            Err(Error::NotFound(_, _)) => {
                return Err(RegistryError::RegistryKeyNotFound(self.config.key_path.clone()));
            },
            Err(e) => return Err(RegistryError::RegistryKey(e)),
        }
    }
}

fn convert_reg_value(value: &Data) -> Result<RegistryValueData, RegistryError> {
    match value {
        Data::String(s) => Ok(RegistryValueData::String(s.to_string_lossy())),
        Data::ExpandString(s) => Ok(RegistryValueData::ExpandString(s.to_string_lossy())),
        Data::Binary(b) => Ok(RegistryValueData::Binary(b.clone())),
        Data::U32(d) => Ok(RegistryValueData::DWord(*d)),
        Data::MultiString(m) => {
            let m: Vec<String> = m.iter().map(|s| s.to_string_lossy()).collect();
            Ok(RegistryValueData::MultiString(m))
        },
        Data::U64(q) => Ok(RegistryValueData::QWord(*q)),
        _ => Err(RegistryError::UnsupportedValueDataType)
    }
}

#[test]
fn get_hklm_key() {
    let reg_helper = RegistryHelper::new(r#"{"keyPath":"HKEY_LOCAL_MACHINE"}"#).unwrap();
    let reg_config = reg_helper.get().unwrap();
    assert_eq!(reg_config.key_path, r#"HKEY_LOCAL_MACHINE"#);
    assert_eq!(reg_config.value_name, None);
    assert_eq!(reg_config.value_data, None);
}

#[test]
fn get_product_name() {
    let reg_helper = RegistryHelper::new(r#"{"keyPath":"HKLM\\SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion","valueName":"ProductName"}"#).unwrap();
    let reg_config = reg_helper.get().unwrap();
    assert_eq!(reg_config.key_path, r#"HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion"#);
    assert_eq!(reg_config.value_name, Some("ProductName".to_string()));
    // value data starts with "Windows 10"
    assert!(matches!(reg_config.value_data, Some(RegistryValueData::String(s)) if s.starts_with("Windows 10")));
}

#[test]
fn set_and_remove_test_value() {
    let reg_helper = RegistryHelper::new(r#"{"keyPath":"HKCU\\DSCTest","valueName":"TestValue","valueData": { "String": "Hello"} }"#).unwrap();
    reg_helper.set().unwrap();
    let result = reg_helper.get().unwrap();
    assert_eq!(result.key_path, r#"HKCU\DSCTest"#);
    assert_eq!(result.value_name, Some("TestValue".to_string()));
    assert_eq!(result.value_data, Some(RegistryValueData::String("Hello".to_string())));
    reg_helper.remove().unwrap();
    assert!(matches!(reg_helper.get(), Err(RegistryError::RegistryValue(_))));
    let reg_helper = RegistryHelper::new(r#"{"keyPath":"HKCU\\DSCTest"}"#).unwrap();
    let result = reg_helper.get().unwrap();
    assert_eq!(result.key_path, r#"HKCU\DSCTest"#);
    assert_eq!(result.value_name, None);
    assert_eq!(result.value_data, None);
    reg_helper.remove().unwrap();
    assert!(matches!(reg_helper.get(), Err(RegistryError::Registry(_))));
}
