// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use args::Args;
use clap::Parser;
use std::process::exit;
use tracing::{debug, error};
use crate::config::{File, Directory, FileContent};
use file_helper::{get_file, set_file, delete_file, export_file_path};
use dir_helpers::{get_dir, set_dir, delete_dir, export_dir_path};
use filecontent_helper::{get_file_content, delete_file_content};
use schemars::schema_for;

mod args;
pub mod config;
mod file_helper;
mod dir_helpers;
mod filecontent_helper;

const EXIT_SUCCESS: i32 = 0;
const EXIT_INVALID_INPUT: i32 = 2;

fn main() {
    let args = Args::parse();
    match args.subcommand {
        args::SubCommand::Get { input } => {
            debug!("Getting at path: {}", input);

            match is_file_type(input.as_str()) {
                Some(file) => {
                    let file = get_file(&file).unwrap();
                    let json = serde_json::to_string(&file).unwrap();
                    println!("{}", json);
                }
                None => {
                    let dir = match is_directory_type(input.as_str()) {
                        Some(dir) => {
                            let dir = get_dir(&dir).unwrap();
                            let json = serde_json::to_string(&dir).unwrap();
                            println!("{}", json);
                        }
                        None => {
                            let filecontent = match is_fillecontent_type(input.as_str()) {
                                Some(filecontent) => {
                                    let filecontent = get_file_content(&filecontent).unwrap();
                                    let json = serde_json::to_string(&filecontent).unwrap();
                                    println!("{}", json);
                                }
                                None => {
                                    error!("Invalid input.");
                                    exit(EXIT_INVALID_INPUT);
                                }
                            };
                        }
                    };
                }
            };
        }
        args::SubCommand::Delete { input } => {
            debug!("Deleting file at path: {}", input);

            match is_file_type(input.as_str()) {
                Some(file) => {
                    let file = delete_file(&file).unwrap();
                    let json = serde_json::to_string(&file).unwrap();
                    println!("{}", json);
                }
                None => {
                    let dir = match is_directory_type(input.as_str()) {
                        Some(dir) => {
                            let dir = delete_dir(&dir).unwrap();
                            let json = serde_json::to_string(&dir).unwrap();
                            println!("{}", json);
                        }
                        None => {
                            let filecontent = match is_fillecontent_type(input.as_str()) {
                                Some(filecontent) => {
                                    let filecontent = delete_file_content(&filecontent).unwrap();
                                    let json = serde_json::to_string(&filecontent).unwrap();
                                    println!("{}", json);
                                }
                                None => {
                                    error!("Invalid input.");
                                    exit(EXIT_INVALID_INPUT);
                                }
                            };
                        }
                    };
                }
            };
        }
        args::SubCommand::Set { input } => {
            debug!("Setting file at path: {}", input);

            match is_file_type(input.as_str()) {
                Some(file) => {
                    let file = set_file(&file).unwrap();
                    let json = serde_json::to_string(&file).unwrap();
                    println!("{}", json);
                    debug!("File set successfully.");
                }
                None => {
                    let dir = match is_directory_type(input.as_str()) {
                        Some(dir) => {
                            let dir = set_dir(&dir).unwrap();
                            let json = serde_json::to_string(&dir).unwrap();
                            println!("{}", json);
                        }
                        None => {
                            let filecontent = match is_fillecontent_type(input.as_str()) {
                                Some(filecontent) => {
                                    // let filecontent = get_file(&filecontent).unwrap();
                                    // let json = serde_json::to_string(&filecontent).unwrap();
                                    // println!("{}", json);
                                }
                                None => {
                                    error!("Invalid input.");
                                    exit(EXIT_INVALID_INPUT);
                                }
                            };
                        }
                    };
                }
            };
        }
        args::SubCommand::Export { input } => {
            debug!("Exporting file at path: {}", input);

            match is_file_type(input.as_str()) {
                Some(file) => {
                    let file = export_file_path(&file).unwrap();
                    let json = serde_json::to_string(&file).unwrap();
                    println!("{}", json);
                    debug!("File exported successfully.");
                }
                None => {
                    let dir = match is_directory_type(input.as_str()) {
                        Some(dir) => {
                            let exported_dir = export_dir_path(&dir).unwrap();
                            let json = serde_json::to_string(&exported_dir).unwrap();
                            println!("{}", json);
                            debug!("File exported successfully.");
                        }
                        None => {
                            let filecontent = match is_fillecontent_type(input.as_str()) {
                                Some(filecontent) => {
                                    // let filecontent = get_file(&filecontent).unwrap();
                                    // let json = serde_json::to_string(&filecontent).unwrap();
                                    // println!("{}", json);
                                }
                                None => {
                                    error!("Invalid input.");
                                    exit(EXIT_INVALID_INPUT);
                                }
                            };
                        }
                    };
                }
            };
        }
        args::SubCommand::Schema { schema_type }=> {
            match schema_type {
                args::FileSystemObjectType::File => {
                    let schema = schema_for!(File);
                    let json = serde_json::to_string(&schema).unwrap();
                    println!("{}", json);
                }
                args::FileSystemObjectType::Directory => {
                    let schema = schema_for!(Directory);
                    let json = serde_json::to_string(&schema).unwrap();
                    println!("{}", json);
                }
                args::FileSystemObjectType::FileContent => {
                    let schema = schema_for!(FileContent);
                    let json = serde_json::to_string(&schema).unwrap();
                    println!("{}", json);
                }
            }
        }
    }

    exit(EXIT_SUCCESS);
}

fn is_file_type(input: &str) -> Option<File> {
    let file: File = match serde_json::from_str(input) {
        Ok(input) => input,
        Err(_) => return None,
    };

    Some(file)
}

fn is_directory_type(input: &str) -> Option<Directory> {
    let dir: Directory = match serde_json::from_str(input) {
        Ok(input) => input,
        Err(_) => return None,
    };

    Some(dir)
}

fn is_fillecontent_type(input: &str) -> Option<FileContent> {
    let filecontent: FileContent = match serde_json::from_str(input) {
        Ok(input) => input,
        Err(_) => return None,
    };

    Some(filecontent)
}