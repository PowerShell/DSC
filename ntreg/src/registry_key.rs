// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use core::mem::size_of;
use ntapi::ntregapi::{self, KEY_FULL_INFORMATION, KEY_BASIC_INFORMATION, PKEY_BASIC_INFORMATION, PKEY_FULL_INFORMATION};
use ntapi::winapi::ctypes::c_void;
use ntapi::winapi::shared::ntdef::{HANDLE, NTSTATUS, NT_SUCCESS};
use ntapi::winapi::shared::ntstatus::{STATUS_OBJECT_PATH_SYNTAX_BAD};
use ntapi::winapi::um::handleapi::{INVALID_HANDLE_VALUE};
use ntapi::winapi::um::winnt::{KEY_ALL_ACCESS, KEY_CREATE_SUB_KEY, KEY_READ, ACCESS_MASK, KEY_SET_VALUE};
use ntuserinfo::{NtCurrentUserInfo};
use ntstatuserror::{NtStatusError};
use std::ptr::null_mut;

use super::*;
use registry_value::*;

const REG_OPTION_NON_VOLATILE: u32 = 0x00000000;
const REG_OPENED_EXISTING_KEY: u32 = 0x00000002;

/// Represents a registry key including its properties.
#[derive(Clone)]
pub struct RegistryKey {
    pub handle: HANDLE,
    pub path: String,
    pub name: String,
    pub sub_key_count: u32,
    pub max_key_name_len: u32,
    pub value_count: u32,
    pub max_value_name_len: u32,
    pub max_data_len: u32,
}

impl RegistryKey {
    /// Returns a RegistryKey object for a valid existing path.
    ///
    /// # Arguments
    ///
    /// * `path` - A string containing the path to the registry key.
    ///   The path can be a NT path `\Registry\Machine\...` or a Win32 path `HKLM\...`.
    ///   The path will be opened for read access.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ntreg::registry_key::*;
    /// let key = RegistryKey::new(r"HKLM\Software\Microsoft\Windows NT\CurrentVersion");
    /// assert!(key.is_ok());
    /// ```
    pub fn new(path: &str) -> Result<Self, NtStatusError> {
        match open_key(path, KEY_READ) {
            Ok((key, key_information)) => {
                Ok(
                    RegistryKey {
                        handle: key,
                        path: path.to_string(),
                        name: path.split('\\').last().unwrap().to_string(),
                        sub_key_count: key_information.SubKeys,
                        max_key_name_len: key_information.MaxNameLen,
                        value_count: key_information.Values,
                        max_value_name_len: key_information.MaxValueNameLen,
                        max_data_len: key_information.MaxValueDataLen,
                    }
                )
            }
            Err(err) => {
                Err(err)
            }
        }
    }

    /// Returns an iterator over the sub keys of the current key.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ntreg::registry_key::*;
    /// let key = RegistryKey::new(r"HKCU\Software").unwrap();
    /// for sub_key in key.subkeys() {
    ///    println!("{}", sub_key.name);
    /// }
    /// ```
    pub fn subkeys(self) -> RegistrySubkeys {
        RegistrySubkeys {
            key: self,
            index: 0,
        }
    }

    /// Returns an iterator over the values of the current key.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ntreg::registry_key::*;
    /// let key = RegistryKey::new(r"HKCU\Environment").unwrap();
    /// for value in key.values() {
    ///   println!("{}", value.name);
    /// }
    /// ```
    pub fn values(self) -> RegistryValues {
        RegistryValues {
            key: self,
            index: 0,
        }
    }

    fn create_key_helper(&self, name: &str, must_create: bool) -> Result<RegistryKey, NtStatusError> {
        let mut key_handle: HANDLE = INVALID_HANDLE_VALUE;
        let mut disposition: u32 = 0;
        let mut new_path = match convert_registry_path(&self.path) {
            Ok(path) => path,
            Err(err) => return Err(err),
        };
        new_path.push('\\');
        new_path.push_str(name);
        let key_path = name;
        let obj = ObjectAttributes::new(self.handle, &key_path.to_string());
        let status = unsafe {
            ntregapi::NtCreateKey(
                &mut key_handle,
                KEY_CREATE_SUB_KEY,
                &mut obj.as_struct(),
                0,
                null_mut(),
                REG_OPTION_NON_VOLATILE,
                &mut disposition
            )
        };

        if !NT_SUCCESS(status) {
            return Err(NtStatusError::new(status, "Failed to create key."));
        }

        if must_create && disposition == REG_OPENED_EXISTING_KEY {
            return Err(NtStatusError::new(status, "Key already exists."));
        }

        Ok(
            RegistryKey {
                handle: key_handle,
                path: new_path,
                name: name.to_string(),
                sub_key_count: 0,
                max_key_name_len: 0,
                value_count: 0,
                max_value_name_len: 0,
                max_data_len: 0,
            }
        )
    }

    /// Creates a new sub key.
    ///
    /// # Arguments
    ///
    /// * `name` - A string containing the name of the new sub key.
    ///   This will fail if the sub key already exists.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ntreg::registry_key::*;
    /// let key = RegistryKey::new("HKCU").unwrap();
    /// let new_key = key.create_key("NewKey");
    /// assert!(new_key.is_ok());
    /// new_key.unwrap().delete(false);
    /// ```
    pub fn create_key(&self, name: &str) -> Result<RegistryKey, NtStatusError> {
        self.create_key_helper(name, true)
    }

    /// Creates or opens an existing sub key.
    ///
    /// # Arguments
    ///
    /// * `name` - A string containing the name of the new sub key.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ntreg::registry_key::*;
    /// let key = RegistryKey::new("HKCU").unwrap();
    /// let new_key = key.create_or_get_key("Software");
    /// assert!(new_key.is_ok());
    /// ```
    pub fn create_or_get_key(&self, name: &str) -> Result<RegistryKey, NtStatusError> {
        self.create_key_helper(name, false)
    }

    /// Deletes a sub key.
    ///
    /// # Arguments
    ///
    /// * `recurse` - A boolean indicating if the sub key should be deleted recursively.
    ///   If this is false and the sub key has sub keys, the deletion will fail.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ntreg::registry_key::*;
    /// let key = RegistryKey::new("HKCU").unwrap();
    /// let new_key = key.create_key("NewKey");
    /// assert!(new_key.is_ok());
    /// new_key.unwrap().delete(true);
    /// ```
    pub fn delete(&self, recurse: bool) -> Result<(), NtStatusError> {
        let key = match open_key(&self.path.clone(), KEY_ALL_ACCESS) {
            Ok((key, _)) => key,
            Err(err) => return Err(err),
        };

        if recurse {
            let mykey = match RegistryKey::new(&self.path) {
                Ok(key) => key,
                Err(err) => return Err(err),
            };

            for subkey in mykey.subkeys() {
                subkey.delete(true)?;
            }
        }

        let status = unsafe {
            ntregapi::NtDeleteKey(key)
        };

        if !NT_SUCCESS(status) {
            return Err(NtStatusError::new(status, "Failed to delete key."));
        }

        Ok(())
    }

    /// Deletes a value for the current key.
    ///
    /// # Arguments
    ///
    /// * `name` - A string containing the name of the value.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ntreg::registry_key::*;
    /// # use ntreg::registry_value::*;
    /// let key = RegistryKey::new("HKCU").unwrap();
    /// key.set_value("TestValue", &RegistryValueData::String("TestValueData".to_string()));
    /// key.delete_value("TestValue");
    /// key.delete(false);
    /// ```
    pub fn delete_value(&self, name: &str) ->Result<(), NtStatusError> {
        let key = match open_key(&self.path.clone(), KEY_SET_VALUE) {
            Ok((key, _)) => key,
            Err(err) => return Err(err),
        };

        let status = unsafe {
            ntregapi::NtDeleteValueKey(key, &mut name.to_string().as_unicode_string().as_struct() as *mut UNICODE_STRING)
        };

        if !NT_SUCCESS(status) {
            return Err(NtStatusError::new(status, "Failed to delete value."));
        }

        Ok(())
    }

    /// Gets a value for the current key.
    ///
    /// # Arguments
    ///
    /// * `name` - A string containing the name of the value.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ntreg::registry_key::*;
    /// let key = RegistryKey::new(r"HKCU\Environment").unwrap();
    /// let value = key.get_value("Path");
    /// assert!(value.is_ok());
    /// ```
    pub fn get_value(&self, name: &str) -> Result<RegistryValue, NtStatusError> {
        let mut reg_value = RegistryValue {
            key_path: self.path.clone(),
            name: name.to_string(),
            data: RegistryValueData::None,
        };
        reg_value.query()?;
        Ok(reg_value)
    }

    /// Creates or sets a value for the current key.
    ///
    /// # Arguments
    /// * `name` - A string containing the name of the value.
    ///
    /// # Remarks
    /// Returns a RegistryValue representing the new value.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ntreg::registry_key::*;
    /// # use ntreg::registry_value::*;
    /// let key = RegistryKey::new("HKCU").unwrap();
    /// key.set_value("TestValue", &RegistryValueData::String("TestValueData".to_string()));
    /// key.delete(false);
    /// ```
    pub fn set_value(&self, name: &str, value: &RegistryValueData) -> Result<RegistryValue, NtStatusError> {
        let key = match open_key(&self.path.clone(), KEY_ALL_ACCESS) {
            Ok((key, _)) => key,
            Err(err) => return Err(err),
        };

        let mut data: *mut c_void = null_mut();
        let mut string_data: Vec<u16>;
        let mut binary_data: Vec<u8>;
        let mut len: usize = 0;
        match value {
            RegistryValueData::None => {}
            RegistryValueData::String(ref string) | RegistryValueData::ExpandString(ref string) | RegistryValueData::Link(ref string) => {
                string_data = string.encode_utf16().collect();
                string_data.push(0);
                data = string_data.as_mut_ptr() as *mut c_void;
                len = string_data.len() * size_of::<u16>();
            }
            RegistryValueData::MultiString(ref multi_string) => {
                string_data = Vec::new();
                for s in multi_string {
                    let mut s: Vec<u16> = s.encode_utf16().collect();
                    s.push(0);
                    string_data.extend(s)
                }
                string_data.push(0);
                data = string_data.as_mut_ptr() as *mut c_void;
                len = string_data.len() * size_of::<u16>();
            }
            RegistryValueData::Binary(ref binary) | RegistryValueData::FullResourceDescriptor(ref binary) |
                RegistryValueData::ResourceList(ref binary) | RegistryValueData::ResourceRequirementsList(ref binary) => {
                binary_data = binary.clone();
                data = binary_data.as_mut_ptr() as *mut c_void;
                len = binary_data.len();
            }
            RegistryValueData::DWord(ref value) => {
                let mut value = *value;
                data = &mut value as *mut u32 as *mut c_void;
                len = size_of::<u32>();
            }
            RegistryValueData::QWord(ref value) => {
                let mut value = *value;
                data = &mut value as *mut u64 as *mut c_void;
                len = size_of::<u64>();
            }
        };

        let data_type_value: u32 = match value {
            RegistryValueData::None => 0,
            RegistryValueData::String(_) => 1,
            RegistryValueData::ExpandString(_) => 2,
            RegistryValueData::Binary(_) => 3,
            RegistryValueData::DWord(_) => 4,
            RegistryValueData::Link(_) => 6,
            RegistryValueData::MultiString(_) => 7,
            RegistryValueData::ResourceList(_) => 8,
            RegistryValueData::FullResourceDescriptor(_) => 9,
            RegistryValueData::ResourceRequirementsList(_) => 10,
            RegistryValueData::QWord(_) => 11,
        };

        let status = unsafe {
            ntregapi::NtSetValueKey(
                key,
                &mut name.to_string().as_unicode_string().as_struct() as *mut UNICODE_STRING,
                0,
                data_type_value,
                data,
                len as u32
            )
        };

        if !NT_SUCCESS(status) {
            return Err(NtStatusError::new(status, "Failed to set value."));
        }

        let mut new_value = RegistryValue {
            key_path: self.path.clone(),
            name: name.to_string(),
            data: value.clone(),
        };

        new_value.query()?;
        Ok(new_value)
    }
}

/// Closes a registry key when it goes out of scope.
impl Drop for RegistryKey {
    fn drop(&mut self) {
        if self.handle == INVALID_HANDLE_VALUE {
            return;
        }

        unsafe {
            ntapi::ntobapi::NtClose(self.handle)
        };

        self.handle = INVALID_HANDLE_VALUE;
    }
}

/// Represents a registry sub key iterator
pub struct RegistrySubkeys {
    key: RegistryKey,
    index: u32,
}

impl Iterator for RegistrySubkeys {
    type Item = RegistryKey;
    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.key.sub_key_count {
            return None;
        }

        let max_key_name_len = self.key.max_key_name_len + 1;
        let buf_len = size_of::<KEY_BASIC_INFORMATION>() as u32 + (max_key_name_len * size_of::<u16>() as u32);
        let mut len: u32 = buf_len;
        let buf: Vec<u8> = vec![0; len as usize];
        let subkey_information: PKEY_BASIC_INFORMATION = buf.as_ptr() as PKEY_BASIC_INFORMATION;

        let status = unsafe {
            ntregapi::NtEnumerateKey(
                self.key.handle,
                self.index,
                ntregapi::KeyBasicInformation,
                subkey_information as *mut c_void,
                len,
                &mut len
            )
        };

        if status == ERROR_NO_MORE_ITEMS {
            return None;
        }

        if !NT_SUCCESS(status) {
            return None;
        }

        let name_length = unsafe {
            ((*subkey_information).NameLength / size_of::<u16>() as u32) as usize
        };

        let name = String::from_utf16_lossy( unsafe {
            std::slice::from_raw_parts((*subkey_information).Name.as_ptr(), name_length)
        });

        self.index += 1;

        let sub_key_path = format!("{}\\{}", self.key.path, name);
        Some(RegistryKey::new(sub_key_path.as_str()).unwrap())
    }
}

fn open_key(path: &str, desired_access: ACCESS_MASK) -> Result<(HANDLE, KEY_FULL_INFORMATION), NtStatusError> {
    let path : String = match convert_registry_path(path) {
        Ok(path) => path,
        Err(err) => return Err(err),
    };
    let obj = ObjectAttributes::new(null_mut(), &path);
    let mut key: HANDLE = null_mut();
    let status: NTSTATUS = unsafe {
        ntregapi::NtOpenKey(&mut key, desired_access, &mut obj.as_struct())
    };

    if !NT_SUCCESS(status) {
        return Err(NtStatusError::new(status, format!("Failed to open key '{}'.", path).as_str()));
    }

    let key_information: KEY_FULL_INFORMATION = unsafe {
        let buf: Vec<u8> = vec![0; size_of::<KEY_FULL_INFORMATION>()];
        let key_information_ptr: PKEY_FULL_INFORMATION = buf.as_ptr() as PKEY_FULL_INFORMATION;
        let mut key_information: KEY_FULL_INFORMATION = *key_information_ptr;
        let mut result_length: u32 = 0;
        let status = ntregapi::NtQueryKey(
            key,
            ntregapi::KeyFullInformation,
            &mut key_information as *mut KEY_FULL_INFORMATION as *mut c_void,
            size_of::<KEY_FULL_INFORMATION>() as u32,
            &mut result_length);

        if !NT_SUCCESS(status) {
            return Err(NtStatusError::new(status, "Failed to query key."));
        }

        key_information
    };

    Ok((key, key_information))
}

fn convert_registry_path(path: &str) -> Result<String, NtStatusError> {
    if path == r"\Registry\Machine" || path.starts_with(r"\Registry\Machine\") ||
        path == r"\Registry\User" || path.starts_with(r"\Registry\User\") {
        return Ok(path.to_string());
    }

    let hive : &str;
    let mut key_path = String::from("");
    if path.contains('\\') {
        hive = path.split('\\').next().unwrap();
        key_path = path.split('\\').skip(1).collect::<Vec<&str>>().join("\\");
    }
    else {
        hive = path;
    }

    let mut path: String = match hive.to_uppercase().as_str() {
        "HKEY_CLASSES_ROOT" | "HKCR" => {
            format!("\\Registry\\Machine\\Software\\Classes\\{}", key_path)
        },
        "HKEY_CURRENT_USER" | "HKCU" => {
            let user = NtCurrentUserInfo::new()?;
            format!("\\Registry\\User\\{}\\{}", user.sid, key_path)
        },
        "HKEY_LOCAL_MACHINE" | "HKLM" => {
            format!("\\Registry\\Machine\\{}", key_path)
        },
        "HKEY_USERS" | "HKU" => {
            format!("\\Registry\\User\\{}", key_path)
        },
        "HKEY_CURRENT_CONFIG" | "HKCC" => {
            format!("\\Registry\\Machine\\System\\CurrentControlSet\\Hardware Profiles\\Current\\{}", key_path)
        },
        _ => {
            return Err(NtStatusError::new(STATUS_OBJECT_PATH_SYNTAX_BAD, "Invalid registry path."));
        }
    };

    if path.ends_with('\\') {
        path.pop();
    }

    Ok(path)
}
