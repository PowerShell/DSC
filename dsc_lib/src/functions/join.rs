// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::DscError;
use crate::configure::context::Context;
use crate::functions::{FunctionArgKind, Function, FunctionCategory, FunctionMetadata};
use rust_i18n::t;
use serde_json::Value;
use tracing::debug;

#[derive(Debug, Default)]
pub struct Join {}

fn stringify_value(v: &Value) -> Result<String, DscError> {
    match v {
        Value::String(s) => Ok(s.clone()),
        Value::Number(n) => Ok(n.to_string()),
        Value::Bool(b) => Ok(b.to_string()),
        Value::Null => Err(DscError::Parser(t!("functions.join.invalidNullElement").to_string())),
        Value::Array(_) => Err(DscError::Parser(t!("functions.join.invalidArrayElement").to_string())),
        Value::Object(_) => Err(DscError::Parser(t!("functions.join.invalidObjectElement").to_string())),
    }
}

impl Function for Join {
    fn get_metadata(&self) -> FunctionMetadata {
        FunctionMetadata {
            name: "join".to_string(),
            description: t!("functions.join.description").to_string(),
            category: FunctionCategory::String,
            min_args: 2,
            max_args: 2,
            accepted_arg_ordered_types: vec![
                vec![FunctionArgKind::Array],
                vec![FunctionArgKind::String],
            ],
            remaining_arg_accepted_types: None,
            return_types: vec![FunctionArgKind::String],
        }
    }

    fn invoke(&self, args: &[Value], _context: &Context) -> Result<Value, DscError> {
        debug!("{}", t!("functions.join.invoked"));

        let delimiter = args[1].as_str().unwrap();

        if let Some(array) = args[0].as_array() {
            let items: Result<Vec<String>, DscError> = array.iter().map(stringify_value).collect();
            let items = items?;
            return Ok(Value::String(items.join(delimiter)));
        }

        Err(DscError::Parser(t!("functions.join.invalidArrayArg").to_string()))
    }
}

#[cfg(test)]
mod tests {
    use crate::configure::context::Context;
    use crate::parser::Statement;
    use super::Join;
    use crate::functions::Function;

    #[test]
    fn join_array_of_strings() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[join(createArray('a','b','c'), '-')]", &Context::new()).unwrap();
        assert_eq!(result, "a-b-c");
    }

    #[test]
    fn join_empty_array_returns_empty() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[join(createArray(), '-')]", &Context::new()).unwrap();
        assert_eq!(result, "");
    }

    #[test]
    fn join_array_of_integers() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[join(createArray(1,2,3), ',')]", &Context::new()).unwrap();
        assert_eq!(result, "1,2,3");
    }

    #[test]
    fn join_array_with_null_fails() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[join(createArray('a', null()), ',')]", &Context::new());
        assert!(result.is_err());
        // The error comes from argument validation, not our function
        assert!(result.unwrap_err().to_string().contains("does not accept null arguments"));
    }

    #[test]
    fn join_array_with_array_fails() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[join(createArray('a', createArray('b')), ',')]", &Context::new());
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("Arguments must all be arrays") || error_msg.contains("mixed types"));
    }

    #[test]
    fn join_array_with_object_fails() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[join(createArray('a', createObject('key', 'value')), ',')]", &Context::new());
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("Arguments must all be") || error_msg.contains("mixed types"));
    }

    #[test]
    fn join_direct_test_with_mixed_array() {
        use serde_json::json;
        use crate::configure::context::Context;
        
        let join_fn = Join::default();
        let args = vec![
            json!(["hello", {"key": "value"}]), // Array with string and object
            json!(",")
        ];
        let result = join_fn.invoke(&args, &Context::new());
        
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("Array elements cannot be objects"));
    }
}
