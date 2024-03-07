// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::DscError;
use crate::configure::context::Context;
use crate::functions::{AcceptedArgKind, Function};
use serde_json::Value;

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

    fn invoke(&self, args: &[Value], _context: &Context) -> Result<Value, DscError> {
        let mut result = String::new();
        // first argument is the type and must contain only 1 slash
        let resource_type = &args[0];
        if let Some(value) = resource_type.as_str() {
            let slash_count = value.chars().filter(|c| *c == '/').count();
            if slash_count != 1 {
                return Err(DscError::Function("resourceId".to_string(), "Type argument must contain exactly one slash".to_string()));
            }
            result.push_str(value);
        } else {
            return Err(DscError::Parser("Invalid argument type for first parameter".to_string()));
        }
        // ARM uses a slash separator, but here we use a colon which is not allowed for the type nor name
        result.push(':');
        // second argument is the name and must contain no slashes
        let resource_name = &args[1];
        if let Some(value) = resource_name.as_str() {
            if value.contains('/') {
                return Err(DscError::Function("resourceId".to_string(), "Name argument cannot contain a slash".to_string()));
            }

            result.push_str(value);
        } else {
            return Err(DscError::Parser("Invalid argument type for second parameter".to_string()));
        }

        Ok(Value::String(result))
    }
}

#[cfg(test)]
mod tests {
    use crate::configure::context::Context;
    use crate::parser::Statement;

    #[test]
    fn strings() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[resourceId('a/b', 'c')]", &Context::new()).unwrap();
        assert_eq!(result, "a/b:c");
    }

    #[test]
    fn strings_with_dots() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[resourceId('a.b/c', 'd')]", &Context::new()).unwrap();
        assert_eq!(result, "a.b/c:d");
    }

    #[test]
    fn invalid_type() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[resourceId('a/b/c','d')]", &Context::new());
        assert!(result.is_err());
    }

    #[test]
    fn invalid_name() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[resourceId('a','b/c')]", &Context::new());
        assert!(result.is_err());
    }

    #[test]
    fn invalid_one_parameter() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[resourceId('a')]", &Context::new());
        assert!(result.is_err());
    }
}
