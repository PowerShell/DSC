// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::DscError;
use crate::configure::context::Context;
use crate::functions::{AcceptedArgKind, Function, FunctionCategory};
use rust_i18n::t;
use serde_json::Value;
use tracing::debug;

#[derive(Debug, Default)]
pub struct LessOrEquals {}

impl Function for LessOrEquals {
    fn description(&self) -> String {
        t!("functions.lessOrEquals.description").to_string()
    }

    fn category(&self) -> FunctionCategory {
        FunctionCategory::Comparison
    }

    fn min_args(&self) -> usize {
        2
    }

    fn max_args(&self) -> usize {
        2
    }

    fn accepted_arg_types(&self) -> Vec<AcceptedArgKind> {
        vec![AcceptedArgKind::Number, AcceptedArgKind::Number]
    }

    fn invoke(&self, args: &[Value], _context: &Context) -> Result<Value, DscError> {
        debug!("{}", t!("functions.lessOrEquals.invoked"));
        
        let num1 = match &args[0] {
            Value::Number(n) => n.as_f64().ok_or_else(|| DscError::Parser(t!("functions.invalidArguments").to_string()))?,
            _ => return Err(DscError::Parser(t!("functions.invalidArguments").to_string())),
        };

        let num2 = match &args[1] {
            Value::Number(n) => n.as_f64().ok_or_else(|| DscError::Parser(t!("functions.invalidArguments").to_string()))?,
            _ => return Err(DscError::Parser(t!("functions.invalidArguments").to_string())),
        };

        Ok(Value::Bool(num1 <= num2))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::configure::context::Context;

    #[test]
    fn test_less_or_equals() {
        let less_or_equals = LessOrEquals {};
        let context = Context::new().unwrap();
        
        let result = less_or_equals.invoke(&[Value::Number(3.into()), Value::Number(5.into())], &context);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Bool(true));
        
        let result = less_or_equals.invoke(&[Value::Number(5.into()), Value::Number(3.into())], &context);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Bool(false));
        
        let result = less_or_equals.invoke(&[Value::Number(5.into()), Value::Number(5.into())], &context);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Bool(true));
    }

    #[test]
    fn test_invalid_args() {
        let less_or_equals = LessOrEquals {};
        let context = Context::new().unwrap();
        
        let result = less_or_equals.invoke(&[Value::Number(5.into())], &context);
        assert!(result.is_err());
    }
}
