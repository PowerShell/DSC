// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use base64::{Engine as _, engine::general_purpose};
use crate::DscError;
use crate::parser::functions::{FunctionArg, FunctionResult};
use super::{Function, AcceptedArgKind};

#[derive(Debug)]
pub struct Base64 {}

impl Function for Base64 {
    fn accepted_arg_types(&self) -> Vec<AcceptedArgKind> {
        vec![AcceptedArgKind::String]
    }

    fn min_args(&self) -> usize {
        1
    }

    fn max_args(&self) -> usize {
        1
    }

    fn invoke(&self, args: &[FunctionArg]) -> Result<FunctionResult, DscError> {
        let FunctionArg::String(arg) = args.get(0).unwrap() else {
            return Err(DscError::Parser("Invalid argument type".to_string()));
        };
        Ok(FunctionResult::String(general_purpose::STANDARD.encode(arg)))
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::Statement;

    #[test]
    fn strings() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[base64('hello world')]").unwrap();
        assert_eq!(result, "aGVsbG8gd29ybGQ=");
    }

    #[test]
    fn numbers() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[base64(123)]");
        assert!(result.is_err());
    }

    #[test]
    fn nested() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[base64(base64('hello world'))]").unwrap();
        assert_eq!(result, "YUdWc2JHOGdkMjl5YkdRPQ==");
    }
}
