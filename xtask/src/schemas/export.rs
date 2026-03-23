// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use std::{fs, ops::Add, path::PathBuf, sync::LazyLock};

use dsc_lib::schemas::{
    dsc_repo::{DscRepoSchema, RecognizedSchemaVersion},
    schema_utility_extensions::SchemaUtilityExtensions
};
use rust_i18n::t;
use schemars::Schema;
use thiserror::Error;

/// Defines the errors to raise when exporting a DSC schema to the file system.
#[derive(Debug, Error)]
pub(crate) enum SchemaExportError {
    /// Raised when a schema fails to serialize to a pretty-formatted string.
    #[error("{t}: {0}", t = t!("schemas.export.serializationFailure"))]
    SerializationFailure(#[from]serde_json::Error),
    /// Raised when an IO error prevents exporting a schema to the file system.
    #[error("{t}: {0}", t = t!("schemas.export.ioError"))]
    IOError(#[from] std::io::Error),
}

/// Helper static to retrieve the root folder once and use repeatedly when exporting schemas to the
/// filesystem.
static PROJECT_DIR: LazyLock<PathBuf> = LazyLock::new(|| {
    let p = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    p.parent().unwrap().into()
});

/// Writes the given JSON Schema to the filesystem relative to the project folder.
pub(crate) fn write_schema(relative_path: PathBuf, schema: Schema) -> Result<(), SchemaExportError> {
    
    let json_schema = serde_json::to_string_pretty(&schema.to_value_with_stable_order())?.add("\n");
    let path = PROJECT_DIR.clone().join("schemas").join(relative_path);
    let folder = path.parent().unwrap();
    println!("Exporting schema at '{}'", path.display());

    if !folder.exists() {
        fs::create_dir_all(folder)?;
    }
    fs::write(path, json_schema)?;

    Ok(())
}

macro_rules! export_type_schemas {
    ($schema_version:expr => $($type_to_export:ty),+) => {
        {
            $(
                for schema_form in <$type_to_export>::get_valid_schema_forms() {
                    write_schema(
                        <$type_to_export>::get_schema_relative_path($schema_version, schema_form).into(),
                        <$type_to_export>::generate_exportable_schema($schema_version, schema_form)
                    )?;
                }
            )+
        }
    };
}

pub(crate) fn export_schemas(
    schema_version: RecognizedSchemaVersion
) -> Result<(), SchemaExportError> {
    export_type_schemas!(
        schema_version =>
        dsc_lib::configure::config_doc::Configuration,
        dsc_lib::configure::config_doc::DataType,
        dsc_lib::configure::config_doc::ExecutionKind,
        dsc_lib::configure::config_doc::Metadata,
        dsc_lib::configure::config_doc::Operation,
        dsc_lib::configure::config_doc::Output,
        dsc_lib::configure::config_doc::Parameter,
        dsc_lib::configure::config_doc::Resource,
        dsc_lib::configure::config_doc::ResourceDiscoveryMode,
        dsc_lib::configure::config_doc::RestartRequired,
        dsc_lib::configure::config_doc::SecurityContextKind,
        dsc_lib::configure::config_doc::UserFunction,
        dsc_lib::configure::config_doc::UserFunctionDefinition,
        dsc_lib::configure::config_doc::UserFunctionOutput,
        dsc_lib::configure::config_doc::UserFunctionParameter,
        dsc_lib::configure::config_result::ConfigurationExportResult,
        dsc_lib::configure::config_result::ConfigurationGetResult,
        dsc_lib::configure::config_result::ConfigurationSetResult,
        dsc_lib::configure::config_result::ConfigurationTestResult,
        dsc_lib::configure::config_result::ResourceGetResult,
        dsc_lib::configure::config_result::ResourceMessage,
        dsc_lib::configure::config_result::ResourceSetResult,
        dsc_lib::dscresources::adapted_resource_manifest::AdaptedDscResourceManifest,
        dsc_lib::dscresources::dscresource::Capability,
        dsc_lib::dscresources::dscresource::DscResource,
        dsc_lib::dscresources::invoke_result::DeleteResult,
        dsc_lib::dscresources::invoke_result::DeleteWhatIfResult,
        dsc_lib::dscresources::invoke_result::ExportResult,
        dsc_lib::dscresources::invoke_result::GetResult,
        dsc_lib::dscresources::invoke_result::ResolveResult,
        dsc_lib::dscresources::invoke_result::ResourceGetResponse,
        dsc_lib::dscresources::invoke_result::ResourceSetResponse,
        dsc_lib::dscresources::invoke_result::ResourceTestResponse,
        dsc_lib::dscresources::invoke_result::SetResult,
        dsc_lib::dscresources::invoke_result::TestResult,
        dsc_lib::dscresources::invoke_result::ValidateResult,
        dsc_lib::dscresources::resource_manifest::Adapter,
        dsc_lib::dscresources::resource_manifest::DeleteMethod,
        dsc_lib::dscresources::resource_manifest::ExportMethod,
        dsc_lib::dscresources::resource_manifest::GetArgKind,
        dsc_lib::dscresources::resource_manifest::GetMethod,
        dsc_lib::dscresources::resource_manifest::InputKind,
        dsc_lib::dscresources::resource_manifest::Kind,
        dsc_lib::dscresources::resource_manifest::ResolveMethod,
        dsc_lib::dscresources::resource_manifest::ResourceManifest,
        dsc_lib::dscresources::resource_manifest::ReturnKind,
        dsc_lib::dscresources::resource_manifest::SchemaKind,
        dsc_lib::dscresources::resource_manifest::SetDeleteArgKind,
        dsc_lib::dscresources::resource_manifest::SetMethod,
        dsc_lib::dscresources::resource_manifest::TestMethod,
        dsc_lib::dscresources::resource_manifest::ValidateMethod,
        dsc_lib::extensions::discover::DiscoverMethod,
        dsc_lib::extensions::discover::DiscoverResult,
        dsc_lib::extensions::dscextension::Capability,
        dsc_lib::extensions::dscextension::DscExtension,
        dsc_lib::extensions::extension_manifest::ExtensionManifest,
        dsc_lib::extensions::import::ImportMethod,
        dsc_lib::extensions::secret::SecretMethod,
        dsc_lib::functions::FunctionArgKind,
        dsc_lib::functions::FunctionCategory,
        dsc_lib::functions::FunctionDefinition,
        dsc_lib::types::ExitCodesMap,
        dsc_lib::types::FullyQualifiedTypeName,
        dsc_lib::types::ResourceVersion,
        dsc_lib::types::ResourceVersionReq,
        dsc_lib::types::SemanticVersion,
        dsc_lib::types::SemanticVersionReq,
        dsc_lib::types::TagList
    );

    Ok(())
}
