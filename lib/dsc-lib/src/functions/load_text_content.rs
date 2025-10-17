// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::DscError;
use crate::configure::context::Context;
use crate::functions::{FunctionArgKind, FunctionCategory, FunctionMetadata};
use rust_i18n::t;
use serde_json::Value;
use std::fs;
use std::path::Path;
use encoding_rs::{Encoding, WINDOWS_1252, UTF_8, UTF_16BE, UTF_16LE};

const MAX_CONTENT_SIZE: usize = 131_072; // 131072 characters

#[derive(Debug, Default)]
pub struct LoadTextContent {}

#[derive(Debug, Default)]
pub struct LoadTextContent {}

impl super::Function for LoadTextContent {
    fn get_metadata(&self) -> FunctionMetadata {
        FunctionMetadata {
            name: "loadTextContent".to_string(),
            description: t!("functions.loadTextContent.description").to_string(),
            category: vec![FunctionCategory::String],
            min_args: 1,
            max_args: 2,
            accepted_arg_ordered_types: vec![
                vec![FunctionArgKind::String],
                vec![FunctionArgKind::String],
            ],
            remaining_arg_accepted_types: None,
            return_types: vec![FunctionArgKind::String],
        }
    }

    fn invoke(&self, args: &[Value], _context: &Context) -> Result<Value, DscError> {
        let Some(file_path_str) = args[0].as_str() else {
            return Err(DscError::Parser(t!("functions.loadTextContent.notString").to_string()));
        };

        let encoding = if args.len() > 1 {
            let Some(encoding_str) = args[1].as_str() else {
                return Err(DscError::Parser(t!("functions.loadTextContent.invalidEncoding").to_string()));
            };
            parse_encoding(encoding_str)?
        } else {
            UTF_8
        };

        let path = Path::new(file_path_str);
        
        if !path.exists() {
            return Err(DscError::Parser(t!("functions.loadTextContent.fileNotFound", filePath = file_path_str).to_string()));
        }

        if !path.is_file() {
            return Err(DscError::Parser(t!("functions.loadTextContent.notAFile", filePath = file_path_str).to_string()));
        }

        let file_bytes = fs::read(path).map_err(|e| {
            DscError::Parser(t!("functions.loadTextContent.cannotReadFile", filePath = file_path_str, error = e.to_string()).to_string())
        })?;

        let (decoded, _encoding_used, had_errors) = encoding.decode(&file_bytes);
        
        if had_errors {
            return Err(DscError::Parser(t!("functions.loadTextContent.decodingError", filePath = file_path_str).to_string()));
        }

        let content = decoded.into_owned();

        if content.chars().count() > MAX_CONTENT_SIZE {
            return Err(DscError::Parser(t!("functions.loadTextContent.contentTooLarge", filePath = file_path_str, maxSize = MAX_CONTENT_SIZE.to_string()).to_string()));
        }

        Ok(Value::String(content))
    }
}

fn parse_encoding(encoding_str: &str) -> Result<&'static Encoding, DscError> {
    match encoding_str.to_lowercase().as_str() {
        "utf-8" => Ok(UTF_8),
        "utf-16" => Ok(UTF_16LE),
        "utf-16be" => Ok(UTF_16BE),
        "iso-8859-1" | "us-ascii" => Ok(WINDOWS_1252), // Per Encoding Standard, both map to windows-1252
        _ => Err(DscError::Parser(t!("functions.loadTextContent.unsupportedEncoding", encoding = encoding_str).to_string())),
    }
}

#[cfg(test)]
mod tests {
    use crate::configure::context::Context;
    use crate::parser::Statement;
    use std::fs;

    #[test]
    fn test_load_text_content_basic_utf8() {
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("test_text.txt");
        
        let test_content = "Hello, World!";
        fs::write(&test_file, test_content).unwrap();
        
        let mut parser = Statement::new().unwrap();
        let file_path = test_file.to_str().unwrap();
        let expression = format!("[loadTextContent('{}')]", file_path);
        let result = parser.parse_and_execute(&expression, &Context::new()).unwrap();
        
        assert_eq!(result, test_content);
        fs::remove_file(test_file).ok();
    }

    #[test]
    fn test_load_text_content_with_utf8_explicit() {
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("test_utf8.txt");
        
        let test_content = "Hello, 世界! 🌍";
        fs::write(&test_file, test_content).unwrap();
        
        let mut parser = Statement::new().unwrap();
        let file_path = test_file.to_str().unwrap();
        let expression = format!("[loadTextContent('{}', 'utf-8')]", file_path);
        let result = parser.parse_and_execute(&expression, &Context::new()).unwrap();
        
        assert_eq!(result, test_content);
        fs::remove_file(test_file).ok();
    }

    #[test]
    fn test_load_text_content_empty_file() {
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("test_empty.txt");
        
        fs::write(&test_file, "").unwrap();
        
        let mut parser = Statement::new().unwrap();
        let file_path = test_file.to_str().unwrap();
        let expression = format!("[loadTextContent('{}')]", file_path);
        let result = parser.parse_and_execute(&expression, &Context::new()).unwrap();
        
        assert_eq!(result, "");
        fs::remove_file(test_file).ok();
    }

    #[test]
    fn test_load_text_content_with_newlines() {
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("test_newlines.txt");
        
        let test_content = "Line 1\nLine 2\r\nLine 3\n";
        fs::write(&test_file, test_content).unwrap();
        
        let mut parser = Statement::new().unwrap();
        let file_path = test_file.to_str().unwrap();
        let expression = format!("[loadTextContent('{}')]", file_path);
        let result = parser.parse_and_execute(&expression, &Context::new()).unwrap();
        
        assert_eq!(result, test_content);
        fs::remove_file(test_file).ok();
    }

    #[test]
    fn test_load_text_content_file_not_found() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[loadTextContent('/nonexistent/file.txt')]", &Context::new());
        
        assert!(result.is_err());
    }

    #[test]
    fn test_load_text_content_content_too_large() {
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("test_large_text.txt");
        
        // Create a file with more than 131072 characters
        let large_content = "a".repeat(131073);
        fs::write(&test_file, &large_content).unwrap();
        
        let mut parser = Statement::new().unwrap();
        let file_path = test_file.to_str().unwrap();
        let expression = format!("[loadTextContent('{}')]", file_path);
        let result = parser.parse_and_execute(&expression, &Context::new());
        
        assert!(result.is_err());
        fs::remove_file(test_file).ok();
    }

    #[test]
    fn test_load_text_content_utf16_encoding() {
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("test_utf16.txt");
        
        let test_content = "Hello, UTF-16!";
        // Write as UTF-16LE
        let utf16_bytes: Vec<u8> = test_content
            .encode_utf16()
            .flat_map(|c| c.to_le_bytes())
            .collect();
        fs::write(&test_file, &utf16_bytes).unwrap();
        
        let mut parser = Statement::new().unwrap();
        let file_path = test_file.to_str().unwrap();
        let expression = format!("[loadTextContent('{}', 'utf-16')]", file_path);
        let result = parser.parse_and_execute(&expression, &Context::new()).unwrap();
        
        assert_eq!(result, test_content);
        fs::remove_file(test_file).ok();
    }

    #[test]
    fn test_load_text_content_iso_8859_1_encoding() {
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("test_iso.txt");
        
        // ISO-8859-1 encoded bytes for "Café"
        let iso_bytes: Vec<u8> = vec![0x43, 0x61, 0x66, 0xE9]; // Café
        fs::write(&test_file, &iso_bytes).unwrap();
        
        let mut parser = Statement::new().unwrap();
        let file_path = test_file.to_str().unwrap();
        let expression = format!("[loadTextContent('{}', 'iso-8859-1')]", file_path);
        let result = parser.parse_and_execute(&expression, &Context::new()).unwrap();
        
        assert_eq!(result, "Café");
        fs::remove_file(test_file).ok();
    }

    #[test]
    fn test_load_text_content_unsupported_encoding() {
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("test_invalid_encoding.txt");
        
        fs::write(&test_file, "test").unwrap();
        
        let mut parser = Statement::new().unwrap();
        let file_path = test_file.to_str().unwrap();
        let expression = format!("[loadTextContent('{}', 'invalid-encoding')]", file_path);
        let result = parser.parse_and_execute(&expression, &Context::new());
        
        assert!(result.is_err());
        fs::remove_file(test_file).ok();
    }
}
