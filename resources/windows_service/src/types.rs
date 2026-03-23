// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use serde::{Deserialize, Serialize};

/// Represents the start type of a Windows service.
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum StartType {
    /// The service is started automatically by the Service Control Manager during system startup.
    Automatic,
    /// The service is started automatically, with a delayed start after other auto-start services.
    AutomaticDelayedStart,
    /// The service is started only manually (e.g., via `sc start` or the Services console).
    Manual,
    /// The service is disabled and cannot be started.
    Disabled,
}

/// Represents the current status of a Windows service.
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum ServiceStatus {
    Running,
    Stopped,
    Paused,
    StartPending,
    StopPending,
    PausePending,
    ContinuePending,
}

/// Represents the error control level for a Windows service.
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum ErrorControl {
    /// The startup program ignores the error and continues.
    Ignore,
    /// The startup program logs the error and continues.
    Normal,
    /// The startup program logs the error. If the last-known-good configuration is being started,
    /// startup continues. Otherwise, the system is restarted with the last-known-good configuration.
    Severe,
    /// The startup program logs the error. If the last-known-good configuration is being started,
    /// the startup operation fails. Otherwise, the system is restarted with the last-known-good
    /// configuration.
    Critical,
}

/// Represents a Windows service resource.
#[derive(Debug, Default, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct WindowsService {
    /// The name of the service (used to identify the service in the Service Control Manager).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// The display name of the service shown in the Services console.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,

    /// A description of the service.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Indicates if the service exists.
    #[serde(rename = "_exist", skip_serializing_if = "Option::is_none")]
    pub exist: Option<bool>,

    /// The current status of the service.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<ServiceStatus>,

    /// The start type of the service.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_type: Option<StartType>,

    /// The fully qualified path to the service binary.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub executable_path: Option<String>,

    /// The account under which the service runs (e.g., `LocalSystem`, `NT AUTHORITY\NetworkService`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logon_account: Option<String>,

    /// The error control level for the service.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_control: Option<ErrorControl>,

    /// A list of service names that this service depends on.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dependencies: Option<Vec<String>>,
}

impl WindowsService {
    #[must_use]
    #[allow(dead_code)]
    pub fn new(name: &str) -> Self {
        Self {
            name: Some(name.to_string()),
            display_name: None,
            description: None,
            exist: None,
            status: None,
            start_type: None,
            executable_path: None,
            logon_account: None,
            error_control: None,
            dependencies: None,
        }
    }
}

impl std::fmt::Display for StartType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StartType::Automatic => write!(f, "Automatic"),
            StartType::AutomaticDelayedStart => write!(f, "AutomaticDelayedStart"),
            StartType::Manual => write!(f, "Manual"),
            StartType::Disabled => write!(f, "Disabled"),
        }
    }
}

impl std::fmt::Display for ServiceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ServiceStatus::Running => write!(f, "Running"),
            ServiceStatus::Stopped => write!(f, "Stopped"),
            ServiceStatus::Paused => write!(f, "Paused"),
            ServiceStatus::StartPending => write!(f, "StartPending"),
            ServiceStatus::StopPending => write!(f, "StopPending"),
            ServiceStatus::PausePending => write!(f, "PausePending"),
            ServiceStatus::ContinuePending => write!(f, "ContinuePending"),
        }
    }
}

impl std::fmt::Display for ErrorControl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorControl::Ignore => write!(f, "Ignore"),
            ErrorControl::Normal => write!(f, "Normal"),
            ErrorControl::Severe => write!(f, "Severe"),
            ErrorControl::Critical => write!(f, "Critical"),
        }
    }
}

/// Represents an error from a Windows service operation.
#[derive(Debug)]
pub struct ServiceError {
    pub message: String,
}

impl std::fmt::Display for ServiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for ServiceError {}

impl From<String> for ServiceError {
    fn from(message: String) -> Self {
        Self { message }
    }
}
