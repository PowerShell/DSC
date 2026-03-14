// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use std::ffi::c_void;
use std::os::windows::ffi::OsStrExt;

use rust_i18n::t;

use crate::optional_feature::types::{FeatureState, OptionalFeatureInfo, RestartType};

const DISM_ONLINE_IMAGE: &str = "DISM_{53BFAE52-B167-4E2F-A258-0A37B57FF845}";
const DISM_LOG_ERRORS: i32 = 0;
const DISM_PACKAGE_NONE: i32 = 0;
const ERROR_SUCCESS_REBOOT_REQUIRED: i32 = 3010;
const DISMAPI_E_UNKNOWN_FEATURE: i32 = 0x800F080Cu32 as i32;
const LOAD_LIBRARY_SEARCH_SYSTEM32: u32 = 0x0000_0800;

#[link(name = "kernel32")]
unsafe extern "system" {
    fn LoadLibraryExW(
        lpLibFileName: *const u16,
        hFile: *mut c_void,
        dwFlags: u32,
    ) -> *mut c_void;
}

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

#[repr(C, packed)]
struct DismCapability {
    name: *const u16,
    state: i32,
}

#[repr(C, packed)]
struct DismCapabilityDetail {
    name: *const u16,
    state: i32,
    display_name: *const u16,
    description: *const u16,
    download_size: u32,
    install_size: u32,
}

/// Raw capability information returned from the DISM API.
pub struct DismCapabilityResult {
    pub name: String,
    pub unknown: bool,
    pub state: i32,
    pub display_name: String,
    pub description: String,
    pub download_size: u32,
    pub install_size: u32,
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
type DismGetCapabilitiesFn =
    unsafe extern "system" fn(u32, *mut *mut DismCapability, *mut u32) -> i32;
type DismGetCapabilityInfoFn =
    unsafe extern "system" fn(u32, *const u16, *mut *mut DismCapabilityDetail) -> i32;
type DismAddCapabilityFn = unsafe extern "system" fn(
    u32,              // Session
    *const u16,       // Name
    i32,              // LimitAccess (BOOL)
    *const *const u16,// SourcePaths (NULL)
    u32,              // SourcePathCount
    *mut c_void,      // CancelEvent (NULL)
    *mut c_void,      // Progress callback (NULL)
    *mut c_void,      // UserData (NULL)
) -> i32;
type DismRemoveCapabilityFn = unsafe extern "system" fn(
    u32,              // Session
    *const u16,       // Name
    *mut c_void,      // CancelEvent (NULL)
    *mut c_void,      // Progress callback (NULL)
    *mut c_void,      // UserData (NULL)
) -> i32;
type DismCloseSessionFn = unsafe extern "system" fn(u32) -> i32;
type DismShutdownFn = unsafe extern "system" fn() -> i32;
type DismDeleteFn = unsafe extern "system" fn(*const c_void) -> i32;

// Kernel32 functions for dynamic loading
unsafe extern "system" {
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
    unsafe {
        let len = (0..65536).take_while(|&i| *ptr.add(i) != 0).count();
        let slice = std::slice::from_raw_parts(ptr, len);
        String::from_utf16_lossy(slice)
    }
}

unsafe fn load_fn<T>(lib: *mut c_void, name: &[u8]) -> Result<T, String> {
    unsafe {
        let ptr = GetProcAddress(lib, name.as_ptr());
        if ptr.is_null() {
            let fn_name = std::str::from_utf8(&name[..name.len() - 1]).unwrap_or("?");
            return Err(t!("dism.functionNotFound", name = fn_name).to_string());
        }
        Ok(std::mem::transmute_copy(&ptr))
    }
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
    get_capabilities: Option<DismGetCapabilitiesFn>,
    get_capability_info: Option<DismGetCapabilityInfoFn>,
    add_capability: Option<DismAddCapabilityFn>,
    remove_capability: Option<DismRemoveCapabilityFn>,
}

impl DismApi {
    fn load() -> Result<Self, String> {
        // Load dismapi.dll from the trusted System32 directory to avoid DLL search order hijacking.
        // Use LoadLibraryExW with LOAD_LIBRARY_SEARCH_SYSTEM32 so the DLL location cannot be
        // redirected via environment variables or the default DLL search order.
        let lib_name = to_wide_null("dismapi.dll");
        let lib = unsafe {
            LoadLibraryExW(
                lib_name.as_ptr(),
                std::ptr::null_mut(),
                LOAD_LIBRARY_SEARCH_SYSTEM32,
            )
        };
        if lib.is_null() {
            return Err(t!("dism.failedLoadLibrary").to_string());
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
                get_capabilities: load_fn(lib, b"DismGetCapabilities\0").ok(),
                get_capability_info: load_fn(lib, b"DismGetCapabilityInfo\0").ok(),
                add_capability: load_fn(lib, b"DismAddCapability\0").ok(),
                remove_capability: load_fn(lib, b"DismRemoveCapability\0").ok(),
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
    /// Opens a new DISM session for the online image.
    ///
    /// NOTE: `DismInitialize` and `DismShutdown` are per-process globals.
    /// Only one `DismSessionHandle` should exist at a time. Creating a
    /// second session while one is already open (or after one has been
    /// dropped) will call `DismInitialize` again, which returns
    /// `DISMAPI_E_DISMAPI_ALREADY_INITIALIZED`.
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
                return Err(t!("dism.initializeFailed", hr = format!("0x{:08X}", hr as u32)).to_string());
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
                return Err(t!("dism.openSessionFailed", hr = format!("0x{:08X}", hr as u32)).to_string());
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

        if hr == DISMAPI_E_UNKNOWN_FEATURE {
            return Ok(OptionalFeatureInfo {
                feature_name: Some(feature_name.to_string()),
                exist: Some(false),
                ..OptionalFeatureInfo::default()
            });
        }

        if hr < 0 {
            return Err(t!("dism.getFeatureInfoFailed", name = feature_name, hr = format!("0x{:08X}", hr as u32)).to_string());
        }

        let result = unsafe {
            let info = &*info_ptr;
            let feature_info = OptionalFeatureInfo {
                feature_name: Some(from_wide_ptr(info.feature_name)),
                exist: None,
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

    /// Returns `Ok(true)` if DISM reports a reboot is required (HRESULT 3010).
    pub fn enable_feature(&self, feature_name: &str) -> Result<bool, String> {
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
            return Err(t!("dism.enableFeatureFailed", name = feature_name, hr = format!("0x{:08X}", hr as u32)).to_string());
        }
        Ok(hr == ERROR_SUCCESS_REBOOT_REQUIRED)
    }

    /// Returns `Ok(true)` if DISM reports a reboot is required (HRESULT 3010).
    pub fn disable_feature(&self, feature_name: &str, remove_payload: bool) -> Result<bool, String> {
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
            return Err(t!("dism.disableFeatureFailed", name = feature_name, hr = format!("0x{:08X}", hr as u32)).to_string());
        }
        Ok(hr == ERROR_SUCCESS_REBOOT_REQUIRED)
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
            return Err(t!("dism.getFeaturesFailed", hr = format!("0x{:08X}", hr as u32)).to_string());
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

    pub fn get_capability_info(&self, name: &str) -> Result<DismCapabilityResult, String> {
        let get_cap_info = self.api.get_capability_info
            .ok_or_else(|| t!("dism.capabilitiesNotSupported").to_string())?;

        let wide_name = to_wide_null(name);
        let mut info_ptr: *mut DismCapabilityDetail = std::ptr::null_mut();

        let hr = unsafe {
            get_cap_info(
                self.handle,
                wide_name.as_ptr(),
                &mut info_ptr,
            )
        };

        if hr == DISMAPI_E_UNKNOWN_FEATURE {
            return Ok(DismCapabilityResult {
                name: name.to_string(),
                unknown: true,
                state: 0,
                display_name: String::new(),
                description: String::new(),
                download_size: 0,
                install_size: 0,
            });
        }

        if hr < 0 {
            return Err(t!("dism.getCapabilityInfoFailed", name = name, hr = format!("0x{:08X}", hr as u32)).to_string());
        }

        let result = unsafe {
            let detail = &*info_ptr;
            let cap_info = DismCapabilityResult {
                name: from_wide_ptr(detail.name),
                unknown: false,
                state: detail.state,
                display_name: from_wide_ptr(detail.display_name),
                description: from_wide_ptr(detail.description),
                download_size: detail.download_size,
                install_size: detail.install_size,
            };
            (self.api.delete)(info_ptr as *const c_void);
            cap_info
        };

        Ok(result)
    }

    /// Returns `Ok(true)` if DISM reports a reboot is required (HRESULT 3010).
    pub fn add_capability(&self, name: &str) -> Result<bool, String> {
        let add_cap = self.api.add_capability
            .ok_or_else(|| t!("dism.capabilitiesNotSupported").to_string())?;

        let wide_name = to_wide_null(name);
        let hr = unsafe {
            add_cap(
                self.handle,
                wide_name.as_ptr(),
                0,                      // LimitAccess = FALSE
                std::ptr::null(),       // SourcePaths
                0,                      // SourcePathCount
                std::ptr::null_mut(),   // CancelEvent
                std::ptr::null_mut(),   // Progress
                std::ptr::null_mut(),   // UserData
            )
        };
        if hr < 0 {
            return Err(t!("dism.addCapabilityFailed", name = name, hr = format!("0x{:08X}", hr as u32)).to_string());
        }
        Ok(hr == ERROR_SUCCESS_REBOOT_REQUIRED)
    }

    /// Returns `Ok(true)` if DISM reports a reboot is required (HRESULT 3010).
    pub fn remove_capability(&self, name: &str) -> Result<bool, String> {
        let remove_cap = self.api.remove_capability
            .ok_or_else(|| t!("dism.capabilitiesNotSupported").to_string())?;

        let wide_name = to_wide_null(name);
        let hr = unsafe {
            remove_cap(
                self.handle,
                wide_name.as_ptr(),
                std::ptr::null_mut(),   // CancelEvent
                std::ptr::null_mut(),   // Progress
                std::ptr::null_mut(),   // UserData
            )
        };
        if hr < 0 {
            return Err(t!("dism.removeCapabilityFailed", name = name, hr = format!("0x{:08X}", hr as u32)).to_string());
        }
        Ok(hr == ERROR_SUCCESS_REBOOT_REQUIRED)
    }

    pub fn get_all_capability_basics(&self) -> Result<Vec<(String, i32)>, String> {
        let get_caps = self.api.get_capabilities
            .ok_or_else(|| t!("dism.capabilitiesNotSupported").to_string())?;

        let mut caps_ptr: *mut DismCapability = std::ptr::null_mut();
        let mut count: u32 = 0;

        let hr = unsafe {
            get_caps(
                self.handle,
                &mut caps_ptr,
                &mut count,
            )
        };

        if hr < 0 {
            return Err(t!("dism.getCapabilitiesFailed", hr = format!("0x{:08X}", hr as u32)).to_string());
        }

        let mut result = Vec::new();
        unsafe {
            for i in 0..count as usize {
                let cap = &*caps_ptr.add(i);
                let name = from_wide_ptr(cap.name);
                result.push((name, cap.state));
            }
            (self.api.delete)(caps_ptr as *const c_void);
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
