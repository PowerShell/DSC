// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::export::invoke_export;

mod error;
mod export;
mod metadata;
mod parser;
mod util;

fn main() {
    // only supports export for sshdconfig for now
    match invoke_export() {
        Ok(result) => {
            println!("{result}");
        }
        Err(e) => {
            eprintln!("Error exporting SSHD config: {e:?}");
        }
    }
}
