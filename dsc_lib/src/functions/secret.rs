// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::DscError;
use crate::configure::context::Context;
use crate::functions::AcceptedArgKind;
use serde_json::Value;
use super::Function;

#[derive(Debug, Default)]
pub struct Secret {}

impl Function for Secret {
    fn accepted_arg_types(&self) -> Vec<AcceptedArgKind> {
        vec![AcceptedArgKind::String]
    }

    fn min_args(&self) -> usize {
        1
    }

    fn max_args(&self) -> usize {
        2
    }

    fn invoke(&self, args: &[Value], context: &Context) -> Result<Value, DscError> {
        let secret = args[0].as_str().ok_or_else(|| {
            DscError::InvalidArgumentType("Secret function requires a string argument".to_string())
        })?.to_string();
        let vault: Option<String> = if args.len() > 1 {
            args[1].as_str().map(|s| s.to_string())
        } else {
            None
        };

        // if no vault name is provided, we query all extensions supporting the secret method
        // to see if any of them can provide the secret.  If none can or if multiple can, we return an error.

    }
}

#[cfg(test)]
mod tests {
    use crate::configure::context::Context;
    use crate::parser::Statement;

    #[test]
    fn strings() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[base64('hello world')]", &Context::new()).unwrap();
        assert_eq!(result, "aGVsbG8gd29ybGQ=");
    }

    #[test]
    fn numbers() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[base64(123)]", &Context::new());
        assert!(result.is_err());
    }

    #[test]
    fn nested() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[base64(base64('hello world'))]", &Context::new()).unwrap();
        assert_eq!(result, "YUdWc2JHOGdkMjl5YkdRPQ==");
    }
}
