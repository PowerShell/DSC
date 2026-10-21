// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! Defines and manages a global instance of the DSC settings.
//! 
//! This module provides functions to access and initialize the global DSC
//! settings instance. This avoids needing to pass the DSC settings to every
//! function or module that leverages the settings.

use std::sync::{OnceLock, Mutex, MutexGuard};

use crate::settings::{DscSettings, sources::CommandLineData};

/// Defines a global instance of the DSC settings, accessible throughout the
/// application.
pub static DSC_SETTINGS_GLOBAL: OnceLock<Mutex<DscSettings>> = OnceLock::new();

/// Retrieves the global instance of the DSC settings, initializing it if
/// needed.
///
/// # Returns
///
/// A `MutexGuard` to the global `DscSettings` instance.
pub fn get_or_init() -> MutexGuard<'static, DscSettings> {
    DSC_SETTINGS_GLOBAL.get_or_init(|| Mutex::new(DscSettings::new()))
        .lock()
        .unwrap()
}

/// Retrieves the global instance of the DSC settings, initializing it if
/// needed with the provided command line data.
///
/// # Returns
///
/// A `MutexGuard` to the global `DscSettings` instance.
pub fn get_or_init_with_command_line(cli_data: CommandLineData) -> MutexGuard<'static, DscSettings> {
    DSC_SETTINGS_GLOBAL.get_or_init(|| Mutex::new(DscSettings::new_with_command_line(cli_data)))
        .lock()
        .unwrap()
}

/// Retrieves the global instance of the DSC settings if it's been initialized.
///
/// # Returns
///
/// An `Option` containing a `MutexGuard` to the global `DscSettings` instance
/// if it's available.
pub fn get() -> Option<MutexGuard<'static, DscSettings>> {
    DSC_SETTINGS_GLOBAL.get().map(|mutex| mutex.lock().unwrap())
}
