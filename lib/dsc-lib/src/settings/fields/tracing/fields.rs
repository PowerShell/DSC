use std::{fmt::Display, str::FromStr};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::settings::DscSettingsError;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", try_from = "String", into = "String")]
pub enum TraceLevelField {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

impl FromStr for TraceLevelField {
    type Err = DscSettingsError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "error" => Ok(TraceLevelField::Error),
            "warn" => Ok(TraceLevelField::Warn),
            "info" => Ok(TraceLevelField::Info),
            "debug" => Ok(TraceLevelField::Debug),
            "trace" => Ok(TraceLevelField::Trace),
            _ => Err(DscSettingsError::InvalidTraceLevel(s.to_string())),
        }
    }
}

impl TryFrom<String> for TraceLevelField {
    type Error = DscSettingsError;

    fn try_from(value: String) -> Result<Self, <Self as TryFrom<String>>::Error> {
        TraceLevelField::from_str(&value)
    }
}

impl From<TraceLevelField> for String {
    fn from(level: TraceLevelField) -> Self {
        level.to_string()
    }
}

impl Display for TraceLevelField {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let format_str = match self {
            TraceLevelField::Error => "error",
            TraceLevelField::Warn => "warn",
            TraceLevelField::Info => "info",
            TraceLevelField::Debug => "debug",
            TraceLevelField::Trace => "trace",
        };
        write!(f, "{}", format_str)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", try_from = "String", into = "String")]
pub enum TraceFormatField {
    Default,
    Plaintext,
    Json,
}

impl FromStr for TraceFormatField {
    type Err = DscSettingsError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "default" => Ok(TraceFormatField::Default),
            "plaintext" => Ok(TraceFormatField::Plaintext),
            "json" => Ok(TraceFormatField::Json),
            _ => Err(DscSettingsError::InvalidTraceFormat(s.to_string())),
        }
    }
}

impl Display for TraceFormatField {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let format_str = match self {
            TraceFormatField::Default => "default",
            TraceFormatField::Plaintext => "plaintext",
            TraceFormatField::Json => "json",
        };
        write!(f, "{}", format_str)
    }
}

impl From<TraceFormatField> for String {
    fn from(format: TraceFormatField) -> Self {
        format.to_string()
    }
}

impl TryFrom<String> for TraceFormatField {
    type Error = DscSettingsError;

    fn try_from(value: String) -> Result<Self, <Self as TryFrom<String>>::Error> {
        TraceFormatField::from_str(&value)
    }
}

impl From<TraceLevelField> for tracing::Level {
    fn from(level: TraceLevelField) -> Self {
        match level {
            TraceLevelField::Error => tracing::Level::ERROR,
            TraceLevelField::Warn => tracing::Level::WARN,
            TraceLevelField::Info => tracing::Level::INFO,
            TraceLevelField::Debug => tracing::Level::DEBUG,
            TraceLevelField::Trace => tracing::Level::TRACE,
        }
    }
}

impl From<tracing::Level> for TraceLevelField {
    fn from(level: tracing::Level) -> Self {
        match level {
            tracing::Level::ERROR => TraceLevelField::Error,
            tracing::Level::WARN => TraceLevelField::Warn,
            tracing::Level::INFO => TraceLevelField::Info,
            tracing::Level::DEBUG => TraceLevelField::Debug,
            tracing::Level::TRACE => TraceLevelField::Trace,
        }
    }
}
