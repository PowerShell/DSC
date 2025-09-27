// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::DscError;
use crate::configure::context::Context;
use crate::functions::{FunctionArgKind, FunctionCategory, FunctionMetadata};
use rust_i18n::t;
use serde_json::Value;
use super::Function;
use tracing::debug;

#[derive(Debug, Default)]
pub struct StartsWith {}

impl Function for StartsWith {
    fn get_metadata(&self) -> FunctionMetadata {
        FunctionMetadata {
            name: "startsWith".to_string(),
            description: t!("functions.startsWith.description").to_string(),
            category: vec![FunctionCategory::String],
            min_args: 2,
            max_args: 2,
            accepted_arg_ordered_types: vec![
                vec![FunctionArgKind::String],
                vec![FunctionArgKind::String],
            ],
            remaining_arg_accepted_types: None,
            return_types: vec![FunctionArgKind::Boolean],
        }
    }

    fn invoke(&self, args: &[Value], _context: &Context) -> Result<Value, DscError> {
        debug!("{}", t!("functions.startsWith.invoked"));
        if let (Some(string_to_search), Some(string_to_find)) = (args[0].as_str(), args[1].as_str()) {
            Ok(Value::Bool(string_to_search.starts_with(string_to_find)))
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
    fn does_start_with() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[startsWith('hello', 'he')]", &Context::new()).unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn does_not_start_with() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[startsWith('hello', 'world')]", &Context::new()).unwrap();
        assert_eq!(result, Value::Bool(false));
    }
}
