// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use authenticode::check_authenticode;
use std::{path::Path, sync::LazyLock};

use crate::dscerror::DscError;

#[cfg(windows)]
mod authenticode;

static CHECKED_FILES: LazyLock<std::sync::Mutex<Vec<String>>> = LazyLock::new(|| std::sync::Mutex::new(vec![]));

fn add_file_as_checked(file_path: &Path) {
    let file_str = file_path.to_string_lossy().to_string();
    let mut checked_files = CHECKED_FILES.lock().unwrap();
    if !checked_files.contains(&file_str) {
        checked_files.push(file_str);
    }
}

fn is_file_checked(file_path: &Path) -> bool {
    let file_str = file_path.to_string_lossy().to_string();
    let checked_files = CHECKED_FILES.lock().unwrap();
    checked_files.contains(&file_str)
}

/// Check the security of a file.
///
/// # Arguments
/// 
/// * `file_path` - The path to the file to check.
/// 
/// # Returns
/// 
/// * `Ok(())` if the file passes the security checks.
/// * `Err(DscError)` if the file fails the security checks.
/// 
/// # Errors
/// 
/// This function will return an error if the Authenticode check fails on Windows.
pub fn check_file_security(file_path: &Path) -> Result<(), DscError> {
    #[cfg(windows)]
    {
        check_authenticode(file_path)?;
        Ok(())
    }
    #[cfg(not(windows))]
    {
        // On non-Windows platforms, we skip the Authenticode check.
        Ok(())
    }
}
