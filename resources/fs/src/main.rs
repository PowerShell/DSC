// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use args::Args;
use clap::Parser;
use std::process::exit;
use tracing::{debug, error};
use crate::config::{File};
use file_helper::{get_file, delete_file};
use schemars::schema_for;

mod args;
pub mod config;
mod file_helper;

const EXIT_SUCCESS: i32 = 0;
const EXIT_INVALID_INPUT: i32 = 2;

fn main() {
    let args = Args::parse();
    match args.subcommand {
        args::SubCommand::Get { input } => {
            debug!("Getting file at path: {}", input);
            let file: File = match serde_json::from_str(input.as_str()) {
                Ok(input) => input,
                Err(e) => return Err(e).unwrap(),
            };

            match get_file(&file) {
                Ok(file) => {
                    let json = serde_json::to_string(&file).unwrap();
                    println!("{}", json);
                }
                Err(e) => {
                    error!("Failed to get file: {}", e);
                    exit(EXIT_INVALID_INPUT);
                }
            }
        }
        args::SubCommand::Delete { input } => {
            debug!("Deleting file at path: {}", input);
            let file: File = match serde_json::from_str(input.as_str()) {
                Ok(input) => input,
                Err(e) => return Err(e).unwrap(),
            };

            match delete_file(&file) {
                Ok(_) => {
                    debug!("File deleted successfully.");
                }
                Err(e) => {
                    error!("Failed to delete file: {}", e);
                    exit(EXIT_INVALID_INPUT);
                }
            }
        }
        // args::SubCommand::Export { path } => {
        //     println!("Exporting file at path: {}", path);
        //     match export_path(&path) {
        //         Ok(dir) => {
        //             let json = serde_json::to_string(&dir).unwrap();
        //             println!("{}", json);
        //             debug!("File exported successfully.");
        //         }
        //         Err(e) => {
        //             println!("Failed to export file: {}", e);
        //             exit(EXIT_INVALID_INPUT);
        //         }
        //     }
        //     // Export the file or directory

        // }
        args::SubCommand::Schema => {
            debug!("Retrieving JSON schema.");
            let schema = schema_for!(File);
            let json =serde_json::to_string(&schema).unwrap();
            println!("{json}");

            // let schema = schema_for!(Directory);
            // let json = serde_json::to_string(&schema).unwrap();
            // println!("{json}");
        }
    }

    exit(EXIT_SUCCESS);
}