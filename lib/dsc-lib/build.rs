// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

fn main() {
    // Always rebuild if translations are updated.
    println!("cargo:rerun-if-changed=locales");
}
