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
pub struct EndsWith {}

impl Function for EndsWith {
    fn get_metadata(&self) -> FunctionMetadata {
        FunctionMetadata {
            name: "endsWith".to_string(),
            description: t!("functions.endsWith.description").to_string(),
            category: vec![FunctionCategory::String],
            min_args: 2,
            max_args: 2,
            accepted_arg_ordered_types: vec![vec![FunctionArgKind::String], vec![FunctionArgKind::String]],
            remaining_arg_accepted_types: None,
            return_types: vec![FunctionArgKind::Boolean],
        }
    }

    fn invoke(&self, args: &[Value], _context: &Context) -> Result<Value, DscError> {
        debug!("{}", t!("functions.endsWith.invoked"));
        if let (Some(string_to_search), Some(string_to_find)) = (args[0].as_str(), args[1].as_str()) {
            Ok(Value::Bool(string_to_search.ends_with(string_to_find)))
        } else {
            Err(DscError::Parser(t!("functions.invalidArguments").to_string()))
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::configure::context::Context;
    use crate::parser::Statement;
    use serde_json::Value;

    #[test]
    fn does_ends_with() {
        let mut parser = Statement::new().unwrap();
        let result = parser
            .parse_and_execute("[endsWith('hello', 'lo')]", &Context::new())
            .unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn does_not_ends_with() {
        let mut parser = Statement::new().unwrap();
        let result = parser
            .parse_and_execute("[endsWith('hello', 'world')]", &Context::new())
            .unwrap();
        assert_eq!(result, Value::Bool(false));
    }
}
