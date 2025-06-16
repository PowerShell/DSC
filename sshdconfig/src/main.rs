// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use args::{Args, Command};
use clap::{Parser};
use export::invoke_export;
use set::invoke_set;

mod args;
mod error;
mod export;
mod metadata;
mod parser;
mod set;
mod util;

fn main() {
    let args = Args::parse();
    match &args.command {
        Command::Export => {
            match invoke_export() {
                Ok(result) => {
                    println!("{result}");
                }
                Err(e) => {
                    eprintln!("Error exporting sshd_config: {e:?}");
                }
            }
        }
        Command::Set { input , subcommand} => {
            match invoke_set(input, subcommand) {
                Ok(()) => {
                    println!("success");
                }
                Err(e) => {
                    eprintln!("Error setting sshd_config: {e:?}");
                }
            }
        }
    }
}
