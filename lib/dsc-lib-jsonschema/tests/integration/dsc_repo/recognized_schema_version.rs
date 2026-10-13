// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use dsc_lib_jsonschema::dsc_repo::RecognizedSchemaVersion;

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
