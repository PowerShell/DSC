// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::DscError;
use crate::configure::context::Context;
use crate::functions::{FunctionArgKind, Function, FunctionCategory, FunctionMetadata};
use rust_i18n::t;
use serde_json::Value;
use tracing::debug;

#[derive(Debug, Default)]
pub struct Null {}

impl Function for Null {
    fn get_metadata(&self) -> FunctionMetadata {
        FunctionMetadata {
            name: "null".to_string(),
            description: t!("functions.null.description").to_string(),
            category: vec![FunctionCategory::Logical],
            min_args: 0,
            max_args: 0,
            accepted_arg_ordered_types: vec![],
            remaining_arg_accepted_types: None,
            return_types: vec![FunctionArgKind::Null],
        }
    }

    fn invoke(&self, _args: &[Value], _context: &Context) -> Result<Value, DscError> {
        debug!("{}", t!("functions.null.invoked"));
        Ok(Value::Null)
    }
}

#[cfg(test)]
mod tests {
    use crate::configure::context::Context;
    use crate::parser::Statement;
    use super::*;

    #[test]
    fn direct_function_call() {
        let null_fn = Null {};
        let context = Context::new();

        let result = null_fn.invoke(&[], &context).unwrap();
        assert_eq!(result, Value::Null);
    }

    #[test]
    fn parser_with_null() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[null()]", &Context::new()).unwrap();
        assert_eq!(result, Value::Null);
    }

    #[test]
    fn null_with_coalesce() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[coalesce(null(), 'fallback')]", &Context::new()).unwrap();
        assert_eq!(result, "fallback");
    }

    #[test]
    fn null_in_object() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[createObject('key', null())]", &Context::new()).unwrap();
        assert_eq!(result.to_string(), r#"{"key":null}"#);
    }
}
