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
mod acl;

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
            access_control_list: None,
            metadata: None,
            exist: None,
            registry_file_path: None,
            in_desired_state: None,
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

        if self.config.value_name.is_none()
            && let Some(requested_acl) = &self.config.access_control_list
        {
            return match acl::get(&self.config.key_path, requested_acl) {
                Ok(current_acl) => Ok(Registry {
                    key_path: self.config.key_path.clone(),
                    access_control_list: Some(current_acl),
                    ..Default::default()
                }),
                Err(RegistryError::RegistryKeyNotFound(_)) => Ok(Registry {
                    key_path: self.config.key_path.clone(),
                    exist: Some(false),
                    ..Default::default()
                }),
                Err(error) => Err(error),
            };
        }

        if let Some(requested_acl) = &self.config.access_control_list {
            acl::validate(requested_acl)?;
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
        let access_control_list = match &self.config.access_control_list {
            Some(requested_acl) => Some(acl::get(&self.config.key_path, requested_acl)?),
            None => None,
        };

        if let Some(value_name) = &self.config.value_name {
            let value = match reg_key.value(value_name) {
                Ok(value) => value,
                Err(value::Error::NotFound(_,_)) => {
                    exist = false;
                    return Ok(Registry {
                        key_path: self.config.key_path.clone(),
                        value_name: Some(value_name.clone()),
                        access_control_list,
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
                access_control_list,
                ..Default::default()
            })
        } else {
            Ok(Registry {
                key_path: self.config.key_path.clone(),
                access_control_list,
                ..Default::default()
            })
        }
    }

    /// Enumerate all named values under the configured registry key.
    ///
    /// A missing key is returned as an empty collection.
    ///
    /// # Errors
    ///
    /// * `RegistryError` - The error that occurred.
    pub fn get_values(&self) -> Result<Vec<(String, RegistryValueData)>, RegistryError> {
        if self.offline_hive.is_some() {
            return Err(RegistryError::OfflineRegistry(
                t!("error.offlineValueEnumeration").to_string(),
            ));
        }

        let (reg_key, _) = match self.open(Security::Read) {
            Ok(result) => result,
            Err(RegistryError::RegistryKeyNotFound(_)) => return Ok(Vec::new()),
            Err(error) => return Err(error),
        };

        let mut values = Vec::new();
        for value in reg_key.values() {
            let value = value.map_err(|error| RegistryError::Registry(error.into()))?;
            if let Some(data) = convert_reg_value(value.data())? {
                values.push((value.name().to_string_lossy(), data));
            }
        }

        Ok(values)
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
            if self.config.access_control_list.is_some() {
                return Err(RegistryError::OfflineRegistry(
                    t!("error.offlineAcl").to_string(),
                ));
            }
            return self.set_offline();
        }
        if let Some(acl) = &self.config.access_control_list {
            acl::preflight(acl)?;
        }

        let mut what_if_metadata: Vec<String> = Vec::new();
        if self.config.value_name.is_none()
            && let Some(config_acl) = &self.config.access_control_list
            && acl::can_set(&self.config.key_path, config_acl)?
        {
            if self.what_if {
                what_if_metadata.push(
                    t!("registry_helper.whatIfSetAcl", key_path = &self.config.key_path).to_string(),
                );
                let effective_acl = acl::effective(&self.config.key_path, config_acl)?;
                return Ok(Some(Registry {
                    key_path: self.config.key_path.clone(),
                    access_control_list: Some(effective_acl),
                    metadata: Some(Metadata { what_if: Some(what_if_metadata) }),
                    ..Default::default()
                }));
            }
            acl::set(&self.config.key_path, config_acl)?;
            return Ok(None);
        }

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
        // Prepare the complete descriptor before writing a value. In merge mode this reads and
        // parses the existing ACL, so unsupported existing ACEs fail before registry data changes.
        let mut prepared_acl = match (&self.config.access_control_list, reg_key.is_some()) {
            (Some(requested), true) => Some(acl::prepare(&self.config.key_path, requested)?),
            _ => None,
        };
        let what_if_acl = if let Some(prepared) = &prepared_acl {
            Some(prepared.effective().clone())
        } else if self.what_if {
            self.config
                .access_control_list
                .as_ref()
                .map(|requested| acl::effective_for_new(&self.config.key_path, requested))
                .transpose()?
        } else {
            None
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
                if self.config.access_control_list.is_some() {
                    what_if_metadata.push(
                        t!("registry_helper.whatIfSetAcl", key_path = &self.config.key_path).to_string(),
                    );
                }
                return Ok(Some(Registry {
                    key_path: self.config.key_path.clone(),
                    value_data: convert_reg_value(&data)?,
                    value_name: self.config.value_name.clone(),
                    access_control_list: what_if_acl,
                    metadata: if what_if_metadata.is_empty() { None } else { Some(Metadata { what_if: Some(what_if_metadata) })},
                    ..Default::default()
                }));
            }

            if let Some(reg_key) = reg_key {
                reg_key.set_value(&value_name, &data)?;
            }
        }

        if self.what_if {
            if self.config.access_control_list.is_some() {
                what_if_metadata.push(
                    t!("registry_helper.whatIfSetAcl", key_path = &self.config.key_path).to_string(),
                );
            }
            return Ok(Some(Registry {
                key_path: self.config.key_path.clone(),
                access_control_list: what_if_acl,
                metadata: if what_if_metadata.is_empty() { None } else { Some(Metadata { what_if: Some(what_if_metadata) })},
                ..Default::default()
            }));
        }

        if let Some(prepared) = &mut prepared_acl {
            acl::set_prepared(&self.config.key_path, prepared)?;
        } else if let Some(requested_acl) = &self.config.access_control_list {
            acl::set(&self.config.key_path, requested_acl)?;
        }

        Ok(None)
    }

    /// Test whether the registry matches the desired state.
    ///
    /// Compares `_exist`, `valueData` (when `valueName` is specified), and `accessControlList`
    /// (when specified) against the actual state returned by [`Self::get`]. The `accessControlList`
    /// comparison uses the shared access control list satisfaction semantics (principal
    /// normalization, alias-aware access mask comparison, and `_purge`-aware entry matching)
    /// instead of raw JSON equality, so that, for example, a differently cased or
    /// differently-named-but-equivalent principal, or an access mask expressed as an alias versus
    /// its decomposed rights, doesn't produce a false drift report.
    ///
    /// # Returns
    ///
    /// * `Registry` - The registry struct with `_inDesiredState` populated.
    ///
    /// # Errors
    ///
    /// * `RegistryError` - The error that occurred.
    pub fn test(&self) -> Result<Registry, RegistryError> {
        if self.offline_hive.is_some() {
            return self.test_offline();
        }

        // `get` performs the same accessControlList validation (for example rejecting
        // `accessControlList: {}`) that `set` does, so there's no need to duplicate it here.
        let actual = self.get()?;

        let desired_exist = self.config.exist.unwrap_or(true);
        let actual_exist = actual.exist.unwrap_or(true);
        let mut in_desired_state = desired_exist == actual_exist;

        // Value data and the access control list only matter when the instance is (and should
        // be) present; when existence itself doesn't match, that alone determines compliance.
        if desired_exist && actual_exist {
            if self.config.value_name.is_some() {
                let desired_value_data = self.config.value_data.clone().unwrap_or(RegistryValueData::None);
                let actual_value_data = actual.value_data.clone().unwrap_or(RegistryValueData::None);
                if desired_value_data != actual_value_data {
                    in_desired_state = false;
                }
            }

            if let Some(requested_acl) = &self.config.access_control_list {
                let actual_acl = actual.access_control_list.clone().unwrap_or_default();
                if !acl::is_satisfied(requested_acl, &actual_acl)? {
                    in_desired_state = false;
                }
            }
        }

        Ok(Registry {
            in_desired_state: Some(in_desired_state),
            ..actual
        })
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

        // Accumulate what-if metadata like set()
        let mut what_if_metadata: Vec<String> = Vec::new();

        // For deleting a value, we need SetValue permission (KEY_SET_VALUE).
        // Try to open with the minimal required permission.
        // If that fails due to permission, try with AllAccess as a fallback.
        let (reg_key, _subkey) = match self.open(Security::SetValue) {
            Ok(reg_key) => reg_key,
            // handle NotFound error
            Err(RegistryError::RegistryKeyNotFound(_)) => {
                if self.what_if {
                    what_if_metadata.push(t!("registry_helper.whatIfDeleteNonexistingKey", subkey = &self.config.key_path).to_string());
                    return Ok(Some(Registry {
                        key_path: self.config.key_path.clone(),
                        value_name: self.config.value_name.clone(),
                        metadata: Some(Metadata { what_if: Some(what_if_metadata) }),
                        ..Default::default()
                    }));
                }
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
        if self.config.access_control_list.is_some() {
            return Err(RegistryError::OfflineRegistry(
                t!("error.offlineAcl").to_string(),
            ));
        }
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

    /// Test whether an offline registry hive matches the desired state. Access control lists
    /// are never supported for offline hives, so [`Self::get_offline`] rejects a request that
    /// specifies one; otherwise, only existence and (when `valueName` is specified) value data
    /// are compared.
    fn test_offline(&self) -> Result<Registry, RegistryError> {
        let actual = self.get_offline()?;

        let desired_exist = self.config.exist.unwrap_or(true);
        let actual_exist = actual.exist.unwrap_or(true);
        let mut in_desired_state = desired_exist == actual_exist;

        if desired_exist && actual_exist && self.config.value_name.is_some() {
            let desired_value_data = self.config.value_data.clone().unwrap_or(RegistryValueData::None);
            let actual_value_data = actual.value_data.clone().unwrap_or(RegistryValueData::None);
            if desired_value_data != actual_value_data {
                in_desired_state = false;
            }
        }

        Ok(Registry {
            in_desired_state: Some(in_desired_state),
            ..actual
        })
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
                // Check if the value actually exists before reporting what-if
                let value_exists = hive.get_value(&self.subkey, value_name)?.is_some();
                hive.close_key(key);
                if !value_exists {
                    return Ok(Some(Registry {
                        key_path: self.config.key_path.clone(),
                        value_name: Some(value_name.clone()),
                        exist: Some(false),
                        ..Default::default()
                    }));
                }
                what_if_metadata.push(t!("registry_helper.whatIfDeleteValue", value_name = value_name).to_string());
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
    let u16_slice: Vec<u16> = data.as_chunks::<2>().0.iter()
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
    let u16_slice: Vec<u16> = data.as_chunks::<2>().0.iter()
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
fn decode_utf16_bytes_ignores_incomplete_code_unit() {
    assert_eq!(decode_utf16_bytes(&[b'A', 0, 0xff]), "A");
}

#[test]
fn decode_multi_sz_ignores_incomplete_code_unit() {
    let data = [b'A', 0, 0, 0, 0, 0, 0xff];
    assert_eq!(decode_multi_sz(&data), vec!["A"]);
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

#[test]
fn test_reports_in_desired_state_when_value_matches() {
    let helper = RegistryHelper::new_from_json(r#"{"keyPath":"HKCU\\DSCTestNative\\Value","valueName":"TestValue","valueData": { "String": "Hello"} }"#).unwrap();
    helper.set().unwrap();

    let result = helper.test().unwrap();

    assert_eq!(result.in_desired_state, Some(true));

    RegistryHelper::new_from_json(r#"{"keyPath":"HKCU\\DSCTestNative"}"#).unwrap().remove().unwrap();
}

#[test]
fn test_reports_not_in_desired_state_when_value_differs() {
    let helper = RegistryHelper::new_from_json(r#"{"keyPath":"HKCU\\DSCTestNative2\\Value","valueName":"TestValue","valueData": { "String": "Hello"} }"#).unwrap();
    helper.set().unwrap();

    let desired = RegistryHelper::new_from_json(r#"{"keyPath":"HKCU\\DSCTestNative2\\Value","valueName":"TestValue","valueData": { "String": "Goodbye"} }"#).unwrap();
    let result = desired.test().unwrap();

    assert_eq!(result.in_desired_state, Some(false));

    RegistryHelper::new_from_json(r#"{"keyPath":"HKCU\\DSCTestNative2"}"#).unwrap().remove().unwrap();
}

#[test]
fn test_reports_not_in_desired_state_when_exist_mismatches() {
    let desired = RegistryHelper::new_from_json(r#"{"keyPath":"HKCU\\DSCTestNativeDoesNotExist"}"#).unwrap();

    let result = desired.test().unwrap();

    assert_eq!(result.in_desired_state, Some(false));
}

#[test]
fn test_satisfies_purge_true_acl_via_alias_equivalent_access_mask() {
    // The individual rights below, combined, are bit-for-bit identical to `fullControl`; a real
    // system read always reports the most comprehensive alias that fits exactly, so a
    // subsequent Get/Test will see `fullControl` for this entry even though it was set using
    // the decomposed list below. A raw per-field/array comparison between the two
    // representations would (incorrectly) treat this as a mismatch.
    //
    // The principal is the well-known "Everyone" SID rather than, say, LocalSystem: replacement
    // mode writes the DACL with only this one entry, and the test process must remain able to read
    // (and ultimately clean up) the key afterward.
    let decomposed_json = r#"{"keyPath":"HKCU\\DSCTestNativeAlias","accessControlList":{"discretionary":[{"principal":"S-1-1-0","type":"allow","accessMask":["queryValue","setValue","createSubKey","enumerateSubKeys","notify","createLink","delete","readPermissions","changePermissions","takeOwnership"]}],"discretionaryProtected":false}}"#;
    RegistryHelper::new_from_json(decomposed_json).unwrap().set().unwrap();

    let result = RegistryHelper::new_from_json(decomposed_json).unwrap().test().unwrap();

    assert_eq!(result.in_desired_state, Some(true), "{result:?}");

    RegistryHelper::new_from_json(r#"{"keyPath":"HKCU\\DSCTestNativeAlias","_exist":false}"#).unwrap().remove().unwrap();
}

#[test]
fn set_rejects_access_control_list_with_no_sections() {
    let helper = RegistryHelper::new_from_json(r#"{"keyPath":"HKCU\\DSCTestNativeEmptyAcl","accessControlList":{}}"#).unwrap();

    let error = helper.set().unwrap_err();

    assert!(matches!(error, RegistryError::Acl(_)));
}

#[test]
fn registry_access_right_rejects_removed_wow64_variants() {
    // `wow6464Key`/`wow6432Key` are registry *view* selectors for `RegOpenKeyExW`'s
    // `samDesired`, not ACE permissions, and must no longer be accepted as accessMask values.
    let wow64_64: Result<crate::config::RegistryAccessRight, _> =
        serde_json::from_str("\"wow6464Key\"");
    let wow64_32: Result<crate::config::RegistryAccessRight, _> =
        serde_json::from_str("\"wow6432Key\"");
    assert!(wow64_64.is_err());
    assert!(wow64_32.is_err());
}

#[test]
fn registry_access_right_generic_variants_have_expected_bits() {
    use crate::config::RegistryAccessRight;
    use lib_acl::AccessMask;

    assert_eq!(RegistryAccessRight::GenericRead.bits(), 0x8000_0000);
    assert_eq!(RegistryAccessRight::GenericWrite.bits(), 0x4000_0000);
    assert_eq!(RegistryAccessRight::GenericExecute.bits(), 0x2000_0000);
    assert_eq!(RegistryAccessRight::GenericAll.bits(), 0x1000_0000);
}

#[test]
fn set_what_if_with_value_name_returns_effective_acl_for_omitted_and_false_purge() {
    // Note the doubled backslash: `key_path` is embedded directly into JSON string literals
    // below, so it must already be JSON-escaped.
    let key_path = r"HKCU\\DSCTestNativeWhatIfExisting";
    // Create the key (and an initial ACE) first, representing an "existing key" scenario. The
    // principal is the well-known "Everyone" SID rather than, say, LocalSystem: replacement mode
    // writes the DACL with only this one entry, and the test process must remain able to read (and
    // ultimately clean up) the key afterward.
    let initial_json = format!(
        r#"{{"keyPath":"{key_path}","accessControlList":{{"discretionary":[{{"principal":"S-1-1-0","type":"allow","accessMask":["fullControl"]}}]}}}}"#
    );
    RegistryHelper::new_from_json(&initial_json).unwrap().set().unwrap();

    for (purge_property, should_merge) in [
        ("", false),
        (r#","_purge":false"#, true),
    ] {
        let what_if_json = format!(
            r#"{{"keyPath":"{key_path}","valueName":"Hello","valueData":{{"String":"World"}},"accessControlList":{{"discretionary":[{{"principal":"S-1-5-19","type":"allow","accessMask":["read"]}}]{purge_property}}}}}"#
        );
        let mut helper = RegistryHelper::new_from_json(&what_if_json).unwrap();
        helper.enable_what_if();

        let result = helper.set().unwrap().expect("what-if returns a preview");
        let principals: Vec<String> = result
            .access_control_list
            .unwrap()
            .discretionary
            .unwrap()
            .into_iter()
            .map(|entry| entry.principal)
            .collect();

        assert_eq!(
            principals.contains(&"S-1-1-0".to_string()),
            should_merge,
            "{principals:?}"
        );
        assert!(principals.contains(&"S-1-5-19".to_string()), "{principals:?}");
    }

    // What-if must not have actually written the value.
    let actual = RegistryHelper::new_from_json(&format!(
        r#"{{"keyPath":"{key_path}","valueName":"Hello"}}"#
    ))
    .unwrap()
    .get()
    .unwrap();
    assert_eq!(actual.exist, Some(false));

    RegistryHelper::new_from_json(&format!(r#"{{"keyPath":"{key_path}"}}"#))
        .unwrap()
        .remove()
        .unwrap();
}

#[test]
fn set_what_if_for_missing_key_returns_effective_acl_for_omitted_and_false_purge() {
    for (purge_property, expected_count, expected_protection) in [
        ("", Some(3), Some(false)),
        (r#","_purge":false"#, None, Some(false)),
    ] {
        let what_if_json = format!(
            r#"{{"keyPath":"HKCU\\DSCTestNativeWhatIfMissing","valueName":"Hello","valueData":{{"String":"World"}},"accessControlList":{{"discretionary":[{{"principal":"S-1-5-19","type":"allow","accessMask":["read"]}},{{"principal":"S-1-5-7","type":"deny","accessMask":["read"]}},{{"principal":"S-1-5-19","type":"allow","accessMask":["read"]}}]{purge_property}}}}}"#
        );
        let mut helper = RegistryHelper::new_from_json(&what_if_json).unwrap();
        helper.enable_what_if();

        let result = helper.set().unwrap().expect("what-if returns a preview");
        let acl = result.access_control_list.unwrap();
        let entries = acl.discretionary.unwrap();

        if let Some(expected_count) = expected_count {
            assert_eq!(entries.len(), expected_count);
        }
        assert_eq!(acl.discretionary_protected, expected_protection);
        if purge_property.contains("false") {
            assert_eq!(entries[0].principal, "S-1-5-7");
            assert_eq!(
                entries
                    .iter()
                    .filter(|entry| entry.principal == "S-1-5-19")
                    .count(),
                1
            );
        }
    }
}

#[test]
fn set_what_if_without_value_name_returns_effective_acl_for_missing_key() {
    let mut helper = RegistryHelper::new_from_json(
        r#"{"keyPath":"HKCU\\DSCTestNativeWhatIfMissingAclOnly","accessControlList":{"discretionary":[{"principal":"S-1-5-19","type":"allow","accessMask":["read"]}]}}"#,
    )
    .unwrap();
    helper.enable_what_if();

    let result = helper.set().unwrap().expect("what-if returns a preview");
    let acl = result.access_control_list.unwrap();

    assert_eq!(acl.discretionary_protected, Some(false));
    assert_eq!(
        acl.discretionary.unwrap()[0].principal,
        "S-1-5-19".to_string()
    );
}

#[test]
fn set_what_if_for_missing_key_includes_the_acl_set_will_inherit() {
    let parent_path = r"HKCU\\DSCTestNativeWhatIfInheritance";
    let target_path = format!(r"{parent_path}\\Child\\Grandchild");
    let _ = RegistryHelper::new_from_json(&format!(r#"{{"keyPath":"{parent_path}"}}"#))
        .unwrap()
        .remove();

    let parent_json = format!(
        r#"{{"keyPath":"{parent_path}","accessControlList":{{"discretionary":[{{"principal":"S-1-1-0","type":"allow","accessMask":["fullControl"],"flags":["containerInherit"]}}]}}}}"#
    );
    RegistryHelper::new_from_json(&parent_json)
        .unwrap()
        .set()
        .unwrap();

    let request_json = format!(
        r#"{{"keyPath":"{target_path}","valueName":"Value","valueData":{{"String":"Data"}},"accessControlList":{{"discretionary":[{{"principal":"S-1-5-19","type":"allow","accessMask":["read"]}}],"_purge":false}}}}"#
    );
    let mut what_if_helper = RegistryHelper::new_from_json(&request_json).unwrap();
    what_if_helper.enable_what_if();
    let preview = what_if_helper
        .set()
        .unwrap()
        .expect("what-if returns a preview")
        .access_control_list
        .unwrap();

    RegistryHelper::new_from_json(&request_json)
        .unwrap()
        .set()
        .unwrap();
    let actual = RegistryHelper::new_from_json(&format!(
        r#"{{"keyPath":"{target_path}","accessControlList":{{"discretionary":[]}}}}"#
    ))
    .unwrap()
    .get()
    .unwrap()
    .access_control_list
    .unwrap();

    RegistryHelper::new_from_json(&format!(r#"{{"keyPath":"{parent_path}"}}"#))
        .unwrap()
        .remove()
        .unwrap();

    assert_eq!(preview.discretionary, actual.discretionary);
    assert_eq!(
        preview.discretionary_protected,
        actual.discretionary_protected
    );
}
