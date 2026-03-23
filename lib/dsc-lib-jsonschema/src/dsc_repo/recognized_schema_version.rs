// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! The definition for [`RecognizedSchemaVersion`] is generated with the `build.rs` script. The build
//! script depends on the `.versions.json` and `.versions.ps1` files in the crate root. The script
//! checks the git tags for non-prerelease versions of DSC to generate the enum type with all of the
//! correct values. The enum can be used transparently throughout the rest of the libraries.

include!(concat!(env!("OUT_DIR"), "/recognized_schema_version.rs"));
