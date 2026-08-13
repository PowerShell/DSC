// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::types::{ExportState, FileContent, FileState};
use rust_i18n::t;
use sha2::{Digest, Sha256, Sha512};
use std::fs::{self, File};
use std::io::{BufReader, Read};
use std::path::Path;

pub fn get(input: &FileContent) -> Result<FileState, String> {
    validate_input(input)?;
    read_state(&input.path)
}

pub fn set(input: &FileContent) -> Result<FileState, String> {
    validate_input(input)?;
    let path = Path::new(&input.path);

    if input.exist == Some(false) {
        match fs::remove_file(path) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => {
                return Err(t!(
                    "set.removeError",
                    path = input.path.as_str(),
                    error = error.to_string()
                )
                .to_string());
            }
        }
        return read_state(&input.path);
    }

    let Some(content) = input.content.as_deref() else {
        return Err(t!("set.contentRequired").to_string());
    };
    validate_content_hashes(input, content)?;

    fs::write(path, content.as_bytes()).map_err(|error| {
        t!(
            "set.writeError",
            path = input.path.as_str(),
            error = error.to_string()
        )
        .to_string()
    })?;

    read_state(&input.path)
}

pub fn test(input: &FileContent) -> Result<FileState, String> {
    validate_input(input)?;
    let mut actual = read_state(&input.path)?;

    let in_desired_state = if input.exist == Some(false) {
        !actual.exist
    } else if !actual.exist {
        false
    } else {
        desired_hashes(input)
            .iter()
            .all(|(algorithm, desired)| match *algorithm {
                HashAlgorithm::Sha256 => actual
                    .sha256
                    .as_deref()
                    .is_some_and(|value| value.eq_ignore_ascii_case(desired)),
                HashAlgorithm::Sha512 => actual
                    .sha512
                    .as_deref()
                    .is_some_and(|value| value.eq_ignore_ascii_case(desired)),
            })
    };

    actual.in_desired_state = Some(in_desired_state);
    Ok(actual)
}

pub fn export(input: &FileContent) -> Result<ExportState, String> {
    validate_input(input)?;
    let state = read_state(&input.path)?;
    if !state.exist {
        return Ok(ExportState {
            path: state.path,
            content: None,
            sha256: None,
            sha512: None,
            exist: false,
        });
    }

    let bytes = fs::read(&input.path).map_err(|error| {
        t!(
            "export.readError",
            path = input.path.as_str(),
            error = error.to_string()
        )
        .to_string()
    })?;
    let content = String::from_utf8(bytes).map_err(|error| {
        t!(
            "export.readError",
            path = input.path.as_str(),
            error = error.to_string()
        )
        .to_string()
    })?;
    let (sha256, sha512) = hash_bytes(content.as_bytes());

    Ok(ExportState {
        path: state.path,
        content: Some(content),
        sha256: Some(sha256),
        sha512: Some(sha512),
        exist: true,
    })
}

#[derive(Clone, Copy)]
enum HashAlgorithm {
    Sha256,
    Sha512,
}

fn desired_hashes(input: &FileContent) -> Vec<(HashAlgorithm, String)> {
    let mut hashes = Vec::with_capacity(2);
    if let Some(content) = input.content.as_deref() {
        let (sha256, _) = hash_bytes(content.as_bytes());
        hashes.push((HashAlgorithm::Sha256, sha256));
    }
    if let Some(sha256) = input.sha256.as_ref() {
        hashes.push((HashAlgorithm::Sha256, sha256.clone()));
    }
    if let Some(sha512) = input.sha512.as_ref() {
        hashes.push((HashAlgorithm::Sha512, sha512.clone()));
    }
    hashes
}

fn validate_content_hashes(input: &FileContent, content: &str) -> Result<(), String> {
    let (sha256, sha512) = hash_bytes(content.as_bytes());
    if input
        .sha256
        .as_deref()
        .is_some_and(|desired| !sha256.eq_ignore_ascii_case(desired))
    {
        return Err(t!("set.sha256Mismatch").to_string());
    }
    if input
        .sha512
        .as_deref()
        .is_some_and(|desired| !sha512.eq_ignore_ascii_case(desired))
    {
        return Err(t!("set.sha512Mismatch").to_string());
    }
    Ok(())
}

fn validate_input(input: &FileContent) -> Result<(), String> {
    if input.path.is_empty() {
        return Err(t!("input.emptyPath").to_string());
    }
    validate_hash(input.sha256.as_deref(), 64, "sha256")?;
    validate_hash(input.sha512.as_deref(), 128, "sha512")
}

fn validate_hash(value: Option<&str>, length: usize, name: &str) -> Result<(), String> {
    if let Some(value) = value
        && (value.len() != length || !value.bytes().all(|byte| byte.is_ascii_hexdigit()))
    {
        return Err(t!("input.invalidHash", name = name, length = length).to_string());
    }
    Ok(())
}

fn read_state(path: &str) -> Result<FileState, String> {
    let file = match File::open(path) {
        Ok(file) => file,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(FileState {
                path: path.to_string(),
                sha256: None,
                sha512: None,
                exist: false,
                in_desired_state: None,
            });
        }
        Err(error) => {
            return Err(t!("get.readError", path = path, error = error.to_string()).to_string());
        }
    };

    let (sha256, sha512) = hash_reader(file)
        .map_err(|error| t!("get.readError", path = path, error = error.to_string()).to_string())?;
    Ok(FileState {
        path: path.to_string(),
        sha256: Some(sha256),
        sha512: Some(sha512),
        exist: true,
        in_desired_state: None,
    })
}

fn hash_reader(file: File) -> std::io::Result<(String, String)> {
    let mut reader = BufReader::new(file);
    let mut sha256 = Sha256::new();
    let mut sha512 = Sha512::new();
    let mut buffer = vec![0_u8; 64 * 1024];
    loop {
        let count = reader.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        sha256.update(&buffer[..count]);
        sha512.update(&buffer[..count]);
    }
    Ok((
        format!("{:x}", sha256.finalize()),
        format!("{:x}", sha512.finalize()),
    ))
}

fn hash_bytes(bytes: &[u8]) -> (String, String) {
    let mut sha256 = Sha256::new();
    let mut sha512 = Sha512::new();
    sha256.update(bytes);
    sha512.update(bytes);
    (
        format!("{:x}", sha256.finalize()),
        format!("{:x}", sha512.finalize()),
    )
}
