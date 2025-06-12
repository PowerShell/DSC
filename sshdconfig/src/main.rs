// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use schemars::schema_for;
use serde_json::to_string_pretty;
use std::{env::args, process::exit};

use crate::export::invoke_export;
use crate::parser::SshdConfigParser;

mod error;
mod export;
mod metadata;
mod parser;
mod util;

fn main() {

    // TODO: add support for other commands and use clap for argument parsing
    let args: Vec<String> = args().collect();

    if args.len() != 2 || (args[1] != "export" && args[1] != "schema") {
        eprintln!("Usage: {} <export|schema>", args[0]);
        exit(1);
    }

    if args[1] == "schema" {
        // for dsc tests on linux/mac
        let schema = schema_for!(SshdConfigParser);
        println!("{}", to_string_pretty(&schema).unwrap());
        return;
    }

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
