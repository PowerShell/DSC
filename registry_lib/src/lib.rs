// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::config::{Metadata, Registry, RegistryValueData};
use crate::error::RegistryError;
use registry::{key, value, Data, Hive, RegKey, Security};
use rust_i18n::t;
use utfx::{U16CString, UCString};

rust_i18n::i18n!("locales", fallback = "en-us");

pub mod config;
pub mod error;

pub struct RegistryHelper {
    config: Registry,
    hive: Hive,
    subkey: String,
    what_if: bool,
}

impl RegistryHelper {
    /// Create a new `RegistryHelper` from json.
    ///
    /// # Arguments
    ///
    /// * `config` - The string with registry configuration information.
    ///
    /// # Errors
    ///
    /// * `RegistryError` - The error that occurred.
    pub fn new_from_json(config: &str) -> Result<Self, RegistryError> {
        let registry: Registry = match serde_json::from_str(config) {
            Ok(config) => config,
            Err(e) => return Err(RegistryError::Json(e)),
        };
        let key_path = registry.key_path.clone();
        let (hive, subkey) = get_hive_from_path(&key_path)?;

        Ok(Self {
            config: registry,
            hive,
            subkey: subkey.to_string(),
            what_if: false,
        })
    }

    /// Create a new `RegistryHelper`.
    ///
    /// # Arguments
    ///
    /// * `config` - The string with registry configuration information.
    ///
    /// # Errors
    ///
    /// * `RegistryError` - The error that occurred.
    pub fn new(
        key_path: &str,
        value_name: Option<String>,
        value_data: Option<RegistryValueData>,
    ) -> Result<Self, RegistryError> {
        let (hive, subkey) = get_hive_from_path(key_path)?;
        let config = Registry {
            key_path: key_path.to_string(),
            value_name,
            value_data,
            metadata: None,
            exist: None,
        };
        Ok(Self {
            config,
            hive,
            subkey: subkey.to_string(),
            what_if: false,
        })
    }

    pub fn enable_what_if(&mut self) {
        self.what_if = true;
    }

    /// Get from registry.
    ///
    /// # Returns
    ///
    /// * `Registry` - The registry struct.
    ///
    /// # Errors
    ///
    /// * `RegistryError` - The error that occurred.
    pub fn get(&self) -> Result<Registry, RegistryError> {
        let exist: bool;
        let (reg_key, _subkey) = match self.open(Security::Read) {
            Ok((reg_key, subkey)) => (reg_key, subkey),
            Err(RegistryError::RegistryKeyNotFound(_)) => {
                exist = false;
                return Ok(Registry {
                    key_path: self.config.key_path.clone(),
                    exist: Some(exist),
                    ..Default::default()
                });
            }
            Err(e) => return Err(e),
        };

        if let Some(value_name) = &self.config.value_name {
            let value = match reg_key.value(value_name) {
                Ok(value) => value,
                Err(value::Error::NotFound(_, _)) => {
                    exist = false;
                    return Ok(Registry {
                        key_path: self.config.key_path.clone(),
                        value_name: Some(value_name.clone()),
                        exist: Some(exist),
                        ..Default::default()
                    });
                }
                Err(e) => return Err(RegistryError::RegistryValue(e)),
            };

            Ok(Registry {
                key_path: self.config.key_path.clone(),
                value_name: Some(value_name.clone()),
                value_data: convert_reg_value(&value)?,
                ..Default::default()
            })
        } else {
            Ok(Registry {
                key_path: self.config.key_path.clone(),
                ..Default::default()
            })
        }
    }

    /// Set in registry.
    ///
    /// # Returns
    ///
    /// * `Registry` - The registry struct.
    ///
    /// # Errors
    ///
    /// * `RegistryError` - The error that occurred.
    pub fn set(&self) -> Result<Option<Registry>, RegistryError> {
        let mut what_if_metadata: Vec<String> = Vec::new();
        let reg_key = match self.open(Security::Write) {
            Ok((reg_key, _subkey)) => Some(reg_key),
            // handle NotFound error
            Err(RegistryError::RegistryKeyNotFound(_)) => {
                // if the key doesn't exist, some of the parent keys may
                // not exist either, so we need to find the valid parent key
                // and then create the subkeys that don't exist
                let (parent_key, subkeys) = self.get_valid_parent_key_and_subkeys()?;
                let mut reg_key = parent_key;
                for subkey in subkeys {
                    let Ok(path) = UCString::<u16>::from_str(subkey) else {
                        return self.handle_error_or_what_if(RegistryError::Utf16Conversion("subkey".to_string()));
                    };

                    if self.what_if {
                        what_if_metadata.push(t!("registry_helper.whatIfCreateKey", subkey = subkey).to_string());
                    } else {
                        reg_key = reg_key.create(path, Security::CreateSubKey)?;
                    }
                }
                if self.what_if {
                    None
                } else {
                    Some(self.open(Security::Write)?.0)
                }
            }
            Err(e) => return self.handle_error_or_what_if(e),
        };

        if let Some(value_name) = &self.config.value_name {
            let value_data = match &self.config.value_data {
                Some(value_data) => value_data,
                None => &RegistryValueData::None,
            };

            let Ok(value_name) = U16CString::from_str(value_name) else {
                return self.handle_error_or_what_if(RegistryError::Utf16Conversion("valueName".to_string()));
            };

            let data = match value_data {
                RegistryValueData::String(s) => {
                    let Ok(utf16) = U16CString::from_str(s) else {
                        return self.handle_error_or_what_if(RegistryError::Utf16Conversion("valueData".to_string()));
                    };
                    Data::String(utf16)
                }
                RegistryValueData::ExpandString(s) => {
                    let Ok(utf16) = U16CString::from_str(s) else {
                        return self.handle_error_or_what_if(RegistryError::Utf16Conversion("valueData".to_string()));
                    };
                    Data::ExpandString(utf16)
                }
                RegistryValueData::Binary(b) => Data::Binary(b.clone()),
                RegistryValueData::DWord(d) => Data::U32(*d),
                RegistryValueData::MultiString(m) => {
                    let mut m16: Vec<UCString<u16>> = Vec::<UCString<u16>>::new();
                    for s in m {
                        let Ok(utf16) = U16CString::from_str(s) else {
                            return self
                                .handle_error_or_what_if(RegistryError::Utf16Conversion("valueData".to_string()));
                        };
                        m16.push(utf16);
                    }
                    Data::MultiString(m16)
                }
                RegistryValueData::QWord(q) => Data::U64(*q),
                RegistryValueData::None => Data::None,
            };

            if self.what_if {
                return Ok(Some(Registry {
                    key_path: self.config.key_path.clone(),
                    value_data: convert_reg_value(&data)?,
                    value_name: self.config.value_name.clone(),
                    metadata: if what_if_metadata.is_empty() {
                        None
                    } else {
                        Some(Metadata {
                            what_if: Some(what_if_metadata),
                        })
                    },
                    ..Default::default()
                }));
            }

            if let Some(reg_key) = reg_key {
                reg_key.set_value(&value_name, &data)?;
            }
        }

        if self.what_if {
            return Ok(Some(Registry {
                key_path: self.config.key_path.clone(),
                metadata: if what_if_metadata.is_empty() {
                    None
                } else {
                    Some(Metadata {
                        what_if: Some(what_if_metadata),
                    })
                },
                ..Default::default()
            }));
        }

        Ok(None)
    }

    /// Delete from registry.
    ///
    /// # Returns
    ///
    /// Nothing on success.
    ///
    /// # Errors
    ///
    /// * `RegistryError` - The error that occurred.
    pub fn remove(&self) -> Result<Option<Registry>, RegistryError> {
        let (reg_key, _subkey) = match self.open(Security::AllAccess) {
            Ok(reg_key) => reg_key,
            // handle NotFound error
            Err(RegistryError::RegistryKeyNotFound(_)) => {
                eprintln!("{}", t!("registry_helper.removeErrorKeyNotExist"));
                return Ok(None);
            }
            Err(e) => return self.handle_error_or_what_if(e),
        };

        // Accumulate what-if metadata like set()
        let mut what_if_metadata: Vec<String> = Vec::new();

        if let Some(value_name) = &self.config.value_name {
            if self.what_if {
                what_if_metadata.push(t!("registry_helper.whatIfDeleteValue", value_name = value_name).to_string());
                return Ok(Some(Registry {
                    key_path: self.config.key_path.clone(),
                    value_name: Some(value_name.clone()),
                    metadata: Some(Metadata {
                        what_if: Some(what_if_metadata),
                    }),
                    ..Default::default()
                }));
            }
            match reg_key.delete_value(value_name) {
                Ok(()) | Err(value::Error::NotFound(_, _)) => {
                    // if the value doesn't exist, we don't need to do anything
                }
                Err(e) => return self.handle_error_or_what_if(RegistryError::RegistryValue(e)),
            }
        } else {
            // to delete the key, we need to open the parent key first
            let parent_path = get_parent_key_path(&self.config.key_path);
            let (hive, parent_subkey) = get_hive_from_path(parent_path)?;
            let parent_reg_key = match hive.open(parent_subkey, Security::AllAccess) {
                Ok(k) => k,
                Err(e) => return self.handle_error_or_what_if(RegistryError::RegistryKey(e)),
            };

            // get the subkey name
            let subkey_name = &self.config.key_path[parent_path.len() + 1..];

            if self.what_if {
                what_if_metadata.push(t!("registry_helper.whatIfDeleteSubkey", subkey_name = subkey_name).to_string());
                return Ok(Some(Registry {
                    key_path: self.config.key_path.clone(),
                    metadata: Some(Metadata {
                        what_if: Some(what_if_metadata),
                    }),
                    ..Default::default()
                }));
            }
            eprintln!(
                "{}",
                t!(
                    "registry_helper.removeDeletingSubKey",
                    name = subkey_name,
                    parent = parent_reg_key
                )
            );
            let Ok(subkey_name) = UCString::<u16>::from_str(subkey_name) else {
                return self.handle_error_or_what_if(RegistryError::Utf16Conversion("subkey_name".to_string()));
            };

            match parent_reg_key.delete(subkey_name, true) {
                Ok(()) | Err(key::Error::NotFound(_, _)) => {
                    // if the subkey doesn't exist, we don't need to do anything
                }
                Err(e) => return self.handle_error_or_what_if(RegistryError::RegistryKey(e)),
            }
        }
        Ok(None)
    }

    fn open(&self, permission: Security) -> Result<(RegKey, &str), RegistryError> {
        open_regkey(&self.config.key_path, permission)
    }

    // Find the valid parent key that exists and the subkeys that don't exist
    // the subkeys are returned in reverse order (the closest subkey is the last one in the vector)
    fn get_valid_parent_key_and_subkeys(&self) -> Result<(RegKey, Vec<&str>), RegistryError> {
        let parent_key: RegKey;
        let mut subkeys: Vec<&str> = Vec::new();
        let parent_key_path = get_parent_key_path(&self.subkey);
        let subkey_name = if parent_key_path.is_empty() {
            &self.subkey
        } else {
            &self.subkey[parent_key_path.len() + 1..]
        };
        if !subkey_name.is_empty() {
            subkeys.push(subkey_name);
        }
        let mut current_key_path = parent_key_path;

        loop {
            // we try to open with CreateSubKey permission to know if we can create the key
            match self.hive.open(current_key_path, Security::CreateSubKey) {
                Ok(regkey) => {
                    parent_key = regkey;
                    break;
                }
                Err(key::Error::NotFound(_, _)) => {
                    let parent_key_path = get_parent_key_path(current_key_path);
                    if parent_key_path.is_empty() {
                        subkeys.insert(0, current_key_path);
                        current_key_path = "";
                    } else {
                        let subkey_name = &current_key_path[parent_key_path.len() + 1..];
                        subkeys.insert(0, subkey_name);
                        current_key_path = parent_key_path;
                    }
                }
                Err(e) => {
                    return Err(RegistryError::RegistryKey(e));
                }
            }
        }

        Ok((parent_key, subkeys))
    }

    fn handle_error_or_what_if(&self, error: RegistryError) -> Result<Option<Registry>, RegistryError> {
        if self.what_if {
            return Ok(Some(Registry {
                key_path: self.config.key_path.clone(),
                metadata: Some(Metadata {
                    what_if: Some(vec![error.to_string()]),
                }),
                ..Default::default()
            }));
        }
        Err(error)
    }
}

fn get_hive_from_path(path: &str) -> Result<(Hive, &str), RegistryError> {
    // split the key path to hive and subkey otherwise it's just a hive
    let (hive, subkey) = match path.find('\\') {
        Some(index) => {
            // split at index, but don't include the character at index
            let (hive, subkey) = path.split_at(index);
            (hive, &subkey[1..])
        }
        None => (path, ""),
    };

    match hive {
        "HKCC" | "HKEY_CURRENT_CONFIG" => Ok((Hive::CurrentConfig, subkey)),
        "HKCU" | "HKEY_CURRENT_USER" => Ok((Hive::CurrentUser, subkey)),
        "HKCR" | "HKEY_CLASSES_ROOT" => Ok((Hive::ClassesRoot, subkey)),
        "HKLM" | "HKEY_LOCAL_MACHINE" => Ok((Hive::LocalMachine, subkey)),
        "HKU" | "HKEY_USERS" => Ok((Hive::Users, subkey)),
        _ => Err(RegistryError::InvalidHive(hive.to_string())),
    }
}

fn open_regkey(path: &str, permission: Security) -> Result<(RegKey, &str), RegistryError> {
    let (hive, subkey) = get_hive_from_path(path)?;
    match hive.open(subkey, permission) {
        Ok(regkey) => Ok((regkey, subkey)),
        // handle NotFound error
        Err(key::Error::NotFound(_, _)) => Err(RegistryError::RegistryKeyNotFound(path.to_string())),
        Err(e) => Err(RegistryError::RegistryKey(e)),
    }
}

fn get_parent_key_path(key_path: &str) -> &str {
    match key_path.rfind('\\') {
        Some(index) => &key_path[..index],
        None => "",
    }
}

fn convert_reg_value(value: &Data) -> Result<Option<RegistryValueData>, RegistryError> {
    match value {
        Data::String(s) => Ok(Some(RegistryValueData::String(s.to_string_lossy()))),
        Data::ExpandString(s) => Ok(Some(RegistryValueData::ExpandString(s.to_string_lossy()))),
        Data::Binary(b) => Ok(Some(RegistryValueData::Binary(b.clone()))),
        Data::U32(d) => Ok(Some(RegistryValueData::DWord(*d))),
        Data::MultiString(m) => {
            let m: Vec<String> = m.iter().map(|s| s.to_string_lossy()).collect();
            Ok(Some(RegistryValueData::MultiString(m)))
        }
        Data::U64(q) => Ok(Some(RegistryValueData::QWord(*q))),
        Data::None => Ok(None),
        _ => Err(RegistryError::UnsupportedValueDataType),
    }
}

#[test]
fn get_hklm_key() {
    let reg_helper = RegistryHelper::new_from_json(r#"{"keyPath":"HKEY_LOCAL_MACHINE"}"#).unwrap();
    let reg_config = reg_helper.get().unwrap();
    assert_eq!(reg_config.key_path, r#"HKEY_LOCAL_MACHINE"#);
    assert_eq!(reg_config.value_name, None);
    assert_eq!(reg_config.value_data, None);
}

#[test]
fn get_product_name() {
    let reg_helper = RegistryHelper::new_from_json(
        r#"{"keyPath":"HKLM\\SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion","valueName":"ProductName"}"#,
    )
    .unwrap();
    let reg_config = reg_helper.get().unwrap();
    assert_eq!(
        reg_config.key_path,
        r#"HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion"#
    );
    assert_eq!(reg_config.value_name, Some("ProductName".to_string()));
    assert!(matches!(reg_config.value_data, Some(RegistryValueData::String(s)) if s.starts_with("Windows ")));
}

#[test]
fn get_nonexisting_key() {
    let reg_helper = RegistryHelper::new_from_json(r#"{"keyPath":"HKCU\\DoesNotExist"}"#).unwrap();
    let reg_config = reg_helper.get().unwrap();
    assert_eq!(reg_config.key_path, r#"HKCU\DoesNotExist"#);
    assert_eq!(reg_config.value_name, None);
    assert_eq!(reg_config.value_data, None);
    assert_eq!(reg_config.exist, Some(false));
}

#[test]
fn get_nonexisting_value() {
    let reg_helper =
        RegistryHelper::new_from_json(r#"{"keyPath":"HKCU\\Software","valueName":"DoesNotExist"}"#).unwrap();
    let reg_config = reg_helper.get().unwrap();
    assert_eq!(reg_config.key_path, r#"HKCU\Software"#);
    assert_eq!(reg_config.value_name, Some("DoesNotExist".to_string()));
    assert_eq!(reg_config.value_data, None);
    assert_eq!(reg_config.exist, Some(false));
}

#[test]
fn set_and_remove_test_value() {
    let reg_helper = RegistryHelper::new_from_json(
        r#"{"keyPath":"HKCU\\DSCTest\\DSCSubKey","valueName":"TestValue","valueData": { "String": "Hello"} }"#,
    )
    .unwrap();
    reg_helper.set().unwrap();
    let result = reg_helper.get().unwrap();
    assert_eq!(result.key_path, r#"HKCU\DSCTest\DSCSubKey"#);
    assert_eq!(result.value_name, Some("TestValue".to_string()));
    assert_eq!(result.value_data, Some(RegistryValueData::String("Hello".to_string())));
    reg_helper.remove().unwrap();
    let result = reg_helper.get().unwrap();
    assert_eq!(result.key_path, r#"HKCU\DSCTest\DSCSubKey"#);
    assert_eq!(result.value_name, Some("TestValue".to_string()));
    assert_eq!(result.value_data, None);
    assert_eq!(result.exist, Some(false));
    let reg_helper = RegistryHelper::new_from_json(r#"{"keyPath":"HKCU\\DSCTest"}"#).unwrap();
    let result = reg_helper.get().unwrap();
    assert_eq!(result.key_path, r#"HKCU\DSCTest"#);
    assert_eq!(result.value_name, None);
    assert_eq!(result.value_data, None);
    reg_helper.remove().unwrap();
    let result = reg_helper.get().unwrap();
    assert_eq!(result.key_path, r#"HKCU\DSCTest"#);
    assert_eq!(result.value_name, None);
    assert_eq!(result.value_data, None);
    assert_eq!(result.exist, Some(false));
}

#[test]
fn delete_tree() {
    let reg_helper = RegistryHelper::new_from_json(
        r#"{"keyPath":"HKCU\\DSCTest2\\DSCSubKey","valueName":"TestValue","valueData": { "String": "Hello"} }"#,
    )
    .unwrap();
    reg_helper.set().unwrap();
    let result = reg_helper.get().unwrap();
    assert_eq!(result.key_path, r#"HKCU\DSCTest2\DSCSubKey"#);
    assert_eq!(result.value_name, Some("TestValue".to_string()));
    assert_eq!(result.value_data, Some(RegistryValueData::String("Hello".to_string())));
    let reg_helper = RegistryHelper::new_from_json(r#"{"keyPath":"HKCU\\DSCTest2"}"#).unwrap();
    reg_helper.remove().unwrap();
    let result = reg_helper.get().unwrap();
    assert_eq!(result.key_path, r#"HKCU\DSCTest2"#);
    assert_eq!(result.value_name, None);
    assert_eq!(result.value_data, None);
    assert_eq!(result.exist, Some(false));
}
