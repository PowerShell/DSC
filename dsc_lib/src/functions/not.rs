// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::DscError;
use crate::configure::context::Context;
use crate::functions::{FunctionArgKind, Function, FunctionCategory, FunctionMetadata};
use rust_i18n::t;
use serde_json::Value;
use tracing::debug;

#[derive(Debug, Default)]
pub struct Not {}

impl Function for Not {
    fn get_metadata(&self) -> FunctionMetadata {
        FunctionMetadata {
            name: "not".to_string(),
            description: t!("functions.not.description").to_string(),
            category: FunctionCategory::Logical,
            min_args: 1,
            max_args: 1,
            accepted_arg_ordered_types: vec![vec![FunctionArgKind::Boolean]],
            remaining_arg_accepted_types: None,
            return_types: vec![FunctionArgKind::Boolean],
        }
    }

    fn invoke(&self, args: &[Value], _context: &Context) -> Result<Value, DscError> {
        debug!("{}", t!("functions.not.invoked"));
        if let Some(arg1) = args[0].as_bool() {
            Ok(Value::Bool(!arg1))
        } else {
            Err(DscError::Parser(t!("functions.invalidArguments").to_string()))
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::configure::context::Context;
    use crate::parser::Statement;

    #[test]
    fn not_true() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[not(true)]", &Context::new(), true).unwrap();
        assert_eq!(result, false);
    }

    #[test]
    fn not_false() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[not(false)]", &Context::new(), true).unwrap();
        assert_eq!(result, true);
    }
}
