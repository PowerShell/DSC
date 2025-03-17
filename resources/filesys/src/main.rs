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
const EXIT_INVALID_INPUT: i32 = 1;
const EXIT_JSON_SERIALIZATION_FAILED: i32 = 2;

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
                            match get_file(&parsed_file) {
                                Ok(file) => {
                                    match serde_json::to_string(&file) {
                                        Ok(json) => println!("{}", json),
                                        Err(e) => {
                                            error!("Failed to serialize file: {}", e);
                                            exit(EXIT_JSON_SERIALIZATION_FAILED);
                                        }
                                    }
                                }
                                Err(e) => {
                                    error!("Failed to get file: {}", e);
                                    exit(EXIT_INVALID_INPUT);
                                }
                            }
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
                           match get_dir(&parsed_directory) {
                                Ok(dir) => {
                                    match serde_json::to_string(&dir) {
                                        Ok(json) => println!("{}", json),
                                        Err(e) => {
                                            error!("Failed to serialize directory: {}", e);
                                            exit(EXIT_JSON_SERIALIZATION_FAILED);
                                        }
                                    }
                                }
                                Err(e) => {
                                    error!("Failed to get directory: {}", e);
                                    exit(EXIT_INVALID_INPUT);
                                }
                           }
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
                            match get_file_content(&parsed_filecontent) {
                                Ok(filecontent) => {
                                    match serde_json::to_string(&filecontent) {
                                        Ok(json) => println!("{}", json),
                                        Err(e) => {
                                            error!("Failed to serialize file content: {}", e);
                                            exit(EXIT_JSON_SERIALIZATION_FAILED);
                                        }
                                    }
                                }
                                Err(e) => {
                                    error!("Failed to get file content: {}", e);
                                    exit(EXIT_INVALID_INPUT);
                                }
                            }
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
                            match delete_file(&parsed_file) {
                                Ok(file) => {
                                    match serde_json::to_string(&file) {
                                        Ok(json) => println!("{}", json),
                                        Err(e) => {
                                            error!("Failed to serialize file: {}", e);
                                            exit(EXIT_INVALID_INPUT);
                                        }
                                    }
                                }
                                Err(e) => {
                                    error!("Failed to delete file: {}", e);
                                    exit(EXIT_INVALID_INPUT);
                                }
                            }
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
                            match delete_dir(&parsed_directory) {
                                Ok(dir) => {
                                    match serde_json::to_string(&dir) {
                                        Ok(json) => println!("{}", json),
                                        Err(e) => {
                                            error!("Failed to serialize directory: {}", e);
                                            exit(EXIT_INVALID_INPUT);
                                        }
                                    }
                                }
                                Err(e) => {
                                    error!("Failed to delete directory: {}", e);
                                    exit(EXIT_INVALID_INPUT);
                                }
                            }
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
                            match delete_file_content(&parsed_filecontent) {
                                Ok(filecontent) => {
                                    match serde_json::to_string(&filecontent) {
                                        Ok(json) => println!("{}", json),
                                        Err(e) => {
                                            error!("Failed to serialize file content: {}", e);
                                            exit(EXIT_INVALID_INPUT);
                                        }
                                    }
                                }
                                Err(e) => {
                                    error!("Failed to delete file content: {}", e);
                                    exit(EXIT_INVALID_INPUT);
                                }
                            }
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
                            match set_file(&parsed_file) {
                                Ok(file) => {
                                    match serde_json::to_string(&file) {
                                        Ok(json) => println!("{}", json),
                                        Err(e) => {
                                            error!("Failed to serialize file: {}", e);
                                            exit(EXIT_INVALID_INPUT);
                                        }
                                    }
                                }
                                Err(e) => {
                                    error!("Failed to set file: {}", e);
                                    exit(EXIT_INVALID_INPUT);
                                }
                            }
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
                            match set_dir(&parsed_directory) {
                                Ok(dir) => {
                                    match serde_json::to_string(&dir) {
                                        Ok(json) => println!("{}", json),
                                        Err(e) => {
                                            error!("Failed to serialize directory: {}", e);
                                            exit(EXIT_INVALID_INPUT);
                                        }
                                    }
                                }
                                Err(e) => {
                                    error!("Failed to set directory: {}", e);
                                    exit(EXIT_INVALID_INPUT);
                                }
                            }
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
                            match set_file_content(&parsed_filecontent) {
                                Ok(filecontent) => {
                                    match serde_json::to_string(&filecontent) {
                                        Ok(json) => println!("{}", json),
                                        Err(e) => {
                                            error!("Failed to serialize file content: {}", e);
                                            exit(EXIT_INVALID_INPUT);
                                        }
                                    }
                                }
                                Err(e) => {
                                    error!("Failed to set file content: {}", e);
                                    exit(EXIT_INVALID_INPUT);
                                }
                            }
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
                            match export_file_path(&parsed_file) {
                                Ok(file) => {
                                    match serde_json::to_string(&file) {
                                        Ok(json) => println!("{}", json),
                                        Err(e) => {
                                            error!("Failed to serialize file: {}", e);
                                            exit(EXIT_INVALID_INPUT);
                                        }
                                    }
                                }
                                Err(e) => {
                                    error!("Failed to export file: {}", e);
                                    exit(EXIT_INVALID_INPUT);
                                }
                            }
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
                            match export_dir_path(&parsed_directory) {
                                Ok(dir) => {
                                    match serde_json::to_string(&dir) {
                                        Ok(json) => println!("{}", json),
                                        Err(e) => {
                                            error!("Failed to serialize directory: {}", e);
                                            exit(EXIT_INVALID_INPUT);
                                        }
                                    }
                                }
                                Err(e) => {
                                    error!("Failed to export directory: {}", e);
                                    exit(EXIT_INVALID_INPUT);
                                }
                            }
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
                            match get_file_content(&parsed_filecontent) {
                                Ok(filecontent) => {
                                    match serde_json::to_string(&filecontent) {
                                        Ok(json) => println!("{}", json),
                                        Err(e) => {
                                            error!("Failed to serialize file content: {}", e);
                                            exit(EXIT_INVALID_INPUT);
                                        }
                                    }
                                }
                                Err(e) => {
                                    error!("Failed to export file content: {}", e);
                                    exit(EXIT_INVALID_INPUT);
                                }
                            }
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
                    match serde_json::to_string(&schema) {
                        Ok(json) => println!("{}", json),
                        Err(e) => {
                            error!("Failed to serialize file schema: {}", e);
                            exit(EXIT_JSON_SERIALIZATION_FAILED);
                        }
                    }
                }
                args::FileSystemObjectType::Directory => {
                    let schema = schema_for!(Directory);
                    match serde_json::to_string(&schema){
                        Ok(json) => println!("{}", json),
                        Err(e) => {
                            error!("Failed to serialize directory schema: {}", e);
                            exit(EXIT_JSON_SERIALIZATION_FAILED);
                        }
                    }
                }
                args::FileSystemObjectType::FileContent => {
                    let schema = schema_for!(FileContent);
                    match serde_json::to_string(&schema) {
                        Ok(json) => println!("{}", json),
                        Err(e) => {
                            error!("Failed to serialize file content schema: {}", e);
                            exit(EXIT_JSON_SERIALIZATION_FAILED);
                        }
                    }
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