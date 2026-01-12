// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use base64::{Engine as _, engine::general_purpose};

use crate::DscError;
use crate::configure::context::Context;
use crate::functions::{FunctionArgKind, FunctionCategory, FunctionMetadata};
use rust_i18n::t;
use serde_json::Value;
use super::Function;
use tracing::debug;

#[derive(Debug, Default)]
pub struct DataUriToString {}

impl Function for DataUriToString {
    fn get_metadata(&self) -> FunctionMetadata {
        FunctionMetadata {
            name: "dataUriToString".to_string(),
            description: t!("functions.dataUriToString.description").to_string(),
            category: vec![FunctionCategory::String],
            min_args: 1,
            max_args: 1,
            accepted_arg_ordered_types: vec![vec![FunctionArgKind::String]],
            remaining_arg_accepted_types: None,
            return_types: vec![FunctionArgKind::String],
        }
    }

    fn invoke(&self, args: &[Value], _context: &Context) -> Result<Value, DscError> {
        debug!("{}", t!("functions.dataUriToString.invoked"));

        let data_uri = args[0].as_str().unwrap();

        if !data_uri.starts_with("data:") {
            return Err(DscError::FunctionArg(
                "dataUriToString".to_string(),
                t!("functions.dataUriToString.invalidDataUri").to_string(),
            ));
        }

        // Find the base64 marker and extract the encoded content
        // Format: data:[<mediatype>][;base64],<data>
        let Some(comma_pos) = data_uri.find(',') else {
            return Err(DscError::FunctionArg(
                "dataUriToString".to_string(),
                t!("functions.dataUriToString.invalidDataUri").to_string(),
            ));
        };

        let metadata = &data_uri[5..comma_pos]; // Skip "data:"
        let encoded_data = &data_uri[comma_pos + 1..];

        // Require base64 encoding (matching ARM behavior)
        if !metadata.contains(";base64") {
            return Err(DscError::FunctionArg(
                "dataUriToString".to_string(),
                t!("functions.dataUriToString.notBase64").to_string(),
            ));
        }

        // Parse charset from metadata if present and validate
        // Format: data:[<mediatype>][;charset=<charset>][;base64],<data>
        if let Some(charset_part) = metadata.split(';').find(|part| part.starts_with("charset=")) {
            let charset_value = &charset_part[8..]; // Skip "charset="
            let charset_lower = charset_value.to_lowercase();
            if charset_lower != "utf-8" && charset_lower != "utf8" {
                return Err(DscError::FunctionArg(
                    "dataUriToString".to_string(),
                    t!("functions.dataUriToString.unsupportedCharset", charset = charset_value).to_string(),
                ));
            }
        }
        // TODO: In the future add more support for charsets

        // Decode base64
        let decoded_bytes = general_purpose::STANDARD.decode(encoded_data).map_err(|_| {
            DscError::FunctionArg(
                "dataUriToString".to_string(),
                t!("functions.dataUriToString.invalidBase64").to_string(),
            )
        })?;

        let result = String::from_utf8(decoded_bytes).map_err(|_| {
            DscError::FunctionArg(
                "dataUriToString".to_string(),
                t!("functions.dataUriToString.invalidUtf8").to_string(),
            )
        })?;

        Ok(Value::String(result))
    }
}

#[cfg(test)]
mod tests {
    use crate::configure::context::Context;
    use crate::parser::Statement;
    use serde_json::Value;

    #[test]
    fn test_data_uri_to_string_basic() {
        let mut parser = Statement::new().unwrap();
        let result = parser
            .parse_and_execute("[dataUriToString('data:text/plain;charset=utf8;base64,SGVsbG8=')]", &Context::new())
            .unwrap();
        assert_eq!(result, Value::String("Hello".to_string()));
    }

    #[test]
    fn test_data_uri_to_string_with_comma() {
        let mut parser = Statement::new().unwrap();
        let result = parser
            .parse_and_execute("[dataUriToString('data:;base64,SGVsbG8sIFdvcmxkIQ==')]", &Context::new())
            .unwrap();
        assert_eq!(result, Value::String("Hello, World!".to_string()));
    }

    #[test]
    fn test_data_uri_to_string_empty() {
        let mut parser = Statement::new().unwrap();
        let result = parser
            .parse_and_execute("[dataUriToString('data:text/plain;base64,')]", &Context::new())
            .unwrap();
        assert_eq!(result, Value::String("".to_string()));
    }

    #[test]
    fn test_data_uri_to_string_unicode() {
        let mut parser = Statement::new().unwrap();
        let result = parser
            .parse_and_execute("[dataUriToString('data:text/plain;charset=utf8;base64,aMOpbGxv')]", &Context::new())
            .unwrap();
        assert_eq!(result, Value::String("héllo".to_string()));
    }

    #[test]
    fn test_data_uri_to_string_invalid_uri() {
        let mut parser = Statement::new().unwrap();
        let result = parser
            .parse_and_execute("[dataUriToString('not a data uri')]", &Context::new());
        assert!(result.is_err());
    }

    #[test]
    fn test_data_uri_to_string_no_comma() {
        let mut parser = Statement::new().unwrap();
        let result = parser
            .parse_and_execute("[dataUriToString('data:text/plain;base64')]", &Context::new());
        assert!(result.is_err());
    }

    #[test]
    fn test_data_uri_to_string_round_trip() {
        let mut parser = Statement::new().unwrap();
        let result = parser
            .parse_and_execute("[dataUriToString(dataUri('Hello, World!'))]", &Context::new())
            .unwrap();
        assert_eq!(result, Value::String("Hello, World!".to_string()));
    }

    #[test]
    fn test_data_uri_to_string_url_encoded_fails() {
        let mut parser = Statement::new().unwrap();
        let result = parser
            .parse_and_execute("[dataUriToString('data:text/plain,Hello%20World')]", &Context::new());
        assert!(result.is_err());
    }

    #[test]
    fn test_data_uri_to_string_unsupported_charset() {
        let mut parser = Statement::new().unwrap();
        let result = parser
            .parse_and_execute("[dataUriToString('data:text/plain;charset=utf-16;base64,SGVsbG8=')]", &Context::new());
        assert!(result.is_err());
    }

    #[test]
    fn test_data_uri_to_string_no_charset_assumes_utf8() {
        let mut parser = Statement::new().unwrap();
        let result = parser
            .parse_and_execute("[dataUriToString('data:application/json;base64,SGVsbG8=')]", &Context::new())
            .unwrap();
        assert_eq!(result, Value::String("Hello".to_string()));
    }

    #[test]
    fn test_data_uri_to_string_double_colon() {
        let mut parser = Statement::new().unwrap();
        let result = parser
            .parse_and_execute("[dataUriToString('data::text/plain;base64,SGVsbG8=')]", &Context::new());

        assert!(result.is_ok());
    }

    #[test]
    fn test_data_uri_to_string_double_semicolon() {
        let mut parser = Statement::new().unwrap();
        let result = parser
            .parse_and_execute("[dataUriToString('data:text/plain;;base64,SGVsbG8=')]", &Context::new());
        assert!(result.is_ok());
    }

    #[test]
    fn test_data_uri_to_string_double_charset() {
        let mut parser = Statement::new().unwrap();
        let result = parser
            .parse_and_execute("[dataUriToString('data:text/plain;charset=utf-8;charset=utf-8;base64,SGVsbG8=')]", &Context::new());
        assert!(result.is_ok());
    }

    #[test]
    fn test_data_uri_to_string_double_comma() {
        let mut parser = Statement::new().unwrap();
        let result = parser
            .parse_and_execute("[dataUriToString('data:text/plain;base64,,SGVsbG8=')]", &Context::new());
        assert!(result.is_err());
    }

    #[test]
    fn test_data_uri_to_string_invalid_property() {
        let mut parser = Statement::new().unwrap();
        let result = parser
            .parse_and_execute("[dataUriToString('data:text/plain;foo=bar;base64,SGVsbG8=')]", &Context::new());
        assert!(result.is_ok());
    }
}
