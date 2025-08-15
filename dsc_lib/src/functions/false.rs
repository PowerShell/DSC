// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::DscError;
use crate::configure::context::Context;
use crate::functions::{FunctionArgKind, Function, FunctionCategory, FunctionMetadata};
use rust_i18n::t;
use serde_json::Value;
use tracing::debug;

#[derive(Debug, Default)]
pub struct False {}

impl Function for False {
    fn get_metadata(&self) -> FunctionMetadata {
        FunctionMetadata {
            name: "false".to_string(),
            description: t!("functions.false.description").to_string(),
            category: FunctionCategory::Logical,
            min_args: 0,
            max_args: 0,
            accepted_arg_ordered_types: vec![],
            remaining_arg_accepted_types: None,
            return_types: vec![FunctionArgKind::Boolean],
        }
    }

    fn invoke(&self, _args: &[Value], _context: &Context) -> Result<Value, DscError> {
        debug!("{}", t!("functions.false.invoked"));
        Ok(Value::Bool(false))
    }
}

#[cfg(test)]
mod tests {
    use crate::configure::context::Context;
    use crate::parser::Statement;

    #[test]
    fn false_function() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[false()]", &Context::new()).unwrap();
        assert_eq!(result, false);
    }
}
