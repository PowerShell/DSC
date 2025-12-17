// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use std::{env, path::PathBuf};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let descriptor_path = PathBuf::from(env::var("OUT_DIR").unwrap()).join("bicep.bin");

    tonic_prost_build::configure()
        .build_client(false)
        .file_descriptor_set_path(&descriptor_path)
        // TODO: Configure and commit the out_dir to avoid dependency on protoc
        // .out_dir(out_dir)
        .compile_protos(&["proto/bicep.proto"], &["proto"])?;
    Ok(())
}
