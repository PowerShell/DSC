// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_prost_build::compile_protos("proto/bicep.proto")?;
    Ok(())
}
