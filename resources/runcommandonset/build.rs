// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

fn main() {
    // Always rebuild if translations are updated.
    println!("cargo::rerun-if-changed=locales");

    if std::env::var("CARGO_CFG_TARGET_OS").unwrap() == "windows" {
        let res = winresource::WindowsResource::new();
        res.compile().unwrap();
    }
}
