// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use base64::{Engine as _, engine::general_purpose};

use crate::DscError;
use crate::configure::context::Context;
use crate::functions::{FunctionArgKind, FunctionCategory, FunctionMetadata};
use rust_i18n::t;
use serde_json::Value;
use super::Function;

#[derive(Debug, Default)]
pub struct DataUri {}

impl Function for DataUri {
    fn get_metadata(&self) -> FunctionMetadata {
        FunctionMetadata {
            name: "dataUri".to_string(),
            description: t!("functions.dataUri.description").to_string(),
            category: vec![FunctionCategory::String],
            min_args: 1,
            max_args: 1,
            accepted_arg_ordered_types: vec![vec![FunctionArgKind::String]],
            remaining_arg_accepted_types: None,
            return_types: vec![FunctionArgKind::String],
        }
    }

    fn invoke(&self, args: &[Value], _context: &Context) -> Result<Value, DscError> {
        let string_to_convert = args[0].as_str().unwrap_or_default();
        let base64_encoded = general_purpose::STANDARD.encode(string_to_convert);
        let result = format!("data:text/plain;charset=utf8;base64,{base64_encoded}");
        Ok(Value::String(result))
    }
}

#[cfg(test)]
mod tests {
    use crate::configure::context::Context;
    use crate::parser::Statement;

    #[test]
    fn test_data_uri_basic() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[dataUri('Hello')]", &Context::new()).unwrap();
        assert_eq!(result, "data:text/plain;charset=utf8;base64,SGVsbG8=");
    }

    #[test]
    fn test_data_uri_empty_string() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[dataUri('')]", &Context::new()).unwrap();
        assert_eq!(result, "data:text/plain;charset=utf8;base64,");
    }

    #[test]
    fn test_data_uri_with_spaces() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[dataUri('Hello, World!')]", &Context::new()).unwrap();
        assert_eq!(result, "data:text/plain;charset=utf8;base64,SGVsbG8sIFdvcmxkIQ==");
    }

    #[test]
    fn test_data_uri_unicode() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[dataUri('héllo')]", &Context::new()).unwrap();
        assert_eq!(result, "data:text/plain;charset=utf8;base64,aMOpbGxv");
    }

    #[test]
    fn test_data_uri_number_arg_fails() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[dataUri(123)]", &Context::new());
        assert!(result.is_err());
    }
}
