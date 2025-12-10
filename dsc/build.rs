// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // TODO: We can save the compiled code so not every build needs protoc.
    // See: https://github.com/hyperium/tonic/blob/master/tonic-build/README.md
    tonic_prost_build::compile_protos("src/bicep/bicep.proto")?;
    Ok(())
}
