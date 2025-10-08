// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::configure::context::Context;
use crate::functions::{Function, FunctionArgKind, FunctionCategory, FunctionMetadata};
use crate::DscError;
use rust_i18n::t;
use serde_json::Value;
use tracing::debug;

#[derive(Debug, Default)]
pub struct Skip {}

impl Function for Skip {
    fn get_metadata(&self) -> FunctionMetadata {
        FunctionMetadata {
            name: "skip".to_string(),
            description: t!("functions.skip.description").to_string(),
            category: vec![FunctionCategory::Array, FunctionCategory::String],
            min_args: 2,
            max_args: 2,
            accepted_arg_ordered_types: vec![
                vec![FunctionArgKind::Array, FunctionArgKind::String],
                vec![FunctionArgKind::Number],
            ],
            remaining_arg_accepted_types: None,
            return_types: vec![FunctionArgKind::Array, FunctionArgKind::String],
        }
    }

    fn invoke(&self, args: &[Value], _context: &Context) -> Result<Value, DscError> {
        debug!("{}", t!("functions.skip.invoked"));

        if let Some(count_i64) = args[1].as_i64() {
            let count: usize = if count_i64 < 0 {
                0
            } else {
                count_i64.try_into().unwrap_or(usize::MAX)
            };

            if let Some(array) = args[0].as_array() {
                if count >= array.len() {
                    return Ok(Value::Array(vec![]));
                }
                let skipped = array.iter().skip(count).cloned().collect::<Vec<Value>>();
                return Ok(Value::Array(skipped));
            }

            if let Some(s) = args[0].as_str() {
                let result: String = s.chars().skip(count).collect();
                return Ok(Value::String(result));
            }

            return Err(DscError::Parser(t!("functions.skip.invalidOriginalValue").to_string()));
        }

        Err(DscError::Parser(t!("functions.skip.invalidNumberToSkip").to_string()))
    }
}

#[cfg(test)]
mod tests {
    use crate::configure::context::Context;
    use crate::parser::Statement;
    use serde_json::Value;

    #[test]
    fn skip_array_basic() {
        let mut parser = Statement::new().unwrap();
        let result = parser
            .parse_and_execute("[skip(createArray('a','b','c','d'), 2)]", &Context::new())
            .unwrap();
        assert_eq!(
            result,
            Value::Array(vec![Value::String("c".into()), Value::String("d".into())])
        );
    }

    #[test]
    fn skip_string_basic() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[skip('hello', 2)]", &Context::new()).unwrap();
        assert_eq!(result, Value::String("llo".into()));
    }

    #[test]
    fn skip_more_than_length() {
        let mut parser = Statement::new().unwrap();
        let result = parser
            .parse_and_execute("[skip(createArray('a','b'), 5)]", &Context::new())
            .unwrap();
        assert_eq!(result, Value::Array(vec![]));
    }

    #[test]
    fn skip_array_negative_is_zero() {
        let mut parser = Statement::new().unwrap();
        let result = parser
            .parse_and_execute("[skip(createArray('a','b','c'), -1)]", &Context::new())
            .unwrap();
        assert_eq!(
            result,
            Value::Array(vec![
                Value::String("a".into()),
                Value::String("b".into()),
                Value::String("c".into()),
            ])
        );
    }

    #[test]
    fn skip_string_negative_is_zero() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[skip('ab', -2)]", &Context::new()).unwrap();
        assert_eq!(result, Value::String("ab".into()));
    }
}
