// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use rust_i18n::t;
use std::mem;
use windows::core::{PCWSTR, PWSTR};
use windows::Win32::Foundation::{ERROR_INSUFFICIENT_BUFFER, ERROR_MORE_DATA, ERROR_SERVICE_DOES_NOT_EXIST};
use windows::Win32::System::Services::*;

const SERVICE_NO_CHANGE_VALUE: u32 = 0xFFFF_FFFF;
const STATUS_WAIT_TIMEOUT_SECS: u64 = 30;
const STATUS_POLL_INTERVAL_MS: u64 = 250;

use crate::types::*;

/// Encode a Rust string as a null-terminated UTF-16 buffer.
fn to_wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

/// Convert a `PWSTR` to an `Option<String>`.
///
/// # Safety
///
/// The pointer must be valid or null.
unsafe fn pwstr_to_string(p: PWSTR) -> Option<String> {
    if p.is_null() {
        None
    } else {
        unsafe { p.to_string().ok() }
    }
}

/// Parse a double-null-terminated multi-string into individual strings.
///
/// # Safety
///
/// The pointer must point to a valid double-null-terminated buffer, or be null.
unsafe fn parse_multi_string(ptr: PWSTR) -> Vec<String> {
    if ptr.is_null() {
        return Vec::new();
    }
    let mut result = Vec::new();
    let mut current = ptr.0;
    unsafe {
        loop {
            if *current == 0 {
                break;
            }
            let pcwstr = PCWSTR(current);
            match pcwstr.to_string() {
                Ok(s) => {
                    current = current.add(s.encode_utf16().count() + 1);
                    result.push(s);
                }
                Err(_) => break,
            }
        }
    }
    result
}

/// RAII wrapper for `SC_HANDLE` that calls `CloseServiceHandle` on drop.
struct ScHandle(SC_HANDLE);

impl Drop for ScHandle {
    fn drop(&mut self) {
        unsafe {
            let _ = CloseServiceHandle(self.0);
        }
    }
}

impl From<windows::core::Error> for ServiceError {
    fn from(e: windows::core::Error) -> Self {
        Self { message: e.to_string() }
    }
}

/// Read the full configuration and status of an already-opened service handle.
///
/// # Safety
///
/// `service_handle` must be a valid, open service handle with `SERVICE_QUERY_CONFIG`
/// and `SERVICE_QUERY_STATUS` access rights.
unsafe fn read_service_state(
    service_handle: SC_HANDLE,
    service_name: &str,
) -> Result<WindowsService, ServiceError> {
    // Query basic configuration
    let mut bytes_needed: u32 = 0;
    let sizing_result = unsafe {
        QueryServiceConfigW(service_handle, None, 0, &mut bytes_needed)
    };
    if let Err(e) = sizing_result {
        if e.code() != ERROR_INSUFFICIENT_BUFFER.to_hresult() {
            return Err(t!("get.queryConfigFailed", error = e.to_string()).to_string().into());
        }
    }
    if bytes_needed == 0 {
        return Err(t!("get.queryConfigFailed", error = "buffer size is 0").to_string().into());
    }
    let mut config_buffer = vec![0u8; bytes_needed as usize];
    let config_ptr = config_buffer.as_mut_ptr().cast::<QUERY_SERVICE_CONFIGW>();
    unsafe {
        QueryServiceConfigW(
            service_handle,
            Some(&mut *config_ptr),
            bytes_needed,
            &mut bytes_needed,
        )
        .map_err(|e| ServiceError::from(t!("get.queryConfigFailed", error = e.to_string()).to_string()))?;
    }

    let config = unsafe { &*config_ptr };
    let display_name = unsafe { pwstr_to_string(config.lpDisplayName) };

    let start_type = match config.dwStartType {
        SERVICE_AUTO_START => {
            if unsafe { is_delayed_auto_start(service_handle) } {
                Some(StartType::AutomaticDelayedStart)
            } else {
                Some(StartType::Automatic)
            }
        }
        SERVICE_DEMAND_START => Some(StartType::Manual),
        SERVICE_DISABLED => Some(StartType::Disabled),
        _ => None,
    };

    let error_control = match config.dwErrorControl {
        SERVICE_ERROR_IGNORE => Some(ErrorControl::Ignore),
        SERVICE_ERROR_NORMAL => Some(ErrorControl::Normal),
        SERVICE_ERROR_SEVERE => Some(ErrorControl::Severe),
        SERVICE_ERROR_CRITICAL => Some(ErrorControl::Critical),
        _ => None,
    };

    let executable_path = unsafe { pwstr_to_string(config.lpBinaryPathName) };
    let logon_account = unsafe { pwstr_to_string(config.lpServiceStartName) };
    let deps = unsafe { parse_multi_string(config.lpDependencies) };
    let dependencies = if deps.is_empty() { None } else { Some(deps) };

    let description = unsafe { query_description(service_handle) };
    let status = unsafe { query_status(service_handle) }?;

    Ok(WindowsService {
        name: Some(service_name.to_string()),
        display_name,
        description,
        exist: Some(true),
        status: Some(status),
        start_type,
        executable_path,
        logon_account,
        error_control,
        dependencies,
    })
}

/// Look up a service by `name` and/or `display_name` and return the full service info.
///
/// - If only `name` is provided, the service is looked up by name.
/// - If only `display_name` is provided, the service key name is resolved first.
/// - If both are provided, the service is looked up by name and the display name is verified.
///
/// If the service does not exist, returns a `WindowsService` with `_exist: false`.
pub fn get_service(input: &WindowsService) -> Result<WindowsService, ServiceError> {
    if input.name.is_none() && input.display_name.is_none() {
        return Err(t!("get.nameOrDisplayNameRequired").to_string().into());
    }

    // Open Service Control Manager
    let scm = unsafe { OpenSCManagerW(None, None, SC_MANAGER_CONNECT) }
        .map_err(|e| ServiceError::from(t!("get.openScmFailed", error = e.to_string()).to_string()))?;
    let scm = ScHandle(scm);

    // Resolve the service key name
    let service_key_name = match &input.name {
        Some(name) => Some(name.clone()),
        None => {
            let display_name = input.display_name.as_ref().unwrap();
            unsafe { resolve_key_name_from_display_name(scm.0, display_name) }?
        }
    };

    // If we couldn't resolve a key name, the service doesn't exist
    let service_key_name = match service_key_name {
        Some(n) => n,
        None => {
            return Ok(WindowsService {
                name: input.name.clone(),
                display_name: input.display_name.clone(),
                exist: Some(false),
                ..Default::default()
            });
        }
    };

    // Open the service
    let name_wide = to_wide(&service_key_name);
    let service_handle = match unsafe {
        OpenServiceW(
            scm.0,
            PCWSTR(name_wide.as_ptr()),
            SERVICE_QUERY_CONFIG | SERVICE_QUERY_STATUS,
        )
    } {
        Ok(h) => ScHandle(h),
        Err(e) if e.code() == ERROR_SERVICE_DOES_NOT_EXIST.to_hresult() => {
            return Ok(WindowsService {
                name: input.name.clone(),
                display_name: input.display_name.clone(),
                exist: Some(false),
                ..Default::default()
            });
        }
        Err(e) => {
            return Err(ServiceError::from(t!("get.openServiceFailed", error = e.to_string()).to_string()));
        }
    };

    let svc = unsafe { read_service_state(service_handle.0, &service_key_name) }?;

    // If both name and display_name were provided, verify they match
    if input.name.is_some() && input.display_name.is_some() {
        let expected_dn = input.display_name.as_ref().unwrap();
        let actual_dn = svc.display_name.as_deref().unwrap_or("");
        if !actual_dn.eq_ignore_ascii_case(expected_dn) {
            return Err(
                t!("get.displayNameMismatch", expected = expected_dn, actual = actual_dn)
                    .to_string()
                    .into(),
            );
        }
    }

    Ok(svc)
}

/// Resolve a service key name from its display name via SCM.
unsafe fn resolve_key_name_from_display_name(
    scm: SC_HANDLE,
    display_name: &str,
) -> Result<Option<String>, ServiceError> {
    let dn_wide = to_wide(display_name);
    let mut size: u32 = 0;

    // First call to determine the required buffer size
    let sizing_result = unsafe {
        GetServiceKeyNameW(scm, PCWSTR(dn_wide.as_ptr()), None, &mut size)
    };

    if let Err(e) = sizing_result {
        if e.code() == ERROR_SERVICE_DOES_NOT_EXIST.to_hresult() {
            return Ok(None);
        }
        if size == 0 {
            return Err(
                t!("get.getKeyNameFailed", error = e.to_string()).to_string().into(),
            );
        }
    }

    size += 1; // null terminator
    let mut buffer = vec![0u16; size as usize];

    unsafe {
        GetServiceKeyNameW(
            scm,
            PCWSTR(dn_wide.as_ptr()),
            Some(PWSTR(buffer.as_mut_ptr())),
            &mut size,
        )
        .map_err(|e| t!("get.getKeyNameFailed", error = e.to_string()).to_string())?;
    }

    Ok(Some(String::from_utf16_lossy(&buffer[..size as usize])))
}

/// Check whether the service is configured for delayed automatic start.
unsafe fn is_delayed_auto_start(service_handle: SC_HANDLE) -> bool {
    let mut bytes_needed: u32 = 0;
    let _ = unsafe {
        QueryServiceConfig2W(
            service_handle,
            SERVICE_CONFIG_DELAYED_AUTO_START_INFO,
            None,
            &mut bytes_needed,
        )
    };

    if bytes_needed == 0 {
        return false;
    }

    let mut buffer = vec![0u8; bytes_needed as usize];
    if unsafe {
        QueryServiceConfig2W(
            service_handle,
            SERVICE_CONFIG_DELAYED_AUTO_START_INFO,
            Some(&mut buffer),
            &mut bytes_needed,
        )
    }
    .is_ok()
    {
        let info =
            unsafe { &*(buffer.as_ptr().cast::<SERVICE_DELAYED_AUTO_START_INFO>()) };
        info.fDelayedAutostart.as_bool()
    } else {
        false
    }
}

/// Query the service description string.
unsafe fn query_description(service_handle: SC_HANDLE) -> Option<String> {
    let mut bytes_needed: u32 = 0;
    let _ = unsafe {
        QueryServiceConfig2W(
            service_handle,
            SERVICE_CONFIG_DESCRIPTION,
            None,
            &mut bytes_needed,
        )
    };

    if bytes_needed == 0 {
        return None;
    }

    let mut buffer = vec![0u8; bytes_needed as usize];
    if unsafe {
        QueryServiceConfig2W(
            service_handle,
            SERVICE_CONFIG_DESCRIPTION,
            Some(&mut buffer),
            &mut bytes_needed,
        )
    }
    .is_ok()
    {
        let desc = unsafe { &*(buffer.as_ptr().cast::<SERVICE_DESCRIPTIONW>()) };
        if desc.lpDescription.is_null() {
            None
        } else {
            unsafe { desc.lpDescription.to_string().ok() }
        }
    } else {
        None
    }
}

/// Query the current runtime status of a service.
unsafe fn query_status(service_handle: SC_HANDLE) -> Result<ServiceStatus, ServiceError> {
    let mut buffer = vec![0u8; mem::size_of::<SERVICE_STATUS_PROCESS>()];
    let mut bytes_needed: u32 = 0;

    unsafe {
        QueryServiceStatusEx(
            service_handle,
            SC_STATUS_PROCESS_INFO,
            Some(&mut buffer),
            &mut bytes_needed,
        )
        .map_err(|e| t!("get.queryStatusFailed", error = e.to_string()).to_string())?;
    }

    let status = unsafe { &*(buffer.as_ptr().cast::<SERVICE_STATUS_PROCESS>()) };

    match status.dwCurrentState {
        SERVICE_STOPPED => Ok(ServiceStatus::Stopped),
        SERVICE_START_PENDING => Ok(ServiceStatus::StartPending),
        SERVICE_STOP_PENDING => Ok(ServiceStatus::StopPending),
        SERVICE_RUNNING => Ok(ServiceStatus::Running),
        SERVICE_CONTINUE_PENDING => Ok(ServiceStatus::ContinuePending),
        SERVICE_PAUSE_PENDING => Ok(ServiceStatus::PausePending),
        SERVICE_PAUSED => Ok(ServiceStatus::Paused),
        other => Err(
            t!("get.queryStatusFailed", error = format!("unknown state: {}", other.0))
                .to_string()
                .into(),
        ),
    }
}

/// Convert a `ServiceStatus` to the corresponding Windows `SERVICE_STATUS_CURRENT_STATE` constant.
fn status_to_current_state(status: &ServiceStatus) -> SERVICE_STATUS_CURRENT_STATE {
    match status {
        ServiceStatus::Running => SERVICE_RUNNING,
        ServiceStatus::Stopped => SERVICE_STOPPED,
        ServiceStatus::Paused => SERVICE_PAUSED,
        ServiceStatus::StartPending => SERVICE_START_PENDING,
        ServiceStatus::StopPending => SERVICE_STOP_PENDING,
        ServiceStatus::PausePending => SERVICE_PAUSE_PENDING,
        ServiceStatus::ContinuePending => SERVICE_CONTINUE_PENDING,
    }
}

/// Export (enumerate) all services, optionally filtering by the provided criteria.
/// Returns a list of matching services.
pub fn export_services(filter: Option<&WindowsService>) -> Result<Vec<WindowsService>, ServiceError> {
    let scm = unsafe { OpenSCManagerW(None, None, SC_MANAGER_CONNECT | SC_MANAGER_ENUMERATE_SERVICE) }
        .map_err(|e| ServiceError::from(t!("get.openScmFailed", error = e.to_string()).to_string()))?;
    let scm = ScHandle(scm);

    let services = unsafe { enumerate_services(scm.0) }?;
    let mut results = Vec::new();

    // Pre-compute the status filter value for early rejection before expensive per-service queries
    let status_filter_dw = filter.and_then(|f| f.status.as_ref()).map(status_to_current_state);

    for (service_name, current_state) in &services {
        // Quick reject based on status before opening the service handle
        if let Some(expected_state) = status_filter_dw {
            if *current_state != expected_state {
                continue;
            }
        }

        let svc = match unsafe { get_service_details(scm.0, service_name) } {
            Ok(s) => s,
            Err(_) => continue, // skip services we can't query
        };

        if let Some(f) = filter {
            if !matches_filter(&svc, f) {
                continue;
            }
        }

        results.push(svc);
    }

    Ok(results)
}

/// Enumerate all Win32 services using `EnumServicesStatusExW`, handling pagination.
/// Returns a list of `(service_name, current_state)` tuples.
unsafe fn enumerate_services(scm: SC_HANDLE) -> Result<Vec<(String, SERVICE_STATUS_CURRENT_STATE)>, ServiceError> {
    unsafe {
        let mut all_services = Vec::new();
        let mut bytes_needed: u32 = 0;
        let mut services_returned: u32 = 0;
        let mut resume_handle: u32 = 0;

        // Sizing call to determine required buffer size
        let sizing_result = EnumServicesStatusExW(
            scm,
            SC_ENUM_PROCESS_INFO,
            SERVICE_WIN32,
            SERVICE_STATE_ALL,
            None,
            &mut bytes_needed,
            &mut services_returned,
            Some(&mut resume_handle),
            None,
        );

        if let Err(e) = sizing_result {
            if e.code() != ERROR_MORE_DATA.to_hresult()
                && e.code() != ERROR_INSUFFICIENT_BUFFER.to_hresult()
            {
                return Err(t!("export.enumServicesFailed", error = e.to_string()).to_string().into());
            }
        }

        if bytes_needed == 0 {
            return Ok(Vec::new());
        }

        resume_handle = 0;

        loop {
            let mut buffer = vec![0u8; bytes_needed as usize];
            services_returned = 0;

            let result = EnumServicesStatusExW(
                scm,
                SC_ENUM_PROCESS_INFO,
                SERVICE_WIN32,
                SERVICE_STATE_ALL,
                Some(&mut buffer),
                &mut bytes_needed,
                &mut services_returned,
                Some(&mut resume_handle),
                None,
            );

            if services_returned > 0 {
                let entries = std::slice::from_raw_parts(
                    buffer.as_ptr().cast::<ENUM_SERVICE_STATUS_PROCESSW>(),
                    services_returned as usize,
                );

                for entry in entries {
                    if let Ok(name) = entry.lpServiceName.to_string() {
                        all_services.push((name, entry.ServiceStatusProcess.dwCurrentState));
                    }
                }
            }

            match result {
                Ok(()) => break,
                Err(e) => {
                    if e.code() == ERROR_MORE_DATA.to_hresult() {
                        // More services to enumerate; bytes_needed has the required buffer size
                        continue;
                    }
                    return Err(t!("export.enumServicesFailed", error = e.to_string()).to_string().into());
                }
            }
        }

        Ok(all_services)
    }
}

/// Get full service details given an SCM handle and a service key name.
unsafe fn get_service_details(scm: SC_HANDLE, service_name: &str) -> Result<WindowsService, ServiceError> {
    let name_wide = to_wide(service_name);
    let service_handle = unsafe {
        OpenServiceW(
            scm,
            PCWSTR(name_wide.as_ptr()),
            SERVICE_QUERY_CONFIG | SERVICE_QUERY_STATUS,
        )
    }
    .map_err(|e| ServiceError::from(t!("export.openServiceFailed", name = service_name, error = e.to_string()).to_string()))?;
    let service_handle = ScHandle(service_handle);

    unsafe { read_service_state(service_handle.0, service_name) }
}

/// Match a string against a pattern supporting `*` wildcards.
/// If no wildcard is present, performs an exact case-insensitive comparison.
fn matches_wildcard(text: &str, pattern: &str) -> bool {
    if pattern == "*" {
        return true;
    }

    let text_lower = text.to_lowercase();
    let pattern_lower = pattern.to_lowercase();

    let parts: Vec<&str> = pattern_lower.split('*').collect();

    // No wildcard → exact match
    if parts.len() == 1 {
        return text_lower == pattern_lower;
    }

    let starts_with_wildcard = pattern_lower.starts_with('*');
    let ends_with_wildcard = pattern_lower.ends_with('*');

    let mut pos = 0;

    for (i, part) in parts.iter().enumerate() {
        if part.is_empty() {
            continue;
        }

        if i == 0 && !starts_with_wildcard {
            if !text_lower.starts_with(part) {
                return false;
            }
            pos = part.len();
        } else if let Some(found) = text_lower[pos..].find(part) {
            pos += found + part.len();
        } else {
            return false;
        }
    }

    if !ends_with_wildcard {
        if let Some(last) = parts.last() {
            if !last.is_empty() && !text_lower.ends_with(last) {
                return false;
            }
        }
    }

    true
}

/// Build a double-null-terminated UTF-16 multi-string from a list of dependency names.
fn deps_to_multi_string(deps: &[String]) -> Vec<u16> {
    let mut buf = Vec::new();
    for dep in deps {
        buf.extend(dep.encode_utf16());
        buf.push(0); // null terminator for each string
    }
    if buf.is_empty() {
        buf.push(0); // first null for empty multi-string
    }
    buf.push(0); // final null terminator (double-null)
    buf
}

/// Apply the desired service configuration and status changes, then return the final state.
/// Check whether the given account name is a built-in service account.
fn is_builtin_service_account(account: &str) -> bool {
    let normalized = account.to_ascii_uppercase();
    matches!(
        normalized.as_str(),
        "LOCALSYSTEM"
            | "LOCAL SYSTEM"
            | "NT AUTHORITY\\LOCALSERVICE"
            | "NT AUTHORITY\\NETWORKSERVICE"
            | "NT AUTHORITY\\LOCAL SERVICE"
            | "NT AUTHORITY\\NETWORK SERVICE"
            | ".\\LOCALSYSTEM"
    )
}

pub fn set_service(input: &WindowsService) -> Result<WindowsService, ServiceError> {
    let name = input.name.as_deref()
        .ok_or_else(|| t!("set.nameRequired").to_string())?;

    if let Some(ref account) = input.logon_account {
        if !is_builtin_service_account(account) {
            return Err(ServiceError::from(
                t!("set.unsupportedLogonAccount", account = account).to_string(),
            ));
        }
    }

    unsafe {
        let scm = OpenSCManagerW(None, None, SC_MANAGER_CONNECT)
            .map_err(|e| t!("set.openScmFailed", error = e.to_string()).to_string())?;
        let scm = ScHandle(scm);

        let name_wide = to_wide(name);
        let needs_config_change = input.start_type.is_some()
            || input.executable_path.is_some()
            || input.logon_account.is_some()
            || input.error_control.is_some()
            || input.display_name.is_some()
            || input.dependencies.is_some()
            || input.description.is_some();

        let mut access = SERVICE_QUERY_CONFIG | SERVICE_QUERY_STATUS;
        if needs_config_change {
            access |= SERVICE_CHANGE_CONFIG;
        }
        if let Some(ref status) = input.status {
            match status {
                ServiceStatus::Running => {
                    access |= SERVICE_START | SERVICE_PAUSE_CONTINUE;
                }
                ServiceStatus::Stopped => {
                    access |= SERVICE_STOP;
                }
                ServiceStatus::Paused => {
                    access |= SERVICE_PAUSE_CONTINUE;
                }
                _ => {}
            }
        }

        let service_handle = OpenServiceW(scm.0, PCWSTR(name_wide.as_ptr()), access)
            .map_err(|e| t!("set.openServiceFailed", error = e.to_string()).to_string())?;
        let service_handle = ScHandle(service_handle);

        // 1. Apply configuration changes via ChangeServiceConfigW
        let has_config = input.start_type.is_some()
            || input.executable_path.is_some()
            || input.logon_account.is_some()
            || input.error_control.is_some()
            || input.display_name.is_some()
            || input.dependencies.is_some();

        if has_config {
            let dw_start_type = match &input.start_type {
                Some(StartType::Automatic | StartType::AutomaticDelayedStart) => SERVICE_AUTO_START,
                Some(StartType::Manual) => SERVICE_DEMAND_START,
                Some(StartType::Disabled) => SERVICE_DISABLED,
                None => SERVICE_START_TYPE(SERVICE_NO_CHANGE_VALUE),
            };

            let dw_error_control = match &input.error_control {
                Some(ErrorControl::Ignore) => SERVICE_ERROR_IGNORE,
                Some(ErrorControl::Normal) => SERVICE_ERROR_NORMAL,
                Some(ErrorControl::Severe) => SERVICE_ERROR_SEVERE,
                Some(ErrorControl::Critical) => SERVICE_ERROR_CRITICAL,
                None => SERVICE_ERROR(SERVICE_NO_CHANGE_VALUE),
            };

            // When setting AutomaticDelayedStart, the service must not belong to a load
            // order group — Windows rejects ChangeServiceConfig2W for the delayed auto-start
            // flag with ERROR_INVALID_PARAMETER if one is set. Clear the group by passing an
            // empty string instead of null (which means "no change").
            let clear_group_wide;
            let load_order_group_ptr = if matches!(&input.start_type, Some(StartType::AutomaticDelayedStart)) {
                clear_group_wide = to_wide("");
                PCWSTR(clear_group_wide.as_ptr())
            } else {
                PCWSTR::null()
            };

            // Build wide strings; they must live through the API call.
            let exe_wide = input.executable_path.as_ref().map(|s| to_wide(s));
            let logon_wide = input.logon_account.as_ref().map(|s| to_wide(s));
            let display_wide = input.display_name.as_ref().map(|s| to_wide(s));
            let deps_wide = input.dependencies.as_ref().map(|d| deps_to_multi_string(d));

            let exe_ptr = exe_wide.as_ref().map_or(PCWSTR::null(), |w| PCWSTR(w.as_ptr()));
            let logon_ptr = logon_wide.as_ref().map_or(PCWSTR::null(), |w| PCWSTR(w.as_ptr()));
            let display_ptr = display_wide.as_ref().map_or(PCWSTR::null(), |w| PCWSTR(w.as_ptr()));
            let deps_ptr = deps_wide.as_ref().map_or(PCWSTR::null(), |w| PCWSTR(w.as_ptr()));

            ChangeServiceConfigW(
                service_handle.0,
                ENUM_SERVICE_TYPE(SERVICE_NO_CHANGE_VALUE), // service type unchanged
                dw_start_type,
                dw_error_control,
                exe_ptr,
                load_order_group_ptr,
                None, // tag id unchanged
                deps_ptr,
                logon_ptr,
                PCWSTR::null(), // password unchanged
                display_ptr,
            )
            .map_err(|e| t!("set.changeConfigFailed", error = e.to_string()).to_string())?;
        }

        // 2. Set delayed auto-start flag when start type is specified
        if let Some(ref start_type) = input.start_type {
            let delayed = matches!(start_type, StartType::AutomaticDelayedStart);
            let mut info = SERVICE_DELAYED_AUTO_START_INFO {
                fDelayedAutostart: delayed.into(),
            };
            ChangeServiceConfig2W(
                service_handle.0,
                SERVICE_CONFIG_DELAYED_AUTO_START_INFO,
                Some(std::ptr::from_mut(&mut info).cast()),
            )
            .map_err(|e| t!("set.changeConfig2Failed", error = e.to_string()).to_string())?;
        }

        // 3. Set description
        if let Some(ref desc) = input.description {
            let mut desc_wide = to_wide(desc);
            let mut info = SERVICE_DESCRIPTIONW {
                lpDescription: PWSTR(desc_wide.as_mut_ptr()),
            };
            ChangeServiceConfig2W(
                service_handle.0,
                SERVICE_CONFIG_DESCRIPTION,
                Some(std::ptr::from_mut(&mut info).cast()),
            )
            .map_err(|e| t!("set.changeConfig2Failed", error = e.to_string()).to_string())?;
        }

        // 4. Handle service status transitions
        if let Some(ref desired_status) = input.status {
            let mut current = query_status(service_handle.0)?;

            // Wait for any pending state to resolve first
            current = match current {
                ServiceStatus::StopPending => {
                    wait_for_status(service_handle.0, &ServiceStatus::Stopped)?;
                    ServiceStatus::Stopped
                }
                ServiceStatus::StartPending | ServiceStatus::ContinuePending => {
                    wait_for_status(service_handle.0, &ServiceStatus::Running)?;
                    ServiceStatus::Running
                }
                ServiceStatus::PausePending => {
                    wait_for_status(service_handle.0, &ServiceStatus::Paused)?;
                    ServiceStatus::Paused
                }
                other => other,
            };

            if current != *desired_status {
                match desired_status {
                    ServiceStatus::Running => {
                        match current {
                            ServiceStatus::Stopped => {
                                StartServiceW(service_handle.0, None)
                                    .map_err(|e| t!("set.startFailed", error = e.to_string()).to_string())?;
                            }
                            ServiceStatus::Paused => {
                                let mut svc_status = SERVICE_STATUS::default();
                                ControlService(service_handle.0, SERVICE_CONTROL_CONTINUE, &mut svc_status)
                                    .map_err(|e| t!("set.continueFailed", error = e.to_string()).to_string())?;
                            }
                            _ => {
                                return Err(t!("set.unsupportedTransition",
                                    current = current.to_string(),
                                    desired = desired_status.to_string()
                                ).to_string().into());
                            }
                        }
                        wait_for_status(service_handle.0, desired_status)?;
                    }
                    ServiceStatus::Stopped => {
                        let mut svc_status = SERVICE_STATUS::default();
                        ControlService(service_handle.0, SERVICE_CONTROL_STOP, &mut svc_status)
                            .map_err(|e| t!("set.stopFailed", error = e.to_string()).to_string())?;
                        wait_for_status(service_handle.0, desired_status)?;
                    }
                    ServiceStatus::Paused => {
                        if current != ServiceStatus::Running {
                            return Err(t!("set.unsupportedTransition",
                                current = current.to_string(),
                                desired = desired_status.to_string()
                            ).to_string().into());
                        }
                        let mut svc_status = SERVICE_STATUS::default();
                        ControlService(service_handle.0, SERVICE_CONTROL_PAUSE, &mut svc_status)
                            .map_err(|e| t!("set.pauseFailed", error = e.to_string()).to_string())?;
                        wait_for_status(service_handle.0, desired_status)?;
                    }
                    _ => {
                        return Err(t!("set.unsupportedStatus",
                            status = desired_status.to_string()
                        ).to_string().into());
                    }
                }
            }
        }

        // Return final state
        read_service_state(service_handle.0, name)
    }
}

/// Wait for a service to reach the desired status, with a timeout.
unsafe fn wait_for_status(
    service_handle: SC_HANDLE,
    desired: &ServiceStatus,
) -> Result<(), ServiceError> {
    let start = std::time::Instant::now();
    let timeout = std::time::Duration::from_secs(STATUS_WAIT_TIMEOUT_SECS);
    loop {
        let current = unsafe { query_status(service_handle)? };
        if current == *desired {
            return Ok(());
        }
        if start.elapsed() > timeout {
            return Err(t!("set.statusTimeout",
                expected = desired.to_string(),
                actual = current.to_string()
            ).to_string().into());
        }
        std::thread::sleep(std::time::Duration::from_millis(STATUS_POLL_INTERVAL_MS));
    }
}

/// Check whether `service` matches all non-`None` fields in `filter`.
fn matches_filter(service: &WindowsService, filter: &WindowsService) -> bool {
    // name — wildcard match
    if let Some(ref pattern) = filter.name {
        let name = service.name.as_deref().unwrap_or("");
        if !matches_wildcard(name, pattern) {
            return false;
        }
    }

    // display_name — wildcard match
    if let Some(ref pattern) = filter.display_name {
        let dn = service.display_name.as_deref().unwrap_or("");
        if !matches_wildcard(dn, pattern) {
            return false;
        }
    }

    // description — wildcard match
    if let Some(ref pattern) = filter.description {
        let desc = service.description.as_deref().unwrap_or("");
        if !matches_wildcard(desc, pattern) {
            return false;
        }
    }

    // exist — exact match
    if let Some(expected_exist) = filter.exist {
        let actual_exist = service.exist.unwrap_or(false);
        if actual_exist != expected_exist {
            return false;
        }
    }

    // status — exact match
    if let Some(ref expected_status) = filter.status {
        match &service.status {
            Some(actual_status) if actual_status == expected_status => {}
            _ => return false,
        }
    }

    // start_type — exact match
    if let Some(ref expected_start) = filter.start_type {
        match &service.start_type {
            Some(actual_start) if actual_start == expected_start => {}
            _ => return false,
        }
    }

    // logon_account — exact case-insensitive match
    if let Some(ref expected_account) = filter.logon_account {
        let actual = service.logon_account.as_deref().unwrap_or("");
        if !actual.eq_ignore_ascii_case(expected_account) {
            return false;
        }
    }

    // Note: executable_path and error_control are intentionally not filtered.

    // dependencies — service must have at least all specified dependencies
    if let Some(ref expected_deps) = filter.dependencies {
        let actual_deps = service.dependencies.as_deref().unwrap_or(&[]);
        for dep in expected_deps {
            let dep_lower = dep.to_lowercase();
            if !actual_deps.iter().any(|d| d.to_lowercase() == dep_lower) {
                return false;
            }
        }
    }

    true
}
