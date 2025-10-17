// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::DscError;
use crate::configure::context::Context;
use crate::functions::{FunctionArgKind, FunctionCategory, FunctionMetadata};
use rust_i18n::t;
use serde_json::Value;
use std::fs;
use std::path::Path;
use base64::{Engine as _, engine::general_purpose};

const MAX_FILE_SIZE: u64 = 96 * 1024; // 96 KB

#[derive(Debug, Default)]
pub struct LoadFileAsBase64 {}

impl super::Function for LoadFileAsBase64 {
    fn get_metadata(&self) -> FunctionMetadata {
        FunctionMetadata {
            name: "loadFileAsBase64".to_string(),
            description: t!("functions.loadFileAsBase64.description").to_string(),
            category: vec![FunctionCategory::String],
            min_args: 1,
            max_args: 1,
            accepted_arg_ordered_types: vec![vec![FunctionArgKind::String]],
            remaining_arg_accepted_types: None,
            return_types: vec![FunctionArgKind::String],
        }
    }

    fn invoke(&self, args: &[Value], _context: &Context) -> Result<Value, DscError> {
        let Some(file_path_str) = args[0].as_str() else {
            return Err(DscError::Parser(t!("functions.loadFileAsBase64.notString").to_string()));
        };

        let path = Path::new(file_path_str);
        
        if !path.exists() {
            return Err(DscError::Parser(t!("functions.loadFileAsBase64.fileNotFound", filePath = file_path_str).to_string()));
        }

        if !path.is_file() {
            return Err(DscError::Parser(t!("functions.loadFileAsBase64.notAFile", filePath = file_path_str).to_string()));
        }

        let metadata = fs::metadata(path).map_err(|e| {
            DscError::Parser(t!("functions.loadFileAsBase64.cannotReadMetadata", filePath = file_path_str, error = e.to_string()).to_string())
        })?;

        if metadata.len() > MAX_FILE_SIZE {
            return Err(DscError::Parser(t!("functions.loadFileAsBase64.fileTooLarge", filePath = file_path_str, maxSize = MAX_FILE_SIZE.to_string()).to_string()));
        }

        let file_contents = fs::read(path).map_err(|e| {
            DscError::Parser(t!("functions.loadFileAsBase64.cannotReadFile", filePath = file_path_str, error = e.to_string()).to_string())
        })?;

        let encoded = general_purpose::STANDARD.encode(&file_contents);
        
        Ok(Value::String(encoded))
    }
}

#[cfg(test)]
mod tests {
    use crate::configure::context::Context;
    use crate::parser::Statement;
    use std::fs;
    use base64::{Engine as _, engine::general_purpose};

    #[test]
    fn test_load_file_as_base64_basic() {
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("test_base64.txt");
        
        let test_content = b"Hello, World!";
        fs::write(&test_file, test_content).unwrap();
        
        let mut parser = Statement::new().unwrap();
        let file_path = test_file.to_str().unwrap();
        let expression = format!("[loadFileAsBase64('{}')]", file_path);
        let result = parser.parse_and_execute(&expression, &Context::new()).unwrap();
        
        let expected = general_purpose::STANDARD.encode(test_content);
        assert_eq!(result, expected);
        fs::remove_file(test_file).ok();
    }

    #[test]
    fn test_load_file_as_base64_empty_file() {
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("test_empty.txt");
        
        fs::write(&test_file, b"").unwrap();
        
        let mut parser = Statement::new().unwrap();
        let file_path = test_file.to_str().unwrap();
        let expression = format!("[loadFileAsBase64('{}')]", file_path);
        let result = parser.parse_and_execute(&expression, &Context::new()).unwrap();
        
        assert_eq!(result, "");
        fs::remove_file(test_file).ok();
    }

    #[test]
    fn test_load_file_as_base64_binary_content() {
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("test_binary.bin");
        
        let binary_content: Vec<u8> = vec![0x00, 0xFF, 0x42, 0xAA, 0x55];
        fs::write(&test_file, &binary_content).unwrap();
        
        let mut parser = Statement::new().unwrap();
        let file_path = test_file.to_str().unwrap();
        let expression = format!("[loadFileAsBase64('{}')]", file_path);
        let result = parser.parse_and_execute(&expression, &Context::new()).unwrap();
        
        let expected = general_purpose::STANDARD.encode(&binary_content);
        assert_eq!(result, expected);
        fs::remove_file(test_file).ok();
    }

    #[test]
    fn test_load_file_as_base64_file_not_found() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[loadFileAsBase64('/nonexistent/file.txt')]", &Context::new());
        
        assert!(result.is_err());
    }

    #[test]
    fn test_load_file_as_base64_file_too_large() {
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("test_large.bin");
        
        // Create a file larger than 96 KB
        let large_content = vec![0u8; 97 * 1024]; // 97 KB
        fs::write(&test_file, &large_content).unwrap();
        
        let mut parser = Statement::new().unwrap();
        let file_path = test_file.to_str().unwrap();
        let expression = format!("[loadFileAsBase64('{}')]", file_path);
        let result = parser.parse_and_execute(&expression, &Context::new());
        
        assert!(result.is_err());
        fs::remove_file(test_file).ok();
    }

    #[test]
    fn test_load_file_as_base64_round_trip() {
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("test_roundtrip.txt");
        
        let original_content = "The quick brown fox jumps over the lazy dog";
        fs::write(&test_file, original_content.as_bytes()).unwrap();
        
        let mut parser = Statement::new().unwrap();
        let file_path = test_file.to_str().unwrap();
        let expression = format!("[base64ToString(loadFileAsBase64('{}'))]", file_path);
        let result = parser.parse_and_execute(&expression, &Context::new()).unwrap();
        
        assert_eq!(result, original_content);
        fs::remove_file(test_file).ok();
    }
}
