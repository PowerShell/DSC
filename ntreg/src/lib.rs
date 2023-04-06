// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! Abstraction over the Windows Registry API.
//! Limited dependency using only NT APIs.

use core::mem::size_of;
use ntapi::winapi::shared::ntdef::{HANDLE, NTSTATUS, OBJECT_ATTRIBUTES, OBJ_CASE_INSENSITIVE, UNICODE_STRING, ULONG, USHORT};
use std::ptr::null_mut;

pub mod registry_key;
pub mod registry_value;

const ERROR_NO_MORE_ITEMS: NTSTATUS = 259;

/// Represents the buffer for a `UNICODE_STRING`.
pub struct UnicodeString {
    buffer: Vec<u16>,
}

impl UnicodeString {
    /// Create a new `UnicodeString`.
    /// 
    /// # Arguments
    ///
    /// * `string` - The string to create the `UnicodeString` from.
    #[must_use]
    pub fn new(string: &str) -> UnicodeString {
        let mut buffer: Vec<u16> = string.encode_utf16().collect();
        buffer.push(0);
        UnicodeString { buffer }
    }

    /// Get the `UNICODE_STRING` representation of the buffer.
    /// 
    /// # Panics
    /// 
    /// Will panic if the size of `UNICODE_STRING` cannot be converted to `USHORT`.
    ///
    #[must_use]
    pub fn as_struct(&self) -> UNICODE_STRING {
        let Ok(length) = USHORT::try_from((self.buffer.len() - 1) * 2) else {
            panic!("Failed to convert size of UNICODE_STRING to USHORT");
        };
        let Ok(max_length) = USHORT::try_from(self.buffer.len() * 2) else {
            panic!("Failed to convert size of UNICODE_STRING to USHORT");
        };
        UNICODE_STRING {
            Length: length,
            MaximumLength: max_length,
            Buffer: self.buffer.as_ptr() as *mut u16,
        }
    }
}

trait AsUnicodeString {
    fn as_unicode_string(&self) -> UnicodeString;
}

impl AsUnicodeString for String {
    fn as_unicode_string(&self) -> UnicodeString {
        UnicodeString::new(self.as_str())
    }
}

pub struct ObjectAttributes {
    unicode_string: UnicodeString,
    root_directory: HANDLE,
}

impl ObjectAttributes {
    /// Create a new `ObjectAttributes`.
    /// 
    /// # Arguments
    ///
    /// * `root_directory` - The root directory to use.
    /// * `object_name` - The object name to use.
    pub fn new(root_directory: HANDLE, object_name: &String) -> ObjectAttributes {
        ObjectAttributes {
            unicode_string: object_name.as_unicode_string(),
            root_directory,
        }
    }

    /// Get the `OBJECT_ATTRIBUTES` representation of the struct.
    /// 
    /// # Panics
    /// 
    /// Will panic if the size of `UNICODE_STRING` cannot be converted to `ULONG`.
    /// 
    /// # Safety
    ///
    /// The returned `OBJECT_ATTRIBUTES` struct is only valid as long as the
    /// `UnicodeString` struct is valid.
    /// 
    #[must_use]
    pub fn as_struct(&self) -> OBJECT_ATTRIBUTES {
        let mut unicode_string = self.unicode_string.as_struct();
        let Ok(length) = ULONG::try_from(size_of::<UNICODE_STRING>()) else {
            panic!("Failed to convert size of UNICODE_STRING to ULONG");
        };
        OBJECT_ATTRIBUTES {
            Length: length,
            RootDirectory: self.root_directory,
            ObjectName: std::ptr::addr_of_mut!(unicode_string),
            Attributes: OBJ_CASE_INSENSITIVE,
            SecurityDescriptor: null_mut(),
            SecurityQualityOfService: null_mut(),
        }
    }
}
