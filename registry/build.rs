#[cfg(windows)]
fn main() {
    // Prevent this build script from rerunning unnecessarily.
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rustc-link-lib=onecore_apiset");
    println!("cargo:rustc-link-lib=onecoreuap_apiset");
    static_vcruntime::metabuild();
}

#[cfg(not(windows))]
fn main() {
    // Prevent this build script from rerunning unnecessarily.
    println!("cargo:rerun-if-changed=build.rs");
}
