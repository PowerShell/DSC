// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::DscError;
use crate::functions::{Function, FunctionArg, FunctionResult, AcceptedArgKind};

#[derive(Debug, Default)]
pub struct ResourceId {}

impl Function for ResourceId {
    fn min_args(&self) -> usize {
        2
    }

    fn max_args(&self) -> usize {
        2
    }

    fn accepted_arg_types(&self) -> Vec<AcceptedArgKind> {
        vec![AcceptedArgKind::String]
    }

    fn invoke(&self, args: &[FunctionArg]) -> Result<FunctionResult, DscError> {
        let mut result = String::new();
        // verify that the arguments do not contain a slash
        for arg in args {
            match arg {
                FunctionArg::String(value) => {
                    if value.contains('/') {
                        return Err(DscError::Function("resourceId".to_string(), "Argument cannot contain a slash".to_string()));
                    }

                    result.push_str(value);
                },
                _ => {
                    return Err(DscError::Parser("Invalid argument type".to_string()));
                }
            }
            result.push('/');
        }
        Ok(FunctionResult::String(result))
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::Statement;

    #[test]
    fn strings() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[resourceId('a', 'b')]").unwrap();
        assert_eq!(result, "a/b");
    }

    #[test]
    fn strings_with_dots() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[resourceId('a.b.c', 'd')]").unwrap();
        assert_eq!(result, "a.b.c/d");
    }

    #[test]
    fn invalid_type() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[resourceId('a/b','c')]");
        assert!(result.is_err());
    }

    #[test]
    fn invalid_name() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[resourceId('a','b/c')]");
        assert!(result.is_err());
    }

    #[test]
    fn invalid_one_parameter() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[resourceId('a')]");
        assert!(result.is_err());
    }
}
