// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use super::Function;
use crate::configure::context::Context;
use crate::functions::{FunctionArgKind, FunctionCategory, FunctionMetadata};
use crate::DscError;
use rust_i18n::t;
use serde_json::Value;
use tracing::debug;

#[derive(Debug, Default)]
pub struct Substring {}

impl Function for Substring {
    fn get_metadata(&self) -> FunctionMetadata {
        FunctionMetadata {
            name: "substring".to_string(),
            description: t!("functions.substring.description").to_string(),
            category: vec![FunctionCategory::String],
            min_args: 2,
            max_args: 3,
            accepted_arg_ordered_types: vec![
                vec![FunctionArgKind::String],
                vec![FunctionArgKind::Number],
                vec![FunctionArgKind::Number],
            ],
            remaining_arg_accepted_types: None,
            return_types: vec![FunctionArgKind::String],
        }
    }

    fn invoke(&self, args: &[Value], _context: &Context) -> Result<Value, DscError> {
        debug!("{}", t!("functions.substring.invoked"));

        let string_to_parse = args[0].as_str().unwrap();
        let start_index_value = args[1].as_i64().unwrap();

        if start_index_value < 0 {
            return Err(DscError::FunctionArg(
                "substring".to_string(),
                t!("functions.substring.startIndexNegative").to_string(),
            ));
        }

        let start_index = usize::try_from(start_index_value).map_err(|_| {
            DscError::FunctionArg(
                "substring".to_string(),
                t!("functions.substring.startIndexValueTooLarge").to_string(),
            )
        })?;
        let string_length = string_to_parse.chars().count();

        if start_index > string_length {
            return Err(DscError::FunctionArg(
                "substring".to_string(),
                t!("functions.substring.startIndexTooLarge").to_string(),
            ));
        }

        let length = if args.len() == 2 {
            string_length - start_index
        } else {
            let length_value = args[2].as_i64().unwrap();

            if length_value < 0 {
                return Err(DscError::FunctionArg(
                    "substring".to_string(),
                    t!("functions.substring.lengthNegative").to_string(),
                ));
            }

            let length = usize::try_from(length_value).map_err(|_| {
                DscError::FunctionArg(
                    "substring".to_string(),
                    t!("functions.substring.lengthValueTooLarge").to_string(),
                )
            })?;

            if start_index + length > string_length {
                return Err(DscError::FunctionArg(
                    "substring".to_string(),
                    t!("functions.substring.lengthTooLarge").to_string(),
                ));
            }

            length
        };

        let result: String = string_to_parse
            .chars()
            .skip(start_index)
            .take(length)
            .collect();

        Ok(Value::String(result))
    }
}

#[cfg(test)]
mod tests {
    use crate::configure::context::Context;
    use crate::parser::Statement;
    use serde_json::Value;

    #[test]
    fn substring_basic() {
        let mut parser = Statement::new().unwrap();
        let result = parser
            .parse_and_execute("[substring('hello', 1, 3)]", &Context::new())
            .unwrap();
        assert_eq!(result, Value::String("ell".to_string()));
    }

    #[test]
    fn substring_from_start() {
        let mut parser = Statement::new().unwrap();
        let result = parser
            .parse_and_execute("[substring('hello', 0, 2)]", &Context::new())
            .unwrap();
        assert_eq!(result, Value::String("he".to_string()));
    }

    #[test]
    fn substring_to_end() {
        let mut parser = Statement::new().unwrap();
        let result = parser
            .parse_and_execute("[substring('hello', 2)]", &Context::new())
            .unwrap();
        assert_eq!(result, Value::String("llo".to_string()));
    }

    #[test]
    fn substring_empty_result() {
        let mut parser = Statement::new().unwrap();
        let result = parser
            .parse_and_execute("[substring('hello', 5, 0)]", &Context::new())
            .unwrap();
        assert_eq!(result, Value::String("".to_string()));
    }

    #[test]
    fn substring_entire_string() {
        let mut parser = Statement::new().unwrap();
        let result = parser
            .parse_and_execute("[substring('hello', 0)]", &Context::new())
            .unwrap();
        assert_eq!(result, Value::String("hello".to_string()));
    }

    #[test]
    fn substring_single_char() {
        let mut parser = Statement::new().unwrap();
        let result = parser
            .parse_and_execute("[substring('hello', 1, 1)]", &Context::new())
            .unwrap();
        assert_eq!(result, Value::String("e".to_string()));
    }

    #[test]
    fn substring_unicode_support() {
        let mut parser = Statement::new().unwrap();
        let result = parser
            .parse_and_execute("[substring('héllo', 1, 2)]", &Context::new())
            .unwrap();
        assert_eq!(result, Value::String("él".to_string()));
    }

    #[test]
    fn substring_emoji_support() {
        let mut parser = Statement::new().unwrap();
        let result = parser
            .parse_and_execute("[substring('🚀🎉🔥', 1, 1)]", &Context::new())
            .unwrap();
        assert_eq!(result, Value::String("🎉".to_string()));
    }

    #[test]
    fn substring_negative_start_index() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[substring('hello', -1, 2)]", &Context::new());
        assert!(result.is_err());
    }

    #[test]
    fn substring_negative_length() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[substring('hello', 1, -1)]", &Context::new());
        assert!(result.is_err());
    }

    #[test]
    fn substring_start_index_too_large() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[substring('hello', 10, 1)]", &Context::new());
        assert!(result.is_err());
    }

    #[test]
    fn substring_length_too_large() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[substring('hello', 2, 10)]", &Context::new());
        assert!(result.is_err());
    }

    #[test]
    fn substring_empty_string() {
        let mut parser = Statement::new().unwrap();
        let result = parser
            .parse_and_execute("[substring('', 0, 0)]", &Context::new())
            .unwrap();
        assert_eq!(result, Value::String("".to_string()));
    }

    #[test]
    fn substring_empty_string_to_end() {
        let mut parser = Statement::new().unwrap();
        let result = parser
            .parse_and_execute("[substring('', 0)]", &Context::new())
            .unwrap();
        assert_eq!(result, Value::String("".to_string()));
    }

    #[test]
    fn substring_at_string_boundary() {
        let mut parser = Statement::new().unwrap();
        let result = parser
            .parse_and_execute("[substring('hello', 5)]", &Context::new())
            .unwrap();
        assert_eq!(result, Value::String("".to_string()));
    }
}

