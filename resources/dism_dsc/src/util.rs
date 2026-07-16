// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use serde::{Deserialize, Serialize};
use std::fmt;

/// DISM package/feature state values shared by both Optional Features and Features on Demand.
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum DismState {
    NotPresent,
    UninstallPending,
    Staged,
    Removed,
    Installed,
    InstallPending,
    Superseded,
    PartiallyInstalled,
}

impl fmt::Display for DismState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DismState::NotPresent => write!(f, "NotPresent"),
            DismState::UninstallPending => write!(f, "UninstallPending"),
            DismState::Staged => write!(f, "Staged"),
            DismState::Removed => write!(f, "Removed"),
            DismState::Installed => write!(f, "Installed"),
            DismState::InstallPending => write!(f, "InstallPending"),
            DismState::Superseded => write!(f, "Superseded"),
            DismState::PartiallyInstalled => write!(f, "PartiallyInstalled"),
        }
    }
}

impl DismState {
    pub fn from_dism(state: i32) -> Option<Self> {
        match state {
            0 => Some(DismState::NotPresent),
            1 => Some(DismState::UninstallPending),
            2 => Some(DismState::Staged),
            3 => Some(DismState::Removed),
            4 => Some(DismState::Installed),
            5 => Some(DismState::InstallPending),
            6 => Some(DismState::Superseded),
            7 => Some(DismState::PartiallyInstalled),
            _ => None,
        }
    }
}

/// Returns the computer name from the COMPUTERNAME environment variable, or "localhost" as fallback.
pub fn get_computer_name() -> String {
    std::env::var("COMPUTERNAME").unwrap_or_else(|_| "localhost".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dism_state_from_dism() {
        assert_eq!(DismState::from_dism(0), Some(DismState::NotPresent));
        assert_eq!(DismState::from_dism(4), Some(DismState::Installed));
        assert_eq!(DismState::from_dism(7), Some(DismState::PartiallyInstalled));
        assert_eq!(DismState::from_dism(8), None);
        assert_eq!(DismState::from_dism(-1), None);
    }
}
