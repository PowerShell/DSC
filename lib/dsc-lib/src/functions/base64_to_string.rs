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
pub struct Base64ToString {}

impl Function for Base64ToString {
    fn get_metadata(&self) -> FunctionMetadata {
        FunctionMetadata {
            name: "base64ToString".to_string(),
            description: t!("functions.base64ToString.description").to_string(),
            category: vec![FunctionCategory::String],
            min_args: 1,
            max_args: 1,
            accepted_arg_ordered_types: vec![vec![FunctionArgKind::String]],
            remaining_arg_accepted_types: None,
            return_types: vec![FunctionArgKind::String],
        }
    }

    fn invoke(&self, args: &[Value], _context: &Context) -> Result<Value, DscError> {
        debug!("{}", t!("functions.base64ToString.invoked"));
        
        let base64_value = args[0].as_str().unwrap();

        let decoded_bytes = general_purpose::STANDARD.decode(base64_value).map_err(|_| {
            DscError::FunctionArg(
                "base64ToString".to_string(),
                t!("functions.base64ToString.invalidBase64").to_string(),
            )
        })?;

        let result = String::from_utf8(decoded_bytes).map_err(|_| {
            DscError::FunctionArg(
                "base64ToString".to_string(),
                t!("functions.base64ToString.invalidUtf8").to_string(),
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
    fn base64_to_string_simple() {
        let mut parser = Statement::new().unwrap();
        let result = parser
            .parse_and_execute("[base64ToString('aGVsbG8gd29ybGQ=')]", &Context::new())
            .unwrap();
        assert_eq!(result, Value::String("hello world".to_string()));
    }

    #[test]
    fn base64_to_string_empty() {
        let mut parser = Statement::new().unwrap();
        let result = parser
            .parse_and_execute("[base64ToString('')]", &Context::new())
            .unwrap();
        assert_eq!(result, Value::String("".to_string()));
    }

    #[test]
    fn base64_to_string_unicode() {
        let mut parser = Statement::new().unwrap();
        let result = parser
            .parse_and_execute("[base64ToString('aMOpbGxv')]", &Context::new())
            .unwrap();
        assert_eq!(result, Value::String("héllo".to_string()));
    }

    #[test]
    fn base64_to_string_round_trip() {
        let mut parser = Statement::new().unwrap();
        let result = parser
            .parse_and_execute("[base64ToString(base64('test message'))]", &Context::new())
            .unwrap();
        assert_eq!(result, Value::String("test message".to_string()));
    }

    #[test]
    fn base64_to_string_invalid_base64() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[base64ToString('invalid!@#')]", &Context::new());
        assert!(result.is_err());
    }

    #[test]
    fn base64_to_string_invalid_utf8() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[base64ToString('/w==')]", &Context::new());
        assert!(result.is_err());
    }

    #[test]
    fn base64_to_string_json_string() {
        let mut parser = Statement::new().unwrap();
        let result = parser
            .parse_and_execute("[base64ToString('eyJrZXkiOiJ2YWx1ZSJ9')]", &Context::new())
            .unwrap();
        assert_eq!(result, Value::String("{\"key\":\"value\"}".to_string()));
    }
}