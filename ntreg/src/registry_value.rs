// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use core::mem::size_of;
use ntapi::winapi::ctypes::c_void;
use ntapi::ntregapi::{self, KEY_VALUE_FULL_INFORMATION, PKEY_VALUE_FULL_INFORMATION};
use ntapi::winapi::shared::ntdef::{NT_SUCCESS};
use ntstatuserror::{NtStatusError};
use std::fmt;
use std::fmt::Write;

use super::*;
use registry_key::*;

/// Represents registry value data.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RegistryValueData {
    None,
    String(String),
    ExpandString(String),
    Binary(Vec<u8>),
    DWord(u32), // excluding REG_DWORD_BIG_ENDIAN as it's intended for UNIX systems and not used on Windows
    Link(String),
    MultiString(Vec<String>),
    ResourceList(Vec<u8>),
    FullResourceDescriptor(Vec<u8>),
    ResourceRequirementsList(Vec<u8>),
    QWord(u64),
}

/// Represents a registry value.
#[derive(Clone)]
pub struct RegistryValue {
    pub key_path: String,
    pub name: String,
    pub data: RegistryValueData,
}

impl RegistryValue {
    /// Refresh the contents of a registry value.
    ///
    /// # Example
    ///
    /// ```
    /// # use ntreg::registry_key::RegistryKey;
    /// let key = RegistryKey::new(r"HKCU\Environment").unwrap();
    /// let mut value = key.get_value("Path").unwrap();
    /// assert!(value.query().is_ok());
    pub fn query(&mut self) -> Result<(), NtStatusError> {
        let key = RegistryKey::new(self.key_path.clone().as_str())?;
        let value_name_unicode = self.name.as_unicode_string();
        let mut len = size_of::<KEY_VALUE_FULL_INFORMATION>() as u32 + key.max_value_name_len + key.max_data_len + 2; // +2 for the null terminators
        let buf: Vec<u8> = vec![0; len as usize];
        let value_information: PKEY_VALUE_FULL_INFORMATION = buf.as_ptr() as PKEY_VALUE_FULL_INFORMATION;
        let status = unsafe {
            ntregapi::NtQueryValueKey(
                key.handle,
                &mut value_name_unicode.as_struct() as *mut UNICODE_STRING,
                ntregapi::KeyValueFullInformation,
                value_information as *mut c_void,
                len,
                &mut len)
        };

        if !NT_SUCCESS(status) {
            return Err(NtStatusError::new(status, "Failed to query registry value"));
        }

        self.data = get_data_value(value_information);
        Ok(())
    }
}

/// Represents a registry key's values iterator.
pub struct RegistryValues {
    pub key: RegistryKey,
    pub index: u32,
}

impl Iterator for RegistryValues {
    type Item = RegistryValue;
    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.key.sub_key_count {
            return None;
        }

        let max_value_len = self.key.max_value_name_len + 1;
        let max_data_len = self.key.max_data_len + 1;
        let buf_len = size_of::<KEY_VALUE_FULL_INFORMATION>() as u32 + max_value_len + max_data_len;
        let mut len: u32 = buf_len;
        let buf: Vec<u8> = vec![0; len as usize];
        let value_information: PKEY_VALUE_FULL_INFORMATION = buf.as_ptr() as PKEY_VALUE_FULL_INFORMATION;
        let status = unsafe {
            ntregapi::NtEnumerateValueKey(
                self.key.handle,
                self.index,
                ntregapi::KeyValueFullInformation,
                value_information as *mut c_void,
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
            ((*value_information).NameLength / size_of::<u16>() as u32) as usize
        };

        let reg_value_name = String::from_utf16_lossy( unsafe {
            std::slice::from_raw_parts((*value_information).Name.as_ptr(), name_length)
        });

        self.index += 1;

        Some(RegistryValue {
            key_path: self.key.path.clone(),
            name: if name_length > 0 { reg_value_name } else { "".to_string() },
            data: get_data_value(value_information),
        })
    }
}

impl fmt::Display for RegistryValueData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let data : String = match self {
            RegistryValueData::None => "None".to_string(),
            RegistryValueData::String(ref data) => format!("String: {data}"),
            RegistryValueData::ExpandString(ref data) => format!("ExpandString: {data}"),
            RegistryValueData::Binary(ref data) => format!("Binary: {}", convert_vec_to_string(&data.to_vec())),
            RegistryValueData::DWord(ref data) => format!("Dword: {data}"),
            RegistryValueData::Link(ref data) => format!("Link: {data}"),
            RegistryValueData::MultiString(ref data) => format!("MultiString: {data:?}"),
            RegistryValueData::ResourceList(ref data) => format!("ResourceList: {:?}", convert_vec_to_string(&data.to_vec())),
            RegistryValueData::FullResourceDescriptor(ref data) => format!("FullResourceDescriptor: {:?}", convert_vec_to_string(&data.to_vec())),
            RegistryValueData::ResourceRequirementsList(ref data) => format!("ResourceRequirementsList: {:?}", convert_vec_to_string(&data.to_vec())),
            RegistryValueData::QWord(ref data) => format!("Qword: {data}"),
        };

        write!(f, "{}", data)
    }
}

fn get_data_value(value_information: PKEY_VALUE_FULL_INFORMATION) -> RegistryValueData {
    let data_length = unsafe {
        (*value_information).DataLength as usize
    };

    let data_ptr = unsafe {
        (value_information as *const u8).add((*value_information).DataOffset as usize) as *const u8
    };

    let data = unsafe {
        std::slice::from_raw_parts(data_ptr, data_length)
    };

    match unsafe { (*value_information).Type } {
        1 => RegistryValueData::String(String::from_utf16_lossy(unsafe {
            // remove null terminator
            std::slice::from_raw_parts(data_ptr as *const u16, (data_length / size_of::<u16>()) - 1)
        })),
        2 => RegistryValueData::ExpandString(String::from_utf16_lossy(unsafe {
            // remove null terminator
            std::slice::from_raw_parts(data_ptr as *const u16, (data_length / size_of::<u16>()) - 1)
        })),
        3 => RegistryValueData::Binary(data.to_vec()),
        4 => RegistryValueData::DWord(unsafe {
            *(data_ptr as *const u32)
        }),
        6 => RegistryValueData::Link(String::from_utf16_lossy(unsafe {
            // remove null terminator
            std::slice::from_raw_parts(data_ptr as *const u16, (data_length / size_of::<u16>()) - 1)
        })),
        7 => {
            let mut multi_string: Vec<String> = Vec::new();
            let mut offset = 0;
            while offset < data_length {
                let mut string_length = 0;
                while offset + string_length < data_length && unsafe { *(data_ptr.add(offset + string_length) as *mut u16) } != 0 {
                    string_length += size_of::<u16>();
                }

                let string = String::from_utf16_lossy(unsafe {
                    std::slice::from_raw_parts(data_ptr.add(offset) as *const u16, string_length / size_of::<u16>())
                });

                if string.is_empty() {
                    // end of multi string null terminator
                    break;
                }

                multi_string.push(string);
                offset += string_length + size_of::<u16>();
            }

            RegistryValueData::MultiString(multi_string)
        },
        8 => RegistryValueData::ResourceList(data.to_vec()),
        9 => RegistryValueData::FullResourceDescriptor(data.to_vec()),
        10 => RegistryValueData::ResourceRequirementsList(data.to_vec()),
        11 => RegistryValueData::QWord(unsafe {
            data_ptr.read_unaligned().into()
        }),
        _ => RegistryValueData::None,
    }
}

fn convert_vec_to_string(data: &Vec<u8>) -> String {
    let mut result = String::new();
    for byte in data {
        write!(result, "{byte:02x}").unwrap();
    }
    result
}
