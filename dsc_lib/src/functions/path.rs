// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::DscError;
use crate::configure::context::Context;
use crate::functions::{AcceptedArgKind, Function};
use serde_json::Value;
use std::path::PathBuf;
use tracing::debug;

#[derive(Debug, Default)]
pub struct Path {}

/// Implements the 'mountedpath' function.
/// This function returns the value of the mounted path.
/// The optional parameter is a path appended to the mounted path.
/// Path is not validated as it might be used for creation.
impl Function for Path {
    fn min_args(&self) -> usize {
        2
    }

    fn max_args(&self) -> usize {
        usize::MAX
    }

    fn accepted_arg_types(&self) -> Vec<AcceptedArgKind> {
        vec![AcceptedArgKind::String]
    }

    fn invoke(&self, args: &[Value], _context: &Context) -> Result<Value, DscError> {
        debug!("Executing path function with args: {:?}", args);

        let mut path = PathBuf::new();
        for arg in args {
            if let Value::String(s) = arg {
                path.push(s);
            } else {
                return Err(DscError::Parser("Arguments must all be strings".to_string()));
            }
        }

        Ok(Value::String(path.to_string_lossy().to_string()))
    }
}

#[cfg(test)]
mod tests {
    use crate::configure::context::Context;
    use crate::parser::Statement;

    #[test]
    fn no_arg() {
        let mut parser = Statement::new().unwrap();
        let separator = std::path::MAIN_SEPARATOR;
        let result = parser.parse_and_execute("[path('a','b'", &Context::new()).unwrap();
        assert_eq!(result, format!("a{separator}b"));
    }

    #[test]
    fn with_arg() {
        let mut parser = Statement::new().unwrap();
        let separator = std::path::MAIN_SEPARATOR;
        let result = parser.parse_and_execute("[path('a','b','c')]", &Context::new()).unwrap();
        assert_eq!(result, format!("{separator}a{separator}b{separator}c"));
    }
}
