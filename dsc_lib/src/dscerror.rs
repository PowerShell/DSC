// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use reqwest::StatusCode;
use thiserror::Error;
use chrono::{Local, DateTime};
use tracing::{error, warn};

#[derive(Error, Debug)]
pub enum DscError {
    #[error("Command: Resource '{0}' [Exit code {1}] {2}")]
    Command(String, i32, String),

    #[error("CommandOperation: {0} for executable '{1}'")]
    CommandOperation(String, String),

    #[error("HTTP: {0}")]
    Http(#[from] reqwest::Error),

    #[error("HTTP status: {0}")]
    HttpStatus(StatusCode),

    #[error("Regex: {0}")]
    Regex(#[from] regex::Error),

    #[error("Invalid configuration:\n{0}")]
    InvalidConfiguration(String),

    #[error("Unsupported manifest version: {0}.  Must be: {1}")]
    InvalidManifestSchemaVersion(String, String),

    #[error("IO: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Manifest: {0}\nJSON: {1}")]
    Manifest(String, serde_json::Error),

    #[error("Missing manifest: {0}")]
    MissingManifest(String),

    #[error("Provider source '{0}' missing 'requires' property for resource '{1}'")]
    MissingRequires(String, String),

    #[error("Schema missing from manifest: {0}")]
    MissingSchema(String),

    #[error("Not implemented: {0}")]
    NotImplemented(String),

    #[error("Operation: {0}")]
    Operation(String),

    #[error("Resource not found: {0}")]
    ResourceNotFound(String),

    #[error("Schema: {0}")]
    Schema(String),

    #[error("No Schema: {0}")]
    SchemaNotAvailable(String),

    #[error("Unknown: {code:?} {message:?}")]
    Unknown {
        code: i32,
        message: String,
    },

    #[error("Validation: {0}")]
    Validation(String),
}

#[derive(Debug)]
#[derive(PartialEq)]
pub enum StreamMessageType {
    None = 0,
    Data = 1,
    Error = 2,
    Warning = 3,
    Verbose = 4,
    Custom = 5
}

pub struct StreamMessage {
    pub message: String,
    pub message_type: StreamMessageType,
    pub time: DateTime<Local>,
    pub resource_type_name: String,
    pub resource_path: String
}

impl Default for StreamMessage {
    fn default() -> Self {
        Self::new()
    }
}

impl StreamMessage {
    /// Create a new message
    #[must_use]
    pub fn new() -> Self {
        Self {
            message: String::new(),
            message_type: StreamMessageType::None,
            time: Local::now(),
            resource_type_name: String::new(),
            resource_path: String::new(),
        }
    }

    /// Create a new error message
    ///
    /// # Arguments
    ///
    /// * `message` - The message to display
    /// * `resource_type_name` - The name of the resource type
    /// * `resource_path` - The path to the resource
    ///
    /// # Returns
    ///
    /// * `StreamMessage` - The new message
    #[must_use]
    pub fn new_error(message: String, resource_type_name: Option<String>, resource_path: Option<String>) -> StreamMessage {
        StreamMessage {
            message,
            message_type: StreamMessageType::Error,
            time: Local::now(),
            resource_type_name: resource_type_name.unwrap_or("None".to_string()),
            resource_path: resource_path.unwrap_or("None".to_string())
        }
    }

    /// Create a new warning message
    ///
    /// # Arguments
    ///
    /// * `message` - The message to display
    /// * `resource_type_name` - The name of the resource type
    /// * `resource_path` - The path to the resource
    ///
    /// # Returns
    ///
    /// * `StreamMessage` - The new message
    #[must_use]
    pub fn new_warning(message: String, resource_type_name: Option<String>, resource_path: Option<String>) -> StreamMessage {
        StreamMessage {
            message,
            message_type: StreamMessageType::Warning,
            time: Local::now(),
            resource_type_name: resource_type_name.unwrap_or("None".to_string()),
            resource_path: resource_path.unwrap_or("None".to_string())
        }
    }

    /// Print the message to the console
    ///
    /// # Arguments
    ///
    /// * `error_format` - The format to use for error messages
    /// * `warning_format` - The format to use for warning messages
    ///
    /// # Errors
    ///
    /// * `DscError` - If there is an error writing to the console
    pub fn print(&self, error_format:&StreamMessageType, warning_format:&StreamMessageType) -> Result<(), DscError>{
        if self.message_type == StreamMessageType::Error
        {
            if error_format == &StreamMessageType::Error
            {
                error!("{:?} -> {} {}", error_format, self.resource_type_name, self.message);
            }
            else
            {
                warn!("{:?} -> {} {}", warning_format, self.resource_type_name, self.message);
            }
        }
        if self.message_type == StreamMessageType::Warning
        {
            if warning_format == &StreamMessageType::Error
            {
                error!("{:?} -> {} {}", warning_format, self.resource_type_name, self.message);
            }
            else
            {
                warn!("{:?} -> {} {}", warning_format, self.resource_type_name, self.message);
            }
        }

        Ok(())
    }
}
