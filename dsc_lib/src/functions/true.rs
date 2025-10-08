// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::configure::context::Context;
use crate::functions::{Function, FunctionArgKind, FunctionCategory, FunctionMetadata};
use crate::DscError;
use rust_i18n::t;
use serde_json::Value;
use tracing::debug;

#[derive(Debug, Default)]
pub struct True {}

impl Function for True {
    fn get_metadata(&self) -> FunctionMetadata {
        FunctionMetadata {
            name: "true".to_string(),
            description: t!("functions.true.description").to_string(),
            category: vec![FunctionCategory::Logical],
            min_args: 0,
            max_args: 0,
            accepted_arg_ordered_types: vec![],
            remaining_arg_accepted_types: None,
            return_types: vec![FunctionArgKind::Boolean],
        }
    }

    fn invoke(&self, _args: &[Value], _context: &Context) -> Result<Value, DscError> {
        debug!("{}", t!("functions.true.invoked"));
        Ok(Value::Bool(true))
    }
}

#[cfg(test)]
mod tests {
    use crate::configure::context::Context;
    use crate::parser::Statement;

    #[test]
    fn true_function() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[true()]", &Context::new()).unwrap();
        assert_eq!(result, true);
    }
}
