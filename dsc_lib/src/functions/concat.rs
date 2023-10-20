// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::DscError;
use crate::functions::{Function, FunctionArg, FunctionResult, AcceptedArgKind};

#[derive(Debug)]
pub struct Concat {}

impl Function for Concat {
    fn min_args(&self) -> usize {
        2
    }

    fn max_args(&self) -> usize {
        usize::MAX
    }

    fn accepted_arg_types(&self) -> Vec<AcceptedArgKind> {
        vec![AcceptedArgKind::String, AcceptedArgKind::Integer]
    }

    fn invoke(&self, args: &Vec<FunctionArg>) -> Result<FunctionResult, DscError> {
        let mut result = String::new();
        for arg in args {
            match arg {
                FunctionArg::String(value) => {
                    result.push_str(value);
                },
                FunctionArg::Integer(value) => {
                    result.push_str(&value.to_string());
                },
                _ => {
                    return Err(DscError::Parser("Invalid argument type".to_string()));
                }
            }
        }
        Ok(FunctionResult::String(result))
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::StatementParser;

    #[test]
    fn strings() {
        let mut parser = StatementParser::new().unwrap();
        let result = parser.parse_and_execute("[concat('a', 'b')]").unwrap();
        assert_eq!(result, "ab");
    }

    #[test]
    fn strings_with_spaces() {
        let mut parser = StatementParser::new().unwrap();
        let result = parser.parse_and_execute("[concat('a ', ' ', ' b')]").unwrap();
        assert_eq!(result, "a   b");
    }

    #[test]
    fn numbers() {
        let mut parser = StatementParser::new().unwrap();
        let result = parser.parse_and_execute("[concat(1, 2)]").unwrap();
        assert_eq!(result, "12");
    }

    #[test]
    fn string_and_numbers() {
        let mut parser = StatementParser::new().unwrap();
        let result = parser.parse_and_execute("[concat('a', 1, 'b', 2)]").unwrap();
        assert_eq!(result, "a1b2");
    }

    #[test]
    fn nested() {
        let mut parser = StatementParser::new().unwrap();
        let result = parser.parse_and_execute("[concat('a', concat('b', 'c'), 'd')]").unwrap();
        assert_eq!(result, "abcd");
    }

    #[test]
    fn invalid_one_parameter() {
        let mut parser = StatementParser::new().unwrap();
        let result = parser.parse_and_execute("[concat('a')]");
        assert!(result.is_err());
    }
}
