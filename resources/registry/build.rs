// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

fn main() {
    // Prevent this build script from rerunning unnecessarily.
    println!("cargo:rerun-if-changed=build.rs");
}
