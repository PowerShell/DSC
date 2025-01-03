// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::config::File;
use crate::config::Directory;
use crate::file_helper::get_file;
use std::fs;
use std::path::Path;
use tracing::{debug};
use fs_extra::dir::get_size;

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
            recurse: Some(false),
            exist: None,
        }
    }
}

pub fn get_dir(dir: &Directory) -> Result<Directory, Box<dyn std::error::Error>> {
    debug!("In get_dir");
    match compare_dir_state(dir) {
        Ok(d) => {
            Ok(d)
        },
        Err(e) => {
            Err(e)?
        }
    }
}

pub fn set_dir(dir: &Directory) -> Result<Directory, Box<dyn std::error::Error>> {
    match compare_dir_state(dir) {
        Ok(current_dir) => {
            debug!("In set_dir");
            debug!("dir exist {:?}", dir.exist);
            debug!("expected dir exist {:?}", dir.exist.unwrap_or(true));

            match (current_dir.exist.unwrap_or(true), dir.exist.unwrap_or(true)) {
                // if the current dir exists and expected state is exist == true, do nothing
                (true, true) | (false, false) => {
                    return Ok(current_dir);
                }

                // if the current dir exists and expected state is exist == true, create it
                (true, false) => {
                    debug!("Deleting directory: {:?}", dir.path);

                    if dir.recurse.unwrap_or(false) {
                        fs::remove_dir_all(dir.path.as_str())?;
                    } else {
                        fs::remove_dir(dir.path.as_str())?;
                    }

                    return Ok(get_dir(&dir)?)
                }

                // if the current dir does not exist and expected state is exist == true, create it
                (false, true) => {
                    debug!("Creating directory: {:?}", dir.path);
                    fs::create_dir_all(dir.path.as_str())?;
                    return Ok(get_dir(&dir)?)
                }
            }
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

pub fn delete_dir(dir: &Directory) -> Result<(), Box<dyn std::error::Error>> {
    match compare_dir_state(dir) {
        Ok(d) => {

            if d.exist == Some(false) {
                return Ok(());
            }

            if d.recurse == Some(true) {
                debug!("Deleting directory: {:?}", d.path);
                fs::remove_dir_all(d.path)?;
                return Ok(());
            }
            else {
                debug!("Deleting directory: {:?}", d.path);
                fs::remove_dir(d.path)?;
                return Ok(());
            }
        },
        Err(e) => {
            Err(e)?
        }
    }
}

pub fn compare_dir_state(dir: &Directory) -> Result<Directory, Box<dyn std::error::Error>> {
    let path = Path::new(dir.path.as_str());

    match path.exists() {
        false => {
            return Ok(Directory { path: path.to_str().unwrap().to_string(), size: None, files: None, recurse: dir.recurse, exist: Some(false) });
        }
        true => {
            match path.is_dir() {
                false => {
                    return Err("Path is not a directory")?;
                }
                _ => {}
            }
        }
    }

    let dir_size = get_size(path)?;

    match dir.size {
        Some(size) => {
            if size != dir_size {
                Ok(Directory { path: path.to_str().unwrap().to_string(), size: Some(dir_size), files: None, recurse: dir.recurse, exist: Some(true) })
            } else {
                Ok(Directory { path: path.to_str().unwrap().to_string(), size: Some(dir_size), files: None, recurse: dir.recurse, exist: Some(true) })
            }
        }
        None => {
            Ok(Directory { path: path.to_str().unwrap().to_string(), size: Some(dir_size), files: None, recurse: dir.recurse, exist: Some(true) })
        }
    }
}