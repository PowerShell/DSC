// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::DscError;
use crate::configure::context::Context;
use crate::functions::{FunctionArgKind, FunctionCategory, FunctionMetadata};
use rust_i18n::t;
use serde_json::Value;
use super::Function;

#[derive(Debug, Default)]
pub struct StringFn {}

impl Function for StringFn {
    fn get_metadata(&self) -> FunctionMetadata {
        FunctionMetadata {
            name: "string".to_string(),
            description: t!("functions.string.description").to_string(),
            category: vec![FunctionCategory::String],
            min_args: 1,
            max_args: 1,
            accepted_arg_ordered_types: vec![vec![
                FunctionArgKind::String,
                FunctionArgKind::Number,
                FunctionArgKind::Boolean,
                FunctionArgKind::Null,
                FunctionArgKind::Array,
                FunctionArgKind::Object,
            ]],
            remaining_arg_accepted_types: None,
            return_types: vec![FunctionArgKind::String],
        }
    }

    fn invoke(&self, args: &[Value], _context: &Context) -> Result<Value, DscError> {
        let value = &args[0];
        let result = match value {
            Value::String(s) => s.clone(),
            Value::Number(n) => n.to_string(),
            Value::Bool(b) => b.to_string(),
            Value::Null => "null".to_string(),
            Value::Array(_) | Value::Object(_) => serde_json::to_string(value)?,
        };
        Ok(Value::String(result))
    }
}

#[cfg(test)]
mod tests {
    use crate::configure::context::Context;
    use crate::parser::Statement;
    use serde_json::json;

    #[test]
    fn string_from_string() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[string('hello')]", &Context::new()).unwrap();
        assert_eq!(result, "hello");
    }

    #[test]
    fn string_from_number() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[string(123)]", &Context::new()).unwrap();
        assert_eq!(result, "123");
    }

    #[test]
    fn string_from_bool() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[string(true)]", &Context::new()).unwrap();
        assert_eq!(result, "true");
    }

    #[test]
    fn string_from_null() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[string(null())]", &Context::new()).unwrap();
        assert_eq!(result, "null");
    }

    #[test]
    fn string_from_array() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[string(createArray('a', 'b'))]", &Context::new()).unwrap();
        assert_eq!(result, json!(["a", "b"]).to_string());
    }

    #[test]
    fn string_from_object() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[string(createObject('a', 'hello'))]", &Context::new()).unwrap();
        assert_eq!(result, json!({"a": "hello"}).to_string());
    }
}
