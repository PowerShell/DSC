// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! Safe wrapper around the Windows Offline Registry Library (offreg.dll).
//!
//! This module dynamically loads offreg.dll and exposes safe Rust abstractions
//! for creating, opening, reading, writing, and deleting keys/values in offline
//! registry hive files.

use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::path::Path;
use std::ptr;

use rust_i18n::t;

use crate::error::RegistryError;

// Windows type aliases
type Dword = u32;
type Pvoid = *mut std::ffi::c_void;
type OrHkey = Pvoid;
type POrHkey = *mut OrHkey;
type Pcwstr = *const u16;
type PDword = *mut Dword;
type Hmodule = Pvoid;

const ERROR_SUCCESS: Dword = 0;
const ERROR_FILE_NOT_FOUND: Dword = 2;

// Registry value types
pub const REG_NONE: Dword = 0;
pub const REG_SZ: Dword = 1;
pub const REG_EXPAND_SZ: Dword = 2;
pub const REG_BINARY: Dword = 3;
pub const REG_DWORD: Dword = 4;
pub const REG_MULTI_SZ: Dword = 7;
pub const REG_QWORD: Dword = 11;

// Function pointer types for offreg.dll exports
type FnOrCreateHive = unsafe extern "system" fn(POrHkey) -> Dword;
type FnOrOpenHive = unsafe extern "system" fn(Pcwstr, POrHkey) -> Dword;
type FnOrSaveHive = unsafe extern "system" fn(OrHkey, Pcwstr, Dword, Dword) -> Dword;
type FnOrCloseHive = unsafe extern "system" fn(OrHkey) -> Dword;
type FnOrOpenKey = unsafe extern "system" fn(OrHkey, Pcwstr, POrHkey) -> Dword;
type FnOrCreateKey = unsafe extern "system" fn(OrHkey, Pcwstr, Pcwstr, Dword, Pvoid, POrHkey, PDword) -> Dword;
type FnOrCloseKey = unsafe extern "system" fn(OrHkey) -> Dword;
type FnOrGetValue = unsafe extern "system" fn(OrHkey, Pcwstr, Pcwstr, PDword, Pvoid, PDword) -> Dword;
type FnOrSetValue = unsafe extern "system" fn(OrHkey, Pcwstr, Dword, *const u8, Dword) -> Dword;
type FnOrDeleteValue = unsafe extern "system" fn(OrHkey, Pcwstr) -> Dword;
type FnOrDeleteKey = unsafe extern "system" fn(OrHkey, Pcwstr) -> Dword;

unsafe extern "system" {
    fn LoadLibraryW(lpFileName: Pcwstr) -> Hmodule;
    fn GetProcAddress(Hmodule: Hmodule, lpProcName: *const u8) -> Pvoid;
    fn FreeLibrary(Hmodule: Hmodule) -> i32;
}

/// Holds function pointers to offreg.dll exports.
struct OffRegLib {
    _module: Hmodule,
    or_create_hive: FnOrCreateHive,
    or_open_hive: FnOrOpenHive,
    or_save_hive: FnOrSaveHive,
    or_close_hive: FnOrCloseHive,
    or_open_key: FnOrOpenKey,
    or_create_key: FnOrCreateKey,
    or_close_key: FnOrCloseKey,
    or_get_value: FnOrGetValue,
    or_set_value: FnOrSetValue,
    or_delete_value: FnOrDeleteValue,
    or_delete_key: FnOrDeleteKey,
}

impl OffRegLib {
    fn load() -> Result<Self, RegistryError> {
        let dll_name: Vec<u16> = OsStr::new("offreg.dll").encode_wide().chain(std::iter::once(0)).collect();
        let module = unsafe { LoadLibraryW(dll_name.as_ptr()) };
        if module.is_null() {
            return Err(RegistryError::OfflineRegistry(t!("offreg.loadFailed").to_string()));
        }

        macro_rules! load_fn {
            ($module:expr, $name:expr, $ty:ty) => {{
                let name = concat!($name, "\0");
                let ptr = unsafe { GetProcAddress($module, name.as_ptr()) };
                if ptr.is_null() {
                    unsafe { FreeLibrary($module); }
                    return Err(RegistryError::OfflineRegistry(
                        t!("offreg.procNotFound", name = $name).to_string()
                    ));
                }
                unsafe { std::mem::transmute::<Pvoid, $ty>(ptr) }
            }};
        }

        Ok(Self {
            _module: module,
            or_create_hive: load_fn!(module, "ORCreateHive", FnOrCreateHive),
            or_open_hive: load_fn!(module, "OROpenHive", FnOrOpenHive),
            or_save_hive: load_fn!(module, "ORSaveHive", FnOrSaveHive),
            or_close_hive: load_fn!(module, "ORCloseHive", FnOrCloseHive),
            or_open_key: load_fn!(module, "OROpenKey", FnOrOpenKey),
            or_create_key: load_fn!(module, "ORCreateKey", FnOrCreateKey),
            or_close_key: load_fn!(module, "ORCloseKey", FnOrCloseKey),
            or_get_value: load_fn!(module, "ORGetValue", FnOrGetValue),
            or_set_value: load_fn!(module, "ORSetValue", FnOrSetValue),
            or_delete_value: load_fn!(module, "ORDeleteValue", FnOrDeleteValue),
            or_delete_key: load_fn!(module, "ORDeleteKey", FnOrDeleteKey),
        })
    }
}

/// Represents an open offline registry hive.
pub struct OfflineHive {
    lib: OffRegLib,
    handle: OrHkey,
    path: String,
}

impl OfflineHive {
    /// Open an existing offline registry hive file.
    ///
    /// # Errors
    ///
    /// Returns `RegistryError::OfflineRegistry` if the hive cannot be opened.
    pub fn open(path: &Path) -> Result<Self, RegistryError> {
        let lib = OffRegLib::load()?;
        let wide_path: Vec<u16> = path.as_os_str().encode_wide().chain(std::iter::once(0)).collect();
        let mut handle: OrHkey = ptr::null_mut();
        let result = unsafe { (lib.or_open_hive)(wide_path.as_ptr(), &mut handle) };
        if result != ERROR_SUCCESS {
            return Err(RegistryError::OfflineRegistry(
                t!("offreg.openHiveFailed", code = result, path = path.display().to_string()).to_string()
            ));
        }
        Ok(Self { lib, handle, path: path.to_string_lossy().to_string() })
    }

    /// Create a new empty offline registry hive.
    ///
    /// # Errors
    ///
    /// Returns `RegistryError::OfflineRegistry` if the hive cannot be created.
    pub fn create() -> Result<Self, RegistryError> {
        let lib = OffRegLib::load()?;
        let mut handle: OrHkey = ptr::null_mut();
        let result = unsafe { (lib.or_create_hive)(&mut handle) };
        if result != ERROR_SUCCESS {
            return Err(RegistryError::OfflineRegistry(
                t!("offreg.createHiveFailed", code = result).to_string()
            ));
        }
        Ok(Self { lib, handle, path: String::new() })
    }

    /// Save the offline registry hive to a file.
    /// If the file already exists, it will be replaced.
    ///
    /// # Errors
    ///
    /// Returns `RegistryError::OfflineRegistry` if the hive cannot be saved.
    pub fn save(&self, path: &Path) -> Result<(), RegistryError> {
        // ORSaveHive cannot overwrite an existing file, so remove it first
        if path.exists()
            && let Err(e) = std::fs::remove_file(path)
        {
            return Err(RegistryError::OfflineRegistry(
                t!("offreg.saveHiveRemoveFailed", error = e.to_string()).to_string()
            ));
        }
        let wide_path: Vec<u16> = path.as_os_str().encode_wide().chain(std::iter::once(0)).collect();
        // Use OS version 6.2 (Windows 8/Server 2012) for broad compatibility
        let result = unsafe { (self.lib.or_save_hive)(self.handle, wide_path.as_ptr(), 6, 2) };
        if result != ERROR_SUCCESS {
            return Err(RegistryError::OfflineRegistry(
                t!("offreg.saveHiveFailed", code = result, path = path.display().to_string()).to_string()
            ));
        }
        Ok(())
    }

    /// Open a subkey within the hive.
    ///
    /// # Errors
    ///
    /// Returns `RegistryError::RegistryKeyNotFound` if the key does not exist.
    pub fn open_key(&self, subkey: &str) -> Result<OfflineKey, RegistryError> {
        if subkey.is_empty() {
            return Ok(OfflineKey { handle: self.handle, owned: false });
        }
        let wide_subkey: Vec<u16> = OsStr::new(subkey).encode_wide().chain(std::iter::once(0)).collect();
        let mut key_handle: OrHkey = ptr::null_mut();
        let result = unsafe { (self.lib.or_open_key)(self.handle, wide_subkey.as_ptr(), &mut key_handle) };
        if result == ERROR_FILE_NOT_FOUND {
            return Err(RegistryError::RegistryKeyNotFound(subkey.to_string()));
        }
        if result != ERROR_SUCCESS {
            return Err(RegistryError::OfflineRegistry(
                t!("offreg.openKeyFailed", code = result, key = subkey).to_string()
            ));
        }
        Ok(OfflineKey { handle: key_handle, owned: true })
    }

    /// Create a subkey within the hive (creates intermediate keys as needed).
    ///
    /// # Errors
    ///
    /// Returns `RegistryError::OfflineRegistry` on failure.
    pub fn create_key(&self, subkey: &str) -> Result<OfflineKey, RegistryError> {
        if subkey.is_empty() {
            return Ok(OfflineKey { handle: self.handle, owned: false });
        }

        // ORCreateKey may not support creating deeply nested keys in one call,
        // so we create each level one at a time.
        let parts: Vec<&str> = subkey.split('\\').collect();
        let mut current_handle = self.handle;
        let mut owned_handles: Vec<OrHkey> = Vec::new();

        for (i, _part) in parts.iter().enumerate() {
            let partial_path: String = parts[..=i].join("\\");
            let wide_subkey: Vec<u16> = OsStr::new(&partial_path).encode_wide().chain(std::iter::once(0)).collect();
            let mut key_handle: OrHkey = ptr::null_mut();
            let mut disposition: Dword = 0;
            let result = unsafe {
                (self.lib.or_create_key)(
                    self.handle,
                    wide_subkey.as_ptr(),
                    ptr::null(),
                    0,
                    ptr::null_mut(),
                    &mut key_handle,
                    &mut disposition,
                )
            };
            if result != ERROR_SUCCESS {
                // Close any intermediate handles we opened
                for h in owned_handles {
                    unsafe { (self.lib.or_close_key)(h); }
                }
                return Err(RegistryError::OfflineRegistry(
                    t!("offreg.createKeyFailed", code = result, key = partial_path).to_string()
                ));
            }
            // Close previous intermediate handle (not the root)
            if current_handle != self.handle {
                // The intermediate handles are tracked for cleanup on error
            }
            current_handle = key_handle;
            owned_handles.push(key_handle);
        }

        // Close all intermediate handles except the last one
        for h in &owned_handles[..owned_handles.len().saturating_sub(1)] {
            unsafe { (self.lib.or_close_key)(*h); }
        }

        Ok(OfflineKey { handle: current_handle, owned: true })
    }

    /// Get a value from the hive root or a subkey.
    ///
    /// # Errors
    ///
    /// Returns error if the value cannot be read.
    pub fn get_value(&self, subkey: &str, value_name: &str) -> Result<Option<(u32, Vec<u8>)>, RegistryError> {
        let wide_subkey: Vec<u16> = if subkey.is_empty() {
            vec![0]
        } else {
            OsStr::new(subkey).encode_wide().chain(std::iter::once(0)).collect()
        };
        let wide_value: Vec<u16> = OsStr::new(value_name).encode_wide().chain(std::iter::once(0)).collect();

        // First call to get the size
        let mut value_type: Dword = 0;
        let mut data_size: Dword = 0;
        let result = unsafe {
            (self.lib.or_get_value)(
                self.handle,
                wide_subkey.as_ptr(),
                wide_value.as_ptr(),
                &mut value_type,
                ptr::null_mut(),
                &mut data_size,
            )
        };

        if result == ERROR_FILE_NOT_FOUND {
            return Ok(None);
        }
        if result != ERROR_SUCCESS && result != 234 /* ERROR_MORE_DATA */ {
            return Err(RegistryError::OfflineRegistry(
                t!("offreg.getValueSizeFailed", code = result, value = value_name).to_string()
            ));
        }

        if data_size == 0 {
            return Ok(Some((value_type, Vec::new())));
        }

        // Second call to get the data
        let mut data = vec![0u8; data_size as usize];
        let result = unsafe {
            (self.lib.or_get_value)(
                self.handle,
                wide_subkey.as_ptr(),
                wide_value.as_ptr(),
                &mut value_type,
                data.as_mut_ptr().cast(),
                &mut data_size,
            )
        };

        if result != ERROR_SUCCESS {
            return Err(RegistryError::OfflineRegistry(
                t!("offreg.getValueFailed", code = result, value = value_name).to_string()
            ));
        }

        data.truncate(data_size as usize);
        Ok(Some((value_type, data)))
    }

    /// Set a value in the hive at the given key handle.
    ///
    /// # Errors
    ///
    /// Returns error if the value cannot be written.
    pub fn set_value(&self, key: &OfflineKey, value_name: &str, value_type: u32, data: &[u8]) -> Result<(), RegistryError> {
        let wide_value: Vec<u16> = OsStr::new(value_name).encode_wide().chain(std::iter::once(0)).collect();
        let result = unsafe {
            (self.lib.or_set_value)(
                key.handle,
                wide_value.as_ptr(),
                value_type,
                data.as_ptr(),
                data.len() as Dword,
            )
        };
        if result != ERROR_SUCCESS {
            return Err(RegistryError::OfflineRegistry(
                t!("offreg.setValueFailed", code = result, value = value_name).to_string()
            ));
        }
        Ok(())
    }

    /// Delete a value from the given key.
    ///
    /// # Errors
    ///
    /// Returns error if the value cannot be deleted.
    pub fn delete_value(&self, key: &OfflineKey, value_name: &str) -> Result<(), RegistryError> {
        let wide_value: Vec<u16> = OsStr::new(value_name).encode_wide().chain(std::iter::once(0)).collect();
        let result = unsafe { (self.lib.or_delete_value)(key.handle, wide_value.as_ptr()) };
        if result == ERROR_FILE_NOT_FOUND {
            return Ok(());
        }
        if result != ERROR_SUCCESS {
            return Err(RegistryError::OfflineRegistry(
                t!("offreg.deleteValueFailed", code = result, value = value_name).to_string()
            ));
        }
        Ok(())
    }

    /// Delete a subkey from the hive.
    ///
    /// # Errors
    ///
    /// Returns error if the key cannot be deleted.
    pub fn delete_key(&self, parent: &OfflineKey, subkey_name: &str) -> Result<(), RegistryError> {
        let wide_subkey: Vec<u16> = OsStr::new(subkey_name).encode_wide().chain(std::iter::once(0)).collect();
        let result = unsafe { (self.lib.or_delete_key)(parent.handle, wide_subkey.as_ptr()) };
        if result == ERROR_FILE_NOT_FOUND {
            return Ok(());
        }
        if result != ERROR_SUCCESS {
            return Err(RegistryError::OfflineRegistry(
                t!("offreg.deleteKeyFailed", code = result, key = subkey_name).to_string()
            ));
        }
        Ok(())
    }

    /// Close a key handle that was opened with `open_key` or `create_key`.
    pub fn close_key(&self, key: OfflineKey) {
        if key.owned && !key.handle.is_null() {
            unsafe { (self.lib.or_close_key)(key.handle); }
        }
    }

    /// Get the file path this hive was opened from.
    #[must_use]
    pub fn path(&self) -> &str {
        &self.path
    }
}

impl Drop for OfflineHive {
    fn drop(&mut self) {
        if !self.handle.is_null() {
            unsafe { (self.lib.or_close_hive)(self.handle); }
        }
    }
}

/// Represents an open key within an offline hive.
pub struct OfflineKey {
    handle: OrHkey,
    owned: bool,
}

impl OfflineKey {
    /// Get the raw handle (for use with `set_value`, etc.)
    #[must_use]
    pub fn is_root(&self) -> bool {
        !self.owned
    }
}
