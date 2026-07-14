// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::dscerror::DscError;
use rust_i18n::t;
use serde_json::Value;
use std::{
    fs,
    fs::{canonicalize, File},
    io::BufReader,
    path::{Path, PathBuf},
    env,
};
use tracing::{debug, warn};
use which::which;

pub struct DscSettingValue {
    pub setting:  Value,
    pub policy: Value,
}

impl Default for DscSettingValue {
    fn default() -> DscSettingValue {
        DscSettingValue {
            setting: Value::Null,
            policy: Value::Null,
        }
    }
}

/// Return JSON string whether the input is JSON or YAML
///
/// # Arguments
///
/// * `value` - A string slice that holds the input value
///
/// # Returns
///
/// A string that holds the JSON value
///
/// # Errors
///
/// This function will return an error if the input value is not valid JSON or YAML
pub fn parse_input_to_json(value: &str) -> Result<String, DscError> {
    match serde_json::from_str(value) {
        Ok(json) => Ok(json),
        Err(_) => {
            match serde_yaml::from_str::<Value>(value) {
                Ok(yaml) => {
                    match serde_json::to_value(yaml) {
                        Ok(json) => Ok(json.to_string()),
                        Err(err) => {
                            Err(DscError::Json(err))
                        }
                    }
                },
                Err(err) => {
                    Err(DscError::Yaml(err))
                }
            }
        }
    }
}

/// Converts a wildcard string to a regex pattern.
///
/// # Arguments
///
/// * `wildcard` - A string slice that holds the wildcard pattern.
///
/// # Returns
/// A string that holds the regex pattern.
#[must_use]
pub fn convert_wildcard_to_regex(wildcard: &str) -> String {
    let mut regex = wildcard.to_string().replace('.', "\\.").replace('?', ".").replace('*', ".*?");
    regex.insert(0, '^');
    regex.push('$');
    regex
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_wildcard_to_regex() {
        let wildcard = "*";
        let regex = convert_wildcard_to_regex(wildcard);
        assert_eq!(regex, "^.*?$");

        let wildcard = "File";
        let regex = convert_wildcard_to_regex(wildcard);
        assert_eq!(regex, "^File$");

        let wildcard = "r*";
        let regex = convert_wildcard_to_regex(wildcard);
        assert_eq!(regex, "^r.*?$");
    }

    #[cfg(not(target_os = "windows"))]
    mod linux_permission_tests {
        use super::*;
        use std::fs as stdfs;

        #[test]
        fn test_verify_linux_permissions_nonexistent_path() {
            let result = verify_linux_permissions(Path::new("/nonexistent/path/that/does/not/exist"));
            assert!(!result, "nonexistent path should return false (fail closed)");
        }

        #[test]
        fn test_verify_linux_permissions_non_root_owned() {
            // A temp dir is owned by the current (non-root) user on CI
            let dir = stdfs::canonicalize(env::temp_dir()).expect("temp dir");
            let test_dir = dir.join("dsc_test_permissions");
            let _ = stdfs::create_dir(&test_dir);
            let result = verify_linux_permissions(&test_dir);
            let _ = stdfs::remove_dir(&test_dir);
            // On CI (non-root), the temp dir is not owned by root
            if nix::unistd::Uid::effective().is_root() {
                // If running as root, dir is root-owned, check mode bits instead
                return;
            }
            assert!(!result, "directory not owned by root should return false");
        }

        #[test]
        fn test_get_settings_policy_file_path_returns_some_when_dir_missing() {
            // If /etc/dsc doesn't exist, the function should return Some with the path
            // (the file may not exist but the path is still returned for attempted loading)
            if Path::new("/etc/dsc").exists() {
                // Can't test this case when the dir exists
                return;
            }
            let result = get_settings_policy_file_path();
            assert!(result.is_some(), "should return Some when /etc/dsc doesn't exist");
            let path = result.unwrap();
            assert!(path.to_string_lossy().contains("dsc.settings.json"));
        }

        #[test]
        fn test_get_settings_policy_file_path_with_insecure_dir() {
            // When /etc/dsc exists but is not root-owned, returns None
            if !Path::new("/etc/dsc").exists() || nix::unistd::Uid::effective().is_root() {
                return;
            }
            // /etc/dsc exists and we're not root - if it's owned by root
            // it should return Some; this test just exercises the code path
            let _result = get_settings_policy_file_path();
        }
    }
}

/// Will search setting files for the specified setting.
/// Performance implication: Use this function economically as every call opens/reads several config files.
/// TODO: cache the config
///
/// # Arguments
///
/// * `value_name` - The name of the setting.
///
/// # Errors
///
/// Will return `Err` if could not find requested setting.
pub fn get_setting(value_name: &str) -> Result<DscSettingValue, DscError> {

    const SETTINGS_FILE_NAME: &str = "dsc.settings.json";
    // Note that default settings file has root nodes as settings schema version that is specific to this version of dsc
    const DEFAULT_SETTINGS_FILE_NAME: &str = "dsc_default.settings.json";
    const DEFAULT_SETTINGS_SCHEMA_VERSION: &str = "1";

    let mut result: DscSettingValue = DscSettingValue::default();
    let mut settings_file_path : PathBuf;

    if let Some(exe_home) = get_exe_path()?.parent() {
        // First, get setting from the default settings file
        settings_file_path = exe_home.join(DEFAULT_SETTINGS_FILE_NAME);
        if let Ok(v) = load_value_from_json(&settings_file_path, DEFAULT_SETTINGS_SCHEMA_VERSION) {
            if let Some(n) = v.get(value_name) {
                result.setting = n.clone();
                debug!("{}", t!("util.foundSetting", name = value_name, path = settings_file_path.to_string_lossy()));
            }
        } else {
            debug!("{}", t!("util.notFoundSetting", name = value_name, path = settings_file_path.to_string_lossy()));
        }

        // Second, get setting from the active settings file overwriting previous value
        settings_file_path = exe_home.join(SETTINGS_FILE_NAME);
        if let Ok(v) = load_value_from_json(&settings_file_path, value_name) {
            result.setting = v;
            debug!("{}", t!("util.foundSetting", name = value_name, path = settings_file_path.to_string_lossy()));
        } else {
            debug!("{}", t!("util.notFoundSetting", name = value_name, path = settings_file_path.to_string_lossy()));
        }
    } else {
        debug!("{}", t!("util.failedToGetExePath"));
    }

    // Third, get setting from the policy
    if let Some(settings_file_path) = get_settings_policy_file_path() {
        if let Ok(v) = load_value_from_json(&settings_file_path, value_name) {
            result.policy = v;
            debug!("{}", t!("util.foundSetting", name = value_name, path = settings_file_path.to_string_lossy()));
        } else {
            debug!("{}", t!("util.notFoundSetting", name = value_name, path = settings_file_path.to_string_lossy()));
        }
    }

    if (result.setting == serde_json::Value::Null) && (result.policy == serde_json::Value::Null) {
        return Err(DscError::NotSupported(t!("util.settingNotFound", name = value_name).to_string()));
    }

    Ok(result)
}

fn load_value_from_json(path: &PathBuf, value_name: &str) -> Result<serde_json::Value, DscError> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let root: serde_json::Value = match serde_json::from_reader(reader) {
        Ok(j) => j,
        Err(err) => {
            return Err(DscError::Json(err));
        }
    };

    if let Some(r) = root.as_object() {
        for (key, value) in r {
            if *key == value_name {
                return Ok(value.clone())
            }
        }
    }

    Err(DscError::NotSupported(value_name.to_string()))
}

/// Gets path to the current dsc process.
/// If dsc is started using a symlink, this functon returns target of the symlink.
///
/// # Errors
///
/// Will return `Err` if path to the current exe can't be retrived.
pub fn get_exe_path() -> Result<PathBuf, DscError> {
    if let Ok(exe) = env::current_exe() {
        if let Ok(target_path) = fs::read_link(exe.clone()) {
            return Ok(target_path);
        }

        return Ok(exe);
    }

    Err(DscError::NotSupported(t!("util.failedToGetExePath").to_string()))
}

#[cfg(target_os = "windows")]
fn get_settings_policy_file_path() -> Option<PathBuf>
{
    // $env:ProgramData+"\dsc\dsc.settings.json"
    let Ok(local_program_data_path) = std::env::var("ProgramData") else { return None; };
    let dsc_folder = Path::new(&local_program_data_path).join("dsc");
    let settings_path = dsc_folder.join("dsc.settings.json").display().to_string();

    if !dsc_folder.exists() {
        return Some(PathBuf::from(settings_path));
    }

    if !verify_windows_acl(&dsc_folder) {
        let required = t!("util.policyFolderNotSecureWindows", path = dsc_folder.display());
        warn!("{}", t!("util.policyFolderNotSecure", path = dsc_folder.display(), required = required));
        return None;
    }

    Some(PathBuf::from(settings_path))
}

/// Returns `true` if only SYSTEM and Administrators have write access; `false` otherwise.
#[cfg(target_os = "windows")]
fn verify_windows_acl(dsc_folder: &Path) -> bool {
    use windows::Win32::Security::{
        Authorization::{GetNamedSecurityInfoW, SE_FILE_OBJECT},
        IsWellKnownSid, GetAce,
        WinBuiltinAdministratorsSid, WinLocalSystemSid,
        ACL, ACCESS_ALLOWED_ACE, PSECURITY_DESCRIPTOR,
    };
    use windows::Win32::Foundation::{LocalFree, HLOCAL};
    use windows::core::PCWSTR;
    use std::ptr;

    const FILE_WRITE_DATA: u32 = 0x0002;
    const FILE_APPEND_DATA: u32 = 0x0004;
    const WRITE_DAC: u32 = 0x0004_0000;
    const WRITE_OWNER: u32 = 0x0008_0000;
    const GENERIC_WRITE: u32 = 0x4000_0000;
    const GENERIC_ALL: u32 = 0x1000_0000;
    const WRITE_MASK: u32 = FILE_WRITE_DATA | FILE_APPEND_DATA | WRITE_DAC | WRITE_OWNER | GENERIC_WRITE | GENERIC_ALL;

    const INHERIT_ONLY_ACE: u8 = 0x08;

    let folder_wide: Vec<u16> = dsc_folder
        .to_string_lossy()
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect();

    let mut p_sd: PSECURITY_DESCRIPTOR = PSECURITY_DESCRIPTOR(ptr::null_mut());
    let mut p_dacl: *mut ACL = ptr::null_mut();

    let result = unsafe {
        GetNamedSecurityInfoW(
            PCWSTR(folder_wide.as_ptr()),
            SE_FILE_OBJECT,
            windows::Win32::Security::DACL_SECURITY_INFORMATION,
            None,
            None,
            Some(&mut p_dacl),
            None,
            &mut p_sd,
        )
    };

    // Fail closed: if we can't read the security descriptor, treat as insecure
    if result.is_err() {
        return false;
    }

    // A NULL DACL means full access to everyone
    if p_dacl.is_null() {
        unsafe { let _ = LocalFree(Some(HLOCAL(p_sd.0.cast()))); }
        return false;
    }

    let ace_count = unsafe { (*p_dacl).AceCount };

    for i in 0..ace_count {
        let mut ace_ptr: *mut core::ffi::c_void = ptr::null_mut();
        let ok = unsafe { GetAce(p_dacl, i as u32, &mut ace_ptr) };
        if ok.is_err() {
            // Fail closed: if we can't read an ACE, treat as insecure
            unsafe { let _ = LocalFree(Some(HLOCAL(p_sd.0.cast()))); }
            return false;
        }

        let header = unsafe { &*(ace_ptr as *const windows::Win32::Security::ACE_HEADER) };

        // Skip inherit-only ACEs as they don't apply to this folder
        if (header.AceFlags & INHERIT_ONLY_ACE) != 0 {
            continue;
        }

        // Skip deny ACEs (types 1, 6, 10) - they restrict access, not grant it
        if header.AceType == 1 || header.AceType == 6 || header.AceType == 10 {
            continue;
        }

        // For ACCESS_ALLOWED_ACE_TYPE (0), check the SID
        if header.AceType == 0 {
            let ace = unsafe { &*(ace_ptr as *const ACCESS_ALLOWED_ACE) };
            if (ace.Mask & WRITE_MASK) == 0 {
                continue;
            }

            let sid_ptr = &ace.SidStart as *const u32 as *const core::ffi::c_void;
            let psid = windows::Win32::Security::PSID(sid_ptr as *mut core::ffi::c_void);

            let is_system = unsafe { IsWellKnownSid(psid, WinLocalSystemSid).as_bool() };
            let is_admin = unsafe { IsWellKnownSid(psid, WinBuiltinAdministratorsSid).as_bool() };

            if !is_system && !is_admin {
                unsafe { let _ = LocalFree(Some(HLOCAL(p_sd.0.cast()))); }
                return false;
            }
            continue;
        }

        // For any other allow-type ACE (callback, object, etc.) we cannot
        // reliably extract the SID, so fail closed to be safe
        let ace = unsafe { &*(ace_ptr as *const ACCESS_ALLOWED_ACE) };
        if (ace.Mask & WRITE_MASK) != 0 {
            unsafe { let _ = LocalFree(Some(HLOCAL(p_sd.0.cast()))); }
            return false;
        }
    }

    unsafe { let _ = LocalFree(Some(HLOCAL(p_sd.0.cast()))); }
    true
}

#[cfg(not(target_os = "windows"))]
fn get_settings_policy_file_path() -> Option<PathBuf>
{
    // "/etc/dsc/dsc.settings.json"
    // This location is writable only by root, but readable by all users
    let dsc_folder = Path::new("/etc").join("dsc");
    let settings_path = dsc_folder.join("dsc.settings.json");

    if !dsc_folder.exists() {
        return Some(settings_path);
    }

    if !verify_linux_permissions(&dsc_folder) {
        let required = t!("util.policyFolderNotSecureLinux");
        warn!("{}", t!("util.policyFolderNotSecure", path = dsc_folder.display(), required = required));
        return None;
    }

    Some(settings_path)
}

/// Returns `true` if the folder is owned by root and has no group/other write bits; `false` otherwise.
#[cfg(not(target_os = "windows"))]
fn verify_linux_permissions(folder: &Path) -> bool {
    use std::os::unix::fs::MetadataExt;

    // Fail closed: if we can't read metadata, treat as insecure
    let Ok(metadata) = fs::metadata(folder) else {
        return false;
    };

    let mode = metadata.mode();
    let uid = metadata.uid();

    // Verify owner is root and no group or other write bits are set
    uid == 0 && (mode & 0o020) == 0 && (mode & 0o002) == 0
}

/// Generates a resource ID from the specified type and name.
///
/// # Arguments
/// * `type_name` - The resource type in the format "namespace/type".
/// * `name` - The resource name.
///
/// # Returns
/// A string that holds the resource ID in the format "namespace/type:name".
#[must_use]
pub fn resource_id(type_name: &str, name: &str) -> String {
    let mut result = String::new();
    result.push_str(type_name);
    result.push(':');
    let encoded = urlencoding::encode(name);
    result.push_str(&encoded);
    result
}

pub fn canonicalize_which(executable: &str, cwd: Option<&Path>) -> Result<String, DscError> {
    // Use PathBuf to handle path separators robustly
    let mut executable_path = PathBuf::from(executable);
    if cfg!(target_os = "windows") && executable_path.extension().is_none() {
        executable_path.set_extension("exe");
    }
    if which(executable).is_err() {
        if let Some(cwd_path) = cwd {
            if let Ok(canonical_path) = canonicalize(cwd_path.join(&executable_path)) {
                return Ok(canonical_path.to_string_lossy().to_string());
            }
            return Err(DscError::CommandOperation(t!("util.executableNotFoundInWorkingDirectory", executable = &executable, cwd = cwd_path.display()).to_string(), executable_path.to_string_lossy().to_string()));
        }
        return Err(DscError::CommandOperation(t!("util.executableNotFound", executable = &executable).to_string(), executable.to_string()));
    }
    Ok(executable.to_string())
}

#[macro_export]
macro_rules! locked_clear {
    ($lockable:expr) => {{
        $lockable.write().unwrap().clear();
    }};
}

#[macro_export]
macro_rules! locked_is_empty {
    ($lockable:expr) => {{
        $lockable.read().unwrap().is_empty()
    }};
}

#[macro_export]
macro_rules! locked_extend {
    ($lockable:expr, $items:expr) => {{
        $lockable.write().unwrap().extend($items);
    }};
}

#[macro_export]
macro_rules! locked_clone {
    ($lockable:expr) => {{
        $lockable.read().unwrap().clone()
    }};
}

#[macro_export]
macro_rules! locked_get {
    ($lockable:expr, $key:expr) => {{
        if let Some(v) = $lockable.read().unwrap().get($key) {
            Some(v.clone())
        } else {
            None
        }
    }};
}
