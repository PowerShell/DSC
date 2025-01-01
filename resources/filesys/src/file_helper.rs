// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::config::File;
use crate::config::Directory;
use std::fs;
use std::fs::File as fsFile;
use std::path::Path;
use tracing::{debug};
use fs_extra::dir::get_size;

impl File {
    /// Create a new `File`.
    ///
    /// # Arguments
    ///
    /// * `string` - The string for the Path
    #[must_use]
    pub fn new(path: &str) -> File {
        File {
            path: path.to_string(),
            size: None,
            hash: None,
            exist: None,
        }
    }
}

impl Directory {
    /// Create a new `Directory`.
    ///
    /// # Arguments
    ///
    /// * `string` - The string for the Path
    #[must_use]
    pub fn new(path: &str) -> Directory {
        Directory {
            path: path.to_string(),
            size: None,
            files: None,
            recurse: false,
            exist: None,
        }
    }
}

pub fn get_file(file: &File) -> Result<File, Box<dyn std::error::Error>> {
    debug!("In get_file");
    match compare_file_state(file) {
        Ok(f) => {
            Ok(f)
        },
        Err(e) => {
            Err(e)?
        }
    }
}

pub fn set_file(file: &File) -> Result<File, Box<dyn std::error::Error>> {
    match compare_file_state(file) {
        Ok(_) => {
            debug!("In set_file");
            debug!("file exist {:?}", file_exists(file.path.as_str()));
            debug!("expected file exist {:?}", file.exist.unwrap_or(true));

            match (file_exists(file.path.as_str()), file.exist.unwrap_or(true)) {
                // if the current file exists and expected state is exist == false, delete it
                (true, false) => {
                    debug!("Deleting file: {:?}", file.path);
                    fs::remove_file(file.path.as_str())?;
                    Ok(get_file(&file)?)
                }

                // if the current file does not exist and expected state is exist == true, create it
                (false, true) => {
                    debug!("Creating file: {:?}", file.path);
                    fsFile::create(file.path.as_str())?;
                    let new_file = File::new(file.path.as_str());

                    Ok(get_file(&new_file)?)
                }

                // if the current file exists and expected state is exist == true or both are false update and return
                (true, true) | (false, false) => {
                    debug!("Updating file: {:?}", file.path);
                    let new_file = File::new(file.path.as_str());
                    Ok(get_file(&new_file)?)
                }
            }
        },
        Err(e) => {
            Err(e)?
        }
    }
}

pub fn export_file_path(file: &File) -> Result<File, Box<dyn std::error::Error>> {
    match compare_file_state(file) {
        Ok(f) => {
            Ok(f)
        },
        Err(e) => {
            Err(e)?
        }
    }
}

pub fn export_dir_path(dir: &Directory) -> Result<Directory, Box<dyn std::error::Error>> {
    // Export the file or directory
    let path = Path::new(dir.path.as_str());

    match path.exists() {
        false => {
            return Ok(Directory { path: path.to_str().unwrap().to_string(), size: None, files: None, recurse: dir.recurse, exist: Some(false) });
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
                    let f = File::new(path.to_str().unwrap());
                    files.push(get_file(&f)?);
                }
                files
            };

            let dir_size = get_size(path)?;

            Ok(Directory { path: path.to_str().unwrap().to_string(), size: Some(dir_size), files: Some(files), recurse: dir.recurse, exist: Some(true) })
        }
        false => {
            let path = Path::new(path);
            let f = File::new(path.to_str().unwrap());
            let file = get_file(&f)?;
            let parent = path.parent();
            match parent {
                Some(parent) => {
                    Ok(Directory { path: parent.to_str().unwrap().to_string(), size: file.size, files: vec![file].into(), recurse: dir.recurse, exist: Some(true) })
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
            // debug!("Path is a directory");
            // let mut updated_file = file.clone();
            // updated_file.exist = Some(false);
            // return Ok(updated_file)
            return Err("Path is a directory")?
        }
        false => {}
    }
    let f: fsFile = match fsFile::open(resolved_path) {
        Ok(f) => {
            debug!("File found: {:?}", file.path);
            f
        },
        Err(e) => {
            debug!("Error: {:?}", e);
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

    match file.hash.as_ref() {
        Some(h) => {
            if h.to_lowercase() != hash.to_lowercase() {
                debug!("Hash mismatch: {:?}", file.path);
                let mut updated_file = file.clone();
                updated_file.exist = Some(false);
                return Ok(updated_file)
            }
            else {
                let metadata = f.metadata()?;
                let mut updated_file = file.clone();
                updated_file.size = Some(metadata.len());
                updated_file.exist = Some(true);
                return Ok(updated_file)
            }
         }
        None => {
            let metadata = f.metadata()?;
            let mut updated_file = file.clone();
            updated_file.hash = Some(hash);
            updated_file.size = Some(metadata.len());
            updated_file.exist = Some(true);
            return Ok(updated_file)
        }
    };
}

fn calculate_hash(path: &str) -> Result<String, Box<dyn std::error::Error>> {
    let bytes = fs::read(path)?;
    let digest = sha256::digest(&bytes);
    Ok(digest)
}

fn file_exists(path: &str) -> bool {
    let resolved_path = Path::new(path);
    return resolved_path.exists();
}
