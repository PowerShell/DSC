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

fn stringify_value(v: &Value) -> String {
    if let Some(s) = v.as_str() { return s.to_string(); }
    if let Some(n) = v.as_i64() { return n.to_string(); }
    if let Some(b) = v.as_bool() { return b.to_string(); }
    if v.is_null() { return "null".to_string(); }
    // Fallback to JSON for arrays/objects or other numbers
    serde_json::to_string(v).unwrap_or_default()
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
                // delimiter: accept any type (no validation), convert to string
                vec![
                    FunctionArgKind::Array,
                    FunctionArgKind::Boolean,
                    FunctionArgKind::Null,
                    FunctionArgKind::Number,
                    FunctionArgKind::Object,
                    FunctionArgKind::String,
                ],
            ],
            remaining_arg_accepted_types: None,
            return_types: vec![FunctionArgKind::String],
        }
    }

    fn invoke(&self, args: &[Value], _context: &Context) -> Result<Value, DscError> {
        debug!("{}", t!("functions.join.invoked"));

        let delimiter = stringify_value(&args[1]);

        if let Some(array) = args[0].as_array() {
            let items: Vec<String> = array.iter().map(stringify_value).collect();
            return Ok(Value::String(items.join(&delimiter)));
        }

        Err(DscError::Parser(t!("functions.join.invalidArrayArg").to_string()))
    }
}

#[cfg(test)]
mod tests {
    use crate::configure::context::Context;
    use crate::parser::Statement;

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
}
