// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::config::File;
use crate::config::Directory;
use std::fs;
use std::fs::File as fsFile;
use std::path::Path;
use tracing::{debug};
use fs_extra::dir::get_size;

pub fn get_file(file: &File) -> Result<File, Box<dyn std::error::Error>> {
    match compare_file_state(file) {
        Ok(f) => {
            Ok(f)
        },
        Err(e) => {
            Err(e)?
        }
    }
}

pub fn export_path(path: &str) -> Result<Directory, Box<dyn std::error::Error>> {
    // Export the file or directory
    let path = Path::new(path);

    match path.exists() {
        false => {
            return Ok(Directory { path: path.to_str().unwrap().to_string(), size: 0, files: vec![], exist: Some(false) });
        }
        _ => {}
    }

    match path.is_dir() {
        true => {
            let files: Vec<File> = {
                let dir = fs::read_dir(path)?;
                let mut files = Vec::new();
                for entry in dir {
                    let entry = entry?;
                    let path = entry.path();
                    files.push(get_file(path.to_str().unwrap())?);
                }
                files
            };

            let dir_size = get_size(path)?;

            Ok(Directory { path: path.to_str().unwrap().to_string(), size: dir_size, files, exist: Some(true) })
        }
        false => {
            let file = get_file(path.to_str().unwrap())?;
            let parent = path.parent();
            match parent {
                Some(parent) => {
                    Ok(Directory { path: parent.to_str().unwrap().to_string(), size: file.size, files: vec![file], exist: Some(true) })
                }
                _ => {
                    return Err("Path is not a file or directory")?;
                }
            }
        }
    }
}

pub fn delete_file(file: &File) -> Result<(), Box<dyn std::error::Error>> {
    match compare_file_state(file) {
        Ok(f) => {
            debug!("Deleting file: {:?}", f.path);
            fs::remove_file(f.path)?;
            Ok(())
        },
        Err(e) => {
            Err(e)?
        }
    }
}

fn compare_file_state(file: &File) -> Result<File, Box<dyn std::error::Error>> {
    let resolved_path = Path::new(file.path.as_str());
    debug!("Resolved path: {:?}", resolved_path);
    match resolved_path.is_dir() {
        true => {
            debug!("Path is a directory");
            let mut updated_file = file.clone();
            updated_file.exist = Some(false);
            return Ok(updated_file)
        }
        false => {}
    }
    let f: fsFile = match fsFile::open(resolved_path) {
        Ok(f) => {
            debug!("File found: {:?}", file.path);
            f
        },
        Err(e) => {
            if e.kind() == std::io::ErrorKind::NotFound {
                debug!("File not found: {:?}", file.path);
                let mut updated_file = file.clone();
                updated_file.exist = Some(false);
                return Ok(updated_file)
            } else {
                return Err(e)?
            }
        }
    };

    let hash = calculate_hash(file.path.as_str())?;

    if hash.to_lowercase() != file.hash.to_lowercase() {
        debug!("Hash mismatch: {:?}", file.path);
        let mut updated_file = file.clone();
        updated_file.exist = Some(false);
        return Ok(updated_file)
    }

    let metadata = f.metadata()?;
    let mut updated_file = file.clone();
    updated_file.size = Some(metadata.len());
    updated_file.exist = Some(true);

    Ok(updated_file)
}

fn calculate_hash(path: &str) -> Result<String, Box<dyn std::error::Error>> {
    let bytes = fs::read(path)?;
    let digest = sha256::digest(&bytes);
    Ok(digest)
}
