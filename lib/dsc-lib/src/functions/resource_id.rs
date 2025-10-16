// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::DscError;
use crate::configure::context::Context;
use crate::functions::{FunctionArgKind, Function, FunctionCategory, FunctionMetadata};
use rust_i18n::t;
use serde_json::Value;

#[derive(Debug, Default)]
pub struct ResourceId {}

impl Function for ResourceId {
    fn get_metadata(&self) -> FunctionMetadata {
        FunctionMetadata {
            name: "resourceId".to_string(),
            description: t!("functions.resourceId.description").to_string(),
            category: vec![FunctionCategory::Resource],
            min_args: 2,
            max_args: 2,
            accepted_arg_ordered_types: vec![
                vec![FunctionArgKind::String],
                vec![FunctionArgKind::String],
            ],
            remaining_arg_accepted_types: None,
            return_types: vec![FunctionArgKind::String],
        }
    }

    fn invoke(&self, args: &[Value], _context: &Context) -> Result<Value, DscError> {
        let mut result = String::new();
        // first argument is the type and must contain only 1 slash
        let resource_type = &args[0];
        if let Some(value) = resource_type.as_str() {
            let slash_count = value.chars().filter(|c| *c == '/').count();
            if slash_count != 1 {
                return Err(DscError::Function("resourceId".to_string(), t!("functions.resourceId.incorrectTypeFormat").to_string()));
            }
            result.push_str(value);
        } else {
            return Err(DscError::Parser(t!("functions.resourceId.invalidFirstArgType").to_string()));
        }
        // ARM uses a slash separator, but here we use a colon which is not allowed for the type nor name
        result.push(':');
        // second argument is the name and we url encode it to ensure no unexpected characters are present
        let resource_name = &args[1];
        if let Some(value) = resource_name.as_str() {
            let encoded = urlencoding::encode(value);
            result.push_str(&encoded);
        } else {
            return Err(DscError::Parser(t!("functions.resourceId.invalidSecondArgType").to_string()));
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
    fn valid_name_with_slashes() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[resourceId('a/a','b/c/d')]", &Context::new()).unwrap();
        assert_eq!(result, "a/a:b%2Fc%2Fd");
    }

    #[test]
    fn invalid_one_parameter() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[resourceId('a')]", &Context::new());
        assert!(result.is_err());
    }
}
