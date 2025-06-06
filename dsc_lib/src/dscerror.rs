// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use rust_i18n::t;
use std::str::Utf8Error;

use indicatif::style::TemplateError;
use thiserror::Error;
use tracing::error;
use tree_sitter::LanguageError;

#[derive(Error, Debug)]
pub enum DscError {
    #[error("{t}: {0}", t = t!("dscerror.adapterNotFound"))]
    AdapterNotFound(String),

    #[error("{t}: {0}", t = t!("dscerror.booleanConversion"))]
    BooleanConversion(#[from] std::str::ParseBoolError),

    #[error("{t} '{0}' [{t2} {1}] {2}", t = t!("dscerror.commandResource"), t2 = t!("dscerror.exitCode"))]
    Command(String, i32, String),

    #[error("{t} '{0}' [{t2} {1}] {2}", t = t!("dscerror.commandExecutable"), t2 = t!("dscerror.exitCode"))]
    CommandExit(String, i32, String),

    #[error("{t} '{0}' [{t2} {1}] {t3}: {2}", t = t!("dscerror.commandResource"), t2 = t!("dscerror.exitCode"), t3 = t!("dscerror.manifestDescription"))]
    CommandExitFromManifest(String, i32, String),

    #[error("{t} {0} {t2} '{1}'", t = t!("dscerror.commandOperation"), t2 = t!("dscerror.forExecutable"))]
    CommandOperation(String, String),

    #[error("{t} '{0}' {t2}: {1}", t = t!("dscerror.function"), t2 = t!("dscerror.error"))]
    Function(String, String),

    #[error("{t} '{0}' {t2}: {1}", t = t!("dscerror.function"), t2 = t!("dscerror.error"))]
    FunctionArg(String, String),

    #[error("{t}: {0}", t = t!("dscerror.integerConversion"))]
    IntegerConversion(#[from] std::num::ParseIntError),

    #[error("{t}:\n{0}", t = t!("dscerror.invalidConfiguration"))]
    InvalidConfiguration(String),

    #[error("{t}: {0}.  {t2}: {1}", t = t!("dscerror.unsupportedManifestVersion"), t2 = t!("dscerror.mustBe"))]
    InvalidManifestSchemaVersion(String, String),

    #[error("{t} '{0}', {t2} {1}, {t3} {2}", t = t!("dscerror.invalidFunctionParameterCount"), t2 = t!("dscerror.expected"), t3 = t!("dscerror.got"))]
    InvalidFunctionParameterCount(String, usize, usize),

    #[error("IO: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON: {0}")]
    Json(#[from] serde_json::Error),

    #[error("{t}: {0}", t = t!("dscerror.language"))]
    Language(#[from] LanguageError),

    #[error("{t}: {0}\nJSON: {1}", t = t!("dscerror.manifest"))]
    Manifest(String, serde_json::Error),

    #[error("{t}: {0}\nYAML: {1}", t = t!("dscerror.manifest"))]
    ManifestYaml(String, serde_yaml::Error),

    #[error("{t}: {0}", t = t!("dscerror.missingManifest"))]
    MissingManifest(String),

    #[error("{t} '{0}' {t2} '{1}'", t = t!("dscerror.adapterBasedResource"), t2 = t!("dscerror.missingRequires"))]
    MissingRequires(String, String),

    #[error("{t}: {0}", t = t!("dscerror.schemaMissing"))]
    MissingSchema(String),

    #[error("{t}: {0}", t = t!("dscerror.notImplemented"))]
    NotImplemented(String),

    #[error("{t}: {0}", t = t!("dscerror.notSupported"))]
    NotSupported(String),

    #[error("{t}: {0}", t = t!("dscerror.numberConversion"))]
    NumberConversion(#[from] std::num::TryFromIntError),

    #[error("{t}: {0}", t = t!("dscerror.operation"))]
    Operation(String),

    #[error("{t}: {0}", t = t!("dscerror.parser"))]
    Parser(String),

    #[error("{t}: {0}", t = t!("dscerror.progress"))]
    Progress(#[from] TemplateError),

    #[error("{t}: {0}", t = t!("dscerror.resourceNotFound"))]
    ResourceNotFound(String),

    #[error("{t}: {0}", t = t!("dscerror.resourceManifestNotFound"))]
    ResourceManifestNotFound(String),

    #[error("{t}: {0}", t = t!("dscerror.schema"))]
    Schema(String),

    #[error("{t}: {0}", t = t!("dscerror.schemaNotAvailable"))]
    SchemaNotAvailable(String),

    #[error("{t}: {0}", t = t!("dscerror.securityContext"))]
    SecurityContext(String),

    #[error("{t}: {0}", t = t!("dscerror.utf8Conversion"))]
    Utf8Conversion(#[from] Utf8Error),

    #[error("{t}: {code:?} {message:?}", t = t!("dscerror.unknown"))]
    Unknown {
        code: i32,
        message: String,
    },

    #[error("{t}: {0}.  {t2}: {1:?}", t = t!("dscerror.unrecognizedSchemaUri"), t2 = t!("dscerror.validSchemaUrisAre"))]
    UnrecognizedSchemaUri(String, Vec<String>),

    #[error("{t}: {0}", t = t!("dscerror.validation"))]
    Validation(String),

    #[error("YAML: {0}")]
    Yaml(#[from] serde_yaml::Error),

    #[error("{t}: {0}", t = t!("dscerror.setting"))]
    Setting(String),
}
