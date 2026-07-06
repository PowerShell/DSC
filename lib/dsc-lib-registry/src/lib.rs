// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use registry::{Data, Hive, RegKey, Security, key, value};
use rust_i18n::t;
use utfx::{U16CString, UCString};
use crate::config::{Metadata, Registry, RegistryValueData};
use crate::error::RegistryError;

rust_i18n::i18n!("locales", fallback = "en-us");

pub mod error;
pub mod config;
pub mod offreg;

use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::path::Path;
use offreg::{OfflineHive, REG_SZ, REG_EXPAND_SZ, REG_BINARY, REG_DWORD, REG_MULTI_SZ, REG_QWORD, REG_NONE};

pub struct RegistryHelper {
    config: Registry,
    hive: Hive,
    subkey: String,
    what_if: bool,
    offline_hive: Option<OfflineHive>,
}

impl RegistryHelper {
    /// Create a new `RegistryHelper` from json.
    ///
    /// # Arguments
    ///
    /// * `registry_entry` - The string with registry configuration information.
    ///
    /// # Errors
    ///
    /// * `RegistryError` - The error that occurred.
    pub fn new_from_json(registry_entry: &str) -> Result<Self, RegistryError> {
        let registry: Registry = match serde_json::from_str(registry_entry) {
            Ok(config) => config,
            Err(e) => return Err(RegistryError::Json(e)),
        };
        let key_path = registry.key_path.clone();
        let (hive, subkey) = get_hive_from_path(&key_path)?;

        let offline_hive = if let Some(ref file_path) = registry.registry_file_path {
            Some(OfflineHive::open(Path::new(file_path))?)
        } else {
            None
        };

        Ok(
            Self {
                config: registry,
                hive,
                subkey: subkey.to_string(),
                what_if: false,
                offline_hive,
            }
        )
    }

    /// Create a new `RegistryHelper` from registry configuration.
    /// 
    /// # Arguments
    /// 
    /// * `registry_entry` - The registry configuration struct.
    /// 
    /// # Errors
    /// 
    /// * `RegistryError` - The error that occurred.
    pub fn new_from_registry(registry_entry: &Registry) -> Result<Self, RegistryError> {
        let (hive, subkey) = get_hive_from_path(&registry_entry.key_path)?;

        let offline_hive = if let Some(ref file_path) = registry_entry.registry_file_path {
            Some(OfflineHive::open(Path::new(file_path))?)
        } else {
            None
        };

        Ok(
            Self {
                config: registry_entry.clone(),
                hive,
                subkey: subkey.to_string(),
                what_if: false,
                offline_hive,
            }
        )
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
    pub fn new(key_path: &str, value_name: Option<String>, value_data: Option<RegistryValueData>) -> Result<Self, RegistryError> {
        let (hive, subkey) = get_hive_from_path(key_path)?;
        let config = Registry {
            key_path: key_path.to_string(),
            value_name,
            value_data,
            metadata: None,
            exist: None,
            registry_file_path: None,
        };
        Ok(
            Self {
                config,
                hive,
                subkey: subkey.to_string(),
                what_if: false,
                offline_hive: None,
            }
        )
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
        if self.offline_hive.is_some() {
            return self.get_offline();
        }

        let exist: bool;
        let (reg_key, _subkey) = match self.open(Security::Read) {
            Ok((reg_key, subkey)) => {
                (reg_key, subkey)
            },
            Err(RegistryError::RegistryKeyNotFound(_)) => {
                exist = false;
                return Ok(Registry {
                    key_path: self.config.key_path.clone(),
                    exist: Some(exist),
                    ..Default::default()
                });
            },
            Err(e) => return Err(e),
        };

        if let Some(value_name) = &self.config.value_name {
            let value = match reg_key.value(value_name) {
                Ok(value) => value,
                Err(value::Error::NotFound(_,_)) => {
                    exist = false;
                    return Ok(Registry {
                        key_path: self.config.key_path.clone(),
                        value_name: Some(value_name.clone()),
                        exist: Some(exist),
                        ..Default::default()
                    });
                },
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
        if self.offline_hive.is_some() {
            return self.set_offline();
        }

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
                    }
                    else {
                        reg_key = reg_key.create(path, Security::CreateSubKey)?;
                    }
                }
                if self.what_if {
                    None
                }
                else {
                    Some(self.open(Security::Write)?.0)
                }
            },
            Err(e) => return self.handle_error_or_what_if(e)
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
                },
                RegistryValueData::ExpandString(s) => {
                    let Ok(utf16) = U16CString::from_str(s) else {
                        return self.handle_error_or_what_if(RegistryError::Utf16Conversion("valueData".to_string()));
                    };
                    Data::ExpandString(utf16)
                },
                RegistryValueData::Binary(b) => {
                    Data::Binary(b.clone())
                },
                RegistryValueData::DWord(d) => {
                    Data::U32(*d)
                },
                RegistryValueData::MultiString(m) => {
                    let mut m16: Vec<UCString<u16>> = Vec::<UCString<u16>>::new();
                    for s in m {
                        let Ok(utf16) = U16CString::from_str(s) else {
                            return self.handle_error_or_what_if(RegistryError::Utf16Conversion("valueData".to_string()));
                        };
                        m16.push(utf16);
                    }
                    Data::MultiString(m16)
                },
                RegistryValueData::QWord(q) => {
                    Data::U64(*q)
                },
                RegistryValueData::None => {
                    Data::None
                },
            };

            if self.what_if {
                return Ok(Some(Registry {
                    key_path: self.config.key_path.clone(),
                    value_data: convert_reg_value(&data)?,
                    value_name: self.config.value_name.clone(),
                    metadata: if what_if_metadata.is_empty() { None } else { Some(Metadata { what_if: Some(what_if_metadata) })},
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
                metadata: if what_if_metadata.is_empty() { None } else { Some(Metadata { what_if: Some(what_if_metadata) })},
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
        if self.offline_hive.is_some() {
            return self.remove_offline();
        }

        // For deleting a value, we need SetValue permission (KEY_SET_VALUE).
        // Try to open with the minimal required permission.
        // If that fails due to permission, try with AllAccess as a fallback.
        let (reg_key, _subkey) = match self.open(Security::SetValue) {
            Ok(reg_key) => reg_key,
            // handle NotFound error
            Err(RegistryError::RegistryKeyNotFound(_)) => {
                return Ok(None);
            },
            Err(RegistryError::RegistryKey(key::Error::PermissionDenied(_, _))) => {
                match self.open(Security::AllAccess) {
                    Ok(reg_key) => reg_key,
                    Err(e) => return self.handle_error_or_what_if(e),
                }
            },
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
                    metadata: Some(Metadata { what_if: Some(what_if_metadata) }),
                    ..Default::default()
                }));
            }
            match reg_key.delete_value(value_name) {
                Ok(()) | Err(value::Error::NotFound(_, _)) => {
                    // if the value doesn't exist, we don't need to do anything
                },
                Err(e) => return self.handle_error_or_what_if(RegistryError::RegistryValue(e)),
            }
        } else {
            // to delete the key, we need to open the parent key with CreateSubKey permission
            // which includes the ability to delete subkeys
            let parent_path = get_parent_key_path(&self.config.key_path);
            let (hive, parent_subkey) = get_hive_from_path(parent_path)?;
            let parent_reg_key = match hive.open(parent_subkey, Security::CreateSubKey) {
                Ok(k) => k,
                Err(e) => return self.handle_error_or_what_if(RegistryError::RegistryKey(e)),
            };

            // get the subkey name
            let subkey_name = &self.config.key_path[parent_path.len() + 1..];

            if self.what_if {
                what_if_metadata.push(t!("registry_helper.whatIfDeleteSubkey", subkey_name = subkey_name).to_string());
                return Ok(Some(Registry {
                    key_path: self.config.key_path.clone(),
                    metadata: Some(Metadata { what_if: Some(what_if_metadata) }),
                    ..Default::default()
                }));
            }
            let Ok(subkey_name) = UCString::<u16>::from_str(subkey_name) else {
                return self.handle_error_or_what_if(RegistryError::Utf16Conversion("subkey_name".to_string()));
            };

            match parent_reg_key.delete(subkey_name, true) {
                Ok(()) | Err(key::Error::NotFound(_, _)) => {
                    // if the subkey doesn't exist, we don't need to do anything
                },
                Err(e) => return self.handle_error_or_what_if(RegistryError::RegistryKey(e)),
            }
        }
        Ok(None)
    }

    fn open(&self, permission: Security) -> Result<(RegKey, &str), RegistryError> {
        open_regkey(&self.config.key_path, permission)
    }

    // --- Offline registry implementations ---

    fn get_offline(&self) -> Result<Registry, RegistryError> {
        let hive = self.offline_hive.as_ref().unwrap();

        // Check if the subkey exists
        match hive.open_key(&self.subkey) {
            Ok(key) => {
                hive.close_key(key);
            },
            Err(RegistryError::RegistryKeyNotFound(_)) => {
                return Ok(Registry {
                    key_path: self.config.key_path.clone(),
                    exist: Some(false),
                    ..Default::default()
                });
            },
            Err(e) => return Err(e),
        }

        if let Some(value_name) = &self.config.value_name {
            match hive.get_value(&self.subkey, value_name)? {
                Some((value_type, data)) => {
                    let value_data = convert_offline_reg_value(value_type, &data)?;
                    Ok(Registry {
                        key_path: self.config.key_path.clone(),
                        value_name: Some(value_name.clone()),
                        value_data,
                        ..Default::default()
                    })
                },
                None => {
                    Ok(Registry {
                        key_path: self.config.key_path.clone(),
                        value_name: Some(value_name.clone()),
                        exist: Some(false),
                        ..Default::default()
                    })
                }
            }
        } else {
            Ok(Registry {
                key_path: self.config.key_path.clone(),
                ..Default::default()
            })
        }
    }

    fn set_offline(&self) -> Result<Option<Registry>, RegistryError> {
        let hive = self.offline_hive.as_ref().unwrap();
        let mut what_if_metadata: Vec<String> = Vec::new();

        // Ensure the key exists (create if needed)
        let key = match hive.open_key(&self.subkey) {
            Ok(k) => k,
            Err(RegistryError::RegistryKeyNotFound(_)) => {
                if self.what_if {
                    what_if_metadata.push(t!("registry_helper.whatIfCreateKey", subkey = &self.subkey).to_string());
                    if let Some(value_name) = &self.config.value_name {
                        let value_data = self.config.value_data.clone().unwrap_or(RegistryValueData::None);
                        return Ok(Some(Registry {
                            key_path: self.config.key_path.clone(),
                            value_name: Some(value_name.clone()),
                            value_data: Some(value_data),
                            metadata: Some(Metadata { what_if: Some(what_if_metadata) }),
                            ..Default::default()
                        }));
                    }
                    return Ok(Some(Registry {
                        key_path: self.config.key_path.clone(),
                        metadata: Some(Metadata { what_if: Some(what_if_metadata) }),
                        ..Default::default()
                    }));
                }
                hive.create_key(&self.subkey)?
            },
            Err(e) => return self.handle_error_or_what_if(e),
        };

        if let Some(value_name) = &self.config.value_name {
            let value_data = self.config.value_data.clone().unwrap_or(RegistryValueData::None);

            if self.what_if {
                hive.close_key(key);
                return Ok(Some(Registry {
                    key_path: self.config.key_path.clone(),
                    value_data: Some(value_data),
                    value_name: Some(value_name.clone()),
                    metadata: if what_if_metadata.is_empty() { None } else { Some(Metadata { what_if: Some(what_if_metadata) }) },
                    ..Default::default()
                }));
            }

            let (reg_type, data) = convert_value_data_to_offline(&value_data)?;
            hive.set_value(&key, value_name, reg_type, &data)?;
        }

        hive.close_key(key);

        if self.what_if {
            return Ok(Some(Registry {
                key_path: self.config.key_path.clone(),
                metadata: if what_if_metadata.is_empty() { None } else { Some(Metadata { what_if: Some(what_if_metadata) }) },
                ..Default::default()
            }));
        }

        // Save the hive back to disk
        hive.save(Path::new(hive.path()))?;

        Ok(None)
    }

    fn remove_offline(&self) -> Result<Option<Registry>, RegistryError> {
        let hive = self.offline_hive.as_ref().unwrap();
        let mut what_if_metadata: Vec<String> = Vec::new();

        let key = match hive.open_key(&self.subkey) {
            Ok(k) => k,
            Err(RegistryError::RegistryKeyNotFound(_)) => {
                return Ok(None);
            },
            Err(e) => return self.handle_error_or_what_if(e),
        };

        if let Some(value_name) = &self.config.value_name {
            if self.what_if {
                what_if_metadata.push(t!("registry_helper.whatIfDeleteValue", value_name = value_name).to_string());
                hive.close_key(key);
                return Ok(Some(Registry {
                    key_path: self.config.key_path.clone(),
                    value_name: Some(value_name.clone()),
                    metadata: Some(Metadata { what_if: Some(what_if_metadata) }),
                    ..Default::default()
                }));
            }
            hive.delete_value(&key, value_name)?;
        } else {
            // Delete the key itself
            let subkey_name = match self.subkey.rfind('\\') {
                Some(idx) => &self.subkey[idx + 1..],
                None => &self.subkey,
            };
            let parent_subkey = get_parent_key_path(&self.subkey);

            if self.what_if {
                what_if_metadata.push(t!("registry_helper.whatIfDeleteSubkey", subkey_name = subkey_name).to_string());
                hive.close_key(key);
                return Ok(Some(Registry {
                    key_path: self.config.key_path.clone(),
                    metadata: Some(Metadata { what_if: Some(what_if_metadata) }),
                    ..Default::default()
                }));
            }

            hive.close_key(key);
            let parent_key = hive.open_key(parent_subkey)?;
            hive.delete_key(&parent_key, subkey_name)?;
            hive.close_key(parent_key);

            // Save the hive back to disk
            hive.save(Path::new(hive.path()))?;
            return Ok(None);
        }

        hive.close_key(key);

        // Save the hive back to disk
        hive.save(Path::new(hive.path()))?;

        Ok(None)
    }

    // Find the valid parent key that exists and the subkeys that don't exist
    // the subkeys are returned in reverse order (the closest subkey is the last one in the vector)
    fn get_valid_parent_key_and_subkeys(&self) -> Result<(RegKey, Vec<&str>), RegistryError> {
        let parent_key: RegKey;
        let mut subkeys: Vec<&str> = Vec::new();
        let parent_key_path = get_parent_key_path(&self.subkey);
        let subkey_name = if parent_key_path.is_empty() { &self.subkey } else {
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
                },
                Err(key::Error::NotFound(_,_)) => {
                    let parent_key_path = get_parent_key_path(current_key_path);
                    if parent_key_path.is_empty() {
                        subkeys.insert(0, current_key_path);
                        current_key_path = "";
                    } else {
                        let subkey_name = &current_key_path[parent_key_path.len() + 1..];
                        subkeys.insert(0, subkey_name);
                        current_key_path = parent_key_path;
                    }
                },
                Err(e) => {
                    return Err(RegistryError::RegistryKey(e));
                },
            }
        }

        Ok((parent_key, subkeys))
    }

    fn handle_error_or_what_if(&self, error: RegistryError) -> Result<Option<Registry>, RegistryError> {
        if self.what_if {
            return Ok(Some(Registry {
                key_path: self.config.key_path.clone(),
                metadata: Some(Metadata { what_if: Some(vec![error.to_string()]) }),
                ..Default::default()
            }));
        }
        Err(error)
    }
}

fn get_hive_from_path(path: &str) -> Result<(Hive, &str), RegistryError> {
    // split the key path to hive and subkey otherwise it's just a hive
    let (hive, subkey)= match path.find('\\') {
        Some(index) => {
            // split at index, but don't include the character at index
            let (hive, subkey) = path.split_at(index);
            (hive, &subkey[1..])
        },
        None => (path, ""),
    };

    match hive {
        "HKCC" | "HKEY_CURRENT_CONFIG" => Ok((Hive::CurrentConfig, subkey)),
        "HKCU" | "HKEY_CURRENT_USER" => Ok((Hive::CurrentUser, subkey)),
        "HKCR" | "HKEY_CLASSES_ROOT" => Ok((Hive::ClassesRoot, subkey)),
        "HKLM" | "HKEY_LOCAL_MACHINE" => Ok((Hive::LocalMachine, subkey)),
        "HKU"  | "HKEY_USERS" => Ok((Hive::Users, subkey)),
        _ => Err(RegistryError::InvalidHive(hive.to_string()))
    }
}

fn open_regkey(path: &str, permission: Security) -> Result<(RegKey, &str), RegistryError> {
    let (hive, subkey) = get_hive_from_path(path)?;
    match hive.open(subkey, permission) {
        Ok(regkey) => Ok((regkey, subkey)),
        // handle NotFound error
        Err(key::Error::NotFound(_, _)) => {
            Err(RegistryError::RegistryKeyNotFound(path.to_string()))
        },
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
        },
        Data::U64(q) => Ok(Some(RegistryValueData::QWord(*q))),
        Data::None => Ok(None),
        _ => Err(RegistryError::UnsupportedValueDataType)
    }
}

/// Convert raw offline registry value data (type + bytes) to `RegistryValueData`.
fn convert_offline_reg_value(value_type: u32, data: &[u8]) -> Result<Option<RegistryValueData>, RegistryError> {
    match value_type {
        REG_SZ => {
            let s = decode_utf16_bytes(data);
            Ok(Some(RegistryValueData::String(s)))
        },
        REG_EXPAND_SZ => {
            let s = decode_utf16_bytes(data);
            Ok(Some(RegistryValueData::ExpandString(s)))
        },
        REG_BINARY => Ok(Some(RegistryValueData::Binary(data.to_vec()))),
        REG_DWORD => {
            if data.len() >= 4 {
                let val = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
                Ok(Some(RegistryValueData::DWord(val)))
            } else {
                Ok(Some(RegistryValueData::DWord(0)))
            }
        },
        REG_MULTI_SZ => {
            let strings = decode_multi_sz(data);
            Ok(Some(RegistryValueData::MultiString(strings)))
        },
        REG_QWORD => {
            if data.len() >= 8 {
                let val = u64::from_le_bytes([data[0], data[1], data[2], data[3], data[4], data[5], data[6], data[7]]);
                Ok(Some(RegistryValueData::QWord(val)))
            } else {
                Ok(Some(RegistryValueData::QWord(0)))
            }
        },
        REG_NONE => Ok(None),
        _ => Err(RegistryError::UnsupportedValueDataType)
    }
}

/// Convert `RegistryValueData` to raw bytes + type for offline registry.
fn convert_value_data_to_offline(value_data: &RegistryValueData) -> Result<(u32, Vec<u8>), RegistryError> {
    match value_data {
        RegistryValueData::String(s) => {
            let data = encode_utf16_bytes(s);
            Ok((REG_SZ, data))
        },
        RegistryValueData::ExpandString(s) => {
            let data = encode_utf16_bytes(s);
            Ok((REG_EXPAND_SZ, data))
        },
        RegistryValueData::Binary(b) => Ok((REG_BINARY, b.clone())),
        RegistryValueData::DWord(d) => Ok((REG_DWORD, d.to_le_bytes().to_vec())),
        RegistryValueData::MultiString(m) => {
            let data = encode_multi_sz(m);
            Ok((REG_MULTI_SZ, data))
        },
        RegistryValueData::QWord(q) => Ok((REG_QWORD, q.to_le_bytes().to_vec())),
        RegistryValueData::None => Ok((REG_NONE, Vec::new())),
    }
}

/// Decode a null-terminated UTF-16LE byte slice to a String.
fn decode_utf16_bytes(data: &[u8]) -> String {
    let u16_slice: Vec<u16> = data.chunks_exact(2)
        .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
        .collect();
    // Strip trailing null
    let len = u16_slice.iter().position(|&c| c == 0).unwrap_or(u16_slice.len());
    String::from_utf16_lossy(&u16_slice[..len])
}

/// Encode a String to null-terminated UTF-16LE bytes.
fn encode_utf16_bytes(s: &str) -> Vec<u8> {
    let wide: Vec<u16> = OsStr::new(s).encode_wide().chain(std::iter::once(0)).collect();
    wide.iter().flat_map(|&c| c.to_le_bytes()).collect()
}

/// Decode REG_MULTI_SZ: double-null-terminated list of null-terminated UTF-16LE strings.
fn decode_multi_sz(data: &[u8]) -> Vec<String> {
    let u16_slice: Vec<u16> = data.chunks_exact(2)
        .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
        .collect();
    let mut strings = Vec::new();
    let mut start = 0;
    for (i, &c) in u16_slice.iter().enumerate() {
        if c == 0 {
            if i == start {
                break; // double null - end of multi-sz
            }
            strings.push(String::from_utf16_lossy(&u16_slice[start..i]));
            start = i + 1;
        }
    }
    strings
}

/// Encode a list of strings to REG_MULTI_SZ format (double-null-terminated UTF-16LE).
fn encode_multi_sz(strings: &[String]) -> Vec<u8> {
    let mut result: Vec<u16> = Vec::new();
    for s in strings {
        result.extend(OsStr::new(s).encode_wide());
        result.push(0); // null terminator for each string
    }
    result.push(0); // final null terminator
    result.iter().flat_map(|&c| c.to_le_bytes()).collect()
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
    let reg_helper = RegistryHelper::new_from_json(r#"{"keyPath":"HKLM\\SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion","valueName":"ProductName"}"#).unwrap();
    let reg_config = reg_helper.get().unwrap();
    assert_eq!(reg_config.key_path, r#"HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion"#);
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
    let reg_helper = RegistryHelper::new_from_json(r#"{"keyPath":"HKCU\\Software","valueName":"DoesNotExist"}"#).unwrap();
    let reg_config = reg_helper.get().unwrap();
    assert_eq!(reg_config.key_path, r#"HKCU\Software"#);
    assert_eq!(reg_config.value_name, Some("DoesNotExist".to_string()));
    assert_eq!(reg_config.value_data, None);
    assert_eq!(reg_config.exist, Some(false));
}

#[test]
fn set_and_remove_test_value() {
    let reg_helper = RegistryHelper::new_from_json(r#"{"keyPath":"HKCU\\DSCTest\\DSCSubKey","valueName":"TestValue","valueData": { "String": "Hello"} }"#).unwrap();
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
    let reg_helper = RegistryHelper::new_from_json(r#"{"keyPath":"HKCU\\DSCTest2\\DSCSubKey","valueName":"TestValue","valueData": { "String": "Hello"} }"#).unwrap();
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
