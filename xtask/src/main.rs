// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use clap::Parser;
use dsc_lib::schemas::dsc_repo::RecognizedSchemaVersion;
use rust_i18n::{i18n, t};
use thiserror::Error;

use crate::{
    args::{Args, SchemaSubCommand, SubCommand},
    schemas::export::{SchemaExportError, export_schemas}
};

mod args;
pub(crate) mod schemas {
    pub(crate) mod export;
}

#[derive(Debug, Error)]
pub(crate) enum XTaskError {
    #[error(transparent)]
    SchemaExport(#[from] SchemaExportError),
    #[error("{t}: {0}", t = t!("main.invalidReleaseVersion"))]
    InvalidReleaseVersion(String),
    #[error("{t}: {0}", t = t!("main.unrecognizedReleaseFolder"))]
    UnrecognizedReleaseFolder(String),
}

i18n!("locales", fallback = "en-us");

fn main() -> Result<(), XTaskError> {
    let args = Args::parse();

    match args.subcommand {
        SubCommand::Schema { sub_command } => match sub_command {
            SchemaSubCommand::Export { schema_versions, release } => {
                for schema_version in resolve_export_versions(schema_versions, release.as_deref())? {
                    export_schemas(schema_version)?;
                }
                Ok(())
            },
        },
    }
}

fn resolve_export_versions(
    schema_versions: Vec<RecognizedSchemaVersion>,
    release: Option<&str>
) -> Result<Vec<RecognizedSchemaVersion>, XTaskError> {
    let Some(release) = release else {
        return Ok(if schema_versions.is_empty() {
            vec![RecognizedSchemaVersion::VNext]
        } else {
            schema_versions
        });
    };

    let version = release.trim().trim_start_matches('v');
    let segments: Vec<&str> = version.split('.').collect();
    let is_numeric = |segment: &&str| !segment.is_empty() && segment.chars().all(|c| c.is_ascii_digit());
    if segments.len() != 3 || !segments.iter().all(is_numeric) {
        return Err(XTaskError::InvalidReleaseVersion(release.to_string()));
    }

    let folders = [
        format!("v{}.{}.{}", segments[0], segments[1], segments[2]),
        format!("v{}.{}", segments[0], segments[1]),
        format!("v{}", segments[0]),
    ];
    folders.iter().map(|folder| {
        folder.parse::<RecognizedSchemaVersion>()
            .map_err(|_| XTaskError::UnrecognizedReleaseFolder(folder.clone()))
    }).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_defaults_to_vnext() {
        let versions = resolve_export_versions(Vec::new(), None).unwrap();
        assert_eq!(versions, vec![RecognizedSchemaVersion::VNext]);
    }

    #[test]
    fn resolve_passes_through_explicit_versions() {
        let requested = vec![RecognizedSchemaVersion::VNext, RecognizedSchemaVersion::default()];
        let versions = resolve_export_versions(requested.clone(), None).unwrap();
        assert_eq!(versions, requested);
    }

    #[test]
    fn resolve_release_expands_to_patch_minor_and_major_folders() {
        let latest = RecognizedSchemaVersion::latest().to_string();
        let release = latest.trim_start_matches('v').to_string();
        let versions = resolve_export_versions(Vec::new(), Some(&release)).unwrap();
        assert_eq!(versions.len(), 3);
        assert_eq!(versions[0].to_string(), latest);
    }

    #[test]
    fn resolve_release_rejects_partial_versions() {
        assert!(matches!(
            resolve_export_versions(Vec::new(), Some("3.2")),
            Err(XTaskError::InvalidReleaseVersion(_))
        ));
    }

    #[test]
    fn resolve_release_rejects_unrecognized_versions() {
        assert!(matches!(
            resolve_export_versions(Vec::new(), Some("99.0.0")),
            Err(XTaskError::UnrecognizedReleaseFolder(_))
        ));
    }
}
