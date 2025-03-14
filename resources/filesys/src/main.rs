// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use args::Args;
use clap::Parser;
use std::process::exit;
use tracing::{debug, error};
use crate::config::{File, Directory, FileContent};
use file_helper::{get_file, set_file, delete_file, export_file_path};
use dir_helpers::{get_dir, set_dir, delete_dir, export_dir_path};
use filecontent_helper::{get_file_content, set_file_content, delete_file_content};
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
        args::SubCommand::Get { input, schema_type } => {
            debug!("Getting at path: {}", input);
            match schema_type {
                args::FileSystemObjectType::File => {
                    debug!("Getting file at path: {}", input);
                    match parse_file(input) {
                        Some(parsed_file) => {
                            let file = get_file(&parsed_file).unwrap();
                            let json = serde_json::to_string(&file).unwrap();
                            println!("{}", json);
                        }
                        None => {
                            error!("Invalid input for file.");
                            exit(EXIT_INVALID_INPUT);
                        }
                    }
                }
                args::FileSystemObjectType::Directory => {
                    debug!("Getting directory at path: {}", input);
                    match parse_directory(input) {
                        Some(parsed_directory) => {
                            let dir = get_dir(&parsed_directory).unwrap();
                            let json = serde_json::to_string(&dir).unwrap();
                            println!("{}", json);
                        }
                        None => {
                            error!("Invalid input for directory.");
                            exit(EXIT_INVALID_INPUT);
                        }
                    }
                }
                args::FileSystemObjectType::FileContent => {
                    debug!("Getting file content at path: {}", input);
                    match parse_filecontent(input) {
                        Some(parsed_filecontent) => {
                            let filecontent = get_file_content(&parsed_filecontent).unwrap();
                            let json = serde_json::to_string(&filecontent).unwrap();
                            println!("{}", json);
                        }
                        None => {
                            error!("Invalid input for file content.");
                            exit(EXIT_INVALID_INPUT);
                        }
                    }
                }
            };
        }

        args::SubCommand::Delete { input, schema_type} => {
            debug!("Deleting file at path: {}", input);

            match schema_type {
                args::FileSystemObjectType::File => {
                    debug!("Deleting file at path: {}", input);
                    match parse_file(input) {
                        Some(parsed_file) => {
                            let file = delete_file(&parsed_file).unwrap();
                            let json = serde_json::to_string(&file).unwrap();
                            println!("{}", json);
                        }
                        None => {
                            error!("Invalid input for file.");
                            exit(EXIT_INVALID_INPUT);
                        }
                    }
                }
                args::FileSystemObjectType::Directory => {
                    debug!("Deleting directory at path: {}", input);
                    match parse_directory(input) {
                        Some(parsed_directory) => {
                            let dir = delete_dir(&parsed_directory).unwrap();
                            let json = serde_json::to_string(&dir).unwrap();
                            println!("{}", json);
                        }
                        None => {
                            error!("Invalid input for directory.");
                            exit(EXIT_INVALID_INPUT);
                        }
                    }
                }
                args::FileSystemObjectType::FileContent => {
                    debug!("Deleting file content at path: {}", input);
                    match parse_filecontent(input) {
                        Some(parsed_filecontent) => {
                            let filecontent = delete_file_content(&parsed_filecontent).unwrap();
                            let json = serde_json::to_string(&filecontent).unwrap();
                            println!("{}", json);
                        }
                        None => {
                            error!("Invalid input for file content.");
                            exit(EXIT_INVALID_INPUT);
                        }
                    }
                }
            };
        }
        args::SubCommand::Set { input, schema_type } => {
            debug!("Setting file at path: {}", input);
            match schema_type {
                args::FileSystemObjectType::File => {
                    debug!("Setting file at path: {}", input);
                    match parse_file(input) {
                        Some(parsed_file) => {
                            let file = set_file(&parsed_file).unwrap();
                            let json = serde_json::to_string(&file).unwrap();
                            println!("{}", json);
                        }
                        None => {
                            error!("Invalid input for file.");
                            exit(EXIT_INVALID_INPUT);
                        }
                    }
                }
                args::FileSystemObjectType::Directory => {
                    debug!("Setting directory at path: {}", input);
                    match parse_directory(input) {
                        Some(parsed_directory) => {
                            let dir = set_dir(&parsed_directory).unwrap();
                            let json = serde_json::to_string(&dir).unwrap();
                            println!("{}", json);
                        }
                        None => {
                            error!("Invalid input for directory.");
                            exit(EXIT_INVALID_INPUT);
                        }
                    }
                }
                args::FileSystemObjectType::FileContent => {
                    debug!("Setting file content at path: {}", input);
                    match parse_filecontent(input) {
                        Some(parsed_filecontent) => {
                            let filecontent = set_file_content(&parsed_filecontent).unwrap();
                            let json = serde_json::to_string(&filecontent).unwrap();
                            println!("{}", json);
                        }
                        None => {
                            error!("Invalid input for file content.");
                            exit(EXIT_INVALID_INPUT);
                        }
                    }
                }
            };
        }
        args::SubCommand::Export { input, schema_type } => {
            debug!("Exporting file at path: {}", input);

            match schema_type {
                args::FileSystemObjectType::File => {
                    debug!("Exporting file at path: {}", input);
                    match parse_file(input) {
                        Some(parsed_file) => {
                            let file = export_file_path(&parsed_file).unwrap();
                            let json = serde_json::to_string(&file).unwrap();
                            println!("{}", json);
                        }
                        None => {
                            error!("Invalid input for file.");
                            exit(EXIT_INVALID_INPUT);
                        }
                    }
                }
                args::FileSystemObjectType::Directory => {
                    debug!("Exporting directory at path: {}", input);
                    match parse_directory(input) {
                        Some(parsed_directory) => {
                            let dir = export_dir_path(&parsed_directory).unwrap();
                            let json = serde_json::to_string(&dir).unwrap();
                            println!("{}", json);
                        }
                        None => {
                            error!("Invalid input for directory.");
                            exit(EXIT_INVALID_INPUT);
                        }
                    }
                }
                args::FileSystemObjectType::FileContent => {
                    debug!("Exporting file content at path: {}", input);
                    match parse_filecontent(input) {
                        Some(parsed_filecontent) => {
                            let filecontent = get_file_content(&parsed_filecontent).unwrap();
                            let json = serde_json::to_string(&filecontent).unwrap();
                            println!("{}", json);
                        }
                        None => {
                            error!("Invalid input for file content.");
                            exit(EXIT_INVALID_INPUT);
                        }
                    }
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

fn parse_file(input: String) -> Option<File> {
    let file: File = match serde_json::from_str(input.to_string().as_str()) {
        Ok(input) => input,
        Err(_) => return None,
    };

    Some(file)
}

fn parse_directory(input: String) -> Option<Directory> {
    let dir: Directory = match serde_json::from_str(input.to_string().as_str()) {
        Ok(input) => input,
        Err(_) => return None,
    };

    Some(dir)
}

fn parse_filecontent(input: String) -> Option<FileContent> {
    let filecontent: FileContent = match serde_json::from_str(input.to_string().as_str()) {
        Ok(input) => input,
        Err(_) => return None,
    };

    Some(filecontent)
}