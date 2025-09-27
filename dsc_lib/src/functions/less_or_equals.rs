// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::DscError;
use crate::configure::context::Context;
use crate::functions::{FunctionArgKind, Function, FunctionCategory, FunctionMetadata};
use rust_i18n::t;
use serde_json::Value;
use tracing::debug;

#[derive(Debug, Default)]
pub struct LessOrEquals {}

impl Function for LessOrEquals {
    fn get_metadata(&self) -> FunctionMetadata {
        FunctionMetadata {
            name: "lessOrEquals".to_string(),
            description: t!("functions.lessOrEquals.description").to_string(),
            category: vec![FunctionCategory::Comparison],
            min_args: 2,
            max_args: 2,
            accepted_arg_ordered_types: vec![
                vec![FunctionArgKind::Number, FunctionArgKind::String],
                vec![FunctionArgKind::Number, FunctionArgKind::String],
            ],
            remaining_arg_accepted_types: None,
            return_types: vec![FunctionArgKind::Boolean],
        }
    }

    fn invoke(&self, args: &[Value], _context: &Context) -> Result<Value, DscError> {
        debug!("{}", t!("functions.lessOrEquals.invoked"));

        let first = &args[0];
        let second = &args[1];

        if let (Some(num1), Some(num2)) = (first.as_i64(), second.as_i64()) {
            return Ok(Value::Bool(num1 <= num2));
        }

        if let (Some(str1), Some(str2)) = (first.as_str(), second.as_str()) {
            return Ok(Value::Bool(str1 <= str2));
        }

        Err(DscError::Parser(t!("functions.typeMismatch").to_string()))
    }
}

#[cfg(test)]
mod tests {
    use crate::configure::context::Context;
    use crate::parser::Statement;

    #[test]
    fn number_less_or_equals() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[lessOrEquals(3,5)]", &Context::new()).unwrap();
        assert_eq!(result, true);
    }

    #[test]
    fn number_not_less_or_equals() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[lessOrEquals(5,3)]", &Context::new()).unwrap();
        assert_eq!(result, false);
    }

    #[test]
    fn number_equal() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[lessOrEquals(5,5)]", &Context::new()).unwrap();
        assert_eq!(result, true);
    }

    #[test]
    fn string_less_or_equals() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[lessOrEquals('a','b')]", &Context::new()).unwrap();
        assert_eq!(result, true);
    }

    #[test]
    fn type_mismatch_string_number() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[lessOrEquals('5', 3)]", &Context::new());
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Arguments must be of the same type"));
    }
}
