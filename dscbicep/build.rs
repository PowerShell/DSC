// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_prost_build::configure()
    .build_client(false)
    // TODO: Configure and commit the out_dir to avoid dependency on protoc
    // .out_dir(out_dir)
    .compile_protos(&["proto/bicep.proto"], &["proto"])?;
    Ok(())
}
