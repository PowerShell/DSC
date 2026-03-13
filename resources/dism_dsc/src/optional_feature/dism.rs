// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use std::ffi::c_void;
use std::os::windows::ffi::OsStrExt;

use crate::optional_feature::types::{FeatureState, OptionalFeatureInfo, RestartType};

const DISM_ONLINE_IMAGE: &str = "DISM_{53BFAE52-B167-4E2F-A258-0A37B57FF845}";
const DISM_LOG_ERRORS: i32 = 0;
const DISM_PACKAGE_NONE: i32 = 0;

#[repr(C, packed)]
struct DismFeature {
    feature_name: *const u16,
    state: i32,
}

#[repr(C, packed)]
struct DismFeatureInfo {
    feature_name: *const u16,
    state: i32,
    display_name: *const u16,
    description: *const u16,
    restart_required: i32,
    custom_property: *const c_void,
    custom_property_count: u32,
}

// Function pointer types for the DISM API
type DismInitializeFn =
    unsafe extern "system" fn(i32, *const u16, *const u16) -> i32;
type DismOpenSessionFn =
    unsafe extern "system" fn(*const u16, *const u16, *const u16, *mut u32) -> i32;
type DismGetFeaturesFn =
    unsafe extern "system" fn(u32, *const u16, i32, *mut *mut DismFeature, *mut u32) -> i32;
type DismGetFeatureInfoFn =
    unsafe extern "system" fn(u32, *const u16, *const u16, i32, *mut *mut DismFeatureInfo) -> i32;
type DismEnableFeatureFn = unsafe extern "system" fn(
    u32,              // Session
    *const u16,       // FeatureName
    *const u16,       // Identifier (NULL)
    i32,              // PackageIdentifier (DismPackageNone)
    i32,              // LimitAccess (BOOL)
    *const *const u16,// SourcePaths (NULL)
    u32,              // SourcePathCount
    i32,              // EnableAll (BOOL)
    *mut c_void,      // CancelEvent (NULL)
    *mut c_void,      // Progress callback (NULL)
    *mut c_void,      // UserData (NULL)
) -> i32;
type DismDisableFeatureFn = unsafe extern "system" fn(
    u32,              // Session
    *const u16,       // FeatureName
    *const u16,       // PackageName (NULL)
    i32,              // RemovePayload (BOOL)
    *mut c_void,      // CancelEvent (NULL)
    *mut c_void,      // Progress callback (NULL)
    *mut c_void,      // UserData (NULL)
) -> i32;
type DismCloseSessionFn = unsafe extern "system" fn(u32) -> i32;
type DismShutdownFn = unsafe extern "system" fn() -> i32;
type DismDeleteFn = unsafe extern "system" fn(*const c_void) -> i32;

// Kernel32 functions for dynamic loading
extern "system" {
    fn LoadLibraryW(lp_lib_file_name: *const u16) -> *mut c_void;
    fn GetProcAddress(h_module: *mut c_void, lp_proc_name: *const u8) -> *mut c_void;
    fn FreeLibrary(h_lib_module: *mut c_void) -> i32;
}

fn to_wide_null(s: &str) -> Vec<u16> {
    std::ffi::OsStr::new(s)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect()
}

unsafe fn from_wide_ptr(ptr: *const u16) -> String {
    if ptr.is_null() {
        return String::new();
    }
    let len = (0..).take_while(|&i| *ptr.add(i) != 0).count();
    let slice = std::slice::from_raw_parts(ptr, len);
    String::from_utf16_lossy(slice)
}

unsafe fn load_fn<T>(lib: *mut c_void, name: &[u8]) -> Result<T, String> {
    let ptr = GetProcAddress(lib, name.as_ptr());
    if ptr.is_null() {
        return Err(format!(
            "Failed to find function '{}' in dismapi.dll",
            std::str::from_utf8(&name[..name.len() - 1]).unwrap_or("?")
        ));
    }
    Ok(std::mem::transmute_copy(&ptr))
}

struct DismApi {
    lib: *mut c_void,
    close_session: DismCloseSessionFn,
    shutdown: DismShutdownFn,
    get_features: DismGetFeaturesFn,
    get_feature_info: DismGetFeatureInfoFn,
    enable_feature: DismEnableFeatureFn,
    disable_feature: DismDisableFeatureFn,
    delete: DismDeleteFn,
}

impl DismApi {
    fn load() -> Result<Self, String> {
        let lib_name = to_wide_null("dismapi.dll");
        let lib = unsafe { LoadLibraryW(lib_name.as_ptr()) };
        if lib.is_null() {
            return Err("Failed to load dismapi.dll. Ensure DISM is available on this system.".to_string());
        }

        unsafe {
            Ok(DismApi {
                lib,
                close_session: load_fn(lib, b"DismCloseSession\0")?,
                shutdown: load_fn(lib, b"DismShutdown\0")?,
                get_features: load_fn(lib, b"DismGetFeatures\0")?,
                get_feature_info: load_fn(lib, b"DismGetFeatureInfo\0")?,
                enable_feature: load_fn(lib, b"DismEnableFeature\0")?,
                disable_feature: load_fn(lib, b"DismDisableFeature\0")?,
                delete: load_fn(lib, b"DismDelete\0")?,
            })
        }
    }
}

impl Drop for DismApi {
    fn drop(&mut self) {
        unsafe {
            FreeLibrary(self.lib);
        }
    }
}

pub struct DismSessionHandle {
    handle: u32,
    api: DismApi,
}

impl DismSessionHandle {
    pub fn open() -> Result<Self, String> {
        let api = DismApi::load()?;

        // Load DismInitialize and DismOpenSession (only needed during open)
        let dism_initialize: DismInitializeFn =
            unsafe { load_fn(api.lib, b"DismInitialize\0")? };
        let dism_open_session: DismOpenSessionFn =
            unsafe { load_fn(api.lib, b"DismOpenSession\0")? };

        unsafe {
            let hr = dism_initialize(DISM_LOG_ERRORS, std::ptr::null(), std::ptr::null());
            if hr < 0 {
                return Err(format!(
                    "DismInitialize failed: HRESULT 0x{:08X}",
                    hr as u32
                ));
            }

            let image_path = to_wide_null(DISM_ONLINE_IMAGE);
            let mut session: u32 = 0;
            let hr = dism_open_session(
                image_path.as_ptr(),
                std::ptr::null(),
                std::ptr::null(),
                &mut session,
            );
            if hr < 0 {
                (api.shutdown)();
                return Err(format!(
                    "DismOpenSession failed: HRESULT 0x{:08X}",
                    hr as u32
                ));
            }

            Ok(DismSessionHandle {
                handle: session,
                api,
            })
        }
    }

    pub fn get_feature_info(&self, feature_name: &str) -> Result<OptionalFeatureInfo, String> {
        let wide_name = to_wide_null(feature_name);
        let mut info_ptr: *mut DismFeatureInfo = std::ptr::null_mut();

        let hr = unsafe {
            (self.api.get_feature_info)(
                self.handle,
                wide_name.as_ptr(),
                std::ptr::null(),
                DISM_PACKAGE_NONE,
                &mut info_ptr,
            )
        };

        if hr < 0 {
            return Err(format!(
                "DismGetFeatureInfo failed for '{}': HRESULT 0x{:08X}",
                feature_name, hr as u32
            ));
        }

        let result = unsafe {
            let info = &*info_ptr;
            let feature_info = OptionalFeatureInfo {
                feature_name: Some(from_wide_ptr(info.feature_name)),
                state: FeatureState::from_dism(info.state),
                display_name: Some(from_wide_ptr(info.display_name)),
                description: Some(from_wide_ptr(info.description)),
                restart_required: RestartType::from_dism(info.restart_required),
            };
            (self.api.delete)(info_ptr as *const c_void);
            feature_info
        };

        Ok(result)
    }

    pub fn enable_feature(&self, feature_name: &str) -> Result<(), String> {
        let wide_name = to_wide_null(feature_name);
        let hr = unsafe {
            (self.api.enable_feature)(
                self.handle,
                wide_name.as_ptr(),
                std::ptr::null(),       // Identifier
                DISM_PACKAGE_NONE,      // PackageIdentifier
                0,                      // LimitAccess = FALSE
                std::ptr::null(),       // SourcePaths
                0,                      // SourcePathCount
                0,                      // EnableAll = FALSE
                std::ptr::null_mut(),   // CancelEvent
                std::ptr::null_mut(),   // Progress
                std::ptr::null_mut(),   // UserData
            )
        };
        if hr < 0 {
            return Err(format!(
                "DismEnableFeature failed for '{}': HRESULT 0x{:08X}",
                feature_name, hr as u32
            ));
        }
        Ok(())
    }

    pub fn disable_feature(&self, feature_name: &str, remove_payload: bool) -> Result<(), String> {
        let wide_name = to_wide_null(feature_name);
        let hr = unsafe {
            (self.api.disable_feature)(
                self.handle,
                wide_name.as_ptr(),
                std::ptr::null(),       // PackageName
                i32::from(remove_payload), // RemovePayload
                std::ptr::null_mut(),   // CancelEvent
                std::ptr::null_mut(),   // Progress
                std::ptr::null_mut(),   // UserData
            )
        };
        if hr < 0 {
            return Err(format!(
                "DismDisableFeature failed for '{}': HRESULT 0x{:08X}",
                feature_name, hr as u32
            ));
        }
        Ok(())
    }

    pub fn get_all_feature_basics(&self) -> Result<Vec<(String, i32)>, String> {
        let mut features_ptr: *mut DismFeature = std::ptr::null_mut();
        let mut count: u32 = 0;

        let hr = unsafe {
            (self.api.get_features)(
                self.handle,
                std::ptr::null(),
                DISM_PACKAGE_NONE,
                &mut features_ptr,
                &mut count,
            )
        };

        if hr < 0 {
            return Err(format!(
                "DismGetFeatures failed: HRESULT 0x{:08X}",
                hr as u32
            ));
        }

        let mut result = Vec::new();
        unsafe {
            for i in 0..count as usize {
                let feature = &*features_ptr.add(i);
                let name = from_wide_ptr(feature.feature_name);
                result.push((name, feature.state));
            }
            (self.api.delete)(features_ptr as *const c_void);
        }

        Ok(result)
    }
}

impl Drop for DismSessionHandle {
    fn drop(&mut self) {
        unsafe {
            (self.api.close_session)(self.handle);
            (self.api.shutdown)();
        }
    }
}
