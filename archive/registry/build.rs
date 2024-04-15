// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[cfg(onecore)]
fn main() {
    // Prevent this build script from rerunning unnecessarily.
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rustc-link-lib=onecore_apiset");
    println!("cargo:rustc-link-lib=onecoreuap_apiset");
    static_vcruntime::metabuild();
}

#[cfg(not(onecore))]
fn main() {
    // Prevent this build script from rerunning unnecessarily.
    println!("cargo:rerun-if-changed=build.rs");
    static_vcruntime::metabuild();
}
