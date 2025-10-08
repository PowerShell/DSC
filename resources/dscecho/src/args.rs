// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use clap::Parser;

#[derive(Debug, Parser)]
#[clap(name = "dscecho", version = "1.0.0", about = "Debugging helper resource that echos the input.", long_about = None)]
pub struct Args {
    #[clap(short, long, help = "The input to the echo command as JSON.  If no input is provided the JSON schema for the input is returned.")]
    pub input: Option<String>,
}
