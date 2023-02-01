//! Abstraction over the Windows Registry API.
//! Limited dependency using only NT APIs.

use core::mem::size_of;
use ntapi::winapi::shared::ntdef::{HANDLE, NTSTATUS, OBJECT_ATTRIBUTES, OBJ_CASE_INSENSITIVE, UNICODE_STRING};
use std::ptr::null_mut;

pub mod registry_key;
pub mod registry_value;

const ERROR_NO_MORE_ITEMS: NTSTATUS = 259;

/// Represents the buffer for a UNICODE_STRING.
pub struct UnicodeString {
    buffer: Vec<u16>,
}

impl UnicodeString {
    /// Create a new UnicodeString.
    /// 
    /// # Arguments
    ///
    /// * `string` - The string to create the UnicodeString from.
    pub fn new(string: String) -> UnicodeString {
        let mut buffer: Vec<u16> = string.encode_utf16().collect();
        buffer.push(0);
        UnicodeString { buffer }
    }

    /// Get the UNICODE_STRING representation of the buffer.
    pub fn as_struct(&self) -> UNICODE_STRING {
        UNICODE_STRING {
            Length: ((self.buffer.len() - 1) * 2) as u16,
            MaximumLength: (self.buffer.len() * 2) as u16,
            Buffer: self.buffer.as_ptr() as *mut u16,
        }
    }
}

trait AsUnicodeString {
    fn as_unicode_string(&self) -> UnicodeString;
}

impl AsUnicodeString for String {
    fn as_unicode_string(&self) -> UnicodeString {
        UnicodeString::new(self.to_string())
    }
}

pub struct ObjectAttributes {
    unicode_string: UnicodeString,
    root_directory: HANDLE,
}

impl ObjectAttributes {
    /// Create a new ObjectAttributes.
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

    /// Get the OBJECT_ATTRIBUTES representation of the struct.
    pub fn as_struct(&self) -> OBJECT_ATTRIBUTES {
        OBJECT_ATTRIBUTES {
            Length: size_of::<OBJECT_ATTRIBUTES>() as u32,
            RootDirectory: self.root_directory,
            ObjectName: &mut self.unicode_string.as_struct() as *mut UNICODE_STRING,
            Attributes: OBJ_CASE_INSENSITIVE,
            SecurityDescriptor: null_mut(),
            SecurityQualityOfService: null_mut(),
        }
    }
}
