// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[cfg(windows)]
use authenticode_windows::check_authenticode;
use rust_i18n::t;
use std::{
    collections::HashMap,
    fmt::Display,
    path::Path,
};
#[cfg(windows)]
use std::sync::LazyLock;
use tracing::warn;

use crate::dscerror::DscError;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TrustLevel {
    Trusted,
    ExplicitlyDistrusted,
    Unsigned,
    Untrusted,
    NotMeetSecuritySettings,
    CannotBeVerified,
    Unknown,
}

impl Display for TrustLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            TrustLevel::Trusted => t!("security.authenticode.trustLevelTrusted"),
            TrustLevel::ExplicitlyDistrusted => t!("security.authenticode.trustLevelExplicitlyDistrusted"),
            TrustLevel::Unsigned => t!("security.authenticode.trustLevelUnsigned"),
            TrustLevel::Untrusted => t!("security.authenticode.trustLevelUntrusted"),
            TrustLevel::NotMeetSecuritySettings => t!("security.authenticode.trustLevelNotMeetSecuritySettings"),
            TrustLevel::CannotBeVerified => t!("security.authenticode.trustLevelCannotBeVerified"),
            TrustLevel::Unknown => t!("security.authenticode.trustLevelUnknown"),
        };
        write!(f, "{s}")
    }
}

#[cfg(windows)]
mod authenticode_windows;

#[cfg(windows)]
static CHECKED_FILES: LazyLock<std::sync::Mutex<HashMap<String, TrustLevel>>> = LazyLock::new(|| std::sync::Mutex::new(HashMap::new()));

#[cfg(windows)]
fn add_file_as_checked(file_path: &Path, trust_level: TrustLevel) {
    let file_str = file_path.to_string_lossy().to_string();
    let mut checked_files = CHECKED_FILES.lock().unwrap();
    checked_files.entry(file_str).or_insert(trust_level);
}

#[cfg(windows)]
fn is_file_checked(file_path: &Path) -> bool {
    let file_str = file_path.to_string_lossy().to_string();
    let checked_files = CHECKED_FILES.lock().unwrap();
    checked_files.contains_key(&file_str)
}

#[cfg(windows)]
fn get_file_trust_level(file_path: &Path) -> TrustLevel {
    let file_str = file_path.to_string_lossy().to_string();
    let checked_files = CHECKED_FILES.lock().unwrap();
    checked_files.get(&file_str).copied().unwrap_or(TrustLevel::Unknown)
}

/// Check the security of a file.
///
/// # Arguments
/// * `file_path` - The path to the file to check.
///
/// # Returns
/// * `Ok(TrustLevel)` if the file was checked successfully, with its trust level.
///
/// # Errors
/// This function will return an error if the Authenticode check fails on Windows.
#[cfg(windows)]
pub fn check_file_security(file_path: &Path) -> Result<TrustLevel, DscError> {
    let trust_level = check_authenticode(file_path)?;
    if !is_file_checked(file_path) {
        add_file_as_checked(file_path, trust_level);
        match trust_level {
            TrustLevel::Trusted => {},
            TrustLevel::ExplicitlyDistrusted => warn!("{}", t!("security.authenticode.signatureExplicitlyDistrusted", file = file_path.display())),
            TrustLevel::Unsigned => warn!("{}", t!("security.authenticode.fileNotSigned", file = file_path.display())),
            TrustLevel::Untrusted => warn!("{}", t!("security.authenticode.signatureNotTrusted", file = file_path.display())),
            TrustLevel::NotMeetSecuritySettings => warn!("{}", t!("security.authenticode.signatureDoesNotMeetSecuritySettings", file = file_path.display())),
            TrustLevel::CannotBeVerified => warn!("{}", t!("security.authenticode.signatureCouldNotBeVerified", file = file_path.display())),
            TrustLevel::Unknown => warn!("{}", t!("security.authenticode.trustLevelIsUnknown", file = file_path.display())),
        }
    }
    Ok(trust_level)
}

/// On non-Windows platforms, this function is a no-op.
///
/// # Arguments
/// * `_file_path` - The path to the file to check.
///
/// # Returns
/// * `Ok(())` always, as there are no security checks on non-Windows platforms.
///
/// # Errors
/// This function does not return any errors on non-Windows platforms.
#[cfg(not(windows))]
pub fn check_file_security(_file_path: &Path) -> Result<(), DscError> {
    Ok(())
}
