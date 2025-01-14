// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::config::{FileContent, Encoding};
use std::path::Path;
use std::io;
use tracing::{debug};

impl Encoding {
    pub fn to_encoding_rs(&self) -> Option<&'static encoding_rs::Encoding> {
        match self {
            Encoding::Utf8 => Some(encoding_rs::UTF_8),
            Encoding::Utf16 => Some(encoding_rs::UTF_16LE), // or UTF_16BE depending on your needs
            Encoding::Ascii => Some(encoding_rs::WINDOWS_1252), // ASCII is a subset of Windows-1252
            Encoding::Binary => None,
        }
    }
}

impl FileContent {
    /// Create a new `FileContent`.
    ///
    /// # Arguments
    ///
    /// * `string` - The string for the Path
    #[must_use]
    pub fn new(path: &str) -> FileContent {
        FileContent {
            path: path.to_string(),
            content: None,
            hash: None,
            encoding: Some(Encoding::Utf8),
            exist: None,
        }
    }
}

pub fn get_file_content(filecontent: &FileContent) -> Result<FileContent, Box<dyn std::error::Error>> {
    debug!("In get_file_content");
    match compare_filecontent_state(filecontent) {
        Ok(f) => {
            Ok(f)
        },
        Err(e) => {
            Err(e)?
        }
    }
}

pub fn set_file_content(filecontent: &FileContent) -> Result<FileContent, Box<dyn std::error::Error>> {
    // debug!("In set_file_content");
    // let path = Path::new(&filecontent.path);
    // let content = filecontent.content.as_ref().unwrap_or(&String::new());
    // let encoding = filecontent.encoding.unwrap_or(Encoding::Utf8).to_encoding_rs().unwrap_or(encoding_rs::UTF_8);
    // let mut file = fsFile::create(path)?;
    // let mut encoder = encoding.new_encoder();
    // let mut bytes = vec![0; content.len() * encoding.new_encoder().max_buffer_length()];
    // let (bytes_written, _, _) = encoder.encode_to_slice(content, &mut bytes, true);
    // file.write_all(&bytes[..bytes_written])?;
    // Ok(filecontent.clone())
    Ok(filecontent.clone())
}

pub fn delete_file_content(filecontent: &FileContent) -> Result<FileContent, Box<dyn std::error::Error>> {
    debug!("In delete_file_content");
    let path = Path::new(&filecontent.path);

    let mut filecontent_to_delete = filecontent.clone();

    if path.exists() {

        filecontent_to_delete.exist = Some(false);
        filecontent_to_delete.content = None;
        set_file_content(&filecontent)?;
    }

    Ok(filecontent_to_delete)
}

pub fn read_file_with_encoding(path: &Path, encoding: &'static encoding_rs::Encoding) -> io::Result<String> {
    let bytes = std::fs::read(path)?;
    let (decoded_str, _encoding_used, had_errors) = encoding.decode(&bytes);

    if had_errors {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid encoding"));
    }

    Ok(decoded_str.to_string())
}

pub fn compare_filecontent_state(filecontent: &FileContent) -> Result<FileContent, Box<dyn std::error::Error>> {
    debug!("In compare_filecontent_state");

    let rs_encoding = filecontent.encoding.as_ref().unwrap_or(&Encoding::Utf8).to_encoding_rs().unwrap_or(encoding_rs::UTF_8);

    let path = Path::new(&filecontent.path);
    if path.exists() {
        let content = read_file_with_encoding(path, rs_encoding)?;
        let content_hash = sha256::digest(content.as_bytes());
        let hash = filecontent.hash.as_ref().unwrap_or(&content_hash);

        match filecontent.hash.as_ref() {
            Some(h) => {
                if h.to_lowercase() == hash.to_lowercase() {
                    let mut updated_file_content = filecontent.clone();
                    updated_file_content.hash = Some(hash.to_string());
                    updated_file_content.content = Some(content);
                    updated_file_content.exist = Some(true);

                    return Ok(updated_file_content)
                }
                else {
                    return Err("Hash does not match")?;
                }
            },
            None => {
                let mut updated_file_content = filecontent.clone();
                updated_file_content.hash = Some(hash.to_string());
                updated_file_content.content = Some(content);
                updated_file_content.exist = Some(true);

                return Ok(updated_file_content)
            }
        }
    }
    else {
        match filecontent.exist {
            Some(true) | None => {
                return Err("File does not exist")?;
            },
            Some(false) => {
                return Ok(filecontent.clone());
            }
        }
    }
}
