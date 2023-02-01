// Copyright (C) Microsoft Corporation. All rights reserved.

use std::env;
use std::process::Command;
use cc::windows_registry;

// Environment variables used in this build file are documented at
// https://doc.rust-lang.org/cargo/reference/environment-variables.html#environment-variables-cargo-sets-for-build-scripts

fn main() {
    // Make import libs for API sets that are not in the SDK.
    println!("cargo:rustc-link-search={}", env::var("OUT_DIR").unwrap());

    for lib in [
        "ext-ms-win-cng-rng-l1-1-0",
    ] {
        make_import_lib(lib);
        println!("cargo:rustc-link-lib={}", lib);
    }
}

// Gets the path to the tool for building '.lib' file from the environment variable, if it's set.
fn get_tool_var(name: &str) -> Option<String> {
    let target = env::var("TARGET").unwrap().replace('-', "_");
    let var = format!("{}_{}", name, target);
    println!("cargo:rerun-if-env-changed={}", var);
    env::var(var)
        .or_else(|_| {
            println!("cargo:rerun-if-env-changed={}", name);
            env::var(name)
        }).ok()
}

fn make_import_lib(name: &str) {
    println!("cargo:rerun-if-changed={}.def", name);

    if let Some(dlltool) = get_tool_var("DLLTOOL") {
        // This branch is for cross compilation from WSL2.
        // When building from WSL2, the tool 'llvm-dlltool-11' will be used for producing the '.lib' file.
        // For details, see https://microsoft.visualstudio.com/HyperVCloud/_git/hvlite?path=/ci/setup_windows_cross.sh&version=GBmain&line=47&lineEnd=69&lineStartColumn=1&lineEndColumn=2&lineStyle=plain&_a=contents
        let mut dlltool = Command::new(dlltool);

        let arch = match env::var("CARGO_CFG_TARGET_ARCH").unwrap().as_str() {
            "x86" => "i386",
            "x86_64" => "i386:x86-64",
            "aarch64" => "arm64",
            a => panic!("unsupported architecture {}", a),
        };

        dlltool.args(["-d", &format!("{}.def", name)]);
        dlltool.args(["-m", arch]);
        dlltool.args([
            "-l",
            &format!("{}/{}.lib", env::var("OUT_DIR").unwrap(), name),
        ]);
        if !dlltool.spawn().unwrap().wait().unwrap().success() {
            panic!("dlltool failed");
        }
    } else {
        // Find the 'lib.exe' from Visual Studio tools' location.
        let mut lib = windows_registry::find(&env::var("TARGET").unwrap(), "lib.exe")
            .expect("cannot find lib.exe");

        let arch = match env::var("CARGO_CFG_TARGET_ARCH").unwrap().as_str() {
            "x86" => "X86",
            "x86_64" => "X64",
            "aarch64" => "ARM64",
            a => panic!("unsupported architecture {}", a),
        };

        lib.arg(format!("/machine:{}", arch));
        lib.arg(format!("/def:{}.def", name));
        lib.arg(format!(
            "/out:{}/{}.lib",
            env::var("OUT_DIR").unwrap(),
            name
        ));
        if !lib.spawn().unwrap().wait().unwrap().success() {
            panic!("lib.exe failed");
        }
    }
}
