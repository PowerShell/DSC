use miette::Diagnostic;
use thiserror::Error;
use rust_i18n::t;

#[derive(Error, Debug, Diagnostic)]
pub enum DscSettingsError {
    #[error("{t}", t = t!(
        "settings.errors.invalidDataFileMultipleErrors",
        path = file_path,
        err = errors.iter().map(|e| e.to_string()).collect::<Vec<_>>().join(", ")
    ))]
    InvalidDataFileMultipleErrors {
        file_path: String,
        #[related]
        errors: Vec<DscSettingsError>,
    },
    #[error("{t}: {0}", t = t!("settings.errors.invalidIgnoreSettingsFileEnvVar"))]
    InvalidIgnoreSettingsFileEnvVar(String),
    #[error("{t}: {0}", t = t!("settings.errors.invalidTraceLevel"))]
    InvalidTraceLevel(String),
    #[error("{t}: {0}", t = t!("settings.errors.invalidTraceFormat"))]
    InvalidTraceFormat(String),
    /// The settings file could not be read.
    #[error("{t}", t = t!("settings.errors.fileReadError", file_path = file_path))]
    FileReadError {
        file_path: String,
        #[source]
        source: std::io::Error,
    },
    /// The settings file could not be written.
    #[error("{t}", t = t!("settings.errors.fileWriteError", file_path = file_path))]
    FileWriteError{
        file_path: String,
        #[source]
        source: std::io::Error,
    },
    /// The settings file could not be found.
    #[error("{t}", t = t!("settings.errors.fileNotFound", path = file_path))]
    FileNotFound{
        file_path: String,
    },
    /// The settings file could not be parsed.
    #[error("{t}", t = t!("settings.errors.parseDataFileError", path = file_path, err = source))]
    ParseDataFileError{
        file_path: String,
        #[source]
        source: serde_json::Error,
    },
    /// Indicates an error when parsing a boolean environment variable.
    #[error("{t}", t = t!("settings.errors.parseBooleanEnvVarError", value = value))]
    ParseBooleanEnvVarError{
        value: String,
    },
    /// Multiple errors occurred while loading settings.
    #[error("{t}: {0:?}", t = t!("settings.errors.loadMultipleErrors"))]
    LoadMultipleErrors(Vec<DscSettingsError>),
    /// Indicates an error when loading an environment variable.
    #[error("{t}", t = t!("settings.errors.loadEnvironmentError"))]
    LoadEnvironmentError{
        env_var: &'static str,
        #[source]
        source: Box<DscSettingsError>,
    },
    /// Multiple errors occurred while loading settings from environment variables.
    #[error("{t}: {0:?}", t = t!("settings.errors.loadEnvironmentMultipleErrors"))]
    LoadEnvironmentMultipleErrors(Vec<DscSettingsError>),
}