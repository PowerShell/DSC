// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! The definition for [`RecognizedSchemaVersion`] is generated with the `build.rs` script. The build
//! script depends on the `.versions.json` and `.versions.ps1` files in the crate root. The script
//! checks the git tags for non-prerelease versions of DSC to generate the enum type with all of the
//! correct values. The enum can be used transparently throughout the rest of the libraries.

use rust_i18n::t;
use thiserror::Error;

include!(concat!(env!("OUT_DIR"), "/recognized_schema_version.rs"));

/// Defines the error when parsing a string that isn't a recognized schema version folder.
#[derive(Error, Debug, Clone, PartialEq)]
#[error(
    "{t}: {0}. {t2}: {1:?}",
    t = t!("dsc_repo.recognized_schema_version.unrecognizedVersion"),
    t2 = t!("dsc_repo.recognized_schema_version.validVersionsAre")
)]
pub struct UnrecognizedSchemaVersion(pub String, pub Vec<String>);

impl std::str::FromStr for RecognizedSchemaVersion {
    type Err = UnrecognizedSchemaVersion;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let candidate = s.trim();
        Self::all()
            .into_iter()
            .find(|version| version.to_string().eq_ignore_ascii_case(candidate))
            .ok_or_else(|| UnrecognizedSchemaVersion(
                candidate.to_string(),
                Self::all().iter().map(ToString::to_string).collect()
            ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_str_round_trips_every_recognized_version() {
        for version in RecognizedSchemaVersion::all() {
            let parsed: RecognizedSchemaVersion = version.to_string().parse().unwrap();
            assert_eq!(parsed, version);
        }
    }

    #[test]
    fn from_str_is_case_insensitive_and_trims() {
        let parsed: RecognizedSchemaVersion = " VNEXT ".parse().unwrap();
        assert_eq!(parsed, RecognizedSchemaVersion::VNext);
    }

    #[test]
    fn from_str_rejects_unrecognized_versions() {
        assert!("v99.0.0".parse::<RecognizedSchemaVersion>().is_err());
        assert!("not-a-version".parse::<RecognizedSchemaVersion>().is_err());
    }
}
